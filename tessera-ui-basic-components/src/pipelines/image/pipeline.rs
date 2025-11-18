use std::collections::HashMap;

use encase::{ShaderType, UniformBuffer};
use glam::Vec4;
use tessera_ui::{PxPosition, PxSize, px::PxRect, renderer::drawer::DrawablePipeline, wgpu};

use super::command::{ImageCommand, ImageData};

#[derive(ShaderType)]
struct ImageUniforms {
    rect: Vec4,
    is_bgra: u32,
}

struct ImageResources {
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

/// Pipeline for rendering images in UI components.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::pipelines::image::ImagePipeline;
/// let pipeline = ImagePipeline::new(&device, &config, sample_count);
/// ```
pub struct ImagePipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    resources: HashMap<ImageData, ImageResources>,
}

impl ImagePipeline {
    /// Create a new ImagePipeline.
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("image.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Image Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: pipeline_cache,
        });

        Self {
            pipeline,
            bind_group_layout,
            resources: HashMap::new(),
        }
    }

    /// Return existing resources for `data` or create them.
    fn get_or_create_resources(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        data: &ImageData,
    ) -> &ImageResources {
        self.resources.entry(data.clone()).or_insert_with(|| {
            Self::create_image_resources(device, queue, config, &self.bind_group_layout, data)
        })
    }

    /// Compute the ImageUniforms for a given command size and position.
    fn compute_uniforms(
        start_pos: PxPosition,
        size: PxSize,
        config: &wgpu::SurfaceConfiguration,
    ) -> ImageUniforms {
        // Convert pixel positions/sizes into normalized device coordinates and size ratios.
        let rect = [
            (start_pos.x.0 as f32 / config.width as f32) * 2.0 - 1.0
                + (size.width.0 as f32 / config.width as f32),
            (start_pos.y.0 as f32 / config.height as f32) * -2.0 + 1.0
                - (size.height.0 as f32 / config.height as f32),
            size.width.0 as f32 / config.width as f32,
            size.height.0 as f32 / config.height as f32,
        ]
        .into();

        let is_bgra = matches!(
            config.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        );

        ImageUniforms {
            rect,
            is_bgra: if is_bgra { 1 } else { 0 },
        }
    }

    // Create GPU resources for an image. Kept as a single helper to avoid duplicating
    // GPU setup logic while keeping `draw` concise.
    fn create_image_resources(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        layout: &wgpu::BindGroupLayout,
        data: &ImageData,
    ) -> ImageResources {
        let texture_size = wgpu::Extent3d {
            width: data.width,
            height: data.height,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * data.width),
                rows_per_image: Some(data.height),
            },
            texture_size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Image Uniform Buffer"),
            size: ImageUniforms::min_size().get(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        ImageResources {
            bind_group: diffuse_bind_group,
            uniform_buffer,
        }
    }
}

impl DrawablePipeline<ImageCommand> for ImagePipeline {
    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        commands: &[(&ImageCommand, PxSize, PxPosition)],
        _scene_texture_view: &wgpu::TextureView,
        _clip_rect: Option<PxRect>,
    ) {
        render_pass.set_pipeline(&self.pipeline);

        for (command, size, start_pos) in commands {
            // Use the extracted helper to obtain or create GPU resources.
            let resources = self.get_or_create_resources(gpu, gpu_queue, config, &command.data);

            // Use the extracted uniforms computation helper (dereference borrowed tuple elements).
            let uniforms = Self::compute_uniforms(*start_pos, *size, config);

            let mut buffer = UniformBuffer::new(Vec::new());
            buffer.write(&uniforms).unwrap();
            gpu_queue.write_buffer(&resources.uniform_buffer, 0, &buffer.into_inner());

            render_pass.set_bind_group(0, &resources.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
    }
}
