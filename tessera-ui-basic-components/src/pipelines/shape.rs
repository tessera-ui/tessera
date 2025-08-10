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

mod command;

use encase::{ArrayLength, ShaderSize, ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    PxPosition, PxSize,
    renderer::DrawablePipeline,
    wgpu::{self, include_wgsl},
};

use self::command::rect_to_uniforms;

pub use command::{RippleProps, ShadowProps, ShapeCommand};

// --- Uniforms ---
/// Uniforms for shape rendering pipeline.
///
/// # Fields
/// - `size_cr_border_width`: Size, corner radius, border width.
/// - `primary_color`: Main fill color.
/// - `shadow_color`: Shadow color.
/// - `render_params`: Additional rendering parameters.
/// - `ripple_params`: Ripple effect parameters.
/// - `ripple_color`: Ripple color.
/// - `g2_k_value`: G2 curve parameter for rounded rectangles.
///
/// # Example
/// ```
/// use tessera_ui_basic_components::pipelines::shape::ShapeUniforms;
/// let uniforms = ShapeUniforms {
///     size_cr_border_width: glam::Vec4::ZERO,
///     primary_color: glam::Vec4::ZERO,
///     shadow_color: glam::Vec4::ZERO,
///     render_params: glam::Vec4::ZERO,
///     ripple_params: glam::Vec4::ZERO,
///     ripple_color: glam::Vec4::ZERO,
///     g2_k_value: 0.0,
///     position: glam::Vec4::ZERO,
///     screen_size: glam::Vec2::ZERO,
/// };
/// ```
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
pub struct ShapeUniforms {
    pub size_cr_border_width: Vec4,
    pub primary_color: Vec4,
    pub shadow_color: Vec4,
    pub render_params: Vec4,
    pub ripple_params: Vec4,
    pub ripple_color: Vec4,
    pub g2_k_value: f32,
    pub position: Vec4,
    pub screen_size: Vec2,
}

#[derive(ShaderType)]
struct ShapeInstances {
    length: ArrayLength,
    #[size(runtime)]
    instances: Vec<ShapeUniforms>,
}

// Define MAX_CONCURRENT_SHAPES, can be adjusted later
pub const MAX_CONCURRENT_SHAPES: wgpu::BufferAddress = 1024;

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
    instances: Vec<ShapeUniforms>,
}

impl ShapePipeline {
    pub fn new(gpu: &wgpu::Device, config: &wgpu::SurfaceConfiguration, sample_count: u32) -> Self {
        let shader = gpu.create_shader_module(include_wgsl!("shape/shape.wgsl"));

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
                buffers: &[],
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            instances: Vec::with_capacity(MAX_CONCURRENT_SHAPES as usize),
        }
    }
}

#[allow(unused_variables)]
impl DrawablePipeline<ShapeCommand> for ShapePipeline {
    fn begin_pass(
        &mut self,
        _gpu: &wgpu::Device,
        _gpu_queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
        _render_pass: &mut wgpu::RenderPass<'_>,
        _scene_texture_view: &wgpu::TextureView,
    ) {
        self.instances.clear();
    }

    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &ShapeCommand,
        size: PxSize,
        start_pos: PxPosition,
        _scene_texture_view: &wgpu::TextureView,
    ) {
        if self.instances.len() >= MAX_CONCURRENT_SHAPES as usize {
            return; // Avoid buffer overflow
        }

        let mut uniforms = rect_to_uniforms(command, size, start_pos);
        uniforms.screen_size = [config.width as f32, config.height as f32].into();

        // Check if shadow needs to be drawn
        let has_shadow = uniforms.shadow_color[3] > 0.0 && uniforms.render_params[2] > 0.0;

        if has_shadow {
            let mut uniforms_for_shadow = uniforms;
            uniforms_for_shadow.render_params[3] = 2.0;
            self.instances.push(uniforms_for_shadow);
        }

        self.instances.push(uniforms);
    }

    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        _scene_texture_view: &wgpu::TextureView,
    ) {
        if self.instances.is_empty() {
            return;
        }

        let uniform_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shape Storage Buffer"),
            size: 16 + ShapeUniforms::SHADER_SIZE.get() * MAX_CONCURRENT_SHAPES,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms = ShapeInstances {
            length: Default::default(),
            instances: self.instances.clone(),
        };

        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content.write(&uniforms).unwrap();
        queue.write_buffer(&uniform_buffer, 0, buffer_content.as_ref());

        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("shape_bind_group"),
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..6, 0..self.instances.len() as u32);
    }
}
