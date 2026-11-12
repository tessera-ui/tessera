use encase::{ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    render::{DrawContext, DrawablePipeline},
    wgpu,
};

use crate::pipelines::fluid_glass::FluidGlassCommand;

#[derive(ShaderType, Clone, PartialEq, Copy, Debug, Default)]
struct GlassUniforms {
    tint_color: Vec4,
    rect_uv_bounds: Vec4,
    corner_radii: Vec4,
    corner_g2: Vec4,
    material_tone: Vec4,
    blur_sigma_px: f32,
    pixels_per_dp: f32,
    opacity: f32,
    material_mode: u32,
    scene_uv: Vec4,
    scene_size_px: Vec2,
    rect_size_px: Vec2,
    ripple_center: Vec2,
    shape_type: f32,
    noise_amount: f32,
    noise_scale: f32,
    time: f32,
    ripple_radius: f32,
    ripple_alpha: f32,
    ripple_strength: f32,
    border_width: f32,
    contrast: f32,
    feedback: f32,
}

#[derive(PartialEq, ShaderType)]
struct GlassInstances {
    #[shader(size(runtime))]
    instances: Vec<GlassUniforms>,
}

// --- Pipeline Definition ---

pub(crate) struct FluidGlassPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl FluidGlassPipeline {
    /// Builds the fluid glass pipeline.
    pub fn new(
        gpu: &wgpu::Device,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let shader = gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fluid Glass Shader"),
            source: wgpu::ShaderSource::Wgsl(
                concat!(
                    include_str!("sdf_common.wgsl"),
                    "\n",
                    include_str!("glass.wgsl")
                )
                .into(),
            ),
        });

        let sampler = Self::create_sampler(gpu);
        let bind_group_layout = Self::create_bind_group_layout(gpu);
        let pipeline = Self::create_render_pipeline(
            gpu,
            config,
            pipeline_cache,
            sample_count,
            &shader,
            &bind_group_layout,
        );
        Self {
            pipeline,
            bind_group_layout,
            sampler,
        }
    }

    /// Create the sampler used by the pipeline.
    fn create_sampler(gpu: &wgpu::Device) -> wgpu::Sampler {
        gpu.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        })
    }

    /// Create the bind group layout for instance buffer + scene texture +
    /// sampler.
    fn create_bind_group_layout(gpu: &wgpu::Device) -> wgpu::BindGroupLayout {
        gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        })
    }

    /// Create the full render pipeline used for drawing the fluid glass quads.
    fn create_render_pipeline(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
        shader: &wgpu::ShaderModule,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluid Glass Pipeline Layout"),
            bind_group_layouts: &[Some(bind_group_layout)],
            immediate_size: 0,
        });

        gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fluid Glass Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
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
            depth_stencil: tessera_ui::render::ui_stencil_state(),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: pipeline_cache,
        })
    }
}

impl DrawablePipeline<FluidGlassCommand> for FluidGlassPipeline {
    fn draw(&mut self, context: &mut DrawContext<FluidGlassCommand>) {
        if context.commands.is_empty() {
            return;
        }
        let instances = GlassInstances {
            instances: context
                .commands
                .iter()
                .map(|(command, geometry)| glass_uniforms(command, *geometry, &*context))
                .collect(),
        };
        let mut content = StorageBuffer::new(Vec::<u8>::new());
        content
            .write(&instances)
            .expect("fluid glass instance serialization failed");
        let buffer = context
            .graphics
            .frame_buffer(wgpu::BufferUsages::STORAGE, content.as_ref());
        let bind_group = context
            .graphics
            .cached_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(context.scene_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
                label: Some("fluid_glass_bind_group"),
            });
        context.render_pass.set_pipeline(&self.pipeline);
        context.render_pass.set_bind_group(0, &bind_group, &[]);
        context
            .render_pass
            .draw(0..6, 0..instances.instances.len() as u32);
    }
}

fn glass_uniforms(
    command: &FluidGlassCommand,
    geometry: tessera_ui::render::DrawGeometry,
    context: &DrawContext<FluidGlassCommand>,
) -> GlassUniforms {
    let render = &command.render;
    let size = geometry.local_size;
    let start_pos = geometry.window_origin();
    let [world_width, world_height] = geometry.window_size();
    let screen_w = context.target_size.width.to_f32();
    let screen_h = context.target_size.height.to_f32();

    let scene = context.scene_rect;
    let texture = context.scene_texture_view.texture();
    let (texture_w, texture_h) = (texture.width() as f32, texture.height() as f32);
    let scene_uv = Vec4::new(
        screen_w / texture_w,
        screen_h / texture_h,
        -scene.x.to_f32() / texture_w,
        -scene.y.to_f32() / texture_h,
    );

    let (corner_radii, corner_g2, shape_type) = match render.shape.resolve_for_size(size) {
        crate::shape_def::ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => (
            corner_radii.into(),
            corner_g2.into(),
            if corner_radii == [0.0; 4] { 2.0 } else { 0.0 },
        ),
        crate::shape_def::ResolvedShape::Ellipse => (Vec4::ZERO, Vec4::ZERO, 1.0),
    };

    GlassUniforms {
        tint_color: render.tint_color.to_array().into(),
        rect_uv_bounds: Vec4::new(
            start_pos.x / screen_w,
            start_pos.y / screen_h,
            (start_pos.x + world_width) / screen_w,
            (start_pos.y + world_height) / screen_h,
        ),
        material_tone: render.material_tone.unwrap_or_default().into(),
        blur_sigma_px: render.blur_sigma_px,
        pixels_per_dp: render.pixels_per_dp,
        opacity: render.opacity,
        material_mode: match (render.material_tone.is_some(), render.dark) {
            (false, _) => 0,
            (true, false) => 1,
            (true, true) => 2,
        },
        scene_uv,
        scene_size_px: Vec2::new(scene.width.to_f32(), scene.height.to_f32()),
        rect_size_px: [size.width.0 as f32, size.height.0 as f32].into(),
        ripple_center: render.ripple_center.unwrap_or([0.0, 0.0]).into(),
        corner_radii,
        corner_g2,
        shape_type,
        noise_amount: render.noise_amount,
        noise_scale: render.noise_scale,
        time: render.time,
        ripple_radius: render.ripple_radius.unwrap_or(0.0),
        ripple_alpha: render.ripple_alpha.unwrap_or(0.0),
        ripple_strength: render.ripple_strength.unwrap_or(0.0),
        contrast: render.contrast.unwrap_or(1.0),
        feedback: render.feedback,
        border_width: render
            .border
            .as_ref()
            .map_or(0.0, |border| border.width.0 as f32),
    }
}
