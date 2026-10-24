use tessera_ui::{
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline, FrameContext},
    wgpu,
};

use super::command::LinearWaveCommand;

#[cfg(test)]
pub(crate) static ALLOCATIONS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 4],
    color: [f32; 4],
    curve: [f32; 4],
    wave: [f32; 4],
    screen: [f32; 4],
}

struct Buffer {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: usize,
}

pub(crate) struct LinearWavePipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    buffers: Vec<Buffer>,
    cursor: usize,
}

impl LinearWavePipeline {
    pub(crate) fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Linear Wave Instances"),
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
        let shader = device.create_shader_module(wgpu::include_wgsl!("linear_wave.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Linear Wave Pipeline Layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Linear Wave Pipeline"),
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

    fn ensure_buffer(&mut self, device: &wgpu::Device, count: usize) {
        if self
            .buffers
            .get(self.cursor)
            .is_some_and(|buffer| buffer.capacity >= count)
        {
            return;
        }
        let capacity = count.max(1).next_power_of_two();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Linear Wave Instances"),
            size: (capacity * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Linear Wave Instances"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        #[cfg(test)]
        ALLOCATIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let buffer = Buffer {
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

impl DrawablePipeline<LinearWaveCommand> for LinearWavePipeline {
    fn begin_frame(&mut self, _context: &FrameContext<'_>) {
        self.cursor = 0;
    }

    fn draw(&mut self, context: &mut DrawContext<'_, '_, '_, LinearWaveCommand>) {
        if context.commands.is_empty() {
            return;
        }
        let instances: Vec<_> = context
            .commands
            .iter()
            .map(|(command, geometry)| {
                let position = geometry.window_origin();
                let [width, height] = geometry.window_size();
                let scale = geometry.local_to_window.scale();
                Instance {
                    position: [position.x, position.y, width, height],
                    color: command.color.to_array(),
                    curve: [
                        command.start * scale,
                        command.end * scale,
                        command.stroke * scale,
                        command.wavelength * scale,
                    ],
                    wave: [
                        command.control_height * scale,
                        command.shift * scale,
                        f32::from(u8::from(command.round)),
                        f32::from(u8::from(command.reverse)),
                    ],
                    screen: [
                        context.target_size.width.to_f32(),
                        context.target_size.height.to_f32(),
                        0.0,
                        0.0,
                    ],
                }
            })
            .collect();
        self.ensure_buffer(context.device, instances.len());
        let buffer = &self.buffers[self.cursor];
        context
            .queue
            .write_buffer(&buffer.buffer, 0, bytemuck::cast_slice(&instances));
        context.render_pass.set_pipeline(&self.pipeline);
        context
            .render_pass
            .set_bind_group(0, &buffer.bind_group, &[]);
        context.render_pass.draw(0..6, 0..instances.len() as u32);
        self.cursor += 1;
    }
}
