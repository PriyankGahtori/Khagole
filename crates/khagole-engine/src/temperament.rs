//! Traditional temperament from weighted primary qualities (`charts/calc/temperament.py`).

use serde::Serialize;

use crate::aspects::Aspect;
use crate::chart::Placement;
use crate::dignities::{element as element_of, traditional_ruler};
use crate::pyfloat;

/// Hot, cold, wet, dry.
type Q = [f64; 4];

fn element_qualities(element: &str) -> Q {
    match element {
        "Fire" => [1.0, 0.0, 0.0, 1.0],
        "Air" => [1.0, 0.0, 1.0, 0.0],
        "Water" => [0.0, 1.0, 1.0, 0.0],
        "Earth" => [0.0, 1.0, 0.0, 1.0],
        _ => [0.0; 4],
    }
}

fn planet_qualities(planet: &str) -> Option<Q> {
    Some(match planet {
        "Sun" | "Mars" | "Uranus" | "Pluto" => [1.0, 0.0, 0.0, 1.0],
        "Moon" | "Venus" | "Neptune" => [0.0, 1.0, 1.0, 0.0],
        "Mercury" | "Saturn" => [0.0, 1.0, 0.0, 1.0],
        "Jupiter" => [1.0, 0.0, 1.0, 0.0],
        _ => return None,
    })
}

/// The four primary qualities.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Qualities {
    /// Hot.
    pub hot: f64,
    /// Cold.
    pub cold: f64,
    /// Wet.
    pub wet: f64,
    /// Dry.
    pub dry: f64,
}

/// The four temperaments.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Scores {
    /// Choleric (hot and dry).
    pub choleric: f64,
    /// Sanguine (hot and wet).
    pub sanguine: f64,
    /// Phlegmatic (cold and wet).
    pub phlegmatic: f64,
    /// Melancholic (cold and dry).
    pub melancholic: f64,
}

/// One contribution to the temperament, with its weighted qualities.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Factor {
    /// What contributes, e.g. "Ascendant Sign".
    pub factor: &'static str,
    /// The placement behind it, e.g. "Leo (Fire)".
    pub details: String,
    /// Weighted hot.
    pub hot: f64,
    /// Weighted cold.
    pub cold: f64,
    /// Weighted wet.
    pub wet: f64,
    /// Weighted dry.
    pub dry: f64,
}

/// Temperament assessment from the chart's qualities.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Temperament {
    /// Highest-scoring temperament.
    pub primary_temperament: &'static str,
    /// Second-highest temperament, or "" when it scores zero.
    pub secondary_temperament: &'static str,
    /// Temperament scores.
    pub scores: Scores,
    /// Temperament scores as percentages of their total.
    pub percentages: Scores,
    /// Totals of the four qualities.
    pub qualities: Qualities,
    /// Quality totals as percentages.
    pub quality_percentages: Qualities,
    /// Every contributing factor.
    pub breakdown: Vec<Factor>,
}

pub fn temperament(planets: &[Placement], aspects: &[Aspect]) -> Temperament {
    let mut totals: Q = [0.0; 4];
    let mut breakdown = Vec::new();
    let mut add = |factor: &'static str, details: String, q: Q, weight: f64| {
        let w = q.map(|v| v * weight);
        for i in 0..4 {
            totals[i] += w[i];
        }
        breakdown.push(Factor {
            factor,
            details,
            hot: pyfloat::round(w[0], 1),
            cold: pyfloat::round(w[1], 1),
            wet: pyfloat::round(w[2], 1),
            dry: pyfloat::round(w[3], 1),
        });
    };
    // Python builds a name → planet dict: the last entry of a name wins.
    let by_name = |name: &str| planets.iter().rev().find(|p| p.name == name);
    let sun = by_name("Sun");
    let moon = by_name("Moon");

    if let Some(asc) = by_name("Ascendant") {
        let element = element_of(asc.sign).unwrap_or("Fire");
        add(
            "Ascendant Sign",
            format!("{} ({element})", asc.sign),
            element_qualities(element),
            4.0,
        );
        let lord = traditional_ruler(asc.sign).unwrap_or("Mars");
        if let Some(lord_planet) = by_name(lord) {
            add(
                "Lord of Ascendant Nature",
                lord.to_string(),
                planet_qualities(lord).unwrap_or_default(),
                3.0,
            );
            let lord_element = element_of(lord_planet.sign).unwrap_or("Fire");
            add(
                "Lord of Ascendant Sign",
                format!("{lord} in {} ({lord_element})", lord_planet.sign),
                element_qualities(lord_element),
                3.0,
            );
        }
    }

    if let Some(sun) = sun {
        let element = element_of(sun.sign).unwrap_or("Fire");
        add(
            "Sun Sign",
            format!("{} ({element})", sun.sign),
            element_qualities(element),
            4.0,
        );
        let (season, q) = match sun.sign {
            "Aries" | "Taurus" | "Gemini" => ("Spring (Hot & Wet)", [1.0, 0.0, 1.0, 0.0]),
            "Cancer" | "Leo" | "Virgo" => ("Summer (Hot & Dry)", [1.0, 0.0, 0.0, 1.0]),
            "Libra" | "Scorpio" | "Sagittarius" => ("Autumn (Cold & Dry)", [0.0, 1.0, 0.0, 1.0]),
            _ => ("Winter (Cold & Wet)", [0.0, 1.0, 1.0, 0.0]),
        };
        add("Solar Season", season.to_string(), q, 2.0);
    }

    if let Some(moon) = moon {
        let element = element_of(moon.sign).unwrap_or("Water");
        add(
            "Moon Sign",
            format!("{} ({element})", moon.sign),
            element_qualities(element),
            4.0,
        );
        if let Some(sun) = sun {
            let diff = pyfloat::rem(moon.ecliptic_longitude - sun.ecliptic_longitude, 360.0);
            let (phase, q) = if diff < 90.0 {
                ("1st Quarter Waxing (Hot & Wet)", [1.0, 0.0, 1.0, 0.0])
            } else if diff < 180.0 {
                ("2nd Quarter Gibbous (Hot & Dry)", [1.0, 0.0, 0.0, 1.0])
            } else if diff < 270.0 {
                ("3rd Quarter Full/Waning (Cold & Dry)", [0.0, 1.0, 0.0, 1.0])
            } else {
                ("4th Quarter Crescent (Cold & Wet)", [0.0, 1.0, 1.0, 0.0])
            };
            add("Lunar Phase", phase.to_string(), q, 2.0);
        }
    }

    for p in planets {
        if matches!(
            p.name,
            "Ascendant" | "Midheaven" | "North Node" | "South Node" | "Part of Fortune"
        ) {
            continue;
        }
        if p.house == 1 {
            let pq = planet_qualities(p.name).unwrap_or_default();
            let sq = element_qualities(element_of(p.sign).unwrap_or("Fire"));
            let combined = [0, 1, 2, 3].map(|i| pq[i] + sq[i]);
            add(
                "Planet in House 1",
                format!("{} in {}", p.name, p.sign),
                combined,
                1.0,
            );
        }
    }

    for a in aspects {
        if a.body1 == "Ascendant" || a.body2 == "Ascendant" {
            let other = if a.body1 == "Ascendant" {
                &a.body2
            } else {
                &a.body1
            };
            if let Some(q) = planet_qualities(other) {
                add(
                    "Aspect to Ascendant",
                    format!("{other} {} ASC", a.aspect_type),
                    q,
                    1.0,
                );
            }
        }
    }

    let [hot, cold, wet, dry] = totals;
    let (choleric, sanguine, phlegmatic, melancholic) =
        (hot + dry, hot + wet, cold + wet, cold + dry);
    let total_q = (hot + cold + wet + dry).max(1.0);
    let total_t = (choleric + sanguine + phlegmatic + melancholic).max(1.0);
    let pct = |v: f64, total: f64| pyfloat::round((v / total) * 100.0, 1);

    // Stable sort, highest first: ties keep this order, as Python's sorted(reverse=True).
    let mut ranked = [
        ("Choleric", choleric),
        ("Sanguine", sanguine),
        ("Phlegmatic", phlegmatic),
        ("Melancholic", melancholic),
    ];
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    Temperament {
        primary_temperament: ranked[0].0,
        secondary_temperament: if ranked[1].1 > 0.0 { ranked[1].0 } else { "" },
        scores: Scores {
            choleric: pyfloat::round(choleric, 1),
            sanguine: pyfloat::round(sanguine, 1),
            phlegmatic: pyfloat::round(phlegmatic, 1),
            melancholic: pyfloat::round(melancholic, 1),
        },
        percentages: Scores {
            choleric: pct(choleric, total_t),
            sanguine: pct(sanguine, total_t),
            phlegmatic: pct(phlegmatic, total_t),
            melancholic: pct(melancholic, total_t),
        },
        qualities: Qualities {
            hot: pyfloat::round(hot, 1),
            cold: pyfloat::round(cold, 1),
            wet: pyfloat::round(wet, 1),
            dry: pyfloat::round(dry, 1),
        },
        quality_percentages: Qualities {
            hot: pct(hot, total_q),
            cold: pct(cold, total_q),
            wet: pct(wet, total_q),
            dry: pct(dry, total_q),
        },
        breakdown,
    }
}
