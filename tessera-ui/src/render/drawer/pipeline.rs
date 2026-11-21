//! Custom drawing pipelines.
//!
//! ## Usage
//!
//! Implement [`DrawablePipeline`] for a command and register it with [`PipelineRegistry`].

use std::{any::TypeId, collections::HashMap};

use smallvec::SmallVec;

use crate::{
    px::{PxRect, PxSize},
    render::DrawCommand,
    render_graph::DrawGeometry,
};

/// Resources available throughout one frame.
pub struct FrameContext<'a> {
    /// Renderer-owned resource creation and uploads.
    pub graphics: &'a crate::render::GraphicsDevice,
    /// The WGPU device.
    pub device: &'a wgpu::Device,
    /// The WGPU queue.
    pub queue: &'a wgpu::Queue,
    /// The current surface configuration.
    pub config: &'a wgpu::SurfaceConfiguration,
}

/// A batch of drawing commands within an active render pass.
pub struct DrawContext<'a, 'b, 'c, T> {
    /// Renderer-owned resource creation and uploads.
    pub graphics: &'a crate::render::GraphicsDevice,
    /// The WGPU device.
    pub device: &'a wgpu::Device,
    /// The WGPU queue.
    pub queue: &'a wgpu::Queue,
    /// The current surface configuration.
    pub config: &'a wgpu::SurfaceConfiguration,
    /// Target texture size for the current pass.
    pub target_size: PxSize,
    /// The active render pass encoder.
    pub render_pass: &'a mut wgpu::RenderPass<'b>,
    /// The draw commands to be processed.
    pub commands: &'c [(&'c T, DrawGeometry)],
    /// Scene pixels sampled by commands that declare a sample region.
    pub scene_texture_view: &'a wgpu::TextureView,
    /// Window rectangle of the valid pixels in `scene_texture_view`, whose
    /// first texel is at the rectangle's origin. The texture can be larger.
    pub scene_rect: PxRect,
    /// An optional clipping rectangle for the draw call.
    pub clip_rect: Option<PxRect>,
}

/// Resources for preparing one draw before its render pass begins.
///
/// Encode dependent GPU work through `encoder`; it runs after the declared
/// scene snapshot is copied and before any draws in the pass. Preparation is
/// not clipped; the subsequent draw still owns clipping and opacity.
pub struct DrawPreparationContext<'a, T> {
    /// Renderer-owned uploads and pooled resources.
    pub graphics: &'a crate::render::GraphicsDevice,
    /// The WGPU device.
    pub device: &'a wgpu::Device,
    /// The WGPU queue.
    pub queue: &'a wgpu::Queue,
    /// The current surface configuration.
    pub config: &'a wgpu::SurfaceConfiguration,
    /// Target texture size for the upcoming pass.
    pub target_size: PxSize,
    /// Encoder for work consumed by the upcoming draw.
    pub encoder: &'a mut wgpu::CommandEncoder,
    /// The draw being prepared.
    pub command: &'a T,
    /// Local dimensions and their window mapping.
    pub geometry: DrawGeometry,
    /// Scene pixels, valid only for commands declaring a sample region.
    pub scene_texture_view: &'a wgpu::TextureView,
    /// Window rectangle of valid pixels at the scene texture's origin.
    pub scene_rect: PxRect,
}

pub(crate) struct ErasedDrawPreparationContext<'a> {
    pub graphics: &'a crate::render::GraphicsDevice,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub target_size: PxSize,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub scene_texture_view: &'a wgpu::TextureView,
    pub scene_rect: PxRect,
}

/// Type-erased context used for dispatching draw pipelines.
pub(crate) struct ErasedDrawContext<'a, 'b> {
    /// Renderer-owned resource creation and uploads.
    pub graphics: &'a crate::render::GraphicsDevice,
    /// WGPU device used for pipeline resource access.
    pub device: &'a wgpu::Device,
    /// WGPU queue used for submissions.
    pub queue: &'a wgpu::Queue,
    /// Current surface configuration for the render target.
    pub config: &'a wgpu::SurfaceConfiguration,
    /// Target texture size for the current pass.
    pub target_size: PxSize,
    /// Active render pass that receives draw calls.
    pub render_pass: &'a mut wgpu::RenderPass<'b>,
    /// Scene pixels sampled by commands that declare a sample region.
    pub scene_texture_view: &'a wgpu::TextureView,
    /// Window rectangle of the valid pixels in `scene_texture_view`, whose
    /// first texel is at the rectangle's origin. The texture can be larger.
    pub scene_rect: PxRect,
    /// Optional clipping rectangle applied to the submission.
    pub clip_rect: Option<PxRect>,
}

/// Draws commands of type `T`.
///
/// Each frame calls `begin_frame`, then `prepare` before each pass's draws and
/// `draw` for each batch; `end_frame` runs after all passes and before submission.
#[allow(unused_variables)]
pub trait DrawablePipeline<T: DrawCommand> {
    /// Prepares resources once per frame, before render passes.
    fn begin_frame(&mut self, context: &FrameContext<'_>) {}

    /// Encodes intermediate GPU work for one command before its render pass.
    /// Keep pooled resources alive until all draws consuming them are recorded.
    fn prepare(&mut self, context: &mut DrawPreparationContext<'_, T>) {}

    /// Renders the command batch. Honor `clip_rect` when setting scissor bounds.
    /// The scene texture is available for commands that declare background sampling.
    fn draw(&mut self, context: &mut DrawContext<'_, '_, '_, T>);

    /// Finalizes the frame after all passes and before GPU submission.
    fn end_frame(&mut self, context: &FrameContext<'_>) {}
}

pub(crate) trait ErasedDrawablePipeline {
    /// Called at the beginning of a frame to prepare pipeline resources.
    fn begin_frame(&mut self, context: &FrameContext<'_>);
    /// Called at the end of a frame for cleanup or readback.
    fn end_frame(&mut self, context: &FrameContext<'_>);
    fn prepare_erased(
        &mut self,
        context: ErasedDrawPreparationContext<'_>,
        command: &dyn DrawCommand,
        geometry: DrawGeometry,
    );
    /// Draws a batch of commands with type-erased dispatch.
    fn draw_erased(
        &mut self,
        context: ErasedDrawContext<'_, '_>,
        commands: &[(&dyn DrawCommand, DrawGeometry)],
    ) -> bool;
}

struct DrawablePipelineImpl<T: DrawCommand, P: DrawablePipeline<T>> {
    pipeline: P,
    _marker: std::marker::PhantomData<T>,
}

impl<T: DrawCommand + 'static, P: DrawablePipeline<T> + 'static> ErasedDrawablePipeline
    for DrawablePipelineImpl<T, P>
{
    fn begin_frame(&mut self, context: &FrameContext<'_>) {
        self.pipeline.begin_frame(context);
    }

    fn end_frame(&mut self, context: &FrameContext<'_>) {
        self.pipeline.end_frame(context);
    }

    fn prepare_erased(
        &mut self,
        context: ErasedDrawPreparationContext<'_>,
        command: &dyn DrawCommand,
        geometry: DrawGeometry,
    ) {
        self.pipeline.prepare(&mut DrawPreparationContext {
            graphics: context.graphics,
            device: context.device,
            queue: context.queue,
            config: context.config,
            target_size: context.target_size,
            encoder: context.encoder,
            command: command
                .downcast_ref::<T>()
                .expect("registered draw command type"),
            geometry,
            scene_texture_view: context.scene_texture_view,
            scene_rect: context.scene_rect,
        });
    }

    fn draw_erased(
        &mut self,
        context: ErasedDrawContext<'_, '_>,
        commands: &[(&dyn DrawCommand, DrawGeometry)],
    ) -> bool {
        let timing = tracing::enabled!(tracing::Level::DEBUG).then(crate::time::Instant::now);
        if commands.is_empty() {
            return true;
        }

        let ErasedDrawContext {
            graphics,
            device,
            queue,
            config,
            target_size,
            render_pass,
            scene_texture_view,
            scene_rect,
            clip_rect,
        } = context;

        if commands[0].0.is::<T>() {
            let typed_commands: SmallVec<[(&T, DrawGeometry); 16]> = commands
                .iter()
                .map(|(cmd, geometry)| {
                    (
                        cmd.downcast_ref::<T>().expect(
                            "FATAL: A command in a batch has a different type than the first one.",
                        ),
                        *geometry,
                    )
                })
                .collect();

            self.pipeline.draw(&mut DrawContext {
                graphics,
                device,
                queue,
                config,
                target_size,
                render_pass,
                commands: &typed_commands,
                scene_texture_view,
                scene_rect,
                clip_rect,
            });
            if let Some(timing) = timing {
                tracing::debug!(
                    pipeline = std::any::type_name::<P>(),
                    phase = "draw",
                    commands = commands.len(),
                    elapsed_us = timing.elapsed().as_micros(),
                    frame_nanos = crate::runtime::frame_nanos_since_origin(),
                    "pipeline_cpu"
                );
            }
            true
        } else {
            false
        }
    }
}

/// Registered drawing pipelines, selected by command type.
#[derive(Default)]
pub struct PipelineRegistry {
    pub(crate) pipelines: HashMap<TypeId, Box<dyn ErasedDrawablePipeline>>,
}

impl PipelineRegistry {
    /// Creates an empty pipeline registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a pipeline for its command type, replacing any prior registration.
    pub fn register<T: DrawCommand + 'static, P: DrawablePipeline<T> + 'static>(
        &mut self,
        pipeline: P,
    ) {
        let erased = Box::new(DrawablePipelineImpl::<T, P> {
            pipeline,
            _marker: std::marker::PhantomData,
        });
        self.pipelines.insert(TypeId::of::<T>(), erased);
    }

    pub(crate) fn begin_all_frames(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        graphics: &crate::render::GraphicsDevice,
    ) {
        graphics.begin_frame();
        for pipeline in self.pipelines.values_mut() {
            pipeline.begin_frame(&FrameContext {
                graphics,
                device,
                queue,
                config,
            });
        }
    }

    pub(crate) fn end_all_frames(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        graphics: &crate::render::GraphicsDevice,
    ) {
        for pipeline in self.pipelines.values_mut() {
            pipeline.end_frame(&FrameContext {
                graphics,
                device,
                queue,
                config,
            });
        }
    }

    pub(crate) fn dispatch(
        &mut self,
        context: ErasedDrawContext<'_, '_>,
        commands: &[(&dyn DrawCommand, DrawGeometry)],
    ) {
        if commands.is_empty() {
            return;
        }

        let command_type_id = commands[0].0.as_any().type_id();
        if let Some(pipeline) = self.pipelines.get_mut(&command_type_id) {
            if !pipeline.draw_erased(context, commands) {
                panic!(
                    "FATAL: A command in a batch has a different type than the first one. This should not happen."
                )
            }
        } else {
            panic!(
                "No pipeline found for command {:?}",
                std::any::type_name_of_val(commands[0].0)
            );
        }
    }

    pub(crate) fn prepare(
        &mut self,
        context: ErasedDrawPreparationContext<'_>,
        command: &dyn DrawCommand,
        geometry: DrawGeometry,
    ) {
        self.pipelines
            .get_mut(&command.as_any().type_id())
            .expect("registered draw pipeline")
            .prepare_erased(context, command, geometry);
    }
}
