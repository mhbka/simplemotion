/// An inherited time. This time has been offset by its ancestor motions' frame (start) times and `timeOffset`s.
///
/// ### Example
/// The active motion frame has `time` 10 and `motion.timeOffset` 5.
///
/// The global time is 15. Say the motion frame has no ancestors (ie is a root), so the inherited time is 15.
///
/// Since the frame starts at 10, the frame's children are relative to 10, so to them, the global time is 15 - 10 = 5.
///
/// Then, `timeOffset` offsets the time forward, so the final inherited time is 5 + 5 = 10.
/// This is then the "global" time for grandchildren motions.
///
/// ### Typical usage of `timeOffset`
/// For examples I've seen, `timeOffset` is used to offset `time` completely, making child motions run on the same time as their parent.
///
/// For eg, if the frame's `time` is 10 and `motion.timeOffset` is 10,
/// the inherited time at 15 is 15 - 10 + 10 = 15.
///
/// An implication is, if you didn't have a motion frame before, the child motion will only run at time 10 onwards,
/// and you'll never see it from time 0 to 9.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InheritedTime(u32);

impl InheritedTime {
    /// Calculate the next inherited time from the current one `self`.
    pub fn inherit(self, frame_time: u32, time_offset: Option<u32>) -> Self {
        let offset = if let Some(offset) = time_offset {
            offset
        } else {
            0
        };
        // in case our offset pushes us under 0
        let inherited_time = self.0 as i32 - frame_time as i32 + offset as i32;
        if inherited_time < 0 {
            panic!(
                "got an inherited time less than 0! (parent time: {}, frame time: {}, offset: {:?})",
                self.0,
                frame_time,
                time_offset
            );
        }
        Self(inherited_time as u32)
    }
}

impl From<u32> for InheritedTime {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<InheritedTime> for u32 {
    fn from(value: InheritedTime) -> Self {
        value.0
    }
}

/// A newtype for "loop-local" time (local to a motion). This can be directly used to calculate the active keyframe of a layer.
///
/// This can be directly passed down descendant layers.
/// Only when entering a new Motion should this be recalculated.
///
/// Basically, it's recalculated at the same time as `InheritedTime`.
///
/// ### Example
/// We have a loop from 0 -> 90, with `loopTime` 30 (meaning loops start at 30 instead of 0).
///
/// If the time is 100, the local time is 40 (first loop of 90 -> 10 left, start at 30 -> end up at 40).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionLocalTime(u32);

impl MotionLocalTime {
    /// Get the local time, given the `loop_time` (time to start loops at) and `last_time` (time the loop ends at).
    pub fn calculate(time: InheritedTime, loop_time: i32, last_time: u32) -> Self {
        if loop_time != -1 && (loop_time < 0 || loop_time as u32 >= last_time) {
            panic!("`last_time` must be after `loop_time`");
        }
        // have not passed last_time, so just return time
        if time.0 <= last_time {
            return Self(time.0);
        }
        // if loop_time is -1 (or just negative in general?), no looping -- just return last_time
        if loop_time < 0 {
            return Self(last_time);
        }
        // looping and we passed last_time, so start looping at loop_time
        let time_in_loop = time.0 - last_time;
        let loop_start = loop_time as u32;
        let loop_period = last_time - loop_start;
        let in_loop_time = time_in_loop % loop_period;
        let local_time = loop_start + in_loop_time;
        Self(local_time)
    }

    /// Get the time.
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl From<MotionLocalTime> for u32 {
    fn from(value: MotionLocalTime) -> Self {
        value.0
    }
}
