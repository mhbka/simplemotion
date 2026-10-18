use serde::Deserialize;
use std::collections::BTreeMap;
use crate::models::objects::{Icon, Motion, ObjectType, ScreenSize};

/// The unmodified document shape read directly from Motion JSON.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawMotionDocument {
    /// Document name.
    pub label: Option<String>,
    /// The logical canvas used for the motion.
    pub screen_size: ScreenSize,
    /// The motion's objects indexed by their name.
    pub object: BTreeMap<String, RawMotionObject>,
    /// Image-resource groups indexed by their exported source name.
    #[serde(default)]
    pub source: BTreeMap<String, RawSource>,
}

/// A named object containing one or more raw motions.
#[derive(Debug, Deserialize)]
pub struct RawMotionObject {
    #[serde(rename = "type")]
    pub object_type: ObjectType,
    pub motion: BTreeMap<String, RawMotion>,
}

/// A raw motion whose frame sources are still label strings.
pub type RawMotion = Motion<String>;

/// One raw source group read from Motion JSON.
#[derive(Debug, Default, Deserialize)]
pub struct RawSource {
    /// Image resources indexed by their exported name.
    #[serde(default)]
    pub icon: BTreeMap<String, Icon>,
}
