# Roadmap

Where `khagole-engine` stands and what could come next. Nothing here is a commitment
or has a date; items move into issues when someone picks them up.

## Principles

These shape every item below and are not up for change without a deliberate decision:

- **JPL ephemerides only, never approximated.** Positions come from JPL SPK kernels
  (DE440s, DE441). A date no loaded kernel covers is an `ephemeris_out_of_range` error,
  never an extrapolation.
- **The reference implementation is the spec.** The golden fixtures (Khagole's
  Python on Skyfield) must be reproduced to 1e-9 with the same JSON shape. Additions to the
  reference are tested on their own and kept out of the fixture comparison.
- **One pure-Rust core, everywhere.** No C code, no platform I/O, only `serde` and
  `serde_json`; it builds for servers, `wasm32-unknown-unknown`, Android and iOS.
- **Permissive licence:** MIT OR Apache-2.0.

## Where it stands (0.0.x)

| Area | What exists |
|---|---|
| Ephemeris | SPK types 2 and 3 from a file (reads cached) or from memory; kernel sets in preference order; excerpts; bodies split over several segments |
| Reduction | Skyfield 1.55 port: ΔT, IAU 2000A nutation, light-time, deflection, aberration; checked stage by stage |
| Bodies | Sun to Pluto, mean nodes, mean Black Moon Lilith, Chiron (1849–2151 table) |
| Houses | 11 systems: Placidus, Koch, Regiomontanus, Campanus, Topocentric, Alcabitius, Morinus, Porphyry, Equal, Vehlow, Whole Sign |
| Zodiac | tropical and sidereal, 14 ayanamsas |
| Chart | 7 aspects, 61 fixed stars, 11 lots, temperament, lunar status and mansions, essential and accidental dignities (Lilly and Dorothean), sect, receptions, antiscia, planetary hours |
| Techniques | horary (strictures, judgment), transits, synastry, derived charts, elections (scoring and search), profections and firdaria, Lilly's degree qualities |
| Chinese | calendar (solar terms, lunar and leap months), Ba Zi, Zi Wei Dou Shu |
| Bindings | Python (PyO3, abi3 wheels) and WebAssembly (npm), same JSON as the Rust types |

Rough timings on de440s from disk (release build, one core): a natal chart 2.9 ms, of
which 2.2 ms is the sunrise search for the planetary hours; a horary chart 4.6 ms; a
30-day election search 105 ms (`tests/timing.rs`).

## Candidates

### Astronomy

- **True lunar node and osculating (true) Lilith**, next to the mean ones. Both follow from
  the Moon's state vector, which the kernels already give.
- **Vertex and East Point**, from the sidereal time and latitude the houses already use.
- **Asteroids** (Ceres, Pallas, Juno, Vesta) from JPL small-body SPK kernels, loaded like
  the planetary ones. Check the segment types those kernels use first; the reader handles
  types 2 and 3.
- **Topocentric positions** (the Moon's parallax) as an option.
- **Placidus at polar latitudes.** Inside the polar circles some cusps have no solution,
  and today the iteration stops on its first guess. Koch already falls back to Porphyry,
  as Swiss Ephemeris does. Decide the rule for Placidus, document it, and flag the fallback
  in the chart.

### Astrology

- **Minor aspects** (semi-square, sesquiquadrate, quintile, biquintile) behind an orb-setting
  switch, so the reference output is unchanged by default.
- **Aspect patterns**: grand trine, T-square, grand cross, yod, kite, stellium.
- **Declinations**: parallels and contraparallels.
- **Returns and progressions**: solar and lunar returns and secondary progressions. The
  Chinese calendar's Sun-longitude search already finds the moments returns need.

### Verification

- **Cross-check the seven house systems the fixtures lack** (K, R, C, T, B, M, V) against
  Swiss Ephemeris over many latitudes and dates. Today they are checked at a single place
  and date.
- **Fuzz the SPK reader** (cargo-fuzz). A sweep of hostile header values exists in
  `tests/kernels.rs`; a fuzzer would cover the rest of the byte space.

### Performance

- **The sunrise search dominates a chart.** Caching the solar day per place and date across
  calls, or evaluating the Sun's altitude more cheaply, would speed up charts and horary
  several times over. Any change must keep sunrise times bit-identical, because the
  planetary hours are in the fixtures.

### Bindings and distribution

- **TypeScript definitions** for requests and results, instead of `any`.
- **Typed Python results** (`TypedDict`s in the `.pyi` stub).
- **Native mobile bindings** (Swift and Kotlin) via UniFFI or a C ABI crate, so the apps
  need not go through WebAssembly.
- **An npm package for both Node and browsers**: today only the `--target web` build is
  published.
- **A `cargo-semver-checks` job** once the API leaves 0.0.x.

## Not planned

- **Analytical theories (VSOP87, ELP2000) as a built-in data-free tier.** They would
  approximate where the engine promises JPL positions or an error. If a data-free mode is
  ever wanted, it should be a separate, opt-in crate whose accuracy is stated in its name
  and documentation, never a silent fallback.
- **Relicensing** (for example to MPL-2.0). The crate, the wheels and the npm package are
  published as MIT OR Apache-2.0.
