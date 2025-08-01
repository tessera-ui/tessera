use std::{mem, sync::Arc};

use log::{error, info, warn};
use parking_lot::RwLock;
use wgpu::TextureFormat;
use winit::window::Window;

use crate::{
    ComputeCommand, PxPosition, compute::resource::ComputeResourceManager, dp::SCALE_FACTOR,
    px::PxSize, renderer::command::Command,
};

use super::{compute::ComputePipelineRegistry, drawer::Drawer};

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

    // --- Compute resources ---
    compute_target_a: PassTarget,
    compute_target_b: PassTarget,
    compute_commands: Vec<Box<dyn ComputeCommand>>,
    pub resource_manager: Arc<RwLock<ComputeResourceManager>>,
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
            format: caps.formats[0],
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

        // --- Create Pass Targets (A and B and Compute) ---
        let pass_a = Self::create_pass_target(&gpu, &config, "A");
        let pass_b = Self::create_pass_target(&gpu, &config, "B");
        let compute_target_a =
            Self::create_compute_pass_target(&gpu, &config, TextureFormat::Rgba8Unorm, "Compute A");
        let compute_target_b =
            Self::create_compute_pass_target(&gpu, &config, TextureFormat::Rgba8Unorm, "Compute B");

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
            compute_target_a,
            compute_target_b,
            compute_commands: Vec::new(),
            resource_manager: Arc::new(RwLock::new(ComputeResourceManager::new())),
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
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let texture = gpu.create_texture(&texture_descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        PassTarget { texture, view }
    }

    fn create_compute_pass_target(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        format: TextureFormat,
        label_suffix: &str,
    ) -> PassTarget {
        let label = format!("Compute {label_suffix} Texture");
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
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
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
            self.compute_target_a.texture.destroy();
            self.compute_target_b.texture.destroy();

            self.pass_a = Self::create_pass_target(&self.gpu, &self.config, "A");
            self.pass_b = Self::create_pass_target(&self.gpu, &self.config, "B");
            self.compute_target_a = Self::create_compute_pass_target(
                &self.gpu,
                &self.config,
                TextureFormat::Rgba8Unorm,
                "Compute A",
            );
            self.compute_target_b = Self::create_compute_pass_target(
                &self.gpu,
                &self.config,
                TextureFormat::Rgba8Unorm,
                "Compute B",
            );

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

    /// Resize the surface if needed.
    pub(crate) fn resize_if_needed(&mut self) {
        if self.size_changed {
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            self.resize_pass_targets_if_needed();
            self.surface.configure(&self.gpu, &self.config);
            self.size_changed = false;
        }
    }

    /// Render the surface using the unified command system.
    ///
    /// This method processes a stream of commands (both draw and compute) and renders
    /// them to the surface using a multi-pass rendering approach with ping-pong buffers.
    /// Commands that require barriers will trigger texture copies between passes.
    ///
    /// # Arguments
    /// * `commands` - An iterable of (Command, PxSize, PxPosition) tuples representing
    ///   the rendering operations to perform.
    ///
    /// # Returns
    /// * `Ok(())` if rendering succeeds
    /// * `Err(wgpu::SurfaceError)` if there are issues with the surface
    pub(crate) fn render(
        &mut self,
        commands: impl IntoIterator<Item = (Command, PxSize, PxPosition)>,
    ) -> Result<(), wgpu::SurfaceError> {
        let output_frame = self.surface.get_current_texture()?;
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

        // Initialization
        let (mut read_target, mut write_target) = (&mut self.pass_a, &mut self.pass_b);

        // Clear any existing compute commands
        if !self.compute_commands.is_empty() {
            // This is a warning to developers that not all compute commands were used in the last frame.
            warn!("Not every compute command is used in last frame. This is likely a bug.");
            self.compute_commands.clear();
        }

        // Initial clear pass
        {
            let (view, resolve_target) = if let Some(msaa_view) = &self.msaa_view {
                (msaa_view, Some(&write_target.view))
            } else {
                (&write_target.view, None)
            };
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Initial Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            self.drawer
                .begin_pass(&self.gpu, &self.queue, &self.config, &mut rpass);
            self.drawer
                .end_pass(&self.gpu, &self.queue, &self.config, &mut rpass);
        }

        // Frame-level begin for all pipelines
        self.drawer
            .pipeline_registry
            .begin_all_frames(&self.gpu, &self.queue, &self.config);

        // Main command processing loop with barrier handling
        let mut commands_iter = commands.into_iter().peekable();
        let mut scene_texture_view = &read_target.view;
        while let Some((command, size, start_pos)) = commands_iter.next() {
            // Handle barrier requirements by swapping buffers and copying content
            if command.barrier().is_some() {
                // Perform a ping-pong operation
                std::mem::swap(&mut read_target, &mut write_target);
                encoder.copy_texture_to_texture(
                    read_target.texture.as_image_copy(),
                    write_target.texture.as_image_copy(),
                    texture_size,
                );
                // --- Apply compute effect ---
                let final_view_after_compute = if !self.compute_commands.is_empty() {
                    let compute_commands = mem::take(&mut self.compute_commands);
                    Self::do_compute(
                        &mut encoder,
                        compute_commands,
                        &mut self.compute_pipeline_registry,
                        &self.gpu,
                        &self.queue,
                        &self.config,
                        &mut self.resource_manager.write(),
                        &read_target.view,
                        &self.compute_target_a,
                        &self.compute_target_b,
                    )
                } else {
                    &read_target.view
                };
                scene_texture_view = final_view_after_compute;
            }

            match command {
                // Process draw commands using the graphics pipeline
                Command::Draw(command) => {
                    let (view, resolve_target) = if let Some(msaa_view) = &self.msaa_view {
                        (msaa_view, Some(&write_target.view))
                    } else {
                        (&write_target.view, None)
                    };
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            depth_slice: None,
                            resolve_target,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        ..Default::default()
                    });
                    self.drawer
                        .begin_pass(&self.gpu, &self.queue, &self.config, &mut rpass);

                    // Submit the first command
                    self.drawer.submit(
                        &self.gpu,
                        &self.queue,
                        &self.config,
                        &mut rpass,
                        &*command,
                        size,
                        start_pos,
                        scene_texture_view,
                    );

                    // Batch subsequent draw commands that don't require barriers
                    while let Some((Command::Draw(command), _, _)) = commands_iter.peek() {
                        if command.barrier().is_some() {
                            break; // Break if a barrier is required
                        }
                        if let Some((Command::Draw(command), size, start_pos)) =
                            commands_iter.next()
                        {
                            self.drawer.submit(
                                &self.gpu,
                                &self.queue,
                                &self.config,
                                &mut rpass,
                                &*command,
                                size,
                                start_pos,
                                scene_texture_view,
                            );
                        }
                    }
                    self.drawer
                        .end_pass(&self.gpu, &self.queue, &self.config, &mut rpass);
                }
                // Process compute commands using the compute pipeline
                Command::Compute(command) => {
                    self.compute_commands.push(command);
                    // batch subsequent compute commands
                    while let Some((Command::Compute(_), _, _)) = commands_iter.peek() {
                        if let Some((Command::Compute(command), _, _)) = commands_iter.next() {
                            self.compute_commands.push(command);
                        }
                    }
                }
            }
        }

        // Frame-level end for all pipelines
        self.drawer
            .pipeline_registry
            .end_all_frames(&self.gpu, &self.queue, &self.config);

        // Final copy to surface
        encoder.copy_texture_to_texture(
            write_target.texture.as_image_copy(),
            output_frame.texture.as_image_copy(),
            texture_size,
        );

        self.queue.submit(Some(encoder.finish()));
        output_frame.present();

        Ok(())
    }

    fn do_compute<'a>(
        encoder: &mut wgpu::CommandEncoder,
        commands: Vec<Box<dyn ComputeCommand>>,
        compute_pipeline_registry: &mut ComputePipelineRegistry,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        resource_manager: &mut ComputeResourceManager,
        // The initial scene content
        scene_view: &'a wgpu::TextureView,
        // Ping-pong targets
        target_a: &'a PassTarget,
        target_b: &'a PassTarget,
    ) -> &'a wgpu::TextureView {
        if commands.is_empty() {
            return scene_view;
        }

        let mut read_view = scene_view;
        let (mut write_target, mut read_target) = (target_a, target_b);

        for command in commands {
            // Ensure the write target is cleared before use
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Compute Target Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &write_target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            drop(rpass);

            // Create and dispatch the compute pass
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                    timestamp_writes: None,
                });

                compute_pipeline_registry.dispatch_erased(
                    gpu,
                    queue,
                    config,
                    &mut cpass,
                    &*command,
                    resource_manager,
                    read_view,
                    &write_target.view,
                );
            } // cpass is dropped here, ending the pass

            // The result of this pass is now in write_target.
            // For the next iteration, this will be our read source.
            read_view = &write_target.view;
            // Swap targets for the next iteration
            std::mem::swap(&mut write_target, &mut read_target);
        }

        // After the loop, the final result is in the `read_view`,
        // because we swapped one last time at the end of the loop.
        read_view
    }
}
