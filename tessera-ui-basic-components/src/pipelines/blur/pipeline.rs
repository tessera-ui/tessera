use tessera_ui::{
    renderer::compute::ComputablePipeline,
    wgpu::{self, util::DeviceExt},
};

use super::command::BlurCommand;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurUniforms {
    radius: f32,
    direction_x: f32,
    direction_y: f32,
    _padding: f32,
}

pub struct BlurPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl BlurPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("blur.wgsl").into()),
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
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 1: Source Texture (Sampled)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
                        // This format needs to match the destination texture format.
                        // We assume a common format here, but a robust implementation might
                        // create pipelines on the fly based on the texture's actual format.
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: Some("blur_bind_group_layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blur Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Blur Pipeline"),
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

impl ComputablePipeline<BlurCommand> for BlurPipeline {
    fn dispatch(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &BlurCommand,
        _resource_manager: &mut tessera_ui::ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    ) {
        let uniforms = BlurUniforms {
            radius: command.radius,
            direction_x: command.direction.0,
            direction_y: command.direction.1,
            _padding: 0.0,
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Blur Uniform Buffer"),
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
            ],
            label: Some("blur_bind_group"),
        });

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(config.width.div_ceil(8), config.height.div_ceil(8), 1);
    }
}
