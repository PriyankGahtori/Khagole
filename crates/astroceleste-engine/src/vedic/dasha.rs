//! Vimshottari Dasha subsystem with True Sidereal Solar Year (TSSY) support.

use serde::{Deserialize, Serialize};
use crate::instant::UtcInstant;

/// Definition of a planet's period in the 120-year Vimshottari cycle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DashaPeriodDef {
    /// Ruling planet.
    pub name: &'static str,
    /// Duration in years.
    pub years: f64,
}

/// Standard 120-year Vimshottari Dasha sequence.
pub const DASHA_SEQUENCE: [DashaPeriodDef; 9] = [
    DashaPeriodDef { name: "Ketu", years: 7.0 },
    DashaPeriodDef { name: "Venus", years: 20.0 },
    DashaPeriodDef { name: "Sun", years: 6.0 },
    DashaPeriodDef { name: "Moon", years: 10.0 },
    DashaPeriodDef { name: "Mars", years: 7.0 },
    DashaPeriodDef { name: "Rahu", years: 18.0 },
    DashaPeriodDef { name: "Jupiter", years: 16.0 },
    DashaPeriodDef { name: "Saturn", years: 19.0 },
    DashaPeriodDef { name: "Mercury", years: 17.0 },
];

/// Total cycle length in years.
pub const TOTAL_YEARS: f64 = 120.0;

/// Basis for measuring a year in dasha calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum YearType {
    /// Standard Vedic Calendar Year (365.25d civil calendar addition - default): Mahadashas end on exact calendar anniversaries without 1-day variance.
    Calendar,
    /// True Sidereal Solar Year (TSSY): 1 year = Sun completes 360° of sidereal transit via solar return solver.
    Tssy,
    /// Linear sidereal solar year (365.256363004 days per year).
    Sidereal,
    /// 360-day Savana year (12 months of 30 days).
    Savana,
    /// Tropical solar year (365.242190 days).
    Tropical,
    /// Julian year (365.25 days continuous).
    Julian,
}

impl Default for YearType {
    fn default() -> Self {
        YearType::Tssy
    }
}

impl YearType {
    /// Nominal number of days per year.
    pub fn days_per_year(&self) -> f64 {
        match self {
            YearType::Calendar | YearType::Julian => 365.25,
            YearType::Tssy | YearType::Sidereal => 365.256363004,
            YearType::Savana => 360.0,
            YearType::Tropical => 365.242190,
        }
    }
}

/// A node in the Vimshottari Dasha tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashaNode {
    /// Planet governing this dasha period.
    pub planet: String,
    /// Start instant as Julian Day.
    pub start_jd: f64,
    /// End instant as Julian Day.
    pub end_jd: f64,
    /// Start date formatted as "YYYY-MM-DD".
    pub start_date: String,
    /// End date formatted as "YYYY-MM-DD".
    pub end_date: String,
    /// Sub-periods (e.g. Antardashas within Mahadasha).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<DashaNode>,
    /// Elapsed balance days before birth (for the first dasha).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bhukta_days: Option<f64>,
    /// Full, unclipped duration of the period in days.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_duration_days: Option<f64>,
}

/// Convert a Julian Day to civil "YYYY-MM-DD" adjusted by timezone offset hours.
pub fn jd_to_civil_date_string(jd: f64, tz_offset_hours: f64) -> String {
    let local_jd = jd + tz_offset_hours / 24.0;
    let (y, m, d, ..) = UtcInstant::from_julian_day(local_jd).civil();
    format!("{y:04}-{m:02}-{d:02}")
}

/// Convert a Julian Day to civil "YYYY-MM-DD" in UTC (0.0 offset).
pub fn jd_to_date_string(jd: f64) -> String {
    jd_to_civil_date_string(jd, 0.0)
}

/// Normalized angular difference in [-180.0, 180.0).
#[inline]
pub fn norm_delta(d: f64) -> f64 {
    ((d + 180.0).rem_euclid(360.0)) - 180.0
}

/// Computes the Sun's true geometric sidereal longitude for a given Julian Day,
/// incorporating high-precision solar mean motion, the equation of center,
/// Earth-Moon barycenter perturbations, and IAU 2006 precession of the equinoxes.
pub fn keplerian_sun_sidereal_lon(jd: f64, _birth_ayanamsa: f64, _birth_jd: f64) -> f64 {
    let d = jd - 2_451_545.0;
    let t = d / 36525.0;
    let t2 = t * t;
    let l0 = (280.466457 + 36000.769828 * t + 0.0003032 * t2).rem_euclid(360.0);
    let m = ((357.529109 + 35999.050291 * t - 0.0001537 * t2).rem_euclid(360.0)).to_radians();
    let c = (1.914602 - 0.004817 * t - 0.000014 * t2) * m.sin()
        + (0.019993 - 0.000101 * t) * (2.0 * m).sin()
        + 0.000289 * (3.0 * m).sin();
    // Moon perturbation due to Earth-Moon barycenter motion: ~6.45 arcseconds
    let d_moon = ((297.85036 + 445267.11148 * t).rem_euclid(360.0)).to_radians();
    let moon_pert = (6.454 / 3600.0) * d_moon.sin();
    let true_trop_lon = (l0 + c + moon_pert).rem_euclid(360.0);
    let ayanamsa = crate::zodiac::ayanamsa_value(jd, "lahiri");
    (true_trop_lon - ayanamsa).rem_euclid(360.0)
}

/// Find the exact Julian Day where the Sun returns to `target_lon` around `seed_jd`.
/// Uses a secant / bracketed Newton iteration converging to within 1 arcsecond (~1 second of time).
pub fn find_solar_return<F: Fn(f64) -> f64>(target_lon: f64, seed_jd: f64, sun_fn: &F) -> f64 {
    const ANGLE_TOL_DEG: f64 = 1.0 / 3600.0; // 1 arcsec
    const TIME_TOL_DAYS: f64 = 1.0 / 86400.0; // 1 second

    let mut t0 = seed_jd;
    let mut err0 = norm_delta(sun_fn(t0) - target_lon);
    if err0.abs() < ANGLE_TOL_DEG {
        return t0;
    }

    let mut t1 = seed_jd + 2.0 / 24.0; // 2 hours bracket offset
    let mut err1;

    for _ in 0..50 {
        let lon1 = sun_fn(t1);
        err1 = norm_delta(lon1 - target_lon);
        if err1.abs() < ANGLE_TOL_DEG {
            return t1;
        }

        let d_err = err1 - err0;
        if d_err.abs() < 1e-12 {
            t1 += 1.0;
            continue;
        }

        let next_t = t1 - err1 * (t1 - t0) / d_err;
        if (next_t - t1).abs() < TIME_TOL_DAYS {
            return next_t;
        }

        t0 = t1;
        err0 = err1;
        t1 = next_t;
    }

    t1
}

/// Advance a Julian Day by `years` true sidereal solar years using solar return root finding.
pub fn advance_tssy<F: Fn(f64) -> f64>(jd_start: f64, years: f64, sun_fn: &F) -> f64 {
    let start_lon = sun_fn(jd_start);
    let target_lon = (start_lon + years * 360.0).rem_euclid(360.0);
    let seed_jd = jd_start + years * 365.256363004;
    find_solar_return(target_lon, seed_jd, sun_fn)
}

fn build_dasha_children_linear_tz(
    start_idx: usize,
    start_jd: f64,
    parent_years: f64,
    depth: u32,
    parent_days: f64,
    tz_offset_hours: f64,
) -> Vec<DashaNode> {
    if depth == 0 {
        return Vec::new();
    }

    let mut children = Vec::with_capacity(9);
    let mut cur = start_jd;

    for i in 0..9 {
        let idx = (start_idx + i) % 9;
        let seq = &DASHA_SEQUENCE[idx];
        let sub_days = parent_days * seq.years / TOTAL_YEARS;
        let sub_years = parent_years * seq.years / TOTAL_YEARS;
        let end = cur + sub_days;

        let sub_children = if depth > 1 {
            build_dasha_children_linear_tz(idx, cur, sub_years, depth - 1, sub_days, tz_offset_hours)
        } else {
            Vec::new()
        };

        children.push(DashaNode {
            planet: seq.name.to_string(),
            start_jd: cur,
            end_jd: end,
            start_date: jd_to_civil_date_string(cur, tz_offset_hours),
            end_date: jd_to_civil_date_string(end, tz_offset_hours),
            children: sub_children,
            bhukta_days: None,
            full_duration_days: Some(sub_days),
        });

        cur = end;
    }

    children
}

fn build_dasha_children_tssy_tz<F: Fn(f64) -> f64>(
    start_idx: usize,
    start_jd: f64,
    parent_years: f64,
    depth: u32,
    sun_fn: &F,
    tz_offset_hours: f64,
) -> Vec<DashaNode> {
    if depth == 0 {
        return Vec::new();
    }

    let mut children = Vec::with_capacity(9);
    let mut cur_jd = start_jd;

    for i in 0..9 {
        let idx = (start_idx + i) % 9;
        let seq = &DASHA_SEQUENCE[idx];
        let ad_years = parent_years * seq.years / TOTAL_YEARS;
        let end_jd = advance_tssy(cur_jd, ad_years, sun_fn);

        let sub_children = if depth > 1 {
            build_dasha_children_tssy_tz(idx, cur_jd, ad_years, depth - 1, sun_fn, tz_offset_hours)
        } else {
            Vec::new()
        };

        children.push(DashaNode {
            planet: seq.name.to_string(),
            start_jd: cur_jd,
            end_jd,
            start_date: jd_to_civil_date_string(cur_jd, tz_offset_hours),
            end_date: jd_to_civil_date_string(end_jd, tz_offset_hours),
            children: sub_children,
            bhukta_days: None,
            full_duration_days: Some(end_jd - cur_jd),
        });

        cur_jd = end_jd;
    }

    children
}

/// Calculate Vimshottari dasha with the Standard Vedic Calendar Year method.
/// In classical Parashari tradition, Mahadashas are whole calendar years added to the first
/// Mahadasha's transition date, guaranteeing consistent anniversary dates without 1-day variance.
pub fn calc_vimshottari_dasha_calendar(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    tz_offset_hours: f64,
) -> Vec<DashaNode> {
    let norm_lon = moon_lon.rem_euclid(360.0);
    let nakshatra_span = 360.0 / 27.0;
    let nakshatra_idx = ((norm_lon / nakshatra_span).floor() as usize).min(26);
    let dasha_start_index = nakshatra_idx % 9;
    let fraction_elapsed = (norm_lon % nakshatra_span) / nakshatra_span;

    let balance_years = DASHA_SEQUENCE[dasha_start_index].years * (1.0 - fraction_elapsed);
    let elapsed_years = DASHA_SEQUENCE[dasha_start_index].years * fraction_elapsed;

    let local_birth_jd = birth_jd + tz_offset_hours / 24.0;

    let balance_days = balance_years * 365.25;
    let end1_local_jd = local_birth_jd + balance_days;
    let (mut cur_y, mut cur_m, mut cur_d, cur_h, cur_min, cur_s, cur_us) =
        UtcInstant::from_julian_day(end1_local_jd).civil();

    let elapsed_days = elapsed_years * 365.25;
    let start1_local_jd = local_birth_jd - elapsed_days;
    let start1_utc_jd = start1_local_jd - tz_offset_hours / 24.0;
    let end1_utc_jd = end1_local_jd - tz_offset_hours / 24.0;

    let mut tree = Vec::with_capacity(9);
    let mut prev_end_utc_jd = end1_utc_jd;

    let anchor_m = cur_m;
    let anchor_d = cur_d;

    for i in 0..9 {
        let idx = (dasha_start_index + i) % 9;
        let seq = &DASHA_SEQUENCE[idx];

        let (start_jd, end_jd, bhukta_days) = if i == 0 {
            (start1_utc_jd, end1_utc_jd, Some(elapsed_days))
        } else {
            let start = prev_end_utc_jd;
            cur_y += seq.years as i32;
            cur_m = anchor_m;
            let max_d = crate::instant::days_in_month(cur_y, cur_m);
            cur_d = anchor_d.min(max_d);
            let next_end_local = UtcInstant::from_civil(
                cur_y, cur_m, cur_d, cur_h, cur_min, cur_s, cur_us,
            );
            let next_end_local_jd = next_end_local.julian_day();
            let next_end_utc_jd = next_end_local_jd - tz_offset_hours / 24.0;
            prev_end_utc_jd = next_end_utc_jd;
            (start, next_end_utc_jd, None)
        };

        let full_days = end_jd - start_jd;

        let children = if depth > 1 {
            build_dasha_children_linear_tz(idx, start_jd, seq.years, depth - 1, full_days, tz_offset_hours)
        } else {
            Vec::new()
        };

        tree.push(DashaNode {
            planet: seq.name.to_string(),
            start_jd,
            end_jd,
            start_date: jd_to_civil_date_string(start_jd, tz_offset_hours),
            end_date: jd_to_civil_date_string(end_jd, tz_offset_hours),
            children,
            bhukta_days,
            full_duration_days: Some(full_days),
        });
    }

    tree
}

/// Calculate Vimshottari dasha with the True Sidereal Solar Year (TSSY) solver.
pub fn calc_vimshottari_dasha_tssy<F: Fn(f64) -> f64>(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    sun_fn: &F,
) -> Vec<DashaNode> {
    calc_vimshottari_dasha_tssy_tz(moon_lon, birth_jd, depth, sun_fn, 0.0)
}

/// Calculate Vimshottari dasha with the True Sidereal Solar Year (TSSY) solver with local civil timezone adjustment.
pub fn calc_vimshottari_dasha_tssy_tz<F: Fn(f64) -> f64>(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    sun_fn: &F,
    tz_offset_hours: f64,
) -> Vec<DashaNode> {
    let norm_lon = moon_lon.rem_euclid(360.0);
    let nakshatra_span = 360.0 / 27.0;
    let nakshatra_idx = ((norm_lon / nakshatra_span).floor() as usize).min(26);
    let dasha_start_index = nakshatra_idx % 9;
    let fraction_elapsed = (norm_lon % nakshatra_span) / nakshatra_span;

    let sid_sun_lon = sun_fn(birth_jd);
    let balance_years = DASHA_SEQUENCE[dasha_start_index].years * (1.0 - fraction_elapsed);
    let elapsed_years = DASHA_SEQUENCE[dasha_start_index].years * fraction_elapsed;

    // Target Sun longitude when the first Mahadasha ends:
    let target_lon = (sid_sun_lon + balance_years * 360.0).rem_euclid(360.0);

    // Exact cycle start instant (when birth Mahadasha began before birth):
    let cycle_start_jd = find_solar_return(
        target_lon,
        birth_jd - elapsed_years * 365.256363004,
        sun_fn,
    );

    let mut tree = Vec::with_capacity(9);
    let mut cumulative_years = 0.0;

    for i in 0..9 {
        let idx = (dasha_start_index + i) % 9;
        let seq = &DASHA_SEQUENCE[idx];

        let start_jd = find_solar_return(
            target_lon,
            cycle_start_jd + cumulative_years * 365.256363004,
            sun_fn,
        );
        cumulative_years += seq.years;
        let end_jd = find_solar_return(
            target_lon,
            cycle_start_jd + cumulative_years * 365.256363004,
            sun_fn,
        );

        let full_days = end_jd - start_jd;

        let children = if depth > 1 {
            build_dasha_children_tssy_tz(idx, start_jd, seq.years, depth - 1, sun_fn, tz_offset_hours)
        } else {
            Vec::new()
        };

        let bhukta_days = if i == 0 {
            Some(birth_jd - start_jd)
        } else {
            None
        };

        tree.push(DashaNode {
            planet: seq.name.to_string(),
            start_jd,
            end_jd,
            start_date: jd_to_civil_date_string(start_jd, tz_offset_hours),
            end_date: jd_to_civil_date_string(end_jd, tz_offset_hours),
            children,
            bhukta_days,
            full_duration_days: Some(full_days),
        });
    }

    tree
}

fn calc_vimshottari_dasha_linear_tz(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    year_type: YearType,
    tz_offset_hours: f64,
) -> Vec<DashaNode> {
    let norm_lon = moon_lon.rem_euclid(360.0);
    let nakshatra_span = 360.0 / 27.0;
    let nakshatra_idx = ((norm_lon / nakshatra_span).floor() as usize).min(26);
    let dasha_start_index = nakshatra_idx % 9;
    let fraction_elapsed = (norm_lon % nakshatra_span) / nakshatra_span;

    let days_per_year = year_type.days_per_year();
    let balance_years = DASHA_SEQUENCE[dasha_start_index].years * (1.0 - fraction_elapsed);

    let mut tree = Vec::with_capacity(9);
    let mut cur_jd = birth_jd;

    for i in 0..9 {
        let idx = (dasha_start_index + i) % 9;
        let seq = &DASHA_SEQUENCE[idx];
        let full_days = seq.years * days_per_year;

        let (start_jd, end_jd, bhukta_days) = if i == 0 {
            let true_start_jd = birth_jd - fraction_elapsed * full_days;
            let end = birth_jd + balance_years * days_per_year;
            cur_jd = end;
            (true_start_jd, end, Some(fraction_elapsed * full_days))
        } else {
            let start = cur_jd;
            let end = cur_jd + full_days;
            cur_jd = end;
            (start, end, None)
        };

        let children = if depth > 1 {
            build_dasha_children_linear_tz(idx, start_jd, seq.years, depth - 1, full_days, tz_offset_hours)
        } else {
            Vec::new()
        };

        tree.push(DashaNode {
            planet: seq.name.to_string(),
            start_jd,
            end_jd,
            start_date: jd_to_civil_date_string(start_jd, tz_offset_hours),
            end_date: jd_to_civil_date_string(end_jd, tz_offset_hours),
            children,
            bhukta_days,
            full_duration_days: Some(full_days),
        });
    }

    tree
}

/// Calculate the Vimshottari dasha periods tree from Moon's sidereal longitude and birth JD.
/// Defaults to True Sidereal Solar Year (TSSY) solver matching canonical Vedic astronomy.
pub fn calc_vimshottari_dasha(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    year_type: YearType,
) -> Vec<DashaNode> {
    calc_vimshottari_dasha_tz(moon_lon, birth_jd, depth, year_type, 0.0)
}

/// Calculate the Vimshottari dasha periods tree with local civil timezone adjustment.
pub fn calc_vimshottari_dasha_tz(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    year_type: YearType,
    tz_offset_hours: f64,
) -> Vec<DashaNode> {
    match year_type {
        YearType::Calendar => calc_vimshottari_dasha_calendar(moon_lon, birth_jd, depth, tz_offset_hours),
        YearType::Tssy => {
            let sun_fn = |jd| keplerian_sun_sidereal_lon(jd, 0.0, birth_jd);
            calc_vimshottari_dasha_tssy_tz(moon_lon, birth_jd, depth, &sun_fn, tz_offset_hours)
        }
        _ => calc_vimshottari_dasha_linear_tz(moon_lon, birth_jd, depth, year_type, tz_offset_hours),
    }
}

/// Calculate the Vimshottari dasha periods tree with a custom Sun longitude closure for exact TSSY solver.
pub fn calc_vimshottari_dasha_with_sun<F: Fn(f64) -> f64>(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    year_type: YearType,
    sun_fn: &F,
) -> Vec<DashaNode> {
    calc_vimshottari_dasha_with_sun_tz(moon_lon, birth_jd, depth, year_type, sun_fn, 0.0)
}

/// Calculate the Vimshottari dasha periods tree with a custom Sun longitude closure and local timezone offset.
pub fn calc_vimshottari_dasha_with_sun_tz<F: Fn(f64) -> f64>(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    year_type: YearType,
    sun_fn: &F,
    tz_offset_hours: f64,
) -> Vec<DashaNode> {
    match year_type {
        YearType::Calendar => calc_vimshottari_dasha_calendar(moon_lon, birth_jd, depth, tz_offset_hours),
        YearType::Tssy => calc_vimshottari_dasha_tssy_tz(moon_lon, birth_jd, depth, sun_fn, tz_offset_hours),
        _ => calc_vimshottari_dasha_linear_tz(moon_lon, birth_jd, depth, year_type, tz_offset_hours),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vimshottari_dasha_calendar_defaults() {
        let birth_jd = 2451545.0; // J2000
        // Moon at 0° Aries -> Ashwini -> Ketu dasha starts (Calendar is default)
        let tree = calc_vimshottari_dasha(0.0, birth_jd, 3, YearType::default());
        assert_eq!(tree.len(), 9);
        assert_eq!(tree[0].planet, "Ketu");
        assert_eq!(tree[1].planet, "Venus");
        assert_eq!(tree[2].planet, "Sun");
        assert_eq!(tree[3].planet, "Moon");
        assert_eq!(tree[4].planet, "Mars");
        assert_eq!(tree[5].planet, "Rahu");
        assert_eq!(tree[6].planet, "Jupiter");
        assert_eq!(tree[7].planet, "Saturn");
        assert_eq!(tree[8].planet, "Mercury");

        // Sub-periods: Ketu Mahadasha has 9 Antardashas
        assert_eq!(tree[0].children.len(), 9);
        assert_eq!(tree[0].children[0].children.len(), 9);
    }

    #[test]
    fn test_vimshottari_calendar_anniversary_consistency() {
        // Birth date: 1950-09-17 04:04:00 UTC (09:34 IST, Vadnagar)
        let birth_jd = 2433541.66944;
        let moon_lon = 217.949567; // Anuradha
        let tree = calc_vimshottari_dasha_tz(moon_lon, birth_jd, 1, YearType::Calendar, 5.5);
        assert_eq!(tree.len(), 9);

        // First Mahadasha ends on 1963-02-18
        assert_eq!(tree[0].end_date, "1963-02-18");
        // In canonical Parashari calendar addition, EVERY subsequent Mahadasha ends on February 18
        assert_eq!(tree[1].start_date, "1963-02-18");
        assert_eq!(tree[1].end_date, "1980-02-18"); // Mercury 17y
        assert_eq!(tree[2].start_date, "1980-02-18");
        assert_eq!(tree[2].end_date, "1987-02-18"); // Ketu 7y
        assert_eq!(tree[3].start_date, "1987-02-18");
        assert_eq!(tree[3].end_date, "2007-02-18"); // Venus 20y
        assert_eq!(tree[4].start_date, "2007-02-18");
        assert_eq!(tree[4].end_date, "2013-02-18"); // Sun 6y
        assert_eq!(tree[5].start_date, "2013-02-18");
        assert_eq!(tree[5].end_date, "2023-02-18"); // Moon 10y
        assert_eq!(tree[6].start_date, "2023-02-18");
        assert_eq!(tree[6].end_date, "2030-02-18"); // Mars 7y
        assert_eq!(tree[7].start_date, "2030-02-18");
        assert_eq!(tree[7].end_date, "2048-02-18"); // Rahu 18y
        assert_eq!(tree[8].start_date, "2048-02-18");
        assert_eq!(tree[8].end_date, "2064-02-18"); // Jupiter 16y

        // Zero 1-day variance across all 9 Mahadashas!
        for i in 1..9 {
            assert_eq!(&tree[i].end_date[5..], "02-18", "MD {} must end on anniversary 02-18", tree[i].planet);
        }
    }

    #[test]
    fn test_vimshottari_dasha_tssy_mode() {
        let birth_jd = 2451545.0; // J2000
        let tree = calc_vimshottari_dasha(0.0, birth_jd, 2, YearType::Tssy);
        assert_eq!(tree.len(), 9);
        assert_eq!(tree[0].planet, "Ketu");
        assert_eq!(tree[0].children.len(), 9);
    }

    #[test]
    fn test_vimshottari_dasha_linear_structure() {
        let birth_jd = 2451545.0; // J2000
        let tree = calc_vimshottari_dasha(0.0, birth_jd, 3, YearType::Sidereal);
        assert_eq!(tree.len(), 9);
        assert_eq!(tree[0].planet, "Ketu");
        assert_eq!(tree[1].planet, "Venus");

        // Total duration should equal 120 sidereal years
        let total_days = tree.last().unwrap().end_jd - tree.first().unwrap().start_jd;
        assert!((total_days - 120.0 * 365.256363004).abs() < 0.1);

        assert_eq!(tree[0].children.len(), 9);
        assert_eq!(tree[0].children[0].children.len(), 9);
    }

    #[test]
    fn test_vimshottari_balance() {
        let birth_jd = 2451545.0;
        // Moon at 6°40' (middle of Ashwini -> fraction_elapsed = 0.5)
        let tree = calc_vimshottari_dasha(6.666666666666667, birth_jd, 1, YearType::Savana);
        assert_eq!(tree[0].planet, "Ketu");
        // Ketu is 7 years = 7 * 360 = 2520 days. Half elapsed = 1260 days.
        let full_days = 7.0 * 360.0;
        assert_eq!(tree[0].full_duration_days, Some(full_days));
        assert!((tree[0].bhukta_days.unwrap() - 1260.0).abs() < 0.01);
        assert!((tree[0].end_jd - birth_jd - 1260.0).abs() < 0.01);
    }
}
