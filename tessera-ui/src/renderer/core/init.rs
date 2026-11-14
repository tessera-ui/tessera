use std::sync::Arc;

use tracing::{error, info, warn};
use wgpu::TextureFormat;
use winit::window::Window;

use crate::{
    pipeline_cache::initialize_cache,
    render::{ComputePipelineRegistry, drawer::Drawer},
    renderer::window_drawable_size,
};

use super::{BlitState, FrameTargets, RenderCore, RenderPipelines};

impl RenderCore {
    fn pick_alpha_mode(
        caps: &wgpu::SurfaceCapabilities,
        window_transparent: bool,
    ) -> wgpu::CompositeAlphaMode {
        if window_transparent {
            if caps
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
            {
                return wgpu::CompositeAlphaMode::PreMultiplied;
            }
            if caps
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
            {
                return wgpu::CompositeAlphaMode::PostMultiplied;
            }
            if caps
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::Inherit)
            {
                return wgpu::CompositeAlphaMode::Inherit;
            }
        } else if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
            return wgpu::CompositeAlphaMode::Opaque;
        }
        caps.alpha_modes
            .first()
            .copied()
            .unwrap_or(wgpu::CompositeAlphaMode::Opaque)
    }

    async fn request_adapter_for_surface(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface<'_>,
    ) -> wgpu::Adapter {
        match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::from_env().unwrap_or_default(),
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
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

    /// Requests an adapter for offscreen rendering without a compatible
    /// surface.
    async fn request_adapter_offscreen(instance: &wgpu::Instance) -> wgpu::Adapter {
        match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::from_env()
                    .unwrap_or(wgpu::PowerPreference::HighPerformance),
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
        {
            Ok(adapter) => adapter,
            Err(e) => {
                error!("Failed to find an offscreen adapter: {e:?}");
                panic!("Failed to find an offscreen adapter: {e:?}");
            }
        }
    }

    async fn request_device_and_queue_for_adapter(
        adapter: &wgpu::Adapter,
        max_bindings: u32,
    ) -> (wgpu::Device, wgpu::Queue) {
        let mut required_limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };
        // D3D12 reserves the entire descriptor heap when the device is created.
        required_limits.max_non_sampler_bindings =
            max_bindings.min(adapter.limits().max_non_sampler_bindings);
        required_limits.max_uniform_buffer_binding_size = required_limits
            .max_uniform_buffer_binding_size
            .min(adapter.limits().max_uniform_buffer_binding_size);
        required_limits.max_inter_stage_shader_variables = required_limits
            .max_inter_stage_shader_variables
            .min(adapter.limits().max_inter_stage_shader_variables);
        let base_features = wgpu::Features::CLEAR_TEXTURE;
        #[cfg(all(feature = "profiling", not(target_arch = "wasm32")))]
        let base_features = if super::gpu_timing::GpuTimingProbe::requested() {
            let features = super::gpu_timing::GpuTimingProbe::FEATURES;
            if adapter.features().contains(features) {
                base_features | features
            } else {
                tracing::warn!("GPU timing is unsupported by this adapter");
                base_features
            }
        } else {
            base_features
        };
        let supports_pipeline_cache = adapter.features().contains(wgpu::Features::PIPELINE_CACHE);
        let requested_features = if supports_pipeline_cache {
            base_features | wgpu::Features::PIPELINE_CACHE
        } else {
            info!("Adapter does not support PIPELINE_CACHE; continuing without it");
            base_features
        };

        let make_descriptor = |required_features| wgpu::DeviceDescriptor {
            required_features,
            required_limits: required_limits.clone(),
            label: None,
            // Fixed blocks keep upload heaps from growing after transient staging.
            // They stay small because retained target sizes already make
            // full-window reallocations rare; larger blocks reserve memory at idle
            // and stay fragmented after resizes.
            memory_hints: wgpu::MemoryHints::Manual {
                suballocated_device_memory_block_size: 8 << 20..8 << 20,
            },
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::default(),
        };

        match adapter
            .request_device(&make_descriptor(requested_features))
            .await
        {
            Ok((device, queue)) => (device, queue),
            Err(first_err) if requested_features.contains(wgpu::Features::PIPELINE_CACHE) => {
                warn!(
                    "Failed to create device with PIPELINE_CACHE enabled: {first_err:?}; retrying without PIPELINE_CACHE"
                );
                match adapter
                    .request_device(&make_descriptor(base_features))
                    .await
                {
                    Ok((device, queue)) => (device, queue),
                    Err(second_err) => {
                        error!("Failed to create device: {second_err:?}");
                        panic!("Failed to create device: {second_err:?}");
                    }
                }
            }
            Err(err) => {
                error!("Failed to create device: {err:?}");
                panic!("Failed to create device: {err:?}");
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

    pub(crate) async fn new(
        window: Arc<Window>,
        sample_count: u32,
        window_transparent: bool,
        max_bindings: u32,
    ) -> Self {
        let size = window_drawable_size(&window);
        let scale_factor = window.scale_factor();
        Self::new_inner(
            Some(window),
            size,
            scale_factor,
            None,
            sample_count,
            window_transparent,
            max_bindings,
        )
        .await
    }

    #[cfg(all(feature = "headless", not(target_family = "wasm")))]
    pub(crate) async fn new_offscreen(
        size: winit::dpi::PhysicalSize<u32>,
        format: TextureFormat,
        sample_count: u32,
        max_bindings: u32,
    ) -> Self {
        Self::new_inner(
            None,
            size,
            1.0,
            Some(format),
            sample_count,
            false,
            max_bindings,
        )
        .await
    }

    async fn new_inner(
        window: Option<Arc<Window>>,
        size: winit::dpi::PhysicalSize<u32>,
        scale_factor: f64,
        offscreen_format: Option<TextureFormat>,
        sample_count: u32,
        window_transparent: bool,
        max_bindings: u32,
    ) -> Self {
        let mut instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
        instance_desc.flags = wgpu::InstanceFlags::from_env_or_default();
        #[cfg(not(target_os = "windows"))]
        {
            instance_desc.backends = wgpu::Backends::all();
        }
        #[cfg(target_os = "windows")]
        {
            instance_desc.backends = wgpu::Backends::DX12;
            if window.is_some() {
                instance_desc.backend_options.dx12.presentation_system =
                    wgpu::Dx12SwapchainKind::DxgiFromVisual;
            }
        }
        instance_desc.backends = wgpu::Backends::from_env().unwrap_or(instance_desc.backends);
        info!("Using WGPU instance config: {instance_desc:#?}");
        let instance: wgpu::Instance = wgpu::Instance::new(instance_desc);
        let surface = match &window {
            Some(window) => match instance.create_surface(window.clone()) {
                Ok(surface) => Some(surface),
                Err(e) => {
                    error!("Failed to create surface: {e:?}");
                    panic!("Failed to create surface: {e:?}");
                }
            },
            None => None,
        };
        let adapter = match &surface {
            Some(surface) => Self::request_adapter_for_surface(&instance, surface).await,
            None => Self::request_adapter_offscreen(&instance).await,
        };
        let adapter_info = adapter.get_info();
        info!("Using WGPU adapter: {adapter_info:#?}");
        let (device, queue) =
            Self::request_device_and_queue_for_adapter(&adapter, max_bindings).await;
        // Offscreen cores have no surface capabilities to query.
        let config = match &surface {
            Some(surface) => {
                let caps = surface.get_capabilities(&adapter);
                let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
                    wgpu::PresentMode::Fifo
                } else {
                    wgpu::PresentMode::Immediate
                };
                let alpha_mode = Self::pick_alpha_mode(&caps, window_transparent);
                info!("Using present mode: {present_mode:?}");
                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: caps.formats[0],
                    color_space: wgpu::SurfaceColorSpace::Auto,
                    width: size.width.max(1),
                    height: size.height.max(1),
                    present_mode,
                    alpha_mode,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2,
                };
                surface.configure(&device, &config);
                config
            }
            None => wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: offscreen_format.unwrap_or(TextureFormat::Rgba8UnormSrgb),
                color_space: wgpu::SurfaceColorSpace::Auto,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
        };
        let mut draw_config = config.clone();
        // sRGB attachments preserve low tones while GPU blending stays linear.
        draw_config.format = match config.format {
            TextureFormat::Rgba8Unorm => TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Bgra8Unorm => TextureFormat::Bgra8UnormSrgb,
            format => format,
        };
        if draw_config.format != config.format {
            let features = adapter.get_texture_format_features(draw_config.format);
            let usage = wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST;
            assert!(
                features.allowed_usages.contains(usage),
                "Scene format {:?} does not support required drawing usages",
                draw_config.format
            );
            assert!(
                features.flags.sample_count_supported(sample_count),
                "Scene format {:?} does not support {sample_count} samples",
                draw_config.format
            );
        }
        info!(
            "Presentation format: {:?}; scene format: {:?}; alpha mode: {:?}; surface: {}",
            config.format,
            draw_config.format,
            config.alpha_mode,
            surface.is_some(),
        );

        // Create pipeline cache if supported
        let pipeline_cache = initialize_cache(&device, &adapter_info);

        // Create MSAA Target
        let (msaa_texture, msaa_view) =
            Self::make_msaa_resources(&device, sample_count, &draw_config);

        let offscreen_texture = Self::create_pass_target(&device, &draw_config, "Offscreen");
        let offscreen_copy_texture = surface
            .is_none()
            .then(|| Self::create_pass_target(&device, &config, "Offscreen Copy"));

        let drawer = Drawer::new();

        // Set scale factor for dp conversion
        info!("Render core scale factor: {scale_factor}");
        crate::dp::set_scale_factor(scale_factor);

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
            bind_group_layouts: &[Some(&blit_bind_group_layout)],
            immediate_size: 0,
        });

        let create_blit_pipeline = |label, format, entry_point| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&blit_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &blit_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &blit_shader,
                    entry_point: Some(entry_point),
                    targets: &[Some(wgpu::ColorTargetState::from(format))],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: pipeline_cache.as_ref(),
            })
        };
        let blit_pipeline_rgba = create_blit_pipeline(
            "Blit Pipeline Rgba8",
            wgpu::TextureFormat::Rgba8Unorm,
            "fs_main",
        );
        let clear_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Damage Clear Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: Some("fs_clear"),
                targets: &[Some(wgpu::ColorTargetState::from(draw_config.format))],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            multiview_mask: None,
            cache: pipeline_cache.as_ref(),
        });
        let present_pipeline = create_blit_pipeline(
            "Present Pipeline",
            config.format,
            if config.format.is_srgb() {
                "fs_main"
            } else {
                "fs_main_srgb"
            },
        );
        #[cfg(feature = "debug-dirty-overlay")]
        let dirty_overlay_shader =
            device.create_shader_module(wgpu::include_wgsl!("../shaders/dirty_overlay.wgsl"));
        #[cfg(feature = "debug-dirty-overlay")]
        let dirty_overlay_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Dirty Overlay Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        #[cfg(feature = "debug-dirty-overlay")]
        let dirty_overlay_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Dirty Overlay Pipeline"),
                layout: Some(&dirty_overlay_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &dirty_overlay_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &dirty_overlay_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: pipeline_cache.as_ref(),
            });

        let pipelines = RenderPipelines {
            clipper: super::clip::ShapeClipper::new(&device, draw_config.format, sample_count),
            drawer,
            compute_registry: ComputePipelineRegistry::new(),
        };

        let targets = FrameTargets {
            offscreen: offscreen_texture,
            offscreen_copy: offscreen_copy_texture,
            msaa_texture,
            msaa_view,
            stencil: Self::create_stencil(&device, &draw_config, sample_count),
            sample_count,
        };

        let blit = BlitState {
            clear_pipeline,
            present_pipeline,
            pipeline_rgba: blit_pipeline_rgba,
            bind_group_layout: blit_bind_group_layout,
            sampler: blit_sampler,
            #[cfg(feature = "debug-dirty-overlay")]
            dirty_overlay_pipeline,
        };

        #[cfg(all(feature = "profiling", not(target_arch = "wasm32")))]
        let gpu_timing = (super::gpu_timing::GpuTimingProbe::requested()
            && device
                .features()
                .contains(super::gpu_timing::GpuTimingProbe::FEATURES))
        .then(|| super::gpu_timing::GpuTimingProbe::new(&device, &queue));

        Self {
            window,
            graphics: crate::render::GraphicsDevice::new(device.clone(), queue.clone()),
            instance,
            device,
            surface,
            queue,
            config,
            draw_config,
            size,
            size_changed: false,
            pipelines,
            pipeline_factories: Default::default(),
            pipeline_cache,
            adapter_info,
            targets,
            blit,
            frame_index: 0,
            previous_graph: None,
            last_render_breakdown: None,
            #[cfg(all(feature = "profiling", not(target_arch = "wasm32")))]
            gpu_timing,
        }
    }

    pub(super) fn create_pass_target(
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

    fn create_stencil(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("UI stencil"),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Stencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
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

    /// Applies the pending surface/target resize.
    ///
    /// When a swapchain surface is present it is reconfigured; offscreen cores
    /// only rebuild their render targets.
    pub(crate) fn resize_surface(&mut self) {
        self.apply_resize(true);
    }

    fn apply_resize(&mut self, force: bool) {
        if self.size.width > 0 && self.size.height > 0 {
            let limit = self.device.limits().max_texture_dimension_2d;
            // DirectComposition clips the unscaled visual to the HWND client area.
            // Other presentation systems can stretch it and need exact extents.
            let retain_surface = cfg!(target_os = "windows")
                && self.surface.is_some()
                && self.adapter_info.backend == wgpu::Backend::Dx12;
            let width = if retain_surface {
                retained_dimension(self.config.width, self.size.width, limit)
            } else {
                self.size.width
            };
            let height = if retain_surface {
                retained_dimension(self.config.height, self.size.height, limit)
            } else {
                self.size.height
            };
            let configure = force || self.config.width != width || self.config.height != height;
            self.config.width = width;
            self.config.height = height;
            if let Some(surface) = &self.surface {
                if configure {
                    surface.configure(&self.device, &self.config);
                }
            }
            self.rebuild_pass_targets(force);
        }
    }

    pub(crate) fn recreate_surface(&mut self) {
        let Some(window) = self.window.clone() else {
            // Offscreen cores have no surface to recreate.
            return;
        };
        let surface = match self.instance.create_surface(window) {
            Ok(surface) => surface,
            Err(err) => {
                error!("Failed to recreate surface: {err:?}");
                return;
            }
        };
        self.surface = Some(surface);
        self.resize_surface();
    }

    fn rebuild_pass_targets(&mut self, force: bool) {
        self.previous_graph = None;
        self.draw_config.width = self.size.width;
        self.draw_config.height = self.size.height;
        let limit = self.device.limits().max_texture_dimension_2d;
        let current = self.targets.offscreen.texture().size();
        let mut allocation = self.draw_config.clone();
        allocation.width = retained_dimension(current.width, self.size.width, limit);
        allocation.height = retained_dimension(current.height, self.size.height, limit);
        if self.surface.is_none()
            && (force
                || self.targets.offscreen_copy.as_ref().is_none_or(|view| {
                    view.texture().width() != self.size.width
                        || view.texture().height() != self.size.height
                }))
        {
            if let Some(view) = self.targets.offscreen_copy.take() {
                view.texture().destroy();
            }
            self.targets.offscreen_copy = Some(Self::create_pass_target(
                &self.device,
                &self.config,
                "Offscreen Copy",
            ));
        }
        if !force && current.width == allocation.width && current.height == allocation.height {
            return;
        }
        self.targets.offscreen.texture().destroy();
        self.targets.stencil.texture().destroy();

        self.targets.offscreen = Self::create_pass_target(&self.device, &allocation, "Offscreen");
        self.targets.stencil =
            Self::create_stencil(&self.device, &allocation, self.targets.sample_count);

        if self.targets.sample_count > 1 {
            if let Some(t) = self.targets.msaa_texture.take() {
                t.destroy();
            }
            let (msaa_texture, msaa_view) =
                Self::make_msaa_resources(&self.device, self.targets.sample_count, &allocation);
            self.targets.msaa_texture = msaa_texture;
            self.targets.msaa_view = msaa_view;
        }
    }

    /// Resize the surface if needed.
    pub(crate) fn resize_if_needed(&mut self) {
        if self.size_changed {
            self.apply_resize(false);
            self.size_changed = false;
        }
    }
}

fn retained_dimension(current: u32, requested: u32, limit: u32) -> u32 {
    const STEP: u32 = 128;
    let rounded = requested.div_ceil(STEP).saturating_mul(STEP).min(limit);
    if current >= requested
        && current <= rounded.saturating_add(STEP)
        && current <= requested.saturating_mul(2).max(STEP)
    {
        current
    } else {
        rounded
    }
}

#[cfg(test)]
mod tests {
    use super::retained_dimension;

    #[test]
    fn resize_capacity_bounds_slack_and_avoids_bucket_boundary_oscillation() {
        assert_eq!(retained_dimension(256, 257, 16384), 384);
        assert_eq!(retained_dimension(384, 255, 16384), 384);
        assert_eq!(retained_dimension(384, 127, 16384), 128);
        assert_eq!(retained_dimension(384, 257, 16384), 384);
        assert_eq!(retained_dimension(384, 999, 1000), 1000);
        for requested in [1, 127, 128, 129, 1000, 4096, 16383, 16384] {
            for current in [1, 128, 256, 4096, 16384] {
                let capacity = retained_dimension(current, requested, 16384);
                assert!(capacity >= requested);
                assert!(capacity <= 16384);
                assert!(capacity - requested < 256);
            }
        }
    }
}
