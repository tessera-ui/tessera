use encase::{ShaderSize, ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    PxSize,
    px::PxPosition,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline},
    wgpu::{self, include_wgsl, util::DeviceExt},
};

use super::command::{ProgressArcCap, ProgressArcCommand};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
struct ArcUniform {
    position: Vec4,
    color: Vec4,
    screen_size: Vec2,
    stroke_width: f32,
    start_angle_degrees: f32,
    sweep_angle_degrees: f32,
    cap: u32,
    _pad: u32,
}

#[derive(PartialEq, ShaderType)]
struct ArcInstances {
    #[shader(size(runtime))]
    instances: Vec<ArcUniform>,
}

/// Render pipeline for drawing circular arc strokes.
pub struct ProgressArcPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
}

impl ProgressArcPipeline {
    /// Creates the arc pipeline with the provided surface configuration.
    pub fn new(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let shader = gpu.create_shader_module(include_wgsl!("progress_arc.wgsl"));

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
            label: Some("progress_arc_bind_group_layout"),
        });

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Progress Arc Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Progress Arc Pipeline"),
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
            label: Some("Progress Arc Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_indices: [u16; 6] = [0, 2, 1, 0, 3, 2];
        let quad_index_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Progress Arc Quad Index Buffer"),
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
    commands: &[(&ProgressArcCommand, PxSize, PxPosition)],
    target_size: PxSize,
) -> Vec<ArcUniform> {
    commands
        .iter()
        .map(|(command, size, position)| ArcUniform {
            position: Vec4::new(
                position.x.raw() as f32,
                position.y.raw() as f32,
                size.width.raw() as f32,
                size.height.raw() as f32,
            ),
            color: Vec4::from_array(command.color.to_array()),
            screen_size: Vec2::new(target_size.width.to_f32(), target_size.height.to_f32()),
            stroke_width: command.stroke_width_px,
            start_angle_degrees: command.start_angle_degrees,
            sweep_angle_degrees: command.sweep_angle_degrees,
            cap: match command.cap {
                ProgressArcCap::Round => 1,
                ProgressArcCap::Butt => 0,
            },
            _pad: 0,
        })
        .collect()
}

impl DrawablePipeline<ProgressArcCommand> for ProgressArcPipeline {
    fn draw(&mut self, context: &mut DrawContext<ProgressArcCommand>) {
        if context.commands.is_empty() {
            return;
        }

        let instances = build_instances(context.commands, context.target_size);
        if instances.is_empty() {
            return;
        }

        let uniform_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Progress Arc Storage Buffer"),
            size: 16 + ArcUniform::SHADER_SIZE.get() * instances.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms = ArcInstances { instances };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&uniforms)
            .expect("buffer write failed");
        context
            .queue
            .write_buffer(&uniform_buffer, 0, buffer_content.as_ref());

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
                label: Some("progress_arc_bind_group"),
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
            .draw_indexed(0..6, 0, 0..context.commands.len() as u32);
    }
}
