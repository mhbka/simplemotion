use crate::{Layer, MotionResources, ResourceReference, animation::{LayerState, ResolvedLayerStates}, animator::{fns::get_local_linear_interpolated_state, time::{InheritedTime, MotionLocalTime}}};

use super::LayerPath;
use glam::Affine2;
use tracing::{debug, trace};
use std::collections::HashMap;

/// An override of a layer's state.
#[derive(Clone, Copy, Debug)]
pub enum LayerOverride {
    /// Completely replace the layer's transform.
    Completely(OverrideLayerState),
    /// Append the override transform to the layer's transform.
    Append(OverrideLayerState)
}

impl LayerOverride {
    /// Apply the override the `state` depending on `scope`.
    pub(crate) fn apply(self, state: LayerState, scope: LayerOverrideScope) -> ResolvedLayerStates {
        let mut new_state = state;
        match self {
            Self::Completely(override_state) => {
                new_state.transform = override_state.transform;
                new_state.opacity = override_state.opa;
            },  
            Self::Append(append_state) => {
                new_state.transform *= append_state.transform;
                new_state.opacity *= append_state.opa;
            }
        };
        match scope {
            LayerOverrideScope::DirectIconOnly => ResolvedLayerStates { 
                inherited: state, 
                icon: new_state 
            },
            LayerOverrideScope::Inherit => ResolvedLayerStates { 
                inherited: new_state, 
                icon: new_state 
            }
        }
    }
}

/// The scope of the override.
#[derive(Clone, Copy, Debug)]
pub enum LayerOverrideScope {
    /// The override should be inherited by child layers/nested motion.
    Inherit,
    /// The override only applies for the layer's icon, if it has one.
    DirectIconOnly
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
    layer_states: HashMap<LayerPath, (LayerOverrideScope, LayerOverride)>,
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
    pub fn override_layer(&mut self, path: LayerPath, value: LayerOverride, scope: LayerOverrideScope) {
        self.layer_states.insert(path, (scope, value));
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

    /// Return the layer's inherited and local times,
    /// checking `overrides` if an override is applicable.
    pub(crate) fn get_layer_times_with_override(
        &self,
        layer_path: &LayerPath,
        layer_time: InheritedTime,
        layer_local_time: MotionLocalTime,
        motion_loop_time: i32,
        motion_last_time: u32,
    ) -> (InheritedTime, MotionLocalTime) {
        if let Some(&override_time) = self.inherited_time_for(layer_path) {
            let override_local_time = MotionLocalTime::calculate(override_time, motion_loop_time, motion_last_time);
            return (layer_time, override_local_time);
        }
        (layer_time, layer_local_time) 
    }

    /// Resolve a layer's state (both the inherited and the layer's icon state), 
    /// applying an external override if present.
    pub(crate) fn resolve_states_with_override(
        &self,
        local_time: MotionLocalTime,
        layer: &Layer<ResourceReference>,
        layer_path: &LayerPath,
    ) -> ResolvedLayerStates {
        let state = get_local_linear_interpolated_state(local_time, &layer.frame_list);
        if let Some((scope, over)) = self.state_for(layer_path) {
            return over.apply(state, scope);
        }
        ResolvedLayerStates {
            inherited: state,
            icon: state
        }
    }

    /// Check if the given layer should be replaced with a different layer
    /// if specified in `overrides`.
    pub(crate) fn replace_layer_if_in_overrides<'a>(
        &self,
        resources: &'a MotionResources,
        layer: &'a Layer<ResourceReference>,
        local_time: MotionLocalTime,
        time: InheritedTime,
        layer_path: &LayerPath,
    ) -> (&'a Layer<ResourceReference>, MotionLocalTime) {
        let Some(replacement_path) = self.replacement_for(layer_path) else {
            trace!("no replacement for layer: {layer_path:?}");
            return (layer, local_time);
        };
        let Some(replacement_context) = resources.layer_context_at(replacement_path) else {
            debug!("no layer for replacement path: {replacement_path:?}");
            return (layer, local_time);
        };
        let Some(replacement_motion) = resources.motion(replacement_context.motion) else {
            debug!("no motion found for replacement path: {replacement_path:?}");
            return (layer, local_time);
        };
        let replacement_local_time =
            MotionLocalTime::calculate(time, replacement_motion.loop_time, replacement_motion.last_time);
        trace!("donor layer obtained for replacement path: {replacement_path:?}");

        (replacement_context.layer, replacement_local_time)
    }


    /// Gets the replacement state for the currently evaluated layer.
    pub(crate) fn state_for(&self, path: &LayerPath) -> Option<(LayerOverrideScope, LayerOverride)> {
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
