//! Text cursor component for the text edit core system.
//!
//! ## Usage
//!
//! Show insertion point feedback inside text editing components.
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
fn cursor_node(args: &CursorArgs) {
    let elapsed_nanos = args
        .current_frame_nanos
        .saturating_sub(args.blink_start_frame_nanos);
    let visible = elapsed_nanos % 1_000_000_000 >= 500_000_000;

    layout(CursorLayout {
        height: args.height_px,
        visible,
        color: args.color,
    });
}

#[derive(Clone, PartialEq)]
struct CursorArgs {
    height_px: Px,
    blink_start_frame_nanos: u64,
    current_frame_nanos: u64,
    color: Color,
}

pub(super) fn cursor(
    height_px: Px,
    blink_start_frame_nanos: u64,
    current_frame_nanos: u64,
    color: Color,
) {
    let args = CursorArgs {
        height_px,
        blink_start_frame_nanos,
        current_frame_nanos,
        color,
    };
    cursor_node(&args);
}
