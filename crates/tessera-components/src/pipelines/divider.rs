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

use super::instanced::{InstancePages, storage_quad_pipeline};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DividerCommand {
    pub color: Color,
    pub stroke: f32,
    pub vertical: bool,
}

impl DrawCommand for DividerCommand {
    fn supports_partial_redraw(&self) -> bool {
        true
    }

    fn draw_region(&self) -> DrawRegion {
        DrawRegion::PaddedLocal(PaddingRect::uniform(Px(1)))
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.color = self
            .color
            .with_alpha(self.color.a * opacity.clamp(0.0, 1.0));
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    bounds: [f32; 4],
    color: [f32; 4],
    screen: [f32; 4],
}

pub(crate) struct DividerPipeline {
    pipeline: wgpu::RenderPipeline,
    instances: InstancePages<Instance>,
}

impl DividerPipeline {
    pub(crate) fn new(
        device: &tessera_ui::GraphicsDevice,
        cache: Option<&wgpu::PipelineCache>,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let (layout, pipeline) = storage_quad_pipeline(
            device,
            include_wgsl!("divider/divider.wgsl"),
            config.format,
            sample_count,
            cache,
            "Divider Pipeline",
        );
        Self {
            pipeline,
            instances: InstancePages::new(
                device,
                layout,
                "divider_instances",
                "Divider Instance Page",
            ),
        }
    }
}

impl DrawablePipeline<DividerCommand> for DividerPipeline {
    fn begin_frame(&mut self, _: &FrameContext<'_>) {
        self.instances.begin_frame();
    }

    fn draw(&mut self, context: &mut DrawContext<DividerCommand>) {
        for commands in context.commands.chunks(self.instances.max_instances) {
            let (bind_group, instances) = self.instances.write(
                context.graphics,
                commands.iter().map(|(command, geometry)| {
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
                }),
            );
            context.render_pass.set_pipeline(&self.pipeline);
            context.render_pass.set_bind_group(0, bind_group, &[]);
            context.render_pass.draw(0..6, instances);
        }
    }
}
