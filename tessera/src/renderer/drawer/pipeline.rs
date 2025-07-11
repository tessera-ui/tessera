use crate::{PxPosition, px::PxSize, renderer::DrawCommand};

#[allow(unused_variables)]
pub trait DrawablePipeline<T: DrawCommand> {
    fn begin_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        compute_texture_view: &wgpu::TextureView,
    ) {
    }

    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        compute_texture_view: &wgpu::TextureView,
    ) {
    }

    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &T,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: Option<&wgpu::TextureView>,
        compute_texture_view: &wgpu::TextureView,
    );
}

pub trait ErasedDrawablePipeline {
    fn begin_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        compute_texture_view: &wgpu::TextureView,
    );

    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        compute_texture_view: &wgpu::TextureView,
    );

    fn draw_erased(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &dyn DrawCommand,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: Option<&wgpu::TextureView>,
        compute_texture_view: &wgpu::TextureView,
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
        compute_texture_view: &wgpu::TextureView,
    ) {
        self.pipeline
            .begin_pass(gpu, gpu_queue, config, render_pass, compute_texture_view);
    }

    fn end_pass(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        compute_texture_view: &wgpu::TextureView,
    ) {
        self.pipeline
            .end_pass(gpu, gpu_queue, config, render_pass, compute_texture_view);
    }

    fn draw_erased(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &dyn DrawCommand,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: Option<&wgpu::TextureView>,
        compute_texture_view: &wgpu::TextureView,
    ) -> bool {
        if let Some(cmd) = command.as_any().downcast_ref::<T>() {
            self.pipeline.draw(
                gpu,
                gpu_queue,
                config,
                render_pass,
                cmd,
                size,
                start_pos,
                scene_texture_view,
                compute_texture_view,
            );
            true
        } else {
            false
        }
    }
}

pub struct PipelineRegistry {
    pub(crate) pipelines: Vec<Box<dyn ErasedDrawablePipeline>>,
}

impl Default for PipelineRegistry {
    fn default() -> Self {
        Self::new()
    }
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
        compute_texture_view: &wgpu::TextureView,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.begin_pass(gpu, gpu_queue, config, render_pass, compute_texture_view);
        }
    }

    pub(crate) fn end_all_passes(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        compute_texture_view: &wgpu::TextureView,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.end_pass(gpu, gpu_queue, config, render_pass, compute_texture_view);
        }
    }

    pub(crate) fn dispatch(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        cmd: &dyn DrawCommand,
        size: PxSize,
        start_pos: PxPosition,
        scene_texture_view: Option<&wgpu::TextureView>,
        compute_texture_view: &wgpu::TextureView,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            if pipeline.draw_erased(
                gpu,
                gpu_queue,
                config,
                render_pass,
                cmd,
                size,
                start_pos,
                scene_texture_view,
                compute_texture_view,
            ) {
                return;
            }
        }

        panic!(
            "No pipeline found for command {:?}",
            std::any::type_name_of_val(cmd)
        );
    }
}
