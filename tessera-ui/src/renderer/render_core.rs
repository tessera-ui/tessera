//! WGPU render core for Tessera frames.
//!
//! ## Usage
//!
//! Drive frame submission and GPU resource setup for Tessera applications.

use std::{io, sync::Arc};

use parking_lot::RwLock;
use winit::window::Window;

use crate::{
    ComputablePipeline, ComputeCommand, DrawCommand, DrawablePipeline, PxPosition,
    compute::resource::ComputeResourceManager,
    pipeline_cache::save_cache,
    px::{PxRect, PxSize},
};

use super::{compute::ComputePipelineRegistry, drawer::Drawer};

mod render_core_frame;
mod render_core_init;

struct PendingComputeCommand {
    command: Box<dyn ComputeCommand>,
    size: PxSize,
    start_pos: PxPosition,
    target_rect: PxRect,
    sampling_rect: PxRect,
}

struct RenderPipelines {
    drawer: Drawer,
    compute_registry: ComputePipelineRegistry,
}

struct FrameTargets {
    offscreen: wgpu::TextureView,
    msaa_texture: Option<wgpu::Texture>,
    msaa_view: Option<wgpu::TextureView>,
    sample_count: u32,
}

struct ComputeState {
    target_a: wgpu::TextureView,
    target_b: wgpu::TextureView,
    pending: Vec<PendingComputeCommand>,
    resource_manager: Arc<RwLock<ComputeResourceManager>>,
}

struct BlitState {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    compute_pipeline: wgpu::RenderPipeline,
}

/// Render core holding device, surface, pipelines, and frame resources.
pub struct RenderCore {
    /// Avoiding release the window
    #[allow(unused)]
    window: Arc<Window>,
    /// WGPU device
    device: wgpu::Device,
    /// WGPU surface
    surface: wgpu::Surface<'static>,
    /// WGPU queue
    queue: wgpu::Queue,
    /// WGPU surface configuration
    config: wgpu::SurfaceConfiguration,
    /// size of the window
    size: winit::dpi::PhysicalSize<u32>,
    /// if size is changed
    size_changed: bool,
    /// Draw and compute pipeline registries.
    pipelines: RenderPipelines,

    /// WGPU pipeline cache for faster pipeline creation when supported.
    pipeline_cache: Option<wgpu::PipelineCache>,
    /// Gpu adapter info
    adapter_info: wgpu::AdapterInfo,

    /// Render target resources for the current frame.
    targets: FrameTargets,
    /// Compute resources for ping-pong passes.
    compute: ComputeState,
    /// Blit resources for partial copies.
    blit: BlitState,
}

/// Shared GPU resources used when creating pipelines.
pub struct RenderResources<'a> {
    /// WGPU device used for pipeline creation.
    pub device: &'a wgpu::Device,
    /// WGPU queue used by pipelines that upload data.
    pub queue: &'a wgpu::Queue,
    /// Surface configuration used for render pipeline setup.
    pub surface_config: &'a wgpu::SurfaceConfiguration,
    /// Optional pipeline cache when supported by the adapter.
    pub pipeline_cache: Option<&'a wgpu::PipelineCache>,
    /// MSAA sample count for render pipelines.
    pub sample_count: u32,
}

impl RenderCore {
    /// Returns shared GPU resources used for pipeline creation.
    pub fn resources(&self) -> RenderResources<'_> {
        RenderResources {
            device: &self.device,
            queue: &self.queue,
            surface_config: &self.config,
            pipeline_cache: self.pipeline_cache.as_ref(),
            sample_count: self.targets.sample_count,
        }
    }

    /// Returns the current window handle.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Returns a cloned window handle for external storage.
    pub fn window_arc(&self) -> Arc<Window> {
        self.window.clone()
    }

    /// Returns the WGPU device.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Returns the WGPU queue.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Returns the current surface configuration.
    pub fn surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    /// Returns the pipeline cache if available.
    pub fn pipeline_cache(&self) -> Option<&wgpu::PipelineCache> {
        self.pipeline_cache.as_ref()
    }

    /// Returns the configured MSAA sample count.
    pub fn sample_count(&self) -> u32 {
        self.targets.sample_count
    }

    /// Returns the shared compute resource manager.
    pub fn compute_resource_manager(&self) -> Arc<RwLock<ComputeResourceManager>> {
        self.compute.resource_manager.clone()
    }

    /// Registers a new drawable pipeline for a specific command type.
    ///
    /// This method takes ownership of the pipeline and wraps it in a
    /// type-erased container that can be stored alongside other pipelines of
    /// different types.
    pub fn register_draw_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: DrawCommand + 'static,
        P: DrawablePipeline<T> + 'static,
    {
        self.pipelines.drawer.pipeline_registry.register(pipeline);
    }

    /// Registers a new compute pipeline for a specific command type.
    ///
    /// This method takes ownership of the pipeline and wraps it in a
    /// type-erased container that can be stored alongside other pipelines of
    /// different types.
    pub fn register_compute_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: ComputeCommand + 'static,
        P: ComputablePipeline<T> + 'static,
    {
        self.pipelines.compute_registry.register(pipeline);
    }

    pub(crate) fn save_pipeline_cache(&self) -> io::Result<()> {
        if let Some(cache) = self.pipeline_cache.as_ref() {
            save_cache(cache, &self.adapter_info)?;
        }
        Ok(())
    }
}
