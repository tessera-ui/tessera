use tessera_ui::{
    compute::pipeline::{ComputablePipeline, ComputeContext},
    wgpu,
};

use super::command::ContrastCommand;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    contrast: f32,
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
}

/// Measures the mean luminance of each command's area and scales its colors
/// away from it.
pub struct ContrastPipeline {
    reset: wgpu::ComputePipeline,
    accumulate: wgpu::ComputePipeline,
    apply: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    mean: wgpu::Buffer,
}

impl ContrastPipeline {
    /// Builds the contrast compute pipelines, optionally reusing a cache.
    pub fn new(device: &wgpu::Device, pipeline_cache: Option<&wgpu::PipelineCache>) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Contrast Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("contrast.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("contrast_bind_group_layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Contrast Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let create = |entry_point| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Contrast Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                cache: pipeline_cache,
            })
        };

        Self {
            reset: create("reset"),
            accumulate: create("accumulate"),
            apply: create("apply"),
            bind_group_layout,
            // Dispatches in a pass run in order, so one accumulator serves every item.
            mean: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Contrast Mean Buffer"),
                size: 8,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            }),
        }
    }
}

impl ComputablePipeline<ContrastCommand> for ContrastPipeline {
    fn dispatch(&mut self, context: &mut ComputeContext<ContrastCommand>) {
        for item in context.items {
            let area = item.target_area;
            let uniforms = Uniforms {
                contrast: item.command.contrast,
                area_x: area.x.0 as u32,
                area_y: area.y.0 as u32,
                area_width: area.width.0 as u32,
                area_height: area.height.0 as u32,
            };
            let uniform_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Contrast Uniform Buffer"),
                size: std::mem::size_of::<Uniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            context
                .queue
                .write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

            let bind_group = context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: uniform_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(context.input_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(context.output_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: self.mean.as_entire_binding(),
                        },
                    ],
                    label: Some("contrast_bind_group"),
                });

            let pass = &mut *context.compute_pass;
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_pipeline(&self.reset);
            pass.dispatch_workgroups(1, 1, 1);
            pass.set_pipeline(&self.accumulate);
            pass.dispatch_workgroups(
                uniforms.area_width.div_ceil(8),
                uniforms.area_height.div_ceil(8),
                1,
            );
            pass.set_pipeline(&self.apply);
            pass.dispatch_workgroups(
                context.target_size.width.positive().div_ceil(8),
                context.target_size.height.positive().div_ceil(8),
                1,
            );
        }
    }
}
