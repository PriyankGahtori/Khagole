//! The traditional additions to a chart (planet condition, sect, receptions, antiscia,
//! planetary hours), which the reference fixtures predate.

mod common;

use khagole_engine::{
    calculate_chart, calculate_horary_chart, calculate_transit_chart, chart_dignities,
    ChartRequest, UtcInstant,
};
use common::kernels;
use serde_json::{json, Value};

const SEVEN: [&str; 7] = [
    "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn",
];

fn request(utc: &str) -> ChartRequest<'static> {
    ChartRequest::new(UtcInstant::parse(utc).unwrap(), 41.9028, 12.4964)
}

fn chart_json(req: &ChartRequest) -> Option<Value> {
    let kernels = kernels()?;
    Some(serde_json::to_value(calculate_chart(&kernels, req).unwrap()).unwrap())
}

fn planet<'a>(chart: &'a Value, name: &str) -> &'a Value {
    chart["planets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == name)
        .unwrap()
}

#[test]
fn only_the_seven_planets_have_a_condition() {
    let Some(chart) = chart_json(&request("1987-05-17T14:30:00Z")) else {
        return;
    };
    for p in chart["planets"].as_array().unwrap() {
        let name = p["name"].as_str().unwrap();
        assert_eq!(
            p.get("condition").is_some(),
            SEVEN.contains(&name),
            "{name}"
        );
    }
    assert_eq!(chart["dignity_scheme"], "lilly");
}

#[test]
fn sect_follows_the_sun_above_the_horizon() {
    for (utc, expected) in [
        ("1987-05-17T11:00:00Z", "diurnal"),
        ("1987-05-17T23:00:00Z", "nocturnal"),
    ] {
        let Some(chart) = chart_json(&request(utc)) else {
            return;
        };
        assert_eq!(chart["sect"], expected, "{utc}");
        let sun = planet(&chart, "Sun");
        let house = sun["house"].as_u64().unwrap();
        assert_eq!(house >= 7, expected == "diurnal", "Sun in house {house}");
        assert_eq!(sun["condition"]["above_horizon"], expected == "diurnal");
        assert_eq!(sun["condition"]["in_sect"], expected == "diurnal");
        assert_eq!(
            planet(&chart, "Moon")["condition"]["in_sect"],
            expected == "nocturnal"
        );
    }
}

#[test]
fn essential_dignities_agree_with_the_positions() {
    let Some(chart) = chart_json(&request("1987-05-17T14:30:00Z")) else {
        return;
    };
    for name in SEVEN {
        let p = planet(&chart, name);
        let e = &p["condition"]["essential"];
        let lords = &e["lords"];
        assert_eq!(e["domicile"], lords["domicile"] == name, "{name}");
        assert_eq!(e["bound"], lords["bound"] == name, "{name}");
        assert_eq!(e["face"], lords["face"] == name, "{name}");
        assert_eq!(
            lords["triplicity"].as_array().unwrap().len(),
            2,
            "Lilly's triplicity has a day and a night lord"
        );
        let dignified = ["domicile", "exaltation", "triplicity", "bound", "face"]
            .iter()
            .any(|k| e[*k] == true);
        assert_eq!(e["peregrine"], !dignified, "{name}");
    }
    // 17 May 1987, by day: the Sun at 26° Taurus is peregrine (Venus's sign, Mars's Lilly
    // term from 26°, Saturn's face); Venus at 0° Taurus has her sign, her day triplicity and
    // her term (5 + 3 + 2); the Moon at 22° Capricorn is in detriment and peregrine.
    let essential = |name| planet(&chart, name)["condition"]["essential"].clone();
    let sun = essential("Sun");
    assert_eq!(sun["lords"]["bound"], "Mars");
    assert_eq!(sun["lords"]["face"], "Saturn");
    assert_eq!(sun["score"], -5);
    assert_eq!(essential("Venus")["score"], 10);
    assert_eq!(essential("Moon")["score"], -10);
}

#[test]
fn the_dorothean_scheme_uses_three_triplicity_lords_and_the_egyptian_bounds() {
    let mut req = request("1987-05-17T14:30:00Z");
    req.dignity_scheme = "dorothean";
    let Some(chart) = chart_json(&req) else {
        return;
    };
    assert_eq!(chart["dignity_scheme"], "dorothean");
    let sun = &planet(&chart, "Sun")["condition"]["essential"]["lords"];
    assert_eq!(sun["triplicity"], json!(["Venus", "Moon", "Mars"]));
    // 26° Taurus: Saturn's Egyptian bound (22-27), Mars's Lilly term (26-30).
    assert_eq!(sun["bound"], "Saturn");
}

#[test]
fn receptions_and_antiscia_name_chart_points() {
    for utc in [
        "1987-05-17T14:30:00Z",
        "2001-09-11T12:46:00Z",
        "1969-07-20T20:17:00Z",
    ] {
        let Some(chart) = chart_json(&request(utc)) else {
            return;
        };
        for r in chart["receptions"].as_array().unwrap() {
            assert!(SEVEN.contains(&r["planet"].as_str().unwrap()));
            assert!(SEVEN.contains(&r["receiver"].as_str().unwrap()));
            assert_ne!(r["planet"], r["receiver"]);
            assert!(!r["dignities"].as_array().unwrap().is_empty());
            assert!(r["aspect"].is_string() || r["mutual"] == true);
        }
        for a in chart["antiscia"].as_array().unwrap() {
            let lon = |n: &Value| {
                planet(&chart, n.as_str().unwrap())["ecliptic_longitude"]
                    .as_f64()
                    .unwrap()
            };
            let (l1, l2) = (lon(&a["body1"]), lon(&a["body2"]));
            let mirror = if a["kind"] == "antiscion" {
                180.0 - l1
            } else {
                360.0 - l1
            };
            let d = (mirror - l2).rem_euclid(360.0);
            let d = d.min(360.0 - d);
            assert!(d <= 1.0 + 1e-9, "{a}");
            assert!((d - a["orb"].as_f64().unwrap()).abs() < 0.006, "{a}");
        }
    }
}

#[test]
fn every_chart_but_a_transit_sky_has_its_planetary_hour() {
    let Some(kernels) = kernels() else { return };
    let req = request("1987-05-17T14:30:00Z");
    let chart = serde_json::to_value(calculate_chart(&kernels, &req).unwrap()).unwrap();
    assert!(chart["planetary_hours"]["hour_ruler"].is_string());

    let horary =
        serde_json::to_value(calculate_horary_chart(&kernels, &req, None).unwrap()).unwrap();
    assert_eq!(
        horary["planetary_hours"],
        horary["horary_data"]["planetary_hours"]
    );

    let transit =
        serde_json::to_value(calculate_transit_chart(&kernels, &json!([]), &req).unwrap()).unwrap();
    assert!(transit.get("planetary_hours").is_none());
}

/// A chart's own condition and receptions, by planet name.
fn judged(chart: &Value) -> (Vec<(String, Value)>, Value, Value) {
    let conditions = chart["planets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p.get("condition").is_some())
        .map(|p| {
            (
                p["name"].as_str().unwrap().to_string(),
                p["condition"].clone(),
            )
        })
        .collect();
    (
        conditions,
        chart["receptions"].clone(),
        chart["sect"].clone(),
    )
}

fn rejudged(chart: &Value, scheme: &str) -> (Vec<(String, Value)>, Value, Value) {
    let out = serde_json::to_value(chart_dignities(chart, scheme).unwrap()).unwrap();
    let conditions = out["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["name"].as_str().unwrap().to_string(),
                c["condition"].clone(),
            )
        })
        .collect();
    (conditions, out["receptions"].clone(), out["sect"].clone())
}

#[test]
fn a_stored_chart_can_be_judged_again_under_either_scheme() {
    for utc in [
        "1987-05-17T14:30:00Z",
        "2001-09-11T12:46:00Z",
        "1969-07-20T20:17:00Z",
    ] {
        let Some(lilly) = chart_json(&request(utc)) else {
            return;
        };
        let mut req = request(utc);
        req.dignity_scheme = "dorothean";
        let dorothean = chart_json(&req).unwrap();
        // The same scheme reproduces the chart; the other one matches a chart computed with it.
        assert_eq!(rejudged(&lilly, "lilly"), judged(&lilly), "{utc}");
        assert_eq!(rejudged(&lilly, "dorothean"), judged(&dorothean), "{utc}");
        assert_eq!(rejudged(&dorothean, "lilly"), judged(&lilly), "{utc}");
    }
}

#[test]
fn judging_needs_the_sun_and_the_ascendant() {
    let chart =
        json!({ "planets": [{ "name": "Moon", "ecliptic_longitude": 10.0, "speed": 13.0 }] });
    assert!(chart_dignities(&chart, "lilly").is_err());
    assert!(chart_dignities(&json!({}), "lilly").is_err());
}
