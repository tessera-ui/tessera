pub(crate) mod fluid_glass;
mod pos_misc;
mod shape;
mod text;
use self::shape::g2_corner::G2RoundedRectPipeline;
use self::shape::g2_corner_outline::G2RoundedOutlineRectPipeline;

pub use shape::{RippleProps, ShadowProps, ShapeCommand};
pub use text::{TextCommand, TextConstraint, TextData, read_font_system, write_font_system};

pub fn register_pipelines(app: &mut tessera::renderer::WgpuApp) {
    // Register shape pipeline
    let shape_pipeline = shape::ShapePipeline::new(&app.gpu, &app.config);
    app.drawer.pipeline_registry.register(shape_pipeline);
    // Register text pipeline
    let text_pipeline = text::GlyphonTextRender::new(&app.gpu, &app.queue, &app.config);
    app.drawer.pipeline_registry.register(text_pipeline);
    // Register fluid glass pipeline
    let fluid_glass_pipeline = fluid_glass::FluidGlassPipeline::new(&app.gpu, &app.config);
    // Register G2 rounded rect pipeline
    let g2_pipeline = G2RoundedRectPipeline::new(&app.gpu);
    app.compute_pipeline_registry.register(g2_pipeline);
    // Register G2 rounded outline rect pipeline
    let g2_outline_pipeline = G2RoundedOutlineRectPipeline::new(&app.gpu);
    app.compute_pipeline_registry.register(g2_outline_pipeline);
    app.drawer.pipeline_registry.register(fluid_glass_pipeline);
}
