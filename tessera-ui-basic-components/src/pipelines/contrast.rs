use tessera_ui::{
    BarrierRequirement,
    compute::{ComputeResourceRef, resource::ComputeResourceManager},
    renderer::compute::{ComputablePipeline, command::ComputeCommand},
    wgpu::{self, util::DeviceExt},
};

// --- Command ---

/// Command to apply a contrast adjustment using a pre-calculated mean luminance.
///
/// # Parameters
/// - `contrast`: The contrast adjustment factor.
/// - `mean_result_handle`: Handle to the buffer containing mean luminance data.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::pipelines::contrast::ContrastCommand;
/// let command = ContrastCommand::new(1.2, mean_result_handle);
/// ```
#[derive(Debug, Clone, Copy)]
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
    /// - `contrast`: The contrast adjustment factor.
    /// - `mean_result_handle`: Handle to the buffer containing mean luminance data.
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

// --- Pipeline ---

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    contrast: f32,
}

/// Pipeline for applying contrast adjustment to an image using a compute shader.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::pipelines::contrast::ContrastPipeline;
/// let pipeline = ContrastPipeline::new(&device);
/// ```
pub struct ContrastPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ContrastPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Contrast Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("contrast/contrast.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // 0: Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::new(4).unwrap()),
                    },
                    count: None,
                },
                // 1: Source Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
                // 3: Mean Result Buffer (Storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::new(8).unwrap()),
                    },
                    count: None,
                },
            ],
            label: Some("contrast_bind_group_layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Contrast Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Contrast Pipeline"),
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

impl ComputablePipeline<ContrastCommand> for ContrastPipeline {
    fn dispatch(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &ContrastCommand,
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    ) {
        if let Some(mean_buffer) = resource_manager.get(&command.mean_result_handle) {
            let uniforms = Uniforms {
                contrast: command.contrast,
            };

            let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Contrast Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(input_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(output_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: mean_buffer.as_entire_binding(),
                    },
                ],
                label: Some("contrast_bind_group"),
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(
                config.width.div_ceil(8),
                config.height.div_ceil(8),
                1,
            );
        }
    }
}
