use crate::{
    FrameType, KeyFrame, PriorityFrame, ResourceReference, animation::LayerState,
    animator::time::MotionLocalTime,
};

/// Gets the active layer state. Interpolate only a dynamic frame toward the next frame,
/// otherwise just use the active frame.
pub(crate) fn get_local_linear_interpolated_state(
    local_time: MotionLocalTime,
    keyframes: &[KeyFrame<ResourceReference>],
) -> LayerState {
    // get current frame's content
    let Some(current_index) = active_frame_at(local_time, keyframes) else {
        return LayerState::identity();
    };
    let current = &keyframes[current_index];
    let Some(content) = current.content.as_ref() else {
        return LayerState::identity();
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
