// Vedic Astrology (Jyotish) module for Khagole.
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

export function renderPanchang(containerEl, panchang, timings, tzOffset, ayanamsa) {
  if (!panchang) {
    containerEl.hidden = true;
    return;
  }
  containerEl.hidden = false;

  const tithiLeft = panchang.tithi ? panchang.tithi.percent_left?.toFixed(1) : "";
  const nakLeft = panchang.nakshatra_percent_left !== undefined ? panchang.nakshatra_percent_left?.toFixed(1) : "";
  const yogaLeft = panchang.yoga ? panchang.yoga.percent_left?.toFixed(1) : "";
  const karanaLeft = panchang.karana ? panchang.karana.percent_left?.toFixed(1) : "";

  // Ayanamsa display badge
  let ayanamsaHtml = "";
  if (ayanamsa) {
    const isObj = typeof ayanamsa === "object";
    const name = isObj ? (ayanamsa.name || "Lahiri") : (ayanamsa === "lahiri" ? "Lahiri (Chitra Paksha)" : ayanamsa);
    const valDms = isObj ? (ayanamsa.formatted || (ayanamsa.value !== undefined && ayanamsa.value !== null ? formatDms(ayanamsa.value) : "")) : "";
    const valDeg = isObj && ayanamsa.value !== undefined && ayanamsa.value !== null ? `${ayanamsa.value.toFixed(6)}°` : "";
    ayanamsaHtml = `
      <div class="panchang-ayanamsa-group">
        <span class="panchang-ayanamsa-badge" title="Precession / Sidereal Zodiac Offset">
          <span class="ayanamsa-title">Ayanamsa:</span>
          <strong>${name}</strong>
          ${valDms ? `<span class="ayanamsa-val">${valDms}</span>` : ""}
          ${valDeg ? `<span class="ayanamsa-dec">(${valDeg})</span>` : ""}
        </span>
      </div>
    `;
  }

  // Time formatting helper
  const fmtTime = (dateObj) => {
    if (!dateObj) return "–";
    const mSign = tzOffset && tzOffset.startsWith("-") ? -1 : 1;
    const parts = tzOffset ? tzOffset.replace(/^[+-]/, "").split(":").map(Number) : [5, 30];
    const tzMinutes = mSign * (parts[0] * 60 + (parts[1] || 0));
    const localMs = dateObj.getTime() + tzMinutes * 60 * 1000;
    const d = new Date(localMs);
    const hh = d.getUTCHours();
    const mm = String(d.getUTCMinutes()).padStart(2, "0");
    const ampm = hh >= 12 ? "PM" : "AM";
    const h12 = hh % 12 === 0 ? 12 : hh % 12;
    return `${h12}:${mm} ${ampm}`;
  };

  // Vara label and subtitle
  const isBeforeSr = panchang.vara?.is_before_sunrise;
  const varaTitle = panchang.vara?.name || "–";
  const varaSub = isBeforeSr
    ? `Vedic: <strong>${panchang.vara?.vedic_name}</strong> (${panchang.vara?.vedic_lord}) · Until Sunrise`
    : `Lord: <strong>${panchang.vara?.lord || "–"}</strong>`;

  // Masa and Ritu
  const amantaMonth = panchang.masa?.amanta || "";
  const purnimantaMonth = panchang.masa?.purnimanta || "";
  const paksha = panchang.masa?.paksha || panchang.tithi?.paksha || "";
  const ritu = panchang.masa?.ritu || "";
  const ayana = panchang.masa?.ayana || "";
  const monthDisplay = amantaMonth === purnimantaMonth
    ? `${amantaMonth} Masa`
    : `${amantaMonth} (Amanta) / ${purnimantaMonth} (Purnimanta)`;

  // Timings strip HTML
  let timingsHtml = "";
  if (timings && timings.sunrise && timings.sunset) {
    const sunriseStr = fmtTime(timings.sunrise);
    const sunsetStr = fmtTime(timings.sunset);
    const rahuStr = timings.rahuKalam ? `${fmtTime(timings.rahuKalam.start)} – ${fmtTime(timings.rahuKalam.end)}` : "–";
    const yamaStr = timings.yamaganda ? `${fmtTime(timings.yamaganda.start)} – ${fmtTime(timings.yamaganda.end)}` : "–";
    const gulikaStr = timings.gulikaKalam ? `${fmtTime(timings.gulikaKalam.start)} – ${fmtTime(timings.gulikaKalam.end)}` : "–";
    const abhijitStr = timings.abhijit ? `${fmtTime(timings.abhijit.start)} – ${fmtTime(timings.abhijit.end)}` : "Not on Wed";
    const dayHours = Math.floor(timings.dayDurationMinutes / 60);
    const dayMins = timings.dayDurationMinutes % 60;
    const dinaMana = `${dayHours}h ${dayMins}m`;
    const horaStr = timings.hourRuler ? `${timings.hourRuler} (${timings.hourType || ""} H${timings.hourNumber || ""})` : "–";

    timingsHtml = `
      <div class="panchang-timings-container">
        <div class="panchang-timings-title">Auspicious &amp; Inauspicious Timings (सूर्य एवं काल समय)</div>
        <div class="panchang-timings-grid">
          <div class="panchang-timing-chip solar">
            <span class="panchang-timing-name">Sunrise (सूर्योदय)</span>
            <strong class="panchang-timing-val">${sunriseStr}</strong>
          </div>
          <div class="panchang-timing-chip solar">
            <span class="panchang-timing-name">Sunset (सूर्यास्त)</span>
            <strong class="panchang-timing-val">${sunsetStr}</strong>
          </div>
          <div class="panchang-timing-chip solar">
            <span class="panchang-timing-name">Dina Mana (दिन मान)</span>
            <strong class="panchang-timing-val">${dinaMana}</strong>
          </div>
          <div class="panchang-timing-chip solar">
            <span class="panchang-timing-name">Hora Lord (होरा)</span>
            <strong class="panchang-timing-val">${horaStr}</strong>
          </div>
          <div class="panchang-timing-chip inauspicious">
            <span class="panchang-timing-name">Rahu Kalam (राहु काल)</span>
            <strong class="panchang-timing-val">${rahuStr}</strong>
          </div>
          <div class="panchang-timing-chip inauspicious">
            <span class="panchang-timing-name">Yamaganda (यमगण्ड)</span>
            <strong class="panchang-timing-val">${yamaStr}</strong>
          </div>
          <div class="panchang-timing-chip auspicious">
            <span class="panchang-timing-name">Gulika Kalam (गुलिक काल)</span>
            <strong class="panchang-timing-val">${gulikaStr}</strong>
          </div>
          <div class="panchang-timing-chip auspicious">
            <span class="panchang-timing-name">Abhijit (अभिजीत मुहूर्त)</span>
            <strong class="panchang-timing-val">${abhijitStr}</strong>
          </div>
        </div>
      </div>
    `;
  }

  containerEl.innerHTML = `
    <div class="panchang-card">
      <div class="panchang-header">
        <div class="panchang-hero-group">
          <span class="panchang-masa-badge">
            <span>📅 ${monthDisplay}</span>
            <span>· ${paksha} Paksha</span>
          </span>
          <span class="panchang-season-tag">🍃 ${ritu} Ritu · ☀️ ${ayana}</span>
        </div>
        ${ayanamsaHtml}
        <div class="panchang-samvat-tag">
          Vikram ${panchang.samvat?.vikram_year || "–"} (${panchang.samvat?.vikram_name || "–"}) · Shaka ${panchang.samvat?.shaka_year || "–"} (${panchang.samvat?.shaka_name || "–"})
        </div>
      </div>
      <div class="panchang-grid">
        <div class="panchang-item">
          <span class="panchang-label">1. Tithi (तिथि)</span>
          <strong class="panchang-val">${panchang.tithi?.name || "–"}</strong>
          <small class="panchang-sub">${paksha} Paksha · ${tithiLeft}% left</small>
        </div>
        <div class="panchang-item">
          <span class="panchang-label">2. Vara (वार)</span>
          <strong class="panchang-val">${varaTitle}</strong>
          <small class="panchang-sub">${varaSub}</small>
        </div>
        <div class="panchang-item">
          <span class="panchang-label">3. Nakshatra (नक्षत्र)</span>
          <strong class="panchang-val">${panchang.nakshatra?.name || "–"} (Pada ${panchang.nakshatra?.pada || "–"})</strong>
          <small class="panchang-sub">Lord: <strong>${panchang.nakshatra?.lord || "–"}</strong> · ${nakLeft}% left</small>
        </div>
        <div class="panchang-item">
          <span class="panchang-label">4. Yoga (योग)</span>
          <strong class="panchang-val">${panchang.yoga?.name || "–"}</strong>
          <small class="panchang-sub">${yogaLeft ? `${yogaLeft}% left` : ""}</small>
        </div>
        <div class="panchang-item">
          <span class="panchang-label">5. Karana (करण)</span>
          <strong class="panchang-val">${panchang.karana?.name || "–"}</strong>
          <small class="panchang-sub">${karanaLeft ? `${karanaLeft}% left` : ""}</small>
        </div>
      </div>
      ${timingsHtml}
    </div>
  `;
}

// ---------------------------------------------------------------------------- Vedic Planetary Table

export function renderVedicPlanetsTable(tableEl, d1Chart, d9Chart, natalChart, wasm) {
  tableEl.classList.add("vedic-planets-table");
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

// // ---------------------------------------------------------------------------- Vimshottari Dasha Explorer (Classical Levels 1 to 6)

export const VIMSHOTTARI_LORDS = [
  { name: "Ketu", years: 7, color: "#94a3b8", devanagari: "के", abbr: "Ke" },
  { name: "Venus", years: 20, color: "#ec4899", devanagari: "शु", abbr: "Ve" },
  { name: "Sun", years: 6, color: "#f59e0b", devanagari: "सू", abbr: "Su" },
  { name: "Moon", years: 10, color: "#38bdf8", devanagari: "च", abbr: "Mo" },
  { name: "Mars", years: 7, color: "#ef4444", devanagari: "मं", abbr: "Ma" },
  { name: "Rahu", years: 18, color: "#6366f1", devanagari: "रा", abbr: "Ra" },
  { name: "Jupiter", years: 16, color: "#eab308", devanagari: "गु", abbr: "Ju" },
  { name: "Saturn", years: 19, color: "#8b5cf6", devanagari: "श", abbr: "Sa" },
  { name: "Mercury", years: 17, color: "#10b981", devanagari: "बु", abbr: "Me" },
];

export const VIMSHOTTARI_LORD_MAP = Object.fromEntries(
  VIMSHOTTARI_LORDS.map((l, i) => [l.name, { ...l, index: i }])
);

export const DASHA_LEVEL_META = [
  { level: 0, code: "MD", name: "Mahadasha", devanagari: "महादशा", spanText: "Years" },
  { level: 1, code: "AD", name: "Antardasha", devanagari: "अन्तर्दशा", spanText: "Months" },
  { level: 2, code: "PD", name: "Pratyantardasha", devanagari: "प्रत्यन्तर", spanText: "Days" },
  { level: 3, code: "SD", name: "Sookshmadasha", devanagari: "सूक्ष्म", spanText: "Hours" },
  { level: 4, code: "PrD", name: "Pranadasha", devanagari: "प्राण", spanText: "Minutes" },
  { level: 5, code: "DeD", name: "Dehadasha", devanagari: "देह", spanText: "Seconds" },
];

export function parseTzOffsetHours(tz) {
  if (typeof tz === "number") return isNaN(tz) ? 0 : tz;
  if (!tz || typeof tz !== "string") return 0;
  const sign = tz.trim().startsWith("-") ? -1 : 1;
  const parts = tz.trim().replace(/^[+-]/, "").split(":");
  const h = parseInt(parts[0] || "0", 10) || 0;
  const m = parseInt(parts[1] || "0", 10) || 0;
  return sign * (h + m / 60.0);
}

export function formatDashaDate(jd, tzOffset = 0) {
  const offH = parseTzOffsetHours(tzOffset);
  const d = new Date((jd - 2440587.5) * 86400000 + offH * 3600000);
  const y = d.getUTCFullYear();
  const m = String(d.getUTCMonth() + 1).padStart(2, "0");
  const day = String(d.getUTCDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export function formatDashaDateTime(jd, tzOffset = 0, includeSeconds = false) {
  const offH = parseTzOffsetHours(tzOffset);
  const d = new Date((jd - 2440587.5) * 86400000 + offH * 3600000);
  const y = d.getUTCFullYear();
  const m = String(d.getUTCMonth() + 1).padStart(2, "0");
  const day = String(d.getUTCDate()).padStart(2, "0");
  const hh = String(d.getUTCHours()).padStart(2, "0");
  const mm = String(d.getUTCMinutes()).padStart(2, "0");
  if (includeSeconds) {
    const ss = String(d.getUTCSeconds()).padStart(2, "0");
    return `${y}-${m}-${day} ${hh}:${mm}:${ss}`;
  }
  return `${y}-${m}-${day} ${hh}:${mm}`;
}

export function formatDashaTimeOnly(jd, tzOffset = 0, includeSeconds = false) {
  const offH = parseTzOffsetHours(tzOffset);
  const d = new Date((jd - 2440587.5) * 86400000 + offH * 3600000);
  const hh = String(d.getUTCHours()).padStart(2, "0");
  const mm = String(d.getUTCMinutes()).padStart(2, "0");
  if (includeSeconds) {
    const ss = String(d.getUTCSeconds()).padStart(2, "0");
    return `${hh}:${mm}:${ss}`;
  }
  return `${hh}:${mm}`;
}

export function formatDashaDuration(days) {
  if (days >= 365.25) {
    const y = Math.floor(days / 365.25);
    const mo = Math.floor((days - y * 365.25) / 30.4375);
    return mo > 0 ? `${y}y ${mo}m` : `${y}y`;
  }
  if (days >= 30.4375) {
    const mo = Math.floor(days / 30.4375);
    const d = Math.round(days - mo * 30.4375);
    return d > 0 ? `${mo}m ${d}d` : `${mo}m`;
  }
  if (days >= 1) {
    const d = Math.floor(days);
    const h = Math.round((days - d) * 24);
    return h > 0 ? `${d}d ${h}h` : `${d}d`;
  }
  const totalMins = Math.round(days * 1440);
  const h = Math.floor(totalMins / 60);
  const m = totalMins % 60;
  if (h > 0) return m > 0 ? `${h}h ${m}m` : `${h}h`;
  return `${m}m`;
}

export function formatDashaRemaining(endJd, nowJd) {
  const ms = (endJd - nowJd) * 86400000;
  if (ms <= 0) return "";
  const totalDays = ms / 86400000;
  if (totalDays >= 365.25) {
    const y = Math.floor(totalDays / 365.25);
    const mo = Math.floor((totalDays - y * 365.25) / 30.4375);
    return mo > 0 ? `${y}y ${mo}mo left` : `${y}y left`;
  }
  if (totalDays >= 30) {
    const mo = Math.floor(totalDays / 30.4375);
    const d = Math.round(totalDays - mo * 30.4375);
    return d > 0 ? `${mo}mo ${d}d left` : `${mo}mo left`;
  }
  if (totalDays >= 1) {
    const d = Math.floor(totalDays);
    const h = Math.round((totalDays - d) * 24);
    return h > 0 ? `${d}d ${h}h left` : `${d}d left`;
  }
  const h = Math.floor(ms / 3600000);
  const m = Math.round((ms % 3600000) / 60000);
  if (h > 0) return m > 0 ? `${h}h ${m}m left` : `${h}h left`;
  return m > 0 ? `${m}m left` : "< 1m left";
}

export function getDashaSubperiods(parent, tzOffset = 0) {
  const meta = VIMSHOTTARI_LORD_MAP[parent.planet];
  if (!meta) return [];
  const startJd = parent.start_jd;
  const endJd = parent.end_jd;
  const totalDays = endJd - startJd;
  const parentYears = parent.years != null ? parent.years : (meta.years || 120);

  const children = [];
  let curJd = startJd;

  for (let i = 0; i < 9; i++) {
    const idx = (meta.index + i) % 9;
    const lord = VIMSHOTTARI_LORDS[idx];
    const subDays = totalDays * (lord.years / 120.0);
    const subYears = parentYears * (lord.years / 120.0);
    const subEndJd = curJd + subDays;

    children.push({
      planet: lord.name,
      start_jd: curJd,
      end_jd: subEndJd,
      years: subYears,
      start_date: formatDashaDate(curJd, tzOffset),
      end_date: formatDashaDate(subEndJd, tzOffset),
      start_time: formatDashaDateTime(curJd, tzOffset),
      end_time: formatDashaDateTime(subEndJd, tzOffset),
      duration_days: subDays,
      children: [],
    });

    curJd = subEndJd;
  }

  return children;
}

export function resolveActiveHierarchy(dashaList, targetJd, tzOffset = 0) {
  const activePath = [];
  let cur = dashaList.find((m) => targetJd >= m.start_jd && targetJd < m.end_jd) || dashaList[0];
  activePath.push({ ...cur, level: 0 });

  for (let lvl = 1; lvl <= 5; lvl++) {
    const children = (cur.children && cur.children.length === 9)
      ? cur.children
      : getDashaSubperiods(cur, tzOffset);
    cur = children.find((c) => targetJd >= c.start_jd && targetJd < c.end_jd) || children[0];
    activePath.push({ ...cur, level: lvl });
  }

  return activePath;
}

export function renderVimshottariDasha(containerEl, dashaList, moonLon, birthJd, wasm, tzOffset = 0) {
  if (!dashaList || dashaList.length === 0) {
    containerEl.hidden = true;
    return;
  }
  containerEl.hidden = false;

  const nakInfo = wasm.nakshatraInfo ? wasm.nakshatraInfo(moonLon) : null;
  const nowMs = Date.now();
  const nowJd = 2440587.5 + nowMs / 86400000;

  // Preserve or initialize explorer state
  let state = containerEl._dashaState;
  if (!state || state.dashaList !== dashaList) {
    state = {
      dashaList,
      moonLon,
      birthJd,
      tzOffset,
      nowJd,
      activeTargetJd: nowJd,
      focusedPath: [],
      activeLevel: 2, // Default to Level 2 (PD) for instant deep granularity
      viewMode: "explorer", // "explorer" | "tree"
    };
    containerEl._dashaState = state;
  } else {
    state.nowJd = nowJd;
    state.tzOffset = tzOffset;
  }

  // Active running hierarchy across all 6 levels
  const hierarchy = resolveActiveHierarchy(dashaList, state.activeTargetJd, tzOffset);

  // If focusedPath is empty, initialize with running MD -> AD -> PD
  if (!state.focusedPath || state.focusedPath.length === 0) {
    state.focusedPath = [hierarchy[0].planet, hierarchy[1].planet, hierarchy[2].planet];
    state.activeLevel = 2;
  }

  function render() {
    // 1. Birth balance from first mahadasha
    const firstMd = dashaList[0];
    const balanceDays = Math.max(0, (firstMd.end_jd - birthJd));
    const balYearsFloat = balanceDays / 365.256363;
    const balY = Math.floor(balYearsFloat);
    const balM = Math.floor((balYearsFloat - balY) * 12);
    const balD = Math.floor(((balYearsFloat - balY) * 12 - balM) * 30.4);

    // 2. 120-year timeline calculation
    const firstJd = dashaList[0].start_jd || birthJd;
    const lastJd = dashaList[dashaList.length - 1].end_jd || (firstJd + 120 * 365.256363);
    const totalDays = Math.max(1, lastJd - firstJd);
    let todayPercent = ((nowJd - firstJd) / totalDays) * 100;
    const showTodayMarker = todayPercent >= 0 && todayPercent <= 100;

    // Timeline segments
    let timelineSegments = "";
    for (const md of dashaList) {
      const mdSpan = md.end_jd - md.start_jd;
      const pct = Math.max(2, (mdSpan / totalDays) * 100);
      const info = VIMSHOTTARI_LORD_MAP[md.planet] || { color: "var(--accent)", abbr: md.planet.slice(0, 2), devanagari: "" };
      const isMdActive = nowJd >= md.start_jd && nowJd < md.end_jd;
      const isSelected = state.focusedPath[0] === md.planet;
      timelineSegments += `
        <div class="dasha-timeline-seg ${isMdActive ? 'active' : ''} ${isSelected ? 'selected' : ''}" 
             data-dasha-md="${md.planet}"
             style="width:${pct.toFixed(2)}%; background-color:${info.color}28; border-color:${info.color}88"
             title="${md.planet} Mahadasha (${formatDashaDateTime(md.start_jd, tzOffset)} to ${formatDashaDateTime(md.end_jd, tzOffset)}) · Click to explore">
          <span class="seg-label" style="color:${info.color}">${info.abbr}</span>
        </div>
      `;
    }

    // 3. Active Running Dasha Hero Banner (Connected pills for all running levels)
    let heroPillsHtml = "";
    hierarchy.forEach((h, idx) => {
      if (idx > 4) return; // MD, AD, PD, SD, PrD (5 levels prominent)
      const meta = DASHA_LEVEL_META[idx];
      const lordMeta = VIMSHOTTARI_LORD_MAP[h.planet] || { color: "var(--accent)", devanagari: "" };
      const rem = formatDashaRemaining(h.end_jd, nowJd);
      const isCurrentLevel = state.activeLevel === idx;
      heroPillsHtml += `
        ${idx > 0 ? '<span class="graha-arrow">→</span>' : ''}
        <button type="button" class="dasha-hero-pill-btn ${isCurrentLevel ? 'focused' : ''}" 
                data-dasha-hero-level="${idx}" 
                title="Level ${idx + 1}: ${meta.name} (${h.planet}) · Click to inspect"
                style="--lord-color: ${lordMeta.color}">
          <span class="pill-badge" style="background:${lordMeta.color}22; color:${lordMeta.color}; border-color:${lordMeta.color}66">
            ${meta.code}
          </span>
          <span class="pill-lord">${lordMeta.devanagari ? `<span class="pill-dev">${lordMeta.devanagari}</span> ` : ""}${h.planet}</span>
          ${rem ? `<span class="pill-rem">● ${rem}</span>` : ""}
        </button>
      `;
    });

    // 4. Breadcrumbs Trail
    let breadcrumbsHtml = `
      <button type="button" class="dasha-crumb-btn ${state.activeLevel === 0 ? 'active' : ''}" data-dasha-crumb="-1">
        All Mahadashas (MD)
      </button>
    `;
    for (let i = 0; i < state.focusedPath.length && i < state.activeLevel; i++) {
      const planetName = state.focusedPath[i];
      const lordMeta = VIMSHOTTARI_LORD_MAP[planetName] || { color: "var(--accent)", devanagari: "" };
      const meta = DASHA_LEVEL_META[i];
      const isLast = i === state.activeLevel - 1;
      breadcrumbsHtml += `
        <span class="dasha-crumb-sep">›</span>
        <button type="button" class="dasha-crumb-btn ${isLast ? 'active' : ''}" data-dasha-crumb="${i}">
          <span class="crumb-dot" style="background:${lordMeta.color}"></span>
          ${planetName} <span class="crumb-level">${meta.code}</span>
        </button>
      `;
    }

    // 5. Level Tabs (Level 1 to 6)
    let levelTabsHtml = "";
    DASHA_LEVEL_META.forEach((meta) => {
      const isCurrent = state.activeLevel === meta.level;
      levelTabsHtml += `
        <button type="button" class="dasha-level-tab ${isCurrent ? 'active' : ''}" data-dasha-level="${meta.level}">
          <span class="level-tab-code">${meta.code}</span>
          <span class="level-tab-name">${meta.name}</span>
          <span class="level-tab-span">(${meta.spanText})</span>
        </button>
      `;
    });

    // 6. Period Table Rows for current activeLevel
    let currentRows = [];
    let parentTitle = "";
    let parentRange = "";

    if (state.activeLevel === 0) {
      // Level 0: Show all 9 Mahadashas
      parentTitle = "All 9 Mahadashas (विंशोत्तरी महादशा चक्र)";
      parentRange = "120-Year Full Parashari Cycle";
      currentRows = dashaList.map((m) => {
        const isMdActive = nowJd >= m.start_jd && nowJd < m.end_jd;
        const isPast = nowJd >= m.end_jd;
        const status = isMdActive ? "active" : isPast ? "past" : "future";
        const rem = isMdActive ? formatDashaRemaining(m.end_jd, nowJd) : "";
        const dur = formatDashaDuration(m.end_jd - m.start_jd);
        return {
          planet: m.planet,
          level: 0,
          code: "MD",
          start_jd: m.start_jd,
          end_jd: m.end_jd,
          start_date_val: formatDashaDate(m.start_jd, tzOffset),
          end_date_val: formatDashaDate(m.end_jd, tzOffset),
          start_time_val: formatDashaTimeOnly(m.start_jd, tzOffset, false),
          end_time_val: formatDashaTimeOnly(m.end_jd, tzOffset, false),
          start_date: formatDashaDateTime(m.start_jd, tzOffset, false),
          end_date: formatDashaDateTime(m.end_jd, tzOffset, false),
          start_time: formatDashaDateTime(m.start_jd, tzOffset, false),
          end_time: formatDashaDateTime(m.end_jd, tzOffset, false),
          duration: dur,
          status,
          statusText: isMdActive ? `★ Active Now · ${rem}` : isPast ? "✓ Past" : "› Upcoming",
          canDrill: true,
        };
      });
    } else {
      // Deeper level: find parent node by walking focusedPath
      let parentNode = dashaList.find((m) => m.planet === state.focusedPath[0]) || dashaList[0];
      for (let lvl = 1; lvl < state.activeLevel; lvl++) {
        const childPlanet = state.focusedPath[lvl];
        const children = (parentNode.children && parentNode.children.length === 9)
          ? parentNode.children
          : getDashaSubperiods(parentNode, tzOffset);
        parentNode = children.find((c) => c.planet === childPlanet) || children[0];
      }

      const parentMeta = DASHA_LEVEL_META[state.activeLevel - 1];
      const currentMeta = DASHA_LEVEL_META[state.activeLevel];
      const withSec = state.activeLevel >= 4;
      parentTitle = `${state.focusedPath.slice(0, state.activeLevel).join(" → ")} (${currentMeta.name})`;
      parentRange = `${formatDashaDateTime(parentNode.start_jd, tzOffset, withSec)} to ${formatDashaDateTime(parentNode.end_jd, tzOffset, withSec)}`;

      const children = (parentNode.children && parentNode.children.length === 9)
        ? parentNode.children
        : getDashaSubperiods(parentNode, tzOffset);

      currentRows = children.map((c) => {
        const isActive = nowJd >= c.start_jd && nowJd < c.end_jd;
        const isPast = nowJd >= c.end_jd;
        const status = isActive ? "active" : isPast ? "past" : "future";
        const rem = isActive ? formatDashaRemaining(c.end_jd, nowJd) : "";
        const dur = formatDashaDuration(c.end_jd - c.start_jd);
        return {
          planet: c.planet,
          level: state.activeLevel,
          code: currentMeta.code,
          start_jd: c.start_jd,
          end_jd: c.end_jd,
          start_date_val: formatDashaDate(c.start_jd, tzOffset),
          end_date_val: formatDashaDate(c.end_jd, tzOffset),
          start_time_val: formatDashaTimeOnly(c.start_jd, tzOffset, withSec),
          end_time_val: formatDashaTimeOnly(c.end_jd, tzOffset, withSec),
          start_date: formatDashaDateTime(c.start_jd, tzOffset, withSec),
          end_date: formatDashaDateTime(c.end_jd, tzOffset, withSec),
          start_time: formatDashaDateTime(c.start_jd, tzOffset, withSec),
          end_time: formatDashaDateTime(c.end_jd, tzOffset, withSec),
          duration: dur,
          status,
          statusText: isActive ? `★ Active Now · ${rem}` : isPast ? "✓ Past" : "› Upcoming",
          canDrill: state.activeLevel < 5,
        };
      });
    }

    // Build Explorer Table HTML
    let tableRowsHtml = "";
    currentRows.forEach((r) => {
      const lordMeta = VIMSHOTTARI_LORD_MAP[r.planet] || { color: "var(--accent)", devanagari: "" };
      const isSelected = state.focusedPath[state.activeLevel] === r.planet;
      tableRowsHtml += `
        <tr class="dasha-table-row ${r.status} ${isSelected ? 'selected' : ''}" 
            data-dasha-drill="${r.planet}" 
            data-can-drill="${r.canDrill}">
          <td class="dasha-col-planet">
            <span class="dasha-planet-pill" style="--pill-color: ${lordMeta.color}">
              <span class="dasha-planet-dot" style="background:${lordMeta.color}"></span>
              ${lordMeta.devanagari ? `<span class="dasha-planet-dev">${lordMeta.devanagari}</span>` : ""}
              <strong class="dasha-planet-name">${r.planet}</strong>
              <span class="dasha-level-tag">${r.code}</span>
            </span>
          </td>
          <td class="dasha-col-dates">
            <span class="dasha-date-start">
              <span class="d-date">${r.start_date_val}</span>
              <span class="d-time">${r.start_time_val}</span>
            </span>
            <span class="dasha-date-sep">→</span>
            <span class="dasha-date-end">
              <span class="d-date">${r.end_date_val}</span>
              <span class="d-time">${r.end_time_val}</span>
            </span>
          </td>
          <td class="dasha-col-duration">
            <span class="dasha-duration-badge">${r.duration}</span>
          </td>
          <td class="dasha-col-status">
            <span class="dasha-status-pill ${r.status}">${r.statusText}</span>
          </td>
          <td class="dasha-col-action">
            ${r.canDrill 
              ? `<button type="button" class="dasha-drill-btn" data-dasha-drill="${r.planet}" title="Explore deeper sub-periods">
                  Explore <span class="arrow">›</span>
                 </button>` 
              : `<span class="dasha-leaf-tag">Deha (Final)</span>`
            }
          </td>
        </tr>
      `;
    });

    // 7. Tree View HTML (Collapsible hierarchy)
    let treeViewHtml = `<div class="dasha-tree">`;
    dashaList.forEach((md) => {
      const isMdActive = nowJd >= md.start_jd && nowJd < md.end_jd;
      const isPast = nowJd >= md.end_jd;
      const statusClass = isMdActive ? "active" : isPast ? "past" : "future";
      const statusLabel = isMdActive ? "Active Now" : isPast ? "Past" : "Future";
      const info = VIMSHOTTARI_LORD_MAP[md.planet] || { color: "var(--accent)", devanagari: "" };
      const mdChildren = (md.children && md.children.length === 9) ? md.children : getDashaSubperiods(md, tzOffset);

      treeViewHtml += `
        <details class="dasha-node md-node ${statusClass}" ${isMdActive ? "open" : ""}>
          <summary class="dasha-node-summary">
            <span class="dasha-badge" style="background:${info.color}22; color:${info.color}; border-color:${info.color}55">
              ${info.devanagari ? info.devanagari + ' ' : ''}${md.planet}
            </span>
            <span class="dasha-period-name"><strong>${md.planet} Mahadasha</strong></span>
            <span class="dasha-dates">${formatDashaDateTime(md.start_jd, tzOffset)} → ${formatDashaDateTime(md.end_jd, tzOffset)}</span>
            <span class="dasha-status-tag ${statusClass}">${statusLabel}</span>
          </summary>
          <div class="dasha-children">
      `;

      mdChildren.forEach((ad) => {
        const isAdActive = nowJd >= ad.start_jd && nowJd < ad.end_jd;
        const isAdPast = nowJd >= ad.end_jd;
        const adStatusClass = isAdActive ? "active" : isAdPast ? "past" : "future";
        const adInfo = VIMSHOTTARI_LORD_MAP[ad.planet] || { color: "var(--accent)", devanagari: "" };
        const adChildren = (ad.children && ad.children.length === 9) ? ad.children : getDashaSubperiods(ad, tzOffset);

        treeViewHtml += `
          <details class="dasha-node ad-node ${adStatusClass}" ${isAdActive ? "open" : ""}>
            <summary class="dasha-node-summary sub">
              <span class="dasha-badge small" style="background:${adInfo.color}22; color:${adInfo.color}">
                ${adInfo.devanagari ? adInfo.devanagari + ' ' : ''}${ad.planet}
              </span>
              <span class="dasha-period-name">${md.planet} – ${ad.planet} (AD)</span>
              <span class="dasha-dates">${formatDashaDateTime(ad.start_jd, tzOffset)} → ${formatDashaDateTime(ad.end_jd, tzOffset)}</span>
              <span class="dasha-status-tag ${adStatusClass}">${isAdActive ? "Active" : ""}</span>
            </summary>
            <div class="dasha-children sub">
              <div class="pd-grid">
        `;

        adChildren.forEach((pd) => {
          const isPdActive = nowJd >= pd.start_jd && nowJd < pd.end_jd;
          const pdClass = isPdActive ? "active" : "";
          const pdInfo = VIMSHOTTARI_LORD_MAP[pd.planet] || { color: "var(--accent)" };
          treeViewHtml += `
            <div class="pd-item ${pdClass}">
              <div class="pd-header">
                <span class="pd-name" style="color:${pdInfo.color}"><strong>${pd.planet}</strong> <small>PD</small></span>
                ${isPdActive ? '<span class="pd-active-dot">● Active</span>' : ""}
              </div>
              <span class="pd-dates">${formatDashaDateTime(pd.start_jd, tzOffset)} → ${formatDashaDateTime(pd.end_jd, tzOffset)}</span>
              <span class="pd-dur">${formatDashaDuration(pd.end_jd - pd.start_jd)}</span>
            </div>
          `;
        });

        treeViewHtml += `
              </div>
            </div>
          </details>
        `;
      });

      treeViewHtml += `
          </div>
        </details>
      `;
    });
    treeViewHtml += `</div>`;

    // 8. Assemble Full Component HTML
    let fullHtml = `
      <div class="dasha-overview">
        <!-- Hero Running Dasha Banner -->
        <div class="dasha-hero-banner">
          <div class="dasha-hero-left">
            <div class="dasha-hero-badge-row">
              <span class="dasha-hero-pill">● Current Active Vimshottari Hierarchy</span>
              <span class="dasha-hero-sub-text">Classical 5-Level Breakdown</span>
            </div>
            <div class="dasha-hero-pills-row">
              ${heroPillsHtml}
            </div>
            <p class="dasha-hero-sub">
              Mahadasha: <strong>${hierarchy[0].planet}</strong> (${formatDashaDateTime(hierarchy[0].start_jd, tzOffset)} to ${formatDashaDateTime(hierarchy[0].end_jd, tzOffset)}) · 
              Antardasha: <strong>${hierarchy[1].planet}</strong> · 
              Pratyantar: <strong>${hierarchy[2].planet}</strong> · 
              Sookshma: <strong>${hierarchy[3].planet}</strong> · 
              Prana: <strong>${hierarchy[4].planet}</strong>
            </p>
          </div>
          <div class="dasha-hero-stats">
            <div class="stat-box">
              <span class="stat-label">Birth Balance</span>
              <span class="stat-value">${balY}y ${balM}m ${balD}d</span>
              <span class="stat-sub">${firstMd.planet} Dasha Lord</span>
            </div>
            <div class="stat-box">
              <span class="stat-label">Moon Nakshatra</span>
              <span class="stat-value">${nakInfo ? nakInfo.name : "–"}</span>
              <span class="stat-sub">Pada ${nakInfo ? nakInfo.pada : "–"} · Lord ${nakInfo ? nakInfo.lord : "–"}</span>
            </div>
          </div>
        </div>

        <!-- 120-Year Timeline -->
        <div class="dasha-timeline-card">
          <div class="timeline-header">
            <span class="timeline-title">120-Year Vimshottari Life Cycle Timeline</span>
            <span class="timeline-legend">Click any period to explore</span>
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

      <!-- Level Explorer Panel -->
      <div class="dasha-explorer-card">
        <div class="dasha-explorer-header">
          <div class="dasha-nav-group">
            <!-- Breadcrumbs -->
            <div class="dasha-breadcrumb-wrap">
              ${breadcrumbsHtml}
            </div>
          </div>
          <div class="dasha-view-toggles">
            <button type="button" class="btn-view-toggle ${state.viewMode === 'explorer' ? 'active' : ''}" data-dasha-mode="explorer" title="Interactive Level Explorer">
              ⚡ Level Explorer
            </button>
            <button type="button" class="btn-view-toggle ${state.viewMode === 'tree' ? 'active' : ''}" data-dasha-mode="tree" title="Collapsible Tree View">
              𖡡 Full Tree
            </button>
          </div>
        </div>

        <!-- Level Selector Tabs -->
        <div class="dasha-level-tabs-wrap">
          <div class="dasha-level-tabs">
            ${levelTabsHtml}
          </div>
        </div>

        <!-- Content Area: Explorer Table or Tree View -->
        ${
          state.viewMode === "explorer"
            ? `
          <div class="dasha-level-context-bar">
            <div class="context-title-group">
              <span class="context-level-badge">Level ${state.activeLevel + 1}: ${DASHA_LEVEL_META[state.activeLevel].name}</span>
              <h4 class="context-heading">${parentTitle}</h4>
              <span class="context-range">${parentRange}</span>
            </div>
            <div class="context-hint">
              ${state.activeLevel < 5 ? "Click any row to drill down into sub-periods ›" : "Deepest Classical Dehadasha Level"}
            </div>
          </div>
          <div class="table-scroll dasha-table-wrap">
            <table class="dasha-table">
              <thead>
                <tr>
                  <th>Period (Lord)</th>
                  <th>Date &amp; Time Range</th>
                  <th>Duration</th>
                  <th>Status &amp; Time Remaining</th>
                  <th style="text-align:right">Action</th>
                </tr>
              </thead>
              <tbody>
                ${tableRowsHtml}
              </tbody>
            </table>
          </div>
          <div class="dasha-table-legend">
            <span><span class="dot-legend active">★</span> Active Now</span>
            <span><span class="dot-legend past">✓</span> Past</span>
            <span><span class="dot-legend upcoming">›</span> Upcoming</span>
            <span class="legend-note">All 6 levels supported: MD → AD → PD → SD → PrD → DeD</span>
          </div>
          `
            : treeViewHtml
        }
      </div>
    `;

    containerEl.innerHTML = fullHtml;
  }

  // Initial render
  render();

  // Attach interactive click handlers via delegation
  containerEl.onclick = (e) => {
    // 1. Breadcrumbs
    const crumbBtn = e.target.closest("[data-dasha-crumb]");
    if (crumbBtn) {
      e.preventDefault();
      const crumbIdx = parseInt(crumbBtn.dataset.dashaCrumb, 10);
      if (crumbIdx === -1) {
        state.activeLevel = 0;
        state.focusedPath = [];
      } else {
        state.activeLevel = crumbIdx + 1;
        state.focusedPath = state.focusedPath.slice(0, state.activeLevel);
      }
      render();
      return;
    }

    // 2. Level tabs
    const levelTab = e.target.closest("[data-dasha-level]");
    if (levelTab) {
      e.preventDefault();
      const lvl = parseInt(levelTab.dataset.dashaLevel, 10);
      state.activeLevel = lvl;
      if (state.focusedPath.length < lvl) {
        for (let i = state.focusedPath.length; i < lvl; i++) {
          state.focusedPath.push(hierarchy[i].planet);
        }
      } else if (state.focusedPath.length > lvl) {
        state.focusedPath = state.focusedPath.slice(0, lvl);
      }
      render();
      return;
    }

    // 3. Timeline Mahadasha click
    const timelineSeg = e.target.closest("[data-dasha-md]");
    if (timelineSeg) {
      e.preventDefault();
      const mdPlanet = timelineSeg.dataset.dashaMd;
      state.focusedPath = [mdPlanet];
      state.activeLevel = 1; // Show Antardashas of this MD
      render();
      return;
    }

    // 4. Hero pill click
    const heroPill = e.target.closest("[data-dasha-hero-level]");
    if (heroPill) {
      e.preventDefault();
      const lvl = parseInt(heroPill.dataset.dashaHeroLevel, 10);
      state.activeLevel = lvl;
      state.focusedPath = hierarchy.slice(0, lvl).map(h => h.planet);
      render();
      return;
    }

    // 5. Table row or drill button click
    const drillEl = e.target.closest("[data-dasha-drill]");
    if (drillEl) {
      e.preventDefault();
      const planet = drillEl.dataset.dashaDrill;
      const canDrill = drillEl.dataset.canDrill !== "false";
      if (!canDrill || state.activeLevel >= 5) return;

      state.focusedPath = state.focusedPath.slice(0, state.activeLevel);
      state.focusedPath.push(planet);
      state.activeLevel += 1;
      render();
      return;
    }

    // 6. View mode toggle (explorer vs tree)
    const modeBtn = e.target.closest("[data-dasha-mode]");
    if (modeBtn) {
      e.preventDefault();
      state.viewMode = modeBtn.dataset.dashaMode;
      render();
      return;
    }
  };
}

function table(el, head, rows) {
  el.innerHTML =
    `<thead><tr>${head.map((h) => `<th>${h}</th>`).join("")}</tr></thead>` +
    `<tbody>${rows.map((r) => `<tr>${r.join("")}</tr>`).join("")}</tbody>`;
}
