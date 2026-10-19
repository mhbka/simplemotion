use super::LayerPath;
use glam::Affine2;

/// Collects layer transforms during animation evaluation, for debugging.
pub struct DebugInfo {
    /// Whether layer transform diagnostics should be collected.
    pub debug_layers: bool,
    /// The path and accumulated transform for each visited layer.
    pub layer_transforms: Vec<(LayerPath, LayerEntry, Affine2)>,
}

impl DebugInfo {
    /// Create a new one b-baka!
    pub fn new(debug_layers: bool) -> Self {
        Self {
            debug_layers,
            layer_transforms: Vec::new(),
        }
    }

    /// Clears diagnostics from the previous animation evaluation.
    pub(crate) fn clear(&mut self) {
        self.layer_transforms.clear();
    }

    /// Records a layer entered from a motion frame.
    pub(crate) fn record_motion_layer(&mut self, path: &LayerPath, transform: Affine2) {
        self.record_layer_transform(path, LayerEntry::Motion, transform);
    }

    /// Records a child layer.
    pub(crate) fn record_child_layer(&mut self, path: &LayerPath, transform: Affine2) {
        self.record_layer_transform(path, LayerEntry::Child, transform);
    }

    /// Records a layer.
    fn record_layer_transform(&mut self, path: &LayerPath, kind: LayerEntry, transform: Affine2) {
        if !self.debug_layers {
            return;
        }
        self.layer_transforms.push((path.clone(), kind, transform));
    }
}

/// How the layer was entered.
#[derive(Debug, Clone, Copy)]
pub enum LayerEntry {
    Child,
    Motion,
}
