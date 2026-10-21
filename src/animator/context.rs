use crate::{
    DebugInfo, ExternalOverrides, Layer, LayerPath, LayerType, Motion, MotionKey, MotionResources, ResourceReference, animation::{AnimationCommand, LayerState, ResolvedLayerStates}, animator::{
        fns::{active_frame_at, active_priority_at}, layer_output::{LayerOutput, MotionLayerOutputs}, time::{InheritedTime, MotionLocalTime},
    },
};
use glam::Affine2;
use tracing::debug;

/// Context of an animation.
pub struct AnimationContext {
    resources: MotionResources,
    overrides: ExternalOverrides,
    debug_info: DebugInfo
}

impl AnimationContext {
    /// Instantiate the context b-baka!
    pub fn new(
        resources: MotionResources,
    ) -> Self {
        Self {
            resources,
            overrides: ExternalOverrides::new(),
            debug_info: DebugInfo::new(false)
        }
    }

    /// Get the animation resources.
    pub fn resources(&self) -> &MotionResources {
        &self.resources
    }

    /// Get the external overrides.
    pub fn overrides(&self) -> &ExternalOverrides {
        &self.overrides
    }

    /// Get the external overrides mutably.
    pub fn overrides_mut(&mut self) -> &mut ExternalOverrides {
        &mut self.overrides
    }

    /// Get the debug information.
    pub fn debug_info(&self) -> &DebugInfo {
        &self.debug_info
    }

    /// Get the debug information mutably.
    pub fn debug_info_mut(&mut self) -> &mut DebugInfo {
        &mut self.debug_info
    }

    /// Obtain the animation commands at the given time for the given motion.
    pub fn to_anim_commands(
        &mut self,
        motion: MotionKey,
        time: u32,
    ) -> Vec<AnimationCommand> {
        let mut evaluator = AnimationEvaluator {
            resources: &self.resources,
            overrides: &self.overrides,
            debug_info: &mut self.debug_info,
        };
        evaluator.to_anim_commands(motion, time)
    }
}

/// Evaluator for a single `time`'s animation commands.
/// 
/// Mostly exists because borrow checker angry if I put these fns directly on the context struct :p
/// 
/// Also look how clean the context struct's impl block is...
struct AnimationEvaluator<'resources, 'debug> {
    resources: &'resources MotionResources,
    overrides: &'resources ExternalOverrides,
    debug_info: &'debug mut DebugInfo,
}

impl AnimationEvaluator<'_, '_> {
    /// Evaluate animation commands from `motion`.
    fn to_anim_commands(
        &mut self,
        motion: MotionKey,
        time: u32,
    ) -> Vec<AnimationCommand> {
        self.debug_info.clear();
        let root_label = self.resources
            .motion_label(motion)
            .map(String::as_str)
            .unwrap_or("<unknown>");
        self.get_motion_commands_inner(
            motion,
            LayerPath::root(root_label),
            LayerState::identity(),
            time.into(),
        )
    }

    /// Get the commands for a layer, then its children.
    fn get_layer_commands(
        &mut self,
        layer: &Layer<ResourceReference>,
        layer_root_motion: &Motion<ResourceReference>,
        outputs: &mut MotionLayerOutputs,
        layer_states: ResolvedLayerStates,
        layer_path: LayerPath,
        local_time: MotionLocalTime,
        time: InheritedTime,
    ) {
        let mut output = LayerOutput::new();
        let inherited_state = layer_states.inherited;
        let icon_state = layer_states.icon;
        match layer.layer_type {
            LayerType::Icon => {
                let command = self.get_icon_command(layer, icon_state, local_time);
                if let Some(command) = command {
                    output.set_own_command(command);
                }
            }
            LayerType::Motion => {
                let motion_commands = self.get_motion_commands(
                    layer,
                    inherited_state,
                    layer_path.clone(),
                    time,
                    local_time,
                );
                if let Some(motion_commands) = motion_commands {
                    output.set_motion_commands(motion_commands);
                }
            }
            LayerType::Layout => {} // already applied layer transform, so nothing needs to be done
            LayerType::Mask => debug!("SKIPPING MASK FOR NOW"),
            LayerType::Other(other) => debug!("UNKNOWN LAYER TYPE: {other}"),
        }

        outputs.push(output);

        self.evaluate_descendant_layers(
            &layer.children, 
            layer_root_motion, 
            outputs, 
            layer_path, 
            inherited_state, 
            time, 
            local_time, 
            false
        );
    }

    /// Get the animation command for an icon.
    fn get_icon_command(
        &self,
        layer: &Layer<ResourceReference>,
        layer_state: LayerState,
        local_time: MotionLocalTime,
    ) -> Option<AnimationCommand> {
        let Some(active_keyframe_index) = active_frame_at(local_time, &layer.frame_list) else {
            return None;
        };
        let active_keyframe = &layer.frame_list[active_keyframe_index];
        match &active_keyframe.content {
            None => return None,
            Some(content) => {
                let ResourceReference::Icon(icon) = content.src else {
                    panic!(
                        "Icon layer contains frame with non-icon source: {:?}",
                        active_keyframe
                    );
                };
                let icon_info = self.resources.icon(icon).expect("icon should exist");
                // origin translation (equation comes from the krkr2 docs)
                // (or maybe just common sense)
                let origin_offset = glam::Vec2::new(
                    -icon_info.value.origin_x - layer_state.anchor_offset.x,
                    -icon_info.value.origin_y - layer_state.anchor_offset.y,
                );
                let mut transform = layer_state.transform * Affine2::from_translation(origin_offset);

                // screen translation -- basically "move" the icon to the centre of the screen
                let screen_translation = Affine2::from_translation(glam::Vec2 {
                    x: self.resources.screen_size.width / 2.0,
                    y: self.resources.screen_size.height / 2.0,
                });
                transform = screen_translation * transform;

                // scale from unit quad to the icon's size
                // (we work with unit quad first, scale up, then sample for the image)
                transform *= Affine2::from_scale(glam::Vec2::new(
                    icon_info.value.width,
                    icon_info.value.height,
                ));

                let command = AnimationCommand {
                    icon,
                    transform,
                    opacity: layer_state.opacity,
                };
                return Some(command);
            }
        }
    }

    /// Get the animation commands for a motion.
    fn get_motion_commands(
        &mut self,
        layer: &Layer<ResourceReference>,
        layer_state: LayerState,
        layer_path: LayerPath,
        mut time: InheritedTime,
        local_time: MotionLocalTime,
    ) -> Option<Vec<AnimationCommand>> {
        let Some(active_keyframe_index) = active_frame_at(local_time, &layer.frame_list) else {
            return None;
        };
        let active_keyframe = &layer.frame_list[active_keyframe_index];
        if let Some(content) = &active_keyframe.content {
            let ResourceReference::Motion(motion) = content.src else {
                debug!("Motion layer has non-motion keyframe, not animating: {active_keyframe:?}");
                return None;
            };
            time = time.inherit(
                active_keyframe.time,
                content
                    .motion
                    .as_ref()
                    .and_then(|motion| motion.time_offset),
            );
            let commands = self.get_motion_commands_inner(
                motion,
                layer_path,
                layer_state.into_motion_parent_state(),
                time,
            );
            return Some(commands);
        } else {
            return None;
        }
    }

    /// Gets the animation commands for a motion.
    fn get_motion_commands_inner(
        &mut self,
        motion: MotionKey,
        parent_path: LayerPath,
        parent_state: LayerState,
        time: InheritedTime,
    ) -> Vec<AnimationCommand> {
        let resources = self.resources;
        let motion = resources.motion(motion).expect("motion should exist");
        let mut motion_outputs = MotionLayerOutputs::new();
        let local_time = MotionLocalTime::calculate(time, motion.loop_time, motion.last_time);

        self.evaluate_descendant_layers(
            &motion.layer,
            motion,
            &mut motion_outputs,
            parent_path,
            parent_state,
            time,
            local_time,
            true
        );

        let priority = active_priority_at(local_time, &motion.priority).map(|i| &motion.priority[i]);
        let motion_commands = motion_outputs.finalize(priority);
        motion_commands
    }

    /// Evaluate the descendant layers, 
    /// appending their `LayerOutput`s into `outputs`.
    fn evaluate_descendant_layers(
        &mut self,
        layers: &[Layer<ResourceReference>],
        layer_root_motion: &Motion<ResourceReference>,
        outputs: &mut MotionLayerOutputs,
        parent_path: LayerPath,
        parent_state: LayerState,
        time: InheritedTime,
        local_time: MotionLocalTime,
        direct_from_motion: bool
    ) {
        for layer in layers {
            let resources = self.resources;
            let overrides = self.overrides;
            let layer_path = parent_path.child(&layer.label);
            let (layer_time, layer_local_time) = 
                overrides.get_layer_times_with_override(&layer_path, time, local_time, layer_root_motion.loop_time, layer_root_motion.last_time);
            let (layer, layer_local_time) =
                overrides.replace_layer_if_in_overrides(resources, layer, layer_local_time, layer_time, &layer_path);
            let resolved_local_layer_states =
                overrides.resolve_states_with_override(layer_local_time, layer, &layer_path);
            let local_layer_states = resolved_local_layer_states.as_child(parent_state);

            if direct_from_motion {
                self.debug_info.record_motion_layer(&layer_path, local_layer_states.inherited.transform);
            } else {
                self.debug_info.record_child_layer(&layer_path, local_layer_states.inherited.transform);
            }

            self.get_layer_commands(
                layer,
                layer_root_motion,
                outputs,
                local_layer_states,
                layer_path,
                layer_local_time,
                layer_time,
            );
        }
    }
}
