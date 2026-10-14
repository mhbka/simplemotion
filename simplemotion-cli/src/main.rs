use simplemotion_lib::parser::parse_motion_resources;
use simplemotion_lib::parser::IconKey;
use simplemotion_lib::parser::Layer;
use simplemotion_lib::parser::MotionKey;
use simplemotion_lib::parser::MotionResources;
use simplemotion_lib::parser::ResourceReference;
use std::env;
use std::fs;
use std::process;

/// Reads a motion JSON file and prints a compact summary.
fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: simplemotion-cli <motion.json>");
        process::exit(2);
    });

    let json = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("Could not read {path}: {error}");
        process::exit(1);
    });
    let resources = parse_motion_resources(&json).unwrap_or_else(|error| {
        eprintln!("Could not parse {path}: {error}");
        process::exit(1);
    });

    println!(
        "Document: {}",
        resources.label.as_deref().unwrap_or("<unnamed>")
    );

    for &root in &resources.roots {
        println!(" === ROOT ===");
        print_motion(&resources, root, 0, 0);
    }
}

fn traverse_layer(
    resources: &MotionResources,
    layer: &Layer<ResourceReference>,
    layer_level: usize,
    frame_level: usize,
) {
    let arrow = get_arrow(layer_level, frame_level);
    println!(
        "{arrow} layer: {} ({} frames, {} children)",
        layer.label,
        layer.frame_list.len(),
        layer.children.len()
    );
    for frame in &layer.frame_list {
        if let Some(frame_content) = &frame.content {
            match &frame_content.src {
                ResourceReference::Icon(icon_key) => {
                    print_icon(resources, *icon_key, layer_level, frame_level + 1)
                }
                ResourceReference::Motion(motion_key) => {
                    print_motion(resources, *motion_key, layer_level, frame_level + 1)
                }
                ResourceReference::Layout => print_layout(layer_level, frame_level + 1),
                ResourceReference::Unresolved(content) => {
                    print_unresolved(&content, layer_level, frame_level + 1)
                }
            }
        }
    }
    for child in &layer.children {
        traverse_layer(resources, child, layer_level + 1, frame_level);
    }
}

/// Prints an image-resource reference in the traversed layer tree.
fn print_icon(resources: &MotionResources, key: IconKey, layer_level: usize, frame_level: usize) {
    let icon = resources.icon(key).unwrap();
    let arrow = get_arrow(layer_level, frame_level);
    println!("{arrow} icon: {}", icon.label);
}

/// Prints a nested-motion reference in the traversed layer tree.
fn print_motion(
    resources: &MotionResources,
    key: MotionKey,
    layer_level: usize,
    frame_level: usize,
) {
    let arrow = get_arrow(layer_level, frame_level);
    let label = resources.motion_label(key).unwrap();
    println!("{arrow} motion: {label}");

    let motion = resources.motion(key).unwrap();
    for layer in &motion.layer {
        traverse_layer(resources, layer, layer_level + 1, frame_level);
    }
}

/// Prints a built-in layout reference in the traversed layer tree.
fn print_layout(layer_level: usize, frame_level: usize) {
    let arrow = get_arrow(layer_level, frame_level);
    println!("{arrow} layout");
}

/// Prints a reference that could not be resolved from this document.
fn print_unresolved(content: &str, layer_level: usize, frame_level: usize) {
    let arrow = get_arrow(layer_level, frame_level);
    println!("{arrow} unresolved: {content}");
}

/// Builds a compact tree prefix for a layer or frame reference.
fn get_arrow(layer_level: usize, frame_level: usize) -> String {
    std::iter::repeat_n('=', layer_level).collect::<String>()
        + &std::iter::repeat_n('-', frame_level).collect::<String>()
        + ">"
}
