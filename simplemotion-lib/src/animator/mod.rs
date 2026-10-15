use glam::Affine2;

use crate::{
    animation::AnimationCommand, FrameType, KeyFrame, LayerType, MotionKey, MotionResources,
    ResourceReference,
};

/// Animates `motion` at the given `time`, outputting animation commands into `buffer`.
pub fn animation_commands(
    buffer: &mut Vec<AnimationCommand>,
    resources: &MotionResources,
    motion: MotionKey,
    time: u32,
) {
    buffer.clear();

    let motion = resources.motion(motion).expect("motion should exist");
    let motion_time = get_local_time(time, motion.loop_time, motion.last_time);

    for layer in &motion.layer {
        let _local_transform =
            get_local_linear_interpolated_transform(motion_time, &layer.frame_list);

        match layer.layer_type {
            LayerType::Icon => todo!(),
            LayerType::Layout => todo!(),
            LayerType::Motion => todo!(),
            LayerType::Mask => println!("SKIPPING MASK FOR NOW"),
            LayerType::Other(other) => println!("UNKNOWN LAYER TYPE: {other}"),
        }
    }
}

/// Obtains the time relative to the motion.
///
/// Returns `last_time` if we've passed `last_time` and the motion shouldn't loop.
fn get_local_time(time: u32, loop_time: i32, last_time: u32) -> u32 {
    if loop_time != -1 && (loop_time < 0 || loop_time as u32 >= last_time) {
        panic!("`last_time` must be after `loop_time`");
    }
    // have not passed last_time, so just return time
    if time <= last_time {
        return time;
    }
    // if loop_time is -1 (or just negative in general?), no looping -- just return last_time
    if loop_time < 0 {
        return last_time;
    }
    // looping and we passed last_time, so start looping at loop_time
    let time_in_loop = time - last_time;
    let loop_start = loop_time as u32;
    let loop_period = last_time - loop_start;
    let in_loop_time = time_in_loop % loop_period;
    let local_time = loop_start + in_loop_time;
    local_time
}

/// Gets the active layer transform, interpolating/blending only a dynamic frame toward the next frame.
fn get_local_linear_interpolated_transform(
    local_time: u32,
    keyframes: &[KeyFrame<ResourceReference>],
) -> Affine2 {
    // try get the "left" keyframe
    let Some(current_index) = keyframes
        .iter()
        .rposition(|frame| frame.time <= local_time) 
    else {
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
    let progress = (local_time - current.time) as f32 / duration as f32;
    content.linearly_interpolate_to_affine2(next_content, progress)
}
