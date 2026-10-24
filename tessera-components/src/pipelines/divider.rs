//! Fractional divider strokes with analytic pixel coverage.
//!
//! ## Usage
//!
//! Render thin separators consistently across display densities.

use bytemuck::{Pod, Zeroable};
use tessera_ui::{
    Color, DrawCommand, DrawRegion, PaddingRect, Px,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline, FrameContext},
    wgpu::{self, include_wgsl},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DividerCommand {
    pub color: Color,
    pub stroke: f32,
    pub vertical: bool,
}

impl DrawCommand for DividerCommand {
    fn draw_region(&self) -> DrawRegion {
        DrawRegion::PaddedLocal(PaddingRect::uniform(Px(1)))
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.color = self
            .color
            .with_alpha(self.color.a * opacity.clamp(0.0, 1.0));
    }
}

#[cfg(test)]
pub(crate) static ALLOCATIONS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    bounds: [f32; 4],
    color: [f32; 4],
    screen: [f32; 4],
}

struct Page {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: usize,
}

pub(crate) struct DividerPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    pages: Vec<Page>,
    page: usize,
    offset: usize,
    max_instances: usize,
    staging: Vec<Instance>,
}

impl DividerPipeline {
    pub(crate) fn new(
        device: &wgpu::Device,
        cache: Option<&wgpu::PipelineCache>,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Divider Instances"),
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
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Divider Pipeline Layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(include_wgsl!("divider/divider.wgsl"));
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Divider Pipeline"),
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
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: tessera_ui::ui_stencil_state(),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            multiview_mask: None,
            cache,
        });
        let limits = device.limits();
        let max_bytes = limits
            .max_buffer_size
            .min(u64::from(limits.max_storage_buffer_binding_size));
        Self {
            pipeline,
            layout,
            pages: Vec::new(),
            page: 0,
            offset: 0,
            max_instances: (max_bytes / size_of::<Instance>() as u64).min(u64::from(u32::MAX))
                as usize,
            staging: Vec::new(),
        }
    }

    fn reserve(&mut self, device: &wgpu::Device, count: usize) {
        if self
            .pages
            .get(self.page)
            .is_some_and(|page| self.offset > 0 && self.offset + count > page.capacity)
        {
            self.page += 1;
            self.offset = 0;
        }
        if self
            .pages
            .get(self.page)
            .is_some_and(|page| page.capacity >= count)
        {
            return;
        }
        let capacity = count.max(1024).next_power_of_two().min(self.max_instances);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Divider Instance Page"),
            size: (capacity * size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Divider Instance Page"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        let page = Page {
            buffer,
            bind_group,
            capacity,
        };
        #[cfg(test)]
        ALLOCATIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if self.page < self.pages.len() {
            self.pages[self.page] = page;
        } else {
            self.pages.push(page);
        }
    }
}

impl DrawablePipeline<DividerCommand> for DividerPipeline {
    fn begin_frame(&mut self, _: &FrameContext<'_>) {
        self.page = 0;
        self.offset = 0;
    }

    fn draw(&mut self, context: &mut DrawContext<DividerCommand>) {
        for commands in context.commands.chunks(self.max_instances) {
            self.reserve(context.device, commands.len());
            self.staging.clear();
            self.staging
                .extend(commands.iter().map(|(command, geometry)| {
                    let position = geometry.window_origin();
                    let [width, height] = geometry.window_size();
                    let scale = geometry.local_to_window.scale();
                    Instance {
                        bounds: [
                            position.x,
                            position.y,
                            position.x
                                + if command.vertical {
                                    command.stroke * scale
                                } else {
                                    width
                                },
                            position.y
                                + if command.vertical {
                                    height
                                } else {
                                    command.stroke * scale
                                },
                        ],
                        color: command.color.to_array(),
                        screen: [
                            context.target_size.width.to_f32(),
                            context.target_size.height.to_f32(),
                            0.0,
                            0.0,
                        ],
                    }
                }));
            let page = &self.pages[self.page];
            // Later batches and scene passes must not overwrite earlier draws
            // before this frame is submitted.
            context.queue.write_buffer(
                &page.buffer,
                (self.offset * size_of::<Instance>()) as u64,
                bytemuck::cast_slice(&self.staging),
            );
            context.render_pass.set_pipeline(&self.pipeline);
            context.render_pass.set_bind_group(0, &page.bind_group, &[]);
            context.render_pass.draw(
                0..6,
                self.offset as u32..(self.offset + commands.len()) as u32,
            );
            self.offset += commands.len();
        }
    }
}
