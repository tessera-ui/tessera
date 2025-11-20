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
    /// Ordered blur passes to execute.
    pub passes: [BlurCommand; 2],
}

/// Choose a downscale factor that balances quality and performance for a blur radius.
pub fn downscale_factor_for_radius(radius: f32) -> u32 {
    if radius <= 6.0 {
        1
    } else if radius <= 18.0 {
        2
    } else {
        4
    }
}

impl DualBlurCommand {
    /// Creates a dual-pass blur command with explicit passes.
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
        // Calculate maximum radius from both passes to determine required padding
        // The barrier padding must be at least as large as the blur radius to ensure
        // all pixels needed for the blur are available in the captured background
        let max_radius = self
            .passes
            .iter()
            .map(|pass| pass.radius)
            .fold(0.0f32, f32::max);

        let downscale = downscale_factor_for_radius(max_radius) as f32;
        // When downsampling, each texel covers a larger source region, so extend
        // the barrier padding proportionally to the chosen downscale factor.
        let sampling_padding = (max_radius * downscale).ceil() as i32;

        // The sampling padding is the actual padding needed for the blur effect.
        // The renderer still relies on the component bounds for dependency checks,
        // so orthogonal blur components can batch even if their sampling regions overlap.
        BarrierRequirement::uniform_padding_local(Px(sampling_padding))
    }
}
