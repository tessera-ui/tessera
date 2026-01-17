//! Rectangular highlight component for selection visuals.
//!
//! ## Usage
//!
//! Highlight selected text ranges or focusable regions inside editors.
use tessera_ui::{
    Color, ComputedData, LayoutInput, LayoutOutput, LayoutSpec, MeasurementError, Px, RenderInput,
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
pub fn selection_highlight_rect(
    width: Px,
    height: Px,
    color: Color, // RGBA color with alpha for transparency
) {
    layout(SelectionHighlightLayout {
        width,
        height,
        color,
    });
}
#[derive(Clone, PartialEq)]
struct SelectionHighlightLayout {
    width: Px,
    height: Px,
    color: Color,
}

impl LayoutSpec for SelectionHighlightLayout {
    fn measure(
        &self,
        _input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        Ok(ComputedData {
            width: self.width,
            height: self.height,
        })
    }

    fn record(&self, input: &RenderInput<'_>) {
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
