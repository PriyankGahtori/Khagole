use crate::constants::{ASEC2RAD, T0};

use super::Mat3;

/// IAU 2006 precession matrix for a TDB Julian date (Skyfield `compute_precession`).
pub fn compute_precession(jd_tdb: f64) -> Mat3 {
    let eps0 = 84381.406;
    let t = (jd_tdb - T0) / 36525.0;
    let psia = ((((-0.0000000951 * t + 0.000132851) * t - 0.00114045) * t - 1.0790069) * t
        + 5038.481507)
        * t;
    let omegaa =
        ((((0.0000003337 * t - 0.000000467) * t - 0.00772503) * t + 0.0512623) * t - 0.025754) * t
            + eps0;
    let chia = ((((-0.0000000560 * t + 0.000170663) * t - 0.00121197) * t - 2.3814292) * t
        + 10.556403)
        * t;
    let (sa, ca) = (eps0 * ASEC2RAD).sin_cos();
    let (sb, cb) = (-psia * ASEC2RAD).sin_cos();
    let (sc, cc) = (-omegaa * ASEC2RAD).sin_cos();
    let (sd, cd) = (chia * ASEC2RAD).sin_cos();
    [
        [
            cd * cb - sb * sd * cc,
            cd * sb * ca + sd * cc * cb * ca - sa * sd * sc,
            cd * sb * sa + sd * cc * cb * sa + ca * sd * sc,
        ],
        [
            -sd * cb - sb * cd * cc,
            -sd * sb * ca + cd * cc * cb * ca - sa * cd * sc,
            -sd * sb * sa + cd * cc * cb * sa + ca * cd * sc,
        ],
        [sb * sc, -sc * cb * ca - sa * cc, -sc * cb * sa + cc * ca],
    ]
}
