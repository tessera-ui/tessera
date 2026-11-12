//! Draw command traits and barrier requirements.
//!
//! This module defines the core traits and types for graphics rendering
//! commands in the unified command system.

use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::DynClone;

use crate::{
    px::PxRect,
    render_graph::DrawGeometry,
    render_scene::{DrawRegion, PaddingRect, SampleRegion},
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
/// use tessera_ui::render::DrawCommand;
///
/// #[derive(Clone, PartialEq)]
/// struct RectangleCommand {
///     color: [f32; 4],
///     corner_radius: f32,
/// }
///
/// impl DrawCommand for RectangleCommand {
///     fn apply_opacity(&mut self, opacity: f32) {
///         self.color[3] *= opacity.clamp(0.0, 1.0);
///     }
/// }
/// ```
pub trait DrawCommand: DynClone + Downcast + Send + Sync {
    /// Whether unchanged commands can keep their pixels between frames.
    ///
    /// Opt in only when every visual change invalidates the command's paint
    /// fragment or geometry. Leave this disabled for commands whose textures or
    /// other resources can change independently, including deferred uploads.
    /// Applying opacity must preserve the declared draw and sampling regions.
    fn supports_partial_redraw(&self) -> bool {
        false
    }

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

    /// Returns an absolute rectangle used for ordering decisions.
    ///
    /// The default implementation returns `None`, which falls back to the draw
    /// region derived from the command size and position.
    fn ordering_rect(&self, _geometry: DrawGeometry) -> Option<PxRect> {
        None
    }
}

impl_downcast!(DrawCommand);

dyn_clone::clone_trait_object!(DrawCommand);
