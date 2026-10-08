use std::{num::NonZeroU64, sync::Arc};

use encase::{ShaderSize, ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    PxSize,
    px::PxPosition,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline, FrameContext},
    wgpu::{self, include_wgsl},
};

use crate::pipelines::SharedQuad;

use super::command::SimpleRectCommand;

const INITIAL_INSTANCE_CAPACITY: usize = 64;

#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
struct RectUniform {
    position: Vec4,
    color: Vec4,
    screen_size: Vec2,
}

#[derive(PartialEq, ShaderType)]
struct RectInstances {
    #[shader(size(runtime))]
    instances: Vec<RectUniform>,
}

/// Render pipeline for drawing batches of simple rectangles.
pub struct SimpleRectPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    quad_vertex_buffer: Arc<wgpu::Buffer>,
    quad_index_buffer: Arc<wgpu::Buffer>,
    /// Grow-only staging storage shared by every simple-rect draw in a frame.
    /// Each draw writes into its own aligned slice; `staging_len` is the next
    /// free offset and is reset in `end_frame` without shrinking capacity.
    staging_buffer: Option<wgpu::Buffer>,
    staging_capacity_bytes: u64,
    staging_len: u64,
}

fn storage_bytes_for(instance_count: usize) -> u64 {
    // encase runtime storage arrays are prefixed by a 16-byte count.
    16 + RectUniform::SHADER_SIZE.get() * instance_count as u64
}

impl SimpleRectPipeline {
    /// Creates the rectangle pipeline with the provided surface configuration.
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
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Simple Rect Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                })],
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

        let shared_quad = SharedQuad::get(gpu);

        Self {
            pipeline,
            bind_group_layout,
            quad_vertex_buffer: Arc::clone(shared_quad.vertex()),
            quad_index_buffer: Arc::clone(shared_quad.index()),
            staging_buffer: None,
            staging_capacity_bytes: 0,
            staging_len: 0,
        }
    }

    /// Ensures the staging buffer can hold `required_bytes`, doubling until it
    /// fits. The existing buffer is kept when it is already large enough.
    fn ensure_staging(&mut self, device: &wgpu::Device, required_bytes: u64) {
        let fits = self
            .staging_buffer
            .as_ref()
            .is_some_and(|_| required_bytes <= self.staging_capacity_bytes);
        if fits {
            return;
        }

        let mut capacity = self
            .staging_capacity_bytes
            .max(storage_bytes_for(INITIAL_INSTANCE_CAPACITY));
        while capacity < required_bytes {
            capacity = capacity.saturating_mul(2);
        }

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Simple Rect Storage Buffer"),
            size: capacity,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.staging_buffer = Some(buffer);
        self.staging_capacity_bytes = capacity;
    }
}

fn build_instances(
    commands: &[(&SimpleRectCommand, PxSize, PxPosition)],
    target_size: PxSize,
) -> Vec<RectUniform> {
    commands
        .iter()
        .map(|(command, size, position)| RectUniform {
            position: Vec4::new(
                position.x,
                position.y,
                size.width.raw() as f32,
                size.height.raw() as f32,
            ),
            color: Vec4::from_array(command.color.to_array()),
            screen_size: Vec2::new(target_size.width.to_f32(), target_size.height.to_f32()),
        })
        .collect()
}

impl DrawablePipeline<SimpleRectCommand> for SimpleRectPipeline {
    fn draw(&mut self, context: &mut DrawContext<SimpleRectCommand>) {
        if context.commands.is_empty() {
            return;
        }
        let instances = build_instances(context.commands, context.target_size);
        if instances.is_empty() {
            return;
        }

        let uniforms = RectInstances { instances };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&uniforms)
            .expect("buffer write failed");
        let bytes = buffer_content.into_inner();

        // Each draw gets its own slice of the shared staging buffer. Writing
        // every draw at offset 0 would make all queued draws read the last
        // command's data, so the bind group binds this draw's sub-range.
        let alignment =
            u64::from(context.device.limits().min_storage_buffer_offset_alignment).max(1);
        let offset = self.staging_len.next_multiple_of(alignment);
        let required = offset + bytes.len() as u64;
        self.ensure_staging(context.device, required);

        let staging = self
            .staging_buffer
            .as_ref()
            .expect("staging buffer is created by ensure_staging");
        context.queue.write_buffer(staging, offset, &bytes);
        self.staging_len = required;

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: staging,
                        offset,
                        size: NonZeroU64::new(bytes.len() as u64),
                    }),
                }],
                label: Some("simple_rect_bind_group"),
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

    fn end_frame(&mut self, _context: &FrameContext<'_>) {
        // Clear the logical length only. Capacity and the bind group stay put,
        // and nothing here keeps the frame encoder or render pass alive.
        self.staging_len = 0;
    }
}
