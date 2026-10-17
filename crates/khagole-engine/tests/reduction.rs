//! Stage-by-stage comparison with Skyfield 1.55 (scripts/make_reduction_fixtures.py).
//! Needs the full de440s kernel (scripts/fetch-kernels.sh); skipped without it.

use std::path::PathBuf;

use khagole_engine::ephemeris::observe::{apparent, ecliptic_latlon, observe};
use khagole_engine::ephemeris::{Kernel, Spk};
use khagole_engine::frames::Orientation;
use khagole_engine::time::{delta_t, Time};
use serde_json::Value;

const BODIES: [(&str, i32); 10] = [
    ("Sun", 10),
    ("Moon", 301),
    ("Mercury", 199),
    ("Venus", 299),
    ("Mars", 4),
    ("Jupiter", 5),
    ("Saturn", 6),
    ("Uranus", 7),
    ("Neptune", 8),
    ("Pluto", 9),
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture() -> Value {
    let text = std::fs::read_to_string(root().join("tests/fixtures/reduction.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn max_matrix_diff(a: &[[f64; 3]; 3], b: &Value) -> f64 {
    let b = b.as_array().unwrap();
    (0..9)
        .map(|i| (a[i / 3][i % 3] - f(&b[i])).abs())
        .fold(0.0, f64::max)
}

/// Worst-case error of each stage; asserted below and printed with --nocapture.
#[derive(Default, Debug)]
struct Worst {
    delta_t_s: f64,
    tdb_fraction_s: f64,
    gmst_s: f64,
    gast_s: f64,
    nutation_rad: f64,
    matrices: f64,
    astrometric_rel: f64,
    apparent_rel: f64,
    lon_arcsec: f64,
    lat_arcsec: f64,
    lon_plus_arcsec: f64,
}

fn angle_diff_deg(a: f64, b: f64) -> f64 {
    ((a - b + 180.0).rem_euclid(360.0) - 180.0).abs()
}

fn rel_diff(a: &[f64; 3], b: &Value) -> f64 {
    let b: Vec<f64> = b.as_array().unwrap().iter().map(f).collect();
    let norm = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    d / norm
}

#[test]
fn delta_t_matches_skyfield() {
    let dt = delta_t();
    let mut worst: f64 = 0.0;
    for row in fixture()["delta_t"].as_array().unwrap() {
        worst = worst.max((dt.at(f(&row["tt"])) - f(&row["delta_t"])).abs());
    }
    println!("ΔT worst: {worst:e} s");
    assert!(worst < 1e-9, "ΔT differs by {worst} s");
}

#[test]
fn reduction_matches_skyfield() {
    let path = root().join("kernels/de440s.bsp");
    if !path.exists() {
        assert!(
            std::env::var_os("ASTROCELESTE_REQUIRE_KERNEL").is_none(),
            "{} not found and ASTROCELESTE_REQUIRE_KERNEL is set",
            path.display()
        );
        eprintln!("skipping: {} not found", path.display());
        return;
    }
    let kernel = Kernel::new("de440s.bsp", Spk::open(path).unwrap()).unwrap();
    let mut w = Worst::default();
    for inst in fixture()["instants"].as_array().unwrap() {
        let t = Time::from_ut1(f(&inst["jd_ut1"]));
        w.delta_t_s = w
            .delta_t_s
            .max((t.tt_fraction - f(&inst["tt_fraction"])).abs() * 86400.0);
        w.tdb_fraction_s = w
            .tdb_fraction_s
            .max((t.tdb_fraction - f(&inst["tdb_fraction"])).abs() * 86400.0);
        let o = Orientation::at(&t);
        w.gmst_s = w
            .gmst_s
            .max((o.gmst_hours - f(&inst["gmst_hours"])).abs() * 3600.0);
        w.gast_s = w
            .gast_s
            .max((o.gast_hours - f(&inst["gast_hours"])).abs() * 3600.0);
        w.nutation_rad = w
            .nutation_rad
            .max((o.d_psi - f(&inst["d_psi_rad"])).abs())
            .max((o.d_eps - f(&inst["d_eps_rad"])).abs());
        for (m, key) in [
            (&o.precession, "precession"),
            (&o.nutation, "nutation"),
            (&o.m, "m"),
            (&o.ecliptic, "ecliptic"),
        ] {
            w.matrices = w.matrices.max(max_matrix_diff(m, &inst[key]));
        }

        let t_plus = Time::from_ut1(t.ut1() + 1.0 / 24.0);
        let o_plus = Orientation::at(&t_plus);
        for (name, code) in BODIES {
            let want = &inst["bodies"][name];
            let astrometric = observe(&kernel, code, &t).unwrap();
            w.astrometric_rel = w
                .astrometric_rel
                .max(rel_diff(&astrometric.position, &want["astrometric_au"]));
            let app = apparent(&kernel, &astrometric, &t).unwrap();
            w.apparent_rel = w.apparent_rel.max(rel_diff(&app, &want["apparent_au"]));
            let (lat, lon, _) = ecliptic_latlon(&o, &app);
            w.lon_arcsec = w
                .lon_arcsec
                .max(angle_diff_deg(lon, f(&want["lon_deg"])) * 3600.0);
            w.lat_arcsec = w.lat_arcsec.max((lat - f(&want["lat_deg"])).abs() * 3600.0);

            let a_plus = observe(&kernel, code, &t_plus).unwrap();
            let app_plus = apparent(&kernel, &a_plus, &t_plus).unwrap();
            let (_, lon_plus, _) = ecliptic_latlon(&o_plus, &app_plus);
            w.lon_plus_arcsec = w
                .lon_plus_arcsec
                .max(angle_diff_deg(lon_plus, f(&want["lon_plus_1h_deg"])) * 3600.0);
        }
    }
    println!("{w:#?}");
    // Observed: time scales and sidereal time bit-identical, matrices within 1 ulp,
    // longitudes within 1e-7". The bounds leave room for platform libm differences.
    assert!(w.delta_t_s < 1e-9, "{w:?}");
    assert!(w.tdb_fraction_s < 1e-9, "{w:?}");
    assert!(w.gmst_s < 1e-9 && w.gast_s < 1e-9, "{w:?}");
    assert!(w.nutation_rad < 1e-15, "{w:?}");
    assert!(w.matrices < 1e-14, "{w:?}");
    assert!(w.astrometric_rel < 1e-11 && w.apparent_rel < 1e-11, "{w:?}");
    assert!(w.lon_arcsec < 1e-5 && w.lat_arcsec < 1e-5, "{w:?}");
    assert!(w.lon_plus_arcsec < 1e-5, "{w:?}");
}
