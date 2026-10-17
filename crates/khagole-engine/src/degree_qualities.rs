//! The qualities of the zodiac degrees after Lilly (*Christian Astrology*, 1659, p. 116):
//! masculine or feminine; light, dark, smoky or void; and whether a degree is deep or
//! pitted, lame (azimene) or increasing fortune.

use serde::Serialize;

use crate::catalog::{DegreeRun, SignDegrees, DEGREE_QUALITIES};
use crate::pyfloat;

/// The qualities of the degree an ecliptic longitude falls in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DegreeQualities {
    /// Zodiac sign name, e.g. "Aries".
    pub sign: &'static str,
    /// Ordinal degree (1-30) of the sign: 0°00'-0°59' is the 1st.
    pub degree: u8,
    /// "masculine" or "feminine".
    pub gender: &'static str,
    /// "light", "dark", "smoky" or "void".
    pub light: &'static str,
    /// Deep or pitted degree.
    pub pitted: bool,
    /// Lame or deficient (azimene) degree.
    pub azimene: bool,
    /// Degree increasing fortune.
    pub fortune: bool,
}

/// Lilly's table of the degree qualities, one entry per sign from Aries.
pub fn degree_quality_table() -> &'static [SignDegrees; 12] {
    &DEGREE_QUALITIES
}

/// The qualities of the degree `longitude` (ecliptic, in degrees) falls in. The zodiac is
/// whatever the longitude is measured in, as for every sign-based rule.
pub fn degree_qualities(longitude: f64) -> DegreeQualities {
    let lon = pyfloat::rem(longitude, 360.0);
    let sign = &DEGREE_QUALITIES[(pyfloat::floordiv(lon, 30.0) as usize) % 12];
    let degree = (pyfloat::floordiv(pyfloat::rem(lon, 30.0), 1.0) as u8 + 1).clamp(1, 30);
    DegreeQualities {
        sign: sign.sign,
        degree,
        gender: run_quality(sign.gender, degree),
        light: run_quality(sign.light, degree),
        pitted: sign.pitted.contains(&degree),
        azimene: sign.azimene.contains(&degree),
        fortune: sign.fortune.contains(&degree),
    }
}

fn run_quality(runs: &[DegreeRun], degree: u8) -> &'static str {
    runs.iter()
        .find(|run| degree <= run.end)
        .map_or("", |run| run.quality)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sign_is_covered() {
        for sign in degree_quality_table() {
            for runs in [sign.gender, sign.light] {
                assert!(
                    runs.windows(2).all(|w| w[0].end < w[1].end),
                    "{}",
                    sign.sign
                );
                assert!(runs.windows(2).all(|w| w[0].quality != w[1].quality));
                assert_eq!(runs.last().unwrap().end, 30, "{}", sign.sign);
            }
            assert!(sign
                .gender
                .iter()
                .all(|r| matches!(r.quality, "masculine" | "feminine")));
            assert!(sign
                .light
                .iter()
                .all(|r| matches!(r.quality, "light" | "dark" | "smoky" | "void")));
            for list in [sign.pitted, sign.azimene, sign.fortune] {
                assert!(list.windows(2).all(|w| w[0] < w[1]), "{}", sign.sign);
                assert!(list.iter().all(|d| (1..=30).contains(d)));
            }
        }
        let signs: Vec<_> = degree_quality_table().iter().map(|s| s.sign).collect();
        assert_eq!(signs, crate::dignities::SIGNS);
    }

    #[test]
    fn degrees_are_ordinal() {
        // 5°59' Aries is its 6th degree, pitted; 6°00' is the 7th.
        let sixth = degree_qualities(5.99);
        assert_eq!((sixth.sign, sixth.degree), ("Aries", 6));
        assert!(sixth.pitted);
        let seventh = degree_qualities(6.0);
        assert_eq!(seventh.degree, 7);
        assert!(!seventh.pitted);
        assert_eq!(degree_qualities(0.0).degree, 1);
        assert_eq!(degree_qualities(29.999).degree, 30);
    }

    #[test]
    fn runs_end_on_their_last_degree() {
        // Aries: masculine to the 8th, feminine the 9th; dark to the 3rd, then light.
        assert_eq!(degree_qualities(7.5).gender, "masculine");
        assert_eq!(degree_qualities(8.5).gender, "feminine");
        assert_eq!(degree_qualities(9.0).gender, "masculine");
        assert_eq!(degree_qualities(2.9).light, "dark");
        assert_eq!(degree_qualities(3.0).light, "light");
    }

    #[test]
    fn longitudes_wrap() {
        assert_eq!(degree_qualities(360.0), degree_qualities(0.0));
        assert_eq!(degree_qualities(-0.5).sign, "Pisces");
        assert_eq!(degree_qualities(-0.5).degree, 30);
        assert_eq!(degree_qualities(725.0), degree_qualities(5.0));
    }

    #[test]
    fn spot_checks() {
        // Taurus 6-10 are azimene; 3 increases fortune.
        let taurus_8th = degree_qualities(37.5);
        assert!(taurus_8th.azimene && !taurus_8th.pitted);
        assert_eq!(taurus_8th.light, "void");
        assert!(degree_qualities(32.0).fortune);
        // Leo 28: pitted and azimene, light.
        let leo_28th = degree_qualities(147.2);
        assert_eq!(leo_28th.sign, "Leo");
        assert!(leo_28th.pitted && leo_28th.azimene);
        assert_eq!(leo_28th.light, "light");
        // Capricorn 11-15 smoky, 12-14 increase fortune.
        let cap_13th = degree_qualities(282.5);
        assert_eq!((cap_13th.light, cap_13th.fortune), ("smoky", true));
        assert_eq!(cap_13th.gender, "feminine");
        // Aquarius ends feminine and light.
        let last = degree_qualities(329.9);
        assert_eq!((last.gender, last.light), ("feminine", "light"));
    }
}
