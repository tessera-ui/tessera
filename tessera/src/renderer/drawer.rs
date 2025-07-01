mod command;
mod pipeline;

use super::compute::ComputePipelineRegistry;
use crate::{PxPosition, px::PxSize};
pub use command::{DrawCommand, RenderRequirement};
pub use pipeline::{DrawablePipeline, PipelineRegistry};

/// Drawer is a struct that handles pipelines and draw commands.
pub struct Drawer {
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
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: Option<&wgpu::TextureView>,
        compute_registry: &mut ComputePipelineRegistry,
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
            compute_registry,
        );
    }
}
