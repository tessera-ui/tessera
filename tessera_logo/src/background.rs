use bytemuck::{Pod, Zeroable};
use derive_builder::Builder;
use std::sync::Arc;
use tessera::{
    place_node,
    wgpu::{self, util::DeviceExt},
    winit::window::CursorIcon,
    ComputedData, Constraint, CursorEventContent, DimensionValue, DrawCommand, DrawablePipeline,
    PressKeyEventType, Px, PxPosition, PxSize, StateHandlerInput,
};
use tessera_basic_components::{
    alignment::Alignment, boxed::BoxedItem, pos_misc::is_position_in_component,
};
use tessera_macros::tessera;

#[derive(Clone, Debug)]
pub struct BackgroundCommand {
    pub time: f32,
}

impl DrawCommand for BackgroundCommand {
    fn barrier(&self) -> Option<tessera::BarrierRequirement> {
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
        let shader = gpu.create_shader_module(wgpu::include_wgsl!("../shaders/background.wgsl"));
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
        command: &BackgroundCommand,
        size: PxSize,
        _start_pos: PxPosition,
        _scene_texture_view: Option<&wgpu::TextureView>,
        _compute_texture_view: &wgpu::TextureView,
    ) {
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

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct BackgroundArgs {
    #[builder(default)]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    #[builder(default = "0.0")]
    pub time: f32,
    #[builder(default)]
    pub alignment: Alignment,
}

#[tessera]
pub fn background<const N: usize>(args: BackgroundArgs, children: [BoxedItem; N]) {
    let args_clone = Arc::new(args);
    let args_for_state = args_clone.clone();

    measure(Box::new(move |input| {
        let args = &args_clone;
        let width = match input.parent_constraint.width {
            DimensionValue::Fixed(v) => v,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(Px(0)),
            DimensionValue::Fill { max, .. } => max.unwrap_or(Px(0)),
        };
        let height = match input.parent_constraint.height {
            DimensionValue::Fixed(v) => v,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(Px(0)),
            DimensionValue::Fill { max, .. } => max.unwrap_or(Px(0)),
        };

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.push_draw_command(BackgroundCommand { time: args.time });
        }

        let child_constraint =
            Constraint::new(DimensionValue::Fixed(width), DimensionValue::Fixed(height));

        if N > 0 {
            for i in 0..input.children_ids.len() {
                let child_id = input.children_ids[i];
                let child_size = tessera::measure_node(
                    child_id,
                    &child_constraint,
                    input.tree,
                    input.metadatas,
                )?;

                let (x, y) = match args.alignment {
                    Alignment::TopStart => (Px(0), Px(0)),
                    Alignment::TopCenter => ((width - child_size.width) / 2, Px(0)),
                    Alignment::TopEnd => (width - child_size.width, Px(0)),
                    Alignment::CenterStart => (Px(0), (height - child_size.height) / 2),
                    Alignment::Center => (
                        (width - child_size.width) / 2,
                        (height - child_size.height) / 2,
                    ),
                    Alignment::CenterEnd => {
                        (width - child_size.width, (height - child_size.height) / 2)
                    }
                    Alignment::BottomStart => (Px(0), height - child_size.height),
                    Alignment::BottomCenter => {
                        ((width - child_size.width) / 2, height - child_size.height)
                    }
                    Alignment::BottomEnd => (width - child_size.width, height - child_size.height),
                };

                place_node(child_id, PxPosition::new(x, y), input.metadatas);
            }
        }

        Ok(ComputedData { width, height })
    }));

    state_handler(Box::new(move |input: StateHandlerInput| {
        let args = &args_for_state;
        if let Some(on_click) = &args.on_click {
            let size = input.computed_data;
            let is_cursor_in = input
                .cursor_position
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);

            if is_cursor_in {
                input.requests.cursor_icon = CursorIcon::Pointer;

                let is_released = input.cursor_events.iter().any(|event| {
                    matches!(
                        event.content,
                        CursorEventContent::Released(PressKeyEventType::Left)
                    )
                });

                if is_released {
                    on_click();
                    input.cursor_events.clear();
                }
            }
        }
    }));

    let mut child_closures = Vec::with_capacity(N);
    for child_item in children {
        child_closures.push(child_item.child);
    }
    (child_closures.into_iter()).for_each(|c| c());
}
