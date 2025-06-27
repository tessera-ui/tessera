use std::sync::Arc;

use log::{error, info};
use parking_lot::RwLock;
use winit::window::Window;

use crate::{dp::SCALE_FACTOR, Px, PxPosition};

use super::drawer::{DrawCommand, Drawer, RenderRequirement};

// Render pass resources for ping-pong operation
struct PassTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

pub struct WgpuApp {
    /// Avoiding release the window
    #[allow(unused)]
    pub window: Arc<Window>,
    /// WGPU device
    pub gpu: wgpu::Device,
    /// WGPU surface
    surface: wgpu::Surface<'static>,
    /// WGPU queue
    pub queue: wgpu::Queue,
    /// WGPU surface configuration
    pub config: wgpu::SurfaceConfiguration,
    /// size of the window
    size: winit::dpi::PhysicalSize<u32>,
    /// if size is changed
    size_changed: bool,
    /// draw pipelines
    pub drawer: Drawer,

    // --- New ping-pong rendering resources ---
    pass_a: PassTarget,
    pass_b: PassTarget,
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
            .request_device(
                &wgpu::DeviceDescriptor {
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
                },
            )
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
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
            // Fifo is the fallback, it is the most compatible and stable
            wgpu::PresentMode::Fifo
        } else {
            // Immediate is the least preferred, it can cause tearing and is not recommended
            wgpu::PresentMode::Immediate
        };
        info!("Using present mode: {present_mode:?}");
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu, &config);

        // --- Create Pass Targets (A and B) ---
        let pass_a = Self::create_pass_target(&gpu, &config, "A");
        let pass_b = Self::create_pass_target(&gpu, &config, "B");

        let drawer = Drawer::new();
        // Set scale factor for dp conversion
        let scale_factor = window.scale_factor();
        info!("Window scale factor: {scale_factor}");
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
            pass_a,
            pass_b,
        }
    }

    fn create_pass_target(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label_suffix: &str,
    ) -> PassTarget {
        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some(&format!("Pass {} Texture", label_suffix)),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let texture = gpu.create_texture(&texture_descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        PassTarget { texture, view }
    }

    pub fn register_pipelines(&mut self, register_fn: impl FnOnce(&mut Self)) {
        register_fn(self);
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

    pub(crate) fn resize_pass_targets_if_needed(&mut self) {
        if self.size_changed {
            self.pass_a.texture.destroy();
            self.pass_b.texture.destroy();

            self.pass_a = Self::create_pass_target(&self.gpu, &self.config, "A");
            self.pass_b = Self::create_pass_target(&self.gpu, &self.config, "B");
        }
    }

    /// Resize the surface if needed
    pub(crate) fn resize_if_needed(&mut self) {
        if self.size_changed {
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            self.resize_pass_targets_if_needed();
            self.surface.configure(&self.gpu, &self.config);
            self.size_changed = false;
        }
    }

    /// Render the surface
    pub(crate) fn render(
        &mut self,
        drawer_commands: impl IntoIterator<Item = (PxPosition, [Px; 2], Box<dyn DrawCommand>)>,
    ) -> Result<(), wgpu::SurfaceError> {
        let output_frame = self.surface.get_current_texture()?;
        let _output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let texture_size = wgpu::Extent3d {
            width: self.config.width,
            height: self.config.height,
            depth_or_array_layers: 1,
        };

        // 1. Initialization
        let (mut read_target, mut write_target) = (&mut self.pass_a, &mut self.pass_b);

        // Initial clear: Clear the first "canvas"
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Initial Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &write_target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        // 2. Main loop and command processing
        let mut commands_iter = drawer_commands.into_iter().peekable();
        while commands_iter.peek().is_some() {
            // --- Step A: Batch all consecutive standard commands ---
            let standard_batch: Vec<_> = commands_iter
                .by_ref()
                .take_while(|c| c.2.requirement() == RenderRequirement::Standard)
                .collect();

            if !standard_batch.is_empty() {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Standard Batch Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &write_target.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });

                self.drawer
                    .begin_pass(&self.gpu, &self.queue, &self.config, &mut pass);
                for (pos, size, cmd) in standard_batch {
                    self.drawer.submit(
                        &self.gpu,
                        &self.queue,
                        &self.config,
                        &mut pass,
                        &*cmd,
                        size,
                        pos,
                        None,
                    );
                }
                self.drawer
                    .end_pass(&self.gpu, &self.queue, &self.config, &mut pass);
            }

            // --- Step B: Process a single render barrier ---
            if let Some((pos, size, barrier_command)) = commands_iter.next() {
                // 1. Swap read and write targets (ping-pong operation)
                std::mem::swap(&mut read_target, &mut write_target);

                // 2. Copy the content of the new background (read_target) to the new canvas (write_target)
                encoder.copy_texture_to_texture(
                    read_target.texture.as_image_copy(),
                    write_target.texture.as_image_copy(),
                    texture_size,
                );

                // 3. Begin a new render pass to draw the barrier component
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Barrier Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &write_target.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });

                // 4. Draw the barrier command
                self.drawer
                    .begin_pass(&self.gpu, &self.queue, &self.config, &mut pass);
                self.drawer.submit(
                    &self.gpu,
                    &self.queue,
                    &self.config,
                    &mut pass,
                    &*barrier_command,
                    size,
                    pos,
                    Some(&read_target.view),
                );
                self.drawer
                    .end_pass(&self.gpu, &self.queue, &self.config, &mut pass);
            }
        }

        // 3. Final output
        encoder.copy_texture_to_texture(
            write_target.texture.as_image_copy(),
            output_frame.texture.as_image_copy(),
            texture_size,
        );

        self.queue.submit(Some(encoder.finish()));
        output_frame.present();
        Ok(())
    }
}
