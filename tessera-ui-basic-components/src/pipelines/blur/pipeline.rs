use encase::{ShaderType, UniformBuffer, internal::WriteInto};
use tessera_ui::{
    renderer::compute::{ComputablePipeline, ComputeBatchItem},
    wgpu,
};

use super::command::DualBlurCommand;

const DOWNSCALE_FACTOR: u32 = 2;

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

#[derive(ShaderType)]
struct DownsampleUniforms {
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
    scale: u32,
}

#[derive(ShaderType)]
struct UpsampleUniforms {
    area_x: u32,
    area_y: u32,
    area_width: u32,
    area_height: u32,
    scale: u32,
}

pub struct BlurPipeline {
    downsample_pipeline: wgpu::ComputePipeline,
    blur_pipeline: wgpu::ComputePipeline,
    upsample_pipeline: wgpu::ComputePipeline,
    downsample_bind_group_layout: wgpu::BindGroupLayout,
    blur_bind_group_layout: wgpu::BindGroupLayout,
    upsample_bind_group_layout: wgpu::BindGroupLayout,
}

impl BlurPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let downsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur Downsample Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("downsample.wgsl").into()),
        });
        let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("blur.wgsl").into()),
        });
        let upsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blur Upsample Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("upsample.wgsl").into()),
        });

        let downsample_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
                label: Some("blur_downsample_bind_group_layout"),
            });

        let blur_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
                label: Some("blur_pass_bind_group_layout"),
            });

        let upsample_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
                label: Some("blur_upsample_bind_group_layout"),
            });

        let downsample_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Blur Downsample Pipeline Layout"),
                bind_group_layouts: &[&downsample_bind_group_layout],
                push_constant_ranges: &[],
            });
        let blur_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blur Pipeline Layout"),
            bind_group_layouts: &[&blur_bind_group_layout],
            push_constant_ranges: &[],
        });
        let upsample_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Blur Upsample Pipeline Layout"),
                bind_group_layouts: &[&upsample_bind_group_layout],
                push_constant_ranges: &[],
            });

        let downsample_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Blur Downsample Pipeline"),
                layout: Some(&downsample_pipeline_layout),
                module: &downsample_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });
        let blur_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Blur Pipeline"),
            layout: Some(&blur_pipeline_layout),
            module: &blur_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let upsample_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Blur Upsample Pipeline"),
            layout: Some(&upsample_pipeline_layout),
            module: &upsample_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            downsample_pipeline,
            blur_pipeline,
            upsample_pipeline,
            downsample_bind_group_layout,
            blur_bind_group_layout,
            upsample_bind_group_layout,
        }
    }

    fn create_uniform_buffer<T: ShaderType + WriteInto>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
        data: &T,
    ) -> wgpu::Buffer {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(data).unwrap();
        let bytes = buffer.into_inner();
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform_buffer, 0, &bytes);
        uniform_buffer
    }
}

impl ComputablePipeline<DualBlurCommand> for BlurPipeline {
    /// Dispatches one or more blur compute commands within the active pass.
    fn dispatch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        items: &[ComputeBatchItem<'_, DualBlurCommand>],
        _resource_manager: &mut tessera_ui::ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    ) {
        for item in items {
            let target_area = item.target_area;
            let area_x = target_area.x.0 as u32;
            let area_y = target_area.y.0 as u32;
            let area_width = target_area.width.0 as u32;
            let area_height = target_area.height.0 as u32;

            if area_width == 0 || area_height == 0 {
                continue;
            }

            let scale = DOWNSCALE_FACTOR.max(1);
            let down_width = area_width.div_ceil(scale);
            let down_height = area_height.div_ceil(scale);

            if down_width == 0 || down_height == 0 {
                continue;
            }

            let texture_descriptor = wgpu::TextureDescriptor {
                label: Some("Blur Downscaled Texture"),
                size: wgpu::Extent3d {
                    width: down_width.max(1),
                    height: down_height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };

            let downsample_texture = device.create_texture(&texture_descriptor);
            let downsample_view =
                downsample_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let blur_texture = device.create_texture(&texture_descriptor);
            let blur_view = blur_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Downsample pass
            let downsample_uniforms = DownsampleUniforms {
                area_x,
                area_y,
                area_width,
                area_height,
                scale,
            };
            let downsample_uniform_buffer = Self::create_uniform_buffer(
                device,
                queue,
                "Blur Downsample Uniform Buffer",
                &downsample_uniforms,
            );
            let downsample_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.downsample_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: downsample_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(input_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&downsample_view),
                    },
                ],
                label: Some("blur_downsample_bind_group"),
            });
            compute_pass.set_pipeline(&self.downsample_pipeline);
            compute_pass.set_bind_group(0, &downsample_bind_group, &[]);
            let downsample_workgroups_x = down_width.div_ceil(8);
            let downsample_workgroups_y = down_height.div_ceil(8);
            if downsample_workgroups_x == 0 || downsample_workgroups_y == 0 {
                continue;
            }
            compute_pass.dispatch_workgroups(downsample_workgroups_x, downsample_workgroups_y, 1);

            // Directional blur pass
            let mut read_view = downsample_view.clone();
            let mut write_view = blur_view.clone();
            for pass in &item.command.passes {
                let effective_radius = (pass.radius / scale as f32).max(0.0);
                let blur_uniforms = BlurUniforms {
                    radius: effective_radius,
                    direction_x: pass.direction.0,
                    direction_y: pass.direction.1,
                    area_x: 0,
                    area_y: 0,
                    area_width: down_width,
                    area_height: down_height,
                };
                let blur_uniform_buffer = Self::create_uniform_buffer(
                    device,
                    queue,
                    "Blur Pass Uniform Buffer",
                    &blur_uniforms,
                );
                let blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.blur_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: blur_uniform_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&read_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&write_view),
                        },
                    ],
                    label: Some("blur_directional_bind_group"),
                });
                compute_pass.set_pipeline(&self.blur_pipeline);
                compute_pass.set_bind_group(0, &blur_bind_group, &[]);
                compute_pass.dispatch_workgroups(
                    downsample_workgroups_x,
                    downsample_workgroups_y,
                    1,
                );

                std::mem::swap(&mut read_view, &mut write_view);
            }

            // Upsample pass
            let upsample_uniforms = UpsampleUniforms {
                area_x,
                area_y,
                area_width,
                area_height,
                scale,
            };
            let upsample_uniform_buffer = Self::create_uniform_buffer(
                device,
                queue,
                "Blur Upsample Uniform Buffer",
                &upsample_uniforms,
            );
            let upsample_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.upsample_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: upsample_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&read_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(output_view),
                    },
                ],
                label: Some("blur_upsample_bind_group"),
            });
            compute_pass.set_pipeline(&self.upsample_pipeline);
            compute_pass.set_bind_group(0, &upsample_bind_group, &[]);
            let upsample_workgroups_x = area_width.div_ceil(8);
            let upsample_workgroups_y = area_height.div_ceil(8);
            if upsample_workgroups_x == 0 || upsample_workgroups_y == 0 {
                continue;
            }
            compute_pass.dispatch_workgroups(upsample_workgroups_x, upsample_workgroups_y, 1);
        }
    }
}
