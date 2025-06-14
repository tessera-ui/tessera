use std::time::Instant;

use tessera::{BasicDrawable, ComponentNodeMetaData, ComputedData, Dp};
use tessera_macros::tessera;

const CURSOR_WIDRH: Dp = Dp(2.5);

/// A blink cursor component for text editor.
#[tessera]
pub(super) fn cursor(height_px: u32, bink_timer: Instant) {
    // skip the cursor based on the timer
    // to make it blink
    if bink_timer.elapsed().as_millis() % 1000 < 500 {
        return;
    }

    measure(Box::new(move |input| {
        // Cursor is a rectangle with a fixed width and variable height
        let drawable = BasicDrawable::Rect {
            color: [0.0, 0.0, 0.0, 1.0],
            corner_radius: 0.0,
            shadow: None,
        };
        // Add the drawable to the metadata
        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.basic_drawable = Some(drawable);
        } else {
            let default_meta = ComponentNodeMetaData {
                basic_drawable: Some(drawable),
                ..Default::default()
            };
            input.metadatas.insert(input.current_node_id, default_meta);
        }
        // Return the computed data for the cursor
        Ok(ComputedData {
            width: CURSOR_WIDRH.to_pixels_u32(),
            height: height_px,
        })
    }));
}
