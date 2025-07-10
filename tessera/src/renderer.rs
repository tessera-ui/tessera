pub mod app;
pub mod compute;
pub mod drawer;

use std::{sync::Arc, time::Instant};

use log::{debug, warn};

pub use app::WgpuApp;
#[cfg(target_os = "android")]
use winit::platform::android::{
    ActiveEventLoopExtAndroid, EventLoopBuilderExtAndroid, activity::AndroidApp,
};
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    ImeState, PxPosition,
    cursor::{CursorEvent, CursorEventContent, CursorState},
    dp::SCALE_FACTOR,
    keyboard_state::KeyboardState,
    px::PxSize,
    runtime::TesseraRuntime,
    thread_utils, tokio_runtime,
};

pub use compute::{ComputePipelineRegistry, SyncComputablePipeline};
pub use drawer::{DrawCommand, DrawablePipeline, PipelineRegistry, RenderRequirement};

/// Configuration for the Tessera runtime and renderer.
#[derive(Clone)]
pub struct TesseraConfig {
    /// The number of samples to use for MSAA.
    ///
    /// Common values are 1, 2, 4, 8.
    /// A value of 1 effectively disables MSAA.
    /// Note: The GPU must support the chosen sample count.
    pub sample_count: u32,
}

impl Default for TesseraConfig {
    /// Creates a default configuration with 4x MSAA enabled.
    fn default() -> Self {
        Self { sample_count: 4 }
    }
}

/// The bind group set index where the scene texture is bound for pipelines that
/// require `RenderRequirement::SamplesBackground`.
///
/// Pipelines that sample the background should expect the background texture at this set index.
pub const SCENE_TEXTURE_BIND_GROUP_SET: u32 = 1;
pub struct Renderer<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> {
    /// WGPU app
    app: Option<WgpuApp>,
    /// Entry UI Function
    entry_point: F,
    /// The state of the cursor
    cursor_state: CursorState,
    /// The state of the keyboard
    keyboard_state: KeyboardState,
    /// The state of the IME
    ime_state: ImeState,
    /// Register pipelines function
    register_pipelines_fn: R,
    /// Tessera configuration
    config: TesseraConfig,
    #[cfg(target_os = "android")]
    /// Android ime opened state
    android_ime_opened: bool,
}

impl<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> Renderer<F, R> {
    #[cfg(not(target_os = "android"))]
    pub fn run(entry_point: F, register_pipelines_fn: R) -> Result<(), EventLoopError> {
        Self::run_with_config(entry_point, register_pipelines_fn, Default::default())
    }

    #[cfg(not(target_os = "android"))]
    /// Create event loop and run application
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
        let mut renderer = Self {
            app,
            entry_point,
            cursor_state,
            keyboard_state,
            register_pipelines_fn,
            ime_state,
            config,
        };
        thread_utils::set_thread_name("Tessera Renderer");
        event_loop.run_app(&mut renderer)
    }

    #[cfg(target_os = "android")]
    #[cfg(target_os = "android")]
    /// Create event loop and run application on Android
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
    /// Create event loop and run application on Android
    pub fn run_with_config(
        entry_point: F,
        register_pipelines_fn: R,
        android_app: AndroidApp,
        config: TesseraConfig,
    ) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::builder()
            .with_android_app(android_app)
            .build()
            .unwrap();
        let app = None;
        let cursor_state = CursorState::default();
        let keyboard_state = KeyboardState::default();
        let ime_state = ImeState::default();
        let mut renderer = Self {
            app,
            entry_point,
            cursor_state,
            keyboard_state,
            register_pipelines_fn,
            ime_state,
            android_ime_opened: false,
            config,
        };
        thread_utils::set_thread_name("Tessera Renderer");
        event_loop.run_app(&mut renderer)
    }
}

impl<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> Renderer<F, R> {
    /// Render a single frame - either to surface or offscreen
    fn execute_render_frame(
        entry_point: &F,
        cursor_state: &mut CursorState,
        keyboard_state: &mut KeyboardState,
        ime_state: &mut ImeState,
        #[cfg(target_os = "android")] android_ime_opened: &mut bool,
        app: &mut WgpuApp,
        #[cfg(target_os = "android")] event_loop: &ActiveEventLoop,
    ) {
        // notify the windowing system before rendering
        // this will help winit to properly schedule and make assumptions about its internal state
        app.window.pre_present_notify();
        // and tell runtime the new size
        TesseraRuntime::write().window_size = app.size().into();
        // render the surface
        // timer for performance measurement
        let tree_timer = Instant::now();
        // build the component tree
        debug!("Building component tree...");
        entry_point();
        let build_tree_cost = tree_timer.elapsed();
        debug!("Component tree built in {build_tree_cost:?}");
        // get the component tree from the runtime
        let component_tree = &mut TesseraRuntime::write().component_tree;
        // timer for performance measurement
        let draw_timer = Instant::now();
        // Compute the draw commands then we can clear component tree for next build
        debug!("Computing draw commands...");
        let cursor_position = cursor_state.position();
        let cursor_events = cursor_state.take_events();
        let keyboard_events = keyboard_state.take_events();
        let ime_events = ime_state.take_events();
        let screen_size: PxSize = app.size().into();
        let (commands, window_requests) = component_tree.compute(
            screen_size,
            cursor_position,
            cursor_events,
            keyboard_events,
            ime_events,
        );
        let draw_cost = draw_timer.elapsed();
        debug!("Draw commands computed in {draw_cost:?}");
        component_tree.clear();
        // Handle the window requests
        // After compute, check for cursor change requests
        app.window
            .set_cursor(winit::window::Cursor::Icon(window_requests.cursor_icon));
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
        // Render the commands
        debug!("Rendering draw commands...");
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

impl<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> ApplicationHandler for Renderer<F, R> {
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

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        todo!("Handle suspend event");
    }

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
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    todo!("Handle minimize");
                } else {
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
                );
            }
            _ => (),
        }
    }
}

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
