use std::any::Any;

/// A command representing a request for a GPU-based computation.
///
/// This trait uses the `Any` trait to allow for dynamic typing, enabling different
/// command structs to be passed through the compute system. The blanket implementation
/// ensures that any struct that is `'static + Send + Sync` can be used as a command.
///
/// # Important
/// While any `'static + Send + Sync` type can be a `ComputeCommand`, to be practically
/// useful in a [`ComputablePipeline`], the specific command struct **must** also implement
/// `Hash`, `PartialEq`, and `Eq`. These are required by `ComputablePipeline`'s internal
/// caching mechanism to uniquely identify computation requests and store their results.
///
/// [`ComputablePipeline`]: super::pipeline::ComputablePipeline
pub trait ComputeCommand: Any + Send + Sync {
    /// Provides the command as a `&dyn Any`, allowing for downcasting.
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static + Send + Sync> ComputeCommand for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
