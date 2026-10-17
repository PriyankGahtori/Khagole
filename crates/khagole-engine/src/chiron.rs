//! Chiron from a precomputed geocentric longitude table (4-day step, 1849-2151), with
//! cubic Lagrange interpolation. Outside the table Chiron is reported unavailable rather
//! than extrapolated: its chaotic, Saturn-perturbed orbit has no honest closed form.

use std::sync::OnceLock;

use crate::pyfloat;

struct Table {
    jd_start: f64,
    step: f64,
    lons_unwrapped: Vec<f64>,
}

static TABLE: OnceLock<Table> = OnceLock::new();

fn table() -> &'static Table {
    TABLE.get_or_init(|| {
        let bytes: &[u8] = include_bytes!("data/chiron.bin");
        let f = |i: usize| f64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        let n = u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as usize;
        Table {
            jd_start: f(0),
            step: f(8),
            lons_unwrapped: (0..n).map(|i| f(20 + 8 * i)).collect(),
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chiron {
    pub longitude: f64,
    /// Degrees per day, rounded to 4 decimals.
    pub speed: f64,
    pub is_retrograde: bool,
}

/// Chiron's longitude (shifted by `shift` degrees for sidereal charts) at a Julian date.
pub fn chiron(jd: f64, shift: f64) -> Option<Chiron> {
    let tab = table();
    let y = &tab.lons_unwrapped;
    let step = tab.step;
    let idx = ((jd - tab.jd_start) / step) as i64;
    if !(1 <= idx && (idx as usize) < y.len() - 2) {
        return None;
    }
    let i = idx as usize;
    let t = (jd - (tab.jd_start + idx as f64 * step)) / step;
    let (t2, t3) = (t.powf(2.0), t.powf(3.0));
    let c0 = -0.5 * t + t2 - 0.5 * t3;
    let c1 = 1.0 - 2.5 * t2 + 1.5 * t3;
    let c2 = 0.5 * t + 2.0 * t2 - 1.5 * t3;
    let c3 = -0.5 * t2 + 0.5 * t3;
    let dc0 = (-0.5 + 2.0 * t - 1.5 * t2) / step;
    let dc1 = (-5.0 * t + 4.5 * t2) / step;
    let dc2 = (0.5 + 4.0 * t - 4.5 * t2) / step;
    let dc3 = (-t + 1.5 * t2) / step;
    let (y0, y1, y2, y3) = (y[i - 1], y[i], y[i + 1], y[i + 2]);
    let lon_rad = c0 * y0 + c1 * y1 + c2 * y2 + c3 * y3;
    let speed_rad_day = dc0 * y0 + dc1 * y1 + dc2 * y2 + dc3 * y3;
    let speed = speed_rad_day.to_degrees();
    Some(Chiron {
        longitude: pyfloat::rem(lon_rad.to_degrees() - shift, 360.0),
        speed: pyfloat::round(speed, 4),
        is_retrograde: speed < 0.0,
    })
}
