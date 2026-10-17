//! Planetary positions (`SkyfieldEngine.calculate_planets`): apparent geocentric
//! longitudes of date from the JPL kernels, the mean lunar nodes, Chiron and Black Moon
//! Lilith.

use crate::chiron::chiron;
use crate::ephemeris::observe::{apparent, ecliptic_latlon, observe};
use crate::ephemeris::{Kernel, KernelSet};
use crate::error::EngineError;
use crate::frames::Orientation;
use crate::pyfloat;
use crate::time::Time;

/// Position and kinematic state of a celestial body or astrological point.
#[derive(Debug, Clone, PartialEq)]
pub struct BodyPosition {
    /// Common name of the body (e.g. "Sun", "Moon", "Mars").
    pub name: &'static str,
    /// Astronomical or astrological glyph symbol (e.g. "☉", "☽").
    pub symbol: &'static str,
    /// Degrees, tropical or sidereal as requested.
    pub longitude: f64,
    /// Ecliptic latitude in degrees.
    pub latitude: f64,
    /// Distance from Earth center in astronomical units (au).
    pub distance: f64,
    /// Daily apparent motion speed in degrees per day.
    pub speed: f64,
    /// Whether the body is currently in retrograde motion.
    pub is_retrograde: bool,
}

/// (name, NAIF code, symbol, can be retrograde)
const TARGETS: [(&str, i32, &str, bool); 10] = [
    ("Sun", 10, "☉", false),
    ("Moon", 301, "☽", false),
    ("Mercury", 199, "☿", true),
    ("Venus", 299, "♀", true),
    ("Mars", 4, "♂", true),
    ("Jupiter", 5, "♃", true),
    ("Saturn", 6, "♄", true),
    ("Uranus", 7, "♅", true),
    ("Neptune", 8, "♆", true),
    ("Pluto", 9, "♇", true),
];

/// The kernel covering `jd`, raising when none does (`SkyfieldEngine._require_eph`).
pub fn require_kernel(kernels: &KernelSet, jd: f64) -> Result<&Kernel, EngineError> {
    kernels.for_jd(jd).ok_or(EngineError::OutOfRange {
        jd,
        coverage: kernels.coverage(),
    })
}

/// A collection of computed celestial body positions.
pub struct Planets {
    /// Bodies that could not be computed (e.g. Chiron outside its table) are left out.
    pub bodies: Vec<BodyPosition>,
}

/// All chart bodies at UT1 Julian date `jd`, shifted by `shift` degrees (the ayanamsa for
/// sidereal charts, 0 for tropical), with the Greenwich apparent sidereal time (hours)
/// at `jd`.
pub fn planets_and_sidereal_time(
    kernels: &KernelSet,
    jd: f64,
    shift: f64,
) -> Result<(Planets, f64), EngineError> {
    planets_and_sidereal_time_with_mode(kernels, jd, shift, false)
}

/// All chart bodies at UT1 Julian date `jd`, shifted by `shift` degrees (the ayanamsa for
/// sidereal charts, 0 for tropical), with the Greenwich apparent sidereal time (hours)
/// at `jd`. If `geometric` is true, positions are true geometric coordinates without light-time
/// or aberration.
pub fn planets_and_sidereal_time_with_mode(
    kernels: &KernelSet,
    jd: f64,
    shift: f64,
    geometric: bool,
) -> Result<(Planets, f64), EngineError> {
    planets_and_sidereal_time_with_options(kernels, jd, shift, geometric, false)
}

/// All chart bodies at UT1 Julian date `jd`, shifted by `shift` degrees (the ayanamsa for
/// sidereal charts, 0 for tropical), with the Greenwich apparent sidereal time (hours)
/// at `jd`.
/// - `geometric`: true geometric coordinates without light-time or aberration.
/// - `true_node`: true osculating lunar node (from instantaneous DE440 state vectors) instead of mean node.
pub fn planets_and_sidereal_time_with_options(
    kernels: &KernelSet,
    jd: f64,
    shift: f64,
    geometric: bool,
    true_node: bool,
) -> Result<(Planets, f64), EngineError> {
    let sample = SpeedSample::at(kernels, jd, geometric)?;
    let mut bodies = Vec::with_capacity(14);

    for (name, code, symbol, can_retrograde) in TARGETS {
        let Ok((lat, lon, dist, speed)) = sample.observe(code) else {
            continue;
        };
        bodies.push(BodyPosition {
            name,
            symbol,
            longitude: pyfloat::rem(lon - shift, 360.0),
            latitude: lat,
            distance: dist,
            speed,
            is_retrograde: can_retrograde && speed < 0.0,
        });
    }

    // Mean lunar ascending node (IAU polynomial).
    let t_cent = (jd - 2_451_545.0) / 36525.0;
    let omega = pyfloat::rem(
        125.0445479 - 1934.1362891 * t_cent
            + 0.0020754 * t_cent.powf(2.0)
            + t_cent.powf(3.0) / 467440.0
            - t_cent.powf(4.0) / 60616000.0,
        360.0,
    );

    if true_node {
        if let Ok((lon, speed, is_retrograde)) = sample.true_lunar_node() {
            let node_lon = pyfloat::rem(lon - shift, 360.0);
            for (name, symbol, longitude) in [
                ("North Node", "☊", node_lon),
                ("South Node", "☋", pyfloat::rem(node_lon + 180.0, 360.0)),
            ] {
                bodies.push(BodyPosition {
                    name,
                    symbol,
                    longitude,
                    latitude: 0.0,
                    distance: 0.00257,
                    speed,
                    is_retrograde,
                });
            }
        }
    } else {
        let mean_node = pyfloat::rem(omega - shift, 360.0);
        for (name, symbol, longitude) in [
            ("North Node", "☊", mean_node),
            ("South Node", "☋", pyfloat::rem(mean_node + 180.0, 360.0)),
        ] {
            bodies.push(BodyPosition {
                name,
                symbol,
                longitude,
                latitude: 0.0,
                distance: 0.00257,
                speed: -0.05295,
                is_retrograde: true,
            });
        }
    }

    if let Some(c) = chiron(jd, shift) {
        bodies.push(BodyPosition {
            name: "Chiron",
            symbol: "⚷",
            longitude: c.longitude,
            latitude: 0.0,
            distance: 0.0,
            speed: c.speed,
            is_retrograde: c.is_retrograde,
        });
    }

    // Black Moon Lilith: mean lunar apogee projected from the lunar orbit onto the ecliptic.
    let perigee = pyfloat::rem(
        83.3532465 + 4069.0137287 * t_cent
            - 0.01032 * t_cent.powf(2.0)
            - t_cent.powf(3.0) / 80053.0,
        360.0,
    );
    let apogee = pyfloat::rem(perigee + 180.0, 360.0);
    let u = pyfloat::rem(apogee - omega, 360.0).to_radians();
    let inc = 5.1453964_f64.to_radians();
    let proj_u = (u.sin() * inc.cos()).atan2(u.cos());
    bodies.push(BodyPosition {
        name: "Lilith",
        symbol: "⚸",
        longitude: pyfloat::rem(proj_u.to_degrees() + omega - shift, 360.0),
        latitude: 0.0,
        distance: 0.0027,
        speed: 0.1114,
        is_retrograde: false,
    });

    Ok((Planets { bodies }, sample.orientation.gast_hours))
}

/// The apparent speed (degrees per day) of the planet `name` at UT1 Julian date `jd`,
/// exactly as [`planets_and_sidereal_time`] gives it, without computing the other bodies.
pub fn planet_speed(kernels: &KernelSet, jd: f64, name: &str) -> Result<f64, EngineError> {
    let (.., code, _, _) = TARGETS
        .iter()
        .find(|(n, ..)| *n == name)
        .ok_or_else(|| EngineError::InvalidInput(format!("not a planet: {name}")))?;
    Ok(SpeedSample::at(kernels, jd, false)?.observe(*code)?.3)
}

/// An instant and the instant an hour later (for the speed and the retrograde
/// flag), with the kernel and the orientations of both.
struct SpeedSample<'k> {
    kernel: &'k Kernel,
    t: Time,
    t_plus: Time,
    orientation: Orientation,
    orientation_plus: Orientation,
    is_geometric: bool,
}

impl<'k> SpeedSample<'k> {
    fn at(kernels: &'k KernelSet, jd: f64, is_geometric: bool) -> Result<Self, EngineError> {
        let t = Time::from_ut1(jd);
        let t_plus = Time::from_ut1(t.ut1() + 1.0 / 24.0);

        // Both ends of the speed sample must be covered by the same kernel.
        let mut kernel = require_kernel(kernels, jd)?;
        let plus_kernel = kernels.for_jd(t_plus.ut1());
        if !plus_kernel.is_some_and(|k| std::ptr::eq(k, kernel)) {
            kernel = require_kernel(kernels, t_plus.ut1())?;
        }
        Ok(SpeedSample {
            kernel,
            orientation: Orientation::at(&t),
            orientation_plus: Orientation::at(&t_plus),
            t,
            t_plus,
            is_geometric,
        })
    }

    /// Ecliptic latitude, longitude (degrees, tropical), distance (au) and speed
    /// in longitude (degrees per day) of the body with NAIF code `code`.
    fn observe(&self, code: i32) -> Result<(f64, f64, f64, f64), EngineError> {
        let (kernel, t, t_plus) = (self.kernel, &self.t, &self.t_plus);
        if self.is_geometric {
            let (pos, _) = kernel.barycentric(code, t)?;
            let (earth_pos, _) = kernel.barycentric(crate::ephemeris::observe::EARTH, t)?;
            let geom = [pos[0] - earth_pos[0], pos[1] - earth_pos[1], pos[2] - earth_pos[2]];
            let (lat, lon, dist) = ecliptic_latlon(&self.orientation, &geom);

            let (pos_plus, _) = kernel.barycentric(code, t_plus)?;
            let (earth_pos_plus, _) = kernel.barycentric(crate::ephemeris::observe::EARTH, t_plus)?;
            let geom_plus = [
                pos_plus[0] - earth_pos_plus[0],
                pos_plus[1] - earth_pos_plus[1],
                pos_plus[2] - earth_pos_plus[2],
            ];
            let lon_plus = ecliptic_latlon(&self.orientation_plus, &geom_plus).1;
            let d_lon = pyfloat::wrap180(lon_plus - lon);
            Ok((lat, lon, dist, d_lon * 24.0))
        } else {
            let a = observe(kernel, code, t)?;
            let app = apparent(kernel, &a, t)?;
            let a_plus = observe(kernel, code, t_plus)?;
            let app_plus = apparent(kernel, &a_plus, t_plus)?;
            let (lat, lon, dist) = ecliptic_latlon(&self.orientation, &app);
            let lon_plus = ecliptic_latlon(&self.orientation_plus, &app_plus).1;
            let d_lon = pyfloat::wrap180(lon_plus - lon);
            Ok((lat, lon, dist, d_lon * 24.0))
        }
    }

    /// Osculating (true) lunar node: intersection of the instantaneous osculating lunar
    /// orbit plane with the true ecliptic of date. Returns (longitude_deg_tropical, speed_deg_day, is_retrograde).
    fn true_lunar_node(&self) -> Result<(f64, f64, bool), EngineError> {
        let compute = |t: &Time, ori: &Orientation| -> Result<f64, EngineError> {
            let (pos_moon, vel_moon) = self.kernel.barycentric(301, t)?;
            let (pos_earth, vel_earth) = self.kernel.barycentric(crate::ephemeris::observe::EARTH, t)?;
            let r = [pos_moon[0] - pos_earth[0], pos_moon[1] - pos_earth[1], pos_moon[2] - pos_earth[2]];
            let v = [vel_moon[0] - vel_earth[0], vel_moon[1] - vel_earth[1], vel_moon[2] - vel_earth[2]];
            let r_ecl = crate::frames::mxv(&ori.ecliptic, &r);
            let v_ecl = crate::frames::mxv(&ori.ecliptic, &v);
            let h_x = r_ecl[1] * v_ecl[2] - r_ecl[2] * v_ecl[1];
            let h_y = r_ecl[2] * v_ecl[0] - r_ecl[0] * v_ecl[2];
            Ok(h_x.atan2(-h_y).to_degrees().rem_euclid(360.0))
        };
        let lon = compute(&self.t, &self.orientation)?;
        let lon_plus = compute(&self.t_plus, &self.orientation_plus)?;
        let d_lon = pyfloat::wrap180(lon_plus - lon);
        let speed = d_lon * 24.0;
        let is_retrograde = speed < 0.0;
        Ok((lon, speed, is_retrograde))
    }
}

