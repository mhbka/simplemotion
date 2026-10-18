use crate::{
    animation::{AnimationCommand, InterpolatedLayerState},
    animator::time::{InheritedTime, MotionLocalTime},
    FrameType, KeyFrame, Layer, LayerType, MotionKey, MotionResources, ResourceReference,
};
use glam::Affine2;
use tracing::{debug, trace, warn};

mod debug;
mod ext_override;
mod path;
mod time;

pub use debug::DebugInfo;
pub use ext_override::{ExternalOverrides, OverrideLayerState};
pub use path::LayerPath;

/// Animates `motion` at the given `time`, outputting animation commands into `buffer`.
///
/// This is the animation entrypoint, where `motion` is typically one of the root motions.
pub fn to_anim_commands(
    buffer: &mut Vec<AnimationCommand>,
    resources: &MotionResources,
    motion: MotionKey,
    time: u32,
    debug_info: &mut DebugInfo,
) {
    to_anim_commands_with_overrides(
        buffer,
        resources,
        motion,
        time,
        &ExternalOverrides::default(),
        debug_info,
    );
}

/// Animates `motion` with external layer-state replacements.
pub fn to_anim_commands_with_overrides(
    buffer: &mut Vec<AnimationCommand>,
    resources: &MotionResources,
    motion: MotionKey,
    time: u32,
    overrides: &ExternalOverrides,
    debug_info: &mut DebugInfo,
) {
    buffer.clear();
    debug_info.clear();
    let root_label = resources
        .motion_label(motion)
        .map(String::as_str)
        .unwrap_or("<unknown>");
    get_motion_commands_inner(
        buffer,
        motion,
        resources,
        LayerPath::root(root_label),
        InterpolatedLayerState::identity(),
        time.into(),
        overrides,
        debug_info,
    );
}

/// Get the commands for a layer, then its children.
fn get_layer_commands(
    layer: &Layer<ResourceReference>,
    resources: &MotionResources,
    buffer: &mut Vec<AnimationCommand>,
    layer_state: InterpolatedLayerState,
    layer_path: LayerPath,
    local_time: MotionLocalTime,
    time: InheritedTime,
    overrides: &ExternalOverrides,
    debug_info: &mut DebugInfo,
) {
    match layer.layer_type {
        LayerType::Icon => get_icon_command(resources, buffer, layer, layer_state, local_time),
        LayerType::Motion => get_motion_commands(
            buffer,
            resources,
            layer,
            layer_state,
            layer_path.clone(),
            time,
            local_time,
            overrides,
            debug_info,
        ),
        LayerType::Layout => {} // already applied layer transform, so nothing needs to be done
        LayerType::Mask => debug!("SKIPPING MASK FOR NOW"),
        LayerType::Other(other) => debug!("UNKNOWN LAYER TYPE: {other}"),
    }
    for child_layer in &layer.children {
        let child_layer_path = layer_path.child(&child_layer.label);
        let (child_layer, child_local_time) = check_layer_to_evaluate(
            resources,
            child_layer,
            local_time,
            time,
            &child_layer_path,
            overrides,
        );
        let child_local_state =
            get_local_overridden_state(child_local_time, child_layer, &child_layer_path, overrides);
        let child_layer_state = layer_state.with_child(child_local_state);

        debug_info.record_child_layer(&child_layer_path, child_layer_state.transform);

        get_layer_commands(
            child_layer,
            resources,
            buffer,
            child_layer_state,
            child_layer_path,
            child_local_time,
            time,
            overrides,
            debug_info,
        );
    }
}

/// Get the animation command for an icon.
fn get_icon_command(
    resources: &MotionResources,
    buffer: &mut Vec<AnimationCommand>,
    layer: &Layer<ResourceReference>,
    layer_state: InterpolatedLayerState,
    local_time: MotionLocalTime,
) {
    let Some(active_keyframe_index) = active_frame_at(local_time, &layer.frame_list) else {
        return;
    };
    let active_keyframe = &layer.frame_list[active_keyframe_index];
    if let Some(content) = &active_keyframe.content {
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
        buffer.push(command);
    }
}

/// Get the animation commands for a motion.
fn get_motion_commands(
    buffer: &mut Vec<AnimationCommand>,
    resources: &MotionResources,
    layer: &Layer<ResourceReference>,
    layer_state: InterpolatedLayerState,
    layer_path: LayerPath,
    mut time: InheritedTime,
    local_time: MotionLocalTime,
    overrides: &ExternalOverrides,
    debug_info: &mut DebugInfo,
) {
    let Some(active_keyframe_index) = active_frame_at(local_time, &layer.frame_list) else {
        return;
    };
    let active_keyframe = &layer.frame_list[active_keyframe_index];
    if let Some(content) = &active_keyframe.content {
        let ResourceReference::Motion(motion) = content.src else {
            debug!("Motion layer has non-motion keyframe, not animating: {active_keyframe:?}");
            return;
        };
        time = time.inherit(
            active_keyframe.time,
            content
                .motion
                .as_ref()
                .and_then(|motion| motion.time_offset),
        );
        get_motion_commands_inner(
            buffer,
            motion,
            resources,
            layer_path,
            layer_state.into_motion_parent_state(),
            time,
            overrides,
            debug_info,
        );
    }
}

/// Gets the animation commands for a motion.
fn get_motion_commands_inner(
    buffer: &mut Vec<AnimationCommand>,
    motion: MotionKey,
    resources: &MotionResources,
    parent_path: LayerPath,
    parent_state: InterpolatedLayerState,
    time: InheritedTime,
    overrides: &ExternalOverrides,
    debug_info: &mut DebugInfo,
) {
    let motion = resources.motion(motion).expect("motion should exist");
    let local_time = MotionLocalTime::calculate(time, motion.loop_time, motion.last_time);
    for layer in &motion.layer {
        let layer_path = parent_path.child(&layer.label);
        let (layer, layer_local_time) =
            check_layer_to_evaluate(resources, layer, local_time, time, &layer_path, overrides);
        let local_state =
            get_local_overridden_state(layer_local_time, layer, &layer_path, overrides);
        let layer_state = parent_state.with_child(local_state);

        debug_info.record_motion_layer(&layer_path, layer_state.transform);

        get_layer_commands(
            layer,
            resources,
            buffer,
            layer_state,
            layer_path,
            layer_local_time,
            time,
            overrides,
            debug_info,
        );
    }
}

/// Check if the layer to evaluate should be replaced with a different layer.
fn check_layer_to_evaluate<'a>(
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

/// Gets a layer's interpolated state, applying an external replacement when present.
fn get_local_overridden_state(
    local_time: MotionLocalTime,
    layer: &Layer<ResourceReference>,
    layer_path: &LayerPath,
    overrides: &ExternalOverrides,
) -> InterpolatedLayerState {
    let mut state = get_local_linear_interpolated_state(local_time, &layer.frame_list);
    if let Some(override_state) = overrides.state_for(layer_path) {
        state.transform = override_state.transform;
        state.opacity = override_state.opa;
    }
    state
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
