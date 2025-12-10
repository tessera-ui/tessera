//! An invisible component for creating empty space in a layout.
//!
//! ## Usage
//!
//! Use to add gaps between components or to create flexible, expanding regions.
use derive_builder::Builder;
use tessera_ui::{ComputedData, Constraint, DimensionValue, Dp, Px, tessera};

/// Arguments for configuring the [`spacer`] component.
///
/// `SpacerArgs` allows you to specify the width and height behavior of a spacer
/// in a layout. By default, both width and height are fixed to zero pixels. To
/// create a flexible spacer that expands to fill available space, use
/// [`DimensionValue::Fill`] for the desired axis.
///
/// # Example
/// ```
/// use tessera_ui::{DimensionValue, Px};
/// use tessera_ui_basic_components::spacer::{SpacerArgs, spacer};
///
/// // Fixed-size spacer (default)
/// spacer(SpacerArgs::default());
///
/// // Expanding spacer (fills available width)
/// spacer(SpacerArgs {
///     width: DimensionValue::Fill {
///         min: None,
///         max: None,
///     },
///     height: DimensionValue::Fixed(Px(0)),
/// });
/// ```
#[derive(Default, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
pub struct SpacerArgs {
    /// The desired width behavior of the spacer.
    ///
    /// Defaults to `Fixed(Px(0))`.
    #[builder(default = "DimensionValue::Fixed(Px(0))", setter(into))]
    pub width: DimensionValue,
    /// The desired height behavior of the spacer.
    ///
    /// Defaults to `Fixed(Px(0))`.
    #[builder(default = "DimensionValue::Fixed(Px(0))", setter(into))]
    pub height: DimensionValue,
}

impl SpacerArgs {
    /// Creates a spacer that tries to fill available space in both width and
    /// height.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::spacer::SpacerArgs;
    /// let args = SpacerArgs::fill_both();
    /// ```
    pub fn fill_both() -> Self {
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .expect("builder construction failed") // build() should not fail with these defaults
    }

    /// Creates a spacer that tries to fill available width.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::spacer::SpacerArgs;
    /// let args = SpacerArgs::fill_width();
    /// ```
    pub fn fill_width() -> Self {
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fixed(Px(0))) // Default height if only filling width
            .build()
            .expect("builder construction failed")
    }

    /// Creates a spacer that tries to fill available height.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::spacer::SpacerArgs;
    /// let args = SpacerArgs::fill_height();
    /// ```
    pub fn fill_height() -> Self {
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fixed(Px(0))) // Default width if only filling height
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .expect("builder construction failed")
    }
}

impl From<Dp> for SpacerArgs {
    /// Creates a fixed-size spacer from a [`Dp`] value for both width and
    /// height.
    ///
    /// # Example
    /// ```
    /// use tessera_ui::Dp;
    /// use tessera_ui_basic_components::spacer::SpacerArgs;
    /// let args = SpacerArgs::from(Dp(8.0));
    /// ```
    fn from(value: Dp) -> Self {
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fixed(value.to_px()))
            .height(DimensionValue::Fixed(value.to_px()))
            .build()
            .expect("builder construction failed")
    }
}

impl From<Px> for SpacerArgs {
    /// Creates a fixed-size spacer from a [`Px`] value for both width and
    /// height.
    ///
    /// # Example
    /// ```
    /// use tessera_ui::Px;
    /// use tessera_ui_basic_components::spacer::SpacerArgs;
    /// let args = SpacerArgs::from(Px(16));
    /// ```
    fn from(value: Px) -> Self {
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fixed(value))
            .height(DimensionValue::Fixed(value))
            .build()
            .expect("builder construction failed")
    }
}

/// # spacer
///
/// Renders an empty, flexible space to influence layout.
///
/// ## Usage
///
/// Add fixed-size gaps or create flexible space that pushes other components
/// apart.
///
/// ## Parameters
///
/// - `args` — configures the spacer's width and height; see [`SpacerArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::{
///     row::{RowArgs, row},
///     spacer::{SpacerArgs, spacer},
///     text::{TextArgsBuilder, text},
/// };
///
/// row(RowArgs::default(), |scope| {
///     scope.child(|| {
///         text(
///             TextArgsBuilder::default()
///                 .text("Left".to_string())
///                 .build()
///                 .expect("builder construction failed"),
///         )
///     });
///     // This spacer will fill the available width, pushing "Right" to the end.
///     scope.child(|| spacer(SpacerArgs::fill_width()));
///     scope.child(|| {
///         text(
///             TextArgsBuilder::default()
///                 .text("Right".to_string())
///                 .build()
///                 .expect("builder construction failed"),
///         )
///     });
/// });
/// ```
#[tessera]
pub fn spacer(args: impl Into<SpacerArgs>) {
    let args: SpacerArgs = args.into();

    measure(Box::new(move |input| {
        let spacer_intrinsic_constraint = Constraint::new(args.width, args.height);
        let effective_spacer_constraint =
            spacer_intrinsic_constraint.merge(input.parent_constraint);

        let final_spacer_width = match effective_spacer_constraint.width {
            DimensionValue::Fixed(w) => w,
            DimensionValue::Wrap { min, .. } => min.unwrap_or(Px(0)), /* Spacer has no content, so it's its min or 0. */
            DimensionValue::Fill { min, max: _ } => {
                // If the effective constraint is Fill, it means the parent allows filling.
                // However, a simple spacer has no content to expand beyond its minimum.
                // The actual size it gets if parent is Fill and allocates space
                // would be determined by the parent's layout logic (e.g. row/column giving it a
                // Fixed size). Here, based purely on
                // `effective_spacer_constraint` being Fill, it should take at
                // least its `min` value. If parent constraint was Fixed(v),
                // merge would result in Fixed(v.clamp(min, max)). If parent was
                // Wrap, merge would result in Fill{min,max} (if spacer was Fill).
                // If parent was Fill{p_min, p_max}, merge would result in Fill{combined_min,
                // combined_max}. In all Fill cases, the spacer itself doesn't
                // "push" for more than its min.
                min.unwrap_or(Px(0))
            }
        };

        let final_spacer_height = match effective_spacer_constraint.height {
            DimensionValue::Fixed(h) => h,
            DimensionValue::Wrap { min, .. } => min.unwrap_or(Px(0)),
            DimensionValue::Fill { min, max: _ } => min.unwrap_or(Px(0)),
        };

        Ok(ComputedData {
            width: final_spacer_width,
            height: final_spacer_height,
        })
    }));
}
