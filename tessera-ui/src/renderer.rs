//! The core rendering system for the Tessera UI framework. This module provides
//! the main [`Renderer`] struct that manages the application lifecycle, event
//! handling, and rendering pipeline for cross-platform UI applications.

pub(crate) mod core;

#[cfg(target_os = "macos")]
mod macos_titlebar;

use std::{
    cell::Cell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use accesskit::{self, TreeUpdate};
use accesskit_winit::{Adapter as AccessKitAdapter, Event as AccessKitEvent};
use tracing::{debug, error, instrument, warn};
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{ResizeDirection, Window, WindowId},
};

use crate::{
    ImeRequest, ImeState, PxPosition, ScrollEventContent,
    build_tree::build_component_tree,
    component_tree::{
        LayoutFrameDiagnostics, WindowDragRequest, WindowRequests, clear_layout_snapshots,
    },
    context::reset_context_read_dependencies,
    cursor::{
        CursorEventContent, CursorState, GestureState, MOUSE_POINTER_ID, PointerChange,
        PressKeyEventType,
    },
    dp::set_scale_factor,
    focus::{FocusDirection, flush_pending_focus_callbacks},
    keyboard_state::KeyboardState,
    pipeline_context::PipelineContext,
    plugin::{DesktopPlatformContext, DesktopWindowAction, PluginContext},
    px::PxSize,
    render_graph::RenderGraph,
    render_module::RenderModule,
    runtime::{
        TesseraRuntime, begin_frame_build_invalidations, begin_frame_clock,
        clear_persistent_focus_handles, clear_redraw_waker, has_pending_build_invalidations,
        has_pending_frame_waiters, has_pending_layout_invalidations,
        has_pending_persistent_focus_prune, has_ready_build_invalidations, install_redraw_waker,
        prune_persistent_focus_handles, release_retired_replay_snapshots,
        reset_build_invalidations, reset_focus_read_dependencies, reset_frame_clock,
        reset_layout_dirty_tracking, reset_render_slot_read_dependencies,
        reset_state_read_dependencies, take_layout_dirty_nodes, tick_frame_clock,
    },
    thread_utils,
    time::Instant,
};

use self::core::{RenderCore, RenderTimingBreakdown};

#[cfg(feature = "debug-dirty-overlay")]
use crate::PxRect;
#[cfg(feature = "debug-dirty-overlay")]
use crate::build_tree::{BuildTreeMode, BuildTreeResult};

#[cfg(feature = "profiling")]
use crate::profiler::{
    FrameMeta, Phase as ProfilerPhase, RedrawReason, RuntimeEventKind, RuntimeMeta,
    ScopeGuard as ProfilerScopeGuard, WakeMeta, WakeSource, begin_frame as profiler_begin_frame,
    end_frame as profiler_end_frame, submit_frame_meta, submit_runtime_meta, submit_wake_meta,
};
#[cfg(feature = "profiling")]
use crate::runtime::frame_delta;
#[cfg(feature = "profiling")]
use std::collections::BTreeSet;
#[cfg(feature = "profiling")]
use std::path::PathBuf;

#[cfg(target_family = "wasm")]
use std::cell::RefCell;
#[cfg(target_family = "wasm")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_family = "wasm")]
use web_sys::HtmlCanvasElement;
#[cfg(target_family = "wasm")]
use web_sys::wasm_bindgen::JsCast;
#[cfg(target_family = "wasm")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(target_os = "android")]
use winit::platform::android::{
    ActiveEventLoopExtAndroid, EventLoopBuilderExtAndroid, activity::AndroidApp,
};

#[derive(Debug)]
enum RendererUserEvent {
    AccessKit(AccessKitEvent),
    RuntimeRedrawWake,
    #[cfg(target_family = "wasm")]
    WebInitReady(u64),
}

impl From<AccessKitEvent> for RendererUserEvent {
    fn from(event: AccessKitEvent) -> Self {
        Self::AccessKit(event)
    }
}

type RenderComputationOutput = (
    RenderGraph,
    WindowRequests,
    std::time::Duration,
    LayoutFrameDiagnostics,
    std::time::Duration,
    Option<FocusDirection>,
    bool,
    bool,
);

#[derive(Clone, Copy, Debug, Default)]
struct RuntimePendingWork {
    invalidation_pending: bool,
    layout_pending: bool,
    frame_waiter_pending: bool,
}

pub(crate) const MAX_FRAME_SETTLING_RETRIES: usize = 8;

pub(crate) fn frame_settling_can_progress(pending_keyboard_retry: bool) -> bool {
    pending_keyboard_retry || has_ready_build_invalidations() || has_pending_layout_invalidations()
}

impl RuntimePendingWork {
    fn capture(pending_keyboard_retry: bool) -> Self {
        Self {
            invalidation_pending: has_pending_build_invalidations()
                || crate::dispatcher::has_pending()
                || crate::clipboard::has_pending_requests()
                || pending_keyboard_retry
                || has_pending_persistent_focus_prune()
                || TesseraRuntime::with(|runtime| {
                    runtime.component_tree.has_pending_focus_reveal()
                        || runtime
                            .component_tree
                            .focus_owner()
                            .has_pending_navigation()
                }),
            layout_pending: has_pending_layout_invalidations(),
            frame_waiter_pending: has_pending_frame_waiters(),
        }
    }

    fn requires_redraw(self) -> bool {
        self.invalidation_pending || self.layout_pending || self.frame_waiter_pending
    }

    #[cfg(feature = "profiling")]
    fn redraw_reasons(self) -> Vec<RedrawReason> {
        let mut reasons = Vec::new();
        if self.invalidation_pending || self.layout_pending {
            reasons.push(RedrawReason::RuntimeInvalidation);
        }
        if self.frame_waiter_pending {
            reasons.push(RedrawReason::RuntimeFrameAwaiter);
        }
        reasons
    }
}

#[cfg(feature = "profiling")]
pub(crate) fn resolve_profiler_output_path(config: &TesseraConfig) -> PathBuf {
    if let Ok(path) = std::env::var("TESSERA_PROFILING_OUTPUT")
        && !path.trim().is_empty()
    {
        return PathBuf::from(path);
    }
    if let Some(path) = option_env!("TESSERA_PROFILING_OUTPUT")
        && !path.trim().is_empty()
    {
        return PathBuf::from(path);
    }
    config.profiler_output_path.clone()
}

/// Window creation options for desktop platforms.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Application identifier used to associate desktop windows with their launcher.
    pub application_id: Option<String>,
    /// Icon displayed by desktop window managers.
    pub icon: Option<winit::window::Icon>,
    /// Whether to show the system window decorations (title bar and borders).
    pub decorations: bool,
    /// Integrates the title bar with app content. Keep decorations enabled for
    /// native macOS controls and disable them to draw controls in the app bar on
    /// other desktops. Use [`Self::with_app_titlebar`] to select these settings.
    pub integrated_titlebar: bool,
    /// Whether the window supports transparency.
    pub transparent: bool,
    /// Whether the window is resizable.
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            application_id: None,
            icon: None,
            decorations: true,
            integrated_titlebar: false,
            transparent: true,
            resizable: true,
        }
    }
}

impl WindowConfig {
    /// Integrates the app bar with the desktop window. macOS keeps its native
    /// controls. Other desktops can draw window controls in the app bar.
    pub fn with_app_titlebar(mut self) -> Self {
        if desktop_window_controls_available() {
            self.decorations = cfg!(target_os = "macos");
            self.integrated_titlebar = true;
        }
        self
    }

    fn app_titlebar(&self) -> AppTitleBar {
        if !self.integrated_titlebar || !desktop_window_controls_available() {
            AppTitleBar::None
        } else if cfg!(target_os = "macos") && self.decorations {
            AppTitleBar::Native
        } else if !self.decorations {
            AppTitleBar::Custom
        } else {
            AppTitleBar::None
        }
    }
}

/// The title bar integration available to the current app's content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppTitleBar {
    /// The app bar has no window controls or drag region.
    #[default]
    None,
    /// The app bar can drag the window while the OS draws the controls.
    Native,
    /// The app bar supplies both the drag region and the controls.
    Custom,
}

impl AppTitleBar {
    /// Reads the current window's title bar integration.
    pub fn current() -> Self {
        crate::local_runtime_resource::<Cell<Self>>().get()
    }
}

const fn desktop_window_controls_available() -> bool {
    !cfg!(any(
        target_family = "wasm",
        target_os = "android",
        target_os = "ios"
    ))
}

/// Web host configuration for browser platforms.
#[derive(Debug, Clone, Default)]
pub struct WebConfig {
    /// An optional canvas element id to mount into before falling back to the
    /// default body append behavior.
    pub canvas_id: Option<String>,
}

impl WebConfig {
    /// Returns a config that looks up the canvas with the provided id.
    pub fn with_canvas_id(mut self, canvas_id: impl Into<String>) -> Self {
        self.canvas_id = Some(canvas_id.into());
        self
    }
}

/// Configuration for the UI runtime, window, and renderer.
#[derive(Debug, Clone)]
pub struct TesseraConfig {
    /// MSAA sample count; defaults to `1` (disabled). The GPU must support the chosen count.
    pub sample_count: u32,
    /// Positive D3D12 non-sampler binding capacity; defaults to 65,536. Larger budgets use more memory.
    pub max_non_sampler_bindings: u32,
    /// The title of the application window.
    /// Defaults to "Tessera" if not specified.
    pub window_title: String,
    /// Window configuration (desktop only).
    pub window: WindowConfig,
    /// Web host configuration for browser platforms.
    pub web: WebConfig,
    /// Path to write profiler output when `profiling` is enabled.
    #[cfg(feature = "profiling")]
    pub profiler_output_path: PathBuf,
}

impl Default for TesseraConfig {
    fn default() -> Self {
        Self {
            sample_count: 1,
            max_non_sampler_bindings: DEFAULT_MAX_NON_SAMPLER_BINDINGS,
            window_title: "Tessera".to_string(),
            window: WindowConfig::default(),
            web: WebConfig::default(),
            #[cfg(feature = "profiling")]
            profiler_output_path: PathBuf::from("tessera-profiler.jsonl"),
        }
    }
}

pub(crate) const DEFAULT_MAX_NON_SAMPLER_BINDINGS: u32 = 64 * 1024;

/// Owns a UI window, rendering resources, and event processing.
///
/// Use [`crate::EntryPoint`] for shared desktop, Android, and web startup.
/// The renderer and its component builds run on the UI thread.
pub struct Renderer<F> {
    session: crate::RuntimeSession,
    app: Option<RenderCore>,
    entry_point: F,
    cursor_state: CursorState,
    keyboard_state: KeyboardState,
    ime_state: ImeState,
    ime_bridge_state: RendererImeBridgeState,
    modules: Vec<Box<dyn RenderModule>>,
    config: TesseraConfig,
    accessibility_adapter: Option<AccessKitAdapter>,
    accessibility_active: bool,
    event_loop_proxy: Option<winit::event_loop::EventLoopProxy<RendererUserEvent>>,
    #[cfg(target_os = "android")]
    android_back: Option<crate::android::system_back::BackBridge>,
    pending_desktop_window_action: Rc<Cell<Option<DesktopWindowAction>>>,
    #[cfg(target_os = "macos")]
    macos_titlebar: macos_titlebar::TitleBarInput,
    frame_index: u64,
    redraw_request_pending: Arc<AtomicBool>,
    pending_close_requested: bool,
    resize_in_progress: bool,
    pending_window_size: Option<winit::dpi::PhysicalSize<u32>>,
    #[cfg(target_family = "wasm")]
    pending_web_inits: Rc<RefCell<Vec<(u64, RenderCore)>>>,
    #[cfg(target_family = "wasm")]
    web_init_epoch: u64,
    #[cfg(target_family = "wasm")]
    web_init_in_progress: bool,
    #[cfg(feature = "profiling")]
    pending_redraw_reasons: BTreeSet<RedrawReason>,
}

impl<F, Output> Renderer<F>
where
    F: Fn() -> Output,
{
    #[cfg(target_family = "wasm")]
    fn resolve_web_canvas(config: &TesseraConfig) -> Option<HtmlCanvasElement> {
        let canvas_id = config.web.canvas_id.as_deref()?;
        let window = web_sys::window()?;
        let document = window.document()?;
        let element = document.get_element_by_id(canvas_id)?;
        element.dyn_into::<HtmlCanvasElement>().ok()
    }

    /// Runs the desktop application with default configuration.
    ///
    /// Returns an error if the event loop cannot start or fails.
    #[cfg(all(not(target_os = "android"), not(target_family = "wasm")))]
    #[tracing::instrument(level = "info", skip(entry_point, modules))]
    pub fn run(entry_point: F, modules: Vec<Box<dyn RenderModule>>) -> Result<(), EventLoopError> {
        Self::run_with_config(entry_point, modules, Default::default())
    }

    /// Runs the desktop application with the supplied configuration.
    ///
    /// Returns an error if the event loop cannot start or fails.
    #[tracing::instrument(level = "info", skip(entry_point, modules))]
    #[cfg(all(not(target_os = "android"), not(target_family = "wasm")))]
    pub fn run_with_config(
        entry_point: F,
        modules: Vec<Box<dyn RenderModule>>,
        config: TesseraConfig,
    ) -> Result<(), EventLoopError> {
        let session = crate::RuntimeSession::for_root();
        session.activate();
        let event_loop = EventLoop::<RendererUserEvent>::with_user_event().build()?;
        let event_loop_proxy = event_loop.create_proxy();
        let app = None;
        let cursor_state = CursorState::default();
        let keyboard_state = KeyboardState::default();
        let ime_state = ImeState::default();
        let ime_bridge_state = RendererImeBridgeState::default();
        #[cfg(feature = "profiling")]
        crate::profiler::set_output_path(resolve_profiler_output_path(&config));
        let mut renderer = Self {
            session,
            app,
            entry_point,
            cursor_state,
            keyboard_state,
            modules,
            ime_state,
            ime_bridge_state,
            config,
            accessibility_adapter: None,
            accessibility_active: false,
            event_loop_proxy: Some(event_loop_proxy),
            pending_desktop_window_action: Rc::default(),
            #[cfg(target_os = "macos")]
            macos_titlebar: macos_titlebar::TitleBarInput::default(),
            frame_index: 0,
            redraw_request_pending: Arc::new(AtomicBool::new(false)),
            pending_close_requested: false,
            resize_in_progress: false,
            pending_window_size: None,
            #[cfg(target_family = "wasm")]
            pending_web_inits: Rc::new(RefCell::new(Vec::new())),
            #[cfg(target_family = "wasm")]
            web_init_epoch: 0,
            #[cfg(target_family = "wasm")]
            web_init_in_progress: false,
            #[cfg(feature = "profiling")]
            pending_redraw_reasons: BTreeSet::new(),
        };
        thread_utils::set_thread_name("TesseraMain");
        event_loop.run_app(&mut renderer)
    }

    /// Runs the Tessera application with default configuration on web
    /// platforms.
    #[cfg(target_family = "wasm")]
    #[tracing::instrument(level = "info", skip(entry_point, modules))]
    pub fn run(entry_point: F, modules: Vec<Box<dyn RenderModule>>) -> Result<(), EventLoopError>
    where
        F: 'static,
    {
        Self::run_web_with_config(entry_point, modules, Default::default())
    }

    /// Runs the Tessera application with custom configuration on web
    /// platforms.
    #[cfg(target_family = "wasm")]
    #[tracing::instrument(level = "info", skip(entry_point, modules))]
    pub fn run_web_with_config(
        entry_point: F,
        modules: Vec<Box<dyn RenderModule>>,
        config: TesseraConfig,
    ) -> Result<(), EventLoopError>
    where
        F: 'static,
    {
        let session = crate::RuntimeSession::for_root();
        session.activate();
        let event_loop = EventLoop::<RendererUserEvent>::with_user_event().build()?;
        let event_loop_proxy = event_loop.create_proxy();
        let app = None;
        let cursor_state = CursorState::default();
        let keyboard_state = KeyboardState::default();
        let ime_state = ImeState::default();
        let ime_bridge_state = RendererImeBridgeState::default();
        #[cfg(feature = "profiling")]
        crate::profiler::set_output_path(resolve_profiler_output_path(&config));
        let renderer = Self {
            session,
            app,
            entry_point,
            cursor_state,
            keyboard_state,
            modules,
            ime_state,
            ime_bridge_state,
            config,
            accessibility_adapter: None,
            accessibility_active: false,
            event_loop_proxy: Some(event_loop_proxy),
            pending_desktop_window_action: Rc::default(),
            frame_index: 0,
            redraw_request_pending: Arc::new(AtomicBool::new(false)),
            pending_close_requested: false,
            resize_in_progress: false,
            pending_window_size: None,
            pending_web_inits: Rc::new(RefCell::new(Vec::new())),
            web_init_epoch: 0,
            web_init_in_progress: false,
            #[cfg(feature = "profiling")]
            pending_redraw_reasons: BTreeSet::new(),
        };
        thread_utils::set_thread_name("TesseraMain");
        event_loop.spawn_app(renderer);
        Ok(())
    }

    /// Runs the Android application with default configuration and its activity.
    ///
    /// Returns an error if the event loop cannot start or fails.
    #[cfg(target_os = "android")]
    #[tracing::instrument(level = "info", skip(entry_point, modules, android_app))]
    pub fn run(
        entry_point: F,
        modules: Vec<Box<dyn RenderModule>>,
        android_app: AndroidApp,
    ) -> Result<(), EventLoopError> {
        Self::run_with_config(entry_point, modules, android_app, Default::default())
    }

    /// Runs the Android application with its activity and supplied configuration.
    ///
    /// Returns an error if the event loop cannot start or fails.
    #[cfg(target_os = "android")]
    #[tracing::instrument(level = "info", skip(entry_point, modules, android_app))]
    pub fn run_with_config(
        entry_point: F,
        modules: Vec<Box<dyn RenderModule>>,
        android_app: AndroidApp,
        config: TesseraConfig,
    ) -> Result<(), EventLoopError> {
        let session = crate::RuntimeSession::for_root();
        session.activate();
        let event_loop = EventLoop::<RendererUserEvent>::with_user_event()
            .with_android_app(android_app.clone())
            .build()
            .unwrap();
        let event_loop_proxy = event_loop.create_proxy();
        let android_back = crate::android::system_back::BackBridge::new(
            &android_app,
            session.run(crate::dispatcher::register_current),
        );
        let app = None;
        let cursor_state = CursorState::default();
        let keyboard_state = KeyboardState::default();
        let ime_state = ImeState::default();
        let ime_bridge_state = RendererImeBridgeState::default();
        #[cfg(feature = "profiling")]
        crate::profiler::set_output_path(resolve_profiler_output_path(&config));
        let mut renderer = Self {
            session,
            app,
            entry_point,
            cursor_state,
            keyboard_state,
            modules,
            ime_state,
            ime_bridge_state,
            config,
            accessibility_adapter: None,
            accessibility_active: false,
            event_loop_proxy: Some(event_loop_proxy),
            android_back,
            pending_desktop_window_action: Rc::default(),
            frame_index: 0,
            redraw_request_pending: Arc::new(AtomicBool::new(false)),
            pending_close_requested: false,
            resize_in_progress: false,
            pending_window_size: None,
            #[cfg(feature = "profiling")]
            pending_redraw_reasons: BTreeSet::new(),
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
    pub ime_bridge_state: &'a mut RendererImeBridgeState,
    pub app: &'a mut RenderCore,
}

struct RenderFrameContext<'a, F> {
    entry_point: &'a F,
    args: &'a mut RenderFrameArgs<'a>,
    accessibility_enabled: bool,
    decorations: bool,
    window_label: &'a str,
    frame_idx: u64,
    #[cfg(feature = "profiling")]
    redraw_reasons: Vec<RedrawReason>,
}

struct RenderFrameOutcome {
    accessibility_update: Option<TreeUpdate>,
    request_window_drag: Option<WindowDragRequest>,
    runtime_pending_work: RuntimePendingWork,
    #[cfg(feature = "profiling")]
    frame_meta: FrameMeta,
    #[cfg(feature = "debug-dirty-overlay")]
    overlay_clear_pending: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RendererImeBridgeState {
    current_request: Option<ImeRequest>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RendererImeBridgeUpdate {
    allowed: Option<bool>,
    cursor_area: Option<(PxPosition, PxSize)>,
    text_state: Option<winit::event::ImeTextState>,
    options: Option<winit::window::ImeOptions>,
    warn_missing_position: bool,
    show_keyboard: bool,
    change_cause: winit::event::ImeChangeCause,
}

impl RendererImeBridgeState {
    fn sync_window(&mut self, window: &Window, request: Option<crate::ImeRequest>) {
        let ime_was_allowed = self.current_request.is_some();
        let ime_bridge_update = self.update_request(request);
        if ime_was_allowed && ime_bridge_update.allowed == Some(true) {
            window.set_ime_allowed(false);
        }
        if let Some(options) = ime_bridge_update.options {
            window.set_ime_options(options);
        }
        if let Some(state) = ime_bridge_update.text_state {
            let _ = window.set_ime_text_state(state, ime_bridge_update.change_cause);
        }
        if let Some(allowed) = ime_bridge_update.allowed {
            window.set_ime_allowed(allowed);
        }
        if ime_bridge_update.show_keyboard {
            window.set_ime_allowed(true);
        }
        if let Some((position, size)) = ime_bridge_update.cursor_area {
            window.set_ime_cursor_area::<PxPosition, PxSize>(position, size);
        } else if ime_bridge_update.warn_missing_position {
            warn!("IME request missing position; skipping IME cursor area update");
        }
    }

    #[cfg(test)]
    fn request(&self) -> Option<&ImeRequest> {
        self.current_request.as_ref()
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn receive_event(&mut self, event: &winit::event::Ime) -> Option<crate::focus::FocusHandleId> {
        let request = self.current_request.as_mut()?;
        let client = request.client_id?;
        if let winit::event::Ime::TextState(state) = event {
            // Echoing native edits can restart the input connection and end composition.
            request.text_state.clone_from(state);
        }
        Some(client)
    }

    fn update_request(&mut self, mut request: Option<ImeRequest>) -> RendererImeBridgeUpdate {
        let change_cause = request
            .as_mut()
            .map_or(winit::event::ImeChangeCause::Other, |request| {
                std::mem::take(&mut request.change_cause)
            });

        let show_keyboard = request
            .as_mut()
            .is_some_and(|request| std::mem::take(&mut request.show_keyboard));
        let previous = self.current_request.as_ref();
        let context_changed = previous.map(|request| (request.client_id, request.options))
            != request
                .as_ref()
                .map(|request| (request.client_id, request.options));
        let allowed = context_changed.then_some(request.is_some());
        let cursor_area = request.as_ref().and_then(|next| {
            let old_area =
                previous.and_then(|old| old.position.map(|position| (position, old.size)));
            let area = next.position.map(|position| (position, next.size));
            (context_changed || old_area != area)
                .then_some(area)
                .flatten()
        });
        let text_state = request
            .as_ref()
            .filter(|next| {
                context_changed || previous.is_none_or(|old| old.text_state != next.text_state)
            })
            .map(|next| next.text_state.clone());
        let options = request
            .as_ref()
            .filter(|_| context_changed)
            .map(|next| next.options);
        let warn_missing_position = self.current_request != request
            && request
                .as_ref()
                .is_some_and(|request| request.position.is_none());
        self.current_request = request;
        RendererImeBridgeUpdate {
            change_cause,
            allowed,
            cursor_area,
            text_state,
            options,
            warn_missing_position,
            show_keyboard: show_keyboard && allowed != Some(true),
        }
    }
}

impl<F, Output> Renderer<F>
where
    F: Fn() -> Output,
{
    const RESIZE_EDGE_THRESHOLD: f64 = 8.0;

    #[cfg(target_os = "windows")]
    fn update_native_window_shape(&self, window: &Window) {
        let _session = self.session.enter();
        use winit::platform::windows::{CornerPreference, WindowExtWindows};

        if self.config.window.decorations {
            window.set_corner_preference(CornerPreference::Default);
            return;
        }

        let preference = if window.is_maximized() || window.fullscreen().is_some() {
            CornerPreference::DoNotRound
        } else {
            CornerPreference::Round
        };
        window.set_corner_preference(preference);
    }

    #[cfg(target_os = "macos")]
    fn update_native_window_shape(&self, window: &Window) {
        let _session = self.session.enter();
        use objc2::rc::Retained;
        use objc2_app_kit::NSView;
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        let raw_window_handle = match window.window_handle() {
            Ok(handle) => handle.as_raw(),
            Err(err) => {
                warn!("Failed to fetch native window handle: {}", err);
                return;
            }
        };
        let RawWindowHandle::AppKit(appkit) = raw_window_handle else {
            return;
        };

        // SAFETY: The pointer comes from winit's window handle and is valid for
        // the lifetime of this call on the main thread.
        let create_retained = Retained::<NSView>::retain;
        let Some(ns_view) = (unsafe { create_retained(appkit.ns_view.as_ptr().cast()) }) else {
            warn!("Failed to retain NSView from raw window handle");
            return;
        };

        let radius_px = if self.config.window.decorations
            || window.is_maximized()
            || window.fullscreen().is_some()
        {
            0.0
        } else {
            8.0 * window.scale_factor()
        };

        ns_view.setWantsLayer(true);
        let layer = if let Some(layer) = ns_view.layer() {
            layer
        } else {
            let layer = ns_view.makeBackingLayer();
            ns_view.setLayer(Some(&layer));
            layer
        };
        layer.setCornerRadius(radius_px as _);
        layer.setMasksToBounds(radius_px > 0.0);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    fn update_native_window_shape(&self, _window: &Window) {
        let _session = self.session.enter();
    }

    fn should_set_cursor_pos(
        cursor_position: Option<crate::PxPosition>,
        window_width: f64,
        window_height: f64,
        edge_threshold: f64,
    ) -> bool {
        if let Some(pos) = cursor_position {
            let x = pos.x as f64;
            let y = pos.y as f64;
            x > edge_threshold
                && x < window_width - edge_threshold
                && y > edge_threshold
                && y < window_height - edge_threshold
        } else {
            false
        }
    }

    fn cursor_resize_direction(
        cursor_position: Option<crate::PxPosition>,
        window_size: winit::dpi::PhysicalSize<u32>,
        edge_threshold: f64,
    ) -> Option<ResizeDirection> {
        let position = cursor_position?;
        let x = position.x as f64;
        let y = position.y as f64;
        let width = window_size.width as f64;
        let height = window_size.height as f64;

        let near_left = x <= edge_threshold;
        let near_right = x >= width - edge_threshold;
        let near_top = y <= edge_threshold;
        let near_bottom = y >= height - edge_threshold;

        match (near_left, near_right, near_top, near_bottom) {
            (true, _, true, _) => Some(ResizeDirection::NorthWest),
            (_, true, true, _) => Some(ResizeDirection::NorthEast),
            (true, _, _, true) => Some(ResizeDirection::SouthWest),
            (_, true, _, true) => Some(ResizeDirection::SouthEast),
            (true, _, _, _) => Some(ResizeDirection::West),
            (_, true, _, _) => Some(ResizeDirection::East),
            (_, _, true, _) => Some(ResizeDirection::North),
            (_, _, _, true) => Some(ResizeDirection::South),
            _ => None,
        }
    }

    fn cursor_icon_for_resize(direction: ResizeDirection) -> winit::window::CursorIcon {
        match direction {
            ResizeDirection::East | ResizeDirection::West => winit::window::CursorIcon::EwResize,
            ResizeDirection::North | ResizeDirection::South => winit::window::CursorIcon::NsResize,
            ResizeDirection::NorthEast | ResizeDirection::SouthWest => {
                winit::window::CursorIcon::NeswResize
            }
            ResizeDirection::NorthWest | ResizeDirection::SouthEast => {
                winit::window::CursorIcon::NwseResize
            }
        }
    }

    fn log_frame_stats(
        build_tree_cost: std::time::Duration,
        draw_cost: std::time::Duration,
        render_cost: std::time::Duration,
        render_breakdown: Option<RenderTimingBreakdown>,
    ) {
        let total = build_tree_cost + draw_cost + render_cost;
        let fps = 1.0 / total.as_secs_f32();
        if fps < 30.0 {
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
        retry_focus_move: Option<FocusDirection>,
        retry_focus_reveal: bool,
        is_retry: bool,
    ) -> RenderComputationOutput {
        let draw_timer = Instant::now();
        debug!("Computing draw commands...");
        let cursor_position = args.cursor_state.position();
        let pointer_changes = if is_retry {
            Vec::new()
        } else {
            args.cursor_state.take_events()
        };
        let keyboard_events = if is_retry {
            Vec::new()
        } else {
            args.keyboard_state.take_events()
        };
        let ime_events = if is_retry {
            Vec::new()
        } else {
            args.ime_state.take_events()
        };

        let layout_dirty_nodes = take_layout_dirty_nodes();

        let (
            graph,
            window_requests,
            layout_diagnostics,
            record_cost,
            pending_focus_move_retry,
            pending_focus_reveal_retry,
            pending_keyboard_retry,
        ) = TesseraRuntime::with_mut(|rt| {
            let component_tree = &mut rt.component_tree;
            component_tree.compute(
                crate::component_tree::ComputeParams {
                    screen_size,
                    cursor_position,
                    pointer_changes,
                    keyboard_events,
                    ime_events,
                    retry_focus_move,
                    retry_focus_reveal,
                    modifiers: args.keyboard_state.modifiers(),
                    layout_dirty_nodes,
                },
                crate::component_tree::ComputeMode::Full,
            )
        });
        flush_pending_focus_callbacks();
        crate::component_tree::flush_pending_globally_positioned_callbacks();
        let pending_keyboard_retry = pending_keyboard_retry
            || TesseraRuntime::with(|runtime| runtime.component_tree.has_pending_focus_reveal());

        let draw_cost = draw_timer.elapsed();
        debug!("Draw commands computed in {draw_cost:?}");
        (
            graph,
            window_requests,
            draw_cost,
            layout_diagnostics,
            record_cost,
            pending_focus_move_retry,
            pending_focus_reveal_retry,
            pending_keyboard_retry,
        )
    }

    #[cfg(feature = "debug-dirty-overlay")]
    fn layout_node_overlay_rect(
        metadatas: &crate::component_tree::ComponentNodeMetaDatas,
        node_id: crate::NodeId,
    ) -> Option<PxRect> {
        let metadata = metadatas.get(&node_id)?;
        let abs_position = metadata.abs_position?;
        let computed_data = metadata.computed_data?;
        if computed_data.width.0 <= 0 || computed_data.height.0 <= 0 {
            return None;
        }
        let node_rect = PxRect::from_position_size(
            abs_position,
            PxSize::new(computed_data.width, computed_data.height),
        );
        Some(
            metadata
                .event_clip_rect()
                .and_then(|clip_rect| clip_rect.intersection(&node_rect))
                .unwrap_or(node_rect),
        )
    }

    #[cfg(feature = "debug-dirty-overlay")]
    fn collect_dirty_overlay_rects(
        screen_size: PxSize,
        build_tree_result: &BuildTreeResult,
    ) -> Vec<PxRect> {
        use crate::component_tree::{NodeRole, direct_layout_children};

        if matches!(build_tree_result.mode(), BuildTreeMode::SkipNoInvalidation) {
            return Vec::new();
        }
        if matches!(build_tree_result.mode(), BuildTreeMode::RootRecompose) {
            if build_tree_result.had_invalidations() {
                return vec![PxRect::from_position_size(PxPosition::ZERO, screen_size)];
            }
            return Vec::new();
        }
        TesseraRuntime::with(|rt| {
            let tree = rt.component_tree.tree();
            let metadatas = rt.component_tree.metadatas();
            let mut rects = Vec::new();
            for instance_key in build_tree_result.dirty_replay_roots() {
                let Some(node_id) = rt
                    .component_tree
                    .find_node_id_by_instance_key(*instance_key)
                else {
                    continue;
                };
                let layout_nodes = match tree.get(node_id) {
                    Some(node_ref) if node_ref.get().role == NodeRole::Layout => vec![node_id],
                    Some(_) => direct_layout_children(node_id, tree).to_vec(),
                    None => continue,
                };
                for layout_node_id in layout_nodes {
                    if let Some(rect) = Self::layout_node_overlay_rect(metadatas, layout_node_id) {
                        rects.push(rect);
                    }
                }
            }
            rects
        })
    }

    /// Perform the actual GPU rendering for the provided commands and return
    /// the render duration.
    #[cfg(not(feature = "debug-dirty-overlay"))]
    #[instrument(level = "debug", skip(args, graph))]
    fn perform_render<'a>(
        args: &mut RenderFrameArgs<'a>,
        graph: RenderGraph,
    ) -> std::time::Duration {
        #[cfg(feature = "profiling")]
        let _profiler_guard =
            ProfilerScopeGuard::new(ProfilerPhase::RenderFrame, None, None, Some("render_frame"));
        let render_timer = Instant::now();

        // skip actual rendering if window is minimized
        if TesseraRuntime::with(|rt| rt.window_minimized) {
            return render_timer.elapsed();
        }

        debug!("Rendering draw commands...");
        args.app.render(graph);
        let render_cost = render_timer.elapsed();
        debug!("Rendered to surface in {render_cost:?}");
        render_cost
    }

    #[cfg(feature = "debug-dirty-overlay")]
    #[instrument(level = "debug", skip(args, graph, dirty_overlay_rects))]
    fn perform_render<'a>(
        args: &mut RenderFrameArgs<'a>,
        graph: RenderGraph,
        dirty_overlay_rects: &[PxRect],
    ) -> std::time::Duration {
        #[cfg(feature = "profiling")]
        let _profiler_guard =
            ProfilerScopeGuard::new(ProfilerPhase::RenderFrame, None, None, Some("render_frame"));
        let render_timer = Instant::now();

        // skip actual rendering if window is minimized
        if TesseraRuntime::with(|rt| rt.window_minimized) {
            return render_timer.elapsed();
        }

        debug!("Rendering draw commands...");
        args.app.render(graph, dirty_overlay_rects);
        let render_cost = render_timer.elapsed();
        debug!("Rendered to surface in {render_cost:?}");
        render_cost
    }

    #[instrument(level = "debug", skip(context))]
    fn execute_render_frame(context: RenderFrameContext<'_, F>) -> RenderFrameOutcome {
        let RenderFrameContext {
            entry_point,
            args,
            accessibility_enabled,
            decorations,
            window_label,
            frame_idx,
            #[cfg(feature = "profiling")]
            redraw_reasons,
        } = context;
        #[cfg(feature = "profiling")]
        profiler_begin_frame(frame_idx);
        begin_frame_clock(Instant::now());
        begin_frame_build_invalidations();
        // Tick frame-nanos receivers before build so their state writes are
        // consumed by the current recomposition pass.
        tick_frame_clock();
        // notify the windowing system before rendering
        // this will help winit to properly schedule and make assumptions about
        // its internal state
        args.app.window().pre_present_notify();
        // and tell runtime the new size
        TesseraRuntime::with_mut(|rt| {
            rt.component_tree.begin_frame_diagnostics();
            rt.window_size = args.app.size().into();
        });
        // Build the component tree and measure time
        let mut build_tree_result = build_component_tree(entry_point);
        debug!("Component tree build mode: {:?}", build_tree_result.mode());

        // Compute draw commands
        let screen_size: PxSize = args.app.size().into();
        let (
            mut new_graph,
            mut window_requests,
            mut draw_cost,
            mut layout_diagnostics,
            mut record_cost,
            mut pending_focus_move_retry,
            mut pending_focus_reveal_retry,
            mut pending_keyboard_retry,
        ) = Self::compute_draw_commands(args, screen_size, frame_idx, None, false, false);
        let mut settling_retries = 0usize;
        while pending_focus_move_retry.is_some()
            || pending_focus_reveal_retry
            || frame_settling_can_progress(pending_keyboard_retry)
        {
            let focus_retry = pending_focus_move_retry.is_some()
                || pending_focus_reveal_retry
                || pending_keyboard_retry;
            if settling_retries >= MAX_FRAME_SETTLING_RETRIES
                || (focus_retry && !frame_settling_can_progress(pending_keyboard_retry))
            {
                break;
            }

            settling_retries += 1;
            if focus_retry || has_ready_build_invalidations() {
                build_tree_result.absorb_retry(build_component_tree(entry_point));
            }

            let (
                retry_graph,
                retry_window_requests,
                retry_draw_cost,
                retry_layout_diagnostics,
                retry_record_cost,
                next_focus_move_retry,
                next_focus_reveal_retry,
                next_keyboard_retry,
            ) = Self::compute_draw_commands(
                args,
                screen_size,
                frame_idx,
                pending_focus_move_retry,
                pending_focus_reveal_retry,
                true,
            );
            new_graph = retry_graph;
            window_requests.settle(retry_window_requests);
            draw_cost += retry_draw_cost;
            layout_diagnostics.accumulate_pass(retry_layout_diagnostics);
            record_cost += retry_record_cost;
            pending_focus_move_retry = next_focus_move_retry;
            pending_focus_reveal_retry = next_focus_reveal_retry;
            pending_keyboard_retry = next_keyboard_retry;
        }
        if pending_focus_reveal_retry
            && (has_pending_build_invalidations() || has_pending_layout_invalidations())
        {
            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .focus_owner_mut()
                    .retain_reveal_retry();
            });
        }
        prune_persistent_focus_handles();
        #[cfg(not(feature = "profiling"))]
        let _ = (layout_diagnostics, record_cost);
        #[cfg(feature = "debug-dirty-overlay")]
        let dirty_overlay_rects =
            Self::collect_dirty_overlay_rects(screen_size, &build_tree_result);
        #[cfg(feature = "debug-dirty-overlay")]
        let overlay_clear_pending = !dirty_overlay_rects.is_empty();
        let render_cost = Self::perform_render(
            args,
            new_graph,
            #[cfg(feature = "debug-dirty-overlay")]
            &dirty_overlay_rects,
        );
        // Log frame statistics
        let render_breakdown = args.app.last_render_breakdown();
        Self::log_frame_stats(
            build_tree_result.duration(),
            draw_cost,
            render_cost,
            render_breakdown,
        );

        #[cfg(feature = "profiling")]
        let frame_meta = {
            let render_duration_ns = Some(render_cost.as_nanos());
            let render_acquire_ns = render_breakdown.map(|breakdown| breakdown.acquire.as_nanos());
            let render_build_passes_ns =
                render_breakdown.map(|breakdown| breakdown.build_passes.as_nanos());
            let render_encode_ns = render_breakdown.map(|breakdown| breakdown.encode.as_nanos());
            let render_submit_ns = render_breakdown.map(|breakdown| breakdown.submit.as_nanos());
            let render_present_ns = render_breakdown.map(|breakdown| breakdown.present.as_nanos());
            #[cfg(not(target_arch = "wasm32"))]
            let render_gpu_ns = render_breakdown
                .and_then(|_| args.app.read_gpu_time())
                .map(|time| time.as_nanos());
            #[cfg(target_arch = "wasm32")]
            let render_gpu_ns = None;
            let inter_frame_wait_ns = (frame_idx > 0).then(|| frame_delta().as_nanos());
            let nodes = TesseraRuntime::with_mut(|rt| rt.component_tree.profiler_nodes());
            FrameMeta {
                frame_idx,
                build_mode: build_tree_result.profiler_build_mode(),
                redraw_reasons,
                inter_frame_wait_ns,
                partial_replay_nodes: build_tree_result.partial_replay_nodes(),
                total_nodes_before_build: build_tree_result.total_nodes_before_build(),
                render_time_ns: render_duration_ns,
                render_prepare_ns: None,
                render_acquire_ns,
                render_build_passes_ns,
                render_encode_ns,
                render_submit_ns,
                render_present_ns,
                render_gpu_ns,
                build_tree_time_ns: Some(build_tree_result.duration().as_nanos()),
                draw_time_ns: Some(draw_cost.as_nanos()),
                record_time_ns: Some(record_cost.as_nanos()),
                frame_total_ns: None,
                layout_diagnostics: Some(layout_diagnostics),
                nodes,
            }
        };

        // Prepare accessibility tree update before clearing the component tree
        // if needed
        let accessibility_update = if accessibility_enabled {
            Self::build_accessibility_update(window_label, false)
        } else {
            None
        };

        // Handle the window requests (cursor / IME)
        // Only set cursor when not at window edges to let window manager handle
        // resize cursors
        let cursor_position = args.cursor_state.position();
        let window_size = args.app.size();
        let resize_direction = (desktop_window_controls_available() && !decorations).then(|| {
            Self::cursor_resize_direction(cursor_position, window_size, Self::RESIZE_EDGE_THRESHOLD)
        });

        if let Some(direction) = resize_direction.flatten() {
            let icon = Self::cursor_icon_for_resize(direction);
            args.app
                .window()
                .set_cursor(winit::window::Cursor::Icon(icon));
        } else {
            let should_set_cursor = Self::should_set_cursor_pos(
                cursor_position,
                window_size.width as f64,
                window_size.height as f64,
                Self::RESIZE_EDGE_THRESHOLD,
            );

            if should_set_cursor {
                args.app
                    .window()
                    .set_cursor(winit::window::Cursor::Icon(window_requests.cursor_icon));
            }
        }

        let request_window_drag = window_requests
            .request_window_drag
            .filter(|_| desktop_window_controls_available());

        args.ime_bridge_state
            .sync_window(args.app.window(), window_requests.ime_request);

        // End of frame cleanup
        args.cursor_state.frame_cleanup();

        let mut runtime_pending_work = RuntimePendingWork::capture(pending_keyboard_retry);
        runtime_pending_work.invalidation_pending |=
            args.app.resources().graphics.take_deferred_upload();

        RenderFrameOutcome {
            accessibility_update,
            request_window_drag,
            runtime_pending_work,
            #[cfg(feature = "profiling")]
            frame_meta,
            #[cfg(feature = "debug-dirty-overlay")]
            overlay_clear_pending,
        }
    }
}

impl<F, Output> Renderer<F>
where
    F: Fn() -> Output,
{
    fn desktop_platform_context(&self) -> Option<DesktopPlatformContext> {
        let _session = self.session.enter();
        let app = self.app.as_ref()?;
        let redraw_pending = self.redraw_request_pending.clone();
        let window = app.window_arc();
        let wake_handler = Rc::new(move || {
            Self::try_request_redraw(window.as_ref(), redraw_pending.as_ref());
        });
        Some(DesktopPlatformContext::new(
            app.window_arc(),
            self.pending_desktop_window_action.clone(),
            wake_handler,
        ))
    }

    fn take_pending_desktop_window_action(&self) -> Option<DesktopWindowAction> {
        let _session = self.session.enter();
        self.pending_desktop_window_action.take()
    }

    #[cfg(target_os = "android")]
    fn plugin_context(&self, event_loop: &ActiveEventLoop) -> Option<PluginContext> {
        let _session = self.session.enter();
        let desktop = self.desktop_platform_context()?;
        Some(PluginContext::new(
            desktop,
            event_loop.android_app().clone(),
        ))
    }

    #[cfg(not(target_os = "android"))]
    fn plugin_context(&self, _event_loop: &ActiveEventLoop) -> Option<PluginContext> {
        let _session = self.session.enter();
        let desktop = self.desktop_platform_context()?;
        Some(PluginContext::new(desktop))
    }

    fn try_request_redraw(window: &Window, redraw_pending: &AtomicBool) {
        if redraw_pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            window.request_redraw();
        }
    }

    fn request_redraw_now(&self) {
        let _session = self.session.enter();
        if let Some(app) = self.app.as_ref() {
            Self::try_request_redraw(app.window(), self.redraw_request_pending.as_ref());
        }
    }

    fn install_runtime_redraw_waker(&self) {
        let _session = self.session.enter();
        let Some(proxy) = self.event_loop_proxy.clone() else {
            clear_redraw_waker();
            return;
        };
        install_redraw_waker(Arc::new(move || {
            let _ = proxy.send_event(RendererUserEvent::RuntimeRedrawWake);
        }));
    }

    #[cfg(target_family = "wasm")]
    fn begin_web_initialization(&mut self, window: Arc<Window>) {
        let _session = self.session.enter();
        let Some(proxy) = self.event_loop_proxy.clone() else {
            error!("Missing event loop proxy for web renderer initialization");
            return;
        };

        self.web_init_in_progress = true;
        let epoch = self.web_init_epoch;
        let pending_web_inits = self.pending_web_inits.clone();
        let sample_count = self.config.sample_count;
        let transparent = self.config.window.transparent;
        let max_bindings = self.config.max_non_sampler_bindings;
        spawn_local(async move {
            let render_core =
                RenderCore::new(window, sample_count, transparent, max_bindings).await;
            pending_web_inits.borrow_mut().push((epoch, render_core));
            let _ = proxy.send_event(RendererUserEvent::WebInitReady(epoch));
        });
    }

    #[cfg(target_family = "wasm")]
    fn finish_web_initialization(&mut self, event_loop: &ActiveEventLoop, epoch: u64) -> bool {
        let _session = self.session.enter();
        let pending_render_core = {
            let mut pending_web_inits = self.pending_web_inits.borrow_mut();
            pending_web_inits
                .iter()
                .position(|(ready_epoch, _)| *ready_epoch == epoch)
                .map(|position| pending_web_inits.remove(position).1)
        };
        let Some(mut render_core) = pending_render_core else {
            return false;
        };

        if epoch != self.web_init_epoch {
            return false;
        }

        let mut context = PipelineContext::new(&mut render_core);
        for module in &self.modules {
            module.register_pipelines(&mut context);
        }

        self.app = Some(render_core);
        self.web_init_in_progress = false;
        self.apply_pending_window_size();
        self.install_runtime_redraw_waker();
        #[cfg(feature = "profiling")]
        self.request_redraw_with_reasons(WakeSource::Lifecycle, vec![RedrawReason::Startup]);
        #[cfg(not(feature = "profiling"))]
        self.request_redraw_now();

        if let Some(context) = self.plugin_context(event_loop) {
            let _session = self.session.enter();
            crate::plugin::plugins_resumed(context);
        }
        true
    }

    #[cfg(feature = "profiling")]
    fn request_redraw_with_reasons(&mut self, source: WakeSource, mut reasons: Vec<RedrawReason>) {
        let _session = self.session.enter();
        reasons.sort_unstable();
        reasons.dedup();
        if reasons.is_empty() {
            return;
        }

        self.pending_redraw_reasons.extend(reasons.iter().copied());

        submit_wake_meta(WakeMeta {
            frame_idx: self.frame_index,
            source,
            reasons: reasons.clone(),
        });

        self.request_redraw_now();
    }

    #[cfg(feature = "profiling")]
    fn take_pending_redraw_reasons(&mut self) -> Vec<RedrawReason> {
        let _session = self.session.enter();
        std::mem::take(&mut self.pending_redraw_reasons)
            .into_iter()
            .collect()
    }

    // These keep behavior identical but reduce per-function complexity.
    fn handle_close_requested(&mut self, event_loop: &ActiveEventLoop) {
        let _session = self.session.enter();
        if let Some(context) = self.plugin_context(event_loop) {
            crate::plugin::plugins_shutdown(&context);
        }
        if let Some(ref app) = self.app
            && let Err(e) = app.save_pipeline_cache()
        {
            warn!("Failed to save pipeline cache: {}", e);
        }
        event_loop.exit();
    }

    fn apply_window_drag(&mut self, window: &Window, _request: WindowDragRequest) {
        let _session = self.session.enter();
        if !desktop_window_controls_available() {
            return;
        }
        #[cfg(target_os = "macos")]
        if let Some(press) = _request.mouse_press {
            self.macos_titlebar.handle_press(press);
            self.update_native_window_shape(window);
            return;
        }
        if let Err(err) = window.drag_window() {
            warn!("Failed to start window drag: {}", err);
        }
        self.update_native_window_shape(window);
    }

    fn apply_desktop_window_action(&mut self, window: &Window, action: DesktopWindowAction) {
        let _session = self.session.enter();
        match action {
            DesktopWindowAction::Minimize => {
                window.set_minimized(true);
            }
            DesktopWindowAction::Maximize => {
                window.set_maximized(true);
            }
            DesktopWindowAction::ToggleMaximize => {
                window.set_maximized(!window.is_maximized());
            }
            DesktopWindowAction::Close => {
                self.pending_close_requested = true;
            }
        }
        self.update_native_window_shape(window);
    }

    /// Applies the most recent window size observed before the render core was
    /// installed so the surface matches the real canvas size on the first
    /// frame.
    fn apply_pending_window_size(&mut self) {
        let _session = self.session.enter();
        let live_size = self
            .app
            .as_ref()
            .map(|app| window_drawable_size(app.window()));
        let size = resolve_startup_window_size(self.pending_window_size.take(), live_size);
        let Some(size) = size else {
            return;
        };
        self.handle_resized(size);
    }

    fn handle_resized(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        let _session = self.session.enter();
        // Obtain the app inside the method to avoid holding a mutable borrow
        // across other borrows of `self`.
        let app = match self.app.as_mut() {
            Some(app) => app,
            None => {
                // The render core may still be initializing (asynchronously on
                // the web). Keep the size so it is applied once the core is
                // installed instead of leaving the surface stale until the next
                // resize.
                self.pending_window_size = Some(size);
                return;
            }
        };
        let window = app.window_arc();
        #[cfg(target_family = "wasm")]
        let size = if size.width > 0 && size.height > 0 {
            web_canvas_physical_size(&window).unwrap_or(size)
        } else {
            size
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
        self.update_native_window_shape(&window);
    }

    fn handle_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        let _session = self.session.enter();
        if self.resize_in_progress {
            return;
        }
        let px_position = PxPosition::from_f64_arr2([position.x, position.y]);
        // Update cursor position
        self.cursor_state.update_position(px_position);
        self.cursor_state.push_event(PointerChange::new(
            Instant::now(),
            MOUSE_POINTER_ID,
            CursorEventContent::Moved(px_position),
            GestureState::TapCandidate,
        ));
        debug!("Cursor moved to: {}, {}", position.x, position.y);
    }

    fn handle_cursor_left(&mut self) {
        let _session = self.session.enter();
        // Clear cursor position when it leaves the window
        // This also set the position to None
        self.cursor_state.clear();
        debug!("Cursor left the window");
    }

    fn push_accessibility_update(&mut self, tree_update: TreeUpdate) {
        let _session = self.session.enter();
        if let Some(adapter) = self.accessibility_adapter.as_mut() {
            adapter.update_if_active(|| tree_update);
        }
    }

    fn send_accessibility_update(&mut self) {
        let _session = self.session.enter();
        if let Some(tree_update) = Self::build_accessibility_update(&self.config.window_title, true)
        {
            self.push_accessibility_update(tree_update);
        }
    }

    fn build_accessibility_update(window_label: &str, full: bool) -> Option<TreeUpdate> {
        TesseraRuntime::with_mut(|runtime| {
            runtime
                .component_tree
                .accessibility_update(Some(window_label), full)
        })
    }

    fn handle_mouse_input(
        &mut self,
        state: winit::event::ElementState,
        button: winit::event::MouseButton,
    ) {
        let _session = self.session.enter();
        let Some(event_content) = CursorEventContent::from_press_event(state, button) else {
            return; // Ignore unsupported buttons
        };

        if self.resize_in_progress {
            if matches!(
                event_content,
                CursorEventContent::Released(PressKeyEventType::Left)
            ) {
                self.resize_in_progress = false;
            }
            return;
        }

        if matches!(
            event_content,
            CursorEventContent::Pressed(PressKeyEventType::Left)
        ) && desktop_window_controls_available()
            && !self.config.window.decorations
            && let Some(app) = self.app.as_ref()
        {
            let window_size = app.size();
            let direction = Self::cursor_resize_direction(
                self.cursor_state.position(),
                window_size,
                Self::RESIZE_EDGE_THRESHOLD,
            );
            if let Some(direction) = direction {
                if let Err(err) = app.window().drag_resize_window(direction) {
                    warn!("Failed to start window resize: {}", err);
                } else {
                    self.resize_in_progress = true;
                    self.cursor_state.clear();
                    debug!("Started native border resize; suppressing cursor input");
                }
                return;
            }
        }
        let event = PointerChange::new(
            Instant::now(),
            MOUSE_POINTER_ID,
            event_content,
            GestureState::TapCandidate,
        );
        #[cfg(target_os = "macos")]
        if matches!(
            event.content,
            CursorEventContent::Pressed(PressKeyEventType::Left)
        ) {
            self.macos_titlebar.record_press(event.timestamp);
        }
        if state == winit::event::ElementState::Pressed {
            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .focus_owner_mut()
                    .record_pointer_press(event.timestamp);
            });
        }
        self.cursor_state.push_event(event);
        debug!("Mouse input: {state:?} button {button:?}");
    }

    fn handle_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        let _session = self.session.enter();
        if self.resize_in_progress {
            return;
        }
        let scroll = ScrollEventContent::from_wheel(delta);
        let discrete = crate::scroll::is_discrete_platform_wheel(&scroll);
        if !scroll.delta_x.is_finite()
            || !scroll.delta_y.is_finite()
            || (scroll.delta_x == 0.0 && scroll.delta_y == 0.0)
        {
            return;
        }
        let mut event = PointerChange::new(
            Instant::now(),
            MOUSE_POINTER_ID,
            CursorEventContent::Scroll(scroll),
            GestureState::Dragged,
        );
        event.scroll_discrete = Some(discrete);
        self.cursor_state.push_event(event);
        debug!("Mouse scroll: {delta:?}");
    }

    fn handle_touch(&mut self, touch_event: winit::event::Touch) {
        let _session = self.session.enter();
        if self.resize_in_progress {
            return;
        }
        let pos = PxPosition::from_f64_arr2([touch_event.location.x, touch_event.location.y]);
        debug!(
            "Touch event: id {}, phase {:?}, position {:?}",
            touch_event.id, touch_event.phase, pos
        );
        match touch_event.phase {
            winit::event::TouchPhase::Started => {
                TesseraRuntime::with_mut(|runtime| {
                    runtime
                        .component_tree
                        .focus_owner_mut()
                        .record_pointer_press(Instant::now());
                });
                // Use new touch start handling method
                self.cursor_state.handle_touch_start(touch_event.id, pos);
            }
            winit::event::TouchPhase::Moved => {
                // Use new touch move handling method, may generate scroll event
                if let Some(scroll_event) = self.cursor_state.handle_touch_move(touch_event.id, pos)
                {
                    // Scroll event is already added to event queue in
                    // handle_touch_move
                    self.cursor_state.push_event(scroll_event);
                }
            }
            winit::event::TouchPhase::Ended => {
                self.cursor_state.handle_touch_end(touch_event.id);
            }
            winit::event::TouchPhase::Cancelled => {
                self.cursor_state.handle_touch_cancel(touch_event.id);
            }
        }
    }

    fn handle_keyboard_input(&mut self, event: winit::event::KeyEvent) {
        let _session = self.session.enter();
        debug!("Keyboard input: {event:?}");
        TesseraRuntime::with_mut(|runtime| {
            runtime
                .component_tree
                .focus_owner_mut()
                .record_keyboard_input(event.state, Instant::now());
        });
        self.keyboard_state.push_event(event.into());
    }

    fn flush_focused_input(&mut self) {
        if self.app.is_none() {
            return;
        }
        let modifiers = self.keyboard_state.modifiers();
        let keyboard = self.keyboard_state.take_events();
        let ime = self.ime_state.take_events();
        let requests = TesseraRuntime::with_mut(|runtime| {
            runtime.component_tree.flush_focused_input(
                &mut self.cursor_state,
                modifiers,
                keyboard,
                ime,
            )
        });
        flush_pending_focus_callbacks();
        if let Some(request) = requests.request_window_drag {
            let window = self
                .app
                .as_ref()
                .expect("window requests follow app creation")
                .window_arc();
            self.apply_window_drag(&window, request);
        }
        let request = requests.ime_request.filter(|request| {
            TesseraRuntime::with(|runtime| {
                request.client_id == runtime.component_tree.focus_owner().active_handle_id()
            })
        });
        // The next native event must see edits made by this event, before the next frame.
        self.ime_bridge_state.sync_window(
            self.app
                .as_ref()
                .expect("IME requests follow app creation")
                .window(),
            request,
        );
    }

    fn handle_redraw_requested(&mut self, event_loop: &ActiveEventLoop) {
        let _session = self.session.enter();
        self.redraw_request_pending.store(false, Ordering::Release);
        let mut app = match self.app.take() {
            Some(app) => app,
            None => return,
        };

        #[cfg(feature = "profiling")]
        let frame_timer = Instant::now();
        crate::window_insets::platform::refresh(app.window(), event_loop);
        app.resize_if_needed();
        let accessibility_enabled =
            self.accessibility_adapter.is_some() && self.accessibility_active;
        let frame_idx = self.frame_index;
        #[cfg(feature = "profiling")]
        let redraw_reasons = self.take_pending_redraw_reasons();
        let window_label = &self.config.window_title;
        let RenderFrameOutcome {
            accessibility_update,
            request_window_drag,
            runtime_pending_work,
            #[cfg(feature = "profiling")]
            mut frame_meta,
            #[cfg(feature = "debug-dirty-overlay")]
            overlay_clear_pending,
        } = {
            let mut args = RenderFrameArgs {
                cursor_state: &mut self.cursor_state,
                keyboard_state: &mut self.keyboard_state,
                ime_state: &mut self.ime_state,
                ime_bridge_state: &mut self.ime_bridge_state,
                app: &mut app,
            };
            Self::execute_render_frame(RenderFrameContext {
                entry_point: &self.entry_point,
                args: &mut args,
                accessibility_enabled,
                decorations: self.config.window.decorations,
                window_label,
                frame_idx,
                #[cfg(feature = "profiling")]
                redraw_reasons,
            })
        };

        if let Some(request) = request_window_drag {
            self.apply_window_drag(app.window(), request);
        }
        #[cfg(target_os = "macos")]
        self.macos_titlebar.clear();
        if let Some(action) = self.take_pending_desktop_window_action() {
            self.apply_desktop_window_action(app.window(), action);
        }

        self.frame_index = self.frame_index.wrapping_add(1);

        if let Some(tree_update) = accessibility_update {
            self.push_accessibility_update(tree_update);
        }

        #[cfg(feature = "profiling")]
        {
            frame_meta.frame_total_ns = Some(frame_timer.elapsed().as_nanos());
            submit_frame_meta(frame_meta);
            profiler_end_frame();
        }

        self.app = Some(app);
        #[cfg(target_os = "android")]
        if let Some(back) = self.android_back.as_mut() {
            back.set_enabled(crate::system_back::enabled());
        }

        if runtime_pending_work.requires_redraw() {
            #[cfg(feature = "profiling")]
            self.request_redraw_with_reasons(
                WakeSource::Runtime,
                runtime_pending_work.redraw_reasons(),
            );
            #[cfg(not(feature = "profiling"))]
            self.request_redraw_now();
        }
        #[cfg(feature = "debug-dirty-overlay")]
        if overlay_clear_pending {
            self.request_redraw_now();
        }
    }
}

#[cfg(target_family = "wasm")]
fn web_canvas_physical_size(window: &Window) -> Option<winit::dpi::PhysicalSize<u32>> {
    use winit::platform::web::WindowExtWebSys;

    let canvas = window.canvas()?;
    let style = web_sys::window()?.get_computed_style(&canvas).ok()??;
    let pixels = |property: &str| -> Option<f64> {
        style
            .get_property_value(property)
            .ok()?
            .strip_suffix("px")?
            .parse()
            .ok()
    };
    let mut width = pixels("width")?;
    let mut height = pixels("height")?;
    if style.get_property_value("box-sizing").ok()? == "border-box" {
        width -= pixels("padding-left")?
            + pixels("padding-right")?
            + pixels("border-left-width")?
            + pixels("border-right-width")?;
        height -= pixels("padding-top")?
            + pixels("padding-bottom")?
            + pixels("border-top-width")?
            + pixels("border-bottom-width")?;
    }
    // Device-pixel content boxes can disagree with emulated DPI. Use the same
    // density as pointer coordinates so the canvas and input stay aligned.
    canvas_physical_size([width, height], window.scale_factor())
}

#[cfg(any(target_family = "wasm", test))]
fn canvas_physical_size(logical: [f64; 2], density: f64) -> Option<winit::dpi::PhysicalSize<u32>> {
    if !density.is_finite() || density <= 0.0 {
        return None;
    }
    let physical = logical.map(|size| size * density);
    if physical
        .iter()
        .any(|size| !size.is_finite() || *size <= 0.0 || *size > u32::MAX as f64)
    {
        return None;
    }
    Some(winit::dpi::PhysicalSize::new(
        physical[0].round().max(1.0) as u32,
        physical[1].round().max(1.0) as u32,
    ))
}

fn window_drawable_size(window: &Window) -> winit::dpi::PhysicalSize<u32> {
    // UIKit draws across the entire window, while Winit's inner size excludes the safe area.
    #[cfg(target_os = "ios")]
    {
        window.outer_size()
    }
    #[cfg(not(target_os = "ios"))]
    {
        window.inner_size()
    }
}

// Resize events may arrive before the asynchronous web render core is ready.
fn resolve_startup_window_size(
    pending: Option<winit::dpi::PhysicalSize<u32>>,
    live: Option<winit::dpi::PhysicalSize<u32>>,
) -> Option<winit::dpi::PhysicalSize<u32>> {
    fn usable(size: &winit::dpi::PhysicalSize<u32>) -> bool {
        size.width > 0 && size.height > 0
    }

    match pending {
        Some(size) if usable(&size) => Some(size),
        _ => live.filter(usable),
    }
}

/// Handle application lifecycle and window events.
impl<F, Output> ApplicationHandler<RendererUserEvent> for Renderer<F>
where
    F: Fn() -> Output,
{
    #[tracing::instrument(level = "debug", skip(self, event_loop))]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let _session = self.session.enter();
        event_loop.set_control_flow(ControlFlow::Wait);
        let app_titlebar = self.config.window.app_titlebar();
        crate::local_runtime_resource::<Cell<AppTitleBar>>().set(app_titlebar);
        #[cfg(feature = "profiling")]
        submit_runtime_meta(RuntimeMeta {
            kind: RuntimeEventKind::Resumed,
        });
        #[cfg(target_family = "wasm")]
        self.finish_web_initialization(event_loop, self.web_init_epoch);
        // Just return if the app is already created
        if self.app.is_some() {
            self.install_runtime_redraw_waker();
            #[cfg(feature = "profiling")]
            self.request_redraw_with_reasons(WakeSource::Lifecycle, vec![RedrawReason::Startup]);
            #[cfg(not(feature = "profiling"))]
            self.request_redraw_now();
            return;
        }
        #[cfg(target_family = "wasm")]
        if self.web_init_in_progress {
            return;
        }

        // Create a new window (initially hidden for AccessKit initialization)
        let window_attributes = Window::default_attributes()
            .with_title(&self.config.window_title)
            .with_window_icon(self.config.window.icon.clone())
            .with_decorations(self.config.window.decorations)
            .with_resizable(self.config.window.resizable)
            .with_transparent(self.config.window.transparent)
            .with_visible(false); // Hide initially for AccessKit
        #[cfg(target_os = "windows")]
        let window_attributes = {
            use winit::platform::windows::WindowAttributesExtWindows;
            if let Some(identifier) = &self.config.window.application_id {
                let identifier = windows::core::HSTRING::from(identifier);
                if let Err(error) = unsafe {
                    windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(&identifier)
                } {
                    tracing::warn!(%error, "Failed to set application identity");
                }
            }
            window_attributes.with_taskbar_icon(self.config.window.icon.clone())
        };
        #[cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        let window_attributes = if let Some(identifier) = &self.config.window.application_id {
            let attributes = winit::platform::wayland::WindowAttributesExtWayland::with_name(
                window_attributes,
                identifier,
                identifier,
            );
            winit::platform::x11::WindowAttributesExtX11::with_name(
                attributes, identifier, identifier,
            )
        } else {
            window_attributes
        };
        #[cfg(target_os = "macos")]
        let window_attributes = {
            use winit::platform::macos::WindowAttributesExtMacOS;

            if app_titlebar == AppTitleBar::Native {
                window_attributes
                    .with_titlebar_transparent(true)
                    .with_title_hidden(true)
                    .with_fullsize_content_view(true)
            } else {
                window_attributes
            }
        };
        #[cfg(target_family = "wasm")]
        let window_attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;

            if let Some(canvas) = Self::resolve_web_canvas(&self.config) {
                window_attributes
                    .with_canvas(Some(canvas))
                    .with_append(false)
            } else {
                window_attributes.with_append(true)
            }
        };
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                error!("Failed to create window: {err}");
                return;
            }
        };

        // Initialize AccessKit adapter BEFORE showing the window
        if let Some(proxy) = self.event_loop_proxy.clone() {
            self.accessibility_active = false;
            self.accessibility_adapter = Some(AccessKitAdapter::with_event_loop_proxy(
                event_loop, &window, proxy,
            ));
        }

        // Now show the window after AccessKit is initialized
        window.set_visible(true);
        self.update_native_window_shape(&window);

        #[cfg(target_family = "wasm")]
        {
            self.begin_web_initialization(window);
            return;
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let mut render_core = pollster::block_on(RenderCore::new(
                window.clone(),
                self.config.sample_count,
                self.config.window.transparent,
                self.config.max_non_sampler_bindings,
            ));

            // Register pipelines
            let mut context = PipelineContext::new(&mut render_core);
            for module in &self.modules {
                module.register_pipelines(&mut context);
            }

            self.app = Some(render_core);
            self.apply_pending_window_size();
            self.install_runtime_redraw_waker();
            #[cfg(feature = "profiling")]
            self.request_redraw_with_reasons(WakeSource::Lifecycle, vec![RedrawReason::Startup]);
            #[cfg(not(feature = "profiling"))]
            self.request_redraw_now();

            if let Some(context) = self.plugin_context(event_loop) {
                let _session = self.session.enter();
                crate::plugin::plugins_resumed(context);
            }
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _session = self.session.enter();
        debug!("Suspending renderer; tearing down WGPU resources.");
        #[cfg(target_os = "android")]
        if let Some(back) = self.android_back.as_mut() {
            back.set_enabled(false);
            crate::system_back::dispatch(crate::system_back::SystemBackEvent::Cancelled);
        }
        #[cfg(feature = "profiling")]
        submit_runtime_meta(RuntimeMeta {
            kind: RuntimeEventKind::Suspended,
        });

        if let Some(context) = self.plugin_context(event_loop) {
            crate::plugin::plugins_suspended(&context);
        }

        self.app = None;

        // Clean up AccessKit adapter
        self.accessibility_adapter = None;
        self.accessibility_active = false;

        self.cursor_state = CursorState::default();
        self.keyboard_state = KeyboardState::default();
        self.ime_state = ImeState::default();
        self.ime_bridge_state.reset();
        self.resize_in_progress = false;
        self.redraw_request_pending.store(false, Ordering::Release);
        #[cfg(target_family = "wasm")]
        {
            self.web_init_epoch = self.web_init_epoch.wrapping_add(1);
            self.web_init_in_progress = false;
            self.pending_web_inits.borrow_mut().clear();
        }

        TesseraRuntime::with_mut(|runtime| {
            runtime.component_tree.reset();
            runtime.cursor_icon_request = None;
            runtime.window_minimized = false;
            runtime.window_size = [0, 0];
        });
        clear_layout_snapshots();
        reset_layout_dirty_tracking();
        release_retired_replay_snapshots();
        reset_focus_read_dependencies();
        reset_render_slot_read_dependencies();
        reset_state_read_dependencies();
        reset_context_read_dependencies();
        reset_build_invalidations();
        reset_frame_clock();
        clear_redraw_waker();
        clear_persistent_focus_handles();
        crate::runtime::reset_component_slots();
        #[cfg(feature = "profiling")]
        self.pending_redraw_reasons.clear();
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _session = self.session.enter();
        if cfg!(target_os = "ios") && self.redraw_request_pending.load(Ordering::Acquire) {
            // UIKit can coalesce view redraws while the Metal sublayer presents.
            self.handle_redraw_requested(event_loop);
        } else if let Some(app) = self.app.as_ref() {
            crate::window_insets::platform::refresh(app.window(), event_loop);
        }
        #[cfg(target_os = "ios")]
        {
            let flow = if self.redraw_request_pending.load(Ordering::Acquire) {
                ControlFlow::Poll
            } else {
                ControlFlow::Wait
            };
            event_loop.set_control_flow(flow);
        }
    }

    #[tracing::instrument(level = "debug", skip(self, event_loop))]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let _session = self.session.enter();
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
        let mut request_redraw = false;
        #[cfg(feature = "profiling")]
        let mut redraw_reasons = Vec::new();
        match event {
            WindowEvent::CloseRequested => {
                self.handle_close_requested(event_loop);
            }
            WindowEvent::Resized(size) => {
                self.handle_resized(size);
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::WindowResized);
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.handle_cursor_moved(position);
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::CursorMoved);
            }
            WindowEvent::CursorLeft { device_id: _ } => {
                self.handle_cursor_left();
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::CursorLeft);
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                self.handle_mouse_input(state, button);
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::MouseInput);
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                self.handle_mouse_wheel(delta);
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::MouseWheel);
            }
            WindowEvent::Touch(touch_event) => {
                self.handle_touch(touch_event);
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::TouchInput);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                set_scale_factor(scale_factor);
                #[cfg(target_family = "wasm")]
                if let Some(size) = self
                    .app
                    .as_ref()
                    .map(|app| window_drawable_size(app.window()))
                {
                    self.handle_resized(size);
                }
                if let Some(app) = self.app.as_ref() {
                    self.update_native_window_shape(app.window());
                }
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::ScaleFactorChanged);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event);
                self.flush_focused_input();
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::KeyboardInput);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                debug!("Modifiers changed: {modifiers:?}");
                self.keyboard_state.update_modifiers(modifiers.state());
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::ModifiersChanged);
            }
            WindowEvent::Ime(ime_event) => {
                debug!("IME event: {ime_event:?}");
                if let Some(client) = self.ime_bridge_state.receive_event(&ime_event) {
                    self.ime_state.push_event_for(Some(client), ime_event);
                    self.flush_focused_input();
                }
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::ImeEvent);
            }
            WindowEvent::Focused(focused) => {
                TesseraRuntime::with_mut(|runtime| {
                    runtime
                        .component_tree
                        .focus_owner_mut()
                        .set_owner_focused(focused);
                });
                flush_pending_focus_callbacks();
                if self.resize_in_progress {
                    self.resize_in_progress = false;
                    self.cursor_state.clear();
                }
                request_redraw = true;
                #[cfg(feature = "profiling")]
                redraw_reasons.push(RedrawReason::FocusChanged);
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw_requested(event_loop);
            }
            _ => (),
        }

        if request_redraw {
            #[cfg(feature = "profiling")]
            self.request_redraw_with_reasons(WakeSource::WindowEvent, redraw_reasons);
            #[cfg(not(feature = "profiling"))]
            self.request_redraw_now();
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: RendererUserEvent) {
        let _session = self.session.enter();
        use accesskit_winit::WindowEvent as AccessKitWindowEvent;
        #[cfg(not(target_family = "wasm"))]
        let _ = event_loop;

        match event {
            RendererUserEvent::RuntimeRedrawWake => {
                // A frame can consume invalidations before their queued wake arrives.
                let pending = RuntimePendingWork::capture(false);
                if !pending.requires_redraw() {
                    return;
                }
                #[cfg(feature = "profiling")]
                self.request_redraw_with_reasons(WakeSource::Runtime, pending.redraw_reasons());
                #[cfg(not(feature = "profiling"))]
                self.request_redraw_now();
            }
            #[cfg(target_family = "wasm")]
            RendererUserEvent::WebInitReady(epoch) => {
                self.finish_web_initialization(event_loop, epoch);
            }
            RendererUserEvent::AccessKit(event) => {
                if self.accessibility_adapter.is_none() {
                    return;
                }
                match event.window_event {
                    AccessKitWindowEvent::InitialTreeRequested => {
                        self.accessibility_active = true;
                        self.send_accessibility_update();
                    }
                    AccessKitWindowEvent::ActionRequested(action_request) => {
                        // Dispatch action to the appropriate component handler
                        let handled = TesseraRuntime::with_mut(|runtime| {
                            let (tree, metadatas, focus_owner) =
                                runtime.component_tree.accessibility_dispatch_context();
                            crate::accessibility::dispatch_action(
                                tree,
                                metadatas,
                                focus_owner,
                                action_request,
                            )
                        });
                        flush_pending_focus_callbacks();

                        if !handled {
                            debug!("Action was not handled by any component");
                        }
                    }
                    AccessKitWindowEvent::AccessibilityDeactivated => {
                        self.accessibility_active = false;
                        TesseraRuntime::with_mut(|runtime| {
                            runtime.component_tree.stop_accessibility_updates();
                        });
                        debug!("AccessKit deactivated");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use winit::dpi::PhysicalSize;

    use crate::{
        FocusRequester, Modifier, PxRect, State,
        component_tree::dispatch_pending_focus_reveal_request,
        focus::flush_pending_focus_callbacks,
        layout::{LayoutResult, MeasureScope, Stack, layout},
        modifier::{FocusModifierExt as _, LayoutPolicy},
        remember,
        runtime::{TesseraRuntime, prune_persistent_focus_handles},
        testing::{layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use super::{
        RendererImeBridgeState, RendererImeBridgeUpdate, RuntimePendingWork, canvas_physical_size,
        resolve_startup_window_size,
    };

    #[derive(Clone, Default, PartialEq)]
    struct FrameEndProbe {
        focus: crate::testing::TestProbe<Option<(State<bool>, FocusRequester, FocusRequester)>>,
        events: crate::testing::TestProbe<usize>,
        reveals: crate::testing::TestProbe<Vec<PxRect>>,
    }

    #[derive(Clone, PartialEq)]
    struct FocusLeafLayout;

    impl LayoutPolicy for FocusLeafLayout {
        fn measure(
            &self,
            _scope: &mut MeasureScope<'_>,
        ) -> Result<LayoutResult, crate::MeasurementError> {
            Ok(LayoutResult::new(crate::ComputedData {
                width: crate::Px(64),
                height: crate::Px(24),
            }))
        }
    }

    #[crate::tessera]
    fn frame_end_removable_target(requester: FocusRequester) {
        crate::layout::layout(FocusLeafLayout)
            .modifier(Modifier::new().focusable().focus_requester(requester));
    }

    #[crate::tessera]
    fn frame_end_focus_fixture(probe: FrameEndProbe) {
        let visible = remember(|| true);
        let removed = remember(FocusRequester::new).get();
        let fallback = remember(FocusRequester::new).get();
        probe.focus.set(Some((visible, removed, fallback)));
        let reveals = probe.reveals.clone();
        let events = probe.events.clone();
        layout(Stack)
            .modifier(
                Modifier::new()
                    .focus_restorer(Some(fallback))
                    .focus_reveal_handler(move |request: crate::focus::FocusRevealRequest| {
                        reveals.with_mut(|rects| rects.push(request.target_rect));
                        false
                    }),
            )
            .child(move || {
                let events = events.clone();
                if visible.get() {
                    frame_end_removable_target(removed);
                }
                crate::layout::layout(FocusLeafLayout).modifier(
                    Modifier::new()
                        .focusable()
                        .focus_requester(fallback)
                        .on_focus_changed(move |state: crate::focus::FocusState| {
                            if state.is_focused() {
                                events.with_mut(|events| *events += 1);
                            }
                        }),
                );
            });
    }

    #[derive(Clone, Default, PartialEq)]
    struct VisibilityProbe {
        focus: crate::testing::TestProbe<Option<(FocusRequester, FocusRequester)>>,
        reads: crate::testing::TestProbe<Vec<bool>>,
    }

    #[crate::tessera]
    fn focus_visibility_observer(
        requester: FocusRequester,
        reads: crate::testing::TestProbe<Vec<bool>>,
    ) {
        reads.with_mut(|reads| reads.push(requester.is_focus_visible()));
        crate::layout::layout(FocusLeafLayout);
    }

    #[crate::tessera]
    fn focus_visibility_fixture(probe: VisibilityProbe) {
        let first = remember(FocusRequester::new).get();
        let second = remember(FocusRequester::new).get();
        probe.focus.set(Some((first, second)));
        layout(Stack).child(move || {
            frame_end_removable_target(first);
            frame_end_removable_target(second);
            focus_visibility_observer(first, probe.reads.clone());
        });
    }

    #[test]
    fn focus_visibility_replays_subscribers_without_changing_same_target_ownership() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        use std::time::Duration;

        use winit::event::ElementState;

        use crate::{
            CursorEventContent, GestureState, MOUSE_POINTER_ID, PointerChange, PxPosition,
            cursor::PressKeyEventType, testing::pump_layout_test_session_with_input, time::Instant,
        };

        let probe = VisibilityProbe::default();
        let mut session =
            start_layout_test_session(layout_test(|| focus_visibility_fixture(probe.clone())));
        pump_layout_test_session(&mut session, 0);
        let (first, second) = probe.focus.get().expect("test fixture entry exists");
        first.request_focus();
        first.capture_focus();
        pump_layout_test_session(&mut session, 1);
        assert!(!first.is_focus_visible());
        let origin = Instant::now();
        let keyboard = origin + Duration::from_millis(10);
        let press = origin + Duration::from_millis(20);
        let newer_keyboard = origin + Duration::from_millis(30);
        let key = |timestamp| {
            TesseraRuntime::with_mut(|runtime| {
                runtime
                    .component_tree
                    .focus_owner_mut()
                    .record_keyboard_input(ElementState::Pressed, timestamp);
            });
        };
        key(keyboard);
        pump_layout_test_session(&mut session, 2);
        assert!(first.is_captured());
        assert_eq!(probe.reads.with(|reads| reads.last().copied()), Some(true));
        let pointer = |timestamp| PointerChange {
            timestamp,
            pointer_id: MOUSE_POINTER_ID,
            content: CursorEventContent::Pressed(PressKeyEventType::Left),
            gesture_state: GestureState::TapCandidate,
            sampled_position: Default::default(),
            consumed: false,
            tap_claimed: false,
            scroll_discrete: None,
        };
        pump_layout_test_session_with_input(
            &mut session,
            3,
            vec![pointer(press)],
            Some(PxPosition::ZERO),
        );
        pump_layout_test_session(&mut session, 4);
        assert!(first.is_captured());
        assert_eq!(probe.reads.with(|reads| reads.last().copied()), Some(false));
        key(keyboard);
        first.request_focus();
        pump_layout_test_session(&mut session, 5);
        assert!(!first.is_focus_visible());
        key(newer_keyboard);
        pump_layout_test_session_with_input(
            &mut session,
            6,
            vec![pointer(press)],
            Some(PxPosition::ZERO),
        );
        pump_layout_test_session(&mut session, 7);
        assert!(first.is_captured());
        assert_eq!(probe.reads.with(|reads| reads.last().copied()), Some(true));
        let mut hover = pointer(origin + Duration::from_millis(40));
        hover.content = CursorEventContent::Moved(PxPosition::ZERO);
        let mut wheel = pointer(origin + Duration::from_millis(50));
        wheel.content = CursorEventContent::Scroll(crate::cursor::ScrollEventContent {
            delta_x: 0.0,
            delta_y: -24.0,
            unit: crate::cursor::ScrollDeltaUnit::Pixel,
            source: crate::cursor::ScrollEventSource::Wheel,
        });
        pump_layout_test_session_with_input(
            &mut session,
            8,
            vec![hover, wheel],
            Some(PxPosition::ZERO),
        );
        assert!(first.is_captured());
        assert!(first.is_focus_visible());
        first.free_focus();
        second.request_focus();
        assert!(second.is_focus_visible());
        assert!(!first.is_focus_visible());
        pump_layout_test_session_with_input(
            &mut session,
            9,
            vec![pointer(origin + Duration::from_millis(60))],
            Some(PxPosition::ZERO),
        );
        first.request_focus();
        assert!(first.is_focused());
        assert!(!first.is_focus_visible());
    }

    fn dispatch_frame_end_reveal() {
        TesseraRuntime::with_mut(|runtime| {
            let (tree, metadatas, focus_owner) =
                runtime.component_tree.accessibility_dispatch_context();
            dispatch_pending_focus_reveal_request(tree, metadatas, focus_owner, false);
        });
    }

    #[test]
    fn pruned_focus_restoration_keeps_a_frame_scheduled_until_its_reveal_is_dispatched() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let probe = FrameEndProbe::default();
        let mut session =
            start_layout_test_session(layout_test(|| frame_end_focus_fixture(probe.clone())));
        pump_layout_test_session(&mut session, 0);
        let (visible, removed, fallback) = probe.focus.get().expect("test fixture entry exists");
        removed.request_focus();
        assert!(removed.is_focused());
        dispatch_frame_end_reveal();
        pump_layout_test_session(&mut session, 1);
        assert!(!RuntimePendingWork::capture(false).requires_redraw());

        visible.set(false);
        pump_layout_test_session(&mut session, 2);
        assert!(!fallback.is_focused());
        prune_persistent_focus_handles();
        assert!(!fallback.is_focused());
        assert!(RuntimePendingWork::capture(false).requires_redraw());
        pump_layout_test_session(&mut session, 3);
        prune_persistent_focus_handles();
        flush_pending_focus_callbacks();
        assert!(fallback.is_focused());
        assert_eq!(probe.events.get(), 1);

        pump_layout_test_session(&mut session, 4);
        assert!(!crate::runtime::has_pending_build_invalidations());
        assert!(!crate::runtime::has_pending_layout_invalidations());
        assert!(!crate::runtime::has_pending_frame_waiters());
        assert!(!crate::runtime::has_pending_persistent_focus_prune());
        assert!(!crate::clipboard::has_pending_requests());
        assert!(!TesseraRuntime::with(|runtime| {
            runtime
                .component_tree
                .focus_owner()
                .has_pending_navigation()
        }));
        assert!(RuntimePendingWork::capture(false).requires_redraw());
        assert_eq!(probe.reveals.with(Vec::len), 1);

        dispatch_frame_end_reveal();
        assert_eq!(
            probe.reveals.with(|rects| rects.clone()),
            vec![
                PxRect::new(crate::Px(0), crate::Px(0), crate::Px(64), crate::Px(24)),
                PxRect::new(crate::Px(0), crate::Px(0), crate::Px(64), crate::Px(24)),
            ]
        );
        assert!(fallback.is_focused());
        assert!(!RuntimePendingWork::capture(false).requires_redraw());
    }

    #[derive(Clone, Default, PartialEq)]
    struct ColdProbe {
        child: crate::testing::TestProbe<Option<PxRect>>,
        writes: crate::testing::TestProbe<usize>,
        wakes: crate::testing::TestProbe<usize>,
        counts: crate::testing::TestProbe<(usize, usize)>,
    }

    #[crate::tessera]
    fn cold_geometry_child(probe: crate::testing::TestProbe<Option<PxRect>>) {
        crate::layout::layout(FocusLeafLayout)
            .modifier(Modifier::new().on_globally_positioned(move |rect| probe.set(Some(rect))));
    }

    #[crate::tessera]
    fn cold_geometry_fixture(probe: ColdProbe) {
        let observed = remember(|| None::<PxRect>);
        let output = probe.clone();
        layout(Stack)
            .modifier(Modifier::new().on_globally_positioned(move |rect| {
                if observed.get() != Some(rect) {
                    let before = output.wakes.get();
                    observed.set(Some(rect));
                    output.counts.set((before, output.wakes.get()));
                    output.writes.set(output.writes.get() + 1);
                }
            }))
            .child(move || {
                crate::layout::layout(FocusLeafLayout);
                if observed.get().is_some() {
                    cold_geometry_child(probe.child.clone());
                }
            });
    }

    #[test]
    fn first_geometry_publication_settles_its_conditional_child_before_the_frame_finishes() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        struct WakerGuard;
        impl Drop for WakerGuard {
            fn drop(&mut self) {
                crate::runtime::clear_redraw_waker();
            }
        }

        let probe = ColdProbe::default();
        let mut session =
            start_layout_test_session(layout_test(|| cold_geometry_fixture(probe.clone())));
        let _guard = WakerGuard;
        crate::runtime::install_redraw_waker(std::sync::Arc::new({
            let wakes = probe.wakes.clone();
            move || wakes.with_mut(|wakes| *wakes += 1)
        }));
        pump_layout_test_session(&mut session, 0);
        assert_eq!(probe.writes.get(), 1);
        let (before, after) = probe.counts.get();
        assert!(
            after > before,
            "the observer's state write must wake the runtime"
        );
        assert_eq!(
            probe.child.get(),
            Some(PxRect::new(
                crate::Px(0),
                crate::Px(0),
                crate::Px(64),
                crate::Px(24)
            )),
            "the geometry update must place the newly inserted child in the same frame"
        );
        assert_eq!(probe.writes.get(), 1);
        assert!(!RuntimePendingWork::capture(false).requires_redraw());
        let wakes = probe.wakes.get();
        pump_layout_test_session(&mut session, 16_666_667);
        assert_eq!(probe.writes.get(), 1);
        assert_eq!(
            probe.wakes.get(),
            wakes,
            "an unchanged idle layout must not wake again"
        );
        assert!(!RuntimePendingWork::capture(false).requires_redraw());
    }

    #[test]
    fn web_canvas_size_tracks_css_density_and_fractional_dimensions() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        assert_eq!(
            canvas_physical_size([390.0, 780.0], 1.0000000298023224),
            Some(PhysicalSize::new(390, 780))
        );
        assert_eq!(
            canvas_physical_size([877.333374, 871.333374], 1.5),
            Some(PhysicalSize::new(1316, 1307))
        );
        assert_eq!(
            canvas_physical_size([390.0, 780.0], 2.0),
            Some(PhysicalSize::new(780, 1560))
        );
        assert_eq!(canvas_physical_size([0.0, 780.0], 1.0), None);
        assert_eq!(canvas_physical_size([390.0, 780.0], f64::NAN), None);
    }

    #[test]
    fn startup_window_size_prefers_recorded_size_over_live_size() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let recorded = PhysicalSize::new(800, 600);
        let live = PhysicalSize::new(1024, 768);
        assert_eq!(
            resolve_startup_window_size(Some(recorded), Some(live)),
            Some(recorded)
        );
    }

    #[test]
    fn startup_window_size_falls_back_to_live_size_for_empty_recordings() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let live = PhysicalSize::new(1024, 768);
        assert_eq!(
            resolve_startup_window_size(Some(PhysicalSize::new(0, 0)), Some(live)),
            Some(live)
        );
        assert_eq!(
            resolve_startup_window_size(Some(PhysicalSize::new(0, 600)), Some(live)),
            Some(live)
        );
    }

    #[test]
    fn startup_window_size_uses_live_size_without_recorded_events() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let live = PhysicalSize::new(1024, 768);
        assert_eq!(resolve_startup_window_size(None, Some(live)), Some(live));
    }

    #[test]
    fn startup_window_size_is_none_when_every_reading_is_empty() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        assert_eq!(
            resolve_startup_window_size(Some(PhysicalSize::new(0, 0)), None),
            None
        );
        assert_eq!(
            resolve_startup_window_size(None, Some(PhysicalSize::new(0, 0))),
            None
        );
    }

    use crate::{ImeRequest, Px, PxPosition, px::PxSize};

    fn positioned_request(
        position: PxPosition,
        size: PxSize,
        selection: std::ops::Range<usize>,
        composition: Option<std::ops::Range<usize>>,
    ) -> ImeRequest {
        let mut request = ImeRequest::new(size)
            .with_local_position(position)
            .with_text_state(winit::event::ImeTextState {
                text: "0123456789".into(),
                selection,
                composition,
            });
        request.position = Some(position);
        request
    }

    #[test]
    fn ime_bridge_enables_and_updates_cursor_area_for_first_request() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut bridge = RendererImeBridgeState::default();
        let request = positioned_request(
            PxPosition::new(Px(12), Px(18)),
            PxSize::new(Px(7), Px(13)),
            2..6,
            Some(3..5),
        );

        let update = bridge.update_request(Some(request.clone()));

        assert_eq!(
            update,
            RendererImeBridgeUpdate {
                allowed: Some(true),
                cursor_area: Some((PxPosition::new(Px(12), Px(18)), PxSize::new(Px(7), Px(13)))),
                text_state: Some(request.text_state.clone()),
                options: Some(request.options),
                ..Default::default()
            }
        );
        assert_eq!(bridge.request(), Some(&request));
    }

    #[test]
    fn ime_bridge_reuses_identical_request_without_window_updates() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let request = positioned_request(
            PxPosition::new(Px(12), Px(18)),
            PxSize::new(Px(7), Px(13)),
            2..6,
            Some(3..5),
        );
        let mut bridge = RendererImeBridgeState::default();
        bridge.update_request(Some(request.clone()));

        let update = bridge.update_request(Some(request.clone()));

        assert_eq!(update, RendererImeBridgeUpdate::default());
        assert_eq!(bridge.request(), Some(&request));
        let mut show = request.clone();
        show.show_keyboard = true;
        show.change_cause = winit::event::ImeChangeCause::InputMethod;
        let mut requests = crate::component_tree::WindowRequests {
            ime_request: Some(show),
            ..Default::default()
        };
        requests.settle(crate::component_tree::WindowRequests {
            ime_request: Some(request.clone()),
            ..Default::default()
        });
        let update = bridge.update_request(requests.ime_request);
        assert_eq!(
            update,
            RendererImeBridgeUpdate {
                show_keyboard: true,
                change_cause: winit::event::ImeChangeCause::InputMethod,
                ..Default::default()
            }
        );
        assert_eq!(
            bridge.update_request(Some(request)),
            RendererImeBridgeUpdate::default()
        );
    }

    #[test]
    fn ime_bridge_preserves_selection_and_composition_changes_without_cursor_reapply() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let size = PxSize::new(Px(7), Px(13));
        let mut bridge = RendererImeBridgeState::default();
        bridge.update_request(Some(positioned_request(
            PxPosition::new(Px(12), Px(18)),
            size,
            2..6,
            Some(3..5),
        )));
        let next_request =
            positioned_request(PxPosition::new(Px(12), Px(18)), size, 4..8, Some(5..7));

        let update = bridge.update_request(Some(next_request.clone()));

        assert_eq!(update.allowed, None);
        assert_eq!(update.cursor_area, None);
        assert_eq!(update.text_state, Some(next_request.text_state.clone()));
        assert_eq!(bridge.request(), Some(&next_request));
        let mut moved = next_request.clone();
        moved.position = Some(PxPosition::new(Px(20), Px(30)));
        let update = bridge.update_request(Some(moved));
        assert_eq!(update.allowed, None);
        assert_eq!(update.text_state, None);
        assert_eq!(update.options, None);
        assert_eq!(
            update.cursor_area,
            Some((PxPosition::new(Px(20), Px(30)), size))
        );
        let mut another_client = next_request.clone();
        another_client.client_id = Some(42);
        let update = bridge.update_request(Some(another_client.clone()));
        assert_eq!(update.allowed, Some(true));
        assert_eq!(
            update.cursor_area,
            Some((PxPosition::new(Px(12), Px(18)), size))
        );
        another_client.text_state.text = "native edit".into();
        another_client.text_state.selection = 11..11;
        another_client.text_state.composition = Some(7..11);
        assert_eq!(
            bridge.receive_event(&winit::event::Ime::TextState(
                another_client.text_state.clone()
            )),
            Some(42)
        );
        assert_eq!(
            bridge.update_request(Some(another_client.clone())),
            RendererImeBridgeUpdate::default()
        );
        another_client.text_state.text = "transformed edit".into();
        let update = bridge.update_request(Some(another_client.clone()));
        assert_eq!(update.text_state, Some(another_client.text_state));
        assert_eq!(update.allowed, None);
    }

    #[test]
    fn ime_bridge_disables_and_clears_snapshot_when_request_is_removed() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let request = positioned_request(
            PxPosition::new(Px(12), Px(18)),
            PxSize::new(Px(7), Px(13)),
            2..6,
            Some(3..5),
        );
        let mut bridge = RendererImeBridgeState::default();
        bridge.update_request(Some(request));

        let update = bridge.update_request(None);

        assert_eq!(
            update,
            RendererImeBridgeUpdate {
                allowed: Some(false),
                cursor_area: None,
                text_state: None,
                options: None,
                ..Default::default()
            }
        );
        assert_eq!(bridge.request(), None);
    }
}
