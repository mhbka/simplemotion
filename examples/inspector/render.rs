use crate::animation::LoadedAnimation;
use macroquad::prelude::*;
use simplemotion_lib::{animation::AnimationCommand, DebugInfo, MotionKey};

const STATUS_LINE_HEIGHT: f32 = 18.0;

/// Selects the information shown above the animation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Hidden,
    Layers,
    Icons,
}

impl DisplayMode {
    /// Returns the next mode in the inspector's display cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Hidden => Self::Layers,
            Self::Layers => Self::Icons,
            Self::Icons => Self::Hidden,
        }
    }

    /// Returns whether animation evaluation should collect layer transforms.
    pub fn collects_layer_details(self) -> bool {
        self == Self::Layers
    }
}

/// Draws all animation commands using their complete affine transforms.
pub fn draw_commands(animation: &LoadedAnimation, commands: &[AnimationCommand]) {
    for command in commands {
        let Some(texture) = animation.textures.get(&command.icon) else {
            continue;
        };
        draw_command(texture, command.transform, command.opacity);
    }
}

/// Draws playback and command details above the animation.
pub fn draw_status_panel(
    animation: &LoadedAnimation,
    root: MotionKey,
    frame: u32,
    commands: &[AnimationCommand],
    debug_info: &DebugInfo,
    display_mode: DisplayMode,
    font: &Font,
) {
    let detail_count = match display_mode {
        DisplayMode::Hidden => 0,
        DisplayMode::Layers => debug_info.layer_transforms.len(),
        DisplayMode::Icons => commands.len(),
    };
    let height = (detail_count as f32 + 1.0) * STATUS_LINE_HEIGHT + 12.0;
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        height,
        Color::new(0.0, 0.0, 0.0, 0.7),
    );

    let root_label = animation
        .resources
        .motion_label(root)
        .map(String::as_str)
        .unwrap_or("<missing root>");
    draw_text(
        &format!(
            "{} / {root_label} — frame: {frame} (Space: info)",
            animation.label
        ),
        8.0,
        STATUS_LINE_HEIGHT,
        font,
    );

    match display_mode {
        DisplayMode::Hidden => {}
        DisplayMode::Layers => draw_layer_details(debug_info, font),
        DisplayMode::Icons => draw_command_details(animation, commands, font),
    }
}

/// Draws the current layer transforms.
fn draw_layer_details(debug_info: &DebugInfo, font: &Font) {
    for (index, (path, kind, transform)) in debug_info.layer_transforms.iter().enumerate() {
        let transform = transform.to_scale_angle_translation();
        let text = format!(
            "{} ({kind:?}) — angle: {:.1}, translation: ({:.1}, {:.1})",
            path.as_str(),
            transform.1.to_degrees(),
            transform.2.x,
            transform.2.y,
        );
        draw_text(&text, 8.0, detail_y(index), font);
    }
}

/// Draws the current render commands.
fn draw_command_details(animation: &LoadedAnimation, commands: &[AnimationCommand], font: &Font) {
    for (index, command) in commands.iter().enumerate() {
        let label = animation
            .resources
            .icon(command.icon)
            .map(|icon| icon.label.as_str())
            .unwrap_or("<missing icon>");
        let transform = command.transform.to_scale_angle_translation();
        let text = format!(
            "{label} — scale: {:.1}, angle: {:.1}, translation: ({:.1}, {:.1})",
            transform.0,
            transform.1.to_degrees(),
            transform.2.x,
            transform.2.y,
        );
        draw_text(&text, 8.0, detail_y(index), font);
    }
}

/// Returns the baseline for one status detail row.
fn detail_y(index: usize) -> f32 {
    2.0 * STATUS_LINE_HEIGHT + index as f32 * STATUS_LINE_HEIGHT
}

/// Draws standard status text.
fn draw_text(text: &str, x: f32, y: f32, font: &Font) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font: Some(font),
            font_size: 16,
            color: WHITE,
            ..Default::default()
        },
    );
}

/// Draws one textured quad through an affine transform.
fn draw_command(texture: &Texture2D, transform: Affine2, opacity: f32) {
    let color = Color::new(1.0, 1.0, 1.0, opacity.clamp(0.0, 1.0));
    let top_left = transform.transform_point2(Vec2::ZERO);
    let top_right = transform.transform_point2(Vec2::X);
    let bottom_right = transform.transform_point2(Vec2::ONE);
    let bottom_left = transform.transform_point2(Vec2::Y);
    let mesh = Mesh {
        vertices: vec![
            Vertex::new(top_left.x, top_left.y, 0.0, 0.0, 0.0, color),
            Vertex::new(top_right.x, top_right.y, 0.0, 1.0, 0.0, color),
            Vertex::new(bottom_right.x, bottom_right.y, 0.0, 1.0, 1.0, color),
            Vertex::new(bottom_left.x, bottom_left.y, 0.0, 0.0, 1.0, color),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: Some(texture.clone()),
    };
    draw_mesh(&mesh);
}
