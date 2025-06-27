use bytemuck::{Pod, Zeroable};
use tessera::renderer::{DrawCommand, DrawablePipeline, RenderRequirement};
use tessera::{Px, PxPosition};

use crate::pipelines::pos_misc::pixel_to_ndc;

// --- Command Definition ---

pub struct GlassCommand;

impl DrawCommand for GlassCommand {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn requirement(&self) -> RenderRequirement {
        RenderRequirement::SamplesBackground
    }
}

// --- Pipeline Definition ---

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Uniforms {
    pos: [f32; 2],
    size: [f32; 2],
    _padding: [f32; 4],
}

pub struct GlassPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl GlassPipeline {
    pub fn new(
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        scene_sampler_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = gpu.create_shader_module(wgpu::include_wgsl!("glass/glass.wgsl"));

        let uniform_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Glass Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Glass Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Glass Bind Group"),
        });

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Glass Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout, scene_sampler_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Glass Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group,
        }
    }
}

#[allow(unused_variables)]
impl DrawablePipeline<GlassCommand> for GlassPipeline {
    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass,
        command: &GlassCommand,
        size: [Px; 2],
        start_pos: PxPosition,
    ) {
        let screen_size = [config.width, config.height];

        let ndc_pos = pixel_to_ndc(start_pos, screen_size);
        let ndc_size = [
            size[0].0 as f32 / screen_size[0] as f32 * 2.0,
            size[1].0 as f32 / screen_size[1] as f32 * 2.0,
        ];

        let uniforms = Uniforms {
            pos: ndc_pos,
            size: ndc_size,
            _padding: [0.0; 4],
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
