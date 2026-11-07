use std::sync::Arc;

use encase::{ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    PxSize,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline},
    wgpu::{self, include_wgsl},
};

use crate::pipelines::{
    SharedQuad,
    instanced::{QUAD_VERTICES, quad_pipeline},
};

use super::command::SimpleRectCommand;

#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
struct RectUniform {
    position: Vec4,
    color: Vec4,
    screen_size: Vec2,
}

#[derive(PartialEq, ShaderType)]
struct RectInstances {
    #[shader(size(runtime))]
    instances: Vec<RectUniform>,
}

/// Render pipeline for drawing batches of simple rectangles.
pub struct SimpleRectPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    quad_vertex_buffer: Arc<wgpu::Buffer>,
    quad_index_buffer: Arc<wgpu::Buffer>,
}

impl SimpleRectPipeline {
    /// Creates the rectangle pipeline with the provided surface configuration.
    pub fn new(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let shader = gpu.create_shader_module(include_wgsl!("simple_rect.wgsl"));

        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("simple_rect_bind_group_layout"),
        });

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Simple Rect Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = quad_pipeline(
            gpu,
            &pipeline_layout,
            &shader,
            config.format,
            sample_count,
            pipeline_cache,
            "Simple Rect Pipeline",
            Some(wgpu::Face::Back),
            QUAD_VERTICES,
        );

        let shared_quad = SharedQuad::get(gpu);

        Self {
            pipeline,
            bind_group_layout,
            quad_vertex_buffer: Arc::clone(shared_quad.vertex()),
            quad_index_buffer: Arc::clone(shared_quad.index()),
        }
    }
}

fn build_instances(
    commands: &[(&SimpleRectCommand, tessera_ui::DrawGeometry)],
    target_size: PxSize,
) -> Vec<RectUniform> {
    commands
        .iter()
        .map(|(command, geometry)| {
            let position = geometry.window_origin();
            let [width, height] = geometry.window_size();
            RectUniform {
                position: Vec4::new(position.x, position.y, width, height),
                color: Vec4::from_array(command.color.to_array()),
                screen_size: Vec2::new(target_size.width.to_f32(), target_size.height.to_f32()),
            }
        })
        .collect()
}

impl DrawablePipeline<SimpleRectCommand> for SimpleRectPipeline {
    fn draw(&mut self, context: &mut DrawContext<SimpleRectCommand>) {
        if context.commands.is_empty() {
            return;
        }
        let instances = build_instances(context.commands, context.target_size);

        let uniforms = RectInstances { instances };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&uniforms)
            .expect("buffer write failed");
        let bytes = buffer_content.into_inner();

        let buffer = context
            .graphics
            .frame_buffer(wgpu::BufferUsages::STORAGE, &bytes);

        let bind_group = context
            .graphics
            .cached_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.binding(),
                }],
                label: Some("simple_rect_bind_group"),
            });

        context.render_pass.set_pipeline(&self.pipeline);
        context.render_pass.set_bind_group(0, &bind_group, &[]);
        context
            .render_pass
            .set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        context
            .render_pass
            .set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        context
            .render_pass
            .draw_indexed(0..6, 0, 0..context.commands.len() as u32);
    }
}
