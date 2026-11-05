//! Custom GPU pipelines.
//!
//! Implement [`crate::DrawCommand`] and [`crate::DrawablePipeline`] for drawing,
//! or [`crate::ComputeCommand`] and [`crate::ComputablePipeline`] for compute.
//! Register the pipeline through [`PipelineContext::register_draw_pipeline`] or
//! [`PipelineContext::register_compute_pipeline`] in a [`RenderModule`]. Install
//! the module with [`crate::EntryPoint::module`], or share it through a
//! [`crate::TesseraPackage`] using [`crate::EntryRegistry::add_module`].
//!
//! Emit commands from a render policy through [`crate::RenderInput`], or attach
//! a draw policy to a layout. Reuse GPU resources across frames and upload through
//! [`crate::GraphicsDevice`]. Pipeline creation resources specify the surface
//! format and sample count; use both when constructing WGPU pipelines.
//!
//! UI render pipelines require [`crate::ui_stencil_state`]. Honor the supplied
//! clip rectangle, draw geometry and local-to-window transform. Keep measurements
//! local and apply the transform once. Declare background reads with
//! [`crate::DrawCommand::sample_region`]; [`crate::SampleRegion`] describes the
//! sampled scene, while [`crate::DrawRegion`] describes the command's output.
//! Prefer bounded sampling regions when an effect does not need the whole scene.
//!
//! [`crate::DrawablePipeline`] provides frame and pass hooks for batched uploads
//! and drawing. The renderer owns pass boundaries, clipping and submission.

use crate::PipelineContext;

/// A render module that registers pipelines.
pub trait RenderModule: Send + Sync {
    /// Registers pipelines using the provided context.
    fn register_pipelines(&self, context: &mut PipelineContext<'_>);
}
