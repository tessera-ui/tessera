use std::{any::Any, hash::Hash, sync::Arc};
use wgpu::{Device, Queue};

use super::command::ComputeCommand;

/// A trait for a GPU compute pipeline that calculates a result based on a command.
///
/// This trait defines the interface for a self-contained compute task. An implementation
/// is responsible for managing its own state, including any caching of results to avoid
/// re-dispatching identical computations. The system is designed for asynchronous, "fire-and-forget"
/// dispatch, with results being polled on subsequent frames.
///
/// # Type Parameters
///
/// * `T`: The specific type of [`ComputeCommand`] that this pipeline can process. The command
///   must implement `Hash` and `Eq` to be used as a key in caching mechanisms (e.g., a `HashMap`).
///
/// # Workflow
///
/// 1.  On each frame, a `DrawablePipeline` (or other system) creates a command struct `T`.
/// 2.  It calls [`ComputePipelineRegistry::dispatch_once`] with this command.
/// 3.  The registry passes the command to this pipeline's [`dispatch_once`] method. The pipeline
///     checks if the command has been seen before. If not, it runs a new compute shader job.
/// 4.  On the same or a subsequent frame, the system calls [`ComputePipelineRegistry::get_result`].
/// 5.  The registry calls this pipeline's [`get_result`] method. If the GPU work is complete,
///     the pipeline returns the result (`Some(Arc<...>)`). Otherwise, it returns `None`.
pub trait ComputablePipeline<T: ComputeCommand + Hash + Eq>: Send + Sync {
    /// Dispatches a compute job for the given command, if one is not already running or cached.
    ///
    /// This method should be idempotent. If called multiple times with the same command,
    /// it should only dispatch the GPU work once. Implementations are responsible for
    /// maintaining a cache (e.g., a `HashMap`) of commands to results or in-progress jobs.
    fn dispatch_once(&mut self, device: &Device, queue: &Queue, command: &T);

    /// Returns the result of a computation if it is ready.
    ///
    /// This method must be non-blocking. It should check the status of a computation and
    /// return the result if it's available. If the computation is still in progress or has
    /// not been dispatched, it should return `None`. The result is returned as a type-erased
    /// `Arc<dyn Any + Send + Sync>` for use in the registry.
    fn get_result(&self, command: &T) -> Option<Arc<dyn Any + Send + Sync>>;
}

/// (Internal) A type-erased version of `ComputablePipeline`.
///
/// This trait allows the `ComputePipelineRegistry` to store and interact with different
/// `ComputablePipeline` implementations in a heterogeneous collection, without needing
/// to know their specific `ComputeCommand` types.
pub(crate) trait ErasedComputablePipeline: Send + Sync {
    /// Type-erased version of [`ComputablePipeline::dispatch_once`].
    /// It attempts to downcast the `command` to the concrete type its pipeline understands.
    fn dispatch_once_erased(
        &mut self,
        device: &Device,
        queue: &Queue,
        command: &dyn ComputeCommand,
    );

    /// Type-erased version of [`ComputablePipeline::get_result`].
    /// It attempts to downcast the `command` before querying for a result.
    fn get_result_erased(&self, command: &dyn ComputeCommand)
    -> Option<Arc<dyn Any + Send + Sync>>;
}

/// (Internal) A wrapper that implements `ErasedComputablePipeline` for a concrete `ComputablePipeline`.
struct ComputablePipelineImpl<T: ComputeCommand + Hash + Eq, P: ComputablePipeline<T>> {
    pipeline: P,
    _marker: std::marker::PhantomData<T>,
}

impl<T: ComputeCommand + Hash + Eq + 'static, P: ComputablePipeline<T> + 'static>
    ErasedComputablePipeline for ComputablePipelineImpl<T, P>
{
    fn dispatch_once_erased(
        &mut self,
        device: &Device,
        queue: &Queue,
        command: &dyn ComputeCommand,
    ) {
        if let Some(c) = command.as_any().downcast_ref::<T>() {
            self.pipeline.dispatch_once(device, queue, c);
        }
    }

    fn get_result_erased(
        &self,
        command: &dyn ComputeCommand,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        if let Some(c) = command.as_any().downcast_ref::<T>() {
            self.pipeline.get_result(c)
        } else {
            None
        }
    }
}

/// A registry for all `ComputablePipeline`s in the application.
///
/// This struct acts as the central hub for the compute system. It holds all the different
/// compute pipelines (e.g., for rounded rectangles, text glyphs, etc.) and dispatches
/// commands to them.
///
/// This allows different parts of the application to request GPU-computed resources
/// without needing to know about the specific pipeline implementations.
#[derive(Default)]
pub struct ComputePipelineRegistry {
    pipelines: Vec<Box<dyn ErasedComputablePipeline>>,
}

impl ComputePipelineRegistry {
    /// Creates a new, empty registry.
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }

    /// Registers a new compute pipeline with the system.
    ///
    /// This is typically called once at application startup to add a specific compute
    /// capability. For example, a `ShapePipeline` might register a `G2RoundedRectPipeline`.
    ///
    /// # Example
    /// ```ignore
    /// let mut registry = ComputePipelineRegistry::new();
    /// registry.register(G2RoundedRectPipeline::new(gpu));
    /// ```
    pub fn register<T: ComputeCommand + Hash + Eq + 'static>(
        &mut self,
        pipeline: impl ComputablePipeline<T> + 'static,
    ) {
        let erased = Box::new(ComputablePipelineImpl::<T, _> {
            pipeline,
            _marker: std::marker::PhantomData,
        });
        self.pipelines.push(erased);
    }

    /// Dispatches a command to all registered pipelines.
    ///
    /// Each pipeline in the registry will receive the command. It is the responsibility
    /// of the pipeline's implementation to determine if it can handle the command's type
    /// via downcasting. If it can, it processes the command; otherwise, it ignores it.
    pub fn dispatch_once(&mut self, device: &Device, queue: &Queue, command: &dyn ComputeCommand) {
        for pipeline in self.pipelines.iter_mut() {
            pipeline.dispatch_once_erased(device, queue, command);
        }
    }

    /// Queries all registered pipelines for the result of a command.
    ///
    /// This method iterates through the pipelines and returns the first successful result it finds.
    /// This implies that for any given `ComputeCommand` type, only one `ComputablePipeline`
    /// should be registered that can handle it.
    pub fn get_result(&self, command: &dyn ComputeCommand) -> Option<Arc<dyn Any + Send + Sync>> {
        for pipeline in self.pipelines.iter() {
            if let Some(result) = pipeline.get_result_erased(command) {
                return Some(result);
            }
        }
        None
    }
}
