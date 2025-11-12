use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use encase::{ShaderType, StorageBuffer, UniformBuffer};
use glam::{Vec2, Vec4};
use lru::LruCache;
use tessera_ui::{PxPosition, PxSize, px::PxRect, renderer::DrawablePipeline, wgpu};

use crate::fluid_glass::FluidGlassCommand;

// Define MAX_CONCURRENT_SHAPES, can be adjusted later
pub const MAX_CONCURRENT_GLASSES: usize = 256;
const FLUID_GLASS_SDF_CACHE_CAPACITY: usize = 64;
/// Minimum number of frames an SDF must be requested before being cached.
const SDF_CACHE_HEAT_THRESHOLD: u32 = 3;
/// Number of frames to keep SDF heat tracking data before cleanup.
const SDF_HEAT_TRACKING_WINDOW: u32 = 10;

#[derive(ShaderType)]
struct SdfUniforms {
    size: Vec2,
    corner_radii: Vec4,
    shape_type: f32,
    g2_k_value: f32,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct FluidGlassSdfCacheKey {
    shape_type: u32,
    corner_radii: [u32; 4],
    g2_k_value: u32,
    width: u32,
    height: u32,
}

struct FluidGlassSdfCacheEntry {
    view: wgpu::TextureView,
}

struct PreparedGlassInstance {
    uniforms: GlassUniforms,
    sdf_entry: Option<Arc<FluidGlassSdfCacheEntry>>,
}

struct FluidGlassSdfGenerator {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl FluidGlassSdfCacheKey {
    fn new(shape_type: f32, corner_radii: Vec4, g2_k_value: f32, width: u32, height: u32) -> Self {
        Self {
            shape_type: shape_type.to_bits(),
            corner_radii: [
                corner_radii.x.to_bits(),
                corner_radii.y.to_bits(),
                corner_radii.z.to_bits(),
                corner_radii.w.to_bits(),
            ],
            g2_k_value: g2_k_value.to_bits(),
            width,
            height,
        }
    }
}

impl FluidGlassSdfGenerator {
    fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fluid Glass SDF Cache Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("fluid_glass/sdf_cache.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: Some("fluid_glass_sdf_cache_bind_group_layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fluid Glass SDF Cache Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Fluid Glass SDF Cache Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    fn generate(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        corner_radii: Vec4,
        shape_type: f32,
        g2_k_value: f32,
    ) -> FluidGlassSdfCacheEntry {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fluid Glass Cached SDF Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sdf_uniforms = SdfUniforms {
            size: Vec2::new(width as f32, height as f32),
            corner_radii,
            shape_type,
            g2_k_value,
        };

        let mut uniform_buffer = UniformBuffer::new(Vec::new());
        uniform_buffer.write(&sdf_uniforms).unwrap();
        let uniform_data = uniform_buffer.into_inner();
        let uniform_buffer_gpu = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fluid Glass SDF Uniform Buffer"),
            size: uniform_data.len() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform_buffer_gpu, 0, &uniform_data);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer_gpu.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
            label: Some("fluid_glass_sdf_cache_bind_group"),
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Fluid Glass SDF Cache Encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Fluid Glass SDF Cache Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let workgroups_x = width.div_ceil(8);
            let workgroups_y = height.div_ceil(8);
            pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }

        queue.submit(Some(encoder.finish()));

        FluidGlassSdfCacheEntry { view }
    }
}

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
    sdf_cache_enabled: f32,
    screen_size: Vec2,  // Screen dimensions
    light_source: Vec2, // Light source position in world coordinates
    light_scale: f32,   // Light intensity scale factor
}

#[derive(ShaderType)]
struct GlassInstances {
    #[shader(size(runtime))]
    instances: Vec<GlassUniforms>,
}

/// Tracks how frequently an SDF is requested to decide if it should be cached.
#[derive(Debug, Clone)]
struct SdfHeatTracker {
    /// Number of frames this SDF has been requested
    hit_count: u32,
    /// Frame number when last seen
    last_seen_frame: u32,
}

// --- Pipeline Definition ---

pub(crate) struct FluidGlassPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    sdf_sampler: wgpu::Sampler,
    sdf_generator: FluidGlassSdfGenerator,
    sdf_cache: LruCache<FluidGlassSdfCacheKey, Arc<FluidGlassSdfCacheEntry>>,
    /// Tracks SDF usage frequency to avoid caching transient SDFs
    sdf_heat_tracker: HashMap<FluidGlassSdfCacheKey, SdfHeatTracker>,
    /// Current frame number for heat tracking
    current_frame: u32,
    dummy_sdf_view: wgpu::TextureView,
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
        let sdf_sampler = Self::create_sampler(gpu);
        let bind_group_layout = Self::create_bind_group_layout(gpu);
        let pipeline =
            Self::create_render_pipeline(gpu, config, sample_count, &shader, &bind_group_layout);
        let sdf_generator = FluidGlassSdfGenerator::new(gpu);
        let dummy_sdf_texture = gpu.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fluid Glass Dummy SDF Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let dummy_sdf_view = dummy_sdf_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sdf_cache = LruCache::new(
            NonZeroUsize::new(FLUID_GLASS_SDF_CACHE_CAPACITY)
                .expect("SDF cache capacity must be greater than zero"),
        );

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            sdf_sampler,
            sdf_generator,
            sdf_cache,
            sdf_heat_tracker: HashMap::new(),
            current_frame: 0,
            dummy_sdf_view,
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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
        // Advance frame counter and cleanup old SDF heat tracking data
        self.current_frame = self.current_frame.wrapping_add(1);
        self.sdf_heat_tracker.retain(|_, tracker| {
            // Remove entries not seen in the last SDF_HEAT_TRACKING_WINDOW frames
            self.current_frame.saturating_sub(tracker.last_seen_frame) < SDF_HEAT_TRACKING_WINDOW
        });

        let instances = self.build_instances(commands, config, clip_rect, gpu, queue);
        if instances.is_empty() {
            return;
        }

        let groups = self.group_instances_by_sdf(instances);

        render_pass.set_pipeline(&self.pipeline);
        let mut alive_buffers: Vec<wgpu::Buffer> = Vec::new();

        for (entry, uniforms) in groups {
            if uniforms.is_empty() {
                continue;
            }
            let uniform_buffer = match Self::create_and_upload_buffer(gpu, queue, &uniforms) {
                Ok(buf) => buf,
                Err(_) => continue,
            };
            let sdf_view = entry
                .as_ref()
                .map(|entry| &entry.view)
                .unwrap_or(&self.dummy_sdf_view);
            let bind_group =
                self.create_bind_group(gpu, &uniform_buffer, scene_texture_view, sdf_view);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..uniforms.len() as u32);
            alive_buffers.push(uniform_buffer);
        }
    }
}

impl FluidGlassPipeline {
    fn build_instance(
        &mut self,
        command: &FluidGlassCommand,
        size: &PxSize,
        start_pos: &PxPosition,
        config: &wgpu::SurfaceConfiguration,
        clip_rect: Option<PxRect>,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> PreparedGlassInstance {
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

        let sdf_entry =
            self.maybe_get_sdf_entry(gpu, queue, size, corner_radii, shape_type, g2_k_value);

        let uniforms = GlassUniforms {
            tint_color: args.tint_color.to_array().into(),
            rect_uv_bounds: rect_uv_bounds.into(),
            clip_rect_uv,
            rect_size_px: [size.width.0 as f32, size.height.0 as f32].into(),
            ripple_center: args.ripple_center.unwrap_or([0.0, 0.0]).into(),
            corner_radii,
            shape_type,
            g2_k_value,
            dispersion_height: args.dispersion_height.to_pixels_f32(),
            chroma_multiplier: args.chroma_multiplier,
            refraction_height: args.refraction_height.to_pixels_f32(),
            refraction_amount: args.refraction_amount,
            eccentric_factor: args.eccentric_factor,
            noise_amount: args.noise_amount,
            noise_scale: args.noise_scale,
            time: args.time,
            ripple_radius: args.ripple_radius.unwrap_or(0.0),
            ripple_alpha: args.ripple_alpha.unwrap_or(0.0),
            ripple_strength: args.ripple_strength.unwrap_or(0.0),
            border_width,
            sdf_cache_enabled: if sdf_entry.is_some() { 1.0 } else { 0.0 },
            screen_size: [screen_w, screen_h].into(),
            light_source: [screen_w * 0.1, screen_h * 0.1].into(),
            light_scale: 1.0,
        };

        PreparedGlassInstance {
            uniforms,
            sdf_entry,
        }
    }

    fn build_instances(
        &mut self,
        commands: &[(&FluidGlassCommand, PxSize, PxPosition)],
        config: &wgpu::SurfaceConfiguration,
        clip_rect: Option<PxRect>,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Vec<PreparedGlassInstance> {
        let mut instances = commands
            .iter()
            .map(|(cmd, size, pos)| {
                self.build_instance(cmd, size, pos, config, clip_rect, gpu, queue)
            })
            .collect::<Vec<_>>();
        Self::enforce_instance_limit(&mut instances);
        instances
    }

    fn enforce_instance_limit(instances: &mut Vec<PreparedGlassInstance>) -> u32 {
        if instances.len() > MAX_CONCURRENT_GLASSES {
            instances.truncate(MAX_CONCURRENT_GLASSES);
        }
        instances.len() as u32
    }

    fn maybe_get_sdf_entry(
        &mut self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        size: &PxSize,
        corner_radii: Vec4,
        shape_type: f32,
        g2_k_value: f32,
    ) -> Option<Arc<FluidGlassSdfCacheEntry>> {
        if !(shape_type == 0.0 || shape_type == 1.0) {
            return None;
        }

        let width = size.width.0.max(0) as u32;
        let height = size.height.0.max(0) as u32;
        if width == 0 || height == 0 {
            return None;
        }

        let key = FluidGlassSdfCacheKey::new(shape_type, corner_radii, g2_k_value, width, height);

        // Check if already cached
        if let Some(entry) = self.sdf_cache.get(&key) {
            return Some(entry.clone());
        }

        // Update heat tracking
        let tracker = self
            .sdf_heat_tracker
            .entry(key.clone())
            .or_insert(SdfHeatTracker {
                hit_count: 0,
                last_seen_frame: self.current_frame,
            });

        // Update tracker
        if tracker.last_seen_frame != self.current_frame {
            tracker.hit_count += 1;
            tracker.last_seen_frame = self.current_frame;
        }

        // Only cache if SDF has been requested frequently enough
        if tracker.hit_count >= SDF_CACHE_HEAT_THRESHOLD {
            let entry = Arc::new(self.sdf_generator.generate(
                gpu,
                queue,
                width,
                height,
                corner_radii,
                shape_type,
                g2_k_value,
            ));

            _ = self.sdf_cache.put(key, entry.clone());
            Some(entry)
        } else {
            // SDF is not hot enough yet, don't cache
            None
        }
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

    fn create_bind_group(
        &self,
        gpu: &wgpu::Device,
        uniform_buffer: &wgpu::Buffer,
        scene_texture_view: &wgpu::TextureView,
        sdf_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        gpu.create_bind_group(&wgpu::BindGroupDescriptor {
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(sdf_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.sdf_sampler),
                },
            ],
            label: Some("fluid_glass_bind_group"),
        })
    }

    fn group_instances_by_sdf(
        &self,
        instances: Vec<PreparedGlassInstance>,
    ) -> Vec<(Option<Arc<FluidGlassSdfCacheEntry>>, Vec<GlassUniforms>)> {
        let mut groups: Vec<(Option<Arc<FluidGlassSdfCacheEntry>>, Vec<GlassUniforms>)> =
            Vec::new();

        for instance in instances {
            if let Some((_, uniforms)) = groups.iter_mut().find(|(entry, _)| {
                Self::sdf_entries_match(entry.as_ref(), instance.sdf_entry.as_ref())
            }) {
                uniforms.push(instance.uniforms);
            } else {
                groups.push((instance.sdf_entry.clone(), vec![instance.uniforms]));
            }
        }

        groups
    }

    fn sdf_entries_match(
        a: Option<&Arc<FluidGlassSdfCacheEntry>>,
        b: Option<&Arc<FluidGlassSdfCacheEntry>>,
    ) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}
