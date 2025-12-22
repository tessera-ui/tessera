//! Draw command traits and barrier requirements.
//!
//! This module defines the core traits and types for graphics rendering
//! commands in the unified command system.

use crate::{
    dyn_eq::DynPartialEqDraw,
    renderer::command::{DrawRegion, PaddingRect, SampleRegion},
};

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
    /// Specifies sample requirements for this draw operation.
    ///
    /// As a default implementation, this returns `None`, indicating that
    /// the command does not need to sample from previously rendered content.
    ///
    /// Override this method if your command requires sampling from prior
    /// contents.
    fn sample_region(&self) -> Option<SampleRegion> {
        None
    }

    /// Specifies the drawing region for this command.
    ///
    /// As a default implementation, this returns `DrawRegion::PaddedLocal` with
    /// zero padding, indicating that the command draws within its own bounds.
    ///
    /// Override this method if your command draws to a different region but do
    /// not want to affect layout calculations.
    fn draw_region(&self) -> DrawRegion {
        DrawRegion::PaddedLocal(PaddingRect::ZERO)
    }

    /// Applies an opacity multiplier to this command.
    ///
    /// In most cases you must implement this on your command to support
    /// opacity changes in the UI.
    fn apply_opacity(&mut self, opacity: f32);
}
