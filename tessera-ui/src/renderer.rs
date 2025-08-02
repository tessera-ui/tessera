//! # Tessera Renderer
//!
//! The core rendering system for the Tessera UI framework. This module provides the main
//! [`Renderer`] struct that manages the application lifecycle, event handling, and rendering
//! pipeline for cross-platform UI applications.
//!
//! ## Overview
//!
//! The renderer is built on top of WGPU and winit, providing:
//! - Cross-platform window management (Windows, Linux, macOS, Android)
//! - Event handling (mouse, touch, keyboard, IME)
//! - Pluggable rendering pipeline system
//! - Component tree management and rendering
//! - Performance monitoring and optimization
//!
//! ## Architecture
//!
//! The renderer follows a modular architecture with several key components:
//!
//! - **[`app`]**: WGPU application management and surface handling
//! - **[`command`]**: Rendering command abstraction
//! - **[`compute`]**: Compute shader pipeline management
//! - **[`drawer`]**: Drawing pipeline management and execution
//!
//! ## Basic Usage
//!
//! The most common way to use the renderer is through the [`Renderer::run`] method:
//!
//! ```rust,no_run
//! use tessera_ui::Renderer;
//!
//! // Define your UI entry point
//! fn my_app() {
//!     // Your UI components go here
//! }
//!
//! // Run the application
//! Renderer::run(
//!     my_app,  // Entry point function
//!     |_app| {
//!         // Register rendering pipelines
//!         // tessera_ui_basic_components::pipelines::register_pipelines(app);
//!     }
//! ).unwrap();
//! ```
//!
//! ## Configuration
//!
//! You can customize the renderer behavior using [`TesseraConfig`]:
//!
//! ```rust,no_run
//! use tessera_ui::{Renderer, renderer::TesseraConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = TesseraConfig {
//!     sample_count: 8,  // 8x MSAA
//! };
//!
//! Renderer::run_with_config(
//!     || { /* my_app */ },
//!     |_app| { /* register_pipelines */ },
//!     config
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Platform Support
//!
//! ### Desktop Platforms (Windows, Linux, macOS)
//!
//! ```rust,ignore
//! use tessera_ui::Renderer;
//! use tessera_ui_macros::tessera;
//!
//! #[tessera] // You need to mark every component function with `#[tessera_macros::tessera]`
//! fn entry_point() {}
//! fn register_pipelines(_: &mut tessera_ui::renderer::WgpuApp) {}
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! Renderer::run(entry_point, register_pipelines)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Android
//!
//! ```rust,no_run
//! use tessera_ui::Renderer;
//! # #[cfg(target_os = "android")]
//! use winit::platform::android::activity::AndroidApp;
//!
//! fn entry_point() {}
//! fn register_pipelines(_: &mut tessera_ui::renderer::WgpuApp) {}
//!
//! # #[cfg(target_os = "android")]
//! fn android_main(android_app: AndroidApp) {
//!     Renderer::run(entry_point, register_pipelines, android_app).unwrap();
//! }
//! ```
//!
//! ## Event Handling
//!
//! The renderer automatically handles various input events:
//!
//! - **Mouse Events**: Click, move, scroll, enter/leave
//! - **Touch Events**: Multi-touch support with gesture recognition
//! - **Keyboard Events**: Key press/release, with platform-specific handling
//! - **IME Events**: Input method support for international text input
//!
//! Events are processed and forwarded to the component tree for handling.
//!
//! ## Performance Monitoring
//!
//! The renderer includes built-in performance monitoring that logs frame statistics
//! when performance drops below 60 FPS:
//!
//! ```text
//! WARN Jank detected! Frame statistics:
//!     Build tree cost: 2.1ms
//!     Draw commands cost: 1.8ms
//!     Render cost: 12.3ms
//!     Total frame cost: 16.2ms
//!     Fps: 61.73
//! ```
//!
//! ## Examples
//!
//! ### Simple Counter Application
//!
//! ```rust,ignore
//! use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
//!
//! use tessera_ui::{Renderer, Color, Dp};
//! use tessera_ui_macros::tessera;
//!
//! struct AppState {
//!     count: AtomicU32,
//! }
//!
//! #[tessera] // You need to mark every component function with `#[tessera_macros::tessera]`
//! fn counter_app(state: Arc<AppState>) {
//!     let _count = state.count.load(Ordering::Relaxed);
//!     // Your UI components would go here
//!     // This is a simplified example without actual UI components
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let state = Arc::new(AppState {
//!         count: AtomicU32::new(0),
//!     });
//!
//!     Renderer::run(
//!         move || counter_app(state.clone()),
//!         |_app| {
//!             // Register your rendering pipelines here
//!             // tessera_ui_basic_components::pipelines::register_pipelines(app);
//!         }
//!     )?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Custom Rendering Pipeline
//!
//! ```rust,no_run
//! use tessera_ui::{Renderer, renderer::WgpuApp};
//!
//! fn register_custom_pipelines(app: &mut WgpuApp) {
//!     // Register basic components first
//!     // tessera_ui_basic_components::pipelines::register_pipelines(app);
//!     
//!     // Add your custom pipelines
//!     // app.drawer.register_pipeline("my_custom_shader", my_pipeline);
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     Renderer::run(
//!         || { /* your UI */ },
//!         register_custom_pipelines
//!     )?;
//!     Ok(())
//! }
//! ```

pub mod app;
pub mod command;
pub mod compute;
pub mod drawer;

use std::{sync::Arc, time::Instant};

use log::{debug, warn};
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    Clipboard, ImeState, PxPosition,
    cursor::{CursorEvent, CursorEventContent, CursorState},
    dp::SCALE_FACTOR,
    keyboard_state::KeyboardState,
    px::PxSize,
    runtime::TesseraRuntime,
    thread_utils, tokio_runtime,
};

pub use app::WgpuApp;
pub use command::Command;
pub use compute::{ComputablePipeline, ComputePipelineRegistry};
pub use drawer::{BarrierRequirement, DrawCommand, DrawablePipeline, PipelineRegistry};

#[cfg(target_os = "android")]
use winit::platform::android::{
    ActiveEventLoopExtAndroid, EventLoopBuilderExtAndroid, activity::AndroidApp,
};

/// Configuration for the Tessera runtime and renderer.
///
/// This struct allows you to customize various aspects of the renderer's behavior,
/// including anti-aliasing settings and other rendering parameters.
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
/// };
///
/// // Disable MSAA for better performance
/// let config = TesseraConfig {
///     sample_count: 1,
/// };
/// ```
#[derive(Clone)]
pub struct TesseraConfig {
    /// The number of samples to use for Multi-Sample Anti-Aliasing (MSAA).
    ///
    /// MSAA helps reduce aliasing artifacts (jagged edges) in rendered graphics
    /// by sampling multiple points per pixel and averaging the results.
    ///
    /// ## Supported Values
    /// - `1`: Disables MSAA (best performance, lower quality)
    /// - `2`: 2x MSAA (moderate performance impact)
    /// - `4`: 4x MSAA (balanced quality/performance)
    /// - `8`: 8x MSAA (high quality, higher performance cost)
    ///
    /// ## Notes
    /// - Higher sample counts provide better visual quality but consume more GPU resources
    /// - The GPU must support the chosen sample count; unsupported values may cause errors
    /// - Mobile devices may have limited support for higher sample counts
    /// - Consider using lower values on resource-constrained devices
    pub sample_count: u32,
}

impl Default for TesseraConfig {
    /// Creates a default configuration with 4x MSAA enabled.
    fn default() -> Self {
        Self { sample_count: 4 }
    }
}

/// The main renderer struct that manages the application lifecycle and rendering.
///
/// The `Renderer` is the core component of the Tessera UI framework, responsible for:
/// - Managing the application window and WGPU context
/// - Handling input events (mouse, touch, keyboard, IME)
/// - Coordinating the component tree building and rendering process
/// - Managing rendering pipelines and resources
///
/// ## Type Parameters
///
/// - `F`: The entry point function type that defines your UI. Must implement `Fn()`.
/// - `R`: The pipeline registration function type. Must implement `Fn(&mut WgpuApp) + Clone + 'static`.
///
/// ## Lifecycle
///
/// The renderer follows this lifecycle:
/// 1. **Initialization**: Create window, initialize WGPU context, register pipelines
/// 2. **Event Loop**: Handle window events, input events, and render requests
/// 3. **Frame Rendering**: Build component tree → Compute draw commands → Render to surface
/// 4. **Cleanup**: Automatic cleanup when the application exits
///
/// ## Thread Safety
///
/// The renderer runs on the main thread and coordinates with other threads for:
/// - Component tree building (potentially parallelized)
/// - Resource management
/// - Event processing
///
/// ## Examples
///
/// See the module-level documentation for usage examples.
pub struct Renderer<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> {
    /// The WGPU application context, initialized after window creation
    app: Option<WgpuApp>,
    /// The entry point function that defines the root of your UI component tree
    entry_point: F,
    /// Tracks cursor/mouse position and button states
    cursor_state: CursorState,
    /// Tracks keyboard key states and events
    keyboard_state: KeyboardState,
    /// Tracks Input Method Editor (IME) state for international text input
    ime_state: ImeState,
    /// Function called during initialization to register rendering pipelines
    register_pipelines_fn: R,
    /// Configuration settings for the renderer
    config: TesseraConfig,
    /// Clipboard manager
    clipboard: Clipboard,
    #[cfg(target_os = "android")]
    /// Android-specific state tracking whether the soft keyboard is currently open
    android_ime_opened: bool,
}

impl<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> Renderer<F, R> {
    #[cfg(not(target_os = "android"))]
    /// Runs the Tessera application with default configuration on desktop platforms.
    ///
    /// This is the most convenient way to start a Tessera application on Windows, Linux, or macOS.
    /// It uses the default [`TesseraConfig`] settings (4x MSAA).
    ///
    /// # Parameters
    ///
    /// - `entry_point`: A function that defines your UI. This function will be called every frame
    ///   to build the component tree. It should contain your root UI components.
    /// - `register_pipelines_fn`: A function that registers rendering pipelines with the WGPU app.
    ///   Typically, you'll call `tessera_ui_basic_components::pipelines::register_pipelines(app)` here.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the application exits normally, or an `EventLoopError` if the
    /// event loop fails to start or encounters a critical error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tessera_ui::Renderer;
    ///
    /// fn my_ui() {
    ///     // Your UI components go here
    /// }
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     Renderer::run(
    ///         my_ui,
    ///         |_app| {
    ///             // Register your rendering pipelines here
    ///             // tessera_ui_basic_components::pipelines::register_pipelines(app);
    ///         }
    ///     )?;
    ///     Ok(())
    /// }
    /// ```
    pub fn run(entry_point: F, register_pipelines_fn: R) -> Result<(), EventLoopError> {
        Self::run_with_config(entry_point, register_pipelines_fn, Default::default())
    }

    #[cfg(not(target_os = "android"))]
    /// Runs the Tessera application with custom configuration on desktop platforms.
    ///
    /// This method allows you to customize the renderer behavior through [`TesseraConfig`].
    /// Use this when you need to adjust settings like MSAA sample count or other rendering parameters.
    ///
    /// # Parameters
    ///
    /// - `entry_point`: A function that defines your UI
    /// - `register_pipelines_fn`: A function that registers rendering pipelines
    /// - `config`: Custom configuration for the renderer
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the application exits normally, or an `EventLoopError` if the
    /// event loop fails to start.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tessera_ui::{Renderer, renderer::TesseraConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TesseraConfig {
    ///     sample_count: 8,  // 8x MSAA for higher quality
    /// };
    ///
    /// Renderer::run_with_config(
    ///     || { /* my_ui */ },
    ///     |_app| { /* register_pipelines */ },
    ///     config
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn run_with_config(
        entry_point: F,
        register_pipelines_fn: R,
        config: TesseraConfig,
    ) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        let app = None;
        let cursor_state = CursorState::default();
        let keyboard_state = KeyboardState::default();
        let ime_state = ImeState::default();
        let clipboard = Clipboard::new();
        let mut renderer = Self {
            app,
            entry_point,
            cursor_state,
            keyboard_state,
            register_pipelines_fn,
            ime_state,
            config,
            clipboard,
        };
        thread_utils::set_thread_name("Tessera Renderer");
        event_loop.run_app(&mut renderer)
    }

    #[cfg(target_os = "android")]
    /// Runs the Tessera application with default configuration on Android.
    ///
    /// This method is specifically for Android applications and requires an `AndroidApp` instance
    /// that is typically provided by the `android_main` function.
    ///
    /// # Parameters
    ///
    /// - `entry_point`: A function that defines your UI
    /// - `register_pipelines_fn`: A function that registers rendering pipelines
    /// - `android_app`: The Android application context
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the application exits normally, or an `EventLoopError` if the
    /// event loop fails to start.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tessera_ui::Renderer;
    /// use winit::platform::android::activity::AndroidApp;
    ///
    /// fn my_ui() {}
    /// fn register_pipelines(_: &mut tessera_ui::renderer::WgpuApp) {}
    ///
    /// #[unsafe(no_mangle)]
    /// fn android_main(android_app: AndroidApp) {
    ///     Renderer::run(
    ///         my_ui,
    ///         register_pipelines,
    ///         android_app
    ///     ).unwrap();
    /// }
    /// ```
    pub fn run(
        entry_point: F,
        register_pipelines_fn: R,
        android_app: AndroidApp,
    ) -> Result<(), EventLoopError> {
        Self::run_with_config(
            entry_point,
            register_pipelines_fn,
            android_app,
            Default::default(),
        )
    }

    #[cfg(target_os = "android")]
    /// Runs the Tessera application with custom configuration on Android.
    ///
    /// This method allows you to customize the renderer behavior on Android through [`TesseraConfig`].
    ///
    /// # Parameters
    ///
    /// - `entry_point`: A function that defines your UI
    /// - `register_pipelines_fn`: A function that registers rendering pipelines
    /// - `android_app`: The Android application context
    /// - `config`: Custom configuration for the renderer
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the application exits normally, or an `EventLoopError` if the
    /// event loop fails to start.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tessera_ui::{Renderer, renderer::TesseraConfig};
    /// use winit::platform::android::activity::AndroidApp;
    ///
    /// fn my_ui() {}
    /// fn register_pipelines(_: &mut tessera_ui::renderer::WgpuApp) {}
    ///
    /// #[unsafe(no_mangle)]
    /// fn android_main(android_app: AndroidApp) {
    ///     let config = TesseraConfig {
    ///         sample_count: 2,  // Lower MSAA for mobile performance
    ///     };
    ///     
    ///     Renderer::run_with_config(
    ///         my_ui,
    ///         register_pipelines,
    ///         android_app,
    ///         config
    ///     ).unwrap();
    /// }
    /// ```
    pub fn run_with_config(
        entry_point: F,
        register_pipelines_fn: R,
        android_app: AndroidApp,
        config: TesseraConfig,
    ) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::builder()
            .with_android_app(android_app.clone())
            .build()
            .unwrap();
        let app = None;
        let cursor_state = CursorState::default();
        let keyboard_state = KeyboardState::default();
        let ime_state = ImeState::default();
        let clipboard = Clipboard::new(android_app);
        let mut renderer = Self {
            app,
            entry_point,
            cursor_state,
            keyboard_state,
            register_pipelines_fn,
            ime_state,
            android_ime_opened: false,
            config,
            clipboard,
        };
        thread_utils::set_thread_name("Tessera Renderer");
        event_loop.run_app(&mut renderer)
    }
}

impl<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> Renderer<F, R> {
    /// Executes a single frame rendering cycle.
    ///
    /// This is the core rendering method that orchestrates the entire frame rendering process.
    /// It follows a three-phase approach:
    ///
    /// 1. **Component Tree Building**: Calls the entry point function to build the UI component tree
    /// 2. **Draw Command Computation**: Processes the component tree to generate rendering commands
    /// 3. **Surface Rendering**: Executes the commands to render the final frame
    ///
    /// ## Performance Monitoring
    ///
    /// This method includes built-in performance monitoring that logs detailed timing information
    /// when frame rates drop below 60 FPS, helping identify performance bottlenecks.
    ///
    /// ## Parameters
    ///
    /// - `entry_point`: The UI entry point function to build the component tree
    /// - `cursor_state`: Mutable reference to cursor/mouse state for event processing
    /// - `keyboard_state`: Mutable reference to keyboard state for event processing
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
    /// This method runs on the main thread but coordinates with other threads for
    /// component tree processing and resource management.
    fn execute_render_frame(
        entry_point: &F,
        cursor_state: &mut CursorState,
        keyboard_state: &mut KeyboardState,
        ime_state: &mut ImeState,
        #[cfg(target_os = "android")] android_ime_opened: &mut bool,
        app: &mut WgpuApp,
        #[cfg(target_os = "android")] event_loop: &ActiveEventLoop,
        clipboard: &mut Clipboard,
    ) {
        // notify the windowing system before rendering
        // this will help winit to properly schedule and make assumptions about its internal state
        app.window.pre_present_notify();
        // and tell runtime the new size
        TesseraRuntime::write().window_size = app.size().into();
        // render the surface
        // Clear any registered callbacks
        TesseraRuntime::write().clear_frame_callbacks();
        // timer for performance measurement
        let tree_timer = Instant::now();
        // build the component tree
        debug!("Building component tree...");
        entry_point();
        let build_tree_cost = tree_timer.elapsed();
        debug!("Component tree built in {build_tree_cost:?}");
        // timer for performance measurement
        let draw_timer = Instant::now();
        // Compute the draw commands then we can clear component tree for next build
        debug!("Computing draw commands...");
        let cursor_position = cursor_state.position();
        let cursor_events = cursor_state.take_events();
        let keyboard_events = keyboard_state.take_events();
        let ime_events = ime_state.take_events();
        let screen_size: PxSize = app.size().into();
        // Clear any existing compute resources
        app.resource_manager.write().clear();
        // Compute the draw commands
        let (commands, window_requests) = TesseraRuntime::write().component_tree.compute(
            screen_size,
            cursor_position,
            cursor_events,
            keyboard_events,
            ime_events,
            keyboard_state.modifiers(),
            app.resource_manager.clone(),
            &app.gpu,
            clipboard,
        );
        let draw_cost = draw_timer.elapsed();
        debug!("Draw commands computed in {draw_cost:?}");
        TesseraRuntime::write().component_tree.clear();
        // Handle the window requests
        // After compute, check for cursor change requests
        // Only set cursor when not at window edges to let window manager handle resize cursors
        let cursor_position = cursor_state.position();
        let window_size = app.size();
        let edge_threshold = 8.0; // Slightly larger threshold for better UX

        let should_set_cursor = if let Some(pos) = cursor_position {
            let x = pos.x.0 as f64;
            let y = pos.y.0 as f64;
            let width = window_size.width as f64;
            let height = window_size.height as f64;

            // Check if cursor is within the safe area (not at edges)
            x > edge_threshold
                && x < width - edge_threshold
                && y > edge_threshold
                && y < height - edge_threshold
        } else {
            false // If no cursor position, disallow setting cursor
        };

        if should_set_cursor {
            app.window
                .set_cursor(winit::window::Cursor::Icon(window_requests.cursor_icon));
        }
        // When cursor is at edges, don't set cursor and let window manager handle it
        // Handle IME requests
        if let Some(ime_request) = window_requests.ime_request {
            app.window.set_ime_allowed(true);
            #[cfg(target_os = "android")]
            {
                if !*android_ime_opened {
                    show_soft_input(true, event_loop.android_app());
                    *android_ime_opened = true;
                }
            }
            app.window.set_ime_cursor_area::<PxPosition, PxSize>(
                ime_request.position.unwrap(),
                ime_request.size,
            );
        } else {
            app.window.set_ime_allowed(false);
            #[cfg(target_os = "android")]
            {
                if *android_ime_opened {
                    hide_soft_input(event_loop.android_app());
                    *android_ime_opened = false;
                }
            }
        }
        // timer for performance measurement
        let render_timer = Instant::now();
        // skip actual rendering if window is minimized
        if TesseraRuntime::read().window_minimized {
            app.window.request_redraw();
            return;
        }
        // Render the commands
        debug!("Rendering draw commands...");
        // Render the commands to the surface
        app.render(commands).unwrap();
        let render_cost = render_timer.elapsed();
        debug!("Rendered to surface in {render_cost:?}");

        // print frame statistics
        let fps = 1.0 / (build_tree_cost + draw_cost + render_cost).as_secs_f32();
        if fps < 60.0 {
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
                build_tree_cost + draw_cost + render_cost,
                1.0 / (build_tree_cost + draw_cost + render_cost).as_secs_f32()
            );
        }

        // Currently we render every frame
        app.window.request_redraw();
    }
}

/// Implementation of winit's `ApplicationHandler` trait for the Tessera renderer.
///
/// This implementation handles the application lifecycle events from winit, including
/// window creation, suspension/resumption, and various window events. It bridges the
/// gap between winit's event system and Tessera's component-based UI framework.
impl<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> ApplicationHandler for Renderer<F, R> {
    /// Called when the application is resumed or started.
    ///
    /// This method is responsible for:
    /// - Creating the application window with appropriate attributes
    /// - Initializing the WGPU context and surface
    /// - Registering rendering pipelines
    /// - Setting up the initial application state
    ///
    /// On desktop platforms, this is typically called once at startup.
    /// On mobile platforms (especially Android), this may be called multiple times
    /// as the app is suspended and resumed.
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
    /// After WGPU initialization, the `register_pipelines_fn` is called to set up
    /// all rendering pipelines. This typically includes basic component pipelines
    /// and any custom shaders your application requires.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Just return if the app is already created
        if self.app.is_some() {
            return;
        }

        // Create a new window
        let window_attributes = Window::default_attributes()
            .with_title("Tessera")
            .with_transparent(true);
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let register_pipelines_fn = self.register_pipelines_fn.clone();

        let mut wgpu_app =
            tokio_runtime::get().block_on(WgpuApp::new(window, self.config.sample_count));

        // Register pipelines
        wgpu_app.register_pipelines(register_pipelines_fn);

        self.app = Some(wgpu_app);
    }

    /// Called when the application is suspended.
    ///
    /// This method should handle cleanup and state preservation when the application
    /// is being suspended (e.g., on mobile platforms when the app goes to background).
    ///
    /// ## Current Status
    ///
    /// This method is currently not fully implemented (`todo!`). In a complete
    /// implementation, it should:
    /// - Save application state
    /// - Release GPU resources if necessary
    /// - Prepare for potential termination
    ///
    /// ## Platform Considerations
    ///
    /// - **Desktop**: Rarely called, mainly during shutdown
    /// - **Android**: Called when app goes to background
    /// - **iOS**: Called during app lifecycle transitions
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        todo!("Handle suspend event");
    }

    /// Handles window-specific events from the windowing system.
    ///
    /// This method processes all window events including user input, window state changes,
    /// and rendering requests. It's the main event processing hub that translates winit
    /// events into Tessera's internal event system.
    ///
    /// ## Event Categories
    ///
    /// ### Window Management
    /// - `CloseRequested`: User requested to close the window
    /// - `Resized`: Window size changed
    /// - `ScaleFactorChanged`: Display scaling changed (high-DPI support)
    ///
    /// ### Input Events
    /// - `CursorMoved`: Mouse cursor position changed
    /// - `CursorLeft`: Mouse cursor left the window
    /// - `MouseInput`: Mouse button press/release
    /// - `MouseWheel`: Mouse wheel scrolling
    /// - `Touch`: Touch screen interactions (mobile)
    /// - `KeyboardInput`: Keyboard key press/release
    /// - `Ime`: Input Method Editor events (international text input)
    ///
    /// ### Rendering
    /// - `RedrawRequested`: System requests a frame to be rendered
    ///
    /// ## Event Processing Flow
    ///
    /// 1. **Input Events**: Captured and stored in respective state managers
    /// 2. **State Updates**: Internal state (cursor, keyboard, IME) is updated
    /// 3. **Rendering**: On redraw requests, the full rendering pipeline is executed
    ///
    /// ## Platform-Specific Handling
    ///
    /// Some events have platform-specific behavior, particularly:
    /// - Touch events (mobile platforms)
    /// - IME events (different implementations per platform)
    /// - Scale factor changes (high-DPI displays)
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let app = match self.app.as_mut() {
            Some(app) => app,
            None => return,
        };

        // Handle window events
        match event {
            WindowEvent::CloseRequested => {
                TesseraRuntime::read().trigger_close_callbacks();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    // Window minimize handling & callback API
                    if !TesseraRuntime::write().window_minimized {
                        TesseraRuntime::write().window_minimized = true;
                        TesseraRuntime::read().trigger_minimize_callbacks(true);
                    }
                } else {
                    // Window (un)minimize handling & callback API
                    if TesseraRuntime::write().window_minimized {
                        TesseraRuntime::write().window_minimized = false;
                        TesseraRuntime::read().trigger_minimize_callbacks(false);
                    }
                    app.resize(size);
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                // Update cursor position
                self.cursor_state
                    .update_position(PxPosition::from_f64_arr2([position.x, position.y]));
                debug!("Cursor moved to: {}, {}", position.x, position.y);
            }
            WindowEvent::CursorLeft { device_id: _ } => {
                // Clear cursor position when it leaves the window
                // This also set the position to None
                self.cursor_state.clear();
                debug!("Cursor left the window");
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                let Some(event_content) = CursorEventContent::from_press_event(state, button)
                else {
                    return; // Ignore unsupported buttons
                };
                let event = CursorEvent {
                    timestamp: Instant::now(),
                    content: event_content,
                };
                self.cursor_state.push_event(event);
                debug!("Mouse input: {state:?} button {button:?}");
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                let event_content = CursorEventContent::from_scroll_event(delta);
                let event = CursorEvent {
                    timestamp: Instant::now(),
                    content: event_content,
                };
                self.cursor_state.push_event(event);
                debug!("Mouse scroll: {delta:?}");
            }
            WindowEvent::Touch(touch_event) => {
                let pos =
                    PxPosition::from_f64_arr2([touch_event.location.x, touch_event.location.y]);
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
                        if let Some(scroll_event) =
                            self.cursor_state.handle_touch_move(touch_event.id, pos)
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
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                *SCALE_FACTOR.get().unwrap().write() = scale_factor;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                debug!("Keyboard input: {event:?}");
                self.keyboard_state.push_event(event);
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
                app.resize_if_needed();
                Self::execute_render_frame(
                    &self.entry_point,
                    &mut self.cursor_state,
                    &mut self.keyboard_state,
                    &mut self.ime_state,
                    #[cfg(target_os = "android")]
                    &mut self.android_ime_opened,
                    app,
                    #[cfg(target_os = "android")]
                    event_loop,
                    &mut self.clipboard,
                );
            }
            _ => (),
        }
    }
}

/// Shows the Android soft keyboard (virtual keyboard).
///
/// This function uses JNI to interact with the Android system to display the soft keyboard.
/// It's specifically designed for Android applications and handles the complex JNI calls
/// required to show the input method.
///
/// ## Parameters
///
/// - `show_implicit`: Whether to show the keyboard implicitly (without explicit user action)
/// - `android_app`: Reference to the Android application context
///
/// ## Platform Support
///
/// This function is only available on Android (`target_os = "android"`). It will not be
/// compiled on other platforms.
///
/// ## Error Handling
///
/// The function includes comprehensive error handling for JNI operations. If any JNI
/// call fails, the function will return early without crashing the application.
/// Exception handling is also included to clear any Java exceptions that might occur.
///
/// ## Implementation Notes
///
/// This implementation is based on the android-activity crate and follows the pattern
/// established in: https://github.com/rust-mobile/android-activity/pull/178
///
/// The function performs these steps:
/// 1. Get the Java VM and activity context
/// 2. Find the InputMethodManager system service
/// 3. Get the current window's decor view
/// 4. Call `showSoftInput` on the InputMethodManager
///
/// ## Usage
///
/// This function is typically called internally by the renderer when IME input is requested.
/// You generally don't need to call this directly in application code.
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
    // showSoftInput can trigger exceptions if the keyboard is currently animating open/closed
    if env.exception_check().unwrap() {
        let _ = env.exception_clear();
    }
}

/// Hides the Android soft keyboard (virtual keyboard).
///
/// This function uses JNI to interact with the Android system to hide the soft keyboard.
/// It's the counterpart to [`show_soft_input`] and handles the complex JNI calls required
/// to dismiss the input method.
///
/// ## Parameters
///
/// - `android_app`: Reference to the Android application context
///
/// ## Platform Support
///
/// This function is only available on Android (`target_os = "android"`). It will not be
/// compiled on other platforms.
///
/// ## Error Handling
///
/// Like [`show_soft_input`], this function includes comprehensive error handling for JNI
/// operations. If any step fails, the function returns early without crashing. Java
/// exceptions are also properly handled and cleared.
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
/// This function is typically called internally by the renderer when IME input is no longer
/// needed. You generally don't need to call this directly in application code.
///
/// ## Relationship to show_soft_input
///
/// This function is designed to work in tandem with [`show_soft_input`]. The renderer
/// automatically manages the keyboard visibility based on IME requests from components.
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
