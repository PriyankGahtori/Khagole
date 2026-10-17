//! Rough timings of the main entry points on the full kernel (ignored: run with
//! `cargo test --release --test timing -- --ignored --nocapture`). Not a benchmark
//! harness, just a way to compare before and after a change without extra dependencies.

mod common;

use std::time::{Duration, Instant};

use khagole_engine::ephemeris::{Kernel, KernelSet, Spk};
use khagole_engine::{
    calculate_chart, calculate_horary_chart, chinese_calendar, search_elections, ChartRequest,
    ElectionCriteria, UtcInstant,
};
use common::*;

fn report(label: &str, runs: u32, elapsed: Duration) {
    let each = elapsed / runs;
    println!("{label:<28} {runs:>5} runs  {each:>12.2?} each");
}

#[test]
#[ignore]
fn timings() {
    let Some(kernels) = kernels() else { return };
    let start = UtcInstant::parse("1987-05-17T14:30:00Z").unwrap();
    let hour = 3_600_000_000;

    let runs = 300;
    let begun = Instant::now();
    for i in 0..runs {
        let req = ChartRequest::new(start.add_micros(i64::from(i) * 7 * hour), 41.9, 12.5);
        calculate_chart(&kernels, &req).unwrap();
    }
    report("calculate_chart", runs, begun.elapsed());

    // The same kernel held in memory (as in WebAssembly and the mobile apps).
    let bytes = std::fs::read(root().join("kernels/de440s.bsp")).unwrap();
    let mut in_memory = KernelSet::new();
    in_memory.push(Kernel::new("de440s.bsp", Spk::from_bytes(bytes).unwrap()).unwrap());
    let begun = Instant::now();
    for i in 0..runs {
        let req = ChartRequest::new(start.add_micros(i64::from(i) * 7 * hour), 41.9, 12.5);
        calculate_chart(&in_memory, &req).unwrap();
    }
    report("calculate_chart (in memory)", runs, begun.elapsed());

    let runs = 100;
    let begun = Instant::now();
    for i in 0..runs {
        let req = ChartRequest::new(start.add_micros(i64::from(i) * 7 * hour), 41.9, 12.5);
        calculate_horary_chart(&kernels, &req, Some(7)).unwrap();
    }
    report("calculate_horary_chart", runs, begun.elapsed());

    let runs = 10;
    let begun = Instant::now();
    for i in 0..runs {
        chinese_calendar(&kernels, start.add_micros(i64::from(i) * 300 * hour)).unwrap();
    }
    report("chinese_calendar", runs, begun.elapsed());

    let runs = 3;
    let begun = Instant::now();
    for i in 0..runs {
        let from = start.add_micros(i64::from(i) * 24 * 40 * hour);
        let req = ChartRequest::new(from, 41.9, 12.5);
        let end = from.add_micros(30 * 24 * hour);
        search_elections(&kernels, &req, end, &ElectionCriteria::default()).unwrap();
    }
    report("search_elections (30 days)", runs, begun.elapsed());
}
