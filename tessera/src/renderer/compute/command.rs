use std::any::Any;

/// A command for an **asynchronous**, cacheable GPU computation.
///
/// This trait uses `Any` to allow for dynamic dispatch. Implementors must be `'static`
/// and are typically required to also implement `Hash + Eq` to be used as cache keys
/// in an `AsyncComputablePipeline`.
pub trait AsyncComputeCommand: Any + Send + Sync {
    /// Provides the command as a `&dyn Any`, allowing for downcasting.
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static + Send + Sync> AsyncComputeCommand for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// The SyncComputeCommand trait is no longer needed.
// The new SyncComputablePipeline design with an associated type handles both
// commands with and without lifetimes gracefully.
