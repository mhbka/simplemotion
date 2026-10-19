use macroquad::prelude::*;
use simplemotion_lib::{
    parse_motion_resources_from_folder, AnimationContext, IconKey, MotionResources,
    ParseMotionError,
};
use std::{collections::HashMap, path::Path};

/// Holds one parsed animation and its uploaded icon textures.
pub struct LoadedAnimation {
    pub label: String,
    pub context: AnimationContext,
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
        context: AnimationContext::new(resources),
    })
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
