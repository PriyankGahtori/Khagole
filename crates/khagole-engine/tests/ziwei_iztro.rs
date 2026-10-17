//! Zi Wei Dou Shu against 120 charts from iztro (MIT), the reference implementation of the
//! common school: tests/fixtures/ziwei_iztro.json, from scripts/make_ziwei_fixtures.cjs.

mod common;

use khagole_engine::{chinese_calendar, zi_wei, UtcInstant, ZiWeiOptions};
use common::{fixture, kernels};
use serde_json::Value;

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap_or("").to_string())
        .collect()
}

#[test]
fn zi_wei_matches_iztro() {
    let Some(kernels) = kernels() else { return };
    let mut mismatches = Vec::new();
    let mut palaces = 0;
    for case in fixture("ziwei_iztro.json") {
        let utc = case["utc"].as_str().unwrap();
        let birth = UtcInstant::parse(utc).unwrap();
        let calendar = chinese_calendar(&kernels, birth).unwrap().unwrap();
        let options = ZiWeiOptions {
            solar_time: false,
            sex: case["sex"].as_str(),
            ..ZiWeiOptions::default()
        };
        // Beijing civil time.
        let chart = zi_wei(&calendar, birth, 116.4, 480.0, &options).unwrap();
        let mut check = |what: String, ours: String, theirs: String| {
            if ours != theirs {
                mismatches.push(format!("{utc} {what}: ours {ours}, iztro {theirs}"));
            }
        };
        check(
            "life".into(),
            chart.life_palace.into(),
            case["life_palace"].as_str().unwrap().into(),
        );
        check(
            "body".into(),
            chart.body_palace.into(),
            case["body_palace"].as_str().unwrap().into(),
        );
        check(
            "bureau".into(),
            chart.bureau.number.to_string(),
            case["bureau"].to_string(),
        );
        for expected in case["palaces"].as_array().unwrap() {
            let branch = expected["branch"].as_str().unwrap();
            let palace = chart.palaces.iter().find(|p| p.branch == branch).unwrap();
            palaces += 1;
            let at = |field: &str| format!("{branch} {field}");
            check(
                at("stem"),
                palace.stem.into(),
                expected["stem"].as_str().unwrap().into(),
            );
            let mut stars: Vec<String> = palace
                .stars
                .iter()
                .map(|s| {
                    format!(
                        "{}:{}:{}",
                        s.star,
                        s.brightness.unwrap_or(""),
                        s.transformation.unwrap_or("")
                    )
                })
                .collect();
            stars.sort();
            check(
                at("stars"),
                stars.join(" "),
                strings(&expected["stars"]).join(" "),
            );
            let mut minor: Vec<&str> = palace.minor_stars.clone();
            minor.sort_unstable();
            check(
                at("minor"),
                minor.join(" "),
                strings(&expected["minor_stars"]).join(" "),
            );
            let gods = [
                palace.gods.life_stage.unwrap_or(""),
                palace.gods.boshi.unwrap_or(""),
                palace.gods.suiqian,
                palace.gods.jiangqian,
            ];
            check(
                at("gods"),
                gods.join(" "),
                strings(&expected["gods"]).join(" "),
            );
            let decade = palace.decade.as_ref().unwrap();
            check(
                at("decade"),
                format!("[{},{}]", decade.start, decade.end),
                expected["decade"].to_string(),
            );
            check(
                at("ages"),
                format!("{:?}", palace.small_limit_ages).replace(' ', ""),
                expected["ages"].to_string(),
            );
        }
    }
    assert_eq!(palaces, 120 * 12);
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches[..mismatches.len().min(40)].join("\n")
    );
}

#[test]
fn zi_wei_horoscopes_match_iztro() {
    let Some(kernels) = kernels() else { return };
    let mut mismatches = Vec::new();
    let mut count = 0;
    for case in fixture("ziwei_iztro_horoscope.json") {
        let utc = case["utc"].as_str().unwrap();
        let birth = UtcInstant::parse(utc).unwrap();
        let calendar = chinese_calendar(&kernels, birth).unwrap().unwrap();
        let options = ZiWeiOptions {
            solar_time: false,
            sex: case["sex"].as_str(),
            date: case["date"].as_str(),
            ..ZiWeiOptions::default()
        };
        let chart = zi_wei(&calendar, birth, 116.4, 480.0, &options).unwrap();
        let h = chart.horoscope.unwrap();
        count += 1;
        let mut check = |what: &str, ours: String, theirs: String| {
            if ours != theirs {
                mismatches.push(format!(
                    "{utc} on {}: {what}: ours {ours}, iztro {theirs}",
                    case["date"]
                ));
            }
        };
        check(
            "age",
            h.nominal_age.to_string(),
            case["nominal_age"].to_string(),
        );
        check(
            "childhood",
            h.childhood.to_string(),
            case["childhood"].to_string(),
        );
        check(
            "small limit",
            h.small_limit.unwrap_or("").into(),
            case["small_limit"].as_str().unwrap().into(),
        );
        for (name, period) in [
            ("decade", h.decade.as_ref()),
            ("yearly", Some(&h.yearly)),
            ("monthly", h.monthly.as_ref()),
            ("daily", h.daily.as_ref()),
        ] {
            let expected = &case[name];
            let Some(period) = period else {
                check(name, "none".into(), expected.to_string());
                continue;
            };
            check(
                &format!("{name} branch"),
                period.branch.into(),
                expected["branch"].as_str().unwrap().into(),
            );
            check(
                &format!("{name} stem"),
                period.stem.into(),
                expected["stem"].as_str().unwrap().into(),
            );
            let stars: Vec<&str> = period.transformations.iter().map(|t| t.star).collect();
            check(
                &format!("{name} transformations"),
                stars.join(" "),
                strings(&expected["transformations"]).join(" "),
            );
        }
    }
    assert_eq!(count, 80);
    assert!(
        mismatches.is_empty(),
        "{} mismatches:\n{}",
        mismatches.len(),
        mismatches[..mismatches.len().min(40)].join("\n")
    );
}
