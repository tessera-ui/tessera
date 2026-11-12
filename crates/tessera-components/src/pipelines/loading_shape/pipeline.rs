use tessera_ui::{
    render::{DrawContext, DrawablePipeline, FrameContext},
    wgpu,
};

use crate::pipelines::instanced::{DrawBuffers, quad_pipeline, storage_layout};

use super::{
    command::LoadingShapeCommand,
    geometry::{center, geometry},
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 4],
    color: [f32; 4],
    center_rotation: [f32; 4],
    screen_scale_progress: [f32; 4],
    geometry: [u32; 4],
}

pub(crate) struct LoadingShapePipeline {
    pipeline: wgpu::RenderPipeline,
    geometry_bind_group: wgpu::BindGroup,
    buffers: DrawBuffers<Instance>,
    analytic_aa: bool,
}

impl LoadingShapePipeline {
    pub(crate) fn new(
        device: &tessera_ui::render::GraphicsDevice,
        config: &wgpu::SurfaceConfiguration,
        cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let instance_layout = storage_layout(device, "Loading Shape Storage Layout");
        let buffer = device.create_buffer_init(
            "loading_shape_geometry",
            &wgpu::util::BufferInitDescriptor {
                label: Some("Loading Shape Geometry"),
                contents: bytemuck::cast_slice(&geometry().curves),
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let geometry_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Loading Shape Geometry"),
            layout: &instance_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("loading_shape.wgsl"));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Loading Shape Pipeline Layout"),
            bind_group_layouts: &[Some(&instance_layout), Some(&instance_layout)],
            immediate_size: 0,
        });
        let pipeline = quad_pipeline(
            device,
            &layout,
            &shader,
            config.format,
            sample_count,
            cache,
            "Loading Shape Pipeline",
            None,
            &[],
        );
        Self {
            pipeline,
            geometry_bind_group,
            buffers: DrawBuffers::new(
                instance_layout,
                "loading_shape_instances",
                "Loading Shape Instances",
            ),
            analytic_aa: sample_count == 1,
        }
    }
}

impl DrawablePipeline<LoadingShapeCommand> for LoadingShapePipeline {
    fn begin_frame(&mut self, _context: &FrameContext<'_>) {
        self.buffers.begin_frame();
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
            .map(|(command, draw_geometry)| {
                let position = draw_geometry.window_origin();
                let [width, height] = draw_geometry.window_size();

                let sequence = geometry().sequences[command.sequence];
                let side = width.min(height);
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
                        position.x + (width - side) * 0.5,
                        position.y + (height - side) * 0.5,
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
        let bind_group = self.buffers.write(context.graphics, &instances);
        context.render_pass.set_pipeline(&self.pipeline);
        context
            .render_pass
            .set_bind_group(0, &self.geometry_bind_group, &[]);
        context.render_pass.set_bind_group(1, bind_group, &[]);
        context
            .render_pass
            .draw(0..max_curves * subdivisions * 3, 0..instances.len() as u32);
    }
}
