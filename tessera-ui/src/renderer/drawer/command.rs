//! Draw command traits and barrier requirements.
//!
//! This module defines the core traits and types for graphics rendering commands
//! in the unified command system.

use std::any::Any;

/// Specifies synchronization requirements for rendering operations.
///
/// Barrier requirements ensure proper ordering of rendering operations when
/// commands need to sample from previously rendered content.
pub enum BarrierRequirement {
    /// Command needs to sample from the background (previously rendered content).
    ///
    /// This triggers a texture copy operation before the command is executed,
    /// making the previous frame's content available for sampling in shaders.
    SampleBackground,
}

/// Trait providing type erasure capabilities for command objects.
///
/// This trait allows commands to be stored and passed around as trait objects
/// while still providing access to their concrete types when needed for
/// pipeline dispatch.
pub trait AsAny {
    /// Returns a reference to the concrete type as `&dyn Any`.
    fn as_any(&self) -> &dyn Any;
}

/// Blanket implementation of `AsAny` for all types that implement `Any`.
impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Trait for graphics rendering commands that can be processed by draw pipelines.
///
/// Implement this trait for structs that represent graphics operations such as
/// shape drawing, text rendering, image display, or custom visual effects.
///
/// # Example
///
/// ```
/// use tessera_ui::{BarrierRequirement, DrawCommand};
///
/// struct RectangleCommand {
///     color: [f32; 4],
///     corner_radius: f32,
/// }
///
/// impl DrawCommand for RectangleCommand {
///     // Most commands don't need barriers
///     fn barrier(&self) -> Option<BarrierRequirement> {
///         None
///     }
/// }
/// ```
pub trait DrawCommand: AsAny + Send + Sync {
    /// Specifies barrier requirements for this draw operation.
    ///
    /// Return `Some(BarrierRequirement::SampleBackground)` if your command needs
    /// to sample from previously rendered content (e.g., for blur effects or
    /// other post-processing operations).
    ///
    /// # Returns
    ///
    /// - `None` for standard rendering operations (default)
    /// - `Some(BarrierRequirement::SampleBackground)` for operations that sample previous content
    fn barrier(&self) -> Option<BarrierRequirement> {
        None
    }
}
