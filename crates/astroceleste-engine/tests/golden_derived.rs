//! Transits, synastry and derived charts against the reference.

mod common;

use std::collections::HashMap;

use astroceleste_engine::{
    calculate_derived_chart, calculate_synastry, calculate_transit_chart, ChartRequest,
};
use common::*;
use serde_json::Value;

fn natal_by_id() -> HashMap<String, Value> {
    fixture("natal.json")
        .into_iter()
        .map(|c| {
            (
                c["input"]["id"].as_str().unwrap().to_string(),
                c["output"].clone(),
            )
        })
        .collect()
}

#[test]
fn transits_match_reference() {
    let Some(kernels) = kernels() else { return };
    let natal = natal_by_id();
    let cases = fixture("transits.json");
    let mut diffs = Vec::new();
    for case in &cases {
        let t = &case["input"]["transit"];
        let base = &natal[case["input"]["natal"].as_str().unwrap()];
        let req = ChartRequest {
            instant: instant(&t["utc"]),
            latitude: t["lat"].as_f64().unwrap(),
            longitude: t["lon"].as_f64().unwrap(),
            house_system: t["house_system"].as_str().unwrap(),
            zodiac_type: t["zodiac_type"].as_str().unwrap(),
            ayanamsa: t["ayanamsa"].as_str().unwrap(),
            orb_settings: None,
            dignity_scheme: "lilly",
            position_mode: "apparent",
            chinese_calendar: false,
        };
        let chart = calculate_transit_chart(&kernels, &base["planets"], &req).unwrap();
        let mut actual = serde_json::to_value(&chart).unwrap();
        strip_private(&mut actual);
        diff(
            &actual,
            &case["output"],
            t["id"].as_str().unwrap(),
            1e-9,
            &mut diffs,
        );
    }
    assert_no_diffs("transits", cases.len(), &diffs);
}

#[test]
fn synastry_matches_reference() {
    let natal = natal_by_id();
    let cases = fixture("synastry.json");
    let mut diffs = Vec::new();
    for case in &cases {
        let a = &natal[case["input"]["a"].as_str().unwrap()];
        let b = &natal[case["input"]["b"].as_str().unwrap()];
        let result = calculate_synastry(&a["planets"], &b["planets"], None).unwrap();
        let actual = serde_json::to_value(&result).unwrap();
        diff(
            &actual,
            &case["output"],
            case["input"]["a"].as_str().unwrap(),
            1e-9,
            &mut diffs,
        );
    }
    assert_no_diffs("synastry", cases.len(), &diffs);
}

#[test]
fn derived_charts_match_reference() {
    let natal = natal_by_id();
    let cases = fixture("derived.json");
    let mut diffs = Vec::new();
    for case in &cases {
        let base = &natal[case["input"]["base"].as_str().unwrap()];
        let root = case["input"]["root_house"].as_i64().unwrap();
        let mut actual = calculate_derived_chart(base, root, None).unwrap();
        strip_private(&mut actual);
        diff(
            &actual,
            &case["output"],
            case["input"]["base"].as_str().unwrap(),
            0.0,
            &mut diffs,
        );
    }
    assert_no_diffs("derived", cases.len(), &diffs);
}
