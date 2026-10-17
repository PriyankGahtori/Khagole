//! Zi Wei Dou Shu (紫微斗數), the Purple Star chart of a birth, cast from its
//! [`ChineseCalendar`]. No kernel is needed.
//!
//! The chart follows the classic rules of the *Zi Wei Dou Shu Quanshu*:
//!
//! - The year is the lunar year (it changes at the New Year), the month and day are the lunar
//!   ones, and the hour is the birth hour's branch on the clock the Four Pillars use (true
//!   solar or civil time, either Zi-hour convention). A birth in the second half of a leap
//!   month counts in the next month (`leap_month = "split"`, the default) or in the month it
//!   repeats (`"same"`).
//! - The life palace (命宮) is counted from 寅 forward to the month and back to the hour, the
//!   body palace (身宮) forward to the hour. Palace stems follow from the year stem (五虎遁).
//! - The five-element bureau (五行局) is the NaYin element of the life palace's stem and
//!   branch. Zi Wei is placed from the bureau and the lunar day, Tian Fu opposite it across
//!   the 寅-申 axis, and each leads its group of stars.
//! - The auxiliary and malefic stars follow the month (左輔, 右弼), the hour (文昌, 文曲,
//!   地空, 地劫), the year stem (天魁, 天鉞, 祿存, 擎羊, 陀羅) and the year branch (天馬,
//!   and with the hour 火星 and 鈴星).
//! - The Four Transformations (四化) are those of the year stem; for 庚 the 科 is 太陰 and
//!   the 忌 天同, for 壬 the 科 is 左輔.
//! - Brightness (廟旺得利平不陷) follows the common table, as published by the iztro project
//!   (MIT, see NOTICE).
//! - The decade limits (大限) run forward from the life palace for a yang year and a man or
//!   a yin year and a woman, backward otherwise, from the bureau's number of years (nominal
//!   age, 虛歲). The small limits (小限) start from 辰, 戌, 未 or 丑 by the year branch, forward
//!   for a man and backward for a woman.
//! - The 38 minor stars (雜曜) and the four cycles of twelve gods (長生, 博士, 歲前 and 將前)
//!   follow the common rules, as the iztro project publishes them (its default school, with
//!   截路 and 空亡 as two stars).
//! - Each palace's stem flies its own four transformations (飛化) to the palaces of the
//!   stars it transforms.
//! - Given a year, the chart carries its horoscope: the decade (or, before the first decade,
//!   the childhood limit), the small limit and the year (流年), each with its palace, its
//!   palace names, its four transformations and its moving stars (魁鉞昌曲祿羊陀馬鸞喜), and
//!   for the year the 歲前 and 將前 gods. Given a date, the horoscope goes down to its month
//!   (流月, by 斗君: month 1 is counted from the year's palace back to the birth month and
//!   forward to the birth hour) and its day (流日), from an embedded lunar calendar.

use serde::Serialize;

use super::bazi::{bazi, nayin, BaziOptions, LunarDate, LIFE_STAGES};
use super::calendar::{day_number, ChineseCalendar};
use super::lunar_table::lunar_date_of;
use super::{cycle_index, year_cycle_index, BRANCHES, STEMS};
use crate::error::EngineError;
use crate::instant::UtcInstant;

/// The twelve palaces, counted backward through the branches from the life palace.
const PALACES: [&str; 12] = [
    "life", "siblings", "spouse", "children", "wealth", "health", "travel", "friends", "career",
    "property", "fortune", "parents",
];

const ZI_WEI: &str = "zi_wei";
const TIAN_JI: &str = "tian_ji";
const TAI_YANG: &str = "tai_yang";
const WU_QU: &str = "wu_qu";
const TIAN_TONG: &str = "tian_tong";
const LIAN_ZHEN: &str = "lian_zhen";
const TIAN_FU: &str = "tian_fu";
const TAI_YIN: &str = "tai_yin";
const TAN_LANG: &str = "tan_lang";
const JU_MEN: &str = "ju_men";
const TIAN_XIANG: &str = "tian_xiang";
const TIAN_LIANG: &str = "tian_liang";
const QI_SHA: &str = "qi_sha";
const PO_JUN: &str = "po_jun";
const ZUO_FU: &str = "zuo_fu";
const YOU_BI: &str = "you_bi";
const WEN_CHANG: &str = "wen_chang";
const WEN_QU: &str = "wen_qu";
const TIAN_KUI: &str = "tian_kui";
const TIAN_YUE: &str = "tian_yue";
const LU_CUN: &str = "lu_cun";
const TIAN_MA: &str = "tian_ma";
const QING_YANG: &str = "qing_yang";
const TUO_LUO: &str = "tuo_luo";
const HUO_XING: &str = "huo_xing";
const LING_XING: &str = "ling_xing";
const DI_KONG: &str = "di_kong";
const DI_JIE: &str = "di_jie";

/// The Zi Wei group: each star's offset from Zi Wei, counted backward.
const ZI_WEI_GROUP: [(&str, usize); 6] = [
    (ZI_WEI, 0),
    (TIAN_JI, 1),
    (TAI_YANG, 3),
    (WU_QU, 4),
    (TIAN_TONG, 5),
    (LIAN_ZHEN, 8),
];

/// The Tian Fu group: each star's offset from Tian Fu, counted forward.
const TIAN_FU_GROUP: [(&str, usize); 8] = [
    (TIAN_FU, 0),
    (TAI_YIN, 1),
    (TAN_LANG, 2),
    (JU_MEN, 3),
    (TIAN_XIANG, 4),
    (TIAN_LIANG, 5),
    (QI_SHA, 6),
    (PO_JUN, 10),
];

/// The Four Transformations (祿, 權, 科, 忌) of each year stem.
const TRANSFORMATIONS: [[&str; 4]; 10] = [
    [LIAN_ZHEN, PO_JUN, WU_QU, TAI_YANG],       // 甲
    [TIAN_JI, TIAN_LIANG, ZI_WEI, TAI_YIN],     // 乙
    [TIAN_TONG, TIAN_JI, WEN_CHANG, LIAN_ZHEN], // 丙
    [TAI_YIN, TIAN_TONG, TIAN_JI, JU_MEN],      // 丁
    [TAN_LANG, TAI_YIN, YOU_BI, TIAN_JI],       // 戊
    [WU_QU, TAN_LANG, TIAN_LIANG, WEN_QU],      // 己
    [TAI_YANG, WU_QU, TAI_YIN, TIAN_TONG],      // 庚
    [JU_MEN, TAI_YANG, WEN_QU, WEN_CHANG],      // 辛
    [TIAN_LIANG, ZI_WEI, ZUO_FU, WU_QU],        // 壬
    [PO_JUN, JU_MEN, TAI_YIN, TAN_LANG],        // 癸
];

const TRANSFORMATION_KINDS: [&str; 4] = ["lu", "quan", "ke", "ji"];

const MIAO: Option<&str> = Some("miao");
const WANG: Option<&str> = Some("wang");
const DE: Option<&str> = Some("de");
const LI: Option<&str> = Some("li");
const PING: Option<&str> = Some("ping");
const BU: Option<&str> = Some("bu");
const XIAN: Option<&str> = Some("xian");

/// Brightness of a star in each branch, from 寅 (as the iztro project publishes it).
const BRIGHTNESS: [(&str, [Option<&str>; 12]); 20] = [
    (
        ZI_WEI,
        [
            WANG, WANG, DE, WANG, MIAO, MIAO, WANG, WANG, DE, WANG, PING, MIAO,
        ],
    ),
    (
        TIAN_JI,
        [
            DE, WANG, LI, PING, MIAO, XIAN, DE, WANG, LI, PING, MIAO, XIAN,
        ],
    ),
    (
        TAI_YANG,
        [
            WANG, MIAO, WANG, WANG, WANG, DE, DE, PING, BU, XIAN, XIAN, BU,
        ],
    ),
    (
        WU_QU,
        [
            DE, LI, MIAO, PING, WANG, MIAO, DE, LI, MIAO, PING, WANG, MIAO,
        ],
    ),
    (
        TIAN_TONG,
        [
            LI, PING, PING, MIAO, XIAN, BU, WANG, PING, PING, MIAO, WANG, BU,
        ],
    ),
    (
        LIAN_ZHEN,
        [
            MIAO, PING, LI, XIAN, PING, LI, MIAO, PING, LI, XIAN, PING, LI,
        ],
    ),
    (
        TIAN_FU,
        [
            MIAO, DE, MIAO, DE, WANG, MIAO, DE, WANG, MIAO, DE, MIAO, MIAO,
        ],
    ),
    (
        TAI_YIN,
        [
            WANG, XIAN, XIAN, XIAN, BU, BU, LI, WANG, WANG, MIAO, MIAO, MIAO,
        ],
    ),
    (
        TAN_LANG,
        [
            PING, LI, MIAO, XIAN, WANG, MIAO, PING, LI, MIAO, XIAN, WANG, MIAO,
        ],
    ),
    (
        JU_MEN,
        [
            MIAO, MIAO, XIAN, WANG, WANG, BU, MIAO, MIAO, XIAN, WANG, WANG, BU,
        ],
    ),
    (
        TIAN_XIANG,
        [MIAO, XIAN, DE, DE, MIAO, DE, MIAO, XIAN, DE, DE, MIAO, MIAO],
    ),
    (
        TIAN_LIANG,
        [
            MIAO, MIAO, MIAO, XIAN, MIAO, WANG, XIAN, DE, MIAO, XIAN, MIAO, WANG,
        ],
    ),
    (
        QI_SHA,
        [
            MIAO, WANG, MIAO, PING, WANG, MIAO, MIAO, WANG, MIAO, PING, WANG, MIAO,
        ],
    ),
    (
        PO_JUN,
        [
            DE, XIAN, WANG, PING, MIAO, WANG, DE, XIAN, WANG, PING, MIAO, WANG,
        ],
    ),
    (
        WEN_CHANG,
        [XIAN, LI, DE, MIAO, XIAN, LI, DE, MIAO, XIAN, LI, DE, MIAO],
    ),
    (
        WEN_QU,
        [
            PING, WANG, DE, MIAO, XIAN, WANG, DE, MIAO, XIAN, WANG, DE, MIAO,
        ],
    ),
    (
        HUO_XING,
        [MIAO, LI, XIAN, DE, MIAO, LI, XIAN, DE, MIAO, LI, XIAN, DE],
    ),
    (
        LING_XING,
        [MIAO, LI, XIAN, DE, MIAO, LI, XIAN, DE, MIAO, LI, XIAN, DE],
    ),
    (
        QING_YANG,
        [
            None, XIAN, MIAO, None, XIAN, MIAO, None, XIAN, MIAO, None, XIAN, MIAO,
        ],
    ),
    (
        TUO_LUO,
        [
            XIAN, None, MIAO, XIAN, None, MIAO, XIAN, None, MIAO, XIAN, None, MIAO,
        ],
    ),
];

/// The life master (命主) by the life palace's branch.
const LIFE_MASTERS: [&str; 12] = [
    TAN_LANG, JU_MEN, LU_CUN, WEN_QU, LIAN_ZHEN, WU_QU, PO_JUN, WU_QU, LIAN_ZHEN, WEN_QU, LU_CUN,
    JU_MEN,
];

/// The body master (身主) by the year branch.
const BODY_MASTERS: [&str; 12] = [
    HUO_XING, TIAN_XIANG, TIAN_LIANG, TIAN_TONG, WEN_CHANG, TIAN_JI, HUO_XING, TIAN_XIANG,
    TIAN_LIANG, TIAN_TONG, WEN_CHANG, TIAN_JI,
];

/// The 博士 cycle, from 祿存.
const BOSHI: [&str; 12] = [
    "bo_shi",
    "li_shi",
    "qing_long",
    "xiao_hao",
    "jiang_jun",
    "zou_shu",
    "fei_lian",
    "xi_shen",
    "bing_fu",
    "da_hao",
    "fu_bing",
    "guan_fu",
];

/// The 歲前 cycle, from the year's branch.
const SUIQIAN: [&str; 12] = [
    "sui_jian", "hui_qi", "sang_men", "guan_suo", "guan_fu", "xiao_hao", "da_hao", "long_de",
    "bai_hu", "tian_de", "diao_ke", "bing_fu",
];

/// The 將前 cycle, from the general star of the year's trine.
const JIANGQIAN: [&str; 12] = [
    "jiang_xing",
    "pan_an",
    "sui_yi",
    "xi_shen",
    "hua_gai",
    "jie_sha",
    "zai_sha",
    "tian_sha",
    "zhi_bei",
    "xian_chi",
    "yue_sha",
    "wang_shen",
];

/// Branch indices.
const ZI: usize = 0;
const CHOU: usize = 1;
const YIN: usize = 2;
const MAO: usize = 3;
const CHEN: usize = 4;
const SI: usize = 5;
const WU: usize = 6;
const WEI: usize = 7;
const SHEN: usize = 8;
const YOU: usize = 9;
const XU: usize = 10;
const HAI: usize = 11;

/// How to cast the chart.
#[derive(Debug, Clone)]
pub struct ZiWeiOptions<'a> {
    /// Take the birth hour on true solar time at the birthplace (`true`, the default) or on
    /// civil time, as for the Four Pillars.
    pub solar_time: bool,
    /// "next_day" (default): from 23:00 the birth counts on the next lunar day; "split": the
    /// day changes at midnight.
    pub zi_hour: &'a str,
    /// "male" or "female": sets the direction of the decade and small limits, which are left
    /// out without it.
    pub sex: Option<&'a str>,
    /// "split" (default): a birth after the 15th of a leap month counts in the next month;
    /// "same": every day of a leap month counts in the month it repeats.
    pub leap_month: &'a str,
    /// A lunar year to cast the horoscope (decade, small limit, year) for.
    pub year: Option<i32>,
    /// A date, `YYYY-MM-DD`, to cast the horoscope for down to its month (流月) and day
    /// (流日); its lunar year replaces `year`. Dates from the lunar years 1900 to 2100.
    pub date: Option<&'a str>,
}

impl Default for ZiWeiOptions<'_> {
    fn default() -> Self {
        ZiWeiOptions {
            solar_time: true,
            zi_hour: "next_day",
            sex: None,
            leap_month: "split",
            year: None,
            date: None,
        }
    }
}

/// A star in a palace.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ZiWeiStar {
    /// Star code, e.g. "zi_wei", "wen_chang".
    pub star: &'static str,
    /// "major" (the fourteen), "auxiliary" or "malefic".
    pub kind: &'static str,
    /// Brightness in this branch: "miao", "wang", "de", "li", "ping", "bu" or "xian"; `None`
    /// where the table gives none.
    pub brightness: Option<&'static str>,
    /// The natal transformation it takes: "lu", "quan", "ke" or "ji".
    pub transformation: Option<&'static str>,
}

/// A range of nominal ages (虛歲), inclusive.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgeRange {
    /// First age.
    pub start: u8,
    /// Last age.
    pub end: u8,
}

/// One of the twelve palaces.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ZiWeiPalace {
    /// Palace code, from "life" to "parents".
    pub name: &'static str,
    /// Earthly Branch of its place on the board.
    pub branch: &'static str,
    /// Its Heavenly Stem.
    pub stem: &'static str,
    /// Whether the body palace falls here.
    pub is_body: bool,
    /// Its stars: the major ones first.
    pub stars: Vec<ZiWeiStar>,
    /// The decade limit it rules; `None` without the native's sex.
    pub decade: Option<AgeRange>,
    /// The nominal ages whose small limit falls here, up to 120; empty without the sex.
    pub small_limit_ages: Vec<u8>,
    /// The minor stars (雜曜) here.
    pub minor_stars: Vec<&'static str>,
    /// The gods of the four cycles of twelve here.
    pub gods: ZiWeiGods,
    /// The four transformations this palace's stem flies (飛化), to the palaces of the
    /// stars they transform.
    pub flying: Vec<ZiWeiTransformation>,
}

/// The gods of the four cycles of twelve in a palace.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ZiWeiGods {
    /// Stage of the 長生 cycle, from the bureau (as Ba Zi's `life_stage` codes); `None`
    /// without the native's sex.
    pub life_stage: Option<&'static str>,
    /// God of the 博士 cycle, from 祿存; `None` without the native's sex.
    pub boshi: Option<&'static str>,
    /// God of the 歲前 cycle of the birth year.
    pub suiqian: &'static str,
    /// God of the 將前 cycle of the birth year.
    pub jiangqian: &'static str,
}

/// A moving star of a horoscope period.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ZiWeiFlowStar {
    /// Star code: "tian_kui", "tian_yue", "wen_chang", "wen_qu", "lu_cun", "qing_yang",
    /// "tuo_luo", "tian_ma", "hong_luan" or "tian_xi".
    pub star: &'static str,
    /// Branch of the palace it moves to.
    pub branch: &'static str,
}

/// A god of a yearly cycle of twelve.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ZiWeiGod {
    /// God code.
    pub god: &'static str,
    /// Branch of its palace.
    pub branch: &'static str,
}

/// One period of a horoscope: a decade, the childhood limit or a year.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ZiWeiPeriod {
    /// Branch of the palace the period's life palace falls in.
    pub branch: &'static str,
    /// The period's stem (the decade palace's, or the year's).
    pub stem: &'static str,
    /// The palace names of the period, by branch, life first.
    pub palaces: Vec<ZiWeiGod>,
    /// The four transformations of the period's stem, with the natal palace of each star.
    pub transformations: Vec<ZiWeiTransformation>,
    /// The period's moving stars.
    pub stars: Vec<ZiWeiFlowStar>,
}

/// The horoscope of a year.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ZiWeiHoroscope {
    /// The lunar year.
    pub year: i32,
    /// Nominal age (虛歲) in that year.
    pub nominal_age: i32,
    /// The decade limit running in that year; `None` without the native's sex or outside
    /// the decades.
    pub decade: Option<ZiWeiPeriod>,
    /// Whether `decade` is the childhood limit (童限), before the first decade starts.
    pub childhood: bool,
    /// Branch of the small limit's palace; `None` without the native's sex.
    pub small_limit: Option<&'static str>,
    /// The year (流年): its life palace is the year's branch.
    pub yearly: ZiWeiPeriod,
    /// The 歲前 gods of the year.
    pub suiqian: Vec<ZiWeiGod>,
    /// The 將前 gods of the year.
    pub jiangqian: Vec<ZiWeiGod>,
    /// The date read, when one was given.
    pub date: Option<String>,
    /// Its lunar date.
    pub lunar_date: Option<LunarDate>,
    /// The month (流月) of the date: month 1 is counted from the year's palace back to the
    /// birth month and forward to the birth hour (斗君), one palace a month.
    pub monthly: Option<ZiWeiPeriod>,
    /// The day (流日) of the date: from the month's palace, one palace a day.
    pub daily: Option<ZiWeiPeriod>,
}

/// A natal transformation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ZiWeiTransformation {
    /// "lu" (祿), "quan" (權), "ke" (科) or "ji" (忌).
    pub kind: &'static str,
    /// The star transformed.
    pub star: &'static str,
    /// The palace it is in.
    pub palace: &'static str,
}

/// The five-element bureau.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Bureau {
    /// "water", "wood", "metal", "earth" or "fire".
    pub element: &'static str,
    /// 2 (water) to 6 (fire): the age the first decade starts at.
    pub number: u8,
}

/// The Purple Star chart of a birth.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ZiWei {
    /// "solar" or "civil": the clock the birth hour was taken on.
    pub time_basis: &'static str,
    /// "next_day" or "split".
    pub zi_hour: &'static str,
    /// The lunar date of birth.
    pub lunar_date: LunarDate,
    /// The month the chart is cast with (a leap month may count as the next one).
    pub month: u8,
    /// The birth hour's branch.
    pub hour_branch: &'static str,
    /// Branch of the life palace (命宮).
    pub life_palace: &'static str,
    /// Branch of the body palace (身宮).
    pub body_palace: &'static str,
    /// The five-element bureau (五行局).
    pub bureau: Bureau,
    /// The life master (命主), a star code.
    pub life_master: &'static str,
    /// The body master (身主), a star code.
    pub body_master: &'static str,
    /// The twelve palaces, from the life palace in palace order.
    pub palaces: Vec<ZiWeiPalace>,
    /// The four natal transformations, 祿 first.
    pub transformations: Vec<ZiWeiTransformation>,
    /// "forward" or "backward" (through the branches) for the decade limits; `None`
    /// without the native's sex.
    pub decade_direction: Option<&'static str>,
    /// The horoscope of the requested year; `None` when none was asked for.
    pub horoscope: Option<ZiWeiHoroscope>,
}

fn invalid(message: &str) -> EngineError {
    EngineError::InvalidInput(message.to_string())
}

fn wrap(index: i64) -> usize {
    index.rem_euclid(12) as usize
}

/// The branch Zi Wei sits in for a bureau and a lunar day (1-30).
pub(crate) fn zi_wei_branch(bureau: u8, day: u8) -> usize {
    let (bureau, day) = (i64::from(bureau), i64::from(day));
    let quotient = (day + bureau - 1) / bureau;
    let added = quotient * bureau - day;
    let base = YIN as i64 + quotient - 1;
    wrap(if added % 2 == 1 {
        base - added
    } else {
        base + added
    })
}

/// The bureau of a life palace's stem and branch: its NaYin element.
fn bureau_of(stem: usize, branch: usize) -> Bureau {
    let nayin = nayin(cycle_index(stem, branch) / 2);
    let (element, number) = [
        ("water", 2),
        ("wood", 3),
        ("metal", 4),
        ("earth", 5),
        ("fire", 6),
    ]
    .into_iter()
    .find(|(element, _)| nayin.ends_with(element))
    .unwrap_or(("fire", 6));
    Bureau { element, number }
}

fn brightness(star: &str, branch: usize) -> Option<&'static str> {
    BRIGHTNESS
        .iter()
        .find(|(s, _)| *s == star)
        .and_then(|(_, table)| table[wrap(branch as i64 - YIN as i64)])
}

/// 天魁 and 天鉞 of a stem.
fn kui_yue(stem: usize) -> (usize, usize) {
    match stem {
        0 | 4 | 6 => (CHOU, WEI),
        1 | 5 => (ZI, SHEN),
        2 | 3 => (HAI, YOU),
        7 => (WU, YIN),
        _ => (MAO, SI),
    }
}

/// 祿存 of a stem.
fn lu_cun_of(stem: usize) -> usize {
    [YIN, MAO, SI, WU, SI, WU, SHEN, YOU, HAI, ZI][stem]
}

/// The trine of a branch: 0 申子辰, 1 巳酉丑, 2 寅午戌, 3 亥卯未.
fn trine_of(branch: usize) -> usize {
    branch % 4
}

/// 天馬 of a branch.
fn horse_of(branch: usize) -> usize {
    [YIN, HAI, SHEN, SI][trine_of(branch)]
}

/// 紅鸞 and 天喜 of a branch.
fn luan_xi(branch: usize) -> (usize, usize) {
    let luan = wrap(MAO as i64 - branch as i64);
    (luan, wrap(luan as i64 + 6))
}

/// The moving 文昌 and 文曲 of a stem (流昌, 流曲).
fn flow_chang_qu(stem: usize) -> (usize, usize) {
    match stem {
        0 => (SI, YOU),
        1 => (WU, SHEN),
        2 | 4 => (SHEN, WU),
        3 | 5 => (YOU, SI),
        6 => (HAI, MAO),
        7 => (ZI, YIN),
        8 => (YIN, ZI),
        _ => (MAO, HAI),
    }
}

/// The ten moving stars of a period's stem and branch.
fn flow_stars(stem: usize, branch: usize) -> Vec<ZiWeiFlowStar> {
    let (kui, yue) = kui_yue(stem);
    let (chang, qu) = flow_chang_qu(stem);
    let lu = lu_cun_of(stem) as i64;
    let (luan, xi) = luan_xi(branch);
    [
        (TIAN_KUI, kui),
        (TIAN_YUE, yue),
        (WEN_CHANG, chang),
        (WEN_QU, qu),
        (LU_CUN, wrap(lu)),
        (QING_YANG, wrap(lu + 1)),
        (TUO_LUO, wrap(lu - 1)),
        (TIAN_MA, horse_of(branch)),
        ("hong_luan", luan),
        ("tian_xi", xi),
    ]
    .into_iter()
    .map(|(star, b)| ZiWeiFlowStar {
        star,
        branch: BRANCHES[b],
    })
    .collect()
}

/// A cycle of twelve gods laid from `start`, forward or backward.
fn cycle(gods: &[&'static str; 12], start: usize, forward: bool) -> Vec<ZiWeiGod> {
    (0..12)
        .map(|i| ZiWeiGod {
            god: gods[i],
            branch: BRANCHES[wrap(start as i64 + if forward { i as i64 } else { -(i as i64) })],
        })
        .collect()
}

/// The 歲前 and 將前 gods of a year's branch.
fn yearly_gods(branch: usize) -> (Vec<ZiWeiGod>, Vec<ZiWeiGod>) {
    let general = [ZI, YOU, WU, MAO][trine_of(branch)];
    (
        cycle(&SUIQIAN, branch, true),
        cycle(&JIANGQIAN, general, true),
    )
}

/// The 38 minor stars, by branch.
#[allow(clippy::too_many_arguments)]
fn minor_stars(
    year_stem: usize,
    year_branch: usize,
    month: i64,
    day: i64,
    hour: i64,
    life: usize,
    body: usize,
    anchors: [usize; 4],
) -> Vec<(usize, &'static str)> {
    let (s, b, m) = (year_stem, year_branch as i64, month as usize);
    let [zuo, you, chang, qu] = anchors.map(|a| a as i64);
    let d = day - 1;
    let (luan, xi) = luan_xi(year_branch);
    let (hua_gai, xian_chi) =
        [(CHEN, YOU), (CHOU, WU), (XU, MAO), (WEI, ZI)][trine_of(year_branch)];
    let (gu_chen, gua_su) =
        [(SI, CHOU), (SHEN, CHEN), (HAI, WEI), (YIN, XU)][(year_branch + 10) % 12 / 3];
    let mut xun_kong = wrap(b + 10 - s as i64);
    if xun_kong % 2 != year_branch % 2 {
        xun_kong = wrap(xun_kong as i64 + 1);
    }
    // 天傷 sits in the friends palace, 天使 in the health palace.
    let palace_at = |offset: i64| wrap(life as i64 - offset);
    vec![
        (luan, "hong_luan"),
        (xi, "tian_xi"),
        (wrap(CHOU as i64 + month), "tian_yao"),
        (xian_chi, "xian_chi"),
        ([SHEN, XU, ZI, YIN, CHEN, WU][m / 2], "jie_shen"),
        (wrap(zuo + d), "san_tai"),
        (wrap(you - d), "ba_zuo"),
        (wrap(chang + d - 1), "en_guang"),
        (wrap(qu + d - 1), "tian_gui"),
        (wrap(CHEN as i64 + b), "long_chi"),
        (wrap(XU as i64 - b), "feng_ge"),
        (wrap(life as i64 + b), "tian_cai"),
        (wrap(body as i64 + b), "tian_shou"),
        (wrap(WU as i64 + hour), "tai_fu"),
        (wrap(YIN as i64 + hour), "feng_gao"),
        ([SI, SHEN, YIN, HAI][m % 4], "tian_wu"),
        (hua_gai, "hua_gai"),
        (
            [WEI, CHEN, SI, YIN, MAO, YOU, HAI, YOU, XU, WU][s],
            "tian_guan",
        ),
        (
            [YOU, SHEN, ZI, HAI, MAO, YIN, WU, SI, WU, SI][s],
            "tian_fu_blessing",
        ),
        ([SI, WU, ZI, SI, WU, SHEN, YIN, WU, YOU, HAI][s], "tian_chu"),
        (
            [XU, SI, CHEN, YIN, WEI, MAO, HAI, WEI, YIN, WU, XU, YIN][m],
            "tian_yue_moon",
        ),
        (wrap(YOU as i64 + b), "tian_de"),
        (wrap(SI as i64 + b), "yue_de"),
        (wrap(b + 1), "tian_kong"),
        (xun_kong, "xun_kong"),
        ([SHEN, WU, CHEN, YIN, ZI][s % 5], "jie_lu"),
        ([YOU, WEI, SI, MAO, CHOU][s % 5], "kong_wang"),
        (gu_chen, "gu_chen"),
        (gua_su, "gua_su"),
        (
            [SHEN, YOU, XU, SI, WU, WEI, YIN, MAO, CHEN, HAI, ZI, CHOU][year_branch],
            "fei_lian",
        ),
        ([SI, CHOU, YOU][year_branch % 3], "po_sui"),
        (wrap(YOU as i64 + month), "tian_xing"),
        ([YIN, ZI, XU, SHEN, WU, CHEN][m % 6], "yin_sha"),
        (wrap(WU as i64 - b), "tian_ku"),
        (wrap(WU as i64 + b), "tian_xu"),
        (palace_at(5), "tian_shi"),
        (palace_at(7), "tian_shang"),
        (wrap(XU as i64 - b), "nian_jie"),
    ]
}

/// Cast the Purple Star chart of a birth at `birth` (UTC), at `longitude` (degrees east)
/// where civil time was `utc_offset_minutes` ahead of UTC, from its `calendar`.
pub fn zi_wei(
    calendar: &ChineseCalendar,
    birth: UtcInstant,
    longitude: f64,
    utc_offset_minutes: f64,
    options: &ZiWeiOptions,
) -> Result<ZiWei, EngineError> {
    let split_leap = match options.leap_month {
        "" | "split" => true,
        "same" => false,
        _ => return Err(invalid("leap_month must be \"split\" or \"same\"")),
    };
    let male = match options.sex {
        None | Some("") => None,
        Some("male") => Some(true),
        Some("female") => Some(false),
        Some(_) => return Err(invalid("sex must be \"male\" or \"female\"")),
    };
    let pillars = bazi(
        calendar,
        birth,
        longitude,
        utc_offset_minutes,
        &BaziOptions {
            solar_time: options.solar_time,
            zi_hour: options.zi_hour,
            sex: None,
            luck_pillars: 0,
            year: None,
            date: None,
        },
    )?;
    let lunar = pillars
        .lunar_date
        .clone()
        .ok_or_else(|| invalid("the calendar does not cover the birth date"))?;
    let hour = BRANCHES
        .iter()
        .position(|b| *b == pillars.hour.branch)
        .unwrap_or(ZI);
    let month = if lunar.leap && split_leap && lunar.day > 15 {
        lunar.month % 12 + 1
    } else {
        lunar.month
    };
    let year = year_cycle_index(lunar.year);
    let (year_stem, year_branch) = (year % 10, year % 12);

    // Life and body palaces, palace stems and the bureau.
    let month_branch = YIN as i64 + i64::from(month) - 1;
    let life = wrap(month_branch - hour as i64);
    let body = wrap(month_branch + hour as i64);
    let yin_stem = year_stem % 5 * 2 + 2;
    let stem_of = |branch: usize| (yin_stem + wrap(branch as i64 - YIN as i64)) % 10;
    let bureau = bureau_of(stem_of(life), life);

    // Stars, by branch.
    let mut placed: Vec<(usize, &'static str, &'static str)> = Vec::with_capacity(28);
    let zi_wei_at = zi_wei_branch(bureau.number, lunar.day) as i64;
    for (star, offset) in ZI_WEI_GROUP {
        placed.push((wrap(zi_wei_at - offset as i64), star, "major"));
    }
    let tian_fu_at = 4 - zi_wei_at;
    for (star, offset) in TIAN_FU_GROUP {
        placed.push((wrap(tian_fu_at + offset as i64), star, "major"));
    }
    let (m, h) = (i64::from(month) - 1, hour as i64);
    let (kui, yue) = kui_yue(year_stem);
    let lu_cun = lu_cun_of(year_stem) as i64;
    let horse = horse_of(year_branch);
    let (fire, bell, small_start) = match trine_of(year_branch) {
        0 => (YIN, XU, XU),     // 申子辰
        1 => (MAO, XU, WEI),    // 巳酉丑
        2 => (CHOU, MAO, CHEN), // 寅午戌
        _ => (YOU, XU, CHOU),   // 亥卯未
    };
    let anchors = [
        wrap(CHEN as i64 + m),
        wrap(XU as i64 - m),
        wrap(XU as i64 - h),
        wrap(CHEN as i64 + h),
    ];
    let [zuo, you, chang, qu] = anchors;
    for (branch, star, kind) in [
        (zuo, ZUO_FU, "auxiliary"),
        (you, YOU_BI, "auxiliary"),
        (chang, WEN_CHANG, "auxiliary"),
        (qu, WEN_QU, "auxiliary"),
        (kui, TIAN_KUI, "auxiliary"),
        (yue, TIAN_YUE, "auxiliary"),
        (wrap(lu_cun), LU_CUN, "auxiliary"),
        (horse, TIAN_MA, "auxiliary"),
        (wrap(lu_cun + 1), QING_YANG, "malefic"),
        (wrap(lu_cun - 1), TUO_LUO, "malefic"),
        (wrap(fire as i64 + h), HUO_XING, "malefic"),
        (wrap(bell as i64 + h), LING_XING, "malefic"),
        (wrap(HAI as i64 - h), DI_KONG, "malefic"),
        (wrap(HAI as i64 + h), DI_JIE, "malefic"),
    ] {
        placed.push((branch, star, kind));
    }

    let transformed = TRANSFORMATIONS[year_stem];
    let transformation_of = |star: &str| {
        transformed
            .iter()
            .position(|s| *s == star)
            .map(|k| TRANSFORMATION_KINDS[k])
    };

    let minor = minor_stars(
        year_stem,
        year_branch,
        m,
        i64::from(lunar.day),
        h,
        life,
        body,
        anchors,
    );

    // Limits and the cycles of twelve.
    let forward = male.map(|male| male == (year_stem % 2 == 0));
    let palace_name = |branch: usize| PALACES[wrap(life as i64 - branch as i64)];
    let star_branch = |star: &str| placed.iter().find(|(_, s, _)| *s == star).map(|p| p.0);
    let transformations_of = |stem: usize| -> Vec<ZiWeiTransformation> {
        TRANSFORMATIONS[stem]
            .iter()
            .zip(TRANSFORMATION_KINDS)
            .filter_map(|(&star, kind)| {
                Some(ZiWeiTransformation {
                    kind,
                    star,
                    palace: palace_name(star_branch(star)?),
                })
            })
            .collect()
    };
    let life_stages = forward.map(|forward| {
        let start = match bureau.number {
            2 | 5 => SHEN,
            3 => HAI,
            4 => SI,
            _ => YIN,
        };
        cycle(&LIFE_STAGES, start, forward)
    });
    let boshi = forward.map(|forward| cycle(&BOSHI, lu_cun as usize, forward));
    let (suiqian, jiangqian) = yearly_gods(year_branch);
    let god_at = |gods: &[ZiWeiGod], branch: usize| {
        gods.iter()
            .find(|g| g.branch == BRANCHES[branch])
            .map(|g| g.god)
            .unwrap_or_default()
    };

    let palaces: Vec<ZiWeiPalace> = (0..12)
        .map(|i| {
            let branch = wrap(life as i64 - i);
            let stars = placed
                .iter()
                .filter(|(b, ..)| *b == branch)
                .map(|&(_, star, kind)| ZiWeiStar {
                    star,
                    kind,
                    brightness: brightness(star, branch),
                    transformation: transformation_of(star),
                })
                .collect();
            let decade = forward.map(|forward| {
                let steps = if forward {
                    wrap(branch as i64 - life as i64)
                } else {
                    wrap(life as i64 - branch as i64)
                } as u8;
                let start = bureau.number + 10 * steps;
                AgeRange {
                    start,
                    end: start + 9,
                }
            });
            let small_limit_ages = male
                .map(|male| {
                    let steps = if male {
                        wrap(branch as i64 - small_start as i64)
                    } else {
                        wrap(small_start as i64 - branch as i64)
                    } as u8;
                    (0..10).map(|j| steps + 1 + 12 * j).collect()
                })
                .unwrap_or_default();
            ZiWeiPalace {
                name: PALACES[i as usize],
                branch: BRANCHES[branch],
                stem: STEMS[stem_of(branch)],
                is_body: branch == body,
                stars,
                decade,
                small_limit_ages,
                minor_stars: minor
                    .iter()
                    .filter(|(b, _)| *b == branch)
                    .map(|&(_, star)| star)
                    .collect(),
                gods: ZiWeiGods {
                    life_stage: life_stages.as_ref().map(|g| god_at(g, branch)),
                    boshi: boshi.as_ref().map(|g| god_at(g, branch)),
                    suiqian: god_at(&suiqian, branch),
                    jiangqian: god_at(&jiangqian, branch),
                },
                flying: transformations_of(stem_of(branch)),
            }
        })
        .collect();

    let transformations = transformations_of(year_stem);

    // The horoscope of the requested year.
    let target_date = options
        .date
        .map(|text| {
            let day = UtcInstant::parse(&format!("{text}T00:00:00Z"))
                .map(day_number)
                .map_err(|_| invalid("date must be YYYY-MM-DD"))?;
            let (y, m, leap, d) = lunar_date_of(day)
                .ok_or_else(|| invalid("date is outside the lunar years 1900-2100"))?;
            Ok::<_, EngineError>((text.to_string(), day, y, m, leap, d))
        })
        .transpose()?;
    let horoscope = target_date
        .as_ref()
        .map(|t| t.2)
        .or(options.year)
        .map(|target| {
            let age = target - lunar.year + 1;
            let period = |branch: usize, stem: usize, flow_branch: usize| ZiWeiPeriod {
                branch: BRANCHES[branch],
                stem: STEMS[stem],
                palaces: (0..12)
                    .map(|k| ZiWeiGod {
                        god: PALACES[k],
                        branch: BRANCHES[wrap(branch as i64 - k as i64)],
                    })
                    .collect(),
                transformations: transformations_of(stem),
                stars: flow_stars(stem, flow_branch),
            };
            let decade_branch = palaces.iter().find_map(|p: &ZiWeiPalace| {
                let d = p.decade.as_ref()?;
                (i32::from(d.start) <= age && age <= i32::from(d.end)).then_some(p.branch)
            });
            let childhood = forward.is_some()
                && decade_branch.is_none()
                && (1..i32::from(bureau.number)).contains(&age)
                && age <= 6;
            let decade_at = decade_branch
                .and_then(|b| BRANCHES.iter().position(|x| *x == b))
                .or_else(|| {
                    childhood.then(|| {
                        let name = ["life", "wealth", "health", "spouse", "fortune", "career"]
                            [(age - 1) as usize];
                        wrap(
                            life as i64
                                - PALACES.iter().position(|p| *p == name).unwrap_or(0) as i64,
                        )
                    })
                });
            let small_limit = palaces
                .iter()
                .find(|p: &&ZiWeiPalace| {
                    u8::try_from(age).is_ok_and(|a| p.small_limit_ages.contains(&a))
                })
                .map(|p| p.branch);
            let cycle_index = year_cycle_index(target);
            let (y_stem, y_branch) = (cycle_index % 10, cycle_index % 12);
            let (suiqian, jiangqian) = yearly_gods(y_branch);
            // The month and day of the date read.
            let flows = target_date.as_ref().map(|(_, day, _, m, leap, d)| {
                let m = if *leap && *d > 15 { m % 12 + 1 } else { *m };
                let month_palace =
                    wrap(y_branch as i64 - i64::from(month) + hour as i64 + i64::from(m));
                let month_branch = wrap(YIN as i64 + i64::from(m) - 1);
                let month_stem = (y_stem % 5 * 2 + 2 + usize::from(m) - 1) % 10;
                let day_cycle = (day - 11).rem_euclid(60) as usize;
                let day_palace = wrap(month_palace as i64 + i64::from(*d) - 1);
                (
                    period(month_palace, month_stem, month_branch),
                    period(day_palace, day_cycle % 10, day_cycle % 12),
                )
            });
            let (monthly, daily) = flows.map_or((None, None), |(m, d)| (Some(m), Some(d)));
            ZiWeiHoroscope {
                year: target,
                nominal_age: age,
                decade: decade_at.map(|b| period(b, stem_of(b), b)),
                childhood: childhood && decade_branch.is_none(),
                small_limit,
                yearly: period(y_branch, y_stem, y_branch),
                suiqian,
                jiangqian,
                date: target_date.as_ref().map(|t| t.0.clone()),
                lunar_date: target_date.as_ref().map(|&(_, _, y, m, leap, d)| {
                    let cycle = year_cycle_index(y);
                    LunarDate {
                        year: y,
                        month: m,
                        leap,
                        day: d,
                        year_stem: STEMS[cycle % 10],
                        year_branch: BRANCHES[cycle % 12],
                        animal: super::ANIMALS[cycle % 12],
                    }
                }),
                monthly,
                daily,
            }
        });

    Ok(ZiWei {
        time_basis: pillars.time_basis,
        zi_hour: pillars.zi_hour,
        lunar_date: lunar,
        month,
        hour_branch: BRANCHES[hour],
        life_palace: BRANCHES[life],
        body_palace: BRANCHES[body],
        bureau,
        life_master: LIFE_MASTERS[life],
        body_master: BODY_MASTERS[year_branch],
        palaces,
        transformations,
        decade_direction: forward.map(|f| if f { "forward" } else { "backward" }),
        horoscope,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zi_wei_from_the_bureau_and_the_day() {
        // The examples of the placement rhyme: day 27, wood 3 → 戌; day 13, fire 6 → 亥;
        // day 6, earth 5 → 未; and the first two days of the water bureau.
        assert_eq!(BRANCHES[zi_wei_branch(3, 27)], "xu");
        assert_eq!(BRANCHES[zi_wei_branch(6, 13)], "hai");
        assert_eq!(BRANCHES[zi_wei_branch(5, 6)], "wei");
        assert_eq!(BRANCHES[zi_wei_branch(2, 1)], "chou");
        assert_eq!(BRANCHES[zi_wei_branch(2, 2)], "yin");
        assert_eq!(BRANCHES[zi_wei_branch(6, 1)], "you");
    }

    #[test]
    fn the_bureau_is_the_nayin_of_the_life_palace() {
        // 癸未 is willow wood: the wood bureau, 3.
        assert_eq!(
            bureau_of(9, 7),
            Bureau {
                element: "wood",
                number: 3
            }
        );
        // 甲子 is sea metal: 4.
        assert_eq!(bureau_of(0, 0).number, 4);
        // 丙子 is stream water: 2.
        assert_eq!(bureau_of(2, 0).number, 2);
    }
}
