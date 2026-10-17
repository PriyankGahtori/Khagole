# Vedic Astrology Tier B Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port Tier B Vedic astrological calculations (27 Nakshatras, Divisional charts D1–D60, Vimshottari Dasha, and Panchanga) natively into Rust within `khagole-engine`, exposing them to WebAssembly and the native MCP server.

**Architecture:** Implement a modular `vedic` subsystem inside `crates/khagole-engine/src/vedic/` that is pure Rust, `no_std`-ready (using standard math and serde), with zero external C dependencies. Expose these calculations as idiomatic Rust APIs, export them through `crates/khagole-engine-wasm` for web/mobile/Hora Prakash consumption, and register them as JSON-RPC tools in `crates/khagole-mcp`.

**Tech Stack:** Rust (2021 edition), `serde`, `serde_json`, `wasm-bindgen`, `wasm-pack`, JSON-RPC 2.0 stdio MCP.

**Spec:** Port of `hora-prakash/src/core/{calculations.ts, divisional.ts, dasha.ts, panchang.ts}` into `khagole-engine`.

## Global Constraints

- Pure Rust: zero C library bindings or system ephemeris dependencies needed for Tier B math.
- Complete parity with `hora-prakash` and canonical Vedic reference ephemeris for all D1–D60 algorithms, Vimshottari dasha hierarchy, and Panchanga 5 angas.
- Maintain existing codebase quality, doc comments, serialization formats, and tests.
- All WASM targets must build cleanly with `wasm-pack build --target web`.

## Review Focus

1. Boundary conditions at sign changes ($0^\circ, 30^\circ, 360^\circ$ and wrap-around).
2. Vimshottari balance of dasha when Moon is at $0.0^\circ$ vs $13.333^\circ$ (boundary of Nakshatra).
3. Unequal division handling in D30 (Trimshamsha) for odd vs even signs.
4. Karana fixed vs movable index mapping around lunation edges (positions 0, 1–56, 57–59).
5. Serialization format consistency across Rust, WASM JSON, and MCP tool responses.

---

### Task 1: Vedic Nakshatras Subsystem

**Files:**
- Create: `crates/khagole-engine/src/vedic/nakshatra.rs`
- Create: `crates/khagole-engine/src/vedic/mod.rs`
- Modify: `crates/khagole-engine/src/lib.rs`
- Test: `crates/khagole-engine/src/vedic/nakshatra.rs` (inline test module)

**Interfaces:**
- Produces:
  ```rust
  pub struct NakshatraInfo {
      pub index: usize,       // 0..26
      pub name: &'static str, // Ashwini..Revati
      pub lord: &'static str, // Ketu..Mercury
      pub pada: u8,           // 1..4
  }
  pub fn nakshatra_info(lon: f64) -> NakshatraInfo;
  ```

- [ ] **Step 1: Write the failing test for nakshatra calculation**

In `crates/khagole-engine/src/vedic/nakshatra.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nakshatras() {
        let n0 = nakshatra_info(0.0);
        assert_eq!(n0.name, "Ashwini");
        assert_eq!(n0.lord, "Ketu");
        assert_eq!(n0.pada, 1);
        assert_eq!(n0.index, 0);

        // 3°20' = 3.333333° (end of pada 1)
        let n0_p2 = nakshatra_info(3.34);
        assert_eq!(n0_p2.name, "Ashwini");
        assert_eq!(n0_p2.pada, 2);

        // Krittika begins at 26°40' = 26.666667° (index 2, lord Sun)
        let n2 = nakshatra_info(26.7);
        assert_eq!(n2.name, "Krittika");
        assert_eq!(n2.lord, "Sun");
        assert_eq!(n2.index, 2);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package khagole-engine vedic::nakshatra`
Expected: FAIL with "cannot find module or file"

- [ ] **Step 3: Implement `nakshatra.rs` and connect to `vedic/mod.rs` & `lib.rs`**

Implement:
- 27 Nakshatras: Ashwini, Bharani, Krittika, Rohini, Mrigashira, Ardra, Punarvasu, Pushya, Ashlesha, Magha, Purva Phalguni, Uttara Phalguni, Hasta, Chitra, Swati, Vishakha, Anuradha, Jyeshtha, Mula, Purva Ashadha, Uttara Ashadha, Shravana, Dhanishta, Shatabhisha, Purva Bhadrapada, Uttara Bhadrapada, Revati.
- Lords: Ketu, Venus, Sun, Moon, Mars, Rahu, Jupiter, Saturn, Mercury (repeating 3 times).
- `nakshatra_info(lon: f64) -> NakshatraInfo`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package khagole-engine vedic::nakshatra`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/khagole-engine/src/vedic
git commit -m "feat(vedic): implement 27 nakshatras, lords, and pada calculations"
```

---

### Task 2: Divisional Charts Engine (D1 to D60)

**Files:**
- Create: `crates/khagole-engine/src/vedic/divisional.rs`
- Modify: `crates/khagole-engine/src/vedic/mod.rs`
- Test: `crates/khagole-engine/src/vedic/divisional.rs` (inline test module)

**Interfaces:**
- Consumes: `nakshatra_info` from Task 1.
- Produces:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
  pub struct DivisionalPosition {
      pub sign: u8,          // 1..12
      pub degree: f64,       // 0.0..30.0
      pub longitude: f64,    // 0.0..360.0
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct DivisionalPlanet {
      pub name: String,
      pub sign: u8,
      pub degree: f64,
      pub longitude: f64,
      pub house: u8,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct DivisionalChart {
      pub varga: String,
      pub lagna: DivisionalPosition,
      pub planets: Vec<DivisionalPlanet>,
  }

  pub fn calc_divisional_point(lon: f64, varga: &str) -> Result<DivisionalPosition, EngineError>;
  pub fn calc_divisional_chart(planets: &[(String, f64)], lagna_lon: f64, varga: &str) -> Result<DivisionalChart, EngineError>;
  ```

- [ ] **Step 1: Write the failing test for divisional calculations**

Test positions for D1, D2, D3, D4, D5, D7, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60 matching `hora-prakash` rules:
```rust
#[test]
fn test_divisional_d9() {
    // 0° Aries -> Aries (sign 1)
    let p0 = calc_divisional_point(0.0, "D9").unwrap();
    assert_eq!(p0.sign, 1);
    assert_eq!(p0.degree, 0.0);

    // 15° Taurus -> Taurus sign 2, part = floor(15 * 9 / 30) = 4. Earth seed = Cap (10). (9 + 4) % 12 + 1 = 2 (Taurus)
    let p_tau = calc_divisional_point(45.0, "D9").unwrap();
    assert_eq!(p_tau.sign, 2);
}

#[test]
fn test_divisional_d2_hora() {
    // Aries 5° (first half of odd sign) -> 2*1 - 1 = 1 (Aries)
    let p = calc_divisional_point(5.0, "D2").unwrap();
    assert_eq!(p.sign, 1);
}

#[test]
fn test_divisional_d30() {
    // Aries 4° (odd sign, <= 5°) -> Mars sign 1 (Aries)
    let p = calc_divisional_point(4.0, "D30").unwrap();
    assert_eq!(p.sign, 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package khagole-engine vedic::divisional`
Expected: FAIL with "cannot find function `calc_divisional_point`"

- [ ] **Step 3: Implement `divisional.rs`**

Implement:
- Helper functions `sign_of`, `deg_in_sign`, `norm_sign`, `part`, `div_deg`, `parivritti`.
- Exact algorithms for D1, D2 (Hora Udayashakti/PVR), D3 (Drekkana), D4 (Chaturthamsha), D5 (Panchamsha), D6 (Shashthamsha), D7 (Saptamsha), D8 (Ashtamsha), D9 (Navamsha element-triplicity), D10 (Dashamsha), D11 (Rudramsha), D12 (Dwadashamsha), D16 (Shodashamsha), D20 (Vimshamsha), D24 (Siddhamsha), D27 (Nakshatramsha), D30 (Trimshamsha unequal bounds), D40 (Khavedamsha), D45 (Akshavedamsha), D60 (Shashtyamsha).
- `calc_divisional_point` and `calc_divisional_chart`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package khagole-engine vedic::divisional`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/khagole-engine/src/vedic/divisional.rs crates/khagole-engine/src/vedic/mod.rs
git commit -m "feat(vedic): implement D1 to D60 divisional chart calculations"
```

---

### Task 3: Vimshottari Dasha Engine

**Files:**
- Create: `crates/khagole-engine/src/vedic/dasha.rs`
- Modify: `crates/khagole-engine/src/vedic/mod.rs`
- Test: `crates/khagole-engine/src/vedic/dasha.rs` (inline test module)

**Interfaces:**
- Consumes: `nakshatra_info` from Task 1.
- Produces:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub enum YearType {
      Sidereal,
      Savana,
      Tropical,
      Julian,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct DashaNode {
      pub planet: String,
      pub start_jd: f64,
      pub end_jd: f64,
      pub start_date: String,
      pub end_date: String,
      #[serde(skip_serializing_if = "Vec::is_empty", default)]
      pub children: Vec<DashaNode>,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub bhukta_days: Option<f64>,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub full_duration_days: Option<f64>,
  }

  pub fn calc_vimshottari_dasha(
      moon_lon: f64,
      birth_jd: f64,
      depth: u32,
      year_type: YearType,
  ) -> Vec<DashaNode>;
  ```

- [ ] **Step 1: Write the failing test for Vimshottari dasha**

In `crates/khagole-engine/src/vedic/dasha.rs`:
```rust
#[test]
fn test_vimshottari_dasha_structure() {
    let birth_jd = 2451545.0; // J2000
    // Moon at 0° Aries -> Ashwini -> Ketu dasha starts
    let tree = calc_vimshottari_dasha(0.0, birth_jd, 3, YearType::Sidereal);
    assert_eq!(tree.len(), 9);
    assert_eq!(tree[0].planet, "Ketu");
    assert_eq!(tree[1].planet, "Venus");
    assert_eq!(tree[2].planet, "Sun");
    assert_eq!(tree[3].planet, "Moon");
    assert_eq!(tree[4].planet, "Mars");
    assert_eq!(tree[5].planet, "Rahu");
    assert_eq!(tree[6].planet, "Jupiter");
    assert_eq!(tree[7].planet, "Saturn");
    assert_eq!(tree[8].planet, "Mercury");

    // Total duration should equal 120 sidereal years
    let total_days = tree.last().unwrap().end_jd - tree.first().unwrap().start_jd;
    assert!((total_days - 120.0 * 365.256363004).abs() < 0.1);

    // Each Mahadasha has 9 Antardashas
    assert_eq!(tree[0].children.len(), 9);
    // Each Antardasha has 9 Pratyantardashas
    assert_eq!(tree[0].children[0].children.len(), 9);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package khagole-engine vedic::dasha`
Expected: FAIL with "cannot find function `calc_vimshottari_dasha`"

- [ ] **Step 3: Implement `dasha.rs`**

Implement:
- Vimshottari sequence constants: Ketu (7), Venus (20), Sun (6), Moon (10), Mars (7), Rahu (18), Jupiter (16), Saturn (19), Mercury (17).
- Year types and day lengths:
  - Sidereal: 365.256363004
  - Savana: 360.0
  - Tropical: 365.242190
  - Julian: 365.25
- Julian day to YYYY-MM-DD date formatter (pure arithmetic).
- Balance and elapsed bhukta calculations.
- Recursive sub-period generator up to requested depth.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package khagole-engine vedic::dasha`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/khagole-engine/src/vedic/dasha.rs crates/khagole-engine/src/vedic/mod.rs
git commit -m "feat(vedic): implement Vimshottari dasha calculation with configurable depths and year standards"
```

---

### Task 4: Panchanga Calculations Engine

**Files:**
- Create: `crates/khagole-engine/src/vedic/panchang.rs`
- Modify: `crates/khagole-engine/src/vedic/mod.rs`
- Test: `crates/khagole-engine/src/vedic/panchang.rs` (inline test module)

**Interfaces:**
- Consumes: `nakshatra_info` from Task 1.
- Produces:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct TithiInfo {
      pub number: u8,            // 1..30
      pub name: String,          // e.g. "Pratipada (Shukla)"
      pub paksha: String,        // "Shukla" or "Krishna"
      pub percent_left: f64,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct VaraInfo {
      pub number: u8,            // 0..6 (0=Sunday)
      pub name: &'static str,    // "Sunday"..
      pub lord: &'static str,    // "Sun"..
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct YogaInfo {
      pub number: u8,            // 1..27
      pub name: &'static str,    // "Vishkambha"..
      pub percent_left: f64,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct KaranaInfo {
      pub number: u8,            // 1..60
      pub name: &'static str,    // "Bava", "Kimstughna", etc.
      pub percent_left: f64,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct SamvatInfo {
      pub kali_year: i32,
      pub shaka_year: i32,
      pub shaka_name: &'static str,
      pub vikram_year: i32,
      pub vikram_name: &'static str,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct PanchangResult {
      pub tithi: TithiInfo,
      pub vara: VaraInfo,
      pub nakshatra: NakshatraDetails,
      pub yoga: YogaInfo,
      pub karana: KaranaInfo,
      pub samvat: SamvatInfo,
  }

  pub fn calc_panchang(
      sun_lon: f64,
      moon_lon: f64,
      jd: f64,
      local_weekday: Option<u8>,
  ) -> PanchangResult;
  ```

- [ ] **Step 1: Write the failing test for Panchang**

In `crates/khagole-engine/src/vedic/panchang.rs`:
```rust
#[test]
fn test_panchang_limbs() {
    // New moon: Sun=10°, Moon=10° -> Tithi 1 (Shukla Pratipada), Yoga Vishkambha or similar
    let res = calc_panchang(10.0, 10.0, 2451545.0, Some(6));
    assert_eq!(res.tithi.number, 1);
    assert_eq!(res.tithi.paksha, "Shukla");
    assert_eq!(res.karana.name, "Kimstughna");
    assert_eq!(res.vara.name, "Saturday");
    assert_eq!(res.vara.lord, "Saturn");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package khagole-engine vedic::panchang`
Expected: FAIL with "cannot find function `calc_panchang`"

- [ ] **Step 3: Implement `panchang.rs`**

Implement:
- Tithi names, Paksha determination, Karana mapping (Kimstughna, 7 moveable, 3 fixed).
- 27 Yogas table and calculation: `(sun_lon + moon_lon) % 360 / (360/27)`.
- Weekdays (Vara) and rulers.
- 60 Jovian cycle Samvatsara names and conversions.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package khagole-engine vedic::panchang`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/khagole-engine/src/vedic/panchang.rs crates/khagole-engine/src/vedic/mod.rs
git commit -m "feat(vedic): implement five Panchanga angas and Samvatsara cycle"
```

---

### Task 5: Integration Tests & Golden Validation in Core Engine

**Files:**
- Create: `crates/khagole-engine/tests/vedic_tier_b.rs`
- Modify: `crates/khagole-engine/src/lib.rs`

- [ ] **Step 1: Write the integration test suite**

In `crates/khagole-engine/tests/vedic_tier_b.rs`:
- Load Narendra Modi chart (1950-09-17 04:04:00 UTC) with Lahiri ayanamsa.
- Test Navamsha (D9) positions for all 9 planets and Lagna against reference ephemeris fixture values.
- Test Vimshottari dasha starting balance (Saturn Mahadasha at birth).
- Test Panchang on birth date (Tithi, Vara, Nakshatra, Yoga, Karana).

- [ ] **Step 2: Run integration tests to verify they pass**

Run: `cargo test --package khagole-engine --test vedic_tier_b`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/khagole-engine/tests/vedic_tier_b.rs crates/khagole-engine/src/lib.rs
git commit -m "test(vedic): add comprehensive golden tests for Tier B calculations"
```

---

### Task 6: WebAssembly Bindings for Tier B Calculations

**Files:**
- Modify: `crates/khagole-engine-wasm/src/lib.rs`
- Modify: `crates/khagole-engine-wasm/Cargo.toml`
- Test: `crates/khagole-engine-wasm/tests/web_smoke.mjs`

**Interfaces:**
- Exposes on `Engine`:
  - `calc_divisional(chart_json: &str, varga: &str) -> Result<String, JsValue>`
  - `calc_vimshottari_dasha(moon_lon: f64, birth_jd: f64, depth: Option<u32>, year_type: Option<String>) -> Result<JsValue, JsValue>`
  - `calc_panchang(sun_lon: f64, moon_lon: f64, jd: f64, weekday: Option<u8>) -> Result<JsValue, JsValue>`

- [ ] **Step 1: Add methods to `Engine` in `crates/khagole-engine-wasm/src/lib.rs`**
- [ ] **Step 2: Build WASM package using `wasm-pack`**

Run: `wasm-pack build crates/khagole-engine-wasm --target web`
Expected: SUCCESS

- [ ] **Step 3: Add WASM test verifying Tier B methods in `tests/web_smoke.mjs`**

Run: `node crates/khagole-engine-wasm/tests/web_smoke.mjs`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add crates/khagole-engine-wasm
git commit -m "feat(wasm): export calc_divisional, calc_vimshottari_dasha, and calc_panchang in WASM"
```

---

### Task 7: MCP Server Tools for Vedic Calculations

**Files:**
- Modify: `crates/khagole-mcp/src/main.rs`
- Test: Integration test over stdio

**Interfaces:**
- Exposes 3 new tools in MCP tool catalog:
  - `calculate_divisional_chart`
  - `calculate_vimshottari_dasha`
  - `calculate_panchanga`

- [ ] **Step 1: Implement tool declarations and handlers in `crates/khagole-mcp/src/main.rs`**
- [ ] **Step 2: Run `cargo check` and `cargo test` on `khagole-mcp`**

Run: `cargo test --package khagole-mcp`
Expected: PASS

- [ ] **Step 3: Test MCP stdio tool calling with JSON-RPC payload**

Run: send `tools/call` for `calculate_divisional_chart`, `calculate_vimshottari_dasha`, `calculate_panchanga` through CLI test.
Expected: Valid JSON results with status 200.

- [ ] **Step 4: Commit**

```bash
git add crates/khagole-mcp/src/main.rs
git commit -m "feat(mcp): add calculate_divisional_chart, calculate_vimshottari_dasha, and calculate_panchanga tools"
```
