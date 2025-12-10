//! A progress bar with a glassmorphic visual style.
//!
//! ## Usage
//!
//! Use to indicate the completion of a task or a specific value in a range.
use derive_builder::Builder;
use tessera_ui::{Color, ComputedData, Constraint, DimensionValue, Dp, Px, PxPosition, tessera};

use crate::{
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
    shape_def::{RoundedCorner, Shape},
};

/// Arguments for the `glass_progress` component.
#[derive(Builder, Clone, Debug)]
#[builder(pattern = "owned")]
pub struct GlassProgressArgs {
    /// The current value of the progress bar, ranging from 0.0 to 1.0.
    #[builder(default = "0.0")]
    pub value: f32,

    /// The width of the progress bar.
    #[builder(default = "DimensionValue::Fixed(Dp(200.0).to_px())")]
    pub width: DimensionValue,

    /// The height of the progress bar.
    #[builder(default = "Dp(12.0)")]
    pub height: Dp,

    /// Glass tint color for the track background.
    #[builder(default = "Color::new(0.3, 0.3, 0.3, 0.15)")]
    pub track_tint_color: Color,

    /// Glass tint color for the progress fill.
    #[builder(default = "Color::new(0.5, 0.7, 1.0, 0.25)")]
    pub progress_tint_color: Color,

    /// Glass blur radius for all components.
    #[builder(default = "Dp(8.0)")]
    pub blur_radius: Dp,

    /// Border width for the track.
    #[builder(default = "Dp(1.0)")]
    pub track_border_width: Dp,
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

/// Compute progress width and inner effective height (excluding borders).
/// Returns None when progress width is zero or negative.
fn compute_progress_dims(args: &GlassProgressArgs, width_px: Px) -> Option<(Px, f32)> {
    let progress_width = (width_px.to_f32() * args.value.clamp(0.0, 1.0))
        - (args.track_border_width.to_px().to_f32() * 2.0);
    let effective_height =
        args.height.to_px().to_f32() - (args.track_border_width.to_px().to_f32() * 2.0);

    if progress_width > 0.0 {
        Some((Px(progress_width as i32), effective_height))
    } else {
        None
    }
}

fn resolve_width_px(args: &GlassProgressArgs, parent: Option<&Constraint>) -> Px {
    let fallback = Dp(200.0).to_px();
    let base = Constraint::new(args.width, DimensionValue::Fixed(args.height.to_px()));
    let merged = match parent {
        Some(parent_constraint) => base.merge(parent_constraint),
        None => base,
    };

    match merged.width {
        DimensionValue::Fixed(px) => px,
        DimensionValue::Fill { max, .. } | DimensionValue::Wrap { max, .. } => {
            max.unwrap_or(fallback)
        }
    }
}

/// Render the outer track and the inner progress fill.
/// Extracted to reduce the size of `glass_progress` and keep each unit focused.
fn render_track_and_fill(args: GlassProgressArgs, width_px: Px) {
    fluid_glass(
        FluidGlassArgsBuilder::default()
            .width(DimensionValue::Fixed(width_px))
            .height(DimensionValue::Fixed(args.height.to_px()))
            .tint_color(args.track_tint_color)
            .blur_radius(args.blur_radius)
            .shape(capsule_shape_for_height(args.height))
            .border(GlassBorder::new(args.track_border_width.into()))
            .padding(args.track_border_width)
            .build()
            .expect("builder construction failed"),
        move || {
            // Internal progress fill - capsule shape
            if let Some((progress_px, effective_height)) = compute_progress_dims(&args, width_px) {
                fluid_glass(
                    FluidGlassArgsBuilder::default()
                        .width(DimensionValue::Fixed(progress_px))
                        .height(DimensionValue::Fill {
                            min: None,
                            max: None,
                        })
                        .tint_color(args.progress_tint_color)
                        .shape(capsule_shape_for_height(Dp::from_pixels_f32(
                            effective_height,
                        )))
                        .refraction_amount(0.0)
                        .build()
                        .expect("builder construction failed"),
                    || {},
                );
            }
        },
    );
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
/// use tessera_ui_basic_components::glass_progress::{GlassProgressArgsBuilder, glass_progress};
///
/// // Render a progress bar at 75% completion.
/// glass_progress(
///     GlassProgressArgsBuilder::default()
///         .value(0.75)
///         .build()
///         .unwrap(),
/// );
/// ```
#[tessera]
pub fn glass_progress(args: impl Into<GlassProgressArgs>) {
    let args: GlassProgressArgs = args.into();
    let fallback_width = resolve_width_px(&args, None);

    // Render track and inner fill using extracted helper.
    let args_for_render = args.clone();
    render_track_and_fill(args_for_render, fallback_width);

    measure(Box::new(move |input| {
        let self_width = resolve_width_px(&args, Some(input.parent_constraint));
        let self_height = args.height.to_px();

        let track_id = input.children_ids[0];

        // Measure track
        let track_constraint = Constraint::new(
            DimensionValue::Fixed(self_width),
            DimensionValue::Fixed(self_height),
        );
        input.measure_child(track_id, &track_constraint)?;
        input.place_child(track_id, PxPosition::new(Px(0), Px(0)));

        Ok(ComputedData {
            width: self_width,
            height: self_height,
        })
    }));
}
