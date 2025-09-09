use std::{sync::OnceLock, time::Instant};

use bytemuck::{Pod, Zeroable};
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, DrawCommand, DrawablePipeline, Px, PxPosition,
    PxSize, tessera,
    wgpu::{self, util::DeviceExt},
};
use tessera_ui_basic_components::surface::{SurfaceArgsBuilder, surface};

use crate::CalStyle;

#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundCommand {
    pub time: f32,
}

impl DrawCommand for BackgroundCommand {
    fn barrier(&self) -> Option<tessera_ui::BarrierRequirement> {
        None
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    time: f32,
    width: f32,
    height: f32,
    _padding: u32,
}

pub struct BackgroundPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl BackgroundPipeline {
    pub fn new(gpu: &wgpu::Device, config: &wgpu::SurfaceConfiguration, sample_count: u32) -> Self {
        let shader = gpu.create_shader_module(wgpu::include_wgsl!("shaders/background.wgsl"));
        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("background_bind_group_layout"),
        });

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Background Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Background Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // Assuming a passthrough vertex shader
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                cull_mode: None,
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
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

impl DrawablePipeline<BackgroundCommand> for BackgroundPipeline {
    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        _queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass,
        commands: &[(&BackgroundCommand, PxSize, PxPosition)],
        _scene_texture_view: &wgpu::TextureView,
    ) {
        if let Some((command, size, _)) = commands.first() {
            let uniforms = Uniforms {
                time: command.time,
                width: size.width.to_f32(),
                height: size.height.to_f32(),
                _padding: 0,
            };
            let uniform_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Background Uniform Buffer"),
                contents: bytemuck::bytes_of(&uniforms),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
                label: Some("background_bind_group"),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..4, 0..1);
        }
    }
}

static START_AT: OnceLock<Instant> = OnceLock::new();

/// Resolve a DimensionValue into a concrete Px value, using sensible defaults
/// for Wrap/Fill when max is not provided.
fn resolve_dimension(value: &DimensionValue) -> Px {
    match value {
        DimensionValue::Fixed(v) => *v,
        DimensionValue::Wrap { max, .. } => max.unwrap_or(Px(0)),
        DimensionValue::Fill { max, .. } => max.unwrap_or(Px(0)),
    }
}

/// Return the elapsed time in seconds since the pipeline started.
fn current_time() -> f32 {
    START_AT.get_or_init(Instant::now).elapsed().as_secs_f32()
}

#[tessera]
pub fn background(child: impl FnOnce(), style: CalStyle) {
    match style {
        CalStyle::Glass => {
            child();

            measure(Box::new(move |input| {
                let width = resolve_dimension(&input.parent_constraint.width);
                let height = resolve_dimension(&input.parent_constraint.height);

                let time = current_time();

                input
                    .metadata_mut()
                    .push_draw_command(BackgroundCommand { time });

                let child_constraint =
                    Constraint::new(DimensionValue::Fixed(width), DimensionValue::Fixed(height));

                if let Some(child_id) = input.children_ids.first() {
                    input.measure_child(*child_id, &child_constraint)?;
                    input.place_child(*child_id, [0, 0].into());
                }

                Ok(ComputedData { width, height })
            }));
        }
        CalStyle::Material => {
            surface(
                SurfaceArgsBuilder::default()
                    .style(Color::WHITE.into())
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                None,
                || {
                    child();
                },
            );
        }
    }
}
