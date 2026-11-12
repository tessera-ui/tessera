use crate::render::PipelineContext;

/// A render module that registers pipelines.
pub trait RenderModule: Send + Sync {
    /// Registers pipelines using the provided context.
    fn register_pipelines(&self, context: &mut PipelineContext<'_>);
}

impl RenderModule for Vec<Box<dyn RenderModule>> {
    fn register_pipelines(&self, context: &mut PipelineContext<'_>) {
        for module in self {
            module.register_pipelines(context);
        }
    }
}
