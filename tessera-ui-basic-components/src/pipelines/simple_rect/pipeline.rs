//! Lightweight pipeline for rendering solid rectangles.

use encase::{ShaderSize, ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    PxPosition, PxSize,
    px::PxRect,
    renderer::DrawablePipeline,
    wgpu::{self, include_wgsl, util::DeviceExt},
};

use super::command::SimpleRectCommand;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
struct RectUniform {
    position: Vec4,
    color: Vec4,
    screen_size: Vec2,
    #[shader(size(8))]
    _padding: [f32; 2],
}

#[derive(ShaderType)]
struct RectInstances {
    #[shader(size(runtime))]
    instances: Vec<RectUniform>,
}

pub struct SimpleRectPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
}

impl SimpleRectPipeline {
    pub fn new(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let shader = gpu.create_shader_module(include_wgsl!("simple_rect.wgsl"));

        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("simple_rect_bind_group_layout"),
        });

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Simple Rect Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Simple Rect Pipeline"),
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
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
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
            label: Some("Simple Rect Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_indices: [u16; 6] = [0, 2, 1, 0, 3, 2];
        let quad_index_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simple Rect Quad Index Buffer"),
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
}

fn build_instances(
    commands: &[(&SimpleRectCommand, PxSize, PxPosition)],
    config: &wgpu::SurfaceConfiguration,
) -> Vec<RectUniform> {
    commands
        .iter()
        .map(|(command, size, position)| RectUniform {
            position: Vec4::new(
                position.x.raw() as f32,
                position.y.raw() as f32,
                size.width.raw() as f32,
                size.height.raw() as f32,
            ),
            color: Vec4::from_array(command.color.to_array()),
            screen_size: Vec2::new(config.width as f32, config.height as f32),
            _padding: [0.0; 2],
        })
        .collect()
}

impl DrawablePipeline<SimpleRectCommand> for SimpleRectPipeline {
    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        commands: &[(&SimpleRectCommand, PxSize, PxPosition)],
        _scene_texture_view: &wgpu::TextureView,
        _clip_rect: Option<PxRect>,
    ) {
        if commands.is_empty() {
            return;
        }

        let instances = build_instances(commands, config);
        if instances.is_empty() {
            return;
        }

        let uniform_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Simple Rect Storage Buffer"),
            size: 16 + RectUniform::SHADER_SIZE.get() * instances.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms = RectInstances { instances };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content.write(&uniforms).unwrap();
        gpu_queue.write_buffer(&uniform_buffer, 0, buffer_content.as_ref());

        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("simple_rect_bind_group"),
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..commands.len() as u32);
    }
}
