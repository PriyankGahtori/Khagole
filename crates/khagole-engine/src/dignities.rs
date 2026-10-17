//! Essential dignities and the condition of the seven planets: domicile, exaltation,
//! triplicity, bounds and faces (with Lilly's points), sect, the phase to the Sun,
//! orientality and speed, plus the receptions and antiscia between planets.
//!
//! Not part of the reference implementation: these keys are additions, absent from the
//! golden fixtures. The tables are public domain (Ptolemy, Dorotheus, Lilly) and their
//! sources are in `docs/dignities.md`.

use serde::Serialize;

use serde_json::Value;

use crate::aspects::ASPECT_RULES;
use crate::chart::Placement;
use crate::error::EngineError;
use crate::pyfloat;

/// The seven planets, in Chaldean order.
pub(crate) const SEVEN: [&str; 7] = [
    "Saturn", "Jupiter", "Mars", "Sun", "Venus", "Mercury", "Moon",
];

pub(crate) const SIGNS: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

/// Within 17 arc minutes of the Sun a planet is cazimi, in the heart of the Sun.
pub(crate) const CAZIMI_ORB: f64 = 17.0 / 60.0;
/// A planet this close to the Sun (degrees) is combust, unless cazimi.
pub(crate) const COMBUST_ORB: f64 = 8.5;
/// Within this distance from the Sun (degrees) a planet not combust is under the beams.
const UNDER_BEAMS_ORB: f64 = 15.0;
/// Orb (degrees) of an antiscion or contra-antiscion contact.
const ANTISCIA_ORB: f64 = 1.0;
/// Below this fraction of its mean motion a planet is stationary.
const STATIONARY_FRACTION: f64 = 0.1;

/// The doctrine of triplicities and bounds a chart is judged by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DignityScheme {
    /// Lilly's *Christian Astrology*: two triplicity lords (Mars rules water by day and by
    /// night) and the Ptolemaic terms as Lilly printed them.
    #[default]
    Lilly,
    /// Dorotheus and the Hellenistic authors: three triplicity lords (by day, by night and
    /// participating) and the Egyptian bounds.
    Dorothean,
}

impl DignityScheme {
    /// The scheme named by `code` ("lilly", "dorothean"); anything else is Lilly's.
    pub fn from_code(code: &str) -> Self {
        match code.trim().to_lowercase().as_str() {
            "dorothean" | "dorotheus" | "egyptian" => DignityScheme::Dorothean,
            _ => DignityScheme::Lilly,
        }
    }

    /// The stable code of the scheme.
    pub fn code(self) -> &'static str {
        match self {
            DignityScheme::Lilly => "lilly",
            DignityScheme::Dorothean => "dorothean",
        }
    }
}

pub(crate) fn sign_index(sign: &str) -> Option<usize> {
    SIGNS.iter().position(|s| *s == sign)
}

/// The domicile lord of a sign.
pub(crate) fn traditional_ruler(sign: &str) -> Option<&'static str> {
    Some(match sign {
        "Aries" | "Scorpio" => "Mars",
        "Taurus" | "Libra" => "Venus",
        "Gemini" | "Virgo" => "Mercury",
        "Cancer" => "Moon",
        "Leo" => "Sun",
        "Sagittarius" | "Pisces" => "Jupiter",
        "Capricorn" | "Aquarius" => "Saturn",
        _ => return None,
    })
}

/// The planet exalted in a sign, with its degree of greatest exaltation.
fn exaltation(sign: &str) -> Option<(&'static str, i64)> {
    Some(match sign {
        "Aries" => ("Sun", 19),
        "Taurus" => ("Moon", 3),
        "Cancer" => ("Jupiter", 15),
        "Virgo" => ("Mercury", 15),
        "Libra" => ("Saturn", 21),
        "Capricorn" => ("Mars", 28),
        "Pisces" => ("Venus", 27),
        _ => return None,
    })
}

/// Whether `planet` is in its domicile or exaltation in `sign`.
pub(crate) fn is_dignified(planet: &str, sign: &str) -> bool {
    traditional_ruler(sign) == Some(planet) || exaltation(sign).map(|(p, _)| p) == Some(planet)
}

/// Whether `planet` is in its detriment or fall in `sign`.
pub(crate) fn is_debilitated(planet: &str, sign: &str) -> bool {
    sign_index(sign).is_some_and(|i| is_dignified(planet, SIGNS[(i + 6) % 12]))
}

pub(crate) fn element(sign: &str) -> Option<&'static str> {
    Some(match sign {
        "Aries" | "Leo" | "Sagittarius" => "Fire",
        "Taurus" | "Virgo" | "Capricorn" => "Earth",
        "Gemini" | "Libra" | "Aquarius" => "Air",
        "Cancer" | "Scorpio" | "Pisces" => "Water",
        _ => return None,
    })
}

/// The Dorothean triplicity lords of an element: by day, by night and participating.
fn dorothean_triplicity(element: &str) -> Option<[&'static str; 3]> {
    Some(match element {
        "Fire" => ["Sun", "Jupiter", "Saturn"],
        "Earth" => ["Venus", "Moon", "Mars"],
        "Air" => ["Saturn", "Mercury", "Jupiter"],
        "Water" => ["Venus", "Mars", "Moon"],
        _ => return None,
    })
}

/// The triplicity lord of an element by day or by night (Dorothean, as the horary
/// radicality test takes it).
pub(crate) fn triplicity_ruler(element: &str, is_day: bool) -> Option<&'static str> {
    dorothean_triplicity(element).map(|[day, night, _]| if is_day { day } else { night })
}

/// Lilly's triplicity lords of an element: by day and by night.
fn lilly_triplicity(element: &str) -> Option<[&'static str; 2]> {
    Some(match element {
        "Fire" => ["Sun", "Jupiter"],
        "Earth" => ["Venus", "Moon"],
        "Air" => ["Saturn", "Mercury"],
        "Water" => ["Mars", "Mars"],
        _ => return None,
    })
}

/// Bounds of a sign: (lord, end degree), in order from 0°.
type Bounds = [(&'static str, f64); 5];

/// The Egyptian bounds (Ptolemy, Tetrabiblos I.20, after the Egyptians).
const EGYPTIAN_BOUNDS: [Bounds; 12] = [
    [
        ("Jupiter", 6.0),
        ("Venus", 12.0),
        ("Mercury", 20.0),
        ("Mars", 25.0),
        ("Saturn", 30.0),
    ],
    [
        ("Venus", 8.0),
        ("Mercury", 14.0),
        ("Jupiter", 22.0),
        ("Saturn", 27.0),
        ("Mars", 30.0),
    ],
    [
        ("Mercury", 6.0),
        ("Jupiter", 12.0),
        ("Venus", 17.0),
        ("Mars", 24.0),
        ("Saturn", 30.0),
    ],
    [
        ("Mars", 7.0),
        ("Venus", 13.0),
        ("Mercury", 19.0),
        ("Jupiter", 26.0),
        ("Saturn", 30.0),
    ],
    [
        ("Jupiter", 6.0),
        ("Venus", 11.0),
        ("Saturn", 18.0),
        ("Mercury", 24.0),
        ("Mars", 30.0),
    ],
    [
        ("Mercury", 7.0),
        ("Venus", 17.0),
        ("Jupiter", 21.0),
        ("Mars", 28.0),
        ("Saturn", 30.0),
    ],
    [
        ("Saturn", 6.0),
        ("Mercury", 14.0),
        ("Jupiter", 21.0),
        ("Venus", 28.0),
        ("Mars", 30.0),
    ],
    [
        ("Mars", 7.0),
        ("Venus", 11.0),
        ("Mercury", 19.0),
        ("Jupiter", 24.0),
        ("Saturn", 30.0),
    ],
    [
        ("Jupiter", 12.0),
        ("Venus", 17.0),
        ("Mercury", 21.0),
        ("Saturn", 26.0),
        ("Mars", 30.0),
    ],
    [
        ("Mercury", 7.0),
        ("Jupiter", 14.0),
        ("Venus", 22.0),
        ("Saturn", 26.0),
        ("Mars", 30.0),
    ],
    [
        ("Mercury", 7.0),
        ("Venus", 13.0),
        ("Jupiter", 20.0),
        ("Mars", 25.0),
        ("Saturn", 30.0),
    ],
    [
        ("Venus", 12.0),
        ("Jupiter", 16.0),
        ("Mercury", 19.0),
        ("Mars", 28.0),
        ("Saturn", 30.0),
    ],
];

/// The Ptolemaic terms as Lilly printed them (Christian Astrology, 1647, p. 104).
const LILLY_TERMS: [Bounds; 12] = [
    [
        ("Jupiter", 6.0),
        ("Venus", 14.0),
        ("Mercury", 21.0),
        ("Mars", 26.0),
        ("Saturn", 30.0),
    ],
    [
        ("Venus", 8.0),
        ("Mercury", 15.0),
        ("Jupiter", 22.0),
        ("Saturn", 26.0),
        ("Mars", 30.0),
    ],
    [
        ("Mercury", 7.0),
        ("Jupiter", 14.0),
        ("Venus", 21.0),
        ("Saturn", 25.0),
        ("Mars", 30.0),
    ],
    [
        ("Mars", 6.0),
        ("Jupiter", 13.0),
        ("Mercury", 20.0),
        ("Venus", 27.0),
        ("Saturn", 30.0),
    ],
    [
        ("Saturn", 6.0),
        ("Mercury", 13.0),
        ("Venus", 19.0),
        ("Jupiter", 25.0),
        ("Mars", 30.0),
    ],
    [
        ("Mercury", 7.0),
        ("Venus", 13.0),
        ("Jupiter", 18.0),
        ("Saturn", 24.0),
        ("Mars", 30.0),
    ],
    [
        ("Saturn", 6.0),
        ("Venus", 11.0),
        ("Jupiter", 19.0),
        ("Mercury", 24.0),
        ("Mars", 30.0),
    ],
    [
        ("Mars", 6.0),
        ("Jupiter", 14.0),
        ("Venus", 21.0),
        ("Mercury", 27.0),
        ("Saturn", 30.0),
    ],
    [
        ("Jupiter", 8.0),
        ("Venus", 14.0),
        ("Mercury", 19.0),
        ("Saturn", 25.0),
        ("Mars", 30.0),
    ],
    [
        ("Venus", 6.0),
        ("Mercury", 12.0),
        ("Jupiter", 19.0),
        ("Mars", 25.0),
        ("Saturn", 30.0),
    ],
    [
        ("Saturn", 6.0),
        ("Mercury", 12.0),
        ("Venus", 20.0),
        ("Jupiter", 25.0),
        ("Mars", 30.0),
    ],
    [
        ("Venus", 8.0),
        ("Jupiter", 14.0),
        ("Mercury", 20.0),
        ("Mars", 25.0),
        ("Saturn", 30.0),
    ],
];

/// The sign index (0 = Aries) and the degrees within it of a longitude.
fn sign_and_degree(longitude: f64) -> (usize, f64) {
    let lon = pyfloat::rem(longitude, 360.0);
    let index = (pyfloat::floordiv(lon, 30.0) as usize).min(11);
    (index, lon - index as f64 * 30.0)
}

/// The lord of the bound a longitude falls in.
fn bound_lord(longitude: f64, scheme: DignityScheme) -> &'static str {
    let (sign, degree) = sign_and_degree(longitude);
    let table = match scheme {
        DignityScheme::Lilly => &LILLY_TERMS,
        DignityScheme::Dorothean => &EGYPTIAN_BOUNDS,
    };
    table[sign]
        .iter()
        .find(|(_, end)| degree < *end)
        .map_or(table[sign][4].0, |(lord, _)| lord)
}

/// The lord of the face (decan) a longitude falls in: Chaldean order from Mars at 0° Aries.
fn face_lord(longitude: f64) -> &'static str {
    let (sign, degree) = sign_and_degree(longitude);
    let decan = sign * 3 + ((degree / 10.0) as usize).min(2);
    // Mars is third in the Chaldean order (Saturn, Jupiter, Mars).
    SEVEN[(decan + 2) % 7]
}

/// The lords of each essential dignity at a longitude.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DignityLords {
    /// Lord of the sign.
    pub domicile: &'static str,
    /// Planet exalted in the sign, if any.
    pub exaltation: Option<&'static str>,
    /// Triplicity lords: by day and by night (Lilly), plus the participating one
    /// (Dorothean).
    pub triplicity: Vec<&'static str>,
    /// Lord of the bound (term).
    pub bound: &'static str,
    /// Lord of the face (decan).
    pub face: &'static str,
}

/// The lords of each dignity at `longitude`.
fn lords_at(longitude: f64, scheme: DignityScheme) -> DignityLords {
    let (sign, _) = sign_and_degree(longitude);
    let sign = SIGNS[sign];
    let el = element(sign).unwrap_or("Fire");
    DignityLords {
        domicile: traditional_ruler(sign).unwrap_or("Mars"),
        exaltation: exaltation(sign).map(|(planet, _)| planet),
        triplicity: match scheme {
            DignityScheme::Lilly => lilly_triplicity(el).map(|t| t.to_vec()),
            DignityScheme::Dorothean => dorothean_triplicity(el).map(|t| t.to_vec()),
        }
        .unwrap_or_default(),
        bound: bound_lord(longitude, scheme),
        face: face_lord(longitude),
    }
}

/// A planet's essential dignities and debilities where it stands.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EssentialDignity {
    /// In its own sign.
    pub domicile: bool,
    /// In the sign of its exaltation.
    pub exaltation: bool,
    /// Lord of the triplicity (Lilly: of the chart's sect; Dorothean: of the sect or
    /// participating).
    pub triplicity: bool,
    /// In its own bound.
    pub bound: bool,
    /// In its own face.
    pub face: bool,
    /// In the sign opposite its domicile.
    pub detriment: bool,
    /// In the sign opposite its exaltation.
    pub fall: bool,
    /// Without any of the five dignities.
    pub peregrine: bool,
    /// Lilly's points: domicile 5, exaltation 4, triplicity 3, bound 2, face 1; detriment
    /// −5, fall −4, peregrine −5.
    pub score: i64,
    /// The lords of the place, whatever the planet.
    pub lords: DignityLords,
}

/// The condition of one of the seven planets.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Condition {
    /// Its essential dignities.
    pub essential: EssentialDignity,
    /// "diurnal" or "nocturnal": the Sun, Jupiter and Saturn are diurnal; the Moon, Venus and
    /// Mars nocturnal; Mercury is diurnal when oriental, nocturnal when occidental.
    pub sect: &'static str,
    /// Whether its sect is the chart's.
    pub in_sect: bool,
    /// Whether it is above the horizon.
    pub above_horizon: bool,
    /// "cazimi", "combust" or "under_beams" when close to the Sun; `None` otherwise (and for
    /// the Sun).
    pub solar_phase: Option<&'static str>,
    /// "oriental" (rising before the Sun) or "occidental"; `None` for the luminaries.
    pub orientality: Option<&'static str>,
    /// "fast", "slow" or "stationary", against the planet's mean daily motion.
    pub motion: &'static str,
}

/// Mean daily motion in longitude, degrees (Lilly's values).
fn mean_motion(planet: &str) -> f64 {
    match planet {
        "Saturn" => 2.0 / 60.0 + 1.0 / 3600.0,
        "Jupiter" => 4.0 / 60.0 + 59.0 / 3600.0,
        "Mars" => 31.0 / 60.0 + 27.0 / 3600.0,
        "Moon" => 13.0 + 10.0 / 60.0 + 36.0 / 3600.0,
        // The Sun, Venus and Mercury share the Sun's mean motion.
        _ => 59.0 / 60.0 + 8.0 / 3600.0,
    }
}

/// Whether a longitude is above the horizon of an Ascendant.
fn is_above_horizon(longitude: f64, ascendant: f64) -> bool {
    pyfloat::rem(longitude - ascendant, 360.0) >= 180.0
}

/// A planet's phase to the Sun: cazimi, combust or under the beams.
pub(crate) fn solar_phase(planet: &str, longitude: f64, sun: f64) -> Option<&'static str> {
    if planet == "Sun" {
        return None;
    }
    let d = pyfloat::separation(longitude, sun);
    if d <= CAZIMI_ORB {
        Some("cazimi")
    } else if d < COMBUST_ORB {
        Some("combust")
    } else if d < UNDER_BEAMS_ORB {
        Some("under_beams")
    } else {
        None
    }
}

/// Oriental when it rises before the Sun: behind it in the zodiac, by less than 180°.
fn is_oriental(longitude: f64, sun: f64) -> bool {
    let behind = pyfloat::rem(sun - longitude, 360.0);
    behind > 0.0 && behind < 180.0
}

/// Whether the chart is diurnal: the Sun above the horizon.
pub(crate) fn is_diurnal(sun: f64, ascendant: f64) -> bool {
    is_above_horizon(sun, ascendant)
}

/// The essential dignities of `planet` at `longitude` in a day or night chart.
pub(crate) fn essential_dignity(
    planet: &str,
    longitude: f64,
    diurnal: bool,
    scheme: DignityScheme,
) -> EssentialDignity {
    let lords = lords_at(longitude, scheme);
    let (sign, _) = sign_and_degree(longitude);
    let opposite = SIGNS[(sign + 6) % 12];

    let domicile = lords.domicile == planet;
    let exalted = lords.exaltation == Some(planet);
    let triplicity = match (scheme, lords.triplicity.as_slice()) {
        (DignityScheme::Lilly, [day, night]) => planet == if diurnal { *day } else { *night },
        (DignityScheme::Dorothean, [day, night, participating]) => {
            planet == if diurnal { *day } else { *night } || planet == *participating
        }
        _ => false,
    };
    let bound = lords.bound == planet;
    let face = lords.face == planet;
    let detriment = traditional_ruler(opposite) == Some(planet);
    let fall = exaltation(opposite).map(|(p, _)| p) == Some(planet);
    let peregrine = !(domicile || exalted || triplicity || bound || face);

    let score = 5 * domicile as i64
        + 4 * exalted as i64
        + 3 * triplicity as i64
        + 2 * bound as i64
        + face as i64
        - 5 * detriment as i64
        - 4 * fall as i64
        - 5 * peregrine as i64;

    EssentialDignity {
        domicile,
        exaltation: exalted,
        triplicity,
        bound,
        face,
        detriment,
        fall,
        peregrine,
        score,
        lords,
    }
}

/// What judging a planet needs: its name, longitude and speed.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Body {
    pub name: &'static str,
    pub longitude: f64,
    pub speed: f64,
}

impl From<&Placement> for Body {
    fn from(p: &Placement) -> Self {
        Body {
            name: p.name,
            longitude: p.ecliptic_longitude,
            speed: p.speed,
        }
    }
}

/// The condition of a planet of the seven; `None` for any other point.
pub(crate) fn condition(
    planet: Body,
    sun: f64,
    ascendant: f64,
    scheme: DignityScheme,
) -> Option<Condition> {
    if !SEVEN.contains(&planet.name) {
        return None;
    }
    let lon = planet.longitude;
    let diurnal = is_diurnal(sun, ascendant);
    let orientality = match planet.name {
        "Sun" | "Moon" => None,
        _ if is_oriental(lon, sun) => Some("oriental"),
        _ => Some("occidental"),
    };
    let sect = match planet.name {
        "Sun" | "Jupiter" | "Saturn" => "diurnal",
        "Mercury" if orientality == Some("oriental") => "diurnal",
        _ => "nocturnal",
    };
    let mean = mean_motion(planet.name);
    let speed = planet.speed.abs();
    let motion = if !matches!(planet.name, "Sun" | "Moon") && speed < mean * STATIONARY_FRACTION {
        "stationary"
    } else if speed >= mean {
        "fast"
    } else {
        "slow"
    };

    Some(Condition {
        essential: essential_dignity(planet.name, lon, diurnal, scheme),
        sect,
        in_sect: (sect == "diurnal") == diurnal,
        above_horizon: is_above_horizon(lon, ascendant),
        solar_phase: solar_phase(planet.name, lon, sun),
        orientality,
        motion,
    })
}

/// One planet received by another: it stands in a dignity of the receiver.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reception {
    /// The planet received.
    pub planet: &'static str,
    /// The planet receiving it, lord of one or more dignities where it stands.
    pub receiver: &'static str,
    /// The dignities of the receiver at the planet's place: "domicile", "exaltation",
    /// "triplicity", "bound", "face".
    pub dignities: Vec<&'static str>,
    /// Whether the receiver is received in turn by domicile or exaltation, and the planet
    /// receives by domicile or exaltation.
    pub mutual: bool,
    /// The aspect between them, if any.
    pub aspect: Option<&'static str>,
}

/// The dignities `receiver` holds at `longitude`.
fn dignities_of(receiver: &str, longitude: f64, scheme: DignityScheme) -> Vec<&'static str> {
    let lords = lords_at(longitude, scheme);
    let mut out = Vec::new();
    if lords.domicile == receiver {
        out.push("domicile");
    }
    if lords.exaltation == Some(receiver) {
        out.push("exaltation");
    }
    if lords.triplicity.contains(&receiver) {
        out.push("triplicity");
    }
    if lords.bound == receiver {
        out.push("bound");
    }
    if lords.face == receiver {
        out.push("face");
    }
    out
}

fn is_strong(dignities: &[&str]) -> bool {
    dignities
        .iter()
        .any(|d| matches!(*d, "domicile" | "exaltation"))
}

/// Receptions among the seven planets: those joined by an aspect of the chart, by any
/// dignity; and the mutual receptions by domicile or exaltation, aspected or not.
pub(crate) fn receptions(
    planets: &[Body],
    aspect_between: impl Fn(&str, &str) -> Option<&'static str>,
    scheme: DignityScheme,
) -> Vec<Reception> {
    let seven: Vec<&Body> = SEVEN
        .iter()
        .filter_map(|name| planets.iter().find(|p| p.name == *name))
        .collect();
    let mut out = Vec::new();
    for planet in &seven {
        for receiver in &seven {
            if planet.name == receiver.name {
                continue;
            }
            let held = dignities_of(receiver.name, planet.longitude, scheme);
            if held.is_empty() {
                continue;
            }
            let back = dignities_of(planet.name, receiver.longitude, scheme);
            let mutual = is_strong(&held) && is_strong(&back);
            let aspect = aspect_between(planet.name, receiver.name);
            if aspect.is_some() || mutual {
                out.push(Reception {
                    planet: planet.name,
                    receiver: receiver.name,
                    dignities: held,
                    mutual,
                    aspect,
                });
            }
        }
    }
    out
}

/// The condition of one planet, by name.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanetCondition {
    /// One of the seven planets.
    pub name: &'static str,
    /// Its condition under the scheme.
    pub condition: Condition,
}

/// A chart's dignities judged again under a scheme: what [`chart_dignities`] returns.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChartDignities {
    /// The scheme used: "lilly" or "dorothean".
    pub dignity_scheme: &'static str,
    /// "diurnal" or "nocturnal".
    pub sect: &'static str,
    /// The condition of each of the seven planets in the chart.
    pub conditions: Vec<PlanetCondition>,
    /// Receptions among the seven planets.
    pub receptions: Vec<Reception>,
}

/// The static name of a chart point given as text (the seven planets and the Ascendant).
fn static_name(name: &str) -> Option<&'static str> {
    SEVEN
        .iter()
        .copied()
        .chain(["Ascendant"])
        .find(|n| *n == name)
}

/// The static name of a Ptolemaic or minor aspect given as text.
fn static_aspect(name: &str) -> Option<&'static str> {
    ASPECT_RULES.iter().map(|r| r.name).find(|n| *n == name)
}

/// Judge the dignities of an already computed chart (its JSON, as [`calculate_chart`](crate::calculate_chart)
/// returns it: `planets` with `name`, `ecliptic_longitude` and `speed`, and `aspects`) under
/// `scheme` ("lilly" or "dorothean"). No kernel is needed: a chart stored with one scheme can
/// be shown with the other.
pub fn chart_dignities(chart: &Value, scheme: &str) -> Result<ChartDignities, EngineError> {
    let scheme = DignityScheme::from_code(scheme);
    let invalid = |what: &str| EngineError::InvalidInput(format!("chart_dignities: {what}"));
    let planets = chart
        .get("planets")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("the chart has no planets"))?;
    let bodies: Vec<Body> = planets
        .iter()
        .filter_map(|p| {
            Some(Body {
                name: static_name(p.get("name")?.as_str()?)?,
                longitude: p.get("ecliptic_longitude")?.as_f64()?,
                speed: p.get("speed").and_then(Value::as_f64).unwrap_or(0.0),
            })
        })
        .collect();
    let at = |name: &str| bodies.iter().find(|b| b.name == name).map(|b| b.longitude);
    let (sun, asc) = at("Sun")
        .zip(at("Ascendant"))
        .ok_or_else(|| invalid("the chart needs the Sun and the Ascendant"))?;

    let aspects: Vec<(String, String, &'static str)> = chart
        .get("aspects")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|a| {
                    Some((
                        a.get("body1")?.as_str()?.to_string(),
                        a.get("body2")?.as_str()?.to_string(),
                        static_aspect(a.get("aspect_type")?.as_str()?)?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    let aspect_between = |a: &str, b: &str| {
        aspects
            .iter()
            .find(|(x, y, _)| (x == a && y == b) || (x == b && y == a))
            .map(|(_, _, kind)| *kind)
    };

    Ok(ChartDignities {
        dignity_scheme: scheme.code(),
        sect: if is_diurnal(sun, asc) {
            "diurnal"
        } else {
            "nocturnal"
        },
        conditions: bodies
            .iter()
            .filter_map(|b| {
                condition(*b, sun, asc, scheme).map(|condition| PlanetCondition {
                    name: b.name,
                    condition,
                })
            })
            .collect(),
        receptions: receptions(&bodies, aspect_between, scheme),
    })
}

/// Two points joined by antiscion (mirrored across the solstitial axis, Cancer–Capricorn)
/// or contra-antiscion (across the equinoctial axis, Aries–Libra).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AntisciaContact {
    /// First point.
    pub body1: &'static str,
    /// Second point.
    pub body2: &'static str,
    /// "antiscion" or "contra_antiscion".
    pub kind: &'static str,
    /// Distance from exactness, degrees (rounded to 0.01).
    pub orb: f64,
}

/// The antiscion of a longitude: its mirror across 0° Cancer / 0° Capricorn.
pub(crate) fn antiscion(longitude: f64) -> f64 {
    pyfloat::rem(180.0 - longitude, 360.0)
}

/// The contra-antiscion of a longitude: its mirror across 0° Aries / 0° Libra.
pub(crate) fn contra_antiscion(longitude: f64) -> f64 {
    pyfloat::rem(360.0 - longitude, 360.0)
}

/// Antiscia and contra-antiscia among the seven planets and the angles.
pub(crate) fn antiscia(planets: &[Placement]) -> Vec<AntisciaContact> {
    let points: Vec<&Placement> = planets
        .iter()
        .filter(|p| SEVEN.contains(&p.name) || matches!(p.name, "Ascendant" | "Midheaven"))
        .collect();
    let mut out = Vec::new();
    for (i, a) in points.iter().enumerate() {
        for b in &points[i + 1..] {
            for (kind, mirror) in [
                ("antiscion", antiscion(a.ecliptic_longitude)),
                ("contra_antiscion", contra_antiscion(a.ecliptic_longitude)),
            ] {
                let orb = pyfloat::separation(mirror, b.ecliptic_longitude);
                if orb <= ANTISCIA_ORB {
                    out.push(AntisciaContact {
                        body1: a.name,
                        body2: b.name,
                        kind,
                        orb: pyfloat::round(orb, 2),
                    });
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lon(sign: &str, degree: f64) -> f64 {
        sign_index(sign).unwrap() as f64 * 30.0 + degree
    }

    #[test]
    fn bound_tables_partition_every_sign() {
        for table in [&EGYPTIAN_BOUNDS, &LILLY_TERMS] {
            for bounds in table.iter() {
                let mut lords: Vec<&str> = bounds.iter().map(|(l, _)| *l).collect();
                lords.sort_unstable();
                assert_eq!(lords, ["Jupiter", "Mars", "Mercury", "Saturn", "Venus"]);
                assert!(bounds.windows(2).all(|w| w[0].1 < w[1].1));
                assert_eq!(bounds[4].1, 30.0);
            }
        }
    }

    #[test]
    fn egyptian_bounds_give_each_planet_its_years() {
        // The minor years of the planets: Saturn 57, Jupiter 79, Mars 66, Venus 82,
        // Mercury 76 (Valens, Anthology III.13).
        let mut total = std::collections::HashMap::new();
        for bounds in EGYPTIAN_BOUNDS {
            let mut start = 0.0;
            for (lord, end) in bounds {
                *total.entry(lord).or_insert(0.0) += end - start;
                start = end;
            }
        }
        assert_eq!(total["Saturn"], 57.0);
        assert_eq!(total["Jupiter"], 79.0);
        assert_eq!(total["Mars"], 66.0);
        assert_eq!(total["Venus"], 82.0);
        assert_eq!(total["Mercury"], 76.0);
    }

    #[test]
    fn faces_follow_the_chaldean_order_from_mars() {
        assert_eq!(face_lord(lon("Aries", 0.0)), "Mars");
        assert_eq!(face_lord(lon("Aries", 15.0)), "Sun");
        assert_eq!(face_lord(lon("Aries", 29.9)), "Venus");
        assert_eq!(face_lord(lon("Taurus", 5.0)), "Mercury");
        assert_eq!(face_lord(lon("Leo", 0.0)), "Saturn");
        assert_eq!(face_lord(lon("Pisces", 25.0)), "Mars");
    }

    #[test]
    fn bounds_by_scheme() {
        // 10° Aries: Venus in Lilly's terms, Venus in the Egyptian bounds;
        // 13° Aries: Venus (Lilly) but Mercury (Egyptian).
        assert_eq!(
            bound_lord(lon("Aries", 10.0), DignityScheme::Lilly),
            "Venus"
        );
        assert_eq!(
            bound_lord(lon("Aries", 13.0), DignityScheme::Lilly),
            "Venus"
        );
        assert_eq!(
            bound_lord(lon("Aries", 13.0), DignityScheme::Dorothean),
            "Mercury"
        );
        assert_eq!(
            bound_lord(lon("Pisces", 29.99), DignityScheme::Lilly),
            "Saturn"
        );
        assert_eq!(bound_lord(lon("Leo", 0.0), DignityScheme::Lilly), "Saturn");
        assert_eq!(
            bound_lord(lon("Leo", 0.0), DignityScheme::Dorothean),
            "Jupiter"
        );
    }

    #[test]
    fn lillys_points() {
        // The Sun at 19° Aries by day: exaltation 4, triplicity 3, face (Sun 10-20) 1.
        let sun = essential_dignity("Sun", lon("Aries", 19.0), true, DignityScheme::Lilly);
        assert!(sun.exaltation && sun.triplicity && sun.face && !sun.domicile);
        assert_eq!(sun.score, 8);
        // Saturn at 5° Aries: fall, and nothing else: −4 −5.
        let saturn = essential_dignity("Saturn", lon("Aries", 5.0), true, DignityScheme::Lilly);
        assert!(saturn.fall && saturn.peregrine && !saturn.detriment);
        assert_eq!(saturn.score, -9);
        // Mars in water is a triplicity lord by night too, for Lilly only.
        let mars = essential_dignity("Mars", lon("Pisces", 2.0), true, DignityScheme::Lilly);
        assert!(mars.triplicity);
        let mars = essential_dignity("Mars", lon("Pisces", 2.0), true, DignityScheme::Dorothean);
        assert!(!mars.triplicity);
        // Venus in Libra: domicile 5, plus her term at 6-11 (Lilly).
        let venus = essential_dignity("Venus", lon("Libra", 8.0), false, DignityScheme::Lilly);
        assert!(venus.domicile && venus.bound);
        assert_eq!(venus.lords.triplicity, ["Saturn", "Mercury"]);
        // The Moon in Capricorn by night: detriment, but the night lord of earth (Dorothean).
        let moon = essential_dignity(
            "Moon",
            lon("Capricorn", 1.0),
            false,
            DignityScheme::Dorothean,
        );
        assert!(moon.detriment && moon.triplicity && !moon.peregrine);
        let moon = essential_dignity(
            "Moon",
            lon("Capricorn", 1.0),
            true,
            DignityScheme::Dorothean,
        );
        assert!(!moon.triplicity);
        assert_eq!(moon.lords.triplicity, ["Venus", "Moon", "Mars"]);
    }

    #[test]
    fn solar_phases() {
        assert_eq!(solar_phase("Mercury", 100.1, 100.0), Some("cazimi"));
        assert_eq!(solar_phase("Mercury", 105.0, 100.0), Some("combust"));
        assert_eq!(solar_phase("Venus", 88.0, 100.0), Some("under_beams"));
        assert_eq!(solar_phase("Mars", 130.0, 100.0), None);
        assert_eq!(solar_phase("Sun", 100.0, 100.0), None);
        assert_eq!(solar_phase("Moon", 355.0, 3.0), Some("combust"));
    }

    #[test]
    fn mirrors() {
        // 10° Gemini mirrors to 20° Cancer (antiscion) and 20° Capricorn (contra).
        assert!((antiscion(lon("Gemini", 10.0)) - lon("Cancer", 20.0)).abs() < 1e-9);
        assert!((contra_antiscion(lon("Gemini", 10.0)) - lon("Capricorn", 20.0)).abs() < 1e-9);
        assert!((antiscion(lon("Capricorn", 5.0)) - lon("Sagittarius", 25.0)).abs() < 1e-9);
    }

    #[test]
    fn sect_and_orientality() {
        // Ascendant at 0° Cancer puts the Midheaven near 0° Aries: a Sun there culminates.
        assert!(is_diurnal(0.0, 90.0));
        // Ascendant at 0° Capricorn puts 0° Aries on the lower heaven, below the horizon.
        assert!(!is_diurnal(0.0, 270.0));
        assert!(is_oriental(350.0, 0.0));
        assert!(!is_oriental(10.0, 0.0));
    }
}
