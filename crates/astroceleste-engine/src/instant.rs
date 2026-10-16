//! UTC instants with microsecond resolution, mirroring the Python `datetime` behaviour
//! the reference implementation relies on (ISO formatting, float-day arithmetic).

use std::fmt;

use crate::time::julian_day;

const US_PER_SECOND: i64 = 1_000_000;
const US_PER_DAY: i64 = 86_400 * US_PER_SECOND;
/// Latest year [`UtcInstant::parse`] accepts: past every JPL kernel (DE441 ends in 17191)
/// and far from overflowing the microsecond count.
const MAX_YEAR: i32 = 200_000;

/// A UTC instant, as microseconds since 1970-01-01T00:00:00Z.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcInstant {
    micros: i64,
}

/// Days since 1970-01-01 of a proleptic-Gregorian date (H. Hinnant, `days_from_civil`).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// The text given to [`UtcInstant::parse`] is not a supported date-time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid UTC date-time: {}", self.0)
    }
}

impl std::error::Error for ParseError {}

impl UtcInstant {
    /// 2000-01-01T12:00:00Z.
    pub const J2000: UtcInstant = UtcInstant {
        micros: 946_728_000 * US_PER_SECOND,
    };

    /// From microseconds since the Unix epoch.
    pub fn from_micros(micros: i64) -> Self {
        UtcInstant { micros }
    }

    /// From UT Julian day number.
    pub fn from_julian_day(jd: f64) -> Self {
        let days = jd - 2_440_587.5;
        let micros = (days * 86_400.0 * 1_000_000.0).round() as i64;
        UtcInstant { micros }
    }

    /// Microseconds since the Unix epoch.
    pub fn micros(&self) -> i64 {
        self.micros
    }

    /// From proleptic-Gregorian calendar fields (UTC). Out-of-range fields roll over
    /// (day 32 is the next month's first); the result saturates instead of overflowing.
    pub fn from_civil(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        microsecond: u32,
    ) -> Self {
        let days = days_from_civil(year as i64, month as i64, day as i64);
        let secs = hour as i64 * 3600 + minute as i64 * 60 + second as i64;
        UtcInstant {
            micros: days
                .saturating_mul(US_PER_DAY)
                .saturating_add(secs * US_PER_SECOND + microsecond as i64),
        }
    }

    /// (year, month, day, hour, minute, second, microsecond)
    pub fn civil(&self) -> (i32, u32, u32, u32, u32, u32, u32) {
        let days = self.micros.div_euclid(US_PER_DAY);
        let rem = self.micros.rem_euclid(US_PER_DAY);
        let (y, m, d) = civil_from_days(days);
        let secs = rem / US_PER_SECOND;
        (
            y as i32,
            m as u32,
            d as u32,
            (secs / 3600) as u32,
            (secs % 3600 / 60) as u32,
            (secs % 60) as u32,
            (rem % US_PER_SECOND) as u32,
        )
    }

    /// Python `datetime.weekday()`: Monday is 0.
    pub fn weekday(&self) -> u32 {
        // 1970-01-01 was a Thursday (3).
        (self.micros.div_euclid(US_PER_DAY) + 3).rem_euclid(7) as u32
    }

    /// Julian day as the reference computes it from calendar fields.
    pub fn julian_day(&self) -> f64 {
        let (y, m, d, hh, mm, ss, us) = self.civil();
        let hour =
            hh as f64 + (mm as f64 / 60.0) + (ss as f64 / 3600.0) + (us as f64 / 3_600_000_000.0);
        julian_day(y, m, d, hour)
    }

    /// Seconds from `other` to `self`, like Python `(self - other).total_seconds()`.
    pub fn seconds_since(&self, other: &UtcInstant) -> f64 {
        (self.micros - other.micros) as f64 / US_PER_SECOND as f64
    }

    /// `micros` microseconds later (earlier if negative), saturating at the ends of the
    /// representable range.
    pub fn add_micros(&self, micros: i64) -> Self {
        UtcInstant {
            micros: self.micros.saturating_add(micros),
        }
    }

    /// `self + timedelta(days=days)`, with CPython's rounding of float days to
    /// microseconds (whole days exact, fraction rounded half to even).
    pub fn plus_days(&self, days: f64) -> Self {
        self.add_micros(timedelta_days_to_micros(days))
    }

    /// Same day at another time of day (Python `dt.replace(hour=…, minute=…)`).
    pub fn with_time(&self, hour: u32, minute: u32, second: u32, microsecond: u32) -> Self {
        let (y, m, d, ..) = self.civil();
        UtcInstant::from_civil(y, m, d, hour, minute, second, microsecond)
    }

    /// Python `datetime.isoformat()` of an aware UTC datetime.
    pub fn isoformat(&self) -> String {
        let (y, m, d, hh, mm, ss, us) = self.civil();
        let fraction = if us == 0 {
            String::new()
        } else {
            format!(".{us:06}")
        };
        format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}{fraction}+00:00")
    }

    /// Parse `YYYY-MM-DDTHH:MM[:SS[.ffffff]]` with an optional `Z` or `+00:00` suffix.
    /// The date must exist (no 31 February) and the year be at most 200 000.
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let err = || ParseError(text.to_string());
        let body = text
            .strip_suffix('Z')
            .or_else(|| text.strip_suffix("+00:00"))
            .unwrap_or(text);
        let (date, time) = body.split_once(['T', ' ']).ok_or_else(err)?;
        let mut d = date.split('-');
        let year: i32 = d.next().and_then(|v| v.parse().ok()).ok_or_else(err)?;
        let month: u32 = d.next().and_then(|v| v.parse().ok()).ok_or_else(err)?;
        let day: u32 = d.next().and_then(|v| v.parse().ok()).ok_or_else(err)?;
        let mut t = time.split(':');
        let hour: u32 = t.next().and_then(|v| v.parse().ok()).ok_or_else(err)?;
        let minute: u32 = t.next().and_then(|v| v.parse().ok()).ok_or_else(err)?;
        let (second, micro) = match t.next() {
            None => (0, 0),
            Some(s) => {
                let (whole, frac) = s.split_once('.').unwrap_or((s, ""));
                let second: u32 = whole.parse().map_err(|_| err())?;
                let micro = if frac.is_empty() {
                    0
                } else {
                    let digits: String = frac.chars().chain("000000".chars()).take(6).collect();
                    digits.parse().map_err(|_| err())?
                };
                (second, micro)
            }
        };
        if !(0..=MAX_YEAR).contains(&year)
            || !(1..=12).contains(&month)
            || day == 0
            || day > days_in_month(year, month)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return Err(err());
        }
        Ok(UtcInstant::from_civil(
            year, month, day, hour, minute, second, micro,
        ))
    }
}

/// CPython `timedelta(days=x)` for a float `x`, in microseconds.
fn timedelta_days_to_micros(days: f64) -> i64 {
    let int_part = days.trunc();
    let frac_part = days - int_part;
    let mut total = int_part as i64 * US_PER_DAY;
    if frac_part == 0.0 {
        return total;
    }
    let scaled = US_PER_DAY as f64 * frac_part;
    let scaled_int = scaled.trunc();
    let leftover = scaled - scaled_int;
    total += scaled_int as i64;
    if leftover != 0.0 {
        let mut whole = leftover.round();
        if (whole - leftover).abs() == 0.5 {
            // Half-way: round so that the total is even.
            let odd = f64::from(total.rem_euclid(2) == 1);
            whole = 2.0 * ((leftover + odd) * 0.5).round() - odd;
        }
        total += whole as i64;
    }
    total
}

impl fmt::Display for UtcInstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.isoformat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_round_trip_and_weekday() {
        let t = UtcInstant::from_civil(2024, 2, 29, 23, 59, 59, 123_456);
        assert_eq!(t.civil(), (2024, 2, 29, 23, 59, 59, 123_456));
        assert_eq!(t.isoformat(), "2024-02-29T23:59:59.123456+00:00");
        assert_eq!(t.weekday(), 3); // Thursday
        assert_eq!(UtcInstant::J2000.isoformat(), "2000-01-01T12:00:00+00:00");
        assert_eq!(UtcInstant::J2000.weekday(), 5); // Saturday
        let old = UtcInstant::from_civil(1850, 1, 2, 0, 0, 0, 0);
        assert_eq!(old.civil(), (1850, 1, 2, 0, 0, 0, 0));
    }

    #[test]
    fn parses_iso() {
        let t = UtcInstant::parse("1987-03-20T21:52:00").unwrap();
        assert_eq!(t, UtcInstant::from_civil(1987, 3, 20, 21, 52, 0, 0));
        let t = UtcInstant::parse("2000-01-01T05:59:33.272404+00:00").unwrap();
        assert_eq!(t.civil().6, 272_404);
        assert!(UtcInstant::parse("2000-13-01T00:00").is_err());
        assert!(UtcInstant::parse("2001-02-29T00:00").is_err());
        assert!(UtcInstant::parse("2023-04-31T00:00").is_err());
        assert!(UtcInstant::parse("2000-02-29T00:00").is_ok());
        assert!(UtcInstant::parse("1900-02-29T00:00").is_err());
        assert!(UtcInstant::parse("2147483647-01-01T00:00").is_err());
        assert!(UtcInstant::parse("17191-01-01T00:00").is_ok());
    }

    #[test]
    fn julian_day_matches_formula() {
        assert_eq!(UtcInstant::J2000.julian_day(), 2_451_545.0);
    }

    #[test]
    fn float_days_round_like_python() {
        // Python: timedelta(days=0.5) == 12 h; timedelta(days=1/3) == 8:00:00
        assert_eq!(timedelta_days_to_micros(0.5), US_PER_DAY / 2);
        assert_eq!(timedelta_days_to_micros(1.0 / 3.0), 28_800_000_000);
        assert_eq!(timedelta_days_to_micros(-0.25), -US_PER_DAY / 4);
    }
}
