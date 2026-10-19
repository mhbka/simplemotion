use macroquad::prelude::*;
use simplemotion_lib::{
    parse_motion_resources_from_folder, IconKey, MotionResources, ParseMotionError,
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process,
};

/// Holds one parsed animation and its uploaded icon textures.
pub struct LoadedAnimation {
    pub label: String,
    pub resources: MotionResources,
    pub textures: HashMap<IconKey, Texture2D>,
}

/// Loads one animation folder and uploads its icon textures.
pub fn load_animation(folder: &Path) -> Result<LoadedAnimation, ParseMotionError> {
    let resources = parse_motion_resources_from_folder(folder)?;
    let label = folder
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("<unnamed animation>")
        .to_owned();

    Ok(LoadedAnimation {
        label,
        textures: load_textures(&resources),
        resources,
    })
}

/// Loads every playable immediate subfolder of a directory.
pub fn load_animation_folders(folder: &Path) -> Vec<LoadedAnimation> {
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

/// Uploads every available icon image to a GPU texture.
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
