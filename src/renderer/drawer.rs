mod command;
mod pos_misc;
mod shape;
mod text;

pub use command::{DrawCommand, ShapeVertex, TextConstraint};
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
                data,
            } => {
                self.text_renderer
                    .push(data);
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
