//! Shape rendering pipeline for UI components.
//!
//! This module provides the GPU pipeline and associated data structures for
//! rendering vector-based shapes in Tessera UI components. Supported shapes
//! include rectangles, rounded rectangles (with G2 curve support), ellipses,
//! and arbitrary polygons.
//!
//! The pipeline supports advanced visual effects such as drop shadows and
//! interactive ripples, making it suitable for rendering button backgrounds,
//! surfaces, and other interactive or decorative UI elements.
//!
//! Typical usage scenarios include:
//! - Drawing backgrounds and outlines for buttons, surfaces, and containers
//! - Rendering custom-shaped UI elements with smooth corners
//! - Applying shadow and ripple effects for interactive feedback
//!
//! This module is intended to be used internally by basic UI components and
//! registered as part of the rendering pipeline system.

mod cache;
mod draw;

use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use encase::ShaderType;
use glam::{Vec2, Vec3, Vec4};
use lru::LruCache;
use tessera_ui::{
    PxPosition, PxSize,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline},
    wgpu::{self, include_wgsl, util::DeviceExt},
};

use self::cache::{ShapeCacheKey, ShapeHeatTracker};
use super::command::ShapeCommand;

#[allow(dead_code)]
pub const MAX_CONCURRENT_SHAPES: wgpu::BufferAddress = 1024;
const SHAPE_CACHE_CAPACITY: usize = 100;
/// Minimum number of frames a shape must appear before being cached.
/// This prevents caching transient shapes (e.g., resize animations).
const CACHE_HEAT_THRESHOLD: u32 = 3;
/// Number of frames to keep heat tracking data before cleanup.
const HEAT_TRACKING_WINDOW: u32 = 10;

type CachedInstanceBatch = Option<(Arc<ShapeCacheEntry>, Vec<(PxPosition, PxSize)>)>;

struct ShapeCacheEntry {
    _texture: wgpu::Texture,
    _view: wgpu::TextureView,
    texture_bind_group: wgpu::BindGroup,
    padding: Vec2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

/// Uniforms for shape rendering pipeline.
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
pub struct ShapeUniforms {
    pub corner_radii: Vec4, // x:tl, y:tr, z:br, w:bl
    pub corner_g2: Vec4,    // x:tl, y:tr, z:br, w:bl
    pub primary_color: Vec4,
    pub border_color: Vec4,
    pub shadow_ambient_color: Vec4,
    pub shadow_ambient_params: Vec3, // x:y offset, z: smoothness, w: unused
    pub shadow_spot_color: Vec4,
    pub shadow_spot_params: Vec3, // x:y offset, z: smoothness, w: unused
    pub render_mode: f32,
    pub ripple_params: Vec4,
    pub ripple_color: Vec4,
    pub border_width: f32,
    pub position: Vec4, // x, y, width, height
    pub screen_size: Vec2,
}

#[derive(ShaderType)]
struct ShapeInstances {
    #[shader(size(runtime))]
    instances: Vec<ShapeUniforms>,
}

/// Pipeline for rendering vector shapes in UI components.
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
    heat_tracker: HashMap<ShapeCacheKey, ShapeHeatTracker>,
    current_frame: u32,
    render_format: wgpu::TextureFormat,
}

impl ShapePipeline {
    /// Creates the shape rendering pipeline, configuring multisampling and
    /// caches.
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

        let quad_vertices = [
            Vertex {
                position: [0.0, 0.0],
            },
            Vertex {
                position: [1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
            Vertex {
                position: [0.0, 1.0],
            },
        ];
        let quad_vertex_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shape Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_indices: [u16; 6] = [0, 2, 1, 0, 3, 2];
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
}

impl DrawablePipeline<ShapeCommand> for ShapePipeline {
    fn draw(&mut self, context: &mut DrawContext<ShapeCommand>) {
        if context.commands.is_empty() {
            return;
        }

        self.current_frame = self.current_frame.wrapping_add(1);
        self.heat_tracker.retain(|_, tracker| {
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
