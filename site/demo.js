// Live demo: astroceleste-engine compiled to WebAssembly, computing charts in the browser.
// Supports both Vedic (Jyotish Kundli D1/D9, Panchang, Vimshottari Dasha) and Western Wheel modes.
import init, * as wasm from "./pkg/astroceleste_engine_wasm.js";
import {
  renderNorthIndianChart,
  renderSouthIndianChart,
  renderPanchang,
  renderVedicPlanetsTable,
  renderVimshottariDasha,
  formatDegMin,
  formatDms,
} from "./vedic.js";

const { Engine, julianDay, bazi, calcDivisional, calcPanchang, calcVimshottariDasha } = wasm;

const KERNEL_URL = "ephemeris/de440s-1950-2050.bsp";
const KERNEL_NAME = "de440s-1950-2050.bsp";
const COVERAGE_TEXT = "1950–2050"; // keep in sync with scripts/build-site.sh

const SIGNS = [
  ["Aries", "♈", "fire"], ["Taurus", "♉", "earth"], ["Gemini", "♊", "air"],
  ["Cancer", "♋", "water"], ["Leo", "♌", "fire"], ["Virgo", "♍", "earth"],
  ["Libra", "♎", "air"], ["Scorpio", "♏", "water"], ["Sagittarius", "♐", "fire"],
  ["Capricorn", "♑", "earth"], ["Aquarius", "♒", "air"], ["Pisces", "♓", "water"],
];
const STEM_HANZI = { jia: "甲", yi: "乙", bing: "丙", ding: "丁", wu: "戊", ji: "己", geng: "庚", xin: "辛", ren: "壬", gui: "癸" };
const BRANCH_HANZI = { zi: "子", chou: "丑", yin: "寅", mao: "卯", chen: "辰", si: "巳", wu: "午", wei: "未", shen: "申", you: "酉", xu: "戌", hai: "亥" };
const ASPECT_CLASS = {
  Conjunction: "neutral", Sextile: "harmonious", Trine: "harmonious",
  Square: "hard", Opposition: "hard", "Semi-Sextile": "harmonious", Quincunx: "hard",
};

const CITY_PRESETS = {
  vadnagar: { lat: 23.784, lon: 72.636, tz: "+05:30" },
  delhi: { lat: 28.6139, lon: 77.209, tz: "+05:30" },
  mumbai: { lat: 19.076, lon: 72.8777, tz: "+05:30" },
  bengaluru: { lat: 12.9716, lon: 77.5946, tz: "+05:30" },
  ujjain: { lat: 23.1765, lon: 75.7885, tz: "+05:30" },
  greenwich: { lat: 51.4769, lon: 0, tz: "+00:00" },
};

const PRESETS = {
  modi: () => ({
    date: "1950-09-17",
    time: "09:34:00",
    tz_preset: "+05:30",
    latitude: 23.784,
    longitude: 72.636,
  }),
  delhi: () => {
    const now = new Date();
    const dStr = now.toISOString().slice(0, 10);
    const h = String(now.getHours()).padStart(2, "0");
    const m = String(now.getMinutes()).padStart(2, "0");
    const s = String(now.getSeconds()).padStart(2, "0");
    return {
      date: dStr,
      time: `${h}:${m}:${s}`,
      tz_preset: "+05:30",
      latitude: 28.6139,
      longitude: 77.209,
    };
  },
  now: () => {
    const iso = new Date().toISOString();
    return {
      date: iso.slice(0, 10),
      time: iso.slice(11, 19),
      tz_preset: "+00:00",
      latitude: 51.4769,
      longitude: 0,
    };
  },
  rome: () => ({
    date: "1987-05-17",
    time: "14:30:00",
    tz_preset: "+00:00",
    latitude: 41.9,
    longitude: 12.5,
  }),
  y2k: () => ({
    date: "2000-01-01",
    time: "12:00:00",
    tz_preset: "+00:00",
    latitude: 51.4769,
    longitude: 0,
  }),
};

const $ = (id) => document.getElementById(id);
const form = $("chart-form");
const statusEl = $("status");
const submit = $("compute-btn");

let currentMode = "vedic"; // "vedic" | "western"
let chartStyle = "north"; // "north" | "south"
let enginePromise;
let lastWesternJson = "";
let lastVedicJson = "";
let lastVedicData = null;

// Force text (not emoji) presentation of zodiac and planet glyphs.
const text = (glyph) => (/^[A-Z]+$/.test(glyph) ? glyph : `${glyph}\uFE0E`);
const escapeHtml = (s) =>
  String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
const dm = (p) => `${p.degree}°${String(p.minute).padStart(2, "0")}′`;

function setStatus(html, isError = false) {
  statusEl.innerHTML = html;
  statusEl.classList.toggle("error", isError);
}

async function fetchWithProgress(url) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`could not download the ephemeris (HTTP ${res.status})`);
  const total = Number(res.headers.get("content-length")) || 0;
  if (!res.body) return new Uint8Array(await res.arrayBuffer());
  const reader = res.body.getReader();
  const chunks = [];
  let received = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    received += value.length;
    const mb = (received / 1e6).toFixed(1);
    const pct = total ? Math.min(100, (received / total) * 100) : 0;
    setStatus(
      `Downloading JPL ephemeris excerpt… ${mb} MB` +
        (total ? ` of ${(total / 1e6).toFixed(1)} MB` : "") +
        `<span class="progress"><i style="width:${pct.toFixed(1)}%"></i></span>`,
    );
  }
  const bytes = new Uint8Array(received);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.length;
  }
  return bytes;
}

function loadEngine() {
  enginePromise ??= (async () => {
    setStatus("Loading calculation engine…");
    await init();
    const bytes = await fetchWithProgress(KERNEL_URL);
    const engine = new Engine();
    engine.addKernel(KERNEL_NAME, bytes);
    return engine;
  })().catch((err) => {
    enginePromise = undefined; // allow a retry
    throw err;
  });
  return enginePromise;
}

function getSelectedTzOffset() {
  const preset = form.elements.tz_preset.value;
  if (preset === "local") {
    const offsetMin = -new Date().getTimezoneOffset();
    const sign = offsetMin >= 0 ? "+" : "-";
    const absMin = Math.abs(offsetMin);
    const h = String(Math.floor(absMin / 60)).padStart(2, "0");
    const m = String(absMin % 60).padStart(2, "0");
    return `${sign}${h}:${m}`;
  }
  if (preset === "custom") {
    return form.elements.tz_custom.value.trim() || "+00:00";
  }
  return preset;
}

function localToUtc(dateStr, timeStr, tzOffsetStr) {
  const sign = tzOffsetStr.startsWith("-") ? -1 : 1;
  const parts = tzOffsetStr.replace(/^[+-]/, "").split(":");
  const offsetMinutes = sign * (parseInt(parts[0], 10) * 60 + parseInt(parts[1] || "0", 10));

  const [year, month, day] = dateStr.split("-").map(Number);
  const timeParts = timeStr.split(":").map(Number);
  const hour = timeParts[0] || 0;
  const minute = timeParts[1] || 0;
  const second = timeParts[2] || 0;

  const ms = Date.UTC(year, month - 1, day, hour, minute, second) - offsetMinutes * 60000;
  return new Date(ms).toISOString().slice(0, 19) + "Z";
}

function setAstrologyMode(mode) {
  currentMode = mode;
  $("tab-vedic").classList.toggle("active", mode === "vedic");
  $("tab-western").classList.toggle("active", mode === "western");
  $("tab-vedic").setAttribute("aria-selected", mode === "vedic");
  $("tab-western").setAttribute("aria-selected", mode === "western");

  for (const el of form.querySelectorAll(".vedic-field")) {
    el.hidden = mode !== "vedic";
  }
  for (const el of form.querySelectorAll(".western-field")) {
    el.hidden = mode !== "western";
  }

  submit.textContent = mode === "vedic" ? "Calculate Vedic Horoscope" : "Compute Western Chart";
  $("form-hint").textContent =
    mode === "vedic"
      ? "Time is in selected timezone: converted to UTC internally for Vedic ephemeris."
      : "Times are UTC: convert local time, including daylight saving time, first.";

  if (lastVedicData && mode === "vedic") {
    $("vedic-result").hidden = false;
    $("result").hidden = true;
  } else if (lastWesternJson && mode === "western") {
    $("vedic-result").hidden = true;
    $("result").hidden = false;
  }
}

function fillForm(values) {
  for (const [key, value] of Object.entries(values)) {
    if (form.elements[key] !== undefined) form.elements[key].value = value;
  }
  syncTzCustom();
  syncAyanamsa();
}

function syncTzCustom() {
  const isCustom = form.elements.tz_preset.value === "custom";
  $("tz-custom-wrap").hidden = !isCustom;
}

function syncAyanamsa() {
  if (form.elements.ayanamsa) {
    form.elements.ayanamsa.disabled = form.elements.zodiac_type.value !== "sidereal";
  }
}

async function compute() {
  if (!form.reportValidity()) return;
  const data = new FormData(form);
  const dateStr = data.get("date");
  const timeStr = data.get("time") || "00:00:00";
  const tzOffset = getSelectedTzOffset();
  const utc = localToUtc(dateStr, timeStr, tzOffset);

  submit.disabled = true;
  try {
    const engine = await loadEngine();
    const jd = julianDay(utc);
    if (!engine.supports(jd)) {
      setStatus(
        `This demo ships an ephemeris excerpt for ${COVERAGE_TEXT} only. The engine itself ` +
          `covers 1849–2150 with DE440s, and 13200 BC–17191 AD with DE441.`,
        true,
      );
      return;
    }

    const start = performance.now();

    if (currentMode === "vedic") {
      const vedicReq = {
        utc,
        latitude: Number(data.get("latitude")),
        longitude: Number(data.get("longitude")),
        house_system: data.get("vedic_house_system") || "W",
        zodiac_type: "sidereal",
        ayanamsa: data.get("vedic_ayanamsa") || "lahiri",
        position_mode: data.get("vedic_position_mode") || "true",
        node_type: data.get("vedic_node_type") || "mean",
      };

      const chart = engine.chart(vedicReq);
      const ascPoint = chart.planets.find((p) => p.name === "Ascendant") || chart.houses[0];
      const ascLon = ascPoint ? ascPoint.ecliptic_longitude : 0.0;

      const d1 = calcDivisional(chart, ascLon, "D1");
      const d9 = calcDivisional(chart, ascLon, "D9");

      const sun = chart.planets.find((p) => p.name === "Sun");
      const moon = chart.planets.find((p) => p.name === "Moon");
      const sunLon = sun ? sun.ecliptic_longitude : 0.0;
      const moonLon = moon ? moon.ecliptic_longitude : 0.0;

      const panchang = calcPanchang(sunLon, moonLon, jd);
      const yearType = data.get("vedic_year_type") || "tssy";
      const dasha = calcVimshottariDasha(moonLon, jd, 3, yearType);

      lastVedicData = { d1, d9, panchang, dasha, chart, moonLon, jd };
      renderVedicView(lastVedicData);

      const ms = performance.now() - start;
      const took = ms < 1 ? "under 1 ms" : ms < 10 ? `${ms.toFixed(1)} ms` : `${Math.round(ms)} ms`;
      setStatus(`Vedic chart &amp; Dasha computed in ${took}, on your device.`);
    } else {
      const westernReq = {
        utc,
        latitude: Number(data.get("latitude")),
        longitude: Number(data.get("longitude")),
        house_system: data.get("house_system"),
        zodiac_type: data.get("zodiac_type"),
        ayanamsa: form.elements.ayanamsa.value,
        dignity_scheme: data.get("dignity_scheme"),
        chinese_calendar: true,
      };

      const chart = engine.chart(westernReq);
      renderWesternView(chart, westernReq);

      const ms = performance.now() - start;
      const took = ms < 1 ? "under 1 ms" : ms < 10 ? `${ms.toFixed(1)} ms` : `${Math.round(ms)} ms`;
      setStatus(`Western chart computed in ${took}, on your device.`);
    }
  } catch (err) {
    const code = err && err.code ? ` (${err.code})` : "";
    setStatus(`${escapeHtml(err && err.message ? err.message : err)}${code}`, true);
  } finally {
    submit.disabled = false;
  }
}

// ---------------------------------------------------------------------------- Vedic Renderers

function renderVedicCharts(data) {
  if (!data) return;
  const svgD1 = $("chart-d1");
  const svgD9 = $("chart-d9");

  if (chartStyle === "north") {
    renderNorthIndianChart(svgD1, data.d1, "D1 · Rasi");
    renderNorthIndianChart(svgD9, data.d9, "D9 · Navamsha");
  } else {
    renderSouthIndianChart(svgD1, data.d1, "D1 · Rasi");
    renderSouthIndianChart(svgD9, data.d9, "D9 · Navamsha");
  }
}

function renderVedicView(data) {
  $("vedic-result").hidden = false;
  $("result").hidden = true;

  renderVedicCharts(data);
  renderPanchang($("vedic-panchang"), data.panchang);
  renderVedicPlanetsTable($("vedic-planets"), data.d1, data.d9, data.chart, wasm);
  renderVimshottariDasha($("vedic-dasha"), data.dasha, data.moonLon, data.jd, wasm);

  lastVedicJson = JSON.stringify(
    {
      d1: data.d1,
      d9: data.d9,
      panchang: data.panchang,
      dasha: data.dasha,
      planets: data.chart.planets,
    },
    null,
    2,
  );
  $("vedic-json").textContent = lastVedicJson;
}

// ---------------------------------------------------------------------------- Western Renderers

function renderWesternView(chart, request) {
  $("result").hidden = false;
  $("vedic-result").hidden = true;

  renderWheel(chart);
  renderSummary(chart, request);
  renderWesternTables(chart);
  lastWesternJson = JSON.stringify(chart, null, 2);
  $("json").textContent = lastWesternJson;
  $("wheel").setAttribute(
    "aria-label",
    `Chart wheel for ${request.utc} at latitude ${request.latitude}, longitude ${request.longitude}`,
  );
}

function renderSummary(chart, request) {
  const find = (name) => chart.planets.find((p) => p.name === name);
  const rows = [];
  const sun = find("Sun");
  const moon = find("Moon");
  const asc = find("Ascendant");
  if (sun) rows.push(["Sun", `${dm(sun)} ${sun.sign}`]);
  if (moon) {
    const phase = chart.lunar_status && chart.lunar_status.phase_name;
    rows.push([
      "Moon",
      `${dm(moon)} ${moon.sign}` +
        (phase ? `<br><small>${phase}, ${chart.lunar_status.illumination_percentage}% lit</small>` : ""),
    ]);
  }
  if (asc) rows.push(["Ascendant", `${dm(asc)} ${asc.sign}`]);
  if (chart.sect) {
    const hours = chart.planetary_hours;
    rows.push([
      "Sect",
      `${chart.sect === "diurnal" ? "Day" : "Night"} chart` +
        (hours ? `<br><small>day of ${hours.day_ruler}, hour of ${hours.hour_ruler}</small>` : ""),
    ]);
  }
  if (chart.temperament && chart.temperament.primary_temperament) {
    rows.push(["Temperament", `${chart.temperament.primary_temperament} – ${chart.temperament.secondary_temperament}`]);
  }
  if (chart.chinese_calendar) {
    const b = bazi(chart.chinese_calendar, request.utc, request.longitude, 0);
    const pillar = (p) => `${STEM_HANZI[p.stem]}${BRANCH_HANZI[p.branch]}`;
    const lunar = b.lunar_date;
    rows.push([
      "Four Pillars",
      `<span lang="zh">${[b.year, b.month, b.day, b.hour].map(pillar).join(" ")}</span>` +
        `<br><small>Day Master ${b.day_master} (${b.day_master_element})` +
        (lunar ? `, lunar year of the ${lunar.animal}` : "") +
        "</small>",
    ]);
  }
  rows.push([
    "Zodiac",
    chart.zodiac_type === "sidereal"
      ? `Sidereal, ${chart.ayanamsa_name}<br><small>ayanamsa ${chart.ayanamsa_formatted}</small>`
      : "Tropical",
  ]);
  $("summary").innerHTML = rows
    .map(([k, v]) => `<div><dt>${k}</dt><dd>${v}</dd></div>`)
    .join("");
}

function table(el, head, rows) {
  el.innerHTML =
    `<thead><tr>${head.map((h) => `<th>${h}</th>`).join("")}</tr></thead>` +
    `<tbody>${rows.map((r) => `<tr>${r.join("")}</tr>`).join("")}</tbody>`;
}

function renderWesternTables(chart) {
  const glyphOf = new Map(chart.planets.map((p) => [p.name, p.symbol]));
  const dignity = (p) => {
    const e = p.condition && p.condition.essential;
    if (!e) return "–";
    const held = ["domicile", "exaltation", "triplicity", "bound", "face", "detriment", "fall", "peregrine"]
      .filter((k) => e[k])
      .join(", ");
    return `<span title="${held}">${e.score > 0 ? "+" : ""}${e.score}</span>`;
  };
  table($("planets"), ["", "Point", "Position", "House", "Speed °/day", "Dignity"], chart.planets.map((p) => [
    `<td class="glyph">${escapeHtml(text(p.symbol))}</td>`,
    `<td>${escapeHtml(p.name)}${p.is_retrograde ? ' <span class="retro" title="retrograde">℞</span>' : ""}</td>`,
    `<td>${dm(p)} <span class="glyph">${text(p.sign_symbol)}</span> ${p.sign}</td>`,
    `<td class="num">${p.house}</td>`,
    `<td class="num">${p.speed ? p.speed.toFixed(3) : "–"}</td>`,
    `<td class="num">${dignity(p)}</td>`,
  ]));
  table($("houses"), ["House", "Cusp"], chart.houses.map((h) => [
    `<td>${h.house_number}</td>`,
    `<td>${dm(h)} <span class="glyph">${text(h.sign_symbol)}</span> ${h.sign}</td>`,
  ]));
  const label = (name) =>
    glyphOf.has(name) ? `<span class="glyph">${escapeHtml(text(glyphOf.get(name)))}</span> ${escapeHtml(name)}` : `★ ${escapeHtml(name)}`;
  table($("aspects"), ["", "Aspect", "", "Orb"], chart.aspects.map((a) => [
    `<td>${label(a.body1)}</td>`,
    `<td class="${ASPECT_CLASS[a.aspect_type] || ""}"><span class="glyph">${escapeHtml(text(a.symbol))}</span> ${escapeHtml(a.aspect_type)}</td>`,
    `<td>${label(a.body2)}</td>`,
    `<td class="num">${a.orb.toFixed(2)}°</td>`,
  ]));
}

const SVG = "http://www.w3.org/2000/svg";
const C = 200;
const R = { outer: 195, zodiac: 165, tick: 158, glyph: 141, houseNum: 106, inner: 95 };

function renderWheel(chart) {
  const svg = $("wheel");
  svg.replaceChildren();
  const byName = new Map(chart.planets.map((p) => [p.name, p]));
  const asc = (byName.get("Ascendant") || chart.houses[0]).ecliptic_longitude;
  const xy = (lon, r) => {
    const a = ((180 + lon - asc) * Math.PI) / 180;
    return [C + r * Math.cos(a), C - r * Math.sin(a)];
  };
  const add = (tag, attrs, content) => {
    const el = document.createElementNS(SVG, tag);
    for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
    if (content !== undefined) el.textContent = content;
    svg.append(el);
    return el;
  };
  const line = (lon1, r1, lon2, r2, cls) => {
    const [x1, y1] = xy(lon1, r1);
    const [x2, y2] = xy(lon2, r2);
    return add("line", { x1, y1, x2, y2, class: cls });
  };

  add("circle", { cx: C, cy: C, r: R.outer, class: "w-zodiac" });
  add("circle", { cx: C, cy: C, r: R.zodiac, class: "w-ring" });
  add("circle", { cx: C, cy: C, r: R.inner, class: "w-ring" });
  SIGNS.forEach(([name, glyph, element], i) => {
    line(i * 30, R.zodiac, i * 30, R.outer, "w-sign-line");
    const [x, y] = xy(i * 30 + 15, (R.zodiac + R.outer) / 2);
    add("text", { x, y, class: `w-sign ${element}` }, text(glyph)).append(
      Object.assign(document.createElementNS(SVG, "title"), { textContent: name }),
    );
  });

  chart.houses.forEach((h, i) => {
    const next = chart.houses[(i + 1) % 12].ecliptic_longitude;
    const axis = h.house_number === 1 || h.house_number === 4 || h.house_number === 7 || h.house_number === 10;
    line(h.ecliptic_longitude, R.inner, h.ecliptic_longitude, R.zodiac, axis ? "w-axis" : "w-cusp");
    const span = (next - h.ecliptic_longitude + 360) % 360;
    const [x, y] = xy(h.ecliptic_longitude + span / 2, R.houseNum);
    add("text", { x, y, class: "w-house-num" }, h.house_number);
  });

  const points = chart.planets.filter((p) => typeof p.ecliptic_longitude === "number");
  for (const a of chart.aspects) {
    const p1 = byName.get(a.body1);
    const p2 = byName.get(a.body2);
    const cls = ASPECT_CLASS[a.aspect_type];
    if (!p1 || !p2 || cls === "neutral" || !cls) continue;
    const minor = a.is_major === false ? " minor" : "";
    line(p1.ecliptic_longitude, R.inner, p2.ecliptic_longitude, R.inner, `w-aspect ${cls}${minor}`);
  }

  const shown = spread(points.map((p) => p.ecliptic_longitude), 8);
  points.forEach((p, i) => {
    line(p.ecliptic_longitude, R.zodiac, p.ecliptic_longitude, R.tick, "w-tick");
    line(p.ecliptic_longitude, R.tick, shown[i], R.glyph + 9, "w-leader");
    const [x, y] = xy(shown[i], R.glyph);
    const angle = /^[A-Z]+$/.test(p.symbol);
    const el = add("text", { x, y, class: `w-planet${angle ? " angle" : ""}` }, text(p.symbol));
    el.append(Object.assign(document.createElementNS(SVG, "title"), {
      textContent: `${p.name} ${dm(p)} ${p.sign}${p.is_retrograde ? " (retrograde)" : ""}`,
    }));
  });
}

function spread(lons, minSep) {
  const norm = (a) => ((a % 360) + 360) % 360;
  const pos = lons.slice();
  if (pos.length < 2) return pos;
  for (let iter = 0; iter < 200; iter++) {
    let moved = false;
    const order = [...pos.keys()].sort((a, b) => pos[a] - pos[b]);
    for (let k = 0; k < order.length; k++) {
      const a = order[k];
      const b = order[(k + 1) % order.length];
      const gap = norm(pos[b] - pos[a]);
      if (gap < minSep - 1e-6) {
        const push = (minSep - gap) / 2;
        pos[a] = norm(pos[a] - push);
        pos[b] = norm(pos[b] + push);
        moved = true;
      }
    }
    if (!moved) break;
  }
  return pos;
}

// ---------------------------------------------------------------------------- Wiring & Events

form.addEventListener("submit", (e) => {
  e.preventDefault();
  compute();
});

form.elements.tz_preset.addEventListener("change", syncTzCustom);
form.elements.zodiac_type?.addEventListener("change", syncAyanamsa);

$("tab-vedic").addEventListener("click", () => setAstrologyMode("vedic"));
$("tab-western").addEventListener("click", () => setAstrologyMode("western"));

$("style-north").addEventListener("click", () => {
  chartStyle = "north";
  $("style-north").classList.add("active");
  $("style-south").classList.remove("active");
  if (lastVedicData) renderVedicCharts(lastVedicData);
});

$("style-south").addEventListener("click", () => {
  chartStyle = "south";
  $("style-south").classList.add("active");
  $("style-north").classList.remove("active");
  if (lastVedicData) renderVedicCharts(lastVedicData);
});

for (const btn of form.querySelectorAll("[data-preset]")) {
  btn.addEventListener("click", () => {
    fillForm(PRESETS[btn.dataset.preset]());
    compute();
  });
}

for (const chip of form.querySelectorAll("[data-city]")) {
  chip.addEventListener("click", () => {
    const city = CITY_PRESETS[chip.dataset.city];
    if (city) {
      form.elements.latitude.value = city.lat;
      form.elements.longitude.value = city.lon;
      form.elements.tz_preset.value = city.tz;
      syncTzCustom();
    }
  });
}

$("copy-vedic-json").addEventListener("click", async (e) => {
  try {
    await navigator.clipboard.writeText(lastVedicJson);
    e.target.textContent = "Copied!";
  } catch {
    e.target.textContent = "Copy failed";
  }
  setTimeout(() => (e.target.textContent = "Copy"), 1500);
});

$("copy-json").addEventListener("click", async (e) => {
  try {
    await navigator.clipboard.writeText(lastWesternJson);
    e.target.textContent = "Copied!";
  } catch {
    e.target.textContent = "Copy failed";
  }
  setTimeout(() => (e.target.textContent = "Copy"), 1500);
});

// Initial setup
setAstrologyMode("vedic");
syncTzCustom();
syncAyanamsa();

// Auto-compute Narendra Modi reference preset on initial load for instant demonstration
fillForm(PRESETS.modi());
compute();
