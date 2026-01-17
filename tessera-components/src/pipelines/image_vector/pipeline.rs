use std::{collections::HashMap, sync::Arc};

use encase::{ShaderType, UniformBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    Color, PxPosition, PxSize,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline},
    wgpu::{self, util::DeviceExt},
};

use super::command::{ImageVectorCommand, ImageVectorData, ImageVectorVertex, VectorTintMode};

const DEFAULT_ATLAS_SIZE: u32 = 2048;
const MIN_ATLAS_SIZE: u32 = 256;
const ATLAS_PADDING: u32 = 1;
const ATLAS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

struct GeometryResources {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

#[derive(Hash, PartialEq, Eq, Clone)]
struct AtlasKey {
    data: ImageVectorData,
    width: u32,
    height: u32,
}

impl AtlasKey {
    fn new(data: &Arc<ImageVectorData>, width: u32, height: u32) -> Self {
        Self {
            data: (**data).clone(),
            width,
            height,
        }
    }
}

struct AtlasCacheEntry {
    uv_origin: [f32; 2],
    uv_scale: [f32; 2],
    page_index: usize,
}

#[derive(ShaderType, Clone, Copy)]
struct ImageVectorUniforms {
    origin: Vec2,
    scale: Vec2,
    tint: Vec4,
}

#[derive(ShaderType, Clone, Copy)]
struct AtlasSampleUniforms {
    origin: Vec2,
    scale: Vec2,
    uv_origin: Vec2,
    uv_scale: Vec2,
    tint: Vec4,
    tint_mode: u32,
    rotation: f32,
}

/// Render pipeline that rasterizes SVG vector meshes into an atlas for
/// sampling.
pub struct ImageVectorPipeline {
    raster_pipeline: wgpu::RenderPipeline,
    raster_bind_group: wgpu::BindGroup,
    sample_pipeline: wgpu::RenderPipeline,
    sample_bind_group_layout: wgpu::BindGroupLayout,
    atlas_sampler: wgpu::Sampler,
    atlas: VectorAtlas,
    resources: HashMap<ImageVectorData, GeometryResources>,
    cache: HashMap<AtlasKey, AtlasCacheEntry>,
    raster_sample_count: u32,
}

impl ImageVectorPipeline {
    /// Creates the vector atlas pipeline with raster and sampling passes.
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let raster_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image Vector Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("image_vector.wgsl").into()),
        });
        let sample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image Vector Atlas Sample Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("atlas_sample.wgsl").into()),
        });

        let raster_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(ImageVectorUniforms::min_size()),
                    },
                    count: None,
                }],
                label: Some("image_vector_raster_bind_group_layout"),
            });

        let sample_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                            min_binding_size: Some(AtlasSampleUniforms::min_size()),
                        },
                        count: None,
                    },
                ],
                label: Some("image_vector_sample_bind_group_layout"),
            });

        let raster_uniforms = raster_uniforms();
        let mut uniform_data = UniformBuffer::new(Vec::new());
        uniform_data
            .write(&raster_uniforms)
            .expect("raster uniform serialization failed");
        let raster_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_vector_raster_uniform_buffer"),
            contents: &uniform_data.into_inner(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let raster_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &raster_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: raster_uniform_buffer.as_entire_binding(),
            }],
            label: Some("image_vector_raster_bind_group"),
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ImageVectorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };
        let vertex_layouts = [vertex_layout];

        let raster_sample_count = desired_raster_sample_count(sample_count);

        let raster_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Vector Raster Pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("image_vector_raster_pipeline_layout"),
                    bind_group_layouts: &[&raster_bind_group_layout],
                    immediate_size: 0,
                }),
            ),
            vertex: wgpu::VertexState {
                module: &raster_shader,
                entry_point: Some("vs_main"),
                buffers: &vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &raster_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: ATLAS_FORMAT,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: raster_sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: pipeline_cache,
        });

        let sample_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Vector Atlas Sample Pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("image_vector_sample_pipeline_layout"),
                    bind_group_layouts: &[&sample_bind_group_layout],
                    immediate_size: 0,
                }),
            ),
            vertex: wgpu::VertexState {
                module: &sample_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sample_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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
            multiview_mask: None,
            cache: pipeline_cache,
        });

        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("image_vector_atlas_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        Self {
            raster_pipeline,
            raster_bind_group,
            sample_pipeline,
            sample_bind_group_layout,
            atlas_sampler,
            atlas: VectorAtlas::new(device),
            resources: HashMap::new(),
            cache: HashMap::new(),
            raster_sample_count,
        }
    }

    fn create_resources(device: &wgpu::Device, data: &Arc<ImageVectorData>) -> GeometryResources {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_vector_vertex_buffer"),
            contents: bytemuck::cast_slice(data.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_vector_index_buffer"),
            contents: bytemuck::cast_slice(data.indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });

        GeometryResources {
            vertex_buffer,
            index_buffer,
            index_count: data.indices.len() as u32,
        }
    }

    fn ensure_cached_entry(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &Arc<ImageVectorData>,
        width: u32,
        height: u32,
    ) {
        let key = AtlasKey::new(data, width, height);
        if self.cache.contains_key(&key) {
            return;
        }

        let geometry_key = (**data).clone();
        self.resources
            .entry(geometry_key.clone())
            .or_insert_with(|| Self::create_resources(device, data));

        let allocation = self.atlas.allocate(device, queue, width, height);
        let geometry = self
            .resources
            .get(&geometry_key)
            .expect("geometry must exist");
        self.rasterize_into_allocation(device, queue, geometry, &allocation, width, height);

        let uv_origin = [
            allocation.inner_rect.x as f32 / allocation.page_size.0 as f32,
            allocation.inner_rect.y as f32 / allocation.page_size.1 as f32,
        ];
        let uv_scale = [
            allocation.inner_rect.width as f32 / allocation.page_size.0 as f32,
            allocation.inner_rect.height as f32 / allocation.page_size.1 as f32,
        ];

        self.cache.insert(
            key,
            AtlasCacheEntry {
                uv_origin,
                uv_scale,
                page_index: allocation.page_index,
            },
        );
    }

    fn rasterize_into_allocation(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        geometry: &GeometryResources,
        allocation: &AtlasAllocation,
        width: u32,
        height: u32,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("image_vector_atlas_encoder"),
        });

        let resolved_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("image_vector_resolved_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ATLAS_FORMAT,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let resolved_view = resolved_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let (color_view, resolve_target, _msaa_texture) = if self.raster_sample_count > 1 {
            let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("image_vector_msaa_texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.raster_sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: ATLAS_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());
            (msaa_view, Some(resolved_view.clone()), Some(msaa_texture))
        } else {
            (resolved_view.clone(), None, None)
        };

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("image_vector_raster_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    depth_slice: None,
                    resolve_target: resolve_target.as_ref(),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.raster_pipeline);
            pass.set_bind_group(0, &self.raster_bind_group, &[]);
            pass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
            pass.set_index_buffer(geometry.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..geometry.index_count, 0, 0..1);
        }

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &resolved_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: self.atlas.page_texture(allocation.page_index),
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: allocation.inner_rect.x,
                    y: allocation.inner_rect.y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: allocation.inner_rect.width,
                height: allocation.inner_rect.height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(std::iter::once(encoder.finish()));
    }
}

impl DrawablePipeline<ImageVectorCommand> for ImageVectorPipeline {
    fn draw(&mut self, context: &mut DrawContext<ImageVectorCommand>) {
        if context.commands.is_empty() {
            return;
        }

        for (command, size, _) in context.commands.iter() {
            if let Some((width, height)) = physical_dimensions(*size) {
                self.ensure_cached_entry(
                    context.device,
                    context.queue,
                    &command.data,
                    width,
                    height,
                );
            }
        }

        context.render_pass.set_pipeline(&self.sample_pipeline);

        for (command, size, start_pos) in context.commands.iter() {
            let Some((width, height)) = physical_dimensions(*size) else {
                continue;
            };
            let key = AtlasKey::new(&command.data, width, height);
            let entry = match self.cache.get_mut(&key) {
                Some(entry) => entry,
                None => continue,
            };

            let uniforms = compute_sample_uniforms(SampleUniformParams {
                start_pos: *start_pos,
                size: *size,
                tint: command.tint,
                tint_mode: command.tint_mode,
                rotation: command.rotation,
                uv_origin: entry.uv_origin,
                uv_scale: entry.uv_scale,
                target_size: context.target_size,
            });
            let mut buffer = UniformBuffer::new(Vec::new());
            buffer
                .write(&uniforms)
                .expect("sample uniform serialization failed");

            let uniform_buffer =
                context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("image_vector_sample_uniform_buffer"),
                        contents: &buffer.into_inner(),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });

            let bind_group = context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.sample_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                self.atlas.page_view(entry.page_index),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: uniform_buffer.as_entire_binding(),
                        },
                    ],
                    label: Some("image_vector_sample_bind_group"),
                });

            context.render_pass.set_bind_group(0, &bind_group, &[]);
            context.render_pass.draw(0..6, 0..1);
        }
    }
}

fn desired_raster_sample_count(global_sample_count: u32) -> u32 {
    if global_sample_count > 1 {
        global_sample_count
    } else {
        4
    }
}

fn raster_uniforms() -> ImageVectorUniforms {
    ImageVectorUniforms {
        origin: Vec2::new(-1.0, 1.0),
        scale: Vec2::new(2.0, -2.0),
        tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
    }
}

#[derive(Clone, Copy)]
struct SampleUniformParams {
    start_pos: PxPosition,
    size: PxSize,
    tint: Color,
    tint_mode: VectorTintMode,
    rotation: f32,
    uv_origin: [f32; 2],
    uv_scale: [f32; 2],
    target_size: PxSize,
}

fn compute_sample_uniforms(params: SampleUniformParams) -> AtlasSampleUniforms {
    let SampleUniformParams {
        start_pos,
        size,
        tint,
        tint_mode,
        rotation,
        uv_origin,
        uv_scale,
        target_size,
    } = params;

    let left = (start_pos.x.0 as f32 / target_size.width.to_f32()) * 2.0 - 1.0;
    let right = ((start_pos.x.0 + size.width.0) as f32 / target_size.width.to_f32()) * 2.0 - 1.0;
    let top = 1.0 - (start_pos.y.0 as f32 / target_size.height.to_f32()) * 2.0;
    let bottom = 1.0 - ((start_pos.y.0 + size.height.0) as f32 / target_size.height.to_f32()) * 2.0;

    AtlasSampleUniforms {
        origin: Vec2::new(left, top),
        scale: Vec2::new(right - left, bottom - top),
        uv_origin: Vec2::from_array(uv_origin),
        uv_scale: Vec2::from_array(uv_scale),
        tint: Vec4::new(tint.r, tint.g, tint.b, tint.a),
        tint_mode: match tint_mode {
            VectorTintMode::Multiply => 0,
            VectorTintMode::Solid => 1,
        },
        rotation: rotation.to_radians(),
    }
}

fn physical_dimensions(size: PxSize) -> Option<(u32, u32)> {
    if size.width.0 <= 0 || size.height.0 <= 0 {
        None
    } else {
        Some((size.width.0 as u32, size.height.0 as u32))
    }
}

struct AtlasAllocation {
    page_index: usize,
    inner_rect: AtlasRect,
    page_size: (u32, u32),
}

#[derive(Clone, Copy)]
struct AtlasRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

struct VectorAtlas {
    pages: Vec<AtlasPage>,
    default_size: u32,
    max_dimension: u32,
}

impl VectorAtlas {
    fn new(device: &wgpu::Device) -> Self {
        let max_dimension = device.limits().max_texture_dimension_2d;
        let default_size = DEFAULT_ATLAS_SIZE.min(max_dimension);
        Self {
            pages: Vec::new(),
            default_size: default_size.max(MIN_ATLAS_SIZE),
            max_dimension,
        }
    }

    fn allocate(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> AtlasAllocation {
        let padded_width = width.saturating_add(ATLAS_PADDING * 2).max(1);
        let padded_height = height.saturating_add(ATLAS_PADDING * 2).max(1);

        if padded_width > self.max_dimension || padded_height > self.max_dimension {
            panic!(
                "Image vector target {}x{} exceeds GPU atlas limit {}",
                width, height, self.max_dimension
            );
        }

        for (index, page) in self.pages.iter_mut().enumerate() {
            if let Some(rect) = page.allocate(padded_width, padded_height) {
                return AtlasAllocation {
                    page_index: index,
                    inner_rect: AtlasRect {
                        x: rect.x + ATLAS_PADDING,
                        y: rect.y + ATLAS_PADDING,
                        width,
                        height,
                    },
                    page_size: (page.width, page.height),
                };
            }
        }

        let needed = padded_width.max(padded_height);
        let mut page_size = self.default_size.max(needed).max(MIN_ATLAS_SIZE);
        page_size = page_size.min(self.max_dimension);
        let page_index = self.add_page(device, queue, page_size);
        let page = self
            .pages
            .get_mut(page_index)
            .expect("new page should exist");
        let rect = page
            .allocate(padded_width, padded_height)
            .expect("allocation should fit in a new page");
        AtlasAllocation {
            page_index,
            inner_rect: AtlasRect {
                x: rect.x + ATLAS_PADDING,
                y: rect.y + ATLAS_PADDING,
                width,
                height,
            },
            page_size: (page.width, page.height),
        }
    }

    fn add_page(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, size: u32) -> usize {
        let page = AtlasPage::new(device, queue, size, size);
        self.pages.push(page);
        self.pages.len() - 1
    }

    fn page_view(&self, index: usize) -> &wgpu::TextureView {
        &self.pages[index].view
    }

    fn page_texture(&self, index: usize) -> &wgpu::Texture {
        &self.pages[index].texture
    }
}

struct ShelfRow {
    y: u32,
    height: u32,
    cursor_x: u32,
}

struct AtlasPage {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    rows: Vec<ShelfRow>,
    next_y: u32,
}

impl AtlasPage {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("image_vector_atlas_page"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ATLAS_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("image_vector_atlas_clear_encoder"),
        });
        encoder.clear_texture(
            &texture,
            &wgpu::ImageSubresourceRange {
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
            },
        );
        queue.submit(std::iter::once(encoder.finish()));

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            width,
            height,
            rows: Vec::new(),
            next_y: 0,
        }
    }

    fn allocate(&mut self, width: u32, height: u32) -> Option<AtlasRect> {
        for row in &mut self.rows {
            if height <= row.height && row.cursor_x + width <= self.width {
                let rect = AtlasRect {
                    x: row.cursor_x,
                    y: row.y,
                    width,
                    height,
                };
                row.cursor_x += width;
                return Some(rect);
            }
        }

        if self.next_y + height > self.height {
            return None;
        }

        let rect = AtlasRect {
            x: 0,
            y: self.next_y,
            width,
            height,
        };
        self.rows.push(ShelfRow {
            y: self.next_y,
            height,
            cursor_x: width,
        });
        self.next_y += height;
        Some(rect)
    }
}
