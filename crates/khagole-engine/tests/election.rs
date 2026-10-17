//! Electional charts and searches on the real ephemeris.

mod common;

use std::time::Instant;

use khagole_engine::{
    calculate_election_chart, calculate_horary_chart, search_elections, ChartRequest,
    ElectionCriteria, ElectionSearch, EngineError, HourRange, NatalPoint, Purpose, UtcInstant,
    UtcOffset,
};
use common::*;

const ROME: (f64, f64) = (41.9, 12.5);

fn at(utc: &str) -> ChartRequest<'static> {
    ChartRequest::new(UtcInstant::parse(utc).unwrap(), ROME.0, ROME.1)
}

fn search(start: &str, end: &str, criteria: &ElectionCriteria) -> Option<ElectionSearch> {
    let kernels = kernels()?;
    let begun = Instant::now();
    let result = search_elections(
        &kernels,
        &at(start),
        UtcInstant::parse(end).unwrap(),
        criteria,
    )
    .unwrap();
    eprintln!(
        "search {start}..{end}: {} moments assessed in {:?}",
        result.evaluated,
        begun.elapsed()
    );
    Some(result)
}

#[test]
fn election_chart_agrees_with_the_horary_analysis() {
    let Some(kernels) = kernels() else { return };
    // Every hour for four days: the Moon changes sign and goes void of course.
    let mut instant = UtcInstant::parse("2026-10-01T00:00:00Z").unwrap();
    let mut voids = 0;
    for _ in 0..96 {
        let req = ChartRequest::new(instant, ROME.0, ROME.1);
        let election =
            calculate_election_chart(&kernels, &req, &ElectionCriteria::default()).unwrap();
        let horary = calculate_horary_chart(&kernels, &req, None).unwrap();
        let data = &election.election_data;
        assert_eq!(election.chart, horary.chart);
        assert_eq!(data.moon_status, horary.horary_data.moon_status);
        assert_eq!(data.planetary_hours, horary.horary_data.planetary_hours);
        let voc = data.moon_status.void_of_course;
        assert_eq!(data.factors.iter().any(|f| f.code == "MOON_VOC"), voc);
        assert_eq!(data.excluded_by.contains(&"void_moon"), voc);
        voids += usize::from(voc);
        assert!((0.0..=100.0).contains(&data.score));
        instant = instant.add_micros(3_600_000_000);
    }
    assert!(voids > 0, "no void-of-course Moon in four days");
}

#[test]
fn mercury_retrograde_counts_for_contracts() {
    let Some(kernels) = kernels() else { return };
    // Mercury is retrograde from 24 October to 13 November 2026.
    let contract = ElectionCriteria {
        purpose: Purpose::Contract,
        ..Default::default()
    };
    let data = calculate_election_chart(&kernels, &at("2026-11-01T10:00:00Z"), &contract)
        .unwrap()
        .election_data;
    let rx = data
        .factors
        .iter()
        .find(|f| f.code == "MERCURY_RETROGRADE")
        .expect("Mercury is retrograde");
    assert_eq!(rx.weight, -10.0);
    assert!(data.excluded_by.contains(&"mercury_retrograde"));

    // The whole retrograde station is left out of a contract search.
    let Some(result) = search("2026-10-26T00:00:00Z", "2026-11-10T00:00:00Z", &contract) else {
        return;
    };
    assert!(result.windows.is_empty());
    assert!(result.excluded.mercury_retrograde > 0);
}

#[test]
fn search_windows_are_ranked_disjoint_and_exact() {
    let criteria = ElectionCriteria {
        purpose: Purpose::Launch,
        ..Default::default()
    };
    let Some(result) = search("2026-12-01T00:00:00Z", "2026-12-31T00:00:00Z", &criteria) else {
        return;
    };
    let kernels = kernels().unwrap();
    assert!(!result.windows.is_empty(), "nothing found in a month");
    assert!(result.windows.len() <= 20);
    assert!(result.windows.windows(2).all(|w| w[0].score >= w[1].score));

    let mut spans: Vec<(UtcInstant, UtcInstant)> = Vec::new();
    for w in &result.windows {
        let (start, end, best) = (
            UtcInstant::parse(&w.start).unwrap(),
            UtcInstant::parse(&w.end).unwrap(),
            UtcInstant::parse(&w.best).unwrap(),
        );
        assert!(start <= best && best <= end);
        assert!(w.score >= 60.0);
        assert!(
            !spans.iter().any(|(s, e)| start <= *e && *s <= end),
            "overlap"
        );
        spans.push((start, end));

        // The best moment scores exactly what the election chart gives, and passes every
        // filter, as do the ends of the window.
        for (instant, is_best) in [(best, true), (start, false), (end, false)] {
            let req = ChartRequest::new(instant, ROME.0, ROME.1);
            let data = calculate_election_chart(&kernels, &req, &criteria)
                .unwrap()
                .election_data;
            assert!(
                data.excluded_by.is_empty(),
                "{instant:?}: {:?}",
                data.excluded_by
            );
            assert!(data.score >= 60.0);
            if is_best {
                assert_eq!(data.score, w.score);
                assert_eq!(data.factors, w.factors);
            }
        }
    }
}

#[test]
fn filters_by_local_hours_and_daylight() {
    let criteria = ElectionCriteria {
        daytime_only: true,
        local_hours: Some(HourRange { from: 9, to: 18 }),
        utc_offsets: vec![
            UtcOffset {
                from: "2026-03-01T00:00:00Z".into(),
                minutes: 60,
            },
            UtcOffset {
                from: "2026-03-29T01:00:00Z".into(),
                minutes: 120,
            },
        ],
        min_score: 0.0,
        ..Default::default()
    };
    let Some(result) = search("2026-03-20T00:00:00Z", "2026-04-05T00:00:00Z", &criteria) else {
        return;
    };
    assert!(result.excluded.outside_hours > 0);
    for w in &result.windows {
        for t in [&w.start, &w.end, &w.best] {
            let instant = UtcInstant::parse(t).unwrap();
            let offset = if t.as_str() >= "2026-03-29T01:00:00" {
                2
            } else {
                1
            };
            let (_, _, _, hour, ..) = instant.add_micros(offset * 3_600_000_000).civil();
            assert!((9..18).contains(&hour), "{t} is {hour}h local");
        }
    }
}

#[test]
fn natal_contacts_change_the_ranking_only_with_a_natal_chart() {
    let Some(kernels) = kernels() else { return };
    let natal = calculate_horary_chart(&kernels, &at("1987-05-17T14:30:00Z"), None)
        .unwrap()
        .chart
        .planets
        .iter()
        .map(|p| NatalPoint {
            name: p.name.into(),
            ecliptic_longitude: p.ecliptic_longitude,
        })
        .collect::<Vec<_>>();
    let personal = ElectionCriteria {
        natal: Some(natal),
        ..Default::default()
    };
    let Some(result) = search("2026-10-01T00:00:00Z", "2026-10-15T00:00:00Z", &personal) else {
        return;
    };
    assert!(result.criteria.natal);
    let natal_codes = result
        .windows
        .iter()
        .flat_map(|w| &w.factors)
        .filter(|f| f.code.starts_with("NATAL_"))
        .count();
    assert!(natal_codes > 0);
    let generic = search(
        "2026-10-01T00:00:00Z",
        "2026-10-15T00:00:00Z",
        &ElectionCriteria::default(),
    )
    .unwrap();
    assert!(generic
        .windows
        .iter()
        .flat_map(|w| &w.factors)
        .all(|f| !f.code.starts_with("NATAL_")));
}

#[test]
fn rejects_bad_spans() {
    let Some(kernels) = kernels() else { return };
    let criteria = ElectionCriteria::default();
    let start = at("2026-01-01T00:00:00Z");
    let too_long = UtcInstant::parse("2026-04-04T00:00:00Z").unwrap();
    let backwards = UtcInstant::parse("2025-12-31T00:00:00Z").unwrap();
    for end in [too_long, backwards] {
        assert!(matches!(
            search_elections(&kernels, &start, end, &criteria),
            Err(EngineError::InvalidInput(_))
        ));
    }
    // de440s ends in 2150.
    let late = at("2150-01-10T00:00:00Z");
    let end = UtcInstant::parse("2150-02-10T00:00:00Z").unwrap();
    assert!(matches!(
        search_elections(&kernels, &late, end, &criteria),
        Err(EngineError::OutOfRange { .. })
    ));
}
