use macroquad::prelude::*;
use simplemotion_lib::{
    parse_motion_resources_from_folder, to_anim_commands, IconKey, MotionResources,
};
use std::{collections::HashMap, env, process};

/// Opens a native window and plays the first root motion in an animation folder.
#[macroquad::main("SimpleMotion")]
async fn main() {
    let folder = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: simplemotion-cli <animation-folder>");
        process::exit(2);
    });
    let resources = parse_motion_resources_from_folder(&folder).unwrap_or_else(|error| {
        eprintln!("Could not load {folder}: {error}");
        process::exit(1);
    });
    let root = *resources.roots.first().unwrap_or_else(|| {
        eprintln!("Animation has no root motion");
        process::exit(1);
    });
    let textures = load_textures(&resources);
    let mut commands = Vec::new();

    //to_anim_commands(&mut commands, &resources, root, 1);
    //println!("{commands:?}");
    //return;

    loop {
        let frame = (get_time() * 60.0) as u32;
        to_anim_commands(&mut commands, &resources, root, frame);

        clear_background(BLACK);
        for command in &commands {
            let Some(texture) = textures.get(&command.icon) else {
                continue;
            };
            draw_command(texture, command.transform, command.opacity);
        }
        draw_text(&format!("frame: {frame}"), 12.0, 24.0, 24.0, WHITE);

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

/// Draws a command using its translation, scale, rotation, and opacity.
fn draw_command(texture: &Texture2D, transform: ::glam::Affine2, opacity: f32) {
    let x_axis = transform.matrix2.x_axis;
    let y_axis = transform.matrix2.y_axis;
    let rotation = x_axis.y.atan2(x_axis.x);
    let size = vec2(
        texture.width() * x_axis.length(),
        texture.height() * y_axis.length(),
    );
    let color = Color::new(1.0, 1.0, 1.0, opacity.clamp(0.0, 1.0));

    draw_texture_ex(
        texture,
        transform.translation.x,
        transform.translation.y,
        color,
        DrawTextureParams {
            dest_size: Some(size),
            rotation,
            ..Default::default()
        },
    );
}
