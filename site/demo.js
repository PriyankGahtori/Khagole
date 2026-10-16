// Live demo: astroceleste-engine compiled to WebAssembly, computing Vedic Jyotish charts in the browser.
import init, * as wasm from "./pkg/astroceleste_engine_wasm.js?v=2.3.0";
import {
  renderNorthIndianChart,
  renderSouthIndianChart,
  renderPanchang,
  renderVedicPlanetsTable,
  renderVimshottariDasha,
  formatDegMin,
  formatDms,
  VARGA_LIST,
} from "./vedic.js?v=2.3.0";

const { Engine, julianDay, calcDivisional, calcPanchang, calcVimshottariDasha } = wasm;

const KERNEL_URL = "ephemeris/de440s-1950-2050.bsp";
const KERNEL_NAME = "de440s-1950-2050.bsp";
const COVERAGE_TEXT = "1950–2050"; // keep in sync with scripts/build-site.sh

const CITY_PRESETS = {
  delhi: { name: "New Delhi, Delhi, India", lat: 28.6139, lon: 77.2090, tz: "+05:30", iana: "Asia/Kolkata" },
  mumbai: { name: "Mumbai, Maharashtra, India", lat: 19.0760, lon: 72.8777, tz: "+05:30", iana: "Asia/Kolkata" },
  bengaluru: { name: "Bengaluru, Karnataka, India", lat: 12.9716, lon: 77.5946, tz: "+05:30", iana: "Asia/Kolkata" },
  ujjain: { name: "Ujjain, Madhya Pradesh, India", lat: 23.1765, lon: 75.7885, tz: "+05:30", iana: "Asia/Kolkata" },
  varanasi: { name: "Varanasi, Uttar Pradesh, India", lat: 25.3176, lon: 82.9739, tz: "+05:30", iana: "Asia/Kolkata" },
  vadnagar: { name: "Vadnagar, Gujarat, India", lat: 23.7840, lon: 72.6360, tz: "+05:30", iana: "Asia/Kolkata" },
  chennai: { name: "Chennai, Tamil Nadu, India", lat: 13.0827, lon: 80.2707, tz: "+05:30", iana: "Asia/Kolkata" },
  kolkata: { name: "Kolkata, West Bengal, India", lat: 22.5726, lon: 88.3639, tz: "+05:30", iana: "Asia/Kolkata" },
  london: { name: "London, England, UK", lat: 51.5074, lon: -0.1278, tz: "+00:00", iana: "Europe/London" },
  newyork: { name: "New York, USA", lat: 40.7128, lon: -74.0060, tz: "-05:00", iana: "America/New_York" },
  tokyo: { name: "Tokyo, Japan", lat: 35.6762, lon: 139.6503, tz: "+09:00", iana: "Asia/Tokyo" },
  greenwich: { name: "Greenwich, London, UK", lat: 51.4769, lon: 0.0000, tz: "+00:00", iana: "UTC" },
};

// Comprehensive offline index of top Vedic pilgrimage sites, Indian capitals & world metros
const LOCAL_CITIES = [
  // India - Metros & Major Centers
  { name: "New Delhi", admin: "Delhi, India", lat: 28.6139, lon: 77.2090, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Delhi", admin: "Delhi, India", lat: 28.6519, lon: 77.2315, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Mumbai", admin: "Maharashtra, India", lat: 19.0760, lon: 72.8777, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Bengaluru", admin: "Karnataka, India", lat: 12.9716, lon: 77.5946, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Hyderabad", admin: "Telangana, India", lat: 17.3850, lon: 78.4867, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Ahmedabad", admin: "Gujarat, India", lat: 23.0225, lon: 72.5714, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Chennai", admin: "Tamil Nadu, India", lat: 13.0827, lon: 80.2707, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Kolkata", admin: "West Bengal, India", lat: 22.5726, lon: 88.3639, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Pune", admin: "Maharashtra, India", lat: 18.5204, lon: 73.8567, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Jaipur", admin: "Rajasthan, India", lat: 26.9124, lon: 75.7873, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Lucknow", admin: "Uttar Pradesh, India", lat: 26.8467, lon: 80.9462, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Kanpur", admin: "Uttar Pradesh, India", lat: 26.4499, lon: 80.3319, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Nagpur", admin: "Maharashtra, India", lat: 21.1458, lon: 79.0882, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Indore", admin: "Madhya Pradesh, India", lat: 22.7196, lon: 75.8577, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Bhopal", admin: "Madhya Pradesh, India", lat: 23.2599, lon: 77.4126, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Patna", admin: "Bihar, India", lat: 25.5941, lon: 85.1376, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Vadodara", admin: "Gujarat, India", lat: 22.3072, lon: 73.1812, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Surat", admin: "Gujarat, India", lat: 21.1702, lon: 72.8311, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Chandigarh", admin: "Chandigarh, India", lat: 30.7333, lon: 76.7794, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  // Sacred & Historical Vedic Centers
  { name: "Ujjain", admin: "Madhya Pradesh, India", lat: 23.1765, lon: 75.7885, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Varanasi", admin: "Uttar Pradesh, India", lat: 25.3176, lon: 82.9739, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Kashi", admin: "Varanasi, UP, India", lat: 25.3176, lon: 82.9739, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Ayodhya", admin: "Uttar Pradesh, India", lat: 26.7922, lon: 82.1998, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Haridwar", admin: "Uttarakhand, India", lat: 29.9457, lon: 78.1642, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Rishikesh", admin: "Uttarakhand, India", lat: 30.0869, lon: 78.2676, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Mathura", admin: "Uttar Pradesh, India", lat: 27.4924, lon: 77.6737, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Vrindavan", admin: "Uttar Pradesh, India", lat: 27.5806, lon: 77.7006, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Prayagraj", admin: "Uttar Pradesh, India", lat: 25.4358, lon: 81.8463, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Allahabad", admin: "Uttar Pradesh, India", lat: 25.4358, lon: 81.8463, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Puri", admin: "Odisha, India", lat: 19.8135, lon: 85.8312, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Tirupati", admin: "Andhra Pradesh, India", lat: 13.6288, lon: 79.4192, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Dwarka", admin: "Gujarat, India", lat: 22.2442, lon: 68.9685, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Somnath", admin: "Gujarat, India", lat: 20.8880, lon: 70.4013, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Rameshwaram", admin: "Tamil Nadu, India", lat: 9.2876, lon: 79.3129, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Vadnagar", admin: "Gujarat, India", lat: 23.7840, lon: 72.6360, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Dehradun", admin: "Uttarakhand, India", lat: 30.3165, lon: 78.0322, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Noida", admin: "Uttar Pradesh, India", lat: 28.5355, lon: 77.3910, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Gurugram", admin: "Haryana, India", lat: 28.4595, lon: 77.0266, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Gurgaon", admin: "Haryana, India", lat: 28.4595, lon: 77.0266, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Thiruvananthapuram", admin: "Kerala, India", lat: 8.5241, lon: 76.9366, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Kochi", admin: "Kerala, India", lat: 9.9312, lon: 76.2673, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Bhubaneswar", admin: "Odisha, India", lat: 20.2961, lon: 85.8245, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Guwahati", admin: "Assam, India", lat: 26.1445, lon: 91.7362, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Srinagar", admin: "Jammu & Kashmir, India", lat: 34.0837, lon: 74.7973, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Shimla", admin: "Himachal Pradesh, India", lat: 31.1048, lon: 77.1734, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  { name: "Amritsar", admin: "Punjab, India", lat: 31.6340, lon: 74.8723, tz: "+05:30", iana: "Asia/Kolkata", countryCode: "IN" },
  // Major International Metros
  { name: "London", admin: "England, United Kingdom", lat: 51.5074, lon: -0.1278, tz: "+00:00", iana: "Europe/London", countryCode: "GB" },
  { name: "New York", admin: "New York, USA", lat: 40.7128, lon: -74.0060, tz: "-05:00", iana: "America/New_York", countryCode: "US" },
  { name: "San Francisco", admin: "California, USA", lat: 37.7749, lon: -122.4194, tz: "-08:00", iana: "America/Los_Angeles", countryCode: "US" },
  { name: "Los Angeles", admin: "California, USA", lat: 34.0522, lon: -118.2437, tz: "-08:00", iana: "America/Los_Angeles", countryCode: "US" },
  { name: "Chicago", admin: "Illinois, USA", lat: 41.8781, lon: -87.6298, tz: "-06:00", iana: "America/Chicago", countryCode: "US" },
  { name: "Toronto", admin: "Ontario, Canada", lat: 43.6532, lon: -79.3832, tz: "-05:00", iana: "America/Toronto", countryCode: "CA" },
  { name: "Vancouver", admin: "BC, Canada", lat: 49.2827, lon: -123.1207, tz: "-08:00", iana: "America/Vancouver", countryCode: "CA" },
  { name: "Dubai", admin: "United Arab Emirates", lat: 25.2048, lon: 55.2708, tz: "+04:00", iana: "Asia/Dubai", countryCode: "AE" },
  { name: "Singapore", admin: "Singapore", lat: 1.3521, lon: 103.8198, tz: "+08:00", iana: "Asia/Singapore", countryCode: "SG" },
  { name: "Tokyo", admin: "Tokyo, Japan", lat: 35.6762, lon: 139.6503, tz: "+09:00", iana: "Asia/Tokyo", countryCode: "JP" },
  { name: "Sydney", admin: "NSW, Australia", lat: -33.8688, lon: 151.2093, tz: "+10:00", iana: "Australia/Sydney", countryCode: "AU" },
  { name: "Melbourne", admin: "Victoria, Australia", lat: -37.8136, lon: 144.9631, tz: "+10:00", iana: "Australia/Melbourne", countryCode: "AU" },
  { name: "Paris", admin: "France", lat: 48.8566, lon: 2.3522, tz: "+01:00", iana: "Europe/Paris", countryCode: "FR" },
  { name: "Berlin", admin: "Germany", lat: 52.5200, lon: 13.4050, tz: "+01:00", iana: "Europe/Berlin", countryCode: "DE" },
  { name: "Rome", admin: "Italy", lat: 41.9028, lon: 12.4964, tz: "+01:00", iana: "Europe/Rome", countryCode: "IT" },
  { name: "Kuala Lumpur", admin: "Malaysia", lat: 3.1390, lon: 101.6869, tz: "+08:00", iana: "Asia/Kuala_Lumpur", countryCode: "MY" },
  { name: "Bangkok", admin: "Thailand", lat: 13.7563, lon: 100.5018, tz: "+07:00", iana: "Asia/Bangkok", countryCode: "TH" },
  { name: "Greenwich", admin: "London, UK", lat: 51.4769, lon: 0.0000, tz: "+00:00", iana: "UTC", countryCode: "GB" },
];

function findBestLocalCity(query) {
  const q = query.toLowerCase().trim();
  if (!q) return null;
  const exact = LOCAL_CITIES.find((c) => c.name.toLowerCase() === q);
  if (exact) return exact;
  const prefix = LOCAL_CITIES.find((c) => c.name.toLowerCase().startsWith(q));
  if (prefix) return prefix;
  return LOCAL_CITIES.find((c) => c.name.toLowerCase().includes(q) || c.admin.toLowerCase().includes(q)) || null;
}

const PRESETS = {
  delhi: () => {
    const now = new Date();
    const dStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    const h = String(now.getHours()).padStart(2, "0");
    const m = String(now.getMinutes()).padStart(2, "0");
    const s = String(now.getSeconds()).padStart(2, "0");
    return {
      date: dStr,
      time: `${h}:${m}:${s}`,
      tz_preset: "+05:30",
      latitude: 28.6139,
      longitude: 77.2090,
    };
  },
  modi: () => ({
    date: "1950-09-17",
    time: "09:34:00",
    tz_preset: "+05:30",
    latitude: 23.7840,
    longitude: 72.6360,
  }),
  ujjain: () => {
    const now = new Date();
    const dStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    const h = String(now.getHours()).padStart(2, "0");
    const m = String(now.getMinutes()).padStart(2, "0");
    const s = String(now.getSeconds()).padStart(2, "0");
    return {
      date: dStr,
      time: `${h}:${m}:${s}`,
      tz_preset: "+05:30",
      latitude: 23.1765,
      longitude: 75.7885,
    };
  },
  varanasi: () => {
    const now = new Date();
    const dStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    const h = String(now.getHours()).padStart(2, "0");
    const m = String(now.getMinutes()).padStart(2, "0");
    const s = String(now.getSeconds()).padStart(2, "0");
    return {
      date: dStr,
      time: `${h}:${m}:${s}`,
      tz_preset: "+05:30",
      latitude: 25.3176,
      longitude: 82.9739,
    };
  },
};

const $ = (id) => document.getElementById(id);
const form = $("chart-form");
const statusEl = $("status");
const submit = $("compute-btn");

let chartStyle = "north"; // "north" | "south"
let enginePromise;
let lastVedicJson = "";
let lastVedicData = null;

const escapeHtml = (s) =>
  String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

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
  if (!dateStr) dateStr = new Date().toISOString().slice(0, 10);
  if (!timeStr) timeStr = "12:00:00";
  if (!tzOffsetStr || typeof tzOffsetStr !== "string") tzOffsetStr = "+05:30";

  const sign = tzOffsetStr.startsWith("-") ? -1 : 1;
  const parts = tzOffsetStr.replace(/^[+-]/, "").split(":");
  const offsetMinutes = sign * (parseInt(parts[0] || "0", 10) * 60 + parseInt(parts[1] || "0", 10));

  const [year, month, day] = dateStr.split("-").map(Number);
  const timeParts = timeStr.split(":").map(Number);
  const hour = timeParts[0] || 0;
  const minute = timeParts[1] || 0;
  const second = timeParts[2] || 0;

  const ms = Date.UTC(year, (month || 1) - 1, day || 1, hour, minute, second) - offsetMinutes * 60000;
  return new Date(ms).toISOString().slice(0, 19) + "Z";
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
  const raw = (query || "").trim();
  const q = raw.toLowerCase();

  // If query is empty, show top popular cities as suggestions immediately
  if (q.length === 0) {
    return LOCAL_CITIES.slice(0, 8).map((c) => ({
      name: c.name,
      displayName: `${c.name}, ${c.admin}`,
      admin: c.admin,
      countryCode: c.countryCode,
      lat: c.lat,
      lon: c.lon,
      timezone: c.iana,
    }));
  }

  // 1. Instant local index lookup (0ms latency, 100% reliable)
  const localMatches = LOCAL_CITIES.filter(
    (c) => c.name.toLowerCase().includes(q) || c.admin.toLowerCase().includes(q)
  ).map((c) => ({
    name: c.name,
    displayName: `${c.name}, ${c.admin}`,
    admin: c.admin,
    countryCode: c.countryCode,
    lat: c.lat,
    lon: c.lon,
    timezone: c.iana,
  }));

  // If query is at least 2 chars, also attempt Open-Meteo online geocoding to catch any small town/locality
  if (raw.length >= 2) {
    try {
      const url = `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(raw)}&count=8&language=en&format=json`;
      const res = await fetch(url, { mode: "cors" });
      if (res.ok) {
        const data = await res.json();
        if (data.results && data.results.length > 0) {
          const onlineList = data.results.map((r) => {
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

          // Append any online results that aren't already represented in local matches
          for (const item of onlineList) {
            const exists = localMatches.some(
              (l) => Math.abs(l.lat - item.lat) < 0.08 && Math.abs(l.lon - item.lon) < 0.08
            );
            if (!exists) {
              localMatches.push(item);
            }
          }
        }
      }
    } catch (err) {
      console.warn("Online geocoding unavailable, using local index:", err);
    }
  }

  return localMatches.slice(0, 10);
}

function fillForm(values) {
  for (const [key, value] of Object.entries(values)) {
    if (form.elements[key] !== undefined) form.elements[key].value = value;
  }
  syncTzCustom();

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

async function compute() {
  if (!form.reportValidity()) return;

  // If user typed a search query and didn't pick from dropdown, commit best match
  const typedCity = (citySearchInput?.value || "").trim();
  const currentBadge = $("location-name-display")?.textContent || "";
  if (typedCity && !currentBadge.toLowerCase().includes(typedCity.toLowerCase())) {
    const match = findBestLocalCity(typedCity);
    if (match) {
      applyLocation({
        displayName: `${match.name}, ${match.admin}`,
        lat: match.lat,
        lon: match.lon,
        timezone: match.iana,
        tzOffset: match.tz,
      });
      hideCityDropdown();
    }
  }

  const data = new FormData(form);
  const dateStr = data.get("date");
  const timeStr = data.get("time") || "00:00:00";
  const tzOffset = getSelectedTzOffset();
  const utc = localToUtc(dateStr, timeStr, tzOffset);

  submit.disabled = true;
  const origSubmitHtml = submit.innerHTML;
  submit.innerHTML = `<span>⏳ Calculating…</span>`;
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

    // Determine local civil weekday from input date string (0=Sunday .. 6=Saturday)
    const [yNum, mNum, dNum] = dateStr.split("-").map(Number);
    const localDateObj = new Date(Date.UTC(yNum, mNum - 1, dNum, 12, 0, 0));
    const localWeekday = localDateObj.getUTCDay();

    // Astronomical Sunrise & Sunset for the user's selected civil date
    let isBeforeSunrise = false;
    let timings = null;

    try {
      // Query local solar noon to get exact sunrise and sunset for this specific date
      const noonUtc = localToUtc(dateStr, "12:00:00", tzOffset);
      const noonChart = engine.chart({ ...vedicReq, utc: noonUtc });
      if (noonChart && noonChart.planetary_hours) {
        const localSunrise = noonChart.planetary_hours.sunrise;
        const localSunset = noonChart.planetary_hours.sunset;

        const birthTimeUtc = new Date(utc).getTime();
        const sunriseTimeUtc = new Date(localSunrise).getTime();
        const sunsetTimeUtc = new Date(localSunset).getTime();

        if (birthTimeUtc < sunriseTimeUtc) {
          isBeforeSunrise = true;
        }

        // Calculate Day Duration (Dina Mana) in ms
        const dayDurMs = Math.max(1000, sunsetTimeUtc - sunriseTimeUtc);
        const partMs = dayDurMs / 8;

        // Classical Trikalam fraction by weekday (0=Sun .. 6=Sat)
        const RAHU_ORDER = [7, 1, 6, 4, 5, 3, 2];
        const GULIKA_ORDER = [6, 5, 4, 3, 2, 1, 0];
        const YAMAGANDA_ORDER = [4, 3, 2, 1, 0, 6, 5];

        const rahuPeriod = RAHU_ORDER[localWeekday];
        const gulikaPeriod = GULIKA_ORDER[localWeekday];
        const yamagandaPeriod = YAMAGANDA_ORDER[localWeekday];

        const rahuStart = new Date(sunriseTimeUtc + rahuPeriod * partMs);
        const rahuEnd = new Date(rahuStart.getTime() + partMs);

        const gulikaStart = new Date(sunriseTimeUtc + gulikaPeriod * partMs);
        const gulikaEnd = new Date(gulikaStart.getTime() + partMs);

        const yamagandaStart = new Date(sunriseTimeUtc + yamagandaPeriod * partMs);
        const yamagandaEnd = new Date(yamagandaStart.getTime() + partMs);

        // Abhijit Muhurta: 8th muhurta of 15 daytime muhurtas (sr + 7/15 to sr + 8/15)
        const muhurtaLength = dayDurMs / 15;
        const abhijitStart = new Date(sunriseTimeUtc + 7 * muhurtaLength);
        const abhijitEnd = new Date(sunriseTimeUtc + 8 * muhurtaLength);

        // Janma Ghati: elapsed minutes since sunrise / 24
        const elapsedSinceRiseMs = birthTimeUtc - sunriseTimeUtc;
        const ghatisSinceSunrise = elapsedSinceRiseMs / (24 * 60 * 1000);

        timings = {
          sunrise: new Date(localSunrise),
          sunset: new Date(localSunset),
          dayDurationMinutes: Math.round(dayDurMs / 60000),
          hourRuler: chart.planetary_hours ? chart.planetary_hours.hour_ruler : null,
          hourNumber: chart.planetary_hours ? chart.planetary_hours.hour_number : null,
          hourType: chart.planetary_hours ? chart.planetary_hours.hour_type : null,
          rahuKalam: { start: rahuStart, end: rahuEnd },
          gulikaKalam: { start: gulikaStart, end: gulikaEnd },
          yamaganda: { start: yamagandaStart, end: yamagandaEnd },
          abhijit: localWeekday === 3 ? null : { start: abhijitStart, end: abhijitEnd },
          ghatisSinceSunrise: ghatisSinceSunrise >= 0 ? ghatisSinceSunrise : null,
        };
      }
    } catch (e) {
      console.warn("Could not calculate solar timings:", e);
    }

    const panchang = calcPanchang(sunLon, moonLon, jd, localWeekday, isBeforeSunrise);
    const yearType = data.get("vedic_year_type") || "tssy";
    const dasha = calcVimshottariDasha(moonLon, jd, 3, yearType);

    lastVedicData = { d1, d9, panchang, dasha, chart, moonLon, jd, ascLon, timings, tzOffset };
    renderVedicView(lastVedicData);

    const ms = performance.now() - start;
    const took = ms < 1 ? "under 1 ms" : ms < 10 ? `${ms.toFixed(1)} ms` : `${Math.round(ms)} ms`;
    setStatus(`Vedic chart &amp; Dasha computed in ${took}, on your device.`);
  } catch (err) {
    const code = err && err.code ? ` (${err.code})` : "";
    setStatus(`${escapeHtml(err && err.message ? err.message : err)}${code}`, true);
  } finally {
    submit.disabled = false;
    submit.innerHTML = origSubmitHtml;
  }
}

// ---------------------------------------------------------------------------- Vedic Renderers

let activeVargas = ["D1", "D9"];

function renderVedicCharts(data) {
  if (!data || !data.chart) return;
  const container = $("vedic-charts-container");
  if (!container) return;

  if (!activeVargas || activeVargas.length === 0) {
    activeVargas = ["D1", "D9"];
  }

  container.innerHTML = "";

  activeVargas.forEach((vCode, idx) => {
    const vargaInfo = VARGA_LIST.find((v) => v.code === vCode) || {
      code: vCode,
      name: `${vCode} Chart`,
      desc: "Divisional Varga",
    };

    const card = document.createElement("div");
    card.className = "chart-card";

    // Card Header
    const header = document.createElement("div");
    header.className = "chart-card-header";

    const titleGroup = document.createElement("div");
    titleGroup.className = "chart-card-title-group";

    const select = document.createElement("select");
    select.className = "varga-select";
    select.setAttribute("aria-label", `Select Divisional Chart ${idx + 1}`);
    select.innerHTML = VARGA_LIST.map(
      (v) => `<option value="${v.code}" ${v.code === vCode ? "selected" : ""}>${escapeHtml(v.name)}</option>`,
    ).join("");

    select.addEventListener("change", (e) => {
      activeVargas[idx] = e.target.value;
      renderVedicCharts(lastVedicData);
    });

    const sub = document.createElement("span");
    sub.className = "chart-sub";
    sub.textContent = vargaInfo.desc;

    titleGroup.appendChild(select);
    titleGroup.appendChild(sub);
    header.appendChild(titleGroup);

    if (activeVargas.length > 1) {
      const btnRemove = document.createElement("button");
      btnRemove.type = "button";
      btnRemove.className = "btn-remove-chart";
      btnRemove.title = "Remove this chart";
      btnRemove.textContent = "✕";
      btnRemove.addEventListener("click", () => {
        activeVargas.splice(idx, 1);
        renderVedicCharts(lastVedicData);
      });
      header.appendChild(btnRemove);
    }

    card.appendChild(header);

    // Card SVG
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 400 400");
    svg.setAttribute("role", "img");
    svg.setAttribute("aria-label", `${vargaInfo.name} Chart`);
    card.appendChild(svg);

    container.appendChild(card);

    // Compute and render this divisional chart
    try {
      const divData = calcDivisional(data.chart, data.ascLon, vCode);
      if (chartStyle === "north") {
        renderNorthIndianChart(svg, divData, vargaInfo.name);
      } else {
        renderSouthIndianChart(svg, divData, vargaInfo.name);
      }
    } catch (err) {
      console.error(`Failed to calculate divisional ${vCode}:`, err);
    }
  });
}

function renderVedicView(data) {
  $("vedic-result").hidden = false;

  renderVedicCharts(data);
  renderPanchang($("vedic-panchang"), data.panchang, data.timings, data.tzOffset);
  renderVedicPlanetsTable($("vedic-planets"), data.d1, data.d9, data.chart, wasm);
  renderVimshottariDasha($("vedic-dasha"), data.dasha, data.moonLon, data.jd, wasm);

  lastVedicJson = JSON.stringify(
    {
      d1: data.d1,
      d9: data.d9,
      panchang: data.panchang,
      timings: data.timings,
      dasha: data.dasha,
      planets: data.chart.planets,
    },
    null,
    2,
  );
  $("vedic-json").textContent = lastVedicJson;
}

// ---------------------------------------------------------------------------- Wiring & Events

form.addEventListener("submit", (e) => {
  e.preventDefault();
  compute();
});

form.elements.tz_preset.addEventListener("change", syncTzCustom);

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

const btnAddChart = $("btn-add-chart");
if (btnAddChart) {
  btnAddChart.addEventListener("click", () => {
    const nextVarga = VARGA_LIST.find((v) => !activeVargas.includes(v.code));
    if (nextVarga) {
      activeVargas.push(nextVarga.code);
    } else {
      activeVargas.push("D10");
    }
    if (lastVedicData) renderVedicCharts(lastVedicData);
  });
}

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
    if (val.length === 0) {
      // Show popular recommendations when empty
      const popular = LOCAL_CITIES.slice(0, 8).map((c) => ({
        name: c.name,
        displayName: `${c.name}, ${c.admin}`,
        admin: c.admin,
        countryCode: c.countryCode,
        lat: c.lat,
        lon: c.lon,
        timezone: c.iana,
      }));
      renderCityDropdown(popular);
      return;
    }

    // 1. Instant local index lookup (0ms latency, 100% reliable)
    const local = LOCAL_CITIES.filter(
      (c) => c.name.toLowerCase().includes(val.toLowerCase()) || c.admin.toLowerCase().includes(val.toLowerCase())
    ).slice(0, 8).map((c) => ({
      name: c.name,
      displayName: `${c.name}, ${c.admin}`,
      admin: c.admin,
      countryCode: c.countryCode,
      lat: c.lat,
      lon: c.lon,
      timezone: c.iana,
    }));

    if (local.length > 0) {
      renderCityDropdown(local);
    } else if (cityDropdown) {
      cityDropdown.innerHTML = `<div class="city-dropdown-msg">Searching cities worldwide…</div>`;
      cityDropdown.hidden = false;
    }

    // 2. Debounced online lookup (250ms) to augment with global coordinates
    if (val.length >= 2) {
      searchDebounceTimer = setTimeout(async () => {
        const results = await searchCitiesOnline(val);
        if (citySearchInput.value.trim() === val && results.length > 0) {
          renderCityDropdown(results);
        }
      }, 250);
    }
  });

  citySearchInput.addEventListener("focus", () => {
    const val = citySearchInput.value.trim();
    if (val.length === 0) {
      const popular = LOCAL_CITIES.slice(0, 8).map((c) => ({
        name: c.name,
        displayName: `${c.name}, ${c.admin}`,
        admin: c.admin,
        countryCode: c.countryCode,
        lat: c.lat,
        lon: c.lon,
        timezone: c.iana,
      }));
      renderCityDropdown(popular);
    } else {
      const local = LOCAL_CITIES.filter(
        (c) => c.name.toLowerCase().includes(val.toLowerCase()) || c.admin.toLowerCase().includes(val.toLowerCase())
      ).slice(0, 8).map((c) => ({
        name: c.name,
        displayName: `${c.name}, ${c.admin}`,
        admin: c.admin,
        countryCode: c.countryCode,
        lat: c.lat,
        lon: c.lon,
        timezone: c.iana,
      }));
      if (local.length > 0) renderCityDropdown(local);
    }
  });

  citySearchInput.addEventListener("keydown", async (e) => {
    const items = cityDropdown ? cityDropdown.querySelectorAll(".city-item") : [];

    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (!cityDropdown || cityDropdown.hidden || items.length === 0) {
        const results = await searchCitiesOnline(citySearchInput.value);
        renderCityDropdown(results);
        return;
      }
      highlightedIndex = (highlightedIndex + 1) % items.length;
      items.forEach((it, idx) => it.classList.toggle("selected", idx === highlightedIndex));
      items[highlightedIndex]?.scrollIntoView({ block: "nearest" });
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (!cityDropdown || cityDropdown.hidden || items.length === 0) return;
      highlightedIndex = (highlightedIndex - 1 + items.length) % items.length;
      items.forEach((it, idx) => it.classList.toggle("selected", idx === highlightedIndex));
      items[highlightedIndex]?.scrollIntoView({ block: "nearest" });
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (highlightedIndex >= 0 && highlightedIndex < searchResults.length) {
        selectCityResult(searchResults[highlightedIndex]);
        return;
      }
      if (searchResults.length > 0) {
        selectCityResult(searchResults[0]);
        return;
      }
      const val = citySearchInput.value.trim();
      if (val) {
        const local = findBestLocalCity(val);
        if (local) {
          selectCityResult({
            name: local.name,
            displayName: `${local.name}, ${local.admin}`,
            admin: local.admin,
            countryCode: local.countryCode,
            lat: local.lat,
            lon: local.lon,
            timezone: local.iana,
          });
          return;
        }
        const online = await searchCitiesOnline(val);
        if (online.length > 0) {
          selectCityResult(online[0]);
          return;
        }
      }
      hideCityDropdown();
      compute();
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

// GPS Geolocation Button (Google UX pattern)
const btnLocateMe = $("btn-locate-me");
if (btnLocateMe) {
  btnLocateMe.addEventListener("click", () => {
    if (!navigator.geolocation) {
      setStatus("Geolocation is not supported by your browser.", true);
      return;
    }
    btnLocateMe.disabled = true;
    btnLocateMe.title = "Locating via GPS…";
    setStatus("Detecting current coordinates via GPS…");
    navigator.geolocation.getCurrentPosition(
      async (pos) => {
        btnLocateMe.disabled = false;
        btnLocateMe.title = "Use current location (GPS)";
        const lat = Number(pos.coords.latitude.toFixed(4));
        const lon = Number(pos.coords.longitude.toFixed(4));
        const closest = LOCAL_CITIES.find(
          (c) => Math.abs(c.lat - lat) < 0.25 && Math.abs(c.lon - lon) < 0.25
        );
        const bestName = closest ? `${closest.name}, ${closest.admin}` : `GPS (${formatCoords(lat, lon)})`;
        const tz = closest ? closest.tz : null;
        let iana = closest ? closest.iana : null;
        if (!iana) {
          try {
            iana = Intl.DateTimeFormat().resolvedOptions().timeZone;
          } catch {}
        }
        applyLocation({
          displayName: bestName,
          lat,
          lon,
          timezone: iana,
          tzOffset: tz,
        });
        if (citySearchInput) {
          citySearchInput.value = closest ? closest.name : bestName;
          if (citySearchClear) citySearchClear.hidden = false;
        }
        hideCityDropdown();
        compute();
      },
      (err) => {
        btnLocateMe.disabled = false;
        btnLocateMe.title = "Use current location (GPS)";
        setStatus(`Unable to retrieve GPS coordinates: ${err.message}`, true);
      },
      { timeout: 8000 }
    );
  });
}

// Now Button for instant current date & time
const btnNowTime = $("btn-now-time");
if (btnNowTime) {
  btnNowTime.addEventListener("click", () => {
    const now = new Date();
    const dStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    const h = String(now.getHours()).padStart(2, "0");
    const m = String(now.getMinutes()).padStart(2, "0");
    const s = String(now.getSeconds()).padStart(2, "0");
    form.elements.date.value = dStr;
    form.elements.time.value = `${h}:${m}:${s}`;
    compute();
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

// Preset chips (Delhi, Modi, Ujjain, Varanasi)
for (const btn of form.querySelectorAll("[data-preset]")) {
  btn.addEventListener("click", () => {
    form.querySelectorAll("[data-preset]").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
    const pName = btn.dataset.preset;
    if (PRESETS[pName]) {
      fillForm(PRESETS[pName]());
    }

    // Sync city chip active state
    document.querySelectorAll(".btn-city-chip").forEach((chip) => {
      if (pName === "modi" && chip.dataset.city === "vadnagar") chip.classList.add("active");
      else if (pName === "delhi" && chip.dataset.city === "delhi") chip.classList.add("active");
      else if (pName === "ujjain" && chip.dataset.city === "ujjain") chip.classList.add("active");
      else if (pName === "varanasi" && chip.dataset.city === "varanasi") chip.classList.add("active");
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

// Initial setup
syncTzCustom();

// Make New Delhi default location and current date & time on page load
fillForm(PRESETS.delhi());
compute();
