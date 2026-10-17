//! Shared helpers for the golden-fixture tests.
#![allow(dead_code)]

use std::path::PathBuf;

use khagole_engine::ephemeris::{Kernel, KernelSet, Spk};
use khagole_engine::UtcInstant;
use serde_json::Value;

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The full de440s kernel, or `None` (test skipped) when it has not been fetched. With
/// `ASTROCELESTE_REQUIRE_KERNEL` set (as in CI), a missing kernel fails instead.
pub fn kernels() -> Option<KernelSet> {
    let path = root().join("kernels/de440s.bsp");
    if !path.exists() {
        assert!(
            std::env::var_os("ASTROCELESTE_REQUIRE_KERNEL").is_none(),
            "{} not found and ASTROCELESTE_REQUIRE_KERNEL is set",
            path.display()
        );
        eprintln!(
            "skipping: {} not found (scripts/fetch-kernels.sh)",
            path.display()
        );
        return None;
    }
    let mut set = KernelSet::new();
    set.push(Kernel::new("de440s.bsp", Spk::open(path).unwrap()).unwrap());
    Some(set)
}

pub fn fixture(name: &str) -> Vec<Value> {
    let text = std::fs::read_to_string(root().join("tests/fixtures").join(name)).unwrap();
    serde_json::from_str(&text).unwrap()
}

pub fn instant(v: &Value) -> UtcInstant {
    UtcInstant::parse(v.as_str().unwrap()).unwrap()
}

/// Keys the fixtures leave out (degree symbolism is private data of the application).
const PRIVATE_KEYS: [&str; 2] = ["degree_symbol", "degree_symbols"];

/// Chart keys added after the reference implementation, absent from the fixtures and
/// tested on their own (`tests/dignities.rs`).
const ADDED_CHART_KEYS: [&str; 5] = [
    "sect",
    "dignity_scheme",
    "receptions",
    "antiscia",
    "planetary_hours",
];

/// Remove what the fixtures leave out: private keys anywhere, the added chart keys at the
/// top level and each placement's `condition`.
/// Horary considerations added after the reference implementation.
const ADDED_STRICTURES: [&str; 3] = [
    "MOON_VIA_COMBUSTA",
    "MOON_LATE_DEGREES",
    "ASC_RULER_COMBUST",
];

pub fn strip_private(value: &mut Value) {
    if let Value::Object(map) = value {
        for key in ADDED_CHART_KEYS {
            map.shift_remove(key);
        }
        if let Some(Value::Object(horary)) = map.get_mut("horary_data") {
            horary.shift_remove("judgment");
            if let Some(Value::Array(strictures)) = horary.get_mut("strictures") {
                strictures
                    .retain(|s| !ADDED_STRICTURES.contains(&s["code"].as_str().unwrap_or("")));
            }
        }
    }
    strip_nested(value);
}

fn strip_nested(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for key in PRIVATE_KEYS {
                map.shift_remove(key);
            }
            map.shift_remove("condition");
            map.values_mut().for_each(strip_nested);
        }
        Value::Array(items) => items.iter_mut().for_each(strip_nested),
        _ => {}
    }
}

/// Structural diff: same key order, same number types (int vs float), floats within
/// `tol` (relative to magnitude, at least absolute `tol`), everything else identical.
pub fn diff(actual: &Value, expected: &Value, path: &str, tol: f64, out: &mut Vec<String>) {
    match (actual, expected) {
        (Value::Object(a), Value::Object(e)) => {
            let ak: Vec<&String> = a.keys().collect();
            let ek: Vec<&String> = e.keys().collect();
            if ak != ek {
                out.push(format!("{path}: keys {ak:?} != {ek:?}"));
            }
            for (k, ev) in e {
                if let Some(av) = a.get(k) {
                    diff(av, ev, &format!("{path}.{k}"), tol, out);
                }
            }
        }
        (Value::Array(a), Value::Array(e)) => {
            if a.len() != e.len() {
                out.push(format!("{path}: {} items != {}", a.len(), e.len()));
            }
            for (i, (av, ev)) in a.iter().zip(e).enumerate() {
                diff(av, ev, &format!("{path}[{i}]"), tol, out);
            }
        }
        (Value::Number(a), Value::Number(e)) => {
            if a.is_f64() != e.is_f64() {
                out.push(format!("{path}: number type {a} != {e}"));
            } else if a.is_f64() {
                let (x, y) = (a.as_f64().unwrap(), e.as_f64().unwrap());
                if (x - y).abs() > tol * y.abs().max(1.0) {
                    out.push(format!("{path}: {x} != {y}"));
                }
            } else if a != e {
                out.push(format!("{path}: {a} != {e}"));
            }
        }
        (a, e) => {
            if a != e {
                out.push(format!("{path}: {a} != {e}"));
            }
        }
    }
}

/// Report mismatches of many cases, failing with the first few.
pub fn assert_no_diffs(label: &str, cases: usize, diffs: &[String]) {
    println!("{label}: {cases} cases, {} mismatches", diffs.len());
    assert!(
        diffs.is_empty(),
        "{label}: {} mismatches, first ones:\n{}",
        diffs.len(),
        diffs
            .iter()
            .take(25)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
