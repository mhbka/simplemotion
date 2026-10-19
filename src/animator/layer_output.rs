use crate::{animation::AnimationCommand, PriorityFrame};
use tracing::debug;

/// All the layer outputs from a motion.
///
/// Create a new one when you start evaluating a motion.
pub struct MotionLayerOutputs {
    outputs: Vec<LayerOutput>,
}

impl MotionLayerOutputs {
    /// Create a new one b-baka!
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
        }
    }

    /// Add a layer output.
    ///
    /// Should be depth-first and child-layer ordering.
    pub fn push(&mut self, output: LayerOutput) {
        self.outputs.push(output);
    }

    /// Finalize all the layer outputs into commands in the correct order.
    pub fn finalize(self, motion_priority: Option<&PriorityFrame>) -> Vec<AnimationCommand> {
        let Some(priority) = motion_priority else {
            return self.finalize_without_reorder();
        };
        let Some(content) = &priority.content else {
            return self.finalize_without_reorder();
        };
        if content.len() != self.outputs.len() {
            debug!(
                "priority length ({}) isn't same as output length ({})",
                content.len(),
                self.outputs.len()
            );
            return self.finalize_without_reorder();
        }
        self.reorder_and_finalize(content)
    }

    /// Reorders layer outputs by their positions in the priority list (in reverse cuz you draw that way).
    fn reorder_and_finalize(self, order: &[usize]) -> Vec<AnimationCommand> {
        let mut outputs: Vec<_> = self.outputs.into_iter().map(Some).collect();
        order
            .iter()
            .rev()
            .filter_map(|index| outputs[*index].take())
            .flat_map(|output| output.combine())
            .collect()
    }

    /// Finalize our layers in the current order.
    fn finalize_without_reorder(self) -> Vec<AnimationCommand> {
        self.outputs
            .into_iter()
            .map(|output| output.combine())
            .flatten()
            .collect()
    }
}

/// The output commands for a layer.
pub struct LayerOutput {
    /// If this layer is a motion layer, its commands.
    motion: Option<Vec<AnimationCommand>>,
    /// If this layer is an icon layer, the animation command.
    own: Option<AnimationCommand>,
}

impl LayerOutput {
    /// Create an output b-baka!
    pub fn new() -> Self {
        Self {
            motion: None,
            own: None,
        }
    }

    /// Set the motion commands for this layer.
    ///
    /// Panics if set >1 times, as each layer should have max 1 set of motion commands.
    pub fn set_motion_commands(&mut self, commands: Vec<AnimationCommand>) {
        if self.motion.is_some() {
            panic!("a layer's motion commands was set twice");
        }
        self.motion = Some(commands);
    }

    /// Append the layer's own command.
    ///
    /// Panics if set >1 times, as each layer should have max 1 set of motion commands.
    pub fn set_own_command(&mut self, command: AnimationCommand) {
        if self.own.is_some() {
            panic!("a layer's own command was set twice");
        }
        self.own = Some(command);
    }

    /// Return the `LayerOutput` as 1 set of ordered commands.
    pub fn combine(self) -> Vec<AnimationCommand> {
        if let Some(mut motion) = self.motion {
            if let Some(own) = self.own {
                motion.push(own);
            }
            return motion;
        } else {
            match self.own {
                Some(own) => vec![own],
                None => vec![],
            }
        }
    }
}
