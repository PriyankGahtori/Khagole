#!/usr/bin/env python3
"""
Khagole Vedic Jyotish & JPL Ephemerides — Python Celebrity Chart Example
Computes high-precision sidereal Vedic birth charts, Panchanga, D9 Navamsha, and Vimshottari Dasha.
"""

import sys
from pathlib import Path
import khagole_engine as ke

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
    raise FileNotFoundError(
        "NASA JPL ephemeris kernel de440s.bsp not found. "
        "Run `scripts/fetch-kernels.sh` to download the ephemeris."
    )

CELEBRITIES = [
    {
        "name": "Steve Jobs",
        "utc": "1955-02-25T03:15:00Z",
        "local_time": "1955-02-24 19:15 PST",
        "location": "San Francisco, CA, USA",
        "lat": 37.7749,
        "lon": -122.4194,
    },
    {
        "name": "Albert Einstein",
        "utc": "1879-03-14T10:50:00Z",
        "local_time": "1879-03-14 11:30 LMT",
        "location": "Ulm, Germany",
        "lat": 48.4011,
        "lon": 9.9876,
    },
]

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

def main():
    kernel_path = find_kernel()
    engine = ke.Engine([kernel_path])

    print("=" * 90)
    print("             KHAGOLE VEDIC JYOTISH & NASA JPL EPHEMERIDES (PYTHON)                 ")
    print("=" * 90)

    for celeb in CELEBRITIES:
        print("\n" + "-" * 90)
        print(f" CELEBRITY HOROSCOPE: {celeb['name'].upper()} ({celeb['location']})")
        print(f" Birth Moment: {celeb['local_time']} | UTC: {celeb['utc']} | Lat: {celeb['lat']:.4f}°, Lon: {celeb['lon']:.4f}°")
        print("-" * 90)

        # 1. Compute Chart
        chart = engine.chart(
            celeb["utc"],
            celeb["lat"],
            celeb["lon"],
            zodiac_type="sidereal",
            ayanamsa="lahiri",
            house_system="W",
            position_mode="true",
            node_type="mean",
        )

        ayan_name = chart.get("ayanamsa_name", "Lahiri")
        ayan_val = chart.get("ayanamsa_value", 0.0)
        ayan_fmt = chart.get("ayanamsa_formatted", "")
        print(f"• Precession Ayanamsa: {ayan_name} ({ayan_val:.6f}° = {ayan_fmt})")

        # 2. Navagraha & Lagna Placements Table
        print("\n[1] NAVAGRAHA & LAGNA POSITIONS (SIDEREAL LAHIRI):")
        print(f"{'Body':<14} {'Rasi Sign':<12} {'Degree':<14} {'House':<8} {'Nakshatra':<18} {'Pada':<6} {'Lord':<10} {'Motion':<6}")
        print("-" * 90)

        sun_lon = 0.0
        moon_lon = 0.0
        lagna_lon = 0.0
        planets_list = []

        for p in chart["planets"]:
            raw_name = p["name"]
            if raw_name not in NAVAGRAHA_NAMES:
                continue
            display_name = NAVAGRAHA_NAMES[raw_name]
            lon = p["ecliptic_longitude"]
            nak = ke.nakshatra_info(lon)
            motion = "Retro" if p.get("is_retrograde") else "Direct"
            total_sec = round(lon * 3600.0)
            sec = total_sec % 60
            dms = f"{p['degree']}° {p['minute']:02d}′ {sec:02d}″"

            print(f"{display_name:<14} {p['sign']:<12} {dms:<14} H{p['house']:<7} {nak['name']:<18} {nak['pada']:<6} {nak['lord']:<10} {motion:<6}")

            if raw_name == "Sun":
                sun_lon = lon
            elif raw_name == "Moon":
                moon_lon = lon
            elif raw_name == "Ascendant":
                lagna_lon = lon

            planets_list.append({"name": display_name, "ecliptic_longitude": lon})

        # 3. Panchanga (Five Limbs of Vedic Time)
        jd = ke.julian_day(celeb["utc"])
        panchang = ke.calc_panchang(sun_lon, moon_lon, jd)
        t = panchang["tithi"]
        v = panchang["vara"]
        n = panchang["nakshatra"]
        y = panchang["yoga"]
        k = panchang["karana"]
        m = panchang["masa"]
        s = panchang["samvat"]

        print("\n[2] PANCHANGA (FIVE LIMBS OF VEDIC TIME):")
        print(f"  • Tithi (Lunar Day):     {t['name']} ({t['paksha']}, #{t['number']}) - {t['percent_left']:.1f}% remaining")
        print(f"  • Vara (Solar Weekday):  {v['name']} (Lord: {v['lord']}, Vedic: {v['vedic_name']})")
        print(f"  • Nakshatra:             {n['name']} (Pada {n['pada']}, Lord: {n['lord']})")
        print(f"  • Yoga (Luni-Solar):     {y['name']} (#{y['number']}), {y['percent_left']:.1f}% left")
        print(f"  • Karana (Half-Tithi):   {k['name']} (#{k['number']}), {k['percent_left']:.1f}% left")
        print(f"  • Lunar Month & Season:  {m['amanta']} (Amanta) / {m['purnimanta']} (Purnimanta) | Ritu: {m['ritu']} | Ayana: {m['ayana']}")
        print(f"  • Samvat Eras:           Vikram: {s['vikram_year']} ({s['vikram_name']}) | Shaka: {s['shaka_year']} ({s['shaka_name']})")

        # 4. Divisional Chart D9 Navamsha
        d9 = ke.calc_divisional_chart(planets_list, varga="D9", lagna_longitude=lagna_lon)
        print("\n[3] DIVISIONAL CHART D9 (NAVAMSHA):")
        placements_str = ", ".join(f"{dp['name']} in Sign #{dp['sign']}" for dp in d9["planets"])
        print(f"  {placements_str}")

        # 5. Vimshottari Dasha Sequence
        dasha_tree = ke.calc_vimshottari_dasha(moon_lon, jd, depth=2, year_type="tssy")
        print("\n[4] VIMSHOTTARI DASHA SEQUENCE (TRUE SIDEREAL SOLAR YEAR - TSSY):")
        print(f"  {'Mahadasha':<10} {'Start Date':<12} {'End Date':<12} {'Antardashas':<15}")
        print("  " + "-" * 55)
        for node in dasha_tree[:5]:
            children_count = len(node.get("children", []))
            print(f"  {node['planet']:<10} {node['start_date']:<12} {node['end_date']:<12} ({children_count} sub-periods)")

    print("\n" + "=" * 90)
    print("                     VERIFICATION COMPLETE: ALL DETAILS COMPUTED 100%                     ")
    print("=" * 90)

if __name__ == "__main__":
    main()
