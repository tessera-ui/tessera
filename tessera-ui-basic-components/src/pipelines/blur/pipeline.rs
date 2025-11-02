use encase::{ShaderType, UniformBuffer};
use tessera_ui::{
    renderer::compute::{ComputablePipeline, ComputeBatchItem},
    wgpu,
};

use super::command::BlurCommand;

#[derive(ShaderType)]
struct BlurUniforms {
    radius: f32,
    direction_x: f32,
    direction_y: f32,
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
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
    /// Dispatches one or more blur compute commands within the active pass.
    fn dispatch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        items: &[ComputeBatchItem<'_, BlurCommand>],
        _resource_manager: &mut tessera_ui::ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    ) {
        for item in items {
            let target_area = item.target_area;
            let uniforms = BlurUniforms {
                radius: item.command.radius,
                direction_x: item.command.direction.0,
                direction_y: item.command.direction.1,
                area_x: target_area.x.0 as u32,
                area_y: target_area.y.0 as u32,
                area_width: target_area.width.0 as u32,
                area_height: target_area.height.0 as u32,
            };

            let mut buffer = UniformBuffer::new(Vec::new());
            buffer.write(&uniforms).unwrap();
            let uniform_bytes = buffer.into_inner();
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Blur Uniform Buffer"),
                size: uniform_bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&uniform_buffer, 0, &uniform_bytes);

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
            let workgroups_x = config.width.div_ceil(8);
            let workgroups_y = config.height.div_ceil(8);
            if workgroups_x == 0 || workgroups_y == 0 {
                continue;
            }
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }
    }
}
