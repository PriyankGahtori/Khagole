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
  vadnagar: { name: "Vadnagar, Gujarat, India", lat: 23.784, lon: 72.636, tz: "+05:30", iana: "Asia/Kolkata" },
  delhi: { name: "New Delhi, Delhi, India", lat: 28.6139, lon: 77.209, tz: "+05:30", iana: "Asia/Kolkata" },
  mumbai: { name: "Mumbai, Maharashtra, India", lat: 19.076, lon: 72.8777, tz: "+05:30", iana: "Asia/Kolkata" },
  bengaluru: { name: "Bengaluru, Karnataka, India", lat: 12.9716, lon: 77.5946, tz: "+05:30", iana: "Asia/Kolkata" },
  chennai: { name: "Chennai, Tamil Nadu, India", lat: 13.0827, lon: 80.2707, tz: "+05:30", iana: "Asia/Kolkata" },
  kolkata: { name: "Kolkata, West Bengal, India", lat: 22.5726, lon: 88.3639, tz: "+05:30", iana: "Asia/Kolkata" },
  ujjain: { name: "Ujjain, Madhya Pradesh, India", lat: 23.1765, lon: 75.7885, tz: "+05:30", iana: "Asia/Kolkata" },
  varanasi: { name: "Varanasi, Uttar Pradesh, India", lat: 25.3176, lon: 82.9739, tz: "+05:30", iana: "Asia/Kolkata" },
  london: { name: "London, England, UK", lat: 51.5074, lon: -0.1278, tz: "+00:00", iana: "Europe/London" },
  newyork: { name: "New York, USA", lat: 40.7128, lon: -74.006, tz: "-05:00", iana: "America/New_York" },
  tokyo: { name: "Tokyo, Japan", lat: 35.6762, lon: 139.6503, tz: "+09:00", iana: "Asia/Tokyo" },
  greenwich: { name: "Greenwich, London, UK", lat: 51.4769, lon: 0, tz: "+00:00", iana: "UTC" },
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

function formatCoords(lat, lon) {
  const latDir = lat >= 0 ? "N" : "S";
  const lonDir = lon >= 0 ? "E" : "W";
  return `${Math.abs(lat).toFixed(4)}° ${latDir}, ${Math.abs(lon).toFixed(4)}° ${lonDir}`;
}

export function resolveTzOffset(ianaTimezone, dateStr, timeStr) {
  if (!ianaTimezone) return "+00:00";
  try {
    const [y, m, d] = (dateStr || "2000-01-01").split("-").map(Number);
    const [hh, mm, ss] = (timeStr || "12:00:00").split(":").map(Number);
    const date = new Date(Date.UTC(y, (m || 1) - 1, d || 1, hh || 12, mm || 0, ss || 0));
    const formatter = new Intl.DateTimeFormat("en-US", {
      timeZone: ianaTimezone,
      timeZoneName: "shortOffset",
    });
    const parts = formatter.formatToParts(date);
    const tzPart = parts.find((p) => p.type === "timeZoneName");
    if (!tzPart || !tzPart.value) return "+00:00";
    const val = tzPart.value.replace(/^(GMT|UTC)/, "");
    if (!val) return "+00:00";
    const sign = val.startsWith("-") ? "-" : "+";
    const clean = val.replace(/^[+-]/, "");
    const [hoursStr, minsStr] = clean.split(":");
    const hours = String(parseInt(hoursStr, 10) || 0).padStart(2, "0");
    const mins = String(parseInt(minsStr, 10) || 0).padStart(2, "0");
    return `${sign}${hours}:${mins}`;
  } catch (err) {
    console.warn("Timezone resolution failed:", err);
    return "+00:00";
  }
}

function updateActiveLocationDisplay(name, lat, lon, tzOffset) {
  const nameEl = $("location-name-display");
  const coordsEl = $("location-coords-display");
  if (nameEl) nameEl.textContent = name;
  if (coordsEl) {
    const tzStr = tzOffset ? ` · UTC${tzOffset.startsWith("+") || tzOffset.startsWith("-") ? tzOffset : "+" + tzOffset}` : "";
    coordsEl.textContent = `${formatCoords(lat, lon)}${tzStr}`;
  }
}

let activeCityIana = "Asia/Kolkata";

function applyLocation({ displayName, lat, lon, timezone, tzOffset }) {
  form.elements.latitude.value = lat;
  form.elements.longitude.value = lon;
  if (timezone) activeCityIana = timezone;

  let offset = tzOffset;
  if (!offset && activeCityIana) {
    const dStr = form.elements.date.value;
    const tStr = form.elements.time.value;
    offset = resolveTzOffset(activeCityIana, dStr, tStr);
  }
  if (!offset) {
    offset = "+05:30";
  }

  let presetFound = false;
  for (const opt of form.elements.tz_preset.options) {
    if (opt.value === offset) {
      form.elements.tz_preset.value = offset;
      presetFound = true;
      break;
    }
  }
  if (!presetFound) {
    form.elements.tz_preset.value = "custom";
    form.elements.tz_custom.value = offset;
  }
  syncTzCustom();

  updateActiveLocationDisplay(displayName, lat, lon, offset);
}

async function searchCitiesOnline(query) {
  const q = query.trim();
  if (q.length < 2) return [];

  // 1. Primary: Open-Meteo free geocoding API (fast, CORS-enabled, returns IANA timezone)
  try {
    const url = `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(q)}&count=8&language=en&format=json`;
    const res = await fetch(url);
    if (res.ok) {
      const data = await res.json();
      if (data.results && data.results.length > 0) {
        return data.results.map((r) => {
          const parts = [r.name];
          if (r.admin1 && r.admin1 !== r.name) parts.push(r.admin1);
          if (r.country) parts.push(r.country);
          return {
            name: r.name,
            displayName: parts.join(", "),
            admin: [r.admin1, r.country].filter(Boolean).join(", "),
            countryCode: r.country_code || "",
            lat: Number(r.latitude.toFixed(4)),
            lon: Number(r.longitude.toFixed(4)),
            timezone: r.timezone || null,
          };
        });
      }
    }
  } catch (err) {
    console.warn("Open-Meteo geocoding failed, trying Nominatim fallback:", err);
  }

  // 2. Fallback: OpenStreetMap Nominatim API
  try {
    const url = `https://nominatim.openstreetmap.org/search?q=${encodeURIComponent(q)}&format=json&limit=8&addressdetails=1`;
    const res = await fetch(url, { headers: { "Accept-Language": "en" } });
    if (res.ok) {
      const data = await res.json();
      return data.map((item) => {
        const addr = item.address || {};
        const city = addr.city || addr.town || addr.village || item.name || q;
        const adminParts = [addr.state || addr.region, addr.country].filter(Boolean);
        return {
          name: city,
          displayName: item.display_name,
          admin: adminParts.join(", "),
          countryCode: addr.country_code ? addr.country_code.toUpperCase() : "",
          lat: Number(parseFloat(item.lat).toFixed(4)),
          lon: Number(parseFloat(item.lon).toFixed(4)),
          timezone: null,
        };
      });
    }
  } catch (err) {
    console.warn("Nominatim geocoding failed:", err);
  }

  return [];
}

function fillForm(values) {
  for (const [key, value] of Object.entries(values)) {
    if (form.elements[key] !== undefined) form.elements[key].value = value;
  }
  syncTzCustom();
  syncAyanamsa();

  if (values.latitude !== undefined && values.longitude !== undefined) {
    const lat = Number(values.latitude);
    const lon = Number(values.longitude);
    const tz = getSelectedTzOffset();
    const match = Object.values(CITY_PRESETS).find(
      (c) => Math.abs(c.lat - lat) < 0.05 && Math.abs(c.lon - lon) < 0.05
    );
    const name = match ? match.name : `Coordinates (${formatCoords(lat, lon)})`;
    if (match && match.iana) activeCityIana = match.iana;
    updateActiveLocationDisplay(name, lat, lon, tz);
  }
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

// ---------------------------------------------------------------------------- City Search Autocomplete & Quick Chips Wiring
const citySearchInput = $("city-search-input");
const citySearchClear = $("city-search-clear");
const cityDropdown = $("city-search-dropdown");

let searchDebounceTimer = null;
let searchResults = [];
let highlightedIndex = -1;

function hideCityDropdown() {
  if (cityDropdown) {
    cityDropdown.hidden = true;
    cityDropdown.innerHTML = "";
  }
  highlightedIndex = -1;
  searchResults = [];
}

function renderCityDropdown(results) {
  if (!cityDropdown) return;
  searchResults = results;
  highlightedIndex = -1;

  if (results.length === 0) {
    cityDropdown.innerHTML = `<div class="city-dropdown-msg">No cities found matching your search.</div>`;
    cityDropdown.hidden = false;
    return;
  }

  cityDropdown.innerHTML = results
    .map(
      (r, idx) => `
    <div class="city-item" data-idx="${idx}">
      <div class="city-item-main">
        <div class="city-item-name">
          <span>📍</span> <strong>${escapeHtml(r.name)}</strong>
          ${r.countryCode ? `<span class="city-badge">${escapeHtml(r.countryCode)}</span>` : ""}
        </div>
        <div class="city-item-admin">${escapeHtml(r.admin || r.displayName)}</div>
      </div>
      <div class="city-item-meta">
        <div>${formatCoords(r.lat, r.lon)}</div>
        ${r.timezone ? `<span class="city-item-tz">${escapeHtml(r.timezone)}</span>` : ""}
      </div>
    </div>
  `,
    )
    .join("");

  cityDropdown.hidden = false;

  for (const itemEl of cityDropdown.querySelectorAll(".city-item")) {
    itemEl.addEventListener("click", () => {
      const idx = Number(itemEl.dataset.idx);
      const chosen = searchResults[idx];
      if (chosen) {
        selectCityResult(chosen);
      }
    });
  }
}

function selectCityResult(city) {
  applyLocation({
    displayName: city.displayName,
    lat: city.lat,
    lon: city.lon,
    timezone: city.timezone,
    tzOffset: null,
  });

  if (citySearchInput) {
    citySearchInput.value = city.name;
    if (citySearchClear) citySearchClear.hidden = false;
  }
  hideCityDropdown();

  // Highlight popular chip if it matches
  document.querySelectorAll(".btn-city-chip").forEach((b) => {
    const cData = CITY_PRESETS[b.dataset.city];
    const isMatch = cData && Math.abs(cData.lat - city.lat) < 0.05 && Math.abs(cData.lon - city.lon) < 0.05;
    b.classList.toggle("active", isMatch);
  });

  compute();
}

if (citySearchInput) {
  citySearchInput.addEventListener("input", (e) => {
    const val = e.target.value.trim();
    if (citySearchClear) citySearchClear.hidden = val.length === 0;

    clearTimeout(searchDebounceTimer);
    if (val.length < 2) {
      hideCityDropdown();
      return;
    }

    if (cityDropdown) {
      cityDropdown.innerHTML = `<div class="city-dropdown-msg">Searching cities worldwide…</div>`;
      cityDropdown.hidden = false;
    }

    searchDebounceTimer = setTimeout(async () => {
      const results = await searchCitiesOnline(val);
      if (citySearchInput.value.trim() === val) {
        renderCityDropdown(results);
      }
    }, 250);
  });

  citySearchInput.addEventListener("keydown", (e) => {
    if (!cityDropdown || cityDropdown.hidden || searchResults.length === 0) return;

    const items = cityDropdown.querySelectorAll(".city-item");
    if (e.key === "ArrowDown") {
      e.preventDefault();
      highlightedIndex = (highlightedIndex + 1) % items.length;
      items.forEach((it, idx) => it.classList.toggle("selected", idx === highlightedIndex));
      items[highlightedIndex]?.scrollIntoView({ block: "nearest" });
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      highlightedIndex = (highlightedIndex - 1 + items.length) % items.length;
      items.forEach((it, idx) => it.classList.toggle("selected", idx === highlightedIndex));
      items[highlightedIndex]?.scrollIntoView({ block: "nearest" });
    } else if (e.key === "Enter") {
      if (highlightedIndex >= 0 && highlightedIndex < searchResults.length) {
        e.preventDefault();
        selectCityResult(searchResults[highlightedIndex]);
      }
    } else if (e.key === "Escape") {
      hideCityDropdown();
    }
  });
}

if (citySearchClear) {
  citySearchClear.addEventListener("click", () => {
    if (citySearchInput) {
      citySearchInput.value = "";
      citySearchInput.focus();
    }
    citySearchClear.hidden = true;
    hideCityDropdown();
  });
}

document.addEventListener("click", (e) => {
  if (cityDropdown && !cityDropdown.contains(e.target) && e.target !== citySearchInput) {
    hideCityDropdown();
  }
});

// Popular city chips listener
for (const chip of document.querySelectorAll(".btn-city-chip")) {
  chip.addEventListener("click", () => {
    document.querySelectorAll(".btn-city-chip").forEach((b) => b.classList.remove("active"));
    chip.classList.add("active");
    const city = CITY_PRESETS[chip.dataset.city];
    if (city) {
      applyLocation({
        displayName: city.name,
        lat: city.lat,
        lon: city.lon,
        timezone: city.iana,
        tzOffset: city.tz,
      });
      if (citySearchInput) {
        citySearchInput.value = "";
        if (citySearchClear) citySearchClear.hidden = true;
      }
      hideCityDropdown();
      compute();
    }
  });
}

// Preset chips (Modi, Delhi, Now, Rome, Y2K)
for (const btn of form.querySelectorAll("[data-preset]")) {
  btn.addEventListener("click", () => {
    form.querySelectorAll("[data-preset]").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
    const pName = btn.dataset.preset;
    fillForm(PRESETS[pName]());

    // Sync city chip active state
    document.querySelectorAll(".btn-city-chip").forEach((chip) => {
      if (pName === "modi" && chip.dataset.city === "vadnagar") chip.classList.add("active");
      else if (pName === "delhi" && chip.dataset.city === "delhi") chip.classList.add("active");
      else chip.classList.remove("active");
    });

    if (citySearchInput) {
      citySearchInput.value = "";
      if (citySearchClear) citySearchClear.hidden = true;
    }
    hideCityDropdown();

    compute();
  });
}

// Re-calculate timezone if user edits birth date or time while a city is selected
form.elements.date.addEventListener("change", () => {
  if (activeCityIana && form.elements.tz_preset.value !== "custom") {
    const newOffset = resolveTzOffset(activeCityIana, form.elements.date.value, form.elements.time.value);
    applyLocation({
      displayName: $("location-name-display")?.textContent || "Custom Location",
      lat: Number(form.elements.latitude.value),
      lon: Number(form.elements.longitude.value),
      timezone: activeCityIana,
      tzOffset: newOffset,
    });
  }
});

form.elements.time.addEventListener("change", () => {
  if (activeCityIana && form.elements.tz_preset.value !== "custom") {
    const newOffset = resolveTzOffset(activeCityIana, form.elements.date.value, form.elements.time.value);
    applyLocation({
      displayName: $("location-name-display")?.textContent || "Custom Location",
      lat: Number(form.elements.latitude.value),
      lon: Number(form.elements.longitude.value),
      timezone: activeCityIana,
      tzOffset: newOffset,
    });
  }
});

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
