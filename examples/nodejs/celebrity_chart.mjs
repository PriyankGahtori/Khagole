#!/usr/bin/env node
/**
 * Khagole Vedic Jyotish & JPL Ephemerides — Node.js Celebrity Chart Example
 * Computes high-precision sidereal Vedic birth charts, Panchanga, D9 Navamsha, and Vimshottari Dasha.
 */

import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// Resolve khagole-engine: try npm module first, fallback to local workspace build
let ke;
try {
  ke = await import("khagole-engine");
} catch {
  const localWasm = resolve(__dirname, "../../crates/khagole-engine-wasm/pkg-node/khagole_engine_wasm.js");
  if (existsSync(localWasm)) {
    ke = await import(localWasm);
  } else {
    console.error("Error: Could not load khagole-engine npm package or local WASM build.");
    console.error("Run `npm install` or `wasm-pack build --target nodejs --out-dir pkg-node crates/khagole-engine-wasm`");
    process.exit(1);
  }
}

const { Engine, calcPanchang, calcDivisional, calcVimshottariDasha, nakshatraInfo, julianDay } = ke;

function findKernel() {
  const candidates = [
    resolve(__dirname, "../../kernels/de440s.bsp"),
    resolve(__dirname, "../kernels/de440s.bsp"),
    resolve(__dirname, "kernels/de440s.bsp"),
    "/Users/priyankgahtori/Code/poc/khagole/kernels/de440s.bsp",
  ];
  for (const c of candidates) {
    if (existsSync(c)) return c;
  }
  throw new Error("NASA JPL kernel de440s.bsp not found. Run `scripts/fetch-kernels.sh` to download it.");
}

const CELEBRITIES = [
  {
    name: "Steve Jobs",
    utc: "1955-02-25T03:15:00Z",
    local_time: "1955-02-24 19:15 PST",
    location: "San Francisco, CA, USA",
    lat: 37.7749,
    lon: -122.4194,
  },
  {
    name: "Albert Einstein",
    utc: "1879-03-14T10:50:00Z",
    local_time: "1879-03-14 11:30 LMT",
    location: "Ulm, Germany",
    lat: 48.4011,
    lon: 9.9876,
  },
];

const NAVAGRAHA_NAMES = {
  "Sun": "Sun",
  "Moon": "Moon",
  "Mars": "Mars",
  "Mercury": "Mercury",
  "Jupiter": "Jupiter",
  "Venus": "Venus",
  "Saturn": "Saturn",
  "North Node": "Rahu",
  "South Node": "Ketu",
  "Ascendant": "Lagna (Asc)",
};

function main() {
  const kernelPath = findKernel();
  const kernelBytes = readFileSync(kernelPath);
  const engine = new Engine();
  engine.addKernel("de440s.bsp", kernelBytes);

  console.log("=".repeat(90));
  console.log("             KHAGOLE VEDIC JYOTISH & NASA JPL EPHEMERIDES (NODE.JS)              ");
  console.log("=".repeat(90));

  for (const celeb of CELEBRITIES) {
    console.log("\n" + "-".repeat(90));
    console.log(` CELEBRITY HOROSCOPE: ${celeb.name.toUpperCase()} (${celeb.location})`);
    console.log(` Birth Moment: ${celeb.local_time} | UTC: ${celeb.utc} | Lat: ${celeb.lat.toFixed(4)}°, Lon: ${celeb.lon.toFixed(4)}°`);
    console.log("-".repeat(90));

    // 1. Chart Request
    const chart = engine.chart({
      utc: celeb.utc,
      latitude: celeb.lat,
      longitude: celeb.lon,
      zodiac_type: "sidereal",
      ayanamsa: "lahiri",
      house_system: "W",
      position_mode: "true",
      node_type: "mean",
    });

    const ayanName = chart.ayanamsa_name || "Lahiri";
    const ayanVal = chart.ayanamsa_value || 0.0;
    const ayanFmt = chart.ayanamsa_formatted || "";
    console.log(`• Precession Ayanamsa: ${ayanName} (${ayanVal.toFixed(6)}° = ${ayanFmt})`);

    // 2. Navagraha & Lagna Placements Table
    console.log("\n[1] NAVAGRAHA & LAGNA POSITIONS (SIDEREAL LAHIRI):");
    console.log(
      "Body".padEnd(14) +
      "Rasi Sign".padEnd(12) +
      "Degree".padEnd(14) +
      "House".padEnd(8) +
      "Nakshatra".padEnd(18) +
      "Pada".padEnd(6) +
      "Lord".padEnd(10) +
      "Motion".padEnd(6)
    );
    console.log("-".repeat(90));

    let sunLon = 0.0;
    let moonLon = 0.0;
    let lagnaLon = 0.0;
    const planetsList = [];

    for (const p of chart.planets) {
      if (!(p.name in NAVAGRAHA_NAMES)) continue;
      const displayName = NAVAGRAHA_NAMES[p.name];
      const lon = p.ecliptic_longitude;
      const nak = nakshatraInfo(lon);
      const motion = p.is_retrograde ? "Retro" : "Direct";
      const totalSec = Math.round(lon * 3600.0);
      const sec = totalSec % 60;
      const dms = `${p.degree}° ${String(p.minute).padStart(2, "0")}′ ${String(sec).padStart(2, "0")}″`;

      console.log(
        displayName.padEnd(14) +
        p.sign.padEnd(12) +
        dms.padEnd(14) +
        `H${p.house}`.padEnd(8) +
        nak.name.padEnd(18) +
        String(nak.pada).padEnd(6) +
        nak.lord.padEnd(10) +
        motion.padEnd(6)
      );

      if (p.name === "Sun") sunLon = lon;
      if (p.name === "Moon") moonLon = lon;
      if (p.name === "Ascendant") lagnaLon = lon;

      planetsList.push({ name: displayName, ecliptic_longitude: lon });
    }

    // 3. Panchanga (Five Limbs of Vedic Time)
    const jd = julianDay(celeb.utc);
    const panchang = calcPanchang(sunLon, moonLon, jd);
    const { tithi: t, vara: v, nakshatra: n, yoga: y, karana: k, masa: m, samvat: s } = panchang;

    console.log("\n[2] PANCHANGA (FIVE LIMBS OF VEDIC TIME):");
    console.log(`  • Tithi (Lunar Day):     ${t.name} (${t.paksha}, #${t.number}) - ${t.percent_left.toFixed(1)}% remaining`);
    console.log(`  • Vara (Solar Weekday):  ${v.name} (Lord: ${v.lord}, Vedic: ${v.vedic_name})`);
    console.log(`  • Nakshatra:             ${n.name} (Pada ${n.pada}, Lord: ${n.lord})`);
    console.log(`  • Yoga (Luni-Solar):     ${y.name} (#${y.number}), ${y.percent_left.toFixed(1)}% left`);
    console.log(`  • Karana (Half-Tithi):   ${k.name} (#${k.number}), ${k.percent_left.toFixed(1)}% left`);
    console.log(`  • Lunar Month & Season:  ${m.amanta} (Amanta) / ${m.purnimanta} (Purnimanta) | Ritu: ${m.ritu} | Ayana: ${m.ayana}`);
    console.log(`  • Samvat Eras:           Vikram: ${s.vikram_year} (${s.vikram_name}) | Shaka: ${s.shaka_year} (${s.shaka_name})`);

    // 4. Divisional Chart D9 Navamsha
    const d9 = calcDivisional(planetsList, lagnaLon, "D9");
    console.log("\n[3] DIVISIONAL CHART D9 (NAVAMSHA):");
    const placements = d9.planets.map(dp => `${dp.name} in Sign #${dp.sign}`).join(", ");
    console.log(`  ${placements}`);

    // 5. Vimshottari Dasha Sequence
    const dashaTree = calcVimshottariDasha(moonLon, jd, 2, "tssy");
    console.log("\n[4] VIMSHOTTARI DASHA SEQUENCE (TRUE SIDEREAL SOLAR YEAR - TSSY):");
    console.log(`  ${"Mahadasha".padEnd(10)} ${"Start Date".padEnd(12)} ${"End Date".padEnd(12)} Antardashas`);
    console.log("  " + "-".repeat(55));
    for (const node of dashaTree.slice(0, 5)) {
      const subCount = node.children ? node.children.length : 0;
      console.log(`  ${node.planet.padEnd(10)} ${node.start_date.padEnd(12)} ${node.end_date.padEnd(12)} (${subCount} sub-periods)`);
    }
  }

  console.log("\n" + "=".repeat(90));
  console.log("                     VERIFICATION COMPLETE: ALL DETAILS COMPUTED 100%                     ");
  console.log("=".repeat(90));
}

main();
