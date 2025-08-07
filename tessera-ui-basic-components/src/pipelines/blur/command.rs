use tessera_ui::{ComputeCommand, renderer::command::BarrierRequirement};

/// A synchronous command to execute a gaussian blur.
/// BlurCommand only describes blur parameters
pub struct BlurCommand {
    /// Blur radius.
    pub radius: f32,
    /// Blur direction: (1.0, 0.0) for horizontal, (0.0, 1.0) for vertical.
    pub direction: (f32, f32),
}

impl ComputeCommand for BlurCommand {
    fn barrier(&self) -> BarrierRequirement {
        BarrierRequirement::ZERO_PADDING_LOCAL
    }
}
