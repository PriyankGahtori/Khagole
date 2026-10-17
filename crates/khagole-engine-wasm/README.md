# khagole-engine (WebAssembly)

[![npm](https://img.shields.io/npm/v/khagole-engine.svg)](https://www.npmjs.com/package/khagole-engine)

> **Experimental (0.0.x):** the API may change in any release. Pin an exact version.

Astrological chart calculation on NASA JPL ephemerides, compiled to WebAssembly from the
Rust [khagole-engine](https://github.com/PriyankGahtori/Khagole). Charts are
computed entirely in the browser (or Node), identical to the Khagole server.

[Try the live demo](https://ffalcinelli.github.io/khagole-engine/) ·
[API guide](https://github.com/ffalcinelli/khagole-engine/blob/main/docs/api.md)

```js
import init, { Engine, synastry, derivedChart, bazi, ziWei, timeLords } from "khagole-engine";

await init();
const engine = new Engine();
const kernel = await (await fetch("/ephemeris/de440s.bsp")).arrayBuffer();
engine.addKernel("de440s.bsp", new Uint8Array(kernel));

const chart = engine.chart({
  utc: "1987-05-17T14:30:00Z",
  latitude: 41.9,
  longitude: 12.5,
  house_system: "P",        // optional: P, K, R, C, T, B, M, O, E, V, W
  zodiac_type: "sidereal",  // optional: tropical (default) or sidereal
  ayanamsa: "lahiri",       // optional
  chinese_calendar: true,   // optional: add the Chinese calendar of the moment
});
const horary = engine.horary({ ...request, quesited_house: 7 }); // chart + horary_data
const transit = engine.transit(chart.planets, request);
const election = engine.election(request, { purpose: "contract" });
const windows = engine.elections(request, "1987-06-17T00:00:00Z", { purpose: "launch" });
synastry(chartA.planets, chartB.planets);
derivedChart(chart, 5);
timeLords(birthUtc, sunLongitude, ascendantLongitude, fromUtc, toUtc);
bazi(chart.chinese_calendar, request.utc, 12.5, 60);   // Four Pillars, UTC offset in minutes
ziWei(chart.chinese_calendar, request.utc, 12.5, 60);  // Zi Wei Dou Shu
```

Also: `engine.chineseCalendar(utc)`, `engine.kernels`, `engine.coverage`,
`engine.supports(jd)`, `chartDignities(chart, "dorothean")`, `degreeQualities(longitude)`,
`degreeQualityTable()` and `julianDay(utc)`. The [API guide](https://github.com/ffalcinelli/khagole-engine/blob/main/docs/api.md)
describes every option and result.

To ship a smaller kernel, cut the range you need (positions are unchanged inside it):

```js
import { excerptKernel } from "khagole-engine";
const small = excerptKernel(fullKernelBytes, 2415020.5, 2488069.5); // 1900-2100, ~21 MB
```

Errors are thrown as `Error` objects with a `code` (`ephemeris_out_of_range`,
`ephemeris_error`, `invalid_input`, `invalid_kernel`, `internal_error`).

JPL kernels: https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/ (`de440s.bsp` covers 1849-2150,
32 MB). Licensed under MIT OR Apache-2.0.
