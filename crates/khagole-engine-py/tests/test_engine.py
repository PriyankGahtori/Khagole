"""Python bindings: API behaviour, and the golden charts through the Python layer."""

import json
import math
import os
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest

import khagole_engine as ace

ROOT = Path(__file__).resolve().parents[3]
EXCERPT = ROOT / "tests" / "data" / "de440s_2000.bsp"
FULL = ROOT / "kernels" / "de440s.bsp"
FIXTURES = ROOT / "tests" / "fixtures"
PRIVATE_KEYS = {"degree_symbol", "degree_symbols", "condition"}
# Chart keys added after the reference implementation, absent from the fixtures.
ADDED_CHART_KEYS = {"sect", "dignity_scheme", "receptions", "antiscia", "planetary_hours"}

# Skipped without the full kernel, unless KHAGOLE_REQUIRE_KERNEL is set (as in CI): then
# the tests run, and fail on the missing file.
needs_full_kernel = pytest.mark.skipif(
    not FULL.exists() and not (os.environ.get("KHAGOLE_REQUIRE_KERNEL") or os.environ.get("ASTROCELESTE_REQUIRE_KERNEL")),
    reason="scripts/fetch-kernels.sh",
)


# Horary considerations added after the reference implementation.
ADDED_STRICTURES = {"MOON_VIA_COMBUSTA", "MOON_LATE_DEGREES", "ASC_RULER_COMBUST"}


def strip_private(value):
    if isinstance(value, dict):
        value = {k: v for k, v in value.items() if k not in ADDED_CHART_KEYS}
        if isinstance(value.get("horary_data"), dict):
            horary = {k: v for k, v in value["horary_data"].items() if k != "judgment"}
            horary["strictures"] = [
                s for s in horary.get("strictures", []) if s["code"] not in ADDED_STRICTURES
            ]
            value["horary_data"] = horary
    return strip_nested(value)


def strip_nested(value):
    if isinstance(value, dict):
        return {k: strip_nested(v) for k, v in value.items() if k not in PRIVATE_KEYS}
    if isinstance(value, list):
        return [strip_nested(v) for v in value]
    return value


def assert_same(actual, expected, path="$", tol=1e-9):
    if isinstance(expected, dict):
        assert list(actual) == list(expected), f"{path}: keys"
        for key in expected:
            assert_same(actual[key], expected[key], f"{path}.{key}", tol)
    elif isinstance(expected, list):
        assert len(actual) == len(expected), f"{path}: length"
        for i, (a, e) in enumerate(zip(actual, expected, strict=True)):
            assert_same(a, e, f"{path}[{i}]", tol)
    elif isinstance(expected, float):
        assert isinstance(actual, float), f"{path}: {actual!r} is not a float"
        assert math.isclose(actual, expected, rel_tol=tol, abs_tol=tol), f"{path}: {actual} != {expected}"
    else:
        assert type(actual) is type(expected) and actual == expected, f"{path}: {actual!r} != {expected!r}"


@pytest.fixture(scope="module")
def excerpt():
    return ace.Engine([EXCERPT])


def test_engine_reports_its_kernels(excerpt):
    ((name, start, end),) = excerpt.kernels
    assert name == "de440s_2000.bsp"
    assert start < 2451545.0 + 30 < end
    assert excerpt.coverage == (start, end)
    assert excerpt.supports(2451600.0) and not excerpt.supports(2460000.0)


def test_datetime_string_and_offset_are_the_same_instant(excerpt):
    utc = datetime(2000, 6, 1, 12, 0, tzinfo=timezone.utc)
    rome = utc.astimezone(timezone(timedelta(hours=2)))
    a = excerpt.chart(utc, 41.9, 12.5)
    assert a == excerpt.chart(rome, 41.9, 12.5)
    assert a == excerpt.chart("2000-06-01T12:00:00Z", 41.9, 12.5)
    assert a == excerpt.chart(utc.replace(tzinfo=None), 41.9, 12.5)
    assert ace.julian_day(utc) == 2451697.0


def test_chart_shape(excerpt):
    chart = excerpt.chart("2000-06-01T12:00:00Z", 41.9, 12.5, zodiac_type="sidereal", ayanamsa="lahiri")
    assert chart["zodiac_type"] == "sidereal" and chart["ayanamsa"] == "lahiri"
    names = [p["name"] for p in chart["planets"]]
    assert names[:2] == ["Sun", "Moon"] and names[-2:] == ["Ascendant", "Midheaven"]
    assert len(chart["houses"]) == 12
    json.dumps(chart)  # plain JSON types only


def test_out_of_range_raises(excerpt):
    with pytest.raises(ace.EphemerisRangeError) as raised:
        excerpt.chart("2010-01-01T00:00:00Z", 41.9, 12.5)
    assert raised.value.code == "ephemeris_out_of_range"
    assert issubclass(ace.EphemerisRangeError, ace.EngineError)


def test_bad_input_raises_value_error(excerpt):
    bad = [
        lambda: excerpt.chart("not a date", 41.9, 12.5),
        lambda: excerpt.chart("2000-02-30T12:00:00Z", 41.9, 12.5),
        lambda: excerpt.chart("2000-06-01T12:00:00Z", 41.9, 12.5, orb_settings={"fixed_star_orb": "wide"}),
        lambda: excerpt.chart("2000-06-01T12:00:00Z", 41.9, 12.5, orb_settings={"fixed_star_orb": "inf"}),
        lambda: excerpt.chart("2000-06-01T12:00:00Z", 91.0, 12.5),
        lambda: excerpt.chart("2000-06-01T12:00:00Z", float("nan"), 12.5),
        lambda: excerpt.horary("2000-06-01T12:00:00Z", 41.9, 12.5, quesited_house=13),
        lambda: ace.time_lords("1990-01-01T00:00:00Z", 280.0, 130.0, "1990-01-01T00:00:00Z", "3500-01-01T00:00:00Z"),
    ]
    for call in bad:
        with pytest.raises(ValueError) as raised:
            call()
        assert raised.value.code == "invalid_input"


def test_unreadable_kernels_are_invalid_kernel(tmp_path):
    with pytest.raises(ace.EngineError, match="no_such.bsp") as raised:
        ace.Engine(["no_such.bsp"])
    assert raised.value.code == "invalid_kernel"
    garbage = tmp_path / "garbage.bsp"
    garbage.write_bytes(b"DAF/SPK " + bytes(2048))
    with pytest.raises(ace.EngineError) as raised:
        ace.Engine([garbage])
    assert raised.value.code == "invalid_kernel"
    with pytest.raises(ace.EngineError) as raised:
        ace.excerpt_kernel(garbage, 2451600.0, 2451700.0)
    assert raised.value.code == "invalid_kernel"


def test_excerpt_kernel_keeps_positions(excerpt, tmp_path):
    small = tmp_path / "spring.bsp"
    small.write_bytes(ace.excerpt_kernel(EXCERPT, 2451620.0, 2451720.0))
    engine = ace.Engine([small])
    start, end = engine.coverage
    assert start <= 2451620.0 and 2451720.0 <= end < excerpt.coverage[1]
    assert engine.chart("2000-04-15T12:00:00Z", 41.9, 12.5) == excerpt.chart("2000-04-15T12:00:00Z", 41.9, 12.5)


def test_horary_judgment(excerpt):
    chart = excerpt.horary("2000-06-01T12:00:00Z", 41.9, 12.5, quesited_house=7)
    judgment = chart["horary_data"]["judgment"]
    assert judgment["quesited_house"] == 7
    assert judgment["querent"] and judgment["co_significator"] == "Moon"
    assert excerpt.horary("2000-06-01T12:00:00Z", 41.9, 12.5)["horary_data"]["judgment"]["quesited_house"] is None


def test_chinese_calendar_outside_coverage_is_none(excerpt):
    # The calendar needs about 14 months of ephemeris before the moment.
    assert excerpt.chinese_calendar("2000-06-01T12:00:00Z") is None


def load(name):
    return json.loads((FIXTURES / name).read_text())


@needs_full_kernel
def test_golden_natal_and_horary():
    engine = ace.Engine([FULL])
    for name, compute in (("natal.json", engine.chart), ("horary.json", engine.horary)):
        for case in load(name):
            i = case["input"]
            actual = compute(i["utc"], i["lat"], i["lon"], i["house_system"], i["zodiac_type"], i["ayanamsa"])
            assert_same(strip_private(actual), case["output"], i["id"])


@needs_full_kernel
def test_golden_transits_synastry_derived():
    engine = ace.Engine([FULL])
    natal = {c["input"]["id"]: c["output"] for c in load("natal.json")}
    for case in load("transits.json"):
        t = case["input"]["transit"]
        actual = engine.transit(
            natal[case["input"]["natal"]]["planets"],
            t["utc"], t["lat"], t["lon"], t["house_system"], t["zodiac_type"], t["ayanamsa"],
        )
        assert_same(strip_private(actual), case["output"], t["id"])
    for case in load("synastry.json"):
        a, b = natal[case["input"]["a"]], natal[case["input"]["b"]]
        assert_same(ace.synastry(a["planets"], b["planets"]), case["output"])
    for case in load("derived.json"):
        base = natal[case["input"]["base"]]
        actual = ace.derived_chart(base, case["input"]["root_house"])
        assert_same(strip_private(actual), case["output"], case["input"]["base"], tol=0.0)


def test_election_criteria_are_validated(excerpt):
    with pytest.raises(ValueError, match="unknown variant"):
        excerpt.election("2000-06-01T12:00:00Z", 41.9, 12.5, {"purpose": "war"})
    with pytest.raises(ValueError, match="at most"):
        excerpt.elections("2000-01-01T00:00:00Z", "2000-06-01T00:00:00Z", 41.9, 12.5)


def test_election_and_search(excerpt):
    chart = excerpt.election("2000-06-01T12:00:00Z", 41.9, 12.5, {"purpose": "contract"})
    data = chart["election_data"]
    assert chart["planets"] and 0 <= data["score"] <= 100
    assert data["purpose"] == "contract"
    assert {"code", "weight", "planet", "target", "aspect", "house", "sign"} <= set(data["factors"][0])

    result = excerpt.elections(
        datetime(2000, 6, 1, tzinfo=timezone.utc), "2000-06-08T00:00:00Z", 41.9, 12.5, {"step_minutes": 30}
    )
    assert result["criteria"]["step_minutes"] == 30
    assert result["evaluated"] > 0
    for window in result["windows"]:
        assert window["start"] <= window["best"] <= window["end"]
        best = excerpt.election(window["best"], 41.9, 12.5)["election_data"]
        assert best["score"] == window["score"]
    json.dumps(result)


def test_degree_qualities():
    # 5°30' Aries: the 6th degree, pitted, in the light run that ends at the 8th.
    q = ace.degree_qualities(5.5)
    assert q == {
        "sign": "Aries",
        "degree": 6,
        "gender": "masculine",
        "light": "light",
        "pitted": True,
        "azimene": False,
        "fortune": False,
    }
    table = ace.degree_quality_table()
    assert [s["sign"] for s in table][:2] == ["Aries", "Taurus"]
    assert all(s["gender"][-1]["end"] == 30 and s["light"][-1]["end"] == 30 for s in table)
    assert table[1]["azimene"] == [6, 7, 8, 9, 10]
    json.dumps(table)


def test_time_lords():
    # A night birth (the Sun below a Leo Ascendant): the firdaria start with the Moon.
    lords = ace.time_lords("1990-01-01T12:00:00Z", 280.0, 130.0, "2000-06-01T00:00:00Z", "2001-06-01T00:00:00Z")
    assert lords["diurnal"] is False
    assert [p["age"] for p in lords["profections"]] == [10, 11]
    assert lords["profections"][0]["sign"] == "Gemini"  # Leo + 10 signs
    assert lords["profections"][0]["lord"] == "Mercury"
    assert lords["firdaria"][0]["lord"] == "Saturn"  # years 9-20 of a night birth
    assert len(lords["firdaria"][0]["sub_periods"]) == 7
    json.dumps(lords)


@needs_full_kernel
def test_chart_dignities_rejudge_a_stored_chart():
    engine = ace.Engine([FULL])
    lilly = engine.chart("1987-05-17T14:30:00Z", 41.9, 12.5)
    dorothean = engine.chart("1987-05-17T14:30:00Z", 41.9, 12.5, dignity_scheme="dorothean")
    judged = ace.chart_dignities(lilly, "dorothean")
    assert judged["dignity_scheme"] == "dorothean"
    by_name = {c["name"]: c["condition"] for c in judged["conditions"]}
    for p in dorothean["planets"]:
        if "condition" in p:
            assert by_name[p["name"]] == p["condition"]
    assert judged["receptions"] == dorothean["receptions"]
    with pytest.raises(ValueError):
        ace.chart_dignities({"planets": []})



@needs_full_kernel
def test_chinese_calendar_and_bazi():
    engine = ace.Engine([FULL])
    birth = "1940-11-27T15:12:00Z"  # Bruce Lee, San Francisco, 07:12 PST
    assert "chinese_calendar" not in engine.chart(birth, 37.77, -122.42)
    chart = engine.chart(birth, 37.77, -122.42, chinese_calendar=True)
    calendar = chart["chinese_calendar"]
    assert calendar == engine.chinese_calendar(birth)
    assert calendar["solar_terms"][0]["name"] == "lichun"
    pillars = ace.bazi(calendar, birth, -122.42, -480, sex="male")
    assert [(pillars[p]["stem"], pillars[p]["branch"]) for p in ("year", "month", "day", "hour")] == [
        ("geng", "chen"),
        ("ding", "hai"),
        ("jia", "xu"),
        ("wu", "chen"),
    ]
    assert pillars["luck"]["direction"] == "forward"
    assert ace.bazi(calendar, birth, -122.42, -480)["luck"] is None
    with pytest.raises(ValueError):
        ace.bazi(calendar, birth, -122.42, -480, zi_hour="noon")
    json.dumps(pillars)


@needs_full_kernel
def test_zi_wei():
    engine = ace.Engine([FULL])
    birth = "1940-11-27T15:12:00Z"
    calendar = engine.chart(birth, 37.77, -122.42, chinese_calendar=True)["chinese_calendar"]
    chart = ace.zi_wei(calendar, birth, -122.42, -480, solar_time=False, sex="male")
    assert (chart["life_palace"], chart["body_palace"]) == ("wei", "mao")
    assert chart["bureau"] == {"element": "wood", "number": 3}
    assert len(chart["palaces"]) == 12
    assert sum(len(p["stars"]) for p in chart["palaces"]) == 28
    assert ace.zi_wei(calendar, birth, -122.42, -480)["decade_direction"] is None
    assert sum(len(p["minor_stars"]) for p in chart["palaces"]) == 38
    horoscope = ace.zi_wei(calendar, birth, -122.42, -480, sex="male", year=2026)["horoscope"]
    assert horoscope["yearly"]["branch"] == "wu"
    daily = ace.zi_wei(calendar, birth, -122.42, -480, sex="male", date="2026-03-15")["horoscope"]
    assert daily["lunar_date"]["month"] == 1 and daily["daily"] is not None
    assert ace.bazi(calendar, birth, -122.42, -480, year=2026)["annual"]["stem"] == "bing"
    flow = ace.bazi(calendar, birth, -122.42, -480, date="2026-03-15")["flow"]
    assert (flow["month"]["stem"], flow["month"]["branch"]) == ("xin", "mao")
    with pytest.raises(ValueError):
        ace.zi_wei(calendar, birth, -122.42, -480, leap_month="never")
    json.dumps(chart)
