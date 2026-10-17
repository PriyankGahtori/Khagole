//! Vedic Divisional Charts (Vargas D1 to D60).

use serde::{Deserialize, Serialize};
use crate::error::EngineError;

/// Position within a divisional chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivisionalPosition {
    /// 1-based sign number (1 = Aries .. 12 = Pisces).
    pub sign: u8,
    /// Degrees within the divisional sign (0.0° .. 30.0°).
    pub degree: f64,
    /// Absolute longitude within the divisional zodiac (0.0° .. 360.0°).
    pub longitude: f64,
}

/// Planet placement in a divisional chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivisionalPlanet {
    /// Planet name (e.g. "Sun", "Moon", "Mars").
    pub name: String,
    /// 1-based sign number (1..12).
    pub sign: u8,
    /// Degrees within the sign (0.0° .. 30.0°).
    pub degree: f64,
    /// Absolute longitude (0.0° .. 360.0°).
    pub longitude: f64,
    /// 1-based house reckoned from divisional lagna (1..12).
    pub house: u8,
}

/// A complete divisional chart with transformed lagna and planets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivisionalChart {
    /// The varga code (e.g. "D1", "D9", "D60").
    pub varga: String,
    /// Ascendant placement in this varga.
    pub lagna: DivisionalPosition,
    /// Transformed planet placements in this varga.
    pub planets: Vec<DivisionalPlanet>,
}

#[inline]
fn deg_in_sign(lon: f64) -> f64 {
    ((lon % 30.0) + 30.0) % 30.0
}

#[inline]
fn sign_of(lon: f64) -> u8 {
    let norm = ((lon % 360.0) + 360.0) % 360.0;
    (norm / 30.0).floor() as u8 + 1
}

#[inline]
fn norm_sign(s: i32) -> u8 {
    (((s - 1).rem_euclid(12)) + 1) as u8
}

#[inline]
fn part(lon: f64, n: u32) -> u32 {
    let p = ((deg_in_sign(lon) * n as f64) / 30.0).floor() as u32;
    p.min(n - 1)
}

#[inline]
fn div_deg(lon: f64, n: u32) -> f64 {
    (deg_in_sign(lon) * n as f64) % 30.0
}

/// Cyclic Parivritti formula: `((sign - 1) * n + part) % 12 + 1`.
pub fn parivritti(lon: f64, n: u32) -> (u8, f64) {
    let sign = sign_of(lon) as i32;
    let l = part(lon, n) as i32;
    let d_sign = ((sign - 1) * n as i32 + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, n))
}

fn hora_udayashakti(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon);
    let d = deg_in_sign(lon);
    let first = d < 15.0;
    let (d_sign, degree) = if sign % 2 == 1 {
        let s = if first { 2 * sign as i32 - 1 } else { 2 * sign as i32 };
        let deg = if first { d * 2.0 } else { (d - 15.0) * 2.0 };
        (s, deg)
    } else {
        let s = if first { 2 * sign as i32 } else { 2 * sign as i32 - 1 };
        let mut deg = if first { 30.0 - d * 2.0 } else { 30.0 - (d - 15.0) * 2.0 };
        if deg >= 30.0 {
            deg = 0.0;
        }
        (s, deg)
    };
    (norm_sign(d_sign), degree)
}

fn d3_drekkana(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as i32;
    let l = part(lon, 3) as i32;
    let d_sign = ((sign - 1) + l * 4).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 3))
}

fn d4_chaturthamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as i32;
    let l = part(lon, 4) as i32;
    let d_sign = ((sign - 1) + l * 3).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 4))
}

fn d5_panchamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon);
    let l = part(lon, 5) as usize;
    let odd = [1, 11, 9, 3, 7];
    let even = [2, 6, 12, 10, 8];
    let d_sign = if sign % 2 == 1 { odd[l] } else { even[l] };
    (d_sign, div_deg(lon, 5))
}

fn d7_saptamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as i32;
    let l = part(lon, 7) as i32;
    let offset = if sign % 2 == 0 { 6 } else { 0 };
    let d_sign = ((sign - 1) + offset + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 7))
}

fn d9_navamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as usize;
    let l = part(lon, 9) as i32;
    let seeds = [0, 9, 6, 3, 0, 9, 6, 3, 0, 9, 6, 3];
    let d_sign = (seeds[sign - 1] + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 9))
}

fn d10_dashamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as i32;
    let l = part(lon, 10) as i32;
    let offset = if sign % 2 == 0 { 8 } else { 0 };
    let d_sign = ((sign - 1) + offset + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 10))
}

fn d12_dwadashamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as i32;
    let l = part(lon, 12) as i32;
    let d_sign = ((sign - 1) + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 12))
}

fn d16_shodashamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as i32;
    let l = part(lon, 16) as i32;
    let seed = ((sign - 1) % 3) * 4;
    let d_sign = (seed + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 16))
}

fn d20_vimshamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as usize;
    let l = part(lon, 20) as i32;
    let seeds = [0, 8, 4, 0, 8, 4, 0, 8, 4, 0, 8, 4];
    let d_sign = (seeds[sign - 1] + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 20))
}

fn d24_siddhamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon);
    let l = part(lon, 24) as i32;
    let seed = if sign % 2 == 1 { 4 } else { 3 };
    let d_sign = (seed + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 24))
}

fn d27_nakshatramsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as i32;
    let l = part(lon, 27) as i32;
    let seed = ((sign - 1) % 4) * 3;
    let d_sign = (seed + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 27))
}

fn d30_trimshamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon);
    let d = deg_in_sign(lon);
    let d_sign = if sign % 2 == 1 {
        if d <= 5.0 { 1 }
        else if d <= 10.0 { 11 }
        else if d <= 18.0 { 9 }
        else if d <= 25.0 { 3 }
        else { 7 }
    } else {
        if d <= 5.0 { 2 }
        else if d <= 12.0 { 6 }
        else if d <= 20.0 { 12 }
        else if d <= 25.0 { 10 }
        else { 8 }
    };
    (d_sign, div_deg(lon, 30))
}

fn d40_khavedamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon);
    let l = part(lon, 40) as i32;
    let seed = if sign % 2 == 1 { 0 } else { 6 };
    let d_sign = (seed + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 40))
}

fn d45_akshavedamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as i32;
    let l = part(lon, 45) as i32;
    let seed = ((sign - 1) % 3) * 4;
    let d_sign = (seed + l).rem_euclid(12) + 1;
    (d_sign as u8, div_deg(lon, 45))
}

fn d60_shashtyamsha(lon: f64) -> (u8, f64) {
    let sign = sign_of(lon) as i32;
    let l = part(lon, 60) as i32;
    (norm_sign(sign + l), div_deg(lon, 60))
}

/// Transform an absolute sidereal longitude into its divisional position for a given varga.
pub fn calc_divisional_point(lon: f64, varga: &str) -> Result<DivisionalPosition, EngineError> {
    let (sign, degree) = match varga {
        "D1" | "Rashi" => (sign_of(lon), deg_in_sign(lon)),
        "D2" | "Hora" => hora_udayashakti(lon),
        "D3" | "Drekkana" => d3_drekkana(lon),
        "D4" | "Chaturthamsha" => d4_chaturthamsha(lon),
        "D5" | "Panchamsha" => d5_panchamsha(lon),
        "D6" | "Shashthamsha" => parivritti(lon, 6),
        "D7" | "Saptamsha" => d7_saptamsha(lon),
        "D8" | "Ashtamsha" => parivritti(lon, 8),
        "D9" | "Navamsha" => d9_navamsha(lon),
        "D10" | "Dashamsha" => d10_dashamsha(lon),
        "D11" | "Rudramsha" => parivritti(lon, 11),
        "D12" | "Dwadashamsha" => d12_dwadashamsha(lon),
        "D16" | "Shodashamsha" => d16_shodashamsha(lon),
        "D20" | "Vimshamsha" => d20_vimshamsha(lon),
        "D24" | "Siddhamsha" => d24_siddhamsha(lon),
        "D27" | "Nakshatramsha" => d27_nakshatramsha(lon),
        "D30" | "Trimshamsha" => d30_trimshamsha(lon),
        "D40" | "Khavedamsha" => d40_khavedamsha(lon),
        "D45" | "Akshavedamsha" => d45_akshavedamsha(lon),
        "D60" | "Shashtyamsha" => d60_shashtyamsha(lon),
        other => {
            if let Some(n_str) = other.strip_prefix('D') {
                if let Ok(n) = n_str.parse::<u32>() {
                    if n > 0 {
                        parivritti(lon, n)
                    } else {
                        return Err(EngineError::InvalidInput(format!("invalid varga factor: {other}")));
                    }
                } else {
                    return Err(EngineError::InvalidInput(format!("invalid varga format: {other}")));
                }
            } else {
                return Err(EngineError::InvalidInput(format!("unknown divisional varga: {other}")));
            }
        }
    };

    let longitude = (sign as f64 - 1.0) * 30.0 + degree;
    Ok(DivisionalPosition {
        sign,
        degree,
        longitude,
    })
}

/// Transform a chart (represented as named planets with sidereal longitudes and lagna) into a divisional chart.
pub fn calc_divisional_chart(
    planets: &[(String, f64)],
    lagna_lon: f64,
    varga: &str,
) -> Result<DivisionalChart, EngineError> {
    let lagna_pos = calc_divisional_point(lagna_lon, varga)?;
    let mut div_planets = Vec::with_capacity(planets.len());

    for (name, lon) in planets {
        let p_pos = calc_divisional_point(*lon, varga)?;
        let house = (((p_pos.sign as i32 - lagna_pos.sign as i32).rem_euclid(12)) + 1) as u8;
        div_planets.push(DivisionalPlanet {
            name: name.clone(),
            sign: p_pos.sign,
            degree: p_pos.degree,
            longitude: p_pos.longitude,
            house,
        });
    }

    Ok(DivisionalChart {
        varga: varga.to_string(),
        lagna: lagna_pos,
        planets: div_planets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divisional_d9() {
        // 0° Aries -> Aries (sign 1)
        let p0 = calc_divisional_point(0.0, "D9").unwrap();
        assert_eq!(p0.sign, 1);
        assert!((p0.degree - 0.0).abs() < 1e-6);

        // 15° Taurus -> Taurus sign 2, part = floor(15 * 9 / 30) = 4.
        // Earth seed = Cap (10). (9 + 4) % 12 + 1 = 2 (Taurus)
        let p_tau = calc_divisional_point(45.0, "D9").unwrap();
        assert_eq!(p_tau.sign, 2);
    }

    #[test]
    fn test_divisional_d2_hora() {
        // Aries 5° (first half of odd sign) -> 2*1 - 1 = 1 (Aries)
        let p = calc_divisional_point(5.0, "D2").unwrap();
        assert_eq!(p.sign, 1);
    }

    #[test]
    fn test_divisional_d30() {
        // Aries 4° (odd sign, <= 5°) -> Mars sign 1 (Aries)
        let p = calc_divisional_point(4.0, "D30").unwrap();
        assert_eq!(p.sign, 1);
    }

    #[test]
    fn test_divisional_chart_houses() {
        let lagna_lon = 230.0; // Scorpio
        let planets = vec![
            ("Sun".to_string(), 150.0), // Virgo
            ("Moon".to_string(), 230.0), // Scorpio
        ];
        let chart = calc_divisional_chart(&planets, lagna_lon, "D9").unwrap();
        assert_eq!(chart.varga, "D9");
        assert_eq!(chart.planets.len(), 2);
        // Moon has same lon as lagna -> must be in House 1
        assert_eq!(chart.planets[1].house, 1);
    }
}
