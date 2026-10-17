# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While the
version is 0.0.x the API is experimental and any release may break it.

## [Unreleased]

## [0.0.12]

### Added

- **Khagole Vedic Astrometry Engine**:
  - Full Shodashavarga divisional chart calculation engine (D1 Rasi, D9 Navamsha, and D2 through D60).
  - High-precision Vimshottari Dasha calculations supporting True Sidereal Solar Year (TSSY), alongside Savana (360-day) and standard sidereal year.
  - Sub-millisecond Panchanga limbs: Tithi, Vara, Nakshatra with Pada, Yoga, and Karana.
  - Classical Navagrahas (Sun through Ketu) and Lagna ascendant point calculation.
  - Interactive North Indian (diamond) and South Indian (box) SVG chart visualizers with Google Material 3 typography.
  - Integrated Open-Meteo & Nominatim geocoding and offline historical UTC offset detection.
  - Pure Rust core on NASA JPL DE440s ephemerides, with WebAssembly and Python bindings.
  - Model Context Protocol (MCP) server `khagole-mcp` for agentic LLM astrological reasoning.
- Ba Zi: the `date` option gives the year, month and day pillars of a date (`flow`: 流年,
  流月, 流日, with the jie that opened the month), from an embedded table of the jie terms of
  1900-2100 (no kernel needed).

## [0.0.11]

### Added

- Zi Wei Dou Shu: the `date` option casts the horoscope down to the month (流月) and day
  (流日) of a date, read in an embedded lunar calendar for 1900-2100 (no kernel needed).
- 120 charts and 80 horoscopes checked against iztro, the reference implementation of the
  common school.

## [0.0.10]

### Added

- Zi Wei Dou Shu: each palace's `minor_stars` (the 38 minor stars of the common school),
  `gods` (the 長生, 博士, 歲前 and 將前 cycles) and `flying` transformations of its stem;
  the `year` option adds the `horoscope` of that year: the decade (or childhood) limit, the
  small limit and the year, each with its palace names, transformations and moving stars,
  and the year's gods.
- Ba Zi: the `year` option adds the year's pillar (`annual`), seen from the Day Master.

## [0.0.9]

### Added

- Zi Wei Dou Shu: `zi_wei(calendar, birth, longitude, utc_offset_minutes, options)` (no
  kernel needed; Python `zi_wei`, JavaScript `ziWei`) casts the Purple Star chart from a
  chart's Chinese calendar: life and body palaces, palace stems, the five-element bureau,
  the 14 major stars and 14 auxiliary and malefic stars with their brightness, the natal
  Four Transformations, the life and body masters, and (given the native's sex) the decade
  and small limits. Leap months split at the 15th by default. See `docs/chinese.md`.

## [0.0.8]

### Added

- Chinese calendar: `ChartRequest::chinese_calendar` (`chinese_calendar` in Python and
  JavaScript) adds a `chinese_calendar` key to a chart: the solar terms from the last 立春
  to the next jie term, the lunar months around the date (leap months by the modern rule,
  including the 2033 case) and the equation of time. Also `chinese_calendar(kernels, instant)`.
- Ba Zi: `bazi(calendar, birth, longitude, utc_offset_minutes, options)` (no kernel needed;
  Python and JavaScript `bazi`) casts the Four Pillars on true solar or civil time, with
  either Zi-hour convention: hidden stems, Ten Gods, NaYin, life stages, element balance,
  the lunar date and, given the native's sex, the luck pillars. See `docs/chinese.md`.

## [0.0.7]

### Added

- Time-lords: `time_lords(birth, sun_longitude, ascendant_longitude, start, end)` (no kernel
  needed; Python `time_lords`, JavaScript `timeLords`) gives the annual profections (age,
  sign, lord of the year, activated house) and the firdaria with their sub-periods, in the
  day or night sequence with the nodes last, after Bonatti.
- `chart_dignities(chart, scheme)` (no kernel needed; Python `chart_dignities`, JavaScript
  `chartDignities`) judges a computed chart's condition, sect and receptions again under the
  Lilly or the Dorothean scheme.

### Fixed

- The Python type stubs list `dignity_scheme` and `quesited_house`.

## [0.0.6]

### Added

- Essential dignities on every chart. Each of the seven planets carries a `condition`: its
  domicile, exaltation, triplicity, bound and face (with the lords of its place), detriment,
  fall or peregrine, Lilly's points, its sect and whether it is in sect, above the horizon,
  cazimi, combust or under the beams, oriental or occidental, and fast, slow or stationary.
  The chart adds its `sect`, the `receptions` among the planets, `antiscia` and
  contra-antiscia, and its `planetary_hours` (left out of transit skies). Sources and rules:
  `docs/dignities.md`.
- `dignity_scheme` request option: `lilly` (default; Lilly's triplicities and Ptolemaic
  terms) or `dorothean` (three triplicity lords and the Egyptian bounds).
- Horary judgment. `calculate_horary_chart` takes the quesited house (Python
  `quesited_house=`, JavaScript `quesited_house`) and adds `horary_data.judgment`: the
  significators of the querent and of the quesited, the aspect they perfect within their
  signs, the Moon's aspect to the quesited's lord, prohibition, refranation (a station found
  in the ephemeris), translation and collection of light, and the receptions between the
  significators.
- Considerations before judgment: the Moon in the via combusta (`MOON_VIA_COMBUSTA`), in
  the last degrees of her sign (`MOON_LATE_DEGREES`), and the lord of the Ascendant combust
  (`ASC_RULER_COMBUST`).

### Changed

- `ChartRequest` has a `dignity_scheme` field, and `calculate_horary_chart` a third argument
  (`quesited_house`).
- The demo site has a dignity scheme selector and a dignity column.

## [0.0.5]

### Added

- Degree qualities after William Lilly (*Christian Astrology*, 1659, p. 116):
  `degree_qualities` gives the masculine or feminine, light, dark, smoky or void, pitted,
  lame (azimene) and fortune-increasing qualities of a longitude's degree, and
  `degree_quality_table` returns the whole table. Python: `degree_qualities` /
  `degree_quality_table`. JavaScript: `degreeQualities` / `degreeQualityTable`.
- Elections weigh the degree qualities of the Moon and the Ascendant: `MOON_PITTED_DEGREE`,
  `MOON_AZIMENE_DEGREE`, `MOON_FORTUNE_DEGREE`, `ASC_PITTED_DEGREE`, `ASC_AZIMENE_DEGREE`,
  `ASC_FORTUNE_DEGREE`, `ASC_LIGHT_DEGREE` and `ASC_DARK_DEGREE`. Scores change for moments
  where they apply.

## [0.0.4]

### Added

- Electional astrology. `calculate_election_chart` scores a moment by the traditional
  electional rules and returns stable factor codes with weights. The rules cover the Moon,
  the Ascendant and its ruler, benefics and malefics on the angles, retrograde Mercury and
  Venus, the house of the matter and its significator, the planetary hour and, optionally,
  contacts with a natal chart. `search_elections` finds the best windows over up to 92 days
  at a place, with purpose presets and filters (void-of-course Moon, retrogrades, daytime,
  local hours with daylight saving time). Python: `Engine.election` / `Engine.elections`.
  JavaScript: `engine.election` / `engine.elections`.

## [0.0.3]

### Added

- Koch, Regiomontanus, Campanus, Topocentric, Alcabitius, Morinus and Vehlow house
  systems. These codes used to fall back to Placidus silently; Koch now falls back to
  Porphyry inside the polar circles, where it is undefined.
- Landing site with a live WebAssembly demo, API guide and architecture docs, security
  policy and an expanded contributing guide.

### Fixed

- The npm package now ships the licenses and NOTICE (Skyfield's MIT notice).

## [0.0.2]

### Added

- Python wheels for musllinux (Alpine and other musl-based distributions).

## [0.0.1]

### Added

- Initial experimental release: the complete Khagole chart pipeline on JPL SPK
  kernels (natal, horary, transit, synastry and derived charts), with Python and
  WebAssembly bindings.
