//! Vedic Panchanga calculations (Five Angas: Tithi, Vara, Nakshatra, Yoga, Karana, plus Samvat).

use serde::{Deserialize, Serialize};
use super::nakshatra::{nakshatra_info, NakshatraInfo};
use crate::instant::UtcInstant;

/// The 16 Tithi names (15 Shukla, 15 Krishna).
pub const TITHI_NAMES: [&str; 16] = [
    "Pratipada", "Dvitiya", "Tritiya", "Chaturthi", "Panchami", "Shashthi", "Saptami",
    "Ashtami", "Navami", "Dashami", "Ekadashi", "Dwadashi", "Trayodashi", "Chaturdashi",
    "Purnima", "Amavasya",
];

/// The 7 Vara (weekday) names and lords.
pub const VARA_NAMES: [(&str, &str); 7] = [
    ("Sunday", "Sun"),
    ("Monday", "Moon"),
    ("Tuesday", "Mars"),
    ("Wednesday", "Mercury"),
    ("Thursday", "Jupiter"),
    ("Friday", "Venus"),
    ("Saturday", "Saturn"),
];

/// The 27 Yoga names.
pub const YOGA_NAMES: [&str; 27] = [
    "Vishkambha", "Priti", "Ayushman", "Saubhagya", "Shobhana", "Atiganda", "Sukarman",
    "Dhriti", "Shula", "Ganda", "Vriddhi", "Dhruva", "Vyaghata", "Harshana", "Vajra",
    "Siddhi", "Vyatipata", "Variyan", "Parigha", "Shiva", "Siddha", "Sadhya", "Shubha",
    "Shukla", "Brahma", "Indra", "Vaidhriti",
];

/// The 11 Karana names (7 movable, 4 fixed).
pub const KARANA_NAMES: [&str; 11] = [
    "Bava", "Balava", "Kaulava", "Taitila", "Garaja", "Vanija", "Vishti",
    "Shakuni", "Chatushpada", "Naga", "Kimstughna",
];

/// The 60-year Jovian Samvatsara cycle names.
pub const SAMVAT_NAMES: [&str; 60] = [
    "Prabhava", "Vibhava", "Shukla", "Pramoda", "Prajapati", "Angirasa", "Shrimukha", "Bhava",
    "Yuvan", "Dhatri", "Ishvara", "Bahudhanya", "Pramathi", "Vikrama", "Vrisha", "Chitrabhanu",
    "Subhanu", "Tarana", "Parthiva", "Vyaya", "Sarvajit", "Sarvadharin", "Virodhi", "Vikruti",
    "Khara", "Nandana", "Vijaya", "Jaya", "Manmatha", "Durmukhi", "Hevilambi", "Vilambi",
    "Vikari", "Sharvari", "Plava", "Shubhakrit", "Shobhana", "Krodhi", "Vishvavasu", "Parabhava",
    "Plavanga", "Kilaka", "Saumya", "Sadharana", "Virodhikrit", "Paridhavin", "Pramadicha",
    "Ananda", "Rakshasa", "Nala", "Pingala", "Kalayukti", "Siddharthi", "Raudra", "Durmati",
    "Dundubhi", "Rudhirodgarin", "Raktakshi", "Krodhana", "Akshaya",
];

/// The 12 Vedic Lunar Month (Masa) names.
pub const LUNAR_MONTH_NAMES: [&str; 12] = [
    "Chaitra", "Vaisakha", "Jyeshtha", "Ashadha", "Shravana", "Bhadrapada",
    "Ashwina", "Kartika", "Margashirsha", "Pausha", "Magha", "Phalguna",
];

/// The 6 Vedic Seasons (Ritu).
pub const RITU_NAMES: [&str; 6] = [
    "Vasanta", "Grishma", "Varsha", "Sharad", "Hemanta", "Shishira",
];

/// Tithi (lunar day) information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TithiInfo {
    /// 1-based index (1..30).
    pub number: u8,
    /// Traditional name (e.g. "Pratipada (Shukla)", "Purnima", "Amavasya").
    pub name: String,
    /// Fortnight: "Shukla" (waxing) or "Krishna" (waning).
    pub paksha: String,
    /// Percentage of the tithi remaining.
    pub percent_left: f64,
}

/// Vara (solar weekday) information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaraInfo {
    /// 0-based civil weekday index (0 = Sunday .. 6 = Saturday).
    pub number: u8,
    /// Name of the civil weekday (e.g. "Saturday").
    pub name: String,
    /// Planetary lord governing the civil weekday.
    pub lord: String,
    /// Name of the Vedic weekday from local sunrise (e.g. "Friday" if before sunrise).
    pub vedic_name: String,
    /// Planetary lord governing the Vedic weekday.
    pub vedic_lord: String,
    /// Whether the instant is before local sunrise on this civil day.
    pub is_before_sunrise: bool,
}

/// Yoga (luni-solar angular combination) information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YogaInfo {
    /// 1-based yoga index (1..27).
    pub number: u8,
    /// Classical name (e.g. "Vishkambha", "Priti", ..., "Vaidhriti").
    pub name: String,
    /// Percentage of the yoga remaining.
    pub percent_left: f64,
}

/// Karana (half-tithi) information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KaranaInfo {
    /// 1-based karana index in the 60-karana lunation cycle (1..60).
    pub number: u8,
    /// Name of the karana (e.g. "Bava", "Balava", "Kimstughna", etc.).
    pub name: String,
    /// Percentage of the karana remaining.
    pub percent_left: f64,
}

/// Vedic Lunar Month (Masa), Paksha, Season (Ritu), and Sun course (Ayana).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MasaInfo {
    /// Amanta lunar month name (month reckoned from New Moon to New Moon).
    pub amanta: String,
    /// Purnimanta lunar month name (month reckoned from Full Moon to Full Moon).
    pub purnimanta: String,
    /// Fortnight: "Shukla" (waxing) or "Krishna" (waning).
    pub paksha: String,
    /// Season (Ritu): Vasanta, Grishma, Varsha, Sharad, Hemanta, Shishira.
    pub ritu: String,
    /// Sun's course: "Uttarayana" (Northward) or "Dakshinayana" (Southward).
    pub ayana: String,
}

/// Hindu Samvatsara calendar years.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamvatInfo {
    /// Kali Yuga elapsed year.
    pub kali_year: i32,
    /// Shaka Samvat era year.
    pub shaka_year: i32,
    /// 60-year Jovian cycle Samvatsara name for Shaka.
    pub shaka_name: String,
    /// Vikram Samvat era year.
    pub vikram_year: i32,
    /// 60-year Jovian cycle Samvatsara name for Vikram.
    pub vikram_name: String,
}

/// Complete Vedic Panchanga result (Five Angas + Masa + Samvat).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanchangResult {
    /// Tithi limb.
    pub tithi: TithiInfo,
    /// Vara limb.
    pub vara: VaraInfo,
    /// Nakshatra limb.
    pub nakshatra: NakshatraInfo,
    /// Percentage of the nakshatra remaining.
    pub nakshatra_percent_left: f64,
    /// Yoga limb.
    pub yoga: YogaInfo,
    /// Karana limb.
    pub karana: KaranaInfo,
    /// Lunar month (Masa), Paksha, Ritu, Ayana.
    pub masa: MasaInfo,
    /// Samvat eras.
    pub samvat: SamvatInfo,
}

/// Compute Panchanga from Sun and Moon sidereal longitudes and Julian Day.
pub fn calc_panchang(
    sun_lon: f64,
    moon_lon: f64,
    jd: f64,
    local_weekday: Option<u8>,
    is_before_sunrise: Option<bool>,
) -> PanchangResult {
    // 1. Tithi: Moon - Sun difference (12° each, 30 tithis total in lunation)
    let diff = (moon_lon - sun_lon).rem_euclid(360.0);
    let tithi_idx = (diff / 12.0).floor() as usize; // 0..29
    let tithi_num = (tithi_idx + 1) as u8;
    let (tithi_name, paksha) = if tithi_num == 15 {
        ("Purnima (Shukla 15)".to_string(), "Shukla")
    } else if tithi_num == 30 {
        ("Amavasya (Krishna 15)".to_string(), "Krishna")
    } else if tithi_num <= 15 {
        (format!("{} (Shukla)", TITHI_NAMES[tithi_idx]), "Shukla")
    } else {
        (format!("{} (Krishna)", TITHI_NAMES[tithi_idx - 15]), "Krishna")
    };
    let tithi_percent_left = (1.0 - (diff % 12.0) / 12.0) * 100.0;

    let tithi = TithiInfo {
        number: tithi_num,
        name: tithi_name,
        paksha: paksha.to_string(),
        percent_left: tithi_percent_left,
    };

    // 2. Vara: Day of the week (Civil and Vedic Sunrise-based)
    let civil_weekday_num = local_weekday.unwrap_or_else(|| {
        ((jd + 1.5).floor() as i64).rem_euclid(7) as u8
    }) % 7;
    let (civil_name, civil_lord) = VARA_NAMES[civil_weekday_num as usize];

    let before_sunrise = is_before_sunrise.unwrap_or(false);
    let vedic_weekday_num = if before_sunrise {
        (civil_weekday_num + 6) % 7 // Vedic day begins at sunrise; previous day still in effect
    } else {
        civil_weekday_num
    };
    let (vedic_name, vedic_lord) = VARA_NAMES[vedic_weekday_num as usize];

    let vara = VaraInfo {
        number: civil_weekday_num,
        name: civil_name.to_string(),
        lord: civil_lord.to_string(),
        vedic_name: vedic_name.to_string(),
        vedic_lord: vedic_lord.to_string(),
        is_before_sunrise: before_sunrise,
    };

    // 3. Nakshatra: from Moon's sidereal longitude
    let nakshatra = nakshatra_info(moon_lon);
    let nakshatra_span = 360.0 / 27.0;
    let nakshatra_percent_left = (1.0 - (moon_lon.rem_euclid(360.0) % nakshatra_span) / nakshatra_span) * 100.0;

    // 4. Yoga: sidereal Sun + sidereal Moon longitude
    let yoga_val = (sun_lon + moon_lon).rem_euclid(360.0) / (360.0 / 27.0);
    let yoga_idx = (yoga_val.floor() as usize).min(26);
    let yoga = YogaInfo {
        number: (yoga_idx + 1) as u8,
        name: YOGA_NAMES[yoga_idx].to_string(),
        percent_left: (1.0 - (yoga_val % 1.0)) * 100.0,
    };

    // 5. Karana: half-tithis (6° each, 60 total)
    let karana_idx = (diff / 6.0).floor() as usize; // 0..59
    let karana_num = (karana_idx + 1) as u8;
    let karana_name = if karana_idx == 0 {
        "Kimstughna"
    } else if karana_idx >= 57 {
        ["Shakuni", "Chatushpada", "Naga"][karana_idx - 57]
    } else {
        KARANA_NAMES[(karana_idx - 1) % 7]
    };
    let karana_percent_left = (1.0 - (diff % 6.0) / 6.0) * 100.0;
    let karana = KaranaInfo {
        number: karana_num,
        name: karana_name.to_string(),
        percent_left: karana_percent_left,
    };

    // 6. Masa: Lunar month (Amanta and Purnimanta), Paksha, Season (Ritu), Ayana
    // Sun moves ~0.98564°/day, Moon-Sun elongation rate ~12.19075°/day.
    // In the time it took Moon to pull ahead by `diff` degrees from New Moon:
    // Sun progress since New Moon = diff * (0.98564 / 12.19075) = diff * 0.0808515°
    let sun_at_nm = (sun_lon - diff * 0.0808515).rem_euclid(360.0);
    let amanta_idx = ((sun_at_nm / 30.0).floor() as usize + 1) % 12;
    let amanta_month = LUNAR_MONTH_NAMES[amanta_idx];
    let is_krishna = tithi_num > 15;
    let purnimanta_idx = (amanta_idx + if is_krishna { 1 } else { 0 }) % 12;
    let purnimanta_month = LUNAR_MONTH_NAMES[purnimanta_idx];

    let sun_sign_idx = ((sun_lon.rem_euclid(360.0) / 30.0).floor() as usize) % 12;
    let ritu_name = RITU_NAMES[sun_sign_idx / 2];

    let norm_sun = sun_lon.rem_euclid(360.0);
    let ayana_name = if norm_sun >= 270.0 || norm_sun < 90.0 {
        "Uttarayana"
    } else {
        "Dakshinayana"
    };

    let masa = MasaInfo {
        amanta: amanta_month.to_string(),
        purnimanta: purnimanta_month.to_string(),
        paksha: paksha.to_string(),
        ritu: ritu_name.to_string(),
        ayana: ayana_name.to_string(),
    };

    // 7. Samvatsara calculations
    let (civil_year, civil_month, ..) = UtcInstant::from_julian_day(jd).civil();
    let samvat_base_year = if civil_month >= 4 {
        civil_year
    } else {
        civil_year - 1
    };
    let kali_year = samvat_base_year + 3101;
    let samvat_idx = (kali_year + 12).rem_euclid(60) as usize;
    let vikram_name = SAMVAT_NAMES[(samvat_idx + 13) % 60];
    let shaka_name = SAMVAT_NAMES[samvat_idx];
    let samvat = SamvatInfo {
        kali_year,
        shaka_year: samvat_base_year - 78,
        shaka_name: shaka_name.to_string(),
        vikram_year: samvat_base_year + 57,
        vikram_name: vikram_name.to_string(),
    };

    PanchangResult {
        tithi,
        vara,
        nakshatra,
        nakshatra_percent_left,
        yoga,
        karana,
        masa,
        samvat,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panchang_limbs() {
        // New moon: Sun=10°, Moon=10° -> Tithi 1 (Shukla Pratipada), Yoga Vishkambha
        let res = calc_panchang(10.0, 10.0, 2451545.0, Some(6), None);
        assert_eq!(res.tithi.number, 1);
        assert_eq!(res.tithi.paksha, "Shukla");
        assert_eq!(res.karana.name, "Kimstughna");
        assert_eq!(res.vara.name, "Saturday");
        assert_eq!(res.vara.lord, "Saturn");
        assert_eq!(res.vara.vedic_name, "Saturday");
        assert_eq!(res.yoga.name, "Priti"); // 20.0 / (360/27) = 1.5 -> index 1 = Priti
        assert_eq!(res.masa.amanta, "Vaisakha");
    }

    #[test]
    fn test_before_sunrise_vara() {
        // Moment before sunrise on Saturday: Vedic day is Friday (Venus)
        let res = calc_panchang(10.0, 10.0, 2451545.0, Some(6), Some(true));
        assert_eq!(res.vara.name, "Saturday");
        assert_eq!(res.vara.lord, "Saturn");
        assert_eq!(res.vara.vedic_name, "Friday");
        assert_eq!(res.vara.vedic_lord, "Venus");
        assert!(res.vara.is_before_sunrise);
    }

    #[test]
    fn test_full_moon_tithi() {
        // Waxing Gibbous approaching Full Moon: Sun=0°, Moon=175° -> Tithi 15 (Purnima)
        let res = calc_panchang(0.0, 175.0, 2451545.0, None, None);
        assert_eq!(res.tithi.number, 15);
        assert_eq!(res.tithi.paksha, "Shukla");
        assert_eq!(res.tithi.name, "Purnima (Shukla 15)");
    }
}
