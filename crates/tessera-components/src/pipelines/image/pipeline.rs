use std::{
    collections::HashMap,
    num::NonZeroU64,
    sync::{
        Arc, Weak,
        atomic::{AtomicU32, Ordering},
    },
};

use encase::{ShaderType, UniformBuffer};
use glam::Vec4;
use tessera_ui::{
    PxSize,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline, FrameContext},
    wgpu,
};

use crate::pipelines::SharedQuad;

use super::command::{ImageCommand, ImageData};

#[derive(PartialEq, ShaderType)]
struct ImageUniforms {
    rect: Vec4,
    opacity: f32,
}

struct ImageResources {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    uploaded_rows: AtomicU32,
}

impl ImageResources {
    /// Uploads rows within the frame budget and reports whether all rows are
    /// uploaded.
    fn upload(&self, graphics: &tessera_ui::GraphicsDevice, data: &ImageData) -> bool {
        let start = self.uploaded_rows.load(Ordering::Relaxed);
        if start == data.height {
            return true;
        }
        let row_bytes = 4 * data.width;
        let rows = (graphics.upload_budget() / u64::from(row_bytes))
            .min(u64::from(data.height - start)) as u32;
        let end = start + rows;
        if rows > 0 {
            graphics.write_texture(
                "image_textures",
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: start,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &data.data[(start * row_bytes) as usize..(end * row_bytes) as usize],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row_bytes),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: data.width,
                    height: rows,
                    depth_or_array_layers: 1,
                },
            );
            self.uploaded_rows.store(end, Ordering::Relaxed);
        }
        if end < data.height {
            graphics.defer_upload();
        }
        end == data.height
    }
}

struct CachedImage {
    pixels: Weak<Vec<u8>>,
    content_hash: u64,
    resources: Arc<ImageResources>,
}

/// Pipeline for rendering images in UI components.
pub struct ImagePipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_layout: wgpu::BindGroupLayout,
    uniform_buffer: tessera_ui::TrackedBuffer,
    uniform_group: wgpu::BindGroup,
    uniform_capacity: u64,
    uniform_used: u64,
    sampler: wgpu::Sampler,
    quad_vertex_buffer: Arc<wgpu::Buffer>,
    quad_index_buffer: Arc<wgpu::Buffer>,
    resources: HashMap<(usize, u32, u32), CachedImage>,
}

impl ImagePipeline {
    /// Create a new ImagePipeline.
    pub fn new(
        device: &tessera_ui::GraphicsDevice,
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
            ],
            label: Some("texture_bind_group_layout"),
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Image Instance Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ImageUniforms::min_size()),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Image Pipeline Layout"),
            bind_group_layouts: &[Some(&uniform_layout), Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: tessera_ui::ui_stencil_state(),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: pipeline_cache,
        });

        let shared_quad = SharedQuad::get(device);
        let (uniform_buffer, uniform_group) =
            Self::uniform_resources(device, &uniform_layout, 4096);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            pipeline,
            bind_group_layout,
            uniform_layout,
            uniform_buffer,
            uniform_group,
            uniform_capacity: 4096,
            uniform_used: 0,
            sampler,
            quad_vertex_buffer: Arc::clone(shared_quad.vertex()),
            quad_index_buffer: Arc::clone(shared_quad.index()),
            resources: HashMap::new(),
        }
    }

    fn get_or_create_resources(
        &mut self,
        device: &tessera_ui::GraphicsDevice,
        data: &ImageData,
    ) -> Arc<ImageResources> {
        let identity = (Arc::as_ptr(&data.data) as usize, data.width, data.height);
        if !self.resources.contains_key(&identity) {
            let shared = self
                .resources
                .iter()
                .find_map(|(&(_, width, height), cached)| {
                    let pixels = cached.pixels.upgrade()?;
                    let candidate = ImageData {
                        data: pixels,
                        width,
                        height,
                        content_hash: cached.content_hash,
                    };
                    (candidate == *data).then(|| Arc::clone(&cached.resources))
                });
            let resources = shared.unwrap_or_else(|| {
                Arc::new(Self::create_image_resources(
                    device,
                    &self.bind_group_layout,
                    &self.sampler,
                    data,
                ))
            });
            self.resources.insert(
                identity,
                CachedImage {
                    pixels: Arc::downgrade(&data.data),
                    content_hash: data.content_hash,
                    resources,
                },
            );
        }
        Arc::clone(&self.resources[&identity].resources)
    }

    fn uniform_resources(
        device: &tessera_ui::GraphicsDevice,
        layout: &wgpu::BindGroupLayout,
        capacity: u64,
    ) -> (tessera_ui::TrackedBuffer, wgpu::BindGroup) {
        let buffer = device.create_buffer(
            "image_instances",
            &wgpu::BufferDescriptor {
                label: Some("Image Instances"),
                size: capacity,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image Instances"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: NonZeroU64::new(ImageUniforms::min_size().get()),
                }),
            }],
        });
        (buffer, group)
    }

    fn compute_uniforms(
        geometry: tessera_ui::DrawGeometry,
        target_size: PxSize,
        opacity: f32,
    ) -> ImageUniforms {
        let start_pos = geometry.window_origin();
        let [width, height] = geometry.window_size();
        let rect = [
            (start_pos.x / target_size.width.to_f32()) * 2.0 - 1.0
                + (width / target_size.width.to_f32()),
            (start_pos.y / target_size.height.to_f32()) * -2.0 + 1.0
                - (height / target_size.height.to_f32()),
            width / target_size.width.to_f32(),
            height / target_size.height.to_f32(),
        ]
        .into();

        ImageUniforms { rect, opacity }
    }

    fn create_image_resources(
        device: &tessera_ui::GraphicsDevice,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        data: &ImageData,
    ) -> ImageResources {
        let texture_size = wgpu::Extent3d {
            width: data.width,
            height: data.height,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(
            "image_textures",
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
                view_formats: &[],
            },
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        ImageResources {
            texture: diffuse_texture,
            bind_group: diffuse_bind_group,
            uploaded_rows: AtomicU32::new(0),
        }
    }
}

impl DrawablePipeline<ImageCommand> for ImagePipeline {
    fn begin_frame(&mut self, _context: &FrameContext<'_>) {
        self.uniform_used = 0;
    }

    fn end_frame(&mut self, _context: &FrameContext<'_>) {
        self.resources
            .retain(|_, resource| resource.pixels.strong_count() > 0);
    }

    fn draw(&mut self, context: &mut DrawContext<ImageCommand>) {
        context.render_pass.set_pipeline(&self.pipeline);
        context
            .render_pass
            .set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        context
            .render_pass
            .set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for (command, geometry) in context.commands.iter() {
            let resource = self.get_or_create_resources(context.graphics, &command.data);
            if !resource.upload(context.graphics, &command.data) {
                continue;
            }
            let uniforms = Self::compute_uniforms(*geometry, context.target_size, command.opacity);

            let mut buffer = UniformBuffer::new(Vec::new());
            buffer.write(&uniforms).expect("buffer write failed");
            let bytes = buffer.into_inner();
            let alignment = u64::from(
                context
                    .graphics
                    .limits()
                    .min_uniform_buffer_offset_alignment,
            )
            .max(1);
            let offset = self.uniform_used.next_multiple_of(alignment);
            let required = offset + bytes.len() as u64;
            if required > self.uniform_capacity {
                self.uniform_capacity = required.next_power_of_two();
                (self.uniform_buffer, self.uniform_group) = Self::uniform_resources(
                    context.graphics,
                    &self.uniform_layout,
                    self.uniform_capacity,
                );
            }
            context
                .graphics
                .write_buffer(&self.uniform_buffer, offset, &bytes);
            self.uniform_used = required;
            context.render_pass.set_bind_group(
                0,
                &self.uniform_group,
                &[u32::try_from(offset).expect("image instance offset")],
            );
            context
                .render_pass
                .set_bind_group(1, &resource.bind_group, &[]);
            context.render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }
}
