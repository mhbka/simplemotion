use super::LayerPath;
use glam::Affine2;
use std::collections::HashMap;

/// Replaces the externally controllable state of one layer.
#[derive(Clone, Copy, Debug)]
pub struct OverrideLayerState {
    /// The replacement local transform.
    pub transform: Affine2,
    /// The replacement local opacity.
    pub opa: f32,
}

/// Stores external layer-state replacements without modifying parsed resources.
#[derive(Default)]
pub struct ExternalOverrides {
    layer_states: HashMap<LayerPath, OverrideLayerState>,
    layer_replacements: HashMap<LayerPath, LayerPath>,
}

impl ExternalOverrides {
    // New one b-baka!
    pub fn new() -> Self {
        Self {
            layer_states: HashMap::new(),
            layer_replacements: HashMap::new(),
        }
    }

    /// Replaces a layer's local transform and opacity during evaluation.
    pub fn override_layer(&mut self, path: LayerPath, state: OverrideLayerState) {
        self.layer_states.insert(path, state);
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

    /// Gets the replacement state for the currently evaluated layer.
    pub(crate) fn state_for(&self, path: &LayerPath) -> Option<OverrideLayerState> {
        self.layer_states.get(path).copied()
    }

    /// Gets the donor path that replaces the currently evaluated layer.
    pub(crate) fn replacement_for(&self, path: &LayerPath) -> Option<&LayerPath> {
        self.layer_replacements.get(path)
    }
}
