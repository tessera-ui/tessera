pub(crate) mod fluid_glass;
mod pos_misc;
mod shape;
mod text;

pub mod image;

pub use shape::{RippleProps, ShadowProps, ShapeCommand};
pub use text::{TextCommand, TextConstraint, TextData, read_font_system, write_font_system};

pub fn register_pipelines(app: &mut tessera::renderer::WgpuApp) {
    // Register shape pipeline
    let shape_pipeline = shape::ShapePipeline::new(&app.gpu, &app.config, app.sample_count);
    app.drawer.pipeline_registry.register(shape_pipeline);
    // Register text pipeline
    let text_pipeline =
        text::GlyphonTextRender::new(&app.gpu, &app.queue, &app.config, app.sample_count);
    app.drawer.pipeline_registry.register(text_pipeline);
    // Register fluid glass pipeline
    let fluid_glass_pipeline =
        fluid_glass::FluidGlassPipeline::new(&app.gpu, &app.config, app.sample_count);
    app.drawer.pipeline_registry.register(fluid_glass_pipeline);
    // Register image pipeline
    let image_pipeline = image::ImagePipeline::new(&app.gpu, &app.config, app.sample_count);
    app.drawer.pipeline_registry.register(image_pipeline);
}
