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
        Self {
            translation: Vec2::new(
                lerp(start.coord[0], end.coord[0], ratio),
                lerp(start.coord[1], end.coord[1], ratio),
            ),
            angle: lerp_angle_degrees(start.angle, end.angle, ratio).to_radians(),
            scale: Vec2::new(lerp(start.zx, end.zx, ratio), lerp(start.zy, end.zy, ratio)),
            shear: Vec2::new(lerp(start.sx, end.sx, ratio), lerp(start.sy, end.sy, ratio)),
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

/// Blends angles through the shortest turn, treating 0 and 360 degrees as equal.
fn lerp_angle_degrees(start: f32, end: f32, ratio: f32) -> f32 {
    let delta = (end - start + 180.0).rem_euclid(360.0) - 180.0;
    start + delta * ratio
}
