use encase::{ShaderType, StorageBuffer, UniformBuffer, internal::WriteInto};
use glam::IVec4;
use tessera_ui::{render::GraphicsDevice, wgpu};

use super::params::{Elements, GlassParams};

#[derive(ShaderType)]
pub(super) struct BackdropParams {
    pub region: IVec4,
    pub capture: IVec4,
    pub scene: IVec4,
    pub levels: IVec4,
    pub sizes: IVec4,
}

/// Shader modules, layouts and the sampler shared by the glass and lens pipelines.
pub(super) struct Shared {
    glass_shader: wgpu::ShaderModule,
    glass_layout: wgpu::BindGroupLayout,
    glass_pipeline_layout: wgpu::PipelineLayout,
    backdrop_shader: wgpu::ShaderModule,
    backdrop_layout: wgpu::BindGroupLayout,
    backdrop_pipeline_layout: wgpu::PipelineLayout,
    sampler: wgpu::Sampler,
}

/// Textures read by one glass or lens draw.
pub(super) struct GlassTextures<'a> {
    pub surface: &'a wgpu::TextureView,
    pub scene: &'a wgpu::TextureView,
    pub original: &'a wgpu::TextureView,
}

fn bytes<T: ShaderType + WriteInto>(value: &T, uniform: bool) -> Vec<u8> {
    if uniform {
        let mut content = UniformBuffer::new(Vec::<u8>::new());
        content.write(value).expect("liquid glass uniforms");
        content.into_inner()
    } else {
        let mut content = StorageBuffer::new(Vec::<u8>::new());
        content.write(value).expect("liquid glass storage");
        content.into_inner()
    }
}

impl Shared {
    pub fn new(gpu: &wgpu::Device) -> Self {
        let glass_shader = gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("liquid glass"),
            source: wgpu::ShaderSource::Wgsl(include_str!("glass.wgsl").into()),
        });
        let backdrop_shader = gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("liquid glass backdrop"),
            source: wgpu::ShaderSource::Wgsl(include_str!("backdrop.wgsl").into()),
        });
        let texture = |binding, visibility, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let buffer = |binding, visibility, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let fragment = wgpu::ShaderStages::FRAGMENT;
        let glass_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("liquid glass layout"),
            entries: &[
                buffer(
                    0,
                    wgpu::ShaderStages::VERTEX | fragment,
                    wgpu::BufferBindingType::Uniform,
                ),
                buffer(
                    1,
                    fragment,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
                texture(2, fragment, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: fragment,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                texture(4, fragment, false),
                texture(5, fragment, false),
            ],
        });
        let backdrop_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("liquid glass backdrop layout"),
            entries: &[
                buffer(
                    0,
                    wgpu::ShaderStages::COMPUTE,
                    wgpu::BufferBindingType::Uniform,
                ),
                texture(1, wgpu::ShaderStages::COMPUTE, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let layout = |label, bind_group_layout| {
            gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(bind_group_layout)],
                immediate_size: 0,
            })
        };
        Self {
            glass_pipeline_layout: layout("liquid glass pipeline layout", &glass_layout),
            backdrop_pipeline_layout: layout(
                "liquid glass backdrop pipeline layout",
                &backdrop_layout,
            ),
            glass_shader,
            glass_layout,
            backdrop_shader,
            backdrop_layout,
            sampler: gpu.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("liquid glass sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
        }
    }

    /// A render pipeline over glass.wgsl. `sample_count` selects a final UI draw into the
    /// target format; `None` draws into a single-sampled half-float region texture.
    pub fn render(
        &self,
        gpu: &wgpu::Device,
        cache: Option<&wgpu::PipelineCache>,
        entry_points: [&str; 2],
        target: Option<(wgpu::TextureFormat, u32)>,
    ) -> wgpu::RenderPipeline {
        let (format, samples) = target.unwrap_or((wgpu::TextureFormat::Rgba16Float, 1));
        gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(entry_points[1]),
            layout: Some(&self.glass_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.glass_shader,
                entry_point: Some(entry_points[0]),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.glass_shader,
                entry_point: Some(entry_points[1]),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // The shaders composite over the sampled scene themselves.
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: target.and(tessera_ui::render::ui_stencil_state()),
            multisample: wgpu::MultisampleState {
                count: samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache,
        })
    }

    pub fn compute(
        &self,
        gpu: &wgpu::Device,
        cache: Option<&wgpu::PipelineCache>,
        entry_point: &str,
    ) -> wgpu::ComputePipeline {
        gpu.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(entry_point),
            layout: Some(&self.backdrop_pipeline_layout),
            module: &self.backdrop_shader,
            entry_point: Some(entry_point),
            compilation_options: Default::default(),
            cache,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn dispatch(
        &self,
        graphics: &GraphicsDevice,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::ComputePipeline,
        uniforms: &BackdropParams,
        source: &wgpu::TextureView,
        destination: &wgpu::TextureView,
        size: [i32; 2],
    ) {
        let buffer = graphics.frame_buffer(wgpu::BufferUsages::UNIFORM, &bytes(uniforms, true));
        let bindings = graphics.cached_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("liquid glass backdrop bindings"),
            layout: &self.backdrop_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(destination),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("liquid glass backdrop"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bindings, &[]);
        pass.dispatch_workgroups(
            (size[0] as u32).div_ceil(8),
            (size[1] as u32).div_ceil(8),
            1,
        );
    }

    pub fn bindings(
        &self,
        graphics: &GraphicsDevice,
        params: &GlassParams,
        elements: &Elements,
        textures: GlassTextures<'_>,
    ) -> wgpu::BindGroup {
        let uniforms = graphics.frame_buffer(wgpu::BufferUsages::UNIFORM, &bytes(params, true));
        let elements = graphics.frame_buffer(wgpu::BufferUsages::STORAGE, &bytes(elements, false));
        let view = wgpu::BindingResource::TextureView;
        graphics.cached_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("liquid glass bindings"),
            layout: &self.glass_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: elements.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: view(textures.surface),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: view(textures.scene),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: view(textures.original),
                },
            ],
        })
    }
}
