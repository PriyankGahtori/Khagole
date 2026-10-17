//! Physical and astronomical constants, with the values used by the reference
//! implementation (Skyfield 1.55).

use std::f64::consts::PI;

/// J2000.0 epoch as a Julian date.
pub const T0: f64 = 2_451_545.0;
pub const DAY_S: f64 = 86_400.0;
pub const TAU: f64 = 2.0 * PI;
pub const ASEC2RAD: f64 = 4.848_136_811_095_36e-6;
pub const ASEC360: f64 = 1_296_000.0;
/// Astronomical unit (IAU 2012 Resolution B2).
pub const AU_M: f64 = 149_597_870_700.0;
pub const AU_KM: f64 = 149_597_870.700;
/// Speed of light, m/s.
pub const C: f64 = 299_792_458.0;
/// Speed of light, au/day.
pub const C_AUDAY: f64 = C * DAY_S / AU_M;
/// Heliocentric gravitational constant, m³/s².
pub const GS: f64 = 1.327_124_400_179_87e20;
