//! Rectangular highlight component for selection visuals.
//!
//! ## Usage
//!
//! Highlight selected text ranges or focusable regions inside editors.
use tessera_ui::{
    Color, ComputedData, LayoutPolicy, LayoutResult, MeasurementError, Px, RenderInput,
    RenderPolicy,
    layout::{MeasureScope, layout},
    tessera,
};

use crate::pipelines::shape::command::ShapeCommand;

/// Draws a rectangular highlight, typically used to indicate selected text
/// regions in a text editor.
///
/// This component renders a single contiguous rectangle with sharp corners and
/// no shadow, suitable for visually marking selected areas. To highlight
/// selections spanning multiple lines or with complex shapes, use multiple
/// `selection_highlight_rect` components, each representing a segment of the
/// selection.
///
/// # Parameters
///
/// - `width`: The width of the highlight rectangle, in physical pixels (`Px`).
/// - `height`: The height of the highlight rectangle, in physical pixels
///   (`Px`).
/// - `color`: The fill color of the rectangle, including alpha for transparency
///   (`Color`).
#[tessera]
pub fn selection_highlight_rect(width: Px, height: Px, color: Color) {
    let policy = SelectionHighlightLayout {
        width,
        height,
        color,
    };
    layout().layout_policy(policy.clone()).render_policy(policy);
}
#[derive(Clone, PartialEq)]
struct SelectionHighlightLayout {
    width: Px,
    height: Px,
    color: Color,
}

impl LayoutPolicy for SelectionHighlightLayout {
    fn measure(&self, _input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData {
            width: self.width,
            height: self.height,
        }))
    }
}

impl RenderPolicy for SelectionHighlightLayout {
    fn record(&self, input: &mut RenderInput<'_>) {
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
