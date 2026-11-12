//! Shape rendering pipeline for UI components.
//!
//! ## Usage
//!
//! Draw rectangles, ellipses, ripples, and shadows for component surfaces.

mod draw;

use std::sync::Arc;

use encase::ShaderType;
use glam::{Vec2, Vec4};
use tessera_ui::{
    render::{DrawContext, DrawablePipeline, FrameContext},
    wgpu::{self, include_wgsl},
};

use crate::pipelines::SharedQuad;

use super::command::ShapeCommand;

/// Uniforms for shape rendering pipeline.
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
pub struct ShapeUniforms {
    pub corner_radii: Vec4, // x:tl, y:tr, z:br, w:bl
    pub corner_g2: Vec4,    // x:tl, y:tr, z:br, w:bl
    pub primary_color: Vec4,
    pub border_color: Vec4,
    pub render_mode: f32,
    pub ripple_params: Vec4,
    pub ripple_color: Vec4,
    pub border_width: f32,
    pub position: Vec4, // x, y, width, height
    pub screen_size: Vec2,
}

#[derive(PartialEq, ShaderType)]
struct ShapeInstances {
    #[shader(size(runtime))]
    instances: Vec<ShapeUniforms>,
}

/// Pipeline for rendering vector shapes in UI components.
pub struct ShapePipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    quad_vertex_buffer: Arc<wgpu::Buffer>,
    quad_index_buffer: Arc<wgpu::Buffer>,
    batches: draw::BatchResources,
}

impl ShapePipeline {
    /// Creates the shape rendering pipeline for the target format and sample
    /// count.
    pub fn new(
        gpu: &tessera_ui::render::GraphicsDevice,
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
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shape Pipeline"),
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
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: tessera_ui::render::ui_stencil_state(),
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
            multiview_mask: None,
            cache: pipeline_cache,
        });

        let shared_quad = SharedQuad::get(gpu);

        Self {
            pipeline,
            bind_group_layout,
            quad_vertex_buffer: Arc::clone(shared_quad.vertex()),
            quad_index_buffer: Arc::clone(shared_quad.index()),
            batches: draw::BatchResources::default(),
        }
    }
}

impl DrawablePipeline<ShapeCommand> for ShapePipeline {
    fn begin_frame(&mut self, _: &FrameContext<'_>) {
        self.batches.begin_frame();
    }

    fn end_frame(&mut self, context: &FrameContext<'_>) {
        self.batches.finish_frame(context.graphics);
    }

    fn draw(&mut self, context: &mut DrawContext<ShapeCommand>) {
        if context.commands.is_empty() {
            return;
        }
        self.draw_batch(
            context.graphics,
            context.target_size,
            context.render_pass,
            context.commands,
        );
    }
}
