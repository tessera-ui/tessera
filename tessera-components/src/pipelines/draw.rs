use tessera_ui::PipelineContext;

use crate::pipelines::{
    checkmark::pipeline::CheckmarkPipeline,
    divider::DividerPipeline,
    fluid_glass::pipeline::FluidGlassPipeline,
    image::pipeline::ImagePipeline,
    image_vector::pipeline::ImageVectorPipeline,
    linear_wave::pipeline::LinearWavePipeline,
    loading_shape::pipeline::LoadingShapePipeline,
    path_stroke::pipeline::PathStrokePipeline,
    progress_arc::pipeline::ProgressArcPipeline,
    shadow::pipeline::{ShadowCompositePipeline, ShadowMaskPipeline},
    shape::pipeline::ShapePipeline,
    simple_rect::pipeline::SimpleRectPipeline,
    text::pipeline::GlyphonTextRender,
};

pub(super) fn register(context: &mut PipelineContext<'_>) {
    register_simple_rect(context);
    register_divider(context);
    register_shape(context);
    register_shadow(context);
    register_progress_arc(context);
    register_checkmark(context);
    register_text(context);
    register_fluid_glass(context);
    register_image(context);
    register_image_vector(context);
    register_loading_shape(context);
    register_linear_wave(context);
    register_path_stroke(context);
}

fn register_path_stroke(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = PathStrokePipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_linear_wave(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = LinearWavePipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_loading_shape(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = LoadingShapePipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_simple_rect(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = SimpleRectPipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_shape(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = ShapePipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_shadow(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let mask_pipeline = ShadowMaskPipeline::new(
        resources.device,
        resources.pipeline_cache,
        resources.sample_count,
    );
    let composite_pipeline = ShadowCompositePipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(mask_pipeline);
    context.register_draw_pipeline(composite_pipeline);
}

fn register_progress_arc(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = ProgressArcPipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_checkmark(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = CheckmarkPipeline::new(
        resources.device,
        resources.pipeline_cache,
        resources.surface_config,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_text(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = GlyphonTextRender::new(
        resources.device,
        resources.queue,
        resources.surface_config,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_fluid_glass(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = FluidGlassPipeline::new(
        resources.device,
        resources.pipeline_cache,
        resources.surface_config,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_image(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = ImagePipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_image_vector(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = ImageVectorPipeline::new(
        resources.device,
        resources.surface_config,
        resources.pipeline_cache,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}

fn register_divider(context: &mut PipelineContext<'_>) {
    let resources = context.resources();
    let pipeline = DividerPipeline::new(
        resources.device,
        resources.pipeline_cache,
        resources.surface_config,
        resources.sample_count,
    );
    context.register_draw_pipeline(pipeline);
}
