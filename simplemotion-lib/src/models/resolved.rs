use crate::models::{
    objects::{Icon, KeyFrame, KeyFrameContent, Layer, Motion, ObjectType, ScreenSize},
    raw::{RawMotion, RawMotionDocument, RawSource},
};
use slotmap::{new_key_type, SlotMap};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fmt;

new_key_type! {
    /// A handle to a resolved motion.
    pub struct MotionKey;
    /// A handle to a resolved image resource.
    pub struct IconKey;
}

/// A resolved motion whose frame sources use typed resource handles.
pub type ResolvedMotion = Motion<ResourceReference>;

/// A typed target of a resolved frame source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceReference {
    /// Draws an image resource.
    Icon(IconKey),
    /// Plays a nested motion.
    Motion(MotionKey),
    /// Uses the built-in layout source.
    Layout,
    /// Preserves a source path with no corresponding exported resource.
    Unresolved(String),
}

/// A resolved resource carrying the exported name used for lookup.
#[derive(Debug)]
pub struct Named<T> {
    /// The exported resource name.
    pub label: String,
    /// The resource data.
    pub value: T,
}

/// An error raised when a raw document cannot form one resource graph.
#[derive(Debug, PartialEq, Eq)]
pub enum ResolutionError {
    /// The document contains no object definition.
    MissingObject,
    /// The document contains more than one object definition.
    MultipleObjects,
}

impl fmt::Display for ResolutionError {
    /// Describes why the raw document cannot be resolved.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingObject => write!(formatter, "motion document contains no object"),
            Self::MultipleObjects => write!(formatter, "motion document contains multiple objects"),
        }
    }
}

impl Error for ResolutionError {}

/// The parsed motion resource graph.
#[derive(Debug)]
pub struct MotionResources {
    /// Document name.
    pub label: Option<String>,
    /// Canvas of the animation.
    pub screen_size: Option<ScreenSize>,
    /// The type assigned to the top-level object (not sure what it does tbh).
    pub object_type: ObjectType,
    /// Motions that no other exported motion plays (most likely the actual animation(s)).
    pub roots: Vec<MotionKey>,
    /// All the defined motions.
    motions: SlotMap<MotionKey, MotionEntry>,
    /// Basically the actual images that can be rendered.
    icons: SlotMap<IconKey, Named<Icon>>,
    /// Map of "label" strings -> our handles.
    motion_labels: BTreeMap<String, MotionKey>,
    motion_paths: HashMap<String, MotionKey>,
    icon_paths: HashMap<String, IconKey>,
}

/// Stores a motion while the resolver reserves handles for forward references.
#[derive(Debug)]
struct MotionEntry {
    motion: Option<ResolvedMotion>,
}

impl MotionResources {
    /// Resolves one raw object into typed handles owned by this resource graph.
    pub(crate) fn from_raw(raw: RawMotionDocument) -> Result<Self, ResolutionError> {
        let RawMotionDocument {
            label,
            screen_size,
            object,
            source,
        } = raw;
        let (object_label, object) = Self::take_object(object)?;
        let object_type = object.object_type;
        let mut resources = Self {
            label,
            screen_size,
            object_type,
            roots: Vec::new(),
            motions: SlotMap::with_key(),
            icons: SlotMap::with_key(),
            motion_labels: BTreeMap::new(),
            motion_paths: HashMap::new(),
            icon_paths: HashMap::new(),
        };

        resources.insert_icons(source);
        let raw_motions = resources.reserve_motions(&object_label, object.motion);
        resources.resolve_motions(raw_motions);
        resources.find_graph_roots();
        Ok(resources)
    }

    /// Looks up a motion handle by its exported motion name.
    pub fn motion_key(&self, label: &str) -> Option<MotionKey> {
        self.motion_labels.get(label).copied()
    }

    /// Looks up a motion's label by its key.
    pub fn motion_label(&self, key: MotionKey) -> Option<&String> {
        self.motion_labels
            .iter()
            .find(|(_, &m_key)| m_key == key)
            .map(|(s, _)| s)
    }

    /// Looks up an image resource handle using its exported icon name.
    pub fn icon_key(&self, label: &str) -> Option<IconKey> {
        self.icon_paths.get(label).copied()
    }

    /// Gets a resolved motion by handle.
    pub fn motion(&self, key: MotionKey) -> Option<&ResolvedMotion> {
        self.motions.get(key)?.motion.as_ref()
    }

    /// Gets an image resource and its exported name by handle.
    pub fn icon(&self, key: IconKey) -> Option<&Named<Icon>> {
        self.icons.get(key)
    }

    /// Iterates through exported motion names and their handles.
    pub fn motions(&self) -> impl Iterator<Item = (&str, MotionKey)> {
        self.motion_labels
            .iter()
            .map(|(label, key)| (label.as_str(), *key))
    }

    /// Extracts the sole object required by the Motion document format.
    fn take_object(
        mut objects: BTreeMap<String, crate::models::raw::RawMotionObject>,
    ) -> Result<(String, crate::models::raw::RawMotionObject), ResolutionError> {
        match objects.len() {
            0 => Err(ResolutionError::MissingObject),
            1 => Ok(objects.pop_first().expect("one object is present")),
            _ => Err(ResolutionError::MultipleObjects),
        }
    }

    /// Registers every icon by its exported `src/group/name` path.
    fn insert_icons(&mut self, raw_sources: BTreeMap<String, RawSource>) {
        for (group_label, raw_source) in raw_sources {
            for (icon_label, icon) in raw_source.icon {
                let path = format!("src/{group_label}/{icon_label}");
                let key = self.icons.insert(Named {
                    label: path.clone(),
                    value: icon,
                });

                self.icon_paths.insert(path, key);
            }
        }
    }

    /// Reserves all motion handles so motions may reference each other in any order.
    fn reserve_motions(
        &mut self,
        object_label: &str,
        raw_motions: BTreeMap<String, RawMotion>,
    ) -> Vec<(MotionKey, RawMotion)> {
        let mut unprocessed = Vec::new();

        for (motion_label, motion) in raw_motions {
            let key = self.motions.insert(MotionEntry { motion: None });
            let path = format!("motion/{object_label}/{motion_label}");

            self.motion_labels.insert(motion_label, key);
            self.motion_paths.insert(path, key);
            unprocessed.push((key, motion));
        }

        unprocessed
    }

    /// Replaces each raw frame source with its resolved resource reference.
    fn resolve_motions(&mut self, raw_motions: Vec<(MotionKey, RawMotion)>) {
        for (key, raw_motion) in raw_motions {
            let resolved = self.resolve_motion(raw_motion);
            let entry = self
                .motions
                .get_mut(key)
                .expect("reserved motion is present");

            entry.motion = Some(resolved);
        }
    }

    /// Finds graph roots by excluding every motion embedded by another motion.
    fn find_graph_roots(&mut self) {
        let mut nested_motions = HashSet::new();

        for entry in self.motions.values() {
            if let Some(motion) = &entry.motion {
                Self::collect_nested_motions(motion, &mut nested_motions);
            }
        }

        self.roots = self
            .motion_labels
            .values()
            .copied()
            .filter(|key| !nested_motions.contains(key))
            .collect();
    }

    /// Collects nested-motion handles from every layer in a motion tree.
    fn collect_nested_motions(motion: &ResolvedMotion, nested_motions: &mut HashSet<MotionKey>) {
        for layer in &motion.layer {
            Self::collect_nested_motions_from_layer(layer, nested_motions);
        }
    }

    /// Collects nested-motion handles from one layer and its descendants.
    fn collect_nested_motions_from_layer(
        layer: &Layer<ResourceReference>,
        nested_motions: &mut HashSet<MotionKey>,
    ) {
        for frame in &layer.frame_list {
            let Some(content) = &frame.content else {
                continue;
            };
            let ResourceReference::Motion(key) = &content.src else {
                continue;
            };

            nested_motions.insert(*key);
        }

        for child in &layer.children {
            Self::collect_nested_motions_from_layer(child, nested_motions);
        }
    }

    /// Resolves every frame in one motion without changing its transform data.
    fn resolve_motion(&self, raw: RawMotion) -> ResolvedMotion {
        Motion {
            motion_type: raw.motion_type,
            last_time: raw.last_time,
            layer: raw
                .layer
                .into_iter()
                .map(|layer| self.resolve_layer(layer))
                .collect(),
        }
    }

    /// Resolves a layer and all of its children.
    fn resolve_layer(&self, raw: Layer<String>) -> Layer<ResourceReference> {
        Layer {
            layer_type: raw.layer_type,
            label: raw.label,
            frame_list: raw
                .frame_list
                .into_iter()
                .map(|frame| self.resolve_frame(frame))
                .collect(),
            children: raw
                .children
                .into_iter()
                .map(|child| self.resolve_layer(child))
                .collect(),
        }
    }

    /// Resolves the optional source held by a frame.
    fn resolve_frame(&self, raw: KeyFrame<String>) -> KeyFrame<ResourceReference> {
        KeyFrame {
            time: raw.time,
            kind: raw.kind,
            content: raw.content.map(|content| self.resolve_content(content)),
        }
    }

    /// Preserves frame properties while replacing the raw source string.
    fn resolve_content(&self, raw: KeyFrameContent<String>) -> KeyFrameContent<ResourceReference> {
        KeyFrameContent {
            src: self.resolve_reference(raw.src),
            ox: raw.ox,
            oy: raw.oy,
            coord: raw.coord,
            angle: raw.angle,
            zx: raw.zx,
            zy: raw.zy,
            sx: raw.sx,
            sy: raw.sy,
            opa: raw.opa,
            mask: raw.mask,
        }
    }

    /// Converts an exported path to a typed resource handle when possible.
    fn resolve_reference(&self, source: String) -> ResourceReference {
        if source == "layout" {
            return ResourceReference::Layout;
        }
        if let Some(key) = self.motion_paths.get(&source) {
            return ResourceReference::Motion(*key);
        }
        if let Some(key) = self.icon_paths.get(&source) {
            return ResourceReference::Icon(*key);
        }

        ResourceReference::Unresolved(source)
    }
}
