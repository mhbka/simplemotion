use serde::{Deserialize, Deserializer};

/// Implement Deserialize for u8 -> enum variants.
macro_rules! numeric_type_enum {
    ($name:ident { $($number:literal => $variant:ident),+ $(,)? }) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = u8::deserialize(deserializer)?;

                Ok(match value {
                    $($number => Self::$variant,)+
                    other => Self::Other(other),
                })
            }
        }
    };
}

/// A playable motion with its root layer nodes.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "S: Deserialize<'de>"))]
pub struct Motion<S> {
    /// The type of motion.
    #[serde(rename = "type")]
    pub motion_type: MotionType,
    /// The final timeline time.
    pub last_time: f32,
    /// Root nodes in the layer tree.
    pub layer: Vec<Layer<S>>,
}

impl<S> Motion<S> {}

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
    pub time: f32,
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
    pub ox: Option<f32>,
    /// Vertical source offset.
    pub oy: Option<f32>,
    /// Translation in x, y, z order.
    pub coord: Option<[f32; 3]>,
    /// Rotation angle.
    pub angle: Option<f32>,
    /// Horizontal scale.
    pub zx: Option<f32>,
    /// Vertical scale.
    pub zy: Option<f32>,
    /// Horizontal shear.
    pub sx: Option<f32>,
    /// Vertical shear.
    pub sy: Option<f32>,
    /// Opacity.
    pub opa: Option<f32>,
    /// Mask/stencil flags.
    pub mask: Option<u32>,
}

/// One named image resource.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon {
    pub origin_x: Option<f32>,
    pub origin_y: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
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
