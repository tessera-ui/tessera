use bytemuck::{Pod, Zeroable};
use tessera::{
    PxPosition, PxSize,
    renderer::{
        DrawablePipeline,
        compute::{ComputePipelineRegistry, ComputablePipeline},
    },
    wgpu::{self, util::DeviceExt},
};

use super::blur;
use crate::fluid_glass::FluidGlassCommand;

// --- Uniforms ---

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct GlassUniforms {
    // Vector values
    bleed_color: [f32; 4],
    highlight_color: [f32; 4],
    inner_shadow_color: [f32; 4],
    rect_uv_bounds: [f32; 4], // x_min, y_min, x_max, y_max

    // vec2 types
    rect_size_px: [f32; 2],

    // f32 types
    corner_radius: f32,
    g2_k_value: f32,
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
    _padding: [f32; 3], // Struct needs to be aligned to 16 bytes. (33 data f32s + 3 padding f32s = 36 total * 4 bytes/f32 = 144 bytes)
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
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass,
        command: &FluidGlassCommand,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: Option<&wgpu::TextureView>,
        compute_registry: &mut ComputePipelineRegistry,
    ) {
        let Some(original_scene_texture) = scene_texture_view else {
            return;
        };

        let args = &command.args;

        // This will own the blurred texture/view if created, ensuring it lives long enough.
        let blur_storage: Option<(wgpu::Texture, wgpu::TextureView)>;

        let scene_texture = if args.blur_radius > 0.0 {
            // 1. Create intermediate texture for two-pass blur.
            let texture_descriptor = wgpu::TextureDescriptor {
                label: Some("Blur Intermediate Texture"),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: config.format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            };
            let intermediate_texture = gpu.create_texture(&texture_descriptor);
            let intermediate_view =
                intermediate_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // 2. Create final blur destination texture.
            let final_texture = gpu.create_texture(&texture_descriptor);
            let final_view = final_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // 3. Get the blur pipeline from the registry.
            let blur_pipeline = compute_registry.get_sync::<blur::pipeline::BlurPipeline>();

            // 4. First pass: horizontal blur from original to intermediate
            let horizontal_blur_command = blur::command::BlurCommand {
                source_view: original_scene_texture,
                dest_view: &intermediate_view,
                radius: args.blur_radius,
                direction: (1.0, 0.0), // Horizontal
                size: (config.width, config.height),
            };
            blur_pipeline.dispatch_sync(gpu, queue, &horizontal_blur_command);

            // 5. Second pass: vertical blur from intermediate to final
            let vertical_blur_command = blur::command::BlurCommand {
                source_view: &intermediate_view,
                dest_view: &final_view,
                radius: args.blur_radius,
                direction: (0.0, 1.0), // Vertical
                size: (config.width, config.height),
            };
            blur_pipeline.dispatch_sync(gpu, queue, &vertical_blur_command);

            // 6. Store both textures to ensure they live long enough
            // We only return a reference to the final result
            blur_storage = Some((final_texture, final_view));
            &blur_storage.as_ref().unwrap().1
        } else {
            original_scene_texture
        };

        // Per the user's request, we calculate the rectangle's bounds in UV space.
        let screen_w = config.width as f32;
        let screen_h = config.height as f32;

        let rect_uv_bounds = [
            start_pos.x.0 as f32 / screen_w,
            start_pos.y.0 as f32 / screen_h,
            (start_pos.x.0 + size.width.0) as f32 / screen_w,
            (start_pos.y.0 + size.height.0) as f32 / screen_h,
        ];

        let uniforms = GlassUniforms {
            bleed_color: args.bleed_color,
            highlight_color: args.highlight_color,
            inner_shadow_color: args.inner_shadow_color,
            rect_uv_bounds,
            rect_size_px: [size.width.0 as f32, size.height.0 as f32],
            corner_radius: args.corner_radius,
            g2_k_value: args.g2_k_value,
            dispersion_height: args.dispersion_height,
            chroma_multiplier: args.chroma_multiplier,
            refraction_height: args.refraction_height,
            refraction_amount: args.refraction_amount,
            eccentric_factor: args.eccentric_factor,
            bleed_amount: args.bleed_amount,
            highlight_size: args.highlight_size,
            highlight_smoothing: args.highlight_smoothing,
            inner_shadow_radius: args.inner_shadow_radius,
            inner_shadow_smoothing: args.inner_shadow_smoothing,
            noise_amount: args.noise_amount,
            noise_scale: args.noise_scale,
            time: args.time,
            _padding: [0.0; 3],
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
                    resource: wgpu::BindingResource::TextureView(scene_texture),
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
