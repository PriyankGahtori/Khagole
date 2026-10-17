//! Ephemeris sources. Everything above this layer (houses, aspects, lots, stars, …) is
//! independent of where planetary positions come from.
//!
//! - [`Spk`] reads a JPL SPK kernel (segment types 2 and 3) from a file ([`Spk::open`]) or
//!   from memory ([`Spk::from_bytes`]), and can write an excerpt of it ([`Spk::excerpt`]).
//! - [`Kernel`] names a loaded kernel and computes the span over which it has every body
//!   (a body may be split into segments covering consecutive time ranges).
//! - [`KernelSet`] holds kernels in preference order: the first one covering a date is
//!   used, and a date that none covers is an error.
//!
//! Kernels: <https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/>. `de440s.bsp` (1849–2150,
//! 32 MB) is the one the engine is validated against.

mod kernels;
#[doc(hidden)] // exposed for the reduction tests; not part of the API
pub mod observe;
mod spk;

pub use kernels::{Kernel, KernelSet};
pub use spk::{jd_tdb_to_et, Segment, Spk, SpkError, State};
