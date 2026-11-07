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
    let resources = context.resources();
    let pipeline = SimpleRectPipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = DividerPipeline::new(
        resources.graphics,
        resources.pipeline_cache,
        resources.surface_config,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = ShapePipeline::new(
        resources.graphics,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = ProgressArcPipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = CheckmarkPipeline::new(
        resources.graphics,
        resources.pipeline_cache,
        resources.surface_config,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = GlyphonTextRender::new(
        resources.device,
        resources.queue,
        resources.surface_config,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = FluidGlassPipeline::new(
        resources.device,
        resources.pipeline_cache,
        resources.surface_config,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = ImagePipeline::new(
        resources.graphics,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = ImageVectorPipeline::new(
        resources.graphics,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = LoadingShapePipeline::new(
        resources.graphics,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = LinearWavePipeline::new(
        resources.graphics,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);

    let resources = context.resources();
    let pipeline = PathStrokePipeline::new(
        resources.graphics,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}
