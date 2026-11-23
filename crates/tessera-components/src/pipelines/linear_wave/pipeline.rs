use tessera_ui::{
    render::{DrawContext, DrawablePipeline, FrameContext},
    wgpu,
};

use crate::pipelines::instanced::{DrawBuffers, QuadPipeline, storage_layout};

use super::command::LinearWaveCommand;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 4],
    color: [f32; 4],
    curve: [f32; 4],
    wave: [f32; 4],
    screen: [f32; 4],
}

pub(crate) struct LinearWavePipeline {
    pipeline: wgpu::RenderPipeline,
    buffers: DrawBuffers<Instance>,
}

impl LinearWavePipeline {
    pub(crate) fn new(
        device: &tessera_ui::render::GraphicsDevice,
        config: &wgpu::SurfaceConfiguration,
        cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let layout = storage_layout(device, "Linear Wave Instances");
        let pipeline = QuadPipeline {
            label: "Linear Wave Pipeline",
            layout_label: "Linear Wave Pipeline Layout",
            format: config.format,
            sample_count,
            cache,
        }
        .create(
            device,
            &[Some(&layout)],
            wgpu::include_wgsl!("linear_wave.wgsl"),
            None,
            &[],
        );
        Self {
            pipeline,
            buffers: DrawBuffers::new(layout, "linear_wave_instances", "Linear Wave Instances"),
        }
    }
}

impl DrawablePipeline<LinearWaveCommand> for LinearWavePipeline {
    fn begin_frame(&mut self, _context: &FrameContext<'_>) {
        self.buffers.begin_frame();
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
        let bind_group = self.buffers.write(context.graphics, &instances);
        context.render_pass.set_pipeline(&self.pipeline);
        context.render_pass.set_bind_group(0, bind_group, &[]);
        context.render_pass.draw(0..6, 0..instances.len() as u32);
    }
}
