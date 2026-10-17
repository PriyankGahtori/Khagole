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
    // Reference Sun: 150.53918°, SwissEph Truepos: 150.53914°
    // In geometric mode, Sun longitude must be ~150.5393° (within 1 arcsecond = ~0.0003°)
    let diff_arcsec = (sun.ecliptic_longitude - 150.53918).abs() * 3600.0;
    assert!(
        diff_arcsec < 1.0,
        "Sun geometric longitude is off from reference: diff is {diff_arcsec}\" (lon = {})",
        sun.ecliptic_longitude
    );

    // Find Mars placement
    let mars = chart.planets.iter().find(|p| p.name == "Mars").unwrap();
    // Reference Mars: 210.89571°
    let mars_diff = (mars.ecliptic_longitude - 210.89571).abs() * 3600.0;
    assert!(
        mars_diff < 1.0,
        "Mars geometric longitude is off from reference: diff is {mars_diff}\" (lon = {})",
        mars.ecliptic_longitude
    );

    // Find Venus placement
    let venus = chart.planets.iter().find(|p| p.name == "Venus").unwrap();
    // Reference Venus: 135.62509°
    let venus_diff = (venus.ecliptic_longitude - 135.62509).abs() * 3600.0;
    assert!(
        venus_diff < 1.5,
        "Venus geometric longitude is off from reference: diff is {venus_diff}\" (lon = {})",
        venus.ecliptic_longitude
    );

    // Test True Lunar Node calculation
    use astroceleste_engine::ephemeris::observe::EARTH;
    use astroceleste_engine::frames::{mxv, Orientation};
    use astroceleste_engine::time::Time;

    let t = Time::from_ut1(instant.julian_day());
    let t_plus = Time::from_ut1(instant.julian_day() + 1.0 / 24.0);
    let orient = Orientation::at(&t);
    let orient_plus = Orientation::at(&t_plus);
    let kernel = kernels.for_jd(instant.julian_day()).unwrap();

    let compute_node = |t_eval: &Time, ori: &Orientation| {
        let (pos_moon, vel_moon) = kernel.barycentric(301, t_eval).unwrap();
        let (pos_earth, vel_earth) = kernel.barycentric(EARTH, t_eval).unwrap();
        let r = [pos_moon[0] - pos_earth[0], pos_moon[1] - pos_earth[1], pos_moon[2] - pos_earth[2]];
        let v = [vel_moon[0] - vel_earth[0], vel_moon[1] - vel_earth[1], vel_moon[2] - vel_earth[2]];
        let r_ecl = mxv(&ori.ecliptic, &r);
        let v_ecl = mxv(&ori.ecliptic, &v);
        let h_x = r_ecl[1] * v_ecl[2] - r_ecl[2] * v_ecl[1];
        let h_y = r_ecl[2] * v_ecl[0] - r_ecl[0] * v_ecl[2];
        let lon = h_x.atan2(-h_y).to_degrees().rem_euclid(360.0);
        lon
    };

    let node_lon = compute_node(&t, &orient);
    let node_lon_plus = compute_node(&t_plus, &orient_plus);
    let mut d_lon = node_lon_plus - node_lon;
    if d_lon > 180.0 { d_lon -= 360.0; }
    if d_lon < -180.0 { d_lon += 360.0; }
    let speed = d_lon * 24.0;

    println!("Computed True Node (Tropical): lon = {node_lon}°, speed = {speed}°/day");
    println!("Swiss Ephemeris Tropical: lon = 358.379154°, speed = +0.005585°/day");

    let shift = 23.168635; // Lahiri
    let sid_node = (node_lon - shift).rem_euclid(360.0);
    println!("Computed True Node (Lahiri): lon = {sid_node}°");
    println!("Swiss Ephemeris Lahiri: lon = 335.210414°");

    let diff_arcsec = (node_lon - 358.379154).abs() * 3600.0;
    println!("Diff vs SwissEph True Node: {diff_arcsec}\"");
    assert!(diff_arcsec < 0.1, "True node matches Swiss Ephemeris within 0.1 arcsec");

    // Also test calculate_chart with node_type = "true"
    let mut req_true_node = req.clone();
    req_true_node.node_type = "true";
    let chart_true_node = calculate_chart(&kernels, &req_true_node).unwrap();
    let north_node = chart_true_node.planets.iter().find(|p| p.name == "North Node").unwrap();
    let south_node = chart_true_node.planets.iter().find(|p| p.name == "South Node").unwrap();

    // Swiss Ephemeris Lahiri True Node is 335.210414°
    let node_diff = (north_node.ecliptic_longitude - 335.210414).abs() * 3600.0;
    println!("North Node via calculate_chart: lon = {}°, diff vs SwissEph = {}\"", north_node.ecliptic_longitude, node_diff);
    assert!(node_diff < 0.5, "North Node matches Swiss Ephemeris within 0.5 arcsec");

    // South Node should be exactly 180° opposite
    let opposite = (north_node.ecliptic_longitude + 180.0) % 360.0;
    assert!((south_node.ecliptic_longitude - opposite).abs() < 1e-10);
    assert_eq!(chart_true_node.node_type, Some("true"));
}

