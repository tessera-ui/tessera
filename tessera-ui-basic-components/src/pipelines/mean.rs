//! # Mean Luminance Compute Pipeline
//!
//! This module implements a GPU-based compute pipeline for calculating the mean luminance of a texture.
//! It provides both the pipeline and command abstractions for integrating mean luminance computation into Tessera UI rendering flows.
//!
//! ## Functionality
//! - Calculates the average (mean) luminance of a given texture using a compute shader.
//! - Exposes a [`MeanCommand`](MeanCommand) for issuing the operation and a [`MeanPipeline`](MeanPipeline) for dispatching the compute workload.
//!
//! ## Typical Use Cases
//! - Image processing and analysis
//! - Tone mapping and exposure adaptation in UI or graphics applications
//! - Adaptive UI rendering based on scene brightness
//!
//! ## Integration
//! Register and use this pipeline when average brightness information is required for further rendering or UI logic.
//! See [`MeanCommand`] and [`MeanPipeline`] for usage examples.

use tessera_ui::{
    compute::{ComputeResourceRef, resource::ComputeResourceManager},
    renderer::compute::{ComputablePipeline, command::ComputeCommand},
    wgpu,
};

// --- Command ---

/// A command to calculate the mean luminance of the input texture.
#[derive(Debug, Clone, Copy)]
/// Command to calculate the mean luminance of the input texture.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::pipelines::mean::MeanCommand;
/// let command = MeanCommand::new(&device, &mut resource_manager);
/// ```
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
            size: 8, // two u32s: total_luminance, total_pixels
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let result_buffer_ref = compute_resource_manager.push(result_buffer);
        MeanCommand { result_buffer_ref }
    }

    /// Returns the reference to the result buffer.
    pub fn result_buffer_ref(&self) -> ComputeResourceRef {
        self.result_buffer_ref
    }
}

impl ComputeCommand for MeanCommand {}

// --- Pipeline ---

/// Pipeline for calculating mean luminance using a compute shader.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::pipelines::mean::MeanPipeline;
/// let pipeline = MeanPipeline::new(&device);
/// ```
pub struct MeanPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl MeanPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mean Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("mean/mean.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // 0: Source Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // 1: Result Buffer (Storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::new(8).unwrap()),
                    },
                    count: None,
                },
                // 2: Destination Texture (Storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: Some("mean_bind_group_layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mean Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Mean Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

impl ComputablePipeline<MeanCommand> for MeanPipeline {
    /// Dispatches the compute shader to calculate mean luminance.
    ///
    /// # Parameters
    /// - `device`: The wgpu device.
    /// - `config`: Surface configuration.
    /// - `compute_pass`: The compute pass to encode commands.
    /// - `command`: The mean command with buffer reference.
    /// - `resource_manager`: Resource manager for compute buffers.
    /// - `input_view`: Source texture view.
    /// - `output_view`: Destination texture view.
    fn dispatch(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &MeanCommand,
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    ) {
        let result_buffer = resource_manager.get(&command.result_buffer_ref).unwrap();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: result_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(output_view),
                },
            ],
            label: Some("mean_bind_group"),
        });

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(config.width.div_ceil(8), config.height.div_ceil(8), 1);
    }
}
