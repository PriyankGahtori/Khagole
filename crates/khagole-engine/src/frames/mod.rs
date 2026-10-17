//! Reference frames: ICRS → true equator and equinox of date → true ecliptic of date.
//!
//! Ported from Skyfield 1.55 (`framelib`, `precessionlib`, `nutationlib`, `earthlib`),
//! MIT licensed, see NOTICE.

pub mod nutation;
#[allow(clippy::all)]
#[rustfmt::skip]
mod nutation_data;
pub mod precession;
pub mod sidereal;

use crate::constants::ASEC2RAD;
use crate::time::Time;

pub type Vec3 = [f64; 3];
pub type Mat3 = [[f64; 3]; 3];

pub fn mxm(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut out = [[0.0; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

pub fn mxv(m: &Mat3, v: &Vec3) -> Vec3 {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

pub fn rot_x(theta: f64) -> Mat3 {
    let (s, c) = theta.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
}

pub fn rot_z(theta: f64) -> Mat3 {
    let (s, c) = theta.sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}

/// ICRS → dynamical mean equator and equinox of J2000 (IERS 2003 frame bias).
pub fn frame_bias() -> Mat3 {
    let xi0 = -0.0166170 * ASEC2RAD;
    let eta0 = -0.0068192 * ASEC2RAD;
    let da0 = -0.01460 * ASEC2RAD;
    let (yx, zx, xy, zy, xz, yz) = (-da0, xi0, da0, eta0, -xi0, -eta0);
    let xx = 1.0 - 0.5 * (yx * yx + zx * zx);
    let yy = 1.0 - 0.5 * (yx * yx + zy * zy);
    let zz = 1.0 - 0.5 * (zy * zy + zx * zx);
    [[xx, xy, xz], [yx, yy, yz], [zx, zy, zz]]
}

/// Earth orientation quantities for one instant, computed once and shared by the
/// planets, the houses and the ecliptic conversion (Skyfield caches them on `Time`).
#[derive(Debug, Clone)]
pub struct Orientation {
    pub d_psi: f64,
    pub d_eps: f64,
    pub mean_obliquity: f64,
    /// Precession matrix P.
    pub precession: Mat3,
    /// Nutation matrix N.
    pub nutation: Mat3,
    /// M = N · P · B: ICRS → true equator and equinox of date.
    pub m: Mat3,
    /// ICRS → true ecliptic and equinox of date.
    pub ecliptic: Mat3,
    pub gmst_hours: f64,
    pub gast_hours: f64,
}

impl Orientation {
    /// Full IAU 2000A nutation (Skyfield's default for positions).
    pub fn at(t: &Time) -> Self {
        Self::with_nutation(t, nutation::iau2000a_radians(t.tt()))
    }

    /// With caller-supplied nutation angles, e.g. IAU 2000B for rise/set searches.
    pub fn with_nutation(t: &Time, (d_psi, d_eps): (f64, f64)) -> Self {
        let mean_obliquity = nutation::mean_obliquity(t.tdb()) * ASEC2RAD;
        let precession = precession::compute_precession(t.tdb());
        let nutation =
            nutation::build_nutation_matrix(mean_obliquity, mean_obliquity + d_eps, d_psi);
        let m = mxm(&mxm(&nutation, &precession), &frame_bias());
        let ecliptic = mxm(&rot_x(-(mean_obliquity + d_eps)), &m);
        let gmst_hours = sidereal::sidereal_time(t);
        let gast_hours = sidereal::gast(t, gmst_hours, d_psi, mean_obliquity);
        Orientation {
            d_psi,
            d_eps,
            mean_obliquity,
            precession,
            nutation,
            m,
            ecliptic,
            gmst_hours,
            gast_hours,
        }
    }
}
