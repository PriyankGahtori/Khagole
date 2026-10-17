//! Reference tables: fixed stars, lunar mansions, derived-house meanings, degree qualities.

use serde::Serialize;

#[allow(clippy::all)]
#[rustfmt::skip]
mod degree_qualities;
#[allow(clippy::all)]
#[rustfmt::skip]
mod derived_meanings;
#[allow(clippy::all)]
#[rustfmt::skip]
mod fixed_stars;
#[allow(clippy::all)]
#[rustfmt::skip]
mod mansions;

pub use degree_qualities::DEGREE_QUALITIES;
pub use derived_meanings::{DERIVED_HOUSE_MEANINGS, ROOT_HOUSE_THEMES};
pub use fixed_stars::FIXED_STARS;
pub use mansions::LUNAR_MANSIONS;

/// A star with its tropical ecliptic position at J2000.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedStar {
    pub name: &'static str,
    pub symbol: &'static str,
    pub j2000_lon: f64,
    pub j2000_lat: f64,
    pub magnitude: f64,
}

/// One of the 28 lunar mansions (manazil al-qamar).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct LunarMansion {
    /// Mansion number (1-28).
    pub number: u8,
    /// Arabic name.
    pub arabic_name: &'static str,
    /// English name.
    pub name: &'static str,
    /// Sign or signs the mansion spans, e.g. "Aries - Taurus".
    pub sign: &'static str,
    /// Start, ecliptic longitude in degrees.
    pub degree_start: f64,
    /// End, ecliptic longitude in degrees.
    pub degree_end: f64,
}

/// What a derived house signifies, in English and Italian.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Meaning {
    pub en: &'static str,
    pub it: &'static str,
}

/// A run of degrees sharing one quality, from the end of the previous run (or the start of
/// the sign) up to and including `end`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DegreeRun {
    /// "masculine" or "feminine"; or "light", "dark", "smoky" or "void".
    pub quality: &'static str,
    /// Last ordinal degree (1-30) of the run.
    pub end: u8,
}

/// Lilly's qualities of the degrees of one sign. Degrees are ordinal: degree `n` spans
/// `n-1`°00' to `n-1`°59' of the sign.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct SignDegrees {
    /// Zodiac sign name, e.g. "Aries".
    pub sign: &'static str,
    /// Masculine and feminine runs, ending at 30.
    pub gender: &'static [DegreeRun],
    /// Light, dark, smoky and void runs, ending at 30.
    pub light: &'static [DegreeRun],
    /// Deep or pitted degrees.
    pub pitted: &'static [u8],
    /// Lame or deficient (azimene) degrees.
    pub azimene: &'static [u8],
    /// Degrees increasing fortune.
    pub fortune: &'static [u8],
}
