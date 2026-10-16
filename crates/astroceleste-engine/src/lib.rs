//! astroceleste-engine: astrological chart calculation on JPL ephemerides.
//!
//! One implementation shared by the Astroceleste server (Python bindings), desktop and
//! mobile apps (native) and the web app (WASM), so every platform computes identical charts.
//!
//! **Experimental (0.0.x):** the API may change in any release.
//!
//! ```no_run
//! use astroceleste_engine::ephemeris::{Kernel, KernelSet, Spk};
//! use astroceleste_engine::{calculate_chart, ChartRequest, UtcInstant};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Kernels in preference order: the first one covering a date is used.
//! let mut kernels = KernelSet::new();
//! kernels.push(Kernel::new("de440s.bsp", Spk::open("kernels/de440s.bsp")?)?);
//!
//! let mut request = ChartRequest::new(UtcInstant::parse("1987-05-17T14:30:00Z")?, 41.9, 12.5);
//! request.house_system = "P";
//! request.zodiac_type = "sidereal";
//! request.ayanamsa = "lahiri";
//!
//! let chart = calculate_chart(&kernels, &request)?;
//! println!("{}", serde_json::to_string_pretty(&chart)?);
//! # Ok(())
//! # }
//! ```
//!
//! # Entry points
//!
//! | Function | Result |
//! |---|---|
//! | [`calculate_chart`] | a natal or event [`Chart`]: planets (the seven with their [`Condition`]), houses, aspects, fixed stars, lots, temperament, lunar status, sect, [`Reception`]s, antiscia, planetary hours |
//! | [`calculate_horary_chart`] | the chart plus [`HoraryData`]: planetary hours, the Moon's aspects, strictures and the [`Judgment`] (significators, perfection, prohibition, translation, collection) |
//! | [`calculate_transit_chart`] | the sky at a moment and place, with its [`CrossAspect`]s to natal planets |
//! | [`calculate_synastry`] | cross-aspects between two charts' planets (no kernel needed) |
//! | [`calculate_derived_chart`] | a stored chart turned to a new first house (no kernel needed) |
//! | [`calculate_election_chart`] | the chart plus [`ElectionData`]: an electional score with the rules that apply |
//! | [`search_elections`] | the best [`ElectionWindow`]s over a span of time at a place |
//! | [`degree_qualities`] | Lilly's [`DegreeQualities`] of the degree a longitude falls in (no kernel needed) |
//! | [`chart_dignities`] | a computed chart's [`Condition`]s and [`Reception`]s judged again under the Lilly or Dorothean scheme (no kernel needed) |
//! | [`time_lords`] | the annual [`Profection`]s and [`Firdaria`] of a nativity over a span of time (no kernel needed) |
//! | [`chinese_calendar`] | the [`ChineseCalendar`] around a moment: solar terms, lunar months, equation of time (also on a chart, with [`ChartRequest::chinese_calendar`]) |
//! | [`bazi`] | the Four Pillars ([`Bazi`]) of a birth from its Chinese calendar: hidden stems, Ten Gods, NaYin, element balance, luck pillars (no kernel needed) |
//! | [`zi_wei`] | the Zi Wei Dou Shu chart ([`ZiWei`]) of a birth from its Chinese calendar: palaces, bureau, the 14 major and 14 auxiliary stars with brightness, the Four Transformations, minor stars and cycles of gods, flying transformations, decade and small limits and a year's horoscope (no kernel needed) |
//!
//! Every result implements [`serde::Serialize`] and serializes to the JSON of the
//! Astroceleste API, with the same key order and the same integer vs float types. The
//! Python and WebAssembly bindings return exactly that JSON.
//!
//! # Loading kernels
//!
//! Positions come from NASA JPL SPK kernels (`de440s.bsp` covers 1849–2150; DE441 covers
//! 13200 BC–17191). A [`KernelSet`](ephemeris::KernelSet) holds them in preference order,
//! and each date is computed with the first kernel that covers it. [`Spk::open`] reads a
//! file. Where there is no file system (WebAssembly) or the kernel is bundled with an app,
//! load the bytes and use [`Spk::from_bytes`]. [`Spk::excerpt`] cuts a smaller kernel for a
//! date range, with positions unchanged inside it (1950–2050 of DE440s is about 11 MB).
//!
//! ```no_run
//! use astroceleste_engine::ephemeris::{Kernel, KernelSet, Spk};
//!
//! # fn download(_: &str) -> Vec<u8> { Vec::new() }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let bytes: Vec<u8> = download("https://example.com/de440s-1950-2050.bsp");
//! let mut kernels = KernelSet::new();
//! kernels.push(Kernel::new("de440s-1950-2050.bsp", Spk::from_bytes(bytes)?)?);
//! assert!(kernels.coverage().is_some());
//! # Ok(())
//! # }
//! ```
//!
//! # Errors
//!
//! Calculations return [`EngineError`], whose [`code`](EngineError::code) is the stable
//! error code of the Astroceleste API. A date that no loaded kernel covers is
//! [`EngineError::OutOfRange`]: the engine never extrapolates or approximates.
//!
//! ```no_run
//! # use astroceleste_engine::ephemeris::KernelSet;
//! # use astroceleste_engine::{calculate_chart, ChartRequest, EngineError, UtcInstant};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let kernels = KernelSet::new();
//! let request = ChartRequest::new(UtcInstant::parse("1700-01-01T00:00:00Z")?, 41.9, 12.5);
//! match calculate_chart(&kernels, &request) {
//!     Ok(chart) => println!("{} planets", chart.planets.len()),
//!     Err(EngineError::OutOfRange { jd, coverage }) => {
//!         eprintln!("JD {jd} is outside the loaded kernels ({coverage:?})")
//!     }
//!     Err(err) => eprintln!("{}: {err}", err.code()),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Platforms
//!
//! The crate is pure Rust with no C code, and depends only on `serde` and `serde_json`.
//! It builds for servers and desktops, `wasm32-unknown-unknown`, Android and iOS. Python
//! (`pip install astroceleste-engine`) and JavaScript (`npm install astroceleste-engine`)
//! bindings are published from the same repository.
//!
//! # Further reading
//!
//! - [API guide](https://github.com/ffalcinelli/astroceleste-engine/blob/main/docs/api.md):
//!   request options (house systems, ayanamsas, orb settings) and the chart JSON
//! - [Ephemerides](https://github.com/ffalcinelli/astroceleste-engine/blob/main/docs/ephemerides.md):
//!   kernels, coverage and excerpts
//! - [Accuracy](https://github.com/ffalcinelli/astroceleste-engine/blob/main/docs/accuracy.md):
//!   how results are verified against the reference implementation
//! - [Live demo](https://ffalcinelli.github.io/astroceleste-engine/): this crate compiled
//!   to WebAssembly, computing charts in your browser
//!
//! [`Spk::open`]: ephemeris::Spk::open
//! [`Spk::from_bytes`]: ephemeris::Spk::from_bytes
//! [`Spk::excerpt`]: ephemeris::Spk::excerpt

mod almanac;
mod aspects;
mod catalog;
mod chart;
mod chinese;
mod chiron;
mod constants;
mod degree_qualities;
mod derived;
mod dignities;
mod election;
pub mod ephemeris;
mod error;
mod fixed_stars;
#[doc(hidden)] // exposed for the reduction tests; not part of the API
pub mod frames;
mod horary;
mod houses;
mod instant;
mod judgment;
mod lots;
mod lunar;
mod planets;
mod pyfloat;
mod symbolic;
mod temperament;
#[cfg(test)]
mod test_kernel;
#[doc(hidden)] // exposed for the reduction tests; not part of the API
pub mod time;
mod time_lords;
pub mod vedic;
mod zodiac;

pub use aspects::{Aspect, CrossAspect};
pub use catalog::{DegreeRun, LunarMansion, SignDegrees};
pub use chart::{calculate_chart, Chart, ChartRequest, HouseCusp, Placement};
pub use chinese::bazi::{
    bazi, Bazi, BaziFlow, BaziOptions, ElementBalance, HiddenStem, Luck, LuckPillar, LunarDate,
    Pillar, TermMoment,
};
pub use chinese::calendar::{chinese_calendar, ChineseCalendar, LunarMonth, SolarTerm};
pub use chinese::ziwei::{
    zi_wei, AgeRange, Bureau, ZiWei, ZiWeiFlowStar, ZiWeiGod, ZiWeiGods, ZiWeiHoroscope,
    ZiWeiOptions, ZiWeiPalace, ZiWeiPeriod, ZiWeiStar, ZiWeiTransformation,
};
pub use degree_qualities::{degree_qualities, degree_quality_table, DegreeQualities};
pub use derived::{
    calculate_derived_chart, calculate_synastry, calculate_transit_chart, Synastry, TransitChart,
};
pub use dignities::{
    chart_dignities, AntisciaContact, ChartDignities, Condition, DignityLords, DignityScheme,
    EssentialDignity, PlanetCondition, Reception,
};
pub use election::{
    calculate_election_chart, search_elections, CriteriaSummary, ElectionChart, ElectionCriteria,
    ElectionData, ElectionFactor, ElectionSearch, ElectionWindow, Exclusions, HourRange,
    NatalPoint, Purpose, UtcOffset, MAX_SEARCH_DAYS,
};
pub use error::EngineError;
pub use fixed_stars::FixedStarPosition;
pub use horary::{
    calculate_horary_chart, ApplyingAspect, HoraryChart, HoraryData, MoonStatus, PlanetaryHours,
    SeparatingAspect, Stricture,
};
pub use houses::{houses_at_sidereal_time, HouseSystem, Houses};
pub use instant::{ParseError, UtcInstant};
pub use judgment::{Collection, Judgment, Perfection, Prohibition, Refranation, Translation};
pub use lots::Lot;
pub use lunar::LunarStatus;
pub use planets::{planets_and_sidereal_time_with_options, BodyPosition, Planets};
pub use temperament::{Factor, Qualities, Scores, Temperament};
pub use time_lords::{time_lords, Firdaria, Period, Profection, TimeLords, MAX_TIME_LORDS_YEARS};
pub use vedic::{
    advance_tssy, calc_divisional_chart, calc_divisional_point, calc_panchang,
    calc_vimshottari_dasha, calc_vimshottari_dasha_with_sun, find_solar_return, nakshatra_info,
    DashaNode, DivisionalChart, DivisionalPlanet, DivisionalPosition, KaranaInfo, NakshatraInfo,
    PanchangResult, SamvatInfo, TithiInfo, VaraInfo, YearType, YogaInfo,
};
pub use zodiac::{ayanamsa, ayanamsa_info, AyanamsaInfo, DEFAULT_AYANAMSA};
