#!/usr/bin/env python3
"""
Khagole Hosted & Local Model Context Protocol (MCP) Server
Enables AI Agents (Claude Desktop, Cursor, LangChain, CrewAI, AutoGen, Antigravity)
to perform high-precision Vedic Jyotish & NASA JPL ephemerides calculations.

Supported Transports:
  1. SSE (Server-Sent Events) over HTTP for remote / cloud hosting:
     python server.py --transport sse --port 8000
  2. stdio for local AI desktop apps (Claude Desktop, Cursor):
     python server.py --transport stdio
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

from mcp.server.mcpserver import MCPServer
from starlette.responses import JSONResponse
import khagole_engine as ke

# Initialize MCP Server
mcp_server = MCPServer(
    name="khagole-mcp",
    version=ke.__version__,
    description="High-precision Vedic Jyotish & NASA JPL Ephemerides MCP Server for AI Agents",
)

def find_kernel() -> str:
    candidates = [
        os.getenv("KHAGOLE_KERNEL_PATH"),
        "kernels/de440s.bsp",
        "../../kernels/de440s.bsp",
        "../kernels/de440s.bsp",
        str(Path(__file__).resolve().parent.parent.parent / "kernels" / "de440s.bsp"),
    ]
    for c in candidates:
        if c and Path(c).exists():
            return str(Path(c).resolve())
    raise FileNotFoundError(
        "NASA JPL ephemeris kernel de440s.bsp not found. "
        "Run `scripts/fetch-kernels.sh` to download the kernel."
    )

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
    "modi": {
        "name": "Narendra Modi",
        "utc": "1950-09-17T04:04:00Z",
        "local_time": "1950-09-17 09:34 IST",
        "location": "Vadnagar, Gujarat, India",
        "latitude": 23.7840,
        "longitude": 72.6360,
    },
    "einstein": {
        "name": "Albert Einstein",
        "utc": "1879-03-14T10:50:00Z",
        "local_time": "1879-03-14 11:30 LMT",
        "location": "Ulm, Germany",
        "latitude": 48.4011,
        "longitude": 9.9876,
    },
    "sagan": {
        "name": "Carl Sagan",
        "utc": "1934-11-09T22:05:00Z",
        "local_time": "1934-11-09 17:05 EST",
        "location": "Brooklyn, NY, USA",
        "latitude": 40.6782,
        "longitude": -73.9442,
    },
}

NAVAGRAHA_NAMES = {
    "Sun": "Sun",
    "Moon": "Moon",
    "Mars": "Mars",
    "Mercury": "Mercury",
    "Jupiter": "Jupiter",
    "Venus": "Venus",
    "Saturn": "Saturn",
    "North Node": "Rahu",
    "South Node": "Ketu",
    "Ascendant": "Lagna (Asc)",
}

@mcp_server.tool()
def calculate_chart(
    utc: str,
    latitude: float,
    longitude: float,
    house_system: str = "W",
    zodiac_type: str = "sidereal",
    ayanamsa: str = "lahiri",
    position_mode: str = "true",
    node_type: str = "mean",
) -> str:
    """Calculate high-precision Vedic or Western astrological birth chart positions.
    
    Args:
        utc: Birth date and time in UTC ISO 8601 format (e.g. '1955-02-25T03:15:00Z')
        latitude: Geographic latitude in decimal degrees (-90 to +90)
        longitude: Geographic longitude in decimal degrees (-180 to +180)
        house_system: House system: 'W' (Whole Sign - Vedic default), 'S' (Sripati), 'P' (Placidus)
        zodiac_type: 'sidereal' (Vedic standard) or 'tropical' (Western)
        ayanamsa: Precession system: 'lahiri' (Chitra Paksha), 'raman', 'krishnamurti'
        position_mode: 'true' (Geometric Positions without light-time, Parashari default) or 'apparent'
        node_type: 'mean' (standard mean node) or 'true' (instantaneous osculating node)
    
    Returns:
        JSON string containing planetary coordinates, degrees, DMS, houses, nakshatras, and motion.
    """
    chart = engine.chart(
        utc,
        latitude,
        longitude,
        house_system=house_system,
        zodiac_type=zodiac_type,
        ayanamsa=ayanamsa,
        position_mode=position_mode,
        node_type=node_type,
    )
    
    # Enrich with Vedic metadata
    enriched_planets = []
    for p in chart.get("planets", []):
        raw_name = p.get("name")
        if raw_name in NAVAGRAHA_NAMES:
            lon = p.get("ecliptic_longitude", 0.0)
            nak = ke.nakshatra_info(lon)
            total_sec = round(lon * 3600.0)
            sec = total_sec % 60
            dms = f"{p.get('degree')}° {p.get('minute', 0):02d}′ {sec:02d}″"
            enriched_planets.append({
                "body": NAVAGRAHA_NAMES[raw_name],
                "rasi_sign": p.get("sign"),
                "degree": p.get("degree"),
                "minute": p.get("minute"),
                "second": sec,
                "dms": dms,
                "longitude": lon,
                "house": p.get("house"),
                "nakshatra": nak.get("name"),
                "pada": nak.get("pada"),
                "nakshatra_lord": nak.get("lord"),
                "is_retrograde": p.get("is_retrograde", False),
                "motion": "Retrograde" if p.get("is_retrograde") else "Direct",
            })
    
    result = {
        "ayanamsa_name": chart.get("ayanamsa_name", "Lahiri"),
        "ayanamsa_value": chart.get("ayanamsa_value"),
        "ayanamsa_formatted": chart.get("ayanamsa_formatted"),
        "navagrahas": enriched_planets,
        "raw_chart": chart,
    }
    return json.dumps(result, indent=2)

@mcp_server.tool()
def calculate_panchanga(
    sun_longitude: float,
    moon_longitude: float,
    utc: Optional[str] = None,
    jd: Optional[float] = None,
) -> str:
    """Calculate the five limbs of Vedic Time (Panchanga) and Hindu Samvatsara calendar eras.
    
    Args:
        sun_longitude: Ecliptic longitude of the Sun in degrees [0, 360)
        moon_longitude: Ecliptic longitude of the Moon in degrees [0, 360)
        utc: Optional UTC datetime ISO 8601 (e.g. '1955-02-25T03:15:00Z')
        jd: Optional Julian Day number. If omitted, computed from utc.
    
    Returns:
        JSON string containing Tithi, Vara, Nakshatra, Yoga, Karana, Masa, Ritu, Ayana, Vikram & Shaka Samvat.
    """
    julian = jd
    if julian is None:
        if utc:
            julian = ke.julian_day(utc)
        else:
            julian = 2451545.0
            
    panchang = ke.calc_panchang(sun_longitude, moon_longitude, julian)
    return json.dumps(panchang, indent=2)

@mcp_server.tool()
def calculate_divisional_chart(
    varga: str = "D9",
    utc: Optional[str] = None,
    latitude: Optional[float] = None,
    longitude: Optional[float] = None,
) -> str:
    """Calculate Vedic divisional charts (Shodashavarga D1 through D60).
    
    Args:
        varga: Divisional chart code: 'D1' (Rasi), 'D2' (Hora), 'D3' (Drekkana),
               'D7' (Saptamsha), 'D9' (Navamsha), 'D10' (Dashamsha), 'D12' (Dwadashamsha),
               'D16' (Shodashamsha), 'D20' (Vimshamsha), 'D24' (Siddhamsha), 'D27' (Nakshatramsha),
               'D30' (Trimshamsha), 'D60' (Shashtyamsha). Default: 'D9'.
        utc: Birth UTC date & time ISO 8601
        latitude: Birth geographic latitude
        longitude: Birth geographic longitude
    
    Returns:
        JSON string with mapped divisional signs for each planet and Lagna.
    """
    if utc and latitude is not None and longitude is not None:
        chart = engine.chart(
            utc, latitude, longitude,
            zodiac_type="sidereal", ayanamsa="lahiri", house_system="W", position_mode="true"
        )
        div = ke.calc_divisional_chart(chart, varga=varga)
        return json.dumps(div, indent=2)
    return json.dumps({"error": "Must provide 'utc', 'latitude', and 'longitude'"}, indent=2)

@mcp_server.tool()
def calculate_vimshottari_dasha(
    moon_longitude: float,
    birth_utc: str,
    depth: int = 2,
    year_type: str = "tssy",
) -> str:
    """Calculate the 120-year Vimshottari Dasha hierarchical timeline.
    
    Args:
        moon_longitude: Sidereal longitude of natal Moon in degrees [0, 360)
        birth_utc: Birth date and time in UTC ISO 8601 format
        depth: Hierarchy depth: 1 (Mahadasha), 2 (Antardasha), 3 (Pratyantardasha). Default: 2
        year_type: Solar year definition: 'tssy' (True Sidereal Solar Year - Parashari standard),
                   'calendar', 'savana' (360 days), 'sidereal'. Default: 'tssy'
    
    Returns:
        JSON string containing the nested Dasha sequence tree with exact start/end dates.
    """
    jd = ke.julian_day(birth_utc)
    tree = ke.calc_vimshottari_dasha(moon_longitude, jd, depth=depth, year_type=year_type)
    return json.dumps(tree, indent=2)

@mcp_server.tool()
def get_celebrity_chart(celebrity_id: str) -> str:
    """Instantly calculate a complete Vedic horoscope for a famous celebrity.
    
    Args:
        celebrity_id: One of: 'jobs' (Steve Jobs), 'modi' (Narendra Modi),
                      'einstein' (Albert Einstein), 'sagan' (Carl Sagan).
    
    Returns:
        Comprehensive JSON report with Chart, Panchanga, D9 Navamsha, and Vimshottari Dasha.
    """
    key = celebrity_id.lower().strip()
    if key not in CELEBRITY_PROFILES:
        return json.dumps({
            "error": f"Unknown celebrity '{celebrity_id}'. Available: {list(CELEBRITY_PROFILES.keys())}"
        }, indent=2)
    
    prof = CELEBRITY_PROFILES[key]
    chart = engine.chart(
        prof["utc"],
        prof["latitude"],
        prof["longitude"],
        zodiac_type="sidereal",
        ayanamsa="lahiri",
        house_system="W",
        position_mode="true",
        node_type="mean",
    )
    
    sun = next(p for p in chart["planets"] if p["name"] == "Sun")
    moon = next(p for p in chart["planets"] if p["name"] == "Moon")
    jd = ke.julian_day(prof["utc"])
    
    panchang = ke.calc_panchang(sun["ecliptic_longitude"], moon["ecliptic_longitude"], jd)
    d9 = ke.calc_divisional_chart(chart, varga="D9")
    dasha = ke.calc_vimshottari_dasha(moon["ecliptic_longitude"], jd, depth=2, year_type="tssy")
    
    return json.dumps({
        "celebrity": prof,
        "ayanamsa": f"{chart.get('ayanamsa_name')} ({chart.get('ayanamsa_formatted')})",
        "panchanga": panchang,
        "navamsha_d9": d9,
        "vimshottari_dasha": dasha[:5],
    }, indent=2)

def create_asgi_app():
    """Create Starlette ASGI app with SSE routes and health check."""
    app = mcp_server.sse_app()
    
    # Add a root / health endpoint for browser visitors and cloud health checks
    async def index_endpoint(request):
        return JSONResponse({
            "service": "Khagole MCP Server",
            "version": ke.__version__,
            "status": "online",
            "transports": {
                "sse": "/sse",
                "messages": "/messages",
            },
            "tools": [
                "calculate_chart",
                "calculate_panchanga",
                "calculate_divisional_chart",
                "calculate_vimshottari_dasha",
                "get_celebrity_chart",
            ],
            "docs": "https://github.com/PriyankGahtori/Khagole/tree/main/examples/mcp-server",
        })

    async def health_endpoint(request):
        return JSONResponse({"status": "healthy", "kernel": kernel_path})

    app.add_route("/", index_endpoint, methods=["GET"])
    app.add_route("/health", health_endpoint, methods=["GET"])
        
    return app

def main():
    parser = argparse.ArgumentParser(description="Khagole MCP Server for AI Agents")
    parser.add_argument(
        "--transport",
        choices=["sse", "stdio"],
        default="sse",
        help="MCP Transport: 'sse' for hosted HTTP/remote agents, 'stdio' for local desktop apps (Claude/Cursor)",
    )
    parser.add_argument("--host", default="0.0.0.0", help="HTTP host to bind (default: 0.0.0.0)")
    parser.add_argument("--port", type=int, default=8000, help="HTTP port to bind (default: 8000)")
    args = parser.parse_args()

    if args.transport == "stdio":
        print(f"Starting Khagole MCP Server in stdio mode (JPL kernel: {kernel_path})...", file=sys.stderr)
        mcp_server.run(transport="stdio")
    else:
        import uvicorn
        print(f"=======================================================================", file=sys.stderr)
        print(f"   KHAGOLE HOSTED MODEL CONTEXT PROTOCOL (MCP) SERVER                  ", file=sys.stderr)
        print(f"=======================================================================", file=sys.stderr)
        print(f"• SSE Endpoint for Agents: http://{args.host}:{args.port}/sse", file=sys.stderr)
        print(f"• Messages Post Endpoint:  http://{args.host}:{args.port}/messages", file=sys.stderr)
        print(f"• Health Check:            http://{args.host}:{args.port}/health", file=sys.stderr)
        print(f"• NASA JPL Kernel:         {kernel_path}", file=sys.stderr)
        print(f"=======================================================================", file=sys.stderr)
        app = create_asgi_app()
        uvicorn.run(app, host=args.host, port=args.port)

if __name__ == "__main__":
    main()
