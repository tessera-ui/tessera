use std::sync::Arc;

use encase::{ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    PxSize,
    render::drawer::pipeline::{DrawContext, DrawablePipeline},
    wgpu::{self, include_wgsl},
};

use crate::pipelines::{
    SharedQuad,
    instanced::{QUAD_VERTICES, quad_pipeline, storage_layout},
};

use super::command::{ProgressArcCap, ProgressArcCommand};

#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
struct ArcUniform {
    position: Vec4,
    color: Vec4,
    screen_size: Vec2,
    stroke_width: f32,
    start_angle_degrees: f32,
    sweep_angle_degrees: f32,
    cap: u32,
    _pad: u32,
}

#[derive(PartialEq, ShaderType)]
struct ArcInstances {
    #[shader(size(runtime))]
    instances: Vec<ArcUniform>,
}

/// Render pipeline for drawing circular arc strokes.
pub struct ProgressArcPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    quad_vertex_buffer: Arc<wgpu::Buffer>,
    quad_index_buffer: Arc<wgpu::Buffer>,
}

impl ProgressArcPipeline {
    /// Creates the arc pipeline with the provided surface configuration.
    pub fn new(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let shader = gpu.create_shader_module(include_wgsl!("progress_arc.wgsl"));

        let bind_group_layout = storage_layout(gpu, "progress_arc_bind_group_layout");

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Progress Arc Pipeline Layout"),
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
            "Progress Arc Pipeline",
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
    commands: &[(&ProgressArcCommand, tessera_ui::render::DrawGeometry)],
    target_size: PxSize,
) -> Vec<ArcUniform> {
    commands
        .iter()
        .map(|(command, geometry)| {
            let position = geometry.window_origin();
            let [width, height] = geometry.window_size();
            let scale = geometry.local_to_window.scale();
            ArcUniform {
                position: Vec4::new(position.x, position.y, width, height),
                color: Vec4::from_array(command.color.to_array()),
                screen_size: Vec2::new(target_size.width.to_f32(), target_size.height.to_f32()),
                stroke_width: command.stroke_width_px * scale,
                start_angle_degrees: command.start_angle_degrees,
                sweep_angle_degrees: command.sweep_angle_degrees,
                cap: match command.cap {
                    ProgressArcCap::Round => 1,
                    ProgressArcCap::Butt => 0,
                },
                _pad: 0,
            }
        })
        .collect()
}

impl DrawablePipeline<ProgressArcCommand> for ProgressArcPipeline {
    fn draw(&mut self, context: &mut DrawContext<ProgressArcCommand>) {
        if context.commands.is_empty() {
            return;
        }

        let instances = build_instances(context.commands, context.target_size);

        let uniforms = ArcInstances { instances };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&uniforms)
            .expect("buffer write failed");
        let uniform_buffer = context
            .graphics
            .frame_buffer(wgpu::BufferUsages::STORAGE, buffer_content.as_ref());

        let bind_group = context
            .graphics
            .cached_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.binding(),
                }],
                label: Some("progress_arc_bind_group"),
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
