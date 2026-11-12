//! Custom GPU pipelines and renderer-owned graphics resources.
//!
//! ## Usage
//!
//! Draw or compute visual effects that built-in components do not provide.
//!
//! Implement [`DrawCommand`] and [`DrawablePipeline`] for drawing, or
//! [`ComputeCommand`] and [`ComputablePipeline`] for compute. Register the
//! pipeline through [`PipelineContext::register_draw_pipeline`] or
//! [`PipelineContext::register_compute_pipeline`] in a [`RenderModule`]. Install
//! the module with [`crate::EntryPoint::module`], or share it through a
//! [`crate::TesseraPackage`] using [`crate::EntryRegistry::add_module`].
//!
//! Emit commands from a render policy through [`crate::RenderInput`], or attach
//! a draw policy to a layout. Reuse GPU resources across frames and upload through
//! [`GraphicsDevice`]. Pipeline creation resources specify the surface format and
//! sample count; use both when constructing WGPU pipelines.
//!
//! UI render pipelines require [`ui_stencil_state`]. Honor the supplied clip
//! rectangle, draw geometry and local-to-window transform. Keep measurements local
//! and apply the transform once. Declare background reads with
//! [`DrawCommand::sample_region`]; [`SampleRegion`] describes the sampled scene,
//! while [`DrawRegion`] describes the command's output. Prefer bounded sampling
//! regions when an effect does not need the whole scene.
//!
//! [`DrawablePipeline`] provides frame and pass hooks for batched uploads and
//! drawing. The renderer owns pass boundaries, clipping and submission.

pub(crate) mod compute;
pub(crate) mod drawer;

pub use crate::{
    clip::{ClipRegion, ClipShape, ui_stencil_state},
    graphics::{
        FrameBuffer, GraphicsDevice, GraphicsWork, GraphicsWorkDiagnostics, ScratchFit,
        ScratchTexture, ScratchTextureDescriptor, TrackedBuffer,
    },
    pipeline_context::PipelineContext,
    render_graph::{DrawGeometry, RenderFragment, RenderFragmentOp, RenderGraph, RenderGraphOp},
    render_module::RenderModule,
    render_scene::{Command, DrawRegion, PaddingRect, SampleRegion},
    renderer::core::{OffscreenReadbackError, RenderCore, RenderResources},
};

pub use self::{
    compute::{
        ComputablePipeline, ComputeBatchItem, ComputeCommand, ComputeContext,
        ComputePipelineRegistry,
    },
    drawer::{DrawCommand, DrawContext, DrawablePipeline, FrameContext, PipelineRegistry},
};
