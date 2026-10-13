pub use crate::models::*;

/// Parses a Motion JSON document into the simple animation model.
pub fn parse_motion_json(json: &str) -> Result<MotionDocument, serde_json::Error> {
    serde_json::from_str(json)
}