use std::sync::Arc;

use log::{error, info};
use parking_lot::RwLock;
use winit::window::Window;

use crate::{ Px, PxPosition, dp::SCALE_FACTOR};

use super::drawer::{DrawCommand, Drawer, RenderRequirement};

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
    /// Texture for the background pass
    background_texture: wgpu::Texture,
    background_texture_view: wgpu::TextureView,
    pub background_bind_group_layout: wgpu::BindGroupLayout,
    pub background_bind_group: wgpu::BindGroup,
}

impl WgpuApp {
    /// Create a new WGPU app, as the root of Tessera
    pub(crate) async fn new(
        window: Arc<Window>,
    ) -> Self {
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
        let present_mode = /*if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            // Mailbox is the best choice for most cases, it allows for low latency and high FPS
            wgpu::PresentMode::FifoRelaxed
        } else */if caps.present_modes.contains(&wgpu::PresentMode::Fifo) {
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
        // Create drawer
        // Create background texture and bind group
        let background_texture_descriptor = wgpu::TextureDescriptor {
            label: Some("Background Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let background_texture = gpu.create_texture(&background_texture_descriptor);
        let background_texture_view = background_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let background_bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Background Bind Group Layout"),
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
            ],
        });

        let background_bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Background Bind Group"),
            layout: &background_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&background_texture_view),
                },
            ],
        });

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
            background_texture,
            background_texture_view,
            background_bind_group_layout,
            background_bind_group,
        }
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

    pub(crate) fn resize_background_texture_if_needed(&mut self) {
        if self.size_changed {
            self.background_texture.destroy();
            let background_texture_descriptor = wgpu::TextureDescriptor {
                label: Some("Background Texture"),
                size: wgpu::Extent3d {
                    width: self.config.width,
                    height: self.config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            };
            self.background_texture = self.gpu.create_texture(&background_texture_descriptor);
            self.background_texture_view = self
                .background_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.background_bind_group = self.gpu.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Background Bind Group"),
                layout: &self.background_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.background_texture_view),
                }],
            });
        }
    }

    /// Resize the surface if needed
    pub(crate) fn resize_if_needed(&mut self) {
        if self.size_changed {
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            self.resize_background_texture_if_needed();
            self.surface.configure(&self.gpu, &self.config);
            self.size_changed = false;
        }
    }

    /// Render the surface
    pub(crate) fn render(
        &mut self,
        drawer_commands: impl IntoIterator<Item = (PxPosition, [Px; 2], Box<dyn DrawCommand>)>,
    ) -> Result<(), wgpu::SurfaceError> {
        let commands: Vec<_> = drawer_commands.into_iter().collect();

        // 1. Scan for requirements
        let needs_multi_pass = commands.iter().any(|(_, _, cmd)| {
            cmd.requirement() == RenderRequirement::SamplesBackground
        });

        let output = self.surface.get_current_texture()?;
        let final_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // 2. Choose render path
        if !needs_multi_pass {
            // --- FAST PATH (Single Pass) ---
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Single Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &final_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            self.drawer
                .begin_pass(&self.gpu, &self.queue, &self.config, &mut render_pass);
            for (pos, size, command) in commands {
                self.drawer.submit(
                    &self.gpu,
                    &self.queue,
                    &self.config,
                    &mut render_pass,
                    &*command,
                    size,
                    pos,
                );
            }
            self.drawer
                .end_pass(&self.gpu, &self.queue, &self.config, &mut render_pass);
        } else {
            // --- DYNAMIC MULTI-PASS PATH ---

            // --- Pass 1: Render all Standard commands to background_texture ---
            {
                let mut pass1 = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Background Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.background_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });

                self.drawer
                    .begin_pass(&self.gpu, &self.queue, &self.config, &mut pass1);
                for (pos, size, command) in commands.iter().filter(|c| {
                    c.2.requirement() == RenderRequirement::Standard
                }) {
                    self.drawer.submit(
                        &self.gpu,
                        &self.queue,
                        &self.config,
                        &mut pass1,
                        &**command,
                        *size,
                        *pos,
                    );
                }
                self.drawer
                    .end_pass(&self.gpu, &self.queue, &self.config, &mut pass1);
            }

            // --- Pass 2: Render all SamplesBackground commands to final screen, sampling background_texture ---
            {
                // Copy background to final output first, so we have a base to draw on
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.background_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &output.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: self.config.width,
                        height: self.config.height,
                        depth_or_array_layers: 1,
                    },
                );

                let mut pass2 = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Overlay Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &final_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });

                pass2.set_bind_group(1, &self.background_bind_group, &[]);

                self.drawer
                    .begin_pass(&self.gpu, &self.queue, &self.config, &mut pass2);
                for (pos, size, command) in commands.iter().filter(|c| {
                    c.2.requirement() == RenderRequirement::SamplesBackground
                }) {
                    self.drawer.submit(
                        &self.gpu,
                        &self.queue,
                        &self.config,
                        &mut pass2,
                        &**command,
                        *size,
                        *pos,
                    );
                }
                self.drawer
                    .end_pass(&self.gpu, &self.queue, &self.config, &mut pass2);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}
