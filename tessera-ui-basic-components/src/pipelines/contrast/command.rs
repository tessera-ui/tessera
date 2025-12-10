use tessera_ui::{
    BarrierRequirement, compute::ComputeResourceRef, renderer::compute::command::ComputeCommand,
};

/// Command to apply a contrast adjustment using a pre-calculated mean
/// luminance.
///
/// # Parameters
///
/// - `contrast`: The contrast adjustment factor.
/// - `mean_result_handle`: Handle to the buffer containing mean luminance data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastCommand {
    /// The contrast adjustment factor.
    pub contrast: f32,
    /// A handle to the `wgpu::Buffer` containing the mean luminance data.
    pub mean_result_handle: ComputeResourceRef,
}

impl ContrastCommand {
    /// Creates a new `ContrastCommand`.
    ///
    /// # Parameters
    ///
    /// - `contrast`: The contrast adjustment factor.
    /// - `mean_result_handle`: Handle to the buffer containing mean luminance
    ///   data.
    pub fn new(contrast: f32, mean_result_handle: ComputeResourceRef) -> Self {
        Self {
            contrast,
            mean_result_handle,
        }
    }
}

impl ComputeCommand for ContrastCommand {
    fn barrier(&self) -> tessera_ui::BarrierRequirement {
        BarrierRequirement::ZERO_PADDING_LOCAL
    }
}
