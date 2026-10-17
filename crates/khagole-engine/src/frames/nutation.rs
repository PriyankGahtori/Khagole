use crate::constants::{ASEC2RAD, ASEC360, T0, TAU};

use super::nutation_data::*;
use super::Mat3;

const TENTH_USEC_2_RAD: f64 = ASEC2RAD / 1e7;

/// Nutation matrix from mean and true obliquity and the nutation in longitude.
pub fn build_nutation_matrix(mean_obliquity: f64, true_obliquity: f64, psi: f64) -> Mat3 {
    let (sobm, cobm) = mean_obliquity.sin_cos();
    let (sobt, cobt) = true_obliquity.sin_cos();
    let (spsi, cpsi) = psi.sin_cos();
    [
        [cpsi, -spsi * cobm, -spsi * sobm],
        [
            spsi * cobt,
            cpsi * cobm * cobt + sobm * sobt,
            cpsi * sobm * cobt - cobm * sobt,
        ],
        [
            spsi * sobt,
            cpsi * cobm * sobt - sobm * cobt,
            cpsi * sobm * sobt + cobm * cobt,
        ],
    ]
}

/// Mean obliquity of the ecliptic in arcseconds (Capitaine et al. 2003).
pub fn mean_obliquity(jd_tdb: f64) -> f64 {
    let t = (jd_tdb - T0) / 36525.0;
    ((((-0.0000000434 * t - 0.000000576) * t + 0.00200340) * t - 0.0001831) * t - 46.836769) * t
        + 84381.406
}

const FA: [[f64; 5]; 5] = [
    [
        485868.249036,
        1717915923.2178,
        31.8792,
        0.051635,
        -0.00024470,
    ],
    [
        1287104.79305,
        129596581.0481,
        -0.5532,
        0.000136,
        -0.00001149,
    ],
    [
        335779.526232,
        1739527262.8478,
        -12.7512,
        -0.001037,
        0.00000417,
    ],
    [
        1072260.70369,
        1602961601.2090,
        -6.3706,
        0.006593,
        -0.00003169,
    ],
    [450160.398036, -6962890.5431, 7.4722, 0.007702, -0.00005939],
];

/// Delaunay fundamental arguments l, l', F, D, Ω in radians.
pub fn fundamental_arguments(t: f64) -> [f64; 5] {
    let mut out = [0.0; 5];
    for (o, fa) in out.iter_mut().zip(FA.iter()) {
        let mut a = fa[4] * t;
        for c in [fa[3], fa[2], fa[1]] {
            a += c;
            a *= t;
        }
        a += fa[0];
        *o = (a % ASEC360) * ASEC2RAD;
    }
    out
}

const ANOMALY: [(f64, f64); 14] = [
    (2.35555598, 8328.6914269554),
    (6.24006013, 628.301955),
    (1.627905234, 8433.466158131),
    (5.198466741, 7771.3771468121),
    (2.18243920, -33.757045),
    (4.402608842, 2608.7903141574),
    (3.176146697, 1021.3285546211),
    (1.753470314, 628.3075849991),
    (6.203480913, 334.0612426700),
    (0.599546497, 52.9690962641),
    (0.874016757, 21.3299104960),
    (5.481293871, 7.4781598567),
    (5.321159000, 3.8127774000),
    (0.02438175, 0.00000538691),
];

/// IAU 2000A nutation (Δψ, Δε) in tenths of a micro-arcsecond.
pub fn iau2000a(jd_tt: f64) -> (f64, f64) {
    series(jd_tt, LUNISOLAR_LONGITUDE.len(), true)
}

/// IAU 2000A nutation in radians.
pub fn iau2000a_radians(jd_tt: f64) -> (f64, f64) {
    let (d_psi, d_eps) = iau2000a(jd_tt);
    (d_psi * TENTH_USEC_2_RAD, d_eps * TENTH_USEC_2_RAD)
}

/// IAU 2000B nutation in radians (used by Skyfield's sunrise/sunset search).
pub fn iau2000b_radians(jd_tt: f64) -> (f64, f64) {
    let (mut d_psi, mut d_eps) = series(jd_tt, 77, false);
    d_psi += -0.000135e7;
    d_eps += 0.000388e7;
    (d_psi * TENTH_USEC_2_RAD, d_eps * TENTH_USEC_2_RAD)
}

fn series(jd_tt: f64, lunisolar_terms: usize, planetary: bool) -> (f64, f64) {
    let t = (jd_tt - T0) / 36525.0;
    // IAU 2000B uses only the first two polynomial terms of the fundamental arguments.
    let a = if planetary {
        fundamental_arguments(t)
    } else {
        let mut out = [0.0; 5];
        for (o, fa) in out.iter_mut().zip(FA.iter()) {
            *o = ((fa[1] * t + fa[0]) % ASEC360) * ASEC2RAD;
        }
        out
    };

    let (mut s_psi0, mut s_psi1, mut c_psi2) = (0.0, 0.0, 0.0);
    let (mut c_eps0, mut c_eps1, mut s_eps2) = (0.0, 0.0, 0.0);
    for i in 0..lunisolar_terms {
        let n = &NALS_T[i];
        let arg: f64 = (0..5).map(|k| n[k] as f64 * a[k]).sum();
        let (s, c) = arg.sin_cos();
        let lon = &LUNISOLAR_LONGITUDE[i];
        let obl = &LUNISOLAR_OBLIQUITY[i];
        s_psi0 += s * lon[0];
        s_psi1 += s * lon[1];
        c_psi2 += c * lon[2];
        c_eps0 += c * obl[0];
        c_eps1 += c * obl[1];
        s_eps2 += s * obl[2];
    }
    let mut dpsi = s_psi0 + s_psi1 * t + c_psi2;
    let mut deps = c_eps0 + c_eps1 * t + s_eps2;
    if !planetary {
        return (dpsi, deps);
    }

    let mut pa = [0.0; 14];
    for (p, (constant, coefficient)) in pa.iter_mut().zip(ANOMALY.iter()) {
        *p = t * coefficient + constant;
    }
    pa[13] *= t;
    let (mut psi_s, mut psi_c, mut eps_s, mut eps_c) = (0.0, 0.0, 0.0, 0.0);
    for i in 0..NAPL_T.len() {
        let n = &NAPL_T[i];
        let arg: f64 = (0..14).map(|k| n[k] as f64 * pa[k]).sum();
        let (s, c) = arg.sin_cos();
        psi_s += s * PLANETARY_LONGITUDE[i][0];
        psi_c += c * PLANETARY_LONGITUDE[i][1];
        eps_s += s * PLANETARY_OBLIQUITY[i][0];
        eps_c += c * PLANETARY_OBLIQUITY[i][1];
    }
    dpsi += psi_s;
    dpsi += psi_c;
    deps += eps_s;
    deps += eps_c;
    (dpsi, deps)
}

/// Complementary terms of the equation of the equinoxes, radians (IERS 2010 Table 5.2e).
pub fn equation_of_the_equinoxes_complementary_terms(jd_tt: f64) -> f64 {
    let t = (jd_tt - T0) / 36525.0;
    let mut fa = [0.0; 14];
    fa[0] = (485868.249036
        + (715923.2178 + (31.8792 + (0.051635 + (-0.00024470) * t) * t) * t) * t)
        * ASEC2RAD
        + (1325.0 * t).rem_euclid(1.0) * TAU;
    fa[1] = (1287104.793048
        + (1292581.0481 + (-0.5532 + (0.000136 + (-0.00001149) * t) * t) * t) * t)
        * ASEC2RAD
        + (99.0 * t).rem_euclid(1.0) * TAU;
    fa[2] = (335779.526232
        + (295262.8478 + (-12.7512 + (-0.001037 + (0.00000417) * t) * t) * t) * t)
        * ASEC2RAD
        + (1342.0 * t).rem_euclid(1.0) * TAU;
    fa[3] = (1072260.703692
        + (1105601.2090 + (-6.3706 + (0.006593 + (-0.00003169) * t) * t) * t) * t)
        * ASEC2RAD
        + (1236.0 * t).rem_euclid(1.0) * TAU;
    fa[4] = (450160.398036
        + (-482890.5431 + (7.4722 + (0.007702 + (-0.00005939) * t) * t) * t) * t)
        * ASEC2RAD
        + (-5.0 * t).rem_euclid(1.0) * TAU;
    fa[5] = 4.402608842 + 2608.7903141574 * t;
    fa[6] = 3.176146697 + 1021.3285546211 * t;
    fa[7] = 1.753470314 + 628.3075849991 * t;
    fa[8] = 6.203480913 + 334.0612426700 * t;
    fa[9] = 0.599546497 + 52.9690962641 * t;
    fa[10] = 0.874016757 + 21.3299104960 * t;
    fa[11] = 5.481293872 + 7.4781598567 * t;
    fa[12] = 5.311886287 + 3.8133035638 * t;
    fa[13] = (0.024381750 + 0.00000538691 * t) * t;
    for f in fa.iter_mut() {
        *f = f.rem_euclid(TAU);
    }

    const SE1_0: f64 = -0.87e-6;
    const SE1_1: f64 = 0.0;
    let a: f64 = (0..14).map(|k| KE1[k] as f64 * fa[k]).sum();
    let mut c_terms = SE1_0 * a.sin() + SE1_1 * a.cos();
    c_terms *= t;
    let (mut sin_sum, mut cos_sum) = (0.0, 0.0);
    for i in 0..KE0_T.len() {
        let arg: f64 = (0..14).map(|k| KE0_T[i][k] as f64 * fa[k]).sum();
        sin_sum += SE0_T_0[i] * arg.sin();
        cos_sum += SE0_T_1[i] * arg.cos();
    }
    c_terms += sin_sum;
    c_terms += cos_sum;
    c_terms * ASEC2RAD
}
