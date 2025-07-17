pub mod blur;
pub mod checkmark;
pub mod contrast;
pub(crate) mod fluid_glass;
pub mod mean;
mod pos_misc;
pub mod shape;
mod text;

pub mod image;

pub use checkmark::{CheckmarkCommand, CheckmarkPipeline};
pub use shape::{RippleProps, ShadowProps, ShapeCommand};
pub use text::{TextCommand, TextConstraint, TextData, read_font_system, write_font_system};

pub fn register_pipelines(app: &mut tessera_ui::renderer::WgpuApp) {
    // Register shape pipeline
    let shape_pipeline = shape::ShapePipeline::new(&app.gpu, &app.config, app.sample_count);
    app.drawer.pipeline_registry.register(shape_pipeline);
    // Register checkmark pipeline
    let checkmark_pipeline =
        checkmark::CheckmarkPipeline::new(&app.gpu, &app.config, app.sample_count);
    app.drawer.pipeline_registry.register(checkmark_pipeline);
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
    // Register blur pipeline
    let blur_pipeline = blur::pipeline::BlurPipeline::new(&app.gpu);
    app.compute_pipeline_registry.register(blur_pipeline);

    // Register mean pipeline
    let mean_pipeline = mean::MeanPipeline::new(&app.gpu);
    app.compute_pipeline_registry.register(mean_pipeline);

    // Register contrast pipeline
    let contrast_pipeline = contrast::ContrastPipeline::new(&app.gpu);
    app.compute_pipeline_registry.register(contrast_pipeline);
}
