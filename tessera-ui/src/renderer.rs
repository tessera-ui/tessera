//! The core rendering system for the Tessera UI framework. This module provides
//! the main [`Renderer`] struct that manages the application lifecycle, event
//! handling, and rendering pipeline for cross-platform UI applications.

pub mod composite;
pub mod compute;
pub mod core;
pub mod drawer;
pub mod external;

use std::{sync::Arc, time::Instant};

use accesskit::{self, TreeUpdate};
use accesskit_winit::{Adapter as AccessKitAdapter, Event as AccessKitEvent};
use parking_lot::RwLock;
use tessera_macros::tessera;
use tracing::{debug, error, instrument, warn};
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    ImeState, PxPosition,
    component_tree::{WindowAction, WindowRequests},
    context::begin_frame_context_slots,
    cursor::{CursorEvent, CursorEventContent, CursorState, GestureState},
    dp::SCALE_FACTOR,
    keyboard_state::KeyboardState,
    pipeline_context::PipelineContext,
    plugin::{PluginContext, PluginHost},
    px::PxSize,
    render_graph::{RenderGraph, RenderGraphExecution},
    render_module::RenderModule,
    runtime::{TesseraRuntime, begin_frame_slots},
    thread_utils,
};

use self::core::RenderTimingBreakdown;

pub use crate::render_scene::{Command, DrawRegion, PaddingRect, SampleRegion};

pub use compute::{
    ComputablePipeline, ComputeBatchItem, ComputePipelineRegistry, ErasedComputeBatchItem,
};
pub use core::{RenderCore, RenderResources};
pub use drawer::{DrawCommand, DrawablePipeline, PipelineRegistry};
pub use external::{ExternalTextureHandle, ExternalTextureRegistry};

#[cfg(feature = "profiling")]
use crate::profiler::{
    FrameMeta, Phase as ProfilerPhase, ScopeGuard as ProfilerScopeGuard,
    begin_frame as profiler_begin_frame, end_frame as profiler_end_frame, submit_frame_meta,
};
#[cfg(feature = "profiling")]
use std::path::PathBuf;

#[cfg(target_os = "android")]
use winit::platform::android::{
    ActiveEventLoopExtAndroid, EventLoopBuilderExtAndroid, activity::AndroidApp,
};

type RenderComputationOutput = (RenderGraph, WindowRequests, std::time::Duration);

/// Window creation options for desktop platforms.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Whether to show the system window decorations (title bar and borders).
    pub decorations: bool,
    /// Whether the window supports transparency.
    pub transparent: bool,
    /// Whether the window is resizable.
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            decorations: true,
            transparent: true,
            resizable: true,
        }
    }
}

/// Configuration for the Tessera runtime and renderer.
///
/// This struct allows you to customize various aspects of the renderer's
/// behavior, including anti-aliasing settings and other rendering parameters.
///
/// # Examples
///
/// ```
/// use tessera_ui::renderer::TesseraConfig;
///
/// // Default configuration (4x MSAA)
/// let config = TesseraConfig::default();
///
/// // Custom configuration with 8x MSAA
/// let config = TesseraConfig {
///     sample_count: 8,
///     ..Default::default()
/// };
///
/// // Disable MSAA for better performance
/// let config = TesseraConfig {
///     sample_count: 1,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct TesseraConfig {
    /// The number of samples to use for Multi-Sample Anti-Aliasing (MSAA).
    ///
    /// MSAA helps reduce aliasing artifacts (jagged edges) in rendered graphics
    /// by sampling multiple points per pixel and averaging the results.
    ///
    /// ## Supported Values
    /// - `1`: Disables MSAA (best performance, lower quality)
    /// - `4`: 4x MSAA (balanced quality/performance)
    /// - `8`: 8x MSAA (high quality, higher performance cost)
    ///
    /// ## Notes
    /// - Higher sample counts provide better visual quality but consume more
    ///   GPU resources
    /// - The GPU must support the chosen sample count; unsupported values may
    ///   cause errors
    /// - Mobile devices may have limited support for higher sample counts
    /// - Consider using lower values on resource-constrained devices
    pub sample_count: u32,
    /// The title of the application window.
    /// Defaults to "Tessera" if not specified.
    pub window_title: String,
    /// Window configuration (desktop only).
    pub window: WindowConfig,
    /// Path to write profiler output when `profiling` is enabled.
    #[cfg(feature = "profiling")]
    pub profiler_output_path: PathBuf,
}

impl Default for TesseraConfig {
    /// Creates a default configuration without MSAA and "Tessera" as the window
    /// title.
    fn default() -> Self {
        Self {
            sample_count: 1,
            window_title: "Tessera".to_string(),
            window: WindowConfig::default(),
            #[cfg(feature = "profiling")]
            profiler_output_path: PathBuf::from("tessera-profiler.jsonl"),
        }
    }
}

/// # Renderer
///
/// The main renderer struct that manages the application lifecycle and
/// rendering.
///
/// The `Renderer` is the core component of the Tessera UI framework,
/// responsible for:
///
/// - Managing the application window and WGPU context
/// - Handling input events (mouse, touch, keyboard, IME)
/// - Coordinating the component tree building and rendering process
/// - Managing rendering pipelines and resources
///
/// ## Type Parameters
///
/// - `F`: The entry point function type that defines your UI. Must implement
///   `Fn()`.
///
/// ## Lifecycle
///
/// The renderer follows this lifecycle:
/// 1. **Initialization**: Create window, initialize WGPU context, register
///    pipelines
/// 2. **Event Loop**: Handle window events, input events, and render requests
/// 3. **Frame Rendering**: Build component tree → Compute draw commands →
///    Render to surface
/// 4. **Cleanup**: Automatic cleanup when the application exits
///
/// ## Thread Safety
///
/// The renderer runs on the main thread and coordinates with other threads for:
/// - Component tree building (potentially parallelized)
/// - Resource management
/// - Event processing
///
/// ## Usage
///
/// ## Basic Usage
///
/// It's suggested to use `cargo-tessera` to create your project from templates
/// which include all necessary setup. However, here's a minimal example of how
/// to use the renderer through the [`Renderer::run`] method:
///
/// ```no_run
/// use tessera_ui::{PipelineContext, RenderModule, Renderer};
///
/// #[derive(Debug)]
/// struct DemoModule;
///
/// impl RenderModule for DemoModule {
///     fn register_pipelines(&self, _context: &mut PipelineContext<'_>) {}
/// }
///
/// // Define your UI entry point
/// fn my_app() {
///     // Your UI components go here
/// }
///
/// // Run the application
/// Renderer::run(my_app, vec![Box::new(DemoModule)]).unwrap();
/// ```
///
/// ### Android Usage
///
/// On android, [`Renderer::run`] requires an additional `AndroidApp` parameter
/// from app context or `android_main` function.
///
/// ## Configuration
///
/// You can customize the renderer behavior by passing [`TesseraConfig`] when
/// using [`Renderer::run_with_config`]. instead of [`Renderer::run`].
///
/// ```no_run
/// use tessera_ui::{PipelineContext, RenderModule, Renderer, renderer::TesseraConfig};
///
/// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// #[derive(Debug)]
/// struct DemoModule;
///
/// impl RenderModule for DemoModule {
///     fn register_pipelines(&self, _context: &mut PipelineContext<'_>) {}
/// }
///
/// let config = TesseraConfig {
///     sample_count: 8,                            // 8x MSAA
///     window_title: "My Tessera App".to_string(), // Custom window title
///     ..Default::default()
/// };
///
/// Renderer::run_with_config(|| { /* my_app */ }, vec![Box::new(DemoModule)], config)?;
/// # Ok(())
/// # }
/// ```
///
/// ## Performance Monitoring
///
/// The renderer includes built-in performance monitoring that logs frame
/// statistics when performance drops below 60 FPS.
pub struct Renderer<F: Fn()> {
    /// The WGPU application context, initialized after window creation
    app: Option<RenderCore>,
    /// The entry point function that defines the root of your UI component tree
    entry_point: F,
    /// Tracks cursor/mouse position and button states
    cursor_state: CursorState,
    /// Tracks keyboard key states and events
    keyboard_state: KeyboardState,
    /// Tracks Input Method Editor (IME) state for international text input
    ime_state: ImeState,
    /// Render modules providing pipelines.
    modules: Vec<Box<dyn RenderModule>>,
    /// Lifecycle hooks for registered plugins.
    plugins: PluginHost,
    /// Configuration settings for the renderer
    config: TesseraConfig,
    /// AccessKit adapter for accessibility support
    accessibility_adapter: Option<AccessKitAdapter>,
    /// Event loop proxy for sending accessibility events
    event_loop_proxy: Option<winit::event_loop::EventLoopProxy<AccessKitEvent>>,
    /// Incrementing frame index for profiling and debugging.
    frame_index: u64,
    /// Whether a window close was requested during the last frame.
    pending_close_requested: bool,
    #[cfg(target_os = "android")]
    /// Android-specific state tracking whether the soft keyboard is currently
    /// open
    android_ime_opened: bool,
}

impl<F: Fn()> Renderer<F> {
    /// Runs the Tessera application with default configuration on desktop
    /// platforms.
    ///
    /// This is the most convenient way to start a Tessera application on
    /// Windows, Linux, or macOS. It uses the default [`TesseraConfig`]
    /// settings (4x MSAA).
    ///
    /// # Parameters
    ///
    /// - `entry_point`: A function that defines your UI. This function will be
    ///   called every frame to build the component tree. It should contain your
    ///   root UI components.
    /// - `modules`: Render modules that register pipelines.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the application exits normally, or an
    /// `EventLoopError` if the event loop fails to start or encounters a
    /// critical error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tessera_ui::{PipelineContext, RenderModule, Renderer};
    ///
    /// #[derive(Debug)]
    /// struct DemoModule;
    ///
    /// impl RenderModule for DemoModule {
    ///     fn register_pipelines(&self, _context: &mut PipelineContext<'_>) {}
    /// }
    ///
    /// fn my_ui() {
    ///     // Your UI components go here
    /// }
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     Renderer::run(my_ui, vec![Box::new(DemoModule)])?;
    ///     Ok(())
    /// }
    /// ```
    #[cfg(not(target_os = "android"))]
    #[tracing::instrument(level = "info", skip(entry_point, modules))]
    pub fn run(entry_point: F, modules: Vec<Box<dyn RenderModule>>) -> Result<(), EventLoopError> {
        Self::run_with_config(entry_point, modules, Default::default())
    }

    /// Runs the Tessera application with custom configuration on desktop
    /// platforms.
    ///
    /// This method allows you to customize the renderer behavior through
    /// [`TesseraConfig`]. Use this when you need to adjust settings like
    /// MSAA sample count or other rendering parameters.
    ///
    /// # Parameters
    ///
    /// - `entry_point`: A function that defines your UI
    /// - `modules`: Render modules that register pipelines
    /// - `config`: Custom configuration for the renderer
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the application exits normally, or an
    /// `EventLoopError` if the event loop fails to start.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tessera_ui::{PipelineContext, RenderModule, Renderer, renderer::TesseraConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// #[derive(Debug)]
    /// struct DemoModule;
    ///
    /// impl RenderModule for DemoModule {
    ///     fn register_pipelines(&self, _context: &mut PipelineContext<'_>) {}
    /// }
    ///
    /// let config = TesseraConfig {
    ///     sample_count: 8, // 8x MSAA for higher quality
    ///     ..Default::default()
    /// };
    ///
    /// Renderer::run_with_config(|| { /* my_ui */ }, vec![Box::new(DemoModule)], config)?;
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(level = "info", skip(entry_point, modules))]
    #[cfg(not(any(target_os = "android")))]
    pub fn run_with_config(
        entry_point: F,
        modules: Vec<Box<dyn RenderModule>>,
        config: TesseraConfig,
    ) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::<AccessKitEvent>::with_user_event().build()?;
        let event_loop_proxy = event_loop.create_proxy();
        let app = None;
        let cursor_state = CursorState::default();
        let keyboard_state = KeyboardState::default();
        let ime_state = ImeState::default();
        #[cfg(feature = "profiling")]
        crate::profiler::set_output_path(&config.profiler_output_path);
        let mut renderer = Self {
            app,
            entry_point,
            cursor_state,
            keyboard_state,
            modules,
            plugins: PluginHost::new(),
            ime_state,
            config,
            accessibility_adapter: None,
            event_loop_proxy: Some(event_loop_proxy),
            frame_index: 0,
            pending_close_requested: false,
        };
        thread_utils::set_thread_name("TesseraMain");
        event_loop.run_app(&mut renderer)
    }

    /// Runs the Tessera application with default configuration on Android.
    ///
    /// This method is specifically for Android applications and requires an
    /// `AndroidApp` instance that is typically provided by the
    /// `android_main` function.
    ///
    /// # Parameters
    ///
    /// - `entry_point`: A function that defines your UI
    /// - `modules`: Render modules that register pipelines
    /// - `android_app`: The Android application context
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the application exits normally, or an
    /// `EventLoopError` if the event loop fails to start.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tessera_ui::Renderer;
    /// use winit::platform::android::activity::AndroidApp;
    ///
    /// fn my_ui() {}
    ///
    /// #[derive(Debug)]
    /// struct DemoModule;
    ///
    /// impl tessera_ui::RenderModule for DemoModule {
    ///     fn register_pipelines(&self, _context: &mut tessera_ui::PipelineContext<'_>) {}
    /// }
    ///
    /// #[unsafe(no_mangle)]
    /// fn android_main(android_app: AndroidApp) {
    ///     Renderer::run(my_ui, vec![Box::new(DemoModule)], android_app).unwrap();
    /// }
    /// ```
    #[cfg(target_os = "android")]
    #[tracing::instrument(level = "info", skip(entry_point, modules, android_app))]
    pub fn run(
        entry_point: F,
        modules: Vec<Box<dyn RenderModule>>,
        android_app: AndroidApp,
    ) -> Result<(), EventLoopError> {
        Self::run_with_config(entry_point, modules, android_app, Default::default())
    }

    /// Runs the Tessera application with custom configuration on Android.
    ///
    /// This method allows you to customize the renderer behavior on Android
    /// through [`TesseraConfig`].
    ///
    /// # Parameters
    ///
    /// - `entry_point`: A function that defines your UI
    /// - `modules`: Render modules that register pipelines
    /// - `android_app`: The Android application context
    /// - `config`: Custom configuration for the renderer
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the application exits normally, or an
    /// `EventLoopError` if the event loop fails to start.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tessera_ui::{Renderer, renderer::TesseraConfig};
    /// use winit::platform::android::activity::AndroidApp;
    ///
    /// fn my_ui() {}
    ///
    /// #[derive(Debug)]
    /// struct DemoModule;
    ///
    /// impl tessera_ui::RenderModule for DemoModule {
    ///     fn register_pipelines(&self, _context: &mut tessera_ui::PipelineContext<'_>) {}
    /// }
    ///
    /// #[unsafe(no_mangle)]
    /// fn android_main(android_app: AndroidApp) {
    ///     let config = TesseraConfig {
    ///         sample_count: 2, // Lower MSAA for mobile performance
    ///     };
    ///
    ///     Renderer::run_with_config(my_ui, vec![Box::new(DemoModule)], android_app, config).unwrap();
    /// }
    /// ```
    #[cfg(target_os = "android")]
    #[tracing::instrument(level = "info", skip(entry_point, modules, android_app))]
    pub fn run_with_config(
        entry_point: F,
        modules: Vec<Box<dyn RenderModule>>,
        android_app: AndroidApp,
        config: TesseraConfig,
    ) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::<AccessKitEvent>::with_user_event()
            .with_android_app(android_app.clone())
            .build()
            .unwrap();
        let event_loop_proxy = event_loop.create_proxy();
        let app = None;
        let cursor_state = CursorState::default();
        let keyboard_state = KeyboardState::default();
        let ime_state = ImeState::default();
        #[cfg(feature = "profiling")]
        crate::profiler::set_output_path(&config.profiler_output_path);
        let mut renderer = Self {
            app,
            entry_point,
            cursor_state,
            keyboard_state,
            modules,
            plugins: PluginHost::new(),
            ime_state,
            android_ime_opened: false,
            config,
            accessibility_adapter: None,
            event_loop_proxy: Some(event_loop_proxy),
            frame_index: 0,
            pending_close_requested: false,
        };
        thread_utils::set_thread_name("TesseraMain");
        event_loop.run_app(&mut renderer)
    }
}

// Helper struct to group render-frame arguments and reduce parameter count.
// Kept private to this module.
struct RenderFrameArgs<'a> {
    pub cursor_state: &'a mut CursorState,
    pub keyboard_state: &'a mut KeyboardState,
    pub ime_state: &'a mut ImeState,
    #[cfg(target_os = "android")]
    pub android_ime_opened: &'a mut bool,
    pub app: &'a mut RenderCore,
    #[cfg(target_os = "android")]
    pub event_loop: &'a ActiveEventLoop,
}

struct RenderFrameContext<'a, F: Fn()> {
    entry_point: &'a F,
    args: &'a mut RenderFrameArgs<'a>,
    accessibility_enabled: bool,
    window_label: &'a str,
    frame_idx: u64,
}

impl<F: Fn()> Renderer<F> {
    fn should_set_cursor_pos(
        cursor_position: Option<crate::PxPosition>,
        window_width: f64,
        window_height: f64,
        edge_threshold: f64,
    ) -> bool {
        if let Some(pos) = cursor_position {
            let x = pos.x.0 as f64;
            let y = pos.y.0 as f64;
            x > edge_threshold
                && x < window_width - edge_threshold
                && y > edge_threshold
                && y < window_height - edge_threshold
        } else {
            false
        }
    }

    /// Executes a single frame rendering cycle.
    ///
    /// This is the core rendering method that orchestrates the entire frame
    /// rendering process. It follows a three-phase approach:
    ///
    /// 1. **Component Tree Building**: Calls the entry point function to build
    ///    the UI component tree
    /// 2. **Draw Command Computation**: Processes the component tree to
    ///    generate rendering commands
    /// 3. **Surface Rendering**: Executes the commands to render the final
    ///    frame
    ///
    /// ## Performance Monitoring
    ///
    /// This method includes built-in performance monitoring that logs detailed
    /// timing information when frame rates drop below 60 FPS, helping
    /// identify performance bottlenecks.
    ///
    /// ## Parameters
    ///
    /// - `entry_point`: The UI entry point function to build the component tree
    /// - `cursor_state`: Mutable reference to cursor/mouse state for event
    ///   processing
    /// - `keyboard_state`: Mutable reference to keyboard state for event
    ///   processing
    /// - `ime_state`: Mutable reference to IME state for text input processing
    /// - `android_ime_opened`: (Android only) Tracks soft keyboard state
    /// - `app`: Mutable reference to the WGPU application context
    /// - `event_loop`: (Android only) Event loop for IME management
    ///
    /// ## Frame Timing Breakdown
    ///
    /// - **Build Tree Cost**: Time spent building the component tree
    /// - **Draw Commands Cost**: Time spent computing rendering commands
    /// - **Render Cost**: Time spent executing GPU rendering commands
    ///
    /// ## Thread Safety
    ///
    /// This method runs on the main thread but coordinates with other threads
    /// for component tree processing and resource management.
    #[instrument(level = "debug", skip(entry_point))]
    fn build_component_tree(entry_point: &F) -> std::time::Duration {
        let tree_timer = Instant::now();
        debug!("Building component tree...");
        begin_frame_slots();
        begin_frame_context_slots();
        TesseraRuntime::with_mut(|rt| rt.begin_frame_trace());
        let _phase_guard = crate::runtime::push_phase(crate::runtime::RuntimePhase::Build);
        entry_wrapper(entry_point);
        let build_tree_cost = tree_timer.elapsed();
        debug!("Component tree built in {build_tree_cost:?}");
        build_tree_cost
    }

    fn log_frame_stats(
        build_tree_cost: std::time::Duration,
        draw_cost: std::time::Duration,
        render_cost: std::time::Duration,
        render_breakdown: Option<RenderTimingBreakdown>,
    ) {
        let total = build_tree_cost + draw_cost + render_cost;
        let fps = 1.0 / total.as_secs_f32();
        if fps < 60.0 {
            if let Some(breakdown) = render_breakdown {
                warn!(
                    "Jank detected! Frame statistics:
Build tree cost: {:?}
Draw commands cost: {:?}
Render cost: {:?}
Total frame cost: {:?}
Fps: {:.2}
Render breakdown:
Acquire: {:?}
Build passes: {:?}
Encode: {:?}
Submit: {:?}
Present: {:?}
Render total (core): {:?}
",
                    build_tree_cost,
                    draw_cost,
                    render_cost,
                    total,
                    1.0 / total.as_secs_f32(),
                    breakdown.acquire,
                    breakdown.build_passes,
                    breakdown.encode,
                    breakdown.submit,
                    breakdown.present,
                    breakdown.total,
                );
            } else {
                warn!(
                    "Jank detected! Frame statistics:
Build tree cost: {:?}
Draw commands cost: {:?}
Render cost: {:?}
Total frame cost: {:?}
Fps: {:.2}
",
                    build_tree_cost,
                    draw_cost,
                    render_cost,
                    total,
                    1.0 / total.as_secs_f32()
                );
            }
        }
    }

    #[instrument(level = "debug", skip(args))]
    fn compute_draw_commands<'a>(
        args: &mut RenderFrameArgs<'a>,
        screen_size: PxSize,
        frame_idx: u64,
    ) -> RenderComputationOutput {
        let draw_timer = Instant::now();
        debug!("Computing draw commands...");
        let cursor_position = args.cursor_state.position();
        let cursor_events = args.cursor_state.take_events();
        let keyboard_events = args.keyboard_state.take_events();
        let ime_events = args.ime_state.take_events();

        // Clear any existing compute resources
        args.app.compute_resource_manager().write().clear();

        let (graph, window_requests) = TesseraRuntime::with_mut(|rt| {
            let frame_trace = rt.take_frame_trace();
            let layout_cache = &mut rt.layout_cache;
            let component_tree = &mut rt.component_tree;
            component_tree.compute(crate::component_tree::ComputeParams {
                screen_size,
                cursor_position,
                cursor_events,
                keyboard_events,
                ime_events,
                modifiers: args.keyboard_state.modifiers(),
                compute_resource_manager: args.app.compute_resource_manager(),
                gpu: args.app.device(),
                layout_cache,
                frame_trace,
            })
        });

        let draw_cost = draw_timer.elapsed();
        debug!("Draw commands computed in {draw_cost:?}");
        (graph, window_requests, draw_cost)
    }

    /// Perform the actual GPU rendering for the provided commands and return
    /// the render duration.
    #[instrument(level = "debug", skip(args, execution))]
    fn perform_render<'a>(
        args: &mut RenderFrameArgs<'a>,
        execution: RenderGraphExecution,
    ) -> std::time::Duration {
        #[cfg(feature = "profiling")]
        let _profiler_guard =
            ProfilerScopeGuard::new(ProfilerPhase::RenderFrame, None, None, Some("render_frame"));
        let render_timer = Instant::now();

        // skip actual rendering if window is minimized
        if TesseraRuntime::with(|rt| rt.window_minimized) {
            args.app.window().request_redraw();
            return render_timer.elapsed();
        }

        debug!("Rendering draw commands...");
        if let Err(e) = args.app.render(execution) {
            match e {
                wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost => {
                    debug!("Surface outdated/lost, resizing...");
                    args.app.resize_surface();
                }
                wgpu::SurfaceError::Timeout => warn!("Surface timeout. Frame will be dropped."),
                wgpu::SurfaceError::OutOfMemory => {
                    error!("Surface out of memory. Panicking.");
                    panic!("Surface out of memory");
                }
                _ => {
                    error!("Surface error: {e}. Attempting to continue.");
                }
            }
        }
        let render_cost = render_timer.elapsed();
        debug!("Rendered to surface in {render_cost:?}");
        render_cost
    }

    #[instrument(level = "debug", skip(context))]
    fn execute_render_frame(&mut self, context: RenderFrameContext<'_, F>) -> Option<TreeUpdate> {
        let RenderFrameContext {
            entry_point,
            args,
            accessibility_enabled,
            window_label,
            frame_idx,
        } = context;
        #[cfg(feature = "profiling")]
        let frame_timer = std::time::Instant::now();
        #[cfg(feature = "profiling")]
        profiler_begin_frame(frame_idx);
        // notify the windowing system before rendering
        // this will help winit to properly schedule and make assumptions about its
        // internal state
        args.app.window().pre_present_notify();
        // and tell runtime the new size
        TesseraRuntime::with_mut(|rt: &mut TesseraRuntime| rt.window_size = args.app.size().into());
        // Build the component tree and measure time
        let build_tree_cost = Self::build_component_tree(entry_point);

        // Compute draw commands
        let screen_size: PxSize = args.app.size().into();
        let (new_graph, window_requests, draw_cost) =
            Self::compute_draw_commands(args, screen_size, frame_idx);
        let (composite_context, composite_registry) =
            args.app.composite_context_parts(screen_size, frame_idx);
        let new_graph =
            composite::expand_composites(new_graph, composite_context, composite_registry);
        let RenderGraphExecution {
            ops,
            resources,
            external_resources,
        } = new_graph.into_execution();
        #[cfg(feature = "profiling")]
        let mut render_duration_ns: Option<u128> = None;
        // Perform GPU render every frame.
        let render_cost = Self::perform_render(
            args,
            RenderGraphExecution {
                ops,
                resources,
                external_resources,
            },
        );
        // Log frame statistics
        let render_breakdown = args.app.last_render_breakdown();
        Self::log_frame_stats(build_tree_cost, draw_cost, render_cost, render_breakdown);
        #[cfg(feature = "profiling")]
        {
            render_duration_ns = Some(render_cost.as_nanos());
        }

        #[cfg(feature = "profiling")]
        {
            let frame_total_ns = frame_timer.elapsed().as_nanos();
            let nodes = TesseraRuntime::with(|rt| rt.component_tree.profiler_nodes());
            submit_frame_meta(FrameMeta {
                frame_idx,
                render_time_ns: render_duration_ns,
                build_tree_time_ns: Some(build_tree_cost.as_nanos()),
                draw_time_ns: Some(draw_cost.as_nanos()),
                frame_total_ns: Some(frame_total_ns),
                nodes,
            });
        }

        // Prepare accessibility tree update before clearing the component tree if
        // needed
        let accessibility_update = if accessibility_enabled {
            Self::build_accessibility_update(window_label)
        } else {
            None
        };

        #[cfg(feature = "profiling")]
        profiler_end_frame();

        // Clear the component tree (free for next frame)
        TesseraRuntime::with_mut(|rt| rt.component_tree.clear());

        // Handle the window requests (cursor / IME)
        // Only set cursor when not at window edges to let window manager handle resize
        // cursors
        let cursor_position = args.cursor_state.position();
        let window_size = args.app.size();
        let edge_threshold = 8.0; // Slightly larger threshold for better UX

        let should_set_cursor = Self::should_set_cursor_pos(
            cursor_position,
            window_size.width as f64,
            window_size.height as f64,
            edge_threshold,
        );

        if should_set_cursor {
            args.app
                .window()
                .set_cursor(winit::window::Cursor::Icon(window_requests.cursor_icon));
        }

        if let Some(ime_request) = window_requests.ime_request {
            #[cfg(not(target_os = "android"))]
            args.app.window().set_ime_allowed(true);
            #[cfg(target_os = "android")]
            {
                if !*args.android_ime_opened {
                    args.app.window().set_ime_allowed(true);
                    show_soft_input(true, args.event_loop.android_app());
                    *args.android_ime_opened = true;
                }
            }
            if let Some(position) = ime_request.position {
                args.app
                    .window()
                    .set_ime_cursor_area::<PxPosition, PxSize>(position, ime_request.size);
            } else {
                warn!("IME request missing position; skipping IME cursor area update");
            }
        } else {
            #[cfg(not(target_os = "android"))]
            args.app.window().set_ime_allowed(false);
            #[cfg(target_os = "android")]
            {
                if *args.android_ime_opened {
                    args.app.window().set_ime_allowed(false);
                    hide_soft_input(args.event_loop.android_app());
                    *args.android_ime_opened = false;
                }
            }
        }

        if let Some(window_action) = window_requests.window_action {
            self.apply_window_action(args.app.window(), window_action);
        }

        // End of frame cleanup
        args.cursor_state.frame_cleanup();

        // Recycle unused remembered state slots
        crate::runtime::recycle_frame_slots();
        crate::context::recycle_frame_context_slots();
        crate::modifier::clear_modifiers();

        // Request a redraw to keep the event loop spinning for animations.
        args.app.window().request_redraw();

        accessibility_update
    }
}

impl<F: Fn()> Renderer<F> {
    #[cfg(target_os = "android")]
    fn plugin_context(&self, event_loop: &ActiveEventLoop) -> Option<PluginContext> {
        let app = self.app.as_ref()?;
        Some(PluginContext::new(
            app.window_arc(),
            event_loop.android_app().clone(),
        ))
    }

    #[cfg(not(target_os = "android"))]
    fn plugin_context(&self, _event_loop: &ActiveEventLoop) -> Option<PluginContext> {
        let app = self.app.as_ref()?;
        Some(PluginContext::new(app.window_arc()))
    }

    // These keep behavior identical but reduce per-function complexity.
    fn handle_close_requested(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(context) = self.plugin_context(event_loop) {
            self.plugins.shutdown(&context);
        }
        if let Some(ref app) = self.app
            && let Err(e) = app.save_pipeline_cache()
        {
            warn!("Failed to save pipeline cache: {}", e);
        }
        event_loop.exit();
    }

    fn apply_window_action(&mut self, window: &Window, action: WindowAction) {
        match action {
            WindowAction::DragWindow => {
                if let Err(err) = window.drag_window() {
                    warn!("Failed to start window drag: {}", err);
                }
            }
            WindowAction::Minimize => {
                window.set_minimized(true);
            }
            WindowAction::Maximize => {
                window.set_maximized(true);
            }
            WindowAction::ToggleMaximize => {
                window.set_maximized(!window.is_maximized());
            }
            WindowAction::Close => {
                self.pending_close_requested = true;
            }
        }
    }

    fn handle_resized(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // Obtain the app inside the method to avoid holding a mutable borrow across
        // other borrows of `self`.
        let app = match self.app.as_mut() {
            Some(app) => app,
            None => return,
        };

        if size.width == 0 || size.height == 0 {
            TesseraRuntime::with_mut(|rt| {
                if !rt.window_minimized {
                    rt.window_minimized = true;
                }
            });
        } else {
            TesseraRuntime::with_mut(|rt| {
                if rt.window_minimized {
                    rt.window_minimized = false;
                }
            });
            app.resize(size);
        }
    }

    fn handle_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        // Update cursor position
        self.cursor_state
            .update_position(PxPosition::from_f64_arr2([position.x, position.y]));
        debug!("Cursor moved to: {}, {}", position.x, position.y);
    }

    fn handle_cursor_left(&mut self) {
        // Clear cursor position when it leaves the window
        // This also set the position to None
        self.cursor_state.clear();
        debug!("Cursor left the window");
    }

    fn push_accessibility_update(&mut self, tree_update: TreeUpdate) {
        if let Some(adapter) = self.accessibility_adapter.as_mut() {
            adapter.update_if_active(|| tree_update);
        }
    }

    fn send_accessibility_update(&mut self) {
        if let Some(tree_update) = Self::build_accessibility_update(&self.config.window_title) {
            self.push_accessibility_update(tree_update);
        }
    }

    fn build_accessibility_update(window_label: &str) -> Option<TreeUpdate> {
        TesseraRuntime::with(|runtime| {
            let tree = runtime.component_tree.tree();
            let metadatas = runtime.component_tree.metadatas();
            let root_node_id = tree.get_node_id_at(
                std::num::NonZero::new(1).expect("root node index must be non-zero"),
            )?;
            crate::accessibility::build_tree_update(
                tree,
                metadatas,
                root_node_id,
                Some(window_label),
            )
        })
    }

    fn handle_mouse_input(
        &mut self,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    ) {
        let Some(event_content) = CursorEventContent::from_press_event(state, button) else {
            return; // Ignore unsupported buttons
        };
        let event = CursorEvent {
            timestamp: Instant::now(),
            content: event_content,
            gesture_state: GestureState::TapCandidate,
        };
        self.cursor_state.push_event(event);
        debug!("Mouse input: {state:?} button {button:?}");
    }

    fn handle_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        let event_content = CursorEventContent::from_scroll_event(delta);
        let event = CursorEvent {
            timestamp: Instant::now(),
            content: event_content,
            gesture_state: GestureState::Dragged,
        };
        self.cursor_state.push_event(event);
        debug!("Mouse scroll: {delta:?}");
    }

    fn handle_touch(&mut self, touch_event: winit::event::Touch) {
        let pos = PxPosition::from_f64_arr2([touch_event.location.x, touch_event.location.y]);
        debug!(
            "Touch event: id {}, phase {:?}, position {:?}",
            touch_event.id, touch_event.phase, pos
        );
        match touch_event.phase {
            winit::event::TouchPhase::Started => {
                // Use new touch start handling method
                self.cursor_state.handle_touch_start(touch_event.id, pos);
            }
            winit::event::TouchPhase::Moved => {
                // Use new touch move handling method, may generate scroll event
                if let Some(scroll_event) = self.cursor_state.handle_touch_move(touch_event.id, pos)
                {
                    // Scroll event is already added to event queue in handle_touch_move
                    self.cursor_state.push_event(scroll_event);
                }
            }
            winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled => {
                // Use new touch end handling method
                self.cursor_state.handle_touch_end(touch_event.id);
            }
        }
    }

    fn handle_keyboard_input(&mut self, event: winit::event::KeyEvent) {
        debug!("Keyboard input: {event:?}");
        self.keyboard_state.push_event(event);
    }

    fn handle_redraw_requested(
        &mut self,
        #[cfg(target_os = "android")] event_loop: &ActiveEventLoop,
    ) {
        // Borrow the app here to avoid simultaneous mutable borrows of `self`
        let app = match self.app.as_mut() {
            Some(app) => app,
            None => return,
        };

        app.resize_if_needed();
        let mut args = RenderFrameArgs {
            cursor_state: &mut self.cursor_state,
            keyboard_state: &mut self.keyboard_state,
            ime_state: &mut self.ime_state,
            #[cfg(target_os = "android")]
            android_ime_opened: &mut self.android_ime_opened,
            app,
            #[cfg(target_os = "android")]
            event_loop,
        };
        let accessibility_update = self.execute_render_frame(RenderFrameContext {
            entry_point: &self.entry_point,
            args: &mut args,
            accessibility_enabled: self.accessibility_adapter.is_some(),
            window_label: &self.config.window_title,
            frame_idx: self.frame_index,
        });

        self.frame_index = self.frame_index.wrapping_add(1);

        if let Some(tree_update) = accessibility_update {
            self.push_accessibility_update(tree_update);
        }
    }
}

/// Implementation of winit's `ApplicationHandler` trait for the Tessera
/// renderer.
///
/// This implementation handles the application lifecycle events from winit,
/// including window creation, suspension/resumption, and various window events.
/// It bridges the gap between winit's event system and Tessera's
/// component-based UI framework.
impl<F: Fn()> ApplicationHandler<AccessKitEvent> for Renderer<F> {
    /// Called when the application is resumed or started.
    ///
    /// This method is responsible for:
    /// - Creating the application window with appropriate attributes
    /// - Initializing the WGPU context and surface
    /// - Registering rendering pipelines
    /// - Setting up the initial application state
    ///
    /// On desktop platforms, this is typically called once at startup.
    /// On mobile platforms (especially Android), this may be called multiple
    /// times as the app is suspended and resumed.
    ///
    /// ## Window Configuration
    ///
    /// The window is created with:
    /// - Title: "Tessera"
    /// - Transparency: Enabled (allows for transparent backgrounds)
    /// - Default size and position (platform-dependent)
    ///
    /// ## Pipeline Registration
    ///
    /// After WGPU initialization, render modules register pipelines through
    /// [`PipelineContext`]. This typically includes basic component pipelines
    /// and any custom shaders your application requires.
    #[tracing::instrument(level = "debug", skip(self, event_loop))]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        // Just return if the app is already created
        if self.app.is_some() {
            return;
        }

        // Create a new window (initially hidden for AccessKit initialization)
        let window_attributes = Window::default_attributes()
            .with_title(&self.config.window_title)
            .with_decorations(self.config.window.decorations)
            .with_resizable(self.config.window.resizable)
            .with_transparent(self.config.window.transparent)
            .with_visible(false); // Hide initially for AccessKit
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                error!("Failed to create window: {err}");
                return;
            }
        };

        // Initialize AccessKit adapter BEFORE showing the window
        if let Some(proxy) = self.event_loop_proxy.clone() {
            self.accessibility_adapter = Some(AccessKitAdapter::with_event_loop_proxy(
                event_loop, &window, proxy,
            ));
        }

        // Now show the window after AccessKit is initialized
        window.set_visible(true);

        let mut render_core =
            pollster::block_on(RenderCore::new(window.clone(), self.config.sample_count));

        // Register pipelines
        let mut context = PipelineContext::new(&mut render_core);
        for module in &self.modules {
            module.register_pipelines(&mut context);
        }

        self.app = Some(render_core);

        if let Some(context) = self.plugin_context(event_loop) {
            self.plugins.resumed(&context);
        }
    }

    /// Called when the application is suspended.
    ///
    /// This method should handle cleanup and state preservation when the
    /// application is being suspended (e.g., on mobile platforms when the
    /// app goes to background).
    ///
    /// ## Platform Considerations
    ///
    /// - **Desktop**: Rarely called, mainly during shutdown
    /// - **Android**: Called when app goes to background
    /// - **iOS**: Called during app lifecycle transitions
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        debug!("Suspending renderer; tearing down WGPU resources.");

        if let Some(context) = self.plugin_context(event_loop) {
            self.plugins.suspended(&context);
        }

        if let Some(app) = self.app.take() {
            app.compute_resource_manager().write().clear();
        }

        // Clean up AccessKit adapter
        self.accessibility_adapter = None;

        self.cursor_state = CursorState::default();
        self.keyboard_state = KeyboardState::default();
        self.ime_state = ImeState::default();
        #[cfg(target_os = "android")]
        {
            self.android_ime_opened = false;
        }

        TesseraRuntime::with_mut(|runtime| {
            runtime.component_tree.clear();
            runtime.cursor_icon_request = None;
            runtime.window_minimized = false;
            runtime.window_size = [0, 0];
        });
        crate::runtime::reset_slots();
    }

    #[tracing::instrument(level = "debug", skip(self, event_loop))]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.pending_close_requested {
            self.pending_close_requested = false;
            self.handle_close_requested(event_loop);
            return;
        }

        // Forward event to AccessKit adapter
        if let (Some(adapter), Some(app)) = (&mut self.accessibility_adapter, &self.app) {
            adapter.process_event(app.window(), &event);
        }

        // Handle window events
        match event {
            WindowEvent::CloseRequested => {
                self.handle_close_requested(event_loop);
            }
            WindowEvent::Resized(size) => {
                self.handle_resized(size);
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.handle_cursor_moved(position);
            }
            WindowEvent::CursorLeft { device_id: _ } => {
                self.handle_cursor_left();
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                self.handle_mouse_input(state, button);
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                self.handle_mouse_wheel(delta);
            }
            WindowEvent::Touch(touch_event) => {
                self.handle_touch(touch_event);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(scale_factor_lock) = SCALE_FACTOR.get() {
                    *scale_factor_lock.write() = scale_factor;
                } else {
                    let _ = SCALE_FACTOR.set(RwLock::new(scale_factor));
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                debug!("Modifiers changed: {modifiers:?}");
                self.keyboard_state.update_modifiers(modifiers.state());
            }
            WindowEvent::Ime(ime_event) => {
                debug!("IME event: {ime_event:?}");
                self.ime_state.push_event(ime_event);
            }
            WindowEvent::RedrawRequested => {
                #[cfg(target_os = "android")]
                self.handle_redraw_requested(event_loop);
                #[cfg(not(target_os = "android"))]
                self.handle_redraw_requested();
            }
            _ => (),
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AccessKitEvent) {
        use accesskit_winit::WindowEvent as AccessKitWindowEvent;

        if self.accessibility_adapter.is_none() {
            return;
        }

        match event.window_event {
            AccessKitWindowEvent::InitialTreeRequested => {
                self.send_accessibility_update();
            }
            AccessKitWindowEvent::ActionRequested(action_request) => {
                // Dispatch action to the appropriate component handler
                let handled = TesseraRuntime::with(|runtime| {
                    let tree = runtime.component_tree.tree();
                    let metadatas = runtime.component_tree.metadatas();

                    crate::accessibility::dispatch_action(tree, metadatas, action_request)
                });

                if !handled {
                    debug!("Action was not handled by any component");
                }
            }
            AccessKitWindowEvent::AccessibilityDeactivated => {
                debug!("AccessKit deactivated");
            }
        }
    }
}

/// Shows the Android soft keyboard (virtual keyboard).
///
/// This function uses JNI to interact with the Android system to display the
/// soft keyboard. It's specifically designed for Android applications and
/// handles the complex JNI calls required to show the input method.
///
/// ## Parameters
///
/// - `show_implicit`: Whether to show the keyboard implicitly (without explicit
///   user action)
/// - `android_app`: Reference to the Android application context
///
/// ## Platform Support
///
/// This function is only available on Android (`target_os = "android"`). It
/// will not be compiled on other platforms.
///
/// ## Error Handling
///
/// The function includes comprehensive error handling for JNI operations. If
/// any JNI call fails, the function will return early without crashing the
/// application. Exception handling is also included to clear any Java
/// exceptions that might occur.
///
/// ## Implementation Notes
///
/// This implementation is based on the android-activity crate and follows the
/// pattern established in: https://github.com/rust-mobile/android-activity/pull/178
///
/// The function performs these steps:
/// 1. Get the Java VM and activity context
/// 2. Find the InputMethodManager system service
/// 3. Get the current window's decor view
/// 4. Call `showSoftInput` on the InputMethodManager
///
/// ## Usage
///
/// This function is typically called internally by the renderer when IME input
/// is requested. You generally don't need to call this directly in application
/// code.
// https://github.com/rust-mobile/android-activity/pull/178
#[cfg(target_os = "android")]
pub fn show_soft_input(show_implicit: bool, android_app: &AndroidApp) {
    let ctx = android_app;

    let jvm = unsafe { jni::JavaVM::from_raw(ctx.vm_as_ptr().cast()) }.unwrap();
    let na = unsafe { jni::objects::JObject::from_raw(ctx.activity_as_ptr().cast()) };

    let mut env = jvm.attach_current_thread().unwrap();
    if env.exception_check().unwrap() {
        return;
    }
    let class_ctxt = env.find_class("android/content/Context").unwrap();
    if env.exception_check().unwrap() {
        return;
    }
    let ims = env
        .get_static_field(class_ctxt, "INPUT_METHOD_SERVICE", "Ljava/lang/String;")
        .unwrap();
    if env.exception_check().unwrap() {
        return;
    }

    let im_manager = env
        .call_method(
            &na,
            "getSystemService",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[(&ims).into()],
        )
        .unwrap()
        .l()
        .unwrap();
    if env.exception_check().unwrap() {
        return;
    }

    let jni_window = env
        .call_method(&na, "getWindow", "()Landroid/view/Window;", &[])
        .unwrap()
        .l()
        .unwrap();
    if env.exception_check().unwrap() {
        return;
    }
    let view = env
        .call_method(&jni_window, "getDecorView", "()Landroid/view/View;", &[])
        .unwrap()
        .l()
        .unwrap();
    if env.exception_check().unwrap() {
        return;
    }

    let _ = env.call_method(
        im_manager,
        "showSoftInput",
        "(Landroid/view/View;I)Z",
        &[
            jni::objects::JValue::Object(&view),
            if show_implicit {
                (ndk_sys::ANATIVEACTIVITY_SHOW_SOFT_INPUT_IMPLICIT as i32).into()
            } else {
                0i32.into()
            },
        ],
    );
    // showSoftInput can trigger exceptions if the keyboard is currently animating
    // open/closed
    if env.exception_check().unwrap() {
        let _ = env.exception_clear();
    }
}

/// Hides the Android soft keyboard (virtual keyboard).
///
/// This function uses JNI to interact with the Android system to hide the soft
/// keyboard. It's the counterpart to [`show_soft_input`] and handles the
/// complex JNI calls required to dismiss the input method.
///
/// ## Parameters
///
/// - `android_app`: Reference to the Android application context
///
/// ## Platform Support
///
/// This function is only available on Android (`target_os = "android"`). It
/// will not be compiled on other platforms.
///
/// ## Error Handling
///
/// Like [`show_soft_input`], this function includes comprehensive error
/// handling for JNI operations. If any step fails, the function returns early
/// without crashing. Java exceptions are also properly handled and cleared.
///
/// ## Implementation Details
///
/// The function performs these steps:
/// 1. Get the Java VM and activity context
/// 2. Find the InputMethodManager system service
/// 3. Get the current window and its decor view
/// 4. Get the window token from the decor view
/// 5. Call `hideSoftInputFromWindow` on the InputMethodManager
///
/// ## Usage
///
/// This function is typically called internally by the renderer when IME input
/// is no longer needed. You generally don't need to call this directly in
/// application code.
///
/// ## Relationship to show_soft_input
///
/// This function is designed to work in tandem with [`show_soft_input`]. The
/// renderer automatically manages the keyboard visibility based on IME requests
/// from components.
#[cfg(target_os = "android")]
pub fn hide_soft_input(android_app: &AndroidApp) {
    use jni::objects::JValue;

    let ctx = android_app;
    let jvm = match unsafe { jni::JavaVM::from_raw(ctx.vm_as_ptr().cast()) } {
        Ok(jvm) => jvm,
        Err(_) => return, // Early exit if failing to get the JVM
    };
    let activity = unsafe { jni::objects::JObject::from_raw(ctx.activity_as_ptr().cast()) };

    let mut env = match jvm.attach_current_thread() {
        Ok(env) => env,
        Err(_) => return,
    };

    // --- 1. Get the InputMethodManager ---
    // This part is the same as in show_soft_input.
    let class_ctxt = match env.find_class("android/content/Context") {
        Ok(c) => c,
        Err(_) => return,
    };
    let ims_field =
        match env.get_static_field(class_ctxt, "INPUT_METHOD_SERVICE", "Ljava/lang/String;") {
            Ok(f) => f,
            Err(_) => return,
        };
    let ims = match ims_field.l() {
        Ok(s) => s,
        Err(_) => return,
    };

    let im_manager = match env.call_method(
        &activity,
        "getSystemService",
        "(Ljava/lang/String;)Ljava/lang/Object;",
        &[(&ims).into()],
    ) {
        Ok(m) => match m.l() {
            Ok(im) => im,
            Err(_) => return,
        },
        Err(_) => return,
    };

    // --- 2. Get the current window's token ---
    // This is the key step that differs from show_soft_input.
    let window = match env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[]) {
        Ok(w) => match w.l() {
            Ok(win) => win,
            Err(_) => return,
        },
        Err(_) => return,
    };

    let decor_view = match env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[]) {
        Ok(v) => match v.l() {
            Ok(view) => view,
            Err(_) => return,
        },
        Err(_) => return,
    };

    let window_token =
        match env.call_method(&decor_view, "getWindowToken", "()Landroid/os/IBinder;", &[]) {
            Ok(t) => match t.l() {
                Ok(token) => token,
                Err(_) => return,
            },
            Err(_) => return,
        };

    // --- 3. Call hideSoftInputFromWindow ---
    let _ = env.call_method(
        &im_manager,
        "hideSoftInputFromWindow",
        "(Landroid/os/IBinder;I)Z",
        &[
            JValue::Object(&window_token),
            JValue::Int(0), // flags, usually 0
        ],
    );

    // Hiding the keyboard can also cause exceptions, so we clear them.
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
}

/// Entry point wrapper for tessera applications.
///
/// # Why this is needed
///
/// Tessera component entry points must be functions annotated with the
/// `tessera` macro. Unlike some other frameworks, we cannot detect whether a
/// provided closure has been annotated with `tessera`. Wrapping the entry
/// function guarantees it is invoked from a `tessera`-annotated function,
/// ensuring correct behavior regardless of how the user supplied their entry
/// point.
#[tessera(crate)]
fn entry_wrapper(entry: impl Fn()) {
    entry();
}
