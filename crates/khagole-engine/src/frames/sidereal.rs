use crate::constants::{T0, TAU};
use crate::time::Time;

use super::nutation::equation_of_the_equinoxes_complementary_terms;

/// Earth Rotation Angle in whole rotations, [0, 1) (IAU 2000 Resolution B1.8).
pub fn earth_rotation_angle(jd_ut1: f64, fraction_ut1: f64) -> f64 {
    let th = 0.7790572732640 + 0.00273781191135448 * (jd_ut1 - T0 + fraction_ut1);
    (th.rem_euclid(1.0) + jd_ut1.rem_euclid(1.0) + fraction_ut1).rem_euclid(1.0)
}

/// Greenwich Mean Sidereal Time in hours (Skyfield `sidereal_time`).
pub fn sidereal_time(t: &Time) -> f64 {
    let theta = earth_rotation_angle(t.whole, t.ut1_fraction);
    let c = (t.whole - T0 + t.tdb_fraction) / 36525.0;
    let st = 0.014506
        + ((((-0.0000000368 * c - 0.000029956) * c - 0.00000044) * c + 1.3915817) * c
            + 4612.156534)
            * c;
    (st / 54000.0 + theta * 24.0).rem_euclid(24.0)
}

/// Greenwich Apparent Sidereal Time in hours.
pub fn gast(t: &Time, gmst_hours: f64, d_psi: f64, mean_obliquity: f64) -> f64 {
    let c_terms = equation_of_the_equinoxes_complementary_terms(t.tt());
    let eq_eq = d_psi * mean_obliquity.cos() + c_terms;
    (gmst_hours + eq_eq / TAU * 24.0).rem_euclid(24.0)
}
