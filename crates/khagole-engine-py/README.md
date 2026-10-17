# khagole-engine (Python)

[![PyPI](https://img.shields.io/pypi/v/khagole-engine.svg)](https://pypi.org/project/khagole-engine/)

> **Experimental (0.0.x):** the API may change in any release. Pin an exact version.

Python bindings for [khagole-engine](https://github.com/ffalcinelli/khagole-engine),
astrological chart calculation on NASA JPL ephemerides, written in Rust.

[Website and live demo](https://priyankgahtori.github.io/Khagole/) ·
[API guide](https://github.com/PriyankGahtori/Khagole/blob/main/docs/api.md)

```python
from datetime import datetime, timezone
import khagole_engine as ace

engine = ace.Engine(["de440s.bsp"])            # JPL kernels, in preference order
chart = engine.chart(datetime(1987, 5, 17, 14, 30, tzinfo=timezone.utc), 41.9, 12.5,
                     house_system="P", zodiac_type="sidereal", ayanamsa="lahiri")
chart["planets"][0]   # {'name': 'Sun', 'sign': 'Taurus', 'ecliptic_longitude': ..., ...}

engine.horary(moment, lat, lon)                 # chart + chart["horary_data"]
engine.transit(natal["planets"], moment, lat, lon)
ace.synastry(chart_a["planets"], chart_b["planets"])
ace.derived_chart(chart, 5)
```

Dates outside the loaded kernels raise `khagole_engine.EphemerisRangeError`.
Kernels: https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/ (de440s.bsp covers 1849-2150).

Licensed under MIT OR Apache-2.0.
