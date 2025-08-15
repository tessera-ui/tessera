use encase::{ArrayLength, ShaderSize, ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{PxPosition, PxSize, renderer::DrawablePipeline, wgpu};

use crate::fluid_glass::FluidGlassCommand;

// Define MAX_CONCURRENT_SHAPES, can be adjusted later
pub const MAX_CONCURRENT_GLASSES: wgpu::BufferAddress = 256;

// --- Uniforms ---

#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct GlassUniforms {
    tint_color: Vec4,
    rect_uv_bounds: Vec4,
    corner_radii: Vec4,
    rect_size_px: Vec2,
    ripple_center: Vec2,
    shape_type: f32,
    g2_k_value: f32,
    dispersion_height: f32,
    chroma_multiplier: f32,
    refraction_height: f32,
    refraction_amount: f32,
    eccentric_factor: f32,
    noise_amount: f32,
    noise_scale: f32,
    time: f32,
    ripple_radius: f32,
    ripple_alpha: f32,
    ripple_strength: f32,
    border_width: f32,
    screen_size: Vec2,  // Screen dimensions
    light_source: Vec2, // Light source position in world coordinates
    light_scale: f32,   // Light intensity scale factor
}

#[derive(ShaderType)]
struct GlassInstances {
    length: ArrayLength,
    #[size(runtime)]
    instances: Vec<GlassUniforms>,
}

// --- Pipeline Definition ---

pub(crate) struct FluidGlassPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl FluidGlassPipeline {
    pub fn new(gpu: &wgpu::Device, config: &wgpu::SurfaceConfiguration, sample_count: u32) -> Self {
        let shader = gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fluid Glass Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("fluid_glass/glass.wgsl").into()),
        });

        let sampler = gpu.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
            label: Some("fluid_glass_bind_group_layout"),
        });

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluid Glass Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fluid Glass Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
        }
    }
}

impl DrawablePipeline<FluidGlassCommand> for FluidGlassPipeline {
    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        commands: &[(&FluidGlassCommand, PxSize, PxPosition)],
        scene_texture_view: &wgpu::TextureView,
    ) {
        if commands.is_empty() {
            return;
        }

        let mut instances: Vec<GlassUniforms> = commands
            .iter()
            .map(|(command, size, start_pos)| {
                let args = &command.args;
                let screen_w = config.width as f32;
                let screen_h = config.height as f32;

                let rect_uv_bounds = [
                    start_pos.x.0 as f32 / screen_w,
                    start_pos.y.0 as f32 / screen_h,
                    (start_pos.x.0 + size.width.0) as f32 / screen_w,
                    (start_pos.y.0 + size.height.0) as f32 / screen_h,
                ];

                GlassUniforms {
                    tint_color: args.tint_color.to_array().into(),
                    rect_uv_bounds: rect_uv_bounds.into(),
                    rect_size_px: [size.width.0 as f32, size.height.0 as f32].into(),
                    ripple_center: args.ripple_center.unwrap_or([0.0, 0.0]).into(),
                    corner_radii: match args.shape {
                        crate::shape_def::Shape::RoundedRectangle {
                            top_left,
                            top_right,
                            bottom_right,
                            bottom_left,
                            ..
                        } => [top_left, top_right, bottom_right, bottom_left].into(),
                        crate::shape_def::Shape::Ellipse => Vec4::ZERO,
                    },
                    shape_type: match args.shape {
                        crate::shape_def::Shape::RoundedRectangle { .. } => 0.0,
                        crate::shape_def::Shape::Ellipse => 1.0,
                    },
                    g2_k_value: match args.shape {
                        crate::shape_def::Shape::RoundedRectangle { g2_k_value, .. } => g2_k_value,
                        crate::shape_def::Shape::Ellipse => 0.0,
                    },
                    dispersion_height: args.dispersion_height,
                    chroma_multiplier: args.chroma_multiplier,
                    refraction_height: args.refraction_height,
                    refraction_amount: args.refraction_amount,
                    eccentric_factor: args.eccentric_factor,
                    noise_amount: args.noise_amount,
                    noise_scale: args.noise_scale,
                    time: args.time,
                    ripple_radius: args.ripple_radius.unwrap_or(0.0),
                    ripple_alpha: args.ripple_alpha.unwrap_or(0.0),
                    ripple_strength: args.ripple_strength.unwrap_or(0.0),
                    border_width: if let Some(border) = args.border {
                        border.width.0 as f32
                    } else {
                        0.0
                    },
                    screen_size: [screen_w, screen_h].into(),
                    light_source: [screen_w * 0.1, screen_h * 0.1].into(),
                    light_scale: 1.0,
                }
            })
            .collect();

        if instances.len() > MAX_CONCURRENT_GLASSES as usize {
            instances.truncate(MAX_CONCURRENT_GLASSES as usize);
        }

        if instances.is_empty() {
            return;
        }

        let instance_count = instances.len();

        let uniform_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fluid Glass Storage Buffer"),
            size: 16 + GlassUniforms::SHADER_SIZE.get() * instances.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms = GlassInstances {
            length: Default::default(),
            instances,
        };

        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content.write(&uniforms).unwrap();
        queue.write_buffer(&uniform_buffer, 0, buffer_content.as_ref());

        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(scene_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("fluid_glass_bind_group"),
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..6, 0..instance_count as u32);
    }
}
