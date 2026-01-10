//! Text cursor component for the text edit core system.
//!
//! This module provides a blinking cursor component used within text editing
//! interfaces. The cursor provides visual feedback for text insertion point and
//! blinks at regular intervals to maintain user attention.
use std::time::Instant;

use tessera_ui::{
    Color, ComputedData, Dp, LayoutInput, LayoutOutput, LayoutSpec, MeasurementError, Px,
    RenderInput, tessera,
};

use crate::pipelines::shape::command::ShapeCommand;

/// Width of the text cursor in device-independent pixels.
pub(crate) const CURSOR_WIDRH: Dp = Dp(2.5);

#[derive(Clone, PartialEq)]
struct CursorLayout {
    height: Px,
    visible: bool,
    color: Color,
}

impl LayoutSpec for CursorLayout {
    fn measure(
        &self,
        _input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        Ok(ComputedData {
            width: CURSOR_WIDRH.into(),
            height: self.height,
        })
    }

    fn record(&self, input: &RenderInput<'_>) {
        if !self.visible {
            return;
        }

        let drawable = ShapeCommand::Rect {
            color: self.color,
            corner_radii: glam::Vec4::ZERO.into(),
            corner_g2: [3.0; 4],
            shadow: None,
        };

        input.metadata_mut().push_draw_command(drawable);
    }
}

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
pub(super) fn cursor(height_px: Px, bink_timer: Instant, color: Color) {
    let visible = bink_timer.elapsed().as_millis() % 1000 >= 500;

    layout(CursorLayout {
        height: height_px,
        visible,
        color,
    });
}
