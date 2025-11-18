use tessera_ui::{
    compute::resource::ComputeResourceManager,
    renderer::compute::{ComputablePipeline, ComputeBatchItem},
    wgpu,
};

use super::command::ContrastCommand;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    contrast: f32,
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
}

/// Pipeline for applying contrast adjustment to an image using a compute shader.
pub struct ContrastPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ContrastPipeline {
    pub fn new(device: &wgpu::Device, pipeline_cache: Option<&wgpu::PipelineCache>) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Contrast Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("contrast.wgsl").into()),
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
                        min_binding_size: Some(std::num::NonZeroU64::new(20).unwrap()),
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
            cache: pipeline_cache,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

impl ComputablePipeline<ContrastCommand> for ContrastPipeline {
    /// Dispatches one or more contrast adjustment compute commands.
    fn dispatch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        items: &[ComputeBatchItem<'_, ContrastCommand>],
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    ) {
        for item in items {
            let Some(mean_buffer) = resource_manager.get(&item.command.mean_result_handle) else {
                continue;
            };

            let target_area = item.target_area;
            let uniforms = Uniforms {
                contrast: item.command.contrast,
                area_x: target_area.x.0 as u32,
                area_y: target_area.y.0 as u32,
                area_width: target_area.width.0 as u32,
                area_height: target_area.height.0 as u32,
            };

            let uniform_array = [uniforms];
            let uniform_bytes = bytemuck::cast_slice(&uniform_array);
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Contrast Uniform Buffer"),
                size: uniform_bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&uniform_buffer, 0, uniform_bytes);

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
