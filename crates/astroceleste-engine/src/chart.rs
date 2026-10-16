//! A complete chart: positions, houses, aspects, fixed stars, lots, temperament and
//! lunar status, in the JSON shape of the reference's `calculate_chart_data`.

use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::aspects::{natal_aspects, Aspect, OrbSettings, Point};
use crate::chinese::calendar::{chinese_calendar, ChineseCalendar};
use crate::dignities::{
    antiscia, condition, is_diurnal, receptions, AntisciaContact, Body, Condition, DignityScheme,
    Reception,
};
use crate::ephemeris::KernelSet;
use crate::error::EngineError;
use crate::fixed_stars::{fixed_stars, FixedStarPosition};
use crate::horary::{planetary_hours, PlanetaryHours};
use crate::houses::{houses_at_sidereal_time, HouseSystem, Houses};
use crate::instant::UtcInstant;
use crate::lots::{arabic_parts, Lot};
use crate::lunar::{lunar_status, LunarStatus};
use crate::planets::planets_and_sidereal_time_with_options;
use crate::symbolic::symbolic_degree_number;
use crate::temperament::{temperament, Temperament};
use crate::zodiac::{
    ayanamsa, ayanamsa_info, determine_house, longitude_to_zodiac, AyanamsaInfo, DEFAULT_AYANAMSA,
};

/// Every body a complete chart carries; missing ones are reported in
/// `unavailable_bodies` rather than silently dropped.
const EXPECTED_BODIES: [&str; 16] = [
    "Sun",
    "Moon",
    "Mercury",
    "Venus",
    "Mars",
    "Jupiter",
    "Saturn",
    "Uranus",
    "Neptune",
    "Pluto",
    "North Node",
    "South Node",
    "Chiron",
    "Lilith",
    "Ascendant",
    "Midheaven",
];

/// What to compute.
#[derive(Debug, Clone)]
pub struct ChartRequest<'a> {
    /// Moment of the chart.
    pub instant: UtcInstant,
    /// Geographic latitude in degrees, north positive.
    pub latitude: f64,
    /// Geographic longitude in degrees, east positive.
    pub longitude: f64,
    /// House system code, echoed as given; its first letter selects the system.
    pub house_system: &'a str,
    /// "tropical" or "sidereal" (anything else is tropical).
    pub zodiac_type: &'a str,
    /// Ayanamsa code for sidereal charts (unknown codes use the default).
    pub ayanamsa: &'a str,
    /// Caller orb settings, merged over the defaults.
    pub orb_settings: Option<&'a Value>,
    /// Doctrine of triplicities and bounds: "lilly" (default) or "dorothean".
    pub dignity_scheme: &'a str,
    /// "apparent" (default) or "true" (geometric positions without light-time or aberration).
    pub position_mode: &'a str,
    /// "mean" (default) or "true" (osculating lunar node).
    pub node_type: &'a str,
    /// Also compute the [`ChineseCalendar`] of the moment (default `false`): a few dozen
    /// solar-term and new-moon searches.
    pub chinese_calendar: bool,
}

impl<'a> ChartRequest<'a> {
    /// A tropical Placidus request with default orbs.
    pub fn new(instant: UtcInstant, latitude: f64, longitude: f64) -> Self {
        ChartRequest {
            instant,
            latitude,
            longitude,
            house_system: "P",
            zodiac_type: "tropical",
            ayanamsa: DEFAULT_AYANAMSA,
            orb_settings: None,
            dignity_scheme: "lilly",
            position_mode: "apparent",
            node_type: "mean",
            chinese_calendar: false,
        }
    }

    /// The place must be on Earth: a finite latitude within ±90° and a finite longitude
    /// within ±360° (east or west, or 0-360 east).
    pub(crate) fn check_place(&self) -> Result<(), EngineError> {
        let (lat, lon) = (self.latitude, self.longitude);
        if !(lat.is_finite() && lat.abs() <= 90.0) {
            return Err(EngineError::InvalidInput(format!(
                "latitude must be within ±90°, got {lat}"
            )));
        }
        if !(lon.is_finite() && lon.abs() <= 360.0) {
            return Err(EngineError::InvalidInput(format!(
                "longitude must be within ±360°, got {lon}"
            )));
        }
        Ok(())
    }
}

/// A planet, lunar point or angle placed in the chart.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Placement {
    /// Body or angle name, e.g. "Sun", "North Node", "Ascendant".
    pub name: &'static str,
    /// Glyph, e.g. "☉" ("ASC" and "MC" for the angles).
    pub symbol: &'static str,
    /// Zodiac sign name, e.g. "Taurus".
    pub sign: &'static str,
    /// Zodiac sign glyph, e.g. "♉".
    pub sign_symbol: &'static str,
    /// Whole degrees within the sign (0-29).
    pub degree: i64,
    /// Arc minutes past `degree` (0-59).
    pub minute: i64,
    /// Ecliptic longitude in degrees [0, 360), tropical or sidereal as requested.
    pub ecliptic_longitude: f64,
    /// House (1-12) the point falls in.
    pub house: u8,
    /// Apparent speed in ecliptic longitude, degrees per day.
    pub speed: f64,
    /// Whether the apparent motion is retrograde.
    pub is_retrograde: bool,
    /// Symbolic degree (1-30) within the sign, as used by degree symbolism.
    pub symbolic_degree: i64,
    /// Essential dignity, sect, phase to the Sun and speed: the seven planets only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
}

/// The cusp of one house.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HouseCusp {
    /// House number (1-12).
    pub house_number: u8,
    /// Zodiac sign name, e.g. "Taurus".
    pub sign: &'static str,
    /// Zodiac sign glyph, e.g. "♉".
    pub sign_symbol: &'static str,
    /// Whole degrees within the sign (0-29).
    pub degree: i64,
    /// Arc minutes past `degree` (0-59).
    pub minute: i64,
    /// Ecliptic longitude in degrees [0, 360), tropical or sidereal as requested.
    pub ecliptic_longitude: f64,
    /// Symbolic degree (1-30) within the sign, as used by degree symbolism.
    pub symbolic_degree: i64,
}

/// A complete chart (`calculate_chart_data`); serializes to the API's JSON.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Chart {
    /// House system code, echoed from the request.
    pub house_system: String,
    /// "tropical" or "sidereal".
    pub zodiac_type: &'static str,
    /// Position mode: "apparent" or "true" (geometric). Skipped if None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_mode: Option<&'static str>,
    /// Node type: "mean" or "true" (osculating). Skipped if None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_type: Option<&'static str>,
    /// Ayanamsa code; `None` for tropical charts.
    pub ayanamsa: Option<&'static str>,
    /// Human-readable ayanamsa name; `None` for tropical charts.
    pub ayanamsa_name: Option<&'static str>,
    /// Ayanamsa at the chart instant, in degrees; `None` for tropical charts.
    pub ayanamsa_value: Option<f64>,
    /// `ayanamsa_value` as degrees, minutes and seconds; `None` for tropical charts.
    pub ayanamsa_formatted: Option<String>,
    /// Mean precession rate used for the fixed stars, arcseconds per year.
    pub precession_rate_arcsec_yr: f64,
    /// Orb settings in effect: the defaults merged with the request's.
    pub orb_settings: Value,
    /// Planets, lunar points, Chiron, Lilith and the angles.
    pub planets: Vec<Placement>,
    /// Expected bodies that could not be computed (e.g. Chiron outside its table).
    pub unavailable_bodies: Vec<&'static str>,
    /// The twelve house cusps.
    pub houses: Vec<HouseCusp>,
    /// Aspects between chart points, then fixed-star conjunctions.
    pub aspects: Vec<Aspect>,
    /// Fixed stars conjunct a chart point.
    pub fixed_stars: Vec<FixedStarPosition>,
    /// The Arabic parts (lots).
    pub arabic_parts: Vec<Lot>,
    /// Temperament assessment.
    pub temperament: Temperament,
    /// Lunar phase and status; `None` (serialized as `{}`) without a Moon.
    #[serde(serialize_with = "empty_object_if_none")]
    pub lunar_status: Option<LunarStatus>,
    /// "diurnal" (the Sun above the horizon) or "nocturnal"; `None` without Sun and Ascendant.
    pub sect: Option<&'static str>,
    /// Doctrine of triplicities and bounds used for `condition` and `receptions`.
    pub dignity_scheme: &'static str,
    /// Receptions among the seven planets.
    pub receptions: Vec<Reception>,
    /// Antiscia and contra-antiscia among the seven planets and the angles.
    pub antiscia: Vec<AntisciaContact>,
    /// Planetary day and hour; absent from transit skies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planetary_hours: Option<PlanetaryHours>,
    /// The Chinese calendar of the moment, when requested and the kernels cover it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chinese_calendar: Option<ChineseCalendar>,
}

fn empty_object_if_none<S: Serializer>(v: &Option<LunarStatus>, s: S) -> Result<S::Ok, S::Error> {
    match v {
        Some(status) => status.serialize(s),
        None => serde_json::Map::new().serialize(s),
    }
}

impl Chart {
    /// The chart's planets and angles as named ecliptic points.
    pub(crate) fn points(&self) -> Vec<Point<'_>> {
        self.planets
            .iter()
            .map(|p| Point {
                name: p.name,
                longitude: p.ecliptic_longitude,
            })
            .collect()
    }
}

fn symbol_of(name: &str) -> &'static str {
    match name {
        "Sun" => "☉",
        "Moon" => "☽",
        "Mercury" => "☿",
        "Venus" => "♀",
        "Mars" => "♂",
        "Jupiter" => "♃",
        "Saturn" => "♄",
        "Uranus" => "♅",
        "Neptune" => "♆",
        "Pluto" => "♇",
        "North Node" => "☊",
        "South Node" => "☋",
        "Chiron" => "⚷",
        "Lilith" => "⚸",
        "Ascendant" => "ASC",
        "Midheaven" => "MC",
        _ => "",
    }
}

/// The positions part of a chart: placements and house cusps, without aspects, fixed
/// stars, lots or temperament. Electional searches evaluate thousands of these.
pub(crate) struct Sky {
    pub zodiac_type: &'static str,
    pub is_sidereal: bool,
    pub ayanamsa_code: &'static str,
    pub info: AyanamsaInfo,
    pub jd: f64,
    /// Cusps 1-12, degrees.
    pub cusps: Vec<f64>,
    pub house_cusps: Vec<HouseCusp>,
    pub planets: Vec<Placement>,
}

/// Placements and houses of a request (the first half of [`calculate_chart`]).
pub(crate) fn sky(kernels: &KernelSet, req: &ChartRequest) -> Result<Sky, EngineError> {
    req.check_place()?;
    let zodiac_type = match req.zodiac_type.trim().to_lowercase().as_str() {
        "sidereal" => "sidereal",
        _ => "tropical",
    };
    let is_sidereal = zodiac_type == "sidereal";
    let ayanamsa_code = ayanamsa(req.ayanamsa).code;

    let jd = req.instant.julian_day();
    let info = ayanamsa_info(jd, ayanamsa_code);
    let shift = if is_sidereal { info.value } else { 0.0 };

    let is_geometric = req.position_mode.trim().eq_ignore_ascii_case("true")
        || req.position_mode.trim().eq_ignore_ascii_case("geometric");
    let is_true_node = req.node_type.trim().eq_ignore_ascii_case("true")
        || req.node_type.trim().eq_ignore_ascii_case("osculating");

    // The houses reuse the planets' sidereal time (the same instant): one nutation.
    let (engine, gast_hours) =
        planets_and_sidereal_time_with_options(kernels, jd, shift, is_geometric, is_true_node)?;
    let houses = houses_at_sidereal_time(
        jd,
        gast_hours,
        req.latitude,
        req.longitude,
        HouseSystem::from_code(req.house_system),
        shift,
    );
    let cusps = houses.cusps.to_vec();

    let house_cusps: Vec<HouseCusp> = houses
        .cusps
        .iter()
        .enumerate()
        .map(|(i, &lon)| {
            let z = longitude_to_zodiac(lon);
            HouseCusp {
                house_number: i as u8 + 1,
                sign: z.sign.name,
                sign_symbol: z.sign.symbol,
                degree: z.degree,
                minute: z.minute,
                ecliptic_longitude: lon,
                symbolic_degree: symbolic_degree_number(z.degree, z.minute),
            }
        })
        .collect();

    let raw: Vec<(&'static str, f64, f64, bool)> = engine
        .bodies
        .iter()
        .map(|b| (b.name, b.longitude, b.speed, b.is_retrograde))
        .collect();
    let planets = placements(raw, &houses);

    Ok(Sky {
        zodiac_type,
        is_sidereal,
        ayanamsa_code,
        info,
        jd,
        cusps,
        house_cusps,
        planets,
    })
}

/// Placements of bodies given as (name, longitude, speed, retrograde), followed by the
/// Ascendant and the Midheaven, with their houses.
pub(crate) fn placements(
    mut raw: Vec<(&'static str, f64, f64, bool)>,
    houses: &Houses,
) -> Vec<Placement> {
    raw.push(("Ascendant", houses.ascendant, 0.0, false));
    raw.push(("Midheaven", houses.midheaven, 0.0, false));
    raw.iter()
        .map(|&(name, lon, speed, is_retrograde)| {
            let z = longitude_to_zodiac(lon);
            Placement {
                name,
                symbol: symbol_of(name),
                sign: z.sign.name,
                sign_symbol: z.sign.symbol,
                degree: z.degree,
                minute: z.minute,
                ecliptic_longitude: lon,
                house: determine_house(lon, &houses.cusps),
                speed,
                is_retrograde,
                symbolic_degree: symbolic_degree_number(z.degree, z.minute),
                condition: None,
            }
        })
        .collect()
}

/// Set the condition of the seven planets, which needs the Sun and the Ascendant.
fn add_conditions(planets: &mut [Placement], scheme: DignityScheme) -> Option<&'static str> {
    let at = |name: &str| {
        planets
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.ecliptic_longitude)
    };
    let (sun, asc) = (at("Sun")?, at("Ascendant")?);
    for planet in planets.iter_mut() {
        planet.condition = condition(Body::from(&*planet), sun, asc, scheme);
    }
    Some(if is_diurnal(sun, asc) {
        "diurnal"
    } else {
        "nocturnal"
    })
}

/// Compute a chart (`calculate_chart_data`), with its planetary day and hour.
pub fn calculate_chart(kernels: &KernelSet, req: &ChartRequest) -> Result<Chart, EngineError> {
    let mut chart = chart_without_hours(kernels, req)?;
    chart.planetary_hours = Some(planetary_hours(
        kernels,
        req.instant,
        req.latitude,
        req.longitude,
    ));
    Ok(chart)
}

/// A chart without its planetary hours, which cost a sunrise search: for transit skies,
/// and for the callers that compute the hours themselves.
pub(crate) fn chart_without_hours(
    kernels: &KernelSet,
    req: &ChartRequest,
) -> Result<Chart, EngineError> {
    let orbs = OrbSettings::merge(req.orb_settings)?;
    let scheme = DignityScheme::from_code(req.dignity_scheme);
    let Sky {
        zodiac_type,
        is_sidereal,
        ayanamsa_code,
        info,
        jd,
        cusps,
        house_cusps,
        mut planets,
    } = sky(kernels, req)?;
    let sect = add_conditions(&mut planets, scheme);
    let points: Vec<Point> = planets
        .iter()
        .map(|p| Point {
            name: p.name,
            longitude: p.ecliptic_longitude,
        })
        .collect();

    let mut aspects = natal_aspects(&points, &orbs)?;
    let (stars, star_aspects) = fixed_stars(
        jd,
        &cusps,
        &points,
        orbs.fixed_star_orb()?,
        is_sidereal.then_some(info.value),
    );
    aspects.extend(star_aspects);

    let lots = arabic_parts(&points, &cusps);
    let temperament = temperament(&planets, &aspects);
    let lunar = lunar_status(&planets);
    let bodies: Vec<Body> = planets.iter().map(Body::from).collect();
    let aspect_between = |a: &str, b: &str| {
        aspects
            .iter()
            .find(|x| (x.body1 == a && x.body2 == b) || (x.body1 == b && x.body2 == a))
            .map(|x| x.aspect_type)
    };
    let chart_receptions = receptions(&bodies, aspect_between, scheme);
    let chart_antiscia = antiscia(&planets);

    let mut unavailable: Vec<&'static str> = EXPECTED_BODIES
        .iter()
        .copied()
        .filter(|name| !planets.iter().any(|p| p.name == *name))
        .collect();
    unavailable.sort_unstable();

    Ok(Chart {
        house_system: req.house_system.to_string(),
        zodiac_type,
        position_mode: if req.position_mode.trim().eq_ignore_ascii_case("true")
            || req.position_mode.trim().eq_ignore_ascii_case("geometric")
        {
            Some("true")
        } else {
            None
        },
        node_type: if req.node_type.trim().eq_ignore_ascii_case("true")
            || req.node_type.trim().eq_ignore_ascii_case("osculating")
        {
            Some("true")
        } else {
            None
        },
        ayanamsa: is_sidereal.then_some(ayanamsa_code),
        ayanamsa_name: is_sidereal.then_some(info.name),
        ayanamsa_value: is_sidereal.then_some(info.value),
        ayanamsa_formatted: is_sidereal.then(|| info.formatted.clone()),
        precession_rate_arcsec_yr: info.precession_rate_arcsec_yr,
        orb_settings: orbs.to_value(),
        planets,
        unavailable_bodies: unavailable,
        houses: house_cusps,
        aspects,
        fixed_stars: stars,
        arabic_parts: lots,
        temperament,
        lunar_status: lunar,
        sect,
        dignity_scheme: scheme.code(),
        receptions: chart_receptions,
        antiscia: chart_antiscia,
        planetary_hours: None,
        chinese_calendar: if req.chinese_calendar {
            chinese_calendar(kernels, req.instant)?
        } else {
            None
        },
    })
}
