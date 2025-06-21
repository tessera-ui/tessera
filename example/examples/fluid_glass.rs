use std::{iter, mem};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use image::GenericImageView;
use winit::{
    event::*,
    event_loop::{EventLoop},
    window::{Window, WindowBuilder},
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
    Vertex { position: [-0.5, -0.5], uv: [0.0, 1.0] },
    Vertex { position: [0.5, -0.5], uv: [1.0, 1.0] },
    Vertex { position: [0.5, 0.5], uv: [1.0, 0.0] },
    Vertex { position: [-0.5, 0.5], uv: [0.0, 0.0] },
];

const RECT_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct GlassUniforms {
    rect_size_px: [f32; 2],
    corner_radius: f32,
    refraction_height: f32,
    refraction_amount: f32,
    eccentric_factor: f32,
    dispersion_height: f32,
    chroma_multiplier: f32,
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,

    bg_render_pipeline: wgpu::RenderPipeline,
    bg_bind_group: wgpu::BindGroup,

    rect_vertex_buffer: wgpu::Buffer,
    rect_index_buffer: wgpu::Buffer,
    num_indices: u32,

    bg_copy_texture: wgpu::Texture,
    bg_copy_texture_view: wgpu::TextureView,
    
    glass_uniforms: GlassUniforms,
    glass_uniform_buffer: wgpu::Buffer,
    glass_render_pipeline: wgpu::RenderPipeline,
    glass_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    glass_bind_group: wgpu::BindGroup,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
        
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

        // --- Background Resources ---
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
            wgpu::ImageCopyTexture {
                texture: &bg_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bg_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );
        let bg_texture_view = bg_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bg_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let bg_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&bg_texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&bg_sampler) },
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
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &bg_shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });


        // --- Glass Resources ---
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

        let rect_pixel_width = (size.width as f32 * 0.5) as u32;
        let rect_pixel_height = (size.height as f32 * 0.5) as u32;
        let rect_pixel_size = wgpu::Extent3d {
            width: rect_pixel_width.max(1),
            height: rect_pixel_height.max(1),
            depth_or_array_layers: 1,
        };
        let (bg_copy_texture, bg_copy_texture_view) = Self::create_copy_texture(&device, &config, rect_pixel_size);

        let glass_uniforms = GlassUniforms {
            rect_size_px: [rect_pixel_width as f32, rect_pixel_height as f32],
            corner_radius: 30.0,
            refraction_height: 20.0,
            refraction_amount: -60.0,
            eccentric_factor: 1.0,
            dispersion_height: 0.0,
            chroma_multiplier: 1.0,
        };
        let glass_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glass Uniform Buffer"),
            contents: bytemuck::cast_slice(&[glass_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let glass_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("glass_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
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
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let glass_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &glass_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: glass_uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&bg_copy_texture_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
            label: Some("glass_bind_group"),
        });
        
        let glass_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/glass.wgsl"));
        let glass_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Glass Pipeline Layout"),
            bind_group_layouts: &[&glass_bind_group_layout],
            push_constant_ranges: &[],
        });
        let glass_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Glass Render Pipeline"),
            layout: Some(&glass_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &glass_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &glass_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            window, surface, device, queue, config, size,
            bg_render_pipeline, bg_bind_group,
            rect_vertex_buffer, rect_index_buffer, num_indices,
            bg_copy_texture, bg_copy_texture_view,
            glass_uniforms, glass_uniform_buffer,
            glass_render_pipeline, glass_bind_group_layout, sampler, glass_bind_group,
        }
    }

    fn create_copy_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, texture_size: wgpu::Extent3d) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("BG Copy Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            let rect_pixel_width = (new_size.width as f32 * 0.5) as u32;
            let rect_pixel_height = (new_size.height as f32 * 0.5) as u32;
            let rect_pixel_size = wgpu::Extent3d {
                width: rect_pixel_width.max(1),
                height: rect_pixel_height.max(1),
                depth_or_array_layers: 1,
            };
            
            let (new_texture, new_view) = Self::create_copy_texture(&self.device, &self.config, rect_pixel_size);
            self.bg_copy_texture = new_texture;
            self.bg_copy_texture_view = new_view;
            
            self.glass_uniforms.rect_size_px = [rect_pixel_width as f32, rect_pixel_height as f32];
            self.queue.write_buffer(&self.glass_uniform_buffer, 0, bytemuck::cast_slice(&[self.glass_uniforms]));

            self.glass_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.glass_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.glass_uniform_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&self.bg_copy_texture_view) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                ],
                label: Some("glass_bind_group (resized)"),
            });
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // 1. Render background
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Background Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.bg_render_pipeline);
            render_pass.set_bind_group(0, &self.bg_bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }

        // 2. Copy the background area under the rect
        let rect_pixel_width = (self.size.width as f32 * 0.5) as u32;
        let rect_pixel_height = (self.size.height as f32 * 0.5) as u32;
        let copy_x = (self.size.width - rect_pixel_width) / 2;
        let copy_y = (self.size.height - rect_pixel_height) / 2;
        
        if rect_pixel_width > 0 && rect_pixel_height > 0 {
            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: &output.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: copy_x, y: copy_y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                self.bg_copy_texture.as_image_copy(),
                wgpu::Extent3d { width: rect_pixel_width, height: rect_pixel_height, depth_or_array_layers: 1 },
            );
        }

        // 3. Render the glass rect
        {
             let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Glass Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.glass_render_pipeline);
            render_pass.set_bind_group(0, &self.glass_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.rect_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.rect_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

pub fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Fluid Glass Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop).unwrap();
    
    let mut state = pollster::block_on(State::new(&window));
    
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::KeyboardInput {
                        event: KeyEvent { logical_key: key, state: ElementState::Pressed, .. }, ..
                    } => {
                        if key == &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) {
                            elwt.exit();
                        }
                    }
                    WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                    WindowEvent::RedrawRequested => {
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                            Err(e) => eprintln!("Render error: {:?}", e),
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                state.window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
