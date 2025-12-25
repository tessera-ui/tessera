//! Text cursor component for the text edit core system.
//!
//! This module provides a blinking cursor component used within text editing
//! interfaces. The cursor provides visual feedback for text insertion point and
//! blinks at regular intervals to maintain user attention.
use std::time::Instant;

use tessera_ui::{Color, ComputedData, Dp, Px, tessera};

use crate::pipelines::shape::command::ShapeCommand;

/// Width of the text cursor in device-independent pixels.
pub(crate) const CURSOR_WIDRH: Dp = Dp(2.5);

/// A blinking cursor component for text editing interfaces.
///
/// This component renders a vertical line cursor that blinks on and off at
/// regular intervals to indicate the text insertion point. The cursor
/// automatically handles its own blinking animation based on the provided
/// timer.
///
/// # Parameters
///
/// * `height_px` - The height of the cursor in pixels, typically matching the
///   line height
/// * `bink_timer` - Timer used to control the blinking animation cycle
#[tessera]
pub(super) fn cursor(height_px: Px, bink_timer: Instant) {
    // Skip rendering the cursor during the "off" phase of the blink cycle
    // to create the blinking effect (visible for 500ms, hidden for 500ms)
    if bink_timer.elapsed().as_millis() % 1000 < 500 {
        return;
    }

    measure(move |input| {
        // Create a rectangular cursor shape with fixed width and variable height
        let drawable = ShapeCommand::Rect {
            color: Color::BLACK,
            corner_radii: glam::Vec4::ZERO.into(),
            corner_g2: [3.0; 4], // Use G2-like corners
            shadow: None,
        };

        // Add the cursor drawable to the component's metadata for rendering
        input.metadata_mut().push_draw_command(drawable);

        // Return the computed dimensions for layout calculation
        Ok(ComputedData {
            width: CURSOR_WIDRH.into(),
            height: height_px,
        })
    });
}
