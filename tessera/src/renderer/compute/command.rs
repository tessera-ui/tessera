//! Compute command trait and related types.
//!
//! This module defines the `ComputeCommand` trait that marks structs as compute operations
//! that can be processed by the GPU compute pipeline system.

use crate::renderer::drawer::command::{AsAny, BarrierRequirement};

/// Trait for GPU compute operations that can be dispatched through the unified command system.
///
/// Implement this trait for structs that represent compute operations such as post-processing
/// effects, physics simulations, or other GPU-accelerated computations.
///
/// # Example
///
/// ```rust,ignore
/// use crate::renderer::compute::ComputeCommand;
///
/// struct BlurCommand {
///     radius: f32,
///     sigma: f32,
/// }
///
/// impl ComputeCommand for BlurCommand {
///     fn barrier(&self) -> Option<BarrierRequirement> {
///         // Blur needs to sample from previously rendered content
///         Some(BarrierRequirement::SampleBackground)
///     }
/// }
/// ```
pub trait ComputeCommand: AsAny + Send + Sync {
    /// Specifies barrier requirements for this compute operation.
    ///
    /// If your operation must wait for the previous operation to complete,
    /// for example, if you need to post-process previously drawn content,
    /// you should return a `BarrierRequirement`.
    ///
    /// # Important Notes
    ///
    /// - Compute pipelines' output texture will be cleared before each barrier
    ///   compute command, so be careful if you want to use it multiple times.
    /// - Only return a barrier requirement if you actually need to sample from
    ///   previous rendering results.
    ///
    /// # Returns
    ///
    /// - `None` for operations that don't need synchronization (default)
    /// - `Some(BarrierRequirement::SampleBackground)` for operations that sample previous content
    fn barrier(&self) -> Option<BarrierRequirement> {
        None
    }
}
