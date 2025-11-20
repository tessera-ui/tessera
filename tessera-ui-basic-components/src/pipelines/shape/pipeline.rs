//! Shape rendering pipeline for UI components.
//!
//! This module provides the GPU pipeline and associated data structures for rendering
//! vector-based shapes in Tessera UI components. Supported shapes include rectangles,
//! rounded rectangles (with G2 curve support), ellipses, and arbitrary polygons.
//!
//! The pipeline supports advanced visual effects such as drop shadows and interactive
//! ripples, making it suitable for rendering button backgrounds, surfaces, and other
//! interactive or decorative UI elements.
//!
//! Typical usage scenarios include:
//! - Drawing backgrounds and outlines for buttons, surfaces, and containers
//! - Rendering custom-shaped UI elements with smooth corners
//! - Applying shadow and ripple effects for interactive feedback
//!
//! This module is intended to be used internally by basic UI components and registered
//! as part of the rendering pipeline system.

use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use encase::{ShaderSize, ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use lru::LruCache;
use tessera_ui::{
    Color, Px, PxPosition, PxSize,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline},
    wgpu::{self, include_wgsl, util::DeviceExt},
};

use super::command::{RippleProps, ShapeCommand, rect_to_uniforms};

#[allow(dead_code)]
pub const MAX_CONCURRENT_SHAPES: wgpu::BufferAddress = 1024;
const SHAPE_CACHE_CAPACITY: usize = 100;
/// Minimum number of frames a shape must appear before being cached.
/// This prevents caching transient shapes (e.g., resize animations).
const CACHE_HEAT_THRESHOLD: u32 = 3;
/// Number of frames to keep heat tracking data before cleanup.
const HEAT_TRACKING_WINDOW: u32 = 10;

type CachedInstanceBatch = Option<(Arc<ShapeCacheEntry>, Vec<(PxPosition, PxSize)>)>;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

/// Uniforms for shape rendering pipeline.
///
/// # Fields
///
/// - `size_cr_border_width`: Size, corner radius, border width.
/// - `primary_color`: Main fill color.
/// - `shadow_color`: Shadow color.
/// - `render_params`: Additional rendering parameters.
/// - `ripple_params`: Ripple effect parameters.
/// - `ripple_color`: Ripple color.
/// - `g2_k_value`: G2 curve parameter for rounded rectangles.
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
pub struct ShapeUniforms {
    pub corner_radii: Vec4, // x:tl, y:tr, z:br, w:bl
    pub primary_color: Vec4,
    pub border_color: Vec4,
    pub shadow_color: Vec4,
    pub render_params: Vec4,
    pub ripple_params: Vec4,
    pub ripple_color: Vec4,
    pub g2_k_value: f32,
    pub border_width: f32, // separate border_width field
    pub position: Vec4,    // x, y, width, height
    pub screen_size: Vec2,
}

#[derive(ShaderType)]
struct ShapeInstances {
    #[shader(size(runtime))]
    instances: Vec<ShapeUniforms>,
}

/// Tracks how frequently a shape appears to decide if it should be cached.
#[derive(Debug, Clone)]
struct ShapeHeatTracker {
    /// Number of frames this shape has appeared
    hit_count: u32,
    /// Frame number when last seen
    last_seen_frame: u32,
}

/// Pipeline for rendering vector shapes in UI components.
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui_basic_components::pipelines::shape::ShapePipeline;
///
/// let pipeline = ShapePipeline::new(&device, &config, sample_count);
/// ```
pub struct ShapePipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
    sample_count: u32,
    cache_sampler: wgpu::Sampler,
    cache_texture_bind_group_layout: wgpu::BindGroupLayout,
    cache_transform_bind_group_layout: wgpu::BindGroupLayout,
    cached_pipeline: wgpu::RenderPipeline,
    cache: LruCache<ShapeCacheKey, Arc<ShapeCacheEntry>>,
    /// Tracks shape usage frequency to avoid caching transient shapes
    heat_tracker: HashMap<ShapeCacheKey, ShapeHeatTracker>,
    /// Current frame number for heat tracking
    current_frame: u32,
    render_format: wgpu::TextureFormat,
}

impl ShapePipeline {
    /// Creates the shape rendering pipeline, configuring multisampling and caches.
    pub fn new(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let shader = gpu.create_shader_module(include_wgsl!("shape.wgsl"));

        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("shape_bind_group_layout"),
        });

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shape Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shape Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: pipeline_cache,
        });

        // Create a vertex buffer for a unit quad.
        let quad_vertices = [
            Vertex {
                position: [0.0, 0.0],
            }, // Top-left
            Vertex {
                position: [1.0, 0.0],
            }, // Top-right
            Vertex {
                position: [1.0, 1.0],
            }, // Bottom-right
            Vertex {
                position: [0.0, 1.0],
            }, // Bottom-left
        ];
        let quad_vertex_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shape Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create an index buffer for a unit quad.
        let quad_indices: [u16; 6] = [0, 2, 1, 0, 3, 2]; // CCW for backface culling
        let quad_index_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shape Quad Index Buffer"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let cache_sampler = gpu.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shape Cache Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let cache_texture_bind_group_layout =
            gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shape Cache Texture Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
            });

        let cache_transform_bind_group_layout =
            gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shape Cache Transform Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let cached_shader = gpu.create_shader_module(include_wgsl!("cached_quad.wgsl"));
        let cached_pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shape Cached Pipeline Layout"),
            bind_group_layouts: &[
                &cache_texture_bind_group_layout,
                &cache_transform_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let cached_pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shape Cached Pipeline"),
            layout: Some(&cached_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &cached_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &cached_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: pipeline_cache,
        });

        Self {
            pipeline,
            bind_group_layout,
            quad_vertex_buffer,
            quad_index_buffer,
            sample_count,
            cache_sampler,
            cache_texture_bind_group_layout,
            cache_transform_bind_group_layout,
            cached_pipeline,
            cache: LruCache::new(
                NonZeroUsize::new(SHAPE_CACHE_CAPACITY).expect("shape cache capacity must be > 0"),
            ),
            heat_tracker: HashMap::new(),
            current_frame: 0,
            render_format: config.format,
        }
    }

    fn get_or_create_cache_entry(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        command: &ShapeCommand,
        size: PxSize,
    ) -> Option<Arc<ShapeCacheEntry>> {
        let key = ShapeCacheKey::from_command(command, size)?;

        // Check if already cached
        if let Some(entry) = self.cache.get(&key) {
            return Some(entry.clone());
        }

        // Update heat tracking
        let tracker = self
            .heat_tracker
            .entry(key.clone())
            .or_insert(ShapeHeatTracker {
                hit_count: 0,
                last_seen_frame: self.current_frame,
            });

        // Update tracker
        if tracker.last_seen_frame != self.current_frame {
            tracker.hit_count += 1;
            tracker.last_seen_frame = self.current_frame;
        }

        // Only cache if shape has appeared frequently enough
        if tracker.hit_count >= CACHE_HEAT_THRESHOLD {
            let entry = Arc::new(self.build_cache_entry(gpu, gpu_queue, command, size));
            self.cache.put(key, entry.clone());
            Some(entry)
        } else {
            // Shape is not hot enough yet, don't cache
            None
        }
    }

    fn build_cache_entry(
        &self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        command: &ShapeCommand,
        size: PxSize,
    ) -> ShapeCacheEntry {
        let width = size.width.positive().max(1);
        let height = size.height.positive().max(1);

        let cache_texture = gpu.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shape Cache Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.render_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let cache_view = cache_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut uniforms = rect_to_uniforms(
            command,
            size,
            PxPosition {
                x: Px::new(0),
                y: Px::new(0),
            },
        );
        uniforms.screen_size = [width as f32, height as f32].into();

        let has_shadow = uniforms.shadow_color[3] > 0.0 && uniforms.render_params[2] > 0.0;
        let mut instances = Vec::with_capacity(if has_shadow { 2 } else { 1 });
        if has_shadow {
            let mut shadow = uniforms;
            shadow.render_params[3] = 2.0;
            instances.push(shadow);
        }
        instances.push(uniforms);

        let storage_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shape Cache Storage Buffer"),
            size: 16 + ShapeUniforms::SHADER_SIZE.get() * instances.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms = ShapeInstances { instances };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&uniforms)
            .expect("buffer write failed");
        gpu_queue.write_buffer(&storage_buffer, 0, buffer_content.as_ref());

        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
            label: Some("shape_cache_bind_group"),
        });

        let mut encoder = gpu.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Shape Cache Encoder"),
        });

        let run_pass = |pass: &mut wgpu::RenderPass<'_>| {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..6, 0, 0..uniforms.instances.len() as u32);
        };

        if self.sample_count > 1 {
            let msaa_texture = gpu.create_texture(&wgpu::TextureDescriptor {
                label: Some("Shape Cache MSAA Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: self.render_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Shape Cache Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &msaa_view,
                        resolve_target: Some(&cache_view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                run_pass(&mut pass);
            }
        } else {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shape Cache Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &cache_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            run_pass(&mut pass);
        }

        gpu_queue.submit(Some(encoder.finish()));

        let texture_bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.cache_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.cache_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cache_view),
                },
            ],
            label: Some("shape_cache_texture_bind_group"),
        });

        ShapeCacheEntry {
            _texture: cache_texture,
            _view: cache_view,
            texture_bind_group,
        }
    }

    fn draw_uncached_batch(
        &self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        commands: &[(&ShapeCommand, PxSize, PxPosition)],
        indices: &[usize],
    ) {
        if indices.is_empty() {
            return;
        }

        let subset: Vec<_> = indices.iter().map(|&i| commands[i]).collect();
        let instances = build_instances(&subset, config);
        if instances.is_empty() {
            return;
        }

        let storage_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shape Storage Buffer"),
            size: 16 + ShapeUniforms::SHADER_SIZE.get() * instances.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms = ShapeInstances { instances };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&uniforms)
            .expect("buffer write failed");
        gpu_queue.write_buffer(&storage_buffer, 0, buffer_content.as_ref());

        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
            label: Some("shape_bind_group"),
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..uniforms.instances.len() as u32);
    }

    fn draw_cached_run(
        &self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        entry: Arc<ShapeCacheEntry>,
        instances: &[(PxPosition, PxSize)],
    ) {
        if instances.is_empty() {
            return;
        }

        let rects: Vec<CachedRectUniform> = instances
            .iter()
            .map(|(position, size)| CachedRectUniform {
                position: Vec4::new(
                    position.x.raw() as f32,
                    position.y.raw() as f32,
                    size.width.raw() as f32,
                    size.height.raw() as f32,
                ),
                screen_size: Vec2::new(config.width as f32, config.height as f32),
                padding: Vec2::ZERO,
            })
            .collect();

        let rect_instances = CachedRectInstances { rects };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&rect_instances)
            .expect("buffer write failed");

        let instance_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shape Cache Instance Buffer"),
            size: buffer_content.as_ref().len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu_queue.write_buffer(&instance_buffer, 0, buffer_content.as_ref());

        let transform_bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.cache_transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: instance_buffer.as_entire_binding(),
            }],
            label: Some("shape_cache_transform_bind_group"),
        });

        render_pass.set_pipeline(&self.cached_pipeline);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &entry.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &transform_bind_group, &[]);
        render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
    }

    fn flush_cached_run(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        pending: &mut CachedInstanceBatch,
    ) {
        if let Some((entry, instances)) = pending.take() {
            self.draw_cached_run(gpu, gpu_queue, config, render_pass, entry, &instances);
        }
    }
}

fn build_instances(
    commands: &[(&ShapeCommand, PxSize, PxPosition)],
    config: &wgpu::SurfaceConfiguration,
) -> Vec<ShapeUniforms> {
    // Extracted instance-building logic to simplify `draw` and reduce cognitive complexity.
    commands
        .iter()
        .flat_map(|(command, size, start_pos)| {
            let mut uniforms = rect_to_uniforms(command, *size, *start_pos);
            uniforms.screen_size = [config.width as f32, config.height as f32].into();

            let has_shadow = uniforms.shadow_color[3] > 0.0 && uniforms.render_params[2] > 0.0;

            if has_shadow {
                let mut uniforms_for_shadow = uniforms;
                uniforms_for_shadow.render_params[3] = 2.0;
                vec![uniforms_for_shadow, uniforms]
            } else {
                vec![uniforms]
            }
        })
        .collect()
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum ShapeCacheVariant {
    Rect,
    OutlinedRect,
    FilledOutlinedRect,
    Ellipse,
    OutlinedEllipse,
    FilledOutlinedEllipse,
    RippleRect,
    RippleOutlinedRect,
    RippleFilledOutlinedRect,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ShadowKey {
    color: [u32; 4],
    offset: [u32; 2],
    smoothness: u32,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct RippleKey {
    center: [u32; 2],
    radius: u32,
    alpha: u32,
    color: [u32; 4],
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ShapeCacheKey {
    variant: ShapeCacheVariant,
    primary_color: [u32; 4],
    border_color: Option<[u32; 4]>,
    corner_radii: [u32; 4],
    g2_k_value: u32,
    border_width: u32,
    shadow: Option<ShadowKey>,
    ripple: Option<RippleKey>,
    width: u32,
    height: u32,
}

struct ShapeCacheEntry {
    _texture: wgpu::Texture,
    _view: wgpu::TextureView,
    texture_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
struct CachedRectUniform {
    position: Vec4,
    screen_size: Vec2,
    padding: Vec2,
}

#[derive(ShaderType)]
struct CachedRectInstances {
    #[shader(size(runtime))]
    rects: Vec<CachedRectUniform>,
}

fn f32_to_bits(value: f32) -> u32 {
    value.to_bits()
}

fn color_to_bits(color: Color) -> [u32; 4] {
    let arr = color.to_array();
    [
        f32_to_bits(arr[0]),
        f32_to_bits(arr[1]),
        f32_to_bits(arr[2]),
        f32_to_bits(arr[3]),
    ]
}

fn ripple_to_key(ripple: &RippleProps) -> RippleKey {
    RippleKey {
        center: [f32_to_bits(ripple.center[0]), f32_to_bits(ripple.center[1])],
        radius: f32_to_bits(ripple.radius),
        alpha: f32_to_bits(ripple.alpha),
        color: color_to_bits(ripple.color),
    }
}

impl ShapeCacheKey {
    fn from_command(command: &ShapeCommand, size: PxSize) -> Option<Self> {
        let width = size.width.positive();
        let height = size.height.positive();
        if width == 0 || height == 0 {
            return None;
        }

        match command {
            ShapeCommand::Rect {
                color,
                corner_radii,
                g2_k_value,
                shadow,
            } => Some(Self {
                variant: ShapeCacheVariant::Rect,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: corner_radii.map(f32_to_bits),
                g2_k_value: f32_to_bits(*g2_k_value),
                border_width: 0,
                shadow: shadow.as_ref().map(|shadow| ShadowKey {
                    color: color_to_bits(shadow.color),
                    offset: [f32_to_bits(shadow.offset[0]), f32_to_bits(shadow.offset[1])],
                    smoothness: f32_to_bits(shadow.smoothness),
                }),
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::OutlinedRect {
                color,
                corner_radii,
                g2_k_value,
                shadow,
                border_width,
            } => Some(Self {
                variant: ShapeCacheVariant::OutlinedRect,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: corner_radii.map(f32_to_bits),
                g2_k_value: f32_to_bits(*g2_k_value),
                border_width: f32_to_bits(*border_width),
                shadow: shadow.as_ref().map(|shadow| ShadowKey {
                    color: color_to_bits(shadow.color),
                    offset: [f32_to_bits(shadow.offset[0]), f32_to_bits(shadow.offset[1])],
                    smoothness: f32_to_bits(shadow.smoothness),
                }),
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::FilledOutlinedRect {
                color,
                border_color,
                corner_radii,
                g2_k_value,
                shadow,
                border_width,
            } => Some(Self {
                variant: ShapeCacheVariant::FilledOutlinedRect,
                primary_color: color_to_bits(*color),
                border_color: Some(color_to_bits(*border_color)),
                corner_radii: corner_radii.map(f32_to_bits),
                g2_k_value: f32_to_bits(*g2_k_value),
                border_width: f32_to_bits(*border_width),
                shadow: shadow.as_ref().map(|shadow| ShadowKey {
                    color: color_to_bits(shadow.color),
                    offset: [f32_to_bits(shadow.offset[0]), f32_to_bits(shadow.offset[1])],
                    smoothness: f32_to_bits(shadow.smoothness),
                }),
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::Ellipse { color, shadow } => Some(Self {
                variant: ShapeCacheVariant::Ellipse,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: [f32_to_bits(-1.0_f32); 4],
                g2_k_value: f32_to_bits(0.0),
                border_width: 0,
                shadow: shadow.as_ref().map(|shadow| ShadowKey {
                    color: color_to_bits(shadow.color),
                    offset: [f32_to_bits(shadow.offset[0]), f32_to_bits(shadow.offset[1])],
                    smoothness: f32_to_bits(shadow.smoothness),
                }),
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::OutlinedEllipse {
                color,
                shadow,
                border_width,
            } => Some(Self {
                variant: ShapeCacheVariant::OutlinedEllipse,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: [f32_to_bits(-1.0_f32); 4],
                g2_k_value: f32_to_bits(0.0),
                border_width: f32_to_bits(*border_width),
                shadow: shadow.as_ref().map(|shadow| ShadowKey {
                    color: color_to_bits(shadow.color),
                    offset: [f32_to_bits(shadow.offset[0]), f32_to_bits(shadow.offset[1])],
                    smoothness: f32_to_bits(shadow.smoothness),
                }),
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::FilledOutlinedEllipse {
                color,
                border_color,
                shadow,
                border_width,
            } => Some(Self {
                variant: ShapeCacheVariant::FilledOutlinedEllipse,
                primary_color: color_to_bits(*color),
                border_color: Some(color_to_bits(*border_color)),
                corner_radii: [f32_to_bits(-1.0_f32); 4],
                g2_k_value: f32_to_bits(0.0),
                border_width: f32_to_bits(*border_width),
                shadow: shadow.as_ref().map(|shadow| ShadowKey {
                    color: color_to_bits(shadow.color),
                    offset: [f32_to_bits(shadow.offset[0]), f32_to_bits(shadow.offset[1])],
                    smoothness: f32_to_bits(shadow.smoothness),
                }),
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::RippleRect {
                color,
                corner_radii,
                g2_k_value,
                shadow,
                ripple,
            } if ripple.alpha.abs() <= f32::EPSILON => Some(Self {
                variant: ShapeCacheVariant::RippleRect,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: corner_radii.map(f32_to_bits),
                g2_k_value: f32_to_bits(*g2_k_value),
                border_width: 0,
                shadow: shadow.as_ref().map(|shadow| ShadowKey {
                    color: color_to_bits(shadow.color),
                    offset: [f32_to_bits(shadow.offset[0]), f32_to_bits(shadow.offset[1])],
                    smoothness: f32_to_bits(shadow.smoothness),
                }),
                ripple: Some(ripple_to_key(ripple)),
                width,
                height,
            }),
            ShapeCommand::RippleOutlinedRect {
                color,
                corner_radii,
                g2_k_value,
                shadow,
                border_width,
                ripple,
            } if ripple.alpha.abs() <= f32::EPSILON => Some(Self {
                variant: ShapeCacheVariant::RippleOutlinedRect,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: corner_radii.map(f32_to_bits),
                g2_k_value: f32_to_bits(*g2_k_value),
                border_width: f32_to_bits(*border_width),
                shadow: shadow.as_ref().map(|shadow| ShadowKey {
                    color: color_to_bits(shadow.color),
                    offset: [f32_to_bits(shadow.offset[0]), f32_to_bits(shadow.offset[1])],
                    smoothness: f32_to_bits(shadow.smoothness),
                }),
                ripple: Some(ripple_to_key(ripple)),
                width,
                height,
            }),
            ShapeCommand::RippleFilledOutlinedRect {
                color,
                border_color,
                corner_radii,
                g2_k_value,
                shadow,
                border_width,
                ripple,
            } if ripple.alpha.abs() <= f32::EPSILON => Some(Self {
                variant: ShapeCacheVariant::RippleFilledOutlinedRect,
                primary_color: color_to_bits(*color),
                border_color: Some(color_to_bits(*border_color)),
                corner_radii: corner_radii.map(f32_to_bits),
                g2_k_value: f32_to_bits(*g2_k_value),
                border_width: f32_to_bits(*border_width),
                shadow: shadow.as_ref().map(|shadow| ShadowKey {
                    color: color_to_bits(shadow.color),
                    offset: [f32_to_bits(shadow.offset[0]), f32_to_bits(shadow.offset[1])],
                    smoothness: f32_to_bits(shadow.smoothness),
                }),
                ripple: Some(ripple_to_key(ripple)),
                width,
                height,
            }),
            _ => None,
        }
    }
}

impl DrawablePipeline<ShapeCommand> for ShapePipeline {
    fn draw(&mut self, context: &mut DrawContext<ShapeCommand>) {
        if context.commands.is_empty() {
            return;
        }

        // Advance frame counter and cleanup old heat tracking data
        self.current_frame = self.current_frame.wrapping_add(1);
        self.heat_tracker.retain(|_, tracker| {
            // Remove entries not seen in the last HEAT_TRACKING_WINDOW frames
            self.current_frame.saturating_sub(tracker.last_seen_frame) < HEAT_TRACKING_WINDOW
        });

        let mut cache_entries = Vec::with_capacity(context.commands.len());
        for (command, size, _) in context.commands.iter() {
            let entry =
                self.get_or_create_cache_entry(context.device, context.queue, command, *size);
            cache_entries.push(entry);
        }

        let mut pending_uncached: Vec<usize> = Vec::new();
        let mut pending_cached_run: CachedInstanceBatch = None;

        for (idx, ((_, size, position), cache_entry)) in context
            .commands
            .iter()
            .zip(cache_entries.iter())
            .enumerate()
        {
            if let Some(entry) = cache_entry {
                if !pending_uncached.is_empty() {
                    self.draw_uncached_batch(
                        context.device,
                        context.queue,
                        context.config,
                        context.render_pass,
                        context.commands,
                        &pending_uncached,
                    );
                    pending_uncached.clear();
                }

                if let Some((current_entry, transforms)) = pending_cached_run.as_mut() {
                    if Arc::ptr_eq(current_entry, entry) {
                        transforms.push((*position, *size));
                    } else {
                        self.flush_cached_run(
                            context.device,
                            context.queue,
                            context.config,
                            context.render_pass,
                            &mut pending_cached_run,
                        );
                        pending_cached_run = Some((entry.clone(), vec![(*position, *size)]));
                    }
                } else {
                    pending_cached_run = Some((entry.clone(), vec![(*position, *size)]));
                }
            } else {
                self.flush_cached_run(
                    context.device,
                    context.queue,
                    context.config,
                    context.render_pass,
                    &mut pending_cached_run,
                );
                pending_uncached.push(idx);
            }
        }

        self.flush_cached_run(
            context.device,
            context.queue,
            context.config,
            context.render_pass,
            &mut pending_cached_run,
        );

        if !pending_uncached.is_empty() {
            self.draw_uncached_batch(
                context.device,
                context.queue,
                context.config,
                context.render_pass,
                context.commands,
                &pending_uncached,
            );
        }
    }
}
