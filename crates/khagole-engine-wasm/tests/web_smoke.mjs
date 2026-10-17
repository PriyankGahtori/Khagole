// Smoke test of the browser build (`--target web`), the one published to npm, run with Node:
//   wasm-pack build --release --target web crates/khagole-engine-wasm
//   node crates/khagole-engine-wasm/tests/web_smoke.mjs
// golden.mjs covers the API in depth on the Node build; this checks that the published
// package loads and computes.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "../pkg");
const { initSync, Engine, julianDay, degreeQualities, calcDivisional, calcDivisionalPoint, calcVimshottariDasha, calcPanchang, nakshatraInfo } = await import(join(pkg, "khagole_engine_wasm.js"));
initSync({ module: readFileSync(join(pkg, "khagole_engine_wasm_bg.wasm")) });

const engine = new Engine();
engine.addKernel("de440s_2000.bsp", readFileSync(join(here, "../../../tests/data/de440s_2000.bsp")));
const chart = engine.chart({ utc: "2000-06-01T12:00:00Z", latitude: 41.9, longitude: 12.5 });
assert.equal(chart.planets[0].name, "Sun");
assert.equal(chart.planets[0].sign, "Taurus"); // Vedic Sidereal (default)
assert.equal(chart.houses.length, 12);
assert.equal(julianDay("2000-06-01T12:00:00Z"), 2451697);
assert.equal(degreeQualities(5.5).sign, "Aries");
assert.throws(() => engine.chart({ utc: "2010-01-01T00:00:00Z", latitude: 0, longitude: 0 }), (e) => e.code === "ephemeris_out_of_range");

// Vedic Tier B tests
const d9Pt = calcDivisionalPoint(0.0, "D9");
assert.equal(d9Pt.sign, 1);

const d9Chart = calcDivisional(chart, undefined, "D9");
assert.equal(d9Chart.varga, "D9");
assert.equal(d9Chart.planets.length > 0, true);

const dasha = calcVimshottariDasha(0.0, 2451545.0, 3, "sidereal");
assert.equal(dasha.length, 9);
assert.equal(dasha[0].planet, "Ketu");
assert.equal(dasha[0].children.length, 9);

const panchang = calcPanchang(10.0, 10.0, 2451545.0, 6);
assert.equal(panchang.tithi.number, 1);
assert.equal(panchang.tithi.paksha, "Shukla");
assert.equal(panchang.vara.name, "Saturday");

const nak = nakshatraInfo(0.0);
assert.equal(nak.name, "Ashwini");
assert.equal(nak.lord, "Ketu");
assert.equal(nak.pada, 1);

// Engine method calls
const engD9 = engine.calc_divisional(chart, "D9");
assert.equal(engD9.varga, "D9");

console.log("web package: ok (including Vedic Tier B methods)");

