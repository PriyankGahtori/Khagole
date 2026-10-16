//! Zodiac signs, house placement and ayanamsas (`charts/calc/zodiac.py`).

use serde::Serialize;

use crate::pyfloat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sign {
    pub name: &'static str,
    pub symbol: &'static str,
    pub element: &'static str,
    pub quality: &'static str,
}

pub const ZODIAC_SIGNS: [Sign; 12] = [
    Sign {
        name: "Aries",
        symbol: "♈",
        element: "Fire",
        quality: "Cardinal",
    },
    Sign {
        name: "Taurus",
        symbol: "♉",
        element: "Earth",
        quality: "Fixed",
    },
    Sign {
        name: "Gemini",
        symbol: "♊",
        element: "Air",
        quality: "Mutable",
    },
    Sign {
        name: "Cancer",
        symbol: "♋",
        element: "Water",
        quality: "Cardinal",
    },
    Sign {
        name: "Leo",
        symbol: "♌",
        element: "Fire",
        quality: "Fixed",
    },
    Sign {
        name: "Virgo",
        symbol: "♍",
        element: "Earth",
        quality: "Mutable",
    },
    Sign {
        name: "Libra",
        symbol: "♎",
        element: "Air",
        quality: "Cardinal",
    },
    Sign {
        name: "Scorpio",
        symbol: "♏",
        element: "Water",
        quality: "Fixed",
    },
    Sign {
        name: "Sagittarius",
        symbol: "♐",
        element: "Fire",
        quality: "Mutable",
    },
    Sign {
        name: "Capricorn",
        symbol: "♑",
        element: "Earth",
        quality: "Cardinal",
    },
    Sign {
        name: "Aquarius",
        symbol: "♒",
        element: "Air",
        quality: "Fixed",
    },
    Sign {
        name: "Pisces",
        symbol: "♓",
        element: "Water",
        quality: "Mutable",
    },
];

/// A longitude as sign, whole degree and rounded minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZodiacPosition {
    pub sign: Sign,
    pub degree: i64,
    pub minute: i64,
}

/// `longitude_to_zodiac`: minutes are rounded, carrying into the next degree and sign.
pub fn longitude_to_zodiac(lon: f64) -> ZodiacPosition {
    let norm = pyfloat::rem(lon, 360.0);
    let mut sign_index = pyfloat::floordiv(norm, 30.0) as usize;
    let deg_in_sign = norm - (sign_index as f64 * 30.0);
    let mut degree = deg_in_sign as i64;
    let mut minute = pyfloat::round_int((deg_in_sign - degree as f64) * 60.0);
    if minute == 60 {
        minute = 0;
        degree += 1;
        if degree == 30 {
            degree = 0;
            sign_index = (sign_index + 1) % 12;
        }
    }
    ZodiacPosition {
        // `rem` gives 360.0 for a tiny negative longitude: that is Aries 0°.
        sign: ZODIAC_SIGNS[sign_index % 12],
        degree,
        minute,
    }
}

/// House (1-12) containing `lon`, given the twelve cusp longitudes.
pub fn determine_house(lon: f64, cusps: &[f64]) -> u8 {
    if cusps.len() != 12 {
        return 1;
    }
    for h in 0..12 {
        let h1 = cusps[h];
        let h2 = cusps[(h + 1) % 12];
        let inside = if h2 < h1 {
            lon >= h1 || lon < h2
        } else {
            h1 <= lon && lon < h2
        };
        if inside {
            return h as u8 + 1;
        }
    }
    1
}

/// `format_degrees_to_dms`: `DD° MM' SS"`.
pub fn format_degrees_to_dms(value: f64) -> String {
    let norm = pyfloat::rem(pyfloat::rem(value, 360.0) + 360.0, 360.0);
    let mut deg = norm as i64;
    let rem_min = (norm - deg as f64) * 60.0;
    let mut mnt = rem_min as i64;
    let mut sec = pyfloat::round_int((rem_min - mnt as f64) * 60.0);
    if sec == 60 {
        sec = 0;
        mnt += 1;
    }
    if mnt == 60 {
        mnt = 0;
        deg += 1;
    }
    deg = deg.rem_euclid(360);
    format!("{deg}° {mnt:02}' {sec:02}\"")
}

pub const PRECESSION_RATE_ARCSEC_YEAR: f64 = 50.290966;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ayanamsa {
    pub code: &'static str,
    pub name: &'static str,
    /// Degrees at J2000.0.
    pub offset_j2000: f64,
    /// Linear drift on top of general precession, arcseconds per Julian century (systems
    /// tied to a moving reference: Spica's proper motion, the galactic centre).
    pub drift_arcsec_cy: f64,
}

pub const AYANAMSA_CATALOG: [Ayanamsa; 14] = [
    Ayanamsa {
        code: "lahiri",
        name: "Lahiri (Chitra Paksha)",
        offset_j2000: 23.857092,
        drift_arcsec_cy: -0.0016,
    },
    Ayanamsa {
        code: "fagan_bradley",
        name: "Fagan-Bradley",
        offset_j2000: 24.7403,
        drift_arcsec_cy: -0.0015,
    },
    Ayanamsa {
        code: "krishnamurti",
        name: "Krishnamurti (KP)",
        offset_j2000: 23.76024,
        drift_arcsec_cy: -0.0007,
    },
    Ayanamsa {
        code: "raman",
        name: "B.V. Raman",
        offset_j2000: 22.410791,
        drift_arcsec_cy: -0.0007,
    },
    Ayanamsa {
        code: "true_citra",
        name: "True Chitra",
        offset_j2000: 23.840017,
        drift_arcsec_cy: -4.5046,
    },
    Ayanamsa {
        code: "yukteshwar",
        name: "Sri Yukteswar",
        offset_j2000: 22.478803,
        drift_arcsec_cy: -0.0007,
    },
    Ayanamsa {
        code: "deluce",
        name: "De Luce",
        offset_j2000: 27.815753,
        drift_arcsec_cy: -0.0189,
    },
    Ayanamsa {
        code: "ushashashi",
        name: "Usha-Shashi",
        offset_j2000: 20.057541,
        drift_arcsec_cy: -0.0007,
    },
    Ayanamsa {
        code: "jn_bhasin",
        name: "J.N. Bhasin",
        offset_j2000: 22.762137,
        drift_arcsec_cy: -0.0007,
    },
    Ayanamsa {
        code: "aldebaran_15tau",
        name: "Aldebaran 15° Taurus",
        offset_j2000: 24.758924,
        drift_arcsec_cy: -0.0226,
    },
    Ayanamsa {
        code: "hipparchos",
        name: "Hipparchos",
        offset_j2000: 20.247788,
        drift_arcsec_cy: -0.0236,
    },
    Ayanamsa {
        code: "sassanian",
        name: "Sassanian",
        offset_j2000: 19.992959,
        drift_arcsec_cy: -0.0032,
    },
    Ayanamsa {
        code: "galcent_0sag",
        name: "Galactic Center 0° Sagittarius",
        offset_j2000: 26.846048,
        drift_arcsec_cy: -0.2407,
    },
    Ayanamsa {
        code: "j2000",
        name: "J2000.0 Epoch",
        offset_j2000: 0.0,
        drift_arcsec_cy: 0.0,
    },
];

/// Default ayanamsa code ("galcent_0sag").
pub const DEFAULT_AYANAMSA: &str = "galcent_0sag";

/// Catalog entry for a code (case-insensitive), falling back to the default.
pub fn ayanamsa(code: &str) -> &'static Ayanamsa {
    let code = code.trim().to_lowercase();
    AYANAMSA_CATALOG
        .iter()
        .find(|a| a.code == code)
        .or_else(|| AYANAMSA_CATALOG.iter().find(|a| a.code == DEFAULT_AYANAMSA))
        .unwrap()
}

/// Information about an ayanamsa at a specific moment.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AyanamsaInfo {
    /// Ayanamsa code.
    pub code: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Degrees, rounded to 6 decimals.
    pub value: f64,
    /// Degrees in D°MM'SS" format.
    pub formatted: String,
    /// Precession rate in arcseconds per year.
    pub precession_rate_arcsec_yr: f64,
}

/// IAU 2006 general precession in longitude p_A, arcseconds, `t` Julian centuries after
/// J2000.0 (Capitaine, Wallace & Chapront 2003, eq. 39).
pub fn general_precession_arcsec(t: f64) -> f64 {
    ((((-0.0000000383 * t - 0.000023857) * t + 0.00007964) * t + 1.1054348) * t + 5028.796195) * t
}

/// Ayanamsa in degrees at a UT Julian day: the system's J2000 value plus general
/// precession plus its own drift. Tracks Swiss Ephemeris within 0.2" over 1450-2150.
pub fn ayanamsa_value(jd: f64, code: &str) -> f64 {
    let meta = ayanamsa(code);
    let t = (jd - 2_451_545.0) / 36525.0;
    let arcsec = general_precession_arcsec(t) + meta.drift_arcsec_cy * t;
    pyfloat::rem(meta.offset_j2000 + arcsec / 3600.0, 360.0)
}

/// `get_ayanamsa_info`: value rounded to 6 decimals, and as a DMS string.
pub fn ayanamsa_info(jd: f64, code: &str) -> AyanamsaInfo {
    let meta = ayanamsa(code);
    let value = ayanamsa_value(jd, code);
    AyanamsaInfo {
        code: meta.code,
        name: meta.name,
        value: pyfloat::round(value, 6),
        formatted: format_degrees_to_dms(value),
        precession_rate_arcsec_yr: pyfloat::round(PRECESSION_RATE_ARCSEC_YEAR, 4),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minute_rounding_carries_into_next_sign() {
        let p = longitude_to_zodiac(29.9999);
        assert_eq!((p.sign.name, p.degree, p.minute), ("Taurus", 0, 0));
        let p = longitude_to_zodiac(-0.5);
        assert_eq!((p.sign.name, p.degree, p.minute), ("Pisces", 29, 30));
    }

    #[test]
    fn ayanamsa_includes_general_precession() {
        let j2000 = 2_451_545.0;
        assert_eq!(ayanamsa_info(j2000, "lahiri").value, 23.857092);
        assert_eq!(ayanamsa_info(j2000 + 36525.0, "lahiri").value, 25.254286);
        assert_eq!(ayanamsa_info(j2000 - 36525.0, "lahiri").value, 22.460512);
        assert_eq!(ayanamsa_info(j2000, "j2000").value, 0.0);
        assert_eq!(ayanamsa("unknown").code, DEFAULT_AYANAMSA);
    }

    #[test]
    fn house_placement_wraps_at_aries() {
        let cusps: Vec<f64> = (0..12).map(|i| (350.0 + 30.0 * i as f64) % 360.0).collect();
        assert_eq!(determine_house(355.0, &cusps), 1);
        assert_eq!(determine_house(5.0, &cusps), 1);
        assert_eq!(determine_house(20.0, &cusps), 2);
    }
}
