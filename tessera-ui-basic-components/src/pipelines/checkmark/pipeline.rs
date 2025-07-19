use bytemuck::{Pod, Zeroable};
use encase::{ShaderType, UniformBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    PxPosition, PxSize,
    renderer::DrawablePipeline,
    wgpu::{self, include_wgsl, util::DeviceExt},
};

use crate::pipelines::pos_misc::pixel_to_ndc;

use super::command::CheckmarkCommand;

#[derive(ShaderType)]
pub struct CheckmarkUniforms {
    pub size: Vec2,
    pub color: Vec4,
    pub stroke_width: f32,
    pub progress: f32,
    pub padding: Vec2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct CheckmarkVertex {
    /// Position of the vertex (x, y, z)
    position: [f32; 3],
    /// UV coordinates for the vertex
    uv: [f32; 2],
}

impl CheckmarkVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CheckmarkVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct CheckmarkPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_staging_buffer: Vec<u8>,
}

impl CheckmarkPipeline {
    pub fn new(gpu: &wgpu::Device, config: &wgpu::SurfaceConfiguration, sample_count: u32) -> Self {
        let shader = gpu.create_shader_module(include_wgsl!("checkmark.wgsl"));

        // Create uniform buffer
        let uniform_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Checkmark Uniform Buffer"),
            size: CheckmarkUniforms::min_size().get(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Checkmark Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group
        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Checkmark Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create render pipeline layout
        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Checkmark Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Checkmark Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[CheckmarkVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
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
            multiview: None,
            cache: None,
        });

        // Create quad vertices (two triangles forming a rectangle)
        let vertices = [
            CheckmarkVertex {
                position: [-1.0, -1.0, 0.0],
                uv: [0.0, 1.0],
            },
            CheckmarkVertex {
                position: [1.0, -1.0, 0.0],
                uv: [1.0, 1.0],
            },
            CheckmarkVertex {
                position: [1.0, 1.0, 0.0],
                uv: [1.0, 0.0],
            },
            CheckmarkVertex {
                position: [-1.0, 1.0, 0.0],
                uv: [0.0, 0.0],
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

        let vertex_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Checkmark Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Checkmark Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group,
            vertex_buffer,
            index_buffer,
            uniform_staging_buffer: vec![0; CheckmarkUniforms::min_size().get() as usize],
        }
    }
}

impl DrawablePipeline<CheckmarkCommand> for CheckmarkPipeline {
    fn draw(
        &mut self,
        _gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &CheckmarkCommand,
        size: PxSize,
        start_pos: PxPosition,
        _scene_texture_view: &wgpu::TextureView,
    ) {
        // Convert position and size to NDC coordinates
        let ndc_pos = pixel_to_ndc(start_pos, [config.width, config.height]);
        let ndc_size = [
            size.width.to_f32() / config.width as f32 * 2.0,
            size.height.to_f32() / config.height as f32 * 2.0,
        ];

        // Create uniforms
        let uniforms = CheckmarkUniforms {
            size: [size.width.to_f32(), size.height.to_f32()].into(),
            color: command.color.to_array().into(),
            stroke_width: command.stroke_width,
            progress: command.progress,
            padding: command.padding.into(),
        };

        // Update uniform buffer using the staging buffer
        {
            let mut buffer = UniformBuffer::new(&mut self.uniform_staging_buffer);
            buffer.write(&uniforms).unwrap();
        }
        gpu_queue.write_buffer(&self.uniform_buffer, 0, &self.uniform_staging_buffer);

        // Update vertex positions to match the actual position and size
        let vertices = [
            CheckmarkVertex {
                position: [ndc_pos[0], ndc_pos[1] - ndc_size[1], 0.0],
                uv: [0.0, 1.0],
            },
            CheckmarkVertex {
                position: [ndc_pos[0] + ndc_size[0], ndc_pos[1] - ndc_size[1], 0.0],
                uv: [1.0, 1.0],
            },
            CheckmarkVertex {
                position: [ndc_pos[0] + ndc_size[0], ndc_pos[1], 0.0],
                uv: [1.0, 0.0],
            },
            CheckmarkVertex {
                position: [ndc_pos[0], ndc_pos[1], 0.0],
                uv: [0.0, 0.0],
            },
        ];

        // Update vertex buffer
        gpu_queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        // Set pipeline and draw
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
