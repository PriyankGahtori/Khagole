# Accuracy

khagole-engine is a port of the Khagole reference implementation (Python on
Skyfield). The goal is not just "accurate", but **identical**: the server, the apps and the
browser must return the same chart, down to the last digit of every number and the order of
every key. Three independent references check that.

## 1. Golden charts from the reference implementation

`tests/fixtures/*.json` hold the exact output of the reference implementation:

| Fixture | Cases |
|---|---|
| `natal.json` | 161 charts: all four house systems, tropical and sidereal with all 14 ayanamsas, latitudes up to ±75°, longitudes up to the date line, 1850–2149 |
| `horary.json` | 33 horary charts |
| `transits.json`, `synastry.json`, `derived.json` | 27 each |
| `errors.json` | dates before and after the kernel (1700, 2200) |

The reference implementation only had Placidus, Whole Sign, Equal and Porphyry. The other
house systems are unit-tested against Swiss Ephemeris (`src/houses.rs`, within 1′), and the
application cross-checks them at several latitudes.

The engine must reproduce them with:

- floats within 1e-9 (relative to magnitude, absolute below 1);
- the same keys in the same order;
- the same number types: an integer in the reference is an integer here, and a float
  stays a float, even when its value is whole.

Only application-private keys (`degree_symbol`, `degree_symbols`) are left out of the
comparison. The Rust tests (`tests/golden_*.rs`), the Python binding tests and the
WebAssembly test (`tests/golden.mjs`) all check the same fixtures. These tests need the
full DE440s kernel and **skip** without it.

## 2. Kernel states from jplephem

`tests/fixtures/spk_reference.json` holds raw positions and velocities read with
[jplephem](https://github.com/brandon-rhodes/python-jplephem) from the committed one-year
excerpt `tests/data/de440s_2000.bsp`, including segment boundaries. They check the SPK
reader and Chebyshev evaluation on their own (`tests/spk_reference.rs`). Both files come
from `scripts/make_spk_fixtures.py`.

## 3. Stage by stage against Skyfield

`tests/fixtures/reduction.json` (`scripts/make_reduction_fixtures.py`) holds Skyfield 1.55
values for every stage of the reduction, so a discrepancy can be traced to the stage that
introduces it (`tests/reduction.rs`):

| Stage | Tolerance |
|---|---|
| ΔT, TDB − TT | 1e-9 s |
| sidereal time (GMST, GAST) | 1e-9 s |
| nutation angles | 1e-15 rad |
| precession / nutation matrices | 1e-14 |
| astrometric and apparent vectors (light-time, deflection, aberration) | 1e-11 relative |
| ecliptic longitude and latitude | 1e-5″ |

The time scales, frames and apparent-position code are ported from Skyfield 1.55 (MIT, see
`NOTICE`), with the same constants and the same order of floating-point operations.

## Python semantics

The reference is Python, and Python's `%`, `//` and `round()` differ from Rust's `%` and
`f64::round` at boundaries: `-0.5 % 360` is `359.5` in Python but `-0.5` in Rust, and
Python rounds halves to even (`round(0.125, 2) == 0.12`). A planet at 29°59′59.99″ must
land in the same sign, and an orb must round to the same hundredth, so the engine uses its
own implementations (`pyfloat.rs`) wherever the reference relies on them.

## Absolute accuracy

Matching the reference also means inheriting its model: apparent geocentric positions of
date from JPL DE440s/DE441, with light-time, gravitational deflection by the Sun, Jupiter
and Saturn, aberration, IAU 2000A nutation and IAU 2006 precession, the same model as
Skyfield. The mean lunar nodes and mean Lilith follow the reference's formulas. For future
dates ΔT is only predicted (see [Ephemerides](ephemerides.md#time-scales)).

## What CI runs

On every push and pull request: formatting, clippy and rustdoc with warnings denied; the
test suite on Linux, macOS and Windows (stable) and on the MSRV; the golden and full-range
tests with the full DE440s kernel; the Python wheel with pytest; the WebAssembly package
with the Node golden test; builds for `wasm32-unknown-unknown`, Android and iOS; the
packaged crate; and cargo-deny.
