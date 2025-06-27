mod glass;
mod pos_misc;
mod shape;
mod text;

pub use glass::GlassCommand;
pub use shape::{RippleProps, ShadowProps, ShapeCommand};
pub use text::{read_font_system, write_font_system, TextCommand, TextConstraint, TextData};

pub fn register_pipelines(app: &mut tessera::renderer::WgpuApp) {
    // Register shape pipeline
    let shape_pipeline = shape::ShapePipeline::new(&app.gpu, &app.config);
    app.drawer
        .pipeline_registry
        .register(shape_pipeline);
    // Register text pipeline
    let text_pipeline = text::GlyphonTextRender::new(&app.gpu, &app.queue, &app.config);
    app.drawer
        .pipeline_registry
        .register(text_pipeline);
    // Register glass pipeline
    let glass_pipeline = glass::GlassPipeline::new(&app.gpu, &app.config, &app.background_bind_group_layout);
    app.drawer
        .pipeline_registry
        .register(glass_pipeline);
}
