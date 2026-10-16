//! Full natal charts against the reference implementation's golden output.

mod common;

use astroceleste_engine::{calculate_chart, ChartRequest, EngineError};
use common::*;

#[test]
fn natal_charts_match_reference() {
    let Some(kernels) = kernels() else { return };
    let cases = fixture("natal.json");
    let mut diffs = Vec::new();
    for case in &cases {
        let input = &case["input"];
        let id = input["id"].as_str().unwrap();
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
            node_type: "mean",
            chinese_calendar: false,
        };
        let chart = calculate_chart(&kernels, &req).unwrap();
        let mut actual = serde_json::to_value(&chart).unwrap();
        strip_private(&mut actual);
        diff(&actual, &case["output"], id, 1e-9, &mut diffs);
    }
    assert_no_diffs("natal", cases.len(), &diffs);
}

#[test]
fn out_of_range_dates_are_errors() {
    let Some(kernels) = kernels() else { return };
    for case in fixture("errors.json") {
        let input = &case["input"];
        let req = ChartRequest::new(
            instant(&input["utc"]),
            input["lat"].as_f64().unwrap(),
            input["lon"].as_f64().unwrap(),
        );
        let err = calculate_chart(&kernels, &req).unwrap_err();
        assert!(matches!(err, EngineError::OutOfRange { .. }), "{err}");
        assert_eq!(err.code(), case["error"].as_str().unwrap());
    }
}
