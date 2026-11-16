use tessera_ui::{
    Color, Dp,
    render::{DrawCommand, DrawContext, DrawablePipeline, SampleRegion},
    wgpu,
};

#[derive(Clone, PartialEq)]
pub(crate) struct PerceptualFillCommand {
    pub color: Color,
    pub radius: Dp,
}

impl DrawCommand for PerceptualFillCommand {
    fn sample_region(&self) -> Option<SampleRegion> {
        Some(SampleRegion::ZERO_PADDING_LOCAL)
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.color.a *= opacity;
    }
}

pub(crate) struct PerceptualFillPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

impl PerceptualFillPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, samples: u32) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Perceptual fill"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Perceptual fill"),
            source: wgpu::ShaderSource::Wgsl(include_str!("perceptual_fill.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Perceptual fill"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Perceptual fill"),
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
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: tessera_ui::render::ui_stencil_state(),
            multisample: wgpu::MultisampleState {
                count: samples,
                ..Default::default()
            },
            multiview_mask: None,
            cache: None,
        });
        Self { pipeline, layout }
    }
}

impl DrawablePipeline<PerceptualFillCommand> for PerceptualFillPipeline {
    fn draw(&mut self, context: &mut DrawContext<'_, '_, '_, PerceptualFillCommand>) {
        for (command, geometry) in context.commands {
            let origin = geometry.window_origin();
            let size = geometry.window_size();
            let radius = (command.radius.to_pixels_f32() * geometry.local_to_window.scale())
                .min(size[0].min(size[1]) * 0.5);
            let uniforms = [
                [origin.x, origin.y, size[0], size[1]],
                [
                    context.target_size.width.to_f32(),
                    context.target_size.height.to_f32(),
                    context.scene_rect.x.to_f32(),
                    context.scene_rect.y.to_f32(),
                ],
                command.color.to_array(),
                [radius, 0.0, 0.0, 0.0],
            ];
            let buffer = context
                .graphics
                .frame_buffer(wgpu::BufferUsages::UNIFORM, bytemuck::cast_slice(&uniforms));
            let bindings = context
                .graphics
                .cached_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Perceptual fill"),
                    layout: &self.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: buffer.binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                context.scene_texture_view,
                            ),
                        },
                    ],
                });
            context.render_pass.set_pipeline(&self.pipeline);
            context.render_pass.set_bind_group(0, &bindings, &[]);
            context.render_pass.draw(0..6, 0..1);
        }
    }
}
