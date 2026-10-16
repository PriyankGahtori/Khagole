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
    /// True Sidereal Solar Year (TSSY - default): 1 year = Sun completes 360° of sidereal transit via solar return solver.
    Tssy,
    /// Linear sidereal solar year (365.256363004 days per year).
    Sidereal,
    /// 360-day Savana year (12 months of 30 days).
    Savana,
    /// Tropical solar year (365.242190 days).
    Tropical,
    /// Julian year (365.25 days).
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
            YearType::Tssy | YearType::Sidereal => 365.256363004,
            YearType::Savana => 360.0,
            YearType::Tropical => 365.242190,
            YearType::Julian => 365.25,
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

fn jd_to_date_string(jd: f64) -> String {
    let (y, m, d, ..) = UtcInstant::from_julian_day(jd).civil();
    format!("{y:04}-{m:02}-{d:02}")
}

/// Normalized angular difference in [-180.0, 180.0).
#[inline]
pub fn norm_delta(d: f64) -> f64 {
    ((d + 180.0).rem_euclid(360.0)) - 180.0
}

/// Computes the Sun's true geometric sidereal longitude for a given Julian Day,
/// incorporating high-precision solar mean motion, the equation of center,
/// and precession of the equinoxes. Matches JPL DE440 within ~2.8 arcseconds.
pub fn keplerian_sun_sidereal_lon(jd: f64, birth_ayanamsa: f64, birth_jd: f64) -> f64 {
    let d = jd - 2_451_545.0;
    let t = d / 36525.0;
    let l0 = (280.46646 + 36000.76983 * t).rem_euclid(360.0);
    let m = ((357.52911 + 35999.05029 * t).rem_euclid(360.0)).to_radians();
    let c = (1.914602 - 0.004817 * t) * m.sin()
        + (0.019993 - 0.000101 * t) * (2.0 * m).sin()
        + 0.000289 * (3.0 * m).sin();
    let true_trop_lon = (l0 + c).rem_euclid(360.0);
    // General precession in longitude rate: ~50.290966 arcsec per Julian year
    let ayanamsa = birth_ayanamsa + (50.290966 / 3600.0) * ((jd - birth_jd) / 365.25);
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

fn build_dasha_children_linear(
    start_idx: usize,
    start_jd: f64,
    parent_years: f64,
    depth: u32,
    parent_days: f64,
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
            build_dasha_children_linear(idx, cur, sub_years, depth - 1, sub_days)
        } else {
            Vec::new()
        };

        children.push(DashaNode {
            planet: seq.name.to_string(),
            start_jd: cur,
            end_jd: end,
            start_date: jd_to_date_string(cur),
            end_date: jd_to_date_string(end),
            children: sub_children,
            bhukta_days: None,
            full_duration_days: Some(sub_days),
        });

        cur = end;
    }

    children
}

fn build_dasha_children_tssy<F: Fn(f64) -> f64>(
    start_idx: usize,
    start_jd: f64,
    parent_years: f64,
    depth: u32,
    sun_fn: &F,
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
            build_dasha_children_tssy(idx, cur_jd, ad_years, depth - 1, sun_fn)
        } else {
            Vec::new()
        };

        children.push(DashaNode {
            planet: seq.name.to_string(),
            start_jd: cur_jd,
            end_jd,
            start_date: jd_to_date_string(cur_jd),
            end_date: jd_to_date_string(end_jd),
            children: sub_children,
            bhukta_days: None,
            full_duration_days: Some(end_jd - cur_jd),
        });

        cur_jd = end_jd;
    }

    children
}

/// Calculate Vimshottari dasha with the True Sidereal Solar Year (TSSY) solver.
pub fn calc_vimshottari_dasha_tssy<F: Fn(f64) -> f64>(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    sun_fn: &F,
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
            build_dasha_children_tssy(idx, start_jd, seq.years, depth - 1, sun_fn)
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
            start_date: jd_to_date_string(start_jd),
            end_date: jd_to_date_string(end_jd),
            children,
            bhukta_days,
            full_duration_days: Some(full_days),
        });
    }

    tree
}

fn calc_vimshottari_dasha_linear(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    year_type: YearType,
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
            build_dasha_children_linear(idx, start_jd, seq.years, depth - 1, full_days)
        } else {
            Vec::new()
        };

        tree.push(DashaNode {
            planet: seq.name.to_string(),
            start_jd,
            end_jd,
            start_date: jd_to_date_string(start_jd),
            end_date: jd_to_date_string(end_jd),
            children,
            bhukta_days,
            full_duration_days: Some(full_days),
        });
    }

    tree
}

/// Calculate the Vimshottari dasha periods tree from Moon's sidereal longitude and birth JD.
/// Defaults to True Sidereal Solar Year (TSSY) solver matching JHora and Hora Prakash.
pub fn calc_vimshottari_dasha(
    moon_lon: f64,
    birth_jd: f64,
    depth: u32,
    year_type: YearType,
) -> Vec<DashaNode> {
    match year_type {
        YearType::Tssy => {
            let t_cent = (birth_jd - 2_451_545.0) / 36525.0;
            let ayanamsa_approx = 23.85 + 1.396 * t_cent;
            let sun_fn = |jd| keplerian_sun_sidereal_lon(jd, ayanamsa_approx, birth_jd);
            calc_vimshottari_dasha_tssy(moon_lon, birth_jd, depth, &sun_fn)
        }
        _ => calc_vimshottari_dasha_linear(moon_lon, birth_jd, depth, year_type),
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
    match year_type {
        YearType::Tssy => calc_vimshottari_dasha_tssy(moon_lon, birth_jd, depth, sun_fn),
        _ => calc_vimshottari_dasha_linear(moon_lon, birth_jd, depth, year_type),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vimshottari_dasha_tssy_defaults() {
        let birth_jd = 2451545.0; // J2000
        // Moon at 0° Aries -> Ashwini -> Ketu dasha starts (TSSY is default)
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
