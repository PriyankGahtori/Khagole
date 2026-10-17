//! Time scales: UT1, TT and TDB, and ΔT = TT − UT1.
//!
//! Chart instants are UT1 Julian dates. ΔT comes from the daily IERS table bundled with
//! Skyfield 1.55 (1973 onwards, with about a year of predictions) and, outside it, from
//! the Morrison–Stephenson–Hohenkerk splines joined to the long-term parabola, evaluated
//! exactly as Skyfield does (`src/data/deltat.bin`, see `scripts/gen_tables.py`).

use std::sync::OnceLock;

use crate::constants::{DAY_S, T0};

/// Julian day of a proleptic-Gregorian calendar date, `hour` being fractional hours.
/// Same arithmetic as the reference implementation (Meeus, chapter 7).
pub fn julian_day(year: i32, month: u32, day: u32, hour: f64) -> f64 {
    let (mut y, mut m) = (year as f64, month as f64);
    if month <= 2 {
        y -= 1.0;
        m += 12.0;
    }
    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day as f64 + hour / 24.0 + b
        - 1524.5
}

/// TDB − TT in seconds (USNO Circular 179, eq. 2.6). `jd` may be TDB or TT.
pub fn tdb_minus_tt(jd_whole: f64, fraction: f64) -> f64 {
    let t = (jd_whole - T0 + fraction) / 36525.0;
    0.001657 * (628.3076 * t + 6.2401).sin()
        + 0.000022 * (575.3385 * t + 4.2970).sin()
        + 0.000014 * (1256.6152 * t + 6.1969).sin()
        + 0.000005 * (606.9777 * t + 4.0212).sin()
        + 0.000005 * (52.9691 * t + 0.4444).sin()
        + 0.000002 * (21.3299 * t + 5.5431).sin()
        + 0.000010 * t * (628.3076 * t + 4.2490).sin()
}

/// An instant, stored like Skyfield stores it: a `whole` Julian date plus per-scale
/// fractions, so TT and TDB keep their precision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Time {
    pub whole: f64,
    pub tt_fraction: f64,
    pub tdb_fraction: f64,
    pub ut1_fraction: f64,
}

impl Time {
    /// From a UT1 Julian date (Skyfield `Timescale.ut1_jd`).
    pub fn from_ut1(jd_ut1: f64) -> Self {
        let delta_t = delta_t();
        let mut dt = delta_t.at(jd_ut1);
        dt = delta_t.at(jd_ut1 + dt / DAY_S);
        let tt_fraction = dt / DAY_S;
        Time {
            whole: jd_ut1,
            tt_fraction,
            tdb_fraction: tt_fraction + tdb_minus_tt(jd_ut1, tt_fraction) / DAY_S,
            ut1_fraction: 0.0,
        }
    }

    /// From a TT date split in two (Skyfield `Timescale.tt_jd(whole, fraction)`).
    pub fn from_tt(whole: f64, tt_fraction: f64) -> Self {
        Time {
            whole,
            tt_fraction,
            tdb_fraction: tt_fraction + tdb_minus_tt(whole, tt_fraction) / DAY_S,
            ut1_fraction: tt_fraction - delta_t().at(whole + tt_fraction) / DAY_S,
        }
    }

    /// From a single TT Julian date (Skyfield `Timescale.tt_jd(jd)`, which splits it
    /// into whole days and a fraction).
    pub fn from_tt_jd(jd_tt: f64) -> Self {
        let (whole, fraction) = crate::pyfloat::divmod(jd_tt, 1.0);
        Self::from_tt(whole, fraction)
    }

    /// From a TDB date split in two (Skyfield `Timescale.tdb_jd(whole, fraction)`).
    pub fn from_tdb(whole: f64, tdb_fraction: f64) -> Self {
        Time {
            whole,
            tt_fraction: tdb_fraction - tdb_minus_tt(whole, tdb_fraction) / DAY_S,
            tdb_fraction,
            ut1_fraction: f64::NAN,
        }
    }

    pub fn tt(&self) -> f64 {
        self.whole + self.tt_fraction
    }

    pub fn tdb(&self) -> f64 {
        self.whole + self.tdb_fraction
    }

    pub fn ut1(&self) -> f64 {
        self.whole + self.ut1_fraction
    }

    /// This instant minus `days` of TT (Skyfield `Time.__sub__(float)`).
    pub fn minus_days(&self, days: f64) -> Self {
        let w = days.div_euclid(1.0);
        let f = days.rem_euclid(1.0);
        Time::from_tt(self.whole - w, self.tt_fraction - f)
    }
}

/// ΔT model: daily table, then splines.
pub struct DeltaT {
    table_tt: Vec<f64>,
    table_delta_t: Vec<f64>,
    /// Spline rows: [lower, upper, a3, a2, a1, a0], x in Julian years.
    splines: Vec<[f64; 6]>,
}

static DELTA_T: OnceLock<DeltaT> = OnceLock::new();

/// The built-in ΔT model.
pub fn delta_t() -> &'static DeltaT {
    DELTA_T.get_or_init(|| DeltaT::parse(include_bytes!("data/deltat.bin")))
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let out = self.bytes[self.pos..self.pos + N].try_into().unwrap();
        self.pos += N;
        out
    }
    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take())
    }
    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take())
    }
    fn f64(&mut self) -> f64 {
        f64::from_le_bytes(self.take())
    }
}

impl DeltaT {
    fn parse(bytes: &[u8]) -> Self {
        let mut r = Reader { bytes, pos: 0 };
        let days = r.u32() as usize;
        let run_count = r.u32() as usize;
        let runs: Vec<(usize, f64)> = (0..run_count)
            .map(|_| (r.u32() as usize, r.f64()))
            .collect();
        let mut table_tt = Vec::with_capacity(days);
        let mut run = 0;
        for i in 0..days {
            while run + 1 < runs.len() && runs[run + 1].0 <= i {
                run += 1;
            }
            table_tt.push(runs[run].1 + i as f64);
        }
        // Skyfield: (delta_t_1e7 / 1e7).round(7), and NumPy rounds as rint(x * 1e7) / 1e7.
        let table_delta_t = (0..days)
            .map(|_| ((r.i32() as f64 / 1e7) * 1e7).round_ties_even() / 1e7)
            .collect();
        let spline_count = r.u32() as usize;
        let splines = (0..spline_count)
            .map(|_| [r.f64(), r.f64(), r.f64(), r.f64(), r.f64(), r.f64()])
            .collect();
        DeltaT {
            table_tt,
            table_delta_t,
            splines,
        }
    }

    /// ΔT in seconds at a TT Julian date.
    pub fn at(&self, tt: f64) -> f64 {
        match interp(tt, &self.table_tt, &self.table_delta_t) {
            Some(value) => value,
            None => self.long_term((tt - 1_721_045.0) / 365.25),
        }
    }

    /// Skyfield `Splines.__call__` at Julian year `x`.
    fn long_term(&self, x: f64) -> f64 {
        let n = self.splines.len();
        let lower = |i: usize| self.splines[i][0];
        // NumPy interp(x, lower, arange(n)) clamps, then the index is truncated.
        let i = if x <= lower(0) {
            0
        } else if x >= lower(n - 1) {
            n - 1
        } else {
            let j = self.splines.partition_point(|s| s[0] <= x) - 1;
            let frac = (x - lower(j)) / (lower(j + 1) - lower(j));
            (j as f64 + frac) as usize
        };
        let [lo, hi, a3, a2, a1, a0] = self.splines[i];
        let t = (x - lo) / (hi - lo);
        ((a3 * t + a2) * t + a1) * t + a0
    }
}

/// NumPy `interp` with NaN outside the table (returned as `None`).
fn interp(x: f64, xp: &[f64], fp: &[f64]) -> Option<f64> {
    let last = xp.len() - 1;
    if !(x >= xp[0] && x <= xp[last]) {
        return None;
    }
    if x == xp[last] {
        return Some(fp[last]);
    }
    let j = xp.partition_point(|v| *v <= x) - 1;
    let slope = (fp[j + 1] - fp[j]) / (xp[j + 1] - xp[j]);
    Some(slope * (x - xp[j]) + fp[j])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn julian_day_of_j2000() {
        assert_eq!(julian_day(2000, 1, 1, 12.0), T0);
        assert_eq!(julian_day(1858, 11, 17, 0.0), 2_400_000.5);
    }

    #[test]
    fn delta_t_table_loads() {
        let dt = delta_t();
        assert_eq!(dt.table_tt.len(), dt.table_delta_t.len());
        // ΔT at J2000 was about 63.8 s.
        assert!((dt.at(T0) - 63.83).abs() < 0.05, "{}", dt.at(T0));
    }
}
