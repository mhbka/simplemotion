use crate::animation::LoadedAnimation;
use macroquad::prelude::*;

const LIST_WIDTH: f32 = 360.0;
const LINE_HEIGHT: f32 = 22.0;
const PADDING: f32 = 12.0;

/// Identifies which, if any, browser menu is visible.
#[derive(Clone, Copy)]
pub enum BrowserMenu {
    Closed,
    Animations,
    Roots,
}

/// Draws the animation picker and returns a clicked animation index.
pub fn draw_animation_picker(
    animations: &[LoadedAnimation],
    current_index: usize,
    font: &Font,
    scroll: &mut f32,
) -> Option<usize> {
    draw_picker(
        "Animations — Up/Down selects; Enter or click opens roots; Backspace closes",
        animations.iter().map(|animation| animation.label.as_str()),
        current_index,
        font,
        scroll,
    )
}

/// Draws the selected animation's root picker and returns a clicked root index.
pub fn draw_root_picker(
    animation: &LoadedAnimation,
    current_index: usize,
    font: &Font,
    scroll: &mut f32,
) -> Option<usize> {
    let title = format!(
        "{} roots — Up/Down selects; Backspace shows animations",
        animation.label
    );
    let labels = animation
        .context
        .resources()
        .roots
        .iter()
        .enumerate()
        .map(|(index, root)| {
            let label = animation
                .context
                .resources()
                .motion_label(*root)
                .map(String::as_str)
                .unwrap_or("<missing root>");
            format!("{index}: {label}")
        });
    draw_picker(&title, labels, current_index, font, scroll)
}

/// Draws a scrollable selectable list and returns its clicked row.
fn draw_picker<'a>(
    title: &str,
    labels: impl Iterator<Item = impl AsRef<str> + 'a>,
    current_index: usize,
    font: &Font,
    scroll: &mut f32,
) -> Option<usize> {
    let labels: Vec<_> = labels.collect();
    let list_top = PADDING + LINE_HEIGHT + 8.0;
    update_scroll(scroll, labels.len() as f32 * LINE_HEIGHT, list_top);
    let mut y = list_top - *scroll;
    let mouse = mouse_position();

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.88),
    );
    draw_text(title, PADDING, y + 16.0, font);

    for (index, label) in labels.iter().enumerate() {
        if y + LINE_HEIGHT > list_top && y < screen_height() {
            draw_row(y, label.as_ref(), index == current_index, font);
        }
        if row_was_clicked(mouse, y, list_top) {
            return Some(index);
        }
        y += LINE_HEIGHT;
    }

    None
}

/// Draws one selectable row.
fn draw_row(y: f32, label: &str, is_current: bool, font: &Font) {
    let color = if is_current {
        Color::new(0.16, 0.36, 0.60, 0.95)
    } else {
        Color::new(0.12, 0.12, 0.12, 0.9)
    };
    draw_rectangle(PADDING, y, LIST_WIDTH, LINE_HEIGHT - 2.0, color);
    draw_text(label, PADDING + 4.0, y + 16.0, font);
}

/// Returns whether the mouse clicked a row at `y`.
fn row_was_clicked(mouse: (f32, f32), y: f32, list_top: f32) -> bool {
    is_mouse_button_pressed(MouseButton::Left)
        && mouse.0 >= PADDING
        && mouse.0 <= PADDING + LIST_WIDTH
        && mouse.1 >= list_top
        && mouse.1 >= y
        && mouse.1 <= y + LINE_HEIGHT - 2.0
}

/// Updates the list's scroll offset from the mouse wheel.
fn update_scroll(scroll: &mut f32, content_height: f32, list_top: f32) {
    let max_scroll = (content_height - (screen_height() - list_top)).max(0.0);
    let (_, wheel_y) = mouse_wheel();
    *scroll = (*scroll - wheel_y * 3.0 * LINE_HEIGHT).clamp(0.0, max_scroll);
}

/// Draws standard menu text.
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
