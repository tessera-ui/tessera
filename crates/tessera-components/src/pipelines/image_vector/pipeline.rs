use std::{
    collections::{BTreeMap, HashMap},
    num::NonZeroU64,
};

use encase::{ShaderType, UniformBuffer};
use etagere::{AllocId, AtlasAllocator, size2};
use glam::{Vec2, Vec4};
use tessera_ui::{
    DrawGeometry, PxSize,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline, FrameContext},
    wgpu,
};

use super::command::{ImageVectorCommand, ImageVectorData, ImageVectorVertex, VectorTintMode};

const MIN_ATLAS_SIZE: u32 = 256;
const ATLAS_PADDING: u32 = 1;
const ATLAS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const SAMPLES_PER_BUFFER: usize = 64;
pub(super) const ATLAS_CACHE_BYTES: u64 = 32 << 20;
// Entries survive brief absences, such as icons scrolled out and back in.
pub(super) const RETAINED_FRAMES: u64 = 120;

struct GeometryResources {
    vertex_buffer: tessera_ui::TrackedBuffer,
    index_buffer: tessera_ui::TrackedBuffer,
    index_count: u32,
}

#[derive(Hash, PartialEq, Eq, Clone)]
struct AtlasKey {
    data: ImageVectorData,
    width: u32,
    height: u32,
}

impl AtlasKey {
    fn new(data: &ImageVectorData, width: u32, height: u32) -> Self {
        Self {
            data: data.clone(),
            width,
            height,
        }
    }
}

struct AtlasCacheEntry {
    uv_origin: [f32; 2],
    uv_scale: [f32; 2],
    page_index: usize,
    allocation: AllocId,
    last_used: u64,
}

struct SampleResources {
    buffer: tessera_ui::TrackedBuffer,
    bind_groups: HashMap<usize, wgpu::BindGroup>,
    bytes: Vec<u8>,
    used: usize,
}

#[derive(ShaderType, Clone, PartialEq, Copy)]
struct ImageVectorUniforms {
    origin: Vec2,
    scale: Vec2,
    tint: Vec4,
}

#[derive(ShaderType, Clone, PartialEq, Copy)]
struct AtlasSampleUniforms {
    origin: Vec2,
    scale: Vec2,
    uv_origin: Vec2,
    uv_scale: Vec2,
    tint: Vec4,
    tint_mode: u32,
    rotation: f32,
}

pub struct ImageVectorPipeline {
    raster_pipeline: wgpu::RenderPipeline,
    raster_bind_group: wgpu::BindGroup,
    sample_pipeline: wgpu::RenderPipeline,
    sample_bind_group_layout: wgpu::BindGroupLayout,
    atlas_sampler: wgpu::Sampler,
    atlas: VectorAtlas,
    cache: HashMap<AtlasKey, AtlasCacheEntry>,
    frame: u64,
    raster_sample_count: u32,
    sample_resources: Vec<SampleResources>,
    next_sample: usize,
    sample_stride: usize,
}

impl ImageVectorPipeline {
    pub fn new(
        device: &tessera_ui::GraphicsDevice,
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
                            has_dynamic_offset: true,
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
        let raster_uniform_buffer = device.create_buffer_init(
            "vector_geometry",
            &wgpu::util::BufferInitDescriptor {
                label: Some("image_vector_raster_uniform_buffer"),
                contents: &uniform_data.into_inner(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

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
        let vertex_layouts = [Some(vertex_layout)];

        let raster_sample_count = desired_raster_sample_count(sample_count);

        let raster_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Vector Raster Pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("image_vector_raster_pipeline_layout"),
                    bind_group_layouts: &[Some(&raster_bind_group_layout)],
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
                    bind_group_layouts: &[Some(&sample_bind_group_layout)],
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
            depth_stencil: tessera_ui::ui_stencil_state(),
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
            atlas: VectorAtlas::default(),
            cache: HashMap::new(),
            frame: 0,
            raster_sample_count,
            sample_resources: Vec::new(),
            next_sample: 0,
            sample_stride: (AtlasSampleUniforms::min_size().get() as usize)
                .next_multiple_of(device.limits().min_uniform_buffer_offset_alignment as usize),
        }
    }

    fn create_resources(
        device: &tessera_ui::GraphicsDevice,
        data: &ImageVectorData,
    ) -> GeometryResources {
        let vertex_buffer = device.create_buffer_init(
            "vector_geometry",
            &wgpu::util::BufferInitDescriptor {
                label: Some("image_vector_vertex_buffer"),
                contents: bytemuck::cast_slice(data.vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        let index_buffer = device.create_buffer_init(
            "vector_geometry",
            &wgpu::util::BufferInitDescriptor {
                label: Some("image_vector_index_buffer"),
                contents: bytemuck::cast_slice(data.indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            },
        );

        GeometryResources {
            vertex_buffer,
            index_buffer,
            index_count: data.indices.len() as u32,
        }
    }

    fn ensure_cached_entry(
        &mut self,
        device: &tessera_ui::GraphicsDevice,
        data: &ImageVectorData,
        width: u32,
        height: u32,
    ) {
        let key = AtlasKey::new(data, width, height);
        if let Some(entry) = self.cache.get_mut(&key) {
            entry.last_used = self.frame;
            self.atlas
                .pages
                .get_mut(&entry.page_index)
                .unwrap()
                .last_used = self.frame;
            return;
        }

        let geometry = Self::create_resources(device, data);
        let allocation = self.atlas.allocate(device, width, height);
        self.atlas
            .pages
            .get_mut(&allocation.page_index)
            .unwrap()
            .last_used = self.frame;
        self.rasterize_into_allocation(device, &geometry, &allocation, width, height);
        let page_size = self.atlas.page_texture(allocation.page_index).size();
        let uv_origin = [
            (allocation.region.rectangle.min.x as u32 + ATLAS_PADDING) as f32
                / page_size.width as f32,
            (allocation.region.rectangle.min.y as u32 + ATLAS_PADDING) as f32
                / page_size.height as f32,
        ];
        let uv_scale = [
            width as f32 / page_size.width as f32,
            height as f32 / page_size.height as f32,
        ];

        self.cache.insert(
            key,
            AtlasCacheEntry {
                uv_origin,
                uv_scale,
                page_index: allocation.page_index,
                allocation: allocation.region.id,
                last_used: self.frame,
            },
        );
    }

    fn rasterize_into_allocation(
        &mut self,
        device: &tessera_ui::GraphicsDevice,
        geometry: &GeometryResources,
        allocation: &AtlasAllocation,
        width: u32,
        height: u32,
    ) {
        let padded_width = width + ATLAS_PADDING * 2;
        let padded_height = height + ATLAS_PADDING * 2;
        let target = |sample_count, usage| {
            device.scratch_texture(tessera_ui::ScratchTextureDescriptor {
                width: padded_width,
                height: padded_height,
                format: ATLAS_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | usage,
                sample_count,
                fit: tessera_ui::ScratchFit::Approx,
            })
        };
        let resolved = target(1, wgpu::TextureUsages::COPY_SRC);
        let msaa = (self.raster_sample_count > 1)
            .then(|| target(self.raster_sample_count, wgpu::TextureUsages::empty()));
        let page = self.atlas.page_texture(allocation.page_index);
        device.encode_before_frame(|encoder| {
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("image_vector_raster_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: msaa.as_deref().unwrap_or(&resolved),
                        depth_slice: None,
                        resolve_target: msaa.as_ref().map(|_| &*resolved),
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
                pass.set_viewport(
                    ATLAS_PADDING as f32,
                    ATLAS_PADDING as f32,
                    width as f32,
                    height as f32,
                    0.0,
                    1.0,
                );
                pass.set_bind_group(0, &self.raster_bind_group, &[]);
                pass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
                pass.set_index_buffer(geometry.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..geometry.index_count, 0, 0..1);
            }
            encoder.copy_texture_to_texture(
                resolved.texture().as_image_copy(),
                wgpu::TexelCopyTextureInfo {
                    texture: page,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: allocation.region.rectangle.min.x as u32,
                        y: allocation.region.rectangle.min.y as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: padded_width,
                    height: padded_height,
                    depth_or_array_layers: 1,
                },
            );
        });
    }
}

impl DrawablePipeline<ImageVectorCommand> for ImageVectorPipeline {
    fn begin_frame(&mut self, _context: &FrameContext<'_>) {
        self.next_sample = 0;
        self.frame = self.frame.wrapping_add(1);
    }

    fn end_frame(&mut self, context: &FrameContext<'_>) {
        for resources in self
            .sample_resources
            .iter()
            .take(self.next_sample.div_ceil(SAMPLES_PER_BUFFER))
        {
            context
                .graphics
                .write_buffer(&resources.buffer, 0, &resources.bytes[..resources.used]);
        }
        self.cache.retain(|_, entry| {
            if self.frame.wrapping_sub(entry.last_used) <= RETAINED_FRAMES {
                return true;
            }
            self.atlas
                .pages
                .get_mut(&entry.page_index)
                .unwrap()
                .allocator
                .deallocate(entry.allocation);
            false
        });
        self.atlas
            .pages
            .retain(|_, page| !page.allocator.is_empty());
        let mut bytes: u64 = self.atlas.pages.values().map(AtlasPage::bytes).sum();
        while bytes > ATLAS_CACHE_BYTES {
            let oldest = self
                .atlas
                .pages
                .iter()
                .filter(|(_, page)| page.last_used != self.frame)
                .max_by_key(|(_, page)| self.frame.wrapping_sub(page.last_used))
                .map(|(&index, _)| index);
            let Some(index) = oldest else {
                // Pages sampled by this frame remain valid until submission.
                break;
            };
            bytes -= self.atlas.pages.remove(&index).unwrap().bytes();
        }
        self.cache
            .retain(|_, entry| self.atlas.pages.contains_key(&entry.page_index));
        for resources in &mut self.sample_resources {
            resources
                .bind_groups
                .retain(|index, _| self.atlas.pages.contains_key(index));
        }
    }

    fn draw(&mut self, context: &mut DrawContext<ImageVectorCommand>) {
        if context.commands.is_empty() {
            return;
        }

        for (command, geometry) in context.commands.iter() {
            if let Some((width, height)) = physical_dimensions(*geometry) {
                self.ensure_cached_entry(context.graphics, &command.data, width, height);
            }
        }

        context.render_pass.set_pipeline(&self.sample_pipeline);

        for (command, geometry) in context.commands.iter() {
            let Some((width, height)) = physical_dimensions(*geometry) else {
                continue;
            };
            let key = AtlasKey::new(&command.data, width, height);
            let Some(entry) = self.cache.get(&key) else {
                continue;
            };
            let uniforms = compute_sample_uniforms(command, *geometry, entry, context.target_size);
            let page_index = entry.page_index;
            let mut buffer = UniformBuffer::new(Vec::new());
            buffer
                .write(&uniforms)
                .expect("sample uniform serialization failed");

            let bytes = buffer.into_inner();
            let slot_index = self.next_sample / SAMPLES_PER_BUFFER;
            let offset = (self.next_sample % SAMPLES_PER_BUFFER) * self.sample_stride;
            self.next_sample += 1;
            if slot_index == self.sample_resources.len() {
                let buffer = context.graphics.create_buffer(
                    "vector_samples",
                    &wgpu::BufferDescriptor {
                        label: Some("image_vector_sample_uniform_buffer"),
                        size: (self.sample_stride * SAMPLES_PER_BUFFER) as u64,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    },
                );
                self.sample_resources.push(SampleResources {
                    buffer,
                    bind_groups: HashMap::new(),
                    bytes: vec![0; self.sample_stride * SAMPLES_PER_BUFFER],
                    used: 0,
                });
            }
            let resources = &mut self.sample_resources[slot_index];
            resources.bytes[offset..offset + bytes.len()].copy_from_slice(&bytes);
            resources.used = offset + bytes.len();
            let buffer = &resources.buffer;
            let bind_group = resources.bind_groups.entry(page_index).or_insert_with(|| {
                create_sample_bind_group(
                    context.graphics,
                    &self.sample_bind_group_layout,
                    self.atlas.page_view(page_index),
                    &self.atlas_sampler,
                    buffer,
                )
            });
            context.render_pass.set_bind_group(
                0,
                &*bind_group,
                &[u32::try_from(offset).expect("Vector sample offset")],
            );
            context.render_pass.draw(0..6, 0..1);
        }
    }
}

fn create_sample_bind_group(
    device: &tessera_ui::GraphicsDevice,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    size: NonZeroU64::new(AtlasSampleUniforms::min_size().get()),
                }),
            },
        ],
        label: Some("image_vector_sample_bind_group"),
    })
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

fn compute_sample_uniforms(
    command: &ImageVectorCommand,
    geometry: DrawGeometry,
    entry: &AtlasCacheEntry,
    target_size: PxSize,
) -> AtlasSampleUniforms {
    let start_pos = geometry.window_origin();
    let [width, height] = geometry.window_size();
    let left = (start_pos.x / target_size.width.to_f32()) * 2.0 - 1.0;
    let right = ((start_pos.x + width) / target_size.width.to_f32()) * 2.0 - 1.0;
    let top = 1.0 - (start_pos.y / target_size.height.to_f32()) * 2.0;
    let bottom = 1.0 - ((start_pos.y + height) / target_size.height.to_f32()) * 2.0;

    AtlasSampleUniforms {
        origin: Vec2::new(left, top),
        scale: Vec2::new(right - left, bottom - top),
        uv_origin: Vec2::from_array(entry.uv_origin),
        uv_scale: Vec2::from_array(entry.uv_scale),
        tint: Vec4::new(
            command.tint.r,
            command.tint.g,
            command.tint.b,
            command.tint.a,
        ),
        tint_mode: match command.tint_mode {
            VectorTintMode::Multiply => 0,
            VectorTintMode::Solid => 1,
        },
        rotation: command.rotation.to_radians(),
    }
}

fn physical_dimensions(geometry: DrawGeometry) -> Option<(u32, u32)> {
    let [width, height] = geometry.window_size();
    if width <= 0.0 || height <= 0.0 || width > u32::MAX as f32 || height > u32::MAX as f32 {
        None
    } else {
        Some((width.ceil() as u32, height.ceil() as u32))
    }
}

struct AtlasAllocation {
    page_index: usize,
    region: etagere::Allocation,
}

#[derive(Default)]
struct VectorAtlas {
    pages: BTreeMap<usize, AtlasPage>,
}

impl VectorAtlas {
    fn allocate(
        &mut self,
        device: &tessera_ui::GraphicsDevice,
        width: u32,
        height: u32,
    ) -> AtlasAllocation {
        let padded_width = width.saturating_add(ATLAS_PADDING * 2).max(1);
        let padded_height = height.saturating_add(ATLAS_PADDING * 2).max(1);
        let max_dimension = device.limits().max_texture_dimension_2d;

        if padded_width > max_dimension || padded_height > max_dimension {
            panic!(
                "Image vector target {}x{} exceeds GPU atlas limit {}",
                width, height, max_dimension
            );
        }

        let page_size = padded_width
            .max(padded_height)
            .next_power_of_two()
            .max(MIN_ATLAS_SIZE)
            .min(max_dimension);
        for (&index, page) in &mut self.pages {
            if page.texture.width() != page_size {
                continue;
            }
            if let Some(allocation) = page
                .allocator
                .allocate(size2(padded_width as i32, padded_height as i32))
            {
                return AtlasAllocation {
                    page_index: index,
                    region: allocation,
                };
            }
        }

        let mut page = AtlasPage::new(device, page_size);
        let allocation = page
            .allocator
            .allocate(size2(padded_width as i32, padded_height as i32))
            .expect("allocation should fit in a new page");
        let page_index = self.pages.keys().next_back().map_or(0, |index| index + 1);
        self.pages.insert(page_index, page);
        AtlasAllocation {
            page_index,
            region: allocation,
        }
    }

    fn page_view(&self, index: usize) -> &wgpu::TextureView {
        &self.pages[&index].view
    }

    fn page_texture(&self, index: usize) -> &wgpu::Texture {
        &self.pages[&index].texture
    }
}

struct AtlasPage {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    allocator: AtlasAllocator,
    last_used: u64,
}

impl AtlasPage {
    fn bytes(&self) -> u64 {
        u64::from(self.texture.width()) * u64::from(self.texture.height()) * 4
    }

    fn new(device: &tessera_ui::GraphicsDevice, size: u32) -> Self {
        let texture = device.create_texture(
            "vector_atlas",
            &wgpu::TextureDescriptor {
                label: Some("image_vector_atlas_page"),
                size: wgpu::Extent3d {
                    width: size,
                    height: size,
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
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            allocator: AtlasAllocator::new(size2(size as i32, size as i32)),
            last_used: 0,
        }
    }
}
