//! Python bindings: `import astroceleste_engine`.
//!
//! Charts come back as plain dicts with the same JSON shape as the Rust `Serialize`
//! output. Values cross the boundary as JSON text, which keeps floats exact (both sides
//! print and parse shortest round-trip representations). Calculations release the GIL.

use std::path::PathBuf;

use astroceleste_engine::ephemeris::{Kernel, KernelSet, Spk};
use astroceleste_engine::{
    bazi as core_bazi, calculate_chart, calculate_derived_chart, calculate_election_chart,
    calculate_horary_chart, calculate_synastry, calculate_transit_chart,
    chart_dignities as core_chart_dignities, chinese_calendar as core_chinese_calendar,
    degree_qualities as core_degree_qualities, degree_quality_table as core_degree_quality_table,
    search_elections, time_lords as core_time_lords, zi_wei as core_zi_wei, BaziOptions,
    ChartRequest, ChineseCalendar, ElectionCriteria, EngineError as CoreError, UtcInstant,
    ZiWeiOptions,
};
use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyString};
use serde::Serialize;
use serde_json::Value;

create_exception!(
    astroceleste_engine,
    EngineError,
    PyException,
    "A calculation failed: an unreadable kernel (`code` \"invalid_kernel\") or one that \
     cannot be evaluated (\"ephemeris_error\")."
);
create_exception!(
    astroceleste_engine,
    EphemerisRangeError,
    EngineError,
    "No loaded kernel covers the requested date."
);

/// `err` with a `code` attribute: the stable error code of the Astroceleste API.
fn with_code(err: PyErr, code: &str) -> PyErr {
    Python::attach(|py| {
        // Setting an attribute on a fresh exception instance cannot fail.
        let _ = err.value(py).setattr("code", code);
    });
    err
}

fn invalid_input(message: impl ToString) -> PyErr {
    with_code(PyValueError::new_err(message.to_string()), "invalid_input")
}

fn invalid_kernel(message: impl ToString) -> PyErr {
    with_code(EngineError::new_err(message.to_string()), "invalid_kernel")
}

fn to_py_err(err: CoreError) -> PyErr {
    let py_err = match &err {
        CoreError::OutOfRange { .. } => EphemerisRangeError::new_err(err.to_string()),
        CoreError::InvalidInput(msg) => PyValueError::new_err(msg.clone()),
        _ => EngineError::new_err(err.to_string()),
    };
    with_code(py_err, err.code())
}

/// Python object → JSON value, through the standard `json` module.
fn to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    let json = obj.py().import("json")?;
    let text: String = json.call_method1("dumps", (obj,))?.extract()?;
    serde_json::from_str(&text).map_err(invalid_input)
}

/// Serializable → Python object (dicts, lists, floats, …).
fn to_py<T: Serialize>(py: Python<'_>, value: &T) -> PyResult<Py<PyAny>> {
    let text = serde_json::to_string(value)
        .map_err(|e| with_code(EngineError::new_err(e.to_string()), "internal_error"))?;
    let json = py.import("json")?;
    Ok(json.call_method1("loads", (text,))?.unbind())
}

/// A UTC instant from an ISO 8601 string or a `datetime` (naive means UTC).
fn to_instant(moment: &Bound<'_, PyAny>) -> PyResult<UtcInstant> {
    if let Ok(text) = moment.cast::<PyString>() {
        return UtcInstant::parse(text.to_str()?).map_err(invalid_input);
    }
    let py = moment.py();
    let dt = if moment.getattr("tzinfo")?.is_none() {
        moment.clone()
    } else {
        let utc = py.import("datetime")?.getattr("timezone")?.getattr("utc")?;
        moment.call_method1("astimezone", (utc,))?
    };
    let field = |name: &str| -> PyResult<i64> { dt.getattr(name)?.extract() };
    Ok(UtcInstant::from_civil(
        field("year")? as i32,
        field("month")? as u32,
        field("day")? as u32,
        field("hour")? as u32,
        field("minute")? as u32,
        field("second")? as u32,
        field("microsecond")? as u32,
    ))
}

fn optional_value(obj: Option<&Bound<'_, PyAny>>) -> PyResult<Option<Value>> {
    match obj {
        Some(o) if !o.is_none() => Ok(Some(to_value(o)?)),
        _ => Ok(None),
    }
}

/// Election criteria from a dict (None for the defaults).
fn to_criteria(obj: Option<&Bound<'_, PyAny>>) -> PyResult<ElectionCriteria> {
    match optional_value(obj)? {
        Some(value) => ElectionCriteria::from_value(&value).map_err(to_py_err),
        None => Ok(ElectionCriteria::default()),
    }
}

/// Everything a chart request needs, owned, so it can cross into a GIL-free section.
struct Request {
    instant: UtcInstant,
    latitude: f64,
    longitude: f64,
    house_system: String,
    zodiac_type: String,
    ayanamsa: String,
    orb_settings: Option<Value>,
    dignity_scheme: String,
    position_mode: String,
    node_type: String,
    chinese_calendar: bool,
}

impl Request {
    #[allow(clippy::too_many_arguments)]
    fn new(
        moment: &Bound<'_, PyAny>,
        latitude: f64,
        longitude: f64,
        house_system: &str,
        zodiac_type: &str,
        ayanamsa: &str,
        orb_settings: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        Ok(Request {
            instant: to_instant(moment)?,
            latitude,
            longitude,
            house_system: house_system.to_string(),
            zodiac_type: zodiac_type.to_string(),
            ayanamsa: ayanamsa.to_string(),
            orb_settings: optional_value(orb_settings)?,
            dignity_scheme: "lilly".to_string(),
            position_mode: "apparent".to_string(),
            node_type: "mean".to_string(),
            chinese_calendar: false,
        })
    }

    fn with_dignity_scheme(mut self, scheme: &str) -> Self {
        self.dignity_scheme = scheme.to_string();
        self
    }

    fn with_position_mode(mut self, mode: &str) -> Self {
        self.position_mode = mode.to_string();
        self
    }

    fn with_node_type(mut self, node_type: &str) -> Self {
        self.node_type = node_type.to_string();
        self
    }

    fn with_chinese_calendar(mut self, chinese_calendar: bool) -> Self {
        self.chinese_calendar = chinese_calendar;
        self
    }

    fn as_chart_request(&self) -> ChartRequest<'_> {
        ChartRequest {
            instant: self.instant,
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

/// Chart calculator over a set of JPL kernels.
///
/// `Engine(["de440s.bsp", "de441_part-1.bsp"])` loads the kernels in preference order:
/// each date is computed with the first kernel that covers it.
#[pyclass(frozen, module = "astroceleste_engine")]
struct Engine {
    kernels: KernelSet,
}

#[pymethods]
impl Engine {
    #[new]
    fn new(py: Python<'_>, kernels: Vec<PathBuf>) -> PyResult<Self> {
        let set = py.detach(|| {
            let mut set = KernelSet::new();
            for path in &kernels {
                let name = path.file_name().map_or_else(
                    || path.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                );
                let spk = Spk::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
                let kernel =
                    Kernel::new(name, spk).map_err(|e| format!("{}: {e}", path.display()))?;
                set.push(kernel);
            }
            Ok::<_, String>(set)
        });
        Ok(Engine {
            kernels: set.map_err(invalid_kernel)?,
        })
    }

    /// Loaded kernels as (name, first JD, last JD), in preference order.
    #[getter]
    fn kernels(&self) -> Vec<(String, f64, f64)> {
        self.kernels
            .kernels()
            .iter()
            .map(|k| (k.name.clone(), k.start_jd, k.end_jd))
            .collect()
    }

    /// Combined (first JD, last JD) span of the loaded kernels, or None.
    #[getter]
    fn coverage(&self) -> Option<(f64, f64)> {
        self.kernels.coverage()
    }

    /// Whether a loaded kernel covers this UT Julian date.
    fn supports(&self, jd: f64) -> bool {
        self.kernels.for_jd(jd).is_some()
    }

    /// The chart of a moment and place, as the API's chart dict; with
    /// `chinese_calendar`, also the Chinese calendar of the moment.
    #[pyo3(signature = (moment, latitude, longitude, house_system="P", zodiac_type="tropical", ayanamsa="galcent_0sag", orb_settings=None, dignity_scheme="lilly", chinese_calendar=false, position_mode="apparent", node_type="mean"))]
    #[allow(clippy::too_many_arguments)]
    fn chart(
        &self,
        py: Python<'_>,
        moment: &Bound<'_, PyAny>,
        latitude: f64,
        longitude: f64,
        house_system: &str,
        zodiac_type: &str,
        ayanamsa: &str,
        orb_settings: Option<&Bound<'_, PyAny>>,
        dignity_scheme: &str,
        chinese_calendar: bool,
        position_mode: &str,
        node_type: &str,
    ) -> PyResult<Py<PyAny>> {
        let req = Request::new(
            moment,
            latitude,
            longitude,
            house_system,
            zodiac_type,
            ayanamsa,
            orb_settings,
        )?
        .with_dignity_scheme(dignity_scheme)
        .with_position_mode(position_mode)
        .with_node_type(node_type)
        .with_chinese_calendar(chinese_calendar);
        let chart = py
            .detach(|| calculate_chart(&self.kernels, &req.as_chart_request()))
            .map_err(to_py_err)?;
        to_py(py, &chart)
    }

    /// A chart with its horary analysis under the `horary_data` key; `quesited_house`
    /// (1-12) is the house of the matter asked about.
    #[pyo3(signature = (moment, latitude, longitude, house_system="P", zodiac_type="tropical", ayanamsa="galcent_0sag", orb_settings=None, dignity_scheme="lilly", quesited_house=None, chinese_calendar=false))]
    #[allow(clippy::too_many_arguments)]
    fn horary(
        &self,
        py: Python<'_>,
        moment: &Bound<'_, PyAny>,
        latitude: f64,
        longitude: f64,
        house_system: &str,
        zodiac_type: &str,
        ayanamsa: &str,
        orb_settings: Option<&Bound<'_, PyAny>>,
        dignity_scheme: &str,
        quesited_house: Option<u8>,
        chinese_calendar: bool,
    ) -> PyResult<Py<PyAny>> {
        let req = Request::new(
            moment,
            latitude,
            longitude,
            house_system,
            zodiac_type,
            ayanamsa,
            orb_settings,
        )?
        .with_dignity_scheme(dignity_scheme)
        .with_chinese_calendar(chinese_calendar);
        let chart = py
            .detach(|| {
                calculate_horary_chart(&self.kernels, &req.as_chart_request(), quesited_house)
            })
            .map_err(to_py_err)?;
        to_py(py, &chart)
    }

    /// A chart with its electional assessment under the `election_data` key.
    #[pyo3(signature = (moment, latitude, longitude, criteria=None, house_system="P", zodiac_type="tropical", ayanamsa="galcent_0sag", orb_settings=None, dignity_scheme="lilly", chinese_calendar=false))]
    #[allow(clippy::too_many_arguments)]
    fn election(
        &self,
        py: Python<'_>,
        moment: &Bound<'_, PyAny>,
        latitude: f64,
        longitude: f64,
        criteria: Option<&Bound<'_, PyAny>>,
        house_system: &str,
        zodiac_type: &str,
        ayanamsa: &str,
        orb_settings: Option<&Bound<'_, PyAny>>,
        dignity_scheme: &str,
        chinese_calendar: bool,
    ) -> PyResult<Py<PyAny>> {
        let criteria = to_criteria(criteria)?;
        let req = Request::new(
            moment,
            latitude,
            longitude,
            house_system,
            zodiac_type,
            ayanamsa,
            orb_settings,
        )?
        .with_dignity_scheme(dignity_scheme)
        .with_chinese_calendar(chinese_calendar);
        let chart = py
            .detach(|| calculate_election_chart(&self.kernels, &req.as_chart_request(), &criteria))
            .map_err(to_py_err)?;
        to_py(py, &chart)
    }

    /// The best electional windows from `start` to `end` at a place.
    #[pyo3(signature = (start, end, latitude, longitude, criteria=None, house_system="P", zodiac_type="tropical", ayanamsa="galcent_0sag"))]
    #[allow(clippy::too_many_arguments)]
    fn elections(
        &self,
        py: Python<'_>,
        start: &Bound<'_, PyAny>,
        end: &Bound<'_, PyAny>,
        latitude: f64,
        longitude: f64,
        criteria: Option<&Bound<'_, PyAny>>,
        house_system: &str,
        zodiac_type: &str,
        ayanamsa: &str,
    ) -> PyResult<Py<PyAny>> {
        let criteria = to_criteria(criteria)?;
        let end = to_instant(end)?;
        let req = Request::new(
            start,
            latitude,
            longitude,
            house_system,
            zodiac_type,
            ayanamsa,
            None,
        )?;
        let result = py
            .detach(|| search_elections(&self.kernels, &req.as_chart_request(), end, &criteria))
            .map_err(to_py_err)?;
        to_py(py, &result)
    }

    /// The Chinese calendar around `moment` (solar terms, lunar months, equation of time),
    /// or None when the loaded kernels do not cover the year or so it needs.
    fn chinese_calendar(&self, py: Python<'_>, moment: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let instant = to_instant(moment)?;
        let calendar = py
            .detach(|| core_chinese_calendar(&self.kernels, instant))
            .map_err(to_py_err)?;
        to_py(py, &calendar)
    }

    /// The sky at `moment` and place, with its cross-aspects to `natal_planets`.
    #[pyo3(signature = (natal_planets, moment, latitude, longitude, house_system="P", zodiac_type="tropical", ayanamsa="galcent_0sag", orb_settings=None, dignity_scheme="lilly"))]
    #[allow(clippy::too_many_arguments)]
    fn transit(
        &self,
        py: Python<'_>,
        natal_planets: &Bound<'_, PyAny>,
        moment: &Bound<'_, PyAny>,
        latitude: f64,
        longitude: f64,
        house_system: &str,
        zodiac_type: &str,
        ayanamsa: &str,
        orb_settings: Option<&Bound<'_, PyAny>>,
        dignity_scheme: &str,
    ) -> PyResult<Py<PyAny>> {
        let natal = to_value(natal_planets)?;
        let req = Request::new(
            moment,
            latitude,
            longitude,
            house_system,
            zodiac_type,
            ayanamsa,
            orb_settings,
        )?
        .with_dignity_scheme(dignity_scheme);
        let chart = py
            .detach(|| calculate_transit_chart(&self.kernels, &natal, &req.as_chart_request()))
            .map_err(to_py_err)?;
        to_py(py, &chart)
    }
}

/// Cross-aspects from chart B's planets (overlay) to chart A's (base).
#[pyfunction]
#[pyo3(signature = (chart_a_planets, chart_b_planets, orb_settings=None))]
fn synastry(
    py: Python<'_>,
    chart_a_planets: &Bound<'_, PyAny>,
    chart_b_planets: &Bound<'_, PyAny>,
    orb_settings: Option<&Bound<'_, PyAny>>,
) -> PyResult<Py<PyAny>> {
    let a = to_value(chart_a_planets)?;
    let b = to_value(chart_b_planets)?;
    let orbs = optional_value(orb_settings)?;
    let result = calculate_synastry(&a, &b, orbs.as_ref()).map_err(to_py_err)?;
    to_py(py, &result)
}

/// A derived (turned) chart from a chart dict: radix house `root_house` becomes house 1.
#[pyfunction]
#[pyo3(signature = (base_chart, root_house, custom_name=None))]
fn derived_chart(
    py: Python<'_>,
    base_chart: &Bound<'_, PyAny>,
    root_house: i64,
    custom_name: Option<&str>,
) -> PyResult<Py<PyAny>> {
    let base = to_value(base_chart)?;
    let result = calculate_derived_chart(&base, root_house, custom_name).map_err(to_py_err)?;
    to_py(py, &result)
}

/// A smaller kernel (bytes) covering TDB Julian dates `start_jd`..`end_jd` of the kernel
/// at `path`, with identical positions inside that range (e.g. to ship 1900-2100 of
/// de440s with an app).
#[pyfunction]
fn excerpt_kernel(
    py: Python<'_>,
    path: PathBuf,
    start_jd: f64,
    end_jd: f64,
) -> PyResult<Py<pyo3::types::PyBytes>> {
    let bytes = py
        .detach(|| Spk::open(&path)?.excerpt(start_jd, end_jd))
        .map_err(|e| invalid_kernel(format!("{}: {e}", path.display())))?;
    Ok(pyo3::types::PyBytes::new(py, &bytes).unbind())
}

/// Julian day of a moment, as the chart calculation uses it.
#[pyfunction]
fn julian_day(moment: &Bound<'_, PyAny>) -> PyResult<f64> {
    Ok(to_instant(moment)?.julian_day())
}

/// Lilly's qualities of the degree an ecliptic longitude falls in.
#[pyfunction]
fn degree_qualities(py: Python<'_>, longitude: f64) -> PyResult<Py<PyAny>> {
    to_py(py, &core_degree_qualities(longitude))
}

/// Lilly's table of the degree qualities, one dict per sign from Aries.
#[pyfunction]
fn degree_quality_table(py: Python<'_>) -> PyResult<Py<PyAny>> {
    to_py(py, core_degree_quality_table())
}

/// Annual profections and firdaria of a nativity (birth moment, natal Sun and Ascendant
/// longitudes) for the years that overlap `start`..`end`.
#[pyfunction]
fn time_lords(
    py: Python<'_>,
    birth: &Bound<'_, PyAny>,
    sun_longitude: f64,
    ascendant_longitude: f64,
    start: &Bound<'_, PyAny>,
    end: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let lords = core_time_lords(
        to_instant(birth)?,
        sun_longitude,
        ascendant_longitude,
        to_instant(start)?,
        to_instant(end)?,
    )
    .map_err(to_py_err)?;
    to_py(py, &lords)
}

/// A computed chart's dignities (condition, sect, receptions) judged again under
/// `dignity_scheme` ("lilly" or "dorothean"). No kernel is needed.
#[pyfunction]
#[pyo3(signature = (chart, dignity_scheme="lilly"))]
fn chart_dignities(
    py: Python<'_>,
    chart: &Bound<'_, PyAny>,
    dignity_scheme: &str,
) -> PyResult<Py<PyAny>> {
    let chart = to_value(chart)?;
    let judged = core_chart_dignities(&chart, dignity_scheme).map_err(to_py_err)?;
    to_py(py, &judged)
}

/// The Four Pillars of a birth at `moment`, at `longitude` (degrees east) where civil time
/// was `utc_offset_minutes` ahead of UTC, from its Chinese `calendar` (the chart's
/// `chinese_calendar`). No kernel is needed.
#[pyfunction]
#[pyo3(signature = (calendar, moment, longitude, utc_offset_minutes, solar_time=true, zi_hour="next_day", sex=None, luck_pillars=10, year=None, date=None))]
#[allow(clippy::too_many_arguments)]
fn bazi(
    py: Python<'_>,
    calendar: &Bound<'_, PyAny>,
    moment: &Bound<'_, PyAny>,
    longitude: f64,
    utc_offset_minutes: f64,
    solar_time: bool,
    zi_hour: &str,
    sex: Option<&str>,
    luck_pillars: u8,
    year: Option<i32>,
    date: Option<&str>,
) -> PyResult<Py<PyAny>> {
    let calendar: ChineseCalendar =
        serde_json::from_value(to_value(calendar)?).map_err(invalid_input)?;
    let options = BaziOptions {
        solar_time,
        zi_hour,
        sex,
        luck_pillars,
        year,
        date,
    };
    let pillars = core_bazi(
        &calendar,
        to_instant(moment)?,
        longitude,
        utc_offset_minutes,
        &options,
    )
    .map_err(to_py_err)?;
    to_py(py, &pillars)
}

/// The Zi Wei Dou Shu chart of a birth at `moment`, at `longitude` (degrees east) where
/// civil time was `utc_offset_minutes` ahead of UTC, from its Chinese `calendar` (the
/// chart's `chinese_calendar`). No kernel is needed.
#[pyfunction]
#[pyo3(signature = (calendar, moment, longitude, utc_offset_minutes, solar_time=true, zi_hour="next_day", sex=None, leap_month="split", year=None, date=None))]
#[allow(clippy::too_many_arguments)]
fn zi_wei(
    py: Python<'_>,
    calendar: &Bound<'_, PyAny>,
    moment: &Bound<'_, PyAny>,
    longitude: f64,
    utc_offset_minutes: f64,
    solar_time: bool,
    zi_hour: &str,
    sex: Option<&str>,
    leap_month: &str,
    year: Option<i32>,
    date: Option<&str>,
) -> PyResult<Py<PyAny>> {
    let calendar: ChineseCalendar =
        serde_json::from_value(to_value(calendar)?).map_err(invalid_input)?;
    let options = ZiWeiOptions {
        solar_time,
        zi_hour,
        sex,
        leap_month,
        year,
        date,
    };
    let chart = core_zi_wei(
        &calendar,
        to_instant(moment)?,
        longitude,
        utc_offset_minutes,
        &options,
    )
    .map_err(to_py_err)?;
    to_py(py, &chart)
}

#[pymodule(name = "astroceleste_engine")]
fn py_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<Engine>()?;
    m.add_function(wrap_pyfunction!(synastry, m)?)?;
    m.add_function(wrap_pyfunction!(derived_chart, m)?)?;
    m.add_function(wrap_pyfunction!(julian_day, m)?)?;
    m.add_function(wrap_pyfunction!(excerpt_kernel, m)?)?;
    m.add_function(wrap_pyfunction!(degree_qualities, m)?)?;
    m.add_function(wrap_pyfunction!(degree_quality_table, m)?)?;
    m.add_function(wrap_pyfunction!(time_lords, m)?)?;
    m.add_function(wrap_pyfunction!(chart_dignities, m)?)?;
    m.add_function(wrap_pyfunction!(bazi, m)?)?;
    m.add_function(wrap_pyfunction!(zi_wei, m)?)?;
    m.add("EngineError", m.py().get_type::<EngineError>())?;
    m.add(
        "EphemerisRangeError",
        m.py().get_type::<EphemerisRangeError>(),
    )?;
    Ok(())
}
