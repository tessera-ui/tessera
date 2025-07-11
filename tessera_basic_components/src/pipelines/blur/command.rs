use tessera::{BarrierRequirement, ComputeCommand};

/// A synchronous command to execute a gaussian blur.
/// BlurCommand only describes blur parameters
pub struct BlurCommand {
    /// Blur radius.
    pub radius: f32,
    /// Blur direction: (1.0, 0.0) for horizontal, (0.0, 1.0) for vertical.
    pub direction: (f32, f32),
    /// Whether this is the first pass of the blur.
    /// If true, the command will become a barrier command,
    /// and the output texture will be cleared before the command is executed.
    /// If false, the command will not clear the output texture,
    /// and will use the previous texture as input.
    /// This is useful for multi-pass blurs,
    /// since blur always requires two passes.
    pub first_pass: bool,
}

impl ComputeCommand for BlurCommand {
    fn barrier(&self) -> Option<tessera::BarrierRequirement> {
        if self.first_pass {
            Some(BarrierRequirement::SampleBackground)
        } else {
            None
        }
    }
}
