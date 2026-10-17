//! Ba Zi (八字), the Four Pillars of Destiny of a birth, cast from its
//! [`ChineseCalendar`]. No kernel is needed.
//!
//! - The year pillar changes at 立春 and the month pillar at each jie term; both compare
//!   instants, so they do not depend on the clock the birth was recorded in.
//! - The day and hour pillars are cast on true solar time at the birthplace (the default)
//!   or on civil time. The Zi hour (23:00-01:00) belongs to the next day by default; with
//!   `zi_hour = "split"` the day changes at midnight and the late Zi hour keeps its day,
//!   its stem still taken from the next day's.
//! - The luck pillars (大運) run forward from the month pillar for a yang year and a man
//!   or a yin year and a woman, backward otherwise. They start after the days from the
//!   birth to the next (forward) or previous (backward) jie term, three days to a year.
//! - Given a date, the year, month and day pillars of that date (流年, 流月, 流日), read at
//!   noon civil time with the birth's UTC offset, from an embedded table of the jie terms of
//!   1900-2100 (no kernel).

use serde::Serialize;

use super::calendar::{day_number, ChineseCalendar, SolarTerm, LICHUN};
use super::jie_table::jie_before;
use super::{
    branch_element, cycle_index, polarity, stem_element, year_cycle_index, ANIMALS, BRANCHES,
    ELEMENTS, STEMS,
};
use crate::error::EngineError;
use crate::instant::UtcInstant;

const TROPICAL_YEAR: f64 = 365.242_19;
const MICROS_PER_MINUTE: f64 = 60_000_000.0;

/// The stems hidden in each branch (藏干): main qi first, then middle and residual qi.
const HIDDEN_STEMS: [&[usize]; 12] = [
    &[9],       // 子 癸
    &[5, 9, 7], // 丑 己癸辛
    &[0, 2, 4], // 寅 甲丙戊
    &[1],       // 卯 乙
    &[4, 1, 9], // 辰 戊乙癸
    &[2, 6, 4], // 巳 丙庚戊
    &[3, 5],    // 午 丁己
    &[5, 3, 1], // 未 己丁乙
    &[6, 8, 4], // 申 庚壬戊
    &[7],       // 酉 辛
    &[4, 7, 3], // 戌 戊辛丁
    &[8, 0],    // 亥 壬甲
];

/// Share of its branch each hidden stem carries in the element balance, by how many the
/// branch holds.
fn hidden_weights(count: usize) -> &'static [f64] {
    match count {
        1 => &[1.0],
        2 => &[0.7, 0.3],
        _ => &[0.6, 0.3, 0.1],
    }
}

/// NaYin (納音) of each pair of the sexagenary cycle, 甲子乙丑 first.
const NAYIN: [&str; 30] = [
    "sea_metal",
    "furnace_fire",
    "forest_wood",
    "roadside_earth",
    "sword_metal",
    "mountain_fire",
    "stream_water",
    "rampart_earth",
    "wax_metal",
    "willow_wood",
    "spring_water",
    "roof_earth",
    "thunder_fire",
    "pine_wood",
    "river_water",
    "sand_metal",
    "foothill_fire",
    "plain_wood",
    "wall_earth",
    "gold_leaf_metal",
    "lamp_fire",
    "heavenly_river_water",
    "highway_earth",
    "hairpin_metal",
    "mulberry_wood",
    "creek_water",
    "sand_earth",
    "sky_fire",
    "pomegranate_wood",
    "ocean_water",
];

/// The NaYin code of a pair of the sexagenary cycle (0-29).
pub(crate) fn nayin(pair: usize) -> &'static str {
    NAYIN[pair]
}

/// The twelve stages of life (十二長生), from Birth.
pub(crate) const LIFE_STAGES: [&str; 12] = [
    "birth",
    "bath",
    "cap_and_belt",
    "coming_of_age",
    "prosperity",
    "decline",
    "sickness",
    "death",
    "tomb",
    "extinction",
    "conception",
    "nurture",
];

/// The branch where each stem is born (長生); yang stems go forward through the
/// branches, yin stems backward.
const BIRTH_BRANCH: [usize; 10] = [11, 6, 2, 9, 2, 9, 5, 0, 8, 3];

/// The Ten Gods (十神) by the element relation to the Day Master (same, produced by it,
/// controlled by it, controlling it, producing it) and same or opposite polarity.
const TEN_GODS: [[&str; 2]; 5] = [
    ["friend", "rob_wealth"],
    ["eating_god", "hurting_officer"],
    ["indirect_wealth", "direct_wealth"],
    ["seven_killings", "direct_officer"],
    ["indirect_resource", "direct_resource"],
];

/// When the day changes around the Zi hour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZiHour {
    /// 23:00: the whole Zi hour belongs to the next day.
    NextDay,
    /// Midnight: the late Zi hour (夜子時) keeps its day.
    Split,
}

/// How to cast the pillars.
#[derive(Debug, Clone)]
pub struct BaziOptions<'a> {
    /// Cast the day and hour on true solar time at the birthplace (`true`, the default)
    /// or on civil time.
    pub solar_time: bool,
    /// "next_day" (default): the day changes at 23:00; "split": at midnight.
    pub zi_hour: &'a str,
    /// "male" or "female": sets the direction of the luck pillars, which are left out
    /// without it.
    pub sex: Option<&'a str>,
    /// How many ten-year luck pillars to list (default 10).
    pub luck_pillars: u8,
    /// A year whose pillar (流年, changing at 立春) to give, seen from the Day Master.
    pub year: Option<i32>,
    /// A date, `YYYY-MM-DD`, whose year, month and day pillars (流年, 流月, 流日) to give,
    /// read at noon civil time with the birth's UTC offset. Dates from 1900-01-06 to 2100.
    pub date: Option<&'a str>,
}

impl Default for BaziOptions<'_> {
    fn default() -> Self {
        BaziOptions {
            solar_time: true,
            zi_hour: "next_day",
            sex: None,
            luck_pillars: 10,
            year: None,
            date: None,
        }
    }
}

/// A stem hidden in a branch.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HiddenStem {
    /// Stem code, e.g. "gui".
    pub stem: &'static str,
    /// Its element.
    pub element: &'static str,
    /// Its relation to the Day Master.
    pub ten_god: &'static str,
    /// Its share of the branch in the element balance.
    pub weight: f64,
}

/// A stem and a branch: one of the four pillars, or a luck pillar.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Pillar {
    /// Heavenly Stem code, e.g. "jia".
    pub stem: &'static str,
    /// Earthly Branch code, e.g. "zi".
    pub branch: &'static str,
    /// Position in the sexagenary cycle, 0 (甲子) to 59 (癸亥).
    pub cycle: u8,
    /// Element of the stem.
    pub element: &'static str,
    /// "yang" or "yin", of the stem and the branch alike.
    pub polarity: &'static str,
    /// Element of the branch.
    pub branch_element: &'static str,
    /// Animal of the branch.
    pub animal: &'static str,
    /// The stem's relation to the Day Master; `None` for the Day Master itself.
    pub ten_god: Option<&'static str>,
    /// The stems hidden in the branch, main qi first.
    pub hidden_stems: Vec<HiddenStem>,
    /// NaYin element of the pair, e.g. "sea_metal".
    pub nayin: &'static str,
    /// The Day Master's stage of life in this branch.
    pub life_stage: &'static str,
}

/// Weight of each element among the eight characters (stems count 1, each branch 1
/// shared among its hidden stems).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ElementBalance {
    /// 木
    pub wood: f64,
    /// 火
    pub fire: f64,
    /// 土
    pub earth: f64,
    /// 金
    pub metal: f64,
    /// 水
    pub water: f64,
}

/// A solar term the pillars refer to.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TermMoment {
    /// Pinyin code, e.g. "jingzhe".
    pub name: String,
    /// ISO 8601 UTC.
    pub utc: String,
}

/// A date of the lunisolar calendar.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LunarDate {
    /// The lunar year: the Gregorian year its New Year falls in.
    pub year: i32,
    /// Month number, 1-12.
    pub month: u8,
    /// Whether the month is intercalary.
    pub leap: bool,
    /// Day of the month, 1-30.
    pub day: u8,
    /// Stem of the lunar year.
    pub year_stem: &'static str,
    /// Branch of the lunar year.
    pub year_branch: &'static str,
    /// Animal of the lunar year: the popular "Chinese zodiac sign", which changes at the
    /// New Year rather than at 立春.
    pub animal: &'static str,
}

/// A ten-year luck pillar.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LuckPillar {
    /// Age at which it starts, years.
    pub start_age: f64,
    /// Start, ISO 8601 UTC.
    pub start: String,
    /// End (the next one's start), ISO 8601 UTC.
    pub end: String,
    /// Its stem and branch.
    pub pillar: Pillar,
}

/// The luck pillars (大運).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Luck {
    /// "forward" or "backward" through the sexagenary cycle from the month pillar.
    pub direction: &'static str,
    /// Age at which the first one starts, years.
    pub start_age: f64,
    /// The pillars in order.
    pub pillars: Vec<LuckPillar>,
}

/// The pillars of a date read against a chart.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BaziFlow {
    /// The date, `YYYY-MM-DD`.
    pub date: String,
    /// The year's pillar (流年), changing at 立春.
    pub year: Pillar,
    /// The month's pillar (流月), changing at each jie term.
    pub month: Pillar,
    /// The day's pillar (流日).
    pub day: Pillar,
    /// The jie term that opened the month.
    pub month_term: TermMoment,
}

/// The Four Pillars of a birth.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Bazi {
    /// "solar" or "civil": the clock the day and hour pillars were cast on.
    pub time_basis: &'static str,
    /// "next_day" or "split".
    pub zi_hour: &'static str,
    /// Local civil time of birth, `YYYY-MM-DDTHH:MM:SS`.
    pub civil_time: String,
    /// True solar time of birth at the birthplace, `YYYY-MM-DDTHH:MM:SS`.
    pub solar_time: String,
    /// Year pillar (changes at 立春).
    pub year: Pillar,
    /// Month pillar (changes at each jie term).
    pub month: Pillar,
    /// Day pillar; its stem is the Day Master.
    pub day: Pillar,
    /// Hour pillar.
    pub hour: Pillar,
    /// Stem of the day pillar, the self.
    pub day_master: &'static str,
    /// Element of the Day Master.
    pub day_master_element: &'static str,
    /// The weight of each element among the eight characters.
    pub elements: ElementBalance,
    /// The jie term that opened the birth month.
    pub month_term: TermMoment,
    /// The next jie term after the birth.
    pub next_term: TermMoment,
    /// The birth date in the lunisolar calendar; `None` when the calendar lacks it.
    pub lunar_date: Option<LunarDate>,
    /// The luck pillars; `None` without the native's sex.
    pub luck: Option<Luck>,
    /// The pillar of the requested year (流年), seen from the Day Master.
    pub annual: Option<Pillar>,
    /// The pillars of the requested date, seen from the Day Master.
    pub flow: Option<BaziFlow>,
}

fn invalid(message: &str) -> EngineError {
    EngineError::InvalidInput(message.to_string())
}

/// `YYYY-MM-DDTHH:MM:SS` of a wall-clock instant.
fn wall_clock(instant: UtcInstant) -> String {
    let (y, m, d, hh, mm, ss, _) = instant.civil();
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}")
}

/// The relation of `stem` to the Day Master `master`.
fn ten_god(master: usize, stem: usize) -> &'static str {
    let relation = (stem_element(stem) + 5 - stem_element(master)) % 5;
    TEN_GODS[relation][usize::from(master % 2 != stem % 2)]
}

/// The stage of life of `stem` in `branch`.
fn life_stage(stem: usize, branch: usize) -> &'static str {
    let birth = BIRTH_BRANCH[stem] as i64;
    let steps = if stem % 2 == 0 {
        branch as i64 - birth
    } else {
        birth - branch as i64
    };
    LIFE_STAGES[steps.rem_euclid(12) as usize]
}

/// The pillar at `cycle` (0-59), seen from the Day Master `master`.
fn pillar(cycle: usize, master: usize, is_master: bool) -> Pillar {
    let (stem, branch) = (cycle % 10, cycle % 12);
    let hidden = HIDDEN_STEMS[branch];
    Pillar {
        stem: STEMS[stem],
        branch: BRANCHES[branch],
        cycle: cycle as u8,
        element: ELEMENTS[stem_element(stem)],
        polarity: polarity(stem),
        branch_element: ELEMENTS[branch_element(branch)],
        animal: ANIMALS[branch],
        ten_god: (!is_master).then(|| ten_god(master, stem)),
        hidden_stems: hidden
            .iter()
            .zip(hidden_weights(hidden.len()))
            .map(|(&h, &weight)| HiddenStem {
                stem: STEMS[h],
                element: ELEMENTS[stem_element(h)],
                ten_god: ten_god(master, h),
                weight,
            })
            .collect(),
        nayin: NAYIN[cycle / 2],
        life_stage: life_stage(master, branch),
    }
}

fn balance(cycles: [usize; 4]) -> ElementBalance {
    let mut weights = [0.0f64; 5];
    for cycle in cycles {
        let (stem, branch) = (cycle % 10, cycle % 12);
        weights[stem_element(stem)] += 1.0;
        let hidden = HIDDEN_STEMS[branch];
        for (&h, &w) in hidden.iter().zip(hidden_weights(hidden.len())) {
            weights[stem_element(h)] += w;
        }
    }
    let round = |x: f64| (x * 100.0).round() / 100.0;
    ElementBalance {
        wood: round(weights[0]),
        fire: round(weights[1]),
        earth: round(weights[2]),
        metal: round(weights[3]),
        water: round(weights[4]),
    }
}

/// A solar term with its parsed moment.
type Dated<'a> = (UtcInstant, &'a SolarTerm);

/// The last term at or before `birth` matching `pick`, and the first after it.
fn around<'a>(
    terms: &'a [Dated<'a>],
    birth: UtcInstant,
    pick: impl Fn(&SolarTerm) -> bool,
) -> (Option<&'a Dated<'a>>, Option<&'a Dated<'a>>) {
    let before = terms.iter().rev().find(|(t, s)| *t <= birth && pick(s));
    let after = terms.iter().find(|(t, s)| *t > birth && pick(s));
    (before, after)
}

fn term_moment((instant, term): &Dated) -> TermMoment {
    TermMoment {
        name: term.name.clone(),
        utc: instant.isoformat(),
    }
}

/// The jie terms in table order, from 小寒.
const JIE_NAMES: [&str; 12] = [
    "xiaohan",
    "lichun",
    "jingzhe",
    "qingming",
    "lixia",
    "mangzhong",
    "xiaoshu",
    "liqiu",
    "bailu",
    "hanlu",
    "lidong",
    "daxue",
];

/// The year, month and day pillars of `date` read at noon with `utc_offset_minutes`.
fn flow_of(date: &str, utc_offset_minutes: f64, master: usize) -> Result<BaziFlow, EngineError> {
    // Noon on the date, as a wall clock and as the moment it is at the birth's UTC offset.
    let local_noon = UtcInstant::parse(&format!("{date}T12:00:00Z"))
        .map_err(|_| invalid("date must be YYYY-MM-DD"))?;
    let noon = local_noon.add_micros(-(utc_offset_minutes * MICROS_PER_MINUTE).round() as i64);
    let (index, year, at) =
        jie_before(noon).ok_or_else(|| invalid("date is outside the years 1900-2100"))?;
    // 小寒 (index 0) still belongs to the year before 立春.
    let year_cycle = year_cycle_index(if index == 0 { year - 1 } else { year });
    let month_offset = (index + 11) % 12;
    let month_stem = ((year_cycle % 10) % 5 * 2 + 2 + month_offset) % 10;
    let day_cycle = (day_number(local_noon) - 11).rem_euclid(60) as usize;
    Ok(BaziFlow {
        date: date.to_string(),
        year: pillar(year_cycle, master, false),
        month: pillar(
            cycle_index(month_stem, (2 + month_offset) % 12),
            master,
            false,
        ),
        day: pillar(day_cycle, master, false),
        month_term: TermMoment {
            name: JIE_NAMES[index].to_string(),
            utc: at.isoformat(),
        },
    })
}

fn lunar_date(calendar: &ChineseCalendar, day: i64) -> Result<Option<LunarDate>, EngineError> {
    for month in &calendar.lunar_months {
        let start = month.start_day()?;
        if start <= day && day < start + i64::from(month.days) {
            let cycle = year_cycle_index(month.year);
            return Ok(Some(LunarDate {
                year: month.year,
                month: month.month,
                leap: month.leap,
                day: (day - start + 1) as u8,
                year_stem: STEMS[cycle % 10],
                year_branch: BRANCHES[cycle % 12],
                animal: ANIMALS[cycle % 12],
            }));
        }
    }
    Ok(None)
}

/// Cast the Four Pillars of a birth at `birth` (UTC), at `longitude` (degrees east) where
/// civil time was `utc_offset_minutes` ahead of UTC, from its `calendar`
/// ([`chinese_calendar`](super::calendar::chinese_calendar) of the same moment).
pub fn bazi(
    calendar: &ChineseCalendar,
    birth: UtcInstant,
    longitude: f64,
    utc_offset_minutes: f64,
    options: &BaziOptions,
) -> Result<Bazi, EngineError> {
    let zi_hour = match options.zi_hour {
        "" | "next_day" => ZiHour::NextDay,
        "split" => ZiHour::Split,
        _ => return Err(invalid("zi_hour must be \"next_day\" or \"split\"")),
    };
    let male = match options.sex {
        None | Some("") => None,
        Some("male") => Some(true),
        Some("female") => Some(false),
        Some(_) => return Err(invalid("sex must be \"male\" or \"female\"")),
    };
    if !(utc_offset_minutes.is_finite() && utc_offset_minutes.abs() <= 1440.0) {
        return Err(invalid("utc_offset_minutes must be within ±1440"));
    }
    if !(longitude.is_finite() && longitude.abs() <= 360.0) {
        return Err(invalid("longitude must be within ±360°"));
    }
    if !(calendar.equation_of_time.is_finite() && calendar.equation_of_time.abs() <= 60.0) {
        return Err(invalid(
            "the calendar's equation_of_time must be within ±60 minutes",
        ));
    }

    let terms = calendar
        .solar_terms
        .iter()
        .map(|t| Ok((t.instant()?, t)))
        .collect::<Result<Vec<_>, EngineError>>()?;
    let (lichun, _) = around(&terms, birth, |t| t.longitude == LICHUN);
    let (month_term, next_term) = around(&terms, birth, |t| t.jie);
    let (Some(lichun), Some(month_term), Some(next_term)) = (lichun, month_term, next_term) else {
        return Err(invalid("the calendar does not cover the birth"));
    };

    // Year and month: by the solar terms.
    let year_cycle = year_cycle_index(lichun.0.civil().0);
    let month_offset = (usize::from(month_term.1.longitude) + 360 - usize::from(LICHUN)) % 360 / 30;
    let month_stem = ((year_cycle % 10) % 5 * 2 + 2 + month_offset) % 10;
    let month_cycle = cycle_index(month_stem, (2 + month_offset) % 12);

    // Day and hour: by the local clock.
    let to_micros = |minutes: f64| (minutes * MICROS_PER_MINUTE).round() as i64;
    let civil = birth.add_micros(to_micros(utc_offset_minutes));
    let solar = birth.add_micros(to_micros(longitude * 4.0 + calendar.equation_of_time));
    let local = if options.solar_time { solar } else { civil };
    let hour = local.civil().3;
    let date = day_number(local);
    let late_zi = hour >= 23;
    let day = if late_zi && zi_hour == ZiHour::NextDay {
        date + 1
    } else {
        date
    };
    let day_cycle_of = |day: i64| (day - 11).rem_euclid(60) as usize;
    let day_cycle = day_cycle_of(day);
    let master = day_cycle % 10;
    // 子 is 23:00-01:00, 丑 01:00-03:00, …
    let hour_branch = (hour as usize).div_ceil(2) % 12;
    let hour_day_stem = day_cycle_of(if late_zi { date + 1 } else { date }) % 10;
    let hour_cycle = cycle_index((hour_day_stem % 5 * 2 + hour_branch) % 10, hour_branch);

    let luck = male.map(|male| {
        let forward = male == (year_cycle % 2 == 0);
        let days = if forward {
            next_term.0.seconds_since(&birth)
        } else {
            birth.seconds_since(&month_term.0)
        } / 86_400.0;
        let start_age = days / 3.0;
        let at_age = |age: f64| birth.plus_days(age * TROPICAL_YEAR).isoformat();
        let pillars = (1..=i64::from(options.luck_pillars))
            .map(|k| {
                let cycle = (month_cycle as i64 + if forward { k } else { -k }).rem_euclid(60);
                let age = start_age + 10.0 * (k - 1) as f64;
                LuckPillar {
                    start_age: age,
                    start: at_age(age),
                    end: at_age(age + 10.0),
                    pillar: pillar(cycle as usize, master, false),
                }
            })
            .collect();
        Luck {
            direction: if forward { "forward" } else { "backward" },
            start_age,
            pillars,
        }
    });

    Ok(Bazi {
        time_basis: if options.solar_time { "solar" } else { "civil" },
        zi_hour: match zi_hour {
            ZiHour::NextDay => "next_day",
            ZiHour::Split => "split",
        },
        civil_time: wall_clock(civil),
        solar_time: wall_clock(solar),
        year: pillar(year_cycle, master, false),
        month: pillar(month_cycle, master, false),
        day: pillar(day_cycle, master, true),
        hour: pillar(hour_cycle, master, false),
        day_master: STEMS[master],
        day_master_element: ELEMENTS[stem_element(master)],
        elements: balance([year_cycle, month_cycle, day_cycle, hour_cycle]),
        month_term: term_moment(month_term),
        next_term: term_moment(next_term),
        lunar_date: lunar_date(calendar, day)?,
        luck,
        annual: options
            .year
            .map(|year| pillar(year_cycle_index(year), master, false)),
        flow: options
            .date
            .map(|date| flow_of(date, utc_offset_minutes, master))
            .transpose()?,
    })
}
