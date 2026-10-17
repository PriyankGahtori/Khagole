//! Charts built on other charts (`charts/calc/derived.py`): transits against a natal
//! chart, synastry between two charts, and derived (turned) charts.

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::aspects::{cross_aspects, points_from_json, CrossAspect};
use crate::catalog::{DERIVED_HOUSE_MEANINGS, ROOT_HOUSE_THEMES};
use crate::chart::{chart_without_hours, ChartRequest, HouseCusp, Placement};
use crate::ephemeris::KernelSet;
use crate::error::EngineError;
use crate::zodiac::ZODIAC_SIGNS;

/// Transits to a natal chart (`calculate_transit_chart`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TransitChart {
    /// Transiting bodies and angles.
    pub transit_planets: Vec<Placement>,
    /// House cusps at the transit time and place.
    pub transit_houses: Vec<HouseCusp>,
    /// Aspects from transiting (overlay) to natal (base) points.
    pub cross_aspects: Vec<CrossAspect>,
    /// Transit instant, ISO 8601 UTC.
    pub transit_datetime: String,
    /// Transit latitude in degrees, north positive.
    pub transit_lat: f64,
    /// Transit longitude in degrees, east positive.
    pub transit_lon: f64,
    /// "tropical" or "sidereal".
    pub zodiac_type: &'static str,
    /// Ayanamsa code; `None` for tropical charts.
    pub ayanamsa: Option<&'static str>,
    /// Human-readable ayanamsa name; `None` for tropical charts.
    pub ayanamsa_name: Option<&'static str>,
    /// Ayanamsa at the chart instant, in degrees; `None` for tropical charts.
    pub ayanamsa_value: Option<f64>,
    /// `ayanamsa_value` as degrees, minutes and seconds; `None` for tropical charts.
    pub ayanamsa_formatted: Option<String>,
}

/// The sky at `req` (relocatable), with its cross-aspects to `natal_planets`
/// (`calculate_transit_chart`). `req.chinese_calendar` is ignored: a transit chart has
/// no calendar.
pub fn calculate_transit_chart(
    kernels: &KernelSet,
    natal_planets: &Value,
    req: &ChartRequest,
) -> Result<TransitChart, EngineError> {
    let natal = points_from_json(natal_planets)?;
    let req = &ChartRequest {
        chinese_calendar: false,
        ..req.clone()
    };
    let chart = chart_without_hours(kernels, req)?;
    let aspects = cross_aspects(&natal, &chart.points(), req.orb_settings)?;
    Ok(TransitChart {
        cross_aspects: aspects,
        transit_datetime: req.instant.isoformat(),
        transit_lat: req.latitude,
        transit_lon: req.longitude,
        zodiac_type: chart.zodiac_type,
        ayanamsa: chart.ayanamsa,
        ayanamsa_name: chart.ayanamsa_name,
        ayanamsa_value: chart.ayanamsa_value,
        ayanamsa_formatted: chart.ayanamsa_formatted,
        transit_planets: chart.planets,
        transit_houses: chart.houses,
    })
}

/// Synastry between two charts.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Synastry {
    /// Aspects from chart B (overlay) to chart A (base).
    pub cross_aspects: Vec<CrossAspect>,
}

/// Cross-aspects from chart B (overlay) to chart A (base) (`calculate_synastry_chart`).
pub fn calculate_synastry(
    chart_a_planets: &Value,
    chart_b_planets: &Value,
    orb_settings: Option<&Value>,
) -> Result<Synastry, EngineError> {
    let a = points_from_json(chart_a_planets)?;
    let b = points_from_json(chart_b_planets)?;
    Ok(Synastry {
        cross_aspects: cross_aspects(&a, &b, orb_settings)?,
    })
}

/// Python `dict.get(key, default)`.
fn get(obj: &Map<String, Value>, key: &str, default: Value) -> Value {
    obj.get(key).cloned().unwrap_or(default)
}

/// Python truthiness of a JSON value.
fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64() != Some(0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

fn house_number_is(h: &Value, n: i64) -> bool {
    h.get("house_number").and_then(Value::as_f64) == Some(n as f64)
}

/// `((radix_house - root) % 12) + 1` for a stored house number (default 1).
fn turned_house(item: &Map<String, Value>, root: i64) -> (Value, i64) {
    let radix = get(item, "house", json!(1));
    let n = radix.as_f64().unwrap_or(1.0) as i64;
    (radix, (n - root).rem_euclid(12) + 1)
}

fn turn_items(base: &Map<String, Value>, key: &str, root: i64) -> Result<Vec<Value>, EngineError> {
    let items = match base.get(key) {
        None | Some(Value::Null) => return Ok(Vec::new()),
        Some(Value::Array(items)) => items,
        Some(other) => {
            return Err(EngineError::InvalidInput(format!(
                "{key} must be a list, got {other}"
            )))
        }
    };
    items
        .iter()
        .map(|item| {
            let mut copy = item.as_object().cloned().ok_or_else(|| {
                EngineError::InvalidInput(format!("{key} entries must be objects"))
            })?;
            let (radix, turned) = turned_house(&copy, root);
            copy.insert("radix_house".into(), radix);
            copy.insert("house".into(), json!(turned));
            Ok(Value::Object(copy))
        })
        .collect()
}

/// A derived (turned) chart: radix house `root` (1-12) becomes the first house; cusps,
/// angles and house placements follow, with the meaning of each derived house
/// (`calculate_derived_chart_data`). Works on a stored chart payload, keeping every
/// field it does not turn.
pub fn calculate_derived_chart(
    base: &Value,
    root: i64,
    custom_name: Option<&str>,
) -> Result<Value, EngineError> {
    let base = base
        .as_object()
        .ok_or_else(|| EngineError::InvalidInput("base chart must be an object".into()))?;
    let root = if (1..=12).contains(&root) { root } else { 1 };
    let meanings = &DERIVED_HOUSE_MEANINGS[root as usize - 1];

    let stored_houses = base
        .get("houses")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let base_houses: Vec<Value> = if stored_houses.len() >= 12 {
        stored_houses
    } else {
        (0..12)
            .map(|i| {
                json!({
                    "house_number": i + 1,
                    "degree": i * 30,
                    "minute": 0,
                    "sign": ZODIAC_SIGNS[i].name,
                    "sign_symbol": ZODIAC_SIGNS[i].symbol,
                    "ecliptic_longitude": (i * 30) as f64,
                })
            })
            .collect()
    };
    let house_obj = |r: i64| -> Map<String, Value> {
        base_houses
            .iter()
            .find(|h| house_number_is(h, r))
            .unwrap_or(&base_houses[r as usize - 1])
            .as_object()
            .cloned()
            .unwrap_or_default()
    };

    let mut derived_houses: Vec<Map<String, Value>> = Vec::with_capacity(12);
    for d in 1..=12_i64 {
        let r = (root - 1 + d - 1) % 12 + 1;
        let radix = house_obj(r);
        let meaning = meanings[d as usize - 1];
        let mut h = Map::new();
        h.insert("house_number".into(), json!(d));
        h.insert("radix_house_number".into(), json!(r));
        h.insert("sign".into(), get(&radix, "sign", json!("")));
        h.insert("sign_symbol".into(), get(&radix, "sign_symbol", json!("")));
        h.insert("degree".into(), get(&radix, "degree", json!(0)));
        h.insert("minute".into(), get(&radix, "minute", json!(0)));
        h.insert(
            "ecliptic_longitude".into(),
            get(&radix, "ecliptic_longitude", json!(0.0)),
        );
        h.insert(
            "symbolic_degree".into(),
            get(&radix, "symbolic_degree", Value::Null),
        );
        h.insert(
            "degree_symbol".into(),
            get(&radix, "degree_symbol", Value::Null),
        );
        h.insert(
            "degree_symbols".into(),
            get(&radix, "degree_symbols", Value::Null),
        );
        h.insert("meaning_en".into(), json!(meaning.en));
        h.insert("meaning_it".into(), json!(meaning.it));
        h.insert(
            "derived_label".into(),
            json!(format!("House {d} (Radix {r})")),
        );
        derived_houses.push(h);
    }

    let mut planets = Vec::new();
    if let Some(Value::Array(items)) = base.get("planets") {
        for p in items {
            let mut copy = p.as_object().cloned().ok_or_else(|| {
                EngineError::InvalidInput("planets entries must be objects".into())
            })?;
            let (radix, turned) = turned_house(&copy, root);
            copy.insert("radix_house".into(), radix);
            let angle = match copy.get("name").and_then(Value::as_str) {
                Some("Ascendant") => Some((0, 1)),
                Some("Midheaven") => Some((9, 10)),
                _ => None,
            };
            match angle {
                Some((index, house)) => {
                    let cusp = &derived_houses[index];
                    for key in [
                        "sign",
                        "sign_symbol",
                        "degree",
                        "minute",
                        "ecliptic_longitude",
                    ] {
                        copy.insert(key.into(), cusp[key].clone());
                    }
                    copy.insert("house".into(), json!(house));
                    if truthy(cusp.get("symbolic_degree")) {
                        for key in ["symbolic_degree", "degree_symbol", "degree_symbols"] {
                            copy.insert(key.into(), cusp[key].clone());
                        }
                    }
                }
                None => {
                    copy.insert("house".into(), json!(turned));
                }
            }
            planets.push(Value::Object(copy));
        }
    }
    let fixed_stars = turn_items(base, "fixed_stars", root)?;
    let lots = turn_items(base, "arabic_parts", root)?;

    let (theme_en, theme_it) = ROOT_HOUSE_THEMES[root as usize - 1];
    let root_cusp = base_houses
        .iter()
        .find(|h| house_number_is(h, root))
        .unwrap_or(&base_houses[0])
        .as_object()
        .cloned()
        .unwrap_or_default();
    let base_name = match base.get("user_name") {
        Some(v) if truthy(Some(v)) => v.clone(),
        _ => json!("Radix Chart"),
    };
    let name = match custom_name {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => format!(
            "{} [Derived H{root} - {theme_en}]",
            base_name
                .as_str()
                .map_or_else(|| base_name.to_string(), str::to_string)
        ),
    };
    let meanings_map: Map<String, Value> = meanings
        .iter()
        .enumerate()
        .map(|(i, m)| ((i + 1).to_string(), json!({"en": m.en, "it": m.it})))
        .collect();
    let at = |key: &str| root_cusp.get(key).cloned().unwrap_or(Value::Null);

    let derived_data = json!({
        "derived_root_house": root,
        "derived_from_id": get(base, "id", Value::Null),
        "derived_from_name": base_name.clone(),
        "root_house_theme": {"name_en": theme_en, "name_it": theme_it},
        "root_cusp": {
            "house_number": root,
            "sign": at("sign"),
            "degree": at("degree"),
            "minute": at("minute"),
            "ecliptic_longitude": at("ecliptic_longitude"),
        },
        "meanings": meanings_map,
    });

    let mut out = Map::new();
    let mut put = |k: &str, v: Value| {
        out.insert(k.to_string(), v);
    };
    put("id", Value::Null);
    put("chart_type", json!("DERIVED"));
    put("user_name", json!(name));
    put("birth_datetime", get(base, "birth_datetime", Value::Null));
    put("local_datetime", get(base, "local_datetime", Value::Null));
    put("timezone", get(base, "timezone", json!("UTC")));
    put(
        "timezone_offset",
        get(base, "timezone_offset", json!("+00:00")),
    );
    put("timezone_abbr", get(base, "timezone_abbr", json!("UTC")));
    put("is_dst", get(base, "is_dst", json!(false)));
    put("dst_offset", get(base, "dst_offset", json!("+00:00")));
    put("raw_offset", get(base, "raw_offset", json!("+00:00")));
    put("house_system", get(base, "house_system", json!("P")));
    put("zodiac_type", get(base, "zodiac_type", json!("tropical")));
    put(
        "zodiac_type_label",
        get(base, "zodiac_type_label", json!("Tropical")),
    );
    put("ayanamsa", get(base, "ayanamsa", Value::Null));
    put("ayanamsa_name", get(base, "ayanamsa_name", Value::Null));
    put("ayanamsa_value", get(base, "ayanamsa_value", Value::Null));
    put(
        "ayanamsa_formatted",
        get(base, "ayanamsa_formatted", json!("")),
    );
    put("orb_settings", get(base, "orb_settings", json!({})));
    put("coordinates", get(base, "coordinates", json!({})));
    put("planets", Value::Array(planets));
    put(
        "houses",
        Value::Array(derived_houses.into_iter().map(Value::Object).collect()),
    );
    put("aspects", get(base, "aspects", json!([])));
    put("fixed_stars", Value::Array(fixed_stars));
    put("arabic_parts", Value::Array(lots));
    put("temperament", get(base, "temperament", json!({})));
    put("lunar_status", get(base, "lunar_status", json!({})));
    put("derived_from_id", get(base, "id", Value::Null));
    put("derived_from_name", base_name);
    put("derived_house", json!(root));
    put("derived_data", derived_data);
    Ok(Value::Object(out))
}
