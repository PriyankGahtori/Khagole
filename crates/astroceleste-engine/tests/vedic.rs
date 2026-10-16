//! Tests for Vedic astrology extensions: Sripati houses and Geometric (True) position mode.

use std::path::Path;

use astroceleste_engine::ephemeris::{Kernel, KernelSet, Spk};
use astroceleste_engine::{calculate_chart, ChartRequest, UtcInstant};

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
fn sripati_and_geometric_mode() {
    let Some(kernels) = full_kernel() else {
        eprintln!("skipping vedic test: de440s.bsp not found");
        return;
    };

    // Narendra Modi: 1950-09-17 04:04:00 UTC (09:34 IST), Vadnagar (23.783333 N, 72.633333 E)
    let instant = UtcInstant::parse("1950-09-17T04:04:00Z").unwrap();
    let mut req = ChartRequest::new(instant, 23.783333, 72.633333);
    req.zodiac_type = "sidereal";
    req.ayanamsa = "lahiri";
    req.house_system = "S"; // Sripati
    req.position_mode = "true"; // Geometric position mode

    let chart = calculate_chart(&kernels, &req).unwrap();

    // Verify Sripati house cusps
    assert_eq!(chart.houses.len(), 12);
    assert_eq!(chart.house_system, "S");
    assert_eq!(chart.position_mode, Some("true"));

    // Find Sun placement
    let sun = chart.planets.iter().find(|p| p.name == "Sun").unwrap();
    // JHora Sun: 150.53918°, SwissEph Truepos: 150.53914°
    // In geometric mode, Sun longitude must be ~150.5393° (within 1 arcsecond = ~0.0003°)
    let diff_arcsec = (sun.ecliptic_longitude - 150.53918).abs() * 3600.0;
    assert!(
        diff_arcsec < 1.0,
        "Sun geometric longitude is off from JHora: diff is {diff_arcsec}\" (lon = {})",
        sun.ecliptic_longitude
    );

    // Find Mars placement
    let mars = chart.planets.iter().find(|p| p.name == "Mars").unwrap();
    // JHora Mars: 210.89571°
    let mars_diff = (mars.ecliptic_longitude - 210.89571).abs() * 3600.0;
    assert!(
        mars_diff < 1.0,
        "Mars geometric longitude is off from JHora: diff is {mars_diff}\" (lon = {})",
        mars.ecliptic_longitude
    );

    // Find Venus placement
    let venus = chart.planets.iter().find(|p| p.name == "Venus").unwrap();
    // JHora Venus: 135.62509°
    let venus_diff = (venus.ecliptic_longitude - 135.62509).abs() * 3600.0;
    assert!(
        venus_diff < 1.5,
        "Venus geometric longitude is off from JHora: diff is {venus_diff}\" (lon = {})",
        venus.ecliptic_longitude
    );
}
