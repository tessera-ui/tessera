use std::{any::Any, sync::Arc};

/// A navigation destination produced by the `#[shard]` macro.
pub trait RouterDestination: Any + Send + Sync {
    /// Execute the component associated with this destination.
    fn exec_component(&self);

    /// Copies the component entry point so it can run without borrowing `self`.
    fn exec_component_handle(&self) -> Arc<dyn Fn() + Send + Sync>;

    /// Stable destination identifier used for route identity checks.
    fn destination_id() -> &'static str
    where
        Self: Sized;
}
