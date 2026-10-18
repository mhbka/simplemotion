use crate::{IconKey, KeyFrameContent};
use glam::{Affine2, Mat2, Vec2};

/// A command to animate an icon/image with the given transform.
#[derive(Debug, Clone)]
pub struct AnimationCommand {
    pub icon: IconKey,
    pub transform: Affine2,
    pub opacity: f32,
}

/// The supported transforms.
#[derive(Clone, Copy, Debug)]
pub(super) struct TransformState {
    translation: Vec2,
    angle: f32,
    scale: Vec2,
    shear: Vec2,
    anchor_offset: Vec2,
    opacity: f32,
}

/// The transform state carried while traversing a layer tree.
#[derive(Clone, Copy, Debug)]
pub(crate) struct InterpolatedLayerState {
    /// The inherited, interpolated transform. This is (usually) then inherited by children.
    pub transform: Affine2,
    /// The offset of a layer. This is to bring the layer to the correct origin.
    /// We keep this separate from transform because there are 2 cases:
    /// - Motion: Child layers must anchor to its origin
    /// - Icon: Origin is just for itself -- child layers don't care about it
    pub anchor_offset: Vec2,
    /// The inherited, interpolated opacity.
    pub opacity: f32,
}

impl InterpolatedLayerState {
    /// Returns the neutral state used before entering a root motion.
    pub(crate) fn identity() -> Self {
        Self {
            transform: Affine2::IDENTITY,
            anchor_offset: Vec2::ZERO,
            opacity: 1.0,
        }
    }

    /// Combines a child layer's local state with this inherited state.
    pub(crate) fn with_child(self, child: Self) -> Self {
        Self {
            transform: self.transform * child.transform,
            // Structural children do not inherit their parent's drawing anchor.
            anchor_offset: child.anchor_offset,
            opacity: self.opacity * child.opacity,
        }
    }

    /// Consumes this layer's anchor when entering its referenced child motion.
    pub(crate) fn into_motion_parent_state(mut self) -> Self {
        self.transform *= Affine2::from_translation(-self.anchor_offset);
        self.anchor_offset = Vec2::ZERO;
        self
    }
}

impl TransformState {
    /// Linearly interpolates between `start` and `end` into a representative `TransformState`.
    pub fn interpolate<S>(
        start: &KeyFrameContent<S>,
        end: &KeyFrameContent<S>,
        ratio: f32,
    ) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);
        Self {
            translation: Vec2::new(
                lerp(start.coord[0], end.coord[0], ratio),
                lerp(start.coord[1], end.coord[1], ratio),
            ),
            angle: lerp_angle_degrees(start.angle, end.angle, ratio).to_radians(),
            scale: Vec2::new(lerp(start.zx, end.zx, ratio), lerp(start.zy, end.zy, ratio)),
            shear: Vec2::new(lerp(start.sx, end.sx, ratio), lerp(start.sy, end.sy, ratio)),
            anchor_offset: Vec2::new(lerp(start.ox, end.ox, ratio), lerp(start.oy, end.oy, ratio)),
            opacity: lerp(start.opa, end.opa, ratio),
        }
    }

    /// Turns it into the render properties used by the animator.
    pub fn to_interpolated_layer_state(self) -> InterpolatedLayerState {
        let scale = Mat2::from_diagonal(self.scale);
        let shear = Mat2::from_cols(Vec2::new(1.0, self.shear.y), Vec2::new(self.shear.x, 1.0));
        let matrix = Mat2::from_angle(self.angle) * shear * scale;

        let node_transform = Affine2::from_mat2_translation(matrix, self.translation);

        InterpolatedLayerState {
            transform: node_transform,
            anchor_offset: self.anchor_offset,
            opacity: self.opacity,
        }
    }
}

/// Blends two scalar values using a normalized interpolation ratio.
fn lerp(start: f32, end: f32, ratio: f32) -> f32 {
    start + (end - start) * ratio
}

/// Blends angles through the shortest turn, treating 0 and 360 degrees as equal.
fn lerp_angle_degrees(start: f32, end: f32, ratio: f32) -> f32 {
    let delta = (end - start + 180.0).rem_euclid(360.0) - 180.0;
    start + delta * ratio
}
