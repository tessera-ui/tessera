use std::any::Any;

/// A navigation destination produced by the `#[shard]` macro.
pub trait RouterDestination: Any + Send + Sync {
    /// Execute the component associated with this destination.
    fn exec_component(&self);

    /// Stable destination identifier used for route identity checks.
    fn destination_id() -> &'static str
    where
        Self: Sized;
}
