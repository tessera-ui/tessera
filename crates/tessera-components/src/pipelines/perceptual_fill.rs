use tessera_ui::{
    Color, Dp, Px,
    render::{DrawCommand, DrawContext, DrawRegion, DrawablePipeline, PaddingRect, SampleRegion},
    wgpu,
};

#[derive(Clone, PartialEq)]
pub(crate) struct PerceptualFillCommand {
    pub color: Color,
    pub radii: [Dp; 2],
    pub insets: [Dp; 4],
    pub clip_radius: Option<Dp>,
    pub edge_smoothness: f32,
    pub stroke: Option<PerceptualStroke>,
    pub path: Option<PerceptualPath>,
    pub line: Option<PerceptualLine>,
    pub outline_width: Dp,
    pub snap_to_pixels: bool,
}

#[derive(Clone, PartialEq)]
pub(crate) struct PerceptualLine {
    pub vertical: bool,
    pub width: Dp,
    pub offset: Dp,
}

#[derive(Clone, PartialEq)]
pub(crate) struct PerceptualStroke {
    pub color: Color,
    pub width: Dp,
    pub points: [[f32; 2]; 3],
    pub raster_mask: bool,
}

#[derive(Clone, PartialEq)]
pub(crate) struct PerceptualPath {
    pub color: Color,
    pub width: Dp,
    pub points: [[f32; 2]; 13],
    pub dot: [f32; 2],
    pub dot_radius: Dp,
}

#[derive(Clone, PartialEq)]
pub(crate) struct PerceptualSpokesCommand {
    pub tint: Color,
    pub inner: Dp,
    pub outer: Dp,
    pub radius: Dp,
    pub edge_width_px: f32,
    pub weights: [f32; 8],
}

impl DrawCommand for PerceptualSpokesCommand {
    fn sample_region(&self) -> Option<SampleRegion> {
        Some(SampleRegion::PaddedLocal(PaddingRect::uniform(Px::ZERO)))
    }

    fn draw_region(&self) -> DrawRegion {
        DrawRegion::PaddedLocal(PaddingRect::uniform(Px::ZERO))
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.tint.a *= opacity;
    }
}

impl DrawCommand for PerceptualFillCommand {
    fn sample_region(&self) -> Option<SampleRegion> {
        Some(SampleRegion::PaddedLocal(self.padding()))
    }

    fn draw_region(&self) -> DrawRegion {
        DrawRegion::PaddedLocal(self.padding())
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.color.a *= opacity;
        if let Some(stroke) = &mut self.stroke {
            stroke.color.a *= opacity;
        }
        if let Some(path) = &mut self.path {
            path.color.a *= opacity;
        }
    }
}

impl PerceptualFillCommand {
    fn padding(&self) -> PaddingRect {
        PaddingRect::uniform(
            self.line
                .as_ref()
                .map_or(if self.snap_to_pixels { Px(1) } else { Px::ZERO }, |line| {
                    Px::from(line.width) + Px::from(Dp(line.offset.0.abs())) + Px(1)
                }),
        )
    }
}

pub(crate) struct PerceptualFillPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

impl PerceptualFillPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, samples: u32) -> Self {
        Self::with_fragment(device, format, samples, "fs_main")
    }

    fn with_fragment(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        samples: u32,
        entry_point: &str,
    ) -> Self {
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
                entry_point: Some(entry_point),
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

    fn draw_quad<T: DrawCommand>(
        &self,
        context: &mut DrawContext<'_, '_, '_, T>,
        uniforms: &[[f32; 4]; 17],
    ) {
        let buffer = context
            .graphics
            .frame_buffer(wgpu::BufferUsages::UNIFORM, bytemuck::cast_slice(uniforms));
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
                        resource: wgpu::BindingResource::TextureView(context.scene_texture_view),
                    },
                ],
            });
        context.render_pass.set_pipeline(&self.pipeline);
        context.render_pass.set_bind_group(0, &bindings, &[]);
        context.render_pass.draw(0..6, 0..1);
    }
}

impl DrawablePipeline<PerceptualFillCommand> for PerceptualFillPipeline {
    fn draw(&mut self, context: &mut DrawContext<'_, '_, '_, PerceptualFillCommand>) {
        for (command, geometry) in context.commands {
            let mut origin = geometry.window_origin();
            let mut size = geometry.window_size();
            if command.snap_to_pixels {
                // Flipped raster coordinates need opposite tie directions on the two axes.
                origin.x = origin.x.round();
                origin.y = origin.y.floor();
            }
            if let Some(line) = &command.line {
                if size[0] <= 0.0 || size[1] <= 0.0 {
                    continue;
                }
                let scale = geometry.local_to_window.scale();
                let stroke = line.width.to_pixels_f32() * scale;
                let offset = line.offset.to_pixels_f32() * scale;
                if line.vertical {
                    origin.x = (origin.x + size[0] * 0.5).floor() + offset;
                    size[0] = stroke;
                    size[1] = (origin.y + size[1]).floor() - origin.y.floor();
                    origin.y = origin.y.floor();
                } else {
                    origin.y = (origin.y + size[1] * 0.5).floor() + offset;
                    size[1] = stroke;
                    size[0] = (origin.x + size[0]).floor() - origin.x.floor();
                    origin.x = origin.x.floor();
                }
            }
            let raster_rect = [origin.x, origin.y, size[0], size[1]];
            let [top, right, bottom, left] = command
                .insets
                .map(|inset| inset.to_pixels_f32() * geometry.local_to_window.scale());
            origin.x += left;
            origin.y += top;
            size[0] = (size[0] - left - right).max(0.0);
            size[1] = (size[1] - top - bottom).max(0.0);
            if command.snap_to_pixels {
                size = size.map(f32::round);
                if size[0] <= 0.0 || size[1] <= 0.0 {
                    continue;
                }
            }
            let radii = std::array::from_fn::<_, 2, _>(|axis| {
                (command.radii[axis].to_pixels_f32() * geometry.local_to_window.scale()).min(
                    if command.radii[0] == command.radii[1] {
                        size[0].min(size[1]) * 0.5
                    } else {
                        size[1] * 0.5
                    },
                )
            });
            let base_uniforms = [
                raster_rect,
                [origin.x, origin.y, size[0], size[1]],
                [
                    context.target_size.width.to_f32(),
                    context.target_size.height.to_f32(),
                    context.scene_rect.x.to_f32(),
                    context.scene_rect.y.to_f32(),
                ],
                command.color.to_array(),
                [
                    radii[0],
                    radii[1],
                    command.edge_smoothness,
                    command.clip_radius.map_or(-1.0, |radius| {
                        (radius.to_pixels_f32() * geometry.local_to_window.scale())
                            .min(raster_rect[2].min(raster_rect[3]) * 0.5)
                    }),
                ],
                command
                    .stroke
                    .as_ref()
                    .map(|stroke| stroke.color.to_array())
                    .or_else(|| command.path.as_ref().map(|path| path.color.to_array()))
                    .unwrap_or([0.0; 4]),
                command.stroke.as_ref().map_or([0.0; 4], |stroke| {
                    [
                        stroke.points[0][0],
                        stroke.points[0][1],
                        stroke.points[1][0],
                        stroke.points[1][1],
                    ]
                }),
                command.stroke.as_ref().map_or([0.0; 4], |stroke| {
                    [
                        stroke.points[2][0],
                        stroke.points[2][1],
                        stroke.width.to_pixels_f32() * geometry.local_to_window.scale(),
                        if stroke.raster_mask { 1.0 } else { 0.0 },
                    ]
                }),
                [
                    command.outline_width.to_pixels_f32() * geometry.local_to_window.scale(),
                    0.0,
                    0.0,
                    0.0,
                ],
            ];
            let mut uniforms = [[0.0; 4]; 17];
            uniforms[..9].copy_from_slice(&base_uniforms);
            if let Some(path) = &command.path {
                for (index, point) in path.points.iter().enumerate() {
                    uniforms[9 + index / 2][index % 2 * 2] = point[0];
                    uniforms[9 + index / 2][index % 2 * 2 + 1] = point[1];
                }
                uniforms[15][2..].copy_from_slice(&path.dot);
                let scale = geometry.local_to_window.scale();
                uniforms[16] = [
                    path.width.to_pixels_f32() * scale,
                    path.dot_radius.to_pixels_f32() * scale,
                    1.0,
                    0.0,
                ];
            }
            self.draw_quad(context, &uniforms);
        }
    }
}

pub(crate) struct PerceptualSpokesPipeline(PerceptualFillPipeline);

impl PerceptualSpokesPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, samples: u32) -> Self {
        Self(PerceptualFillPipeline::with_fragment(
            device,
            format,
            samples,
            "fs_spokes",
        ))
    }
}

impl DrawablePipeline<PerceptualSpokesCommand> for PerceptualSpokesPipeline {
    fn draw(&mut self, context: &mut DrawContext<'_, '_, '_, PerceptualSpokesCommand>) {
        for (command, geometry) in context.commands {
            let [width, height] = geometry.window_size();
            if width <= 0.0 || height <= 0.0 {
                continue;
            }
            let origin = geometry.window_origin();
            let scale = geometry.local_to_window.scale();
            let rect = [origin.x, origin.y, width, height];
            let mut uniforms = [[0.0; 4]; 17];
            uniforms[0] = rect;
            uniforms[1] = rect;
            uniforms[2] = [
                context.target_size.width.to_f32(),
                context.target_size.height.to_f32(),
                context.scene_rect.x.to_f32(),
                context.scene_rect.y.to_f32(),
            ];
            uniforms[3] = command.tint.to_array();
            uniforms[4] = [
                command.inner.to_pixels_f32() * scale,
                command.outer.to_pixels_f32() * scale,
                command.radius.to_pixels_f32() * scale,
                command.edge_width_px,
            ];
            uniforms[5].copy_from_slice(&command.weights[..4]);
            uniforms[6].copy_from_slice(&command.weights[4..]);
            self.0.draw_quad(context, &uniforms);
        }
    }
}
