use tessera_ui::{ComputeCommand, Px, renderer::command::BarrierRequirement};

/// A synchronous command to execute a gaussian blur.
/// `BlurCommand` describes a single directional blur pass.
#[derive(Debug, Clone, PartialEq)]
pub struct BlurCommand {
    /// Blur radius.
    pub radius: f32,
    /// Blur direction: (1.0, 0.0) for horizontal, (0.0, 1.0) for vertical.
    pub direction: (f32, f32),
}

impl BlurCommand {
    /// Convenience helper for building a horizontal blur pass.
    pub fn horizontal(radius: f32) -> Self {
        Self {
            radius,
            direction: (1.0, 0.0),
        }
    }

    /// Convenience helper for building a vertical blur pass.
    pub fn vertical(radius: f32) -> Self {
        Self {
            radius,
            direction: (0.0, 1.0),
        }
    }
}

/// A compute command that runs two directional blur passes (typically horizontal + vertical)
/// within a single dispatch sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct DualBlurCommand {
    pub passes: [BlurCommand; 2],
}

impl DualBlurCommand {
    pub fn new(passes: [BlurCommand; 2]) -> Self {
        Self { passes }
    }

    /// Creates a dual blur command with horizontal and vertical passes using the same radius/padding.
    pub fn horizontal_then_vertical(radius: f32) -> Self {
        Self {
            passes: [
                BlurCommand::horizontal(radius),
                BlurCommand::vertical(radius),
            ],
        }
    }
}

impl ComputeCommand for DualBlurCommand {
    fn barrier(&self) -> BarrierRequirement {
        BarrierRequirement::uniform_padding_local(Px(10))
    }
}
