use bytemuck::{Pod, Zeroable};
use tessera_ui::{
    render::{DrawContext, DrawablePipeline, FrameContext},
    wgpu::{self, include_wgsl},
};

use crate::pipelines::instanced::{InstancePages, storage_quad_pipeline};

use super::command::CheckmarkCommand;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    position: [f32; 4],
    color: [f32; 4],
    parameters: [f32; 4],
    padding_screen: [f32; 4],
}

/// Pipeline for animated selection strokes.
pub struct CheckmarkPipeline {
    pipeline: wgpu::RenderPipeline,
    instances: InstancePages<Instance>,
}

impl CheckmarkPipeline {
    /// Creates the selection-stroke pipeline.
    pub fn new(
        device: &tessera_ui::render::GraphicsDevice,
        cache: Option<&wgpu::PipelineCache>,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let (layout, pipeline) = storage_quad_pipeline(
            device,
            include_wgsl!("checkmark.wgsl"),
            config.format,
            sample_count,
            cache,
            "Checkmark Pipeline",
        );
        Self {
            pipeline,
            instances: InstancePages::new(
                device,
                layout,
                "checkmark_instances",
                "Checkmark Instance Page",
            ),
        }
    }
}

impl DrawablePipeline<CheckmarkCommand> for CheckmarkPipeline {
    fn begin_frame(&mut self, _: &FrameContext<'_>) {
        self.instances.begin_frame();
    }

    fn draw(&mut self, context: &mut DrawContext<CheckmarkCommand>) {
        for commands in context.commands.chunks(self.instances.max_instances) {
            let (bind_group, instances) = self.instances.write(
                context.graphics,
                commands.iter().map(|(command, geometry)| {
                    let position = geometry.window_origin();
                    let [width, height] = geometry.window_size();
                    let scale = geometry.local_to_window.scale();
                    Instance {
                        position: [position.x, position.y, width, height],
                        color: command.color.to_array(),
                        parameters: [
                            command.stroke_width * scale,
                            command.progress,
                            command.indeterminate_fraction,
                            if command.checkbox_geometry { 1.0 } else { 0.0 },
                        ],
                        padding_screen: [
                            command.padding[0] * scale,
                            command.padding[1] * scale,
                            context.target_size.width.to_f32(),
                            context.target_size.height.to_f32(),
                        ],
                    }
                }),
            );
            context.render_pass.set_pipeline(&self.pipeline);
            context.render_pass.set_bind_group(0, bind_group, &[]);
            context.render_pass.draw(0..6, instances);
        }
    }
}
