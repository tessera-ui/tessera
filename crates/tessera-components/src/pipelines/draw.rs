use tessera_ui::PipelineContext;

use crate::pipelines::{
    checkmark::pipeline::CheckmarkPipeline, divider::DividerPipeline,
    fluid_glass::pipeline::FluidGlassPipeline, image::pipeline::ImagePipeline,
    image_vector::pipeline::ImageVectorPipeline, linear_wave::pipeline::LinearWavePipeline,
    loading_shape::pipeline::LoadingShapePipeline, path_stroke::pipeline::PathStrokePipeline,
    progress_arc::pipeline::ProgressArcPipeline, shape::pipeline::ShapePipeline,
    simple_rect::pipeline::SimpleRectPipeline, text::pipeline::GlyphonTextRender,
};

pub(super) fn register(context: &mut PipelineContext<'_>) {
    context.register_draw_pipeline_factory(|resources| {
        SimpleRectPipeline::new(
            resources.device,
            resources.surface_config,
            resources.pipeline_cache,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        DividerPipeline::new(
            resources.graphics,
            resources.pipeline_cache,
            resources.surface_config,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        ShapePipeline::new(
            resources.graphics,
            resources.surface_config,
            resources.pipeline_cache,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        ProgressArcPipeline::new(
            resources.device,
            resources.surface_config,
            resources.pipeline_cache,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        CheckmarkPipeline::new(
            resources.graphics,
            resources.pipeline_cache,
            resources.surface_config,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        GlyphonTextRender::new(
            resources.device,
            resources.queue,
            resources.surface_config,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        FluidGlassPipeline::new(
            resources.device,
            resources.pipeline_cache,
            resources.surface_config,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        ImagePipeline::new(
            resources.graphics,
            resources.surface_config,
            resources.pipeline_cache,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        ImageVectorPipeline::new(
            resources.graphics,
            resources.surface_config,
            resources.pipeline_cache,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        LoadingShapePipeline::new(
            resources.graphics,
            resources.surface_config,
            resources.pipeline_cache,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        LinearWavePipeline::new(
            resources.graphics,
            resources.surface_config,
            resources.pipeline_cache,
            resources.sample_count,
        )
    });

    context.register_draw_pipeline_factory(|resources| {
        PathStrokePipeline::new(
            resources.graphics,
            resources.surface_config,
            resources.pipeline_cache,
            resources.sample_count,
        )
    });
}
