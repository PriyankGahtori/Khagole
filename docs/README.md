# astroceleste-engine documentation

- **[API guide](api.md)**: requests, options (house systems, zodiacs, ayanamsas, orbs),
  the result shape and error codes, for Rust, Python and JavaScript.
- **[Dignities](dignities.md)**: the essential dignities, sect, receptions and antiscia a
  chart carries, with the tables' sources and the two schemes (Lilly, Dorothean).
- **[Chinese astrology](chinese.md)**: the Chinese calendar of a moment (solar terms,
  lunar months, leap months), and the Four Pillars (Ba Zi) and Zi Wei Dou Shu cast from it.
- **[Ephemerides](ephemerides.md)**: the JPL kernels the engine reads, date coverage,
  loading kernels from files or memory, and cutting smaller excerpts for the web and mobile.
- **[Accuracy](accuracy.md)**: how every number is verified against the reference
  implementation, jplephem and Skyfield.
- **[Architecture](architecture.md)**: the crates, the calculation pipeline and where
  each piece lives.

Elsewhere:

- Rust API reference: [docs.rs/astroceleste-engine](https://docs.rs/astroceleste-engine)
- Python bindings: [crates/astroceleste-engine-py](../crates/astroceleste-engine-py)
- WebAssembly bindings: [crates/astroceleste-engine-wasm](../crates/astroceleste-engine-wasm)
- Website and live demo: [priyankgahtori.github.io/astroceleste-engine](https://priyankgahtori.github.io/astroceleste-engine/)
- Contributing: [CONTRIBUTING.md](../CONTRIBUTING.md) · Security: [SECURITY.md](../SECURITY.md)
  · Releasing: [RELEASING.md](../RELEASING.md)
