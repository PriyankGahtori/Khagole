//! A set of JPL kernels in preference order, and barycentric states of bodies.

use crate::constants::{AU_KM, DAY_S, T0};
use crate::frames::Vec3;
use crate::time::Time;

use super::spk::{Segment, Spk, SpkError};

/// NAIF code of the Solar System Barycenter.
pub const SSB: i32 = 0;

/// One loaded kernel and the Julian-date span over which it has every body (a chart needs
/// all of them, so the usable span is the intersection of the bodies' spans). A body may
/// be split into several segments covering consecutive time ranges.
#[derive(Debug)]
pub struct Kernel {
    /// Label used in error messages and diagnostics, e.g. "de440s.bsp".
    pub name: String,
    spk: Spk,
    /// First Julian date (TDB) at which every body is covered.
    pub start_jd: f64,
    /// Last Julian date (TDB) at which every body is covered.
    pub end_jd: f64,
}

impl Kernel {
    /// Wrap a loaded SPK file; fails if it has no segments or if its bodies share no
    /// common time span.
    pub fn new(name: impl Into<String>, spk: Spk) -> Result<Self, SpkError> {
        let segments = spk.segments();
        if segments.is_empty() {
            return Err(SpkError::Format("kernel has no segments".into()));
        }
        let (mut start_et, mut end_et) = (f64::NEG_INFINITY, f64::INFINITY);
        let mut targets: Vec<i32> = segments.iter().map(|s| s.target).collect();
        targets.sort_unstable();
        targets.dedup();
        for target in targets {
            let (start, end) = body_span(segments, target);
            start_et = start_et.max(start);
            end_et = end_et.min(end);
        }
        if start_et.is_nan() || end_et.is_nan() || start_et > end_et {
            return Err(SpkError::Format(
                "the kernel's bodies share no common time span".into(),
            ));
        }
        let (start_jd, end_jd) = (start_et / DAY_S + T0, end_et / DAY_S + T0);
        Ok(Kernel {
            name: name.into(),
            spk,
            start_jd,
            end_jd,
        })
    }

    /// Whether every segment covers Julian date `jd`.
    pub fn covers(&self, jd: f64) -> bool {
        self.start_jd <= jd && jd <= self.end_jd
    }

    /// Whether the kernel has a segment for this body (Skyfield `code in ephemeris`).
    pub fn contains(&self, code: i32) -> bool {
        self.spk
            .segments()
            .iter()
            .any(|s| s.target == code || s.center == code)
    }

    /// The segment for `target` covering `et` (TDB seconds past J2000): the first one in
    /// file order that covers it, else the body's first segment (which then reports the
    /// date out of range).
    fn segment_for(&self, target: i32, et: f64) -> Option<&Segment> {
        let mut segments = self.spk.segments().iter().filter(|s| s.target == target);
        let first = segments.next()?;
        if first.covers(et) {
            return Some(first);
        }
        segments.find(|s| s.covers(et)).or(Some(first))
    }

    /// Position (au) and velocity (au/day) of `code` relative to the Solar System
    /// Barycenter, summing the chain of segments outward from the SSB like Skyfield's
    /// `VectorSum`.
    #[doc(hidden)] // takes the internal `Time`
    pub fn barycentric(&self, code: i32, t: &Time) -> Result<(Vec3, Vec3), SpkError> {
        const MAX_CHAIN: usize = 8;
        let et = (t.whole - T0 + t.tdb_fraction) * DAY_S;
        let mut chain: [Option<&Segment>; MAX_CHAIN] = [None; MAX_CHAIN];
        let mut len = 0;
        let mut current = code;
        while current != SSB {
            if len == MAX_CHAIN {
                return Err(SpkError::Format(format!("segment chain for {code} loops")));
            }
            let segment = self.segment_for(current, et).ok_or(SpkError::NoSegment {
                target: current,
                center: SSB,
            })?;
            chain[len] = Some(segment);
            len += 1;
            current = segment.center;
        }
        let mut position = [0.0; 3];
        let mut velocity = [0.0; 3];
        for segment in chain[..len].iter().rev().flatten() {
            let (p, v) = self
                .spk
                .segment_state_split(segment, t.whole, t.tdb_fraction)?;
            for k in 0..3 {
                position[k] += p[k] / AU_KM;
                velocity[k] += v[k] * DAY_S / AU_KM;
            }
        }
        Ok((position, velocity))
    }
}

/// The longest contiguous span (TDB seconds past J2000) covered by `target`'s segments.
fn body_span(segments: &[Segment], target: i32) -> (f64, f64) {
    let mut spans: Vec<(f64, f64)> = segments
        .iter()
        .filter(|s| s.target == target)
        .map(|s| (s.start_et, s.end_et))
        .collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut best = (f64::NAN, f64::NAN);
    let mut run = spans[0];
    for &(start, end) in &spans[1..] {
        if start <= run.1 {
            run.1 = run.1.max(end);
        } else {
            best = longer(best, run);
            run = (start, end);
        }
    }
    longer(best, run)
}

fn longer(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    if a.1 - a.0 >= b.1 - b.0 {
        a
    } else {
        b
    }
}

/// Kernels in preference order: a date is computed with the first kernel covering it.
#[derive(Debug, Default)]
pub struct KernelSet {
    kernels: Vec<Kernel>,
}

impl KernelSet {
    /// An empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a kernel, after (so less preferred than) the ones already loaded.
    pub fn push(&mut self, kernel: Kernel) {
        self.kernels.push(kernel);
    }

    /// The kernels, in preference order.
    pub fn kernels(&self) -> &[Kernel] {
        &self.kernels
    }

    /// Whether no kernel is loaded.
    pub fn is_empty(&self) -> bool {
        self.kernels.is_empty()
    }

    /// The preferred kernel covering this Julian date.
    pub fn for_jd(&self, jd: f64) -> Option<&Kernel> {
        self.kernels.iter().find(|k| k.covers(jd))
    }

    /// Combined span of every loaded kernel, as Julian dates.
    pub fn coverage(&self) -> Option<(f64, f64)> {
        if self.kernels.is_empty() {
            return None;
        }
        let start = self
            .kernels
            .iter()
            .map(|k| k.start_jd)
            .fold(f64::INFINITY, f64::min);
        let end = self
            .kernels
            .iter()
            .map(|k| k.end_jd)
            .fold(f64::NEG_INFINITY, f64::max);
        Some((start, end))
    }
}
