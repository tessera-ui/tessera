use bytemuck::{Pod, Zeroable};
use image::GenericImageView;
use std::{iter, mem, sync::Arc, time::Instant};
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const RECT_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5],
        uv: [0.0, 0.0],
    },
];

const RECT_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct GlassUniforms {
    // Vector values
    bleed_color: [f32; 4],
    highlight_color: [f32; 4],
    inner_shadow_color: [f32; 4],

    // vec2 types
    rect_size_px: [f32; 2],

    // f32 types
    corner_radius: f32,
    dispersion_height: f32,
    chroma_multiplier: f32,
    refraction_height: f32,
    refraction_amount: f32,
    eccentric_factor: f32,
    bleed_amount: f32,
    highlight_size: f32,
    highlight_smoothing: f32,
    inner_shadow_radius: f32,
    inner_shadow_smoothing: f32,
    noise_amount: f32,
    noise_scale: f32,
    time: f32,
}

struct FluidGlassState<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    #[allow(dead_code)]
    window: Arc<Window>,
    start_time: Instant,

    // Background resources
    bg_render_pipeline: wgpu::RenderPipeline,
    bg_bind_group: wgpu::BindGroup,

    // Glass effect resources
    glass_render_pipeline: wgpu::RenderPipeline,
    glass_bind_group_layout: wgpu::BindGroupLayout,
    glass_bind_group: wgpu::BindGroup,

    // Shared vertex/index buffers
    rect_vertex_buffer: wgpu::Buffer,
    rect_index_buffer: wgpu::Buffer,
    num_indices: u32,

    // Uniform buffer
    glass_uniforms: GlassUniforms,
    glass_uniform_buffer: wgpu::Buffer,

    // Background copy texture
    bg_copy_texture: wgpu::Texture,
    bg_copy_texture_view: wgpu::TextureView,

    // Shared sampler
    sampler: wgpu::Sampler,

    // Adjustable parameters
    dispersion_height: f32,
    chroma_multiplier: f32,
    refraction_height: f32,
    refraction_amount: f32,
    eccentric_factor: f32,
    corner_radius: f32,
    bleed_color: [f32; 4],
    highlight_color: [f32; 4],
    inner_shadow_color: [f32; 4],
    bleed_amount: f32,
    highlight_size: f32,
    highlight_smoothing: f32,
    inner_shadow_radius: f32,
    inner_shadow_smoothing: f32,
    noise_amount: f32,
    noise_scale: f32,
}

impl FluidGlassState<'_> {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let start_time = Instant::now();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&*window).unwrap())
        }
        .unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create background resources
        let bg_image = image::load_from_memory(include_bytes!("assets/background.png")).unwrap();
        let bg_rgba = bg_image.to_rgba8();
        let dimensions = bg_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let bg_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("background_texture"),
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &bg_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bg_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );
        let bg_texture_view = bg_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bg_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let bg_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
                label: Some("bg_bind_group_layout"),
            });
        let bg_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bg_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bg_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&bg_sampler),
                },
            ],
            label: Some("bg_bind_group"),
        });

        let bg_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/background.wgsl"));
        let bg_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("BG Pipeline Layout"),
            bind_group_layouts: &[&bg_bind_group_layout],
            push_constant_ranges: &[],
        });
        let bg_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Background Render Pipeline"),
            layout: Some(&bg_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &bg_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &bg_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(config.format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create vertex/index buffers
        let rect_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rect Vertex Buffer"),
            contents: bytemuck::cast_slice(RECT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let rect_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rect Index Buffer"),
            contents: bytemuck::cast_slice(RECT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = RECT_INDICES.len() as u32;

        // Create background copy texture
        let rect_pixel_width = (size.width as f32 * 0.5) as u32;
        let rect_pixel_height = (size.height as f32 * 0.5) as u32;
        let rect_pixel_size = wgpu::Extent3d {
            width: rect_pixel_width.max(1),
            height: rect_pixel_height.max(1),
            depth_or_array_layers: 1,
        };

        let (bg_copy_texture, bg_copy_texture_view) =
            Self::create_copy_texture(&device, &config, rect_pixel_size);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        // Initialize parameters - a mix of Android and new defaults
        // Based on user feedback, the default parameters have been adjusted.
        // The primary glass effect (refraction) is preserved, while other
        // effects like bleed, highlight, and inner shadow are disabled by default.
        // A subtle noise is added for a slightly frosted look.
        let dispersion_height = 0.0;
        let chroma_multiplier = 1.0;
        let refraction_height = 20.0;
        let refraction_amount = -60.0;
        let eccentric_factor = 1.0;
        let corner_radius = 30.0;
        let bleed_color = [1.0, 1.0, 1.0, 0.0]; // Disabled
        let highlight_color = [1.0, 1.0, 1.0, 0.0]; // Disabled
        let inner_shadow_color = [0.0, 0.0, 0.0, 0.0]; // Disabled
        let bleed_amount = 0.0; // Disabled
        let highlight_size = 0.0; // Disabled
        let highlight_smoothing = 8.0;
        let inner_shadow_radius = 0.0; // Disabled
        let inner_shadow_smoothing = 2.0;
        let noise_amount = 0.02;
        let noise_scale = 1.5;

        // Initialize uniforms
        let glass_uniforms = GlassUniforms {
            rect_size_px: [rect_pixel_width as f32, rect_pixel_height as f32],
            corner_radius,
            dispersion_height,
            chroma_multiplier,
            refraction_height,
            refraction_amount,
            eccentric_factor,
            bleed_color,
            highlight_color,
            inner_shadow_color,
            bleed_amount,
            highlight_size,
            highlight_smoothing,
            inner_shadow_radius,
            inner_shadow_smoothing,
            noise_amount,
            noise_scale,
            time: 0.0,
        };

        // Create uniform buffer
        let glass_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glass Uniform Buffer"),
            contents: bytemuck::cast_slice(&[glass_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create glass effect bind group layout
        let glass_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("glass_bind_group_layout"),
            });

        let glass_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &glass_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: glass_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&bg_copy_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("glass_bind_group"),
        });

        // Create glass render pipeline
        let glass_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/glass.wgsl"));
        let glass_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Glass Pipeline Layout"),
                bind_group_layouts: &[&glass_bind_group_layout],
                push_constant_ranges: &[],
            });
        let glass_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Glass Render Pipeline"),
                layout: Some(&glass_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &glass_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &glass_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            start_time,
            bg_render_pipeline,
            bg_bind_group,
            glass_render_pipeline,
            glass_bind_group_layout,
            glass_bind_group,
            rect_vertex_buffer,
            rect_index_buffer,
            num_indices,
            glass_uniforms,
            glass_uniform_buffer,
            bg_copy_texture,
            bg_copy_texture_view,
            sampler,
            dispersion_height,
            chroma_multiplier,
            refraction_height,
            refraction_amount,
            eccentric_factor,
            corner_radius,
            bleed_color,
            highlight_color,
            inner_shadow_color,
            bleed_amount,
            highlight_size,
            highlight_smoothing,
            inner_shadow_radius,
            inner_shadow_smoothing,
            noise_amount,
            noise_scale,
        }
    }

    fn create_copy_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        size: wgpu::Extent3d,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("copy_texture"),
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    fn update_uniforms(&mut self) {
        // Update uniform values
        self.glass_uniforms.dispersion_height = self.dispersion_height;
        self.glass_uniforms.chroma_multiplier = self.chroma_multiplier;
        self.glass_uniforms.refraction_height = self.refraction_height;
        self.glass_uniforms.refraction_amount = self.refraction_amount;
        self.glass_uniforms.eccentric_factor = self.eccentric_factor;
        self.glass_uniforms.corner_radius = self.corner_radius;
        self.glass_uniforms.bleed_color = self.bleed_color;
        self.glass_uniforms.highlight_color = self.highlight_color;
        self.glass_uniforms.inner_shadow_color = self.inner_shadow_color;
        self.glass_uniforms.bleed_amount = self.bleed_amount;
        self.glass_uniforms.highlight_size = self.highlight_size;
        self.glass_uniforms.highlight_smoothing = self.highlight_smoothing;
        self.glass_uniforms.inner_shadow_radius = self.inner_shadow_radius;
        self.glass_uniforms.inner_shadow_smoothing = self.inner_shadow_smoothing;
        self.glass_uniforms.noise_amount = self.noise_amount;
        self.glass_uniforms.noise_scale = self.noise_scale;
        self.glass_uniforms.time = self.start_time.elapsed().as_secs_f32();

        self.queue.write_buffer(
            &self.glass_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.glass_uniforms]),
        );
    }

    fn handle_input(&mut self, event: &WindowEvent) -> bool {
        let mut changed = true;
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                ..
            } => {
                match key {
                    KeyCode::KeyQ => self.dispersion_height += 5.0,
                    KeyCode::KeyA => {
                        self.dispersion_height = (self.dispersion_height - 5.0).max(0.0)
                    }
                    KeyCode::KeyW => self.chroma_multiplier += 0.2,
                    KeyCode::KeyS => {
                        self.chroma_multiplier = (self.chroma_multiplier - 0.2).max(0.0)
                    }
                    KeyCode::KeyE => self.refraction_height += 5.0,
                    KeyCode::KeyD => {
                        self.refraction_height = (self.refraction_height - 5.0).max(0.0)
                    }
                    KeyCode::KeyR => self.refraction_amount += 10.0,
                    KeyCode::KeyF => self.refraction_amount -= 10.0,
                    KeyCode::KeyT => self.eccentric_factor = (self.eccentric_factor + 0.1).min(2.0),
                    KeyCode::KeyG => self.eccentric_factor = (self.eccentric_factor - 0.1).max(0.0),
                    KeyCode::KeyY => self.corner_radius += 5.0,
                    KeyCode::KeyH => self.corner_radius = (self.corner_radius - 5.0).max(0.0),

                    // New Controls
                    KeyCode::KeyU => self.bleed_amount = (self.bleed_amount + 0.05).min(1.0),
                    KeyCode::KeyJ => self.bleed_amount = (self.bleed_amount - 0.05).max(0.0),
                    KeyCode::KeyI => self.highlight_size = (self.highlight_size + 0.05).min(1.0),
                    KeyCode::KeyK => self.highlight_size = (self.highlight_size - 0.05).max(0.0),
                    KeyCode::KeyO => self.inner_shadow_radius += 2.0,
                    KeyCode::KeyL => {
                        self.inner_shadow_radius = (self.inner_shadow_radius - 2.0).max(0.0)
                    }
                    KeyCode::KeyZ => self.noise_amount = (self.noise_amount + 0.01).min(0.5),
                    KeyCode::KeyX => self.noise_amount = (self.noise_amount - 0.01).max(0.0),

                    // Toggle Bleed Color
                    KeyCode::Digit1 => {
                        self.bleed_color = if self.bleed_color[3] > 0.0 {
                            [1.0, 1.0, 1.0, 0.0]
                        } else {
                            [0.8, 0.1, 0.2, 0.5]
                        }
                    }
                    // Toggle Highlight
                    KeyCode::Digit2 => {
                        self.highlight_color[3] = if self.highlight_color[3] > 0.0 {
                            0.0
                        } else {
                            0.2
                        }
                    }
                    // Toggle Inner Shadow
                    KeyCode::Digit3 => {
                        self.inner_shadow_color[3] = if self.inner_shadow_color[3] > 0.0 {
                            0.0
                        } else {
                            0.5
                        }
                    }

                    KeyCode::KeyP => {
                        println!("\n--- Current Parameters (Fluid Glass) ---");
                        println!("  Dispersion Height: {:.2}", self.dispersion_height);
                        println!("  Chroma Multiplier: {:.2}", self.chroma_multiplier);
                        println!("  Refraction Height: {:.2}", self.refraction_height);
                        println!("  Refraction Amount: {:.2}", self.refraction_amount);
                        println!("  Eccentric Factor:  {:.2}", self.eccentric_factor);
                        println!("  Corner Radius:     {:.2}", self.corner_radius);
                        println!("--- New Effects ---");
                        println!(
                            "  Bleed Amount: {:.2} (Color: {:.2?})",
                            self.bleed_amount, self.bleed_color
                        );
                        println!(
                            "  Highlight Size: {:.2} (Alpha: {:.2})",
                            self.highlight_size, self.highlight_color[3]
                        );
                        println!("  Highlight Smoothing: {:.2}", self.highlight_smoothing);
                        println!(
                            "  Inner Shadow Radius: {:.2} (Alpha: {:.2})",
                            self.inner_shadow_radius, self.inner_shadow_color[3]
                        );
                        println!(
                            "  Inner Shadow Smoothing: {:.2}",
                            self.inner_shadow_smoothing
                        );
                        println!("  Noise Amount: {:.2}", self.noise_amount);
                        println!("--------------------------------------");
                    }
                    _ => changed = false,
                }
            }
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size);
            }
            _ => changed = false,
        }
        changed
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Recreate background copy texture with new size
            let rect_pixel_width = (new_size.width as f32 * 0.5) as u32;
            let rect_pixel_height = (new_size.height as f32 * 0.5) as u32;
            let rect_pixel_size = wgpu::Extent3d {
                width: rect_pixel_width.max(1),
                height: rect_pixel_height.max(1),
                depth_or_array_layers: 1,
            };

            let (bg_copy_texture, bg_copy_texture_view) =
                Self::create_copy_texture(&self.device, &self.config, rect_pixel_size);
            self.bg_copy_texture = bg_copy_texture;
            self.bg_copy_texture_view = bg_copy_texture_view;

            // Update uniform buffer sizes
            self.glass_uniforms.rect_size_px = [rect_pixel_width as f32, rect_pixel_height as f32];

            // Recreate bind group with new texture
            self.glass_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.glass_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.glass_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&self.bg_copy_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: Some("glass_bind_group"),
            });
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Update uniforms before rendering
        self.update_uniforms();

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // 1. Render background to main surface
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Background Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.bg_render_pipeline);
            render_pass.set_bind_group(0, &self.bg_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }

        // 2. Copy background section to bg_copy_texture
        let rect_pixel_width = self.glass_uniforms.rect_size_px[0] as u32;
        let rect_pixel_height = self.glass_uniforms.rect_size_px[1] as u32;
        let copy_x = (self.size.width - rect_pixel_width) / 2;
        let copy_y = (self.size.height - rect_pixel_height) / 2;

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &output.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: copy_x,
                    y: copy_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.bg_copy_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: rect_pixel_width,
                height: rect_pixel_height,
                depth_or_array_layers: 1,
            },
        );

        // 3. Render glass effect over the background
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Glass Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.glass_render_pipeline);
            render_pass.set_bind_group(0, &self.glass_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.rect_vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(self.rect_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
struct App<'a> {
    state: Option<FluidGlassState<'a>>,
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Fluid Glass Demo - Enhanced")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
        let w = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(w.clone());
        self.state = Some(pollster::block_on(FluidGlassState::new(w)));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if self.window.is_some() && window_id == self.window.as_ref().unwrap().id() {
            let state = self.state.as_mut().unwrap();
            if !state.handle_input(&event) {
                match event {
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::RedrawRequested => match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    },
                    _ => {}
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

pub fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    println!("--- Controls (Fluid Glass) ---");
    println!("  Q/A: Adjust dispersion height");
    println!("  W/S: Adjust chroma multiplier");
    println!("  E/D: Adjust refraction height");
    println!("  R/F: Adjust refraction amount");
    println!("  T/G: Adjust eccentric factor");
    println!("  Y/H: Adjust corner radius");
    println!("--- New Effect Controls ---");
    println!("  U/J: Adjust bleed amount");
    println!("  I/K: Adjust highlight size");
    println!("  O/L: Adjust inner shadow radius");
    println!("  Z/X: Adjust noise amount");
    println!("  1: Toggle bleed effect (Red)");
    println!("  2: Toggle highlight effect");
    println!("  3: Toggle inner shadow effect");
    println!("--- General ---");
    println!("  P: Print current parameters");

    let mut app = App {
        state: None,
        window: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
