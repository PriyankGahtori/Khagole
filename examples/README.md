# Khagole Implementation Examples

Complete, working examples demonstrating how to use Khagole across Rust, Python, Node.js, Web Browser (CDN), and REST API.

All examples compute high-precision Vedic Jyotish horoscopes for renowned celebrities (**Steve Jobs**, **Albert Einstein**, **Narendra Modi**) showcasing:
- **Ayanamsa**: Chitrapaksha Lahiri exact angle and DMS
- **Navagrahas & Lagna**: Degrees, DMS, House placement, Nakshatra, Pada (1–4), Nakshatra Lord, Motion (Direct/Retrograde)
- **Vedic Panchanga**: Five Limbs (Tithi, Vara, Nakshatra, Yoga, Karana, Masa, Ritu, Ayana, Samvat)
- **D9 Navamsha**: Full divisional planetary placements
- **Vimshottari Dasha**: 120-year cycle tree (Mahadasha & Antardashas) with True Sidereal Solar Year (TSSY)

---

## Directory Overview

| Directory | Language / Platform | Runtime / Package | Description |
|---|---|---|---|
| [**`rust/`**](./rust) | Rust | Cargo (`khagole-engine`) | Native Rust implementation with NASA JPL DE440s SPK ephemeris |
| [**`python/`**](./python) | Python | `pip install khagole-engine` | High-performance PyO3 C-extension bindings for Python |
| [**`nodejs/`**](./nodejs) | Node.js | `npm install khagole-engine` | Modern ES Module / Node.js WebAssembly bindings |
| [**`web-cdn/`**](./web-cdn) | Browser (Zero-Build) | jsDelivr CDN (`<script type="module">`) | Pure HTML/JS browser webapp running client-side with zero build tools |
| [**`rest-api/`**](./rest-api) | REST Microservice | FastAPI / Uvicorn | Microservice exposing JSON endpoints and interactive Swagger UI |

---

## 1. Quick Start: Web Browser via CDN (Zero Build Step)

You can include Khagole in any web page with zero bundlers or build steps directly via CDN:

```html
<script type="module">
  import init, { Engine, calcPanchang, calcDivisional, calcVimshottariDasha, nakshatraInfo } 
    from 'https://cdn.jsdelivr.net/npm/khagole-engine@0.0.12/khagole_engine_wasm.js';

  // 1. Initialize WebAssembly directly from jsDelivr CDN
  await init();

  // 2. Query Nakshatra & Pada
  const moonNak = nakshatraInfo(217.9496);
  console.log(`Moon is in ${moonNak.name} Pada ${moonNak.pada} (Lord: ${moonNak.lord})`);

  // 3. Query Panchanga
  const panchang = calcPanchang(150.5392, 217.9496, 2433541.6694);
  console.log(`Tithi: ${panchang.tithi.name}, Vara: ${panchang.vara.name}`);
</script>
```

👉 See [**`examples/web-cdn/index.html`**](./web-cdn/index.html) for a complete interactive celebrity horoscope calculator.

---

## 2. Quick Start: Python

```bash
pip install khagole-engine
python examples/python/celebrity_chart.py
```

```python
import khagole_engine as ke

engine = ke.Engine(["kernels/de440s.bsp"])
chart = engine.chart(
    "1955-02-25T03:15:00Z",
    37.7749,
    -122.4194,
    zodiac_type="sidereal",
    ayanamsa="lahiri",
    house_system="W",
    position_mode="true",
)
print("Sun:", chart["planets"][0]["sign"], chart["planets"][0]["degree"])
```

---

## 3. Quick Start: Node.js

```bash
npm install khagole-engine
node examples/nodejs/celebrity_chart.mjs
```

---

## 4. Quick Start: Rust

```bash
cargo run --manifest-path examples/rust/Cargo.toml
```

---

## 5. Quick Start: REST API

```bash
uvicorn examples.rest-api.app:app --port 8000
curl http://localhost:8000/api/celebrity/jobs
```
Interactive Swagger documentation available at `http://localhost:8000/docs`.
