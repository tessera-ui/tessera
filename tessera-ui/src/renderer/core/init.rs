use std::sync::Arc;

use parking_lot::RwLock;
use tracing::{error, info};
use wgpu::TextureFormat;
use winit::window::Window;

use crate::{
    CompositePipelineRegistry,
    compute::resource::ComputeResourceManager,
    dp::SCALE_FACTOR,
    pipeline_cache::initialize_cache,
    renderer::{compute::ComputePipelineRegistry, drawer::Drawer},
};

use super::{BlitState, ComputeState, FrameTargets, LocalTexturePool, RenderCore, RenderPipelines};

impl RenderCore {
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
            Ok(adapter) => adapter,
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
                required_features: wgpu::Features::empty()
                    | wgpu::Features::CLEAR_TEXTURE
                    | wgpu::Features::PIPELINE_CACHE,
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::default(),
            })
            .await
        {
            Ok((device, queue)) => (device, queue),
            Err(e) => {
                error!("Failed to create device: {e:?}");
                panic!("Failed to create device: {e:?}");
            }
        }
    }

    fn make_msaa_resources(
        device: &wgpu::Device,
        sample_count: u32,
        config: &wgpu::SurfaceConfiguration,
    ) -> (Option<wgpu::Texture>, Option<wgpu::TextureView>) {
        if sample_count > 1 {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
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

    /// Create a new render core as the root of Tessera.
    pub(crate) async fn new(window: Arc<Window>, sample_count: u32) -> Self {
        // Looking for adapters
        let instance: wgpu::Instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            /* Currently the renderer's design only supports VULKAN.
             * Given VULKAN's broad compatibility, this does not affect cross-platform support
             * for now.
             *
             * TODO: Refactor the renderer to support additional backends.
             */
            backends: wgpu::Backends::VULKAN,
            // backends: wgpu::Backends::all(),
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
        // Looking for a compatible adapter
        let adapter = Self::request_adapter_for_surface(&instance, &surface).await;
        // Create a device and queue
        let (device, queue) = Self::request_device_and_queue_for_adapter(&adapter).await;
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create pipeline cache if supported
        let pipeline_cache = initialize_cache(&device, &adapter.get_info());

        // Create MSAA Target
        let (msaa_texture, msaa_view) = Self::make_msaa_resources(&device, sample_count, &config);

        // Create Pass Targets (Offscreen and Compute)
        let offscreen_texture = Self::create_pass_target(&device, &config, "Offscreen");
        let compute_target_a = Self::create_compute_pass_target(
            &device,
            &config,
            TextureFormat::Rgba8Unorm,
            "Compute A",
        );
        let compute_target_b = Self::create_compute_pass_target(
            &device,
            &config,
            TextureFormat::Rgba8Unorm,
            "Compute B",
        );

        let drawer = Drawer::new();

        // Set scale factor for dp conversion
        let scale_factor = window.scale_factor();
        info!("Window scale factor: {scale_factor}");
        let _ = SCALE_FACTOR.set(RwLock::new(scale_factor));

        // Create blit pipeline resources
        let blit_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/blit.wgsl"));
        let blit_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let blit_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&blit_bind_group_layout],
            immediate_size: 0,
        });

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            multiview_mask: None,
            cache: pipeline_cache.as_ref(),
        });
        let blit_pipeline_rgba = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline Rgba8"),
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
                targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: pipeline_cache.as_ref(),
        });

        let pipelines = RenderPipelines {
            drawer,
            compute_registry: ComputePipelineRegistry::new(),
            composite_registry: CompositePipelineRegistry::new(),
        };

        let targets = FrameTargets {
            offscreen: offscreen_texture,
            msaa_texture,
            msaa_view,
            sample_count,
        };

        let compute = ComputeState {
            target_a: compute_target_a,
            target_b: compute_target_b,
            resource_manager: Arc::new(RwLock::new(ComputeResourceManager::new())),
        };

        let blit = BlitState {
            pipeline: blit_pipeline,
            pipeline_rgba: blit_pipeline_rgba,
            bind_group_layout: blit_bind_group_layout,
            sampler: blit_sampler,
        };

        Self {
            window,
            device,
            surface,
            queue,
            config,
            size,
            size_changed: false,
            pipelines,
            pipeline_cache,
            adapter_info: adapter.get_info(),
            targets,
            compute,
            blit,
            local_textures: LocalTexturePool::new(),
            frame_index: 0,
        }
    }

    fn create_pass_target(
        device: &wgpu::Device,
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
        let texture = device.create_texture(&texture_descriptor);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_compute_pass_target(
        device: &wgpu::Device,
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
        let texture = device.create_texture(&texture_descriptor);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
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
            self.surface.configure(&self.device, &self.config);
            self.rebuild_pass_targets();
        }
    }

    pub(crate) fn rebuild_pass_targets(&mut self) {
        self.local_textures.clear();
        self.targets.offscreen.texture().destroy();
        self.compute.target_a.texture().destroy();
        self.compute.target_b.texture().destroy();

        self.targets.offscreen = Self::create_pass_target(&self.device, &self.config, "Offscreen");
        self.compute.target_a = Self::create_compute_pass_target(
            &self.device,
            &self.config,
            TextureFormat::Rgba8Unorm,
            "Compute A",
        );
        self.compute.target_b = Self::create_compute_pass_target(
            &self.device,
            &self.config,
            TextureFormat::Rgba8Unorm,
            "Compute B",
        );

        if self.targets.sample_count > 1 {
            if let Some(t) = self.targets.msaa_texture.take() {
                t.destroy();
            }
            let (msaa_texture, msaa_view) =
                Self::make_msaa_resources(&self.device, self.targets.sample_count, &self.config);
            self.targets.msaa_texture = msaa_texture;
            self.targets.msaa_view = msaa_view;
        }
    }

    /// Resize the surface if needed.
    pub(crate) fn resize_if_needed(&mut self) {
        if self.size_changed {
            self.resize_surface();
            self.size_changed = false;
        }
    }
}
