//! A progress bar with a glassmorphic visual style.
//!
//! ## Usage
//!
//! Use to indicate the completion of a task or a specific value in a range.
use tessera_ui::{
    Color, ComputedData, Constraint, Dp, MeasurementError, Modifier, Px, PxPosition,
    layout::{LayoutInput, LayoutOutput, LayoutPolicy, layout},
    tessera,
};

use crate::{
    fluid_glass::{GlassBorder, fluid_glass},
    modifier::ModifierExt as _,
    shape_def::{RoundedCorner, Shape},
};

fn default_progress_modifier() -> Modifier {
    Modifier::new().width(Dp(200.0))
}

/// Produce a capsule-shaped RoundedRectangle shape for the given height (px).
fn capsule_shape_for_height(height: Dp) -> Shape {
    let radius = Dp(height.0 / 2.0);
    Shape::RoundedRectangle {
        top_left: RoundedCorner::manual(radius, 2.0),
        top_right: RoundedCorner::manual(radius, 2.0),
        bottom_right: RoundedCorner::manual(radius, 2.0),
        bottom_left: RoundedCorner::manual(radius, 2.0),
    }
}

#[tessera]
fn glass_progress_fill(value: f32, tint_color: Color, blur_radius: Dp, shape: Shape) {
    let value = value.clamp(0.0, 1.0);
    layout()
        .layout_policy(GlassProgressFillLayout { value })
        .child(move || {
            fluid_glass()
                .tint_color(tint_color)
                .blur_radius(blur_radius)
                .shape(shape)
                .with_child(|| {});
        });
}

#[derive(Clone, PartialEq)]
struct GlassProgressFillLayout {
    value: f32,
}

impl LayoutPolicy for GlassProgressFillLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let available_width = input
            .parent_constraint()
            .width()
            .resolve_max()
            .unwrap_or(Px(0));
        let available_height = input
            .parent_constraint()
            .height()
            .resolve_max()
            .unwrap_or(Px(0));

        let width_px = Px((available_width.to_f32() * self.value).round() as i32);
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("progress fill child should exist");

        let child_constraint = Constraint::exact(width_px, available_height);
        input.measure_child(child_id, &child_constraint)?;
        output.place_child(child_id, PxPosition::new(Px(0), Px(0)));

        Ok(ComputedData {
            width: width_px,
            height: available_height,
        })
    }
}

/// # glass_progress
///
/// Renders a progress bar with a customizable glass effect.
///
/// ## Usage
///
/// Display a value in a continuous range (0.0 to 1.0) with a modern, glass-like
/// appearance.
///
/// ## Parameters
///
/// - `value` — progress value in the range `0.0..=1.0`.
/// - `modifier` — optional modifier chain for width and layout.
/// - `height` — optional progress bar height.
/// - `track_tint_color` — optional glass tint color for the track background.
/// - `progress_tint_color` — optional glass tint color for the progress fill.
/// - `blur_radius` — optional blur radius for the glass effect.
/// - `track_border_width` — optional border width for the track.
///
/// ## Examples
///
/// ```
/// use tessera_components::glass_progress::glass_progress;
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// // Render a progress bar at 75% completion.
/// glass_progress().value(0.75);
/// # }
/// # component();
/// ```
#[tessera]
pub fn glass_progress(
    value: f32,
    modifier: Option<Modifier>,
    height: Option<Dp>,
    track_tint_color: Option<Color>,
    progress_tint_color: Option<Color>,
    blur_radius: Option<Dp>,
    track_border_width: Option<Dp>,
) {
    let modifier = modifier.unwrap_or_else(default_progress_modifier);
    let height = height.unwrap_or(Dp(12.0));
    let track_tint_color = track_tint_color.unwrap_or(Color::new(0.3, 0.3, 0.3, 0.15));
    let progress_tint_color = progress_tint_color.unwrap_or(Color::new(0.5, 0.7, 1.0, 0.25));
    let blur_radius = blur_radius.unwrap_or(Dp(8.0));
    let track_border_width = track_border_width.unwrap_or(Dp(1.0));
    let height_px = height.to_px();
    layout()
        .modifier(modifier)
        .layout_policy(GlassProgressLayout { height: height_px })
        .child(move || {
            let effective_height = Dp((height.0 - (track_border_width.0 * 2.0)).max(0.0));
            let fill_shape = capsule_shape_for_height(effective_height);

            fluid_glass()
                .tint_color(track_tint_color)
                .blur_radius(blur_radius)
                .shape(capsule_shape_for_height(height))
                .border(GlassBorder::new(track_border_width.into()))
                .padding(track_border_width)
                .with_child(move || {
                    glass_progress_fill()
                        .value(value)
                        .tint_color(progress_tint_color)
                        .blur_radius(blur_radius)
                        .shape(fill_shape);
                });
        });
}

#[derive(Clone, Copy, PartialEq)]
struct GlassProgressLayout {
    height: Px,
}

impl LayoutPolicy for GlassProgressLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let track_id = input
            .children_ids()
            .first()
            .copied()
            .expect("track should exist");
        let constraint = Constraint::new(input.parent_constraint().width(), self.height);
        let track_measurement = input.measure_child(track_id, &constraint)?;
        output.place_child(track_id, PxPosition::new(Px(0), Px(0)));
        Ok(track_measurement)
    }
}
