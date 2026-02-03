//! WGPU render core for Tessera frames.
//!
//! ## Usage
//!
//! Drive frame submission and GPU resource setup for Tessera applications.

use std::{io, sync::Arc, time::Duration};

use parking_lot::RwLock;
use winit::window::Window;

use crate::{
    CompositeCommand, ComputablePipeline, ComputeCommand, DrawCommand, DrawablePipeline, PxSize,
    compute::resource::ComputeResourceManager,
    pipeline_cache::save_cache,
    render_graph::RenderTextureDesc,
    renderer::{
        composite::{CompositeContext, CompositePipelineRegistry},
        external::ExternalTextureRegistry,
    },
};

use super::{compute::ComputePipelineRegistry, drawer::Drawer};

mod frame;
mod init;

struct RenderPipelines {
    drawer: Drawer,
    compute_registry: ComputePipelineRegistry,
    composite_registry: CompositePipelineRegistry,
}

struct FrameTargets {
    offscreen: wgpu::TextureView,
    offscreen_copy: wgpu::TextureView,
    msaa_texture: Option<wgpu::Texture>,
    msaa_view: Option<wgpu::TextureView>,
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

struct ComputeState {
    target_a: wgpu::TextureView,
    target_b: wgpu::TextureView,
    resource_manager: Arc<RwLock<ComputeResourceManager>>,
}

struct BlitState {
    pipeline: wgpu::RenderPipeline,
    pipeline_rgba: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct RenderTextureDescKey {
    size: PxSize,
    format: wgpu::TextureFormat,
    sample_count: u32,
}

impl RenderTextureDescKey {
    fn from_desc(desc: &RenderTextureDesc, sample_count: u32) -> Self {
        Self {
            size: desc.size,
            format: desc.format,
            sample_count,
        }
    }
}

struct TextureHandle {
    view: wgpu::TextureView,
}

struct LocalTextureSlot {
    desc: RenderTextureDescKey,
    front: TextureHandle,
    back: TextureHandle,
    msaa_view: Option<wgpu::TextureView>,
    in_use: bool,
    last_used_frame: u64,
}

impl LocalTextureSlot {
    fn front_view(&self) -> &wgpu::TextureView {
        &self.front.view
    }

    fn back_view(&self) -> &wgpu::TextureView {
        &self.back.view
    }

    fn swap_front_back(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }
}

struct LocalTexturePool {
    slots: Vec<LocalTextureSlot>,
}

impl LocalTexturePool {
    const MAX_SLOTS: usize = 16;

    fn new() -> Self {
        Self { slots: Vec::new() }
    }

    fn clear(&mut self) {
        self.slots.clear();
    }

    fn begin_frame(&mut self, _current_frame: u64) {
        for slot in &mut self.slots {
            slot.in_use = false;
        }
    }

    fn allocate(
        &mut self,
        device: &wgpu::Device,
        desc: &RenderTextureDesc,
        sample_count: u32,
        current_frame: u64,
    ) -> usize {
        let key = RenderTextureDescKey::from_desc(desc, sample_count);
        if let Some((index, slot)) = self
            .slots
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.desc == key && !slot.in_use)
        {
            slot.in_use = true;
            slot.last_used_frame = current_frame;
            return index;
        }

        if self.slots.len() >= Self::MAX_SLOTS
            && let Some(index) = self.lru_unused_index()
        {
            self.slots.swap_remove(index);
        }

        let front = create_local_texture(device, desc, "Local Front");
        let back = create_local_texture(device, desc, "Local Back");
        let msaa_view = if sample_count > 1 {
            Some(create_msaa_view(device, desc, sample_count))
        } else {
            None
        };

        let slot = LocalTextureSlot {
            desc: key,
            front,
            back,
            msaa_view,
            in_use: true,
            last_used_frame: current_frame,
        };
        self.slots.push(slot);
        self.slots.len() - 1
    }

    fn lru_unused_index(&self) -> Option<usize> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| !slot.in_use)
            .min_by_key(|(_, slot)| slot.last_used_frame)
            .map(|(index, _)| index)
    }

    fn slot(&self, index: usize) -> Option<&LocalTextureSlot> {
        self.slots.get(index)
    }

    fn slot_mut(&mut self, index: usize) -> Option<&mut LocalTextureSlot> {
        self.slots.get_mut(index)
    }
}

fn create_local_texture(
    device: &wgpu::Device,
    desc: &RenderTextureDesc,
    label: &str,
) -> TextureHandle {
    let width = desc.size.width.positive().max(1);
    let height = desc.size.height.positive().max(1);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: desc.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    TextureHandle { view }
}

fn create_msaa_view(
    device: &wgpu::Device,
    desc: &RenderTextureDesc,
    sample_count: u32,
) -> wgpu::TextureView {
    let width = desc.size.width.positive().max(1);
    let height = desc.size.height.positive().max(1);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Local MSAA"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: desc.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
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
    /// Pool of local textures declared by render graph resources.
    local_textures: LocalTexturePool,
    /// Registry of external textures owned by pipelines.
    external_textures: ExternalTextureRegistry,
    /// Monotonic frame counter for resource eviction.
    frame_index: u64,
    /// Timing breakdown for the last render call.
    last_render_breakdown: Option<RenderTimingBreakdown>,
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

    /// Returns the timing breakdown for the most recent render call.
    pub(crate) fn last_render_breakdown(&self) -> Option<RenderTimingBreakdown> {
        self.last_render_breakdown
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

    /// Registers a new composite pipeline for a specific command type.
    pub fn register_composite_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: CompositeCommand + 'static,
        P: super::composite::CompositePipeline<T> + 'static,
    {
        self.pipelines.composite_registry.register(pipeline);
    }

    pub(crate) fn composite_context_parts(
        &mut self,
        frame_size: PxSize,
        frame_index: u64,
    ) -> (CompositeContext<'_>, &mut CompositePipelineRegistry) {
        let RenderCore {
            device,
            queue,
            config,
            pipeline_cache,
            targets,
            pipelines,
            ..
        } = self;
        let resources = RenderResources {
            device,
            queue,
            surface_config: config,
            pipeline_cache: pipeline_cache.as_ref(),
            sample_count: targets.sample_count,
        };
        let context = CompositeContext {
            resources,
            external_textures: self.external_textures.clone(),
            frame_size,
            surface_format: config.format,
            sample_count: targets.sample_count,
            frame_index,
        };
        (context, &mut pipelines.composite_registry)
    }

    pub(crate) fn save_pipeline_cache(&self) -> io::Result<()> {
        if let Some(cache) = self.pipeline_cache.as_ref() {
            save_cache(cache, &self.adapter_info)?;
        }
        Ok(())
    }
}
