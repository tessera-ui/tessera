use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::Hash,
    sync::Arc,
};
use wgpu::{Device, Queue};

use super::command::AsyncComputeCommand;

// --- Asynchronous Compute System ---

/// A trait for a GPU compute pipeline that calculates a result asynchronously.
///
/// This system is designed for "fire-and-forget" dispatch of commands that can be
/// cached and polled on subsequent frames.
pub trait AsyncComputablePipeline<T: AsyncComputeCommand + Hash + Eq>: Send + Sync {
    /// Dispatches a compute job for the given command, if one is not already running or cached.
    fn dispatch_once(&mut self, device: &Device, queue: &Queue, command: &T);

    /// Returns the result of a computation if it is ready. This must be non-blocking.
    fn get_result(&self, command: &T) -> Option<Arc<dyn Any + Send + Sync>>;
}

/// (Internal) A type-erased version of `AsyncComputablePipeline` for dynamic dispatch.
trait ErasedAsyncComputablePipeline: Send + Sync {
    fn dispatch_once_erased(
        &mut self,
        device: &Device,
        queue: &Queue,
        command: &dyn AsyncComputeCommand,
    );
    fn get_result_erased(
        &self,
        command: &dyn AsyncComputeCommand,
    ) -> Option<Arc<dyn Any + Send + Sync>>;
}

/// (Internal) Wrapper to implement `ErasedAsyncComputablePipeline` for a concrete pipeline.
struct AsyncComputablePipelineImpl<
    T: AsyncComputeCommand + Hash + Eq,
    P: AsyncComputablePipeline<T>,
> {
    pipeline: P,
    _marker: std::marker::PhantomData<T>,
}

impl<T: AsyncComputeCommand + Hash + Eq + 'static, P: AsyncComputablePipeline<T> + 'static>
    ErasedAsyncComputablePipeline for AsyncComputablePipelineImpl<T, P>
{
    fn dispatch_once_erased(
        &mut self,
        device: &Device,
        queue: &Queue,
        command: &dyn AsyncComputeCommand,
    ) {
        if let Some(c) = command.as_any().downcast_ref::<T>() {
            self.pipeline.dispatch_once(device, queue, c);
        }
    }

    fn get_result_erased(
        &self,
        command: &dyn AsyncComputeCommand,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        if let Some(c) = command.as_any().downcast_ref::<T>() {
            self.pipeline.get_result(c)
        } else {
            None
        }
    }
}

// --- Synchronous Compute System ---

/// A trait for a GPU compute pipeline that executes a command synchronously.
///
/// This system is designed for immediate, blocking execution of commands, often involving
/// borrowed data. It's suitable for one-off tasks like post-processing effects.
pub trait SyncComputablePipeline: Send + Sync + 'static {
    /// The command type associated with this pipeline.
    /// It can have a lifetime `'a`, which will be tied to the `dispatch_sync` call.
    type Command<'a>: Send + Sync;

    /// Dispatches a command and blocks until the computation is complete, returning the result.
    fn dispatch_sync<'a>(
        &mut self,
        device: &Device,
        queue: &Queue,
        command: &Self::Command<'a>,
    ) -> Option<Arc<dyn Any + Send + Sync>>;
}

// --- Registry ---

/// A registry for all compute pipelines, both synchronous and asynchronous.
#[derive(Default)]
pub struct ComputePipelineRegistry {
    async_pipelines: Vec<Box<dyn ErasedAsyncComputablePipeline>>,
    sync_pipelines: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ComputePipelineRegistry {
    pub fn new() -> Self {
        Self {
            async_pipelines: Vec::new(),
            sync_pipelines: HashMap::new(),
        }
    }

    // --- Async Methods ---

    /// Registers a new asynchronous compute pipeline.
    pub fn register_async<T: AsyncComputeCommand + Hash + Eq + 'static>(
        &mut self,
        pipeline: impl AsyncComputablePipeline<T> + 'static,
    ) {
        let erased = Box::new(AsyncComputablePipelineImpl::<T, _> {
            pipeline,
            _marker: std::marker::PhantomData,
        });
        self.async_pipelines.push(erased);
    }

    /// Dispatches a command to all registered asynchronous pipelines.
    pub fn dispatch_async(
        &mut self,
        device: &Device,
        queue: &Queue,
        command: &dyn AsyncComputeCommand,
    ) {
        for pipeline in self.async_pipelines.iter_mut() {
            pipeline.dispatch_once_erased(device, queue, command);
        }
    }

    /// Queries all registered asynchronous pipelines for a result.
    pub fn get_async_result<R: 'static + Send + Sync>(
        &self,
        command: &dyn AsyncComputeCommand,
    ) -> Option<Arc<R>> {
        self.async_pipelines
            .iter()
            .find_map(|p| p.get_result_erased(command))
            .and_then(|any_result| {
                any_result
                    .downcast::<Arc<R>>()
                    .ok()
                    .map(|arc_arc_r| (*arc_arc_r).clone())
            })
    }

    // --- Sync Methods ---

    /// Registers a new synchronous compute pipeline.
    pub fn register_sync<P>(&mut self, pipeline: P)
    where
        P: Any + Send + Sync,
    {
        self.sync_pipelines
            .insert(TypeId::of::<P>(), Box::new(pipeline));
    }

    /// Retrieves a mutable reference to a registered synchronous pipeline by its type.
    ///
    /// This enables static dispatch for synchronous operations.
    ///
    /// # Panics
    /// Panics if the requested pipeline type is not registered.
    pub fn get_sync<P: Any + Send + Sync>(&mut self) -> &mut P {
        self.sync_pipelines
            .get_mut(&TypeId::of::<P>())
            .and_then(|b| b.downcast_mut::<P>())
            .unwrap_or_else(|| {
                panic!(
                    "Requested synchronous pipeline {} not registered",
                    std::any::type_name::<P>()
                )
            })
    }
}
