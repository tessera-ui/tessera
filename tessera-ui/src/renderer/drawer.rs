//! Graphics rendering pipeline management.

pub mod command;
pub mod pipeline;

use crate::{px::PxSize, render_graph::DrawGeometry};

pub use command::DrawCommand;
pub use pipeline::{DrawablePipeline, ErasedDrawContext, PipelineRegistry};

/// Drawer manages graphics pipelines and processes draw commands.
#[derive(Default)]
pub struct Drawer {
    /// Registry containing all registered graphics pipelines
    pub pipeline_registry: PipelineRegistry,
}

impl Drawer {
    /// Create a new drawer
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize all pipelines at the beginning of each render pass.
    pub fn begin_pass(
        &mut self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        target_size: PxSize,
        render_pass: &mut wgpu::RenderPass<'_>,
        scene_texture_view: &wgpu::TextureView,
    ) {
        self.pipeline_registry.begin_all_passes(
            gpu,
            queue,
            config,
            target_size,
            render_pass,
            scene_texture_view,
        );
    }

    /// Finalize all pipelines at the end of each render pass.
    pub fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        target_size: PxSize,
        render_pass: &mut wgpu::RenderPass<'_>,
        scene_texture_view: &wgpu::TextureView,
    ) {
        self.pipeline_registry.end_all_passes(
            gpu,
            queue,
            config,
            target_size,
            render_pass,
            scene_texture_view,
        );
    }

    /// Submit a draw command to the appropriate pipeline for rendering.
    ///
    /// # Arguments
    ///
    /// * `context` - Render-pass resources and the active clipping rectangle.
    /// * `commands` - Commands with local dimensions and precise target
    ///   mappings.
    pub fn submit(
        &mut self,
        context: ErasedDrawContext<'_, '_>,
        commands: &[(&dyn DrawCommand, DrawGeometry)],
    ) {
        self.pipeline_registry.dispatch(context, commands);
    }
}
