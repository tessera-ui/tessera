use tessera_ui::{SampleRegion, renderer::compute::command::ComputeCommand};

/// Command that scales colors away from the mean luminance of its area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastCommand {
    /// The contrast adjustment factor.
    pub contrast: f32,
}

impl ContrastCommand {
    /// Creates a contrast adjustment with the given factor.
    pub fn new(contrast: f32) -> Self {
        Self { contrast }
    }
}

impl ComputeCommand for ContrastCommand {
    fn barrier(&self) -> SampleRegion {
        SampleRegion::ZERO_PADDING_LOCAL
    }
}
