use macroquad::prelude::*;
use simplemotion_lib::{
    parse_motion_resources_from_folder, to_anim_commands,
    DebugInfo, ExternalOverrides, IconKey, LayerPath, MotionResources,
};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process,
};

/// An animation and the textures needed to display it.
struct LoadedAnimation {
    label: String,
    resources: MotionResources,
    textures: HashMap<IconKey, Texture2D>,
}

/// The currently visible browser menu in multi-animation mode.
#[derive(Clone, Copy)]
enum AnimationMenu {
    Closed,
    Animations,
    Roots,
}

/// Opens a native window and plays an animation folder or browses many animation folders.
#[macroquad::main("SimpleMotion")]
async fn main() {
    let arguments: Vec<_> = env::args().skip(1).collect();
    let multiple_animations = arguments.first().is_some_and(|argument| argument == "-m");
    let animations = if multiple_animations {
        let Some(folder) = arguments.get(1) else {
            print_usage();
            process::exit(2);
        };
        load_animation_folders(Path::new(folder))
    } else {
        let Some(folder) = arguments.first() else {
            print_usage();
            process::exit(2);
        };
        vec![load_animation(Path::new(folder)).unwrap_or_else(|error| {
            eprintln!("Could not load {folder}: {error}");
            process::exit(1);
        })]
    };
    if animations.is_empty() {
        eprintln!("No playable animation folders were found.");
        process::exit(1);
    }

    let mut animation_index = 0;
    let mut root_index = if multiple_animations {
        0
    } else {
        arguments
            .get(1)
            .and_then(|root| root.parse().ok())
            .unwrap_or_else(|| {
                print_usage();
                process::exit(2);
            })
    };
    if root_index >= animations[animation_index].resources.roots.len() {
        eprintln!("Root number {root_index} does not exist for this animation.");
        process::exit(2);
    }

    let font_data = include_bytes!("../assets/notosans-jp.ttf");
    let font = load_ttf_font_from_bytes(font_data).unwrap();
    let text_params = TextParams {
        font: Some(&font),
        font_size: 16,
        color: WHITE,
        ..Default::default()
    };

    let mut overrides = ExternalOverrides::new();
    let mut debug_info = DebugInfo::new(true);
    let mut animation_started_at = get_time();
    let mut animation_menu = AnimationMenu::Closed;
    let mut animation_menu_scroll = 0.0;
    let mut root_menu_scroll = 0.0;

    let mut last_second = 0;

    loop {
        if is_key_pressed(KeyCode::Space) {
            debug_info.debug_layers = !debug_info.debug_layers;
        }

        if multiple_animations && is_key_pressed(KeyCode::Enter) {
            animation_menu = match animation_menu {
                AnimationMenu::Closed => AnimationMenu::Animations,
                AnimationMenu::Animations | AnimationMenu::Roots => AnimationMenu::Closed,
            };
        }

        let animation = &animations[animation_index];
        let root = animation.resources.roots[root_index];
        let time = get_time();
        let frame = ((time - animation_started_at) * 60.0) as u32;
        let commands = to_anim_commands(
            &animation.resources,
            root,
            frame,
            &overrides,
            &mut debug_info,
        );

        clear_background(BLACK);
        for command in &commands {
            let Some(texture) = animation.textures.get(&command.icon) else {
                continue;
            };
            draw_command(texture, command.transform, command.opacity);
        }
        let line_height = 18.0;
        let menu_height = (usize::max(commands.len(), debug_info.layer_transforms.len()) as f32
            + 1.0)
            * line_height
            + 12.0;
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            menu_height,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        let root_label = animation
            .resources
            .motion_label(root)
            .map(String::as_str)
            .unwrap_or("<missing root>");
        draw_text_ex(
            &format!(
                "{} / {root_label} — frame: {frame} (Space: debug)",
                animation.label
            ),
            8.0,
            18.0,
            text_params.clone(),
        );

        let now_second = frame / 60;
        if now_second < last_second {
            last_second = now_second;
        }
        if now_second > last_second {
            last_second = now_second;
            let replacee_path1 =
                LayerPath::from_string("SD002AA/mask/bg/nanami/レイヤ1/七海体".to_string());
            let replacement_path1 =
                LayerPath::from_string("SD002AB/mask/bg/nanami/レイヤ1/七海体".to_string());
            if last_second % 2 == 0 {
                overrides.replace_layer(replacee_path1, replacement_path1);
            } else {
                overrides.clear_layer_replacement(&replacee_path1);
            }
        }

        if debug_info.debug_layers {
            for (index, (path, kind, transform)) in debug_info.layer_transforms.iter().enumerate() {
                let transform_info = transform.to_scale_angle_translation();
                let translation = format!("({:.1}, {:.1})", transform_info.2.x, transform_info.2.y);
                let text = format!(
                    "{} (kind: {:?}) ------ angle: {:.1}, translation: {}",
                    path.as_str(),
                    kind,
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
                let label = animation
                    .resources
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

        match animation_menu {
            AnimationMenu::Closed => {}
            AnimationMenu::Animations => {
                if let Some(selected_animation) = draw_animation_menu(
                    &animations,
                    animation_index,
                    &font,
                    &mut animation_menu_scroll,
                ) {
                    animation_index = selected_animation;
                    root_index = 0;
                    animation_started_at = get_time();
                    root_menu_scroll = 0.0;
                    animation_menu = AnimationMenu::Roots;
                }
            }
            AnimationMenu::Roots => {
                if let Some(selected_root) = draw_root_menu(
                    &animations[animation_index],
                    root_index,
                    &font,
                    &mut root_menu_scroll,
                ) {
                    root_index = selected_root;
                    animation_started_at = get_time();
                }
            }
        }

        next_frame().await;
    }
}

/// Prints the CLI usage for single-animation and browser modes.
fn print_usage() {
    eprintln!("Usage: simplemotion-cli <animation-folder> <root-number>");
    eprintln!("       simplemotion-cli -m <animations-folder>");
}

/// Loads one animation folder and uploads its icon textures.
fn load_animation(folder: &Path) -> Result<LoadedAnimation, simplemotion_lib::ParseMotionError> {
    let resources = parse_motion_resources_from_folder(folder)?;
    let label = folder
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("<unnamed animation>")
        .to_owned();
    let textures = load_textures(&resources);

    Ok(LoadedAnimation {
        label,
        resources,
        textures,
    })
}

/// Loads each playable immediate subfolder of a multi-animation directory.
fn load_animation_folders(folder: &Path) -> Vec<LoadedAnimation> {
    let entries = fs::read_dir(folder).unwrap_or_else(|error| {
        eprintln!("Could not read {}: {error}", folder.display());
        process::exit(1);
    });
    let mut folders: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    folders.sort();

    folders
        .into_iter()
        .filter_map(|folder| match load_animation(&folder) {
            Ok(animation) if !animation.resources.roots.is_empty() => Some(animation),
            Ok(_) => {
                eprintln!("Skipping {}: it has no root motions", folder.display());
                None
            }
            Err(error) => {
                eprintln!("Skipping {}: {error}", folder.display());
                None
            }
        })
        .collect()
}

/// Draws the multi-animation picker and returns the clicked animation index.
fn draw_animation_menu(
    animations: &[LoadedAnimation],
    current_animation_index: usize,
    font: &Font,
    scroll: &mut f32,
) -> Option<usize> {
    let line_height = 22.0;
    let padding = 12.0;
    let list_top = padding + line_height + 8.0;
    let content_height = animations.len() as f32 * line_height;
    update_menu_scroll(scroll, content_height, list_top);
    let mut y = list_top - *scroll;
    let mouse = mouse_position();

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.88),
    );
    draw_text_ex(
        "Animations — click one to choose its roots; Enter closes this menu",
        padding,
        y + 16.0,
        TextParams {
            font: Some(font),
            font_size: 16,
            color: WHITE,
            ..Default::default()
        },
    );
    for (animation_index, animation) in animations.iter().enumerate() {
        let is_current = animation_index == current_animation_index;
        let row_color = if is_current {
            Color::new(0.16, 0.36, 0.60, 0.95)
        } else {
            Color::new(0.12, 0.12, 0.12, 0.9)
        };
        if y + line_height > list_top && y < screen_height() {
            draw_rectangle(padding, y, 360.0, line_height - 2.0, row_color);
            draw_text_ex(
                &animation.label,
                padding + 4.0,
                y + 16.0,
                TextParams {
                    font: Some(font),
                    font_size: 16,
                    color: WHITE,
                    ..Default::default()
                },
            );
        }

        let clicked = is_mouse_button_pressed(MouseButton::Left)
            && mouse.0 >= padding
            && mouse.0 <= padding + 360.0
            && mouse.1 >= list_top
            && mouse.1 >= y
            && mouse.1 <= y + line_height - 2.0;
        if clicked {
            return Some(animation_index);
        }
        y += line_height;
    }

    None
}

/// Draws the selected animation's root picker and returns the clicked root index.
fn draw_root_menu(
    animation: &LoadedAnimation,
    current_root_index: usize,
    font: &Font,
    scroll: &mut f32,
) -> Option<usize> {
    let line_height = 22.0;
    let padding = 12.0;
    let list_top = padding + line_height + 8.0;
    let content_height = animation.resources.roots.len() as f32 * line_height;
    update_menu_scroll(scroll, content_height, list_top);
    let mut y = list_top - *scroll;
    let mouse = mouse_position();

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.88),
    );
    draw_text_ex(
        &format!(
            "{} roots - click one to play it; Enter closes this menu",
            animation.label
        ),
        padding,
        y + 16.0,
        TextParams {
            font: Some(font),
            font_size: 16,
            color: WHITE,
            ..Default::default()
        },
    );
    for (root_index, root) in animation.resources.roots.iter().enumerate() {
        let label = animation
            .resources
            .motion_label(*root)
            .map(String::as_str)
            .unwrap_or("<missing root>");
        let is_current = root_index == current_root_index;
        let row_color = if is_current {
            Color::new(0.16, 0.36, 0.60, 0.95)
        } else {
            Color::new(0.12, 0.12, 0.12, 0.9)
        };
        if y + line_height > list_top && y < screen_height() {
            draw_rectangle(padding, y, 360.0, line_height - 2.0, row_color);
            draw_text_ex(
                &format!("{root_index}: {label}"),
                padding + 4.0,
                y + 16.0,
                TextParams {
                    font: Some(font),
                    font_size: 16,
                    color: WHITE,
                    ..Default::default()
                },
            );
        }

        let clicked = is_mouse_button_pressed(MouseButton::Left)
            && mouse.0 >= padding
            && mouse.0 <= padding + 360.0
            && mouse.1 >= list_top
            && mouse.1 >= y
            && mouse.1 <= y + line_height - 2.0;
        if clicked {
            return Some(root_index);
        }
        y += line_height;
    }

    None
}

/// Updates one menu's scroll offset from the mouse wheel.
fn update_menu_scroll(scroll: &mut f32, content_height: f32, list_top: f32) {
    let max_scroll = (content_height - (screen_height() - list_top)).max(0.0);
    let (_, wheel_y) = mouse_wheel();
    *scroll = (*scroll - wheel_y * 3.0 * 22.0).clamp(0.0, max_scroll);
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

/// Draws an exported icon image through the command's complete affine transform.
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
