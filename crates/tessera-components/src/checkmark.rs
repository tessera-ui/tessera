//! Animated checkmark and partial-selection strokes.
//!
//! ## Usage
//!
//! Show selection feedback in boolean and aggregate selection controls.

use tessera_ui::{
    Color, ComputedData, Dp, LayoutResult, MeasurementError, Px,
    layout::{MeasureScope, layout},
    tessera,
};

use crate::pipelines::checkmark::command::CheckmarkCommand;

#[derive(Clone, PartialEq)]
struct CheckmarkLayout {
    size: Px,
    color: Color,
    stroke_width: f32,
    progress: f32,
    padding: [f32; 2],
    indeterminate_fraction: f32,
    checkbox_geometry: bool,
}

impl tessera_ui::LayoutModifierNode for CheckmarkLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.size == other.size
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other)
    }

    fn measure(&self, _input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData {
            width: self.size,
            height: self.size,
        }))
    }
}

impl tessera_ui::DrawModifierNode for CheckmarkLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let input = &mut *ctx.render_input;
        let command = CheckmarkCommand::new()
            .with_color(self.color)
            .with_stroke_width(self.stroke_width)
            .with_progress(self.progress)
            .with_indeterminate_fraction(self.indeterminate_fraction)
            .with_checkbox_geometry(self.checkbox_geometry)
            .with_padding(self.padding[0], self.padding[1]);
        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(command);
        content.draw(input);
    }
}

/// Draw a selection stroke, blending a checkmark into a centered
/// partial-selection dash.
#[tessera]
pub(crate) fn checkmark(
    color: Option<Color>,
    stroke_width: Option<f32>,
    progress: Option<f32>,
    padding: Option<[f32; 2]>,
    size: Option<Dp>,
    indeterminate_fraction: Option<f32>,
    checkbox_geometry: Option<bool>,
) {
    let color = color.unwrap_or(Color::new(0.0, 0.6, 0.0, 1.0));
    let stroke_width = stroke_width.unwrap_or(5.0);
    let progress = progress.unwrap_or(1.0);
    let padding = padding.unwrap_or([2.0, 2.0]);
    let size = size.unwrap_or(Dp(20.0));
    let size_px = size.to_px();
    let policy = CheckmarkLayout {
        size: Px::new(size_px.to_f32() as i32),
        color,
        stroke_width,
        progress,
        padding,
        indeterminate_fraction: indeterminate_fraction
            .filter(|fraction| fraction.is_finite())
            .unwrap_or(0.0),
        checkbox_geometry: checkbox_geometry.unwrap_or(false),
    };
    layout().policy(policy.clone()).draw(policy);
}
