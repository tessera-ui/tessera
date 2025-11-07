use encase::{ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{PxPosition, PxSize, px::PxRect, renderer::DrawablePipeline, wgpu};

use crate::fluid_glass::FluidGlassCommand;

// Define MAX_CONCURRENT_SHAPES, can be adjusted later
pub const MAX_CONCURRENT_GLASSES: usize = 256;

#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct GlassUniforms {
    tint_color: Vec4,
    rect_uv_bounds: Vec4,
    corner_radii: Vec4,
    clip_rect_uv: Vec4,
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
    /// Construct a new FluidGlassPipeline.
    /// This constructor delegates sampler, bind group layout and pipeline construction
    /// to small helpers to keep the top-level function short and easier to reason about.
    pub fn new(gpu: &wgpu::Device, config: &wgpu::SurfaceConfiguration, sample_count: u32) -> Self {
        let shader = gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fluid Glass Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("fluid_glass/glass.wgsl").into()),
        });

        let sampler = Self::create_sampler(gpu);
        let bind_group_layout = Self::create_bind_group_layout(gpu);
        let pipeline =
            Self::create_render_pipeline(gpu, config, sample_count, &shader, &bind_group_layout);

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
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    /// Create the bind group layout for instance buffer + scene texture + sampler.
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
        sample_count: u32,
        shader: &wgpu::ShaderModule,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluid Glass Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
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
        clip_rect: Option<PxRect>,
    ) {
        // Prepare GPU resources (instances, buffer, bind group) in a single helper to keep
        // the draw path compact and easy to reason about.
        let (uniform_buffer, bind_group, instance_count) = match self.prepare_draw_resources(
            gpu,
            queue,
            config,
            commands,
            scene_texture_view,
            clip_rect,
        ) {
            Some(tuple) => tuple,
            None => return, // Nothing to draw or upload failed.
        };

        // Issue draw call.
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..6, 0..instance_count);

        // Keep buffer alive for duration of draw call.
        drop(uniform_buffer);
    }
}

impl FluidGlassPipeline {
    /// Small helper: build a single GlassUniforms for one command.
    fn build_instance(
        command: &FluidGlassCommand,
        size: &PxSize,
        start_pos: &PxPosition,
        config: &wgpu::SurfaceConfiguration,
        clip_rect: Option<PxRect>,
    ) -> GlassUniforms {
        let args = &command.args;
        let screen_w = config.width as f32;
        let screen_h = config.height as f32;

        let clip_rect_uv = if let Some(rect) = clip_rect {
            [
                rect.x.0 as f32 / screen_w,
                rect.y.0 as f32 / screen_h,
                (rect.x.0 + rect.width.0) as f32 / screen_w,
                (rect.y.0 + rect.height.0) as f32 / screen_h,
            ]
            .into()
        } else {
            [0.0, 0.0, 1.0, 1.0].into() // Default to full screen if no clip rect is provided
        };

        let rect_uv_bounds = [
            start_pos.x.0 as f32 / screen_w,
            start_pos.y.0 as f32 / screen_h,
            (start_pos.x.0 + size.width.0) as f32 / screen_w,
            (start_pos.y.0 + size.height.0) as f32 / screen_h,
        ];

        let corner_radii = match args.shape {
            crate::shape_def::Shape::RoundedRectangle {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
                ..
            } => [
                top_left.to_pixels_f32(),
                top_right.to_pixels_f32(),
                bottom_right.to_pixels_f32(),
                bottom_left.to_pixels_f32(),
            ]
            .into(),
            crate::shape_def::Shape::Ellipse => Vec4::ZERO,
            crate::shape_def::Shape::HorizontalCapsule => {
                let radius = size.height.to_f32() / 2.0;
                [radius, radius, radius, radius].into()
            }
            crate::shape_def::Shape::VerticalCapsule => {
                let radius = size.width.to_f32() / 2.0;
                [radius, radius, radius, radius].into()
            }
        };

        let is_axis_aligned_rect =
            matches!(args.shape, crate::shape_def::Shape::RoundedRectangle { .. })
                && corner_radii == Vec4::ZERO;

        let shape_type = match args.shape {
            crate::shape_def::Shape::RoundedRectangle { .. } => 0.0,
            crate::shape_def::Shape::Ellipse => 1.0,
            crate::shape_def::Shape::HorizontalCapsule => 0.0,
            crate::shape_def::Shape::VerticalCapsule => 0.0,
        };
        let shape_type = if is_axis_aligned_rect {
            2.0
        } else {
            shape_type
        };

        let g2_k_value = match args.shape {
            crate::shape_def::Shape::RoundedRectangle { g2_k_value, .. } => g2_k_value,
            crate::shape_def::Shape::Ellipse => 0.0,
            crate::shape_def::Shape::HorizontalCapsule => 2.0,
            crate::shape_def::Shape::VerticalCapsule => 2.0,
        };

        let border_width = args
            .border
            .as_ref()
            .map(|b| b.width.0 as f32)
            .unwrap_or(0.0);

        GlassUniforms {
            tint_color: args.tint_color.to_array().into(),
            rect_uv_bounds: rect_uv_bounds.into(),
            clip_rect_uv,
            rect_size_px: [size.width.0 as f32, size.height.0 as f32].into(),
            ripple_center: args.ripple_center.unwrap_or([0.0, 0.0]).into(),
            corner_radii,
            shape_type,
            g2_k_value,
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
            border_width,
            screen_size: [screen_w, screen_h].into(),
            light_source: [screen_w * 0.1, screen_h * 0.1].into(),
            light_scale: 1.0,
        }
    }

    /// Build per-instance uniforms from commands. Delegates to `build_instance` to keep
    /// complexity low.
    fn build_instances(
        commands: &[(&FluidGlassCommand, PxSize, PxPosition)],
        config: &wgpu::SurfaceConfiguration,
        clip_rect: Option<PxRect>,
    ) -> Vec<GlassUniforms> {
        commands
            .iter()
            .map(|(cmd, size, pos)| Self::build_instance(cmd, size, pos, config, clip_rect))
            .collect()
    }

    /// Enforce instance limit by truncating the instances vector in-place.
    ///
    /// This is an associated helper so it can be called as `Self::enforce_instance_limit`
    /// from other pipeline methods.
    fn enforce_instance_limit(instances: &mut Vec<GlassUniforms>) -> u32 {
        if instances.len() > MAX_CONCURRENT_GLASSES {
            instances.truncate(MAX_CONCURRENT_GLASSES);
        }
        instances.len() as u32
    }

    /// Create GPU buffer and upload the instance data. Returns the created buffer or an error.
    fn create_and_upload_buffer(
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[GlassUniforms],
    ) -> Result<wgpu::Buffer, ()> {
        // Serialize uniforms first so we can determine exact buffer size (avoids magic numbers).
        let uniforms = GlassInstances {
            instances: instances.to_vec(),
        };

        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        if buffer_content.write(&uniforms).is_err() {
            return Err(());
        }

        let size = buffer_content.as_ref().len() as wgpu::BufferAddress;
        let uniform_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fluid Glass Storage Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(&uniform_buffer, 0, buffer_content.as_ref());
        Ok(uniform_buffer)
    }

    /// Helper to create the uniform/storage buffer and corresponding bind group.
    /// Returns (buffer, bind_group) on success, or None on failure.
    fn create_buffer_and_bind_group(
        &self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[GlassUniforms],
        scene_texture_view: &wgpu::TextureView,
    ) -> Option<(wgpu::Buffer, wgpu::BindGroup)> {
        let uniform_buffer = match Self::create_and_upload_buffer(gpu, queue, instances) {
            Ok(buf) => buf,
            Err(_) => return None,
        };

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

        Some((uniform_buffer, bind_group))
    }

    /// Prepare per-draw GPU resources: build instances, enforce limits, create/upload buffer and bind group.
    /// Returns (buffer, bind_group, instance_count) if ready to draw, or None when there is nothing to draw or upload fails.
    fn prepare_draw_resources(
        &self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        commands: &[(&FluidGlassCommand, PxSize, PxPosition)],
        scene_texture_view: &wgpu::TextureView,
        clip_rect: Option<PxRect>,
    ) -> Option<(wgpu::Buffer, wgpu::BindGroup, u32)> {
        if commands.is_empty() {
            return None;
        }

        // Prepare instance list and enforce a maximum concurrent instance limit.
        // This keeps the GPU upload bounded and simplifies reasoning in the draw path.
        let mut instances = Self::build_instances(commands, config, clip_rect);
        if instances.is_empty() {
            return None;
        }
        let instance_count = Self::enforce_instance_limit(&mut instances);
        if instances.is_empty() {
            return None;
        }

        // Reuse existing helper to create buffer + bind group.
        let (uniform_buffer, bind_group) =
            self.create_buffer_and_bind_group(gpu, queue, &instances, scene_texture_view)?;

        Some((uniform_buffer, bind_group, instance_count))
    }
}
