# Khagole Web CDN Example — Zero-Build Browser Vedic Calculator

Demonstrates using `khagole-engine` directly in any web browser via jsDelivr CDN without bundlers, compilation, or `node_modules`.

This example loads the high-precision WebAssembly module directly from jsDelivr:

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

  // 1. Initialize WebAssembly runtime directly from CDN
  await init();

  // 2. Load NASA JPL ephemeris kernel (see Ephemeris Loading section below)
  const res = await fetch('https://priyankgahtori.github.io/Khagole/ephemeris/de440s-1920-2070.bsp');
  const kernelBytes = new Uint8Array(await res.arrayBuffer());

  const engine = new Engine();
  engine.addKernel('de440s-1920-2070.bsp', kernelBytes);

  // 3. Compute live Parashari Vedic chart
  const chart = engine.chart({
    utc: '1955-02-25T03:15:00Z',
    latitude: 37.7749,
    longitude: -122.4194,
    zodiac_type: 'sidereal',
    ayanamsa: 'lahiri',
    position_mode: 'true',
    node_type: 'mean',
    house_system: 'W',
  });
</script>
```

---

## How Ephemeris Data is Provided in the Web Browser

WebAssembly runs entirely client-side inside the user's browser sandbox and requires binary SPK ephemeris bytes to calculate planetary positions with NASA JPL precision.

`examples/web-cdn/index.html` provides the 150-year NASA JPL DE440s ephemeris (`de440s-1920-2070.bsp`, ~15.6 MB) through **3 verified mechanisms**:

### 1. Automatic Remote Fetch via CORS-Enabled Edge CDN
- **Why not NASA JPL directly?** NASA JPL's FTP/HTTP servers (`https://ssd.jpl.nasa.gov/...`) do not send CORS (`Access-Control-Allow-Origin: *`) headers. Modern browsers strictly block cross-origin requests from web applications when CORS headers are absent.
- **The Khagole CDN Solution:** The excerpted 150-year NASA JPL kernel (`de440s-1920-2070.bsp`, covering 1920–2070) is hosted on GitHub Pages with global Fastly Edge caching and `Access-Control-Allow-Origin: *`:
  ```
  https://priyankgahtori.github.io/Khagole/ephemeris/de440s-1920-2070.bsp
  ```
- The page streams this kernel with a live visual download progress bar in real time.

### 2. Automatic Local File Fetch (When Serving Locally)
When running a local web server (e.g., `python3 -m http.server 8080 -d examples/web-cdn`), the page automatically fetches the local copy from `./ephemeris/de440s-1920-2070.bsp` or `../../target/site/ephemeris/de440s-1920-2070.bsp`, loading in milliseconds with zero network latency.

### 3. Interactive File Picker (100% Offline / Custom `.bsp` Kernels)
An interactive file picker (`<input type="file" accept=".bsp">`) allows users to select any `.bsp` file from their local computer (such as the full 32 MB `de440s.bsp` covering 1849–2150). The browser reads the file locally via `FileReader` / `File.arrayBuffer()` without uploading anything to any remote server, giving 100% offline privacy and historical coverage across centuries.

---

## Running Locally

To test locally in your browser, serve this directory with any static web server:

```bash
# Using Python:
python3 -m http.server 8089 -d examples/web-cdn

# Or using Node.js:
npx serve examples/web-cdn
```

Then visit `http://localhost:8089` in your web browser.

---

## Live Deployment on GitHub Pages

The web CDN demo is deployed alongside the Khagole documentation site:
- **Full Landing App**: [https://priyankgahtori.github.io/Khagole/](https://priyankgahtori.github.io/Khagole/)
- **Standalone Web CDN Example**: [https://priyankgahtori.github.io/Khagole/examples/web-cdn/](https://priyankgahtori.github.io/Khagole/examples/web-cdn/)

---

## Verification & Automated Browser Testing

This example is verified with automated Playwright headless Chromium testing:

```bash
# Run the automated browser test suite:
.venv/bin/python scratch/test_browser_web_cdn.py
```

### Verified Test Assertions:
1. **WebAssembly Initialization**: Confirmed `khagole-engine@0.0.12` loads cleanly from jsDelivr CDN.
2. **Ephemeris Streaming**: Confirmed 15.6 MB `de440s-1920-2070.bsp` downloads and mounts into `Engine` memory.
3. **Steve Jobs (1955-02-24)**: Verified Sun in Aquarius 12° 31′ 24″ (H7), Moon in Pisces 14° 31′ 00″ (H8), Lahiri Ayanamsa 23.230654°, Tritiya Shukla Tithi.
4. **Narendra Modi (1950-09-17)**: Verified Sun in Virgo 0° 32′ 21″ (H11), Moon in Scorpio 7° 57′ 58″ (H2), Lagna in Libra 12° 17′ 23″ (H1), Anuradha Nakshatra, Shashthi Shukla Tithi.
5. **Vedic Panchanga**: Verified exact calculations for Tithi, Vara, Nakshatra, Yoga, Karana, Masa, Ritu, Ayana, and Samvat eras.
6. **D9 Navamsha & Vimshottari Dasha**: Verified complete D9 divisional signs and True Sidereal Solar Year (TSSY) Dasha timelines.
