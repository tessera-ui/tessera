use std::any::Any;

use crate::{Px, PxPosition, renderer::DrawCommand};

#[allow(unused_variables)]
pub trait DrawablePipeline<T: DrawCommand> {
    fn begin_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
    }

    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
    }

    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &T,
        size: [Px; 2],
        start_pos: PxPosition,
    );
}

pub trait ErasedDrawablePipeline {
    fn begin_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    );

    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    );

    fn draw_erased(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &dyn DrawCommand,
        size: [Px; 2],
        start_pos: PxPosition,
    ) -> bool;
}

struct DrawablePipelineImpl<T: DrawCommand, P: DrawablePipeline<T>> {
    pipeline: P,
    _marker: std::marker::PhantomData<T>,
}

impl<T: DrawCommand + 'static, P: DrawablePipeline<T> + 'static> ErasedDrawablePipeline
    for DrawablePipelineImpl<T, P>
{
    fn begin_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.pipeline
            .begin_pass(gpu, gpu_queue, config, render_pass);
    }

    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.pipeline.end_pass(gpu, gpu_queue, config, render_pass);
    }

    fn draw_erased(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &dyn DrawCommand,
        size: [Px; 2],
        start_pos: PxPosition,
    ) -> bool {
        if let Some(cmd) = (command as &dyn Any).downcast_ref::<T>() {
            self.pipeline
                .draw(gpu, gpu_queue, config, render_pass, cmd, size, start_pos);
            true
        } else {
            false
        }
    }
}

pub struct PipelineRegistry {
    pub(crate) pipelines: Vec<Box<dyn ErasedDrawablePipeline>>,
}

impl PipelineRegistry {
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }

    pub fn register<T: DrawCommand + 'static, P: DrawablePipeline<T> + 'static>(
        &mut self,
        pipeline: P,
    ) {
        let erased = Box::new(DrawablePipelineImpl::<T, P> {
            pipeline,
            _marker: std::marker::PhantomData,
        });
        self.pipelines.push(erased);
    }

    pub(crate) fn begin_all_passes(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.begin_pass(gpu, gpu_queue, config, render_pass);
        }
    }

    pub(crate) fn end_all_passes(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.end_pass(gpu, gpu_queue, config, render_pass);
        }
    }

    pub(crate) fn dispatch(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        cmd: &dyn DrawCommand,
        size: [Px; 2],
        start_pos: PxPosition,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            if pipeline.draw_erased(gpu, gpu_queue, config, render_pass, cmd, size, start_pos) {
                return;
            }
        }

        panic!(
            "No pipeline found for command {:?}",
            std::any::type_name_of_val(cmd)
        );
    }
}
