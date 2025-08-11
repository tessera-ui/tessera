use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use encase::{ShaderType, UniformBuffer};
use glam::Vec4;
use tessera_ui::{DrawCommand, PxPosition, PxSize, renderer::drawer::DrawablePipeline, wgpu};

#[derive(Debug, Clone)]
/// Image pixel data for rendering.
///
/// # Fields
/// - `data`: Raw pixel data (RGBA).
/// - `width`: Image width in pixels.
/// - `height`: Image height in pixels.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::pipelines::image::ImageData;
/// let img = ImageData { data: Arc::new(vec![255, 0, 0, 255]), width: 1, height: 1 };
/// ```
pub struct ImageData {
    pub data: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

impl Hash for ImageData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.as_ref().hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl PartialEq for ImageData {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.data.as_ref() == other.data.as_ref()
    }
}

impl Eq for ImageData {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// Command for rendering an image in a UI component.
///
/// # Example
/// ```rust,ignore
/// use tessera_ui_basic_components::pipelines::image::{ImageCommand, ImageData};
/// let cmd = ImageCommand { data: img_data };
/// ```
pub struct ImageCommand {
    pub data: ImageData,
}

impl DrawCommand for ImageCommand {
    fn barrier(&self) -> Option<tessera_ui::BarrierRequirement> {
        // This command does not require any specific barriers.
        None
    }
}

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
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("image/image.wgsl").into()),
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
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            resources: HashMap::new(),
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
    ) {
        render_pass.set_pipeline(&self.pipeline);

        for (command, size, start_pos) in commands {
            let resources = self
                .resources
                .entry(command.data.clone())
                .or_insert_with(|| {
                    let texture_size = wgpu::Extent3d {
                        width: command.data.width,
                        height: command.data.height,
                        depth_or_array_layers: 1,
                    };
                    let diffuse_texture = gpu.create_texture(&wgpu::TextureDescriptor {
                        size: texture_size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: config.format,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        label: Some("diffuse_texture"),
                        view_formats: &[],
                    });

                    gpu_queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &diffuse_texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &command.data.data,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * command.data.width),
                            rows_per_image: Some(command.data.height),
                        },
                        texture_size,
                    );

                    let diffuse_texture_view =
                        diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let diffuse_sampler = gpu.create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Nearest,
                        mipmap_filter: wgpu::FilterMode::Nearest,
                        ..Default::default()
                    });

                    let uniform_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Image Uniform Buffer"),
                        size: ImageUniforms::min_size().get(),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });

                    let diffuse_bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &self.bind_group_layout,
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
                });

            let is_bgra = matches!(
                config.format,
                wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
            );
            let uniforms = ImageUniforms {
                rect: [
                    (start_pos.x.0 as f32 / config.width as f32) * 2.0 - 1.0
                        + (size.width.0 as f32 / config.width as f32),
                    (start_pos.y.0 as f32 / config.height as f32) * -2.0 + 1.0
                        - (size.height.0 as f32 / config.height as f32),
                    size.width.0 as f32 / config.width as f32,
                    size.height.0 as f32 / config.height as f32,
                ]
                .into(),
                is_bgra: if is_bgra { 1 } else { 0 },
            };
            let mut buffer = UniformBuffer::new(Vec::new());
            buffer.write(&uniforms).unwrap();
            gpu_queue.write_buffer(&resources.uniform_buffer, 0, &buffer.into_inner());

            render_pass.set_bind_group(0, &resources.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
    }
}
