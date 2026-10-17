#!/usr/bin/env python3
"""
Khagole Vedic Jyotish REST API Microservice Example
Built with FastAPI and khagole-engine on NASA JPL DE440s ephemerides.
"""

from pathlib import Path
from typing import Any, Optional
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field
import khagole_engine as ke

app = FastAPI(
    title="Khagole Vedic Jyotish REST API",
    description="High-precision Vedic astrological chart, Panchanga, Divisional Charts, and Vimshottari Dasha API on NASA JPL ephemerides.",
    version=ke.__version__,
)

def find_kernel() -> str:
    candidates = [
        Path("kernels/de440s.bsp"),
        Path("../../kernels/de440s.bsp"),
        Path("../kernels/de440s.bsp"),
        Path(__file__).resolve().parent.parent.parent / "kernels" / "de440s.bsp",
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    raise FileNotFoundError("NASA JPL kernel de440s.bsp not found.")

kernel_path = find_kernel()
engine = ke.Engine([kernel_path])

CELEBRITY_PROFILES = {
    "jobs": {
        "name": "Steve Jobs",
        "utc": "1955-02-25T03:15:00Z",
        "local_time": "1955-02-24 19:15 PST",
        "location": "San Francisco, CA, USA",
        "latitude": 37.7749,
        "longitude": -122.4194,
    },
    "einstein": {
        "name": "Albert Einstein",
        "utc": "1879-03-14T10:50:00Z",
        "local_time": "1879-03-14 11:30 LMT",
        "location": "Ulm, Germany",
        "latitude": 48.4011,
        "longitude": 9.9876,
    },
    "modi": {
        "name": "Narendra Modi",
        "utc": "1950-09-17T04:04:00Z",
        "local_time": "1950-09-17 09:34 IST",
        "location": "Vadnagar, Gujarat, India",
        "latitude": 23.7840,
        "longitude": 72.6360,
    },
}

class ChartRequestModel(BaseModel):
    utc: str = Field(..., example="1955-02-25T03:15:00Z", description="UTC birth datetime in ISO 8601")
    latitude: float = Field(..., example=37.7749, description="Geographic latitude in degrees")
    longitude: float = Field(..., example=-122.4194, description="Geographic longitude in degrees")
    zodiac_type: str = Field("sidereal", example="sidereal", description="'sidereal' (Vedic) or 'tropical' (Western)")
    ayanamsa: str = Field("lahiri", example="lahiri", description="Ayanamsa system: 'lahiri', 'raman', 'krishnamurti'")
    house_system: str = Field("W", example="W", description="House system: 'W' (Whole Sign), 'S' (Sripati), 'P' (Placidus)")
    position_mode: str = Field("true", example="true", description="'true' (Geometric Vedic) or 'apparent' (Western)")
    node_type: str = Field("mean", example="mean", description="'mean' or 'true' lunar nodes")

class PanchangRequestModel(BaseModel):
    sun_longitude: float = Field(..., example=150.5392)
    moon_longitude: float = Field(..., example=217.9496)
    utc: Optional[str] = Field(None, example="1950-09-17T04:04:00Z")
    jd: Optional[float] = Field(None, example=2433541.6694)

class DashaRequestModel(BaseModel):
    moon_longitude: float = Field(..., example=217.9496)
    utc: Optional[str] = Field(None, example="1950-09-17T04:04:00Z")
    birth_jd: Optional[float] = Field(None, example=2433541.6694)
    depth: int = Field(2, example=2, description="Hierarchy depth: 1 (Mahadasha), 2 (Antardasha), 3 (Pratyantardasha)")
    year_type: str = Field("tssy", example="tssy", description="Year type: 'tssy', 'calendar', 'savana', 'sidereal'")

class DivisionalRequestModel(BaseModel):
    chart: Any = Field(..., description="Natal chart dict or list of planets")
    varga: str = Field("D9", example="D9", description="Divisional chart code (D1 to D60)")
    lagna_longitude: Optional[float] = Field(None, example=192.2897)

@app.get("/health")
def health_check():
    return {
        "status": "healthy",
        "engine": "Khagole",
        "version": ke.__version__,
        "ephemeris_kernels": engine.kernels,
    }

@app.post("/api/chart")
def calculate_horoscope(req: ChartRequestModel):
    try:
        chart = engine.chart(
            req.utc,
            req.latitude,
            req.longitude,
            house_system=req.house_system,
            zodiac_type=req.zodiac_type,
            ayanamsa=req.ayanamsa,
            position_mode=req.position_mode,
            node_type=req.node_type,
        )
        return chart
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@app.get("/api/celebrity/{name}")
def get_celebrity_horoscope(name: str):
    key = name.lower()
    if key not in CELEBRITY_PROFILES:
        raise HTTPException(status_code=404, detail=f"Celebrity '{name}' not found. Available: {list(CELEBRITY_PROFILES.keys())}")
    
    p = CELEBRITY_PROFILES[key]
    chart = engine.chart(
        p["utc"],
        p["latitude"],
        p["longitude"],
        zodiac_type="sidereal",
        ayanamsa="lahiri",
        house_system="W",
        position_mode="true",
        node_type="mean",
    )

    # Compute Panchang & Dasha
    sun = next(pt for pt in chart["planets"] if pt["name"] == "Sun")
    moon = next(pt for pt in chart["planets"] if pt["name"] == "Moon")
    jd = ke.julian_day(p["utc"])

    panchang = ke.calc_panchang(sun["ecliptic_longitude"], moon["ecliptic_longitude"], jd)
    d9 = ke.calc_divisional_chart(chart, varga="D9")
    dasha = ke.calc_vimshottari_dasha(moon["ecliptic_longitude"], jd, depth=2)

    return {
        "profile": p,
        "chart": chart,
        "panchang": panchang,
        "navamsha_d9": d9,
        "vimshottari_dasha": dasha[:5],
    }

@app.post("/api/panchang")
def get_panchanga(req: PanchangRequestModel):
    jd = req.jd
    if jd is None:
        if req.utc is not None:
            jd = ke.julian_day(req.utc)
        else:
            jd = 2451545.0
    return ke.calc_panchang(req.sun_longitude, req.moon_longitude, jd)

@app.post("/api/dasha")
def get_vimshottari_dasha(req: DashaRequestModel):
    jd = req.birth_jd
    if jd is None:
        if req.utc is not None:
            jd = ke.julian_day(req.utc)
        else:
            raise HTTPException(status_code=400, detail="Must provide 'birth_jd' or 'utc'")
    return ke.calc_vimshottari_dasha(req.moon_longitude, jd, depth=req.depth, year_type=req.year_type)

@app.post("/api/divisional")
def get_divisional_chart(req: DivisionalRequestModel):
    return ke.calc_divisional_chart(req.chart, varga=req.varga, lagna_longitude=req.lagna_longitude)

if __name__ == "__main__":
    import uvicorn
    print("Starting Khagole REST API server on http://localhost:8000 ...")
    uvicorn.run(app, host="0.0.0.0", port=8000)
