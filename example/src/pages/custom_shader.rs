use std::mem;

use tessera_components::{
    lazy_list::lazy_column, modifier::ModifierExt, slider::slider, text::text, theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{
    Dp, DrawCommand, EntryRegistry, FrameNanosControl, Modifier, PipelineContext, RenderInput,
    RenderModule, RenderPolicy, State, TesseraPackage,
    layout::layout,
    px::{Px, PxPosition, PxRect, PxSize},
    receive_frame_nanos, remember,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline},
    tessera, use_context, wgpu,
};

const RAYMARCH_CANVAS_WIDTH: Dp = Dp(560.0);
const RAYMARCH_CANVAS_HEIGHT: Dp = Dp(320.0);
const RAYMARCH_UNIFORM_FLOATS: usize = 28;
const RAYMARCH_UNIFORM_SIZE: usize = RAYMARCH_UNIFORM_FLOATS * mem::size_of::<f32>();
const CAMERA_TARGET: [f32; 3] = [0.0, 0.98, 6.7];
const CAMERA_POSITION: [f32; 3] = [0.0, 1.62, 10.1];

fn vec3_length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let length = vec3_length(v);
    if length <= f32::EPSILON {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / length, v[1] / length, v[2] / length]
}

fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn camera_basis_look_at(
    camera_position: [f32; 3],
    target_position: [f32; 3],
) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let forward = vec3_normalize([
        target_position[0] - camera_position[0],
        target_position[1] - camera_position[1],
        target_position[2] - camera_position[2],
    ]);
    let right = vec3_normalize(vec3_cross([0.0, 1.0, 0.0], forward));
    let up = vec3_normalize(vec3_cross(forward, right));

    (forward, right, up)
}

#[shard]
pub fn custom_shader_page() {
    let theme = use_context::<MaterialTheme>().unwrap();
    let time_scale_slider = remember(|| 0.35_f32);
    let time_scale = 0.2 + time_scale_slider.get().clamp(0.0, 1.0) * 2.8;

    lazy_column()
        .modifier(Modifier::new().fill_max_size())
        .estimated_item_size(Dp(120.0))
        .content_padding(Dp(16.0))
        .item_spacing(Dp(14.0))
        .item(move || {
            text()
                .content("Custom Shader")
                .style(theme.with(|t| t.typography.headline_large));
        })
        .item(move || {
            text().content("A ray tracing demo.");
        })
        .item(move || {
            raymarch_canvas(
                Modifier::new()
                    .size(RAYMARCH_CANVAS_WIDTH, RAYMARCH_CANVAS_HEIGHT)
                    .clip_to_bounds()
                    .border(
                        Dp(1.0),
                        theme.with(|t| t.color_scheme.outline.with_alpha(0.7)),
                    ),
                time_scale_slider,
            );
        })
        .item(move || {
            text().content(format!("Time scale {:.2}x", time_scale));
        })
        .item(move || {
            text().content("Animation Speed");
        })
        .item(move || {
            slider()
                .value(time_scale_slider.get())
                .on_change(move |value| time_scale_slider.set(value.clamp(0.0, 1.0)));
        });
}

#[tessera]
fn raymarch_canvas(modifier: Modifier, time_scale_state: State<f32>) {
    let frame_nanos_state = remember(|| 0_u64);

    receive_frame_nanos(move |frame_nanos| {
        frame_nanos_state.set(frame_nanos);
        FrameNanosControl::Continue
    });

    let time_scale = 0.2 + time_scale_state.get().clamp(0.0, 1.0) * 2.8;
    let elapsed_seconds = frame_nanos_state.get() as f32 / 1_000_000_000.0;
    let time_seconds = elapsed_seconds * time_scale;
    let camera_position = CAMERA_POSITION;
    let (camera_forward, camera_right, camera_up) =
        camera_basis_look_at(camera_position, CAMERA_TARGET);

    let policy = RaymarchRenderPolicy {
        time_seconds,
        mouse: [0.0, 0.0],
        camera_position,
        camera_forward,
        camera_right,
        camera_up,
    };

    layout().modifier(modifier).render_policy(policy);
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct CustomShaderPackage;

impl TesseraPackage for CustomShaderPackage {
    fn register(self, registry: &mut EntryRegistry) {
        registry.add_module(CustomShaderModule);
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
struct CustomShaderModule;

impl RenderModule for CustomShaderModule {
    fn register_pipelines(&self, context: &mut PipelineContext<'_>) {
        let resources = context.resources();
        let pipeline = RaymarchPipeline::new(
            resources.device,
            resources.surface_config,
            resources.pipeline_cache,
            resources.sample_count,
        );
        context.register_draw_pipeline(pipeline);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RaymarchCommand {
    time_seconds: f32,
    mouse: [f32; 2],
    camera_position: [f32; 3],
    camera_forward: [f32; 3],
    camera_right: [f32; 3],
    camera_up: [f32; 3],
    opacity: f32,
}

impl DrawCommand for RaymarchCommand {
    fn apply_opacity(&mut self, opacity: f32) {
        self.opacity *= opacity.clamp(0.0, 1.0);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RaymarchRenderPolicy {
    time_seconds: f32,
    mouse: [f32; 2],
    camera_position: [f32; 3],
    camera_forward: [f32; 3],
    camera_right: [f32; 3],
    camera_up: [f32; 3],
}

impl RenderPolicy for RaymarchRenderPolicy {
    fn record(&self, input: &mut RenderInput<'_>) {
        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(RaymarchCommand {
                time_seconds: self.time_seconds,
                mouse: self.mouse,
                camera_position: self.camera_position,
                camera_forward: self.camera_forward,
                camera_right: self.camera_right,
                camera_up: self.camera_up,
                opacity: 1.0,
            });
    }
}

struct RaymarchPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

impl RaymarchPipeline {
    fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Example Ray Tracing Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("custom_shader_raymarch.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("example_raymarch_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("example_raymarch_pipeline_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("example_raymarch_pipeline"),
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("example_raymarch_uniform_buffer"),
            size: RAYMARCH_UNIFORM_SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("example_raymarch_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            bind_group,
            uniform_buffer,
        }
    }

    fn write_uniforms(
        &self,
        queue: &wgpu::Queue,
        command: &RaymarchCommand,
        position: PxPosition,
        size: PxSize,
        target_size: PxSize,
    ) {
        let values = [
            position.x.raw() as f32,
            position.y.raw() as f32,
            size.width.raw() as f32,
            size.height.raw() as f32,
            command.mouse[0],
            command.mouse[1],
            command.time_seconds,
            command.opacity,
            target_size.width.to_f32(),
            target_size.height.to_f32(),
            0.0,
            0.0,
            command.camera_position[0],
            command.camera_position[1],
            command.camera_position[2],
            0.0,
            command.camera_forward[0],
            command.camera_forward[1],
            command.camera_forward[2],
            0.0,
            command.camera_right[0],
            command.camera_right[1],
            command.camera_right[2],
            0.0,
            command.camera_up[0],
            command.camera_up[1],
            command.camera_up[2],
            0.0,
        ];

        let mut bytes = [0_u8; RAYMARCH_UNIFORM_SIZE];
        for (index, value) in values.iter().enumerate() {
            let start = index * mem::size_of::<f32>();
            let end = start + mem::size_of::<f32>();
            bytes[start..end].copy_from_slice(&value.to_le_bytes());
        }

        queue.write_buffer(&self.uniform_buffer, 0, &bytes);
    }
}

impl DrawablePipeline<RaymarchCommand> for RaymarchPipeline {
    fn draw(&mut self, context: &mut DrawContext<RaymarchCommand>) {
        if context.commands.is_empty() {
            return;
        }

        context.render_pass.set_pipeline(&self.pipeline);
        context.render_pass.set_bind_group(0, &self.bind_group, &[]);

        for (command, size, position) in context.commands {
            let Some((x, y, width, height)) =
                resolve_scissor_rect(*position, *size, context.clip_rect, context.target_size)
            else {
                continue;
            };

            self.write_uniforms(
                context.queue,
                command,
                *position,
                *size,
                context.target_size,
            );
            context.render_pass.set_scissor_rect(x, y, width, height);
            context.render_pass.draw(0..6, 0..1);
        }
    }
}

fn resolve_scissor_rect(
    position: PxPosition,
    size: PxSize,
    clip_rect: Option<PxRect>,
    target_size: PxSize,
) -> Option<(u32, u32, u32, u32)> {
    let viewport = PxRect::new(Px::ZERO, Px::ZERO, target_size.width, target_size.height);
    let base = PxRect::from_position_size(position, size);

    let clipped = match clip_rect {
        Some(clip) => base.intersection(&clip)?,
        None => base,
    }
    .intersection(&viewport)?;

    if clipped.width.raw() <= 0 || clipped.height.raw() <= 0 {
        return None;
    }

    Some((
        clipped.x.positive(),
        clipped.y.positive(),
        clipped.width.positive(),
        clipped.height.positive(),
    ))
}
