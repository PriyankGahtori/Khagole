//! What an election is for and how it is filtered, and the criteria resolved with
//! their defaults.

use super::*;

/// What the election is for. Each purpose has a house of the matter, a natural
/// significator and the planetary hours that favour it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    /// No particular matter: the general rules only.
    #[default]
    General,
    /// Signing a contract or an agreement (7th house, Mercury).
    Contract,
    /// Marriage, engagement or partnership (7th house, Venus).
    Partnership,
    /// Setting out on a journey (9th house, Mercury).
    Travel,
    /// Beginning a treatment or a surgery (1st house, the Sun).
    Health,
    /// Buying, investing or borrowing (2nd house, Jupiter).
    Finance,
    /// Opening a business or launching a venture (10th house, Jupiter).
    Launch,
    /// Starting a job or asking for a promotion (10th house, the Sun).
    Career,
}

impl Purpose {
    /// House of the matter.
    pub(super) fn house(self) -> Option<usize> {
        match self {
            Purpose::General => None,
            Purpose::Contract | Purpose::Partnership => Some(7),
            Purpose::Travel => Some(9),
            Purpose::Health => Some(1),
            Purpose::Finance => Some(2),
            Purpose::Launch | Purpose::Career => Some(10),
        }
    }

    /// Natural significator of the matter.
    pub(super) fn significator(self) -> Option<&'static str> {
        match self {
            Purpose::General => None,
            Purpose::Contract | Purpose::Travel => Some("Mercury"),
            Purpose::Partnership => Some("Venus"),
            Purpose::Health | Purpose::Career => Some("Sun"),
            Purpose::Finance | Purpose::Launch => Some("Jupiter"),
        }
    }

    /// Hour rulers that favour the matter.
    pub(super) fn hour_rulers(self) -> &'static [&'static str] {
        match self {
            Purpose::General => &["Jupiter", "Venus"],
            Purpose::Contract => &["Mercury", "Jupiter"],
            Purpose::Partnership => &["Venus", "Moon"],
            Purpose::Travel => &["Mercury", "Moon"],
            Purpose::Health => &["Sun", "Jupiter"],
            Purpose::Finance => &["Jupiter", "Venus"],
            Purpose::Launch => &["Jupiter", "Sun"],
            Purpose::Career => &["Sun", "Jupiter"],
        }
    }

    /// Whether Mercury retrograde spoils the matter.
    pub(super) fn fears_mercury_retrograde(self) -> bool {
        matches!(self, Purpose::Contract | Purpose::Travel | Purpose::Launch)
    }

    /// Whether Venus retrograde spoils the matter.
    pub(super) fn fears_venus_retrograde(self) -> bool {
        matches!(self, Purpose::Partnership)
    }
}

/// Local clock hours, `from` inclusive to `to` exclusive; `from > to` spans midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HourRange {
    /// First hour allowed (0-23).
    pub from: u8,
    /// Hour at which the range ends (1-24).
    pub to: u8,
}

impl HourRange {
    pub(super) fn contains(&self, minute_of_day: i64) -> bool {
        let from = i64::from(self.from) * 60;
        let to = i64::from(self.to) * 60;
        if from <= to {
            (from..to).contains(&minute_of_day)
        } else {
            minute_of_day >= from || minute_of_day < to
        }
    }
}

/// The UTC offset of local time from an instant on (until the next one), so that
/// [`ElectionCriteria::local_hours`] follows daylight saving time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UtcOffset {
    /// ISO 8601 instant from which the offset applies.
    pub from: String,
    /// Minutes to add to UTC for local time (east positive, e.g. 120 for CEST).
    pub minutes: i32,
}

/// A natal point, as in a stored chart's `planets` list (other fields are ignored).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NatalPoint {
    /// Body or angle name, e.g. "Sun", "Ascendant".
    pub name: String,
    /// Ecliptic longitude in degrees.
    pub ecliptic_longitude: f64,
}

/// What to look for. Every field has a default, so `{}` is a valid criteria object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ElectionCriteria {
    /// What the election is for.
    pub purpose: Purpose,
    /// Leave out moments when the Moon is void of course (default true).
    pub avoid_void_moon: bool,
    /// Leave out moments when Mercury is retrograde (default: when the purpose fears it).
    pub avoid_mercury_retrograde: Option<bool>,
    /// Leave out moments when Venus is retrograde (default: when the purpose fears it).
    pub avoid_venus_retrograde: Option<bool>,
    /// Leave out moments between sunset and sunrise.
    pub daytime_only: bool,
    /// Only moments within these local clock hours.
    pub local_hours: Option<HourRange>,
    /// UTC offsets of local time for `local_hours` (none: local time is UTC).
    pub utc_offsets: Vec<UtcOffset>,
    /// Minutes between the moments a search assesses (5-60, default 10).
    pub step_minutes: u32,
    /// Lowest score a search keeps (default 60).
    pub min_score: f64,
    /// Most windows a search returns (1-50, default 20).
    pub max_results: usize,
    /// The natal chart to elect for, if any.
    pub natal: Option<Vec<NatalPoint>>,
}

impl Default for ElectionCriteria {
    fn default() -> Self {
        ElectionCriteria {
            purpose: Purpose::General,
            avoid_void_moon: true,
            avoid_mercury_retrograde: None,
            avoid_venus_retrograde: None,
            daytime_only: false,
            local_hours: None,
            utc_offsets: Vec::new(),
            step_minutes: 10,
            min_score: 60.0,
            max_results: 20,
            natal: None,
        }
    }
}

impl ElectionCriteria {
    /// Criteria from a JSON object (`null` gives the defaults).
    pub fn from_value(value: &Value) -> Result<Self, EngineError> {
        if value.is_null() {
            return Ok(ElectionCriteria::default());
        }
        serde_json::from_value(value.clone())
            .map_err(|e| EngineError::InvalidInput(format!("election criteria: {e}")))
    }

    /// The criteria with every default resolved and out-of-range values clamped.
    pub(super) fn resolve(&self) -> Result<Resolved, EngineError> {
        if let Some(range) = self.local_hours {
            if range.from > 23 || range.to == 0 || range.to > 24 || range.from == range.to {
                return Err(EngineError::InvalidInput(
                    "local_hours needs 0 <= from <= 23, 1 <= to <= 24 and from != to".into(),
                ));
            }
        }
        let mut offsets = self
            .utc_offsets
            .iter()
            .map(|o| {
                if o.minutes.abs() > 18 * 60 {
                    return Err(EngineError::InvalidInput(format!(
                        "UTC offset {} is out of range",
                        o.minutes
                    )));
                }
                UtcInstant::parse(&o.from)
                    .map(|at| (at, o.minutes))
                    .map_err(|e| EngineError::InvalidInput(format!("utc_offsets: {e}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        offsets.sort_by_key(|(at, _)| *at);
        if !self.min_score.is_finite() {
            return Err(EngineError::InvalidInput(
                "min_score must be a number".into(),
            ));
        }
        Ok(Resolved {
            summary: CriteriaSummary {
                purpose: self.purpose,
                avoid_void_moon: self.avoid_void_moon,
                avoid_mercury_retrograde: self
                    .avoid_mercury_retrograde
                    .unwrap_or(self.purpose.fears_mercury_retrograde()),
                avoid_venus_retrograde: self
                    .avoid_venus_retrograde
                    .unwrap_or(self.purpose.fears_venus_retrograde()),
                daytime_only: self.daytime_only,
                local_hours: self.local_hours,
                step_minutes: self.step_minutes.clamp(5, 60),
                min_score: self.min_score.clamp(0.0, 100.0),
                max_results: self.max_results.clamp(1, 50),
                natal: self.natal.is_some(),
            },
            offsets,
            natal: self.natal.clone().unwrap_or_default(),
        })
    }
}

/// The criteria a search ran with, defaults resolved (the natal chart only as a flag).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CriteriaSummary {
    /// What the election is for.
    pub purpose: Purpose,
    /// Whether void-of-course Moon moments were left out.
    pub avoid_void_moon: bool,
    /// Whether moments with Mercury retrograde were left out.
    pub avoid_mercury_retrograde: bool,
    /// Whether moments with Venus retrograde were left out.
    pub avoid_venus_retrograde: bool,
    /// Whether night-time moments were left out.
    pub daytime_only: bool,
    /// Local clock hours searched, if restricted.
    pub local_hours: Option<HourRange>,
    /// Minutes between assessed moments.
    pub step_minutes: u32,
    /// Lowest score kept.
    pub min_score: f64,
    /// Most windows returned.
    pub max_results: usize,
    /// Whether a natal chart was given.
    pub natal: bool,
}

pub(super) struct Resolved {
    pub(super) summary: CriteriaSummary,
    pub(super) offsets: Vec<(UtcInstant, i32)>,
    pub(super) natal: Vec<NatalPoint>,
}

impl Resolved {
    /// Minutes since local midnight at `instant`.
    pub(super) fn local_minute_of_day(&self, instant: UtcInstant) -> i64 {
        let offset = self
            .offsets
            .iter()
            .rev()
            .find(|(at, _)| *at <= instant)
            .or(self.offsets.first())
            .map_or(0, |(_, minutes)| *minutes);
        let (_, _, _, hh, mm, ..) = instant.add_micros(i64::from(offset) * 60_000_000).civil();
        i64::from(hh) * 60 + i64::from(mm)
    }
}
