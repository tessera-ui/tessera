//! Provides a rectangular highlight component for visually indicating selected regions,
//! typically in text editors or similar UI elements. This module enables rendering of
//! sharp-cornered, shadowless rectangles with configurable size and color, suitable for
//! marking text selections or other highlighted areas. For multi-line or complex selections,
//! multiple highlight rectangles can be composed to cover the desired region.
use tessera_ui::{Color, ComputedData, Px, tessera};

use crate::pipelines::shape::command::ShapeCommand;

/// Draws a rectangular highlight, typically used to indicate selected text regions in a text editor.
///
/// This component renders a single contiguous rectangle with sharp corners and no shadow,
/// suitable for visually marking selected areas. To highlight selections spanning multiple lines
/// or with complex shapes, use multiple `selection_highlight_rect` components, each representing
/// a segment of the selection.
///
/// # Parameters
///
/// - `width`: The width of the highlight rectangle, in physical pixels (`Px`).
/// - `height`: The height of the highlight rectangle, in physical pixels (`Px`).
/// - `color`: The fill color of the rectangle, including alpha for transparency (`Color`).
#[tessera]
pub fn selection_highlight_rect(
    width: Px,
    height: Px,
    color: Color, // RGBA color with alpha for transparency
) {
    measure(Box::new(move |input| {
        let drawable = ShapeCommand::Rect {
            color,
            corner_radii: glam::Vec4::ZERO.into(), // Sharp corners for text selection
            corner_g2: [3.0; 4],                   // g2-like corners
            shadow: None,                          // No shadow for selection highlight
        };

        input.metadata_mut().push_draw_command(drawable);

        // Return the specified size
        Ok(ComputedData { width, height })
    }));
}
