//! The Chinese calendar, Ba Zi and Zi Wei Dou Shu, checked against published dates (Hong Kong
//! Observatory, Purple Mountain Observatory almanacs) and well-known charts.

mod common;

use khagole_engine::ephemeris::{Kernel, KernelSet, Spk};
use khagole_engine::{
    bazi, calculate_chart, chinese_calendar, zi_wei, BaziOptions, ChartRequest, ChineseCalendar,
    UtcInstant, ZiWei, ZiWeiOptions, ZiWeiPalace,
};
use common::{kernels, root};

fn at(utc: &str) -> UtcInstant {
    UtcInstant::parse(utc).unwrap()
}

fn calendar(kernels: &KernelSet, utc: &str) -> ChineseCalendar {
    chinese_calendar(kernels, at(utc)).unwrap().unwrap()
}

/// Seconds between two ISO 8601 instants.
fn seconds_between(a: &str, b: &str) -> f64 {
    at(a).seconds_since(&at(b)).abs()
}

#[test]
fn chinese_new_year() {
    let Some(kernels) = kernels() else { return };
    for (year, date) in [
        (1900, "1900-01-31"),
        (1912, "1912-02-18"),
        (1929, "1929-02-10"),
        (1949, "1949-01-29"),
        (1950, "1950-02-17"),
        (1984, "1984-02-02"),
        (1990, "1990-01-27"),
        (2000, "2000-02-05"),
        (2001, "2001-01-24"),
        (2010, "2010-02-14"),
        (2017, "2017-01-28"),
        (2020, "2020-01-25"),
        (2023, "2023-01-22"),
        (2024, "2024-02-10"),
        (2025, "2025-01-29"),
        (2026, "2026-02-17"),
        (2030, "2030-02-03"),
        (2033, "2033-01-31"),
        (2050, "2050-01-23"),
        (2100, "2100-02-09"),
    ] {
        // Noon in China on New Year's day.
        let cal = calendar(&kernels, &format!("{date}T04:00:00Z"));
        let first = cal
            .lunar_months
            .iter()
            .find(|m| m.month == 1 && !m.leap)
            .unwrap_or_else(|| panic!("{year}: no month 1 in {:?}", cal.lunar_months));
        assert_eq!((first.year, first.start.as_str()), (year, date), "{year}");
    }
}

#[test]
fn leap_months() {
    let Some(kernels) = kernels() else { return };
    for (year, month, start) in [
        (1984, 10, "1984-11-23"),
        (2001, 4, "2001-05-23"),
        (2004, 2, "2004-03-21"),
        (2006, 7, "2006-08-24"),
        (2009, 5, "2009-06-23"),
        (2012, 4, "2012-05-21"),
        (2014, 9, "2014-10-24"),
        (2017, 6, "2017-07-23"),
        (2020, 4, "2020-05-23"),
        (2023, 2, "2023-03-22"),
        (2025, 6, "2025-07-25"),
        // The "2033 problem": a month without a major term comes earlier in the year, but
        // the leap month is the eleventh.
        (2033, 11, "2033-12-22"),
    ] {
        let cal = calendar(&kernels, &format!("{start}T12:00:00Z"));
        let leap = cal
            .lunar_months
            .iter()
            .find(|m| m.leap)
            .unwrap_or_else(|| panic!("{year}: no leap month in {:?}", cal.lunar_months));
        assert_eq!(
            (leap.year, leap.month, leap.start.as_str()),
            (year, month, start),
            "{year}"
        );
    }
}

#[test]
fn no_month_is_leap_in_a_twelve_month_year() {
    let Some(kernels) = kernels() else { return };
    // 2033's seventh month has no major term, but its year has twelve months.
    let cal = calendar(&kernels, "2033-08-30T12:00:00Z");
    assert!(
        cal.lunar_months.iter().all(|m| !m.leap),
        "{:?}",
        cal.lunar_months
    );
}

#[test]
fn solar_terms_match_published_times() {
    let Some(kernels) = kernels() else { return };
    let cal = calendar(&kernels, "2000-12-25T00:00:00Z");
    let term = |name: &str| {
        cal.solar_terms
            .iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("no {name}"))
    };
    // US Naval Observatory: equinox and solstices of 2000.
    for (name, utc) in [
        ("chunfen", "2000-03-20T07:35:00Z"),
        ("xiazhi", "2000-06-21T01:48:00Z"),
        ("qiufen", "2000-09-22T17:27:00Z"),
        ("dongzhi", "2000-12-21T13:37:00Z"),
    ] {
        let t = term(name);
        assert!(seconds_between(&t.utc, utc) < 60.0, "{name}: {}", t.utc);
    }
    assert_eq!(cal.solar_terms[0].name, "lichun");
    let last = cal.solar_terms.last().unwrap();
    assert_eq!((last.name.as_str(), last.jie), ("xiaohan", true));
    assert!(cal.solar_terms.windows(2).all(|w| w[0].utc < w[1].utc));

    // 立春 2024: 16:27 Beijing time.
    let cal = calendar(&kernels, "2024-03-01T00:00:00Z");
    assert!(seconds_between(&cal.solar_terms[0].utc, "2024-02-04T08:27:00Z") < 60.0);
}

#[test]
fn equation_of_time() {
    let Some(kernels) = kernels() else { return };
    for (utc, minutes) in [
        ("2000-11-03T12:00:00Z", 16.4),
        ("2000-02-11T12:00:00Z", -14.2),
        ("2000-04-15T12:00:00Z", 0.0),
    ] {
        let eot = calendar(&kernels, utc).equation_of_time;
        assert!((eot - minutes).abs() < 0.3, "{utc}: {eot}");
    }
}

#[test]
fn outside_the_kernels_there_is_no_calendar() {
    // The committed excerpt covers 2000 only; the calendar needs the year before.
    let mut kernels = KernelSet::new();
    let path = root().join("tests/data/de440s_2000.bsp");
    kernels.push(Kernel::new("excerpt", Spk::open(path).unwrap()).unwrap());
    assert_eq!(
        chinese_calendar(&kernels, at("2000-06-01T00:00:00Z")).unwrap(),
        None
    );
}

#[test]
fn the_chart_carries_the_calendar_only_on_request() {
    let Some(kernels) = kernels() else { return };
    let mut req = ChartRequest::new(at("1987-05-17T14:30:00Z"), 41.9, 12.5);
    assert!(calculate_chart(&kernels, &req)
        .unwrap()
        .chinese_calendar
        .is_none());
    req.chinese_calendar = true;
    let chart = calculate_chart(&kernels, &req).unwrap();
    assert_eq!(
        chart.chinese_calendar,
        chinese_calendar(&kernels, req.instant).unwrap()
    );
}

/// "stem branch" of the four pillars, year first.
fn pillars(
    kernels: &KernelSet,
    utc: &str,
    longitude: f64,
    offset: f64,
    options: &BaziOptions,
) -> [String; 4] {
    let cal = calendar(kernels, utc);
    let b = bazi(&cal, at(utc), longitude, offset, options).unwrap();
    [&b.year, &b.month, &b.day, &b.hour].map(|p| format!("{} {}", p.stem, p.branch))
}

#[test]
fn published_charts() {
    let Some(kernels) = kernels() else { return };
    // Bruce Lee, San Francisco, 1940-11-27 07:12 PST: 庚辰 丁亥 甲戌 戊辰.
    assert_eq!(
        pillars(
            &kernels,
            "1940-11-27T15:12:00Z",
            -122.42,
            -480.0,
            &BaziOptions::default()
        ),
        ["geng chen", "ding hai", "jia xu", "wu chen"]
    );
    // Mao Zedong, Shaoshan, 1893-12-26, 辰 hour: 癸巳 甲子 丁酉 甲辰.
    let civil = BaziOptions {
        solar_time: false,
        ..BaziOptions::default()
    };
    assert_eq!(
        pillars(&kernels, "1893-12-26T00:00:00Z", 112.53, 480.0, &civil),
        ["gui si", "jia zi", "ding you", "jia chen"]
    );
}

#[test]
fn day_pillar_anchors() {
    let Some(kernels) = kernels() else { return };
    let civil = BaziOptions {
        solar_time: false,
        ..BaziOptions::default()
    };
    // Noon in Beijing.
    for (date, day) in [("1949-10-01", "jia zi"), ("2000-01-01", "wu wu")] {
        let p = pillars(&kernels, &format!("{date}T04:00:00Z"), 116.4, 480.0, &civil);
        assert_eq!(p[2], day, "{date}");
    }
}

#[test]
fn the_year_and_month_change_at_the_jie_term() {
    let Some(kernels) = kernels() else { return };
    let lichun = calendar(&kernels, "2024-03-01T00:00:00Z").solar_terms[0]
        .utc
        .clone();
    let lichun = at(&lichun);
    let options = BaziOptions::default();
    let before = lichun.add_micros(-60_000_000).isoformat();
    let after = lichun.add_micros(60_000_000).isoformat();
    let p = pillars(&kernels, &before, 116.4, 480.0, &options);
    assert_eq!(p[..2], ["gui mao", "yi chou"]);
    let p = pillars(&kernels, &after, 116.4, 480.0, &options);
    assert_eq!(p[..2], ["jia chen", "bing yin"]);
}

#[test]
fn zi_hour_conventions() {
    let Some(kernels) = kernels() else { return };
    // 23:30 civil time in Beijing on 2000-01-01 (a 戊午 day).
    let utc = "2000-01-01T15:30:00Z";
    let next_day = BaziOptions {
        solar_time: false,
        ..BaziOptions::default()
    };
    let split = BaziOptions {
        zi_hour: "split",
        ..next_day.clone()
    };
    let a = pillars(&kernels, utc, 116.4, 480.0, &next_day);
    let b = pillars(&kernels, utc, 116.4, 480.0, &split);
    assert_eq!(a[2], "ji wei");
    assert_eq!(b[2], "wu wu");
    // Either way the hour is the 子 hour of the 己 day: 甲子.
    assert_eq!(a[3], "jia zi");
    assert_eq!(b[3], "jia zi");
}

#[test]
fn solar_time_moves_the_hour() {
    let Some(kernels) = kernels() else { return };
    // 12:50 civil time in Ürümqi (87.6° E) on China Standard Time is 10:43 solar.
    let utc = "2010-06-01T04:50:00Z";
    let cal = calendar(&kernels, utc);
    let solar = bazi(&cal, at(utc), 87.6, 480.0, &BaziOptions::default()).unwrap();
    assert_eq!(solar.civil_time, "2010-06-01T12:50:00");
    assert!(
        solar.solar_time.starts_with("2010-06-01T10:4"),
        "{}",
        solar.solar_time
    );
    assert_eq!(solar.hour.branch, "si");
    let civil = BaziOptions {
        solar_time: false,
        ..BaziOptions::default()
    };
    let civil = bazi(&cal, at(utc), 87.6, 480.0, &civil).unwrap();
    assert_eq!(civil.hour.branch, "wu");
}

#[test]
fn luck_pillars() {
    let Some(kernels) = kernels() else { return };
    let utc = "1940-11-27T15:12:00Z";
    let cal = calendar(&kernels, utc);
    let birth = at(utc);
    assert!(bazi(&cal, birth, -122.42, -480.0, &BaziOptions::default())
        .unwrap()
        .luck
        .is_none());

    // A 庚 (yang) year: forward for a man, from the month pillar 丁亥.
    let man = BaziOptions {
        sex: Some("male"),
        ..BaziOptions::default()
    };
    let b = bazi(&cal, birth, -122.42, -480.0, &man).unwrap();
    let luck = b.luck.unwrap();
    assert_eq!(luck.direction, "forward");
    let to_next = at(&b.next_term.utc).seconds_since(&birth) / 86_400.0;
    assert!((luck.start_age - to_next / 3.0).abs() < 1e-9);
    let names: Vec<String> = luck
        .pillars
        .iter()
        .take(3)
        .map(|p| format!("{} {}", p.pillar.stem, p.pillar.branch))
        .collect();
    assert_eq!(names, ["wu zi", "ji chou", "geng yin"]);
    assert_eq!(luck.pillars.len(), 10);
    assert_eq!(luck.pillars[0].end, luck.pillars[1].start);

    let woman = BaziOptions {
        sex: Some("female"),
        ..BaziOptions::default()
    };
    let luck = bazi(&cal, birth, -122.42, -480.0, &woman)
        .unwrap()
        .luck
        .unwrap();
    assert_eq!(luck.direction, "backward");
    assert_eq!(
        (luck.pillars[0].pillar.stem, luck.pillars[0].pillar.branch),
        ("bing", "xu")
    );
}

#[test]
fn ten_gods_hidden_stems_and_balance() {
    let Some(kernels) = kernels() else { return };
    let utc = "1940-11-27T15:12:00Z";
    let cal = calendar(&kernels, utc);
    let b = bazi(&cal, at(utc), -122.42, -480.0, &BaziOptions::default()).unwrap();
    // Day Master 甲 wood.
    assert_eq!((b.day_master, b.day_master_element), ("jia", "wood"));
    assert_eq!(b.day.ten_god, None);
    assert_eq!(b.year.ten_god, Some("seven_killings")); // 庚 yang metal
    assert_eq!(b.month.ten_god, Some("hurting_officer")); // 丁 yin fire
    assert_eq!(b.hour.ten_god, Some("indirect_wealth")); // 戊 yang earth
    let hidden: Vec<_> = b.month.hidden_stems.iter().map(|h| h.stem).collect();
    assert_eq!(hidden, ["ren", "jia"]); // 亥
    assert_eq!(b.month.hidden_stems[0].ten_god, "indirect_resource");
    assert_eq!(b.year.nayin, "wax_metal"); // 庚辰 白蠟金
    assert_eq!(b.day.life_stage, "nurture"); // 甲 in 戌 (養)
    assert_eq!(b.year.life_stage, "decline"); // 甲 in 辰 (衰)
    let e = &b.elements;
    let total = e.wood + e.fire + e.earth + e.metal + e.water;
    assert!((total - 8.0).abs() < 1e-9, "{e:?}");
}

#[test]
fn lunar_date_and_popular_animal() {
    let Some(kernels) = kernels() else { return };
    // 2020-01-30, after the New Year (Rat) but before 立春 (still a Pig year in Ba Zi).
    let utc = "2020-01-30T04:00:00Z";
    let cal = calendar(&kernels, utc);
    let b = bazi(&cal, at(utc), 116.4, 480.0, &BaziOptions::default()).unwrap();
    assert_eq!(b.year.animal, "pig");
    let lunar = b.lunar_date.unwrap();
    assert_eq!(
        (lunar.year, lunar.month, lunar.day, lunar.leap, lunar.animal),
        (2020, 1, 6, false, "rat")
    );
}

#[test]
fn bad_options_are_rejected() {
    let Some(kernels) = kernels() else { return };
    let utc = "2020-01-30T04:00:00Z";
    let cal = calendar(&kernels, utc);
    for options in [
        BaziOptions {
            zi_hour: "noon",
            ..BaziOptions::default()
        },
        BaziOptions {
            sex: Some("other"),
            ..BaziOptions::default()
        },
    ] {
        assert!(bazi(&cal, at(utc), 116.4, 480.0, &options).is_err());
    }
    // A calendar cast for another year does not cover the birth.
    assert!(bazi(
        &cal,
        at("1990-01-01T00:00:00Z"),
        116.4,
        480.0,
        &BaziOptions::default()
    )
    .is_err());
}

fn zi_wei_chart(
    kernels: &KernelSet,
    utc: &str,
    longitude: f64,
    offset: f64,
    options: &ZiWeiOptions,
) -> ZiWei {
    let cal = calendar(kernels, utc);
    zi_wei(&cal, at(utc), longitude, offset, options).unwrap()
}

fn palace_at<'a>(chart: &'a ZiWei, branch: &str) -> &'a ZiWeiPalace {
    chart.palaces.iter().find(|p| p.branch == branch).unwrap()
}

#[test]
fn zi_wei_chart_matches_iztro() {
    let Some(kernels) = kernels() else { return };
    // Bruce Lee: lunar 1940 (庚辰) month 10 day 28, 辰 hour. The values below are those
    // the iztro library (MIT) gives for the same birth.
    let options = ZiWeiOptions {
        solar_time: false,
        sex: Some("male"),
        ..ZiWeiOptions::default()
    };
    let chart = zi_wei_chart(&kernels, "1940-11-27T15:12:00Z", -122.42, -480.0, &options);
    assert_eq!(
        (chart.month, chart.lunar_date.day, chart.hour_branch),
        (10, 28, "chen")
    );
    assert_eq!((chart.life_palace, chart.body_palace), ("wei", "mao"));
    assert_eq!((chart.bureau.element, chart.bureau.number), ("wood", 3));
    assert_eq!(
        (chart.life_master, chart.body_master),
        ("wu_qu", "wen_chang")
    );
    assert_eq!(chart.palaces[0].name, "life");
    assert_eq!(chart.palaces[0].branch, "wei");
    let yin = palace_at(&chart, "yin");
    assert_eq!((yin.name, yin.stem), ("health", "wu"));
    let stars: Vec<_> = yin.stars.iter().map(|s| (s.star, s.brightness)).collect();
    assert_eq!(stars, [("tian_ma", None), ("ling_xing", Some("miao"))]);
    let decade = yin.decade.as_ref().unwrap();
    assert_eq!((decade.start, decade.end), (73, 82));
    assert_eq!(
        yin.small_limit_ages,
        [5, 17, 29, 41, 53, 65, 77, 89, 101, 113]
    );
    assert!(palace_at(&chart, "mao").is_body);
    assert_eq!(chart.decade_direction, Some("forward"));
    // 庚 year: 太陽 祿, 武曲 權, 太陰 科, 天同 忌.
    let kinds: Vec<_> = chart
        .transformations
        .iter()
        .map(|t| (t.kind, t.star))
        .collect();
    assert_eq!(
        kinds,
        [
            ("lu", "tai_yang"),
            ("quan", "wu_qu"),
            ("ke", "tai_yin"),
            ("ji", "tian_tong")
        ]
    );
}

#[test]
fn zi_wei_charts_are_well_formed() {
    let Some(kernels) = kernels() else { return };
    let options = ZiWeiOptions {
        sex: Some("female"),
        ..ZiWeiOptions::default()
    };
    // A birth every 37 days and 5 hours over thirty years.
    let mut moment = at("1970-01-03T01:30:00Z");
    for _ in 0..300 {
        let utc = moment.isoformat();
        let chart = zi_wei_chart(&kernels, &utc, 12.5, 60.0, &options);
        let where_is = |star: &str| {
            let palaces: Vec<_> = chart
                .palaces
                .iter()
                .filter(|p| p.stars.iter().any(|s| s.star == star))
                .collect();
            assert_eq!(palaces.len(), 1, "{star} at {utc}");
            branch_index(palaces[0].branch)
        };
        let all: usize = chart.palaces.iter().map(|p| p.stars.len()).sum();
        assert_eq!(all, 28, "{utc}");
        let minor: usize = chart.palaces.iter().map(|p| p.minor_stars.len()).sum();
        assert_eq!(minor, 38, "{utc}");
        assert!(chart.palaces.iter().all(|p| p.flying.len() == 4));
        for gods in [
            chart
                .palaces
                .iter()
                .map(|p| p.gods.suiqian)
                .collect::<Vec<_>>(),
            chart.palaces.iter().map(|p| p.gods.jiangqian).collect(),
            chart
                .palaces
                .iter()
                .map(|p| p.gods.boshi.unwrap())
                .collect(),
            chart
                .palaces
                .iter()
                .map(|p| p.gods.life_stage.unwrap())
                .collect(),
        ] {
            let mut unique = gods.clone();
            unique.sort_unstable();
            unique.dedup();
            assert_eq!(unique.len(), 12, "{gods:?}");
        }
        let majors: usize = chart
            .palaces
            .iter()
            .map(|p| p.stars.iter().filter(|s| s.kind == "major").count())
            .sum();
        assert_eq!(majors, 14);
        // Tian Fu mirrors Zi Wei across 寅-申; Qi Sha faces Tian Fu; the lambs flank Lu Cun.
        assert_eq!((where_is("zi_wei") + where_is("tian_fu")) % 12, 4, "{utc}");
        assert_eq!((where_is("tian_fu") + 6) % 12, where_is("qi_sha"));
        assert_eq!((where_is("lu_cun") + 1) % 12, where_is("qing_yang"));
        assert_eq!((where_is("lu_cun") + 11) % 12, where_is("tuo_luo"));
        let mut branches: Vec<_> = chart.palaces.iter().map(|p| p.branch).collect();
        branches.sort_unstable();
        branches.dedup();
        assert_eq!(branches.len(), 12);
        assert_eq!(chart.transformations.len(), 4);
        let mut ages: Vec<u8> = chart
            .palaces
            .iter()
            .flat_map(|p| p.small_limit_ages.clone())
            .collect();
        ages.sort_unstable();
        assert_eq!(ages, (1..=120).collect::<Vec<u8>>());
        let mut starts: Vec<u8> = chart
            .palaces
            .iter()
            .map(|p| p.decade.as_ref().unwrap().start)
            .collect();
        starts.sort_unstable();
        let first = chart.bureau.number;
        assert_eq!(starts, (0..12).map(|k| first + 10 * k).collect::<Vec<u8>>());
        moment = moment.plus_days(37.0 + 5.0 / 24.0);
    }
}

fn branch_index(code: &str) -> usize {
    [
        "zi", "chou", "yin", "mao", "chen", "si", "wu", "wei", "shen", "you", "xu", "hai",
    ]
    .iter()
    .position(|b| *b == code)
    .unwrap()
}

#[test]
fn zi_wei_leap_months_and_options() {
    let Some(kernels) = kernels() else { return };
    // 2023-04-10 is the 20th of the leap second month.
    let utc = "2023-04-10T04:00:00Z";
    let split = zi_wei_chart(&kernels, utc, 116.4, 480.0, &ZiWeiOptions::default());
    assert!(split.lunar_date.leap);
    assert_eq!(
        (split.lunar_date.month, split.lunar_date.day, split.month),
        (2, 20, 3)
    );
    let same = ZiWeiOptions {
        leap_month: "same",
        ..ZiWeiOptions::default()
    };
    assert_eq!(zi_wei_chart(&kernels, utc, 116.4, 480.0, &same).month, 2);
    // Without the sex there are no limits.
    assert!(split.decade_direction.is_none());
    assert!(split
        .palaces
        .iter()
        .all(|p| p.decade.is_none() && p.small_limit_ages.is_empty()));
    let cal = calendar(&kernels, utc);
    for bad in [
        ZiWeiOptions {
            leap_month: "never",
            ..ZiWeiOptions::default()
        },
        ZiWeiOptions {
            sex: Some("x"),
            ..ZiWeiOptions::default()
        },
    ] {
        assert!(zi_wei(&cal, at(utc), 116.4, 480.0, &bad).is_err());
    }
}

#[test]
fn zi_wei_minor_stars_and_gods_match_iztro() {
    let Some(kernels) = kernels() else { return };
    let options = ZiWeiOptions {
        solar_time: false,
        sex: Some("male"),
        ..ZiWeiOptions::default()
    };
    let chart = zi_wei_chart(&kernels, "1940-11-27T15:12:00Z", -122.42, -480.0, &options);
    // iztro: 寅 holds 天廚, 天哭 and 天使; 臨官, 飛廉, 歲驛 and 弔客.
    let yin = palace_at(&chart, "yin");
    assert_eq!(yin.minor_stars, ["tian_chu", "tian_ku", "tian_shi"]);
    assert_eq!(yin.gods.life_stage, Some("coming_of_age"));
    assert_eq!(yin.gods.boshi, Some("fei_lian"));
    assert_eq!(yin.gods.jiangqian, "sui_yi");
    assert_eq!(yin.gods.suiqian, "diao_ke");
    // The life palace's stem 癸 flies 破軍 祿, 巨門 權, 太陰 科, 貪狼 忌.
    let flying: Vec<_> = chart.palaces[0].flying.iter().map(|t| t.star).collect();
    assert_eq!(flying, ["po_jun", "ju_men", "tai_yin", "tan_lang"]);
    assert!(chart.horoscope.is_none());
}

#[test]
fn zi_wei_horoscope() {
    let Some(kernels) = kernels() else { return };
    let birth = "1940-11-27T15:12:00Z";
    let options = |year| ZiWeiOptions {
        solar_time: false,
        sex: Some("male"),
        year: Some(year),
        ..ZiWeiOptions::default()
    };
    let chart = zi_wei_chart(&kernels, birth, -122.42, -480.0, &options(2026));
    let h = chart.horoscope.as_ref().unwrap();
    assert_eq!((h.year, h.nominal_age, h.childhood), (2026, 87, false));
    // The decade palace is the one whose limit holds 87.
    let decade = h.decade.as_ref().unwrap();
    let ruling = palace_at(&chart, decade.branch).decade.as_ref().unwrap();
    assert!(ruling.start <= 87 && 87 <= ruling.end);
    assert_eq!(decade.palaces[0].branch, decade.branch);
    assert_eq!(decade.palaces[0].god, "life");
    // 2026 is 丙午: the year's life palace is 午, its transformations those of 丙.
    assert_eq!((h.yearly.branch, h.yearly.stem), ("wu", "bing"));
    let stars: Vec<_> = h.yearly.transformations.iter().map(|t| t.star).collect();
    assert_eq!(stars, ["tian_tong", "tian_ji", "wen_chang", "lian_zhen"]);
    assert_eq!(h.yearly.stars.len(), 10);
    assert_eq!(h.suiqian[0].branch, "wu");
    assert_eq!(h.jiangqian[0].branch, "wu"); // 寅午戌: the general star in 午
    let small = h.small_limit.unwrap();
    assert!(palace_at(&chart, small).small_limit_ages.contains(&87));

    // Before the wood bureau's first decade (age 3): the childhood limit.
    let young = zi_wei_chart(&kernels, birth, -122.42, -480.0, &options(1941));
    let h = young.horoscope.unwrap();
    assert_eq!((h.nominal_age, h.childhood), (2, true));
    let wealth = young.palaces.iter().find(|p| p.name == "wealth").unwrap();
    assert_eq!(h.decade.unwrap().branch, wealth.branch);
}

#[test]
fn bazi_annual_pillar() {
    let Some(kernels) = kernels() else { return };
    let utc = "1940-11-27T15:12:00Z";
    let cal = calendar(&kernels, utc);
    let options = BaziOptions {
        year: Some(2026),
        ..BaziOptions::default()
    };
    let b = bazi(&cal, at(utc), -122.42, -480.0, &options).unwrap();
    // 2026 is 丙午; 丙 is the Eating God of a 甲 Day Master.
    let annual = b.annual.unwrap();
    assert_eq!((annual.stem, annual.branch), ("bing", "wu"));
    assert_eq!(annual.ten_god, Some("eating_god"));
    assert!(
        bazi(&cal, at(utc), -122.42, -480.0, &BaziOptions::default())
            .unwrap()
            .annual
            .is_none()
    );
}

#[test]
fn bazi_flow_pillars() {
    let Some(kernels) = kernels() else { return };
    let utc = "1940-11-27T15:12:00Z";
    let cal = calendar(&kernels, utc);
    let read = |date| {
        let options = BaziOptions {
            date: Some(date),
            ..BaziOptions::default()
        };
        bazi(&cal, at(utc), -122.42, -480.0, &options)
            .unwrap()
            .flow
            .unwrap()
    };
    let name = |p: &khagole_engine::Pillar| format!("{} {}", p.stem, p.branch);
    // 2026-03-15: a 丙午 year, after 驚蟄 (5 March): the 辛卯 month.
    let flow = read("2026-03-15");
    assert_eq!(name(&flow.year), "bing wu");
    assert_eq!(name(&flow.month), "xin mao");
    assert_eq!(flow.month_term.name, "jingzhe");
    assert_eq!(flow.month.ten_god, Some("direct_officer")); // 辛 to a 甲 Day Master
                                                            // The day pillar counts on from 2000-01-01 (戊午).
    assert_eq!(name(&read("2000-01-01").day), "wu wu");
    assert_eq!(name(&read("1949-10-01").day), "jia zi");
    // Before 立春 the year is still the old one; early January is the 子 month of 大雪.
    let january = read("2026-01-02");
    assert_eq!(name(&january.year), "yi si");
    assert_eq!(january.month.branch, "zi");
    assert_eq!(read("2026-01-10").month.branch, "chou");
    // Outside the table, or not a date.
    for bad in ["1899-06-01", "15/03/2026"] {
        let options = BaziOptions {
            date: Some(bad),
            ..BaziOptions::default()
        };
        assert!(
            bazi(&cal, at(utc), -122.42, -480.0, &options).is_err(),
            "{bad}"
        );
    }
}
