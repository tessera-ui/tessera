use crate::render::PipelineContext;

/// A render module that registers pipelines.
pub trait RenderModule: Send + Sync {
    /// Registers pipelines using the provided context.
    fn register_pipelines(&self, context: &mut PipelineContext<'_>);
}
