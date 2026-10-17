use crate::{
    animation::AnimationCommand,
    animator::time::{InheritedTime, MotionLocalTime},
    FrameType, KeyFrame, Layer, LayerType, MotionKey, MotionResources, ResourceReference,
};
use glam::Affine2;
use tracing::debug;

mod time;

/// Animates `motion` at the given `time`, outputting animation commands into `buffer`.
///
/// This is the animation entrypoint, where `motion` is typically one of the root motions.
pub fn to_anim_commands(
    buffer: &mut Vec<AnimationCommand>,
    resources: &MotionResources,
    motion: MotionKey,
    time: u32,
) {
    buffer.clear();
    get_motion_commands_inner(buffer, motion, resources, Affine2::IDENTITY, time.into());
}

/// Get the commands for a layer, then its children.
fn get_layer_commands(
    layer: &Layer<ResourceReference>,
    resources: &MotionResources,
    buffer: &mut Vec<AnimationCommand>,
    layer_transform: Affine2,
    local_time: MotionLocalTime,
    time: InheritedTime,
) {
    match layer.layer_type {
        LayerType::Icon => get_icon_command(buffer, layer, layer_transform, local_time),
        LayerType::Layout => {
            // we already apply layer transform, so nothing needs to be done
        }
        LayerType::Motion => {
            get_motion_commands(buffer, resources, layer, layer_transform, time, local_time)
        }
        LayerType::Mask => debug!("SKIPPING MASK FOR NOW"),
        LayerType::Other(other) => debug!("UNKNOWN LAYER TYPE: {other}"),
    }
    for child_layer in &layer.children {
        get_layer_commands(
            child_layer,
            resources,
            buffer,
            layer_transform,
            local_time,
            time,
        );
    }
}

/// Get the animation command for an icon.
fn get_icon_command(
    buffer: &mut Vec<AnimationCommand>,
    layer: &Layer<ResourceReference>,
    transform: Affine2,
    local_time: MotionLocalTime,
) {
    let Some(active_keyframe_index) = active_frame_at(local_time, &layer.frame_list) else {
        // don't animate anything if we don't have an active frame (should only be if we're before the first frame)
        return;
    };
    let active_keyframe = &layer.frame_list[active_keyframe_index];
    if let Some(content) = &active_keyframe.content {
        let ResourceReference::Icon(icon) = content.src else {
            panic!("Icon layer contains frame with non-icon source: {keyframe:?}");
        };
        let opacity = if let Some(opa) = content.opa {
            opa
        } else {
            1.0
        };
        let command = AnimationCommand {
            icon,
            transform,
            opacity,
        };
        buffer.push(command);
    }
}

/// Get the animation commands for a motion.
///
/// A motion loops on its own local time, not related to any parent motion.
///
/// TODO: is a motion's local time offset by the time it starts in its frame?
/// if so i need to add that logic here
fn get_motion_commands(
    buffer: &mut Vec<AnimationCommand>,
    resources: &MotionResources,
    layer: &Layer<ResourceReference>,
    parent_transform: Affine2,
    mut time: InheritedTime,
    local_time: MotionLocalTime,
) {
    let Some(active_keyframe_index) = active_frame_at(local_time, &layer.frame_list) else {
        return;
    };
    let active_keyframe = &layer.frame_list[active_keyframe_index];
    if let Some(content) = &active_keyframe.content {
        let ResourceReference::Motion(motion) = content.src else {
            panic!("Motion layer has non-motion keyframe: {active_keyframe:?}");
        };
        time = time.inherit(
            active_keyframe.time,
            content
                .motion
                .as_ref()
                .and_then(|motion| motion.time_offset),
        );
        get_motion_commands_inner(buffer, motion, resources, parent_transform, time);
    }
}

/// Gets the animation commands for a motion.
fn get_motion_commands_inner(
    buffer: &mut Vec<AnimationCommand>,
    motion: MotionKey,
    resources: &MotionResources,
    parent_transform: Affine2,
    time: InheritedTime,
) {
    let motion = resources
        .motion(motion)
        .expect("motion should exist");
    let local_time = MotionLocalTime::calculate(time, motion.loop_time, motion.last_time);
    for layer in &motion.layer {
        let local_transform =
            get_local_linear_interpolated_transform(local_time, &layer.frame_list);
        let layer_transform = parent_transform * local_transform;
        get_layer_commands(layer, resources, buffer, layer_transform, local_time, time);
    }
}

/// Gets the active layer transform, interpolating/blending only a dynamic frame toward the next frame.
fn get_local_linear_interpolated_transform(
    local_time: MotionLocalTime,
    keyframes: &[KeyFrame<ResourceReference>],
) -> Affine2 {
    // get current frame's content
    let Some(current_index) = active_frame_at(local_time, keyframes) else {
        return Affine2::IDENTITY;
    };
    let current = &keyframes[current_index];
    let Some(content) = current.content.as_ref() else {
        return Affine2::IDENTITY;
    };

    // try get the next frame
    let Some(next) = keyframes.get(current_index + 1) else {
        return content.linearly_interpolate_to_affine2(content, 0.0);
    };
    // if it has no content, just return current frame's transform
    let Some(next_content) = next.content.as_ref() else {
        return content.linearly_interpolate_to_affine2(content, 0.0);
    };
    // if we're not supposed to dynamically transform, or have some degenerate time,
    // just use current frame's transform
    if current.kind != FrameType::ParentTransformsDynamically || next.time <= current.time {
        return content.linearly_interpolate_to_affine2(content, 0.0);
    }

    // linear interpolate
    // TODO: maybe we want other types of interpolation
    let duration = next.time - current.time;
    let progress = (u32::from(local_time) - current.time) as f32 / duration as f32;
    content.linearly_interpolate_to_affine2(next_content, progress)
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
