//! Nonsensical input is an `invalid_input` error, and extreme but valid input (polar
//! latitudes, every house system) gives finite results. Uses the committed 2000 excerpt.

use std::path::PathBuf;

use khagole_engine::ephemeris::{Kernel, KernelSet, Spk};
use khagole_engine::{
    bazi, calculate_chart, calculate_election_chart, calculate_horary_chart, calculate_synastry,
    calculate_transit_chart, search_elections, time_lords, zi_wei, BaziOptions, ChartRequest,
    ChineseCalendar, ElectionCriteria, EngineError, UtcInstant, ZiWeiOptions,
};
use serde_json::json;

fn kernels() -> KernelSet {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/data/de440s_2000.bsp");
    let mut set = KernelSet::new();
    set.push(Kernel::new("de440s_2000.bsp", Spk::open(path).unwrap()).unwrap());
    set
}

fn utc(text: &str) -> UtcInstant {
    UtcInstant::parse(text).unwrap()
}

fn request(lat: f64, lon: f64) -> ChartRequest<'static> {
    ChartRequest::new(utc("2000-06-15T10:00:00Z"), lat, lon)
}

// Messages name the result type rather than print it: CodeQL's cleartext-logging query
// takes the chart's longitudes for geolocation.
fn assert_invalid<T>(result: Result<T, EngineError>, what: &str) {
    match result {
        Err(err) => assert_eq!(err.code(), "invalid_input", "{what}: {err}"),
        Ok(_) => panic!(
            "{what}: expected invalid_input, got Ok({})",
            std::any::type_name::<T>()
        ),
    }
}

#[test]
fn places_off_the_earth_are_rejected() {
    let kernels = kernels();
    let natal = json!([{"name": "Sun", "ecliptic_longitude": 84.0}]);
    let bad = [
        (f64::NAN, 12.5),
        (91.0, 12.5),
        (-90.5, 12.5),
        (f64::INFINITY, 12.5),
        (41.9, f64::NAN),
        (41.9, 360.5),
        (41.9, f64::NEG_INFINITY),
    ];
    for (lat, lon) in bad {
        let what = format!("lat {lat}, lon {lon}");
        let req = request(lat, lon);
        assert_invalid(calculate_chart(&kernels, &req), &what);
        assert_invalid(calculate_horary_chart(&kernels, &req, None), &what);
        assert_invalid(calculate_transit_chart(&kernels, &natal, &req), &what);
        let criteria = ElectionCriteria::default();
        assert_invalid(calculate_election_chart(&kernels, &req, &criteria), &what);
        let end = utc("2000-06-16T10:00:00Z");
        assert_invalid(search_elections(&kernels, &req, end, &criteria), &what);
    }
    // The edges themselves are places.
    for (lat, lon) in [
        (90.0, 0.0),
        (-90.0, 0.0),
        (0.0, 360.0),
        (0.0, -360.0),
        (0.0, -180.0),
    ] {
        calculate_chart(&kernels, &request(lat, lon)).unwrap();
    }
}

#[test]
fn every_house_system_gives_finite_cusps_at_any_latitude() {
    let kernels = kernels();
    let systems = ["P", "K", "R", "C", "T", "B", "M", "O", "E", "V", "W"];
    for lat in [
        -90.0, -89.99, -80.0, -66.6, -23.4, 0.0, 23.4, 66.6, 80.0, 89.99, 90.0,
    ] {
        for hs in systems {
            for hour in ["00", "06", "12", "18"] {
                let mut req = request(lat, -122.4);
                req.instant = utc(&format!("2000-03-20T{hour}:00:00Z"));
                req.house_system = hs;
                let chart = calculate_chart(&kernels, &req).unwrap();
                for cusp in &chart.houses {
                    let lon = cusp.ecliptic_longitude;
                    assert!(
                        lon.is_finite() && (0.0..360.0).contains(&lon),
                        "{hs} at {lat}° {hour}h: cusp {} outside [0, 360)",
                        cusp.house_number
                    );
                }
                for p in &chart.planets {
                    assert!((1..=12).contains(&p.house), "{hs} at {lat}°: {}", p.name);
                }
            }
        }
    }
}

#[test]
fn infinite_or_nan_orbs_are_rejected() {
    let kernels = kernels();
    for orbs in [
        json!({"fixed_star_orb": "inf"}),
        json!({"fixed_star_orb": "nan"}),
        json!({"aspect_orbs": {"Trine": "-inf"}}),
        json!({"planet_orbs": {"Moon": "NaN"}}),
    ] {
        let mut req = request(41.9, 12.5);
        req.orb_settings = Some(&orbs);
        assert_invalid(calculate_chart(&kernels, &req), &orbs.to_string());
    }
    let a = json!([{"name": "Sun", "ecliptic_longitude": "nan"}]);
    let b = json!([{"name": "Moon", "ecliptic_longitude": 10.0}]);
    assert_invalid(calculate_synastry(&a, &b, None), "NaN longitude");
    // Finite numeric strings stay valid, as in the reference (Python `float()`).
    let orbs = json!({"aspect_orbs": {"Trine": "7.5"}, "fixed_star_orb": "1"});
    let mut req = request(41.9, 12.5);
    req.orb_settings = Some(&orbs);
    calculate_chart(&kernels, &req).unwrap();
}

#[test]
fn quesited_house_must_be_a_house() {
    let kernels = kernels();
    let req = request(41.9, 12.5);
    for house in [0, 13, 255] {
        assert_invalid(
            calculate_horary_chart(&kernels, &req, Some(house)),
            &format!("house {house}"),
        );
    }
    let chart = calculate_horary_chart(&kernels, &req, Some(7)).unwrap();
    assert_eq!(chart.horary_data.judgment.quesited_house, Some(7));
}

#[test]
fn impossible_dates_do_not_parse() {
    for text in [
        "2000-02-30T00:00:00Z",
        "2001-02-29T00:00:00Z",
        "2000-04-31T00:00:00Z",
        "2000-00-10T00:00:00Z",
        "2000-01-00T00:00:00Z",
        "300000-01-01T00:00:00Z",
        "9999999999-01-01T00:00:00Z",
    ] {
        assert!(UtcInstant::parse(text).is_err(), "{text}");
    }
    // Far-future instants built by arithmetic saturate instead of overflowing.
    let far = UtcInstant::from_micros(i64::MAX - 5).add_micros(1_000);
    assert_eq!(far.micros(), i64::MAX);
}

#[test]
fn time_lords_span_is_bounded() {
    let birth = utc("1990-01-01T12:00:00Z");
    assert_invalid(
        time_lords(birth, 280.0, 130.0, birth, utc("3500-01-01T00:00:00Z")),
        "1500 years",
    );
    assert_invalid(
        time_lords(birth, f64::NAN, 130.0, birth, utc("2000-01-01T00:00:00Z")),
        "NaN Sun",
    );
    let lords = time_lords(birth, 280.0, 130.0, birth, utc("2100-01-01T00:00:00Z")).unwrap();
    assert_eq!(lords.profections.len(), 110); // ages 0-109: tropical years from birth
}

#[test]
fn bazi_offsets_must_be_finite_and_bounded() {
    let calendar: ChineseCalendar = serde_json::from_value(json!({
        "equation_of_time": 3.5,
        "solar_terms": [],
        "lunar_months": [],
    }))
    .unwrap();
    let birth = utc("1990-05-01T08:00:00Z");
    let options = BaziOptions::default();
    for (lon, offset) in [
        (f64::NAN, 120.0),
        (f64::INFINITY, 120.0),
        (500.0, 120.0),
        (12.5, f64::NAN),
        (12.5, f64::INFINITY),
        (12.5, 1e12),
    ] {
        let what = format!("lon {lon}, offset {offset}");
        assert_invalid(bazi(&calendar, birth, lon, offset, &options), &what);
        assert_invalid(
            zi_wei(&calendar, birth, lon, offset, &ZiWeiOptions::default()),
            &what,
        );
    }
    let mut hostile = calendar.clone();
    hostile.equation_of_time = f64::INFINITY;
    assert_invalid(
        bazi(&hostile, birth, 12.5, 120.0, &options),
        "infinite equation of time",
    );
    // A term with an impossible longitude is not a crash.
    let hostile: ChineseCalendar = serde_json::from_value(json!({
        "equation_of_time": 3.5,
        "solar_terms": [
            {"name": "x", "longitude": 315, "jie": true, "utc": "1990-02-04T00:00:00Z"},
            {"name": "x", "longitude": 65535, "jie": true, "utc": "1990-03-06T00:00:00Z"},
            {"name": "x", "longitude": 345, "jie": true, "utc": "1990-06-06T00:00:00Z"},
        ],
        "lunar_months": [],
    }))
    .unwrap();
    let _ = bazi(&hostile, birth, 12.5, 120.0, &options);
}
