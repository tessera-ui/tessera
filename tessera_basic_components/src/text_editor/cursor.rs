//! Text cursor component for the text editor system.
//!
//! This module provides a blinking cursor component specifically designed for text editor
//! interfaces. The cursor provides visual feedback for the text insertion point and
//! automatically handles blinking animation to maintain user attention.

use std::time::Instant;

use tessera::{BasicDrawable, ComponentNodeMetaData, ComputedData, Dp};
use tessera_macros::tessera;

/// Width of the text cursor in device-independent pixels.
const CURSOR_WIDRH: Dp = Dp(2.5);

/// A blinking cursor component for text editor interfaces.
///
/// This component renders a vertical line cursor that blinks on and off at regular
/// intervals to indicate the current text insertion point. The cursor automatically
/// manages its own blinking animation cycle based on the provided timer.
///
/// # Parameters
///
/// * `height_px` - The height of the cursor in pixels (u32), typically matching line height
/// * `bink_timer` - Timer instance used to control the blinking animation timing
///
/// # Blinking Animation
///
/// The cursor follows a standard 1-second blinking cycle:
/// - Visible for the first 500ms of each second
/// - Hidden for the remaining 500ms of each second
/// - Cycle repeats continuously while the component is active
///
/// # Example
///
/// ```rust,ignore
/// use std::time::Instant;
///
/// // Create a cursor with specific height and current time
/// cursor(20, Instant::now());
/// ```
///
/// # Rendering Details
///
/// The cursor is rendered as a solid black rectangle with:
/// - Fixed width of 2.5 device-independent pixels
/// - Variable height specified by the `height_px` parameter
/// - Solid black color (RGBA: 0.0, 0.0, 0.0, 1.0)
/// - No corner radius for sharp rectangular appearance
/// - No shadow effects for clean, minimal appearance
///
/// # Implementation Notes
///
/// This component uses the legacy `BasicDrawable` system and may be updated
/// in future versions to use the newer rendering pipeline.
#[tessera]
pub(super) fn cursor(height_px: u32, bink_timer: Instant) {
    // Skip rendering during the "off" phase of the blink cycle
    // Creates the blinking effect: visible for 500ms, hidden for 500ms
    if bink_timer.elapsed().as_millis() % 1000 < 500 {
        return;
    }

    measure(Box::new(move |node_id, _, _, _, metadatas| {
        // Create a rectangular cursor drawable with solid black color
        let drawable = BasicDrawable::Rect {
            color: [0.0, 0.0, 0.0, 1.0], // Solid black (RGBA)
            corner_radius: 0.0,          // Sharp corners
            shadow: None,                // No shadow effect
        };

        // Add the cursor drawable to the component's metadata
        if let Some(mut metadata) = metadatas.get_mut(&node_id) {
            metadata.basic_drawable = Some(drawable);
        } else {
            // Create new metadata if none exists for this node
            let default_meta = ComponentNodeMetaData {
                basic_drawable: Some(drawable),
                ..Default::default()
            };
            metadatas.insert(node_id, default_meta);
        }

        // Return the computed dimensions for layout system
        Ok(ComputedData {
            width: CURSOR_WIDRH.to_pixels_u32(),
            height: height_px,
        })
    }));
}
