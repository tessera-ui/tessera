mod command;
mod pos_misc;
mod shape;
mod text;

pub use crate::renderer::drawer::shape::{ShapeUniforms, Vertex as ShapeVertex};
pub use command::{DrawCommand, TextConstraint};
use pos_misc::pixel_to_ndc;
pub use text::{GlyphonTextRender, TextData};

/// Drawer is a struct that handles pipelines and draw commands.
pub struct Drawer {
    /// Shape pipeline
    shape_pipeline: shape::ShapePipeline,
    /// Text renderer
    pub text_renderer: GlyphonTextRender,
}

impl Drawer {
    /// Create a new drawer
    pub fn new(
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let shape_pipeline = shape::ShapePipeline::new(gpu, config);
        let text_renderer = GlyphonTextRender::new(gpu, queue, config);
        Self {
            shape_pipeline,
            text_renderer,
        }
    }

    /// Draw/Prepare Draw the command
    /// Some commands must be prepared before drawing
    /// and some commands can be drawn directly
    /// so we need to call both [Self::prepare_or_draw] and [Self::final_draw]
    pub fn prepare_or_draw(
        &mut self,
        gpu: &wgpu::Device, // Removed underscore as it's now used
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: DrawCommand,
    ) {
        match command {
            DrawCommand::Shape { vertices, uniforms } => {
                let positions: Vec<[f32; 2]> = vertices
                    .iter()
                    .map(|v| {
                        let pos = [v.position[0], v.position[1]];
                        pixel_to_ndc(pos, [config.width, config.height])
                    })
                    .collect();
                let colors: Vec<[f32; 3]> = vertices.iter().map(|v| v.color).collect();
                let local_positions: Vec<[f32; 2]> = vertices.iter().map(|v| v.local_pos).collect();

                // Two-pass drawing:
                // Pass 1: Draw Shadow (if shadow is enabled in uniforms)
                // A simple check: if shadow color alpha is > 0 and smoothness > 0
                let has_shadow = uniforms.shadow_color[3] > 0.0 && uniforms.shadow_params[2] > 0.0;

                if has_shadow {
                    let mut uniforms_for_shadow = uniforms; // uniforms is Copy
                    uniforms_for_shadow.size_cr_is_shadow[3] = 1.0; // Set is_shadow = true

                    self.shape_pipeline.draw(
                        gpu,
                        queue,
                        render_pass,
                        &positions,
                        &colors,
                        &local_positions,
                        &uniforms_for_shadow,
                    );
                }

                // Pass 2: Draw Object
                let mut uniforms_for_object = uniforms; // uniforms is Copy
                uniforms_for_object.size_cr_is_shadow[3] = 0.0; // Set is_shadow = false

                self.shape_pipeline.draw(
                    gpu,
                    queue,
                    render_pass,
                    &positions,
                    &colors,
                    &local_positions,
                    &uniforms_for_object,
                );
            }
            DrawCommand::Text { data } => {
                self.text_renderer.push(data);
            }
        }
    }

    /// Do the actual drawing for drawers that need to be prepared before drawing
    /// This should be called after all [Self::prepare_or_draw] calls
    /// this should only called once per render
    pub fn final_draw(
        &mut self,
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.text_renderer.draw(gpu, config, queue, render_pass);
    }
}
