//! Time-lords of a nativity: annual profections and the firdaria.
//!
//! Not part of the reference implementation. Neither needs the ephemeris: they unfold from
//! the moment of birth, the natal Ascendant and the sect of the chart.

use serde::Serialize;

use crate::aspects::finite;
use crate::dignities::{is_diurnal, traditional_ruler, SEVEN, SIGNS};
use crate::error::EngineError;
use crate::instant::UtcInstant;
use crate::pyfloat;

/// A tropical year, in days: the years of profections and firdaria are counted from birth.
const YEAR_DAYS: f64 = 365.242_19;

/// Firdaria of a day birth: lord and years (Bonatti, the nodes last).
const DIURNAL_FIRDARIA: [(&str, f64); 9] = [
    ("Sun", 10.0),
    ("Venus", 8.0),
    ("Mercury", 13.0),
    ("Moon", 9.0),
    ("Saturn", 11.0),
    ("Jupiter", 12.0),
    ("Mars", 7.0),
    ("North Node", 3.0),
    ("South Node", 2.0),
];

/// Firdaria of a night birth.
const NOCTURNAL_FIRDARIA: [(&str, f64); 9] = [
    ("Moon", 9.0),
    ("Saturn", 11.0),
    ("Jupiter", 12.0),
    ("Mars", 7.0),
    ("Sun", 10.0),
    ("Venus", 8.0),
    ("Mercury", 13.0),
    ("North Node", 3.0),
    ("South Node", 2.0),
];

/// The years of a full round of firdaria.
const FIRDARIA_CYCLE: f64 = 75.0;

/// One year of life and the sign the Ascendant has profected to.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Profection {
    /// Completed years of age at the start of the year.
    pub age: u32,
    /// Start of the year (a birthday), ISO 8601 UTC.
    pub start: String,
    /// End of the year (the next birthday), ISO 8601 UTC.
    pub end: String,
    /// The profected sign: one sign for each year from the natal Ascendant.
    pub sign: &'static str,
    /// The lord of the year: the ruler of the profected sign.
    pub lord: &'static str,
    /// The activated house (whole signs from the Ascendant), 1-12.
    pub house: u8,
}

/// A span of time ruled by a planet.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Period {
    /// The ruling planet (or lunar node).
    pub lord: &'static str,
    /// Start, ISO 8601 UTC.
    pub start: String,
    /// End, ISO 8601 UTC.
    pub end: String,
}

/// A major period of the firdaria, with its seven sub-periods (none for the nodes).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Firdaria {
    /// The major lord.
    pub lord: &'static str,
    /// Start, ISO 8601 UTC.
    pub start: String,
    /// End, ISO 8601 UTC.
    pub end: String,
    /// Sub-periods: the major lord first, then the planets in Chaldean order.
    pub sub_periods: Vec<Period>,
}

/// The time-lords active over a span of a life.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimeLords {
    /// Whether the birth is diurnal (the Sun above the horizon), which orders the firdaria.
    pub diurnal: bool,
    /// The profection years that overlap the span.
    pub profections: Vec<Profection>,
    /// The firdaria periods that overlap the span.
    pub firdaria: Vec<Firdaria>,
}

fn at(birth: UtcInstant, years: f64) -> UtcInstant {
    birth.plus_days(years * YEAR_DAYS)
}

/// Longest span [`time_lords`] covers, in years.
pub const MAX_TIME_LORDS_YEARS: f64 = 1000.0;

/// Annual profections and firdaria from `birth` (with the natal Sun's and Ascendant's
/// longitudes), for the years that overlap `start`..`end` (at most
/// [`MAX_TIME_LORDS_YEARS`] after the birth or the start, whichever is later).
pub fn time_lords(
    birth: UtcInstant,
    sun_longitude: f64,
    ascendant_longitude: f64,
    start: UtcInstant,
    end: UtcInstant,
) -> Result<TimeLords, EngineError> {
    finite(sun_longitude, "sun_longitude")?;
    finite(ascendant_longitude, "ascendant_longitude")?;
    let (start, end) = if end < start {
        (end, start)
    } else {
        (start, end)
    };
    let start = start.max(birth);
    let end = end.max(birth);
    if end.seconds_since(&start) / 86_400.0 / YEAR_DAYS > MAX_TIME_LORDS_YEARS {
        return Err(EngineError::InvalidInput(format!(
            "time_lords covers at most {MAX_TIME_LORDS_YEARS} years"
        )));
    }
    let diurnal = is_diurnal(sun_longitude, ascendant_longitude);
    let age_at = |t: UtcInstant| t.seconds_since(&birth) / 86_400.0 / YEAR_DAYS;

    let asc_sign =
        (pyfloat::floordiv(pyfloat::rem(ascendant_longitude, 360.0), 30.0) as usize).min(11);
    let first = age_at(start).floor() as u32;
    let last = age_at(end).floor() as u32;
    let profections = (first..=last)
        .map(|age| {
            let sign = SIGNS[(asc_sign + age as usize) % 12];
            Profection {
                age,
                start: at(birth, age as f64).isoformat(),
                end: at(birth, age as f64 + 1.0).isoformat(),
                sign,
                lord: traditional_ruler(sign).unwrap_or("Mars"),
                house: (age % 12) as u8 + 1,
            }
        })
        .collect();

    let sequence = if diurnal {
        DIURNAL_FIRDARIA
    } else {
        NOCTURNAL_FIRDARIA
    };
    let (from, to) = (age_at(start), age_at(end));
    let mut firdaria = Vec::new();
    let mut cycle_start = (from / FIRDARIA_CYCLE).floor() * FIRDARIA_CYCLE;
    'cycles: loop {
        let mut offset = cycle_start;
        for (lord, years) in sequence {
            let (period_start, period_end) = (offset, offset + years);
            offset = period_end;
            if period_start > to {
                break 'cycles;
            }
            if period_end <= from {
                continue;
            }
            let sub_periods = if SEVEN.contains(&lord) {
                let first = SEVEN.iter().position(|p| *p == lord).unwrap_or(0);
                (0..7)
                    .map(|i| {
                        let length = years / 7.0;
                        Period {
                            lord: SEVEN[(first + i) % 7],
                            start: at(birth, period_start + length * i as f64).isoformat(),
                            end: at(birth, period_start + length * (i + 1) as f64).isoformat(),
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };
            firdaria.push(Firdaria {
                lord,
                start: at(birth, period_start).isoformat(),
                end: at(birth, period_end).isoformat(),
                sub_periods,
            });
        }
        cycle_start += FIRDARIA_CYCLE;
    }

    Ok(TimeLords {
        diurnal,
        profections,
        firdaria,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(text: &str) -> UtcInstant {
        UtcInstant::parse(text).unwrap()
    }

    #[test]
    fn firdaria_sequences_last_seventy_five_years() {
        for sequence in [DIURNAL_FIRDARIA, NOCTURNAL_FIRDARIA] {
            assert_eq!(sequence.iter().map(|(_, y)| y).sum::<f64>(), FIRDARIA_CYCLE);
        }
    }

    #[test]
    fn profections_move_one_sign_a_year() {
        // Ascendant at 10° Leo: age 0 Leo (Sun), age 1 Virgo (Mercury), age 12 Leo again.
        let birth = utc("1990-01-01T12:00:00Z");
        let lords = time_lords(birth, 280.0, 130.0, birth, utc("2003-01-01T00:00:00Z")).unwrap();
        let p = &lords.profections;
        assert_eq!(p[0].sign, "Leo");
        assert_eq!(p[0].lord, "Sun");
        assert_eq!(p[0].house, 1);
        assert_eq!(p[1].sign, "Virgo");
        assert_eq!(p[1].lord, "Mercury");
        assert_eq!(p[12].sign, "Leo");
        assert_eq!(p[12].house, 1);
        assert_eq!(p[3].house, 4);
    }

    #[test]
    fn a_day_birth_starts_with_the_sun() {
        // Sun at 0° Aries and Ascendant at 0° Cancer: the Sun culminates, a day birth.
        let birth = utc("2000-03-20T12:00:00Z");
        let lords = time_lords(birth, 0.0, 90.0, birth, birth.plus_days(1.0)).unwrap();
        assert!(lords.diurnal);
        let first = &lords.firdaria[0];
        assert_eq!(first.lord, "Sun");
        let subs: Vec<&str> = first.sub_periods.iter().map(|s| s.lord).collect();
        assert_eq!(
            subs,
            ["Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars"]
        );
        assert_eq!(first.end, at(birth, 10.0).isoformat());
    }

    #[test]
    fn a_night_birth_starts_with_the_moon_and_the_nodes_have_no_sub_periods() {
        let birth = utc("2000-03-20T00:00:00Z");
        // Ascendant at 0° Capricorn puts the Sun at 0° Aries below the horizon.
        let lords = time_lords(birth, 0.0, 270.0, birth, at(birth, 74.0)).unwrap();
        assert!(!lords.diurnal);
        let names: Vec<&str> = lords.firdaria.iter().map(|f| f.lord).collect();
        assert_eq!(
            names,
            [
                "Moon",
                "Saturn",
                "Jupiter",
                "Mars",
                "Sun",
                "Venus",
                "Mercury",
                "North Node",
                "South Node"
            ]
        );
        assert!(lords.firdaria[7].sub_periods.is_empty());
        assert_eq!(lords.firdaria[1].sub_periods[0].lord, "Saturn");
        assert_eq!(lords.firdaria[1].sub_periods[1].lord, "Jupiter");
    }

    #[test]
    fn a_span_late_in_life_wraps_into_a_new_round() {
        let birth = utc("1900-01-01T12:00:00Z");
        let lords = time_lords(birth, 280.0, 130.0, at(birth, 76.0), at(birth, 77.0)).unwrap();
        // Night birth (the Sun below a Leo Ascendant): the round starts again with the Moon.
        assert_eq!(lords.firdaria.len(), 1);
        assert_eq!(lords.firdaria[0].lord, "Moon");
        assert_eq!(lords.profections[0].age, 76);
    }
}
