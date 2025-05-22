mod command;
mod pos_misc;
mod shape;
mod text;

pub use command::{DrawCommand, ShapeVertex};
use pos_misc::pixel_to_ndc;
use text::GlyphonTextRender;

/// Drawer is a struct that handles pipelines and draw commands.
pub struct Drawer {
    /// Shape pipeline
    shape_pipeline: shape::ShapePipeline,
    /// Text renderer
    text_renderer: GlyphonTextRender,
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

    /// Draw the command
    pub fn draw(
        &mut self,
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: DrawCommand,
    ) {
        match command {
            DrawCommand::Shape { vertices } => {
                let colors = vertices.iter().map(|v| v.color).collect();
                let positions = vertices
                    .iter()
                    .map(|v| pixel_to_ndc(v.position, [config.width, config.height]))
                    .collect();
                self.shape_pipeline
                    .draw(gpu, render_pass, positions, colors);
            }
            DrawCommand::Text {
                text,
                position,
                color,
                size,
                line_height,
            } => {
                self.text_renderer.draw(
                    gpu,
                    config,
                    queue,
                    render_pass,
                    &text,
                    pixel_to_ndc(position, [config.width, config.height]),
                    color,
                    size,
                    line_height,
                );
            }
        }
    }
}
