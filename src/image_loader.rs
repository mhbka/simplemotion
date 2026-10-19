use crate::{Icon, IconImage, IconImageKind, IconKey, ImageLoadingError, Named, ParseMotionError};
use slotmap::{SecondaryMap, SlotMap};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

/// Loads every declared icon image from one animation's image directory.
///
/// It attempts to load all images and coalesces all errors.
///
/// It is opinionated. It expects the images to be in a subfolder in the animation folder,
/// named the same as the animation itself,
/// and that images are named exactly the same as their icon labels,
/// or are prefixed with the animation name.
///
/// It only accepts .png images for now.
pub(crate) fn load_icon_images(
    icons: &SlotMap<IconKey, Named<Icon>>,
    image_directory: &Path,
    animation_name: &str,
) -> Result<SecondaryMap<IconKey, IconImage>, Vec<ImageLoadingError>> {
    let icons = icons
        .iter()
        .map(|(key, icon)| (icon.label.clone(), key))
        .collect::<Vec<_>>();

    let image_paths = match image_paths(image_directory) {
        Ok(paths) => paths,
        Err(error) => return Err(vec![error]),
    };
    let mut image_map = SecondaryMap::new();
    let mut errors = Vec::new();

    for (icon_path, key) in icons {
        let Some((image_path, image_kind)) =
            find_icon_image(&image_paths, animation_name, &icon_path)
        else {
            errors.push(ImageLoadingError::Missing {
                icon: icon_path,
                directory: image_directory.to_path_buf(),
            });
            continue;
        };
        match fs::read(&image_path) {
            Ok(bytes) => {
                image_map.insert(key, IconImage::new(image_kind, Arc::from(bytes)));
            }
            Err(source) => {
                errors.push(ImageLoadingError::Io {
                    icon: icon_path,
                    path: image_path,
                    source,
                });
            }
        }
    }

    if errors.is_empty() {
        Ok(image_map)
    } else {
        Err(errors)
    }
}

/// Reads one required resource file with its path retained for errors.
pub(crate) fn read_file(path: &Path) -> Result<Vec<u8>, ParseMotionError> {
    fs::read(path).map_err(|source| ParseMotionError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Collects all direct entries in the animation image directory.
fn image_paths(image_directory: &Path) -> Result<Vec<PathBuf>, ImageLoadingError> {
    let entries = fs::read_dir(image_directory).map_err(|source| ImageLoadingError::Directory {
        path: image_directory.to_path_buf(),
        source,
    })?;
    let mut paths = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|source| ImageLoadingError::Directory {
            path: image_directory.to_path_buf(),
            source,
        })?;
        paths.push(entry.path());
    }

    Ok(paths)
}

/// Finds the unprefixed image first, then the animation-prefixed fallback.
fn find_icon_image(
    image_paths: &[PathBuf],
    animation_name: &str,
    icon_path: &str,
) -> Option<(PathBuf, IconImageKind)> {
    let icon_name = icon_path
        .rsplit('/')
        .next()
        .expect("icon paths have a final name");
    for kind in IconImageKind::ALL {
        let exact_name = format!("{icon_name}.{}", kind.extension());
        if let Some(path) = find_named_image(image_paths, &exact_name) {
            return Some((path, *kind));
        }
    }

    for kind in IconImageKind::ALL {
        let prefixed_name = format!("{animation_name}-{icon_name}.{}", kind.extension());
        if let Some(path) = find_named_image(image_paths, &prefixed_name) {
            return Some((path, *kind));
        }
    }

    None
}

/// Finds one supported image filename without depending on filesystem case rules.
fn find_named_image(image_paths: &[PathBuf], expected_name: &str) -> Option<PathBuf> {
    image_paths.iter().find_map(|path| {
        let file_name = path.file_name()?.to_str()?;
        file_name
            .eq_ignore_ascii_case(expected_name)
            .then(|| path.clone())
    })
}
