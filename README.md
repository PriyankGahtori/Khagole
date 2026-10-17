<div align="center">

# 🌌 Khagole (खगोल)
### High-Precision Vedic Jyotish & Astrometric Calculation Engine on NASA JPL Ephemerides
**100% Free & Open Source Alternative to Swiss Ephemeris & Commercial Cloud Astrology APIs**

[![crates.io](https://img.shields.io/crates/v/khagole-engine.svg?style=flat-square&logo=rust&color=e05d44)](https://crates.io/crates/khagole-engine)
[![docs.rs](https://img.shields.io/docsrs/khagole-engine?style=flat-square&logo=docsdotrs&color=4d76cf)](https://docs.rs/khagole-engine)
[![PyPI](https://img.shields.io/pypi/v/khagole-engine.svg?style=flat-square&logo=pypi&logoColor=white&color=3775a9)](https://pypi.org/project/khagole-engine/)
[![npm](https://img.shields.io/npm/v/khagole-engine.svg?style=flat-square&logo=npm&color=cb3837)](https://www.npmjs.com/package/khagole-engine)
[![GitHub Stars](https://img.shields.io/github/stars/PriyankGahtori/Khagole?style=flat-square&logo=github&color=f4c430)](https://github.com/PriyankGahtori/Khagole/stargazers)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](https://github.com/PriyankGahtori/Khagole/blob/main/LICENSE-MIT)
[![GitHub Pages](https://github.com/PriyankGahtori/Khagole/actions/workflows/pages.yml/badge.svg?style=flat-square)](https://priyankgahtori.github.io/Khagole/)
[![CI](https://github.com/PriyankGahtori/Khagole/actions/workflows/ci.yml/badge.svg?style=flat-square)](https://github.com/PriyankGahtori/Khagole/actions/workflows/ci.yml)

<p align="center">
  <img src="https://img.shields.io/badge/Cost-%240.00%20(100%25%20Free%20%26%20Open%20Source)-success?style=flat-square" alt="Cost: $0 Free">
  <img src="https://img.shields.io/badge/Latency-%3C0.5ms%20Sub--millisecond-blueviolet?style=flat-square" alt="Sub-millisecond Latency">
  <img src="https://img.shields.io/badge/Precision-NASA%20JPL%20DE440s%20Sub--arcsecond-blue?style=flat-square" alt="NASA JPL DE440s">
  <img src="https://img.shields.io/badge/AI%20Protocol-MCP%202.3%20(SSE%20%2B%20stdio)-orange?style=flat-square" alt="Model Context Protocol">
  <img src="https://img.shields.io/badge/Privacy-100%25%20On--Device%20%2F%20Offline-green?style=flat-square" alt="100% Private">
</p>

**[🚀 Live Interactive Web App](https://priyankgahtori.github.io/Khagole/)** · **[🌐 Zero-Build Web CDN Demo](https://priyankgahtori.github.io/Khagole/examples/web-cdn/)** · **[📖 API Docs (docs.rs)](https://docs.rs/khagole-engine)** · **[🤖 MCP Server Guide](examples/mcp-server)** · **[Technical Guides](https://github.com/PriyankGahtori/Khagole/tree/main/docs)**

</div>

---

**Khagole** (खगोल — *The Celestial Sphere* in Sanskrit) is the first modern, memory-safe astrometry and Vedic Jyotish computation engine built directly on NASA JPL DE440s / DE441 development ephemerides in pure Rust.

The same calculation core runs natively on servers (Python bindings), in desktop/mobile apps, in web browsers via WebAssembly (WASM), and across AI agent frameworks via Model Context Protocol (MCP), guaranteeing **bit-for-bit identical results** down to the sub-arcsecond across every platform.

---

## 📑 Table of Contents
- [⚡ Why Khagole? (Architectural Comparison)](#-why-khagole)
- [💰 Cost & Performance ROI Comparison](#-cost--performance-roi-comparison)
- [🪔 Key Capabilities (Vedic, Western, Chinese)](#key-capabilities)
- [📦 Installation across Rust, Python, JavaScript](#installation)
- [🌐 Web Browser via CDN (Zero Build Tools)](#-web-browser-via-cdn-zero-build-tools)
- [🤖 Model Context Protocol (MCP) Server for AI & LLMs](#-model-context-protocol-mcp-server-for-ai--llms)
- [🌟 Complete Working Examples & Celebrity Horoscopes](#-complete-examples--celebrity-horoscopes)
- [💻 Multi-Language Code Usage](#rust-usage)
- [🏛️ System Architecture](#️-system-architecture)
- [🧪 Testing & Verification](#testing--verification)
- [Credits & Reference Works](#credits--reference-works)

---

## ⚡ Why Khagole?

| Capability | **Khagole (खगोल)** | Swiss Ephemeris (`swisseph`) | Flatlib / PyEphem |
|---|---|---|---|
| **Core Architecture** | **100% Pure Rust** (Memory-safe, Zero-panic) | Legacy C (Unsafe pointers) | C wrappers / Old Python |
| **Ephemeris Precision** | **NASA JPL DE440s / DE441** (Sub-arcsecond) | Semi-analytic / Moshier fallback | Low-order approximations |
| **Vedic Kundli & Vargas** | **Native D1 through D60 Shodashavarga** | Requires custom division code | Not supported |
| **Vimshottari Dasha** | **True Sidereal Solar Year (TSSY) Solver** | Not supported | Not supported |
| **Panchanga Limbs** | **Sub-millisecond (Tithi, Vara, Nakshatra, Yoga, Karana)** | Custom manual math required | Not supported |
| **In-Browser Execution** | **WebAssembly (<1MB, 100% Offline & Private)** | Heavy Emscripten build (>5MB) | Server-only |
| **Cross-Platform Parity**| **Bit-for-bit identical results** everywhere | Slight compiler/float drift | Inconsistent |
| **AI / Agentic LLM** | **Native Model Context Protocol (`khagole-mcp`)** | None | None |

---

## 💰 Cost & Performance ROI Comparison

Commercial cloud astrology APIs (such as AstrologyAPI, Prokerala, and VedicAstroAPI) charge recurring subscription fees and metered request costs. Traditional C libraries require either restrictive GPL licenses or expensive commercial buyouts.

**Khagole delivers enterprise-grade astronomical precision at zero cost**:

| Metric / Dimension | **Khagole (खगोल)** | **Commercial Astrology APIs**<br/>*(AstrologyAPI, Prokerala, VedicAstroAPI)* | **Swiss Ephemeris (`swisseph`)**<br/>*(Astrodienst Commercial)* |
|---|---|---|---|
| **Monthly Subscription** | **$0.00 / month** (100% Free Forever) | **$29 to $499+ / month** | Free (GPL) or **$750 – $1,000+** (Commercial) |
| **Cost per 1M API Calls** | **$0.00** (Unlimited local compute) | **$2,500 – $10,000+** (Metered billing) | $0.00 (After license purchase) |
| **Calculation Latency** | **< 0.5 ms** (Sub-millisecond local / WASM) | **250 ms – 1,200 ms** (Network latency) | 1 ms – 5 ms (Native C) |
| **Rate Limits & Throttling** | **Unlimited** (Runs at CPU hardware speed) | **5,000 – 100,000 req/mo** hard limits | Unlimited |
| **User Data Privacy & PII** | **100% Private** (Birth details never leave device) | **0% Private** (Birth dates & GPS sent to 3rd-party) | 100% Private (Self-hosted) |
| **Offline Functionality** | **100% Offline** (Zero internet connection needed) | ❌ **Breaks completely** when offline | 100% Offline |
| **In-Browser Execution** | **WebAssembly (<1 MB)** directly via CDN | ❌ **Server-only** (Exposes API keys in frontend) | ⚠️ Emscripten build (>5 MB) |
| **AI Agent / LLM Native** | **Native MCP 2.3** (SSE over HTTP & stdio) | ❌ Proprietary REST only | ❌ None |
| **Memory Safety** | **100% Pure Rust** (Zero memory vulnerabilities) | Closed-source cloud black box | ⚠️ Legacy C (Unsafe pointers, segfault risks) |
| **Ephemeris Core** | **NASA JPL DE440s / DE441** (Sub-arcsecond) | Approximations or unverified backends | Semi-analytic / Moshier fallback |
| **Vedic Tier B Features** | **Native Vargas (D1–D60), TSSY Dasha, Panchanga** | Often charged as extra premium add-ons | ❌ Requires custom manual division code |

---

## Key Capabilities

### 🪔 Vedic Astrology (Jyotish Kundali)
- **Divisional Charts (Vargas)**: Computes D1 (Rasi), D9 (Navamsha), and all 16 Shodashavargas (D2 Hora, D3 Drekkana, D4 Chaturthamsa, D7 Saptamsa, D10 Dasamsa, D12 Dwadasamsa, D16 Shodasamsa, D20 Vimsamsa, D24 Chaturvimsamsa, D27 Saptavimsamsa, D30 Trimsamsa, D40 Khavedamsa, D45 Akshavedamsa, D60 Shashtiamsa).
- **Interactive SVG Charts**: North Indian (diamond) and South Indian (box) chart renderers with Google Material 3 typography, arcminute precision (`12°17′`), Kendra elevations, golden Lagna badges, and non-overlapping multi-planet conjunction grids.
- **Vimshottari Dasha Engine**: Multi-level Mahadasha, Antardasha, and Pratyantardasha timeline and hierarchical tree. Computations support **True Sidereal Solar Year (TSSY)** solar return solver matching canonical Parashari standards, alongside 365.2564-day sidereal, 360-day savana, and 365.2422-day tropical standards.
- **Canonical Parashari Defaults**: Out-of-the-box configuration matches canonical Vedic defaults:
  - **Zodiac**: Sidereal
  - **Ayanamsa**: Chitrapaksha Lahiri (`UserSelectedAyanamsa=17`)
  - **Lunar Nodes**: Mean nodes (`TrueNodes=0`)
  - **Position Mode**: True / Geometric (`TruePositions=1`)
  - **House System**: Whole Sign / Rashi Bhava (`W`)
  - **Dasha Year Standard**: TSSY (True Sidereal Solar Year)
- **Panchanga (Five Limbs of Time)**: Sub-millisecond calculation of Tithi, Vara, Nakshatra with Pada, Yoga, and Karana.
- **Global City Geocoding & Auto-Timezone**: Integrated Open-Meteo & Nominatim autocomplete search for any city globally, auto-filling coordinates and resolving exact historical/DST UTC offsets offline via browser `Intl.DateTimeFormat`.

### 🌐 Western Astrology
- **Planets & Points**: Sun through Pluto, lunar nodes, Chiron, and Black Moon Lilith with speed and retrograde status, Ascendant, Midheaven, and Vertex.
- **House Systems**: Eleven systems checked against Swiss Ephemeris (Placidus, Koch, Regiomontanus, Campanus, Topocentric, Alcabitius, Morinus, Porphyry, Equal, Vehlow, Whole Sign).
- **Aspects & Orbs**: Conjunction, sextile, square, trine, opposition, semi-sextile, and quincunx with configurable orbs and moiety methods.
- **Horary & Electional**: Planetary day and hour from computed sunrise/sunset, significators, void-of-course Moon, considerations before judgment, and optimal window searching.
- **Derived & Transits**: Secondary progressions, solar arcs, synastry cross-aspects, and turned charts.

### 🎋 Chinese Astrology
- **Solar Terms & Calendars**: Solar terms (Jie Qi), lunar and leap months, Chinese New Year anchors.
- **BaZi (Four Pillars of Destiny)**: Year, Month, Day, and Hour pillars calculated on true solar time, with Ten Gods, hidden stems, and Luck Pillars.
- **Zi Wei Dou Shu (Purple Star Astrology)**: 12-palace grid, 14 major stars, minor stars, four mutagens (Si Hua), and decennial/annual horoscopes validated against `iztro`.

---

## Installation

Add **Khagole** to your project across Rust, Python, or JavaScript/TypeScript:

```bash
# Rust (Cargo)
cargo add khagole-engine

# Python (pip, CPython ≥ 3.12)
pip install khagole-engine

# JavaScript / Node.js / Browser (npm)
npm install khagole-engine
```

### 🌐 Web Browser via CDN (Zero Build Tools)

Import directly in any web page with a single `<script type="module">` tag, with zero bundlers or dependencies:

```html
<script type="module">
  import init, { Engine, calcPanchang, calcDivisional, calcVimshottariDasha, nakshatraInfo } 
    from 'https://cdn.jsdelivr.net/npm/khagole-engine@0.0.12/khagole_engine_wasm.js';

  // Initialize WebAssembly runtime directly from jsDelivr CDN
  await init();

  // 1. Calculate Nakshatra & Pada
  const moonNak = nakshatraInfo(217.9496);
  console.log(`Moon in ${moonNak.name} Pada ${moonNak.pada} (Lord: ${moonNak.lord})`);

  // 2. Calculate Vedic Panchanga
  const panchang = calcPanchang(150.5392, 217.9496, 2433541.6694);
  console.log(`Tithi: ${panchang.tithi.name}, Vara: ${panchang.vara.name}`);
</script>
```

### 🤖 Model Context Protocol (MCP) Server for AI & LLMs

Khagole includes a high-speed, dual-transport MCP server enabling AI Agents (**Claude Desktop**, **Cursor**, **LangChain**, **CrewAI**, **AutoGen**, and web chatbots) to perform live sub-arcsecond astrological reasoning:

#### Option A: Hosted Remote Server (SSE over HTTP) for Cloud Agents
```bash
# Run hosted SSE microservice for remote agents:
python examples/mcp-server/server.py --transport sse --host 0.0.0.0 --port 8000

# Or 1-click Docker container deployment:
docker build -t khagole-mcp -f examples/mcp-server/Dockerfile .
docker run -d -p 8000:8000 khagole-mcp
```
Connect any cloud agent to `http://localhost:8000/sse` using the standard Model Context Protocol.

#### Option B: Local Desktop AI Apps (Claude Desktop & Cursor via stdio)
Add to your `claude_desktop_config.json` or Cursor MCP settings:

```json
{
  "mcpServers": {
    "khagole": {
      "command": "cargo",
      "args": ["run", "--release", "-p", "khagole-mcp"]
    }
  }
}
```

👉 See [**`examples/mcp-server/README.md`**](examples/mcp-server) for full tool definitions and client agent integration guides.

---

## 🌟 Complete Examples & Celebrity Horoscopes

Working, tested examples outputting full Vedic horoscopes (Navagrahas, Lagna, Nakshatras & Padas, Panchanga, D9 Navamsha, and Vimshottari Dasha) for **Steve Jobs** and **Albert Einstein**:

| Example | Platform / Target | Highlights | How to Run | Link |
|---|---|---|---|---|
| **Web Browser (CDN)** | Vanilla HTML / JS | 100% Client-Side WASM on jsDelivr CDN, JPL ephemeris stream | Open in any browser | [**`examples/web-cdn`**](examples/web-cdn) |
| **MCP Server for AI** | AI Agents (SSE / stdio) | Claude Desktop, Cursor, LangChain, Docker cloud deploy | `python server.py --transport sse` | [**`examples/mcp-server`**](examples/mcp-server) |
| **Python** | Python ≥ 3.12 (`khagole-engine`) | Native PyO3 C-extensions, sub-millisecond execution | `python celebrity_chart.py` | [**`examples/python`**](examples/python) |
| **Node.js** | Node.js ESM (`khagole-engine`) | WebAssembly module with TypeScript definitions | `node celebrity_chart.mjs` | [**`examples/nodejs`**](examples/nodejs) |
| **Rust** | Rust / Cargo (`khagole-engine`) | Native NASA JPL SPK Chebyshev reduction core | `cargo run --release` | [**`examples/rust`**](examples/rust) |
| **REST API** | FastAPI / Uvicorn | Microservice with interactive Swagger UI (`/docs`) | `uvicorn app:app --port 8000` | [**`examples/rest-api`**](examples/rest-api) |

---

## Rust Usage

```rust
use khagole_engine::ephemeris::{Kernel, KernelSet, Spk};
use khagole_engine::{calculate_chart, ChartRequest, UtcInstant};

let mut kernels = KernelSet::new();
kernels.push(Kernel::new("de440s.bsp", Spk::open("kernels/de440s.bsp")?)?);

let request = ChartRequest::new(UtcInstant::parse("1987-05-17T14:30:00Z")?, 41.9, 12.5);
let chart = calculate_chart(&kernels, &request)?;
println!("{}", serde_json::to_string_pretty(&chart)?);
```

### Vedic Chart Calculation (WASM / Node.js / Browser)

```javascript
import init, { Engine, calcDivisional, calcPanchang, calcVimshottariDasha, julianDay } from "khagole-engine";

await init();
const engine = new Engine();
engine.addKernel("de440s-1950-2050.bsp", kernelBytes);

// Calculate Sidereal Vedic Chart with canonical Parashari defaults
const chart = engine.chart({
  utc: "1950-09-17T04:04:00Z",
  latitude: 23.784,
  longitude: 72.636,
  zodiac_type: "sidereal",
  ayanamsa: "lahiri",
  position_mode: "true",
  node_type: "mean",
  house_system: "W",
});

const ascLon = chart.planets.find(p => p.name === "Ascendant").ecliptic_longitude;
const moonLon = chart.planets.find(p => p.name === "Moon").ecliptic_longitude;
const jd = julianDay("1950-09-17T04:04:00Z");

const d1 = calcDivisional(chart, ascLon, "D1");
const d9 = calcDivisional(chart, ascLon, "D9");
const dasha = calcVimshottariDasha(moonLon, jd, 3, "tssy");
```

---

## Language Bindings & Packages

- **Rust**: [`khagole-engine`](https://crates.io/crates/khagole-engine) and [`khagole-mcp`](https://crates.io/crates/khagole-mcp) on [crates.io](https://crates.io/crates/khagole-engine).
- **Python** ([`khagole-engine` on PyPI](https://pypi.org/project/khagole-engine/), CPython ≥ 3.12): Fast PyO3 bindings with native precompiled wheels for Linux (x86_64, aarch64, musl), macOS (universal2), and Windows (x64). See [crates/khagole-engine-py](https://github.com/PriyankGahtori/Khagole/tree/main/crates/khagole-engine-py).
- **JavaScript / WebAssembly** ([`khagole-engine` on npm](https://www.npmjs.com/package/khagole-engine)): In-browser and Node.js execution with zero network latency. See [crates/khagole-engine-wasm](https://github.com/PriyankGahtori/Khagole/tree/main/crates/khagole-engine-wasm).
- **CDN**: Available globally via jsDelivr: `https://cdn.jsdelivr.net/npm/khagole-engine@0.0.12/khagole_engine_wasm.js`.

---

## Documentation

- [API Reference on docs.rs](https://docs.rs/khagole-engine)
- [API Guide](https://github.com/PriyankGahtori/Khagole/blob/main/docs/api.md): Request parameters, options, chart JSON schema, and error codes
- [Ephemerides Guide](https://github.com/PriyankGahtori/Khagole/blob/main/docs/ephemerides.md): SPK kernels, date spans, in-memory excerpts
- [Accuracy Verification](https://github.com/PriyankGahtori/Khagole/blob/main/docs/accuracy.md): Stage-by-stage verification against JPL, Skyfield, and Swiss Ephemeris
- [Architecture](https://github.com/PriyankGahtori/Khagole/blob/main/docs/architecture.md): Core engine and pipeline layout

---

## 🏛️ System Architecture

```mermaid
flowchart TD
    subgraph Input["1. Calculation Request"]
        Birth["Date, Time, Latitude, Longitude"]
        Config["Ayanamsa (Lahiri), Zodiac, Nodes (Mean), Houses (Whole Sign)"]
    end

    subgraph Astrometry["2. NASA JPL Astrometric Reduction"]
        JPL[("NASA JPL DE440s / DE441 Ephemeris")] --> State["Barycentric State Vectors"]
        State --> Reduction["ΔT · IAU 2006 Precession · IAU 2000A Nutation · Light-Time · Deflection"]
        Reduction --> Longitudes["Geometric True Ecliptic Longitudes"]
    end

    subgraph Vedic["3. Khagole Vedic Engine"]
        Longitudes --> AyanamsaShift["Chitrapaksha Lahiri Ayanamsa Subtraction"]
        AyanamsaShift --> Vargas["Shodashavargas (D1 Rasi to D60 Shashtiamsa)"]
        AyanamsaShift --> Dasha["Vimshottari Dasha (True Sidereal Solar Year TSSY)"]
        AyanamsaShift --> Panchang["Panchanga Limbs (Tithi, Vara, Nakshatra, Yoga, Karana)"]
    end

    subgraph Ecosystem["4. Multi-Platform Targets"]
        Vargas & Dasha & Panchang --> RustCore["Pure Rust Library (crates.io)"]
        Vargas & Dasha & Panchang --> PyO3["Python Bindings (PyPI)"]
        Vargas & Dasha & Panchang --> WASM["WebAssembly Engine (npm)"]
        Vargas & Dasha & Panchang --> MCP["Model Context Protocol Server (Claude / Cursor)"]
    end

    Input --> Astrometry
    Astrometry --> Vedic
```

---

## Repository Layout

| Path | Description |
|---|---|
| `crates/khagole-engine` | Core Rust calculation library & NASA JPL SPK reduction engine |
| `crates/khagole-engine-py` | Python bindings (PyO3 & maturin) |
| `crates/khagole-engine-wasm` | WebAssembly bindings (wasm-bindgen & wasm-pack) |
| `crates/khagole-mcp` | Native Rust Model Context Protocol (MCP) server |
| `examples/web-cdn` | Zero-build in-browser Vedic horoscope calculator live on CDN |
| `examples/mcp-server` | Hosted HTTP/SSE & stdio MCP Server for AI agents (Claude, Cursor, LangChain) |
| `examples/python` | Python celebrity chart calculation example |
| `examples/nodejs` | Node.js ESM celebrity chart calculation example |
| `examples/rust` | Standalone Rust celebrity chart calculation example |
| `examples/rest-api` | FastAPI REST microservice with interactive Swagger UI |
| `site/` | Modern client-side web application deployed to GitHub Pages |
| `tests/fixtures/*.json` | Acceptance test golden charts |
| `tests/data/de440s_2000.bsp` | One-year DE440s excerpt for automated test suites |
| `scripts/fetch-kernels.sh` | Downloads full JPL DE440s / DE441 kernels |
| `scripts/build-site.sh` | Compiles WASM, extracts DE440s excerpt, and builds `target/site` |

---

## Testing & Verification

```bash
# Run test suite using the committed excerpt
cargo test --workspace --exclude khagole-engine-py

# Download full DE440s (1849–2150) to run full-range ephemeris verification tests
scripts/fetch-kernels.sh

# Lint codebase
cargo clippy --workspace --all-targets -- -D warnings
```

---

## ⭐ Star History & Community

If you find **Khagole** useful for your projects, astrological software, or AI agents, please consider starring the repository! It helps more developers discover the engine.

[![Star History Chart](https://api.star-history.com/svg?repos=PriyankGahtori/Khagole&type=Date)](https://star-history.com/#PriyankGahtori/Khagole&Date)

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!
- Check out the [Contributing Guide](https://github.com/PriyankGahtori/Khagole/blob/main/CONTRIBUTING.md) to get started.
- Open an [Issue](https://github.com/PriyankGahtori/Khagole/issues) for bug reports, divisional chart expansions, or feature suggestions.

---

## Credits & Reference Works

**Khagole** is built on top of and inspired by exceptional foundational works across astronomy, astrometry, and astrology:

- **[astroceleste-engine](https://github.com/ffalcinelli/astroceleste-engine)** by Francesco Falcinelli: The upstream Rust astrometric calculation engine, SPK/DAF NASA JPL kernel parser, astronomical reduction pipeline architecture, and golden chart testing framework.
- **[Skyfield](https://github.com/skyfielders/python-skyfield)** by Brandon Rhodes (MIT License): Precision astronomical reduction routines ported from Python to pure Rust, including ΔT models, IAU 2000A nutation tables, IAU 2006 precession, aberration, light-time correction, and gravitational deflection.
- **NASA Jet Propulsion Laboratory (JPL)**: Planetary and lunar development ephemerides (DE440s / DE441), the primary gold-standard foundation for celestial positions.
- **[Swiss Ephemeris](https://www.astro.com/swisseph/)** by Astrodienst (Alois Treindl & Dieter Koch): Benchmark reference for planetary position verification, ayanamsa values, and house cusp systems.
- **[Jagannatha Hora](https://www.vedicastrologer.org/jh/)** by P.V.R. Narasimha Rao: Standard reference benchmark for canonical Vedic Jyotish calculations, Shodashavarga divisional chart algorithms, and True Sidereal Solar Year (TSSY) Vimshottari Dasha calculations.
- **[iztro](https://github.com/SylarLong/iztro)** by Sylar Long (MIT License): Reference implementation and star brightness tables for Chinese Zi Wei Dou Shu astrology.
- **[Open-Meteo](https://open-meteo.com/) & [OpenStreetMap Nominatim](https://nominatim.openstreetmap.org/)**: Global city geocoding, coordinates, and timezone resolution datasets.

---

## License

Dual-licensed under either:
- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/PriyankGahtori/Khagole/blob/main/LICENSE-APACHE))
- MIT License ([LICENSE-MIT](https://github.com/PriyankGahtori/Khagole/blob/main/LICENSE-MIT))

See [NOTICE](https://github.com/PriyankGahtori/Khagole/blob/main/NOTICE) for full third-party copyright notices and attribution statements.
