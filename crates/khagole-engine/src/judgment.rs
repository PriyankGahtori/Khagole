//! Horary judgment after Lilly: the significators of the querent and of the quesited, and
//! whether and how the matter perfects between them (by aspect, translation of light or
//! collection), or is prevented (prohibition, refranation, a significator leaving its sign).
//!
//! Not part of the reference implementation. Perfection is projected from the planets'
//! present speeds, as the Moon's applying aspects are; refranation checks the ephemeris for
//! a station before the projected perfection.

use serde::Serialize;

use crate::chart::Placement;
use crate::dignities::{traditional_ruler, Reception, SEVEN, SIGNS};
use crate::ephemeris::KernelSet;
use crate::horary::PTOLEMAIC;
use crate::planets::planet_speed;
use crate::pyfloat;

/// The longest span a station is searched for, in days.
const MAX_REFRANATION_DAYS: f64 = 400.0;

/// An aspect that becomes exact between two planets, both still in their present signs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Perfection {
    /// The planet that applies (the faster one).
    pub applying: &'static str,
    /// The planet applied to.
    pub to: &'static str,
    /// Ptolemaic aspect, e.g. "Trine".
    pub aspect: &'static str,
    /// Degrees still to cover before the aspect is exact (rounded to 0.01).
    pub degrees: f64,
    /// Days until it is exact at the present speeds (rounded to 0.01).
    pub days: f64,
}

/// A third planet that perfects an aspect with one of the significators first.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Prohibition {
    /// The planet that comes between.
    pub by: &'static str,
    /// The significator it reaches first.
    pub significator: &'static str,
    /// Ptolemaic aspect.
    pub aspect: &'static str,
    /// Days until it is exact (rounded to 0.01).
    pub days: f64,
}

/// A lighter planet carrying the light of one significator to the other.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Translation {
    /// The planet that translates.
    pub by: &'static str,
    /// The significator it has separated from.
    pub from: &'static str,
    /// The aspect it separated from.
    pub from_aspect: &'static str,
    /// The significator it applies to.
    pub to: &'static str,
    /// The aspect it applies by.
    pub to_aspect: &'static str,
}

/// A heavier planet that both significators apply to.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Collection {
    /// The planet that collects the light.
    pub by: &'static str,
    /// The querent's significator's aspect to it.
    pub querent_aspect: &'static str,
    /// The quesited's significator's aspect to it.
    pub quesited_aspect: &'static str,
}

/// A significator that turns retrograde (or direct) before the aspect perfects.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Refranation {
    /// The planet that stations.
    pub planet: &'static str,
    /// The aspect that would have perfected.
    pub aspect: &'static str,
    /// Days until the station (rounded to 0.1).
    pub days: f64,
}

/// The judgment of a horary question.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Judgment {
    /// The house of the matter asked about (1-12), if given.
    pub quesited_house: Option<u8>,
    /// The querent's significator: the lord of the Ascendant.
    pub querent: &'static str,
    /// The querent's co-significator: the Moon.
    pub co_significator: &'static str,
    /// The quesited's significator: the lord of its house cusp.
    pub quesited: Option<&'static str>,
    /// Whether one planet signifies both (judged then by the Moon).
    pub same_significator: bool,
    /// The aspect that perfects between the two significators, if any.
    pub perfection: Option<Perfection>,
    /// The aspect the Moon perfects with the quesited's significator, if any.
    pub moon_perfection: Option<Perfection>,
    /// Planets that perfect with a significator before the significators perfect.
    pub prohibition: Option<Prohibition>,
    /// A station of a significator before their perfection.
    pub refranation: Option<Refranation>,
    /// Translations of light between the significators.
    pub translation: Vec<Translation>,
    /// Collections of light by a heavier planet.
    pub collection: Vec<Collection>,
    /// Receptions between the two significators.
    pub receptions: Vec<Reception>,
}

/// A planet's place and speed now.
#[derive(Debug, Clone, Copy)]
struct Body {
    name: &'static str,
    lon: f64,
    speed: f64,
}

impl Body {
    /// Days until it leaves its sign (forward or, retrograde, backward); infinite if still.
    fn days_in_sign(self) -> f64 {
        let within = pyfloat::rem(self.lon, 30.0);
        if self.speed > 0.0 {
            (30.0 - within) / self.speed
        } else if self.speed < 0.0 {
            within / -self.speed
        } else {
            f64::INFINITY
        }
    }
}

/// The next time (days, > 0) an aspect between `a` and `b` is exact at present speeds,
/// with both still in their signs: (aspect, days, degrees).
fn next_aspect(a: Body, b: Body) -> Option<(&'static str, f64, f64)> {
    let v = a.speed - b.speed;
    if v.abs() < 1e-9 {
        return None;
    }
    let d0 = a.lon - b.lon;
    let horizon = a.days_in_sign().min(b.days_in_sign());
    let mut best: Option<(&'static str, f64, f64)> = None;
    for (angle, aspect) in PTOLEMAIC {
        for target in [angle, -angle] {
            // d0 + v t ≡ target (mod 360): the smallest t > 0
            let t = pyfloat::rem((target - d0) * v.signum(), 360.0) / v.abs();
            if t > 1e-9 && t < horizon && best.is_none_or(|(_, bt, _)| t < bt) {
                best = Some((aspect, t, (v * t).abs()));
            }
        }
    }
    best
}

/// The last time (days ago, > 0) an aspect between `a` and `b` was exact, with both in their
/// present signs since: (aspect, days ago).
fn last_aspect(a: Body, b: Body) -> Option<(&'static str, f64)> {
    let back = |x: Body| Body {
        speed: -x.speed,
        ..x
    };
    next_aspect(back(a), back(b)).map(|(aspect, t, _)| (aspect, t))
}

fn round2(x: f64) -> f64 {
    pyfloat::round(x, 2)
}

/// The faster of two bodies applies to the slower one.
fn perfection(a: Body, b: Body) -> Option<Perfection> {
    next_aspect(a, b).map(|(aspect, days, degrees)| {
        let (applying, to) = if a.speed.abs() >= b.speed.abs() {
            (a.name, b.name)
        } else {
            (b.name, a.name)
        };
        Perfection {
            applying,
            to,
            aspect,
            degrees: round2(degrees),
            days: round2(days),
        }
    })
}

/// The first station of `planet` within `days`, from the ephemeris (days from now).
fn station_within(kernels: &KernelSet, jd: f64, planet: Body, days: f64) -> Option<f64> {
    if matches!(planet.name, "Sun" | "Moon") || days <= 0.0 {
        return None;
    }
    let span = days.min(MAX_REFRANATION_DAYS);
    let step = (span / 60.0).clamp(0.25, 5.0);
    let speed_at = |t: f64| planet_speed(kernels, jd + t, planet.name).ok();
    let mut t = 0.0;
    let mut previous = planet.speed;
    while t < span {
        t = (t + step).min(span);
        let speed = speed_at(t)?;
        if speed.signum() != previous.signum() {
            return Some(t);
        }
        previous = speed;
    }
    None
}

/// Judge the question: significators, perfection and what helps or hinders it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn judge(
    kernels: &KernelSet,
    jd: f64,
    planets: &[Placement],
    cusp_signs: &[&'static str],
    asc_sign: &str,
    quesited_house: Option<u8>,
    receptions: &[Reception],
) -> Judgment {
    let body = |name: &str| {
        planets.iter().find(|p| p.name == name).map(|p| Body {
            name: p.name,
            lon: p.ecliptic_longitude,
            speed: p.speed,
        })
    };
    let querent = traditional_ruler(asc_sign).unwrap_or("Mars");
    let quesited = quesited_house
        .filter(|h| (1..=12).contains(h))
        .and_then(|h| cusp_signs.get(h as usize - 1))
        .and_then(|sign| traditional_ruler(sign));
    let same_significator = quesited == Some(querent);

    let mut judgment = Judgment {
        quesited_house,
        querent,
        co_significator: "Moon",
        quesited,
        same_significator,
        perfection: None,
        moon_perfection: None,
        prohibition: None,
        refranation: None,
        translation: Vec::new(),
        collection: Vec::new(),
        receptions: Vec::new(),
    };
    let (Some(q), Some(a)) = (quesited.and_then(body), body(querent)) else {
        return judgment;
    };
    if let Some(moon) = body("Moon").filter(|_| q.name != "Moon") {
        judgment.moon_perfection = perfection(moon, q);
    }
    if same_significator {
        return judgment;
    }

    judgment.receptions = receptions
        .iter()
        .filter(|r| {
            (r.planet == a.name && r.receiver == q.name)
                || (r.planet == q.name && r.receiver == a.name)
        })
        .cloned()
        .collect();

    let direct = perfection(a, q);
    let others: Vec<Body> = SEVEN
        .iter()
        .filter(|n| **n != a.name && **n != q.name)
        .filter_map(|n| body(n))
        .collect();

    if let Some(p) = &direct {
        // Prohibition: another planet reaches either significator first.
        judgment.prohibition = others
            .iter()
            .flat_map(|x| {
                [a, q].into_iter().filter_map(move |sig| {
                    next_aspect(*x, sig).map(|(aspect, days, _)| (x.name, sig.name, aspect, days))
                })
            })
            .filter(|(_, _, _, days)| *days < p.days)
            .min_by(|l, r| l.3.total_cmp(&r.3))
            .map(|(by, significator, aspect, days)| Prohibition {
                by,
                significator,
                aspect,
                days: round2(days),
            });
        // Refranation: a significator stations before the aspect is exact.
        judgment.refranation = [a, q]
            .into_iter()
            .filter_map(|sig| station_within(kernels, jd, sig, p.days).map(|days| (sig.name, days)))
            .min_by(|l, r| l.1.total_cmp(&r.1))
            .map(|(planet, days)| Refranation {
                planet,
                aspect: p.aspect,
                days: pyfloat::round(days, 1),
            });
    } else {
        // Translation: a lighter planet separates from one significator and applies to the
        // other; collection: both significators apply to a heavier one.
        for x in &others {
            let lighter = x.speed.abs() > a.speed.abs().max(q.speed.abs());
            let heavier = x.speed.abs() < a.speed.abs().min(q.speed.abs());
            if lighter {
                for (from, to) in [(a, q), (q, a)] {
                    if let (Some((from_aspect, _)), Some((to_aspect, _, _))) =
                        (last_aspect(*x, from), next_aspect(*x, to))
                    {
                        judgment.translation.push(Translation {
                            by: x.name,
                            from: from.name,
                            from_aspect,
                            to: to.name,
                            to_aspect,
                        });
                    }
                }
            }
            if heavier {
                if let (Some((qa, _, _)), Some((qq, _, _))) =
                    (next_aspect(a, *x), next_aspect(q, *x))
                {
                    judgment.collection.push(Collection {
                        by: x.name,
                        querent_aspect: qa,
                        quesited_aspect: qq,
                    });
                }
            }
        }
    }
    judgment.perfection = direct;
    judgment
}

/// The signs on the twelve cusps.
pub(crate) fn cusp_signs(cusps: &[f64]) -> Vec<&'static str> {
    cusps
        .iter()
        .map(|lon| SIGNS[(pyfloat::floordiv(pyfloat::rem(*lon, 360.0), 30.0) as usize) % 12])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(name: &'static str, lon: f64, speed: f64) -> Body {
        Body { name, lon, speed }
    }

    #[test]
    fn applying_trine_within_signs() {
        // Moon 10° Aries (13°/day) to Jupiter 20° Leo (0.1°/day): trine in about 0.77 days.
        let (aspect, days, degrees) =
            next_aspect(b("Moon", 10.0, 13.0), b("Jupiter", 140.0, 0.1)).unwrap();
        assert_eq!(aspect, "Trine");
        assert!((days - 10.0 / 12.9).abs() < 1e-9);
        assert!((degrees - 10.0).abs() < 1e-9);
    }

    #[test]
    fn no_perfection_once_a_planet_leaves_its_sign() {
        // Moon 28° Aries must cover 12° to trine Jupiter at 10° Leo: it leaves Aries first.
        assert!(next_aspect(b("Moon", 28.0, 13.0), b("Jupiter", 130.0, 0.0)).is_none());
    }

    #[test]
    fn separating_aspects_are_in_the_past() {
        // Mercury 15° Gemini (1.5°/day) past a sextile to Mars 10° Aries (0.5°/day).
        let (aspect, ago) = last_aspect(b("Mercury", 75.0, 1.5), b("Mars", 10.0, 0.5)).unwrap();
        assert_eq!(aspect, "Sextile");
        assert!((ago - 5.0).abs() < 1e-9);
    }

    #[test]
    fn retrograde_planets_apply_backwards() {
        // Venus 12° Taurus retrograde (−0.5°/day) to a square with Saturn 10° Leo (0.03°/day).
        let (aspect, days, _) =
            next_aspect(b("Venus", 42.0, -0.5), b("Saturn", 130.0, 0.03)).unwrap();
        assert_eq!(aspect, "Square");
        assert!((days - 2.0 / 0.53).abs() < 1e-9);
    }

    #[test]
    fn mercury_stations_retrograde_on_1_april_2024() {
        let Some(kernels) = crate::test_kernel::full_kernel() else {
            return;
        };
        let jd = crate::UtcInstant::parse("2024-03-25T00:00:00Z")
            .unwrap()
            .julian_day();
        let mercury = crate::planets::planets_and_sidereal_time(&kernels, jd, 0.0)
            .unwrap()
            .0
            .bodies
            .into_iter()
            .find(|b| b.name == "Mercury")
            .unwrap();
        let body = b("Mercury", mercury.longitude, mercury.speed);
        let days = station_within(&kernels, jd, body, 30.0).unwrap();
        // The station is at 2024-04-01 ~22h UTC, 7.9 days on; samples are half a day apart.
        assert!((days - 7.9).abs() <= 0.6, "{days}");
        assert!(station_within(&kernels, jd, body, 5.0).is_none());
    }
}
