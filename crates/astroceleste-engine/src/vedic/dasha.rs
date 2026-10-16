//! Vimshottari Dasha subsystem.

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
    /// True sidereal solar year (365.256363004 days) - traditional default.
    Sidereal,
    /// 360-day Savana year (12 months of 30 days).
    Savana,
    /// Tropical solar year (365.242190 days).
    Tropical,
    /// Julian year (365.25 days).
    Julian,
}

impl YearType {
    /// Number of days per year.
    pub fn days_per_year(&self) -> f64 {
        match self {
            YearType::Sidereal => 365.256363004,
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

fn build_dasha_children(
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
            build_dasha_children(idx, cur, sub_years, depth - 1, sub_days)
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

/// Calculate the Vimshottari dasha periods tree from Moon's sidereal longitude and birth JD.
pub fn calc_vimshottari_dasha(
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
            build_dasha_children(idx, start_jd, seq.years, depth - 1, full_days)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vimshottari_dasha_structure() {
        let birth_jd = 2451545.0; // J2000
        // Moon at 0° Aries -> Ashwini -> Ketu dasha starts
        let tree = calc_vimshottari_dasha(0.0, birth_jd, 3, YearType::Sidereal);
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

        // Total duration should equal 120 sidereal years
        let total_days = tree.last().unwrap().end_jd - tree.first().unwrap().start_jd;
        assert!((total_days - 120.0 * 365.256363004).abs() < 0.1);

        // Each Mahadasha has 9 Antardashas
        assert_eq!(tree[0].children.len(), 9);
        // Each Antardasha has 9 Pratyantardashas
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
