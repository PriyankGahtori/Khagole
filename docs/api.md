# API guide

The engine has the same entry points in every language. Requests and results have the
same shape everywhere: results serialize to the JSON of the Khagole API, with
snake_case keys.

| Rust | Python | JavaScript |
|---|---|---|
| `calculate_chart` | `Engine.chart` | `engine.chart` |
| `calculate_horary_chart` | `Engine.horary` | `engine.horary` |
| `calculate_transit_chart` | `Engine.transit` | `engine.transit` |
| `calculate_synastry` | `synastry` | `synastry` |
| `calculate_derived_chart` | `derived_chart` | `derivedChart` |
| `calculate_election_chart` | `Engine.election` | `engine.election` |
| `search_elections` | `Engine.elections` | `engine.elections` |
| `chinese_calendar` | `Engine.chinese_calendar` | `engine.chineseCalendar` |
| `chart_dignities` | `chart_dignities` | `chartDignities` |
| `time_lords` | `time_lords` | `timeLords` |
| `degree_qualities` | `degree_qualities` | `degreeQualities` |
| `degree_quality_table` | `degree_quality_table` | `degreeQualityTable` |
| `bazi` | `bazi` | `bazi` |
| `zi_wei` | `zi_wei` | `ziWei` |
| `UtcInstant::julian_day` | `julian_day` | `julianDay` |
| `Spk::excerpt` | `excerpt_kernel` | `excerptKernel` |
| `KernelSet::kernels` / `coverage` / `for_jd` | `Engine.kernels` / `coverage` / `supports` | `engine.kernels` / `coverage` / `supports` |

The chart, horary, transit and election calculations and the Chinese calendar need JPL
kernels (see [Ephemerides](ephemerides.md)). The other entry points work on already
computed charts or calendars, or on plain longitudes, and do not.

## A chart request

| Field | Meaning | Default |
|---|---|---|
| moment | UTC instant: `UtcInstant` in Rust, `datetime` or ISO string in Python, ISO string `utc` in JS (a real calendar date, year 0–200000) | required |
| latitude | geographic latitude in degrees, north positive, within ±90 | required |
| longitude | geographic longitude in degrees, east positive, within ±360 | required |
| house system | `P` Placidus, `K` Koch, `R` Regiomontanus, `C` Campanus, `T` Topocentric (Polich-Page), `B` Alcabitius, `M` Morinus, `O` Porphyry, `E` Equal, `V` Vehlow, `W` Whole Sign (the Swiss Ephemeris letters) | `P` |
| zodiac type | `tropical` or `sidereal` | `tropical` |
| ayanamsa | sidereal reference, see below (ignored for tropical charts) | `galcent_0sag` |
| orb settings | overrides merged over the default orbs, see below | none |
| dignity scheme | `lilly` (Lilly's triplicities and terms) or `dorothean` (three triplicity lords, Egyptian bounds), see [dignities](dignities.md) | `lilly` |
| chinese calendar | also compute the Chinese calendar of the moment (`chinese_calendar` key), see [Chinese astrology](chinese.md) | `false` |

Times are always UTC. Convert civil time (with its time zone and daylight saving time) to
UTC before calling the engine. Only the first letter of the house system is used, and
unknown codes fall back to Placidus. A latitude or longitude that is not a finite number
in range is an `invalid_input` error. Koch falls back to Porphyry inside the polar circles,
where it is undefined. Morinus cusps 1 and 10 are not the Ascendant and Midheaven. Zodiac types other than `sidereal` are tropical.

```rust
use khagole_engine::ephemeris::{Kernel, KernelSet, Spk};
use khagole_engine::{calculate_chart, ChartRequest, UtcInstant};

let mut kernels = KernelSet::new();
kernels.push(Kernel::new("de440s.bsp", Spk::open("kernels/de440s.bsp")?)?);

let mut request = ChartRequest::new(UtcInstant::parse("1987-05-17T14:30:00Z")?, 41.9, 12.5);
request.house_system = "W";
request.zodiac_type = "sidereal";
request.ayanamsa = "lahiri";
let chart = calculate_chart(&kernels, &request)?;
```

```python
import khagole_engine as ace

engine = ace.Engine(["kernels/de440s.bsp"])
chart = engine.chart("1987-05-17T14:30:00Z", 41.9, 12.5,
                     house_system="W", zodiac_type="sidereal", ayanamsa="lahiri")
```

```js
const chart = engine.chart({
  utc: "1987-05-17T14:30:00Z", latitude: 41.9, longitude: 12.5,
  house_system: "W", zodiac_type: "sidereal", ayanamsa: "lahiri",
});
```

### Ayanamsas

| Code | Name |
|---|---|
| `lahiri` | Lahiri (Chitra Paksha) |
| `fagan_bradley` | Fagan-Bradley |
| `krishnamurti` | Krishnamurti (KP) |
| `raman` | B.V. Raman |
| `true_citra` | True Chitra |
| `yukteshwar` | Sri Yukteswar |
| `deluce` | De Luce |
| `ushashashi` | Usha-Shashi |
| `jn_bhasin` | J.N. Bhasin |
| `aldebaran_15tau` | Aldebaran 15° Taurus |
| `hipparchos` | Hipparchos |
| `sassanian` | Sassanian |
| `galcent_0sag` | Galactic Center 0° Sagittarius (default) |
| `j2000` | J2000.0 Epoch |

Codes are case-insensitive. Unknown codes use the default.

### Orb settings

Pass a JSON object (a `serde_json::Value` in Rust, a dict in Python, an object in JS).
Only these keys are read, and the orb tables are merged key by key over the defaults:

```json
{
  "method": "moiety_hybrid",
  "fixed_star_orb": 1.5,
  "aspect_orbs": { "Conjunction": 8.0, "Sextile": 6.0, "Square": 7.0, "Trine": 8.0,
                   "Opposition": 8.0, "Semi-Sextile": 2.5, "Quincunx": 3.0 },
  "planet_orbs": { "Sun": 15.0, "Moon": 12.0, "Mercury": 7.0, "Venus": 7.0, "Mars": 8.0,
                   "Jupiter": 9.0, "Saturn": 9.0, "Uranus": 5.0, "Neptune": 5.0,
                   "Pluto": 5.0, "North Node": 5.0, "South Node": 5.0, "Chiron": 5.0,
                   "Lilith": 5.0, "Ascendant": 5.0, "Midheaven": 5.0 }
}
```

`method` selects how the maximum orb of an aspect between two bodies is computed:

- `moiety_traditional`: the mean of the two planets' orbs (moieties);
- `moiety_aspect_weighted`: the moiety scaled by the aspect's orb relative to the
  conjunction;
- `moiety_hybrid` (default): the moiety, capped by the aspect's orb;
- anything else: the aspect's orb alone.

The settings in effect are echoed back in the chart's `orb_settings`.

## The chart

A chart is an object with these keys, in this order:

| Key | Content |
|---|---|
| `house_system`, `zodiac_type` | echoed from the request |
| `ayanamsa`, `ayanamsa_name`, `ayanamsa_value`, `ayanamsa_formatted` | sidereal reference at the chart moment (`null` for tropical charts) |
| `precession_rate_arcsec_yr` | precession rate used for the fixed stars |
| `orb_settings` | the orb settings in effect |
| `planets` | Sun to Pluto, North and South Node, Chiron, Lilith, Ascendant, Midheaven |
| `unavailable_bodies` | expected bodies that could not be computed (Chiron outside 1849–2151) |
| `houses` | the twelve cusps |
| `aspects` | aspects between chart points, then fixed-star conjunctions |
| `fixed_stars` | fixed stars conjunct a chart point |
| `arabic_parts` | the lots (Fortune, Spirit, …) with the formula used |
| `temperament` | traditional temperament: scores, qualities and their breakdown |
| `lunar_status` | phase, illumination, age, speed, dignity and lunar mansion |
| `sect` | `diurnal` (the Sun above the horizon) or `nocturnal` |
| `dignity_scheme` | the scheme the dignities were judged by |
| `receptions` | receptions among the seven planets: `{planet, receiver, dignities, mutual, aspect}` |
| `antiscia` | antiscion and contra-antiscion contacts: `{body1, body2, kind, orb}` |
| `planetary_hours` | the planetary day and hour (left out of transit skies) |
| `chinese_calendar` | only when requested: solar terms, lunar months, equation of time (see [Chinese astrology](chinese.md)) |

The keys from `sect` on and each planet's `condition` are additions to the reference
implementation; [dignities](dignities.md) gives the tables and rules of the first five.

A placement in `planets` (houses, fixed stars and lots use a similar shape):

```json
{ "name": "Sun", "symbol": "☉", "sign": "Capricorn", "sign_symbol": "♑",
  "degree": 10, "minute": 22, "ecliptic_longitude": 280.36891840215475,
  "house": 10, "speed": 1.0194357508121357, "is_retrograde": false,
  "symbolic_degree": 11 }
```

The seven planets also carry a `condition`:

```json
"condition": {
  "essential": { "domicile": false, "exaltation": false, "triplicity": true, "bound": false,
                 "face": false, "detriment": false, "fall": false, "peregrine": false,
                 "score": 3,
                 "lords": { "domicile": "Saturn", "exaltation": "Mars",
                            "triplicity": ["Venus", "Moon"], "bound": "Jupiter",
                            "face": "Mars" } },
  "sect": "diurnal", "in_sect": true, "above_horizon": true,
  "solar_phase": null, "orientality": null, "motion": "fast" }
```

An aspect:

```json
{ "body1": "Sun", "body2": "Moon", "aspect_type": "Sextile", "symbol": "⚹",
  "angle": 60.0, "orb": 2.95, "max_orb": 6.0, "is_major": true, "is_applying": true }
```

`is_applying` is always `true` in chart aspects, as in the reference implementation, which
does not evaluate it there. Horary charts compute the Moon's applying and separating
aspects from its speed (`horary_data`).

Longitudes are apparent geocentric ecliptic longitudes of date, in degrees (sidereal charts
subtract the ayanamsa). In Rust, the fields are typed: see [`Chart`] and [`Placement`] on
docs.rs.

[`Chart`]: https://docs.rs/khagole-engine/latest/khagole_engine/struct.Chart.html
[`Placement`]: https://docs.rs/khagole-engine/latest/khagole_engine/struct.Placement.html

## Other entry points

- **Horary**: the chart for the moment of the question, plus `horary_data` with the
  planetary day and hour (computed from the actual sunrise and sunset at the location),
  significators, the Moon's applying and separating aspects, void of course and the
  considerations before judgment. An optional quesited house (1–12; `quesited_house` in
  Python and JavaScript, the third argument of `calculate_horary_chart` in Rust) adds the
  judgment: see below.
- **Transit**: the sky at a moment and place, with the aspects it forms to a natal chart's
  `planets` (`cross_aspects`).
- **Synastry**: cross-aspects from chart B's `planets` to chart A's `planets`.
- **Derived chart**: a stored chart turned so that radix house *n* (1–12) becomes the first
  house, with the meaning of each derived house. Fields that are not turned are kept.

## Horary judgment

`horary_data.judgment` names the significators after Lilly: the lord of the Ascendant for
the querent (with the Moon as co-significator) and the lord of the quesited house's cusp. It
then reports what brings the matter to perfection or prevents it:

| Key | Content |
|---|---|
| `perfection` | the aspect the significators perfect: `{applying, to, aspect, degrees, days}` |
| `moon_perfection` | the Moon's aspect to the quesited's significator |
| `prohibition` | a third planet that perfects with a significator first: `{by, significator, aspect, days}` |
| `refranation` | a significator that stations before the perfection: `{planet, aspect, days}` |
| `translation` | without perfection, a lighter planet separating from one significator and applying to the other |
| `collection` | without perfection, a heavier planet both significators apply to |
| `receptions` | receptions between the significators |
| `same_significator` | one planet rules both houses (judge by the Moon) |

An aspect perfects only while both planets stay in their present signs. Perfection is
projected from the present speeds, like the Moon's applying aspects; refranation checks the
ephemeris for a station. The considerations before judgment also include the Moon in the via
combusta (`MOON_VIA_COMBUSTA`), in the last degrees of a sign (`MOON_LATE_DEGREES`) and the
lord of the Ascendant combust (`ASC_RULER_COMBUST`).

## Judging a stored chart again

`chart_dignities(chart, scheme)` (Python `chart_dignities`, JavaScript `chartDignities`; no
kernel needed) takes a chart JSON as `calculate_chart` returns it and judges its dignities
under `scheme` (`lilly` or `dorothean`): `{dignity_scheme, sect, conditions: [{name,
condition}], receptions}`. A chart computed and stored with one scheme can so be shown with
the other without computing it again.

## Time-lords

`time_lords(birth, sun_longitude, ascendant_longitude, start, end)` (Python `time_lords`,
JavaScript `timeLords`; no kernel needed) unfolds a nativity's time-lords for the years that
overlap `start`..`end`:

- `profections`: one per year of life, `{age, start, end, sign, lord, house}`. The
  Ascendant moves one sign a year; the lord of the year rules that sign, and `house` is the
  house it activates (whole signs).
- `firdaria`: `{lord, start, end, sub_periods}`. A day birth runs Sun 10 years, Venus 8,
  Mercury 13, Moon 9, Saturn 11, Jupiter 12, Mars 7; a night birth starts from the Moon in
  the same order; the North Node (3) and South Node (2) close both rounds of 75 years, as in
  Bonatti. Each planet's period has seven equal sub-periods, from the planet itself in
  Chaldean order.

Years are tropical years from the moment of birth. `diurnal` says which sequence was used.
A span of more than 1000 years (`MAX_TIME_LORDS_YEARS`) is an `invalid_input` error.

## Chinese astrology

A chart requested with `chinese_calendar` carries the Chinese calendar of its moment: the solar terms, the lunar months and the equation of time. `chinese_calendar(moment)` (Python `Engine.chinese_calendar`, JavaScript `engine.chineseCalendar`) computes it on its own; it is `None`/`null` when the kernels do not cover the year or so of ephemeris it needs. `bazi(calendar, birth, longitude, utc_offset_minutes, …)` (no kernel needed) casts the Four Pillars from it, with the hidden stems, Ten Gods, NaYin, element balance and luck pillars, and with `date` the pillars of that date. `zi_wei(…)` (JavaScript `ziWei`, no kernel needed either) casts the Zi Wei Dou Shu chart: the twelve palaces, the five-element bureau, the fourteen major and fourteen auxiliary stars with their brightness, the 38 minor stars and the cycles of twelve gods, the Four Transformations natal and flying, the decade and small limits, and with `year` the horoscope of that year. The rules and the result are described in [Chinese astrology](chinese.md).

## Elections

Electional astrology looks for a good moment to begin something. `calculate_election_chart`
returns the chart of a moment plus `election_data`, and `search_elections` scans a span of
up to 92 days at a place for the best windows.

```python
engine.election("2026-12-21T01:00:00Z", 41.9, 12.5, {"purpose": "launch"})
engine.elections("2026-12-01T00:00:00Z", "2026-12-31T00:00:00Z", 41.9, 12.5,
                 {"purpose": "contract", "daytime_only": True})
```

```js
engine.election({ utc: "2026-12-21T01:00:00Z", latitude: 41.9, longitude: 12.5 }, { purpose: "launch" });
engine.elections({ utc: "2026-12-01T00:00:00Z", latitude: 41.9, longitude: 12.5 },
                 "2026-12-31T00:00:00Z", { purpose: "contract", daytime_only: true });
```

The criteria object is optional. Every field has a default, and unknown fields are
rejected (`invalid_input`):

| Field | Meaning | Default |
|---|---|---|
| `purpose` | `general`, `contract`, `partnership`, `travel`, `health`, `finance`, `launch`, `career`: sets the house of the matter, its natural significator and the favourable planetary hours | `general` |
| `avoid_void_moon` | leave out moments with the Moon void of course | `true` |
| `avoid_mercury_retrograde` | leave out moments with Mercury retrograde | `true` for contract, travel, launch |
| `avoid_venus_retrograde` | leave out moments with Venus retrograde | `true` for partnership |
| `daytime_only` | leave out moments between sunset and sunrise | `false` |
| `local_hours` | `{from, to}` local clock hours to search (`from > to` spans midnight) | any hour |
| `utc_offsets` | `[{from, minutes}]`: the local UTC offset from each instant on, for `local_hours` (daylight saving time included) | UTC |
| `step_minutes` | minutes between assessed moments (5-60) | 10 |
| `min_score` | lowest score a window keeps | 60 |
| `max_results` | most windows returned (1-50) | 20 |
| `natal` | a natal chart's `planets` (`name`, `ecliptic_longitude`) to elect for | none |

`election_data` holds a `score` from 0 to 100 (50 plus the factors' weights) and a
`verdict`: `favourable` from 65, `mixed` from 45, `unfavourable` below. It also holds
the `factors` that apply (`code`, `weight`, and `planet`, `target`, `aspect`, `house`,
`sign` where relevant), `excluded_by` (the criteria filters the moment fails), and
`planetary_hours` and `moon_status` as in horary charts. Factor codes are stable, and
wording is left to the application:

| Group | Codes |
|---|---|
| Moon | `MOON_VOC`, `MOON_COMBUST`, `MOON_WAXING`, `MOON_VIA_COMBUSTA`, `MOON_PITTED_DEGREE`, `MOON_AZIMENE_DEGREE`, `MOON_FORTUNE_DEGREE`, `MOON_DIGNIFIED`, `MOON_DEBILITATED`, `MOON_SWIFT`, `MOON_SLOW`, `MOON_ANGULAR`, `MOON_IN_DARK_HOUSE`, `MOON_APPLYING_BENEFIC`, `MOON_APPLYING_MALEFIC`, `MOON_APPLYING_SIGNIFICATOR` |
| Ascendant | `ASC_EARLY`, `ASC_LATE`, `ASC_PITTED_DEGREE`, `ASC_AZIMENE_DEGREE`, `ASC_FORTUNE_DEGREE`, `ASC_LIGHT_DEGREE`, `ASC_DARK_DEGREE`, `ASC_RULER_DIGNIFIED`, `ASC_RULER_DEBILITATED`, `ASC_RULER_ANGULAR`, `ASC_RULER_IN_DARK_HOUSE`, `ASC_RULER_RETROGRADE`, `ASC_RULER_COMBUST` |
| Angles | `BENEFIC_IN_1ST`, `BENEFIC_ANGULAR`, `MALEFIC_IN_1ST`, `MALEFIC_ANGULAR` |
| Retrogrades | `MERCURY_RETROGRADE`, `VENUS_RETROGRADE` |
| The matter | `HOUSE_RULER_*` and `SIGNIFICATOR_*` (`DIGNIFIED`, `DEBILITATED`, `ANGULAR`, `IN_DARK_HOUSE`, `RETROGRADE`, `COMBUST`) |
| Hour | `HOUR_RULER_FAVOURS_PURPOSE`, `HOUR_RULER_MALEFIC` |
| Natal | `NATAL_ASC_WELL_PLACED`, `NATAL_ASC_BADLY_PLACED`, `NATAL_BENEFIC_CONTACT`, `NATAL_MALEFIC_CONTACT`, `NATAL_MOON_TO_BENEFIC`, `NATAL_MOON_TO_MALEFIC` |

The `*_DEGREE` codes weigh the [degree qualities](#degree-qualities) of the Moon's and the
Ascendant's degrees: pitted (−4) and lame (−3) degrees hinder, degrees increasing fortune
help (+3 for the Moon, +4 for the Ascendant), and a light Ascendant degree helps (+2) while
a dark or void one hinders (−2). Smoky degrees, between light and dark, are not weighed.

A search assesses a moment every `step_minutes` and skips those that fail a filter.
Consecutive moments that score at least `min_score` form a window (`start`, `end`, `best`,
`score`, `verdict`, and the `factors` of the best moment). Windows are ranked by score.
The result also counts the moments `evaluated` and those `excluded` by each filter. For
speed, a search interpolates positions between hourly exact ones, where the Moon is off by
well under an arc second. It then assesses each window's best moment again on exact
positions, so the reported score is exactly what `election` gives for that moment. A
30-day search at 10-minute steps takes about half a second.

## Degree qualities

`degree_qualities(longitude)` gives the qualities William Lilly tabulates for each degree
of the zodiac (*Christian Astrology*, 1659, p. 116; see
[the transcription notes](degree-qualities.md)). No kernel is needed.

```python
ace.degree_qualities(5.5)
# {"sign": "Aries", "degree": 6, "gender": "masculine", "light": "light",
#  "pitted": True, "azimene": False, "fortune": False}
```

```js
degreeQualities(5.5);
```

Degrees are ordinal, as in Lilly: `degree` 6 is 5°00′ to 5°59′ of the sign. `gender` is
`masculine` or `feminine`, and `light` is `light`, `dark`, `smoky` or `void`. `pitted`
marks the deep or pitted degrees, `azimene` the lame or deficient ones, and `fortune` the
degrees increasing fortune. The longitude is taken in whatever zodiac it is measured in.

`degree_quality_table()` (`degreeQualityTable()` in JavaScript) returns the whole table, one
entry per sign from Aries: `gender` and `light` are runs (`quality`, `end`), each lasting from
the end of the previous run up to and including its `end` degree, and `pitted`, `azimene` and
`fortune` list degrees.

## Errors

Each failure has a stable code, shared with the Khagole HTTP API:

| Code | Cause | Rust | Python | JavaScript |
|---|---|---|---|---|
| `ephemeris_out_of_range` | no loaded kernel covers the date | `EngineError::OutOfRange` | `EphemerisRangeError` | `Error` with `code` |
| `ephemeris_error` | a kernel could not be read or evaluated | `EngineError::Ephemeris` | `EngineError` | `Error` with `code` |
| `invalid_input` | malformed request: a date that does not exist, a place off the Earth (latitude beyond ±90°, longitude beyond ±360°, NaN), a non-numeric, NaN or infinite orb, a quesited house outside 1–12, a span too long, … | `EngineError::InvalidInput` | `ValueError` | `Error` with `code` |
| `invalid_kernel` | a kernel file cannot be read or is not a valid SPK file | `SpkError` when loading | `EngineError` | `Error` with `code` |
| `internal_error` | a result could not be converted for the binding (a bug) | — | `EngineError` | `Error` with `code` |

In Rust, `EngineError::code()` returns the code; in Python and JavaScript every exception
carries it in `code`. Dates outside the loaded kernels are
always reported as errors and never approximated. Check them upfront with `supports(jd)` /
`coverage` in Python and JavaScript, or `KernelSet::for_jd` / `KernelSet::coverage` in
Rust.
