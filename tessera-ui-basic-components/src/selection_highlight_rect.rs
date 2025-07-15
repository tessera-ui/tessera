use tessera_ui::{Color, ComputedData, Px};
use tessera_ui_macros::tessera;

use crate::pipelines::ShapeCommand;

/// A single rectangular highlight for text selection
///
/// This component represents one contiguous rectangular area that should be highlighted
/// as part of a text selection. Multiple instances of this component may be used
/// to represent a complete selection that spans multiple lines or has complex geometry.
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
            shadow: None,       // No shadow for selection highlight
        };

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.push_draw_command(drawable);
        }

        // Return the specified size
        Ok(ComputedData { width, height })
    }));
}
