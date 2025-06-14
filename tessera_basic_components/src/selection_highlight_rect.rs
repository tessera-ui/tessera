use tessera::{BasicDrawable, ComponentNodeMetaData, ComputedData};
use tessera_macros::tessera;

/// A single rectangular highlight for text selection
///
/// This component represents one contiguous rectangular area that should be highlighted
/// as part of a text selection. Multiple instances of this component may be used
/// to represent a complete selection that spans multiple lines or has complex geometry.
#[tessera]
pub fn selection_highlight_rect(
    width: u32,
    height: u32,
    color: [f32; 4], // RGBA color with alpha for transparency
) {
    measure(Box::new(move |input| {
        let drawable = BasicDrawable::Rect {
            color,
            corner_radius: 0.0, // Sharp corners for text selection
            shadow: None,       // No shadow for selection highlight
        };

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.basic_drawable = Some(drawable);
        } else {
            input.metadatas.insert(
                input.current_node_id,
                ComponentNodeMetaData {
                    basic_drawable: Some(drawable),
                    ..Default::default()
                },
            );
        }

        // Return the specified size
        Ok(ComputedData { width, height })
    }));
}
