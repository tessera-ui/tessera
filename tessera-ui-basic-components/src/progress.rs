//! A linear progress bar component.
//!
//! ## Usage
//!
//! Use to indicate the completion of a task or a specific value in a range.
use derive_builder::Builder;
use tessera_ui::{Color, ComputedData, Constraint, DimensionValue, Dp, Px, PxPosition, tessera};

use crate::{
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

/// Arguments for the `progress` component.
#[derive(Builder, Clone, Debug)]
#[builder(pattern = "owned")]
pub struct ProgressArgs {
    /// The current value of the progress bar, ranging from 0.0 to 1.0.
    #[builder(default = "0.0")]
    pub value: f32,

    /// The width of the progress bar.
    #[builder(default = "Dp(200.0)")]
    pub width: Dp,

    /// The height of the progress bar.
    #[builder(default = "Dp(8.0)")]
    pub height: Dp,

    /// The color of the active part of the track.
    #[builder(default = "Color::new(0.2, 0.5, 0.8, 1.0)")]
    pub progress_color: Color,

    /// The color of the inactive part of the track.
    #[builder(default = "Color::new(0.8, 0.8, 0.8, 1.0)")]
    pub track_color: Color,
}

/// # progress
///
/// Renders a linear progress indicator that visualizes a value from 0.0 to 1.0.
///
/// ## Usage
///
/// Display the status of an ongoing operation, such as a download or a setup process.
///
/// ## Parameters
///
/// - `args` — configures the progress bar's value and appearance; see [`ProgressArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::progress::{progress, ProgressArgsBuilder};
///
/// // Creates a progress bar that is 75% complete.
/// progress(
///     ProgressArgsBuilder::default()
///         .value(0.75)
///         .build()
///         .unwrap(),
/// );
/// ```
#[tessera]
pub fn progress(args: impl Into<ProgressArgs>) {
    let args: ProgressArgs = args.into();

    // Child 1: The background track. It's drawn first.
    surface(
        SurfaceArgsBuilder::default()
            .style(args.track_color.into())
            .shape(Shape::capsule())
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );

    // Child 2: The progress fill. It's drawn on top of the track.
    surface(
        SurfaceArgsBuilder::default()
            .style(args.progress_color.into())
            .shape(Shape::capsule())
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .expect("builder construction failed"),
        None,
        || {},
    );

    measure(Box::new(move |input| {
        let self_width = args.width.to_px();
        let self_height = args.height.to_px();

        let track_id = input.children_ids[0];
        let progress_id = input.children_ids[1];

        // Measure and place the background track to take the full size of the component.
        let track_constraint = Constraint::new(
            DimensionValue::Fixed(self_width),
            DimensionValue::Fixed(self_height),
        );
        input.measure_child(track_id, &track_constraint)?;
        input.place_child(track_id, PxPosition::new(Px(0), Px(0)));

        // Measure and place the progress fill based on the `value`.
        let clamped_value = args.value.clamp(0.0, 1.0);
        let progress_width = Px::saturating_from_f32(self_width.to_f32() * clamped_value);
        let progress_constraint = Constraint::new(
            DimensionValue::Fixed(progress_width),
            DimensionValue::Fixed(self_height),
        );
        input.measure_child(progress_id, &progress_constraint)?;
        input.place_child(progress_id, PxPosition::new(Px(0), Px(0)));

        // The progress component itself is a container, its size is defined by the args.
        Ok(ComputedData {
            width: self_width,
            height: self_height,
        })
    }));
}
