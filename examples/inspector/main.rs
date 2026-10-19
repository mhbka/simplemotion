mod animation;
mod menu;
mod render;

use animation::{load_animation, load_animation_folders, LoadedAnimation};
use macroquad::prelude::*;
use menu::{draw_animation_picker, draw_root_picker, BrowserMenu};
use render::{draw_commands, draw_status_panel, DisplayMode};
use simplemotion_lib::{to_anim_commands, DebugInfo, ExternalOverrides};
use std::{env, path::Path, process};

/// Opens a native window and plays an animation folder or browses many animation folders.
#[macroquad::main("SimpleMotion")]
async fn main() {
    let arguments: Vec<_> = env::args().skip(1).collect();
    let multiple_animations = arguments.first().is_some_and(|argument| argument == "-m");
    let animations = load_requested_animations(&arguments, multiple_animations);
    let mut root_index = requested_root_index(&arguments, multiple_animations, &animations);

    let font = load_ttf_font_from_bytes(include_bytes!("assets/notosans-jp.ttf"))
        .expect("inspector font should load");
    let mut animation_index = 0;
    let mut display_mode = DisplayMode::Hidden;
    let mut debug_info = DebugInfo::new(display_mode.collects_layer_details());
    let overrides = ExternalOverrides::new();
    let mut started_at = get_time();
    let mut menu = BrowserMenu::Closed;
    let mut animation_menu_scroll = 0.0;
    let mut root_menu_scroll = 0.0;

    loop {
        toggle_display_mode(&mut display_mode, &mut debug_info);
        update_browser_navigation(multiple_animations, &mut menu);
        update_keyboard_selection(
            &animations,
            menu,
            &mut animation_index,
            &mut root_index,
            &mut started_at,
        );

        let animation = &animations[animation_index];
        let root = animation.resources.roots[root_index];
        let frame = ((get_time() - started_at) * 60.0) as u32;
        let commands = to_anim_commands(
            &animation.resources,
            root,
            frame,
            &overrides,
            &mut debug_info,
        );

        clear_background(BLACK);
        draw_commands(animation, &commands);
        draw_status_panel(
            animation,
            root,
            frame,
            &commands,
            &debug_info,
            display_mode,
            &font,
        );

        update_browser_menu(
            &animations,
            &font,
            &mut animation_index,
            &mut root_index,
            &mut started_at,
            &mut menu,
            &mut animation_menu_scroll,
            &mut root_menu_scroll,
        );

        next_frame().await;
    }
}

/// Loads the requested animation folder or all folders in browser mode.
fn load_requested_animations(
    arguments: &[String],
    multiple_animations: bool,
) -> Vec<LoadedAnimation> {
    let animations = if multiple_animations {
        let Some(folder) = arguments.get(1) else {
            print_usage_and_exit();
        };
        load_animation_folders(Path::new(folder))
    } else {
        let Some(folder) = arguments.first() else {
            print_usage_and_exit();
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
    animations
}

/// Gets the requested root index and ensures it exists in the first animation.
fn requested_root_index(
    arguments: &[String],
    multiple_animations: bool,
    animations: &[LoadedAnimation],
) -> usize {
    let root_index = if multiple_animations {
        0
    } else {
        arguments
            .get(1)
            .and_then(|root| root.parse().ok())
            .unwrap_or_else(|| print_usage_and_exit())
    };
    if root_index >= animations[0].resources.roots.len() {
        eprintln!("Root number {root_index} does not exist for this animation.");
        process::exit(2);
    }
    root_index
}

/// Cycles the visible details with Space.
fn toggle_display_mode(display_mode: &mut DisplayMode, debug_info: &mut DebugInfo) {
    if is_key_pressed(KeyCode::Space) {
        *display_mode = display_mode.next();
        debug_info.debug_layers = display_mode.collects_layer_details();
    }
}

/// Opens, closes, or goes back within the browser menu.
fn update_browser_navigation(multiple_animations: bool, menu: &mut BrowserMenu) {
    if !multiple_animations {
        return;
    }
    if is_key_pressed(KeyCode::Enter) {
        *menu = match menu {
            BrowserMenu::Closed => BrowserMenu::Roots,
            BrowserMenu::Animations => BrowserMenu::Roots,
            BrowserMenu::Roots => BrowserMenu::Closed,
        };
    }
    if is_key_pressed(KeyCode::Backspace) {
        *menu = match menu {
            BrowserMenu::Roots => BrowserMenu::Animations,
            BrowserMenu::Animations => BrowserMenu::Closed,
            BrowserMenu::Closed => BrowserMenu::Closed,
        };
    }
}

/// Selects the previous or next item in the open browser menu.
fn update_keyboard_selection(
    animations: &[LoadedAnimation],
    menu: BrowserMenu,
    animation_index: &mut usize,
    root_index: &mut usize,
    started_at: &mut f64,
) {
    let direction = if is_key_pressed(KeyCode::Up) {
        Some(-1)
    } else if is_key_pressed(KeyCode::Down) {
        Some(1)
    } else {
        None
    };
    let Some(direction) = direction else {
        return;
    };

    match menu {
        BrowserMenu::Closed => {}
        BrowserMenu::Animations => {
            *animation_index = move_selection(*animation_index, animations.len(), direction);
            *root_index = 0;
            *started_at = get_time();
        }
        BrowserMenu::Roots => {
            let roots = animations[*animation_index].resources.roots.len();
            *root_index = move_selection(*root_index, roots, direction);
            *started_at = get_time();
        }
    }
}

/// Moves a selection one step, clamping it to the list bounds.
fn move_selection(current: usize, item_count: usize, direction: i8) -> usize {
    match direction {
        -1 => current.saturating_sub(1),
        1 => (current + 1).min(item_count.saturating_sub(1)),
        _ => current,
    }
}

/// Handles a browser-menu selection and resets playback when necessary.
fn update_browser_menu(
    animations: &[LoadedAnimation],
    font: &Font,
    animation_index: &mut usize,
    root_index: &mut usize,
    started_at: &mut f64,
    menu: &mut BrowserMenu,
    animation_menu_scroll: &mut f32,
    root_menu_scroll: &mut f32,
) {
    match menu {
        BrowserMenu::Closed => {}
        BrowserMenu::Animations => {
            let Some(selected) =
                draw_animation_picker(animations, *animation_index, font, animation_menu_scroll)
            else {
                return;
            };
            *animation_index = selected;
            *root_index = 0;
            *started_at = get_time();
            *root_menu_scroll = 0.0;
            *menu = BrowserMenu::Roots;
        }
        BrowserMenu::Roots => {
            if let Some(selected) = draw_root_picker(
                &animations[*animation_index],
                *root_index,
                font,
                root_menu_scroll,
            ) {
                *root_index = selected;
                *started_at = get_time();
            }
        }
    }
}

/// Prints command usage and exits with a standard argument error.
fn print_usage_and_exit() -> ! {
    eprintln!("Usage: cargo run --example inspector -- <animation-folder> <root-number>");
    eprintln!("       cargo run --example inspector -- -m <animations-folder>");
    process::exit(2);
}
