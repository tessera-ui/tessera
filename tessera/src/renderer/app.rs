use std::sync::Arc;

use log::{error, info};
use parking_lot::RwLock;
use winit::window::Window;

use crate::dp::SCALE_FACTOR;

use super::drawer::{DrawCommand, Drawer};

pub(crate) struct WgpuApp {
    /// Avoiding release the window
    #[allow(unused)]
    pub window: Arc<Window>,
    /// WGPU device
    gpu: wgpu::Device,
    /// WGPU surface
    surface: wgpu::Surface<'static>,
    /// WGPU queue
    queue: wgpu::Queue,
    /// WGPU surface configuration
    config: wgpu::SurfaceConfiguration,
    /// size of the window
    size: winit::dpi::PhysicalSize<u32>,
    /// if size is changed
    size_changed: bool,
    /// draw pipelines
    pub drawer: Drawer,
}

impl WgpuApp {
    /// Create a new WGPU app, as the root of Tessera
    pub(crate) async fn new(window: Arc<Window>) -> Self {
        // Looking for gpus
        let instance: wgpu::Instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        // Create a surface
        let surface = match instance.create_surface(window.clone()) {
            Ok(surface) => surface,
            Err(e) => {
                error!("Failed to create surface: {e:?}");
                panic!("Failed to create surface: {e:?}");
            }
        };
        // Looking for adapter gpu
        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Ok(gpu) => gpu,
            Err(e) => {
                error!("Failed to find an appropriate adapter: {e:?}");
                panic!("Failed to find an appropriate adapter: {e:?}");
            }
        };
        // Create a device and queue
        let (gpu, queue) = match adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL backend does not support all features
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
        {
            Ok((gpu, queue)) => (gpu, queue),
            Err(e) => {
                error!("Failed to create device: {e:?}");
                panic!("Failed to create device: {e:?}");
            }
        };
        // Create surface configuration
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        // Choose the present mode
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            // Mailbox is the best choice for most cases, it allows for low latency and high FPS
            wgpu::PresentMode::Mailbox
        } else if caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
            // Fifo is the fallback, it is the most compatible and stable
            wgpu::PresentMode::Fifo
        } else {
            // Immediate is the least preferred, it can cause tearing and is not recommended
            wgpu::PresentMode::Immediate
        };
        info!("Using present mode: {:?}", present_mode);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu, &config);
        // Create drawer
        let drawer = Drawer::new(&gpu, &queue, &config);
        // Set scale factor for dp conversion
        let scale_factor = window.scale_factor();
        info!("Window scale factor: {}", scale_factor);
        SCALE_FACTOR
            .set(RwLock::new(scale_factor))
            .expect("Failed to set scale factor");

        Self {
            window,
            gpu,
            surface,
            queue,
            config,
            size,
            size_changed: false,
            drawer,
        }
    }

    /// Resize the surface
    /// Real resize will be done in the next frame, in [Self::resize_if_needed]
    pub(crate) fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if self.size == size {
            return;
        }
        self.size = size;
        self.size_changed = true;
    }

    /// Get the size of the surface
    pub(crate) fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    /// Resize the surface if needed
    pub(crate) fn resize_if_needed(&mut self) {
        if self.size_changed {
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            self.surface.configure(&self.gpu, &self.config);
            self.size_changed = false;
        }
    }

    /// Render the surface
    pub(crate) fn render(
        &mut self,
        drawer_commands: impl IntoIterator<Item = DrawCommand>,
    ) -> Result<(), wgpu::SurfaceError> {
        // get a texture from surface
        let output = self.surface.get_current_texture()?;
        // create a command encoder
        let mut encoder = self
            .gpu
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        // encode render commands
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        // Begin frame for drawer
        self.drawer.begin_frame();

        // draw commands
        for command in drawer_commands {
            // Use the collected Vec
            self.drawer.prepare_or_draw(
                &self.gpu,
                &self.config,
                &self.queue,
                &mut render_pass,
                command,
            );
        }
        // we must call [Drawer::final_draw] to render drawers that need to be prepared
        // before drawing
        self.drawer
            .final_draw(&self.gpu, &self.config, &self.queue, &mut render_pass);
        // here we drop render_pass to release borrowed encoder
        drop(render_pass);
        // finish command buffer and submit it to gpu queue
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}
