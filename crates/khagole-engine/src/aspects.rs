//! Aspects, orbs and cross-aspects (`charts/calc/aspects.py`).

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::error::EngineError;
use crate::pyfloat;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AspectRule {
    pub name: &'static str,
    pub angle: f64,
    pub symbol: &'static str,
    pub orb: f64,
    pub is_major: bool,
}

pub const ASPECT_RULES: [AspectRule; 7] = [
    AspectRule {
        name: "Conjunction",
        angle: 0.0,
        symbol: "☌",
        orb: 8.0,
        is_major: true,
    },
    AspectRule {
        name: "Sextile",
        angle: 60.0,
        symbol: "⚹",
        orb: 6.0,
        is_major: true,
    },
    AspectRule {
        name: "Square",
        angle: 90.0,
        symbol: "□",
        orb: 7.0,
        is_major: true,
    },
    AspectRule {
        name: "Trine",
        angle: 120.0,
        symbol: "△",
        orb: 8.0,
        is_major: true,
    },
    AspectRule {
        name: "Opposition",
        angle: 180.0,
        symbol: "☍",
        orb: 8.0,
        is_major: true,
    },
    AspectRule {
        name: "Semi-Sextile",
        angle: 30.0,
        symbol: "⚺",
        orb: 2.5,
        is_major: false,
    },
    AspectRule {
        name: "Quincunx",
        angle: 150.0,
        symbol: "⚻",
        orb: 3.0,
        is_major: false,
    },
];

const ASPECT_WEIGHT_FACTORS: [(&str, f64); 7] = [
    ("Conjunction", 1.0),
    ("Opposition", 1.0),
    ("Trine", 1.0),
    ("Square", 0.875),
    ("Sextile", 0.75),
    ("Semi-Sextile", 0.35),
    ("Quincunx", 0.4),
];

/// `DEFAULT_ORB_SETTINGS`, in the reference's key order.
pub fn default_orb_settings() -> Map<String, Value> {
    let value = json!({
        "method": "moiety_hybrid",
        "include_minor_aspects": false,
        "include_chiron": false,
        "include_lilith": false,
        "aspect_orbs": {
            "Conjunction": 8.0,
            "Sextile": 6.0,
            "Square": 7.0,
            "Trine": 8.0,
            "Opposition": 8.0,
            "Semi-Sextile": 2.5,
            "Quincunx": 3.0,
        },
        "planet_orbs": {
            "Sun": 15.0,
            "Moon": 12.0,
            "Mercury": 7.0,
            "Venus": 7.0,
            "Mars": 8.0,
            "Jupiter": 9.0,
            "Saturn": 9.0,
            "Uranus": 5.0,
            "Neptune": 5.0,
            "Pluto": 5.0,
            "North Node": 5.0,
            "South Node": 5.0,
            "Chiron": 5.0,
            "Lilith": 5.0,
            "Ascendant": 5.0,
            "Midheaven": 5.0,
        },
        "fixed_star_orb": 1.5,
    });
    match value {
        Value::Object(map) => map,
        _ => unreachable!(),
    }
}

/// Python `float(value)` for JSON values: numbers, numeric strings and booleans.
pub fn py_float(value: &Value) -> Result<f64, EngineError> {
    match value {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| EngineError::InvalidInput(format!("not a number: {n}"))),
        Value::Bool(b) => Ok(f64::from(u8::from(*b))),
        Value::String(s) => {
            let t = s.trim().to_ascii_lowercase().replace('_', "");
            match t.as_str() {
                "inf" | "+inf" | "infinity" | "+infinity" => Ok(f64::INFINITY),
                "-inf" | "-infinity" => Ok(f64::NEG_INFINITY),
                "nan" | "+nan" | "-nan" => Ok(f64::NAN),
                _ => t
                    .parse()
                    .map_err(|_| EngineError::InvalidInput(format!("not a number: {s:?}"))),
            }
        }
        other => Err(EngineError::InvalidInput(format!("not a number: {other}"))),
    }
}

/// `value`, or an error naming `what` when it is NaN or infinite.
pub(crate) fn finite(value: f64, what: &str) -> Result<f64, EngineError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::InvalidInput(format!(
            "{what} must be finite, got {value}"
        )))
    }
}

/// Orb settings merged over the defaults, as the orchestrator and cross-aspects do:
/// only `method`, `fixed_star_orb`, `aspect_orbs` and `planet_orbs` are taken from the
/// caller, the orb tables key by key. Values are kept as given (they are echoed back in
/// the chart) and read with Python `float()` semantics when used.
#[derive(Debug, Clone, PartialEq)]
pub struct OrbSettings(Map<String, Value>);

impl Default for OrbSettings {
    fn default() -> Self {
        OrbSettings(default_orb_settings())
    }
}

impl OrbSettings {
    pub fn merge(custom: Option<&Value>) -> Result<Self, EngineError> {
        let mut merged = default_orb_settings();
        // `if orb_settings and isinstance(orb_settings, dict)`: an empty dict keeps defaults.
        let Some(Value::Object(custom)) = custom else {
            return Ok(OrbSettings(merged));
        };
        if custom.is_empty() {
            return Ok(OrbSettings(merged));
        }
        if let Some(method) = custom.get("method") {
            merged.insert("method".into(), method.clone());
        }
        let star_orb = match custom.get("fixed_star_orb") {
            Some(v) => finite(py_float(v)?, "fixed_star_orb")?,
            None => 1.5,
        };
        merged.insert("fixed_star_orb".into(), json!(star_orb));
        for key in ["aspect_orbs", "planet_orbs"] {
            let table = merged.get_mut(key).and_then(Value::as_object_mut).unwrap();
            match custom.get(key) {
                None => {}
                Some(Value::Object(given)) => {
                    for (k, v) in given {
                        // Non-numeric values fail when used, as in the reference; NaN
                        // and infinite orbs would silently drop or match every aspect.
                        if let Ok(orb) = py_float(v) {
                            finite(orb, &format!("{key}.{k}"))?;
                        }
                        table.insert(k.clone(), v.clone());
                    }
                }
                Some(other) => {
                    return Err(EngineError::InvalidInput(format!(
                        "{key} must be an object, got {other}"
                    )))
                }
            }
        }
        Ok(OrbSettings(merged))
    }

    #[cfg(test)]
    pub fn as_map(&self) -> &Map<String, Value> {
        &self.0
    }

    pub fn to_value(&self) -> Value {
        Value::Object(self.0.clone())
    }

    fn table(&self, key: &str) -> Option<&Map<String, Value>> {
        self.0.get(key).and_then(Value::as_object)
    }

    fn lookup(&self, table: &str, name: &str, default: f64) -> Result<f64, EngineError> {
        match self.table(table).and_then(|t| t.get(name)) {
            Some(v) => py_float(v),
            None => Ok(default),
        }
    }

    pub fn fixed_star_orb(&self) -> Result<f64, EngineError> {
        match self.0.get("fixed_star_orb") {
            Some(v) => py_float(v),
            None => Ok(1.5),
        }
    }

    /// `get_effective_max_orb` for two bodies and an aspect.
    pub fn max_orb(&self, body1: &str, body2: &str, aspect: &str) -> Result<f64, EngineError> {
        let method = self.0.get("method").and_then(Value::as_str).unwrap_or("");
        let default_aspect_orb = self.lookup("aspect_orbs", aspect, 8.0)?;
        match method {
            "moiety_traditional" | "moiety_aspect_weighted" | "moiety_hybrid" => {
                let moiety = (self.lookup("planet_orbs", body1, 5.0)?
                    + self.lookup("planet_orbs", body2, 5.0)?)
                    / 2.0;
                Ok(match method {
                    "moiety_aspect_weighted" => {
                        let conj = self.lookup("aspect_orbs", "Conjunction", 8.0)?;
                        let has_aspect = self
                            .table("aspect_orbs")
                            .is_some_and(|t| t.contains_key(aspect));
                        let weight = if conj > 0.0 && has_aspect {
                            self.lookup("aspect_orbs", aspect, 8.0)? / conj
                        } else {
                            ASPECT_WEIGHT_FACTORS
                                .iter()
                                .find(|(name, _)| *name == aspect)
                                .map_or(1.0, |(_, w)| *w)
                        };
                        pyfloat::round(moiety * weight, 2)
                    }
                    "moiety_hybrid" => {
                        let capped = if default_aspect_orb < moiety {
                            default_aspect_orb
                        } else {
                            moiety
                        };
                        pyfloat::round(capped, 2)
                    }
                    _ => pyfloat::round(moiety, 2),
                })
            }
            _ => Ok(default_aspect_orb),
        }
    }
}

/// An aspect between two chart points. Fixed-star conjunctions carry no `is_major`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Aspect {
    /// First point.
    pub body1: String,
    /// Second point.
    pub body2: String,
    /// Aspect name, e.g. "Trine".
    pub aspect_type: &'static str,
    /// Aspect glyph, e.g. "△".
    pub symbol: &'static str,
    /// Exact angle of the aspect, in degrees.
    pub angle: f64,
    /// Distance from exactness, in degrees (rounded to 0.01).
    pub orb: f64,
    /// Largest orb allowed for this pair and aspect, in degrees.
    pub max_orb: f64,
    /// Whether it is a major (Ptolemaic) aspect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_major: Option<bool>,
    /// Always `true`, as in the reference implementation.
    pub is_applying: bool,
}

/// A cross-aspect between an overlay body (transit, partner) and a base body (natal).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CrossAspect {
    /// Overlay point (transit, partner).
    pub body1: String,
    /// Always "overlay".
    pub body1_source: &'static str,
    /// Base point (natal).
    pub body2: String,
    /// Always "base".
    pub body2_source: &'static str,
    /// Aspect name, e.g. "Trine".
    pub aspect_type: &'static str,
    /// Aspect glyph, e.g. "△".
    pub symbol: &'static str,
    /// Exact angle of the aspect, in degrees.
    pub angle: f64,
    /// Distance from exactness, in degrees (rounded to 0.01).
    pub orb: f64,
    /// Largest orb allowed for this pair and aspect, in degrees.
    pub max_orb: f64,
    /// Whether it is a major (Ptolemaic) aspect.
    pub is_major: bool,
    /// Always `true`, as in the reference implementation.
    pub is_applying: bool,
}

/// A named point on the ecliptic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point<'a> {
    pub name: &'a str,
    pub longitude: f64,
}

fn is_node_pair(a: &str, b: &str) -> bool {
    (a == "North Node" && b == "South Node") || (a == "South Node" && b == "North Node")
}

/// The first aspect rule within orb for a separation, with its orb and max orb.
fn find_aspect(
    orbs: &OrbSettings,
    a: &str,
    b: &str,
    diff: f64,
) -> Result<Option<(&'static AspectRule, f64, f64)>, EngineError> {
    for rule in &ASPECT_RULES {
        let max_orb = orbs.max_orb(a, b, rule.name)?;
        let orb = (diff - rule.angle).abs();
        if orb <= max_orb {
            return Ok(Some((rule, orb, max_orb)));
        }
    }
    Ok(None)
}

/// Aspects between every pair of chart points, skipping the nodal axis.
pub fn natal_aspects(points: &[Point], orbs: &OrbSettings) -> Result<Vec<Aspect>, EngineError> {
    let mut out = Vec::new();
    for (i, p1) in points.iter().enumerate() {
        for p2 in &points[i + 1..] {
            if is_node_pair(p1.name, p2.name) {
                continue;
            }
            let diff = pyfloat::separation(p1.longitude, p2.longitude);
            if let Some((rule, orb, max_orb)) = find_aspect(orbs, p1.name, p2.name, diff)? {
                out.push(Aspect {
                    body1: p1.name.to_string(),
                    body2: p2.name.to_string(),
                    aspect_type: rule.name,
                    symbol: rule.symbol,
                    angle: rule.angle,
                    orb: pyfloat::round(orb, 2),
                    max_orb: pyfloat::round(max_orb, 2),
                    is_major: Some(rule.is_major),
                    is_applying: true,
                });
            }
        }
    }
    Ok(out)
}

/// `calculate_cross_aspects`: every overlay body against every base body.
pub fn cross_aspects(
    base: &[Point],
    overlay: &[Point],
    custom_orbs: Option<&Value>,
) -> Result<Vec<CrossAspect>, EngineError> {
    let orbs = OrbSettings::merge(custom_orbs)?;
    let mut out = Vec::new();
    for o in overlay {
        for b in base {
            if is_node_pair(o.name, b.name) {
                continue;
            }
            let diff = pyfloat::separation(o.longitude, b.longitude);
            if let Some((rule, orb, max_orb)) = find_aspect(&orbs, o.name, b.name, diff)? {
                out.push(CrossAspect {
                    body1: o.name.to_string(),
                    body1_source: "overlay",
                    body2: b.name.to_string(),
                    body2_source: "base",
                    aspect_type: rule.name,
                    symbol: rule.symbol,
                    angle: rule.angle,
                    orb: pyfloat::round(orb, 2),
                    max_orb: pyfloat::round(max_orb, 2),
                    is_major: rule.is_major,
                    is_applying: true,
                });
            }
        }
    }
    Ok(out)
}

/// Chart points (`name`, `ecliptic_longitude`) from a stored planets list.
pub fn points_from_json(planets: &Value) -> Result<Vec<Point<'_>>, EngineError> {
    let list = planets
        .as_array()
        .ok_or_else(|| EngineError::InvalidInput("planets must be a list".into()))?;
    list.iter()
        .map(|p| {
            let name = p
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| EngineError::InvalidInput("every planet needs a name".into()))?;
            let longitude = p.get("ecliptic_longitude").map(py_float).ok_or_else(|| {
                EngineError::InvalidInput(format!("{name} has no ecliptic_longitude"))
            })??;
            let longitude = finite(longitude, &format!("{name}'s ecliptic_longitude"))?;
            Ok(Point { name, longitude })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_orb_is_capped_by_aspect() {
        let orbs = OrbSettings::default();
        // Sun 15 + Moon 12 → moiety 13.5, capped by Conjunction 8.
        assert_eq!(orbs.max_orb("Sun", "Moon", "Conjunction").unwrap(), 8.0);
        // Uranus 5 + Neptune 5 → 5, under Conjunction's 8.
        assert_eq!(
            orbs.max_orb("Uranus", "Neptune", "Conjunction").unwrap(),
            5.0
        );
        assert_eq!(orbs.max_orb("Sun", "Moon", "Semi-Sextile").unwrap(), 2.5);
    }

    #[test]
    fn custom_orbs_merge_key_by_key() {
        let custom = json!({"method": "fixed_aspect", "aspect_orbs": {"Trine": "6"}});
        let orbs = OrbSettings::merge(Some(&custom)).unwrap();
        assert_eq!(orbs.max_orb("Sun", "Moon", "Trine").unwrap(), 6.0);
        assert_eq!(orbs.max_orb("Sun", "Moon", "Square").unwrap(), 7.0);
        assert_eq!(orbs.as_map()["aspect_orbs"]["Trine"], json!("6"));
        let weighted = json!({"method": "moiety_aspect_weighted"});
        let orbs = OrbSettings::merge(Some(&weighted)).unwrap();
        // moiety 13.5 × (6 / 8)
        assert_eq!(orbs.max_orb("Sun", "Moon", "Sextile").unwrap(), 10.12); // 10.125, ties to even
    }
}
