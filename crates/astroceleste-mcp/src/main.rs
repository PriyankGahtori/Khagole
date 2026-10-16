//! Astroceleste MCP (Model Context Protocol) Server
//!
//! Provides high-precision NASA JPL DE440 ephemeris calculations, Vedic astrological charts,
//! houses (including Sripati), true/mean lunar nodes, ayanamsas, and transits over JSON-RPC stdio.

use std::env;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use astroceleste_engine::ephemeris::{Kernel, KernelSet, Spk};
use astroceleste_engine::{
    ayanamsa, ayanamsa_info, calc_divisional_chart, calc_panchang, calc_vimshottari_dasha,
    calculate_chart, calculate_transit_chart, chinese_calendar, ChartRequest, UtcInstant,
    YearType, DEFAULT_AYANAMSA,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn locate_kernel() -> Option<PathBuf> {
    if let Ok(path) = env::var("ASTROCELESTE_KERNEL_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    let candidates = [
        "kernels/de440s.bsp",
        "../kernels/de440s.bsp",
        "../../kernels/de440s.bsp",
        "/Users/priyankgahtori/Code/poc/astroceleste-engine/kernels/de440s.bsp",
    ];

    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn build_kernel_set(path: &Path) -> Result<KernelSet, String> {
    let spk = Spk::open(path).map_err(|e| format!("failed to open SPK kernel {path:?}: {e}"))?;
    let kernel = Kernel::new(
        path.file_name().unwrap_or_default().to_string_lossy(),
        spk,
    )
    .map_err(|e| format!("invalid kernel {path:?}: {e}"))?;
    let mut set = KernelSet::new();
    set.push(kernel);
    Ok(set)
}

fn tools_list() -> Value {
    json!([
        {
            "name": "calculate_chart",
            "description": "Calculate high-precision astrological chart positions (Vedic or Western) with planetary coordinates, houses (including Sripati), speeds, retrogrades, and ayanamsas using NASA JPL DE440 ephemeris.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "utc": {
                        "type": "string",
                        "description": "UTC date and time in ISO 8601 format (e.g. '1950-09-17T04:04:00Z' or '2026-10-16T12:00:00Z')"
                    },
                    "latitude": {
                        "type": "number",
                        "description": "Geographic latitude in degrees (north is positive, south is negative)"
                    },
                    "longitude": {
                        "type": "number",
                        "description": "Geographic longitude in degrees (east is positive, west is negative)"
                    },
                    "house_system": {
                        "type": "string",
                        "description": "House system code: 'S' (Sripati / Bhava Madhya - classical Vedic), 'P' (Placidus), 'W' (Whole Sign), 'K' (Koch), 'E' (Equal), 'C' (Campanus), 'R' (Regiomontanus). Default: 'S' for Vedic or 'P' for Western.",
                        "default": "S"
                    },
                    "zodiac_type": {
                        "type": "string",
                        "enum": ["sidereal", "tropical"],
                        "description": "Zodiac coordinate system: 'sidereal' (Vedic) or 'tropical' (Western). Default: 'sidereal'.",
                        "default": "sidereal"
                    },
                    "ayanamsa": {
                        "type": "string",
                        "description": "Ayanamsa system for sidereal zodiac: 'lahiri' (Chitra Paksha), 'raman', 'krishnamurti', 'fagan_bradley', 'yukteshwar', 'true_citra'. Default: 'lahiri'.",
                        "default": "lahiri"
                    },
                    "position_mode": {
                        "type": "string",
                        "enum": ["true", "apparent"],
                        "description": "Calculation mode: 'true' (Geometric / True Positions without light-time, as standard in Vedic JHora) or 'apparent' (with light-time & aberration, standard in Western). Default: 'true'.",
                        "default": "true"
                    },
                    "node_type": {
                        "type": "string",
                        "enum": ["mean", "true"],
                        "description": "Lunar node calculation mode: 'mean' (IAU mean mathematical node) or 'true' (instantaneous osculating true node matching Swiss Ephemeris SE_TRUE_NODE). Default: 'mean'.",
                        "default": "mean"
                    }
                },
                "required": ["utc", "latitude", "longitude"]
            }
        },
        {
            "name": "get_ayanamsa",
            "description": "Get the precise ayanamsa value (precession shift between tropical and sidereal zodiac) for any date.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "utc": {
                        "type": "string",
                        "description": "UTC date and time in ISO 8601 format"
                    },
                    "ayanamsa": {
                        "type": "string",
                        "description": "Ayanamsa code (e.g. 'lahiri', 'raman', 'krishnamurti'). Default: 'lahiri'.",
                        "default": "lahiri"
                    }
                },
                "required": ["utc"]
            }
        },
        {
            "name": "calculate_transits",
            "description": "Calculate transit planetary positions and aspects to natal chart planets for any moment.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "utc": {
                        "type": "string",
                        "description": "Transit UTC date and time in ISO 8601 format"
                    },
                    "latitude": {
                        "type": "number",
                        "description": "Geographic latitude in degrees"
                    },
                    "longitude": {
                        "type": "number",
                        "description": "Geographic longitude in degrees"
                    },
                    "natal_planets": {
                        "type": "array",
                        "description": "Array of natal planets from a previous calculate_chart result"
                    },
                    "zodiac_type": {
                        "type": "string",
                        "default": "sidereal"
                    },
                    "ayanamsa": {
                        "type": "string",
                        "default": "lahiri"
                    }
                },
                "required": ["utc", "latitude", "longitude", "natal_planets"]
            }
        },
        {
            "name": "get_chinese_calendar",
            "description": "Compute Chinese calendar solar terms, lunar months, and equation of time.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "utc": {
                        "type": "string",
                        "description": "UTC date and time in ISO 8601 format"
                    }
                },
                "required": ["utc"]
            }
        },
        {
            "name": "calculate_divisional_chart",
            "description": "Calculate a Vedic divisional chart (varga D1 to D60: D1 Rasi, D2 Hora, D3 Drekkana, D4 Chaturthamsha, D7 Saptamsha, D9 Navamsha, D10 Dashamsha, D12 Dwadashamsha, D16 Shodashamsha, D20 Vimshamsha, D24 Siddhamsha, D27 Nakshatramsha, D30 Trimshamsha, D40 Khavedamsha, D45 Akshavedamsha, D60 Shashtyamsha).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "varga": {
                        "type": "string",
                        "description": "Divisional chart code (e.g. 'D1', 'D2', 'D3', 'D7', 'D9', 'D10', 'D12', 'D16', 'D20', 'D24', 'D27', 'D30', 'D60')",
                        "default": "D9"
                    },
                    "chart": {
                        "type": "object",
                        "description": "A full chart object as returned by calculate_chart (containing planets and houses/lagna)"
                    },
                    "planets": {
                        "type": "array",
                        "description": "Array of planets, each an object with 'name' and 'longitude' (or 'ecliptic_longitude')"
                    },
                    "lagna_longitude": {
                        "type": "number",
                        "description": "Lagna (Ascendant) longitude in degrees [0, 360). Optional if chart is provided."
                    }
                },
                "required": ["varga"]
            }
        },
        {
            "name": "calculate_vimshottari_dasha",
            "description": "Calculate full hierarchical Vimshottari Dasha periods (Mahadashas, Antardashas, Pratyantardashas) with balance of dasha at birth.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "moon_longitude": {
                        "type": "number",
                        "description": "Sidereal longitude of the Moon in degrees [0, 360)"
                    },
                    "utc": {
                        "type": "string",
                        "description": "Birth date and time in UTC ISO 8601 format (e.g. '1950-09-17T04:04:00Z')"
                    },
                    "birth_jd": {
                        "type": "number",
                        "description": "Birth instant as Julian Day. Optional if utc is provided."
                    },
                    "depth": {
                        "type": "integer",
                        "description": "Hierarchy depth: 1 (Mahadasha), 2 (Antardasha), 3 (Pratyantardasha), 4 (Sukshma). Default: 3",
                        "default": 3
                    },
                    "year_type": {
                        "type": "string",
                        "enum": ["sidereal", "savana", "tropical", "julian"],
                        "description": "Year length definition: 'sidereal' (365.256363d - default), 'savana' (360d), 'tropical' (365.24219d), 'julian' (365.25d)",
                        "default": "sidereal"
                    }
                },
                "required": ["moon_longitude"]
            }
        },
        {
            "name": "calculate_panchanga",
            "description": "Calculate the five limbs of the Vedic calendar (Tithi, Vara, Nakshatra, Yoga, Karana) and Hindu Samvatsara eras for any moment.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sun_longitude": {
                        "type": "number",
                        "description": "Sun longitude in degrees [0, 360)"
                    },
                    "moon_longitude": {
                        "type": "number",
                        "description": "Moon longitude in degrees [0, 360)"
                    },
                    "utc": {
                        "type": "string",
                        "description": "Moment in UTC ISO 8601 format (e.g. '1950-09-17T04:04:00Z')"
                    },
                    "jd": {
                        "type": "number",
                        "description": "Moment as Julian Day. Optional if utc is provided."
                    },
                    "weekday": {
                        "type": "integer",
                        "description": "Local civil weekday (0 = Sunday .. 6 = Saturday). Computed from JD if omitted."
                    }
                },
                "required": ["sun_longitude", "moon_longitude"]
            }
        }
    ])
}

fn handle_tool_call(kernels: &KernelSet, name: &str, arguments: &Value) -> Result<Value, String> {
    match name {
        "calculate_chart" => {
            let utc = arguments["utc"].as_str().ok_or("missing 'utc' parameter")?;
            let lat = arguments["latitude"].as_f64().ok_or("missing 'latitude' parameter")?;
            let lon = arguments["longitude"].as_f64().ok_or("missing 'longitude' parameter")?;
            let house_system = arguments["house_system"].as_str().unwrap_or("S");
            let zodiac_type = arguments["zodiac_type"].as_str().unwrap_or("sidereal");
            let ayanamsa_code = arguments["ayanamsa"].as_str().unwrap_or("lahiri");
            let position_mode = arguments["position_mode"].as_str().unwrap_or("true");
            let node_type = arguments["node_type"].as_str().unwrap_or("mean");

            let instant = UtcInstant::parse(utc).map_err(|e| e.to_string())?;
            let req = ChartRequest {
                instant,
                latitude: lat,
                longitude: lon,
                house_system,
                zodiac_type,
                ayanamsa: ayanamsa_code,
                orb_settings: None,
                dignity_scheme: "lilly",
                position_mode,
                node_type,
                chinese_calendar: false,
            };

            let chart = calculate_chart(kernels, &req).map_err(|e| e.to_string())?;
            let chart_json = serde_json::to_value(&chart).map_err(|e| e.to_string())?;
            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&chart_json).unwrap_or_default()
                    }
                ]
            }))
        }
        "get_ayanamsa" => {
            let utc = arguments["utc"].as_str().ok_or("missing 'utc' parameter")?;
            let ayanamsa_name = arguments["ayanamsa"].as_str().unwrap_or(DEFAULT_AYANAMSA);
            let instant = UtcInstant::parse(utc).map_err(|e| e.to_string())?;
            let jd = instant.julian_day();
            let code = ayanamsa(ayanamsa_name).code;
            let info = ayanamsa_info(jd, code);
            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&info).unwrap_or_default()
                    }
                ]
            }))
        }
        "calculate_transits" => {
            let utc = arguments["utc"].as_str().ok_or("missing 'utc' parameter")?;
            let lat = arguments["latitude"].as_f64().ok_or("missing 'latitude' parameter")?;
            let lon = arguments["longitude"].as_f64().ok_or("missing 'longitude' parameter")?;
            let natal = &arguments["natal_planets"];
            let zodiac_type = arguments["zodiac_type"].as_str().unwrap_or("sidereal");
            let ayanamsa_code = arguments["ayanamsa"].as_str().unwrap_or("lahiri");

            let instant = UtcInstant::parse(utc).map_err(|e| e.to_string())?;
            let req = ChartRequest {
                instant,
                latitude: lat,
                longitude: lon,
                house_system: "S",
                zodiac_type,
                ayanamsa: ayanamsa_code,
                orb_settings: None,
                dignity_scheme: "lilly",
                position_mode: "true",
                node_type: "mean",
                chinese_calendar: false,
            };

            let transit = calculate_transit_chart(kernels, natal, &req).map_err(|e| e.to_string())?;
            let transit_json = serde_json::to_value(&transit).map_err(|e| e.to_string())?;
            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&transit_json).unwrap_or_default()
                    }
                ]
            }))
        }
        "get_chinese_calendar" => {
            let utc = arguments["utc"].as_str().ok_or("missing 'utc' parameter")?;
            let instant = UtcInstant::parse(utc).map_err(|e| e.to_string())?;
            let cal = chinese_calendar(kernels, instant).map_err(|e| e.to_string())?;
            let cal_json = serde_json::to_value(&cal).map_err(|e| e.to_string())?;
            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&cal_json).unwrap_or_default()
                    }
                ]
            }))
        }
        "calculate_divisional_chart" => {
            let varga = arguments["varga"].as_str().unwrap_or("D9");
            let mut planets_vec: Vec<(String, f64)> = Vec::new();
            let mut lagna_lon = arguments["lagna_longitude"].as_f64();

            if let Some(chart_obj) = arguments.get("chart") {
                if lagna_lon.is_none() {
                    if let Some(lagna_val) = chart_obj.get("lagna") {
                        lagna_lon = lagna_val.get("lon").or_else(|| lagna_val.get("ecliptic_longitude")).and_then(|v| v.as_f64());
                    }
                }
                if let Some(planets_arr) = chart_obj.get("planets").and_then(|p| p.as_array()) {
                    for p in planets_arr {
                        let name = p["name"].as_str().unwrap_or("").to_string();
                        let lon = p["ecliptic_longitude"]
                            .as_f64()
                            .or_else(|| p["longitude"].as_f64())
                            .or_else(|| p["lon"].as_f64())
                            .unwrap_or(0.0);
                        if name == "Ascendant" && lagna_lon.is_none() {
                            lagna_lon = Some(lon);
                        }
                        if !name.is_empty() {
                            planets_vec.push((name, lon));
                        }
                    }
                }
            } else if let Some(planets_arr) = arguments.get("planets").and_then(|p| p.as_array()) {
                for p in planets_arr {
                    let name = p["name"].as_str().unwrap_or("").to_string();
                    let lon = p["ecliptic_longitude"]
                        .as_f64()
                        .or_else(|| p["longitude"].as_f64())
                        .or_else(|| p["lon"].as_f64())
                        .unwrap_or(0.0);
                    if name == "Ascendant" && lagna_lon.is_none() {
                        lagna_lon = Some(lon);
                    }
                    if !name.is_empty() {
                        planets_vec.push((name, lon));
                    }
                }
            }

            let lagna = lagna_lon.unwrap_or(0.0);
            let div_chart = calc_divisional_chart(&planets_vec, lagna, varga).map_err(|e| e.to_string())?;
            let div_json = serde_json::to_value(&div_chart).map_err(|e| e.to_string())?;
            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&div_json).unwrap_or_default()
                    }
                ]
            }))
        }
        "calculate_vimshottari_dasha" => {
            let moon_lon = arguments["moon_longitude"].as_f64().ok_or("missing 'moon_longitude' parameter")?;
            let birth_jd = if let Some(jd) = arguments["birth_jd"].as_f64() {
                jd
            } else if let Some(utc) = arguments["utc"].as_str() {
                let instant = UtcInstant::parse(utc).map_err(|e| e.to_string())?;
                instant.julian_day()
            } else {
                return Err("missing 'utc' or 'birth_jd' parameter".to_string());
            };

            let depth = arguments["depth"].as_u64().unwrap_or(3) as u32;
            let year_type = match arguments["year_type"].as_str() {
                Some("savana") => YearType::Savana,
                Some("tropical") => YearType::Tropical,
                Some("julian") => YearType::Julian,
                Some("sidereal") => YearType::Sidereal,
                _ => YearType::Tssy,
            };

            let tree = calc_vimshottari_dasha(moon_lon, birth_jd, depth, year_type);
            let dasha_json = serde_json::to_value(&tree).map_err(|e| e.to_string())?;
            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&dasha_json).unwrap_or_default()
                    }
                ]
            }))
        }
        "calculate_panchanga" => {
            let sun_lon = arguments["sun_longitude"].as_f64().ok_or("missing 'sun_longitude' parameter")?;
            let moon_lon = arguments["moon_longitude"].as_f64().ok_or("missing 'moon_longitude' parameter")?;
            let jd = if let Some(jd) = arguments["jd"].as_f64() {
                jd
            } else if let Some(utc) = arguments["utc"].as_str() {
                let instant = UtcInstant::parse(utc).map_err(|e| e.to_string())?;
                instant.julian_day()
            } else {
                2451545.0
            };

            let weekday = arguments["weekday"].as_u64().map(|w| w as u8);
            let panchang = calc_panchang(sun_lon, moon_lon, jd, weekday);
            let panchang_json = serde_json::to_value(&panchang).map_err(|e| e.to_string())?;
            Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&panchang_json).unwrap_or_default()
                    }
                ]
            }))
        }
        _ => Err(format!("unknown tool: {name}")),
    }
}

fn main() {
    let kernel_path = match locate_kernel() {
        Some(p) => p,
        None => {
            eprintln!("Error: NASA JPL kernel de440s.bsp not found. Set ASTROCELESTE_KERNEL_PATH.");
            std::process::exit(1);
        }
    };

    let kernels = match build_kernel_set(&kernel_path) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Error loading kernel: {e}");
            std::process::exit(1);
        }
    };

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    let mut line = String::new();

    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(_) => {
                line.clear();
                continue;
            }
        };

        let response = match req.method.as_str() {
            "initialize" => {
                let resp = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "astroceleste-mcp",
                        "version": "0.1.0"
                    }
                });
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.unwrap_or(Value::Null),
                    result: Some(resp),
                    error: None,
                }
            }
            "notifications/initialized" => {
                // Client acknowledged initialization; no response needed for notifications.
                line.clear();
                continue;
            }
            "tools/list" => {
                let resp = json!({
                    "tools": tools_list()
                });
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.unwrap_or(Value::Null),
                    result: Some(resp),
                    error: None,
                }
            }
            "tools/call" => {
                let tool_name = req.params["name"].as_str().unwrap_or_default();
                let arguments = &req.params["arguments"];
                match handle_tool_call(&kernels, tool_name, arguments) {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id.unwrap_or(Value::Null),
                        result: Some(result),
                        error: None,
                    },
                    Err(err) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id.unwrap_or(Value::Null),
                        result: None,
                        error: Some(json!({
                            "code": -32603,
                            "message": err
                        })),
                    },
                }
            }
            "ping" => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id.unwrap_or(Value::Null),
                result: Some(json!({})),
                error: None,
            },
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id.unwrap_or(Value::Null),
                result: None,
                error: Some(json!({
                    "code": -32601,
                    "message": format!("Method not found: {}", req.method)
                })),
            },
        };

        if let Ok(serialized) = serde_json::to_string(&response) {
            let _ = writeln!(writer, "{serialized}");
            let _ = writer.flush();
        }

        line.clear();
    }
}
