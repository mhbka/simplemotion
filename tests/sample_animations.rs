use simplemotion_lib::{parse_motion_resources, AnimationContext};
use std::{fs, path::Path};

/// Evaluates every frame of every bundled root motion without panicking.
#[test]
fn evaluates_every_bundled_animation_frame() {
    for folder in fs::read_dir("anim").unwrap() {
        let folder = folder.unwrap().path();
        if !folder.is_dir() {
            continue;
        }
        evaluate_folder(&folder);
    }
}

/// Evaluates each root motion for one bundled animation folder.
fn evaluate_folder(folder: &Path) {
    let name = folder.file_name().unwrap().to_str().unwrap();
    let json = fs::read_to_string(folder.join(format!("{name}.json"))).unwrap();
    let resources = parse_motion_resources(&json).unwrap();
    let roots = resources.roots.clone();
    let mut context = AnimationContext::new(resources);

    for root in roots {
        let last_time = context.resources().motion(root).unwrap().last_time;
        for frame in 0..=last_time {
            context.to_anim_commands(root, frame);
        }
    }
}
