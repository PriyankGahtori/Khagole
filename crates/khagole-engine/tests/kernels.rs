//! Kernel sets with several kernels, kernels whose bodies are split into time ranges, and
//! corrupt kernel bytes (which must be errors, never panics or huge allocations).

use std::path::PathBuf;

use khagole_engine::ephemeris::{Kernel, KernelSet, Spk};
use khagole_engine::time::Time;
use khagole_engine::{calculate_chart, ChartRequest, EngineError, UtcInstant};

const RECORD: usize = 1024;

fn excerpt_bytes() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/data/de440s_2000.bsp");
    std::fs::read(path).unwrap()
}

fn kernel(name: &str, bytes: Vec<u8>) -> Kernel {
    Kernel::new(name, Spk::from_bytes(bytes).unwrap()).unwrap()
}

fn set(kernels: Vec<Kernel>) -> KernelSet {
    let mut set = KernelSet::new();
    for k in kernels {
        set.push(k);
    }
    set
}

fn chart_at(kernels: &KernelSet, utc: &str) -> Result<serde_json::Value, EngineError> {
    let req = ChartRequest::new(UtcInstant::parse(utc).unwrap(), 41.9, 12.5);
    calculate_chart(kernels, &req).map(|c| serde_json::to_value(c).unwrap())
}

/// The committed excerpt (2000) cut in two at 2000-07-01 (JD 2451726.5).
fn halves() -> (Vec<u8>, Vec<u8>) {
    let spk = Spk::from_bytes(excerpt_bytes()).unwrap();
    (
        spk.excerpt(2_451_544.5, 2_451_726.5).unwrap(),
        spk.excerpt(2_451_726.5, 2_451_910.5).unwrap(),
    )
}

/// A segment summary: start and end (TDB seconds past J2000) and the six integers
/// (target, center, frame, type, first word, last word).
type Summary = (f64, f64, [i32; 6]);

/// Segment summaries of a little-endian DAF/SPK file.
fn summaries(bytes: &[u8]) -> Vec<Summary> {
    let f64_at = |at: usize| f64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
    let i32_at = |at: usize| i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
    let mut out = Vec::new();
    let mut record = i32_at(76) as usize;
    while record != 0 {
        let base = (record - 1) * RECORD;
        for i in 0..f64_at(base + 16) as usize {
            let at = base + 24 + i * 40;
            let ints: [i32; 6] = std::array::from_fn(|j| i32_at(at + 16 + 4 * j));
            out.push((f64_at(at), f64_at(at + 8), ints));
        }
        record = f64_at(base) as usize;
    }
    out
}

/// One DAF/SPK file holding every segment of `files`, in order (little-endian, the
/// layout of `Spk::excerpt`).
fn merge(files: &[&[u8]]) -> Vec<u8> {
    let segments: Vec<(&[u8], Summary)> = files
        .iter()
        .flat_map(|f| summaries(f).into_iter().map(move |s| (*f, s)))
        .collect();
    let per_record = 25;
    let summary_records = segments.len().div_ceil(per_record);
    let mut out = files[0][..RECORD].to_vec();
    out.resize((1 + 2 * summary_records) * RECORD, b' ');
    let mut word = out.len() / 8 + 1;
    for (k, chunk) in segments.chunks(per_record).enumerate() {
        let base = (1 + 2 * k) * RECORD;
        let next = if k + 1 < summary_records {
            2 * k + 4
        } else {
            0
        };
        out[base..base + 8].copy_from_slice(&(next as f64).to_le_bytes());
        out[base + 8..base + 16]
            .copy_from_slice(&(if k > 0 { 2 * k } else { 0 } as f64).to_le_bytes());
        out[base + 16..base + 24].copy_from_slice(&(chunk.len() as f64).to_le_bytes());
        out[base + 24..base + RECORD].fill(0);
        for (i, (file, (start, end, ints))) in chunk.iter().enumerate() {
            let data = &file[(ints[4] as usize - 1) * 8..ints[5] as usize * 8];
            let at = base + 24 + i * 40;
            out[at..at + 8].copy_from_slice(&start.to_le_bytes());
            out[at + 8..at + 16].copy_from_slice(&end.to_le_bytes());
            let first = word;
            let last = word + data.len() / 8 - 1;
            let fields = [
                ints[0],
                ints[1],
                ints[2],
                ints[3],
                first as i32,
                last as i32,
            ];
            for (j, v) in fields.into_iter().enumerate() {
                out[at + 16 + 4 * j..at + 20 + 4 * j].copy_from_slice(&v.to_le_bytes());
            }
            out.extend_from_slice(data);
            word = last + 1;
        }
    }
    out.resize(out.len().div_ceil(RECORD) * RECORD, 0);
    out[76..80].copy_from_slice(&2i32.to_le_bytes());
    out[80..84].copy_from_slice(&(2 * summary_records as i32).to_le_bytes());
    out[84..88].copy_from_slice(&(word as i32).to_le_bytes());
    out
}

#[test]
fn first_covering_kernel_wins_and_gaps_are_out_of_range() {
    let whole = set(vec![kernel("2000", excerpt_bytes())]);
    let (first, second) = halves();
    let both = set(vec![kernel("H1", first.clone()), kernel("H2", second)]);
    let (start, end) = both.coverage().unwrap();
    assert_eq!((start, end), whole.coverage().unwrap());
    for utc in [
        "2000-03-10T12:00:00Z",
        "2000-07-01T00:00:00Z",
        "2000-10-20T06:00:00Z",
    ] {
        assert_eq!(
            chart_at(&both, utc).unwrap(),
            chart_at(&whole, utc).unwrap(),
            "{utc}"
        );
    }
    assert_eq!(both.for_jd(2_451_600.0).unwrap().name, "H1");
    assert_eq!(both.for_jd(2_451_800.0).unwrap().name, "H2");

    // Only the first half loaded: the second half is out of range.
    let only_first = set(vec![kernel("H1", first)]);
    let err = chart_at(&only_first, "2000-10-20T06:00:00Z").unwrap_err();
    assert!(matches!(err, EngineError::OutOfRange { .. }), "{err}");
}

#[test]
fn bodies_split_into_time_ranges_cover_their_union() {
    let whole = set(vec![kernel("2000", excerpt_bytes())]);
    let (first, second) = halves();
    let split = merge(&[&first, &second]);
    let spk = Spk::from_bytes(split).unwrap();
    assert_eq!(spk.segments().len(), 28);
    let split = set(vec![Kernel::new("split", spk).unwrap()]);
    assert_eq!(split.coverage(), whole.coverage());
    for utc in [
        "2000-01-15T00:00:00Z",
        "2000-06-30T23:00:00Z",
        "2000-12-10T18:00:00Z",
    ] {
        assert_eq!(
            chart_at(&split, utc).unwrap(),
            chart_at(&whole, utc).unwrap(),
            "{utc}"
        );
    }
}

#[test]
fn bodies_without_a_common_span_are_rejected() {
    // Mercury's segment relabelled as covering 2010: no date has every body.
    let mut bytes = excerpt_bytes();
    let base = (i32::from_le_bytes(bytes[76..80].try_into().unwrap()) as usize - 1) * RECORD;
    let count = f64::from_le_bytes(bytes[base + 16..base + 24].try_into().unwrap()) as usize;
    let decade = 3650.0 * 86_400.0;
    for i in 0..count {
        let at = base + 24 + i * 40;
        if i32::from_le_bytes(bytes[at + 16..at + 20].try_into().unwrap()) == 1 {
            for off in [at, at + 8] {
                let v = f64::from_le_bytes(bytes[off..off + 8].try_into().unwrap()) + decade;
                bytes[off..off + 8].copy_from_slice(&v.to_le_bytes());
            }
        }
    }
    assert!(Kernel::new("disjoint", Spk::from_bytes(bytes).unwrap()).is_err());
}

/// Every 8-byte word in the file record, the summary records and each segment's
/// directory replaced by hostile values: loading and evaluating must fail cleanly.
#[test]
fn corrupt_kernels_are_errors_not_panics() {
    let good = excerpt_bytes();
    let mut offsets: Vec<usize> = (0..RECORD).step_by(4).collect();
    let summary_base = (i32::from_le_bytes(good[76..80].try_into().unwrap()) as usize - 1) * RECORD;
    offsets.extend((summary_base..summary_base + RECORD).step_by(4));
    for (_, _, ints) in summaries(&good) {
        let end = ints[5] as usize;
        offsets.extend(((end - 4) * 8..end * 8).step_by(8));
    }
    let hostile_f64 = [f64::NAN, f64::INFINITY, -1.0, 1e18, 0.0, 3.0, 2.5e9];
    let hostile_i32 = [i32::MIN, -1, 0, 1, i32::MAX, 30_000];
    let et = 15_000_000.0;
    let mut cases = 0;
    for &at in &offsets {
        let mut variants: Vec<Vec<u8>> = Vec::new();
        for v in hostile_i32 {
            let mut b = good.clone();
            b[at..at + 4].copy_from_slice(&v.to_le_bytes());
            variants.push(b);
        }
        if at % 8 == 0 && at + 8 <= good.len() {
            for v in hostile_f64 {
                let mut b = good.clone();
                b[at..at + 8].copy_from_slice(&v.to_le_bytes());
                variants.push(b);
            }
        }
        for bytes in variants {
            cases += 1;
            let Ok(spk) = Spk::from_bytes(bytes) else {
                continue;
            };
            for s in spk.segments() {
                let _ = spk.state(s.target, s.center, et);
                let _ = spk.segment_state(s, s.start_et);
                let _ = spk.segment_state(s, s.end_et);
            }
            let _ = spk.excerpt(2_451_600.0, 2_451_700.0);
            if let Ok(kernel) = Kernel::new("corrupt", spk) {
                for code in [10, 199, 299, 301, 399, 4, 5, 6, 7, 8, 9] {
                    let _ = kernel.barycentric(code, &Time::from_tdb(2_451_600.0, 0.25));
                }
            }
        }
    }
    assert!(cases > 1000);

    // Truncated files.
    for len in [
        0,
        7,
        100,
        RECORD,
        RECORD + 1,
        good.len() / 2,
        good.len() - 1,
    ] {
        if let Ok(spk) = Spk::from_bytes(good[..len].to_vec()) {
            for s in spk.segments() {
                let _ = spk.state(s.target, s.center, et);
            }
        }
    }
}
