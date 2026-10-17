use super::search::*;
use super::*;

fn placement(name: &'static str, longitude: f64, house: u8, speed: f64) -> Placement {
    Placement {
        name,
        symbol: "",
        sign: sign_of(longitude),
        sign_symbol: "",
        degree: pyfloat::rem(longitude, 30.0) as i64,
        minute: 0,
        ecliptic_longitude: longitude,
        house,
        speed,
        is_retrograde: speed < 0.0 && name != "Moon" && name != "Sun",
        symbolic_degree: 1,
        condition: None,
    }
}

fn hours(ruler: &'static str, is_day: bool) -> PlanetaryHours {
    PlanetaryHours {
        is_day,
        day_ruler: "Sun",
        hour_ruler: ruler,
        hour_number: 1,
        hour_type: if is_day { "Day" } else { "Night" },
        sunrise: String::new(),
        sunset: String::new(),
    }
}

/// Cusps every 30° from 0° Aries: house n starts at (n-1)·30°.
fn cusps() -> Vec<f64> {
    (0..12).map(|i| f64::from(i) * 30.0).collect()
}

fn assess_with(planets: &[Placement], hour: &PlanetaryHours, c: &ElectionCriteria) -> ElectionData {
    let cusps = cusps();
    assess(
        &Moment {
            instant: UtcInstant::parse("2026-10-01T12:00:00Z").unwrap(),
            planets,
            cusps: &cusps,
            hours: hour,
        },
        &c.resolve().unwrap(),
    )
    .unwrap()
}

fn codes(data: &ElectionData) -> Vec<&'static str> {
    data.factors.iter().map(|f| f.code).collect()
}

/// A pleasant sky: Moon in Taurus applying by trine to Jupiter, Venus in the 1st.
fn good_sky() -> Vec<Placement> {
    vec![
        placement("Sun", 100.0, 4, 1.0),
        placement("Moon", 40.0, 2, 14.0),
        placement("Mercury", 110.0, 4, 1.2),
        placement("Venus", 15.0, 1, 1.1),
        placement("Mars", 160.0, 6, 0.6),
        placement("Jupiter", 165.0, 6, 0.1),
        placement("Saturn", 320.0, 11, 0.05),
        placement("Ascendant", 10.0, 1, 0.0),
        placement("Midheaven", 280.0, 10, 0.0),
    ]
}

#[test]
fn a_good_moment_scores_favourable() {
    let data = assess_with(
        &good_sky(),
        &hours("Jupiter", true),
        &ElectionCriteria::default(),
    );
    let codes = codes(&data);
    assert!(codes.contains(&"MOON_APPLYING_BENEFIC"), "{codes:?}");
    assert!(codes.contains(&"MOON_DIGNIFIED"));
    assert!(codes.contains(&"BENEFIC_IN_1ST"));
    assert!(codes.contains(&"HOUR_RULER_FAVOURS_PURPOSE"));
    assert!(!codes.contains(&"MOON_VOC"));
    assert_eq!(data.verdict, "favourable");
    assert!(data.excluded_by.is_empty());
}

#[test]
fn a_void_moon_is_penalized_and_excluded() {
    // Moon at 29° Taurus: no aspect left before Gemini.
    let mut sky = good_sky();
    sky[1] = placement("Moon", 59.5, 2, 14.0);
    let data = assess_with(&sky, &hours("Jupiter", true), &ElectionCriteria::default());
    assert!(codes(&data).contains(&"MOON_VOC"));
    assert_eq!(data.excluded_by, vec!["void_moon"]);
    let keep = ElectionCriteria {
        avoid_void_moon: false,
        ..Default::default()
    };
    assert!(assess_with(&sky, &hours("Jupiter", true), &keep)
        .excluded_by
        .is_empty());
}

#[test]
fn mercury_retrograde_weighs_on_contracts() {
    let mut sky = good_sky();
    sky[2] = placement("Mercury", 110.0, 4, -0.5);
    let general = assess_with(&sky, &hours("Sun", true), &ElectionCriteria::default());
    let contract = assess_with(
        &sky,
        &hours("Sun", true),
        &ElectionCriteria {
            purpose: Purpose::Contract,
            ..Default::default()
        },
    );
    let weight = |d: &ElectionData| {
        d.factors
            .iter()
            .find(|f| f.code == "MERCURY_RETROGRADE")
            .unwrap()
            .weight
    };
    assert_eq!(weight(&general), -2.0);
    assert_eq!(weight(&contract), -10.0);
    assert!(general.excluded_by.is_empty());
    assert_eq!(contract.excluded_by, vec!["mercury_retrograde"]);
}

#[test]
fn degree_qualities_of_the_moon_and_the_ascendant() {
    // Ascendant in the 11th degree of Aries (pitted, dark); Moon in the 8th of Taurus
    // (lame).
    let mut sky = good_sky();
    sky[7] = placement("Ascendant", 10.5, 1, 0.0);
    sky[1] = placement("Moon", 37.5, 2, 14.0);
    let data = assess_with(&sky, &hours("Jupiter", true), &ElectionCriteria::default());
    let found = codes(&data);
    for code in [
        "ASC_PITTED_DEGREE",
        "ASC_DARK_DEGREE",
        "MOON_AZIMENE_DEGREE",
    ] {
        assert!(found.contains(&code), "{code} missing from {found:?}");
    }
    let lame = data
        .factors
        .iter()
        .find(|f| f.code == "MOON_AZIMENE_DEGREE")
        .unwrap();
    assert_eq!(
        (lame.planet, lame.sign, lame.weight),
        (Some("Moon"), Some("Taurus"), -3.0)
    );

    // Ascendant in the 19th of Aries (light, increasing fortune); Moon in the 3rd of
    // Taurus (increasing fortune).
    sky[7] = placement("Ascendant", 18.5, 1, 0.0);
    sky[1] = placement("Moon", 32.5, 2, 14.0);
    let data = assess_with(&sky, &hours("Jupiter", true), &ElectionCriteria::default());
    let found = codes(&data);
    for code in [
        "ASC_FORTUNE_DEGREE",
        "ASC_LIGHT_DEGREE",
        "MOON_FORTUNE_DEGREE",
    ] {
        assert!(found.contains(&code), "{code} missing from {found:?}");
    }
    assert!(!found.iter().any(|c| c.ends_with("PITTED_DEGREE")));
}

#[test]
fn malefics_on_the_ascendant_and_bad_hours_hurt() {
    let mut sky = good_sky();
    sky[6] = placement("Saturn", 20.0, 1, 0.05);
    let data = assess_with(&sky, &hours("Saturn", false), &ElectionCriteria::default());
    let codes = codes(&data);
    assert!(codes.contains(&"MALEFIC_IN_1ST"));
    assert!(codes.contains(&"HOUR_RULER_MALEFIC"));
}

#[test]
fn natal_factors_only_with_a_natal_chart() {
    let natal = vec![
        NatalPoint {
            name: "Ascendant".into(),
            ecliptic_longitude: 250.0, // Sagittarius: the election's Aries is its 5th
        },
        NatalPoint {
            name: "Sun".into(),
            ecliptic_longitude: 135.5, // trine the election's Venus at 15°
        },
        NatalPoint {
            name: "Moon".into(),
            ecliptic_longitude: 250.0,
        },
        NatalPoint {
            name: "Jupiter".into(),
            ecliptic_longitude: 280.0, // trine the election's Moon at 40°
        },
    ];
    let sky = good_sky();
    let without = assess_with(&sky, &hours("Sun", true), &ElectionCriteria::default());
    assert!(!codes(&without).iter().any(|c| c.starts_with("NATAL_")));
    let with = assess_with(
        &sky,
        &hours("Sun", true),
        &ElectionCriteria {
            natal: Some(natal),
            ..Default::default()
        },
    );
    let codes = codes(&with);
    assert!(codes.contains(&"NATAL_ASC_WELL_PLACED"), "{codes:?}");
    assert!(codes.contains(&"NATAL_BENEFIC_CONTACT"));
    assert!(codes.contains(&"NATAL_MOON_TO_BENEFIC"));
}

#[test]
fn local_hours_follow_the_utc_offset_and_wrap_midnight() {
    let criteria = ElectionCriteria {
        local_hours: Some(HourRange { from: 22, to: 2 }),
        utc_offsets: vec![UtcOffset {
            from: "2026-01-01T00:00:00Z".into(),
            minutes: 120,
        }],
        ..Default::default()
    }
    .resolve()
    .unwrap();
    let at = |s| criteria.local_minute_of_day(UtcInstant::parse(s).unwrap());
    assert_eq!(at("2026-10-01T21:30:00Z"), 23 * 60 + 30);
    let range = criteria.summary.local_hours.unwrap();
    assert!(range.contains(at("2026-10-01T21:30:00Z")));
    assert!(range.contains(at("2026-10-01T23:59:00Z")));
    assert!(!range.contains(at("2026-10-02T00:00:00Z")));
}

#[test]
fn criteria_parse_with_defaults_and_reject_typos() {
    let c = ElectionCriteria::from_value(&serde_json::json!({"purpose": "travel"})).unwrap();
    assert_eq!(c.purpose, Purpose::Travel);
    assert!(c.avoid_void_moon);
    let r = c.resolve().unwrap();
    assert!(r.summary.avoid_mercury_retrograde);
    assert!(!r.summary.avoid_venus_retrograde);
    assert!(ElectionCriteria::from_value(&serde_json::json!({"purpose": "war"})).is_err());
    assert!(ElectionCriteria::from_value(&serde_json::json!({"avoid_voc": true})).is_err());
    let hours = serde_json::json!({"local_hours": {"from": 9, "to": 9}});
    assert!(ElectionCriteria::from_value(&hours)
        .unwrap()
        .resolve()
        .is_err());
}

#[test]
fn dignities() {
    assert_eq!(dignity("Mercury", "Virgo"), Dignity::Dignified);
    assert_eq!(dignity("Mercury", "Pisces"), Dignity::Debilitated);
    assert_eq!(dignity("Saturn", "Libra"), Dignity::Dignified);
    assert_eq!(dignity("Mars", "Gemini"), Dignity::Peregrine);
    assert_eq!(sign_of(359.99), "Pisces");
    assert_eq!(sign_of(-0.5), "Pisces");
}

use crate::test_kernel::full_kernel as kernels;

#[test]
fn interpolated_moments_match_exact_ones() {
    let Some(kernels) = kernels() else { return };
    for (zodiac, system) in [("tropical", "P"), ("sidereal", "W")] {
        let mut origin = ChartRequest::new(
            UtcInstant::parse("2026-10-01T00:00:00Z").unwrap(),
            41.9,
            12.5,
        );
        origin.zodiac_type = zodiac;
        origin.house_system = system;
        origin.ayanamsa = "lahiri";
        let mut sampler = Sampler::new(&kernels, &origin);
        let criteria = ElectionCriteria::default().resolve().unwrap();
        let hours = hours("Sun", true);
        let (mut same, mut total) = (0, 0);
        // Every 7 minutes for four days: off the grid most of the time.
        for k in 0..(4 * 24 * 60 / 7) {
            let instant = origin.instant.add_micros(k * 7 * 60_000_000);
            let (planets, cusps) = sampler.at(instant).unwrap();
            let exact = sky(
                &kernels,
                &ChartRequest {
                    instant,
                    ..origin.clone()
                },
            )
            .unwrap();
            for (p, e) in planets.iter().zip(&exact.planets) {
                assert_eq!(p.name, e.name);
                let off = pyfloat::separation(p.ecliptic_longitude, e.ecliptic_longitude) * 3600.0;
                assert!(off < 1.0, "{} off by {off}\" at {instant:?}", p.name);
                if k % (120 / 7 + 1) == 0 && instant.micros() % 7_200_000_000 == 0 {
                    assert_eq!(p, e, "grid points are exact");
                }
            }
            for (c, e) in cusps.iter().zip(&exact.cusps) {
                assert!(
                    pyfloat::separation(*c, *e) * 3600.0 < 0.1,
                    "cusp off at {instant:?}"
                );
            }
            let moment = |planets, cusps| Moment {
                instant,
                planets,
                cusps,
                hours: &hours,
            };
            let a = assess(&moment(&planets, &cusps), &criteria).unwrap();
            let b = assess(&moment(&exact.planets, &exact.cusps), &criteria).unwrap();
            if codes(&a) != codes(&b) {
                eprintln!("{instant:?}: {:?} vs {:?}", codes(&a), codes(&b));
            }
            same += usize::from(codes(&a) == codes(&b));
            total += 1;
        }
        assert!(
            same * 1000 >= total * 998,
            "{zodiac}: {same}/{total} moments agree"
        );
    }
}
