//! House cusps, Ascendant and Midheaven: Placidus (default), Koch, Regiomontanus, Campanus,
//! Topocentric (Polich-Page), Alcabitius, Morinus, Porphyry, Equal, Vehlow and Whole Sign.
//!
//! Placidus, Whole Sign, Equal and Porphyry follow the reference implementation
//! (`SkyfieldEngine.calculate_houses`); the other systems use the standard formulas, as in
//! Swiss Ephemeris' `swehouse.c`, against which the application tests them.

use std::f64::consts::PI;

use crate::pyfloat;

/// Supported astrological house systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HouseSystem {
    /// Placidus semi-arc system.
    Placidus,
    /// Koch birthplace system.
    Koch,
    /// Regiomontanus rational house system.
    Regiomontanus,
    /// Campanus prime vertical system.
    Campanus,
    /// Topocentric time-based system.
    Topocentric,
    /// Alcabitius semi-arc system.
    Alcabitius,
    /// Morinus equatorial system.
    Morinus,
    /// Porphyry quadrant trisecting system.
    Porphyry,
    /// Equal houses from Ascendant.
    Equal,
    /// Vehlow equal house system with Ascendant at midpoint of 1st house.
    Vehlow,
    /// Whole Sign house system (standard in Hellenistic and Vedic Rashi charts).
    WholeSign,
    /// Sripati house system (traditional Indian Bhava Chalita).
    Sripati,
}

impl HouseSystem {
    /// From the first letter of the code (the Swiss Ephemeris letters), like the reference:
    /// anything unknown is Placidus.
    pub fn from_code(code: &str) -> Self {
        match code.chars().next().map(|c| c.to_ascii_uppercase()) {
            Some('K') => HouseSystem::Koch,
            Some('R') => HouseSystem::Regiomontanus,
            Some('C') => HouseSystem::Campanus,
            Some('T') => HouseSystem::Topocentric,
            Some('B') => HouseSystem::Alcabitius,
            Some('M') => HouseSystem::Morinus,
            Some('O') => HouseSystem::Porphyry,
            Some('E') => HouseSystem::Equal,
            Some('V') => HouseSystem::Vehlow,
            Some('W') => HouseSystem::WholeSign,
            Some('S') => HouseSystem::Sripati,
            _ => HouseSystem::Placidus,
        }
    }
}

/// The calculated astrological houses and chart angles.
#[derive(Debug, Clone, PartialEq)]
pub struct Houses {
    /// Cusps 1-12, degrees.
    pub cusps: [f64; 12],
    /// Ascendant ecliptic longitude in degrees.
    pub ascendant: f64,
    /// Midheaven (Medium Coeli) ecliptic longitude in degrees.
    pub midheaven: f64,
}

/// Houses at UT1 Julian date `jd` for geographic `lat`/`lon` (degrees, east positive),
/// given the Greenwich apparent sidereal time at `jd` (from the planets' computation, or
/// interpolated by an electional search).
pub fn houses_at_sidereal_time(
    jd: f64,
    gast_hours: f64,
    lat: f64,
    lon: f64,
    system: HouseSystem,
    shift: f64,
) -> Houses {
    let ramc = pyfloat::rem(pyfloat::rem(gast_hours * 15.0 + lon, 360.0) + 360.0, 360.0);
    let ramc_r = ramc.to_radians();
    let lat_r = lat.to_radians();

    // Obliquity polynomial of the reference implementation.
    let t_cent = (jd - 2_451_545.0) / 36525.0;
    let eps_deg = 23.4392911 - 0.0130042 * t_cent - 0.00000016 * t_cent.powf(2.0)
        + 0.000000504 * t_cent.powf(3.0);
    let eps_r = eps_deg.to_radians();

    let mc_deg = pyfloat::rem(
        ramc_r.sin().atan2(ramc_r.cos() * eps_r.cos()).to_degrees(),
        360.0,
    );
    let y = ramc_r.cos();
    let x = -ramc_r.sin() * eps_r.cos() - lat_r.tan() * eps_r.sin();
    let asc_deg = pyfloat::rem(y.atan2(x).to_degrees(), 360.0);

    let mut c = [0.0_f64; 13]; // 1-indexed
    match system {
        HouseSystem::WholeSign => {
            let asc_sign = pyfloat::floordiv(asc_deg, 30.0);
            for (i, cusp) in c.iter_mut().enumerate().skip(1) {
                *cusp = pyfloat::rem((asc_sign + i as f64 - 1.0) * 30.0, 360.0);
            }
        }
        HouseSystem::Equal => {
            for (i, cusp) in c.iter_mut().enumerate().skip(1) {
                *cusp = pyfloat::rem(asc_deg + (i as f64 - 1.0) * 30.0, 360.0);
            }
        }
        HouseSystem::Vehlow => {
            // Equal houses with the Ascendant in the middle of the first house.
            for (i, cusp) in c.iter_mut().enumerate().skip(1) {
                *cusp = pyfloat::rem(asc_deg - 15.0 + (i as f64 - 1.0) * 30.0 + 360.0, 360.0);
            }
        }
        HouseSystem::Morinus => {
            // The equator divided into twelve from the RAMC, projected onto the ecliptic
            // along ecliptic meridians: the angles are not cusps.
            for i in 1..=12 {
                let ra = (ramc + 30.0 * i as f64).to_radians();
                c[(i + 9) % 12 + 1] = norm((ra.sin() * eps_r.cos()).atan2(ra.cos()).to_degrees());
            }
        }
        HouseSystem::Koch if lat.abs() >= 90.0 - eps_deg => {
            // Koch is undefined inside the polar circles: Porphyry, as Swiss Ephemeris does.
            porphyry(&mut c, asc_deg, mc_deg);
        }
        HouseSystem::Koch
        | HouseSystem::Regiomontanus
        | HouseSystem::Campanus
        | HouseSystem::Topocentric
        | HouseSystem::Alcabitius => {
            set_angles(&mut c, asc_deg, mc_deg);
            // (right ascension of the house circle's east point, its pole height) for
            // cusps 11, 12, 2 and 3; the others are opposite.
            let circles: [(f64, f64); 4] = match system {
                HouseSystem::Koch => {
                    let sin_a =
                        (mc_deg.to_radians().sin() * eps_r.sin() / lat_r.cos()).clamp(-1.0, 1.0);
                    let cos_a = (1.0 - sin_a * sin_a).sqrt();
                    let pole = (lat_r.tan() / cos_a).atan();
                    let ad3 = (pole.sin() * sin_a).asin().to_degrees() / 3.0;
                    [
                        (ramc + 30.0 - 2.0 * ad3, lat),
                        (ramc + 60.0 - ad3, lat),
                        (ramc + 120.0 + ad3, lat),
                        (ramc + 150.0 + 2.0 * ad3, lat),
                    ]
                }
                HouseSystem::Regiomontanus | HouseSystem::Topocentric => {
                    let (k1, k2) = if system == HouseSystem::Regiomontanus {
                        (0.5, 30.0_f64.to_radians().cos())
                    } else {
                        (1.0 / 3.0, 2.0 / 3.0)
                    };
                    let p1 = (lat_r.tan() * k1).atan().to_degrees();
                    let p2 = (lat_r.tan() * k2).atan().to_degrees();
                    [
                        (ramc + 30.0, p1),
                        (ramc + 60.0, p2),
                        (ramc + 120.0, p2),
                        (ramc + 150.0, p1),
                    ]
                }
                HouseSystem::Campanus => {
                    let p1 = (lat_r.sin() / 2.0).asin().to_degrees();
                    let p2 = (3.0_f64.sqrt() / 2.0 * lat_r.sin()).asin().to_degrees();
                    let x1 = (3.0_f64.sqrt() / lat_r.cos()).atan().to_degrees();
                    let x2 = (1.0 / 3.0_f64.sqrt() / lat_r.cos()).atan().to_degrees();
                    [
                        (ramc + 90.0 - x1, p1),
                        (ramc + 90.0 - x2, p2),
                        (ramc + 90.0 + x2, p2),
                        (ramc + 90.0 + x1, p1),
                    ]
                }
                _ => {
                    // Alcabitius: the Ascendant's diurnal and nocturnal semi-arcs trisected
                    // on the equator, projected along hour circles.
                    let dec = (asc_deg.to_radians().sin() * eps_r.sin()).asin();
                    let sda = (-lat_r.tan() * dec.tan())
                        .clamp(-1.0, 1.0)
                        .acos()
                        .to_degrees();
                    let sna = 180.0 - sda;
                    [
                        (ramc + sda / 3.0, 0.0),
                        (ramc + 2.0 * sda / 3.0, 0.0),
                        (ramc + 180.0 - 2.0 * sna / 3.0, 0.0),
                        (ramc + 180.0 - sna / 3.0, 0.0),
                    ]
                }
            };
            for (house, (ra, pole)) in [11, 12, 2, 3].into_iter().zip(circles) {
                c[house] = ecliptic_intersection(ra, pole, eps_r);
            }
            opposite_cusps(&mut c);
        }
        HouseSystem::Porphyry => porphyry(&mut c, asc_deg, mc_deg),
        HouseSystem::Sripati => sripati(&mut c, asc_deg, mc_deg),
        HouseSystem::Placidus => {
            set_angles(&mut c, asc_deg, mc_deg);
            let solve = |offset: f64, semi_factor: f64| {
                let target_ra = pyfloat::rem(ramc + offset, 360.0).to_radians();
                let mut lon = pyfloat::rem(ramc + offset, 360.0);
                for _ in 0..25 {
                    let l_r = lon.to_radians();
                    let sin_d = eps_r.sin() * l_r.sin();
                    let decl = sin_d.clamp(-1.0, 1.0).asin();
                    let ra = pyfloat::rem((eps_r.cos() * l_r.sin()).atan2(l_r.cos()), 2.0 * PI);
                    let tan_d = decl.tan() * lat_r.tan();
                    if tan_d.abs() >= 1.0 {
                        break;
                    }
                    let ad = tan_d.asin();
                    let diff = (ra - target_ra) - semi_factor * ad;
                    let diff = pyfloat::rem(diff + PI, 2.0 * PI) - PI;
                    if diff.abs() < 1e-8 {
                        break;
                    }
                    lon = pyfloat::rem(lon - diff.to_degrees(), 360.0);
                }
                lon
            };
            c[11] = solve(30.0, 1.0 / 3.0);
            c[12] = solve(60.0, 2.0 / 3.0);
            c[9] = solve(-30.0, -1.0 / 3.0);
            c[8] = solve(-60.0, -2.0 / 3.0);
            c[5] = pyfloat::rem(c[11] + 180.0, 360.0);
            c[6] = pyfloat::rem(c[12] + 180.0, 360.0);
            c[2] = pyfloat::rem(c[8] + 180.0, 360.0);
            c[3] = pyfloat::rem(c[9] + 180.0, 360.0);
        }
    }

    let mut cusps = [0.0; 12];
    for i in 0..12 {
        cusps[i] = pyfloat::rem(c[i + 1] - shift, 360.0);
    }
    Houses {
        cusps,
        ascendant: pyfloat::rem(asc_deg - shift, 360.0),
        midheaven: pyfloat::rem(mc_deg - shift, 360.0),
    }
}

fn norm(deg: f64) -> f64 {
    pyfloat::rem(pyfloat::rem(deg, 360.0) + 360.0, 360.0)
}

/// Longitude where the ecliptic meets the house circle whose east point has right ascension
/// `ra` and whose pole height is `pole` (degrees). With `ra` = RAMC + 90° and `pole` = the
/// latitude it is the Ascendant; with `pole` = 0 it projects `ra` along its hour circle.
fn ecliptic_intersection(ra: f64, pole: f64, eps_r: f64) -> f64 {
    let ra = ra.to_radians();
    let x = ra.cos() * eps_r.cos() - pole.to_radians().tan() * eps_r.sin();
    norm(ra.sin().atan2(x).to_degrees())
}

/// Porphyry: each quadrant between the angles trisected in longitude.
fn porphyry(c: &mut [f64; 13], asc_deg: f64, mc_deg: f64) {
    set_angles(c, asc_deg, mc_deg);
    let q1 = pyfloat::rem(asc_deg - mc_deg, 360.0);
    c[11] = pyfloat::rem(mc_deg + q1 / 3.0, 360.0);
    c[12] = pyfloat::rem(mc_deg + (2.0 * q1) / 3.0, 360.0);
    let q2 = pyfloat::rem(c[4] - asc_deg, 360.0);
    c[2] = pyfloat::rem(asc_deg + q2 / 3.0, 360.0);
    c[3] = pyfloat::rem(asc_deg + (2.0 * q2) / 3.0, 360.0);
    opposite_cusps(c);
}

/// Sripati: Porphyry cusps are house centers (Bhava Madhya);
/// cusps (Sandhis) are midpoints between adjacent house centers.
fn sripati(c: &mut [f64; 13], asc_deg: f64, mc_deg: f64) {
    let mut p = [0.0; 13];
    porphyry(&mut p, asc_deg, mc_deg);
    for i in 1..=12 {
        let prev = if i == 1 { p[12] } else { p[i - 1] };
        let curr = p[i];
        let diff = pyfloat::rem(curr - prev + 360.0, 360.0);
        c[i] = pyfloat::rem(prev + diff / 2.0, 360.0);
    }
}

/// Houses 5, 6 oppose 11, 12; houses 8, 9 oppose 2, 3.
fn opposite_cusps(c: &mut [f64; 13]) {
    c[5] = pyfloat::rem(c[11] + 180.0, 360.0);
    c[6] = pyfloat::rem(c[12] + 180.0, 360.0);
    c[8] = pyfloat::rem(c[2] + 180.0, 360.0);
    c[9] = pyfloat::rem(c[3] + 180.0, 360.0);
}

fn set_angles(c: &mut [f64; 13], asc: f64, mc: f64) {
    c[1] = asc;
    c[10] = mc;
    c[7] = pyfloat::rem(asc + 180.0, 360.0);
    c[4] = pyfloat::rem(mc + 180.0, 360.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frames::Orientation;
    use crate::time::Time;

    /// Houses at UT1 Julian date `jd`, with the sidereal time computed for it.
    fn calculate_houses(jd: f64, lat: f64, lon: f64, system: HouseSystem, shift: f64) -> Houses {
        let gast_hours = Orientation::at(&Time::from_ut1(jd)).gast_hours;
        houses_at_sidereal_time(jd, gast_hours, lat, lon, system, shift)
    }

    const JD: f64 = 2_448_027.104_166_666_5; // 1990-05-15 14:30 UT, Rome (41.9° N, 12.5° E)

    fn gap(a: f64, b: f64) -> f64 {
        let d = (a - b).rem_euclid(360.0);
        d.min(360.0 - d)
    }

    /// Cusps within 1′ of Swiss Ephemeris 2.10 (`swe.houses`) for the same moment.
    #[test]
    fn systems_match_swiss_ephemeris() {
        let reference: [(&str, [f64; 12]); 8] = [
            (
                "K",
                [
                    190.2408, 219.3528, 248.6584, 282.0137, 311.8995, 340.9197, 10.2408, 39.3528,
                    68.6584, 102.0137, 131.8995, 160.9197,
                ],
            ),
            (
                "R",
                [
                    190.2408, 214.8788, 245.0128, 282.0137, 317.7672, 346.2050, 10.2408, 34.8788,
                    65.0128, 102.0137, 137.7672, 166.2050,
                ],
            ),
            (
                "C",
                [
                    190.2408, 222.0070, 252.8709, 282.0137, 310.3369, 339.4310, 10.2408, 42.0070,
                    72.8709, 102.0137, 130.3369, 159.4310,
                ],
            ),
            (
                "T",
                [
                    190.2408, 216.9311, 248.0089, 282.0137, 315.5716, 345.3618, 10.2408, 36.9311,
                    68.0089, 102.0137, 135.5716, 165.3618,
                ],
            ),
            (
                "B",
                [
                    190.2408, 223.0794, 253.2543, 282.0137, 309.4047, 339.0304, 10.2408, 43.0794,
                    73.2543, 102.0137, 129.4047, 159.0304,
                ],
            ),
            (
                "M",
                [
                    192.0137, 220.6069, 251.6333, 284.1881, 315.5252, 344.3861, 12.0137, 40.6069,
                    71.6333, 104.1881, 135.5252, 164.3861,
                ],
            ),
            (
                "V",
                [
                    175.2408, 205.2408, 235.2408, 265.2408, 295.2408, 325.2408, 355.2408, 25.2408,
                    55.2408, 85.2408, 115.2408, 145.2408,
                ],
            ),
            (
                "S",
                [
                    175.5363, 205.5363, 236.1272, 266.7182, 296.7182, 326.1272,
                    355.5363, 25.5363, 56.1272, 86.7182, 116.7182, 146.1272,
                ],
            ),
        ];
        for (code, expected) in reference {
            let houses = calculate_houses(JD, 41.9, 12.5, HouseSystem::from_code(code), 0.0);
            for (i, (ours, theirs)) in houses.cusps.iter().zip(expected).enumerate() {
                assert!(
                    gap(*ours, theirs) < 1.0 / 60.0,
                    "{code} cusp {}: {ours:.4} vs {theirs:.4}",
                    i + 1
                );
            }
        }
    }

    #[test]
    fn codes_select_their_systems() {
        for (code, system) in [
            ("P", HouseSystem::Placidus),
            ("koch", HouseSystem::Koch),
            ("R", HouseSystem::Regiomontanus),
            ("C", HouseSystem::Campanus),
            ("T", HouseSystem::Topocentric),
            ("B", HouseSystem::Alcabitius),
            ("M", HouseSystem::Morinus),
            ("O", HouseSystem::Porphyry),
            ("E", HouseSystem::Equal),
            ("V", HouseSystem::Vehlow),
            ("W", HouseSystem::WholeSign),
            ("S", HouseSystem::Sripati),
            ("sripati", HouseSystem::Sripati),
            ("?", HouseSystem::Placidus),
            ("", HouseSystem::Placidus),
        ] {
            assert_eq!(HouseSystem::from_code(code), system, "{code}");
        }
    }

    #[test]
    fn koch_falls_back_to_porphyry_inside_the_polar_circles() {
        let koch = calculate_houses(JD, 70.0, 25.0, HouseSystem::Koch, 0.0);
        let porphyry = calculate_houses(JD, 70.0, 25.0, HouseSystem::Porphyry, 0.0);
        assert_eq!(koch, porphyry);
    }
}
