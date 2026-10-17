"""Vedic Jyotish & Astrological chart calculation on NASA JPL ephemerides."""

from datetime import datetime
from os import PathLike
from typing import Any

__version__: str

Moment = datetime | str
"""An aware (or naive, taken as UTC) datetime, or an ISO 8601 string."""

class EngineError(Exception):
    """A kernel could not be read (`code` "invalid_kernel") or evaluated ("ephemeris_error")."""

    code: str

class EphemerisRangeError(EngineError):
    """No loaded kernel covers the date (`code` "ephemeris_out_of_range")."""

class Engine:
    """Chart calculator over JPL kernels, given in preference order."""

    def __init__(self, kernels: list[str | PathLike[str]]) -> None: ...
    @property
    def kernels(self) -> list[tuple[str, float, float]]:
        """(name, first JD, last JD) of each loaded kernel."""
    @property
    def coverage(self) -> tuple[float, float] | None: ...
    def supports(self, jd: float) -> bool: ...
    def chart(
        self,
        moment: Moment,
        latitude: float,
        longitude: float,
        house_system: str = "P",
        zodiac_type: str = "tropical",
        ayanamsa: str = "galcent_0sag",
        orb_settings: dict[str, Any] | None = None,
        dignity_scheme: str = "lilly",
        chinese_calendar: bool = False,
        position_mode: str = "apparent",
        node_type: str = "mean",
    ) -> dict[str, Any]:
        """Compute natal chart. For Vedic sidereal: zodiac_type='sidereal', ayanamsa='lahiri', house_system='W', position_mode='true'."""
    def horary(
        self,
        moment: Moment,
        latitude: float,
        longitude: float,
        house_system: str = "P",
        zodiac_type: str = "tropical",
        ayanamsa: str = "galcent_0sag",
        orb_settings: dict[str, Any] | None = None,
        dignity_scheme: str = "lilly",
        quesited_house: int | None = None,
        chinese_calendar: bool = False,
    ) -> dict[str, Any]: ...
    def transit(
        self,
        natal_planets: list[dict[str, Any]],
        moment: Moment,
        latitude: float,
        longitude: float,
        house_system: str = "P",
        zodiac_type: str = "tropical",
        ayanamsa: str = "galcent_0sag",
        orb_settings: dict[str, Any] | None = None,
        dignity_scheme: str = "lilly",
    ) -> dict[str, Any]: ...
    def election(
        self,
        moment: Moment,
        latitude: float,
        longitude: float,
        criteria: dict[str, Any] | None = None,
        house_system: str = "P",
        zodiac_type: str = "tropical",
        ayanamsa: str = "galcent_0sag",
        orb_settings: dict[str, Any] | None = None,
        dignity_scheme: str = "lilly",
        chinese_calendar: bool = False,
    ) -> dict[str, Any]: ...
    def elections(
        self,
        start: Moment,
        end: Moment,
        latitude: float,
        longitude: float,
        criteria: dict[str, Any] | None = None,
        house_system: str = "P",
        zodiac_type: str = "tropical",
        ayanamsa: str = "galcent_0sag",
        orb_settings: dict[str, Any] | None = None,
        dignity_scheme: str = "lilly",
        chinese_calendar: bool = False,
    ) -> list[dict[str, Any]]: ...
    def chinese_calendar(self, moment: Moment) -> dict[str, Any] | None: ...

def synastry(
    chart_a_planets: list[dict[str, Any]],
    chart_b_planets: list[dict[str, Any]],
    orb_settings: dict[str, Any] | None = None,
) -> dict[str, Any]: ...
def derived_chart(
    base_chart: dict[str, Any], root_house: int, custom_name: str | None = None
) -> dict[str, Any]: ...
def julian_day(moment: Moment) -> float: ...
def excerpt_kernel(
    path: str | PathLike[str], start_jd: float, end_jd: float
) -> bytes: ...
def degree_qualities(longitude: float) -> dict[str, Any]: ...
def degree_quality_table() -> list[dict[str, Any]]: ...
def time_lords(
    birth: Moment,
    sun_longitude: float,
    ascendant_longitude: float,
    start: Moment,
    end: Moment,
) -> dict[str, Any]: ...
def chart_dignities(chart: dict[str, Any], dignity_scheme: str = "lilly") -> dict[str, Any]: ...
def bazi(
    calendar: dict[str, Any],
    moment: Moment,
    longitude: float,
    utc_offset_minutes: float,
    solar_time: bool = True,
    zi_hour: str = "next_day",
    sex: str | None = None,
    luck_pillars: int = 10,
    year: int | None = None,
    date: str | None = None,
) -> dict[str, Any]: ...
def zi_wei(
    calendar: dict[str, Any],
    moment: Moment,
    longitude: float,
    utc_offset_minutes: float,
    solar_time: bool = True,
    zi_hour: str = "next_day",
    sex: str | None = None,
    leap_month: str = "split",
    year: int | None = None,
    date: str | None = None,
) -> dict[str, Any]: ...

# Vedic Jyotish subsystem
def calc_panchang(
    sun_longitude: float,
    moon_longitude: float,
    jd: float,
    weekday: int | None = None,
    is_before_sunrise: bool | None = None,
) -> dict[str, Any]:
    """Calculate the Five Limbs of Vedic Time (Tithi, Vara, Nakshatra, Yoga, Karana, Masa, Samvat)."""

def calc_divisional_chart(
    chart_or_planets: dict[str, Any] | list[dict[str, Any]],
    varga: str = "D9",
    lagna_longitude: float | None = None,
) -> dict[str, Any]:
    """Calculate a Vedic divisional chart (D1 to D60: D9 Navamsha, D10 Dashamsha, etc.)."""

def calc_divisional_point(longitude: float, varga: str = "D9") -> dict[str, Any]:
    """Transform a single sidereal longitude to its divisional position."""

def calc_vimshottari_dasha(
    moon_longitude: float,
    birth_jd: float,
    depth: int = 3,
    year_type: str = "tssy",
    tz_offset_hours: float = 0.0,
) -> list[dict[str, Any]]:
    """Calculate the 120-year Vimshottari Dasha timeline tree (Mahadasha, Antardasha, Pratyantardasha)."""

def nakshatra_info(longitude: float) -> dict[str, Any]:
    """Get Nakshatra name, Vimshottari lord, and Pada (1-4) for any sidereal longitude."""

def ayanamsa_info(jd: float, ayanamsa_name: str = "lahiri") -> dict[str, Any]:
    """Get precise ayanamsa precession value and DMS string for any Julian Day."""
