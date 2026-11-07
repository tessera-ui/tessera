//! WGPU render core for Tessera frames.
//!
//! ## Usage
//!
//! Drive frame submission and GPU resource setup for Tessera applications.

use std::{any::TypeId, io, sync::Arc, time::Duration};

use rustc_hash::FxHashMap;
use winit::window::Window;

use crate::{
    ComputablePipeline, ComputeCommand, DrawCommand, DrawablePipeline, pipeline_cache::save_cache,
};

use super::{compute::ComputePipelineRegistry, drawer::Drawer};

mod clip;
mod frame;
#[cfg(all(feature = "profiling", not(target_arch = "wasm32")))]
mod gpu_timing;
mod init;

struct RenderPipelines {
    clipper: clip::ShapeClipper,
    drawer: Drawer,
    compute_registry: ComputePipelineRegistry,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum PipelineKind {
    Draw,
    Compute,
}

type PipelineFactory = Box<dyn FnOnce(&mut RenderCore) + Send>;

struct FrameTargets {
    offscreen: wgpu::TextureView,
    // Only surface-less cores render into a copy for readback.
    offscreen_copy: Option<wgpu::TextureView>,
    msaa_texture: Option<wgpu::Texture>,
    msaa_view: Option<wgpu::TextureView>,
    stencil: wgpu::TextureView,
    sample_count: u32,
}

/// Timing breakdown for the most recent render call.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RenderTimingBreakdown {
    /// Time spent acquiring the swapchain texture.
    pub acquire: Duration,
    /// Time spent building the render pass graph and pass plan.
    pub build_passes: Duration,
    /// Time spent encoding GPU commands.
    pub encode: Duration,
    /// Time spent submitting command buffers to the queue.
    pub submit: Duration,
    /// Time spent presenting the swapchain image.
    pub present: Duration,
    /// Total render duration for the frame.
    pub total: Duration,
}

/// Error returned when reading back an offscreen frame fails.
#[derive(Clone, Debug)]
pub enum OffscreenReadbackError {
    /// The offscreen target has a zero width or height.
    EmptyTarget,
    /// The GPU device poll failed while waiting for the readback buffer.
    Poll(String),
    /// Mapping the readback buffer failed.
    Map(String),
}

impl std::fmt::Display for OffscreenReadbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTarget => write!(f, "offscreen render target has zero size"),
            Self::Poll(err) => write!(f, "device poll failed during offscreen readback: {err}"),
            Self::Map(err) => write!(f, "failed to map offscreen readback buffer: {err}"),
        }
    }
}

impl std::error::Error for OffscreenReadbackError {}

struct BlitState {
    clear_pipeline: wgpu::RenderPipeline,
    present_pipeline: wgpu::RenderPipeline,
    pipeline_rgba: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    #[cfg(feature = "debug-dirty-overlay")]
    dirty_overlay_pipeline: wgpu::RenderPipeline,
}

/// Render core holding device, surface, pipelines, and frame resources.
pub struct RenderCore {
    graphics: crate::GraphicsDevice,
    #[allow(unused)]
    window: Option<Arc<Window>>,
    pub(crate) instance: wgpu::Instance,
    device: wgpu::Device,
    surface: Option<wgpu::Surface<'static>>,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    draw_config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    size_changed: bool,
    pipelines: RenderPipelines,
    pipeline_factories: FxHashMap<(PipelineKind, TypeId), PipelineFactory>,

    pipeline_cache: Option<wgpu::PipelineCache>,
    adapter_info: wgpu::AdapterInfo,

    targets: FrameTargets,
    blit: BlitState,
    frame_index: u64,
    previous_graph: Option<crate::RenderGraph>,
    last_render_breakdown: Option<RenderTimingBreakdown>,
    #[cfg(all(feature = "profiling", not(target_arch = "wasm32")))]
    gpu_timing: Option<gpu_timing::GpuTimingProbe>,
}

/// Shared GPU resources used when creating pipelines.
pub struct RenderResources<'a> {
    /// Renderer-owned resource creation, uploads, and work observations.
    pub graphics: &'a crate::GraphicsDevice,
    /// WGPU device used for pipeline creation.
    pub device: &'a wgpu::Device,
    /// WGPU queue used by pipelines that upload data.
    pub queue: &'a wgpu::Queue,
    /// Configuration of the color targets used by render pipelines.
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
            graphics: &self.graphics,
            device: &self.device,
            queue: &self.queue,
            surface_config: &self.draw_config,
            pipeline_cache: self.pipeline_cache.as_ref(),
            sample_count: self.targets.sample_count,
        }
    }

    /// Returns the current window handle.
    ///
    /// Panics if this core was created for offscreen rendering; check
    /// [`RenderCore::has_surface`] first.
    pub fn window(&self) -> &Window {
        self.window
            .as_deref()
            .expect("RenderCore has no window in offscreen mode")
    }

    /// Returns a cloned window handle for external storage.
    ///
    /// Panics if this core was created for offscreen rendering.
    pub fn window_arc(&self) -> Arc<Window> {
        self.window
            .clone()
            .expect("RenderCore has no window in offscreen mode")
    }

    /// Returns whether this core renders into a window surface.
    ///
    /// Offscreen cores return `false`.
    pub fn has_surface(&self) -> bool {
        self.surface.is_some()
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

    /// Returns the timing breakdown for the most recent render call.
    pub(crate) fn last_render_breakdown(&self) -> Option<RenderTimingBreakdown> {
        self.last_render_breakdown
    }

    /// Registers a new drawable pipeline for a specific command type.
    pub fn register_draw_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: DrawCommand + 'static,
        P: DrawablePipeline<T> + 'static,
    {
        self.pipeline_factories
            .remove(&(PipelineKind::Draw, TypeId::of::<T>()));
        self.pipelines.drawer.pipeline_registry.register(pipeline);
    }

    /// Registers a new compute pipeline for a specific command type.
    pub fn register_compute_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: ComputeCommand + 'static,
        P: ComputablePipeline<T> + 'static,
    {
        self.pipeline_factories
            .remove(&(PipelineKind::Compute, TypeId::of::<T>()));
        self.pipelines.compute_registry.register(pipeline);
    }

    pub(crate) fn register_draw_pipeline_factory<T, P>(
        &mut self,
        factory: impl FnOnce(RenderResources<'_>) -> P + Send + 'static,
    ) where
        T: DrawCommand + 'static,
        P: DrawablePipeline<T> + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.pipelines
            .drawer
            .pipeline_registry
            .pipelines
            .remove(&type_id);
        self.pipeline_factories.insert(
            (PipelineKind::Draw, type_id),
            Box::new(move |core| {
                let pipeline = factory(core.resources());
                core.register_draw_pipeline(pipeline);
            }),
        );
    }

    pub(crate) fn register_compute_pipeline_factory<T, P>(
        &mut self,
        factory: impl FnOnce(RenderResources<'_>) -> P + Send + 'static,
    ) where
        T: ComputeCommand + 'static,
        P: ComputablePipeline<T> + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.pipelines.compute_registry.remove(type_id);
        self.pipeline_factories.insert(
            (PipelineKind::Compute, type_id),
            Box::new(move |core| {
                let pipeline = factory(core.resources());
                core.register_compute_pipeline(pipeline);
            }),
        );
    }

    pub(crate) fn save_pipeline_cache(&self) -> io::Result<()> {
        if let Some(cache) = self.pipeline_cache.as_ref() {
            save_cache(cache, &self.adapter_info)?;
        }
        Ok(())
    }
}
