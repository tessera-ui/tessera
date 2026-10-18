use tessera_ui::{
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline, FrameContext},
    wgpu,
};

use super::command::PathStrokeCommand;

#[cfg(test)]
pub(crate) static ALLOCATIONS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 4],
    bounds: [f32; 4],
    color: [f32; 4],
    screen: [f32; 4],
    geometry: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Segment {
    points: [f32; 4],
    caps: [f32; 4],
}

struct Buffer {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: usize,
    segments: wgpu::Buffer,
    segment_capacity: usize,
}

pub(crate) struct PathStrokePipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    buffers: Vec<Buffer>,
    cursor: usize,
}

impl PathStrokePipeline {
    pub(crate) fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Path Stroke Instances"),
            entries: &(0..2)
                .map(|binding| wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                })
                .collect::<Vec<_>>(),
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("path_stroke.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Path Stroke Pipeline Layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Path Stroke Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: tessera_ui::ui_stencil_state(),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                ..Default::default()
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
            cache,
        });
        Self {
            pipeline,
            layout,
            buffers: Vec::new(),
            cursor: 0,
        }
    }

    fn ensure_buffer(&mut self, device: &wgpu::Device, count: usize, segment_count: usize) {
        if self.buffers.get(self.cursor).is_some_and(|buffer| {
            buffer.capacity >= count && buffer.segment_capacity >= segment_count
        }) {
            return;
        }
        let capacity = count
            .max(64)
            .max(
                self.buffers
                    .get(self.cursor)
                    .map_or(0, |buffer| buffer.capacity),
            )
            .next_power_of_two();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Path Stroke Instances"),
            size: (capacity * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let segment_capacity = segment_count
            .max(1024)
            .max(
                self.buffers
                    .get(self.cursor)
                    .map_or(0, |buffer| buffer.segment_capacity),
            )
            .next_power_of_two();
        let segments = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Path Stroke Segments"),
            size: (segment_capacity * std::mem::size_of::<Segment>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Path Stroke Instances"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: segments.as_entire_binding(),
                },
            ],
        });
        #[cfg(test)]
        ALLOCATIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let buffer = Buffer {
            buffer,
            bind_group,
            capacity,
            segments,
            segment_capacity,
        };
        if self.cursor < self.buffers.len() {
            self.buffers[self.cursor] = buffer;
        } else {
            self.buffers.push(buffer);
        }
    }
}

impl DrawablePipeline<PathStrokeCommand> for PathStrokePipeline {
    fn begin_frame(&mut self, _context: &FrameContext<'_>) {
        self.cursor = 0;
    }

    fn draw(&mut self, context: &mut DrawContext<'_, '_, '_, PathStrokeCommand>) {
        if context.commands.is_empty() {
            return;
        }
        let mut instances = Vec::new();
        let mut segments = Vec::new();
        for (command, size, position) in context.commands {
            if command.points.len() < 2 || command.stroke <= 0.0 {
                continue;
            }
            let radius = command.stroke * 0.5;
            let padding = glam::Vec2::splat(radius + 1.0);
            let mut path: Vec<_> = command
                .points
                .windows(2)
                .filter_map(|points| {
                    let a = points[0];
                    let b = points[1];
                    (a.distance_squared(b) > 1e-15).then_some((
                        a,
                        b,
                        Segment {
                            points: [a.x, a.y, b.x, b.y],
                            caps: [0.0; 4],
                        },
                    ))
                })
                .collect();
            if path.is_empty() {
                continue;
            }
            if !command.round && !command.closed {
                path.first_mut().unwrap().2.caps[0] = 1.0;
                path.last_mut().unwrap().2.caps[1] = 1.0;
            }
            let low = path
                .iter()
                .fold(glam::Vec2::splat(f32::INFINITY), |value, (a, b, _)| {
                    value.min(*a).min(*b)
                })
                - padding;
            let high = path
                .iter()
                .fold(glam::Vec2::splat(f32::NEG_INFINITY), |value, (a, b, _)| {
                    value.max(*a).max(*b)
                })
                + padding;
            let low = low.max(glam::Vec2::ZERO);
            let high = high.min(glam::Vec2::new(size.width.to_f32(), size.height.to_f32()));
            if high.x <= low.x || high.y <= low.y {
                continue;
            }
            // Disjoint tiles blend each path once, including translucent joins.
            // Scatter segments to local bins to avoid scanning the path per
            // tile.
            const TILE: f32 = 8.0;
            let first = (low / TILE).floor().as_ivec2();
            let last = (high / TILE).ceil().as_ivec2();
            let columns = (last.x - first.x) as usize;
            let rows = (last.y - first.y) as usize;
            let mut bins: Vec<Vec<Segment>> = vec![Vec::new(); columns * rows];
            for (a, b, segment) in &path {
                let start = ((a.min(*b) - padding).max(low) / TILE)
                    .floor()
                    .as_ivec2()
                    .max(first);
                let end = ((a.max(*b) + padding).min(high) / TILE)
                    .ceil()
                    .as_ivec2()
                    .min(last);
                for y in start.y..end.y {
                    for x in start.x..end.x {
                        bins[(y - first.y) as usize * columns + (x - first.x) as usize]
                            .push(*segment);
                    }
                }
            }
            for (index, bin) in bins.into_iter().enumerate() {
                if bin.is_empty() {
                    continue;
                }
                let x = first.x as f32 + (index % columns) as f32;
                let y = first.y as f32 + (index / columns) as f32;
                let start = glam::Vec2::new(x * TILE, y * TILE).max(low);
                let end = glam::Vec2::new((x + 1.0) * TILE, (y + 1.0) * TILE).min(high);
                let offset = segments.len();
                segments.extend(bin);
                instances.push(Instance {
                    position: [position.x, position.y, radius, 0.0],
                    bounds: [start.x, start.y, end.x, end.y],
                    color: command.color.to_array(),
                    screen: [
                        context.target_size.width.to_f32(),
                        context.target_size.height.to_f32(),
                        0.0,
                        0.0,
                    ],
                    geometry: [offset as u32, (segments.len() - offset) as u32, 0, 0],
                });
            }
        }
        if instances.is_empty() {
            return;
        }
        self.ensure_buffer(context.device, instances.len(), segments.len());
        let buffer = &self.buffers[self.cursor];
        context
            .queue
            .write_buffer(&buffer.buffer, 0, bytemuck::cast_slice(&instances));
        context
            .queue
            .write_buffer(&buffer.segments, 0, bytemuck::cast_slice(&segments));
        context.render_pass.set_pipeline(&self.pipeline);
        context
            .render_pass
            .set_bind_group(0, &buffer.bind_group, &[]);
        context.render_pass.draw(0..6, 0..instances.len() as u32);
        self.cursor += 1;
    }
}
