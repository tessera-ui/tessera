use tessera_ui::renderer::WgpuApp;

use crate::pipelines::{
    checkmark::pipeline::CheckmarkPipeline, fluid_glass::pipeline::FluidGlassPipeline,
    image::pipeline::ImagePipeline, image_vector::pipeline::ImageVectorPipeline,
    progress_arc::pipeline::ProgressArcPipeline, shape::pipeline::ShapePipeline,
    simple_rect::pipeline::SimpleRectPipeline, text::pipeline::GlyphonTextRender,
};

pub(super) fn register(app: &mut WgpuApp) {
    register_simple_rect(app);
    register_shape(app);
    register_progress_arc(app);
    register_checkmark(app);
    register_text(app);
    register_fluid_glass(app);
    register_image(app);
    register_image_vector(app);
}

fn register_simple_rect(app: &mut WgpuApp) {
    let pipeline = SimpleRectPipeline::new(
        &app.gpu,
        &app.config,
        app.pipeline_cache.as_ref(),
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_shape(app: &mut WgpuApp) {
    let pipeline = ShapePipeline::new(
        &app.gpu,
        &app.config,
        app.pipeline_cache.as_ref(),
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_progress_arc(app: &mut WgpuApp) {
    let pipeline = ProgressArcPipeline::new(
        &app.gpu,
        &app.config,
        app.pipeline_cache.as_ref(),
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_checkmark(app: &mut WgpuApp) {
    let pipeline = CheckmarkPipeline::new(
        &app.gpu,
        app.pipeline_cache.as_ref(),
        &app.config,
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_text(app: &mut WgpuApp) {
    let pipeline = GlyphonTextRender::new(&app.gpu, &app.queue, &app.config, app.sample_count);
    app.register_draw_pipeline(pipeline);
}

fn register_fluid_glass(app: &mut WgpuApp) {
    let pipeline = FluidGlassPipeline::new(
        &app.gpu,
        app.pipeline_cache.as_ref(),
        &app.config,
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_image(app: &mut WgpuApp) {
    let pipeline = ImagePipeline::new(
        &app.gpu,
        &app.config,
        app.pipeline_cache.as_ref(),
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_image_vector(app: &mut WgpuApp) {
    let pipeline = ImageVectorPipeline::new(
        &app.gpu,
        &app.config,
        app.pipeline_cache.as_ref(),
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}
