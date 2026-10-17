//! The electional rules: a moment assessed into an [`ElectionData`].

use super::*;

/// The sign a longitude falls in (by its degree, not the rounded display).
pub(super) fn sign_of(longitude: f64) -> &'static str {
    SIGNS[(pyfloat::floordiv(pyfloat::rem(longitude, 360.0), 30.0) as usize) % 12]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Dignity {
    Dignified,
    Debilitated,
    Peregrine,
}

/// Essential dignity by domicile and exaltation, debility by detriment and fall.
pub(super) fn dignity(planet: &str, sign: &str) -> Dignity {
    if is_dignified(planet, sign) {
        Dignity::Dignified
    } else if is_debilitated(planet, sign) {
        Dignity::Debilitated
    } else {
        Dignity::Peregrine
    }
}

pub(super) fn is_angular(house: u8) -> bool {
    matches!(house, 1 | 4 | 7 | 10)
}

/// The 6th, 8th and 12th: houses of illness, death and undoing.
pub(super) fn is_dark_house(house: u8) -> bool {
    matches!(house, 6 | 8 | 12)
}

pub(super) fn is_soft(aspect: &str) -> bool {
    matches!(aspect, "Conjunction" | "Sextile" | "Trine")
}

/// How a contact with a benefic or a malefic counts: a benefic helps by conjunction,
/// sextile or trine; a malefic hurts by conjunction, square or opposition. `None` for the
/// contacts that count for nothing (a benefic's square, a malefic's trine).
pub(super) fn contact(benefic: bool, aspect: &str) -> Option<bool> {
    match (benefic, aspect) {
        (true, "Conjunction" | "Sextile" | "Trine") => Some(true),
        (false, "Conjunction" | "Square" | "Opposition") => Some(false),
        _ => None,
    }
}

/// A Ptolemaic aspect between two longitudes within `orb`, if any.
pub(super) fn ptolemaic_aspect(a: f64, b: f64, orb: f64) -> Option<&'static str> {
    let d = pyfloat::separation(a, b);
    [
        (0.0, "Conjunction"),
        (60.0, "Sextile"),
        (90.0, "Square"),
        (120.0, "Trine"),
        (180.0, "Opposition"),
    ]
    .into_iter()
    .find(|(angle, _)| (d - angle).abs() <= orb)
    .map(|(_, name)| name)
}

/// Whether `planet` is combust (within 8.5° of the Sun, but not cazimi).
pub(super) fn is_combust(planet: &Placement, sun: Option<&Placement>) -> bool {
    sun.is_some_and(|sun| {
        solar_phase(
            planet.name,
            planet.ecliptic_longitude,
            sun.ecliptic_longitude,
        ) == Some("combust")
    })
}

/// The condition of a significator (the Ascendant's ruler, the ruler of the house of the
/// matter, the natural significator), as factors named `{prefix}_{condition}`.
pub(super) struct Condition {
    pub(super) dignified: &'static str,
    pub(super) debilitated: &'static str,
    pub(super) angular: &'static str,
    pub(super) dark_house: &'static str,
    pub(super) retrograde: &'static str,
    pub(super) combust: &'static str,
}

pub(super) const ASC_RULER: Condition = Condition {
    dignified: "ASC_RULER_DIGNIFIED",
    debilitated: "ASC_RULER_DEBILITATED",
    angular: "ASC_RULER_ANGULAR",
    dark_house: "ASC_RULER_IN_DARK_HOUSE",
    retrograde: "ASC_RULER_RETROGRADE",
    combust: "ASC_RULER_COMBUST",
};
pub(super) const HOUSE_RULER: Condition = Condition {
    dignified: "HOUSE_RULER_DIGNIFIED",
    debilitated: "HOUSE_RULER_DEBILITATED",
    angular: "HOUSE_RULER_ANGULAR",
    dark_house: "HOUSE_RULER_IN_DARK_HOUSE",
    retrograde: "HOUSE_RULER_RETROGRADE",
    combust: "HOUSE_RULER_COMBUST",
};
pub(super) const SIGNIFICATOR: Condition = Condition {
    dignified: "SIGNIFICATOR_DIGNIFIED",
    debilitated: "SIGNIFICATOR_DEBILITATED",
    angular: "SIGNIFICATOR_ANGULAR",
    dark_house: "SIGNIFICATOR_IN_DARK_HOUSE",
    retrograde: "SIGNIFICATOR_RETROGRADE",
    combust: "SIGNIFICATOR_COMBUST",
};

/// Weights of a significator's conditions.
pub(super) struct ConditionWeights {
    pub(super) dignity: f64,
    pub(super) angular: f64,
    pub(super) dark_house: f64,
    pub(super) affliction: f64,
}

pub(super) fn condition(
    factors: &mut Vec<ElectionFactor>,
    names: &Condition,
    weights: &ConditionWeights,
    planet: &Placement,
    sun: Option<&Placement>,
    skip_retrograde: bool,
) {
    let sign = sign_of(planet.ecliptic_longitude);
    let tag = |code, weight| {
        ElectionFactor::new(code, weight)
            .planet(planet.name)
            .house(planet.house)
            .sign(sign)
    };
    match dignity(planet.name, sign) {
        Dignity::Dignified => factors.push(tag(names.dignified, weights.dignity)),
        Dignity::Debilitated => factors.push(tag(names.debilitated, -weights.dignity)),
        Dignity::Peregrine => {}
    }
    if is_angular(planet.house) {
        factors.push(tag(names.angular, weights.angular));
    } else if is_dark_house(planet.house) {
        factors.push(tag(names.dark_house, -weights.dark_house));
    }
    if planet.is_retrograde && !skip_retrograde {
        factors.push(tag(names.retrograde, -weights.affliction));
    }
    if is_combust(planet, sun) {
        factors.push(tag(names.combust, -weights.affliction));
    }
}

/// A moment's positions, ready to be assessed.
pub(super) struct Moment<'a> {
    pub(super) instant: UtcInstant,
    pub(super) planets: &'a [Placement],
    pub(super) cusps: &'a [f64],
    pub(super) hours: &'a PlanetaryHours,
}

impl Moment<'_> {
    pub(super) fn get(&self, name: &str) -> Option<&Placement> {
        self.planets.iter().find(|p| p.name == name)
    }
}

/// The filters a moment fails, in the order a search checks them.
pub(super) fn exclusions(
    moment: &Moment,
    moon: &MoonStatus,
    criteria: &Resolved,
) -> Vec<&'static str> {
    let c = &criteria.summary;
    let mut out = Vec::new();
    if let Some(range) = c.local_hours {
        if !range.contains(criteria.local_minute_of_day(moment.instant)) {
            out.push("outside_hours");
        }
    }
    if c.daytime_only && !moment.hours.is_day {
        out.push("night");
    }
    if c.avoid_void_moon && moon.void_of_course {
        out.push("void_moon");
    }
    let retrograde = |name| moment.get(name).is_some_and(|p| p.is_retrograde);
    if c.avoid_mercury_retrograde && retrograde("Mercury") {
        out.push("mercury_retrograde");
    }
    if c.avoid_venus_retrograde && retrograde("Venus") {
        out.push("venus_retrograde");
    }
    out
}

/// Score, factors and failed filters of a moment.
pub(super) fn assess(moment: &Moment, criteria: &Resolved) -> Result<ElectionData, EngineError> {
    let purpose = criteria.summary.purpose;
    let moon = moment
        .get("Moon")
        .ok_or_else(|| EngineError::InvalidInput("chart has no Moon".into()))?;
    let sun = moment.get("Sun");
    let status = moon_status(moon, moment.planets);
    let mut factors = Vec::new();

    // The Moon: her condition, and the next aspect she perfects.
    let moon_sign = sign_of(moon.ecliptic_longitude);
    if status.void_of_course {
        factors.push(ElectionFactor::new("MOON_VOC", -20.0).planet("Moon"));
    }
    if is_combust(moon, sun) {
        factors.push(ElectionFactor::new("MOON_COMBUST", -12.0).planet("Moon"));
    } else if let Some(sun) = sun {
        if pyfloat::rem(moon.ecliptic_longitude - sun.ecliptic_longitude, 360.0) < 180.0 {
            factors.push(ElectionFactor::new("MOON_WAXING", 5.0).planet("Moon"));
        }
    }
    let lon = pyfloat::rem(moon.ecliptic_longitude, 360.0);
    if (195.0..225.0).contains(&lon) {
        factors.push(ElectionFactor::new("MOON_VIA_COMBUSTA", -8.0).planet("Moon"));
    }
    degree_factors(&mut factors, moon.ecliptic_longitude, true);
    match dignity("Moon", moon_sign) {
        Dignity::Dignified => factors.push(
            ElectionFactor::new("MOON_DIGNIFIED", 6.0)
                .planet("Moon")
                .sign(moon_sign),
        ),
        Dignity::Debilitated => factors.push(
            ElectionFactor::new("MOON_DEBILITATED", -6.0)
                .planet("Moon")
                .sign(moon_sign),
        ),
        Dignity::Peregrine => {}
    }
    if moon.speed > 13.5 {
        factors.push(ElectionFactor::new("MOON_SWIFT", 2.0).planet("Moon"));
    } else if moon.speed < 12.5 {
        factors.push(ElectionFactor::new("MOON_SLOW", -2.0).planet("Moon"));
    }
    if is_angular(moon.house) {
        factors.push(
            ElectionFactor::new("MOON_ANGULAR", 3.0)
                .planet("Moon")
                .house(moon.house),
        );
    } else if is_dark_house(moon.house) {
        factors.push(
            ElectionFactor::new("MOON_IN_DARK_HOUSE", -5.0)
                .planet("Moon")
                .house(moon.house),
        );
    }
    if let Some(next) = &status.next_applying_aspect {
        let soft = is_soft(next.aspect);
        let applying = |code, weight| {
            ElectionFactor::new(code, weight)
                .planet("Moon")
                .target(next.planet)
                .aspect(next.aspect)
        };
        if BENEFICS.contains(&next.planet) {
            factors.push(applying(
                "MOON_APPLYING_BENEFIC",
                if soft { 10.0 } else { 4.0 },
            ));
        } else if MALEFICS.contains(&next.planet) {
            // A conjunction with a malefic is no help.
            let hard = !soft || next.aspect == "Conjunction";
            factors.push(applying(
                "MOON_APPLYING_MALEFIC",
                if hard { -10.0 } else { -3.0 },
            ));
        }
        if purpose.significator() == Some(next.planet) {
            factors.push(applying(
                "MOON_APPLYING_SIGNIFICATOR",
                if soft { 5.0 } else { -3.0 },
            ));
        }
    }

    // The Ascendant and its ruler.
    let asc_lon = moment
        .get("Ascendant")
        .map_or(moment.cusps[0], |a| a.ecliptic_longitude);
    let asc_sign = sign_of(asc_lon);
    let asc_degree = pyfloat::rem(asc_lon, 30.0);
    if asc_degree < 3.0 {
        factors.push(ElectionFactor::new("ASC_EARLY", -5.0).sign(asc_sign));
    } else if asc_degree >= 27.0 {
        factors.push(ElectionFactor::new("ASC_LATE", -5.0).sign(asc_sign));
    }
    degree_factors(&mut factors, asc_lon, false);
    let asc_ruler_name = traditional_ruler(asc_sign);
    if let Some(ruler) = asc_ruler_name.and_then(|r| moment.get(r)) {
        condition(
            &mut factors,
            &ASC_RULER,
            &ConditionWeights {
                dignity: 6.0,
                angular: 4.0,
                dark_house: 5.0,
                affliction: 6.0,
            },
            ruler,
            sun,
            false,
        );
    }

    // Benefics and malefics on the angles.
    for p in moment.planets {
        let benefic = BENEFICS.contains(&p.name);
        let malefic = MALEFICS.contains(&p.name);
        if !(benefic || malefic) {
            continue;
        }
        let (code, weight) = match (benefic, p.house) {
            (true, 1) => ("BENEFIC_IN_1ST", 8.0),
            (true, h) if is_angular(h) => ("BENEFIC_ANGULAR", 4.0),
            (false, 1) => ("MALEFIC_IN_1ST", -10.0),
            (false, h) if is_angular(h) => ("MALEFIC_ANGULAR", -5.0),
            _ => continue,
        };
        factors.push(
            ElectionFactor::new(code, weight)
                .planet(p.name)
                .house(p.house),
        );
    }

    // Retrograde Mercury and Venus: heavy when the matter needs them.
    let feared = [
        (
            "Mercury",
            "MERCURY_RETROGRADE",
            criteria.summary.avoid_mercury_retrograde,
        ),
        (
            "Venus",
            "VENUS_RETROGRADE",
            criteria.summary.avoid_venus_retrograde,
        ),
    ];
    for (name, code, avoided) in feared {
        if moment.get(name).is_some_and(|p| p.is_retrograde) {
            let weight = if avoided || purpose.significator() == Some(name) {
                -10.0
            } else {
                -2.0
            };
            factors.push(ElectionFactor::new(code, weight).planet(name));
        }
    }

    // The matter: the ruler of its house, and its natural significator.
    let house_ruler_name = purpose
        .house()
        .filter(|h| *h != 1)
        .and_then(|h| traditional_ruler(sign_of(moment.cusps[h - 1])));
    if let Some(ruler) = house_ruler_name.and_then(|r| moment.get(r)) {
        if Some(ruler.name) != asc_ruler_name {
            condition(
                &mut factors,
                &HOUSE_RULER,
                &ConditionWeights {
                    dignity: 5.0,
                    angular: 3.0,
                    dark_house: 4.0,
                    affliction: 5.0,
                },
                ruler,
                sun,
                false,
            );
        }
    }
    if let Some(sig) = purpose.significator().and_then(|s| moment.get(s)) {
        if Some(sig.name) != asc_ruler_name && Some(sig.name) != house_ruler_name {
            condition(
                &mut factors,
                &SIGNIFICATOR,
                &ConditionWeights {
                    dignity: 4.0,
                    angular: 2.0,
                    dark_house: 3.0,
                    affliction: 4.0,
                },
                sig,
                sun,
                // Mercury's and Venus's retrogradation is weighed above.
                matches!(sig.name, "Mercury" | "Venus"),
            );
        }
    }

    // The planetary hour.
    let hour_ruler = moment.hours.hour_ruler;
    if purpose.hour_rulers().contains(&hour_ruler) {
        factors.push(ElectionFactor::new("HOUR_RULER_FAVOURS_PURPOSE", 5.0).planet(hour_ruler));
    } else if MALEFICS.contains(&hour_ruler) {
        factors.push(ElectionFactor::new("HOUR_RULER_MALEFIC", -3.0).planet(hour_ruler));
    }

    // The natal chart.
    let natal = &criteria.natal;
    if !natal.is_empty() {
        natal_factors(&mut factors, moment, asc_sign, natal);
    }

    let score = pyfloat::round(
        (50.0 + factors.iter().map(|f| f.weight).sum::<f64>()).clamp(0.0, 100.0),
        1,
    );
    Ok(ElectionData {
        score,
        verdict: verdict(score),
        purpose,
        factors,
        excluded_by: exclusions(moment, &status, criteria),
        planetary_hours: moment.hours.clone(),
        moon_status: status,
    })
}

/// Lilly's qualities of the Moon's or the Ascendant's degree: pitted and lame degrees
/// hinder, degrees increasing fortune help and, for the Ascendant, light degrees help while
/// dark and void ones hinder (smoky degrees are between the two).
pub(super) fn degree_factors(factors: &mut Vec<ElectionFactor>, longitude: f64, moon: bool) {
    let q = degree_qualities(longitude);
    let tag = |code: &'static str, weight: f64| {
        let factor = ElectionFactor::new(code, weight).sign(q.sign);
        if moon {
            factor.planet("Moon")
        } else {
            factor
        }
    };
    let pick = |moon_code, asc_code| if moon { moon_code } else { asc_code };
    if q.pitted {
        factors.push(tag(pick("MOON_PITTED_DEGREE", "ASC_PITTED_DEGREE"), -4.0));
    }
    if q.azimene {
        factors.push(tag(pick("MOON_AZIMENE_DEGREE", "ASC_AZIMENE_DEGREE"), -3.0));
    }
    if q.fortune {
        let weight = if moon { 3.0 } else { 4.0 };
        factors.push(tag(
            pick("MOON_FORTUNE_DEGREE", "ASC_FORTUNE_DEGREE"),
            weight,
        ));
    }
    if !moon {
        match q.light {
            "light" => factors.push(tag("ASC_LIGHT_DEGREE", 2.0)),
            "dark" | "void" => factors.push(tag("ASC_DARK_DEGREE", -2.0)),
            _ => {}
        }
    }
}

pub(super) fn verdict(score: f64) -> &'static str {
    if score >= FAVOURABLE {
        "favourable"
    } else if score >= MIXED {
        "mixed"
    } else {
        "unfavourable"
    }
}

/// The election's contacts with a natal chart: the election's Ascendant counted from the
/// natal one, benefics and malefics on the natal lights, angles and Ascendant ruler, and
/// the election Moon's aspects to natal benefics and malefics.
pub(super) fn natal_factors(
    factors: &mut Vec<ElectionFactor>,
    moment: &Moment,
    asc_sign: &'static str,
    natal: &[NatalPoint],
) {
    let natal_point = |name: &str| natal.iter().find(|p| p.name == name);

    if let Some(natal_asc) = natal_point("Ascendant") {
        let natal_asc_sign = sign_of(natal_asc.ecliptic_longitude);
        let index = |sign| sign_index(sign).unwrap_or(0);
        let place = (index(asc_sign) + 12 - index(natal_asc_sign)) % 12 + 1;
        let place = place as u8;
        if matches!(place, 1 | 5 | 9 | 10 | 11) {
            factors.push(ElectionFactor::new("NATAL_ASC_WELL_PLACED", 5.0).house(place));
        } else if is_dark_house(place) {
            factors.push(ElectionFactor::new("NATAL_ASC_BADLY_PLACED", -6.0).house(place));
        }
    }

    // Sensitive natal points: the lights, the angles and the Ascendant's ruler.
    let mut sensitive: Vec<&NatalPoint> = ["Sun", "Moon", "Ascendant", "Midheaven"]
        .iter()
        .filter_map(|n| natal_point(n))
        .collect();
    if let Some(ruler) = natal_point("Ascendant")
        .and_then(|a| traditional_ruler(sign_of(a.ecliptic_longitude)))
        .and_then(natal_point)
    {
        if !sensitive.iter().any(|p| p.name == ruler.name) {
            sensitive.push(ruler);
        }
    }
    for p in moment.planets {
        let benefic = BENEFICS.contains(&p.name);
        if !(benefic || MALEFICS.contains(&p.name)) {
            continue;
        }
        for point in &sensitive {
            let Some(aspect) =
                ptolemaic_aspect(p.ecliptic_longitude, point.ecliptic_longitude, NATAL_ORB)
            else {
                continue;
            };
            let (code, weight) = match contact(benefic, aspect) {
                Some(true) => ("NATAL_BENEFIC_CONTACT", 4.0),
                Some(false) => ("NATAL_MALEFIC_CONTACT", -6.0),
                None => continue,
            };
            factors.push(
                ElectionFactor::new(code, weight)
                    .planet(p.name)
                    .target(point.name.clone())
                    .aspect(aspect),
            );
        }
    }

    if let Some(moon) = moment.get("Moon") {
        for point in natal {
            let benefic = BENEFICS.contains(&point.name.as_str());
            if !(benefic || MALEFICS.contains(&point.name.as_str())) {
                continue;
            }
            let Some(aspect) =
                ptolemaic_aspect(moon.ecliptic_longitude, point.ecliptic_longitude, NATAL_ORB)
            else {
                continue;
            };
            let (code, weight) = match contact(benefic, aspect) {
                Some(true) => ("NATAL_MOON_TO_BENEFIC", 3.0),
                Some(false) => ("NATAL_MOON_TO_MALEFIC", -4.0),
                None => continue,
            };
            factors.push(
                ElectionFactor::new(code, weight)
                    .planet("Moon")
                    .target(point.name.clone())
                    .aspect(aspect),
            );
        }
    }
}
