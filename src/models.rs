use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

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

/// The top-level Motion JSON document.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotionDocument {
    /// Document name.
    pub label: Option<String>,
    /// The logical canvas used for the motion.
    pub screen_size: Option<ScreenSize>,
    /// The motion's objects indexed by their name.
    pub object: BTreeMap<String, MotionObject>,
}

impl MotionDocument {
    /// Looks up a motion by its object and motion names.
    pub fn motion(&self, object_name: &str, motion_name: &str) -> Option<&Motion> {
        self.object.get(object_name)?.motion.get(motion_name)
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

/// A named object containing one or more motions.
#[derive(Debug, Deserialize)]
pub struct MotionObject {
    #[serde(rename = "type")]
    pub object_type: ObjectType,
    pub motion: BTreeMap<String, Motion>,
}

/// A playable motion with its root layer nodes.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Motion {
    /// The type of motion.
    #[serde(rename = "type")]
    pub motion_type: MotionType,
    /// The final timeline time.
    pub last_time: f32,
    /// Root nodes in the layer tree.
    pub layer: Vec<Layer>,
}

impl Motion {
    /// Visits this motion's layers in draw-tree order, parents before children.
    pub fn layers_depth_first(&self) -> Vec<&Layer> {
        let mut layers = Vec::new();
        for layer in &self.layer {
            layer.collect_depth_first(&mut layers);
        }
        layers
    }
}

/// A layer node and its child layers.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    /// The layer type assigned by the exporter.
    #[serde(rename = "type")]
    pub layer_type: LayerType,
    /// Exported layer name.
    pub label: String,
    /// Timed changes applied to this layer.
    #[serde(rename = "frameList")]
    pub frame_list: Vec<KeyFrame>,
    /// Nested layers that inherit this layer's transform.
    #[serde(default)]
    pub children: Vec<Layer>,
}

impl Layer {
    /// Appends this layer and every descendant in depth-first order.
    fn collect_depth_first<'a>(&'a self, layers: &mut Vec<&'a Layer>) {
        layers.push(self);
        for child in &self.children {
            child.collect_depth_first(layers);
        }
    }
}

/// A keyframe for a layer.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyFrame {
    /// The time on the motion timeline.
    pub time: f32,
    /// The type of frame.
    #[serde(rename = "type")]
    pub kind: FrameType,
    /// The simple image and transform properties carried by this frame.
    pub content: Option<KeyFrameContent>,
}

/// The supported, non-deforming properties of a frame.
#[derive(Debug, Deserialize)]
pub struct KeyFrameContent {
    /// The image or nested-motion reference.
    pub src: Option<String>,
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
    /// Mask/stencil flags (dunno how to use this yet).
    pub mask: Option<u32>,
}

/// The numeric type assigned to an object in a Motion JSON document.
#[derive(Debug, PartialEq, Eq)]
pub enum ObjectType {
    /// Type `0`.
    Zero,
    /// An exporter type not yet modelled by this parser.
    Other(u8),
}

numeric_type_enum!(ObjectType { 0 => Zero });

/// The numeric type assigned to a motion in a Motion JSON document.
#[derive(Debug, PartialEq, Eq)]
pub enum MotionType {
    /// Type `0`.
    Zero,
    /// An exporter type not yet modelled by this parser.
    Other(u8),
}

numeric_type_enum!(MotionType { 0 => Zero });

/// The numeric type assigned to a layer in a Motion JSON document.
#[derive(Debug, PartialEq, Eq)]
pub enum LayerType {
    /// Type `0`.
    Zero,
    /// Type `2`.
    Two,
    /// Type `3`.
    Three,
    /// Type `12`.
    Twelve,
    /// An exporter type not yet modelled by this parser.
    Other(u8),
}

numeric_type_enum!(LayerType {
    0 => Zero,
    2 => Two,
    3 => Three,
    12 => Twelve,
});

/// The numeric type assigned to a layer frame in a Motion JSON document.
#[derive(Debug, PartialEq, Eq)]
pub enum FrameType {
    /// Type `0`.
    Zero,
    /// Type `2`.
    Two,
    /// Type `3`.
    Three,
    /// An exporter type not yet modelled by this parser.
    Other(u8),
}

numeric_type_enum!(FrameType {
    0 => Zero,
    2 => Two,
    3 => Three,
});

