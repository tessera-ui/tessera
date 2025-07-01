pub mod app;
pub mod compute;
pub mod drawer;

use std::{sync::Arc, time::Instant};

use log::{debug, warn};
use parking_lot::Mutex;

pub use app::WgpuApp;
#[cfg(target_os = "android")]
use winit::platform::android::{EventLoopBuilderExtAndroid, activity::AndroidApp};
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

pub use compute::{ComputablePipeline, ComputeCommand, ComputePipelineRegistry};
pub use drawer::{DrawCommand, DrawablePipeline, PipelineRegistry, RenderRequirement};

/// The bind group set index where the scene texture is bound for pipelines that
/// require `RenderRequirement::SamplesBackground`.
///
/// Pipelines that sample the background should expect the background texture at this set index.
pub const SCENE_TEXTURE_BIND_GROUP_SET: u32 = 1;
pub struct Renderer<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> {
    /// WGPU app
    app: Arc<Mutex<Option<WgpuApp>>>,
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
}

impl<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> Renderer<F, R> {
    #[cfg(not(target_os = "android"))]
    /// Create event loop and run application
    pub fn run(entry_point: F, register_pipelines_fn: R) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        let app = Arc::new(Mutex::new(None));
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
        };
        thread_utils::set_thread_name("Tessera Renderer");
        event_loop.run_app(&mut renderer)
    }

    #[cfg(target_os = "android")]
    /// Create event loop and run application on Android
    pub fn run(
        entry_point: F,
        register_pipelines_fn: R,
        android_app: AndroidApp,
    ) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::builder()
            .with_android_app(android_app)
            .build()
            .unwrap();
        let app = Arc::new(Mutex::new(None));
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
        };
        thread_utils::set_thread_name("Tessera Renderer");
        event_loop.run_app(&mut renderer)
    }
}

impl<F: Fn(), R: Fn(&mut WgpuApp) + Clone + 'static> ApplicationHandler for Renderer<F, R> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Just return if the app is already created
        if self.app.as_ref().lock().is_some() {
            return;
        }

        // Create a new window
        let window_attributes = Window::default_attributes()
            .with_title("Tessera")
            .with_transparent(true);
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let register_pipelines_fn = self.register_pipelines_fn.clone();

        let mut wgpu_app = tokio_runtime::get().block_on(WgpuApp::new(window));

        // Register pipelines
        wgpu_app.register_pipelines(register_pipelines_fn);

        self.app.lock().replace(wgpu_app);
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
        let mut app_opt = self.app.lock();
        let app = match app_opt.as_mut() {
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
                // notify the windowing system before rendering
                // this will help winit to properly schedule and make assumptions about its internal state
                app.window.pre_present_notify();
                // resize the surface if needed
                app.resize_if_needed();
                // and tell runtime the new size
                TesseraRuntime::write().window_size = app.size().into();
                // render the surface
                // timer for performance measurement
                let tree_timer = Instant::now();
                // build the component tree
                debug!("Building component tree...");
                (self.entry_point)();
                let build_tree_cost = tree_timer.elapsed();
                debug!("Component tree built in {build_tree_cost:?}");
                // get the component tree from the runtime
                let component_tree = &mut TesseraRuntime::write().component_tree;
                // timer for performance measurement
                let draw_timer = Instant::now();
                // Compute the draw commands then we can clear component tree for next build
                debug!("Computing draw commands...");
                let cursor_position = self.cursor_state.position();
                let cursor_events = self.cursor_state.take_events();
                let keyboard_events = self.keyboard_state.take_events();
                let screen_size: PxSize = app.size().into();
                let (commands, window_requests) = component_tree.compute(
                    screen_size,
                    cursor_position,
                    cursor_events,
                    keyboard_events,
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
                    app.window.set_ime_cursor_area::<PxPosition, PxSize>(
                        ime_request.position.unwrap(),
                        ime_request.size,
                    );
                } else {
                    app.window.set_ime_allowed(false);
                }
                // timer for performance measurement
                let render_timer = Instant::now();
                // Render the commands
                debug!("Rendering draw commands...");
                app.render(commands).unwrap();
                let render_cost = render_timer.elapsed();
                debug!("Rendered in {render_cost:?}");
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
            _ => (),
        }
    }
}
