mod pos_misc;
mod shape;
mod text;

pub use shape::{RippleProps, ShadowProps, ShapeCommand};
pub use text::{TextCommand, TextConstraint, TextData, read_font_system, write_font_system};

pub fn register_pipelines(
    gpu: &wgpu::Device,
    gpu_queue: &wgpu::Queue,
    config: &wgpu::SurfaceConfiguration,
    pipelines: &mut tessera::PipelineRegistry,
) {
    // Register shape pipeline
    let shape_pipeline = shape::ShapePipeline::new(gpu, config);
    pipelines.register(shape_pipeline);
    // Register text pipeline
    let text_pipeline = text::GlyphonTextRender::new(gpu, gpu_queue, config);
    pipelines.register(text_pipeline);
}
