//! Horary charts (`charts/horary.py`): planetary day and hour, significators, Moon's
//! applying and separating aspects, void of course, and considerations before judgment.

use serde::Serialize;

use crate::almanac::rise_set;
use crate::chart::{chart_without_hours, Chart, ChartRequest, Placement};
use crate::dignities::{element, solar_phase, triplicity_ruler};
pub(crate) use crate::dignities::{traditional_ruler, SIGNS};
use crate::ephemeris::KernelSet;
use crate::error::EngineError;
use crate::instant::UtcInstant;
use crate::judgment::{cusp_signs, judge, Judgment};
use crate::pyfloat;

const CHALDEAN_ORDER: [&str; 7] = [
    "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon",
];
/// Ruler of each weekday, Monday first (Python `weekday()`).
const DAY_RULERS: [&str; 7] = [
    "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Sun",
];
pub(crate) const PTOLEMAIC: [(f64, &str); 5] = [
    (0.0, "Conjunction"),
    (60.0, "Sextile"),
    (90.0, "Square"),
    (120.0, "Trine"),
    (180.0, "Opposition"),
];

fn modern_ruler(sign: &str) -> Option<&'static str> {
    Some(match sign {
        "Scorpio" => "Pluto",
        "Aquarius" => "Uranus",
        "Pisces" => "Neptune",
        _ => return None,
    })
}

/// Planetary day and hour of a horary chart.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanetaryHours {
    /// Whether the instant falls between sunrise and sunset.
    pub is_day: bool,
    /// Planet ruling the day (from the weekday of sunrise).
    pub day_ruler: &'static str,
    /// Planet ruling the hour, in Chaldean order from the day ruler.
    pub hour_ruler: &'static str,
    /// Planetary hour (1-12) within the day or the night.
    pub hour_number: i64,
    /// "Day" or "Night".
    pub hour_type: &'static str,
    /// Sunrise, ISO 8601 UTC (06:00 when it cannot be computed).
    pub sunrise: String,
    /// Sunset, ISO 8601 UTC (18:00 when it cannot be computed).
    pub sunset: String,
}

/// The sunrise at or before an instant (with the reference's slack), the sunset after it
/// and the next sunrise: the day planetary hours are counted in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SolarDay {
    pub sunrise: UtcInstant,
    pub sunset: UtcInstant,
    pub next_sunrise: UtcInstant,
    /// Whether the times were computed, rather than the 06:00/18:00 UTC stand-ins.
    pub computed: bool,
}

/// How far past an instant `rise_set` still takes a sunrise as the day's, in days.
const SUNRISE_SLACK_DAYS: f64 = 0.05;

/// The Julian date sunrise and sunset searches take for an instant.
pub(crate) fn jd_utc(instant: UtcInstant) -> f64 {
    2_451_545.0 + instant.seconds_since(&UtcInstant::J2000) / 86_400.0
}

impl SolarDay {
    /// From computed (sunrise, sunset, next sunrise) UT1 Julian dates.
    pub fn from_julian_days((rise, set, next): (f64, f64, f64)) -> Self {
        let at = |jd: f64| UtcInstant::J2000.plus_days(jd - 2_451_545.0);
        SolarDay {
            sunrise: at(rise),
            sunset: at(set),
            next_sunrise: at(next),
            computed: true,
        }
    }

    /// Whether [`solar_day`] would return this same day for `instant`, so a search can
    /// reuse it instead of finding the sunrise again (always false for stand-in times).
    pub fn covers(&self, instant: UtcInstant) -> bool {
        let probe = instant.plus_days(SUNRISE_SLACK_DAYS);
        self.computed && self.sunrise <= probe && probe < self.next_sunrise
    }
}

/// The solar day of `instant` for a place. Where the times cannot be computed, 06:00
/// and 18:00 UTC stand in.
pub(crate) fn solar_day(
    kernels: &KernelSet,
    instant: UtcInstant,
    latitude: f64,
    longitude: f64,
) -> SolarDay {
    match rise_set(kernels, jd_utc(instant), latitude, longitude) {
        Ok(times) => SolarDay::from_julian_days(times),
        Err(_) => {
            let rise = instant.with_time(6, 0, 0, 0);
            SolarDay {
                sunrise: rise,
                sunset: instant.with_time(18, 0, 0, 0),
                next_sunrise: rise.add_micros(86_400_000_000),
                computed: false,
            }
        }
    }
}

/// Planetary day and hour at `instant` for a place, from the actual sunrise and sunset.
/// Where they cannot be computed, 06:00 and 18:00 UTC stand in.
pub fn planetary_hours(
    kernels: &KernelSet,
    instant: UtcInstant,
    latitude: f64,
    longitude: f64,
) -> PlanetaryHours {
    hours_in(&solar_day(kernels, instant, latitude, longitude), instant)
}

/// Planetary day and hour at `instant`, within its solar day.
pub(crate) fn hours_in(day: &SolarDay, instant: UtcInstant) -> PlanetaryHours {
    let SolarDay {
        sunrise,
        sunset,
        next_sunrise,
        ..
    } = *day;

    let is_day = sunrise <= instant && instant <= sunset;
    let day_ruler = DAY_RULERS[sunrise.weekday() as usize];
    let start = CHALDEAN_ORDER.iter().position(|p| *p == day_ruler).unwrap() as i64;

    let (hour_number, offset, hour_type) = if is_day {
        let hour_length = sunset.seconds_since(&sunrise) / 12.0;
        let elapsed = instant.seconds_since(&sunrise);
        let n = (elapsed / hour_length.max(1.0)) as i64 + 1;
        (n.clamp(1, 12), 0, "Day")
    } else {
        let hour_length = next_sunrise.seconds_since(&sunset) / 12.0;
        let elapsed = instant.seconds_since(&sunset);
        let n = if elapsed < 0.0 {
            12
        } else {
            (elapsed / hour_length.max(1.0)) as i64 + 1
        };
        (n.clamp(1, 12), 12, "Night")
    };
    let hour_ruler = CHALDEAN_ORDER[(start + offset + hour_number - 1).rem_euclid(7) as usize];

    PlanetaryHours {
        is_day,
        day_ruler,
        hour_ruler,
        hour_number,
        hour_type,
        sunrise: sunrise.isoformat(),
        sunset: sunset.isoformat(),
    }
}

/// A Ptolemaic aspect the Moon perfects before leaving its sign.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ApplyingAspect {
    /// Planet aspected (Sun to Saturn).
    pub planet: &'static str,
    /// Aspect name, e.g. "Trine".
    pub aspect: &'static str,
    /// Degrees the Moon has left to travel to exactness.
    pub degrees_to_exact: f64,
    /// The Moon's current sign.
    pub target_sign: &'static str,
}

/// A Ptolemaic aspect the Moon has perfected since entering its sign.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SeparatingAspect {
    /// Planet aspected (Sun to Saturn).
    pub planet: &'static str,
    /// Aspect name, e.g. "Trine".
    pub aspect: &'static str,
    /// Degrees the Moon has travelled since exactness.
    pub degrees_ago: f64,
}

/// The Moon's condition in a horary chart.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MoonStatus {
    /// Whether the Moon makes no applying aspect before leaving its sign.
    pub void_of_course: bool,
    /// Degrees left in the current sign.
    pub degrees_to_next_sign: f64,
    /// Hours until the Moon enters the next sign, at its current speed.
    pub hours_to_next_sign: f64,
    /// Sign the Moon enters next.
    pub next_sign: &'static str,
    /// "swift" (> 13.5°/day), "slow" (< 12.5°/day) or "average".
    pub speed_status: &'static str,
    /// Applying aspects, closest first.
    pub applying_aspects: Vec<ApplyingAspect>,
    /// The closest applying aspect.
    pub next_applying_aspect: Option<ApplyingAspect>,
    /// The most recent separating aspect.
    pub last_aspect: Option<SeparatingAspect>,
    /// Separating aspects, most recent first.
    pub separating_aspects: Vec<SeparatingAspect>,
}

/// The Moon's Ptolemaic aspects to the traditional planets before it leaves its sign,
/// and void-of-course status (`analyze_moon_horary`).
pub fn moon_status(moon: &Placement, planets: &[Placement]) -> MoonStatus {
    let moon_lon = moon.ecliptic_longitude;
    let moon_speed = moon.speed;
    let sign_start = (moon_lon / 30.0).floor() * 30.0;
    let to_sign_end = ((moon_lon / 30.0).floor() + 1.0) * 30.0 - moon_lon;
    let from_sign_start = moon_lon - sign_start;
    let speed_abs = if moon_speed.abs() > 1.0 {
        moon_speed.abs()
    } else {
        13.18
    };
    let current = ((moon_lon / 30.0).floor() as i64).rem_euclid(12) as usize;

    let mut applying: Vec<ApplyingAspect> = Vec::new();
    let mut separating: Vec<SeparatingAspect> = Vec::new();
    for p in planets {
        if !matches!(
            p.name,
            "Sun" | "Mercury" | "Venus" | "Mars" | "Jupiter" | "Saturn"
        ) {
            continue;
        }
        for (angle, aspect) in PTOLEMAIC {
            let mut targets = vec![pyfloat::rem(p.ecliptic_longitude + angle, 360.0)];
            if angle != 0.0 && angle != 180.0 {
                targets.push(pyfloat::rem(p.ecliptic_longitude - angle, 360.0));
            }
            for target in targets {
                let ahead = pyfloat::rem(target - moon_lon, 360.0);
                if 0.0 < ahead
                    && ahead <= to_sign_end
                    && moon_speed - p.speed > 0.0
                    && !applying
                        .iter()
                        .any(|a| a.planet == p.name && a.aspect == aspect)
                {
                    applying.push(ApplyingAspect {
                        planet: p.name,
                        aspect,
                        degrees_to_exact: pyfloat::round(ahead, 2),
                        target_sign: moon.sign,
                    });
                }
                let behind = pyfloat::rem(moon_lon - target, 360.0);
                if 0.0 < behind
                    && behind <= from_sign_start
                    && !separating
                        .iter()
                        .any(|s| s.planet == p.name && s.aspect == aspect)
                {
                    separating.push(SeparatingAspect {
                        planet: p.name,
                        aspect,
                        degrees_ago: pyfloat::round(behind, 2),
                    });
                }
            }
        }
    }
    applying.sort_by(|a, b| a.degrees_to_exact.total_cmp(&b.degrees_to_exact));
    separating.sort_by(|a, b| a.degrees_ago.total_cmp(&b.degrees_ago));

    MoonStatus {
        void_of_course: applying.is_empty(),
        degrees_to_next_sign: pyfloat::round(to_sign_end, 2),
        hours_to_next_sign: pyfloat::round(to_sign_end / (speed_abs / 24.0), 1),
        next_sign: SIGNS[(current + 1) % 12],
        speed_status: if moon_speed > 13.5 {
            "swift"
        } else if moon_speed < 12.5 {
            "slow"
        } else {
            "average"
        },
        next_applying_aspect: applying.first().cloned(),
        last_aspect: separating.first().cloned(),
        applying_aspects: applying,
        separating_aspects: separating,
    }
}

/// A consideration before judgement (stricture against judging the chart).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stricture {
    /// Stable code, e.g. "EARLY_ASC", "MOON_VOC".
    pub code: &'static str,
    /// "warning" or "info".
    pub severity: &'static str,
    /// Explanation for the reader, in English.
    pub message: String,
}

/// The horary-specific part of a horary chart.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HoraryData {
    /// Planetary day and hour.
    pub planetary_hours: PlanetaryHours,
    /// Sign on the Ascendant.
    pub ascendant_sign: &'static str,
    /// Ascendant as text, e.g. "12° 34' Leo".
    pub ascendant_degree: String,
    /// Traditional ruler of the Ascendant sign.
    pub traditional_asc_ruler: &'static str,
    /// Modern ruler of the Ascendant sign.
    pub modern_asc_ruler: &'static str,
    /// Whether the hour ruler matches the Ascendant's ruler or triplicity ruler.
    pub is_radical: bool,
    /// Considerations before judgement that apply.
    pub strictures: Vec<Stricture>,
    /// The Moon's condition.
    pub moon_status: MoonStatus,
    /// Significators and perfection (an addition to the reference implementation).
    pub judgment: Judgment,
}

/// A horary chart: a complete chart plus `horary_data` (serialized flattened).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HoraryChart {
    /// The chart, serialized inline.
    #[serde(flatten)]
    pub chart: Chart,
    /// Horary-specific data.
    pub horary_data: HoraryData,
}

/// A chart for the moment of the question, with its horary analysis
/// (`calculate_horary_chart_data`). `quesited_house` (1-12) is the house of the matter
/// asked about; its lord is judged against the lord of the Ascendant.
pub fn calculate_horary_chart(
    kernels: &KernelSet,
    req: &ChartRequest,
    quesited_house: Option<u8>,
) -> Result<HoraryChart, EngineError> {
    if let Some(house) = quesited_house.filter(|h| !(1..=12).contains(h)) {
        return Err(EngineError::InvalidInput(format!(
            "quesited_house must be 1-12, got {house}"
        )));
    }
    let mut chart = chart_without_hours(kernels, req)?;
    let hours = planetary_hours(kernels, req.instant, req.latitude, req.longitude);
    chart.planetary_hours = Some(hours.clone());

    let by_name = |name: &str| chart.planets.iter().find(|p| p.name == name);
    let (asc_sign, asc_deg, asc_min) = match (by_name("Ascendant"), chart.houses.first()) {
        (Some(asc), _) => (asc.sign, asc.degree, asc.minute),
        (None, Some(h)) => (h.sign, h.degree, h.minute),
        _ => ("Aries", 0, 0),
    };
    let traditional = traditional_ruler(asc_sign).unwrap_or("Mars");
    let modern = modern_ruler(asc_sign).unwrap_or(traditional);

    let mut strictures = Vec::new();
    if asc_deg < 3 {
        strictures.push(Stricture {
            code: "EARLY_ASC",
            severity: "warning",
            message: format!(
                "Ascendant is very early ({asc_deg}° {asc_sign}): The question may be premature, or circumstances are still developing."
            ),
        });
    } else if asc_deg >= 27 {
        strictures.push(Stricture {
            code: "LATE_ASC",
            severity: "warning",
            message: format!(
                "Ascendant is very late ({asc_deg}° {asc_sign}): The situation has already been decided or is out of the querent's control."
            ),
        });
    }
    match by_name("Saturn").map(|s| s.house) {
        Some(1) => strictures.push(Stricture {
            code: "SATURN_IN_1ST",
            severity: "warning",
            message: "Saturn in the 1st House: Querent may be obstructed, anxious, or facing delays."
                .into(),
        }),
        Some(7) => strictures.push(Stricture {
            code: "SATURN_IN_7TH",
            severity: "info",
            message: "Saturn in the 7th House: Astrologer's judgment may be challenged or the matter may be difficult to judge clearly."
                .into(),
        }),
        _ => {}
    }

    let moon_place =
        by_name("Moon").ok_or_else(|| EngineError::InvalidInput("chart has no Moon".into()))?;
    let moon = moon_status(moon_place, &chart.planets);
    if moon.void_of_course {
        strictures.push(Stricture {
            code: "MOON_VOC",
            severity: "info",
            message: "Moon is Void of Course: Nothing will come of the matter in question, or no immediate action will yield changes."
                .into(),
        });
    }

    // Lilly's further considerations (additions to the reference implementation).
    let moon_lon = pyfloat::rem(moon_place.ecliptic_longitude, 360.0);
    if (195.0..225.0).contains(&moon_lon) {
        strictures.push(Stricture {
            code: "MOON_VIA_COMBUSTA",
            severity: "warning",
            message: "Moon in the Via Combusta (15° Libra to 15° Scorpio): the matter is troubled and its outcome uncertain."
                .into(),
        });
    }
    if moon_place.degree >= 27 {
        strictures.push(Stricture {
            code: "MOON_LATE_DEGREES",
            severity: "info",
            message: format!(
                "Moon in the last degrees of {} ({}°): she has little left to do in her sign.",
                moon_place.sign, moon_place.degree
            ),
        });
    }
    if let (Some(ruler), Some(sun)) = (by_name(traditional), by_name("Sun")) {
        if solar_phase(ruler.name, ruler.ecliptic_longitude, sun.ecliptic_longitude)
            == Some("combust")
        {
            strictures.push(Stricture {
                code: "ASC_RULER_COMBUST",
                severity: "warning",
                message: format!(
                    "The lord of the Ascendant ({traditional}) is combust: the querent is fearful, overpowered or without hope."
                ),
            });
        }
    }

    let cusps: Vec<f64> = chart.houses.iter().map(|h| h.ecliptic_longitude).collect();
    let judgment = judge(
        kernels,
        req.instant.julian_day(),
        &chart.planets,
        &cusp_signs(&cusps),
        asc_sign,
        quesited_house,
        &chart.receptions,
    );

    let triplicity = triplicity_ruler(element(asc_sign).unwrap_or("Fire"), hours.is_day);
    let is_radical = hours.hour_ruler == traditional || Some(hours.hour_ruler) == triplicity;

    Ok(HoraryChart {
        horary_data: HoraryData {
            planetary_hours: hours,
            ascendant_sign: asc_sign,
            ascendant_degree: format!("{asc_deg}° {asc_min}' {asc_sign}"),
            traditional_asc_ruler: traditional,
            modern_asc_ruler: modern,
            is_radical,
            strictures,
            moon_status: moon,
            judgment,
        },
        chart,
    })
}
