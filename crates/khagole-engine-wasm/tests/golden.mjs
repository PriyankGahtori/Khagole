// Golden charts through the WASM build, run with Node:
//   wasm-pack build --target nodejs --out-dir pkg-node crates/khagole-engine-wasm
//   node crates/khagole-engine-wasm/tests/golden.mjs
// Needs kernels/de440s.bsp (scripts/fetch-kernels.sh); otherwise only the excerpt checks run.
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "../../..");
const require = createRequire(import.meta.url);
const { Engine, synastry, derivedChart, julianDay, excerptKernel, degreeQualities, degreeQualityTable, timeLords, chartDignities, bazi, ziWei } = require(join(here, "../pkg-node/khagole_engine_wasm.js"));

const PRIVATE = new Set(["degree_symbol", "degree_symbols", "condition"]);
// Chart keys added after the reference implementation, absent from the fixtures.
const ADDED = new Set(["sect", "dignity_scheme", "receptions", "antiscia", "planetary_hours", "position_mode"]);
const stripNested = (v) =>
  Array.isArray(v) ? v.map(stripNested)
  : v && typeof v === "object" ? Object.fromEntries(Object.entries(v).filter(([k]) => !PRIVATE.has(k)).map(([k, x]) => [k, stripNested(x)]))
  : v;
// Horary considerations added after the reference implementation.
const ADDED_STRICTURES = new Set(["MOON_VIA_COMBUSTA", "MOON_LATE_DEGREES", "ASC_RULER_COMBUST"]);
const stripHorary = (h) => {
  const { judgment, ...rest } = h;
  return { ...rest, strictures: (rest.strictures || []).filter((s) => !ADDED_STRICTURES.has(s.code)) };
};
const strip = (v) => {
  if (!v || typeof v !== "object" || Array.isArray(v)) return stripNested(v);
  const top = Object.fromEntries(Object.entries(v).filter(([k]) => !ADDED.has(k)));
  if (top.horary_data) top.horary_data = stripHorary(top.horary_data);
  return stripNested(top);
};

function same(actual, expected, path, tol = 1e-9) {
  if (Array.isArray(expected)) {
    assert.equal(actual.length, expected.length, `${path}: length`);
    expected.forEach((e, i) => same(actual[i], e, `${path}[${i}]`, tol));
  } else if (expected && typeof expected === "object") {
    assert.deepEqual(Object.keys(actual), Object.keys(expected), `${path}: keys`);
    for (const k of Object.keys(expected)) same(actual[k], expected[k], `${path}.${k}`, tol);
  } else if (typeof expected === "number") {
    assert.ok(Math.abs(actual - expected) <= tol * Math.max(1, Math.abs(expected)), `${path}: ${actual} != ${expected}`);
  } else {
    assert.equal(actual, expected, path);
  }
}

const fixture = (name) => JSON.parse(readFileSync(join(root, "tests/fixtures", name), "utf8"));
const request = (i) => ({ utc: i.utc, latitude: i.lat, longitude: i.lon, house_system: i.house_system, zodiac_type: i.zodiac_type, ayanamsa: i.ayanamsa, position_mode: i.position_mode || "apparent" });

// Excerpt (always available): API behaviour.
const small = new Engine();
small.addKernel("de440s_2000.bsp", readFileSync(join(root, "tests/data/de440s_2000.bsp")));
assert.ok(small.supports(2451600) && !small.supports(2460000));
const chart = small.chart({ utc: "2000-06-01T12:00:00Z", latitude: 41.9, longitude: 12.5 });
assert.equal(chart.planets[0].name, "Sun");
assert.equal(chart.houses.length, 12);
assert.equal(julianDay("2000-06-01T12:00:00Z"), 2451697);
assert.throws(() => small.chart({ utc: "2010-01-01T00:00:00Z", latitude: 0, longitude: 0 }), (e) => e.code === "ephemeris_out_of_range");
assert.throws(() => small.chart({ utc: "nope", latitude: 0, longitude: 0 }), (e) => e.code === "invalid_input");
// An excerpt of the excerpt computes the same charts inside its range.
const march = excerptKernel(readFileSync(join(root, "tests/data/de440s_2000.bsp")), 2451604.5, 2451696.5);
const tiny = new Engine();
tiny.addKernel("march.bsp", march);
const req = { utc: "2000-04-01T12:00:00Z", latitude: 41.9, longitude: 12.5 };
assert.deepEqual(tiny.chart(req), small.chart(req));
assert.ok(!tiny.supports(2451800));
// Elections: a chart with its assessment, and a search whose best moments score exactly.
const rome = { latitude: 41.9, longitude: 12.5 };
const elected = small.election({ utc: "2000-06-01T12:00:00Z", ...rome }, { purpose: "contract" });
assert.equal(elected.election_data.purpose, "contract");
assert.ok(elected.planets.length > 0 && elected.election_data.factors.length > 0);
const found = small.elections({ utc: "2000-06-01T00:00:00Z", ...rome }, "2000-06-08T00:00:00Z", { step_minutes: 30 });
assert.ok(found.evaluated > 0 && found.criteria.step_minutes === 30);
for (const w of found.windows) {
  assert.equal(small.election({ utc: w.best, ...rome }).election_data.score, w.score);
}
assert.throws(() => small.elections({ utc: "2000-01-01T00:00:00Z", ...rome }, "2000-06-01T00:00:00Z"), (e) => e.code === "invalid_input");
assert.throws(() => small.election({ utc: "2000-06-01T12:00:00Z", ...rome }, { purpose: "war" }), (e) => e.code === "invalid_input");
// Kernels, the Chinese calendar (the excerpt is too short for one) and input checks.
assert.deepStrictEqual(small.kernels.map((k) => k[0]), ["de440s_2000.bsp"]);
assert.deepStrictEqual(small.kernels[0].slice(1), [...small.coverage]);
assert.strictEqual(small.chineseCalendar("2000-06-01T12:00:00Z"), null);
const at = { utc: "2000-06-01T12:00:00Z", ...rome };
for (const call of [
  () => small.chart({ ...at, latitude: 91 }),
  () => small.chart({ ...at, utc: "2000-02-30T12:00:00Z" }),
  () => small.chart({ ...at, orb_settings: { fixed_star_orb: "inf" } }),
  () => small.horary({ ...at, quesited_house: 13 }),
  () => small.chineseCalendar("2000-13-01T00:00:00Z"),
  () => timeLords("1990-01-01T00:00:00Z", 280, 130, "1990-01-01T00:00:00Z", "3500-01-01T00:00:00Z"),
]) {
  assert.throws(call, (e) => e.code === "invalid_input");
}
assert.strictEqual(small.horary({ ...at, quesited_house: 7 }).horary_data.judgment.quesited_house, 7);
assert.throws(() => new Engine().addKernel("garbage.bsp", new Uint8Array(2048)), (e) => e.code === "invalid_kernel");
// No kernel needed.
const q = degreeQualities(5.5);
assert.deepStrictEqual([q.sign, q.degree, q.pitted, q.light], ["Aries", 6, true, "light"]);
assert.strictEqual(degreeQualityTable().length, 12);
const lords = timeLords("1990-01-01T12:00:00Z", 280, 130, "2000-06-01T00:00:00Z", "2001-06-01T00:00:00Z");
assert.deepStrictEqual(lords.profections.map((p) => [p.age, p.sign, p.lord]), [[10, "Gemini", "Mercury"], [11, "Cancer", "Moon"]]);
assert.strictEqual(lords.firdaria[0].lord, "Saturn");
{
  const judged = chartDignities(chart, "dorothean");
  assert.strictEqual(judged.dignity_scheme, "dorothean");
  assert.strictEqual(judged.conditions.length, 7);
  assert.strictEqual(judged.conditions[0].condition.essential.lords.triplicity.length, 3);
}
console.log("excerpt checks: ok");

const full = join(root, "kernels/de440s.bsp");
if (!existsSync(full)) {
  if (process.env.KHAGOLE_REQUIRE_KERNEL || process.env.ASTROCELESTE_REQUIRE_KERNEL) {
    console.error("kernels/de440s.bsp not found and KHAGOLE_REQUIRE_KERNEL is set");
    process.exit(1);
  }
  console.log("golden checks skipped: kernels/de440s.bsp not found");
  process.exit(0);
}
const engine = new Engine();
engine.addKernel("de440s.bsp", readFileSync(full));
const natal = Object.fromEntries(fixture("natal.json").map((c) => [c.input.id, c.output]));
let count = 0;
for (const c of fixture("natal.json")) { same(strip(engine.chart(request(c.input))), c.output, c.input.id); count++; }
for (const c of fixture("horary.json")) { same(strip(engine.horary(request(c.input))), c.output, c.input.id); count++; }
for (const c of fixture("transits.json")) {
  same(strip(engine.transit(natal[c.input.natal].planets, request(c.input.transit))), c.output, c.input.transit.id); count++;
}
for (const c of fixture("synastry.json")) {
  same(synastry(natal[c.input.a].planets, natal[c.input.b].planets), c.output, c.input.a); count++;
}
for (const c of fixture("derived.json")) {
  same(strip(derivedChart(natal[c.input.base], c.input.root_house)), c.output, c.input.base, 0); count++;
}
{
  // Bruce Lee, San Francisco, 1940-11-27 07:12 PST.
  const birth = "1940-11-27T15:12:00Z";
  assert.ok(!("chinese_calendar" in engine.chart({ utc: birth, latitude: 37.77, longitude: -122.42 })));
  const { chinese_calendar: calendar } = engine.chart({ utc: birth, latitude: 37.77, longitude: -122.42, chinese_calendar: true });
  assert.deepStrictEqual(engine.chineseCalendar(birth), calendar);
  const pillars = bazi(calendar, birth, -122.42, -480, { sex: "male" });
  assert.deepStrictEqual(["year", "month", "day", "hour"].map((p) => `${pillars[p].stem} ${pillars[p].branch}`), ["geng chen", "ding hai", "jia xu", "wu chen"]);
  assert.strictEqual(pillars.luck.direction, "forward");
  assert.strictEqual(bazi(calendar, birth, -122.42, -480).luck, null);
  assert.strictEqual(bazi(calendar, birth, -122.42, -480, { solar_time: false }).time_basis, "civil");
  assert.throws(() => bazi(calendar, birth, -122.42, -480, { zi_hour: "noon" }), (e) => e.code === "invalid_input");
  const purple = ziWei(calendar, birth, -122.42, -480, { solar_time: false, sex: "male" });
  assert.deepStrictEqual([purple.life_palace, purple.body_palace, purple.bureau.number], ["wei", "mao", 3]);
  assert.strictEqual(purple.palaces.reduce((n, p) => n + p.stars.length, 0), 28);
  assert.strictEqual(ziWei(calendar, birth, -122.42, -480).decade_direction, null);
  assert.strictEqual(ziWei(calendar, birth, -122.42, -480, { sex: "male", year: 2026 }).horoscope.yearly.branch, "wu");
  assert.ok(ziWei(calendar, birth, -122.42, -480, { date: "2026-03-15" }).horoscope.daily);
  assert.throws(() => ziWei(calendar, birth, -122.42, -480, { date: "1800-01-01" }), (e) => e.code === "invalid_input");
  assert.strictEqual(bazi(calendar, birth, -122.42, -480, { year: 2026 }).annual.stem, "bing");
  assert.strictEqual(bazi(calendar, birth, -122.42, -480, { date: "2026-03-15" }).flow.month.branch, "mao");
  assert.throws(() => ziWei(calendar, birth, -122.42, -480, { leap_month: "never" }), (e) => e.code === "invalid_input");
}
console.log(`golden checks: ${count} charts ok`);
