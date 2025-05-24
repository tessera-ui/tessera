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
    component_tree::{
        BasicDrawable, ComponentNode, ComponentTree, Constraint, DEFAULT_LAYOUT_DESC,
        LayoutDescription, PositionRelation,
    },
    tokio_runtime,
};

pub use drawer::{DrawCommand, ShapeVertex, TextConstraint};

#[derive(Default)]
pub(crate) struct Renderer {
    /// WGPU app
    app: Arc<Mutex<Option<WgpuApp>>>,
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
                // render the surface
                // for now, we have a simple component tree as a test
                let mut component_tree = ComponentTree::new();
                // Add a root node
                // Here we draw a rectangle
                // with a size of 100x100 and a color of red
                // and place child node in center of the rectangle
                component_tree.add_node(ComponentNode {
                    layout_desc: Box::new(|inputs| {
                        let input = inputs[0];
                        let x = 1000 / 2 - input.width / 2;
                        let y = 1000 / 2 - input.height / 2;
                        vec![LayoutDescription {
                            relative_position: PositionRelation {
                                offset_x: x,
                                offset_y: y,
                            },
                        }]
                    }),
                    constraint: Constraint {
                        min_width: 1000,
                        min_height: 1000,
                        max_width: 1000,
                        max_height: 1000,
                    },
                    drawable: Some(BasicDrawable::Rect {
                        color: [1.0, 0.0, 0.0], // Red
                    }),
                });
                // Add a child node
                // Here we draw a rectangle with a size of 50x50 and a color of blue
                component_tree.add_node(ComponentNode {
                    layout_desc: Box::new(DEFAULT_LAYOUT_DESC),
                    constraint: Constraint {
                        min_width: 500,
                        min_height: 500,
                        max_width: 500,
                        max_height: 500,
                    },
                    drawable: Some(BasicDrawable::Rect {
                        color: [0.0, 0.0, 1.0], // Blue
                    }),
                });
                // Add a text node
                component_tree.add_node(ComponentNode {
                    layout_desc: Box::new(DEFAULT_LAYOUT_DESC),
                    constraint: Constraint {
                        min_width: 100,
                        min_height: 500,
                        max_width: 100,
                        max_height: 500,
                    },
                    drawable: Some(BasicDrawable::Text {
                        text: "Hello, this is Tessera~~~~~".into(),
                        color: [1.0, 1.0, 1.0], // Black
                        font_size: 20.0,
                        line_height: 20.0,
                    }),
                });
                // Compute the draw commands
                let commands = component_tree.compute();
                app.render(commands).unwrap();
                // Currently we render every frame
                app.window.request_redraw();
            }
            _ => (),
        }
    }
}
