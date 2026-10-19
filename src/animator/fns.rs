use crate::{
    ExternalOverrides, FrameType, KeyFrame, Layer, LayerPath, MotionResources, PriorityFrame, ResourceReference, animation::InterpolatedLayerState, animator::{
        ext_override::LayerOverride, time::{InheritedTime, MotionLocalTime},
    },
};
use tracing::{debug, trace};

/// Check if the given layer should be replaced with a different layer
/// if specified in `overrides`.
pub(crate) fn replace_layer_if_in_overrides<'a>(
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
pub(crate) fn get_local_state_with_override(
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
pub(crate) fn get_layer_times_with_override(
    layer_path: &LayerPath,
    layer_time: InheritedTime,
    layer_local_time: MotionLocalTime,
    motion_loop_time: i32,
    motion_last_time: u32,
    overrides: &ExternalOverrides
) -> (InheritedTime, MotionLocalTime) {
    if let Some(&override_time) = overrides.inherited_time_for(layer_path) {
        let override_local_time = MotionLocalTime::calculate(override_time, motion_loop_time, motion_last_time);
        return (layer_time, override_local_time);
    }
    (layer_time, layer_local_time) 
}

/// Gets the active layer state. Interpolate only a dynamic frame toward the next frame,
/// otherwise just use the active frame.
pub(crate) fn get_local_linear_interpolated_state(
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
pub(crate) fn active_frame_at(
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
pub(crate) fn active_priority_at(time: MotionLocalTime, mut priorities: &[PriorityFrame]) -> Option<usize> {
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
