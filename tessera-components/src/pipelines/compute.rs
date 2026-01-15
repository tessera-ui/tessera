use tessera_ui::PipelineContext;

use crate::pipelines::{
    blur::pipeline::BlurPipeline, contrast::ContrastPipeline, mean::pipeline::MeanPipeline,
};

pub(super) fn register(context: &mut PipelineContext<'_>) {
    register_blur(context);
    register_mean(context);
    register_contrast(context);
}

fn register_blur(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = BlurPipeline::new(resources.device, resources.pipeline_cache);
    context.register_compute_pipeline(pipeline);
}

fn register_mean(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = MeanPipeline::new(resources.device, resources.pipeline_cache);
    context.register_compute_pipeline(pipeline);
}

fn register_contrast(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = ContrastPipeline::new(resources.device, resources.pipeline_cache);
    context.register_compute_pipeline(pipeline);
}
