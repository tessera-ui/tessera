use std::{any::TypeId, mem, sync::Arc};

use log::{error, info, warn};
use parking_lot::RwLock;
use wgpu::{ImageSubresourceRange, TextureFormat};
use winit::window::Window;

use crate::{
    ComputeCommand, DrawCommand, Px, PxPosition,
    compute::resource::ComputeResourceManager,
    dp::SCALE_FACTOR,
    px::{PxRect, PxSize},
    renderer::command::{BarrierRequirement, Command},
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
    compute_commands: Vec<(Box<dyn ComputeCommand>, PxSize, PxPosition)>,
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
                required_features: wgpu::Features::empty() | wgpu::Features::CLEAR_TEXTURE,
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
        commands: impl IntoIterator<Item = (Command, TypeId, PxSize, PxPosition)>,
    ) -> Result<(), wgpu::SurfaceError> {
        // Collect commands into a Vec to allow reordering
        let commands: Vec<_> = commands.into_iter().collect();
        // Reorder instructions based on dependencies for better batching optimization
        let commands = super::reorder::reorder_instructions(commands);

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

        // Flag for first pass
        let mut is_first_pass = true;

        // Frame-level begin for all pipelines
        self.drawer
            .pipeline_registry
            .begin_all_frames(&self.gpu, &self.queue, &self.config);

        // Main command processing loop with barrier handling
        let mut scene_texture_view = &read_target.view;
        let mut commands_in_pass: Vec<DrawCommandWithMetadata> = Vec::new();
        let mut barrier_draw_rects_in_pass: Vec<PxRect> = Vec::new();
        let mut clip_temp_stack = Vec::new();
        let mut clip_stack = Vec::new();

        for (command, command_type_id, size, start_pos) in commands {
            let need_new_pass = commands_in_pass
                .last()
                .map(|cmd| {
                    match (cmd.command.barrier(), command.barrier()) {
                        (None, Some(_)) => true, // If the last command has no barrier but the next one does, we need a new pass
                        (Some(_), Some(barrier)) => {
                            // If both have barriers, we need to check if they are orthogonal
                            // First extract the last barrier's draw rect
                            let last_draw_rect =
                                extract_draw_rect(Some(barrier), size, start_pos, texture_size);
                            // Then check if the last draw rect is orthogonal to all existing draw rects in this pass
                            !barrier_draw_rects_in_pass
                                .iter()
                                .all(|dr| dr.is_orthogonal(&last_draw_rect)) // We don't need a new pass if the last command's barrier is orthogonal to all existing draw rects
                        }
                        (Some(_), None) => false, // If the last command has a barrier but the next one does not, we can continue in the same pass
                        (None, None) => false, // If both have no barriers, we can continue in the same pass
                    }
                })
                .unwrap_or(false);

            if need_new_pass {
                // A ping-pong operation is needed if the first command in the pass has a barrier
                if commands_in_pass
                    .first()
                    .map(|cmd| cmd.command.barrier().is_some())
                    .unwrap_or(false)
                {
                    // Perform a ping-pong operation
                    mem::swap(&mut read_target, &mut write_target);

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

                // Render the current pass before starting a new one
                render_current_pass(
                    &self.msaa_view,
                    &mut is_first_pass,
                    &mut encoder,
                    write_target,
                    &mut commands_in_pass,
                    scene_texture_view,
                    &mut self.drawer,
                    &self.gpu,
                    &self.queue,
                    &self.config,
                    &mut clip_stack,
                );
                commands_in_pass.clear();
                barrier_draw_rects_in_pass.clear();
            }

            match command {
                Command::Draw(cmd) => {
                    // Extract the draw rectangle based on the command's barrier, size and position
                    let draw_rect = extract_draw_rect(cmd.barrier(), size, start_pos, texture_size);
                    // If the command has a barrier, we need to track the draw rect for orthogonality checks
                    if cmd.barrier().is_some() {
                        barrier_draw_rects_in_pass.push(draw_rect);
                    }
                    // Add the command to the current pass
                    commands_in_pass.push(DrawCommandWithMetadata {
                        command: cmd,
                        type_id: command_type_id,
                        size,
                        start_pos,
                        draw_rect,
                        clip_ops: clip_temp_stack.pop(), // If there is a clipping operation, we take it from the temp stack
                    });
                }
                Command::Compute(cmd) => {
                    // Add the compute command to the current pass
                    self.compute_commands.push((cmd, size, start_pos));
                }
                Command::ClipPush(rect) => {
                    // Push it into temp stack
                    clip_temp_stack.push(ClipOps::Push(rect)); // we'll use this for next command
                }
                Command::ClipPop => {
                    // Push it into temp stack
                    clip_temp_stack.push(ClipOps::Pop); // we'll use this for next command
                }
            }
        }

        // After processing all commands, we need to render the last pass if there are any commands left
        if !commands_in_pass.is_empty() {
            // A ping-pong operation is needed if the first command in the pass has a barrier
            if commands_in_pass
                .first()
                .map(|cmd| cmd.command.barrier().is_some())
                .unwrap_or(false)
            {
                // Perform a ping-pong operation
                mem::swap(&mut read_target, &mut write_target);

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

            // Render the current pass before starting a new one
            render_current_pass(
                &self.msaa_view,
                &mut is_first_pass,
                &mut encoder,
                write_target,
                &mut commands_in_pass,
                scene_texture_view,
                &mut self.drawer,
                &self.gpu,
                &self.queue,
                &self.config,
                &mut clip_stack,
            );
            commands_in_pass.clear();
            barrier_draw_rects_in_pass.clear();
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
        commands: Vec<(Box<dyn ComputeCommand>, PxSize, PxPosition)>,
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

        for (command, size, start_pos) in commands {
            // Ensure the write target is cleared before use
            encoder.clear_texture(
                &write_target.texture,
                &ImageSubresourceRange {
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                },
            );

            // Create and dispatch the compute pass
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                    timestamp_writes: None,
                });

                // Get the area of the compute command
                let area = match command.barrier() {
                    BarrierRequirement::Global => PxRect {
                        x: Px(0),
                        y: Px(0),
                        width: Px(config.width as i32),
                        height: Px(config.height as i32),
                    },
                    BarrierRequirement::PaddedLocal {
                        top,
                        right,
                        bottom,
                        left,
                    } => {
                        let padded_x = (start_pos.x - left).max(Px(0));
                        let padded_y = (start_pos.y - top).max(Px(0));
                        let padded_width =
                            (size.width + left + right).min(Px(config.width as i32 - padded_x.0));
                        let padded_height =
                            (size.height + top + bottom).min(Px(config.height as i32 - padded_y.0));
                        PxRect {
                            x: padded_x,
                            y: padded_y,
                            width: padded_width,
                            height: padded_height,
                        }
                    }
                    BarrierRequirement::Absolute(rect) => rect,
                };

                compute_pipeline_registry.dispatch_erased(
                    gpu,
                    queue,
                    config,
                    &mut cpass,
                    &*command,
                    resource_manager,
                    area,
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

fn extract_draw_rect(
    barrier: Option<BarrierRequirement>,
    size: PxSize,
    start_pos: PxPosition,
    texture_size: wgpu::Extent3d,
) -> PxRect {
    match barrier {
        Some(BarrierRequirement::Global) => PxRect {
            x: Px(0),
            y: Px(0),
            width: Px(texture_size.width as i32),
            height: Px(texture_size.height as i32),
        },
        Some(BarrierRequirement::PaddedLocal {
            top,
            right,
            bottom,
            left,
        }) => {
            let padded_x = (start_pos.x - left).max(Px(0));
            let padded_y = (start_pos.y - top).max(Px(0));
            let padded_width =
                (size.width + left + right).min(Px(texture_size.width as i32 - padded_x.0));
            let padded_height =
                (size.height + top + bottom).min(Px(texture_size.height as i32 - padded_y.0));
            PxRect {
                x: padded_x,
                y: padded_y,
                width: padded_width,
                height: padded_height,
            }
        }
        Some(BarrierRequirement::Absolute(mut rect)) => {
            rect.x = rect.x.positive().min(texture_size.width).into();
            rect.y = rect.y.positive().min(texture_size.height).into();
            rect.width = rect
                .width
                .positive()
                .min(texture_size.width - rect.x.positive())
                .into();
            rect.height = rect
                .height
                .positive()
                .min(texture_size.height - rect.y.positive())
                .into();
            rect
        }
        None => {
            let x = start_pos.x.positive().min(texture_size.width);
            let y = start_pos.y.positive().min(texture_size.height);
            let width = size.width.positive().min(texture_size.width - x);
            let height = size.height.positive().min(texture_size.height - y);
            PxRect {
                x: Px::from(x),
                y: Px::from(y),
                width: Px::from(width),
                height: Px::from(height),
            }
        }
    }
}

fn render_current_pass(
    msaa_view: &Option<wgpu::TextureView>,
    is_first_pass: &mut bool,
    encoder: &mut wgpu::CommandEncoder,
    write_target: &PassTarget,
    commands_in_pass: &mut Vec<DrawCommandWithMetadata>,
    scene_texture_view: &wgpu::TextureView,
    drawer: &mut Drawer,
    gpu: &wgpu::Device,
    queue: &wgpu::Queue,
    config: &wgpu::SurfaceConfiguration,
    clip_stack: &mut Vec<PxRect>,
) {
    let (view, resolve_target) = if let Some(msaa_view) = msaa_view {
        (msaa_view, Some(&write_target.view))
    } else {
        (&write_target.view, None)
    };

    let load_ops = if *is_first_pass {
        *is_first_pass = false;
        // If this is the first pass, we load the texture
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
    } else {
        // Otherwise, we load the existing content
        wgpu::LoadOp::Load
    };

    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target,
            ops: wgpu::Operations {
                load: load_ops,
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });

    drawer.begin_pass(gpu, queue, config, &mut rpass, scene_texture_view);

    // Submit all batched draw commands to the drawer
    let mut buffer: Vec<(Box<dyn DrawCommand>, PxSize, PxPosition)> = Vec::new();
    let mut last_command_type_id = None;
    // Use a separate variable to track the current batch draw rectangle
    let mut current_batch_draw_rect: Option<PxRect> = None;
    for cmd in mem::take(commands_in_pass).into_iter() {
        if last_command_type_id != Some(cmd.type_id) || cmd.clip_ops.is_some() {
            if !buffer.is_empty() {
                let commands = mem::take(&mut buffer); // Clear and take the buffer
                let commands = commands
                    .iter()
                    .map(|(cmd, sz, pos)| (&**cmd, *sz, *pos))
                    .collect::<Vec<_>>();
                if let Some(clipped_rect) = clip_stack.last() {
                    let Some(final_rect) =
                        current_batch_draw_rect.unwrap().intersection(clipped_rect)
                    else {
                        continue;
                    };
                    current_batch_draw_rect = Some(final_rect);
                }
                rpass.set_scissor_rect(
                    current_batch_draw_rect.unwrap().x.positive(),
                    current_batch_draw_rect.unwrap().y.positive(),
                    current_batch_draw_rect.unwrap().width.positive(),
                    current_batch_draw_rect.unwrap().height.positive(),
                );
                drawer.submit(
                    gpu,
                    queue,
                    config,
                    &mut rpass,
                    &commands,
                    scene_texture_view,
                );
                current_batch_draw_rect = None; // Reset the current batch draw rectangle
            }

            last_command_type_id = Some(cmd.type_id);
        }

        // Handle clipping operations
        if let Some(clip_ops) = cmd.clip_ops {
            match clip_ops {
                ClipOps::Push(rect) => {
                    clip_stack.push(rect);
                }
                ClipOps::Pop => {
                    clip_stack.pop();
                }
            }
        }

        // Add the command to the buffer
        buffer.push((cmd.command, cmd.size, cmd.start_pos));
        if let Some(draw_rect) = current_batch_draw_rect {
            // If we already have a draw rectangle, we need to update it
            current_batch_draw_rect = Some(draw_rect.union(&cmd.draw_rect));
        } else {
            // Otherwise, we set the current batch draw rectangle to the command's draw rectangle
            current_batch_draw_rect = Some(cmd.draw_rect);
        }
    }
    // If there are any remaining commands in the buffer, submit them
    if !buffer.is_empty() {
        let commands = mem::take(&mut buffer); // Clear and take the buffer
        let commands = commands
            .iter()
            .map(|(cmd, sz, pos)| (&**cmd, *sz, *pos))
            .collect::<Vec<_>>();
        if let Some(clipped_rect) = clip_stack.last() {
            if let Some(current_batch_draw_rect) =
                current_batch_draw_rect.unwrap().intersection(clipped_rect)
            {
                rpass.set_scissor_rect(
                    current_batch_draw_rect.x.positive(),
                    current_batch_draw_rect.y.positive(),
                    current_batch_draw_rect.width.positive(),
                    current_batch_draw_rect.height.positive(),
                );
                drawer.submit(
                    gpu,
                    queue,
                    config,
                    &mut rpass,
                    &commands,
                    scene_texture_view,
                );
            };
        } else {
            rpass.set_scissor_rect(
                current_batch_draw_rect.unwrap().x.positive(),
                current_batch_draw_rect.unwrap().y.positive(),
                current_batch_draw_rect.unwrap().width.positive(),
                current_batch_draw_rect.unwrap().height.positive(),
            );
            drawer.submit(
                gpu,
                queue,
                config,
                &mut rpass,
                &commands,
                scene_texture_view,
            );
        }
    }

    drawer.end_pass(gpu, queue, config, &mut rpass, scene_texture_view);
}

struct DrawCommandWithMetadata {
    command: Box<dyn DrawCommand>,
    type_id: TypeId,
    size: PxSize,
    start_pos: PxPosition,
    draw_rect: PxRect,
    clip_ops: Option<ClipOps>,
}

enum ClipOps {
    Push(PxRect),
    Pop,
}
