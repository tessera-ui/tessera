use std::{mem, time::Duration};

use tessera_components::{
    column::column, lazy_list::lazy_column, row::row, slider::slider, spacer::spacer, text::text,
    theme::MaterialTheme,
};
use tessera_foundation::modifier::ModifierExt as _;
use tessera_shard::shard;
use tessera_ui::{
    Dp, EntryRegistry, Modifier, State, TesseraPackage, launch_once,
    layout::layout,
    px::{Px, PxRect, PxSize},
    remember,
    render::{DrawCommand, DrawContext, DrawablePipeline, PipelineContext, RenderModule},
    tessera, use_context, wgpu, with_frame_time,
};

use crate::res::I18N;

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
        .content_padding(Dp(24.0))
        .item_spacing(Dp(16.0))
        .item(move || {
            text(I18N.text("ui-custom-shader"))
                .style(theme.with(|t| t.typography.headline_large))
                .color(theme.with(|t| t.color_scheme.on_surface));
        })
        .item(move || {
            text(I18N.text("ui-a-ray-tracing-demo"))
                .style(theme.with(|t| t.typography.body_large))
                .color(theme.with(|t| t.color_scheme.on_surface_variant))
                .modifier(Modifier::new().size_in(None, Some(Dp(720.0)), None, None));
        })
        .item(move || {
            raymarch_canvas(
                Modifier::new()
                    .size(RAYMARCH_CANVAS_WIDTH, RAYMARCH_CANVAS_HEIGHT)
                    .clip_shape(theme.with(|t| t.shapes.extra_large)),
                time_scale_slider,
            );
        })
        .item(move || {
            column()
                .modifier(Modifier::new().width(RAYMARCH_CANVAS_WIDTH))
                .children(move || {
                    row().children(move || {
                        text(I18N.text("ui-animation-speed"))
                            .style(theme.with(|t| t.typography.title_small))
                            .color(theme.with(|t| t.color_scheme.on_surface));
                        spacer().modifier(Modifier::new().weight(1.0));
                        text(I18N.format(
                            "ui-time-scale-valuex",
                            [("v0", format!("{:.2}", time_scale).into())],
                        ))
                        .style(theme.with(|t| t.typography.label_large))
                        .color(theme.with(|t| t.color_scheme.on_surface_variant));
                    });
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    slider(time_scale_slider.get())
                        .modifier(Modifier::new().fill_max_width())
                        .accessibility_label(I18N.text("ui-animation-speed"))
                        .on_change(move |value| time_scale_slider.set(value.clamp(0.0, 1.0)));
                });
        });
}

#[tessera]
fn raymarch_canvas(modifier: Modifier, time_scale_state: State<f32>) {
    let elapsed = remember(|| Duration::ZERO);

    launch_once(async move {
        let mut start = None;
        loop {
            with_frame_time(|now| {
                let start = *start.get_or_insert(now);
                elapsed.set(now.saturating_duration_since(start));
            })
            .await;
        }
    });

    let time_scale = 0.2 + time_scale_state.get().clamp(0.0, 1.0) * 2.8;
    let elapsed_seconds = elapsed.get().as_secs_f32();
    let time_seconds = elapsed_seconds * time_scale;
    let camera_position = CAMERA_POSITION;
    let (camera_forward, camera_right, camera_up) =
        camera_basis_look_at(camera_position, CAMERA_TARGET);

    let policy = RaymarchCommand {
        time_seconds,
        opacity: 1.0,
        camera_position,
        camera_forward,
        camera_right,
        camera_up,
    };

    layout(tessera_ui::layout::Stack)
        .modifier(modifier)
        .draw(policy);
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

impl tessera_ui::DrawModifierNode for RaymarchCommand {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let input = &mut *ctx.render_input;
        input.metadata_mut().fragment_mut().push_draw_command(*self);
        content.draw(input);
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
            depth_stencil: tessera_ui::render::ui_stencil_state(),
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
        geometry: tessera_ui::render::DrawGeometry,
        target_size: PxSize,
    ) {
        let position = geometry.window_origin();
        let [width, height] = geometry.window_size();
        let values = [
            position.x,
            position.y,
            width,
            height,
            0.0,
            0.0,
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

        for (command, geometry) in context.commands {
            let Some((x, y, width, height)) =
                resolve_scissor_rect(*geometry, context.clip_rect, context.target_size)
            else {
                continue;
            };

            self.write_uniforms(context.queue, command, *geometry, context.target_size);
            context.render_pass.set_scissor_rect(x, y, width, height);
            context.render_pass.draw(0..6, 0..1);
        }
    }
}

fn resolve_scissor_rect(
    geometry: tessera_ui::render::DrawGeometry,
    clip_rect: Option<PxRect>,
    target_size: PxSize,
) -> Option<(u32, u32, u32, u32)> {
    let viewport = PxRect::new(Px::ZERO, Px::ZERO, target_size.width, target_size.height);
    let base = geometry.window_bounds().outward();

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
