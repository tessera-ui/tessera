//! Provides a rectangular highlight component for visually indicating selected regions,
//! typically in text editors or similar UI elements. This module enables rendering of
//! sharp-cornered, shadowless rectangles with configurable size and color, suitable for
//! marking text selections or other highlighted areas. For multi-line or complex selections,
//! multiple highlight rectangles can be composed to cover the desired region.
use tessera_ui::{Color, ComputedData, Px};
use tessera_ui_macros::tessera;

use crate::pipelines::ShapeCommand;

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
///
/// # Example
///
/// ```
/// use tessera_ui::{Color, Px};
/// use tessera_ui_basic_components::selection_highlight_rect::selection_highlight_rect;
///
/// // Renders a selection highlight rectangle with a width of 100px, a height of 20px,
/// // and a semi-transparent blue color.
/// selection_highlight_rect(
///     Px(100),
///     Px(20),
///     Color::new(0.2, 0.4, 1.0, 0.3),
/// );
/// ```
///
#[tessera]
pub fn selection_highlight_rect(
    width: Px,
    height: Px,
    color: Color, // RGBA color with alpha for transparency
) {
    measure(Box::new(move |input| {
        let drawable = ShapeCommand::Rect {
            color,
            corner_radius: 0.0, // Sharp corners for text selection
            g2_k_value: 3.0,    // g2-like corners
            shadow: None,       // No shadow for selection highlight
        };

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.push_draw_command(drawable);
        }

        // Return the specified size
        Ok(ComputedData { width, height })
    }));
}
