//! Render module integration for Tessera.
//!
//! ## Usage
//!
//! Compose component render modules for UI rendering.

use crate::{PipelineContext, PxSize, render_graph::RenderGraph};

/// Context provided to render middlewares for per-frame processing.
pub struct RenderMiddlewareContext {
    /// The pixel size of the current frame.
    pub frame_size: PxSize,
    /// The surface format for the current frame.
    pub surface_format: wgpu::TextureFormat,
    /// The MSAA sample count for the current frame.
    pub sample_count: u32,
}

/// Middleware that can transform the per-frame render scene before execution.
pub trait RenderMiddleware: Send {
    /// Returns a human-readable name for this middleware.
    fn name(&self) -> &'static str;
    /// Processes the render scene for the current frame.
    fn process(&mut self, scene: RenderGraph, context: &RenderMiddlewareContext) -> RenderGraph;
}

/// A render module that registers pipelines and provides middleware.
pub trait RenderModule: Send + Sync {
    /// Registers pipelines using the provided context.
    fn register_pipelines(&self, context: &mut PipelineContext<'_>);
    /// Creates middleware instances for per-frame processing.
    fn create_middlewares(&self) -> Vec<Box<dyn RenderMiddleware>> {
        Vec::new()
    }
}
