mod app;
mod drawer;

use std::{sync::Arc, time::Instant};

use log::{debug, warn};
use parking_lot::Mutex;

use app::WgpuApp;
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
    cursor::{CursorEvent, CursorEventContent, CursorState},
    runtime::TesseraRuntime,
    tokio_runtime,
};

pub use drawer::{
    DrawCommand, ShapeUniforms, ShapeVertex, TextConstraint, TextData, read_font_system,
    write_font_system,
};

pub struct Renderer<F: Fn()> {
    /// WGPU app
    app: Arc<Mutex<Option<WgpuApp>>>,
    /// Entry UI Function
    entry_point: F,
    /// The state of the cursor
    cursor_state: CursorState,
}

impl<F: Fn()> Renderer<F> {
    #[cfg(not(target_os = "android"))]
    /// Create event loop and run application
    pub fn run(entry_point: F) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        let app = Arc::new(Mutex::new(None));
        let cursor_state = CursorState::default();
        let mut renderer = Self {
            app,
            entry_point,
            cursor_state,
        };
        event_loop.run_app(&mut renderer)
    }

    #[cfg(target_os = "android")]
    /// Create event loop and run application on Android
    pub fn run(entry_point: F, android_app: AndroidApp) -> Result<(), EventLoopError> {
        use log::info;

        let event_loop = EventLoop::builder()
            .with_android_app(android_app)
            .build()
            .unwrap();
        let app = Arc::new(Mutex::new(None));
        let cursor_state = CursorState::default();
        let mut renderer = Self {
            app,
            entry_point,
            cursor_state,
        };
        info!("Starting Tessera Renderer on Android...");
        event_loop.run_app(&mut renderer)
    }
}

impl<F: Fn()> ApplicationHandler for Renderer<F> {
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

        let wgpu_app = tokio_runtime::get().block_on(WgpuApp::new(window));
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
        let mut app = self.app.lock();
        let app = app.as_mut().unwrap();

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
                let event = CursorEvent {
                    timestamp: Instant::now(),
                    content: CursorEventContent::from_position([
                        position.x as u32,
                        position.y as u32,
                    ]),
                };
                self.cursor_state.push_event(event);
                debug!("Cursor moved to: {}, {}", position.x, position.y);
            }
            WindowEvent::CursorLeft { device_id: _ } => {
                // Clear cursor position when it leaves the window
                self.cursor_state.out_of_window();
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
            WindowEvent::KeyboardInput { .. } => {
                // todo!("Handle keyboard input");
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
                let cursor_events = self.cursor_state.pop_events().unwrap_or_default();
                let commands = component_tree.compute(app.size().into(), cursor_events.into());
                let draw_cost = draw_timer.elapsed();
                debug!("Draw commands computed in {draw_cost:?}");
                component_tree.clear();
                // timer for performance measurement
                let render_timer = Instant::now();
                // Render the commands
                debug!("Rendering draw commands...");
                app.render(commands).unwrap();
                let render_cost = render_timer.elapsed();
                debug!("Rendered in {render_cost:?}");
                // print frame statistics
                let fps = 1.0 / (build_tree_cost + draw_cost + render_cost).as_secs_f32();
                if fps < 30.0 {
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
