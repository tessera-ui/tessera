//! Pipeline registration context for renderer initialization

use crate::{
    ComputablePipeline, ComputeCommand, DrawCommand, DrawablePipeline,
    renderer::{RenderCore, RenderResources},
};

/// Context passed to pipeline initialization functions.
pub struct PipelineContext<'a> {
    core: &'a mut RenderCore,
}

impl<'a> PipelineContext<'a> {
    /// Returns the resource and upload service owned by this renderer.
    pub fn graphics(&self) -> &crate::GraphicsDevice {
        self.core.resources().graphics
    }
    /// Creates a new pipeline context for the given renderer app.
    pub(crate) fn new(core: &'a mut RenderCore) -> Self {
        Self { core }
    }

    /// Returns shared GPU resources used for pipeline creation.
    pub fn resources(&self) -> RenderResources<'_> {
        self.core.resources()
    }

    /// Registers a draw pipeline for a specific command type.
    pub fn register_draw_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: DrawCommand + 'static,
        P: DrawablePipeline<T> + 'static,
    {
        self.core.register_draw_pipeline(pipeline);
    }

    /// Registers a compute pipeline for a specific command type.
    pub fn register_compute_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: ComputeCommand + 'static,
        P: ComputablePipeline<T> + 'static,
    {
        self.core.register_compute_pipeline(pipeline);
    }

    /// Registers a factory that creates a draw pipeline before the first frame that uses it.
    pub fn register_draw_pipeline_factory<T, P>(
        &mut self,
        factory: impl FnOnce(RenderResources<'_>) -> P + Send + 'static,
    ) where
        T: DrawCommand + 'static,
        P: DrawablePipeline<T> + 'static,
    {
        self.core.register_draw_pipeline_factory(factory);
    }

    /// Registers a factory that creates a compute pipeline before its first dispatch.
    pub fn register_compute_pipeline_factory<T, P>(
        &mut self,
        factory: impl FnOnce(RenderResources<'_>) -> P + Send + 'static,
    ) where
        T: ComputeCommand + 'static,
        P: ComputablePipeline<T> + 'static,
    {
        self.core.register_compute_pipeline_factory(factory);
    }
}
