mod app;

use std::sync::Arc;

use log::error;
use parking_lot::Mutex;

use app::WgpuApp;
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::tokio_runtime;

#[derive(Default)]
pub(crate) struct Renderer {
    /// WGPU app
    app: Arc<Mutex<Option<WgpuApp>>>,
    /// Missed resize event
    missed_resize: Arc<Mutex<Option<winit::dpi::PhysicalSize<u32>>>>,
}

impl Renderer {
    /// Create event loop and run application
    pub(crate) fn run() -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new().unwrap();
        let mut renderer = Renderer::default();
        event_loop.run_app(&mut renderer)
    }
}

impl ApplicationHandler for Renderer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Just return if the app is already created
        if self.app.as_ref().lock().is_some() {
            return;
        }

        // Create a new window
        let window_attributes = Window::default_attributes().with_title("tutorial1-window");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let wgpu_app = tokio_runtime::get().block_on(WgpuApp::new(window));
        self.app.lock().replace(wgpu_app);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // 暂停事件
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let mut app = self.app.lock();
        let app = app.as_mut().unwrap();

        // 窗口事件
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
                todo!("Handle keyboard input");
            }
            WindowEvent::RedrawRequested => {
                // notify the windowing system before rendering
                // this will help winit to properly schedule and make assumptions about its internal state
                app.window.pre_present_notify();
                // resize the surface if needed
                app.resize_if_needed();
                // re-render our surface
                if let Err(e) = app.render() {
                    match e {
                        SurfaceError::Lost => {
                            error!("Surface is losted");
                        }
                        _ => {
                            error!("Surface error: {:?}", e);
                        }
                    }
                }
                // Currently we render every frame
                app.window.request_redraw();
            }
            _ => (),
        }
    }
}
