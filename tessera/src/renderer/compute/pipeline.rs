use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use wgpu::{Device, Queue};

// --- Synchronous Compute System ---

/// A trait for a GPU compute pipeline that executes a command synchronously.
///
/// This system is designed for immediate, blocking execution of commands, often involving
/// borrowed data. It's suitable for one-off tasks like post-processing effects.
pub trait ComputablePipeline: Send + Sync + 'static {
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

/// A registry for all compute pipelines.
#[derive(Default)]
pub struct ComputePipelineRegistry {
    sync_pipelines: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ComputePipelineRegistry {
    pub fn new() -> Self {
        Self {
            sync_pipelines: HashMap::new(),
        }
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
