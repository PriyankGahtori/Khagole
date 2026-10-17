//! The Chinese calendar around a moment: the solar terms, the lunar months and the
//! equation of time, which Ba Zi and Zi Wei Dou Shu are cast from.
//!
//! The rules are those of the modern calendar: true solar terms and true new moons (in
//! force since 1645), dated in China Standard Time (UTC+8) from 1929 and on the Beijing
//! meridian (116°25′ E) before. They are applied to every date, so dates before 1645
//! follow them proleptically rather than the mean-term calendars then in use.
//!
//! A lunar month starts on the day (China time) of a new moon. Month 11 is the month that
//! holds the winter solstice. When 13 months run from one month 11 to the next, the first
//! of them without a major term (中氣) is intercalary and repeats the number before it.

use serde::{Deserialize, Serialize};

use crate::constants::TAU;
use crate::ephemeris::observe::{apparent, ecliptic_latlon, observe};
use crate::ephemeris::{KernelSet, SpkError};
use crate::error::EngineError;
use crate::frames::nutation::iau2000b_radians;
use crate::frames::{mxv, Orientation};
use crate::instant::UtcInstant;
use crate::planets::require_kernel;
use crate::pyfloat::wrap180;
use crate::time::Time;

const SUN: i32 = 10;
const MOON: i32 = 301;
const TROPICAL_YEAR: f64 = 365.242_19;
const SYNODIC_MONTH: f64 = 29.530_589;
/// Mean motion of the Sun, degrees per day.
const SUN_RATE: f64 = 360.0 / TROPICAL_YEAR;
/// Mean motion of the Moon away from the Sun, degrees per day.
const ELONGATION_RATE: f64 = 360.0 / SYNODIC_MONTH;
/// Search precision, days (under 0.01 s).
const EPSILON_DAYS: f64 = 1e-7;
/// Julian date of the Unix epoch.
const UNIX_EPOCH_JD: f64 = 2_440_587.5;
/// Julian day number of 1970-01-01.
const UNIX_EPOCH_JDN: i64 = 2_440_588;
const MICROS_PER_DAY: i64 = 86_400_000_000;
/// Births this close (days) to a lunar month boundary may fall either side of it once
/// converted to the local civil or solar date, so the months around are kept too.
const DATE_MARGIN: i64 = 3;

/// The 24 solar terms by the Sun's longitude / 15°, from the spring equinox.
const TERM_NAMES: [&str; 24] = [
    "chunfen",
    "qingming",
    "guyu",
    "lixia",
    "xiaoman",
    "mangzhong",
    "xiazhi",
    "xiaoshu",
    "dashu",
    "liqiu",
    "chushu",
    "bailu",
    "qiufen",
    "hanlu",
    "shuangjiang",
    "lidong",
    "xiaoxue",
    "daxue",
    "dongzhi",
    "xiaohan",
    "dahan",
    "lichun",
    "yushui",
    "jingzhe",
];

/// The longitude of 立春, the Beginning of Spring, which opens the year of the pillars.
pub(crate) const LICHUN: u16 = 315;

/// One of the 24 solar terms (節氣): the moment the Sun reaches a multiple of 15°.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolarTerm {
    /// Pinyin code, e.g. "lichun".
    pub name: String,
    /// The Sun's apparent ecliptic longitude, degrees (a multiple of 15).
    pub longitude: u16,
    /// A "jie" (節) term, which opens a solar month (longitudes 15° + 30°·k); otherwise a
    /// major term (中氣, multiples of 30°).
    pub jie: bool,
    /// The moment, ISO 8601 UTC to the second.
    pub utc: String,
}

/// A month of the lunisolar calendar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LunarMonth {
    /// The lunar year: the Gregorian year its New Year falls in.
    pub year: i32,
    /// Month number, 1-12.
    pub month: u8,
    /// Whether it is the intercalary month (閏月) repeating `month`.
    pub leap: bool,
    /// First day, `YYYY-MM-DD` (China time).
    pub start: String,
    /// Length in days: 29 or 30.
    pub days: u8,
}

/// The Chinese calendar around a moment: what the Four Pillars and the Purple Star chart
/// need from the ephemeris.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChineseCalendar {
    /// Apparent minus mean solar time at the moment, minutes.
    pub equation_of_time: f64,
    /// Every solar term from the last 立春 at or before the moment to the first jie term
    /// after it.
    pub solar_terms: Vec<SolarTerm>,
    /// The lunar months around the date of the moment (a few days either side).
    pub lunar_months: Vec<LunarMonth>,
}

impl SolarTerm {
    /// The moment of the term.
    pub(crate) fn instant(&self) -> Result<UtcInstant, EngineError> {
        UtcInstant::parse(&self.utc).map_err(|e| EngineError::InvalidInput(e.to_string()))
    }
}

impl LunarMonth {
    /// Julian day number of the first day.
    pub(crate) fn start_day(&self) -> Result<i64, EngineError> {
        let start = UtcInstant::parse(&format!("{}T00:00:00Z", self.start))
            .map_err(|e| EngineError::InvalidInput(e.to_string()))?;
        Ok(day_number(start))
    }
}

/// Julian day number of the UTC date of `instant`.
pub(crate) fn day_number(instant: UtcInstant) -> i64 {
    instant.micros().div_euclid(MICROS_PER_DAY) + UNIX_EPOCH_JDN
}

/// `YYYY-MM-DD` of a Julian day number.
fn date_of(day: i64) -> String {
    let (y, m, d, ..) = UtcInstant::from_micros((day - UNIX_EPOCH_JDN) * MICROS_PER_DAY).civil();
    format!("{y:04}-{m:02}-{d:02}")
}

/// The instant of a UT Julian date, to the second.
fn instant_of(jd: f64) -> UtcInstant {
    let seconds = ((jd - UNIX_EPOCH_JD) * 86_400.0).round() as i64;
    UtcInstant::from_micros(seconds * 1_000_000)
}

/// Offset of China time from UT, days: UTC+8 from 1929, the Beijing meridian before.
fn china_offset(jd: f64) -> f64 {
    // 1929-01-01T00:00 at UTC+8.
    const CST_SINCE_JD: f64 = 2_425_612.5 - 8.0 / 24.0;
    if jd < CST_SINCE_JD {
        (116.0 + 25.0 / 60.0) / 360.0
    } else {
        8.0 / 24.0
    }
}

/// Julian day number of the date in China at UT Julian date `jd`.
fn china_day(jd: f64) -> i64 {
    (jd + 0.5 + china_offset(jd)).floor() as i64
}

/// Apparent geocentric positions for the searches.
struct Sky<'a> {
    kernels: &'a KernelSet,
}

impl Sky<'_> {
    /// Apparent ecliptic longitude of date of `target` at UT Julian date `jd`, degrees.
    fn longitude(&self, target: i32, jd: f64) -> Result<f64, EngineError> {
        let kernel = require_kernel(self.kernels, jd)?;
        let t = Time::from_ut1(jd);
        // The faster IAU 2000B nutation: it moves a term by well under a second.
        let orientation = Orientation::with_nutation(&t, iau2000b_radians(t.tt()));
        let position = apparent(kernel, &observe(kernel, target, &t)?, &t)?;
        Ok(ecliptic_latlon(&orientation, &position).1)
    }

    fn sun(&self, jd: f64) -> Result<f64, EngineError> {
        self.longitude(SUN, jd)
    }

    fn elongation(&self, jd: f64) -> Result<f64, EngineError> {
        Ok(self.longitude(MOON, jd)? - self.sun(jd)?)
    }

    /// When the Sun reaches `longitude`, starting from `guess` (secant method).
    fn sun_at(&self, longitude: f64, guess: f64) -> Result<f64, EngineError> {
        crossing(|jd| self.sun(jd), longitude, guess, SUN_RATE)
    }

    /// The new moon nearest `guess`.
    fn new_moon_near(&self, guess: f64) -> Result<f64, EngineError> {
        crossing(|jd| self.elongation(jd), 0.0, guess, ELONGATION_RATE)
    }

    /// The last moment at or before `jd` when the Sun was at `longitude`.
    fn sun_before(&self, longitude: f64, jd: f64) -> Result<f64, EngineError> {
        let behind = (self.sun(jd)? - longitude).rem_euclid(360.0);
        let found = self.sun_at(longitude, jd - behind / SUN_RATE)?;
        if found > jd {
            self.sun_at(longitude, found - TROPICAL_YEAR)
        } else {
            Ok(found)
        }
    }

    /// The last new moon at or before `jd`.
    fn new_moon_before(&self, jd: f64) -> Result<f64, EngineError> {
        let behind = self.elongation(jd)?.rem_euclid(360.0);
        let found = self.new_moon_near(jd - behind / ELONGATION_RATE)?;
        if found > jd {
            self.new_moon_near(found - SYNODIC_MONTH)
        } else {
            Ok(found)
        }
    }

    /// The new moon that starts the month holding the China-time day of the winter
    /// solstice `solstice`: month 11.
    fn month_eleven(&self, solstice: f64) -> Result<f64, EngineError> {
        let solstice_day = china_day(solstice);
        let moon = self.new_moon_before(solstice + 1.0)?;
        if china_day(moon) > solstice_day {
            self.new_moon_before(moon - 1.0)
        } else {
            Ok(moon)
        }
    }

    /// The months of the year (歲) from the month 11 holding `solstice` to the next
    /// month 11, with the first day of that next month 11 and the next solstice.
    fn sui(&self, solstice: f64) -> Result<(Vec<Month>, i64, f64), EngineError> {
        let next_solstice = self.sun_at(270.0, solstice + TROPICAL_YEAR)?;
        let end = china_day(self.month_eleven(next_solstice)?);

        let mut moon = self.month_eleven(solstice)?;
        let mut starts = vec![china_day(moon)];
        loop {
            moon = self.new_moon_near(moon + SYNODIC_MONTH)?;
            let day = china_day(moon);
            if day >= end {
                break;
            }
            starts.push(day);
            if starts.len() > 13 {
                return Err(diverged("a year of more than 13 months"));
            }
        }

        // The major terms: the solstice and the eleven after it (the next solstice
        // already starts the next year).
        let mut major_days = vec![china_day(solstice)];
        let mut term = solstice;
        for k in 1..12 {
            term = self.sun_at(f64::from((270 + 30 * k) % 360), term + 30.44)?;
            major_days.push(china_day(term));
        }

        let (first_year, ..) =
            UtcInstant::from_micros((starts[0] - UNIX_EPOCH_JDN) * MICROS_PER_DAY).civil();
        let mut ends: Vec<i64> = starts[1..].to_vec();
        ends.push(end);
        let mut leap_pending = starts.len() == 13;
        let mut number = 11u8;
        let mut year = first_year;
        let mut months = Vec::with_capacity(starts.len());
        for (i, (&start, &month_end)) in starts.iter().zip(&ends).enumerate() {
            let has_major = major_days.iter().any(|&d| start <= d && d < month_end);
            let leap = i > 0 && leap_pending && !has_major;
            if leap {
                leap_pending = false;
            } else if i > 0 {
                number = number % 12 + 1;
                if number == 1 {
                    year = first_year + 1;
                }
            }
            months.push(Month {
                start,
                end: month_end,
                number,
                leap,
                year,
            });
        }
        Ok((months, end, next_solstice))
    }

    /// Equation of time at `jd`, minutes: apparent minus mean solar time.
    fn equation_of_time(&self, jd: f64) -> Result<f64, EngineError> {
        let kernel = require_kernel(self.kernels, jd)?;
        let t = Time::from_ut1(jd);
        let orientation = Orientation::with_nutation(&t, iau2000b_radians(t.tt()));
        let position = apparent(kernel, &observe(kernel, SUN, &t)?, &t)?;
        let equatorial = mxv(&orientation.m, &position);
        let ra_hours = equatorial[1].atan2(equatorial[0]).rem_euclid(TAU) * 24.0 / TAU;
        let apparent_hours = orientation.gast_hours - ra_hours + 12.0;
        let mean_hours = (jd + 0.5).rem_euclid(1.0) * 24.0;
        Ok(((apparent_hours - mean_hours + 12.0).rem_euclid(24.0) - 12.0) * 60.0)
    }
}

/// A lunar month, by Julian day numbers.
struct Month {
    start: i64,
    /// First day of the next month.
    end: i64,
    number: u8,
    leap: bool,
    year: i32,
}

/// The moment near `guess` when the angle `f` (degrees, growing at about `rate` degrees
/// per day) equals `target`.
/// A search the ephemeris cannot satisfy: only a corrupt kernel gets here.
fn diverged(what: &str) -> EngineError {
    EngineError::Ephemeris(SpkError::Format(format!("Chinese calendar search: {what}")))
}

fn crossing(
    f: impl Fn(f64) -> Result<f64, EngineError>,
    target: f64,
    guess: f64,
    rate: f64,
) -> Result<f64, EngineError> {
    let mut t0 = guess;
    let mut d0 = wrap180(f(t0)? - target);
    let mut t1 = t0 - d0 / rate;
    for _ in 0..50 {
        if (t1 - t0).abs() < EPSILON_DAYS {
            break;
        }
        let d1 = wrap180(f(t1)? - target);
        let slope = (d1 - d0) / (t1 - t0);
        // The angle always grows near the crossing; fall back to the mean rate when the
        // secant says otherwise (a wrap between the samples).
        let slope = if slope.is_finite() && slope > rate / 4.0 {
            slope
        } else {
            rate
        };
        (t0, d0) = (t1, d1);
        t1 -= d1 / slope;
    }
    Ok(t1)
}

/// The Chinese calendar around `instant`, or `None` when the loaded kernels do not cover
/// the year or so of ephemeris it needs (about 14 months before the moment and 2 after).
pub fn chinese_calendar(
    kernels: &KernelSet,
    instant: UtcInstant,
) -> Result<Option<ChineseCalendar>, EngineError> {
    match calendar(kernels, instant) {
        Err(EngineError::OutOfRange { .. }) => Ok(None),
        other => other.map(Some),
    }
}

fn calendar(kernels: &KernelSet, instant: UtcInstant) -> Result<ChineseCalendar, EngineError> {
    let sky = Sky { kernels };
    let jd = instant.julian_day();

    let mut solar_terms = Vec::new();
    let mut term = sky.sun_before(f64::from(LICHUN), jd)?;
    let mut longitude = LICHUN;
    loop {
        let jie = longitude % 30 == 15;
        solar_terms.push(SolarTerm {
            name: TERM_NAMES[usize::from(longitude / 15)].to_string(),
            longitude,
            jie,
            utc: instant_of(term).isoformat(),
        });
        if jie && term > jd {
            break;
        }
        if solar_terms.len() > 48 {
            return Err(diverged("no solar term after the moment"));
        }
        longitude = (longitude + 15) % 360;
        term = sky.sun_at(f64::from(longitude), term + 15.2)?;
    }

    let day = china_day(jd);
    let solstice = sky.sun_before(270.0, jd)?;
    let (mut months, mut end, mut next_solstice) = sky.sui(solstice)?;
    if day - DATE_MARGIN < months[0].start {
        let (mut earlier, ..) = sky.sui(sky.sun_at(270.0, solstice - TROPICAL_YEAR)?)?;
        earlier.append(&mut months);
        months = earlier;
    }
    while day + DATE_MARGIN >= end {
        if months.len() > 40 {
            return Err(diverged("no lunar year after the moment"));
        }
        let (mut later, later_end, later_solstice) = sky.sui(next_solstice)?;
        months.append(&mut later);
        (end, next_solstice) = (later_end, later_solstice);
    }
    let lunar_months = months
        .into_iter()
        .filter(|m| m.start <= day + DATE_MARGIN && m.end > day - DATE_MARGIN)
        .map(|m| LunarMonth {
            year: m.year,
            month: m.number,
            leap: m.leap,
            start: date_of(m.start),
            days: (m.end - m.start) as u8,
        })
        .collect();

    Ok(ChineseCalendar {
        equation_of_time: sky.equation_of_time(jd)?,
        solar_terms,
        lunar_months,
    })
}

/// A lunar year of the calendar: its New Year and its months in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LunarYear {
    /// The Gregorian year its New Year falls in.
    pub year: i32,
    /// Julian day number of the New Year.
    pub start: i64,
    /// Lengths of its months in order (29 or 30), the leap month in its place.
    pub days: Vec<u8>,
    /// Number of the month the leap month repeats, 0 without one.
    pub leap: u8,
}

/// The lunar years `first..=last`, from the ephemeris (for the embedded table and its test).
#[cfg(test)]
pub(crate) fn lunar_years(
    kernels: &KernelSet,
    first: i32,
    last: i32,
) -> Result<Vec<LunarYear>, EngineError> {
    let sky = Sky { kernels };
    let december = |year: i32| UtcInstant::from_civil(year, 12, 10, 0, 0, 0, 0).julian_day();
    let mut solstice = sky.sun_at(270.0, december(first - 1))?;
    let mut months = Vec::new();
    loop {
        let (mut sui, _, next) = sky.sui(solstice)?;
        let done = sui.last().is_some_and(|m| m.year > last);
        months.append(&mut sui);
        if done {
            break;
        }
        solstice = next;
    }
    Ok((first..=last)
        .map(|year| {
            let of_year: Vec<&Month> = months.iter().filter(|m| m.year == year).collect();
            LunarYear {
                year,
                start: of_year[0].start,
                days: of_year.iter().map(|m| (m.end - m.start) as u8).collect(),
                leap: of_year.iter().find(|m| m.leap).map_or(0, |m| m.number),
            }
        })
        .collect())
}

/// The moments (UT Julian dates) of the 12 jie of a Gregorian year, from 小寒 (285°) to
/// 大雪 (255°), from the ephemeris (for the embedded table and its test).
#[cfg(test)]
pub(crate) fn jie_of_year(kernels: &KernelSet, year: i32) -> Result<[f64; 12], EngineError> {
    let sky = Sky { kernels };
    let january = UtcInstant::from_civil(year, 1, 6, 0, 0, 0, 0).julian_day();
    let mut moments = [0.0; 12];
    for (i, moment) in moments.iter_mut().enumerate() {
        let longitude = f64::from((285 + 30 * i as u32) % 360);
        *moment = sky.sun_at(longitude, january + 30.44 * i as f64)?;
    }
    Ok(moments)
}
