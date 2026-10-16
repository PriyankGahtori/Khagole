//! Astroceleste MCP (Model Context Protocol) Server
//!
//! Provides high-precision NASA JPL DE440 ephemeris calculations, Vedic astrological charts,
//! houses (including Sripati), true/mean lunar nodes, ayanamsas, and transits over JSON-RPC stdio.

use std::env;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use astroceleste_engine::ephemeris::{Kernel, KernelSet, Spk};
use astroceleste_engine::{
    ayanamsa, ayanamsa_info, calculate_chart, calculate_transit_chart, chinese_calendar,
    ChartRequest, UtcInstant, DEFAULT_AYANAMSA,
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
