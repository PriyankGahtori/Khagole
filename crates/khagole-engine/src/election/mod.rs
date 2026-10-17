//! Electional astrology: how well a moment suits beginning something, and a search over a
//! span of time for the best moments at a place.
//!
//! A moment is assessed with the traditional electional rules (Bonatti, Lilly and the
//! Arabic authors after Sahl): the Moon's condition and the next aspect she perfects, the
//! Ascendant and its ruler, the qualities of the Moon's and the Ascendant's degrees (Lilly), benefics and malefics on the angles, the retrogradation of the
//! planets the matter needs, the ruler of the house of the matter, the planetary hour and,
//! when a natal chart is given, the election's contacts with it. Each rule that applies is
//! an [`ElectionFactor`] with a stable code and a signed weight; the score is 50 plus the
//! weights, between 0 and 100. Explanations are left to the caller, keyed by code.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::almanac::SunEvents;
use crate::chart::{chart_without_hours, placements, sky, Chart, ChartRequest, Placement};
use crate::degree_qualities::degree_qualities;
use crate::dignities::{is_debilitated, is_dignified, sign_index, solar_phase};
use crate::ephemeris::KernelSet;
use crate::error::EngineError;
use crate::horary::{
    hours_in, jd_utc, moon_status, planetary_hours, solar_day, traditional_ruler, MoonStatus,
    PlanetaryHours, SolarDay, SIGNS,
};
use crate::houses::{houses_at_sidereal_time, HouseSystem};
use crate::instant::UtcInstant;
use crate::planets::{planets_and_sidereal_time, require_kernel, BodyPosition};
use crate::pyfloat;
use crate::zodiac::{ayanamsa, ayanamsa_info};

mod assess;
mod criteria;
mod search;
#[cfg(test)]
mod tests;

use assess::*;
pub use criteria::*;
pub use search::search_elections;

/// Longest span a search covers, in days.
pub const MAX_SEARCH_DAYS: f64 = 92.0;
/// Scores from here up are favourable.
const FAVOURABLE: f64 = 65.0;
/// Scores from here up (and below [`FAVOURABLE`]) are mixed; below, unfavourable.
const MIXED: f64 = 45.0;
/// Orb (degrees) of the election's contacts with natal points.
const NATAL_ORB: f64 = 3.0;
/// Below this latitude, where the Sun rises and sets every day, a search that finds
/// sunrises day by day reuses each day for all its moments.
const SOLAR_DAY_CACHE_MAX_LAT: f64 = 60.0;

const BENEFICS: [&str; 2] = ["Venus", "Jupiter"];
const MALEFICS: [&str; 2] = ["Mars", "Saturn"];

/// One electional rule that applies to a moment.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElectionFactor {
    /// Stable code, e.g. "MOON_VOC", "BENEFIC_ANGULAR".
    pub code: &'static str,
    /// Points added to (or, when negative, taken from) the score.
    pub weight: f64,
    /// The election planet the rule is about, if any.
    pub planet: Option<&'static str>,
    /// The other party: the planet aspected, or the natal point contacted.
    pub target: Option<String>,
    /// Aspect name, e.g. "Trine", for the rules about aspects.
    pub aspect: Option<&'static str>,
    /// House (1-12) of `planet`, for the rules about houses.
    pub house: Option<u8>,
    /// Sign of `planet`, for the rules about dignity.
    pub sign: Option<&'static str>,
}

impl ElectionFactor {
    fn new(code: &'static str, weight: f64) -> Self {
        ElectionFactor {
            code,
            weight,
            planet: None,
            target: None,
            aspect: None,
            house: None,
            sign: None,
        }
    }
    fn planet(mut self, planet: &'static str) -> Self {
        self.planet = Some(planet);
        self
    }
    fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
    fn aspect(mut self, aspect: &'static str) -> Self {
        self.aspect = Some(aspect);
        self
    }
    fn house(mut self, house: u8) -> Self {
        self.house = Some(house);
        self
    }
    fn sign(mut self, sign: &'static str) -> Self {
        self.sign = Some(sign);
        self
    }
}

/// The electional assessment of a moment.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElectionData {
    /// 0-100: 50 plus the factors' weights.
    pub score: f64,
    /// "favourable" (65 and up), "mixed" (45 and up) or "unfavourable".
    pub verdict: &'static str,
    /// What the election is for.
    pub purpose: Purpose,
    /// The rules that apply, Moon first, then the Ascendant, angles, retrogrades, the
    /// matter, the hour and the natal chart.
    pub factors: Vec<ElectionFactor>,
    /// The criteria filters this moment fails ("void_moon", "mercury_retrograde",
    /// "venus_retrograde", "night", "outside_hours"): a search would skip it.
    pub excluded_by: Vec<&'static str>,
    /// Planetary day and hour.
    pub planetary_hours: PlanetaryHours,
    /// The Moon's aspects and void-of-course status.
    pub moon_status: MoonStatus,
}

/// A chart for a candidate moment with its electional assessment (serialized flattened).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElectionChart {
    /// The chart, serialized inline.
    #[serde(flatten)]
    pub chart: Chart,
    /// The electional assessment.
    pub election_data: ElectionData,
}

/// A run of consecutive assessed moments scoring at least the minimum.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElectionWindow {
    /// First moment of the run, ISO 8601 UTC.
    pub start: String,
    /// Last moment of the run, ISO 8601 UTC.
    pub end: String,
    /// The best moment of the run (the earliest, on a tie), ISO 8601 UTC.
    pub best: String,
    /// Score of the best moment.
    pub score: f64,
    /// Verdict of the best moment.
    pub verdict: &'static str,
    /// The factors of the best moment.
    pub factors: Vec<ElectionFactor>,
}

/// How many moments each criteria filter left out.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Exclusions {
    /// Moon void of course.
    pub void_moon: u32,
    /// Mercury retrograde.
    pub mercury_retrograde: u32,
    /// Venus retrograde.
    pub venus_retrograde: u32,
    /// Between sunset and sunrise.
    pub night: u32,
    /// Outside the local hours.
    pub outside_hours: u32,
}

impl Exclusions {
    fn count(&mut self, reason: &str) {
        match reason {
            "void_moon" => self.void_moon += 1,
            "mercury_retrograde" => self.mercury_retrograde += 1,
            "venus_retrograde" => self.venus_retrograde += 1,
            "night" => self.night += 1,
            _ => self.outside_hours += 1,
        }
    }
}

/// The result of a search.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElectionSearch {
    /// Start of the span searched, ISO 8601 UTC.
    pub start: String,
    /// End of the span searched, ISO 8601 UTC.
    pub end: String,
    /// The best windows, best first.
    pub windows: Vec<ElectionWindow>,
    /// Moments assessed.
    pub evaluated: u32,
    /// Moments left out by each filter (a moment counts once, for the first it fails).
    pub excluded: Exclusions,
    /// The criteria, defaults resolved.
    pub criteria: CriteriaSummary,
}

/// A chart for a candidate moment with its electional assessment.
pub fn calculate_election_chart(
    kernels: &KernelSet,
    req: &ChartRequest,
    criteria: &ElectionCriteria,
) -> Result<ElectionChart, EngineError> {
    let resolved = criteria.resolve()?;
    let mut chart = chart_without_hours(kernels, req)?;
    let hours = planetary_hours(kernels, req.instant, req.latitude, req.longitude);
    chart.planetary_hours = Some(hours.clone());
    let cusps: Vec<f64> = chart.houses.iter().map(|h| h.ecliptic_longitude).collect();
    let election_data = assess(
        &Moment {
            instant: req.instant,
            planets: &chart.planets,
            cusps: &cusps,
            hours: &hours,
        },
        &resolved,
    )?;
    Ok(ElectionChart {
        chart,
        election_data,
    })
}
