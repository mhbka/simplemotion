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
}

impl TransformState {
    /// Linearly interpolates between `start` and `end` into a representative `TransformState`.
    pub fn interpolate<S>(
        start: &KeyFrameContent<S>,
        end: &KeyFrameContent<S>,
        ratio: f32,
    ) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);
        let start_coord = start.coord.map(|coord| Vec2::new(coord[0], coord[1]));
        let end_coord = end.coord.map(|coord| Vec2::new(coord[0], coord[1]));
        let offset = Vec2::new(
            lerp_optional(start.ox, end.ox, 0.0, ratio),
            lerp_optional(start.oy, end.oy, 0.0, ratio),
        );
        Self {
            translation: lerp_optional_vec2(start_coord, end_coord, Vec2::ZERO, ratio) + offset,
            angle: lerp_optional(start.angle, end.angle, 0.0, ratio),
            scale: Vec2::new(
                lerp_optional(start.zx, end.zx, 1.0, ratio),
                lerp_optional(start.zy, end.zy, 1.0, ratio),
            ),
            shear: Vec2::new(
                lerp_optional(start.sx, end.sx, 0.0, ratio),
                lerp_optional(start.sy, end.sy, 0.0, ratio),
            ),
        }
    }

    /// Turns it into an `Affine2`.
    pub fn to_affine2(self) -> Affine2 {
        let scale = Mat2::from_diagonal(self.scale);
        let shear = Mat2::from_cols(Vec2::new(1.0, self.shear.y), Vec2::new(self.shear.x, 1.0));
        let matrix = Mat2::from_angle(self.angle) * shear * scale;

        Affine2::from_mat2_translation(matrix, self.translation)
    }
}

/// Blends two scalar values using a normalized interpolation ratio.
fn lerp(start: f32, end: f32, ratio: f32) -> f32 {
    start + (end - start) * ratio
}

/// Blends optional scalars. Returns only the known value if either is `None`,
/// and returns `default` if both are `None`.
fn lerp_optional(start: Option<f32>, end: Option<f32>, default: f32, ratio: f32) -> f32 {
    match (start, end) {
        (Some(start), Some(end)) => lerp(start, end, ratio),
        (Some(value), None) | (None, Some(value)) => value,
        (None, None) => default,
    }
}

/// Blends optional vectors. Returns only the known value if either is `None`,
/// and returns `default` if both are `None`.
fn lerp_optional_vec2(start: Option<Vec2>, end: Option<Vec2>, default: Vec2, ratio: f32) -> Vec2 {
    match (start, end) {
        (Some(start), Some(end)) => start.lerp(end, ratio),
        (Some(value), None) | (None, Some(value)) => value,
        (None, None) => default,
    }
}
