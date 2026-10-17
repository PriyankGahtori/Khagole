//! The search over a span of time: positions sampled on an hourly grid and interpolated
//! between, solar days found once, windows of consecutive good moments.

use super::*;

/// A search's source of positions: exact on an hourly grid, linearly interpolated in
/// between. Each planets computation gives the positions at its instant and an hour later
/// (from which the speeds come), so one computation serves two hours of moments. Between
/// grid points the Moon's longitude is off by well under an arc second.
pub(super) struct Sampler<'a> {
    pub(super) kernels: &'a KernelSet,
    pub(super) req: &'a ChartRequest<'a>,
    pub(super) system: HouseSystem,
    pub(super) sidereal: bool,
    pub(super) ayanamsa: &'static str,
    /// Anchors are every two hours from here.
    pub(super) origin: UtcInstant,
    pub(super) anchors: Vec<(i64, Anchor)>,
}

/// Exact positions (and the Greenwich sidereal time) at a grid instant.
pub(super) struct Anchor {
    pub(super) bodies: Vec<BodyPosition>,
    pub(super) gast_hours: f64,
}

/// Bodies whose apparent motion can turn retrograde, as the planets computation flags them.
pub(super) fn can_retrograde(name: &str) -> bool {
    !matches!(name, "Sun" | "Moon" | "Lilith")
}

impl<'a> Sampler<'a> {
    pub(super) fn new(kernels: &'a KernelSet, req: &'a ChartRequest<'a>) -> Self {
        Sampler {
            kernels,
            req,
            system: HouseSystem::from_code(req.house_system),
            sidereal: req.zodiac_type.trim().eq_ignore_ascii_case("sidereal"),
            ayanamsa: ayanamsa(req.ayanamsa).code,
            origin: req.instant,
            anchors: Vec::new(),
        }
    }

    /// The anchor `index` × 2 hours after the origin (the last three are kept).
    pub(super) fn anchor(&mut self, index: i64) -> Result<&Anchor, EngineError> {
        if let Some(pos) = self.anchors.iter().position(|(i, _)| *i == index) {
            return Ok(&self.anchors[pos].1);
        }
        let jd = self
            .origin
            .add_micros(index * 2 * 3_600_000_000)
            .julian_day();
        let shift = if self.sidereal {
            ayanamsa_info(jd, self.ayanamsa).value
        } else {
            0.0
        };
        let (planets, gast_hours) = planets_and_sidereal_time(self.kernels, jd, shift)?;
        if self.anchors.len() == 3 {
            self.anchors.remove(0);
        }
        self.anchors.push((
            index,
            Anchor {
                bodies: planets.bodies,
                gast_hours,
            },
        ));
        Ok(&self.anchors.last().unwrap().1)
    }

    /// Placements and house cusps at `instant` (not before the origin).
    pub(super) fn at(
        &mut self,
        instant: UtcInstant,
    ) -> Result<(Vec<Placement>, Vec<f64>), EngineError> {
        let hours = instant.seconds_since(&self.origin) / 3600.0;
        let index = (hours / 2.0).floor() as i64;
        let into = hours - index as f64 * 2.0; // 0 ≤ into < 2
        let (first, gast0) = {
            let anchor = self.anchor(index)?;
            let bodies: Vec<(&'static str, f64, f64, bool)> = anchor
                .bodies
                .iter()
                .map(|b| (b.name, b.longitude, b.speed, b.is_retrograde))
                .collect();
            (bodies, anchor.gast_hours)
        };
        let (next_bodies, gast1) = {
            let next = self.anchor(index + 1)?;
            (
                next.bodies
                    .iter()
                    .map(|b| (b.name, b.longitude, b.speed))
                    .collect::<Vec<_>>(),
                next.gast_hours,
            )
        };

        // Each body's track over the three hours from the anchor: exact every hour (an
        // anchor's position an hour later comes from its speed, degrees per day over that
        // hour), linear in between.
        let hour_later = |lon: f64, speed: f64| pyfloat::rem(lon + speed / 24.0, 360.0);
        let mut raw = Vec::with_capacity(first.len());
        for &(name, lon, speed, is_retrograde) in &first {
            if into == 0.0 {
                raw.push((name, lon, speed, is_retrograde));
                continue;
            }
            let Some(&(_, next, next_speed)) = next_bodies.iter().find(|(n, ..)| *n == name) else {
                continue;
            };
            let points = [
                lon,
                hour_later(lon, speed),
                next,
                hour_later(next, next_speed),
            ];
            let track = |hours: f64| {
                let i = (hours.floor() as usize).min(2);
                let (a, b) = (points[i], points[i + 1]);
                pyfloat::rem(a + pyfloat::wrap180(b - a) * (hours - i as f64), 360.0)
            };
            // The speed as the planets computation measures it: over the next hour.
            let (longitude, later) = (track(into), track(into + 1.0));
            let speed = pyfloat::wrap180(later - longitude) * 24.0;
            raw.push((name, longitude, speed, can_retrograde(name) && speed < 0.0));
        }

        let gast1 = if gast1 < gast0 { gast1 + 24.0 } else { gast1 };
        let gast = if into == 0.0 {
            gast0
        } else {
            pyfloat::rem(gast0 + (gast1 - gast0) * into / 2.0, 24.0)
        };
        let jd = instant.julian_day();
        let shift = if self.sidereal {
            ayanamsa_info(jd, self.ayanamsa).value
        } else {
            0.0
        };
        let houses = houses_at_sidereal_time(
            jd,
            gast,
            self.req.latitude,
            self.req.longitude,
            self.system,
            shift,
        );
        let cusps = houses.cusps.to_vec();
        Ok((placements(raw, &houses), cusps))
    }
}

/// The solar days of a search's moments: from the sunrises and sunsets of the whole span
/// when a single kernel covers it, otherwise found day by day.
pub(super) struct SolarDays<'a> {
    pub(super) kernels: &'a KernelSet,
    pub(super) latitude: f64,
    pub(super) longitude: f64,
    pub(super) events: Option<SunEvents>,
    pub(super) last: Option<SolarDay>,
}

impl<'a> SolarDays<'a> {
    pub(super) fn new(kernels: &'a KernelSet, req: &ChartRequest, end: UtcInstant) -> Self {
        let events = SunEvents::find(
            kernels,
            jd_utc(req.instant) - 1.6,
            jd_utc(end) + 1.6,
            req.latitude,
            req.longitude,
        )
        .ok()
        .flatten();
        SolarDays {
            kernels,
            latitude: req.latitude,
            longitude: req.longitude,
            events,
            last: None,
        }
    }

    pub(super) fn hours(&mut self, instant: UtcInstant) -> PlanetaryHours {
        if let Some(times) = self
            .events
            .as_ref()
            .and_then(|e| e.rise_set(jd_utc(instant)))
        {
            return hours_in(&SolarDay::from_julian_days(times), instant);
        }
        let reuse = self.latitude.abs() < SOLAR_DAY_CACHE_MAX_LAT;
        let day = match self.last {
            Some(d) if reuse && d.covers(instant) => d,
            _ => solar_day(self.kernels, instant, self.latitude, self.longitude),
        };
        self.last = Some(day);
        hours_in(&day, instant)
    }
}

/// The best windows from `req.instant` to `end` (at most [`MAX_SEARCH_DAYS`]) at the
/// request's place, with its house system and zodiac.
///
/// Moments are assessed every `step_minutes`; those failing a criteria filter are left
/// out, and runs of consecutive moments scoring at least `min_score` form the windows,
/// ranked by their best score. Positions between hourly grid points are interpolated
/// for speed, then each window's best moment is assessed again on exact positions, so
/// its score and factors are what [`calculate_election_chart`] gives for it. A window
/// whose best moment turns out excluded or below `min_score` there is left out.
pub fn search_elections(
    kernels: &KernelSet,
    req: &ChartRequest,
    end: UtcInstant,
    criteria: &ElectionCriteria,
) -> Result<ElectionSearch, EngineError> {
    req.check_place()?;
    let resolved = criteria.resolve()?;
    let start = req.instant;
    let span_days = end.seconds_since(&start) / 86_400.0;
    if span_days <= 0.0 {
        return Err(EngineError::InvalidInput(
            "the search must end after it starts".into(),
        ));
    }
    if span_days > MAX_SEARCH_DAYS {
        return Err(EngineError::InvalidInput(format!(
            "a search spans at most {MAX_SEARCH_DAYS} days"
        )));
    }
    // Fail fast rather than after assessing most of the span. The grid runs up to two
    // hours past the end, and planets are computed an hour after each grid point.
    require_kernel(kernels, start.julian_day())?;
    require_kernel(kernels, end.julian_day() + 3.0 / 24.0)?;

    let step = i64::from(resolved.summary.step_minutes) * 60_000_000;
    let mut sampler = Sampler::new(kernels, req);
    let mut days = SolarDays::new(kernels, req, end);
    let mut evaluated = 0;
    let mut excluded = Exclusions::default();
    let mut windows: Vec<(UtcInstant, ElectionWindow)> = Vec::new();
    let mut open: Option<(UtcInstant, ElectionWindow)> = None;

    let mut instant = start;
    while instant <= end {
        let kept = 'moment: {
            // Clock hours first: they need no ephemeris.
            if let Some(range) = resolved.summary.local_hours {
                if !range.contains(resolved.local_minute_of_day(instant)) {
                    excluded.count("outside_hours");
                    break 'moment None;
                }
            }
            let hours = days.hours(instant);
            if resolved.summary.daytime_only && !hours.is_day {
                excluded.count("night");
                break 'moment None;
            }
            let (planets, cusps) = sampler.at(instant)?;
            let data = assess(
                &Moment {
                    instant,
                    planets: &planets,
                    cusps: &cusps,
                    hours: &hours,
                },
                &resolved,
            )?;
            evaluated += 1;
            if let Some(reason) = data.excluded_by.first() {
                excluded.count(reason);
                break 'moment None;
            }
            (data.score >= resolved.summary.min_score).then_some(data)
        };

        match kept {
            Some(data) => {
                let at = instant.isoformat();
                match open.as_mut() {
                    Some((best, w)) => {
                        w.end = at.clone();
                        if data.score > w.score {
                            *best = instant;
                            w.best = at;
                            w.score = data.score;
                        }
                    }
                    None => {
                        open = Some((
                            instant,
                            ElectionWindow {
                                start: at.clone(),
                                end: at.clone(),
                                best: at,
                                score: data.score,
                                verdict: data.verdict,
                                factors: Vec::new(),
                            },
                        ))
                    }
                }
            }
            None => windows.extend(open.take()),
        }
        instant = instant.add_micros(step);
    }
    windows.extend(open.take());

    // Best first; earlier first on a tie (windows are already in time order).
    windows.sort_by(|a, b| b.1.score.total_cmp(&a.1.score));

    // The best moments again, on exact positions. A window whose best moment does not
    // hold up (interpolation put it just inside an exclusion or the threshold) is left
    // out, and the next one takes its place.
    let max_results = resolved.summary.max_results;
    let mut confirmed = Vec::with_capacity(max_results.min(windows.len()));
    for (best, mut window) in windows {
        if confirmed.len() == max_results {
            break;
        }
        let exact = sky(
            kernels,
            &ChartRequest {
                instant: best,
                ..req.clone()
            },
        )?;
        let data = assess(
            &Moment {
                instant: best,
                planets: &exact.planets,
                cusps: &exact.cusps,
                hours: &days.hours(best),
            },
            &resolved,
        )?;
        if !data.excluded_by.is_empty() || data.score < resolved.summary.min_score {
            continue;
        }
        window.score = data.score;
        window.verdict = data.verdict;
        window.factors = data.factors;
        confirmed.push(window);
    }
    confirmed.sort_by(|a, b| b.score.total_cmp(&a.score));

    Ok(ElectionSearch {
        start: start.isoformat(),
        end: end.isoformat(),
        windows: confirmed,
        evaluated,
        excluded,
        criteria: resolved.summary,
    })
}
