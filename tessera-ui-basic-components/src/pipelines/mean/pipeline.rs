use tessera_ui::{
    compute::pipeline::{ComputablePipeline, ComputeContext},
    wgpu,
};

use super::command::MeanCommand;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct AreaUniform {
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
}

/// Pipeline for calculating mean luminance using a compute shader.
pub struct MeanPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl MeanPipeline {
    /// Builds the mean (box blur) compute pipeline.
    pub fn new(device: &wgpu::Device, pipeline_cache: Option<&wgpu::PipelineCache>) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mean Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("mean.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // 0: Area Uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(16).expect("binding size must be non-zero"),
                        ),
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
                // 2: Result Buffer (Storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(8).expect("binding size must be non-zero"),
                        ),
                    },
                    count: None,
                },
                // 3: Destination Texture (Storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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
            cache: pipeline_cache,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

impl ComputablePipeline<MeanCommand> for MeanPipeline {
    /// Dispatches one or more mean luminance compute commands.
    fn dispatch(&mut self, context: &mut ComputeContext<MeanCommand>) {
        for item in context.items {
            let buffer_ref = item.command.result_buffer_ref();
            let Some(result_buffer) = context.resource_manager.get(&buffer_ref) else {
                continue;
            };
            context
                .queue
                .write_buffer(result_buffer, 0, bytemuck::cast_slice(&[0u32, 0u32]));
            let target_area = item.target_area;
            let area_uniform = AreaUniform {
                area_x: target_area.x.0 as u32,
                area_y: target_area.y.0 as u32,
                area_width: target_area.width.0 as u32,
                area_height: target_area.height.0 as u32,
            };
            let area_bytes = bytemuck::bytes_of(&area_uniform);
            let area_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mean Area Uniform Buffer"),
                size: area_bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            context.queue.write_buffer(&area_buffer, 0, area_bytes);
            let bind_group = context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: area_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(context.input_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: result_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(context.output_view),
                        },
                    ],
                    label: Some("mean_bind_group"),
                });

            context.compute_pass.set_pipeline(&self.pipeline);
            context.compute_pass.set_bind_group(0, &bind_group, &[]);
            context.compute_pass.dispatch_workgroups(
                context.config.width.div_ceil(8),
                context.config.height.div_ceil(8),
                1,
            );
        }
    }
}
