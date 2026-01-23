//! Pipeline registration context for renderer initialization.
//!
//! ## Usage
//!
//! Register draw and compute pipelines from component libraries at startup.

use crate::{
    CompositeCommand, ComputablePipeline, ComputeCommand, DrawCommand, DrawablePipeline,
    renderer::{RenderCore, RenderResources, composite::CompositePipeline},
};

/// Context passed to pipeline initialization functions.
pub struct PipelineContext<'a> {
    core: &'a mut RenderCore,
}

impl<'a> PipelineContext<'a> {
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

    /// Registers a composite pipeline for a specific command type.
    pub fn register_composite_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: CompositeCommand + 'static,
        P: CompositePipeline<T> + 'static,
    {
        self.core.register_composite_pipeline(pipeline);
    }
}
