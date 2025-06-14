use derive_builder::Builder;
use tessera::{ComputedData, Constraint, DimensionValue};
use tessera_macros::tessera;

/// Arguments for the Spacer component.
#[derive(Default, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
pub struct SpacerArgs {
    /// The desired width behavior of the spacer.
    /// Defaults to Fixed(0). Use Fill { min: None, max: None } for an expanding spacer.
    #[builder(default = "DimensionValue::Fixed(0)")]
    pub width: DimensionValue,
    /// The desired height behavior of the spacer.
    /// Defaults to Fixed(0). Use Fill { min: None, max: None } for an expanding spacer.
    #[builder(default = "DimensionValue::Fixed(0)")]
    pub height: DimensionValue,
}

impl SpacerArgs {
    /// Creates a spacer that tries to fill available space in both dimensions.
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
            .unwrap() // build() should not fail with these defaults
    }

    /// Creates a spacer that tries to fill available width.
    pub fn fill_width() -> Self {
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fixed(0)) // Default height if only filling width
            .build()
            .unwrap()
    }

    /// Creates a spacer that tries to fill available height.
    pub fn fill_height() -> Self {
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fixed(0)) // Default width if only filling height
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap()
    }
}

/// A component that creates an empty space in the layout.
///
/// `Spacer` can be used to add gaps between other components or to fill available space.
/// Its behavior is defined by the `width` and `height` `DimensionValue` parameters.
#[tessera]
pub fn spacer(args: SpacerArgs) {
    measure(Box::new(move |input| {
        let spacer_intrinsic_constraint = Constraint::new(args.width, args.height);
        let effective_spacer_constraint =
            spacer_intrinsic_constraint.merge(input.effective_constraint);

        let final_spacer_width = match effective_spacer_constraint.width {
            DimensionValue::Fixed(w) => w,
            DimensionValue::Wrap { min, .. } => min.unwrap_or(0), // Spacer has no content, so it's its min or 0.
            DimensionValue::Fill { min, max: _ } => {
                // If the effective constraint is Fill, it means the parent allows filling.
                // However, a simple spacer has no content to expand beyond its minimum.
                // The actual size it gets if parent is Fill and allocates space
                // would be determined by the parent's layout logic (e.g. Row/Column giving it a Fixed size).
                // Here, based purely on `effective_spacer_constraint` being Fill,
                // it should take at least its `min` value.
                // If parent constraint was Fixed(v), merge would result in Fixed(v.clamp(min, max)).
                // If parent was Wrap, merge would result in Fill{min,max} (if spacer was Fill).
                // If parent was Fill{p_min, p_max}, merge would result in Fill{combined_min, combined_max}.
                // In all Fill cases, the spacer itself doesn't "push" for more than its min.
                min.unwrap_or(0)
            }
        };

        let final_spacer_height = match effective_spacer_constraint.height {
            DimensionValue::Fixed(h) => h,
            DimensionValue::Wrap { min, .. } => min.unwrap_or(0),
            DimensionValue::Fill { min, max: _ } => min.unwrap_or(0),
        };

        Ok(ComputedData {
            width: final_spacer_width,
            height: final_spacer_height,
        })
    }));
}
