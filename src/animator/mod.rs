use crate::{
    FrameType, KeyFrame, Layer, LayerType, Motion, MotionKey, MotionResources, PriorityFrame, ResourceReference, animation::{AnimationCommand, InterpolatedLayerState}, animator::{
        ext_override::LayerOverride, layer_output::{LayerOutput, MotionLayerOutputs}, time::{InheritedTime, MotionLocalTime},
    },
};
use glam::Affine2;
use tracing::{debug, trace, warn};

mod debug;
mod ext_override;
mod layer_output;
mod path;
mod time;

pub use debug::DebugInfo;
pub use ext_override::{ExternalOverrides, OverrideLayerState};
pub use path::LayerPath;

/// Animates `motion` at the given `time`, outputting animation commands into `buffer`,
/// and using `overrides` for overriding layers behaviour where needed.
///
/// This is the animation entrypoint, where `motion` is typically one of the root motions.
pub fn to_anim_commands(
    resources: &MotionResources,
    motion: MotionKey,
    time: u32,
    overrides: &ExternalOverrides,
    debug_info: &mut DebugInfo,
) -> Vec<AnimationCommand> {
    debug_info.clear();
    let root_label = resources
        .motion_label(motion)
        .map(String::as_str)
        .unwrap_or("<unknown>");
    get_motion_commands_inner(
        motion,
        resources,
        LayerPath::root(root_label),
        InterpolatedLayerState::identity(),
        time.into(),
        overrides,
        debug_info,
    )
}

/// Get the commands for a layer, then its children.
fn get_layer_commands(
    layer: &Layer<ResourceReference>,
    root_motion: &Motion<ResourceReference>,
    resources: &MotionResources,
    all_outputs: &mut MotionLayerOutputs,
    overrides: &ExternalOverrides,
    debug_info: &mut DebugInfo,
    layer_state: InterpolatedLayerState,
    layer_path: LayerPath,
    local_time: MotionLocalTime,
    time: InheritedTime,
) {
    let mut output = LayerOutput::new();
    match layer.layer_type {
        LayerType::Icon => {
            let command = get_icon_command(resources, layer, layer_state, local_time);
            if let Some(command) = command {
                output.set_own_command(command);
            }
        }
        LayerType::Motion => {
            let motion_commands = get_motion_commands(
                resources,
                layer,
                layer_state,
                layer_path.clone(),
                time,
                local_time,
                overrides,
                debug_info,
            );
            if let Some(motion_commands) = motion_commands {
                output.set_motion_commands(motion_commands);
            }
        }
        LayerType::Layout => {} // already applied layer transform, so nothing needs to be done
        LayerType::Mask => debug!("SKIPPING MASK FOR NOW"),
        LayerType::Other(other) => debug!("UNKNOWN LAYER TYPE: {other}"),
    }

    all_outputs.push(output);

    evaluate_descendant_layers(
        &layer.children, 
        root_motion, 
        resources, 
        overrides, 
        debug_info, 
        all_outputs, 
        layer_path, 
        layer_state, 
        time, 
        local_time, 
        false
    );
}

/// Get the animation command for an icon.
fn get_icon_command(
    resources: &MotionResources,
    layer: &Layer<ResourceReference>,
    layer_state: InterpolatedLayerState,
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
            let icon_info = resources.icon(icon).expect("icon should exist");
            // origin translation (equation comes from the krkr2 docs)
            // (or maybe just common sense)
            let origin_offset = glam::Vec2::new(
                -icon_info.value.origin_x - layer_state.anchor_offset.x,
                -icon_info.value.origin_y - layer_state.anchor_offset.y,
            );
            let mut transform = layer_state.transform * Affine2::from_translation(origin_offset);

            // screen translation -- basically "move" the icon to the centre of the screen
            let screen_translation = Affine2::from_translation(glam::Vec2 {
                x: resources.screen_size.width / 2.0,
                y: resources.screen_size.height / 2.0,
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
    resources: &MotionResources,
    layer: &Layer<ResourceReference>,
    layer_state: InterpolatedLayerState,
    layer_path: LayerPath,
    mut time: InheritedTime,
    local_time: MotionLocalTime,
    overrides: &ExternalOverrides,
    debug_info: &mut DebugInfo,
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
        let commands = get_motion_commands_inner(
            motion,
            resources,
            layer_path,
            layer_state.into_motion_parent_state(),
            time,
            overrides,
            debug_info,
        );
        return Some(commands);
    } else {
        return None;
    }
}

/// Gets the animation commands for a motion.
fn get_motion_commands_inner(
    motion: MotionKey,
    resources: &MotionResources,
    parent_path: LayerPath,
    parent_state: InterpolatedLayerState,
    time: InheritedTime,
    overrides: &ExternalOverrides,
    debug_info: &mut DebugInfo,
) -> Vec<AnimationCommand> {
    let motion = resources.motion(motion).expect("motion should exist");
    let mut motion_outputs = MotionLayerOutputs::new();
    let local_time = MotionLocalTime::calculate(time, motion.loop_time, motion.last_time);

    evaluate_descendant_layers(
        &motion.layer,
        motion,
        resources,
        overrides,
        debug_info,
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
/// appending their `LayerOutput`s into `all_outputs`.
fn evaluate_descendant_layers(
    layers: &[Layer<ResourceReference>],
    root_motion: &Motion<ResourceReference>,
    resources: &MotionResources,
    overrides: &ExternalOverrides,
    debug_info: &mut DebugInfo,
    all_outputs: &mut MotionLayerOutputs,
    parent_path: LayerPath,
    parent_state: InterpolatedLayerState,
    time: InheritedTime,
    local_time: MotionLocalTime,
    direct_from_motion: bool
) {
    for layer in layers {
        let layer_path = parent_path.child(&layer.label);
        let (layer_time, layer_local_time) = 
            get_layer_times_with_override(&layer_path, time, local_time, root_motion.loop_time, root_motion.last_time, overrides);
        let (layer, layer_local_time) =
            replace_layer_if_in_overrides(resources, layer, layer_local_time, layer_time, &layer_path, overrides);
        let local_state =
            get_local_state_with_override(layer_local_time, layer, &layer_path, overrides);
        let layer_state = parent_state.with_child(local_state);

        if direct_from_motion {
            debug_info.record_motion_layer(&layer_path, layer_state.transform);
        } else {
            debug_info.record_child_layer(&layer_path, layer_state.transform);
        }

        get_layer_commands(
            layer,
            root_motion,
            resources,
            all_outputs,
            overrides,
            debug_info,
            layer_state,
            layer_path,
            layer_local_time,
            layer_time,
        );
    }
}

/// Check if the given layer should be replaced with a different layer
/// if specified in `overrides`.
fn replace_layer_if_in_overrides<'a>(
    resources: &'a MotionResources,
    layer: &'a Layer<ResourceReference>,
    local_time: MotionLocalTime,
    time: InheritedTime,
    layer_path: &LayerPath,
    overrides: &ExternalOverrides,
) -> (&'a Layer<ResourceReference>, MotionLocalTime) {
    let Some(donor_path) = overrides.replacement_for(layer_path) else {
        trace!("no replement for layer: {layer_path:?}");
        return (layer, local_time);
    };
    let Some(donor_context) = resources.layer_context_at(donor_path) else {
        debug!("no layer for donor path: {donor_path:?}");
        return (layer, local_time);
    };
    let Some(donor_motion) = resources.motion(donor_context.motion) else {
        debug!("no motion found for donor path: {donor_path:?}");
        return (layer, local_time);
    };
    let donor_local_time =
        MotionLocalTime::calculate(time, donor_motion.loop_time, donor_motion.last_time);
    trace!("donor layer obtained for donor path: {donor_path:?}");

    (donor_context.layer, donor_local_time)
}

/// Gets a layer's interpolated state, applying an external override if present.
fn get_local_state_with_override(
    local_time: MotionLocalTime,
    layer: &Layer<ResourceReference>,
    layer_path: &LayerPath,
    overrides: &ExternalOverrides,
) -> InterpolatedLayerState {
    let mut state = get_local_linear_interpolated_state(local_time, &layer.frame_list);
    if let Some(override_state) = overrides.state_for(layer_path) {
        match override_state {
            LayerOverride::Completely(new_state) => {
                state.transform = new_state.transform;
                state.opacity = new_state.opa;
            },
            LayerOverride::Append(append_state) => {
                state.transform *= append_state.transform;
                state.opacity *= append_state.opa;
            }
        };
    }
    state
}

/// Return the layer's inherited and local times,
/// checking `overrides` if an override is applicable.
fn get_layer_times_with_override(
    layer_path: &LayerPath,
    layer_time: InheritedTime,
    layer_local_time: MotionLocalTime,
    motion_loop_time: i32,
    motion_last_time: u32,
    overrides: &ExternalOverrides
) -> (InheritedTime, MotionLocalTime) {
    if let Some(&override_time) = overrides.inherited_time_for(layer_path) {
        let override_local_time = MotionLocalTime::calculate(override_time, motion_loop_time, motion_last_time);
        return (override_time, override_local_time);
    }
    (layer_time, layer_local_time) 
}

/// Gets the active layer state. Interpolate only a dynamic frame toward the next frame,
/// otherwise just use the active frame.
fn get_local_linear_interpolated_state(
    local_time: MotionLocalTime,
    keyframes: &[KeyFrame<ResourceReference>],
) -> InterpolatedLayerState {
    // get current frame's content
    let Some(current_index) = active_frame_at(local_time, keyframes) else {
        return InterpolatedLayerState::identity();
    };
    let current = &keyframes[current_index];
    let Some(content) = current.content.as_ref() else {
        return InterpolatedLayerState::identity();
    };

    // try get the next frame
    let Some(next) = keyframes.get(current_index + 1) else {
        return content.linearly_interpolate(content, 0.0);
    };
    // if it has no content, use current frame's transform
    let Some(next_content) = next.content.as_ref() else {
        return content.linearly_interpolate(content, 0.0);
    };
    // if we're not supposed to dynamically transform, or have some degenerate time,
    // just use current frame's transform
    if current.kind != FrameType::ParentTransformsDynamically || next.time <= current.time {
        return content.linearly_interpolate(content, 0.0);
    }

    // linear interpolate
    // TODO: maybe we want other types of interpolation
    let duration = next.time - current.time;
    let progress = (u32::from(local_time) - current.time) as f32 / duration as f32;
    content.linearly_interpolate(next_content, progress)
}

/// Get the index of the active frame (the frame being interpolated away from).
///
/// Returns `None` if time is before the first frame,
/// or if there are no frames.
///
/// Does not consider empty final frames.
///
/// Returns the last frame if `time` is beyond the last frame's time.
fn active_frame_at(
    time: MotionLocalTime,
    mut keyframes: &[KeyFrame<ResourceReference>],
) -> Option<usize> {
    // NOTE: if the last frame is empty, we filter it out
    // as it messes with getting the correct active frame
    // and doesn't seem to serve any purpose (other than an "end" marker?).
    //
    // However, it's possible they have an actual purpose in certain situations.
    // Should keep an eye out for any bugs.
    if let Some(last_frame) = keyframes.last() {
        if last_frame.kind == FrameType::Empty {
            keyframes = &keyframes[0..keyframes.len() - 1];
        }
    }
    // if `time` is past the last frame's time, just return it
    // if there are no last keyframes, return None
    match keyframes.last() {
        Some(last_frame) => {
            if last_frame.time <= time.into() {
                return Some(keyframes.len() - 1);
            }
        }
        None => return None,
    }
    // search
    for (i, frame) in keyframes.iter().enumerate() {
        if frame.time > time.into() {
            if i == 0 {
                return None;
            } else {
                return Some(i - 1);
            }
        }
    }
    return None;
}

/// Basically the same thing as `active_frame_at`, except for the active priority in a frame's `priority` list.
fn active_priority_at(time: MotionLocalTime, mut priorities: &[PriorityFrame]) -> Option<usize> {
    // again, if the last frame is empty, filter it out
    if let Some(priority) = priorities.last() {
        if priority.content.is_none() {
            priorities = &priorities[0..priorities.len() - 1];
        }
    }
    // if past the last time, use the last frame
    match priorities.last() {
        Some(last_frame) => {
            if last_frame.time <= time.into() {
                return Some(priorities.len() - 1);
            }
        }
        None => return None,
    }
    // else search
    for (i, frame) in priorities.iter().enumerate() {
        if frame.time > time.into() {
            if i == 0 {
                return None;
            } else {
                return Some(i - 1);
            }
        }
    }
    return None;
}
