use bytemuck::{Pod, Zeroable};
use tessera::{
    PxPosition, PxSize,
    renderer::DrawablePipeline,
    wgpu::{self, util::DeviceExt},
};

use crate::fluid_glass::FluidGlassCommand;

// --- Uniforms ---

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct GlassUniforms {
    // Grouped by alignment to match WGSL std140 layout rules.
    // vec4s
    tint_color: [f32; 4],
    highlight_color: [f32; 4],
    inner_shadow_color: [f32; 4],
    rect_uv_bounds: [f32; 4],

    // vec2s
    rect_size_px: [f32; 2],
    ripple_center: [f32; 2],

    // f32s
    corner_radius: f32,
    shape_type: f32,
    g2_k_value: f32,
    dispersion_height: f32,
    chroma_multiplier: f32,
    refraction_height: f32,
    refraction_amount: f32,
    eccentric_factor: f32,
    highlight_size: f32,
    highlight_smoothing: f32,
    inner_shadow_radius: f32,
    inner_shadow_smoothing: f32,
    noise_amount: f32,
    noise_scale: f32,
    time: f32,
    ripple_radius: f32,
    ripple_alpha: f32,
    ripple_strength: f32,

    // Padding to ensure the struct is aligned to 16 bytes.
    // Total size: 4*16 (vec4s) + 2*8 (vec2s) + 18*4 (f32s) = 64 + 16 + 72 = 152 bytes.
    // Padded to 160 bytes.
    _padding: [f32; 2],
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
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &FluidGlassCommand,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: &wgpu::TextureView,
    ) {
        let args = &command.args;
        let screen_w = config.width as f32;
        let screen_h = config.height as f32;

        let rect_uv_bounds = [
            start_pos.x.0 as f32 / screen_w,
            start_pos.y.0 as f32 / screen_h,
            (start_pos.x.0 + size.width.0) as f32 / screen_w,
            (start_pos.y.0 + size.height.0) as f32 / screen_h,
        ];

        let uniforms = GlassUniforms {
            // vec4s
            tint_color: args.tint_color.into(),
            highlight_color: args.highlight_color.into(),
            inner_shadow_color: args.inner_shadow_color.into(),
            rect_uv_bounds,

            // vec2s
            rect_size_px: [size.width.0 as f32, size.height.0 as f32],
            ripple_center: args.ripple_center.unwrap_or([0.0, 0.0]),

            // f32s
            corner_radius: match args.shape {
                crate::shape_def::Shape::RoundedRectangle { corner_radius } => corner_radius,
                crate::shape_def::Shape::Ellipse => 0.0,
            },
            shape_type: match args.shape {
                crate::shape_def::Shape::RoundedRectangle { .. } => 0.0,
                crate::shape_def::Shape::Ellipse => 1.0,
            },
            g2_k_value: args.g2_k_value,
            dispersion_height: args.dispersion_height,
            chroma_multiplier: args.chroma_multiplier,
            refraction_height: args.refraction_height,
            refraction_amount: args.refraction_amount,
            eccentric_factor: args.eccentric_factor,
            highlight_size: args.highlight_size,
            highlight_smoothing: args.highlight_smoothing,
            inner_shadow_radius: args.inner_shadow_radius,
            inner_shadow_smoothing: args.inner_shadow_smoothing,
            noise_amount: args.noise_amount,
            noise_scale: args.noise_scale,
            time: args.time,
            ripple_radius: args.ripple_radius.unwrap_or(0.0),
            ripple_alpha: args.ripple_alpha.unwrap_or(0.0),
            ripple_strength: args.ripple_strength.unwrap_or(0.0),

            _padding: [0.0; 2],
        };

        let uniform_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Temporary Fluid Glass Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

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
        render_pass.draw(0..6, 0..1);
    }
}
