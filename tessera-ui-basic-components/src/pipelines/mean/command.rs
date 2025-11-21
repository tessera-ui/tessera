use tessera_ui::{
    BarrierRequirement,
    compute::{ComputeResourceRef, resource::ComputeResourceManager},
    renderer::compute::command::ComputeCommand,
    wgpu,
};

/// Command to calculate the mean luminance of the input texture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeanCommand {
    result_buffer_ref: ComputeResourceRef,
}

impl MeanCommand {
    /// Creates a new `MeanCommand` and allocates a result buffer.
    ///
    /// # Parameters
    /// - `gpu`: The wgpu device.
    /// - `compute_resource_manager`: Resource manager for compute buffers.
    pub fn new(gpu: &wgpu::Device, compute_resource_manager: &mut ComputeResourceManager) -> Self {
        let result_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mean Result Buffer"),
            size: 8,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let result_buffer_ref = compute_resource_manager.push(result_buffer);
        Self { result_buffer_ref }
    }

    /// Returns the reference to the result buffer.
    pub fn result_buffer_ref(&self) -> ComputeResourceRef {
        self.result_buffer_ref
    }
}

impl ComputeCommand for MeanCommand {
    fn barrier(&self) -> BarrierRequirement {
        BarrierRequirement::ZERO_PADDING_LOCAL
    }
}
