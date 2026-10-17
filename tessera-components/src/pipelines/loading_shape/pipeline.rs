use tessera_ui::{
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline, FrameContext},
    wgpu::{self, util::DeviceExt as _},
};

use super::{
    command::LoadingShapeCommand,
    geometry::{center, geometry},
};

#[cfg(test)]
pub(crate) static INSTANCE_ALLOCATIONS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 4],
    color: [f32; 4],
    center_rotation: [f32; 4],
    screen_scale_progress: [f32; 4],
    geometry: [u32; 4],
}

struct InstanceBuffer {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: usize,
}

pub(crate) struct LoadingShapePipeline {
    pipeline: wgpu::RenderPipeline,
    geometry_bind_group: wgpu::BindGroup,
    instance_layout: wgpu::BindGroupLayout,
    buffers: Vec<InstanceBuffer>,
    cursor: usize,
    analytic_aa: bool,
}

impl LoadingShapePipeline {
    pub(crate) fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let storage_layout = |label| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
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
            })
        };
        let geometry_layout = storage_layout("Loading Shape Geometry Layout");
        let instance_layout = storage_layout("Loading Shape Instance Layout");
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Loading Shape Geometry"),
            contents: bytemuck::cast_slice(&geometry().curves),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let geometry_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Loading Shape Geometry"),
            layout: &geometry_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("loading_shape.wgsl"));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Loading Shape Pipeline Layout"),
            bind_group_layouts: &[Some(&geometry_layout), Some(&instance_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Loading Shape Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: tessera_ui::ui_stencil_state(),
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
            cache,
        });
        Self {
            pipeline,
            geometry_bind_group,
            instance_layout,
            buffers: Vec::new(),
            cursor: 0,
            analytic_aa: sample_count == 1,
        }
    }

    fn ensure_buffer(&mut self, device: &wgpu::Device, count: usize) {
        if self
            .buffers
            .get(self.cursor)
            .is_some_and(|buffer| buffer.capacity >= count)
        {
            return;
        }
        let capacity = count.max(1).next_power_of_two();
        #[cfg(test)]
        INSTANCE_ALLOCATIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Loading Shape Instances"),
            size: (capacity * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Loading Shape Instances"),
            layout: &self.instance_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        let buffer = InstanceBuffer {
            buffer,
            bind_group,
            capacity,
        };
        if self.cursor < self.buffers.len() {
            self.buffers[self.cursor] = buffer;
        } else {
            self.buffers.push(buffer);
        }
    }
}

impl DrawablePipeline<LoadingShapeCommand> for LoadingShapePipeline {
    fn begin_frame(&mut self, _context: &FrameContext<'_>) {
        self.cursor = 0;
    }

    fn draw(&mut self, context: &mut DrawContext<'_, '_, '_, LoadingShapeCommand>) {
        if context.commands.is_empty() {
            return;
        }
        let mut max_curves = 0;
        let mut subdivisions = 1;
        let mut instances: Vec<_> = context
            .commands
            .iter()
            .map(|(command, size, position)| {
                let sequence = geometry().sequences[command.sequence];
                let side = size.width.to_f32().min(size.height.to_f32());
                // The cubic second-derivative bound limits chord error to
                // 0.05px.
                let segments = (0.75
                    * sequence.curvature
                    * side
                    * sequence.scale
                    * (command.progress.abs() + (1.0 - command.progress).abs())
                    / 0.05)
                    .sqrt()
                    .ceil()
                    .clamp(1.0, 256.0) as u32;
                subdivisions = subdivisions.max(segments);
                max_curves = max_curves.max(sequence.count);
                let center = center(sequence, command.progress);
                let (sin, cos) = command.rotation.to_radians().sin_cos();
                Instance {
                    position: [
                        position.x + (size.width.to_f32() - side) * 0.5,
                        position.y + (size.height.to_f32() - side) * 0.5,
                        side,
                        side,
                    ],
                    color: [
                        command.color.r,
                        command.color.g,
                        command.color.b,
                        command.color.a,
                    ],
                    center_rotation: [center[0], center[1], cos, sin],
                    screen_scale_progress: [
                        context.target_size.width.to_f32(),
                        context.target_size.height.to_f32(),
                        sequence.scale,
                        command.progress,
                    ],
                    geometry: [
                        sequence.offset,
                        sequence.count,
                        0,
                        u32::from(self.analytic_aa),
                    ],
                }
            })
            .collect();
        for instance in &mut instances {
            instance.geometry[2] = subdivisions;
        }
        // A separate slot per draw keeps later writes from changing earlier
        // batches.
        self.ensure_buffer(context.device, instances.len());
        let buffer = &self.buffers[self.cursor];
        context
            .queue
            .write_buffer(&buffer.buffer, 0, bytemuck::cast_slice(&instances));
        context.render_pass.set_pipeline(&self.pipeline);
        context
            .render_pass
            .set_bind_group(0, &self.geometry_bind_group, &[]);
        context
            .render_pass
            .set_bind_group(1, &buffer.bind_group, &[]);
        context
            .render_pass
            .draw(0..max_curves * subdivisions * 3, 0..instances.len() as u32);
        self.cursor += 1;
    }
}
