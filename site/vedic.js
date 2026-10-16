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
  Ascendant: { abbr: "Asc", sa: "Lagna", sym: "Asc", color: "#f59e0b" },
  Sun: { abbr: "Su", sa: "Surya", sym: "☉", color: "#f59e0b" },
  Moon: { abbr: "Mo", sa: "Chandra", sym: "☽", color: "#6366f1" },
  Mars: { abbr: "Ma", sa: "Mangala", sym: "♂", color: "#ef4444" },
  Mercury: { abbr: "Me", sa: "Budha", sym: "☿", color: "#10b981" },
  Jupiter: { abbr: "Ju", sa: "Guru", sym: "♃", color: "#f97316" },
  Venus: { abbr: "Ve", sa: "Shukra", sym: "♀", color: "#ec4899" },
  Saturn: { abbr: "Sa", sa: "Shani", sym: "♄", color: "#64748b" },
  "North Node": { abbr: "Ra", sa: "Rahu", sym: "☊", color: "#7c3aed" },
  "South Node": { abbr: "Ke", sa: "Ketu", sym: "☋", color: "#0891b2" },
  Uranus: { abbr: "Ur", sa: "Uranus", sym: "♅", color: "#38bdf8" },
  Neptune: { abbr: "Ne", sa: "Neptune", sym: "♆", color: "#818cf8" },
  Pluto: { abbr: "Pl", sa: "Pluto", sym: "♇", color: "#e879f9" },
};

export const VARGA_LIST = [
  { code: "D1", name: "D1 · Rashi", desc: "Physical Body & General Destiny" },
  { code: "D2", name: "D2 · Hora", desc: "Wealth, Finances & Prosperity" },
  { code: "D3", name: "D3 · Drekkana", desc: "Siblings, Courage & Vitality" },
  { code: "D4", name: "D4 · Chaturthamsa", desc: "Home, Real Estate & Fortune" },
  { code: "D7", name: "D7 · Saptamsa", desc: "Children, Progeny & Creative Lineage" },
  { code: "D9", name: "D9 · Navamsha", desc: "Dharma, Marriage & Soul Purpose" },
  { code: "D10", name: "D10 · Dasamsa", desc: "Career, Status & Public Achievements" },
  { code: "D12", name: "D12 · Dwadasamsa", desc: "Parents, Ancestors & Heritage" },
  { code: "D16", name: "D16 · Shodasamsa", desc: "Vehicles, Conveyances & Luxuries" },
  { code: "D20", name: "D20 · Vimsamsa", desc: "Spiritual Path, Sadhana & Worship" },
  { code: "D24", name: "D24 · Chaturvimsamsa", desc: "Higher Learning & Knowledge" },
  { code: "D27", name: "D27 · Saptavimsamsa", desc: "Inherent Strengths & Weaknesses" },
  { code: "D30", name: "D30 · Trimsamsa", desc: "Arishta, Obstacles & Misfortune" },
  { code: "D40", name: "D40 · Khavedamsa", desc: "Auspicious & Inauspicious Cycles" },
  { code: "D45", name: "D45 · Akshavedamsa", desc: "Character, Integrity & General Well-being" },
  { code: "D60", name: "D60 · Shashtiamsa", desc: "Past Karma & Root Soul Destiny" },
];

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

export function formatCompactDegMin(deg) {
  const norm = ((deg % 30) + 30) % 30;
  const d = Math.floor(norm);
  const m = Math.floor((norm - d) * 60);
  return `${d}°${String(m).padStart(2, "0")}′`;
}

// ---------------------------------------------------------------------------- North Indian Chart (Diamond)

// Sign numbers tucked cleanly in the apex/vertex of each house
const NI_SIGN_POS = [
  null,
  [200, 172], // H1 (bottom vertex of diamond)
  [100, 78],  // H2 (apex pointing down)
  [78, 100],  // H3 (apex pointing right)
  [172, 200], // H4 (right vertex of diamond)
  [78, 300],  // H5 (apex pointing right)
  [100, 322], // H6 (apex pointing up)
  [200, 228], // H7 (top vertex of diamond)
  [300, 322], // H8 (apex pointing up)
  [322, 300], // H9 (apex pointing left)
  [228, 200], // H10 (left vertex of diamond)
  [322, 100], // H11 (apex pointing left)
  [300, 78],  // H12 (apex pointing down)
];

// Mathematical house-aware planet layout: guarantees ZERO overlaps in diamonds and triangles
function getNorthIndianPlanetCoords(house, idx, count) {
  // Houses 1, 4, 7, 10 (Kendra Diamonds: 140x140 diamond interior)
  if (house === 1) {
    if (count <= 3) {
      const startY = 95 - ((count - 1) * 22) / 2;
      return [200, startY + idx * 22];
    }
    const rows = Math.ceil(count / 2);
    const startY = 95 - ((rows - 1) * 20) / 2;
    const px = idx % 2 === 0 ? 166 : 234;
    const py = startY + Math.floor(idx / 2) * 20;
    return [px, py];
  }
  if (house === 4) {
    if (count <= 3) {
      const startY = 200 - ((count - 1) * 22) / 2;
      return [95, startY + idx * 22];
    }
    const rows = Math.ceil(count / 2);
    const startY = 200 - ((rows - 1) * 20) / 2;
    const px = idx % 2 === 0 ? 68 : 124;
    const py = startY + Math.floor(idx / 2) * 20;
    return [px, py];
  }
  if (house === 7) {
    if (count <= 3) {
      const startY = 305 - ((count - 1) * 22) / 2;
      return [200, startY + idx * 22];
    }
    const rows = Math.ceil(count / 2);
    const startY = 305 - ((rows - 1) * 20) / 2;
    const px = idx % 2 === 0 ? 166 : 234;
    const py = startY + Math.floor(idx / 2) * 20;
    return [px, py];
  }
  if (house === 10) {
    if (count <= 3) {
      const startY = 200 - ((count - 1) * 22) / 2;
      return [305, startY + idx * 22];
    }
    const rows = Math.ceil(count / 2);
    const startY = 200 - ((rows - 1) * 20) / 2;
    const px = idx % 2 === 0 ? 276 : 332;
    const py = startY + Math.floor(idx / 2) * 20;
    return [px, py];
  }

  // Houses 3, 5 (Vertical Left Triangles: Base at x=0, tall 140px vertical space)
  if (house === 3) {
    const startY = 100 - ((count - 1) * 21) / 2;
    return [38, startY + idx * 21];
  }
  if (house === 5) {
    const startY = 300 - ((count - 1) * 21) / 2;
    return [38, startY + idx * 21];
  }

  // Houses 9, 11 (Vertical Right Triangles: Base at x=400, tall 140px vertical space)
  if (house === 11) {
    const startY = 100 - ((count - 1) * 21) / 2;
    return [362, startY + idx * 21];
  }
  if (house === 9) {
    const startY = 300 - ((count - 1) * 21) / 2;
    return [362, startY + idx * 21];
  }

  // Houses 2, 12 (Horizontal Top Triangles: Base at y=0, wide 160px horizontal space)
  if (house === 2) {
    if (count === 1) return [100, 36];
    if (count === 2) return [100, idx === 0 ? 24 : 48];
    if (count === 3) {
      if (idx === 0) return [62, 24];
      if (idx === 1) return [138, 24];
      return [100, 48];
    }
    const coords4 = [[58, 22], [142, 22], [76, 44], [124, 44], [100, 56]];
    return coords4[idx] || [100, 36];
  }
  if (house === 12) {
    if (count === 1) return [300, 36];
    if (count === 2) return [300, idx === 0 ? 24 : 48];
    if (count === 3) {
      if (idx === 0) return [262, 24];
      if (idx === 1) return [338, 24];
      return [300, 48];
    }
    const coords4 = [[258, 22], [342, 22], [276, 44], [324, 44], [300, 56]];
    return coords4[idx] || [300, 36];
  }

  // Houses 6, 8 (Horizontal Bottom Triangles: Base at y=400, wide 160px horizontal space)
  if (house === 6) {
    if (count === 1) return [100, 364];
    if (count === 2) return [100, idx === 0 ? 352 : 376];
    if (count === 3) {
      if (idx === 0) return [100, 352];
      if (idx === 1) return [62, 376];
      return [138, 376];
    }
    const coords4 = [[76, 356], [124, 356], [58, 378], [142, 378], [100, 366]];
    return coords4[idx] || [100, 364];
  }
  if (house === 8) {
    if (count === 1) return [300, 364];
    if (count === 2) return [300, idx === 0 ? 352 : 376];
    if (count === 3) {
      if (idx === 0) return [300, 352];
      if (idx === 1) return [262, 376];
      return [338, 376];
    }
    const coords4 = [[276, 356], [324, 356], [258, 378], [342, 378], [300, 366]];
    return coords4[idx] || [300, 364];
  }

  return [200, 200];
}

export function renderNorthIndianChart(svgEl, divChart, title = "") {
  svgEl.replaceChildren();
  const W = 400;
  const H = 400;
  svgEl.setAttribute("viewBox", `0 0 ${W} ${H}`);

  const add = (tag, attrs, content) => {
    const el = document.createElementNS(SVG_NS, tag);
    for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
    if (content !== undefined) el.textContent = content;
    svgEl.append(el);
    return el;
  };

  // Background and Double outer frame (matching hora-prakash)
  add("rect", { x: 0, y: 0, width: W, height: H, rx: 4, class: "ni-bg" });
  add("rect", { x: 1.5, y: 1.5, width: W - 3, height: H - 3, rx: 4, class: "ni-border" });
  add("rect", { x: 6, y: 6, width: W - 12, height: H - 12, rx: 2, class: "ni-border-inset" });

  // Kendra house subtle elevation fills (1, 4, 7, 10)
  add("polygon", { points: `200,6 300,100 200,200 100,100`, class: "ni-kendra-fill" });
  add("polygon", { points: `6,200 100,100 200,200 100,300`, class: "ni-kendra-fill" });
  add("polygon", { points: `200,200 300,300 200,${H - 6} 100,300`, class: "ni-kendra-fill" });
  add("polygon", { points: `300,100 ${W - 6},200 300,300 200,200`, class: "ni-kendra-fill" });

  // Inner diagonals connecting to inset frame
  add("line", { x1: 6, y1: 6, x2: W - 6, y2: H - 6, class: "ni-line" });
  add("line", { x1: W - 6, y1: 6, x2: 6, y2: H - 6, class: "ni-line" });

  // Inner diamond
  add("polygon", {
    points: `200,6 ${W - 6},200 200,${H - 6} 6,200`,
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
    const signPos = NI_SIGN_POS[h];

    // Sign number text
    const signText = add(
      "text",
      {
        x: signPos[0],
        y: signPos[1],
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

    const count = planets.length;

    planets.forEach((p, idx) => {
      const [px, py] = getNorthIndianPlanetCoords(h, idx, count);
      const info = PLANET_INFO[p.name] || { abbr: p.name.slice(0, 2), color: "#e8e9f3" };
      const degStr = p.degree !== undefined ? `${formatCompactDegMin(p.degree)}` : "";

      if (p.isAsc) {
        add("rect", {
          x: px - 38,
          y: py - 10,
          width: 76,
          height: 20,
          rx: 5,
          class: "ni-lagna-badge-bg",
        });

        const textEl = add(
          "text",
          {
            x: px,
            y: py,
            class: "ni-planet ni-lagna",
            "text-anchor": "middle",
            "dominant-baseline": "central",
          },
          `Asc ${degStr}`,
        );
        textEl.append(
          Object.assign(document.createElementNS(SVG_NS, "title"), {
            textContent: `Lagna (Ascendant) in House ${h} (${signObj.sa}) at ${formatDms(p.degree || 0)}`,
          }),
        );
        return;
      }

      const textEl = document.createElementNS(SVG_NS, "text");
      textEl.setAttribute("x", px);
      textEl.setAttribute("y", py);
      textEl.setAttribute("class", `ni-planet ${p.is_retrograde ? "retro" : ""}`);
      textEl.setAttribute("text-anchor", "middle");
      textEl.setAttribute("dominant-baseline", "central");

      const tspanName = document.createElementNS(SVG_NS, "tspan");
      tspanName.setAttribute("fill", info.color || "var(--text)");
      tspanName.textContent = `${info.abbr} ${degStr}`;
      textEl.appendChild(tspanName);

      if (p.is_retrograde) {
        const tspanRetro = document.createElementNS(SVG_NS, "tspan");
        tspanRetro.setAttribute("fill", "#ef4444");
        tspanRetro.setAttribute("font-weight", "800");
        tspanRetro.textContent = " ᴿ";
        textEl.appendChild(tspanRetro);
      }

      textEl.appendChild(
        Object.assign(document.createElementNS(SVG_NS, "title"), {
          textContent: `${p.name} in House ${h} (${signObj.sa}) at ${formatDms(p.degree || 0)}${p.is_retrograde ? " (Retrograde)" : ""}`,
        }),
      );
      svgEl.appendChild(textEl);
    });
  }

  // Title badge in corner or center
  if (title) {
    add("text", { x: 10, y: 18, class: "ni-title", "text-anchor": "start" }, title);
  }
}

// ---------------------------------------------------------------------------- South Indian Chart (Box)

// Cell grid (row, col) for signs 1..12
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
  svgEl.setAttribute("viewBox", `0 0 ${W} ${H}`);

  const add = (tag, attrs, content) => {
    const el = document.createElementNS(SVG_NS, tag);
    for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
    if (content !== undefined) el.textContent = content;
    svgEl.append(el);
    return el;
  };

  // Background and Double outer frame (matching hora-prakash)
  add("rect", { x: 0, y: 0, width: W, height: H, rx: 4, class: "si-bg" });
  add("rect", { x: 1.5, y: 1.5, width: W - 3, height: H - 3, rx: 4, class: "si-border" });
  add("rect", { x: 6, y: 6, width: W - 12, height: H - 12, rx: 2, class: "si-border-inset" });

  const lagnaSign = divChart.lagna.sign;
  const lagnaSignObj = VEDIC_SIGNS[lagnaSign - 1];

  // Grid lines
  for (let i = 1; i < 4; i++) {
    add("line", { x1: 6, y1: i * CELL, x2: W - 6, y2: i * CELL, class: "si-line" });
    add("line", { x1: i * CELL, y1: 6, x2: i * CELL, y2: H - 6, class: "si-line" });
  }

  // Clear center (2x2)
  add("rect", {
    x: CELL + 1,
    y: CELL + 1,
    width: CELL * 2 - 2,
    height: CELL * 2 - 2,
    rx: 4,
    class: "si-center",
  });

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
    const houseFromLagna = ((s - lagnaSign + 12) % 12) + 1;
    const isKendra = houseFromLagna === 1 || houseFromLagna === 4 || houseFromLagna === 7 || houseFromLagna === 10;

    // Kendra / Lagna highlight in South Indian box
    if (isLagna) {
      add("rect", {
        x: x0 + 1,
        y: y0 + 1,
        width: CELL - 2,
        height: CELL - 2,
        class: "si-lagna-cell-fill",
      });
    } else if (isKendra) {
      add("rect", {
        x: x0 + 1,
        y: y0 + 1,
        width: CELL - 2,
        height: CELL - 2,
        class: "si-kendra-fill",
      });
    }

    // Lagna slash in corner if this is the Lagna sign
    if (isLagna) {
      add("line", { x1: x0 + 2, y1: y0 + 2, x2: x0 + 22, y2: y0 + 22, class: "si-lagna-slash" });
      add("line", { x1: x0 + 2, y1: y0 + 7, x2: x0 + 17, y2: y0 + 22, class: "si-lagna-slash" });
    }

    // Horizontal divider line under header (matching hora-prakash)
    add("line", {
      x1: x0 + 2,
      y1: y0 + 24,
      x2: x0 + CELL - 2,
      y2: y0 + 24,
      class: "si-cell-divider",
    });

    // Sign name label in cell
    const signLabel = add(
      "text",
      {
        x: x0 + 6,
        y: y0 + 16,
        class: `si-sign-label ${isLagna ? "is-lagna" : ""}`,
      },
      `${signObj.sa.slice(0, 3)} ${s}`,
    );
    signLabel.append(
      Object.assign(document.createElementNS(SVG_NS, "title"), {
        textContent: `${signObj.sa} (${signObj.en}) - Sign ${s}, Lord: ${signObj.lord}`,
      }),
    );

    // House from Lagna badge in top-right
    const houseLabel = add(
      "text",
      {
        x: x0 + CELL - 6,
        y: y0 + 16,
        class: `si-house-tag ${isKendra ? "kendra" : ""}`,
        "text-anchor": "end",
      },
      isLagna ? "H1 (Lg)" : `${houseFromLagna}`,
    );
    houseLabel.append(
      Object.assign(document.createElementNS(SVG_NS, "title"), {
        textContent: `House ${houseFromLagna} from Lagna`,
      }),
    );

    // Planets in this sign
    const planets = signPlanets[s];
    if (planets.length === 0) continue;

    const count = planets.length;
    const renderSiPlanet = (p, px, py) => {
      const info = PLANET_INFO[p.name] || { abbr: p.name.slice(0, 2), color: "var(--chart-text)" };
      const degStr = p.degree !== undefined
        ? (count >= 4 ? `${Math.floor(p.degree)}°` : `${formatCompactDegMin(p.degree)}`)
        : "";

      if (py < y0 + CELL - 4) {
        if (p.isAsc) {
          add("rect", {
            x: px - 2,
            y: py - 9,
            width: count >= 4 ? 44 : 74,
            height: 17,
            rx: 4,
            class: "si-lagna-badge-bg",
          });

          const textEl = add(
            "text",
            {
              x: px + (count >= 4 ? 20 : 35),
              y: py,
              class: "si-planet si-lagna-text",
              "text-anchor": "middle",
              "dominant-baseline": "central",
            },
            count >= 4 ? `Asc ${Math.floor(p.degree)}°` : `Asc ${degStr}`,
          );
          textEl.append(
            Object.assign(document.createElementNS(SVG_NS, "title"), {
              textContent: `Lagna in ${signObj.sa} at ${formatDms(p.degree || 0)}`,
            }),
          );
          return;
        }

        const textEl = document.createElementNS(SVG_NS, "text");
        textEl.setAttribute("x", px);
        textEl.setAttribute("y", py);
        textEl.setAttribute("class", `si-planet ${p.is_retrograde ? "retro" : ""}`);
        textEl.setAttribute("dominant-baseline", "central");

        const tspanName = document.createElementNS(SVG_NS, "tspan");
        tspanName.setAttribute("fill", info.color || "var(--chart-text)");
        tspanName.textContent = `${info.abbr} ${degStr}`;
        textEl.appendChild(tspanName);

        if (p.is_retrograde) {
          const tspanRetro = document.createElementNS(SVG_NS, "tspan");
          tspanRetro.setAttribute("fill", "#ef4444");
          tspanRetro.setAttribute("font-weight", "800");
          tspanRetro.textContent = " ᴿ";
          textEl.appendChild(tspanRetro);
        }

        textEl.appendChild(
          Object.assign(document.createElementNS(SVG_NS, "title"), {
            textContent: `${p.name} in ${signObj.sa} at ${formatDms(p.degree || 0)}${p.is_retrograde ? " (Retrograde)" : ""}`,
          }),
        );
        svgEl.appendChild(textEl);
      }
    };

    if (count <= 3) {
      const startY = y0 + 38;
      const lineSpacing = 19;
      planets.forEach((p, idx) => {
        renderSiPlanet(p, x0 + 8, startY + idx * lineSpacing);
      });
    } else {
      // 4+ planets: 2-column layout to prevent clipping in cell
      const startY = y0 + 36;
      const lineSpacing = 18;
      planets.forEach((p, idx) => {
        const px = idx % 2 === 0 ? x0 + 5 : x0 + 52;
        const py = startY + Math.floor(idx / 2) * lineSpacing;
        renderSiPlanet(p, px, py);
      });
    }
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
