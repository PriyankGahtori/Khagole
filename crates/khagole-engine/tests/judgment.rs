//! Horary judgment: significators and perfection, which the reference fixtures predate.

mod common;

use khagole_engine::{calculate_horary_chart, ChartRequest, UtcInstant};
use common::kernels;
use serde_json::Value;

const RULERS: [(&str, &str); 12] = [
    ("Aries", "Mars"),
    ("Taurus", "Venus"),
    ("Gemini", "Mercury"),
    ("Cancer", "Moon"),
    ("Leo", "Sun"),
    ("Virgo", "Mercury"),
    ("Libra", "Venus"),
    ("Scorpio", "Mars"),
    ("Sagittarius", "Jupiter"),
    ("Capricorn", "Saturn"),
    ("Aquarius", "Saturn"),
    ("Pisces", "Jupiter"),
];

fn ruler(sign: &str) -> &'static str {
    RULERS.iter().find(|(s, _)| *s == sign).unwrap().1
}

fn wrap180(d: f64) -> f64 {
    let d = d.rem_euclid(360.0);
    if d > 180.0 {
        d - 360.0
    } else {
        d
    }
}

const ANGLES: [(&str, f64); 5] = [
    ("Conjunction", 0.0),
    ("Sextile", 60.0),
    ("Square", 90.0),
    ("Trine", 120.0),
    ("Opposition", 180.0),
];

#[test]
fn significators_follow_the_cusps_and_perfections_are_exact_aspects() {
    let Some(kernels) = kernels() else { return };
    for utc in [
        "1987-05-17T14:30:00Z",
        "2001-09-11T12:46:00Z",
        "2024-03-25T08:00:00Z",
    ] {
        for house in 1..=12u8 {
            let req = ChartRequest::new(UtcInstant::parse(utc).unwrap(), 41.9028, 12.4964);
            let chart = calculate_horary_chart(&kernels, &req, Some(house)).unwrap();
            let json = serde_json::to_value(&chart).unwrap();
            let j = &json["horary_data"]["judgment"];
            assert_eq!(j["quesited_house"], house as u64);
            assert_eq!(
                j["querent"],
                ruler(json["houses"][0]["sign"].as_str().unwrap())
            );
            let quesited = ruler(json["houses"][house as usize - 1]["sign"].as_str().unwrap());
            assert_eq!(j["quesited"], quesited);
            assert_eq!(j["same_significator"], j["querent"] == j["quesited"]);

            let place = |name: &Value| {
                let p = json["planets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| &p["name"] == name)
                    .unwrap();
                (
                    p["ecliptic_longitude"].as_f64().unwrap(),
                    p["speed"].as_f64().unwrap(),
                )
            };
            for key in ["perfection", "moon_perfection"] {
                let p = &j[key];
                if p.is_null() {
                    continue;
                }
                let days = p["days"].as_f64().unwrap();
                let (la, va) = place(&p["applying"]);
                let (lb, vb) = place(&p["to"]);
                let angle = ANGLES.iter().find(|(n, _)| *n == p["aspect"]).unwrap().1;
                let sep = wrap180((la + va * days) - (lb + vb * days)).abs();
                // Days are rounded to 0.01: the Moon moves up to ~0.15° in that time.
                assert!((sep - angle).abs() < 0.2, "{utc} {house} {key}: {p}");
                assert!(days > 0.0);
            }
            if !j["prohibition"].is_null() {
                assert!(
                    j["prohibition"]["days"].as_f64().unwrap()
                        <= j["perfection"]["days"].as_f64().unwrap()
                );
            }
        }
    }
}

#[test]
fn without_a_quesited_house_only_the_querent_is_named() {
    let Some(kernels) = kernels() else { return };
    let req = ChartRequest::new(
        UtcInstant::parse("1987-05-17T14:30:00Z").unwrap(),
        41.9028,
        12.4964,
    );
    let chart = calculate_horary_chart(&kernels, &req, None).unwrap();
    let j = serde_json::to_value(&chart.horary_data.judgment).unwrap();
    assert!(j["quesited"].is_null());
    assert!(j["perfection"].is_null());
    assert_eq!(j["co_significator"], "Moon");
}
