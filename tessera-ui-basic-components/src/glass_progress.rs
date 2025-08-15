//! Provides a glassmorphism-style progress bar component for visualizing task completion.
//!
//! The `glass_progress` module implements a customizable, frosted glass effect progress bar,
//! featuring a blurred background, tint colors, and borders. It is designed to display a
//! progress value from 0.0 to 1.0, making it suitable for loading screens, dashboards, or
//! any interface requiring a modern and visually appealing progress indicator.

use derive_builder::Builder;
use tessera_ui::{Color, ComputedData, Constraint, DimensionValue, Dp, Px, PxPosition, tessera};

use crate::{
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
    shape_def::Shape,
};

/// Arguments for the `glass_progress` component.
#[derive(Builder, Clone, Debug)]
#[builder(pattern = "owned")]
pub struct GlassProgressArgs {
    /// The current value of the progress bar, ranging from 0.0 to 1.0.
    #[builder(default = "0.0")]
    pub value: f32,

    /// The width of the progress bar.
    #[builder(default = "Dp(200.0)")]
    pub width: Dp,

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
    #[builder(default = "8.0")]
    pub blur_radius: f32,

    /// Border width for the track.
    #[builder(default = "Dp(1.0)")]
    pub track_border_width: Dp,
}

/// Creates a progress bar component with a frosted glass effect.
///
/// The `glass_progress` displays a value from a continuous range (0.0 to 1.0)
/// with a modern, semi-transparent "glassmorphism" aesthetic, including a
/// blurred background and subtle highlights.
///
/// # Arguments
///
/// * `args` - An instance of `GlassProgressArgs` or `GlassProgressArgsBuilder`
///   to configure the progress bar's appearance.
///   - `value`: The current progress value, must be between 0.0 and 1.0.
///
/// # Example
///
/// ```rust,no_run
/// use tessera_ui_basic_components::glass_progress::{glass_progress, GlassProgressArgsBuilder};
///
/// // In your component function
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

    // External track (background) with border - capsule shape
    fluid_glass(
        FluidGlassArgsBuilder::default()
            .width(DimensionValue::Fixed(args.width.to_px()))
            .height(DimensionValue::Fixed(args.height.to_px()))
            .tint_color(args.track_tint_color)
            .blur_radius(args.blur_radius)
            .shape({
                let radius = args.height.to_px().to_f32() / 2.0;
                Shape::RoundedRectangle {
                    top_left: radius,
                    top_right: radius,
                    bottom_right: radius,
                    bottom_left: radius,
                    g2_k_value: 2.0,
                }
            })
            .border(GlassBorder::new(args.track_border_width.into()))
            .padding(args.track_border_width)
            .build()
            .unwrap(),
        None,
        move || {
            // Internal progress fill - capsule shape
            let progress_width = (args.width.to_px().to_f32() * args.value.clamp(0.0, 1.0))
                - (args.track_border_width.to_px().to_f32() * 2.0);
            let effective_height =
                args.height.to_px().to_f32() - (args.track_border_width.to_px().to_f32() * 2.0);

            if progress_width > 0.0 {
                fluid_glass(
                    FluidGlassArgsBuilder::default()
                        .width(DimensionValue::Fixed(Px(progress_width as i32)))
                        .height(DimensionValue::Fill {
                            min: None,
                            max: None,
                        })
                        .tint_color(args.progress_tint_color)
                        .shape({
                            let radius = effective_height / 2.0;
                            Shape::RoundedRectangle {
                                top_left: radius,
                                top_right: radius,
                                bottom_right: radius,
                                bottom_left: radius,
                                g2_k_value: 2.0, // Capsule shape
                            }
                        })
                        .refraction_amount(0.0)
                        .build()
                        .unwrap(),
                    None,
                    || {},
                );
            }
        },
    );

    measure(Box::new(move |input| {
        let self_width = args.width.to_px();
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
