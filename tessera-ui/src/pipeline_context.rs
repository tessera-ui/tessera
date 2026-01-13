//! Pipeline registration context for renderer initialization.
//!
//! ## Usage
//!
//! Register draw and compute pipelines from component libraries at startup.

use crate::{ComputablePipeline, ComputeCommand, DrawCommand, DrawablePipeline, renderer::WgpuApp};

/// Context passed to pipeline initialization functions.
pub struct PipelineContext<'a> {
    app: &'a mut WgpuApp,
}

impl<'a> PipelineContext<'a> {
    /// Creates a new pipeline context for the given renderer app.
    pub fn new(app: &'a mut WgpuApp) -> Self {
        Self { app }
    }

    /// Returns the underlying renderer app for direct access.
    pub fn app(&mut self) -> &mut WgpuApp {
        self.app
    }

    /// Registers a draw pipeline for a specific command type.
    pub fn register_draw_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: DrawCommand + 'static,
        P: DrawablePipeline<T> + 'static,
    {
        self.app.register_draw_pipeline(pipeline);
    }

    /// Registers a compute pipeline for a specific command type.
    pub fn register_compute_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: ComputeCommand + 'static,
        P: ComputablePipeline<T> + 'static,
    {
        self.app.register_compute_pipeline(pipeline);
    }
}
