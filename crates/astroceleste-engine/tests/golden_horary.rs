//! Horary charts (planetary hours from computed sunrise/sunset) against the reference.

mod common;

use astroceleste_engine::{calculate_horary_chart, ChartRequest};
use common::*;

#[test]
fn horary_charts_match_reference() {
    let Some(kernels) = kernels() else { return };
    let cases = fixture("horary.json");
    let mut diffs = Vec::new();
    for case in &cases {
        let input = &case["input"];
        let req = ChartRequest {
            instant: instant(&input["utc"]),
            latitude: input["lat"].as_f64().unwrap(),
            longitude: input["lon"].as_f64().unwrap(),
            house_system: input["house_system"].as_str().unwrap(),
            zodiac_type: input["zodiac_type"].as_str().unwrap(),
            ayanamsa: input["ayanamsa"].as_str().unwrap(),
            orb_settings: None,
            dignity_scheme: "lilly",
            position_mode: "apparent",
            chinese_calendar: false,
        };
        let chart = calculate_horary_chart(&kernels, &req, None).unwrap();
        let mut actual = serde_json::to_value(&chart).unwrap();
        strip_private(&mut actual);
        diff(
            &actual,
            &case["output"],
            input["id"].as_str().unwrap(),
            1e-9,
            &mut diffs,
        );
    }
    assert_no_diffs("horary", cases.len(), &diffs);
}
