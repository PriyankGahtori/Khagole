//! Sunrise and sunset (Skyfield `almanac.sunrise_sunset` + `find_discrete`, MIT, see
//! NOTICE), used for planetary hours.

use crate::constants::{AU_M, DAY_S, TAU};
use crate::ephemeris::observe::{apparent, observe_from, Observer, EARTH};
use crate::ephemeris::Kernel;
use crate::ephemeris::KernelSet;
use crate::error::EngineError;
use crate::frames::nutation::iau2000b_radians;
use crate::frames::{mxm, mxv, rot_z, Mat3, Orientation, Vec3};
use crate::planets::require_kernel;
use crate::time::Time;

const SUN: i32 = 10;
/// Earth's rotation rate, rad/s.
const ANGVEL: f64 = 7.2921150e-5;
/// The Sun is up when its center is higher than this (radius + mean refraction).
const SUN_UP_ALTITUDE_DEG: f64 = -0.8333;
/// Initial sampling step of the search: catches days at least an hour long.
const STEP_DAYS: f64 = 0.04;
/// Search precision: one millisecond.
const EPSILON_DAYS: f64 = 0.001 / DAY_S;

/// A point on the WGS84 ellipsoid (elevation 0).
struct Site {
    itrs_position: Vec3,
    itrs_velocity: Vec3,
    /// Rotation from the true equator of date, Earth-fixed, to the local horizon.
    horizon: Mat3,
}

fn degrees_to_radians(deg: f64) -> f64 {
    // Skyfield `Angle(degrees=…)`: degrees / 360 * tau.
    deg / 360.0 * TAU
}

impl Site {
    fn new(latitude_deg: f64, longitude_deg: f64) -> Self {
        const RADIUS_M: f64 = 6378137.0;
        const INVERSE_FLATTENING: f64 = 298.257223563;
        let omf = (INVERSE_FLATTENING - 1.0) / INVERSE_FLATTENING;
        let omf2 = omf * omf;
        let lat = degrees_to_radians(latitude_deg);
        let lon = degrees_to_radians(longitude_deg);
        let radius_au = RADIUS_M / AU_M;
        let (sinphi, cosphi) = lat.sin_cos();
        let c = 1.0 / (cosphi * cosphi + sinphi * sinphi * omf2).sqrt();
        let s = omf2 * c;
        let xy = (radius_au * c + 0.0) * cosphi;
        let (x, y) = (xy * lon.cos(), xy * lon.sin());
        let z = (radius_au * s + 0.0) * sinphi;
        let w = ANGVEL * DAY_S;

        // rot_y(lat) with its rows reversed, times rot_z(-lon).
        let (sl, cl) = lat.sin_cos();
        let r_lat = [[-sl, 0.0, cl], [0.0, 1.0, 0.0], [cl, 0.0, sl]];
        Site {
            itrs_position: [x, y, z],
            itrs_velocity: [w * -y, w * x, w * (0.0 * z)],
            horizon: mxm(&r_lat, &rot_z(-lon)),
        }
    }
}

fn transpose(m: &Mat3) -> Mat3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Whether the Sun's center is above -0.8333° at TT Julian date `jd_tt`.
fn sun_is_up(kernel: &Kernel, site: &Site, jd_tt: f64) -> Result<bool, EngineError> {
    let t = Time::from_tt_jd(jd_tt);
    // Skyfield's search swaps in the faster IAU 2000B nutation.
    let o = Orientation::with_nutation(&t, iau2000b_radians(t.tt()));
    let itrs = mxm(&rot_z(-o.gast_hours * TAU / 24.0), &o.m);
    let to_gcrs = transpose(&itrs);
    let site_position = mxv(&to_gcrs, &site.itrs_position);
    let site_velocity = mxv(&to_gcrs, &site.itrs_velocity);

    let (earth_p, earth_v) = kernel.barycentric(EARTH, &t)?;
    let observer = Observer {
        position: [0, 1, 2].map(|k| earth_p[k] + site_position[k]),
        velocity: [0, 1, 2].map(|k| earth_v[k] + site_velocity[k]),
        gcrs_position: Some(site_position),
    };
    let astrometric = observe_from(kernel, &observer, SUN, &t)?;
    let app = apparent(kernel, &astrometric, &t)?;
    let local = mxv(&mxm(&site.horizon, &itrs), &app);
    let altitude = local[2].atan2(local[0].hypot(local[1]));
    Ok(altitude * 360.0 / TAU >= SUN_UP_ALTITUDE_DEG)
}

/// NumPy `linspace(start, stop, num)`.
fn linspace(start: f64, stop: f64, num: usize) -> Vec<f64> {
    let step = (stop - start) / (num - 1) as f64;
    let mut out: Vec<f64> = (0..num).map(|i| i as f64 * step + start).collect();
    out[num - 1] = stop;
    out
}

/// Skyfield `find_discrete` for the sun-up function: the TT instants where it changes,
/// with the value it changes to (true = sunrise).
fn find_transitions(
    kernel: &Kernel,
    site: &Site,
    jd0: f64,
    jd1: f64,
) -> Result<Vec<(f64, bool)>, EngineError> {
    const NUM: usize = 12;
    let sample_count = ((jd1 - jd0) / STEP_DAYS) as usize + 2;
    let end_mask = linspace(0.0, 1.0, NUM);
    let start_mask: Vec<f64> = end_mask.iter().rev().copied().collect();
    let mut jd = linspace(jd0, jd1, sample_count);
    loop {
        let y = jd
            .iter()
            .map(|&x| sun_is_up(kernel, site, x))
            .collect::<Result<Vec<bool>, _>>()?;
        let changes: Vec<usize> = (0..y.len() - 1).filter(|&i| y[i] != y[i + 1]).collect();
        if changes.is_empty() {
            return Ok(Vec::new());
        }
        let widest = changes
            .iter()
            .map(|&i| jd[i + 1] - jd[i])
            .fold(f64::NEG_INFINITY, f64::max);
        if widest <= EPSILON_DAYS {
            return Ok(changes.iter().map(|&i| (jd[i + 1], y[i + 1])).collect());
        }
        jd = changes
            .iter()
            .flat_map(|&i| {
                let (start, end) = (jd[i], jd[i + 1]);
                (0..NUM).map(move |j| (start, end, j))
            })
            .map(|(start, end, j)| start * start_mask[j] + end * end_mask[j])
            .collect();
    }
}

/// `SkyfieldEngine.calculate_rise_set`: the sunrise at or before `jd_utc` (UT1 Julian
/// date, with 0.05 day of slack), the sunset after it and the next sunrise, as UT1
/// Julian dates. Where the Sun does not rise or set, half-day defaults stand in.
pub fn rise_set(
    kernels: &KernelSet,
    jd_utc: f64,
    latitude: f64,
    longitude: f64,
) -> Result<(f64, f64, f64), EngineError> {
    let kernel = require_kernel(kernels, jd_utc)?;
    let site = Site::new(latitude, longitude);
    let t_start = Time::from_ut1(jd_utc - 1.5);
    let t_end = Time::from_ut1(jd_utc + 1.5);
    let events = find_transitions(kernel, &site, t_start.tt(), t_end.tt())?;
    Ok(select_rise_set(&events, jd_utc))
}

/// Sunrises and sunsets over a span, found once for all the days in it.
pub(crate) struct SunEvents {
    /// (TT Julian date, whether it is a sunrise), in time order.
    events: Vec<(f64, bool)>,
    /// TT span searched.
    span: (f64, f64),
}

impl SunEvents {
    /// Every sunrise and sunset from UT1 Julian date `jd0` to `jd1`, when a single kernel
    /// covers the whole span (`None` otherwise).
    pub fn find(
        kernels: &KernelSet,
        jd0: f64,
        jd1: f64,
        latitude: f64,
        longitude: f64,
    ) -> Result<Option<Self>, EngineError> {
        let kernel = require_kernel(kernels, jd0)?;
        if !(kernel.start_jd..=kernel.end_jd).contains(&jd1) {
            return Ok(None);
        }
        let site = Site::new(latitude, longitude);
        let span = (Time::from_ut1(jd0).tt(), Time::from_ut1(jd1).tt());
        let events = find_transitions(kernel, &site, span.0, span.1)?;
        Ok(Some(SunEvents { events, span }))
    }

    /// [`rise_set`] for `jd_utc`, from the events of its ±1.5 day window; `None` when
    /// the window is not inside the span searched.
    pub fn rise_set(&self, jd_utc: f64) -> Option<(f64, f64, f64)> {
        let (t0, t1) = (
            Time::from_ut1(jd_utc - 1.5).tt(),
            Time::from_ut1(jd_utc + 1.5).tt(),
        );
        if t0 < self.span.0 || t1 > self.span.1 {
            return None;
        }
        let from = self.events.partition_point(|e| e.0 < t0);
        let to = self.events.partition_point(|e| e.0 <= t1);
        Some(select_rise_set(&self.events[from..to], jd_utc))
    }
}

/// The day's sunrise, sunset and next sunrise among the events around `jd_utc`.
fn select_rise_set(events: &[(f64, bool)], jd_utc: f64) -> (f64, f64, f64) {
    let ut1 = |jd_tt: f64| Time::from_tt_jd(jd_tt).ut1();
    let sunrises: Vec<f64> = events.iter().filter(|e| e.1).map(|e| ut1(e.0)).collect();
    let sunsets: Vec<f64> = events.iter().filter(|e| !e.1).map(|e| ut1(e.0)).collect();

    let rise = sunrises
        .iter()
        .rev()
        .copied()
        .find(|&s| s <= jd_utc + 0.05)
        .unwrap_or(jd_utc - 0.5);
    let set = sunsets
        .iter()
        .copied()
        .find(|&s| s >= rise)
        .unwrap_or(rise + 0.5);
    let next_rise = sunrises
        .iter()
        .copied()
        .find(|&s| s > rise + 0.1)
        .unwrap_or(rise + 1.0);
    (rise, set, next_rise)
}
