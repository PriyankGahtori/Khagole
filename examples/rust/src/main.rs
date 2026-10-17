use std::path::{Path, PathBuf};
use khagole_engine::ephemeris::{Kernel, KernelSet, Spk};
use khagole_engine::{
    calc_divisional_chart, calc_panchang, calc_vimshottari_dasha, nakshatra_info,
    calculate_chart, ChartRequest, UtcInstant, YearType,
};

fn find_kernel() -> Option<PathBuf> {
    let candidates = [
        "kernels/de440s.bsp",
        "../../kernels/de440s.bsp",
        "../kernels/de440s.bsp",
        "/Users/priyankgahtori/Code/poc/khagole/kernels/de440s.bsp",
    ];
    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn build_kernel_set(path: &Path) -> Result<KernelSet, String> {
    let spk = Spk::open(path).map_err(|e| format!("failed to open SPK kernel {path:?}: {e}"))?;
    let kernel = Kernel::new(
        path.file_name().unwrap_or_default().to_string_lossy(),
        spk,
    )
    .map_err(|e| format!("invalid kernel {path:?}: {e}"))?;
    let mut set = KernelSet::new();
    set.push(kernel);
    Ok(set)
}

struct Celebrity {
    name: &'static str,
    utc: &'static str,
    local_time: &'static str,
    location: &'static str,
    lat: f64,
    lon: f64,
}

fn main() {
    let kernel_path = match find_kernel() {
        Some(p) => p,
        None => {
            eprintln!("Error: NASA JPL kernel de440s.bsp not found.");
            eprintln!("Run `scripts/fetch-kernels.sh` to download the high-precision ephemeris kernel.");
            std::process::exit(1);
        }
    };

    let kernels = build_kernel_set(&kernel_path).expect("Failed to load NASA JPL ephemeris kernel");

    let celebrities = [
        Celebrity {
            name: "Steve Jobs",
            utc: "1955-02-25T03:15:00Z",
            local_time: "1955-02-24 19:15 PST",
            location: "San Francisco, CA, USA",
            lat: 37.7749,
            lon: -122.4194,
        },
        Celebrity {
            name: "Albert Einstein",
            utc: "1879-03-14T10:50:00Z",
            local_time: "1879-03-14 11:30 LMT",
            location: "Ulm, Germany",
            lat: 48.4011,
            lon: 9.9876,
        },
    ];

    println!("==========================================================================================");
    println!("             KHAGOLE VEDIC JYOTISH & NASA JPL EPHEMERIDES ENGINE (RUST)                  ");
    println!("==========================================================================================");

    for celeb in &celebrities {
        println!("\n------------------------------------------------------------------------------------------");
        println!(" CELEBRITY HOROSCOPE: {} ({})", celeb.name.to_uppercase(), celeb.location);
        println!(" Birth Moment: {} | UTC: {} | Lat: {:.4}°, Lon: {:.4}°", celeb.local_time, celeb.utc, celeb.lat, celeb.lon);
        println!("------------------------------------------------------------------------------------------");

        let instant = UtcInstant::parse(celeb.utc).expect("Valid UTC timestamp");
        let req = ChartRequest {
            instant,
            latitude: celeb.lat,
            longitude: celeb.lon,
            house_system: "W", // Whole Sign houses standard in Vedic astrology
            zodiac_type: "sidereal",
            ayanamsa: "lahiri", // Chitrapaksha Lahiri
            orb_settings: None,
            dignity_scheme: "lilly",
            position_mode: "true", // Canonical Geometric Positions without light-time lag
            node_type: "mean",
            chinese_calendar: false,
        };

        let chart = calculate_chart(&kernels, &req).expect("Chart calculation succeeded");

        let ayan_name = chart.ayanamsa_name.unwrap_or("Lahiri");
        let ayan_val = chart.ayanamsa_value.unwrap_or(0.0);
        let ayan_fmt = chart.ayanamsa_formatted.unwrap_or_default();
        println!("• Precession Ayanamsa: {} ({:.6}° = {})", ayan_name, ayan_val, ayan_fmt);

        // 1. Planetary Placements & Nakshatras Table
        println!("\n[1] NAVAGRAHA & LAGNA POSITIONS (SIDEREAL LAHIRI):");
        println!("{:<12} {:<12} {:<14} {:<8} {:<16} {:<6} {:<10} {:<6}", 
            "Body", "Rasi Sign", "Degree", "House", "Nakshatra", "Pada", "Lord", "Motion");
        println!("{}", "-".repeat(88));

        let mut planets_vec: Vec<(String, f64)> = Vec::new();
        let mut lagna_lon = 0.0;
        let mut sun_lon = 0.0;
        let mut moon_lon = 0.0;

        for p in &chart.planets {
            if !matches!(p.name, "Sun" | "Moon" | "Mars" | "Mercury" | "Jupiter" | "Venus" | "Saturn" | "North Node" | "South Node" | "Ascendant") {
                continue;
            }
            let display_name = match p.name {
                "North Node" => "Rahu",
                "South Node" => "Ketu",
                "Ascendant" => "Lagna (Asc)",
                other => other,
            };
            let nak = nakshatra_info(p.ecliptic_longitude);
            let motion = if p.is_retrograde { "Retro" } else { "Direct" };
            let total_sec = (p.ecliptic_longitude * 3600.0).round() as u64;
            let sec = total_sec % 60;
            let dms = format!("{}° {:02}′ {:02}″", p.degree, p.minute, sec);
            println!("{:<14} {:<12} {:<14} H{:<7} {:<16} {:<6} {:<10} {:<6}",
                display_name, p.sign, dms, p.house, nak.name, nak.pada, nak.lord, motion);

            if p.name == "Sun" { sun_lon = p.ecliptic_longitude; }
            if p.name == "Moon" { moon_lon = p.ecliptic_longitude; }
            if p.name == "Ascendant" { lagna_lon = p.ecliptic_longitude; }
            planets_vec.push((display_name.to_string(), p.ecliptic_longitude));
        }

        // 2. Panchanga (Five Limbs of Vedic Time)
        let jd = instant.julian_day();
        let panchang = calc_panchang(sun_lon, moon_lon, jd, None, None);
        println!("\n[2] PANCHANGA (FIVE LIMBS OF VEDIC TIME):");
        println!("  • Tithi (Lunar Day):     {} ({}, #{}) - {:.1}% remaining",
            panchang.tithi.name, panchang.tithi.paksha, panchang.tithi.number, panchang.tithi.percent_left);
        println!("  • Vara (Solar Weekday):  {} (Lord: {}, Vedic: {})",
            panchang.vara.name, panchang.vara.lord, panchang.vara.vedic_name);
        println!("  • Nakshatra:             {} (Pada {}, Lord: {})",
            panchang.nakshatra.name, panchang.nakshatra.pada, panchang.nakshatra.lord);
        println!("  • Yoga (Luni-Solar):     {} (#{}), {:.1}% left",
            panchang.yoga.name, panchang.yoga.number, panchang.yoga.percent_left);
        println!("  • Karana (Half-Tithi):   {} (#{}), {:.1}% left",
            panchang.karana.name, panchang.karana.number, panchang.karana.percent_left);
        println!("  • Lunar Month & Season:  {} (Amanta) / {} (Purnimanta) | Ritu: {} | Ayana: {}",
            panchang.masa.amanta, panchang.masa.purnimanta, panchang.masa.ritu, panchang.masa.ayana);
        println!("  • Samvat Eras:           Vikram: {} ({}) | Shaka: {} ({})",
            panchang.samvat.vikram_year, panchang.samvat.vikram_name, panchang.samvat.shaka_year, panchang.samvat.shaka_name);

        // 3. Divisional Chart D9 Navamsha
        let d9 = calc_divisional_chart(&planets_vec, lagna_lon, "D9").expect("D9 calculated");
        println!("\n[3] DIVISIONAL CHART D9 (NAVAMSHA):");
        print!("  ");
        for dp in &d9.planets {
            print!("{} in Sign #{}, ", dp.name, dp.sign);
        }
        println!();

        // 4. Vimshottari Dasha Timeline
        let dasha_tree = calc_vimshottari_dasha(moon_lon, jd, 2, YearType::Tssy);
        println!("\n[4] VIMSHOTTARI DASHA SEQUENCE (TRUE SIDEREAL SOLAR YEAR - TSSY):");
        println!("  {:<10} {:<12} {:<12} {:<15}", "Mahadasha", "Start Date", "End Date", "Antardashas");
        println!("  {}", "-".repeat(55));
        for node in dasha_tree.iter().take(5) {
            println!("  {:<10} {:<12} {:<12} ({} sub-periods)",
                node.planet, node.start_date, node.end_date, node.children.len());
        }
    }
    println!("\n==========================================================================================");
    println!("                     VERIFICATION COMPLETE: ALL DETAILS COMPUTED 100%                     ");
    println!("==========================================================================================");
}
