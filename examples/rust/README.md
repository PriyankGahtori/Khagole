# Khagole Rust Example — Celebrity Vedic Horoscopes

Calculate full Vedic Jyotish horoscopes directly in Rust with NASA JPL DE440s ephemerides.

## Features Demonstrated
- High-precision Chitrapaksha Lahiri Ayanamsa calculation
- Navagrahas (Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu) + Lagna (Ascendant)
- Sign, Degree/Minute/Second, House placement (Whole Sign system)
- Nakshatra, Pada (1–4), and Nakshatra Lord for all bodies
- Complete Vedic Panchanga (Tithi, Vara, Nakshatra, Yoga, Karana, Masa, Ritu, Ayana, Samvat)
- D9 Navamsha Divisional Chart calculation
- 120-year Vimshottari Dasha timeline tree with True Sidereal Solar Year (TSSY)

## Quick Start

```bash
cargo run --manifest-path examples/rust/Cargo.toml
```

## Celebrity Charts Included
1. **Steve Jobs** (Born 1955-02-24 19:15 PST in San Francisco, CA)
   - Leo Lagna, Aquarius Sun (Shatabhisha), Pisces Moon (Uttara Bhadrapada), Exalted Mars, Saturn in Libra.
2. **Albert Einstein** (Born 1879-03-14 11:30 LMT in Ulm, Germany)
   - Gemini Lagna, Pisces Sun (Purva Bhadrapada), Scorpio Moon (Jyeshtha), Exalted Venus in Pisces, 10th house stellium.
