//! Comprehensive integration tests for Vedic Tier B calculations.

use std::path::Path;

use astroceleste_engine::ephemeris::{Kernel, KernelSet, Spk};
use astroceleste_engine::{
    calc_divisional_chart, calc_divisional_point, calc_panchang, calc_vimshottari_dasha,
    calculate_chart, ChartRequest, UtcInstant, YearType,
};

fn full_kernel() -> Option<KernelSet> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../kernels/de440s.bsp");
    if !path.exists() {
        return None;
    }
    let mut set = KernelSet::new();
    set.push(Kernel::new("de440s.bsp", Spk::open(path).unwrap()).unwrap());
    Some(set)
}

#[test]
fn test_modi_chart_divisional_d9_and_dasha() {
    let Some(kernels) = full_kernel() else {
        eprintln!("skipping test: de440s.bsp not found");
        return;
    };

    // Narendra Modi: 1950-09-17 04:04:00 UTC (09:34 IST), Vadnagar (23.783333 N, 72.633333 E)
    let instant = UtcInstant::parse("1950-09-17T04:04:00Z").unwrap();
    let mut req = ChartRequest::new(instant, 23.783333, 72.633333);
    req.zodiac_type = "sidereal";
    req.ayanamsa = "lahiri";
    req.house_system = "S";
    req.position_mode = "true";
    req.node_type = "true";

    let chart = calculate_chart(&kernels, &req).unwrap();
    let ascendant = chart.planets.iter().find(|p| p.name == "Ascendant").unwrap();
    let lagna_lon = ascendant.ecliptic_longitude;

    // Collect planet pairs (name, lon)
    let planets: Vec<(String, f64)> = chart
        .planets
        .iter()
        .map(|p| (p.name.to_string(), p.ecliptic_longitude))
        .collect();

    // 1. Test D9 (Navamsha) Chart
    let d9 = calc_divisional_chart(&planets, lagna_lon, "D9").unwrap();
    assert_eq!(d9.varga, "D9");
    assert_eq!(d9.planets.len(), chart.planets.len());

    // Lagna: Libra (sign 7, ~12.29°). Air sign seed = Libra (7, index 6).
    // Part = floor(12.29 * 9 / 30) = 3. (6 + 3) % 12 + 1 = 10 (Capricorn / Makara).
    assert_eq!(d9.lagna.sign, 10, "Modi D9 Lagna is Capricorn (10)");

    // Moon: Scorpio (sign 8, ~9.38°). Water sign seed = Cancer (4, index 3).
    // Part = floor(9.38 * 9 / 30) = 2. (3 + 2) % 12 + 1 = 6 (Virgo / Kanya).
    let d9_moon = d9.planets.iter().find(|p| p.name == "Moon").unwrap();
    assert_eq!(d9_moon.sign, 6, "Modi D9 Moon is Virgo (6)");
    // Since Lagna is Capricorn (10) and Moon is Virgo (6), Moon is in House 9 of D9
    assert_eq!(d9_moon.house, 9);

    // Sun: Virgo (sign 6, ~0.54°). Earth sign seed = Capricorn (10, index 9).
    // Part = floor(0.54 * 9 / 30) = 0. (9 + 0) % 12 + 1 = 10 (Capricorn / Makara).
    let d9_sun = d9.planets.iter().find(|p| p.name == "Sun").unwrap();
    assert_eq!(d9_sun.sign, 10, "Modi D9 Sun is Capricorn (10)");
    assert_eq!(d9_sun.house, 1);

    // 2. Test Vimshottari Dasha (Default YearType is TSSY)
    let moon = chart.planets.iter().find(|p| p.name == "Moon").unwrap();
    let dasha = calc_vimshottari_dasha(
        moon.ecliptic_longitude,
        instant.julian_day(),
        3,
        YearType::default(),
    );

    assert_eq!(dasha.len(), 9);
    // Anuradha nakshatra -> Saturn Mahadasha at birth
    assert_eq!(dasha[0].planet, "Saturn");
    assert_eq!(dasha[1].planet, "Mercury");
    assert_eq!(dasha[2].planet, "Ketu");
    assert_eq!(dasha[3].planet, "Venus");
    assert_eq!(dasha[4].planet, "Sun");
    assert_eq!(dasha[5].planet, "Moon");
    assert_eq!(dasha[6].planet, "Mars");
    assert_eq!(dasha[7].planet, "Rahu");
    assert_eq!(dasha[8].planet, "Jupiter");

    // Exact TSSY dates matching JHora ground truth
    assert_eq!(dasha[1].start_date, "1963-02-15");
    assert_eq!(dasha[2].start_date, "1980-02-15");
    assert_eq!(dasha[3].start_date, "1987-02-15");
    assert_eq!(dasha[4].start_date, "2007-02-15");
    assert_eq!(dasha[6].start_date, "2023-02-15");
    assert_eq!(dasha[7].start_date, "2030-02-15");

    // Sub-periods: Saturn Mahadasha has 9 Antardashas
    assert_eq!(dasha[0].children.len(), 9);
    // Antardashas start with Saturn, then Mercury, etc.
    assert_eq!(dasha[0].children[0].planet, "Saturn");
    assert_eq!(dasha[0].children[1].planet, "Mercury");

    // 3. Test Panchanga on birth date
    let sun = chart.planets.iter().find(|p| p.name == "Sun").unwrap();
    let panchang = calc_panchang(
        sun.ecliptic_longitude,
        moon.ecliptic_longitude,
        instant.julian_day(),
        Some(0), // Sunday in local IST
        Some(false),
    );

    // Tithi: Shukla Shashthi (6)
    assert_eq!(panchang.tithi.number, 6);
    assert_eq!(panchang.tithi.paksha, "Shukla");
    assert_eq!(panchang.vara.name, "Sunday");
    assert_eq!(panchang.vara.lord, "Sun");
    assert_eq!(panchang.nakshatra.name, "Anuradha");
    assert_eq!(panchang.nakshatra.lord, "Saturn");
    assert_eq!(panchang.masa.amanta, "Bhadrapada");
    assert_eq!(panchang.masa.paksha, "Shukla");
    assert_eq!(panchang.samvat.vikram_year, 2007);
}

#[test]
fn test_all_vargas_coverage() {
    let vargas = [
        "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9", "D10", "D11", "D12", "D16", "D20",
        "D24", "D27", "D30", "D40", "D45", "D60",
    ];

    for varga in &vargas {
        for lon in [0.0, 15.0, 29.9, 45.0, 90.0, 180.0, 270.0, 359.9] {
            let p = calc_divisional_point(lon, varga).unwrap();
            assert!(
                p.sign >= 1 && p.sign <= 12,
                "Sign out of range for varga {varga} at lon {lon}: {}",
                p.sign
            );
            assert!(
                p.degree >= 0.0 && p.degree <= 30.0,
                "Degree out of range for varga {varga} at lon {lon}: {}",
                p.degree
            );
            assert!(
                p.longitude >= 0.0 && p.longitude < 360.0,
                "Longitude out of range for varga {varga} at lon {lon}: {}",
                p.longitude
            );
        }
    }
}
