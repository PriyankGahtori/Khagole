# Khagole Web CDN Example — Zero-Build Browser Vedic Calculator

Demonstrates using `khagole-engine` directly in any web browser via jsDelivr CDN without bundlers, compilation, or `node_modules`.

Inspired by projects like [swisseph-wasm](https://github.com/prolaxu/swisseph-wasm), this example loads the high-precision WebAssembly module directly from jsDelivr:

```html
<script type="module">
  import init, {
    Engine,
    calcPanchang,
    calcDivisional,
    calcVimshottariDasha,
    nakshatraInfo,
    julianDay
  } from 'https://cdn.jsdelivr.net/npm/khagole-engine@0.0.12/khagole_engine_wasm.js';

  // Initialize WebAssembly runtime
  await init();

  // Run any calculation!
</script>
```

## Running Locally

To test locally in your browser, serve this directory with any static web server:

```bash
# Using Python
python3 -m http.server 8080 -d examples/web-cdn

# Or using npx serve
npx serve examples/web-cdn
```

Then visit `http://localhost:8080` in your web browser.

## Features Demonstrated
- 100% Client-side WASM execution from jsDelivr CDN
- Instant interactive horoscope calculation for famous celebrities (Steve Jobs, Albert Einstein, Narendra Modi)
- Navagrahas & Lagna table with degree/minute/second, house, nakshatra, pada, lord, motion
- Vedic Panchanga (Tithi, Vara, Nakshatra, Yoga, Karana, Masa, Ritu, Ayana, Samvat)
- D9 Navamsha Divisional Chart
- Vimshottari Dasha sequence tree (TSSY True Sidereal Solar Year)
