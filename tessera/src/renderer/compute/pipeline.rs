use crate::compute::resource::ComputeResourceManager;

use super::command::ComputeCommand;

/// A unified trait for a GPU compute pipeline.
///
/// This pipeline operates within a given `wgpu::ComputePass` and dispatches a specific
/// `ComputeCommand`. It's designed to be part of a larger, strictly sequenced series of
/// rendering and compute passes managed by the renderer.
pub trait ComputablePipeline<C: ComputeCommand>: Send + Sync + 'static {
    /// Dispatches the compute command within an active `ComputePass`.
    ///
    /// # Arguments
    ///
    /// * `input_view`: A view of the texture providing input data for the compute shader.
    ///   This is typically the result of the previous rendering or compute pass.
    /// * `output_view`: A view of the texture where the compute shader will write its output.
    ///
    /// # Texture Format Convention
    ///
    /// Due to `wgpu` limitations, textures with an sRGB format cannot be used as storage
    /// textures (`STORAGE_BINDING`), which is a requirement for compute shader outputs.
    /// Therefore, the `output_view` provided to this method is guaranteed to have a
    /// non-sRGB format that supports storage writes.
    ///
    /// **By convention, all compute pipelines in this framework must be designed to write to
    /// a texture with the `wgpu::TextureFormat::Rgba8Unorm` format.**
    fn dispatch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &C,
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    );
}

/// An internal, type-erased version of `ComputablePipeline` for dynamic dispatch.
pub(crate) trait ErasedComputablePipeline: Send + Sync {
    /// Dispatches a type-erased compute command.
    fn dispatch_erased(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &dyn ComputeCommand,
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    );
}

/// A wrapper to implement `ErasedComputablePipeline` for any `ComputablePipeline`.
struct ComputablePipelineImpl<C: ComputeCommand, P: ComputablePipeline<C>> {
    pipeline: P,
    _command: std::marker::PhantomData<C>,
}

impl<C: ComputeCommand + 'static, P: ComputablePipeline<C>> ErasedComputablePipeline
    for ComputablePipelineImpl<C, P>
{
    fn dispatch_erased(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &dyn ComputeCommand,
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    ) {
        if let Some(command) = command.as_any().downcast_ref::<C>() {
            self.pipeline.dispatch(
                device,
                queue,
                config,
                compute_pass,
                command,
                resource_manager,
                input_view,
                output_view,
            );
        }
    }
}

/// A registry for all compute pipelines.
#[derive(Default)]
pub struct ComputePipelineRegistry {
    pipelines: Vec<Box<dyn ErasedComputablePipeline>>,
}

impl ComputePipelineRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new compute pipeline.
    pub fn register<C: ComputeCommand + 'static>(
        &mut self,
        pipeline: impl ComputablePipeline<C> + 'static,
    ) {
        let erased_pipeline = Box::new(ComputablePipelineImpl {
            pipeline,
            _command: std::marker::PhantomData,
        });
        self.pipelines.push(erased_pipeline);
    }

    /// Dispatches a command to its corresponding registered pipeline.
    pub(crate) fn dispatch_erased(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        compute_pass: &mut wgpu::ComputePass<'_>,
        command: &dyn ComputeCommand,
        resource_manager: &mut ComputeResourceManager,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
    ) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.dispatch_erased(
                device,
                queue,
                config,
                compute_pass,
                command,
                resource_manager,
                input_view,
                output_view,
            );
        }
    }
}
