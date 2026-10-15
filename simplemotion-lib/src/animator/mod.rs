use glam::{Affine2, Vec2};

use crate::{
    FrameType, KeyFrame, Layer, LayerType, MotionKey, MotionResources, ResourceReference, animation::AnimationCommand,
};

/// Animates `motion` at the given `time`, outputting animation commands into `buffer`.
/// 
/// This is the animation entrypoint, where `motion` is typically one of the root motions.
pub fn animation_commands(
    buffer: &mut Vec<AnimationCommand>,
    resources: &MotionResources,
    motion: MotionKey,
    time: u32,
) {
    buffer.clear();
    get_motion_commands_inner(
        buffer,
        motion, 
        resources, 
        Affine2::IDENTITY, 
        time
    );
}   

/// Get the commands for a layer, then its children.
fn get_layer_commands(
    layer: &Layer<ResourceReference>, 
    resources: &MotionResources, 
    buffer: &mut Vec<AnimationCommand>, 
    mut layer_transform: Affine2,
    motion_time: u32,
    time: u32
) {
    match layer.layer_type {
        LayerType::Icon => get_icon_command(
            buffer, 
            layer, 
            layer_transform, 
            motion_time
        ),
        LayerType::Layout => layer_transform *= get_layout_transform(layer),
        LayerType::Motion => get_motion_commands(
            buffer, 
            resources, 
            layer, 
            layer_transform, 
            time
        ),
        LayerType::Mask => println!("SKIPPING MASK FOR NOW"),
        LayerType::Other(other) => println!("UNKNOWN LAYER TYPE: {other}"),
    }
    for child_layer in &layer.children {
        get_layer_commands(
            child_layer, 
            resources, 
            buffer, 
            layer_transform, 
            motion_time,
            time
        );
    }
}

/// Get the animation command for an icon.
fn get_icon_command(
    buffer: &mut Vec<AnimationCommand>,
    layer: &Layer<ResourceReference>,
    layer_transform: Affine2,
    motion_time: u32,
) {
    for keyframe in &layer.frame_list {
        if let Some(content) = &keyframe.content {
            let ResourceReference::Icon(icon) = content.src else { 
                panic!("Icon layer contains frame with non-icon source: {keyframe:?}"); 
            };
            let local_transform = get_local_linear_interpolated_transform(motion_time, &layer.frame_list);
            let final_transform = layer_transform * local_transform;
            let command = AnimationCommand {
                icon,
                transform: final_transform
            };
            buffer.push(command);
        }
    }
}

/// Get the animation commands for a motion.
/// 
/// A motion loops on its own local time, not related to any parent motion.
/// 
/// However, it is offset by the time it starts at in its frame.
fn get_motion_commands(
    buffer: &mut Vec<AnimationCommand>,
    resources: &MotionResources,
    layer: &Layer<ResourceReference>,
    parent_transform: Affine2,
    time: u32
) {
    for keyframe in &layer.frame_list {
        if let Some(content) = &keyframe.content {
            let ResourceReference::Motion(motion) = content.src else {
                panic!("Motion layer has non-motion keyframe: {keyframe:?}");
            };

            // TODO: FOR SURE motion needs more from its frame's content than just the src reference
            get_motion_commands_inner(
                buffer, 
                motion, 
                resources, 
                parent_transform, 
                time
            );
        }
    }
}

/// Gets the animation commands for a motion.
fn get_motion_commands_inner(
    buffer: &mut Vec<AnimationCommand>, 
    motion: MotionKey,
    resources: &MotionResources, 
    parent_transform: Affine2,
    time: u32
) {
    let motion = resources.motion(motion).expect("motion should exist");

    // a motion gets its own motion-local time, not related to any parent motions
    let motion_time = get_local_time(time, motion.loop_time, motion.last_time);

    for layer in &motion.layer {
        let local_transform = get_local_linear_interpolated_transform(motion_time, &layer.frame_list);
        let layer_transform = parent_transform * local_transform;
        get_layer_commands(
            layer, 
            resources, 
            buffer, 
            layer_transform, 
            motion_time,
            time
        );
    }
}

/// Obtain an additional transform for the layer.
/// 
/// ## NOTE
/// Currently I also impose these invariants and panic otherwise (just for sanity):
/// - There's exactly 1 layout frame (the 1st one), which has non-empty `content`
/// - This content's only transformation is `coord` and `mask` (idk what `mask` does)
/// - There can be a 2nd empty frame, or none at all (currently there's always an empty frame but idc)
/// - There are no more frames
fn get_layout_transform(layer: &Layer<ResourceReference>) -> Affine2 {
    if layer.frame_list.len() > 2 {
        panic!("more than 2 frames found in layout layer: {:?}", layer.frame_list);
    }
    let layout_frame = &layer.frame_list[0];
    let Some(content) = layout_frame.content.as_ref() else {
        panic!("layout frame contains no content (layer: {})", layer.label);
    };

    if content.ox.is_some() || content.oy.is_some()
    || content.angle.is_some() || content.zx.is_some()
    || content.zy.is_some() || content.sx.is_some()
    || content.sy.is_some() || content.opa.is_some() {
        panic!("layout frame content contains non-coord stuff: {:?} (layer: {})", content, layer.label);
    }
    let Some(coord) = content.coord else {
        panic!("layout frame contains no coord (layer: {})", layer.label);
    };
    
    let coord = Vec2::new(coord[0], coord[1]);
    let mut transform = Affine2::IDENTITY;
    transform.translation = coord;
    transform
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
    // try get the keyframe before the local time
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
