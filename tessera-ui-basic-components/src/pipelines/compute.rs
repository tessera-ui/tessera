use tessera_ui::renderer::WgpuApp;

use super::{blur, contrast, mean};

pub(super) fn register(app: &mut WgpuApp) {
    register_blur(app);
    register_mean(app);
    register_contrast(app);
}

fn register_blur(app: &mut WgpuApp) {
    let pipeline = blur::pipeline::BlurPipeline::new(&app.gpu, app.pipeline_cache.as_ref());
    app.register_compute_pipeline(pipeline);
}

fn register_mean(app: &mut WgpuApp) {
    let pipeline = mean::MeanPipeline::new(&app.gpu, app.pipeline_cache.as_ref());
    app.register_compute_pipeline(pipeline);
}

fn register_contrast(app: &mut WgpuApp) {
    let pipeline = contrast::ContrastPipeline::new(&app.gpu, app.pipeline_cache.as_ref());
    app.register_compute_pipeline(pipeline);
}
