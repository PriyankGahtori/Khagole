//! WebAssembly bindings (npm package `astroceleste-engine`).
//!
//! ```js
//! import init, { Engine, synastry, derivedChart } from "astroceleste-engine";
//! await init();
//! const engine = new Engine();
//! engine.addKernel("de440s.bsp", new Uint8Array(await (await fetch(url)).arrayBuffer()));
//! const chart = engine.chart({ utc: "1987-05-17T14:30:00Z", latitude: 41.9, longitude: 12.5 });
//! ```
//!
//! Requests and results are plain objects in the API's JSON shape (snake_case keys).
//! Failures throw an `Error` whose `code` is the API error code, e.g.
//! `"ephemeris_out_of_range"`.

use astroceleste_engine::ephemeris::{Kernel, KernelSet, Spk};
use astroceleste_engine::{
    ayanamsa, ayanamsa_info, bazi as core_bazi, calculate_chart, calculate_derived_chart,
    calculate_election_chart, calculate_horary_chart, calculate_synastry,
    calculate_transit_chart, chart_dignities as core_chart_dignities,
    chinese_calendar as core_chinese_calendar, degree_qualities as core_degree_qualities,
    degree_quality_table as core_degree_quality_table, houses_at_sidereal_time,
    planets_and_sidereal_time_with_options, search_elections, time_lords as core_time_lords,
    zi_wei as core_zi_wei, BaziOptions, ChartRequest, ChineseCalendar, ElectionCriteria,
    EngineError, HouseSystem, UtcInstant, ZiWeiOptions,
    calc_divisional_chart, calc_divisional_point, calc_panchang, calc_vimshottari_dasha_tz,
    nakshatra_info, YearType,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::prelude::*;

fn js_error(code: &str, message: &str) -> JsValue {
    let err = js_sys::Error::new(message);
    // Setting a property on a fresh Error object cannot fail.
    let _ = js_sys::Reflect::set(&err, &"code".into(), &code.into());
    err.into()
}

fn engine_error(err: EngineError) -> JsValue {
    js_error(err.code(), &err.to_string())
}

fn invalid(message: impl std::fmt::Display) -> JsValue {
    js_error("invalid_input", &message.to_string())
}

fn from_js<T: for<'de> Deserialize<'de>>(value: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(invalid)
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    // Plain objects (not Maps) and numbers as numbers, like JSON.parse would give.
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value
        .serialize(&serializer)
        .map_err(|e| js_error("internal_error", &e.to_string()))
}

fn default_house_system() -> String {
    "W".into()
}
fn default_zodiac() -> String {
    "sidereal".into()
}
fn default_ayanamsa() -> String {
    "lahiri".into()
}
fn default_dignity_scheme() -> String {
    "lilly".into()
}
fn default_position_mode() -> String {
    "true".into()
}
fn default_node_type() -> String {
    "mean".into()
}

/// `{ utc, latitude, longitude, house_system?, zodiac_type?, ayanamsa?, position_mode?,
/// node_type?, orb_settings?, dignity_scheme?, quesited_house?, chinese_calendar? }`
#[derive(Deserialize)]
struct Request {
    utc: String,
    latitude: f64,
    longitude: f64,
    #[serde(default = "default_house_system")]
    house_system: String,
    #[serde(default = "default_zodiac")]
    zodiac_type: String,
    #[serde(default = "default_ayanamsa")]
    ayanamsa: String,
    #[serde(default = "default_position_mode")]
    position_mode: String,
    #[serde(default = "default_node_type")]
    node_type: String,
    #[serde(default)]
    orb_settings: Option<Value>,
    #[serde(default = "default_dignity_scheme")]
    dignity_scheme: String,
    /// Horary only: the house of the matter asked about (1-12).
    #[serde(default)]
    quesited_house: Option<u8>,
    /// Also compute the Chinese calendar of the moment (not for transits).
    #[serde(default)]
    chinese_calendar: bool,
}

impl Request {
    fn parse(value: JsValue) -> Result<(Self, UtcInstant), JsValue> {
        let req: Request = from_js(value)?;
        let instant = UtcInstant::parse(&req.utc).map_err(invalid)?;
        Ok((req, instant))
    }

    fn as_chart_request(&self, instant: UtcInstant) -> ChartRequest<'_> {
        ChartRequest {
            instant,
            latitude: self.latitude,
            longitude: self.longitude,
            house_system: &self.house_system,
            zodiac_type: &self.zodiac_type,
            ayanamsa: &self.ayanamsa,
            orb_settings: self.orb_settings.as_ref(),
            dignity_scheme: &self.dignity_scheme,
            position_mode: &self.position_mode,
            node_type: &self.node_type,
            chinese_calendar: self.chinese_calendar,
        }
    }
}

/// Chart calculator over JPL kernels loaded from memory.
#[wasm_bindgen]
pub struct Engine {
    kernels: KernelSet,
    ayanamsa_code: String,
    position_mode: String,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Engine {
        Engine {
            kernels: KernelSet::new(),
            ayanamsa_code: "lahiri".to_string(),
            position_mode: "true".to_string(),
        }
    }

    /// Add a JPL SPK kernel (e.g. de440s.bsp). Kernels are used in the order added: a
    /// date is computed with the first one covering it.
    #[wasm_bindgen(js_name = addKernel)]
    pub fn add_kernel(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), JsValue> {
        let spk = Spk::from_bytes(bytes).map_err(|e| js_error("invalid_kernel", &e.to_string()))?;
        let kernel =
            Kernel::new(name, spk).map_err(|e| js_error("invalid_kernel", &e.to_string()))?;
        self.kernels.push(kernel);
        Ok(())
    }

    /// The loaded kernels as `[name, firstJD, lastJD]`, in preference order.
    #[wasm_bindgen(getter)]
    pub fn kernels(&self) -> Result<JsValue, JsValue> {
        let kernels: Vec<(&str, f64, f64)> = self
            .kernels
            .kernels()
            .iter()
            .map(|k| (k.name.as_str(), k.start_jd, k.end_jd))
            .collect();
        to_js(&kernels)
    }

    /// `[firstJD, lastJD]` covered by the loaded kernels, or `undefined`.
    #[wasm_bindgen(getter)]
    pub fn coverage(&self) -> Option<Vec<f64>> {
        self.kernels.coverage().map(|(a, b)| vec![a, b])
    }

    /// Whether a loaded kernel covers this UT Julian date.
    pub fn supports(&self, jd: f64) -> bool {
        self.kernels.for_jd(jd).is_some()
    }

    /// The chart of a moment and place.
    pub fn chart(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let (req, instant) = Request::parse(request)?;
        let chart =
            calculate_chart(&self.kernels, &req.as_chart_request(instant)).map_err(engine_error)?;
        to_js(&chart)
    }

    /// A chart with its horary analysis under `horary_data`.
    pub fn horary(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let (req, instant) = Request::parse(request)?;
        let chart = calculate_horary_chart(
            &self.kernels,
            &req.as_chart_request(instant),
            req.quesited_house,
        )
        .map_err(engine_error)?;
        to_js(&chart)
    }

    /// A chart with its electional assessment under `election_data`. `criteria` may be
    /// `undefined` for the defaults.
    pub fn election(&self, request: JsValue, criteria: JsValue) -> Result<JsValue, JsValue> {
        let (req, instant) = Request::parse(request)?;
        let criteria = to_criteria(criteria)?;
        let chart =
            calculate_election_chart(&self.kernels, &req.as_chart_request(instant), &criteria)
                .map_err(engine_error)?;
        to_js(&chart)
    }

    /// The best electional windows from `request.utc` to `end` (ISO 8601 UTC, at most 92
    /// days later) at the request's place.
    pub fn elections(
        &self,
        request: JsValue,
        end: &str,
        criteria: JsValue,
    ) -> Result<JsValue, JsValue> {
        let (req, instant) = Request::parse(request)?;
        let end = UtcInstant::parse(end).map_err(invalid)?;
        let criteria = to_criteria(criteria)?;
        let result = search_elections(
            &self.kernels,
            &req.as_chart_request(instant),
            end,
            &criteria,
        )
        .map_err(engine_error)?;
        to_js(&result)
    }

    /// The Chinese calendar around `utc` (ISO 8601): solar terms, lunar months and the
    /// equation of time; `null` when the loaded kernels do not cover the year or so it
    /// needs.
    #[wasm_bindgen(js_name = chineseCalendar)]
    pub fn chinese_calendar(&self, utc: &str) -> Result<JsValue, JsValue> {
        let instant = UtcInstant::parse(utc).map_err(invalid)?;
        to_js(&core_chinese_calendar(&self.kernels, instant).map_err(engine_error)?)
    }

    /// The sky of `request`, with its cross-aspects to `natalPlanets`.
    pub fn transit(&self, natal_planets: JsValue, request: JsValue) -> Result<JsValue, JsValue> {
        let natal: Value = from_js(natal_planets)?;
        let (req, instant) = Request::parse(request)?;
        let chart = calculate_transit_chart(&self.kernels, &natal, &req.as_chart_request(instant))
            .map_err(engine_error)?;
        to_js(&chart)
    }

    /// Set sidereal mode / ayanamsa: 1 = Lahiri, 3 = Raman, 5 = Krishnamurti, 0 = Fagan/Bradley.
    #[wasm_bindgen(js_name = set_sid_mode)]
    pub fn set_sid_mode(&mut self, sid_mode: i32, _t0: Option<f64>, _ayan_t0: Option<f64>) {
        self.ayanamsa_code = match sid_mode {
            1 => "lahiri".to_string(),
            3 => "raman".to_string(),
            5 => "krishnamurti".to_string(),
            0 => "fagan_bradley".to_string(),
            27 => "true_citra".to_string(),
            _ => "lahiri".to_string(),
        };
    }

    /// Set sidereal ayanamsa by system code (e.g. "lahiri", "raman", "krishnamurti").
    #[wasm_bindgen(js_name = set_ayanamsa)]
    pub fn set_ayanamsa(&mut self, code: &str) {
        self.ayanamsa_code = code.to_string();
    }

    /// Returns the ayanamsa in degrees at UT Julian date `jd`.
    #[wasm_bindgen(js_name = get_ayanamsa_ut)]
    pub fn get_ayanamsa_ut(&self, jd: f64) -> f64 {
        let code = ayanamsa(&self.ayanamsa_code).code;
        ayanamsa_info(jd, code).value
    }

    /// Returns the ayanamsa in degrees at UT Julian date `jd` with Swiss Ephemeris flags.
    /// If flags & 2 (SEFLG_NONUT) is set, returns mean ayanamsa; otherwise returns true ayanamsa (with nutation).
    #[wasm_bindgen(js_name = get_ayanamsa_ex_ut)]
    pub fn get_ayanamsa_ex_ut(&self, jd: f64, flags: u32) -> f64 {
        let code = ayanamsa(&self.ayanamsa_code).code;
        let mean_ayan = ayanamsa_info(jd, code).value;
        if (flags & 2) != 0 {
            mean_ayan
        } else {
            let t = astroceleste_engine::time::Time::from_ut1(jd);
            let orient = astroceleste_engine::frames::Orientation::at(&t);
            mean_ayan + orient.d_psi.to_degrees()
        }
    }

    /// Topocentric coordinates setter (stub for Swiss Ephemeris compatibility).
    #[wasm_bindgen(js_name = set_topo)]
    pub fn set_topo(&mut self, _lon: f64, _lat: f64, _alt: f64) {}

    /// Swiss Ephemeris compatible `calc_ut`:
    /// Returns Float64Array [longitude, latitude, distance, speed, 0, 0].
    #[wasm_bindgen(js_name = calc_ut)]
    pub fn calc_ut(&self, jd: f64, body: i32, flags: u32) -> Result<js_sys::Float64Array, JsValue> {
        let is_sidereal = (flags & 65536) != 0;
        let is_geometric = (flags & 16) != 0 || self.position_mode == "true";
        let is_true_node = body == 11;

        let shift = if is_sidereal {
            let code = ayanamsa(&self.ayanamsa_code).code;
            let mean_ayan = ayanamsa_info(jd, code).value;
            if (flags & 2) != 0 {
                mean_ayan
            } else {
                let t = astroceleste_engine::time::Time::from_ut1(jd);
                let orient = astroceleste_engine::frames::Orientation::at(&t);
                mean_ayan + orient.d_psi.to_degrees()
            }
        } else {
            0.0
        };

        let (planets, _) = planets_and_sidereal_time_with_options(
            &self.kernels,
            jd,
            shift,
            is_geometric,
            is_true_node,
        )
        .map_err(engine_error)?;

        let target_name = match body {
            0 => "Sun",
            1 => "Moon",
            2 => "Mercury",
            3 => "Venus",
            4 => "Mars",
            5 => "Jupiter",
            6 => "Saturn",
            7 => "Uranus",
            8 => "Neptune",
            9 => "Pluto",
            10 | 11 => "North Node",
            _ => "Sun",
        };

        if let Some(p) = planets.bodies.iter().find(|b| b.name == target_name) {
            let res = [p.longitude, p.latitude, p.distance, p.speed, 0.0, 0.0];
            let arr = js_sys::Float64Array::new_with_length(6);
            arr.copy_from(&res);
            Ok(arr)
        } else {
            Err(js_error("body_not_found", &format!("body {body} not found")))
        }
    }

    /// Swiss Ephemeris compatible `houses_ex`:
    /// Returns plain object `{ cusps: Float64Array(13), ascmc: Float64Array(10) }`.
    #[wasm_bindgen(js_name = houses_ex)]
    pub fn houses_ex(
        &self,
        jd: f64,
        iflag: u32,
        lat: f64,
        lon: f64,
        house_system: &str,
    ) -> Result<JsValue, JsValue> {
        let is_sidereal = (iflag & 65536) != 0;
        let shift = if is_sidereal {
            let code = ayanamsa(&self.ayanamsa_code).code;
            let mean_ayan = ayanamsa_info(jd, code).value;
            if (iflag & 2) != 0 {
                mean_ayan
            } else {
                let t = astroceleste_engine::time::Time::from_ut1(jd);
                let orient = astroceleste_engine::frames::Orientation::at(&t);
                mean_ayan + orient.d_psi.to_degrees()
            }
        } else {
            0.0
        };

        let t = astroceleste_engine::time::Time::from_ut1(jd);
        let orient = astroceleste_engine::frames::Orientation::at(&t);
        let system = HouseSystem::from_code(house_system);
        let houses = houses_at_sidereal_time(jd, orient.gast_hours, lat, lon, system, shift);

        let cusps_arr = js_sys::Float64Array::new_with_length(13);
        let mut cusps_data = [0.0; 13];
        for i in 0..12 {
            cusps_data[i + 1] = houses.cusps[i];
        }
        cusps_arr.copy_from(&cusps_data);

        let ascmc_arr = js_sys::Float64Array::new_with_length(10);
        let mut ascmc_data = [0.0; 10];
        ascmc_data[0] = houses.ascendant;
        ascmc_data[1] = houses.midheaven;
        ascmc_arr.copy_from(&ascmc_data);

        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"cusps".into(), &cusps_arr)?;
        js_sys::Reflect::set(&obj, &"ascmc".into(), &ascmc_arr)?;
        Ok(obj.into())
    }

    /// Swiss Ephemeris compatible `houses`:
    #[wasm_bindgen(js_name = houses)]
    pub fn houses(
        &self,
        jd: f64,
        lat: f64,
        lon: f64,
        house_system: &str,
    ) -> Result<JsValue, JsValue> {
        self.houses_ex(jd, 0, lat, lon, house_system)
    }

    /// Compute a divisional chart (D1 to D60).
    #[wasm_bindgen(js_name = calc_divisional)]
    pub fn calc_divisional(&self, chart: JsValue, varga: &str) -> Result<JsValue, JsValue> {
        calc_divisional(chart, None, varga)
    }

    /// Compute Vimshottari dasha hierarchy from Moon's sidereal longitude and birth JD.
    #[wasm_bindgen(js_name = calc_vimshottari_dasha)]
    pub fn calc_vimshottari_dasha(
        &self,
        moon_lon: f64,
        birth_jd: f64,
        depth: Option<u32>,
        year_type: Option<String>,
        tz_offset_hours: Option<f64>,
    ) -> Result<JsValue, JsValue> {
        let yt = match year_type.as_deref() {
            Some("calendar") => YearType::Calendar,
            Some("savana") => YearType::Savana,
            Some("tropical") => YearType::Tropical,
            Some("julian") => YearType::Julian,
            Some("sidereal") => YearType::Sidereal,
            _ => YearType::Tssy,
        };
        let d = depth.unwrap_or(3);
        let tz = tz_offset_hours.unwrap_or(0.0);

        if yt == YearType::Tssy && !self.kernels.is_empty() {
            let sun_fn = |jd: f64| -> f64 {
                if let Ok(arr) = self.calc_ut(jd, 0, 65536 | 16) {
                    arr.get_index(0)
                } else {
                    astroceleste_engine::vedic::dasha::keplerian_sun_sidereal_lon(jd, 0.0, birth_jd)
                }
            };
            let tree = astroceleste_engine::vedic::dasha::calc_vimshottari_dasha_with_sun_tz(
                moon_lon, birth_jd, d, yt, &sun_fn, tz,
            );
            return to_js(&tree);
        }

        calc_vimshottari_dasha_js(moon_lon, birth_jd, depth, year_type, tz_offset_hours)
    }

    /// Compute Panchanga (five limbs) from Sun and Moon longitudes and Julian Day.
    #[wasm_bindgen(js_name = calc_panchang)]
    pub fn calc_panchang(
        &self,
        sun_lon: f64,
        moon_lon: f64,
        jd: f64,
        weekday: Option<u8>,
        is_before_sunrise: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        calc_panchang_js(sun_lon, moon_lon, jd, weekday, is_before_sunrise)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

fn optional(value: JsValue) -> Result<Option<Value>, JsValue> {
    if value.is_undefined() || value.is_null() {
        Ok(None)
    } else {
        from_js(value).map(Some)
    }
}

fn to_criteria(value: JsValue) -> Result<ElectionCriteria, JsValue> {
    match optional(value)? {
        Some(v) => ElectionCriteria::from_value(&v).map_err(engine_error),
        None => Ok(ElectionCriteria::default()),
    }
}

/// Cross-aspects from chart B's planets (overlay) to chart A's (base).
#[wasm_bindgen]
pub fn synastry(
    chart_a_planets: JsValue,
    chart_b_planets: JsValue,
    orb_settings: JsValue,
) -> Result<JsValue, JsValue> {
    let a: Value = from_js(chart_a_planets)?;
    let b: Value = from_js(chart_b_planets)?;
    let orbs = optional(orb_settings)?;
    to_js(&calculate_synastry(&a, &b, orbs.as_ref()).map_err(engine_error)?)
}

/// A derived (turned) chart: radix house `rootHouse` becomes house 1.
#[wasm_bindgen(js_name = derivedChart)]
pub fn derived_chart(
    base_chart: JsValue,
    root_house: i32,
    custom_name: Option<String>,
) -> Result<JsValue, JsValue> {
    let base: Value = from_js(base_chart)?;
    let chart = calculate_derived_chart(&base, root_house as i64, custom_name.as_deref())
        .map_err(engine_error)?;
    to_js(&chart)
}

/// Lilly's qualities of the degree an ecliptic longitude falls in.
#[wasm_bindgen(js_name = degreeQualities)]
pub fn degree_qualities(longitude: f64) -> Result<JsValue, JsValue> {
    to_js(&core_degree_qualities(longitude))
}

/// Lilly's table of the degree qualities, one object per sign from Aries.
#[wasm_bindgen(js_name = degreeQualityTable)]
pub fn degree_quality_table() -> Result<JsValue, JsValue> {
    to_js(core_degree_quality_table())
}

/// Annual profections and firdaria of a nativity (ISO 8601 UTC birth, natal Sun and
/// Ascendant longitudes) for the years that overlap `start`..`end` (ISO 8601 UTC).
#[wasm_bindgen(js_name = timeLords)]
pub fn time_lords(
    birth: &str,
    sun_longitude: f64,
    ascendant_longitude: f64,
    start: &str,
    end: &str,
) -> Result<JsValue, JsValue> {
    let parse = |s: &str| UtcInstant::parse(s).map_err(invalid);
    to_js(
        &core_time_lords(
            parse(birth)?,
            sun_longitude,
            ascendant_longitude,
            parse(start)?,
            parse(end)?,
        )
        .map_err(engine_error)?,
    )
}

/// A computed chart's dignities (condition, sect, receptions) judged again under `scheme`
/// ("lilly" or "dorothean"). No kernel is needed.
#[wasm_bindgen(js_name = chartDignities)]
pub fn chart_dignities(chart: JsValue, scheme: &str) -> Result<JsValue, JsValue> {
    let chart: Value = from_js(chart)?;
    let judged = core_chart_dignities(&chart, scheme).map_err(engine_error)?;
    to_js(&judged)
}

fn yes() -> bool {
    true
}

/// `{ solar_time?, zi_hour?, sex?, luck_pillars?, year?, date? }`
#[derive(Deserialize)]
struct BaziRequest {
    #[serde(default = "yes")]
    solar_time: bool,
    #[serde(default)]
    zi_hour: Option<String>,
    #[serde(default)]
    sex: Option<String>,
    #[serde(default)]
    luck_pillars: Option<u8>,
    #[serde(default)]
    year: Option<i32>,
    #[serde(default)]
    date: Option<String>,
}

/// The Four Pillars of a birth (ISO 8601 UTC) at `longitude` (degrees east) where civil
/// time was `utcOffsetMinutes` ahead of UTC, from its Chinese `calendar` (the chart's
/// `chinese_calendar`). `options`: `{ solar_time?, zi_hour?, sex?, luck_pillars?, year?, date? }`, or
/// `undefined` for the defaults. No kernel is needed.
#[wasm_bindgen]
pub fn bazi(
    calendar: JsValue,
    birth: &str,
    longitude: f64,
    #[wasm_bindgen(js_name = utcOffsetMinutes)] utc_offset_minutes: f64,
    options: JsValue,
) -> Result<JsValue, JsValue> {
    let calendar: ChineseCalendar = from_js(calendar)?;
    let options = optional(options)?.unwrap_or_else(|| Value::Object(Default::default()));
    let request: BaziRequest = serde_json::from_value(options).map_err(invalid)?;
    let defaults = BaziOptions::default();
    let options = BaziOptions {
        solar_time: request.solar_time,
        zi_hour: request.zi_hour.as_deref().unwrap_or(defaults.zi_hour),
        sex: request.sex.as_deref(),
        luck_pillars: request.luck_pillars.unwrap_or(defaults.luck_pillars),
        year: request.year,
        date: request.date.as_deref(),
    };
    let birth = UtcInstant::parse(birth).map_err(invalid)?;
    to_js(
        &core_bazi(&calendar, birth, longitude, utc_offset_minutes, &options)
            .map_err(engine_error)?,
    )
}

/// `{ solar_time?, zi_hour?, sex?, leap_month?, year?, date? }`
#[derive(Deserialize)]
struct ZiWeiRequest {
    #[serde(default = "yes")]
    solar_time: bool,
    #[serde(default)]
    zi_hour: Option<String>,
    #[serde(default)]
    sex: Option<String>,
    #[serde(default)]
    leap_month: Option<String>,
    #[serde(default)]
    year: Option<i32>,
    #[serde(default)]
    date: Option<String>,
}

/// The Zi Wei Dou Shu chart of a birth (ISO 8601 UTC) at `longitude` (degrees east) where
/// civil time was `utcOffsetMinutes` ahead of UTC, from its Chinese `calendar` (the chart's
/// `chinese_calendar`). `options`: `{ solar_time?, zi_hour?, sex?, leap_month?, year?, date? }`, or
/// `undefined` for the defaults. No kernel is needed.
#[wasm_bindgen(js_name = ziWei)]
pub fn zi_wei(
    calendar: JsValue,
    birth: &str,
    longitude: f64,
    #[wasm_bindgen(js_name = utcOffsetMinutes)] utc_offset_minutes: f64,
    options: JsValue,
) -> Result<JsValue, JsValue> {
    let calendar: ChineseCalendar = from_js(calendar)?;
    let options = optional(options)?.unwrap_or_else(|| Value::Object(Default::default()));
    let request: ZiWeiRequest = serde_json::from_value(options).map_err(invalid)?;
    let defaults = ZiWeiOptions::default();
    let options = ZiWeiOptions {
        solar_time: request.solar_time,
        zi_hour: request.zi_hour.as_deref().unwrap_or(defaults.zi_hour),
        sex: request.sex.as_deref(),
        leap_month: request.leap_month.as_deref().unwrap_or(defaults.leap_month),
        year: request.year,
        date: request.date.as_deref(),
    };
    let birth = UtcInstant::parse(birth).map_err(invalid)?;
    to_js(
        &core_zi_wei(&calendar, birth, longitude, utc_offset_minutes, &options)
            .map_err(engine_error)?,
    )
}

/// Julian day of an ISO 8601 UTC date-time, as the chart calculation uses it.
#[wasm_bindgen(js_name = julianDay)]
pub fn julian_day(utc: &str) -> Result<f64, JsValue> {
    Ok(UtcInstant::parse(utc).map_err(invalid)?.julian_day())
}

/// A smaller kernel covering TDB Julian dates `startJd`..`endJd`, with positions
/// identical to `kernel`'s inside that range (e.g. to ship 1900-2100 of de440s).
#[wasm_bindgen(js_name = excerptKernel)]
pub fn excerpt_kernel(kernel: Vec<u8>, start_jd: f64, end_jd: f64) -> Result<Vec<u8>, JsValue> {
    let spk = Spk::from_bytes(kernel).map_err(|e| js_error("invalid_kernel", &e.to_string()))?;
    spk.excerpt(start_jd, end_jd)
        .map_err(|e| js_error("invalid_kernel", &e.to_string()))
}

/// Transform a chart or planet array into a divisional chart (D1 to D60).
#[wasm_bindgen(js_name = calcDivisional)]
pub fn calc_divisional(
    chart_or_planets: JsValue,
    lagna_lon: Option<f64>,
    varga: &str,
) -> Result<JsValue, JsValue> {
    let val: Value = from_js(chart_or_planets)?;
    let mut planets_vec: Vec<(String, f64)> = Vec::new();
    let mut resolved_lagna_lon = lagna_lon;

    if let Value::Object(ref obj) = val {
        if resolved_lagna_lon.is_none() {
            if let Some(lagna_val) = obj.get("lagna") {
                if let Some(lon) = lagna_val.get("lon").and_then(|v| v.as_f64()) {
                    resolved_lagna_lon = Some(lon);
                } else if let Some(lon) = lagna_val.get("ecliptic_longitude").and_then(|v| v.as_f64()) {
                    resolved_lagna_lon = Some(lon);
                }
            }
        }

        if let Some(Value::Array(planets_arr)) = obj.get("planets") {
            for p in planets_arr {
                if let Value::Object(p_obj) = p {
                    let name = p_obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let lon = p_obj.get("ecliptic_longitude")
                        .or_else(|| p_obj.get("lon"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    if name == "Ascendant" && resolved_lagna_lon.is_none() {
                        resolved_lagna_lon = Some(lon);
                    }
                    if !name.is_empty() {
                        planets_vec.push((name, lon));
                    }
                }
            }
        }
    } else if let Value::Array(arr) = val {
        for p in arr {
            if let Value::Object(p_obj) = p {
                let name = p_obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let lon = p_obj.get("ecliptic_longitude")
                    .or_else(|| p_obj.get("lon"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                if name == "Ascendant" && resolved_lagna_lon.is_none() {
                    resolved_lagna_lon = Some(lon);
                }
                if !name.is_empty() {
                    planets_vec.push((name, lon));
                }
            }
        }
    }

    let lagna = resolved_lagna_lon.unwrap_or(0.0);
    let div_chart = calc_divisional_chart(&planets_vec, lagna, varga).map_err(engine_error)?;
    to_js(&div_chart)
}

/// Transform a single longitude to its divisional position.
#[wasm_bindgen(js_name = calcDivisionalPoint)]
pub fn calc_divisional_point_js(lon: f64, varga: &str) -> Result<JsValue, JsValue> {
    let p = calc_divisional_point(lon, varga).map_err(engine_error)?;
    to_js(&p)
}

/// Compute Vimshottari dasha periods tree from Moon's sidereal longitude and birth JD.
#[wasm_bindgen(js_name = calcVimshottariDasha)]
pub fn calc_vimshottari_dasha_js(
    moon_lon: f64,
    birth_jd: f64,
    depth: Option<u32>,
    year_type: Option<String>,
    tz_offset_hours: Option<f64>,
) -> Result<JsValue, JsValue> {
    let yt = match year_type.as_deref() {
        Some("calendar") => YearType::Calendar,
        Some("savana") => YearType::Savana,
        Some("tropical") => YearType::Tropical,
        Some("julian") => YearType::Julian,
        Some("sidereal") => YearType::Sidereal,
        _ => YearType::Tssy,
    };
    let d = depth.unwrap_or(3);
    let tz = tz_offset_hours.unwrap_or(0.0);
    let tree = calc_vimshottari_dasha_tz(moon_lon, birth_jd, d, yt, tz);
    to_js(&tree)
}

/// Compute Panchanga (five limbs) from Sun and Moon longitudes and Julian Day.
#[wasm_bindgen(js_name = calcPanchang)]
pub fn calc_panchang_js(
    sun_lon: f64,
    moon_lon: f64,
    jd: f64,
    weekday: Option<u8>,
    is_before_sunrise: Option<bool>,
) -> Result<JsValue, JsValue> {
    let result = calc_panchang(sun_lon, moon_lon, jd, weekday, is_before_sunrise);
    to_js(&result)
}

/// Get nakshatra info for a longitude.
#[wasm_bindgen(js_name = nakshatraInfo)]
pub fn nakshatra_info_js(lon: f64) -> Result<JsValue, JsValue> {
    to_js(&nakshatra_info(lon))
}
