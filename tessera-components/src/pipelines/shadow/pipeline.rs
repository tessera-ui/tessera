//! Shadow mask and composite pipelines for MD3-style shadows.
//!
//! ## Usage
//!
//! Draw shadow masks into offscreen textures and composite blurred layers.

use encase::{ShaderSize, ShaderType, StorageBuffer, UniformBuffer};
use glam::Vec4;
use tessera_ui::{
    Color, PxPosition, PxSize,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline},
    wgpu::{self, include_wgsl, util::DeviceExt},
};

use crate::pipelines::shape::{
    command::{ShapeCommand, rect_to_uniforms},
    pipeline::ShapeUniforms,
};

use super::command::{ShadowCompositeCommand, ShadowMaskCommand};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

#[derive(ShaderType)]
struct MaskInstances {
    #[shader(size(runtime))]
    instances: Vec<ShapeUniforms>,
}

/// Pipeline for rendering shadow masks into RGBA textures.
pub struct ShadowMaskPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
}

impl ShadowMaskPipeline {
    /// Creates the shadow mask pipeline for RGBA render targets.
    pub fn new(
        gpu: &wgpu::Device,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let shader = gpu.create_shader_module(include_wgsl!("../shape/shape.wgsl"));

        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("shadow_mask_bind_group_layout"),
        });

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Mask Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Mask Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                }],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: pipeline_cache,
        });

        let quad_vertices = [
            Vertex {
                position: [0.0, 0.0],
            },
            Vertex {
                position: [1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
            Vertex {
                position: [0.0, 1.0],
            },
        ];
        let quad_vertex_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shadow Mask Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_indices: [u16; 6] = [0, 2, 1, 0, 3, 2];
        let quad_index_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shadow Mask Quad Index Buffer"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            bind_group_layout,
            quad_vertex_buffer,
            quad_index_buffer,
        }
    }

    fn build_instances(
        commands: &[(&ShadowMaskCommand, PxSize, PxPosition)],
        target_size: PxSize,
    ) -> Vec<ShapeUniforms> {
        commands
            .iter()
            .map(|(command, size, position)| {
                let shape_command = match command.shape {
                    crate::shape_def::ResolvedShape::Rounded {
                        corner_radii,
                        corner_g2,
                    } => ShapeCommand::Rect {
                        color: command.color,
                        corner_radii,
                        corner_g2,
                    },
                    crate::shape_def::ResolvedShape::Ellipse => ShapeCommand::Ellipse {
                        color: command.color,
                    },
                };
                let mut uniforms = rect_to_uniforms(&shape_command, *size, *position);
                uniforms.screen_size =
                    [target_size.width.to_f32(), target_size.height.to_f32()].into();
                uniforms
            })
            .collect()
    }
}

impl DrawablePipeline<ShadowMaskCommand> for ShadowMaskPipeline {
    fn draw(&mut self, context: &mut DrawContext<ShadowMaskCommand>) {
        if context.commands.is_empty() {
            return;
        }

        let instances = Self::build_instances(context.commands, context.target_size);
        if instances.is_empty() {
            return;
        }

        let storage_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Mask Storage Buffer"),
            size: 16 + ShapeUniforms::SHADER_SIZE.get() * instances.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms = MaskInstances { instances };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&uniforms)
            .expect("shadow mask buffer write failed");
        context
            .queue
            .write_buffer(&storage_buffer, 0, buffer_content.as_ref());

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: storage_buffer.as_entire_binding(),
                }],
                label: Some("shadow_mask_bind_group"),
            });

        context.render_pass.set_pipeline(&self.pipeline);
        context.render_pass.set_bind_group(0, &bind_group, &[]);
        context
            .render_pass
            .set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        context
            .render_pass
            .set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        context
            .render_pass
            .draw_indexed(0..6, 0, 0..uniforms.instances.len() as u32);
    }
}

#[derive(ShaderType)]
struct CompositeUniforms {
    rect: Vec4,
    color: Vec4,
    uv_rect: Vec4,
}

/// Pipeline for compositing blurred shadow textures into the scene.
pub struct ShadowCompositePipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl ShadowCompositePipeline {
    /// Creates the shadow composite pipeline.
    pub fn new(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let shader = gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Composite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shadow_composite.wgsl").into()),
        });

        let sampler = gpu.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Composite Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("shadow_composite_bind_group_layout"),
        });

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Composite Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Composite Pipeline"),
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
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: pipeline_cache,
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
        }
    }

    fn compute_uniforms(
        start_pos: PxPosition,
        size: PxSize,
        target_size: PxSize,
        color: Color,
        uv_origin: [f32; 2],
        uv_size: [f32; 2],
    ) -> CompositeUniforms {
        let rect = [
            (start_pos.x.0 as f32 / target_size.width.to_f32()) * 2.0 - 1.0
                + (size.width.0 as f32 / target_size.width.to_f32()),
            (start_pos.y.0 as f32 / target_size.height.to_f32()) * -2.0 + 1.0
                - (size.height.0 as f32 / target_size.height.to_f32()),
            size.width.0 as f32 / target_size.width.to_f32(),
            size.height.0 as f32 / target_size.height.to_f32(),
        ]
        .into();

        CompositeUniforms {
            rect,
            color: Vec4::from_array(color.to_array()),
            uv_rect: Vec4::new(uv_origin[0], uv_origin[1], uv_size[0], uv_size[1]),
        }
    }
}

impl DrawablePipeline<ShadowCompositeCommand> for ShadowCompositePipeline {
    fn draw(&mut self, context: &mut DrawContext<ShadowCompositeCommand>) {
        if context.commands.is_empty() {
            return;
        }

        context.render_pass.set_pipeline(&self.pipeline);
        let mut alive_buffers: Vec<wgpu::Buffer> = Vec::new();

        for (command, size, start_pos) in context.commands.iter() {
            let uniforms = Self::compute_uniforms(
                *start_pos,
                *size,
                context.target_size,
                command.color,
                command.uv_origin,
                command.uv_size,
            );
            let mut buffer = UniformBuffer::new(Vec::new());
            buffer
                .write(&uniforms)
                .expect("shadow composite uniform serialization failed");

            let uniform_buffer =
                context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("shadow_composite_uniform_buffer"),
                        contents: &buffer.into_inner(),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });

            let bind_group = context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                context.scene_texture_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: uniform_buffer.as_entire_binding(),
                        },
                    ],
                    label: Some("shadow_composite_bind_group"),
                });

            context.render_pass.set_bind_group(0, &bind_group, &[]);
            context.render_pass.draw(0..6, 0..1);
            alive_buffers.push(uniform_buffer);
        }
    }
}
