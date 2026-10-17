//! Lunar phase, illumination, speed, dignity and mansion (`charts/calc/lunar.py`).

use serde::Serialize;

use crate::catalog::{LunarMansion, LUNAR_MANSIONS};
use crate::chart::Placement;
use crate::pyfloat;

/// Lunar phase, speed, dignity and mansion.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LunarStatus {
    /// Phase identifier, e.g. "waxing_gibbous".
    pub phase_key: &'static str,
    /// Phase name, e.g. "Waxing Gibbous".
    pub phase_name: &'static str,
    /// Quarter (1-4) the phase belongs to.
    pub phase_quarter: u8,
    /// Phase emoji, e.g. "🌔".
    pub glyph: &'static str,
    /// Moon's longitude minus the Sun's, degrees [0, 360).
    pub elongation: f64,
    /// Illuminated fraction of the disc, percent.
    pub illumination_percentage: f64,
    /// Days since the new Moon (from the elongation and the mean synodic month).
    pub moon_age_days: f64,
    /// Whether the elongation is below 180°.
    pub is_waxing: bool,
    /// Sign of the Moon.
    pub moon_sign: &'static str,
    /// Whole degrees of the Moon within its sign.
    pub moon_degree: i64,
    /// Arc minutes past `moon_degree`.
    pub moon_minute: i64,
    /// Moon's ecliptic longitude in degrees.
    pub moon_longitude: f64,
    /// House (1-12) of the Moon.
    pub moon_house: u8,
    /// Moon's speed, degrees per day.
    pub moon_speed: f64,
    /// "swift" (> 13.5°/day), "slow" (< 12.5°/day) or "average".
    pub speed_status: &'static str,
    /// "Domicile", "Exaltation", "Detriment", "Fall" or "Peregrine".
    pub essential_dignity: &'static str,
    /// Lunar mansion the Moon is in.
    pub lunar_mansion: LunarMansion,
}

/// Lunar status from the chart's planets; `None` when there is no Moon.
pub fn lunar_status(planets: &[Placement]) -> Option<LunarStatus> {
    let moon = planets.iter().find(|p| p.name == "Moon")?;
    let sun_lon = planets
        .iter()
        .find(|p| p.name == "Sun")
        .map_or(0.0, |s| s.ecliptic_longitude);
    let moon_lon = moon.ecliptic_longitude;
    let diff = pyfloat::rem(moon_lon - sun_lon, 360.0);

    let (phase_key, phase_name, phase_quarter, glyph) = if !(22.5..337.5).contains(&diff) {
        ("new_moon", "New Moon", 1, "🌑")
    } else if diff < 67.5 {
        ("waxing_crescent", "Waxing Crescent", 1, "🌒")
    } else if diff < 112.5 {
        ("first_quarter", "First Quarter", 2, "🌓")
    } else if diff < 157.5 {
        ("waxing_gibbous", "Waxing Gibbous", 2, "🌔")
    } else if diff < 202.5 {
        ("full_moon", "Full Moon", 3, "🌕")
    } else if diff < 247.5 {
        ("waning_gibbous", "Waning Gibbous", 3, "🌖")
    } else if diff < 292.5 {
        ("third_quarter", "Third Quarter", 4, "🌗")
    } else {
        ("waning_crescent", "Waning Crescent", 4, "🌘")
    };

    let speed = moon.speed;
    let mansion_index = ((moon_lon / (360.0 / 28.0)).floor() as i64).rem_euclid(28) as usize;
    Some(LunarStatus {
        phase_key,
        phase_name,
        phase_quarter,
        glyph,
        elongation: pyfloat::round(diff, 2),
        illumination_percentage: pyfloat::round(((1.0 - diff.to_radians().cos()) / 2.0) * 100.0, 1),
        moon_age_days: pyfloat::round((diff / 360.0) * 29.530588853, 1),
        is_waxing: diff < 180.0,
        moon_sign: moon.sign,
        moon_degree: moon.degree,
        moon_minute: moon.minute,
        moon_longitude: pyfloat::round(moon_lon, 3),
        moon_house: moon.house,
        moon_speed: pyfloat::round(speed, 2),
        speed_status: if speed > 13.5 {
            "swift"
        } else if speed < 12.5 {
            "slow"
        } else {
            "average"
        },
        essential_dignity: match moon.sign {
            "Cancer" => "Domicile",
            "Taurus" => "Exaltation",
            "Capricorn" => "Detriment",
            "Scorpio" => "Fall",
            _ => "Peregrine",
        },
        lunar_mansion: LUNAR_MANSIONS[mansion_index],
    })
}
