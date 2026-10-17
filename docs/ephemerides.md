# Ephemerides

The engine takes planetary positions from NASA JPL development ephemerides, distributed as
SPK kernels (`.bsp` files). It reads them with its own pure-Rust reader (NAIF DAF container,
segment types 2 and 3), with no CSPICE and no C code.

## Which kernel

| Kernel | Coverage | Size | Use |
|---|---|---|---|
| `de440s.bsp` | 1849–2150 | 32 MB | the default: modern charts |
| `de441_part-1.bsp` | 13200 BC–1969 | 1.4 GB | historical charts |
| `de441_part-2.bsp` | 1969–17191 | 1.6 GB | far-future charts |

Download them from <https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/>, or run
`scripts/fetch-kernels.sh` (DE440s; `--de441` also fetches DE441 part 1) in a checkout.
DE440 and DE441 share their dynamical model and agree closely over their common span, so a
chart does not change noticeably when it is computed from a different kernel.

Chiron comes from an embedded table covering 1849–2151. Outside it, Chiron is listed in
`unavailable_bodies` instead of being extrapolated.

## Loading kernels

A `KernelSet` holds kernels **in preference order**: each date is computed with the first
kernel that covers it. A kernel's coverage is the span that *all* of its segments cover,
because a chart needs every body.

```rust
use khagole_engine::ephemeris::{Kernel, KernelSet, Spk};

let mut kernels = KernelSet::new();
kernels.push(Kernel::new("de440s.bsp", Spk::open("kernels/de440s.bsp")?)?);    // preferred
kernels.push(Kernel::new("de441_part-1.bsp", Spk::open("kernels/de441_part-1.bsp")?)?);
```

The core does no platform-specific I/O apart from the convenience `Spk::open`. In the
browser or on mobile, load the bytes however the platform allows and use
`Spk::from_bytes(bytes)`:

```js
const engine = new Engine();
const bytes = new Uint8Array(await (await fetch("/ephemeris/de440s.bsp")).arrayBuffer());
engine.addKernel("de440s.bsp", bytes);
engine.coverage;                                   // [first JD, last JD]
engine.supports(julianDay("1987-05-17T14:30:00Z")) // true
```

In Python, `Engine(["de440s.bsp", "de441_part-1.bsp"])` takes paths in preference order.

## Out of range is an error

A date that no loaded kernel covers fails with `ephemeris_out_of_range`
(`EngineError::OutOfRange`, carrying the requested JD and the loaded coverage). The engine
never extrapolates or falls back to an approximate theory. You can check a date before
calculating it with `KernelSet::for_jd` or `supports(jd)`.

## Smaller kernels: excerpts

For the web and for mobile apps, 32 MB is often too much. An excerpt keeps only the records
covering a date range. Positions inside that range are **bit-for-bit identical** to the
full kernel's, because the same Chebyshev records are copied, not refitted.

| Range (de440s) | Size |
|---|---|
| 1849–2150 (full) | 32.7 MB |
| 1900–2100 | 21.8 MB |
| 1950–2050 | 10.9 MB |
| 2000–2050 | 5.4 MB |

Kernel data is dense floating point and barely compresses (gzip saves about 5%).

From a checkout (dates are TDB Julian days):

```bash
cargo run --release -p khagole-engine --example excerpt -- \
    kernels/de440s.bsp de440s-1950-2050.bsp 2433282.5 2469807.5
```

In code, `Spk::excerpt(start_jd, end_jd)` returns the bytes of a new kernel, and both
bindings expose it:

```js
import { excerptKernel } from "khagole-engine";
const small = excerptKernel(fullKernelBytes, 2433282.5, 2469807.5); // 1950-2050
```

```python
import khagole_engine as ace
Path("de440s-1950-2050.bsp").write_bytes(ace.excerpt_kernel("kernels/de440s.bsp", 2433282.5, 2469807.5))
```

The [live demo](https://priyankgahtori.github.io/Khagole/) runs on the 1950–2050
excerpt.

## Time scales

Chart moments are UTC. The engine converts them to UT1, TT and TDB with ΔT from the daily
IERS table bundled with Skyfield 1.55 (1973 onwards, about a year of predictions), and,
outside it, from the Morrison–Stephenson–Hohenkerk splines joined to the long-term
parabola. For future dates beyond the predictions ΔT is extrapolated. An error in ΔT
shifts the bodies along their orbits, the Moon most (about 0.55″ per second of ΔT). This
is inherent to any calculation of future charts.
