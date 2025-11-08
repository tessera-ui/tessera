pub mod blur;
pub mod checkmark;
pub mod contrast;
pub(crate) mod fluid_glass;
pub mod mean;
mod pos_misc;
pub mod shape;
pub mod simple_rect;
pub mod text;

pub mod image;
pub mod image_vector;

pub use checkmark::{CheckmarkCommand, CheckmarkPipeline};
pub use image_vector::{ImageVectorCommand, ImageVectorPipeline};
pub use shape::{RippleProps, ShadowProps, ShapeCommand};
pub use simple_rect::{SimpleRectCommand, SimpleRectPipeline};
pub use text::{TextCommand, TextConstraint, TextData, read_font_system, write_font_system};

pub fn register_pipelines(app: &mut tessera_ui::renderer::WgpuApp) {
    let simple_rect_pipeline =
        simple_rect::SimpleRectPipeline::new(&app.gpu, &app.config, app.sample_count);
    app.register_draw_pipeline(simple_rect_pipeline);
    // Register shape pipeline
    let shape_pipeline = shape::ShapePipeline::new(&app.gpu, &app.config, app.sample_count);
    app.register_draw_pipeline(shape_pipeline);
    // Register checkmark pipeline
    let checkmark_pipeline =
        checkmark::CheckmarkPipeline::new(&app.gpu, &app.config, app.sample_count);
    app.register_draw_pipeline(checkmark_pipeline);
    // Register text pipeline
    let text_pipeline =
        text::GlyphonTextRender::new(&app.gpu, &app.queue, &app.config, app.sample_count);
    app.register_draw_pipeline(text_pipeline);
    // Register fluid glass pipeline
    let fluid_glass_pipeline =
        fluid_glass::FluidGlassPipeline::new(&app.gpu, &app.config, app.sample_count);
    app.register_draw_pipeline(fluid_glass_pipeline);
    // Register image pipeline
    let image_pipeline = image::ImagePipeline::new(&app.gpu, &app.config, app.sample_count);
    app.register_draw_pipeline(image_pipeline);
    // Register image vector pipeline
    let image_vector_pipeline =
        image_vector::ImageVectorPipeline::new(&app.gpu, &app.config, app.sample_count);
    app.register_draw_pipeline(image_vector_pipeline);
    // Register blur pipeline
    let blur_pipeline = blur::pipeline::BlurPipeline::new(&app.gpu);
    app.register_compute_pipeline(blur_pipeline);

    // Register mean pipeline
    let mean_pipeline = mean::MeanPipeline::new(&app.gpu);
    app.register_compute_pipeline(mean_pipeline);

    // Register contrast pipeline
    let contrast_pipeline = contrast::ContrastPipeline::new(&app.gpu);
    app.register_compute_pipeline(contrast_pipeline);
}
