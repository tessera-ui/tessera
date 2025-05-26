mod app;
mod drawer;

use std::sync::Arc;

use parking_lot::Mutex;

use app::WgpuApp;
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    ComponentNode, Constraint, DEFAULT_LAYOUT_DESC, runtime::TesseraRuntime, tokio_runtime,
};

pub use drawer::{DrawCommand, ShapeVertex, TextConstraint, TextData, write_font_system};

pub struct Renderer<F: Fn()> {
    /// WGPU app
    app: Arc<Mutex<Option<WgpuApp>>>,
    /// Entry UI Function
    entry_point: F,
}

impl<F: Fn()> Renderer<F> {
    /// Create event loop and run application
    pub fn run(entry_point: F) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        let app = Arc::new(Mutex::new(None));
        let mut renderer = Self { app, entry_point };
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
        let window_attributes = Window::default_attributes().with_title("Tessera");
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
                // build the component tree
                screen(&self.entry_point);
                // get the component tree from the runtime
                let component_tree = &mut TesseraRuntime::write().component_tree;
                // Compute the draw commands then we can clear component tree for next build
                let commands = component_tree.compute();
                component_tree.clear();
                // Render the commands
                app.render(commands).unwrap();
                // Currently we render every frame
                app.window.request_redraw();
            }
            _ => (),
        }
    }
}

/// root component
fn screen<F: Fn()>(entry_point: &F) {
    {
        let window_size = TesseraRuntime::write().window_size;
        TesseraRuntime::write()
            .component_tree
            .add_node(ComponentNode {
                layout_desc: Box::new(DEFAULT_LAYOUT_DESC),
                constraint: Constraint {
                    min_width: Some(window_size[0]),
                    min_height: Some(window_size[1]),
                    max_width: Some(window_size[0]),
                    max_height: Some(window_size[1]),
                },
                drawable: None,
            });
    }

    entry_point();

    {
        TesseraRuntime::write().component_tree.pop_node();
    }
}
