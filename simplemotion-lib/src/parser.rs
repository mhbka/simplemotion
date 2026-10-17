use crate::models::RawMotionDocument;
pub use crate::models::*;
use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

/// An error returned while parsing or resolving a motion document.
#[derive(Debug)]
pub enum ParseMotionError {
    /// A required Motion JSON file could not be read.
    Io {
        /// The file that could not be read.
        path: PathBuf,
        /// The operating-system error.
        source: std::io::Error,
    },
    /// The supplied directory does not follow the expected animation layout.
    InvalidAnimationDirectory(PathBuf),
    /// One or more declared icon images could not be loaded.
    ImageLoading(Vec<ImageLoadingError>),
    /// The input was not valid Motion JSON.
    Json(serde_json::Error),
    /// The parsed document did not match the supported resource structure.
    Resolution(ResolutionError),
}

impl fmt::Display for ParseMotionError {
    /// Describes the parsing or resolution failure.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "could not read {}: {source}", path.display())
            }
            Self::InvalidAnimationDirectory(path) => write!(
                formatter,
                "animation directory must have a name: {}",
                path.display()
            ),
            Self::ImageLoading(errors) => write!(
                formatter,
                "could not load {} icon image{}",
                errors.len(),
                if errors.len() == 1 { "" } else { "s" }
            ),
            Self::Json(error) => error.fmt(formatter),
            Self::Resolution(error) => error.fmt(formatter),
        }
    }
}

impl Error for ParseMotionError {}

/// An error raised while loading one animation's icon image files.
#[derive(Debug)]
pub enum ImageLoadingError {
    /// The image directory could not be read.
    Directory {
        /// The directory that could not be read.
        path: PathBuf,
        /// The operating-system error.
        source: std::io::Error,
    },
    /// A declared icon did not have a matching supported image.
    Missing {
        /// The exported icon path.
        icon: String,
        /// The directory searched for the image.
        directory: PathBuf,
    },
    /// A matched image file could not be read.
    Io {
        /// The exported icon path.
        icon: String,
        /// The image file that could not be read.
        path: PathBuf,
        /// The operating-system error.
        source: std::io::Error,
    },
}

impl fmt::Display for ImageLoadingError {
    /// Describes one failed icon-image load.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory { path, source } => {
                write!(
                    formatter,
                    "could not read image directory {}: {source}",
                    path.display()
                )
            }
            Self::Missing { icon, directory } => write!(
                formatter,
                "could not find a supported image for icon {icon} in {}",
                directory.display()
            ),
            Self::Io { icon, path, source } => {
                write!(
                    formatter,
                    "could not read image for icon {icon} at {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl Error for ImageLoadingError {}

/// Parses a Motion JSON document into the simple animation model.
pub(crate) fn parse_motion_json(json: &str) -> Result<RawMotionDocument, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parses and resolves Motion JSON into a typed resource graph.
pub fn parse_motion_resources(json: &str) -> Result<MotionResources, ParseMotionError> {
    let raw = parse_motion_json(json).map_err(ParseMotionError::Json)?;

    MotionResources::from_raw(raw).map_err(ParseMotionError::Resolution)
}

/// Parses an example-style animation directory and loads its declared PNG images.
pub fn parse_motion_resources_from_folder(
    folder: impl AsRef<Path>,
) -> Result<MotionResources, ParseMotionError> {
    let folder = folder.as_ref();
    let animation_name = folder
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| ParseMotionError::InvalidAnimationDirectory(folder.to_path_buf()))?;
    let json_path = folder.join(format!("{animation_name}.json"));
    let image_directory = folder.join(animation_name);
    let json = crate::image_loader::read_file(&json_path)?;
    let json = String::from_utf8(json).map_err(|error| {
        ParseMotionError::Json(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            error,
        )))
    })?;
    let mut resources = parse_motion_resources(&json)?;

    let images = crate::image_loader::load_icon_images(
        resources.icon_entries(),
        &image_directory,
        animation_name,
    )
    .map_err(ParseMotionError::ImageLoading)?;
    resources.set_icon_images(images);

    Ok(resources)
}
