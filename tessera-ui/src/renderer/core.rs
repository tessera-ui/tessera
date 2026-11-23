//! WGPU render core for Tessera frames.
//!
//! ## Usage
//!
//! Drive frame submission and GPU resource setup for Tessera applications.

use std::{any::TypeId, io, sync::Arc, time::Duration};

#[cfg(any(
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
))]
use std::{cell::RefCell, rc::Rc};

use rustc_hash::FxHashMap;
use winit::window::Window;

use crate::{
    pipeline_cache::save_cache,
    render::{
        ComputablePipeline, ComputeCommand, ComputePipelineRegistry, DrawCommand, DrawablePipeline,
        drawer::Drawer,
    },
};

#[cfg(target_os = "android")]
mod android_egl;
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
    graphics: crate::render::GraphicsDevice,
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
    #[cfg(any(
        target_os = "android",
        target_os = "ios",
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_family = "wasm"
    ))]
    adapter: wgpu::Adapter,
    #[cfg(target_os = "android")]
    android_slice_surfaces: Vec<Vec<AndroidSliceTarget>>,
    #[cfg(target_os = "android")]
    android_egl: Option<android_egl::SlicePresenter>,
    #[cfg(target_os = "ios")]
    ios_slice_surfaces: Vec<IosSliceTarget>,
    #[cfg(target_os = "ios")]
    ios_composition_active: bool,
    #[cfg(target_os = "macos")]
    macos_slice_surfaces: Vec<MacosSliceTarget>,
    #[cfg(target_os = "macos")]
    macos_composition_active: bool,
    #[cfg(target_os = "windows")]
    windows_slice_surfaces: Vec<WindowsSliceTarget>,
    #[cfg(target_os = "windows")]
    windows_composition_active: bool,
    #[cfg(target_os = "linux")]
    linux_slice_surfaces: Vec<wgpu::Surface<'static>>,
    #[cfg(target_os = "linux")]
    linux_composition_active: bool,
    #[cfg(target_family = "wasm")]
    web_slice_surfaces: Vec<wgpu::Surface<'static>>,

    targets: FrameTargets,
    blit: BlitState,
    frame_index: u64,
    previous_graph: Option<crate::render::RenderGraph>,
    #[cfg(target_os = "android")]
    android_platform_view_host: Option<crate::android::platform_views::PlatformViewHost>,
    #[cfg(target_os = "ios")]
    ios_platform_views: Option<Rc<RefCell<crate::ios::platform_views::IosPlatformViews>>>,
    #[cfg(target_os = "macos")]
    macos_platform_views: Option<Rc<RefCell<crate::macos_platform_views::MacosPlatformViews>>>,
    #[cfg(target_os = "windows")]
    windows_platform_views:
        Option<Rc<RefCell<crate::windows_platform_views::WindowsPlatformViews>>>,
    #[cfg(target_os = "linux")]
    linux_platform_views: Option<Rc<RefCell<crate::linux_platform_views::LinuxPlatformViews>>>,
    #[cfg(target_family = "wasm")]
    web_platform_views: Option<Rc<RefCell<crate::web_platform_views::WebPlatformViews>>>,
    last_render_breakdown: Option<RenderTimingBreakdown>,
    #[cfg(all(feature = "profiling", not(target_arch = "wasm32")))]
    gpu_timing: Option<gpu_timing::GpuTimingProbe>,
}

#[cfg(target_os = "android")]
struct AndroidSliceTarget {
    output: AndroidSliceOutput,
    _native: crate::android::platform_views::AndroidSliceSurface,
    size: (u32, u32),
}

#[cfg(target_os = "android")]
enum AndroidSliceOutput {
    Wgpu(wgpu::Surface<'static>),
    Egl(android_egl::SliceSurface),
}

#[cfg(target_os = "ios")]
struct IosSliceTarget {
    _layer: objc2::rc::Retained<objc2_quartz_core::CAMetalLayer>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

#[cfg(target_os = "macos")]
struct MacosSliceTarget {
    _layer: objc2::rc::Retained<objc2_quartz_core::CAMetalLayer>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

#[cfg(target_os = "windows")]
struct WindowsSliceTarget {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

/// Shared GPU resources used when creating pipelines.
pub struct RenderResources<'a> {
    /// Renderer-owned resource creation, uploads, and work observations.
    pub graphics: &'a crate::render::GraphicsDevice,
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
    #[cfg(target_family = "wasm")]
    pub(crate) fn web_platform_views(
        &self,
    ) -> Option<Rc<RefCell<crate::web_platform_views::WebPlatformViews>>> {
        self.web_platform_views.clone()
    }

    #[cfg(target_family = "wasm")]
    fn ensure_web_slice_surfaces(&mut self, count: usize) -> Result<(), String> {
        let host = self
            .web_platform_views
            .as_ref()
            .ok_or("Web DOM scene host is unavailable")?;
        while self.web_slice_surfaces.len() < count {
            let canvas = host
                .borrow_mut()
                .create_slice_canvas([self.config.width, self.config.height])?;
            let surface = match self
                .instance
                .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            {
                Ok(surface) => surface,
                Err(error) => {
                    host.borrow_mut().discard_last_slice();
                    return Err(error.to_string());
                }
            };
            let capabilities = surface.get_capabilities(&self.adapter);
            let mut config = self.config.clone();
            if capabilities
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
            {
                config.alpha_mode = wgpu::CompositeAlphaMode::PreMultiplied;
            } else if capabilities
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
            {
                config.alpha_mode = wgpu::CompositeAlphaMode::PostMultiplied;
            } else {
                host.borrow_mut().discard_last_slice();
                return Err("Web scene slice does not support alpha composition".to_owned());
            }
            if !capabilities.formats.contains(&config.format) {
                host.borrow_mut().discard_last_slice();
                return Err("Web scene slice does not support the presentation format".to_owned());
            }
            config.width = self.config.width.max(1);
            config.height = self.config.height.max(1);
            surface.configure(&self.device, &config);
            self.web_slice_surfaces.push(surface);
        }
        for surface in &self.web_slice_surfaces[..count] {
            if let Some(mut config) = surface.get_configuration()
                && (config.width != self.config.width || config.height != self.config.height)
            {
                config.width = self.config.width.max(1);
                config.height = self.config.height.max(1);
                surface.configure(&self.device, &config);
            }
        }
        Ok(())
    }
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

    pub(crate) fn last_render_breakdown(&self) -> Option<RenderTimingBreakdown> {
        self.last_render_breakdown
    }

    #[cfg(target_os = "android")]
    pub(crate) fn set_android_platform_view_host(
        &mut self,
        host: Option<crate::android::platform_views::PlatformViewHost>,
    ) {
        self.android_platform_view_host = host;
    }

    #[cfg(target_os = "ios")]
    pub(crate) fn set_ios_platform_views(
        &mut self,
        platform_views: Rc<RefCell<crate::ios::platform_views::IosPlatformViews>>,
    ) {
        self.ios_platform_views = Some(platform_views);
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn set_macos_platform_views(
        &mut self,
        platform_views: Rc<RefCell<crate::macos_platform_views::MacosPlatformViews>>,
    ) {
        self.macos_platform_views = Some(platform_views);
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn set_windows_platform_views(
        &mut self,
        platform_views: Option<Rc<RefCell<crate::windows_platform_views::WindowsPlatformViews>>>,
    ) {
        self.windows_platform_views = platform_views;
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn set_linux_platform_views(
        &mut self,
        platform_views: Option<Rc<RefCell<crate::linux_platform_views::LinuxPlatformViews>>>,
    ) {
        self.linux_platform_views = platform_views;
    }

    #[cfg(target_os = "linux")]
    pub(super) fn linux_platform_views(
        &self,
    ) -> Option<&RefCell<crate::linux_platform_views::LinuxPlatformViews>> {
        self.linux_platform_views.as_deref()
    }

    #[cfg(target_os = "ios")]
    pub(crate) fn ensure_ios_slice_surfaces(&mut self, count: usize) -> Result<(), String> {
        let root_layer = self
            .ios_platform_views
            .as_ref()
            .and_then(|views| views.borrow().root_layer())
            .ok_or_else(|| "UIKit platform-view root is not attached".to_owned())?;
        while self.ios_slice_surfaces.len() < count {
            let layer = objc2_quartz_core::CAMetalLayer::new();
            layer.setOpaque(false);
            layer.setHidden(true);
            layer.setPresentsWithTransaction(true);
            layer.setFrame(objc2_core_foundation::CGRect::new(
                objc2_core_foundation::CGPoint::new(0.0, 0.0),
                objc2_core_foundation::CGSize::new(self.size.width as f64, self.size.height as f64),
            ));
            layer.setContentsScale(
                self.window
                    .as_ref()
                    .map_or(1.0, |window| window.scale_factor()),
            );
            root_layer.addSublayer(&layer);
            let surface = unsafe {
                self.instance
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(
                        objc2::rc::Retained::as_ptr(&layer).cast_mut().cast(),
                    ))
            }
            .map_err(|error| error.to_string())?;
            let capabilities = surface.get_capabilities(&self.adapter);
            let format = capabilities
                .formats
                .iter()
                .copied()
                .find(|format| *format == self.config.format)
                .ok_or_else(|| {
                    "Core Animation slice does not support the presentation format".to_owned()
                })?;
            let alpha_mode = capabilities
                .alpha_modes
                .iter()
                .copied()
                .find(|mode| *mode != wgpu::CompositeAlphaMode::Opaque)
                .ok_or_else(|| {
                    "Core Animation slice does not support alpha composition".to_owned()
                })?;
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                color_space: wgpu::SurfaceColorSpace::Auto,
                width: self.config.width,
                height: self.config.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode,
                view_formats: Vec::new(),
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&self.device, &config);
            self.ios_slice_surfaces.push(IosSliceTarget {
                _layer: layer,
                surface,
                config,
            });
        }
        for target in &mut self.ios_slice_surfaces[..count] {
            if target.config.width != self.config.width
                || target.config.height != self.config.height
            {
                target.config.width = self.config.width;
                target.config.height = self.config.height;
                target.surface.configure(&self.device, &target.config);
            }
        }
        Ok(())
    }

    #[cfg(target_os = "ios")]
    pub(crate) fn ios_compositor_layers(
        &self,
    ) -> Vec<objc2::rc::Retained<objc2_quartz_core::CALayer>> {
        self.ios_slice_surfaces
            .iter()
            .map(|target| target._layer.clone().into_super())
            .collect()
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn ensure_macos_slice_surfaces(&mut self, count: usize) -> Result<(), String> {
        let root_layer = self
            .macos_platform_views
            .as_ref()
            .and_then(|views| views.borrow().root_layer())
            .ok_or_else(|| "AppKit platform-view root is not attached".to_owned())?;
        while self.macos_slice_surfaces.len() < count {
            let layer = objc2_quartz_core::CAMetalLayer::new();
            layer.setOpaque(false);
            layer.setHidden(true);
            layer.setPresentsWithTransaction(true);
            layer.setFrame(objc2_core_foundation::CGRect::new(
                objc2_core_foundation::CGPoint::new(0.0, 0.0),
                objc2_core_foundation::CGSize::new(self.size.width as f64, self.size.height as f64),
            ));
            layer.setContentsScale(
                self.window
                    .as_ref()
                    .map_or(1.0, |window| window.scale_factor()),
            );
            root_layer.addSublayer(&layer);
            let surface = unsafe {
                self.instance
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(
                        objc2::rc::Retained::as_ptr(&layer).cast_mut().cast(),
                    ))
            }
            .map_err(|error| error.to_string())?;
            let capabilities = surface.get_capabilities(&self.adapter);
            let format = capabilities
                .formats
                .iter()
                .copied()
                .find(|format| *format == self.config.format)
                .ok_or_else(|| {
                    "Core Animation slice does not support the presentation format".to_owned()
                })?;
            let alpha_mode = capabilities
                .alpha_modes
                .iter()
                .copied()
                .find(|mode| *mode != wgpu::CompositeAlphaMode::Opaque)
                .ok_or_else(|| {
                    "Core Animation slice does not support alpha composition".to_owned()
                })?;
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                color_space: wgpu::SurfaceColorSpace::Auto,
                width: self.config.width,
                height: self.config.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode,
                view_formats: Vec::new(),
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&self.device, &config);
            self.macos_slice_surfaces.push(MacosSliceTarget {
                _layer: layer,
                surface,
                config,
            });
        }
        for target in &mut self.macos_slice_surfaces[..count] {
            if target.config.width != self.config.width
                || target.config.height != self.config.height
            {
                target.config.width = self.config.width;
                target.config.height = self.config.height;
                target.surface.configure(&self.device, &target.config);
            }
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn macos_compositor_layers(
        &self,
    ) -> Vec<objc2::rc::Retained<objc2_quartz_core::CALayer>> {
        self.macos_slice_surfaces
            .iter()
            .map(|target| target._layer.clone().into_super())
            .collect()
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn ensure_windows_slice_surfaces(&mut self, count: usize) -> Result<(), String> {
        let host = self
            .windows_platform_views
            .as_ref()
            .ok_or_else(|| "DirectComposition host is unavailable".to_owned())?;
        while self.windows_slice_surfaces.len() < count {
            let visual = host.borrow_mut().create_slice_visual()?;
            let surface = unsafe {
                self.instance
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CompositionVisual(visual))
            }
            .map_err(|error| error.to_string())?;
            let capabilities = surface.get_capabilities(&self.adapter);
            let format = capabilities
                .formats
                .iter()
                .copied()
                .find(|format| *format == self.config.format)
                .ok_or_else(|| "DirectComposition slice format is unsupported".to_owned())?;
            let alpha_mode = wgpu::CompositeAlphaMode::PreMultiplied;
            if !capabilities.alpha_modes.contains(&alpha_mode) {
                return Err("DirectComposition slice requires premultiplied alpha".to_owned());
            }
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                color_space: wgpu::SurfaceColorSpace::Auto,
                width: self.config.width,
                height: self.config.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode,
                view_formats: Vec::new(),
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&self.device, &config);
            self.windows_slice_surfaces
                .push(WindowsSliceTarget { surface, config });
        }
        for target in &mut self.windows_slice_surfaces[..count] {
            if target.config.width != self.config.width
                || target.config.height != self.config.height
            {
                target.config.width = self.config.width;
                target.config.height = self.config.height;
                target.surface.configure(&self.device, &target.config);
            }
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn ensure_linux_slice_surfaces(&mut self, count: usize, scale: f64) -> Result<(), String> {
        let host = self
            .linux_platform_views
            .as_ref()
            .ok_or("Linux platform-view host is unavailable")?;
        while self.linux_slice_surfaces.len() < count {
            let handles = host
                .borrow_mut()
                .create_slice_window([self.config.width, self.config.height])?;
            match self.create_linux_overlay_surface(handles) {
                Ok(surface) => self.linux_slice_surfaces.push(surface),
                Err(error) => {
                    host.borrow_mut().discard_last_slice();
                    return Err(error);
                }
            }
        }
        host.borrow()
            .prepare_slices(count, [self.config.width, self.config.height], scale)?;
        for surface in &self.linux_slice_surfaces[..count] {
            if let Some(mut config) = surface.get_configuration()
                && (config.width != self.config.width || config.height != self.config.height)
            {
                config.width = self.config.width;
                config.height = self.config.height;
                surface.configure(&self.device, &config);
            }
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn create_linux_overlay_surface(
        &self,
        (display, window): (
            winit::raw_window_handle::RawDisplayHandle,
            winit::raw_window_handle::RawWindowHandle,
        ),
    ) -> Result<wgpu::Surface<'static>, String> {
        // SAFETY: The host retains the child and display until after this WGPU surface drops.
        let surface = unsafe {
            self.instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: Some(display),
                    raw_window_handle: window,
                })
        }
        .map_err(|error| error.to_string())?;
        let caps = surface.get_capabilities(&self.adapter);
        if !caps.formats.contains(&self.config.format) {
            return Err("Linux overlay does not support the presentation format".to_owned());
        }
        let alpha_mode = [
            wgpu::CompositeAlphaMode::PreMultiplied,
            wgpu::CompositeAlphaMode::Inherit,
        ]
        .into_iter()
        .find(|mode| caps.alpha_modes.contains(mode))
        .ok_or("Linux overlay surface requires alpha composition")?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.config.format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: self.config.width,
            height: self.config.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&self.device, &config);
        Ok(surface)
    }

    #[cfg(target_os = "android")]
    pub(crate) fn ensure_android_slice_surfaces(&mut self, count: usize) -> Result<(), String> {
        if self.adapter_info.backend == wgpu::Backend::Gl && self.android_egl.is_none() {
            self.android_egl = Some(android_egl::SlicePresenter::new(&self.device)?);
        }
        while self.android_slice_surfaces.len() < count {
            let index = self.android_slice_surfaces.len();
            let width = self.config.width;
            let height = self.config.height;
            let mut slots = Vec::with_capacity(2);
            for slot in 0..2 {
                let native = self
                    .android_platform_view_host
                    .as_ref()
                    .and_then(|host| {
                        host.slice_surface(index as i32, slot, width as i32, height as i32)
                    })
                    .ok_or_else(|| {
                        format!("failed to create Android scene slice {index} buffer {slot}")
                    })?;
                let output = if let Some(presenter) = &self.android_egl {
                    AndroidSliceOutput::Egl(presenter.create_surface(&native, (width, height))?)
                } else {
                    let target = wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle: Some(
                            winit::raw_window_handle::RawDisplayHandle::Android(
                                winit::raw_window_handle::AndroidDisplayHandle::new(),
                            ),
                        ),
                        raw_window_handle: native.raw_window_handle(),
                    };
                    let surface = unsafe { self.instance.create_surface_unsafe(target) }
                        .map_err(|error| error.to_string())?;
                    Self::configure_android_slice(
                        &surface,
                        &self.adapter,
                        &self.device,
                        &self.config,
                    )?;
                    AndroidSliceOutput::Wgpu(surface)
                };
                slots.push(AndroidSliceTarget {
                    output,
                    _native: native,
                    size: (width, height),
                });
            }
            self.android_slice_surfaces.push(slots);
        }
        for (index, slots) in self.android_slice_surfaces[..count].iter_mut().enumerate() {
            for (slot, target) in slots.iter_mut().enumerate() {
                let extent = (self.config.width, self.config.height);
                if target.size != extent {
                    // Resize TextureView producers before EGL re-creates their window
                    // surfaces. Configuring WGPU alone only resizes its offscreen buffers.
                    self.android_platform_view_host
                        .as_ref()
                        .and_then(|host| {
                            host.slice_surface(
                                index as i32,
                                slot as i32,
                                self.config.width as i32,
                                self.config.height as i32,
                            )
                        })
                        .ok_or_else(|| {
                            format!("failed to resize Android scene slice {index} buffer {slot}")
                        })?;
                    match &mut target.output {
                        AndroidSliceOutput::Wgpu(surface) => {
                            Self::configure_android_slice(
                                surface,
                                &self.adapter,
                                &self.device,
                                &self.config,
                            )?;
                        }
                        AndroidSliceOutput::Egl(surface) => {
                            surface.resize(&target._native, extent)?
                        }
                    }
                    target.size = extent;
                }
            }
        }
        Ok(())
    }

    #[cfg(target_os = "android")]
    fn configure_android_slice(
        surface: &wgpu::Surface<'_>,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        scene: &wgpu::SurfaceConfiguration,
    ) -> Result<(), String> {
        let caps = surface.get_capabilities(adapter);
        if !caps.formats.contains(&scene.format) {
            return Err(
                "Android slice surface does not support the renderer's presentation format".into(),
            );
        }
        let alpha_mode = Self::pick_alpha_mode(&caps, true);
        if alpha_mode == wgpu::CompositeAlphaMode::Opaque {
            return Err("Android slice surface does not support alpha composition".into());
        }
        surface.configure(
            device,
            &wgpu::SurfaceConfiguration {
                color_space: wgpu::SurfaceColorSpace::Auto,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode,
                view_formats: Vec::new(),
                desired_maximum_frame_latency: 2,
                ..scene.clone()
            },
        );
        Ok(())
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
