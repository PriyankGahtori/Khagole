// Vedic Astrology (Jyotish) module for astroceleste-engine.
// Implements D1 (Rasi) & D9 (Navamsha) SVG chart renderers (North & South Indian),
// Planetary info table with Nakshatras & Padas, Panchanga display, and Vimshottari Dasha tree.

export const VEDIC_SIGNS = [
  { num: 1, en: "Aries", sa: "Mesha", sym: "♈", lord: "Mars", element: "fire" },
  { num: 2, en: "Taurus", sa: "Vrishabha", sym: "♉", lord: "Venus", element: "earth" },
  { num: 3, en: "Gemini", sa: "Mithuna", sym: "♊", lord: "Mercury", element: "air" },
  { num: 4, en: "Cancer", sa: "Karka", sym: "♋", lord: "Moon", element: "water" },
  { num: 5, en: "Leo", sa: "Simha", sym: "♌", lord: "Sun", element: "fire" },
  { num: 6, en: "Virgo", sa: "Kanya", sym: "♍", lord: "Mercury", element: "earth" },
  { num: 7, en: "Libra", sa: "Tula", sym: "♎", lord: "Venus", element: "air" },
  { num: 8, en: "Scorpio", sa: "Vrishchika", sym: "♏", lord: "Mars", element: "water" },
  { num: 9, en: "Sagittarius", sa: "Dhanu", sym: "♐", lord: "Jupiter", element: "fire" },
  { num: 10, en: "Capricorn", sa: "Makara", sym: "♑", lord: "Saturn", element: "earth" },
  { num: 11, en: "Aquarius", sa: "Kumbha", sym: "♒", lord: "Saturn", element: "air" },
  { num: 12, en: "Pisces", sa: "Meena", sym: "♓", lord: "Jupiter", element: "water" },
];

export const PLANET_INFO = {
  Ascendant: { abbr: "Asc", sa: "Lagna", sym: "Asc", color: "#e0b35a" },
  Sun: { abbr: "Su", sa: "Surya", sym: "☉", color: "#f59e0b" },
  Moon: { abbr: "Mo", sa: "Chandra", sym: "☽", color: "#e2e8f0" },
  Mars: { abbr: "Ma", sa: "Mangala", sym: "♂", color: "#ef4444" },
  Mercury: { abbr: "Me", sa: "Budha", sym: "☿", color: "#10b981" },
  Jupiter: { abbr: "Ju", sa: "Guru", sym: "♃", color: "#fbbf24" },
  Venus: { abbr: "Ve", sa: "Shukra", sym: "♀", color: "#f472b6" },
  Saturn: { abbr: "Sa", sa: "Shani", sym: "♄", color: "#60a5fa" },
  "North Node": { abbr: "Ra", sa: "Rahu", sym: "☊", color: "#a78bfa" },
  "South Node": { abbr: "Ke", sa: "Ketu", sym: "☋", color: "#fb923c" },
  Uranus: { abbr: "Ur", sa: "Uranus", sym: "♅", color: "#38bdf8" },
  Neptune: { abbr: "Ne", sa: "Neptune", sym: "♆", color: "#818cf8" },
  Pluto: { abbr: "Pl", sa: "Pluto", sym: "♇", color: "#c084fc" },
};

const SVG_NS = "http://www.w3.org/2000/svg";

export function formatDms(deg) {
  const norm = ((deg % 360) + 360) % 360;
  const d = Math.floor(norm);
  const mFloat = (norm - d) * 60;
  const m = Math.floor(mFloat);
  const s = Math.round((mFloat - m) * 60);
  return `${d}° ${String(m).padStart(2, "0")}′ ${String(s).padStart(2, "0")}″`;
}

export function formatDegMin(deg) {
  const norm = ((deg % 30) + 30) % 30;
  const d = Math.floor(norm);
  const m = Math.floor((norm - d) * 60);
  return `${d}° ${String(m).padStart(2, "0")}′`;
}

// ---------------------------------------------------------------------------- North Indian Chart (Diamond)

const NI_HOUSE_COORDS = [
  null, // 1-indexed
  { signPos: [200, 168], center: [200, 85], anchor: "middle" },   // H1 (Top Diamond)
  { signPos: [100, 24],  center: [100, 58], anchor: "middle" },   // H2 (Top-Left Upper)
  { signPos: [24, 100],  center: [55, 100], anchor: "middle" },   // H3 (Top-Left Lower)
  { signPos: [168, 200], center: [85, 200], anchor: "middle" },   // H4 (Left Diamond)
  { signPos: [24, 300],  center: [55, 300], anchor: "middle" },   // H5 (Bottom-Left Upper)
  { signPos: [100, 376], center: [100, 342], anchor: "middle" },  // H6 (Bottom-Left Lower)
  { signPos: [200, 232], center: [200, 315], anchor: "middle" },  // H7 (Bottom Diamond)
  { signPos: [300, 376], center: [300, 342], anchor: "middle" },  // H8 (Bottom-Right Lower)
  { signPos: [376, 300], center: [345, 300], anchor: "middle" },  // H9 (Bottom-Right Upper)
  { signPos: [232, 200], center: [315, 200], anchor: "middle" },  // H10 (Right Diamond)
  { signPos: [376, 100], center: [345, 100], anchor: "middle" },  // H11 (Top-Right Lower)
  { signPos: [300, 24],  center: [300, 58], anchor: "middle" },   // H12 (Top-Right Upper)
];

export function renderNorthIndianChart(svgEl, divChart, title = "") {
  svgEl.replaceChildren();
  const W = 400;
  const H = 400;

  const add = (tag, attrs, content) => {
    const el = document.createElementNS(SVG_NS, tag);
    for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
    if (content !== undefined) el.textContent = content;
    svgEl.append(el);
    return el;
  };

  // Background and border
  add("rect", { x: 0, y: 0, width: W, height: H, class: "ni-bg" });
  add("rect", { x: 2, y: 2, width: W - 4, height: H - 4, class: "ni-border" });

  // Outer diagonals
  add("line", { x1: 0, y1: 0, x2: W, y2: H, class: "ni-line" });
  add("line", { x1: W, y1: 0, x2: 0, y2: H, class: "ni-line" });

  // Inner diamond
  add("polygon", {
    points: `200,0 ${W},200 200,${H} 0,200`,
    class: "ni-diamond",
  });

  // Lagna sign number (1..12)
  const lagnaSign = divChart.lagna.sign;

  // Group planets by house (1..12)
  const housePlanets = Array.from({ length: 13 }, () => []);
  // Add Ascendant to House 1
  housePlanets[1].push({
    name: "Ascendant",
    isAsc: true,
    degree: divChart.lagna.degree,
    is_retrograde: false,
  });

  for (const p of divChart.planets) {
    if (p.name === "Ascendant" || p.name === "Midheaven") continue;
    const h = p.house || 1;
    if (h >= 1 && h <= 12) {
      housePlanets[h].push(p);
    }
  }

  // Draw house signs and planets
  for (let h = 1; h <= 12; h++) {
    const signNum = ((lagnaSign - 1 + (h - 1)) % 12) + 1;
    const signObj = VEDIC_SIGNS[signNum - 1];
    const coords = NI_HOUSE_COORDS[h];

    // Sign number text
    const signText = add(
      "text",
      {
        x: coords.signPos[0],
        y: coords.signPos[1],
        class: `ni-sign-num ${h === 1 || h === 4 || h === 7 || h === 10 ? "kendra" : ""}`,
        "text-anchor": "middle",
        "dominant-baseline": "central",
      },
      String(signNum),
    );
    signText.append(
      Object.assign(document.createElementNS(SVG_NS, "title"), {
        textContent: `House ${h}: ${signObj.sa} (${signObj.en}) - Lord: ${signObj.lord}`,
      }),
    );

    // Planets inside this house
    const planets = housePlanets[h];
    if (planets.length === 0) continue;

    const [cx, cy] = coords.center;
    const count = planets.length;
    // Calculate layout for planets in this house
    const lineSpacing = 14;
    const startY = cy - ((count - 1) * lineSpacing) / 2;

    planets.forEach((p, idx) => {
      const info = PLANET_INFO[p.name] || { abbr: p.name.slice(0, 2), color: "#e8e9f3" };
      const retro = p.is_retrograde ? " ℞" : "";
      const degStr = p.degree !== undefined ? ` ${Math.floor(p.degree)}°` : "";
      const label = `${info.abbr}${degStr}${retro}`;

      const py = count === 1 ? cy : startY + idx * lineSpacing;
      const textEl = add(
        "text",
        {
          x: cx,
          y: py,
          class: `ni-planet ${p.isAsc ? "ni-lagna" : ""} ${p.is_retrograde ? "retro" : ""}`,
          fill: info.color || "var(--text)",
          "text-anchor": "middle",
          "dominant-baseline": "central",
        },
        label,
      );
      textEl.append(
        Object.assign(document.createElementNS(SVG_NS, "title"), {
          textContent: `${p.name} in House ${h} (${signObj.sa}) at ${formatDegMin(p.degree || 0)}${p.is_retrograde ? " (Retrograde)" : ""}`,
        }),
      );
    });
  }

  // Title badge in corner or center
  if (title) {
    add("text", { x: 8, y: 16, class: "ni-title", "text-anchor": "start" }, title);
  }
}

// ---------------------------------------------------------------------------- South Indian Chart (Box)

// Cell grid (row, col) for signs 1..12
// South Indian layout:
// Row 0: Pisces(12), Aries(1), Taurus(2), Gemini(3)
// Row 1: Aquarius(11), [center], [center], Cancer(4)
// Row 2: Capricorn(10), [center], [center], Leo(5)
// Row 3: Sagittarius(9), Scorpio(8), Libra(7), Virgo(6)
const SI_SIGN_CELLS = {
  12: [0, 0], 1: [0, 1], 2: [0, 2], 3: [0, 3],
  4: [1, 3], 5: [2, 3], 6: [3, 3],
  7: [3, 2], 8: [3, 1], 9: [3, 0],
  10: [2, 0], 11: [1, 0],
};

export function renderSouthIndianChart(svgEl, divChart, title = "") {
  svgEl.replaceChildren();
  const W = 400;
  const H = 400;
  const CELL = 100;

  const add = (tag, attrs, content) => {
    const el = document.createElementNS(SVG_NS, tag);
    for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
    if (content !== undefined) el.textContent = content;
    svgEl.append(el);
    return el;
  };

  // Background and outer border
  add("rect", { x: 0, y: 0, width: W, height: H, class: "si-bg" });
  add("rect", { x: 2, y: 2, width: W - 4, height: H - 4, class: "si-border" });

  // Grid lines
  for (let i = 1; i < 4; i++) {
    // horizontal lines
    add("line", { x1: 0, y1: i * CELL, x2: W, y2: i * CELL, class: "si-line" });
    // vertical lines
    add("line", { x1: i * CELL, y1: 0, x2: i * CELL, y2: H, class: "si-line" });
  }

  // Clear center (2x2)
  add("rect", {
    x: CELL + 1,
    y: CELL + 1,
    width: CELL * 2 - 2,
    height: CELL * 2 - 2,
    class: "si-center",
  });

  const lagnaSign = divChart.lagna.sign;
  const lagnaSignObj = VEDIC_SIGNS[lagnaSign - 1];

  // Center title and Lagna info
  if (title) {
    add("text", { x: 200, y: 185, class: "si-center-title", "text-anchor": "middle" }, title);
    add(
      "text",
      { x: 200, y: 215, class: "si-center-sub", "text-anchor": "middle" },
      `Lagna: ${lagnaSignObj.sa} (${formatDegMin(divChart.lagna.degree)})`,
    );
  }

  // Group planets by sign number (1..12)
  const signPlanets = Array.from({ length: 13 }, () => []);
  // Add Lagna marker
  signPlanets[lagnaSign].push({
    name: "Ascendant",
    isAsc: true,
    degree: divChart.lagna.degree,
    is_retrograde: false,
    house: 1,
  });

  for (const p of divChart.planets) {
    if (p.name === "Ascendant" || p.name === "Midheaven") continue;
    const s = p.sign || 1;
    if (s >= 1 && s <= 12) {
      signPlanets[s].push(p);
    }
  }

  // Render each sign cell
  for (let s = 1; s <= 12; s++) {
    const [row, col] = SI_SIGN_CELLS[s];
    const x0 = col * CELL;
    const y0 = row * CELL;
    const signObj = VEDIC_SIGNS[s - 1];
    const isLagna = s === lagnaSign;

    // Lagna slash in corner if this is the Lagna sign
    if (isLagna) {
      add("line", {
        x1: x0 + 1,
        y1: y0 + 1,
        x2: x0 + 28,
        y2: y0 + 28,
        class: "si-lagna-slash",
      });
    }

    // Sign name label in cell
    const signLabel = add(
      "text",
      {
        x: x0 + 6,
        y: y0 + 14,
        class: `si-sign-label ${isLagna ? "is-lagna" : ""}`,
      },
      `${signObj.sa} ${signObj.sym}`,
    );
    signLabel.append(
      Object.assign(document.createElementNS(SVG_NS, "title"), {
        textContent: `${signObj.sa} (${signObj.en}) - Sign ${s}, Lord: ${signObj.lord}`,
      }),
    );

    // Planets in this sign
    const planets = signPlanets[s];
    if (planets.length === 0) continue;

    const startY = y0 + 30;
    const lineSpacing = 13;

    planets.forEach((p, idx) => {
      const info = PLANET_INFO[p.name] || { abbr: p.name.slice(0, 2), color: "#e8e9f3" };
      const retro = p.is_retrograde ? " ℞" : "";
      const degStr = p.degree !== undefined ? ` ${Math.floor(p.degree)}°` : "";
      const label = p.isAsc ? "ASC (Lg)" : `${info.abbr}${degStr}${retro}`;

      const py = startY + idx * lineSpacing;
      if (py < y0 + CELL - 4) {
        add(
          "text",
          {
            x: x0 + 8,
            y: py,
            class: `si-planet ${p.isAsc ? "si-lagna-text" : ""} ${p.is_retrograde ? "retro" : ""}`,
            fill: info.color || "var(--text)",
          },
          label,
        );
      }
    });
  }
}

// ---------------------------------------------------------------------------- Panchanga Renderer

export function renderPanchang(containerEl, panchang) {
  if (!panchang) {
    containerEl.hidden = true;
    return;
  }
  containerEl.hidden = false;

  const tithiLeft = panchang.tithi ? panchang.tithi.percent_left?.toFixed(1) : "";
  const nakLeft = panchang.nakshatra_percent_left !== undefined ? panchang.nakshatra_percent_left?.toFixed(1) : "";
  const yogaLeft = panchang.yoga ? panchang.yoga.percent_left?.toFixed(1) : "";
  const karanaLeft = panchang.karana ? panchang.karana.percent_left?.toFixed(1) : "";

  containerEl.innerHTML = `
    <div class="panchang-card">
      <div class="panchang-grid">
        <div class="panchang-item">
          <span class="panchang-label">Tithi (तिथि)</span>
          <strong class="panchang-val">${panchang.tithi?.name || "–"}</strong>
          <small class="panchang-sub">${panchang.tithi?.paksha || ""} Paksha · ${tithiLeft}% left</small>
        </div>
        <div class="panchang-item">
          <span class="panchang-label">Vara (वार)</span>
          <strong class="panchang-val">${panchang.vara?.name || "–"}</strong>
          <small class="panchang-sub">Lord: ${panchang.vara?.lord || "–"}</small>
        </div>
        <div class="panchang-item">
          <span class="panchang-label">Nakshatra (नक्षत्र)</span>
          <strong class="panchang-val">${panchang.nakshatra?.name || "–"} (Pada ${panchang.nakshatra?.pada || "–"})</strong>
          <small class="panchang-sub">Lord: ${panchang.nakshatra?.lord || "–"} · ${nakLeft}% left</small>
        </div>
        <div class="panchang-item">
          <span class="panchang-label">Yoga (योग)</span>
          <strong class="panchang-val">${panchang.yoga?.name || "–"}</strong>
          <small class="panchang-sub">${yogaLeft ? `${yogaLeft}% left` : ""}</small>
        </div>
        <div class="panchang-item">
          <span class="panchang-label">Karana (करण)</span>
          <strong class="panchang-val">${panchang.karana?.name || "–"}</strong>
          <small class="panchang-sub">${karanaLeft ? `${karanaLeft}% left` : ""}</small>
        </div>
        <div class="panchang-item">
          <span class="panchang-label">Samvatsara (संवत्सर)</span>
          <strong class="panchang-val">${panchang.samvat?.vikram_name || panchang.samvat?.shaka_name || "–"}</strong>
          <small class="panchang-sub">Vikram ${panchang.samvat?.vikram_year || "–"} · Shaka ${panchang.samvat?.shaka_year || "–"}</small>
        </div>
      </div>
    </div>
  `;
}

// ---------------------------------------------------------------------------- Vedic Planetary Table

export function renderVedicPlanetsTable(tableEl, d1Chart, d9Chart, natalChart, wasm) {
  const d9Map = new Map((d9Chart?.planets || []).map((p) => [p.name, p]));
  const natalMap = new Map((natalChart?.planets || []).map((p) => [p.name, p]));

  // Standard classical order
  const ORDER = [
    "Ascendant",
    "Sun",
    "Moon",
    "Mars",
    "Mercury",
    "Jupiter",
    "Venus",
    "Saturn",
    "North Node",
    "South Node",
    "Uranus",
    "Neptune",
    "Pluto",
  ];

  const rows = [];

  // Ascendant row
  if (d1Chart?.lagna) {
    const lagnaLon = d1Chart.lagna.longitude;
    const nak = wasm.nakshatraInfo ? wasm.nakshatraInfo(lagnaLon) : null;
    const sign = VEDIC_SIGNS[d1Chart.lagna.sign - 1];
    const d9LagnaSign = d9Chart?.lagna ? VEDIC_SIGNS[d9Chart.lagna.sign - 1] : null;

    rows.push([
      `<td class="glyph" style="color:var(--accent)">Asc</td>`,
      `<td><strong>Lagna (Ascendant)</strong></td>`,
      `<td class="num">${formatDms(lagnaLon)}</td>`,
      `<td>${sign.sa} <span class="glyph">${sign.sym}</span></td>`,
      `<td class="num">${formatDegMin(d1Chart.lagna.degree)}</td>`,
      `<td>${nak ? `${nak.name} (Pada ${nak.pada})` : "–"}</td>`,
      `<td>${nak ? nak.lord : "–"}</td>`,
      `<td class="num"><strong>H1</strong></td>`,
      `<td>${d9LagnaSign ? `${d9LagnaSign.sa} (H1)` : "–"}</td>`,
      `<td class="num">–</td>`,
      `<td>Direct</td>`,
    ]);
  }

  // Planets
  const sortedPlanets = [...(d1Chart?.planets || [])].sort((a, b) => {
    let ia = ORDER.indexOf(a.name);
    let ib = ORDER.indexOf(b.name);
    if (ia === -1) ia = 99;
    if (ib === -1) ib = 99;
    return ia - ib;
  });

  for (const p of sortedPlanets) {
    if (p.name === "Ascendant" || p.name === "Midheaven") continue;
    const info = PLANET_INFO[p.name] || { abbr: p.name.slice(0, 2), sa: p.name, sym: "•", color: "var(--text)" };
    const natal = natalMap.get(p.name);
    const d9P = d9Map.get(p.name);
    const sign = VEDIC_SIGNS[p.sign - 1];
    const d9Sign = d9P ? VEDIC_SIGNS[d9P.sign - 1] : null;
    const nak = wasm.nakshatraInfo ? wasm.nakshatraInfo(p.longitude) : null;
    const isRetro = natal?.is_retrograde || p.is_retrograde;
    const speed = natal?.speed !== undefined ? natal.speed.toFixed(3) : "–";

    const displayName = p.name === "North Node" ? "Rahu (North Node)" : p.name === "South Node" ? "Ketu (South Node)" : `${p.name} (${info.sa})`;

    rows.push([
      `<td class="glyph" style="color:${info.color}">${info.sym}</td>`,
      `<td>${displayName}${isRetro ? ' <span class="retro" title="Retrograde">℞</span>' : ""}</td>`,
      `<td class="num">${formatDms(p.longitude)}</td>`,
      `<td>${sign.sa} <span class="glyph">${sign.sym}</span></td>`,
      `<td class="num">${formatDegMin(p.degree)}</td>`,
      `<td>${nak ? `${nak.name} (Pada ${nak.pada})` : "–"}</td>`,
      `<td>${nak ? nak.lord : "–"}</td>`,
      `<td class="num">House ${p.house}</td>`,
      `<td>${d9Sign ? `${d9Sign.sa} (H${d9P.house})` : "–"}</td>`,
      `<td class="num">${speed}</td>`,
      `<td>${isRetro ? '<span class="retro">Retrograde ℞</span>' : "Direct"}</td>`,
    ]);
  }

  table(
    tableEl,
    [
      "",
      "Planet / Graha",
      "Sidereal Longitude",
      "Rasi (Sign)",
      "Degree",
      "Nakshatra",
      "Nak. Lord",
      "D1 House",
      "D9 Navamsha",
      "Speed °/day",
      "Motion",
    ],
    rows,
  );
}

// ---------------------------------------------------------------------------- Vimshottari Dasha Explorer

export function renderVimshottariDasha(containerEl, dashaList, moonLon, birthJd, wasm) {
  if (!dashaList || dashaList.length === 0) {
    containerEl.hidden = true;
    return;
  }
  containerEl.hidden = false;

  const nakInfo = wasm.nakshatraInfo ? wasm.nakshatraInfo(moonLon) : null;
  const todayStr = new Date().toISOString().slice(0, 10);

  // Find currently running Mahadasha, Antardasha, and Pratyantardasha
  let activeMd = null;
  let activeAd = null;
  let activePd = null;

  for (const md of dashaList) {
    if (todayStr >= md.start_date && todayStr < md.end_date) {
      activeMd = md;
      if (md.children) {
        for (const ad of md.children) {
          if (todayStr >= ad.start_date && todayStr < ad.end_date) {
            activeAd = ad;
            if (ad.children) {
              for (const pd of ad.children) {
                if (todayStr >= pd.start_date && todayStr < pd.end_date) {
                  activePd = pd;
                  break;
                }
              }
            }
            break;
          }
        }
      }
      break;
    }
  }

  // Calculate approximate birth balance from first mahadasha
  const firstMd = dashaList[0];
  const balanceDays = Math.max(0, (firstMd.end_jd - birthJd));
  const balYearsFloat = balanceDays / 365.256363;
  const balY = Math.floor(balYearsFloat);
  const balM = Math.floor((balYearsFloat - balY) * 12);
  const balD = Math.floor(((balYearsFloat - balY) * 12 - balM) * 30.4);

  // Calculate total span for timeline
  const firstJd = dashaList[0].start_jd || birthJd;
  const lastJd = dashaList[dashaList.length - 1].end_jd || (firstJd + 120 * 365.256363);
  const totalDays = Math.max(1, lastJd - firstJd);

  // Today's Julian day approx
  const nowMs = Date.now();
  const todayJd = 2440587.5 + nowMs / 86400000;
  let todayPercent = ((todayJd - firstJd) / totalDays) * 100;
  const showTodayMarker = todayPercent >= 0 && todayPercent <= 100;

  // Build 120-year timeline segments
  let timelineSegments = "";
  for (const md of dashaList) {
    const mdSpan = (md.end_jd - md.start_jd);
    const pct = Math.max(2, (mdSpan / totalDays) * 100);
    const info = PLANET_INFO[md.planet] || { color: "var(--accent)", abbr: md.planet.slice(0, 2) };
    const isMdActive = todayStr >= md.start_date && todayStr < md.end_date;
    timelineSegments += `
      <div class="dasha-timeline-seg ${isMdActive ? 'active' : ''}" 
           style="width:${pct.toFixed(2)}%; background-color:${info.color}33; border-color:${info.color}88"
           title="${md.planet} Mahadasha (${md.start_date} to ${md.end_date})">
        <span class="seg-label" style="color:${info.color}">${info.abbr}</span>
      </div>
    `;
  }

  let html = `
    <div class="dasha-overview">
      ${
        activeMd
          ? `
      <div class="dasha-hero-banner">
        <div class="dasha-hero-left">
          <span class="dasha-hero-pill">● Current Active Dasha</span>
          <div class="dasha-hero-title">
            <span class="graha-pill md-graha" style="border-color:${(PLANET_INFO[activeMd.planet]||{}).color}77; color:${(PLANET_INFO[activeMd.planet]||{}).color}">
              ${activeMd.planet}
            </span>
            <span class="graha-arrow">→</span>
            <span class="graha-pill ad-graha" style="border-color:${(PLANET_INFO[activeAd?.planet]||{}).color}77; color:${(PLANET_INFO[activeAd?.planet]||{}).color}">
              ${activeAd ? activeAd.planet : "–"}
            </span>
            <span class="graha-arrow">→</span>
            <span class="graha-pill pd-graha" style="border-color:${(PLANET_INFO[activePd?.planet]||{}).color}77; color:${(PLANET_INFO[activePd?.planet]||{}).color}">
              ${activePd ? activePd.planet : "–"}
            </span>
          </div>
          <p class="dasha-hero-sub">
            Mahadasha: <strong>${activeMd.planet}</strong> (${activeMd.start_date} to ${activeMd.end_date})<br>
            Antardasha: <strong>${activeAd ? activeAd.planet : "–"}</strong> (${activeAd?.start_date || "–"} to ${activeAd?.end_date || "–"})
            ${activePd ? `· Pratyantar: <strong>${activePd.planet}</strong> (until ${activePd.end_date})` : ""}
          </p>
        </div>
        <div class="dasha-hero-stats">
          <div class="stat-box">
            <span class="stat-label">Birth Balance</span>
            <span class="stat-value">${balY}y ${balM}m ${balD}d</span>
            <span class="stat-sub">${firstMd.planet} Dasha</span>
          </div>
          <div class="stat-box">
            <span class="stat-label">Moon Nakshatra</span>
            <span class="stat-value">${nakInfo ? nakInfo.name : "–"}</span>
            <span class="stat-sub">Pada ${nakInfo ? nakInfo.pada : "–"} · Lord ${nakInfo ? nakInfo.lord : "–"}</span>
          </div>
        </div>
      </div>
      `
          : ""
      }

      <div class="dasha-timeline-card">
        <div class="timeline-header">
          <span class="timeline-title">120-Year Vimshottari Life Cycle</span>
          <span class="timeline-legend">TSSY Standard · 9 Planetary Periods</span>
        </div>
        <div class="dasha-timeline-bar-wrap">
          <div class="dasha-timeline-bar">
            ${timelineSegments}
          </div>
          ${
            showTodayMarker
              ? `
          <div class="timeline-today-marker" style="left:${todayPercent.toFixed(2)}%">
            <span class="marker-pin">▼</span>
            <span class="marker-label">Today</span>
          </div>
          `
              : ""
          }
        </div>
      </div>
    </div>

    <div class="dasha-tree">
  `;

  dashaList.forEach((md, mIdx) => {
    const isMdActive = todayStr >= md.start_date && todayStr < md.end_date;
    const isPast = todayStr >= md.end_date;
    const statusClass = isMdActive ? "active" : isPast ? "past" : "future";
    const statusLabel = isMdActive ? "Active Now" : isPast ? "Past" : "Future";
    const info = PLANET_INFO[md.planet] || { color: "var(--accent)" };

    html += `
      <details class="dasha-node md-node ${statusClass}" ${isMdActive ? "open" : ""}>
        <summary class="dasha-node-summary">
          <span class="dasha-badge" style="background:${info.color}22; color:${info.color}; border-color:${info.color}55">
            ${md.planet}
          </span>
          <span class="dasha-period-name"><strong>${md.planet} Mahadasha</strong></span>
          <span class="dasha-dates">${md.start_date} → ${md.end_date}</span>
          <span class="dasha-status-tag ${statusClass}">${statusLabel}</span>
        </summary>
        <div class="dasha-children">
    `;

    if (md.children && md.children.length > 0) {
      md.children.forEach((ad, aIdx) => {
        const isAdActive = todayStr >= ad.start_date && todayStr < ad.end_date;
        const isAdPast = todayStr >= ad.end_date;
        const adStatusClass = isAdActive ? "active" : isAdPast ? "past" : "future";
        const adInfo = PLANET_INFO[ad.planet] || { color: "var(--accent)" };

        html += `
          <details class="dasha-node ad-node ${adStatusClass}" ${isAdActive ? "open" : ""}>
            <summary class="dasha-node-summary sub">
              <span class="dasha-badge small" style="background:${adInfo.color}22; color:${adInfo.color}">
                ${ad.planet}
              </span>
              <span class="dasha-period-name">${md.planet} – ${ad.planet}</span>
              <span class="dasha-dates">${ad.start_date} → ${ad.end_date}</span>
              <span class="dasha-status-tag ${adStatusClass}">${isAdActive ? "Active" : ""}</span>
            </summary>
            <div class="dasha-children sub">
        `;

        if (ad.children && ad.children.length > 0) {
          html += `<div class="pd-grid">`;
          ad.children.forEach((pd) => {
            const isPdActive = todayStr >= pd.start_date && todayStr < pd.end_date;
            const pdClass = isPdActive ? "active" : "";
            html += `
              <div class="pd-item ${pdClass}">
                <span class="pd-name"><strong>${pd.planet}</strong></span>
                <span class="pd-dates">${pd.start_date} → ${pd.end_date}</span>
                ${isPdActive ? '<span class="pd-active-dot">● Active</span>' : ""}
              </div>
            `;
          });
          html += `</div>`;
        }

        html += `
            </div>
          </details>
        `;
      });
    }

    html += `
        </div>
      </details>
    `;
  });

  html += `</div>`;
  containerEl.innerHTML = html;
}

function table(el, head, rows) {
  el.innerHTML =
    `<thead><tr>${head.map((h) => `<th>${h}</th>`).join("")}</tr></thead>` +
    `<tbody>${rows.map((r) => `<tr>${r.join("")}</tr>`).join("")}</tbody>`;
}
