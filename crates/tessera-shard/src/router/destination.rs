use std::any::Any;

/// A navigation destination produced by the `#[shard]` macro.
pub trait RouterDestination: Any {
    /// Renders the destination's page.
    fn render(&self);
}
