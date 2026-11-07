//! WGPU render core for Tessera frames.
//!
//! ## Usage
//!
//! Drive frame submission and GPU resource setup for Tessera applications.

use std::{io, sync::Arc, time::Duration};

use winit::window::Window;

use crate::{
    ComputablePipeline, ComputeCommand, DrawCommand, DrawablePipeline, PxSize,
    compute::resource::ComputeResourceManager, pipeline_cache::save_cache,
    render_graph::RenderTextureDesc, renderer::external::ExternalTextureRegistry,
};

use super::{compute::ComputePipelineRegistry, drawer::Drawer};

mod clip;
mod frame;
#[cfg(all(
    feature = "profiling",
    feature = "headless",
    not(target_arch = "wasm32")
))]
mod gpu_timing;
mod init;

struct RenderPipelines {
    clipper: clip::ShapeClipper,
    drawer: Drawer,
    compute_registry: ComputePipelineRegistry,
}

// Effect-only targets are created on first use so window resizes do not
// allocate full-window textures that the current scene never samples.
struct FrameTargets {
    offscreen: wgpu::TextureView,
    offscreen_copy: Option<wgpu::TextureView>,
    scene_sample: Option<wgpu::TextureView>,
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

struct ComputeState {
    target_a: Option<wgpu::TextureView>,
    target_b: Option<wgpu::TextureView>,
    resource_manager: ComputeResourceManager,
}

struct BlitState {
    pipeline: wgpu::RenderPipeline,
    present_pipeline: wgpu::RenderPipeline,
    pipeline_rgba: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    #[cfg(feature = "debug-dirty-overlay")]
    dirty_overlay_pipeline: wgpu::RenderPipeline,
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

struct LocalTextureSlot {
    desc: RenderTextureDescKey,
    front: wgpu::TextureView,
    back: Option<wgpu::TextureView>,
    msaa_view: Option<wgpu::TextureView>,
    in_use: bool,
    last_used_frame: u64,
}

impl LocalTextureSlot {
    fn front_view(&self) -> &wgpu::TextureView {
        &self.front
    }

    fn back_view(&mut self, device: &wgpu::Device) -> wgpu::TextureView {
        self.back
            .get_or_insert_with(|| {
                create_texture_view(
                    device,
                    &RenderTextureDesc {
                        size: self.desc.size,
                        format: self.desc.format,
                    },
                    "Local Back",
                )
            })
            .clone()
    }

    fn swap_front_back(&mut self) {
        std::mem::swap(
            &mut self.front,
            self.back.as_mut().expect("compute back texture"),
        );
    }

    fn msaa_view(&mut self, device: &wgpu::Device) -> Option<wgpu::TextureView> {
        (self.desc.sample_count > 1).then(|| {
            self.msaa_view
                .get_or_insert_with(|| {
                    create_msaa_view(
                        device,
                        &RenderTextureDesc {
                            size: self.desc.size,
                            format: self.desc.format,
                        },
                        self.desc.sample_count,
                        "Local MSAA",
                    )
                })
                .clone()
        })
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

    fn begin_frame(&mut self, current_frame: u64) {
        // Sizes from earlier layouts, such as before a window resize, are
        // released once they stop being used.
        self.slots
            .retain(|slot| current_frame.wrapping_sub(slot.last_used_frame) <= 2);
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

        let replacement = (self.slots.len() >= Self::MAX_SLOTS)
            .then(|| self.lru_unused_index())
            .flatten();

        let slot = LocalTextureSlot {
            desc: key,
            front: create_texture_view(device, desc, "Local Front"),
            back: None,
            msaa_view: None,
            in_use: true,
            last_used_frame: current_frame,
        };
        if let Some(index) = replacement {
            // Frame resources retain indices into this pool until their last
            // pass, so eviction must not move another live slot.
            self.slots[index] = slot;
            index
        } else {
            self.slots.push(slot);
            self.slots.len() - 1
        }
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

pub(crate) fn create_texture_view(
    device: &wgpu::Device,
    desc: &RenderTextureDesc,
    label: &str,
) -> wgpu::TextureView {
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
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub(crate) fn create_msaa_view(
    device: &wgpu::Device,
    desc: &RenderTextureDesc,
    sample_count: u32,
    label: &str,
) -> wgpu::TextureView {
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

    pipeline_cache: Option<wgpu::PipelineCache>,
    adapter_info: wgpu::AdapterInfo,

    targets: FrameTargets,
    compute: ComputeState,
    blit: BlitState,
    local_textures: LocalTexturePool,
    external_textures: ExternalTextureRegistry,
    frame_index: u64,
    last_render_breakdown: Option<RenderTimingBreakdown>,
    #[cfg(all(
        feature = "profiling",
        feature = "headless",
        not(target_arch = "wasm32")
    ))]
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

    /// Returns the frame-local compute resource manager.
    pub fn compute_resource_manager_mut(&mut self) -> &mut ComputeResourceManager {
        &mut self.compute.resource_manager
    }

    /// Returns the device and compute manager used during record.
    pub fn record_resources(&mut self) -> (&wgpu::Device, &mut ComputeResourceManager) {
        (&self.device, &mut self.compute.resource_manager)
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
        self.pipelines.drawer.pipeline_registry.register(pipeline);
    }

    /// Registers a new compute pipeline for a specific command type.
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

#[cfg(all(test, feature = "headless", not(target_family = "wasm")))]
mod tests {
    use crate::{Px, PxSize, render_graph::RenderTextureDesc};

    use super::{LocalTexturePool, RenderCore};

    #[test]
    #[ignore = "requires a GPU adapter"]
    fn evicting_an_unused_texture_preserves_live_slot_indices() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let core = pollster::block_on(RenderCore::new_offscreen(
            winit::dpi::PhysicalSize::new(1, 1),
            wgpu::TextureFormat::Rgba8Unorm,
            1,
            crate::renderer::DEFAULT_MAX_NON_SAMPLER_BINDINGS,
        ));
        let mut pool = LocalTexturePool::new();
        let texture = |width| RenderTextureDesc {
            size: PxSize::new(Px(width), Px(8)),
            format: wgpu::TextureFormat::Rgba8Unorm,
        };
        let mut last = None;
        for index in 0..LocalTexturePool::MAX_SLOTS {
            let desc = texture(16 + index as i32);
            pool.allocate(&core.device, &desc, 1, 1);
            last = Some(desc);
        }
        pool.begin_frame(2);
        let desc = last.unwrap();
        let live = pool.allocate(&core.device, &desc, 1, 2);
        let front = pool.slot(live).unwrap().front_view().clone();
        let replacement = pool.allocate(&core.device, &texture(128), 1, 2);
        assert_ne!(replacement, live);
        let slot = pool
            .slot(live)
            .expect("the reserved slot must remain attached");
        assert!(slot.in_use);
        assert_eq!(slot.desc.size, desc.size);
        assert_eq!(slot.front_view(), &front);
        assert_eq!(pool.slot(replacement).unwrap().desc.size, texture(128).size);
        assert_eq!(pool.slots.len(), LocalTexturePool::MAX_SLOTS);
    }
}
