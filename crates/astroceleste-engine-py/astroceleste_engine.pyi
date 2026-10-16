"""Astrological chart calculation on JPL ephemerides."""

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

# Invalid input raises ValueError, with `code` "invalid_input".

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
    ) -> dict[str, Any]: ...
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
    ) -> dict[str, Any]:
        """A chart with its electional assessment under `election_data`."""
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
    ) -> dict[str, Any]:
        """The best electional windows from `start` to `end` (at most 92 days)."""
    def chinese_calendar(self, moment: Moment) -> dict[str, Any] | None:
        """The Chinese calendar around a moment, or None outside the kernels' coverage."""
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
) -> bytes:
    """A smaller kernel with identical positions over TDB Julian dates start_jd..end_jd."""
def degree_qualities(longitude: float) -> dict[str, Any]:
    """Lilly's qualities of the degree an ecliptic longitude falls in."""
def degree_quality_table() -> list[dict[str, Any]]:
    """Lilly's table of the degree qualities, one dict per sign from Aries."""
def time_lords(
    birth: Moment,
    sun_longitude: float,
    ascendant_longitude: float,
    start: Moment,
    end: Moment,
) -> dict[str, Any]:
    """Annual profections and firdaria of a nativity for the years overlapping start..end."""
def chart_dignities(chart: dict[str, Any], dignity_scheme: str = "lilly") -> dict[str, Any]:
    """A computed chart's condition, sect and receptions judged again under a scheme."""
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
) -> dict[str, Any]:
    """The Four Pillars of a birth from its Chinese calendar (a chart's chinese_calendar)."""
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
) -> dict[str, Any]:
    """The Zi Wei Dou Shu chart of a birth from its Chinese calendar (a chart's chinese_calendar)."""
