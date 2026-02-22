//! A progress bar with a glassmorphic visual style.
//!
//! ## Usage
//!
//! Use to indicate the completion of a task or a specific value in a range.
use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier, Px,
    PxPosition,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    tessera,
};

use crate::{
    fluid_glass::{FluidGlassArgs, GlassBorder, fluid_glass},
    modifier::ModifierExt as _,
    shape_def::{RoundedCorner, Shape},
};

/// Arguments for the `glass_progress` component.
#[derive(PartialEq, Clone, Debug, Setters)]
pub struct GlassProgressArgs {
    /// The current value of the progress bar, ranging from 0.0 to 1.0.
    pub value: f32,

    /// Layout modifiers applied to the progress bar.
    pub modifier: Modifier,

    /// The height of the progress bar.
    pub height: Dp,

    /// Glass tint color for the track background.
    pub track_tint_color: Color,

    /// Glass tint color for the progress fill.
    pub progress_tint_color: Color,

    /// Glass blur radius for all components.
    pub blur_radius: Dp,

    /// Border width for the track.
    pub track_border_width: Dp,
}

impl Default for GlassProgressArgs {
    fn default() -> Self {
        Self {
            value: 0.0,
            modifier: default_progress_modifier(),
            height: Dp(12.0),
            track_tint_color: Color::new(0.3, 0.3, 0.3, 0.15),
            progress_tint_color: Color::new(0.5, 0.7, 1.0, 0.25),
            blur_radius: Dp(8.0),
            track_border_width: Dp(1.0),
        }
    }
}

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

#[derive(Clone, PartialEq)]
struct GlassProgressFillArgs {
    value: f32,
    tint_color: Color,
    blur_radius: Dp,
    shape: Shape,
}

#[tessera]
fn glass_progress_fill_node(args: &GlassProgressFillArgs) {
    let value = args.value;
    let tint_color = args.tint_color;
    let blur_radius = args.blur_radius;
    let shape = args.shape;
    fluid_glass(&crate::fluid_glass::FluidGlassArgs::with_child(
        FluidGlassArgs::default()
            .tint_color(tint_color)
            .blur_radius(blur_radius)
            .shape(shape)
            .refraction_amount(0.0),
        || {},
    ));

    let value = value.clamp(0.0, 1.0);
    layout(GlassProgressFillLayout { value });
}

#[derive(Clone, PartialEq)]
struct GlassProgressFillLayout {
    value: f32,
}

impl LayoutSpec for GlassProgressFillLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let available_width = match input.parent_constraint().width() {
            DimensionValue::Fixed(px) => px,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(Px(0)),
            DimensionValue::Fill { max, .. } => max.expect(
                "Seems that you are trying to fill an infinite width, which is not allowed",
            ),
        };
        let available_height = match input.parent_constraint().height() {
            DimensionValue::Fixed(px) => px,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(Px(0)),
            DimensionValue::Fill { max, .. } => max.expect(
                "Seems that you are trying to fill an infinite height, which is not allowed",
            ),
        };

        let width_px = Px((available_width.to_f32() * self.value).round() as i32);
        let child_id = input
            .children_ids()
            .first()
            .copied()
            .expect("progress fill child should exist");

        let child_constraint = Constraint::new(
            DimensionValue::Fixed(width_px),
            DimensionValue::Fixed(available_height),
        );
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
/// - `args` — configures the progress bar's value and appearance; see
///   [`GlassProgressArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::glass_progress::{GlassProgressArgs, glass_progress};
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// // Render a progress bar at 75% completion.
/// glass_progress(&GlassProgressArgs::default().value(0.75));
/// # }
/// # component();
/// ```
#[tessera]
pub fn glass_progress(args: &GlassProgressArgs) {
    let args = args.clone();
    let modifier = args.modifier.clone();

    modifier.run(move || {
        let inner_args = args.clone();
        glass_progress_inner_node(&inner_args);
    });
}

#[tessera]
fn glass_progress_inner_node(args: &GlassProgressArgs) {
    let args = args.clone();
    let effective_height = Dp((args.height.0 - (args.track_border_width.0 * 2.0)).max(0.0));
    let fill_shape = capsule_shape_for_height(effective_height);

    fluid_glass(&crate::fluid_glass::FluidGlassArgs::with_child(
        FluidGlassArgs::default()
            .tint_color(args.track_tint_color)
            .blur_radius(args.blur_radius)
            .shape(capsule_shape_for_height(args.height))
            .border(GlassBorder::new(args.track_border_width.into()))
            .padding(args.track_border_width),
        move || {
            let fill_args = GlassProgressFillArgs {
                value: args.value,
                tint_color: args.progress_tint_color,
                blur_radius: args.blur_radius,
                shape: fill_shape,
            };
            glass_progress_fill_node(&fill_args);
        },
    ));

    let height = args.height.to_px();
    layout(GlassProgressLayout { height });
}

#[derive(Clone, Copy, PartialEq)]
struct GlassProgressLayout {
    height: Px,
}

impl LayoutSpec for GlassProgressLayout {
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
        let constraint = Constraint::new(
            input.parent_constraint().width(),
            DimensionValue::Fixed(self.height),
        );
        let track_measurement = input.measure_child(track_id, &constraint)?;
        output.place_child(track_id, PxPosition::new(Px(0), Px(0)));
        Ok(track_measurement)
    }
}
