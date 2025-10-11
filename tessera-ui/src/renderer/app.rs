use std::{any::TypeId, mem, sync::Arc};

use parking_lot::RwLock;
use tracing::{error, info, warn};
use wgpu::{ImageSubresourceRange, TextureFormat};
use winit::window::Window;

use crate::{
    ComputablePipeline, ComputeCommand, DrawCommand, DrawablePipeline, Px, PxPosition,
    compute::resource::ComputeResourceManager,
    dp::SCALE_FACTOR,
    px::{PxRect, PxSize},
    renderer::command::{BarrierRequirement, Command},
};

use super::{compute::ComputePipelineRegistry, drawer::Drawer};

// WGPU context for ping-pong operations
struct WgpuContext<'a> {
    encoder: &'a mut wgpu::CommandEncoder,
    gpu: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
}

// Parameters for render_current_pass function
struct RenderCurrentPassParams<'a> {
    msaa_view: &'a Option<wgpu::TextureView>,
    is_first_pass: &'a mut bool,
    encoder: &'a mut wgpu::CommandEncoder,
    write_target: &'a wgpu::TextureView,
    final_view: Option<&'a wgpu::TextureView>,
    commands_in_pass: &'a mut Vec<DrawOrClip>,
    scene_texture_view: &'a wgpu::TextureView,
    drawer: &'a mut Drawer,
    gpu: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
    clip_stack: &'a mut Vec<PxRect>,
}

// Parameters for do_compute function
struct DoComputeParams<'a> {
    encoder: &'a mut wgpu::CommandEncoder,
    commands: Vec<(Box<dyn ComputeCommand>, PxSize, PxPosition)>,
    compute_pipeline_registry: &'a mut ComputePipelineRegistry,
    gpu: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
    resource_manager: &'a mut ComputeResourceManager,
    scene_view: &'a wgpu::TextureView,
    target_a: &'a wgpu::TextureView,
    target_b: &'a wgpu::TextureView,
}

// Compute resources for ping-pong operations
struct ComputeResources<'a> {
    compute_commands: &'a mut Vec<(Box<dyn ComputeCommand>, PxSize, PxPosition)>,
    compute_pipeline_registry: &'a mut ComputePipelineRegistry,
    resource_manager: &'a mut ComputeResourceManager,
    compute_target_a: &'a wgpu::TextureView,
    compute_target_b: &'a wgpu::TextureView,
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

    // ping-pong rendering resources
    pass_a: wgpu::TextureView,
    pass_b: wgpu::TextureView,

    // MSAA resources
    pub sample_count: u32,
    msaa_texture: Option<wgpu::Texture>,
    msaa_view: Option<wgpu::TextureView>,

    // Compute resources
    compute_target_a: wgpu::TextureView,
    compute_target_b: wgpu::TextureView,
    compute_commands: Vec<(Box<dyn ComputeCommand>, PxSize, PxPosition)>,
    pub resource_manager: Arc<RwLock<ComputeResourceManager>>,

    // Blit resources for partial copies
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group_layout: wgpu::BindGroupLayout,
    blit_sampler: wgpu::Sampler,
}

impl WgpuApp {
    // Small helper functions extracted from `new` to reduce its complexity.
    //
    // These helpers keep behavior unchanged but make `new` shorter and easier to analyze.
    async fn request_adapter_for_surface(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface<'_>,
    ) -> wgpu::Adapter {
        match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Ok(gpu) => gpu,
            Err(e) => {
                error!("Failed to find an appropriate adapter: {e:?}");
                panic!("Failed to find an appropriate adapter: {e:?}");
            }
        }
    }

    async fn request_device_and_queue_for_adapter(
        adapter: &wgpu::Adapter,
    ) -> (wgpu::Device, wgpu::Queue) {
        match adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty() | wgpu::Features::CLEAR_TEXTURE,
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
        }
    }

    fn make_msaa_resources(
        gpu: &wgpu::Device,
        sample_count: u32,
        config: &wgpu::SurfaceConfiguration,
    ) -> (Option<wgpu::Texture>, Option<wgpu::TextureView>) {
        if sample_count > 1 {
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
                format: config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(texture), Some(view))
        } else {
            (None, None)
        }
    }

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
        let adapter = Self::request_adapter_for_surface(&instance, &surface).await;
        // Create a device and queue
        let (gpu, queue) = Self::request_device_and_queue_for_adapter(&adapter).await;
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
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu, &config);

        // Create MSAA Target
        let (msaa_texture, msaa_view) = Self::make_msaa_resources(&gpu, sample_count, &config);

        // Create Pass Targets (A and B and Compute)
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

        // Create blit pipeline resources
        let blit_shader = gpu.create_shader_module(wgpu::include_wgsl!("shaders/blit.wgsl"));
        let blit_sampler = gpu.create_sampler(&wgpu::SamplerDescriptor::default());
        let blit_bind_group_layout =
            gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Blit Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let blit_pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&blit_bind_group_layout],
            push_constant_ranges: &[],
        });

        let blit_pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(config.format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

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
            blit_pipeline,
            blit_bind_group_layout,
            blit_sampler,
        }
    }

    /// Registers a new drawable pipeline for a specific command type.
    ///
    /// This method takes ownership of the pipeline and wraps it in a type-erased container that can be stored alongside other pipelines of different types.
    pub fn register_draw_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: DrawCommand + 'static,
        P: DrawablePipeline<T> + 'static,
    {
        self.drawer.pipeline_registry.register(pipeline);
    }

    /// Registers a new compute pipeline for a specific command type.
    ///
    /// This method takes ownership of the pipeline and wraps it in a type-erased container that can be stored alongside other pipelines of different types.
    pub fn register_compute_pipeline<T, P>(&mut self, pipeline: P)
    where
        T: ComputeCommand + 'static,
        P: ComputablePipeline<T> + 'static,
    {
        self.compute_pipeline_registry.register(pipeline);
    }

    fn create_pass_target(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label_suffix: &str,
    ) -> wgpu::TextureView {
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
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_compute_pass_target(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        format: TextureFormat,
        label_suffix: &str,
    ) -> wgpu::TextureView {
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
        texture.create_view(&wgpu::TextureViewDescriptor::default())
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

    pub(crate) fn resize_surface(&mut self) {
        if self.size.width > 0 && self.size.height > 0 {
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            self.surface.configure(&self.gpu, &self.config);
            self.rebuild_pass_targets();
        }
    }

    pub(crate) fn rebuild_pass_targets(&mut self) {
        self.pass_a.texture().destroy();
        self.pass_b.texture().destroy();
        self.compute_target_a.texture().destroy();
        self.compute_target_b.texture().destroy();

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
                t.destroy();
            }
            let (msaa_texture, msaa_view) =
                Self::make_msaa_resources(&self.gpu, self.sample_count, &self.config);
            self.msaa_texture = msaa_texture;
            self.msaa_view = msaa_view;
        }
    }

    /// Resize the surface if needed.
    pub(crate) fn resize_if_needed(&mut self) -> bool {
        let result = self.size_changed;
        if self.size_changed {
            self.resize_surface();
            self.size_changed = false;
        }
        result
    }

    // Helper does ping-pong copy and optional compute; returns an owned TextureView to avoid
    // holding mutable borrows on pass targets across the caller scope.
    fn handle_ping_pong_and_compute(
        context: WgpuContext<'_>,
        read_target: &mut wgpu::TextureView,
        write_target: &mut wgpu::TextureView,
        compute_resources: ComputeResources<'_>,
        copy_rect: PxRect,
        blit_bind_group_layout: &wgpu::BindGroupLayout,
        blit_sampler: &wgpu::Sampler,
        blit_pipeline: &wgpu::RenderPipeline,
    ) -> wgpu::TextureView {
        std::mem::swap(read_target, write_target);

        let blit_bind_group = context.gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: blit_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(read_target),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(blit_sampler),
                },
            ],
            label: Some("Blit Bind Group"),
        });

        let mut rpass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: write_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

        rpass.set_pipeline(blit_pipeline);
        rpass.set_bind_group(0, &blit_bind_group, &[]);
        // Set a scissor rect to ensure we only write to the required region.
        rpass.set_scissor_rect(
            copy_rect.x.0.max(0) as u32,
            copy_rect.y.0.max(0) as u32,
            copy_rect.width.0.max(0) as u32,
            copy_rect.height.0.max(0) as u32,
        );
        // Draw a single triangle that covers the whole screen. The scissor rect clips it.
        rpass.draw(0..3, 0..1);

        drop(rpass); // End the blit pass

        // Apply compute commands if any, reusing existing do_compute implementation
        if !compute_resources.compute_commands.is_empty() {
            let compute_commands_taken = std::mem::take(compute_resources.compute_commands);
            Self::do_compute(DoComputeParams {
                encoder: context.encoder,
                commands: compute_commands_taken,
                compute_pipeline_registry: compute_resources.compute_pipeline_registry,
                gpu: context.gpu,
                queue: context.queue,
                config: context.config,
                resource_manager: compute_resources.resource_manager,
                scene_view: read_target,
                target_a: compute_resources.compute_target_a,
                target_b: compute_resources.compute_target_b,
            })
        } else {
            // Return an owned clone so caller does not keep a borrow on read_target
            read_target.clone()
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

        let (read_target, write_target) = (&mut self.pass_a, &mut self.pass_b);

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

        let mut scene_texture_view = read_target.clone();
        let mut commands_in_pass: Vec<DrawOrClip> = Vec::new();
        let mut barrier_draw_rects_in_pass: Vec<PxRect> = Vec::new();
        let mut clip_stack: Vec<PxRect> = Vec::new();

        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        for (command, command_type_id, size, start_pos) in commands {
            let need_new_pass = commands_in_pass
                .iter()
                .rev()
                .find_map(|command| match &command {
                    DrawOrClip::Draw(cmd) => Some(cmd),
                    DrawOrClip::Clip(_) => None,
                })
                .map(|cmd| match (cmd.command.barrier(), command.barrier()) {
                    (None, Some(_)) => true,
                    (Some(_), Some(barrier)) => {
                        let last_draw_rect =
                            extract_draw_rect(Some(barrier), size, start_pos, texture_size);
                        !barrier_draw_rects_in_pass
                            .iter()
                            .all(|dr| dr.is_orthogonal(&last_draw_rect))
                    }
                    (Some(_), None) => false,
                    (None, None) => false,
                })
                .unwrap_or(false);

            if need_new_pass {
                // A ping-pong operation is needed if the first command in the pass has a barrier
                if commands_in_pass
                    .iter()
                    .find_map(|command| match &command {
                        DrawOrClip::Draw(cmd) => Some(cmd),
                        DrawOrClip::Clip(_) => None,
                    })
                    .map(|cmd| cmd.command.barrier().is_some())
                    .unwrap_or(false)
                {
                    let mut combined_rect = barrier_draw_rects_in_pass[0];
                    for rect in barrier_draw_rects_in_pass.iter().skip(1) {
                        combined_rect = combined_rect.union(rect);
                    }

                    let final_view_after_compute = Self::handle_ping_pong_and_compute(
                        WgpuContext {
                            encoder: &mut encoder,
                            gpu: &self.gpu,
                            queue: &self.queue,
                            config: &self.config,
                        },
                        read_target,
                        write_target,
                        ComputeResources {
                            compute_commands: &mut self.compute_commands,
                            compute_pipeline_registry: &mut self.compute_pipeline_registry,
                            resource_manager: &mut self.resource_manager.write(),
                            compute_target_a: &self.compute_target_a,
                            compute_target_b: &self.compute_target_b,
                        },
                        combined_rect,
                        &self.blit_bind_group_layout,
                        &self.blit_sampler,
                        &self.blit_pipeline,
                    );
                    scene_texture_view = final_view_after_compute;
                }

                render_current_pass(RenderCurrentPassParams {
                    msaa_view: &self.msaa_view,
                    is_first_pass: &mut is_first_pass,
                    encoder: &mut encoder,
                    write_target,
                    final_view: None,
                    commands_in_pass: &mut commands_in_pass,
                    scene_texture_view: &scene_texture_view,
                    drawer: &mut self.drawer,
                    gpu: &self.gpu,
                    queue: &self.queue,
                    config: &self.config,
                    clip_stack: &mut clip_stack,
                });
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
                    commands_in_pass.push(DrawOrClip::Draw(DrawCommandWithMetadata {
                        command: cmd,
                        type_id: command_type_id,
                        size,
                        start_pos,
                        draw_rect,
                    }));
                }
                Command::Compute(cmd) => {
                    // Add the compute command to the current pass
                    self.compute_commands.push((cmd, size, start_pos));
                }
                Command::ClipPush(rect) => {
                    // Push it into command stack
                    commands_in_pass.push(DrawOrClip::Clip(ClipOps::Push(rect)));
                }
                Command::ClipPop => {
                    // Push it into command stack
                    commands_in_pass.push(DrawOrClip::Clip(ClipOps::Pop));
                }
            }
        }

        // After processing all commands, we need to render the last pass if there are any commands left
        if !commands_in_pass.is_empty() {
            // A ping-pong operation is needed if the first command in the pass has a barrier
            if commands_in_pass
                .iter()
                .find_map(|command| match &command {
                    DrawOrClip::Draw(cmd) => Some(cmd),
                    DrawOrClip::Clip(_) => None,
                })
                .map(|cmd| cmd.command.barrier().is_some())
                .unwrap_or(false)
            {
                let mut combined_rect = barrier_draw_rects_in_pass[0];
                for rect in barrier_draw_rects_in_pass.iter().skip(1) {
                    combined_rect = combined_rect.union(rect);
                }

                let final_view_after_compute = Self::handle_ping_pong_and_compute(
                    WgpuContext {
                        encoder: &mut encoder,
                        gpu: &self.gpu,
                        queue: &self.queue,
                        config: &self.config,
                    },
                    read_target,
                    write_target,
                    ComputeResources {
                        compute_commands: &mut self.compute_commands,
                        compute_pipeline_registry: &mut self.compute_pipeline_registry,
                        resource_manager: &mut self.resource_manager.write(),
                        compute_target_a: &self.compute_target_a,
                        compute_target_b: &self.compute_target_b,
                    },
                    combined_rect,
                    &self.blit_bind_group_layout,
                    &self.blit_sampler,
                    &self.blit_pipeline,
                );
                scene_texture_view = final_view_after_compute;
            }

            // Render the current pass before starting a new one
            render_current_pass(RenderCurrentPassParams {
                msaa_view: &self.msaa_view,
                is_first_pass: &mut is_first_pass,
                encoder: &mut encoder,
                write_target,
                final_view: Some(&output_view),
                commands_in_pass: &mut commands_in_pass,
                scene_texture_view: &scene_texture_view,
                drawer: &mut self.drawer,
                gpu: &self.gpu,
                queue: &self.queue,
                config: &self.config,
                clip_stack: &mut clip_stack,
            });
            commands_in_pass.clear();
            barrier_draw_rects_in_pass.clear();
        }

        // Frame-level end for all pipelines
        self.drawer
            .pipeline_registry
            .end_all_frames(&self.gpu, &self.queue, &self.config);

        self.queue.submit(Some(encoder.finish()));
        output_frame.present();

        Ok(())
    }

    fn do_compute(params: DoComputeParams<'_>) -> wgpu::TextureView {
        if params.commands.is_empty() {
            return params.scene_view.clone();
        }

        let mut read_view = params.scene_view.clone();
        let (mut write_target, mut read_target) = (params.target_a, params.target_b);

        for (command, size, start_pos) in params.commands {
            // Ensure the write target is cleared before use
            params.encoder.clear_texture(
                write_target.texture(),
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
                let mut cpass = params
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Compute Pass"),
                        timestamp_writes: None,
                    });

                // Get the area of the compute command (reuse extract_draw_rect to avoid duplication)
                let texture_size = wgpu::Extent3d {
                    width: params.config.width,
                    height: params.config.height,
                    depth_or_array_layers: 1,
                };
                let area =
                    extract_draw_rect(Some(command.barrier()), size, start_pos, texture_size);

                params.compute_pipeline_registry.dispatch_erased(
                    params.gpu,
                    params.queue,
                    params.config,
                    &mut cpass,
                    &*command,
                    params.resource_manager,
                    area,
                    &read_view,
                    write_target,
                );
            } // cpass is dropped here, ending the pass

            // The result of this pass is now in write_target.
            // For the next iteration, this will be our read source.
            read_view = write_target.clone();
            // Swap targets for the next iteration
            std::mem::swap(&mut write_target, &mut read_target);
        }

        // After the loop, the final result is in the `read_view`,
        // because we swapped one last time at the end of the loop.
        read_view
    }
}

fn compute_padded_rect(
    size: PxSize,
    start_pos: PxPosition,
    top: Px,
    right: Px,
    bottom: Px,
    left: Px,
    texture_size: wgpu::Extent3d,
) -> PxRect {
    let padded_x = (start_pos.x - left).max(Px(0));
    let padded_y = (start_pos.y - top).max(Px(0));
    let padded_width = (size.width + left + right).min(Px(texture_size.width as i32 - padded_x.0));
    let padded_height =
        (size.height + top + bottom).min(Px(texture_size.height as i32 - padded_y.0));
    PxRect {
        x: padded_x,
        y: padded_y,
        width: padded_width,
        height: padded_height,
    }
}

fn clamp_rect_to_texture(mut rect: PxRect, texture_size: wgpu::Extent3d) -> PxRect {
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
        }) => compute_padded_rect(size, start_pos, top, right, bottom, left, texture_size),
        Some(BarrierRequirement::Absolute(rect)) => clamp_rect_to_texture(rect, texture_size),
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

fn render_current_pass(params: RenderCurrentPassParams<'_>) {
    let destination_view = params.final_view.unwrap_or(params.write_target);

    let (view, resolve_target) = if let Some(msaa_view) = params.msaa_view {
        (msaa_view, Some(destination_view))
    } else {
        (destination_view, None)
    };

    let load_ops = if *params.is_first_pass {
        *params.is_first_pass = false;
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
    } else {
        wgpu::LoadOp::Load
    };

    let mut rpass = params
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
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

    params.drawer.begin_pass(
        params.gpu,
        params.queue,
        params.config,
        &mut rpass,
        params.scene_texture_view,
    );

    // Prepare buffered submission state
    let mut buffer: Vec<(Box<dyn DrawCommand>, PxSize, PxPosition)> = Vec::new();
    let mut last_command_type_id = None;
    let mut current_batch_draw_rect: Option<PxRect> = None;
    for cmd in mem::take(params.commands_in_pass).into_iter() {
        let cmd = match cmd {
            DrawOrClip::Clip(clip_ops) => {
                // Must flush any existing buffered commands before changing clip state
                if !buffer.is_empty() {
                    submit_buffered_commands(
                        &mut rpass,
                        params.drawer,
                        params.gpu,
                        params.queue,
                        params.config,
                        &mut buffer,
                        params.scene_texture_view,
                        params.clip_stack,
                        &mut current_batch_draw_rect,
                    );
                    last_command_type_id = None; // Reset batch type after flush
                }
                // Update clip stack
                match clip_ops {
                    ClipOps::Push(rect) => {
                        params.clip_stack.push(rect);
                    }
                    ClipOps::Pop => {
                        params.clip_stack.pop();
                    }
                }
                // continue to next command
                continue;
            }
            DrawOrClip::Draw(cmd) => cmd, // Proceed with draw commands
        };

        // If the incoming command cannot be merged into the current batch, flush first.
        if !can_merge_into_batch(&last_command_type_id, cmd.type_id) && !buffer.is_empty() {
            submit_buffered_commands(
                &mut rpass,
                params.drawer,
                params.gpu,
                params.queue,
                params.config,
                &mut buffer,
                params.scene_texture_view,
                params.clip_stack,
                &mut current_batch_draw_rect,
            );
        }

        // Add the command to the buffer and update the current batch rect (extracted merge helper).
        buffer.push((cmd.command, cmd.size, cmd.start_pos));
        last_command_type_id = Some(cmd.type_id);
        current_batch_draw_rect = Some(merge_batch_rect(current_batch_draw_rect, cmd.draw_rect));
    }

    // If there are any remaining commands in the buffer, submit them
    if !buffer.is_empty() {
        submit_buffered_commands(
            &mut rpass,
            params.drawer,
            params.gpu,
            params.queue,
            params.config,
            &mut buffer,
            params.scene_texture_view,
            params.clip_stack,
            &mut current_batch_draw_rect,
        );
    }

    params.drawer.end_pass(
        params.gpu,
        params.queue,
        params.config,
        &mut rpass,
        params.scene_texture_view,
    );
}

fn submit_buffered_commands(
    rpass: &mut wgpu::RenderPass<'_>,
    drawer: &mut Drawer,
    gpu: &wgpu::Device,
    queue: &wgpu::Queue,
    config: &wgpu::SurfaceConfiguration,
    buffer: &mut Vec<(Box<dyn DrawCommand>, PxSize, PxPosition)>,
    scene_texture_view: &wgpu::TextureView,
    clip_stack: &mut [PxRect],
    current_batch_draw_rect: &mut Option<PxRect>,
) {
    // Take the buffered commands and convert to the transient representation expected by drawer.submit
    let commands = mem::take(buffer);
    let commands = commands
        .iter()
        .map(|(cmd, sz, pos)| (&**cmd, *sz, *pos))
        .collect::<Vec<_>>();

    // Apply clipping to the current batch rectangle; if nothing remains, abort early.
    let (current_clip_rect, anything_to_submit) =
        apply_clip_to_batch_rect(clip_stack, current_batch_draw_rect);
    if !anything_to_submit {
        return;
    }

    let rect = current_batch_draw_rect.unwrap();
    set_scissor_rect_from_pxrect(rpass, rect);

    drawer.submit(
        gpu,
        queue,
        config,
        rpass,
        &commands,
        scene_texture_view,
        current_clip_rect,
    );
    *current_batch_draw_rect = None;
}

fn set_scissor_rect_from_pxrect(rpass: &mut wgpu::RenderPass<'_>, rect: PxRect) {
    rpass.set_scissor_rect(
        rect.x.positive(),
        rect.y.positive(),
        rect.width.positive(),
        rect.height.positive(),
    );
}

/// Apply clip_stack to current_batch_draw_rect. Returns false if intersection yields nothing
/// (meaning there is nothing to submit), true otherwise.
///
/// Also returns the current clipping rectangle (if any) for potential use by the caller.
fn apply_clip_to_batch_rect(
    clip_stack: &[PxRect],
    current_batch_draw_rect: &mut Option<PxRect>,
) -> (Option<PxRect>, bool) {
    if let Some(clipped_rect) = clip_stack.last() {
        let Some(current_rect) = current_batch_draw_rect.as_ref() else {
            return (Some(*clipped_rect), false);
        };
        if let Some(final_rect) = current_rect.intersection(clipped_rect) {
            *current_batch_draw_rect = Some(final_rect);
            return (Some(*clipped_rect), true);
        }
        return (Some(*clipped_rect), false);
    }
    (None, true)
}

/// Determine whether `next_type_id` (with potential clipping) can be merged into the current batch.
/// Equivalent to the negation of the original flush condition:
/// merge allowed when last_command_type_id == Some(next_type_id) or last_command_type_id is None.
fn can_merge_into_batch(last_command_type_id: &Option<TypeId>, next_type_id: TypeId) -> bool {
    match last_command_type_id {
        Some(l) => *l == next_type_id,
        None => true,
    }
}

/// Merge the existing optional batch rect with a new command rect.
fn merge_batch_rect(current: Option<PxRect>, next: PxRect) -> PxRect {
    current.map(|dr| dr.union(&next)).unwrap_or(next)
}

struct DrawCommandWithMetadata {
    command: Box<dyn DrawCommand>,
    type_id: TypeId,
    size: PxSize,
    start_pos: PxPosition,
    draw_rect: PxRect,
}

enum DrawOrClip {
    Draw(DrawCommandWithMetadata),
    Clip(ClipOps),
}

enum ClipOps {
    Push(PxRect),
    Pop,
}
