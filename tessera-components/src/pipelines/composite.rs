use tessera_ui::PipelineContext;

use crate::pipelines::shadow::atlas::ShadowAtlasPipeline;

pub(super) fn register(context: &mut PipelineContext<'_>) {
    context.register_composite_pipeline(ShadowAtlasPipeline::new());
}
