use std::sync::Arc;

use log::{error, info};
use parking_lot::RwLock;
use winit::window::Window;

use crate::{PxPosition, dp::SCALE_FACTOR, px::PxSize};

use super::{
    compute::ComputePipelineRegistry,
    drawer::{DrawCommand, Drawer, RenderRequirement},
};

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
    /// compute pipelines
    pub compute_pipeline_registry: ComputePipelineRegistry,

    // --- New ping-pong rendering resources ---
    pass_a: PassTarget,
    pass_b: PassTarget,

    // --- MSAA resources ---
    pub sample_count: u32,
    msaa_texture: Option<wgpu::Texture>,
    msaa_view: Option<wgpu::TextureView>,

    // --- Blink workaround resources ---
    /// Offscreen texture for first blink render (doesn't get presented)
    blink_offscreen_texture: PassTarget,
}

impl WgpuApp {
    /// Create a new WGPU app, as the root of Tessera
    pub(crate) async fn new(window: Arc<Window>, sample_count: u32) -> Self {
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
            format: wgpu::TextureFormat::Rgba8Unorm,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu, &config);

        // --- Create MSAA Target ---
        let (msaa_texture, msaa_view) = if sample_count > 1 {
            let texture = gpu.create_texture(&wgpu::TextureDescriptor {
                label: Some("MSAA Framebuffer"),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                // Use surface format to match pass targets
                format: config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(texture), Some(view))
        } else {
            (None, None)
        };

        // --- Create Pass Targets (A and B) ---
        let pass_a = Self::create_pass_target(&gpu, &config, "A");
        let pass_b = Self::create_pass_target(&gpu, &config, "B");

        // --- Create Blink Offscreen Target ---
        let blink_offscreen_texture = Self::create_pass_target(&gpu, &config, "BlinkOffscreen");

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
            compute_pipeline_registry: ComputePipelineRegistry::new(),
            sample_count,
            msaa_texture,
            msaa_view,
            blink_offscreen_texture,
        }
    }

    fn create_pass_target(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label_suffix: &str,
    ) -> PassTarget {
        let label = format!("Pass {label_suffix} Texture");
        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some(&label),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Use surface format for compatibility with final copy operations
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::STORAGE_BINDING,
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
            self.blink_offscreen_texture.texture.destroy();

            self.pass_a = Self::create_pass_target(&self.gpu, &self.config, "A");
            self.pass_b = Self::create_pass_target(&self.gpu, &self.config, "B");
            self.blink_offscreen_texture =
                Self::create_pass_target(&self.gpu, &self.config, "BlinkOffscreen");

            if self.sample_count > 1 {
                if let Some(t) = self.msaa_texture.take() {
                    t.destroy()
                }
                let texture = self.gpu.create_texture(&wgpu::TextureDescriptor {
                    label: Some("MSAA Framebuffer"),
                    size: wgpu::Extent3d {
                        width: self.config.width,
                        height: self.config.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: self.sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    // Use surface format to match pass targets
                    format: self.config.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                self.msaa_texture = Some(texture);
                self.msaa_view = Some(view);
            }
        }
    }

    /// Resize the surface if needed
    ///
    /// Returns `true` if resize occurred, indicating the need for a "blink" (double render).
    /// This is a workaround for what appears to be a deep WGPU/GPU driver bug where
    /// FluidGlass components with blur effects disappear during window resize.
    ///
    /// The issue manifests as:
    /// - Only occurs during window resize events
    /// - Only affects components that sample background (FluidGlass with blur)
    /// - All texture dimensions are verified to be correct
    /// - All rendering logic appears sound
    ///
    /// Root cause investigation showed:
    /// - Texture sizes are consistent between source and destination
    /// - GPU commands are properly ordered
    /// - No obvious application-level bugs
    ///
    /// The "blink" mechanism (double rendering on resize) reliably fixes the issue,
    /// suggesting a GPU state synchronization problem at the driver/hardware level.
    /// This workaround prioritizes user experience over ideological purity.
    pub(crate) fn resize_if_needed(&mut self) -> bool {
        let blink = self.size_changed;
        if self.size_changed {
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            self.resize_pass_targets_if_needed();
            self.surface.configure(&self.gpu, &self.config);
            self.size_changed = false;
        }
        blink
    }

    /// Render the surface
    pub(crate) fn render(
        &mut self,
        drawer_commands: impl IntoIterator<Item = (PxPosition, PxSize, Box<dyn DrawCommand>)>,
    ) -> Result<(), wgpu::SurfaceError> {
        self.render_internal(drawer_commands, None)
    }

    /// Render to offscreen texture (for blink workaround)
    pub(crate) fn render_offscreen(
        &mut self,
        drawer_commands: impl IntoIterator<Item = (PxPosition, PxSize, Box<dyn DrawCommand>)>,
    ) {
        // Clone the view to avoid borrow issues
        let offscreen_view = self.blink_offscreen_texture.view.clone();
        let _ = self.render_internal(drawer_commands, Some(&offscreen_view));
    }

    /// Internal render method that can render to surface or offscreen
    fn render_internal(
        &mut self,
        drawer_commands: impl IntoIterator<Item = (PxPosition, PxSize, Box<dyn DrawCommand>)>,
        offscreen_target: Option<&wgpu::TextureView>,
    ) -> Result<(), wgpu::SurfaceError> {
        let (output_frame, _surface_view) = if offscreen_target.is_some() {
            // Render to offscreen - no surface frame needed
            (None, None)
        } else {
            // Render to surface
            let frame = self.surface.get_current_texture()?;
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            (Some(frame), Some(view))
        };
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
        let (view, resolve_target) = if let Some(msaa_view) = &self.msaa_view {
            (msaa_view, Some(&write_target.view))
        } else {
            (&write_target.view, None)
        };
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Initial Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        // 2. Main loop and command processing
        let mut commands_iter = drawer_commands.into_iter();

        'main_loop: loop {
            // --- Step A: Batch all consecutive standard commands ---
            let mut standard_batch = Vec::new();
            for command in commands_iter.by_ref() {
                if command.2.requirement() == RenderRequirement::Standard {
                    standard_batch.push(command);
                } else {
                    // This is a barrier command, first render the collected standard commands
                    if !standard_batch.is_empty() {
                        let (view, resolve_target) = if let Some(msaa_view) = &self.msaa_view {
                            (msaa_view, Some(&write_target.view))
                        } else {
                            (&write_target.view, None)
                        };
                        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Standard Batch Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view,
                                resolve_target,
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
                                &mut self.compute_pipeline_registry,
                            );
                        }
                        self.drawer
                            .end_pass(&self.gpu, &self.queue, &self.config, &mut pass);
                    }

                    // Now process the barrier command
                    // 1. Swap read and write targets (ping-pong operation)
                    std::mem::swap(&mut read_target, &mut write_target);

                    // 2. Copy the content of the new background (read_target) to the new canvas (write_target)
                    encoder.copy_texture_to_texture(
                        read_target.texture.as_image_copy(),
                        write_target.texture.as_image_copy(),
                        texture_size,
                    );

                    // 3. Begin a new render pass to draw the barrier component
                    let (view, resolve_target) = if let Some(msaa_view) = &self.msaa_view {
                        (msaa_view, Some(&write_target.view))
                    } else {
                        (&write_target.view, None)
                    };
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Barrier Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target,
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
                    let (pos, size, barrier_command) = command;
                    self.drawer.submit(
                        &self.gpu,
                        &self.queue,
                        &self.config,
                        &mut pass,
                        &*barrier_command,
                        size,
                        pos,
                        Some(&read_target.view),
                        &mut self.compute_pipeline_registry,
                    );
                    self.drawer
                        .end_pass(&self.gpu, &self.queue, &self.config, &mut pass);

                    // Continue the main loop to process next commands
                    continue 'main_loop;
                }
            }

            // Render the last batch of standard commands if any
            if !standard_batch.is_empty() {
                let (view, resolve_target) = if let Some(msaa_view) = &self.msaa_view {
                    (msaa_view, Some(&write_target.view))
                } else {
                    (&write_target.view, None)
                };
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Final Standard Batch Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target,
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
                        &mut self.compute_pipeline_registry,
                    );
                }
                self.drawer
                    .end_pass(&self.gpu, &self.queue, &self.config, &mut pass);
            }

            // No more commands, break the main loop
            break;
        }

        // 3. Final output
        let final_destination = if let Some(_offscreen_view) = offscreen_target {
            // Render to offscreen texture
            self.blink_offscreen_texture.texture.as_image_copy()
        } else {
            // Render to surface
            output_frame.as_ref().unwrap().texture.as_image_copy()
        };

        encoder.copy_texture_to_texture(
            write_target.texture.as_image_copy(),
            final_destination,
            texture_size,
        );

        self.queue.submit(Some(encoder.finish()));

        // Only present if rendering to surface (not offscreen)
        if let Some(frame) = output_frame {
            frame.present();
        }

        Ok(())
    }
}
