use tessera_ui::PipelineContext;

use crate::pipelines::{blur::pipeline::BlurPipeline, contrast::ContrastPipeline};

pub(super) fn register(context: &mut PipelineContext<'_>) {
    context.register_compute_pipeline_factory(|resources| {
        BlurPipeline::new(resources.device, resources.pipeline_cache)
    });

    context.register_compute_pipeline_factory(|resources| {
        ContrastPipeline::new(resources.device, resources.pipeline_cache)
    });
}
