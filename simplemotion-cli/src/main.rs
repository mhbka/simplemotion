use macroquad::prelude::*;
use simplemotion_lib::{
    parse_motion_resources_from_folder, to_anim_commands, IconClip, IconKey, MotionResources,
};
use std::{collections::HashMap, env, process};

/// Opens a native window and plays the first root motion in an animation folder.
#[macroquad::main("SimpleMotion")]
async fn main() {
    let folder = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: simplemotion-cli <animation-folder> <root-number>");
        process::exit(2);
    });
    let root = env::args()
        .nth(2)
        .unwrap_or_else(|| {
            eprintln!("Usage: simplemotion-cli <animation-folder>  <root-number>");
            process::exit(2);
        })
        .parse::<usize>()
        .unwrap();
    let resources = parse_motion_resources_from_folder(&folder).unwrap_or_else(|error| {
        eprintln!("Could not load {folder}: {error}");
        process::exit(1);
    });
    let root = resources.roots[root];
    let textures = load_textures(&resources);
    let mut commands = Vec::new();

    let font_data = include_bytes!("../assets/notosans-jp.ttf");
    let font = load_ttf_font_from_bytes(font_data).unwrap();
    let text_params = TextParams {
        font: Some(&font),
        font_size: 16,
        color: WHITE,
        ..Default::default()
    };

    let mut debug_layers = true;
    let mut debug_layer_transforms = Vec::new();

    loop {
        // Toggle between layer transforms and final icon commands.
        if is_key_pressed(KeyCode::Enter) {
            debug_layers = !debug_layers;
        }

        let frame = (get_time() * 60.0) as u32;
        to_anim_commands(
            &mut commands,
            &resources,
            root,
            frame,
            debug_layers,
            &mut debug_layer_transforms,
        );

        clear_background(BLACK);
        for command in &commands {
            let Some(texture) = textures.get(&command.icon) else {
                continue;
            };
            let clip = &resources.icon(command.icon).unwrap().value.clip;
            draw_command(texture, clip, command.transform, command.opacity);
        }
        let line_height = 18.0;
        let menu_height = (usize::max(commands.len(), debug_layer_transforms.len()) as f32 + 1.0)
            * line_height
            + 12.0;
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            menu_height,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        draw_text_ex(&format!("frame: {frame}"), 8.0, 18.0, text_params.clone());

        if debug_layers {
            for (index, (parent, current, transform)) in debug_layer_transforms.iter().enumerate() {
                let transform_info = transform.to_scale_angle_translation();
                let translation = format!("({:.1}, {:.1})", transform_info.2.x, transform_info.2.y);
                let text = format!(
                    "{parent} -> {current} -- angle: {:.1}, translation: {}",
                    transform_info.1.to_degrees(),
                    translation
                );
                draw_text_ex(
                    &text,
                    8.0,
                    36.0 + index as f32 * line_height,
                    text_params.clone(),
                );
            }
        } else {
            for (index, command) in commands.iter().enumerate() {
                let label = resources
                    .icon(command.icon)
                    .map(|icon| icon.label.as_str())
                    .unwrap_or("<missing icon>");
                let transform_info = command.transform.to_scale_angle_translation();
                let translation = format!("({:.1}, {:.1})", transform_info.2.x, transform_info.2.y);
                let text = format!(
                    "{label} - scale: {:.1}, angle: {:.1}, translation: {}",
                    transform_info.0,
                    transform_info.1.to_degrees(),
                    translation
                );
                draw_text_ex(
                    &text,
                    8.0,
                    36.0 + index as f32 * line_height,
                    text_params.clone(),
                );
            }
        }

        next_frame().await;
    }
}

/// Uploads every loaded icon image to a GPU texture.
fn load_textures(resources: &MotionResources) -> HashMap<IconKey, Texture2D> {
    resources
        .icons()
        .filter_map(|(_, key)| {
            let image = resources.icon_image(key)?;
            let texture = Texture2D::from_file_with_format(image.data(), None);
            Some((key, texture))
        })
        .collect()
}

/// Draws a clipped unit quad through the command's complete affine transform.
fn draw_command(texture: &Texture2D, clip: &IconClip, transform: Affine2, opacity: f32) {
    let color = Color::new(1.0, 1.0, 1.0, opacity.clamp(0.0, 1.0));
    let top_left = transform.transform_point2(Vec2::ZERO);
    let top_right = transform.transform_point2(Vec2::X);
    let bottom_right = transform.transform_point2(Vec2::ONE);
    let bottom_left = transform.transform_point2(Vec2::Y);
    let mesh = Mesh {
        vertices: vec![
            Vertex::new(top_left.x, top_left.y, 0.0, clip.left, clip.top, color),
            Vertex::new(top_right.x, top_right.y, 0.0, clip.right, clip.top, color),
            Vertex::new(
                bottom_right.x,
                bottom_right.y,
                0.0,
                clip.right,
                clip.bottom,
                color,
            ),
            Vertex::new(
                bottom_left.x,
                bottom_left.y,
                0.0,
                clip.left,
                clip.bottom,
                color,
            ),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: Some(texture.clone()),
    };

    draw_mesh(&mesh);
}
