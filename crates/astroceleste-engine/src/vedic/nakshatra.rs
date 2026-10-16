//! Vedic 27 Nakshatras subsystem.

use serde::{Deserialize, Serialize};

/// The 27 classical Vedic nakshatras in zodiacal order (Ashwini to Revati).
pub const NAKSHATRAS: [&str; 27] = [
    "Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira", "Ardra",
    "Punarvasu", "Pushya", "Ashlesha", "Magha", "Purva Phalguni", "Uttara Phalguni",
    "Hasta", "Chitra", "Swati", "Vishakha", "Anuradha", "Jyeshtha",
    "Mula", "Purva Ashadha", "Uttara Ashadha", "Shravana", "Dhanishta",
    "Shatabhisha", "Purva Bhadrapada", "Uttara Bhadrapada", "Revati",
];

/// The 27 nakshatra lords according to Vimshottari dasha system.
pub const NAKSHATRA_LORDS: [&str; 27] = [
    "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury",
    "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury",
    "Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury",
];

/// Angular span of one nakshatra (360° / 27 = 13°20' = 13.333333333333334°).
pub const NAKSHATRA_SPAN: f64 = 360.0 / 27.0;

/// Angular span of one pada (quarter) of a nakshatra (360° / 108 = 3°20' = 3.3333333333333335°).
pub const PADA_SPAN: f64 = 360.0 / 108.0;

/// Information about a nakshatra placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NakshatraInfo {
    /// 0-based index of the nakshatra (0..26).
    pub index: usize,
    /// Name of the nakshatra (Ashwini..Revati).
    pub name: &'static str,
    /// Vimshottari lord of the nakshatra (Ketu..Mercury).
    pub lord: &'static str,
    /// Pada (quarter) of the nakshatra (1..4).
    pub pada: u8,
}

/// Calculate the nakshatra, lord, and pada for a given sidereal longitude (0°–360°).
pub fn nakshatra_info(lon: f64) -> NakshatraInfo {
    let norm_lon = lon.rem_euclid(360.0);
    let index = ((norm_lon / NAKSHATRA_SPAN).floor() as usize).min(26);
    let pada = (((norm_lon % NAKSHATRA_SPAN) / PADA_SPAN).floor() as u8 + 1).min(4);

    NakshatraInfo {
        index,
        name: NAKSHATRAS[index],
        lord: NAKSHATRA_LORDS[index],
        pada,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nakshatras() {
        let n0 = nakshatra_info(0.0);
        assert_eq!(n0.name, "Ashwini");
        assert_eq!(n0.lord, "Ketu");
        assert_eq!(n0.pada, 1);
        assert_eq!(n0.index, 0);

        // 3°20' = 3.333333° (end of pada 1)
        let n0_p2 = nakshatra_info(3.34);
        assert_eq!(n0_p2.name, "Ashwini");
        assert_eq!(n0_p2.pada, 2);

        // Krittika begins at 26°40' = 26.666667° (index 2, lord Sun)
        let n2 = nakshatra_info(26.7);
        assert_eq!(n2.name, "Krittika");
        assert_eq!(n2.lord, "Sun");
        assert_eq!(n2.index, 2);

        // Edge test: Revati pada 4 at 359.9°
        let n26 = nakshatra_info(359.9);
        assert_eq!(n26.name, "Revati");
        assert_eq!(n26.lord, "Mercury");
        assert_eq!(n26.pada, 4);
        assert_eq!(n26.index, 26);
    }
}
