//! Draw command traits and barrier requirements.
//!
//! This module defines the core traits and types for graphics rendering
//! commands in the unified command system.

use crate::{dyn_eq::DynPartialEqDraw, renderer::command::BarrierRequirement};

/// Trait for graphics rendering commands that can be processed by draw
/// pipelines.
///
/// Implement this trait for structs that represent graphics operations such as
/// shape drawing, text rendering, image display, or custom visual effects.
///
/// # Example
///
/// ```
/// use tessera_ui::{BarrierRequirement, DrawCommand};
///
/// #[derive(PartialEq, Clone)]
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
pub trait DrawCommand: DynPartialEqDraw + Send + Sync {
    /// Specifies barrier requirements for this draw operation.
    ///
    /// Return `Some(BarrierRequirement::SampleBackground)` if your command
    /// needs to sample from previously rendered content (e.g., for blur
    /// effects or other post-processing operations).
    ///
    /// # Returns
    ///
    /// - `None` for standard rendering operations (default)
    /// - `Some(BarrierRequirement::SampleBackground)` for operations that
    ///   sample previous content
    fn barrier(&self) -> Option<BarrierRequirement> {
        None
    }

    /// Applies an opacity multiplier to this command.
    ///
    /// The default implementation is a no-op; override to scale internal color
    /// data when group opacity is applied.
    fn apply_opacity(&mut self, opacity: f32) {
        let _ = opacity;
    }
}
