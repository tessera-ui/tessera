mod command;
mod pipeline;

use crate::{Px, PxPosition};
pub use command::DrawCommand;
pub use pipeline::{DrawablePipeline, PipelineRegistry};

/// Drawer is a struct that handles pipelines and draw commands.
pub struct Drawer {
    pipeline_registry: PipelineRegistry,
}

impl Drawer {
    /// Create a new drawer
    pub fn new(
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        register_pipelines_fn: impl FnOnce(
            &wgpu::Device,
            &wgpu::Queue,
            &wgpu::SurfaceConfiguration,
            &mut PipelineRegistry,
        ),
    ) -> Self {
        let mut pipelines = PipelineRegistry::new();
        register_pipelines_fn(gpu, queue, config, &mut pipelines);

        // Create the drawer
        Self {
            pipeline_registry: pipelines,
        }
    }

    /// Call this at the beginning of each pass.
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

    /// Call this at the end of each pass.
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

    /// Submit a draw command to the appropriate pipeline.
    pub fn submit(
        &mut self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        cmd: &dyn DrawCommand,
        size: [Px; 2],
        start_pos: PxPosition,
    ) {
        self.pipeline_registry
            .dispatch(gpu, queue, config, render_pass, cmd, size, start_pos);
    }
}
