use crate::animator::time::InheritedTime;

use super::LayerPath;
use glam::Affine2;
use std::collections::HashMap;

/// An override of a layer's state.
#[derive(Clone, Copy, Debug)]
pub enum LayerOverride {
    /// Completely replace the layer's transform.
    Completely(OverrideLayerState),
    /// Append the override transform to the layer's transform.
    Append(OverrideLayerState)
}

/// Replaces the externally controllable state of one layer.
#[derive(Clone, Copy, Debug)]
pub struct OverrideLayerState {
    /// The replacement local transform.
    pub transform: Affine2,
    /// The replacement local opacity.
    pub opa: f32,
}

/// Overrides for the animation.
#[derive(Default)]
pub struct ExternalOverrides {
    layer_states: HashMap<LayerPath, LayerOverride>,
    layer_replacements: HashMap<LayerPath, LayerPath>,
    layer_lock_times: HashMap<LayerPath, InheritedTime>
}

impl ExternalOverrides {
    // New one b-baka!
    pub fn new() -> Self {
        Self {
            layer_states: HashMap::new(),
            layer_replacements: HashMap::new(),
            layer_lock_times: HashMap::new()
        }
    }

    /// Replaces a layer's local transform and opacity during evaluation.
    pub fn override_layer(&mut self, path: LayerPath, value: LayerOverride) {
        self.layer_states.insert(path, value);
    }

    /// Removes one externally supplied layer state.
    pub fn clear_layer_override(&mut self, path: &LayerPath) {
        self.layer_states.remove(path);
    }

    /// Replaces one layer with the donor layer selected by its path.
    pub fn replace_layer(&mut self, target: LayerPath, donor: LayerPath) {
        self.layer_replacements.insert(target, donor);
    }

    /// Removes one externally supplied layer replacement.
    pub fn clear_layer_replacement(&mut self, path: &LayerPath) {
        self.layer_replacements.remove(path);
    }

    /// Lock a layer to a given inherited time.
    /// 
    /// ## Note
    /// This is *inherited* time, not global time. Since externally we only know the global time,
    /// you will have to figure it out.
    /// 
    /// Usually they're the same :p
    pub fn lock_layer_at_inherited_time(&mut self, path: LayerPath, inherited_time: u32) {
        self.layer_lock_times.insert(path, inherited_time.into());
    }

    /// Stop a layer from being locked at an inherited time.
    pub fn unlock_layer_from_inherited_time(&mut self, path: &LayerPath) {
        self.layer_lock_times.remove(path);
    }

    /// Gets the replacement state for the currently evaluated layer.
    pub(crate) fn state_for(&self, path: &LayerPath) -> Option<LayerOverride> {
        self.layer_states.get(path).copied()
    }

    /// Gets the donor path that replaces the currently evaluated layer.
    pub(crate) fn replacement_for(&self, path: &LayerPath) -> Option<&LayerPath> {
        self.layer_replacements.get(path)
    }

    /// Get an inherited time to lock the given path to.
    pub(crate) fn inherited_time_for(&self, path: &LayerPath) -> Option<&InheritedTime> {
        self.layer_lock_times.get(path)
    }
}
