//! Graphics rendering pipeline management.
//!
//! This module provides the drawing infrastructure for the unified command system,
//! handling graphics pipeline registration and command dispatch.

pub mod command;
mod pipeline;

use crate::{PxPosition, px::PxSize};
pub use command::{BarrierRequirement, DrawCommand};
pub use pipeline::{DrawablePipeline, PipelineRegistry};

/// Drawer manages graphics pipelines and processes draw commands.
///
/// The Drawer acts as the central coordinator for all graphics rendering operations,
/// maintaining a registry of pipelines and dispatching draw commands to the appropriate
/// pipeline implementations.
pub struct Drawer {
    /// Registry containing all registered graphics pipelines
    pub pipeline_registry: PipelineRegistry,
}

impl Default for Drawer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawer {
    /// Create a new drawer
    pub fn new() -> Self {
        Self {
            pipeline_registry: PipelineRegistry::new(),
        }
    }

    /// Initialize all pipelines at the beginning of each render pass.
    ///
    /// This method calls the `begin_pass` method on all registered pipelines,
    /// allowing them to set up per-pass state such as uniform buffers.
    pub fn begin_pass(
        &mut self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.pipeline_registry
            .begin_all_passes(gpu, queue, config, render_pass);
    }

    /// Finalize all pipelines at the end of each render pass.
    ///
    /// This method calls the `end_pass` method on all registered pipelines,
    /// allowing them to perform cleanup or final operations.
    pub fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.pipeline_registry
            .end_all_passes(gpu, queue, config, render_pass);
    }

    /// Submit a draw command to the appropriate pipeline for rendering.
    ///
    /// This method dispatches the command to the correct pipeline based on its type,
    /// providing all necessary context including GPU resources, render pass, and
    /// positioning information.
    ///
    /// # Arguments
    /// * `cmd` - The draw command to execute
    /// * `size` - Size of the component being drawn
    /// * `start_pos` - Position where drawing should begin
    /// * `scene_texture_view` - Optional background texture for sampling
    /// * `compute_texture_view` - Compute pipeline output texture
    pub fn submit(
        &mut self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        cmd: &dyn DrawCommand,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: &wgpu::TextureView,
    ) {
        self.pipeline_registry.dispatch(
            gpu,
            queue,
            config,
            render_pass,
            cmd,
            size,
            start_pos,
            scene_texture_view,
        );
    }
}
