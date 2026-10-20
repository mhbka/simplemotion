#[path = "../common/mod.rs"]
mod common;

use common::{
    animation::load_animation,
    render::{draw_commands, draw_status_panel, toggle_display_mode, DisplayMode},
};
use glam::Affine2;
use macroquad::prelude::*;
use simplemotion_lib::{LayerOverride, LayerPath, OverrideLayerState};
use std::path::Path;

const ROOT_MOTION: &str = "SD002BA";
const RESTING_ROOT_MOTION: &str = "SD002BB";
const PENDULUM_LAYER: &[&str] = &["mask", "bg", "nanami", "レイヤ1"];
const IMPULSE: f32 = 1.8;
const EXTRA_DECELERATION_ANGLE: f32 = 30.0_f32.to_radians();
const EXTRA_RETURN_ACCELERATION: f32 = 35.0;
const RESTING_ANGLE: f32 = 5.0_f32.to_radians();
const RESTING_DURATION: f32 = 1.0;
const RESTORING_ACCELERATION: f32 = 12.0;
const DAMPING: f32 = 0.7;
const SWAY_X_PER_DEGREE: f32 = -3.0;
const SWAY_UP_PER_DEGREE: f32 = 0.18;
const BODY_LAYER: &str = "七海体";

/// Simulates a lightly damped pendulum in radians.
struct Pendulum {
    angle: f32,
    angular_velocity: f32,
    resting_for: f32,
}

impl Pendulum {
    /// Creates a pendulum with a small initial displacement.
    fn new() -> Self {
        Self {
            angle: 5.0_f32.to_radians(),
            angular_velocity: 0.0,
            resting_for: 0.0,
        }
    }

    /// Advances the pendulum and preserves energy while it is resting.
    fn update(&mut self, preserves_energy: bool) {
        let delta = get_frame_time().min(0.05);
        if is_mouse_button_pressed(MouseButton::Left) {
            self.apply_impulse();
        }

        self.angular_velocity -= self.angle * RESTORING_ACCELERATION * delta;
        self.apply_extra_return_force(delta);
        if !preserves_energy {
            self.angular_velocity *= 1.0 - DAMPING * delta;
        }
        self.angle += self.angular_velocity * delta;
        self.update_resting_duration(delta);
    }

    /// Adds an outward impulse on the pendulum's current side.
    fn apply_impulse(&mut self) {
        let direction = if self.angle < 0.0 { -1.0 } else { 1.0 };
        self.angular_velocity += direction * IMPULSE;
    }

    /// Increases gravity toward the center beyond thirty degrees.
    fn apply_extra_return_force(&mut self, delta: f32) {
        let excess_angle = (self.angle.abs() - EXTRA_DECELERATION_ANGLE).max(0.0);
        if excess_angle == 0.0 {
            return;
        }

        let acceleration = excess_angle * EXTRA_RETURN_ACCELERATION;
        self.angular_velocity -= self.angle.signum() * acceleration * delta;
    }

    /// Tracks how long the pendulum has remained near its resting angle.
    fn update_resting_duration(&mut self, delta: f32) {
        if self.angle.abs() < RESTING_ANGLE {
            self.resting_for += delta;
        } else {
            self.resting_for = 0.0;
        }
    }

    /// Returns whether the pendulum has rested near the center for one second.
    fn has_rested(&self) -> bool {
        self.resting_for >= RESTING_DURATION
    }

    /// Returns whether the pendulum has left the resting range.
    fn has_left_resting_range(&self) -> bool {
        self.angle.abs() > RESTING_ANGLE
    }
}

/// Opens the SD002 pendulum override demonstration.
#[macroquad::main("SimpleMotion Pendulum")]
async fn main() {
    let mut animation =
        load_animation(Path::new("anim/sd002")).expect("the bundled sd002 animation should load");
    let root = animation
        .context
        .resources()
        .motion_key(ROOT_MOTION)
        .expect("the SD002BA root motion should exist");
    let pendulum_layer = layer_path(ROOT_MOTION, PENDULUM_LAYER);
    let body_layer = pendulum_layer.child(BODY_LAYER);
    let resting_body_layer = layer_path(RESTING_ROOT_MOTION, PENDULUM_LAYER).child(BODY_LAYER);
    animation
        .context
        .overrides_mut()
        .lock_layer_at_inherited_time(pendulum_layer.clone(), 0);

    let font = load_ttf_font_from_bytes(include_bytes!("../inspector/assets/notosans-jp.ttf"))
        .expect("pendulum font should load");
    let mut pendulum = Pendulum::new();
    let mut body_is_replaced = false;
    let mut display_mode = DisplayMode::Layers;
    animation.context.debug_info_mut().debug_layers = true;

    loop {
        pendulum.update(body_is_replaced);
        update_body_replacement(
            &mut animation,
            &pendulum,
            &body_layer,
            &resting_body_layer,
            &mut body_is_replaced,
        );
        animation.context.overrides_mut().override_layer(
            pendulum_layer.clone(),
            LayerOverride::Append(OverrideLayerState {
                transform: sway_transform(pendulum.angle),
                opa: 1.0,
            }),
        );
        toggle_display_mode(&mut display_mode, animation.context.debug_info_mut());

        let frame = (get_time() * 60.0) as u32;
        let commands = animation.context.to_anim_commands(root, frame);

        clear_background(BLACK);
        draw_commands(&animation, &commands);
        draw_status_panel(
            &animation,
            root,
            frame,
            &commands,
            animation.context.debug_info(),
            display_mode,
            &font,
        );
        draw_hint(&font, pendulum.angle, pendulum.angular_velocity);

        next_frame().await;
    }
}

/// Swaps in SD002BB's body once the pendulum has rested near the center.
fn update_body_replacement(
    animation: &mut common::animation::LoadedAnimation,
    pendulum: &Pendulum,
    body_layer: &LayerPath,
    resting_body_layer: &LayerPath,
    body_is_replaced: &mut bool,
) {
    if !*body_is_replaced && pendulum.has_rested() {
        animation
            .context
            .overrides_mut()
            .replace_layer(body_layer.clone(), resting_body_layer.clone());
        *body_is_replaced = true;
    }
    if *body_is_replaced && pendulum.has_left_resting_range() {
        animation
            .context
            .overrides_mut()
            .clear_layer_replacement(body_layer);
        *body_is_replaced = false;
    }
}

/// Builds the target layer path from readable path segments.
fn layer_path(root: &str, segments: &[&str]) -> LayerPath {
    segments
        .iter()
        .fold(LayerPath::root(root), |path, segment| path.child(segment))
}

/// Reproduces SD002's extra sideways and upward swing offset.
fn sway_transform(angle: f32) -> Affine2 {
    let angle_degrees = angle.to_degrees();
    let translation = Vec2::new(
        angle_degrees * SWAY_X_PER_DEGREE,
        -angle_degrees.abs() * SWAY_UP_PER_DEGREE,
    );
    Affine2::from_translation(translation) * Affine2::from_angle(angle)
}

/// Draws the pendulum controls beneath the animation.
fn draw_hint(font: &Font, angle: f32, angular_velocity: f32) {
    let text = format!(
        "Click: impulse   angle: {:.1} deg   velocity: {:.2}",
        angle.to_degrees(),
        angular_velocity,
    );
    draw_text_ex(
        &text,
        8.0,
        screen_height() - 12.0,
        TextParams {
            font: Some(font),
            font_size: 16,
            color: WHITE,
            ..Default::default()
        },
    );
}
