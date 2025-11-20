use tessera_ui::renderer::WgpuApp;

use super::{checkmark, fluid_glass, image, image_vector, shape, simple_rect, text};

pub(super) fn register(app: &mut WgpuApp) {
    register_simple_rect(app);
    register_shape(app);
    register_checkmark(app);
    register_text(app);
    register_fluid_glass(app);
    register_image(app);
    register_image_vector(app);
}

fn register_simple_rect(app: &mut WgpuApp) {
    let pipeline = simple_rect::SimpleRectPipeline::new(
        &app.gpu,
        &app.config,
        app.pipeline_cache.as_ref(),
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_shape(app: &mut WgpuApp) {
    let pipeline = shape::ShapePipeline::new(
        &app.gpu,
        &app.config,
        app.pipeline_cache.as_ref(),
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_checkmark(app: &mut WgpuApp) {
    let pipeline = checkmark::CheckmarkPipeline::new(
        &app.gpu,
        app.pipeline_cache.as_ref(),
        &app.config,
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_text(app: &mut WgpuApp) {
    let pipeline =
        text::GlyphonTextRender::new(&app.gpu, &app.queue, &app.config, app.sample_count);
    app.register_draw_pipeline(pipeline);
}

fn register_fluid_glass(app: &mut WgpuApp) {
    let pipeline = fluid_glass::FluidGlassPipeline::new(
        &app.gpu,
        app.pipeline_cache.as_ref(),
        &app.config,
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_image(app: &mut WgpuApp) {
    let pipeline = image::ImagePipeline::new(
        &app.gpu,
        &app.config,
        app.pipeline_cache.as_ref(),
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}

fn register_image_vector(app: &mut WgpuApp) {
    let pipeline = image_vector::ImageVectorPipeline::new(
        &app.gpu,
        &app.config,
        app.pipeline_cache.as_ref(),
        app.sample_count,
    );
    app.register_draw_pipeline(pipeline);
}
