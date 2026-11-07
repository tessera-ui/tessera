//! Custom GPU compute pipelines.
//!
//! ## Usage
//!
//! Implement [`ComputablePipeline`] and register it with [`ComputePipelineRegistry`].

use std::{any::TypeId, collections::HashMap};

use crate::{PxRect, PxSize, SendSync, render_graph::DrawGeometry};

use super::command::ComputeCommand;

/// Type-erased metadata describing a compute command within a batch.
pub struct ErasedComputeBatchItem<'a> {
    /// The compute command to execute.
    pub command: &'a dyn ComputeCommand,
    /// Local dimensions and their mapping into the pass textures.
    pub geometry: DrawGeometry,
    /// The rectangle written, in pass texture coordinates.
    pub target_area: PxRect,
}

/// Strongly typed metadata describing a compute command within a batch.
pub struct ComputeBatchItem<'a, C: ComputeCommand> {
    /// The compute command to execute.
    pub command: &'a C,
    /// Local dimensions and their mapping into the pass textures.
    pub geometry: DrawGeometry,
    /// The rectangle written, in pass texture coordinates.
    pub target_area: PxRect,
}

/// A batch of compute commands within an active compute pass.
///
/// The pass textures cover the scene region the batch reads and writes, so
/// they can be larger than `target_size` and offset from the window.
pub struct ComputeContext<'a, 'b, 'c, C: ComputeCommand> {
    /// Renderer-owned resource creation and uploads.
    pub graphics: &'a crate::GraphicsDevice,
    /// WGPU device used to create and manage GPU resources.
    pub device: &'a wgpu::Device,
    /// Queue for submitting GPU workloads.
    pub queue: &'a wgpu::Queue,
    /// Surface configuration describing output formats and dimensions.
    pub config: &'a wgpu::SurfaceConfiguration,
    /// Size of the scene region covered by the pass textures.
    pub target_size: PxSize,
    /// Active compute pass encoder.
    pub compute_pass: &'a mut wgpu::ComputePass<'b>,
    /// Batch of typed compute items to process.
    pub items: &'c [ComputeBatchItem<'c, C>],
    /// Input texture view sampled by the compute pass.
    pub input_view: &'a wgpu::TextureView,
    /// Output texture view written by the compute pass.
    pub output_view: &'a wgpu::TextureView,
}

/// Type-erased context used when dispatching compute pipelines.
pub(crate) struct ErasedDispatchContext<'a, 'b> {
    pub graphics: &'a crate::GraphicsDevice,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub target_size: PxSize,
    pub compute_pass: &'a mut wgpu::ComputePass<'b>,
    pub input_view: &'a wgpu::TextureView,
    pub output_view: &'a wgpu::TextureView,
}

/// Processes compute commands of type `C`.
pub trait ComputablePipeline<C: ComputeCommand>: SendSync {
    /// Dispatches the command batch over each item's target area. The output
    /// starts as a copy of the input and uses a non-sRGB storage format; draws
    /// that sample the scene next read the result.
    fn dispatch(&mut self, context: &mut ComputeContext<C>);
}

pub(crate) trait ErasedComputablePipeline: SendSync {
    /// Dispatches a type-erased compute command.
    fn dispatch_erased(
        &mut self,
        context: ErasedDispatchContext<'_, '_>,
        items: &[ErasedComputeBatchItem<'_>],
    );
}

/// A wrapper to implement `ErasedComputablePipeline` for any
/// `ComputablePipeline`.
struct ComputablePipelineImpl<C: ComputeCommand, P: ComputablePipeline<C>> {
    pipeline: P,
    _command: std::marker::PhantomData<C>,
}

impl<C: ComputeCommand + 'static, P: ComputablePipeline<C>> ErasedComputablePipeline
    for ComputablePipelineImpl<C, P>
{
    fn dispatch_erased(
        &mut self,
        context: ErasedDispatchContext<'_, '_>,
        items: &[ErasedComputeBatchItem<'_>],
    ) {
        if items.is_empty() {
            return;
        }

        let mut typed_items: Vec<ComputeBatchItem<'_, C>> = Vec::with_capacity(items.len());
        for item in items {
            let command = item
                .command
                .downcast_ref::<C>()
                .expect("Compute batch contained command of unexpected type");
            typed_items.push(ComputeBatchItem {
                command,
                geometry: item.geometry,
                target_area: item.target_area,
            });
        }

        self.pipeline.dispatch(&mut ComputeContext {
            graphics: context.graphics,
            device: context.device,
            queue: context.queue,
            config: context.config,
            target_size: context.target_size,
            compute_pass: context.compute_pass,
            items: &typed_items,
            input_view: context.input_view,
            output_view: context.output_view,
        });
    }
}

/// Registered compute pipelines, selected by command type.
#[derive(Default)]
pub struct ComputePipelineRegistry {
    pipelines: HashMap<TypeId, Box<dyn ErasedComputablePipeline>>,
}

impl ComputePipelineRegistry {
    /// Creates an empty compute pipeline registry.
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn remove(&mut self, type_id: TypeId) {
        self.pipelines.remove(&type_id);
    }

    /// Registers a pipeline for its command type, replacing any prior registration.
    pub fn register<C: ComputeCommand + 'static>(
        &mut self,
        pipeline: impl ComputablePipeline<C> + 'static,
    ) {
        let erased_pipeline = Box::new(ComputablePipelineImpl {
            pipeline,
            _command: std::marker::PhantomData,
        });
        self.pipelines.insert(TypeId::of::<C>(), erased_pipeline);
    }

    /// Dispatches one or more commands to their corresponding registered
    /// pipeline.
    pub(crate) fn dispatch_erased(
        &mut self,
        context: ErasedDispatchContext<'_, '_>,
        items: &[ErasedComputeBatchItem<'_>],
    ) {
        if items.is_empty() {
            return;
        }

        let command_type_id = items[0].command.as_any().type_id();
        if let Some(pipeline) = self.pipelines.get_mut(&command_type_id) {
            pipeline.dispatch_erased(context, items);
        } else {
            panic!(
                "No pipeline found for command {:?}",
                std::any::type_name_of_val(items[0].command)
            );
        }
    }
}
