//! Text cursor component for the text edit core system.
//!
//! ## Usage
//!
//! Show insertion point feedback inside text editing components.
use tessera_ui::{
    Color, ComputedData, Dp, LayoutPolicy, LayoutResult, MeasurementError, Px, RenderInput,
    RenderPolicy,
    layout::{MeasureScope, layout},
    tessera,
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

impl LayoutPolicy for CursorLayout {
    fn measure(&self, _input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData {
            width: CURSOR_WIDRH.into(),
            height: self.height,
        }))
    }
}

impl RenderPolicy for CursorLayout {
    fn record(&self, input: &mut RenderInput<'_>) {
        if !self.visible {
            return;
        }

        let drawable = ShapeCommand::Rect {
            color: self.color,
            corner_radii: glam::Vec4::ZERO.into(),
            corner_g2: [3.0; 4],
        };

        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(drawable);
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
/// * `blink_start_frame_nanos` - Frame timestamp where the blink cycle starts
/// * `current_frame_nanos` - Current frame timestamp used to sample visibility
#[tessera]
fn cursor_visual(
    height_px: Px,
    blink_start_frame_nanos: u64,
    current_frame_nanos: u64,
    color: Color,
) {
    let elapsed_nanos = current_frame_nanos.saturating_sub(blink_start_frame_nanos);
    let visible = elapsed_nanos % 1_000_000_000 < 500_000_000;

    let policy = CursorLayout {
        height: height_px,
        visible,
        color,
    };
    layout().layout_policy(policy.clone()).render_policy(policy);
}

pub(super) fn cursor(
    height_px: Px,
    blink_start_frame_nanos: u64,
    current_frame_nanos: u64,
    color: Color,
) {
    cursor_visual()
        .height_px(height_px)
        .blink_start_frame_nanos(blink_start_frame_nanos)
        .current_frame_nanos(current_frame_nanos)
        .color(color);
}
