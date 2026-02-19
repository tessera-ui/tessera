use bytemuck::{Pod, Zeroable};
use encase::{ShaderType, UniformBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline},
    wgpu::{self, include_wgsl, util::DeviceExt},
};

use crate::pipelines::pos_misc::pixel_to_ndc;

use super::command::CheckmarkCommand;

#[derive(PartialEq, ShaderType)]
pub struct CheckmarkUniforms {
    pub size: Vec2,
    pub color: Vec4,
    pub stroke_width: f32,
    pub progress: f32,
    pub padding: Vec2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug, Pod, Zeroable)]
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

/// Render pipeline for animated checkmark strokes.
pub struct CheckmarkPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_staging_buffer: Vec<u8>,
}

impl CheckmarkPipeline {
    /// Creates the render pipeline for drawing checkmarks.
    pub fn new(
        gpu: &wgpu::Device,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        // Keep the constructor concise by delegating creation details to small helpers.
        let shader = Self::create_shader_module(gpu);
        let uniform_buffer = Self::create_uniform_buffer(gpu);
        let bind_group_layout = Self::create_bind_group_layout(gpu);
        let bind_group = Self::create_bind_group(gpu, &bind_group_layout, &uniform_buffer);
        let pipeline_layout = Self::create_pipeline_layout(gpu, &bind_group_layout);
        let pipeline = Self::create_pipeline(
            gpu,
            pipeline_cache,
            &shader,
            &pipeline_layout,
            config,
            sample_count,
        );
        let (vertex_buffer, index_buffer) = Self::create_buffers(gpu);

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

/// Small helpers extracted to simplify `draw` and reduce function
/// length/complexity.
impl CheckmarkPipeline {
    fn update_uniforms(&mut self, gpu_queue: &wgpu::Queue, uniforms: &CheckmarkUniforms) {
        let mut buffer = UniformBuffer::new(&mut self.uniform_staging_buffer);
        buffer
            .write(uniforms)
            .expect("Failed to write checkmark uniforms");
        gpu_queue.write_buffer(&self.uniform_buffer, 0, &self.uniform_staging_buffer);
    }

    fn update_vertices_for(
        &mut self,
        gpu_queue: &wgpu::Queue,
        ndc_pos: [f32; 2],
        ndc_size: [f32; 2],
    ) {
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

        gpu_queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
    }

    // Below are small factory helpers to keep `new` focused and short.
    fn create_shader_module(gpu: &wgpu::Device) -> wgpu::ShaderModule {
        gpu.create_shader_module(include_wgsl!("checkmark.wgsl"))
    }

    fn create_uniform_buffer(gpu: &wgpu::Device) -> wgpu::Buffer {
        gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Checkmark Uniform Buffer"),
            size: CheckmarkUniforms::min_size().get(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_bind_group_layout(gpu: &wgpu::Device) -> wgpu::BindGroupLayout {
        gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        })
    }

    fn create_bind_group(
        gpu: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Checkmark Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        })
    }

    fn create_pipeline_layout(
        gpu: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::PipelineLayout {
        gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Checkmark Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            immediate_size: 0,
        })
    }

    fn create_pipeline(
        gpu: &wgpu::Device,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        shader: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> wgpu::RenderPipeline {
        gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Checkmark Pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[CheckmarkVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
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
            multiview_mask: None,
            cache: pipeline_cache,
        })
    }

    fn create_buffers(gpu: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
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

        (vertex_buffer, index_buffer)
    }
}

impl DrawablePipeline<CheckmarkCommand> for CheckmarkPipeline {
    fn draw(&mut self, context: &mut DrawContext<CheckmarkCommand>) {
        context.render_pass.set_pipeline(&self.pipeline);
        context.render_pass.set_bind_group(0, &self.bind_group, &[]);
        context
            .render_pass
            .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        context
            .render_pass
            .set_vertex_buffer(0, self.vertex_buffer.slice(..));

        for (command, size, start_pos) in context.commands.iter() {
            // Convert position and size to NDC coordinates
            let ndc_pos = pixel_to_ndc(
                *start_pos,
                [
                    context.target_size.width.positive(),
                    context.target_size.height.positive(),
                ],
            );
            let ndc_size = [
                size.width.to_f32() / context.target_size.width.to_f32() * 2.0,
                size.height.to_f32() / context.target_size.height.to_f32() * 2.0,
            ];

            // Create uniforms
            let uniforms = CheckmarkUniforms {
                size: [size.width.to_f32(), size.height.to_f32()].into(),
                color: command.color.to_array().into(),
                stroke_width: command.stroke_width,
                progress: command.progress,
                padding: command.padding.into(),
            };

            // Update uniform buffer
            self.update_uniforms(context.queue, &uniforms);

            // Update vertex positions
            self.update_vertices_for(context.queue, ndc_pos, ndc_size);

            // Set pipeline and draw
            context.render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }
}
