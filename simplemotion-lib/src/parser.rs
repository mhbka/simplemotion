use crate::models::RawMotionDocument;
pub use crate::models::*;
use std::error::Error;
use std::fmt;

/// An error returned while parsing or resolving a motion document.
#[derive(Debug)]
pub enum ParseMotionError {
    /// The input was not valid Motion JSON.
    Json(serde_json::Error),
    /// The parsed document did not match the supported resource structure.
    Resolution(ResolutionError),
}

impl fmt::Display for ParseMotionError {
    /// Describes the parsing or resolution failure.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => error.fmt(formatter),
            Self::Resolution(error) => error.fmt(formatter),
        }
    }
}

impl Error for ParseMotionError {}

/// Parses a Motion JSON document into the simple animation model.
pub(crate) fn parse_motion_json(json: &str) -> Result<RawMotionDocument, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parses and resolves Motion JSON into a typed resource graph.
pub fn parse_motion_resources(json: &str) -> Result<MotionResources, ParseMotionError> {
    let raw = parse_motion_json(json).map_err(ParseMotionError::Json)?;

    MotionResources::from_raw(raw).map_err(ParseMotionError::Resolution)
}
