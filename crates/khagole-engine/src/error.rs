use std::fmt;

use crate::ephemeris::SpkError;

/// Why a calculation failed. [`EngineError::code`] gives the API error code.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum EngineError {
    /// No loaded kernel covers the requested instant (`ephemeris_out_of_range`).
    OutOfRange {
        /// Requested Julian date.
        jd: f64,
        /// Combined span of the loaded kernels (Julian dates), `None` when none is loaded.
        coverage: Option<(f64, f64)>,
    },
    /// A kernel could not be read or evaluated (`ephemeris_error`).
    Ephemeris(SpkError),
    /// Malformed caller input (e.g. a non-numeric orb).
    InvalidInput(String),
}

impl EngineError {
    /// Stable machine-readable code, shared with the HTTP API.
    pub fn code(&self) -> &'static str {
        match self {
            EngineError::OutOfRange { .. } => "ephemeris_out_of_range",
            EngineError::Ephemeris(_) => "ephemeris_error",
            EngineError::InvalidInput(_) => "invalid_input",
        }
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::OutOfRange {
                jd,
                coverage: Some((start, end)),
            } => write!(
                f,
                "JD {jd:.1} is outside the loaded ephemeris coverage (JD {start:.1} to {end:.1})"
            ),
            EngineError::OutOfRange { jd, coverage: None } => {
                write!(f, "JD {jd:.1} cannot be computed: no ephemeris loaded")
            }
            EngineError::Ephemeris(err) => write!(f, "{err}"),
            EngineError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<SpkError> for EngineError {
    fn from(err: SpkError) -> Self {
        EngineError::Ephemeris(err)
    }
}
