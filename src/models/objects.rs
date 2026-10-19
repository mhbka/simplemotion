use std::sync::Arc;

use crate::{
    animation::{InterpolatedLayerState, TransformState},
    numeric_type_enum,
};
use glam::Affine2;
use serde::{Deserialize, Deserializer};

use super::utils::{
    deserialize_loop_time, deserialize_optional_timeline_frame, deserialize_timeline_frame,
};

/// A playable motion with its root layer nodes.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "S: Deserialize<'de>"))]
pub struct Motion<S> {
    /// The type of motion.
    #[serde(rename = "type")]
    pub motion_type: MotionType,
    /// The final timeline time.
    #[serde(deserialize_with = "deserialize_timeline_frame")]
    pub last_time: u32,
    /// The loop period in timeline frames; negative values disable looping.
    #[serde(deserialize_with = "deserialize_loop_time")]
    pub loop_time: i32,
    /// Root nodes in the layer tree.
    pub layer: Vec<Layer<S>>,
    /// Timed front-to-back ordering of the flattened layer nodes.
    #[serde(default)]
    pub priority: Vec<PriorityFrame>,
}

impl<S> Motion<S> {}

/// Denotes how to re-order the (depth-first) layers of a motion during drawing.
/// 
/// ### Example
/// - By depth-first, we have the layers A, B, C, D
/// - The active priority is [3,1,2,0] -> we evaluate in reverse
/// - The final order to draw commands in is A, C, B, D
/// 
/// ### Why call it a frame?
/// Just cuz it's timed. There's no interpolation or whatever.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriorityFrame {
    /// The time on the motion timeline.
    #[serde(deserialize_with = "deserialize_timeline_frame")]
    pub time: u32,
    /// The exporter-defined frame type.
    #[serde(rename = "type")]
    pub kind: u8,
    /// Flattened layer-node indices, ordered from front to back.
    pub content: Option<Vec<usize>>,
}

/// A layer node and its child layers.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "S: Deserialize<'de>"))]
pub struct Layer<S> {
    /// The layer type assigned by the exporter.
    #[serde(rename = "type")]
    pub layer_type: LayerType,
    /// Exported layer name.
    pub label: String,
    /// Timed changes applied to this layer.
    #[serde(rename = "frameList")]
    pub frame_list: Vec<KeyFrame<S>>,
    /// Nested layers that inherit this layer's transform.
    #[serde(default)]
    pub children: Vec<Layer<S>>,
}

impl<S> Layer<S> {}

/// A keyframe for a layer.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "S: Deserialize<'de>"))]
pub struct KeyFrame<S> {
    /// The time on the motion timeline.
    #[serde(deserialize_with = "deserialize_timeline_frame")]
    pub time: u32,
    /// The type of frame.
    #[serde(rename = "type")]
    pub kind: FrameType,
    /// The simple image and transform properties carried by this frame.
    pub content: Option<KeyFrameContent<S>>,
}

/// The properties of a keyframe.
#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "S: Deserialize<'de>"))]
pub struct KeyFrameContent<S> {
    /// The image or nested-motion reference.
    pub src: S,
    /// Horizontal source offset.
    #[serde(default)]
    pub ox: f32,
    /// Vertical source offset.
    #[serde(default)]
    pub oy: f32,
    /// Translation in x, y, z order.
    #[serde(default)]
    pub coord: [f32; 3],
    /// Settings used when this frame references a child motion.
    pub motion: Option<MotionFrame>,
    /// Rotation angle.
    #[serde(default)]
    pub angle: f32,
    /// Horizontal scale.
    #[serde(default = "default_scale")]
    pub zx: f32,
    /// Vertical scale.
    #[serde(default = "default_scale")]
    pub zy: f32,
    /// Horizontal shear.
    #[serde(default)]
    pub sx: f32,
    /// Vertical shear.
    #[serde(default)]
    pub sy: f32,
    /// Opacity.
    #[serde(default = "default_opacity")]
    pub opa: f32,
    /// Mask/stencil flags.
    pub mask: Option<u32>,
}

/// Supplies the neutral scale for omitted transform fields.
fn default_scale() -> f32 {
    1.0
}

/// Supplies the fully opaque value for omitted opacity fields.
fn default_opacity() -> f32 {
    1.0
}

/// The playback settings of a nested motion frame.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotionFrame {
    /// Child motion timeline phase at this parent frame.
    #[serde(default, deserialize_with = "deserialize_optional_timeline_frame")]
    pub time_offset: Option<u32>,
}

impl<S> KeyFrameContent<S> {
    /// Linearly interpolates transform properties and composes them into an affine matrix.
    pub fn linearly_interpolate_to_affine2(&self, other: &Self, ratio: f32) -> Affine2 {
        self.linearly_interpolate(other, ratio).transform
    }

    /// Linearly interpolates every render property used by the simplified animator.
    pub(crate) fn linearly_interpolate(&self, other: &Self, ratio: f32) -> InterpolatedLayerState {
        TransformState::interpolate(self, other, ratio).to_interpolated_layer_state()
    }
}

/// One named image resource.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon {
    /// The normalized source rectangle to draw from the raw image.
    #[serde(default)]
    pub clip: IconClip,
    /// The horizontal icon origin, defaulting to zero.
    #[serde(default)]
    pub origin_x: f32,
    /// The vertical icon origin, defaulting to zero.
    #[serde(default)]
    pub origin_y: f32,
    /// The clipped icon width, defaulting to zero.
    #[serde(default)]
    pub width: f32,
    /// The clipped icon height, defaulting to zero.
    #[serde(default)]
    pub height: f32,
}

/// A normalized rectangular crop within an icon image.
#[derive(Debug, Deserialize, PartialEq)]
pub struct IconClip {
    /// The left edge, normalized to the raw image width.
    pub left: f32,
    /// The top edge, normalized to the raw image height.
    pub top: f32,
    /// The right edge, normalized to the raw image width.
    pub right: f32,
    /// The bottom edge, normalized to the raw image height.
    pub bottom: f32,
}

impl Default for IconClip {
    /// Uses the whole source image when no crop is supplied.
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
        }
    }
}

/// Contains info/data of the icon's image.
#[derive(Debug)]
pub struct IconImage {
    kind: IconImageKind,
    data: Arc<[u8]>,
}

/// The kind of image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconImageKind {
    Png,
}

impl IconImageKind {
    /// All the supported image kinds.
    pub const ALL: &'static [Self] = &[Self::Png];

    /// Gets the filename extension used for this image kind.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
        }
    }
}

impl IconImage {
    /// Creates an image from its encoded bytes and known kind.
    pub(crate) fn new(kind: IconImageKind, data: Arc<[u8]>) -> Self {
        Self { kind, data }
    }

    /// Gets the encoded image format.
    pub fn kind(&self) -> &IconImageKind {
        &self.kind
    }

    /// Gets the encoded image bytes.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// The dimensions and origin of the motion's canvas.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScreenSize {
    pub width: f32,
    pub height: f32,
    pub origin_x: f32,
    pub origin_y: f32,
}

/// The type assigned to an object in a Motion JSON document.
///
/// TODO: meanings still not known.
#[derive(Debug, PartialEq, Eq)]
pub enum ObjectType {
    /// `0` in the document.
    Zero,
    /// A type not yet supported/known.
    Other(u8),
}

numeric_type_enum!(ObjectType { 0 => Zero });

/// The type assigned to a motion in a Motion JSON document.
///
/// TODO: meanings still not known.
#[derive(Debug, PartialEq, Eq)]
pub enum MotionType {
    /// `0` in the document.
    Zero,
    /// A type not yet supported/known.
    Other(u8),
}

numeric_type_enum!(MotionType { 0 => Zero });

/// The numeric type assigned to a layer in a Motion JSON document.
///
/// TODO: any new variants.
#[derive(Debug, PartialEq, Eq)]
pub enum LayerType {
    /// Draws icon resources from its frames.
    Icon,
    /// Represents the built-in layout source.
    Layout,
    /// Evaluates nested motions from its frames.
    Motion,
    /// Defines a mask using icon resources from its frames.
    Mask,
    /// A type not yet supported/known.
    Other(u8),
}

numeric_type_enum!(LayerType {
    0 => Icon,
    2 => Layout,
    3 => Motion,
    12 => Mask,
});

/// The numeric type assigned to a layer frame in a Motion JSON document.
#[derive(Debug, PartialEq, Eq)]
pub enum FrameType {
    /// Has no content and clears the layer's current resource.
    Empty,
    /// Shows a resource in one fixed spot in this layer.
    ///
    /// This layer sets the child motion's parent-side state at this frame and keeps it there
    /// until another frame changes it. The child motion and its descendants still animate.
    ParentTransformsStatically,
    /// Moves/interpolates the whole child motion in this layer over time.
    ///
    /// This layer animates the child motion's parent-side state, such as its placement or angle.
    /// The child motion and its descendants also animate.
    ParentTransformsDynamically,
    /// A type not yet supported/known.
    Other(u8),
}

numeric_type_enum!(FrameType {
    0 => Empty,
    2 => ParentTransformsStatically,
    3 => ParentTransformsDynamically,
});
