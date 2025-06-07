use derive_builder::Builder;
use tessera::{ComputedData, Constraint, DimensionValue}; // Removed NodeId, ComponentNodeMetaDatas
use tessera_macros::tessera;

/// Arguments for the Spacer component.
#[derive(Default, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
pub struct SpacerArgs {
    /// The desired width behavior of the spacer.
    /// Defaults to Fixed(0). Use Fill { max: None } for an expanding spacer.
    #[builder(default = "DimensionValue::Fixed(0)")]
    pub width: DimensionValue,
    /// The desired height behavior of the spacer.
    /// Defaults to Fixed(0). Use Fill { max: None } for an expanding spacer.
    #[builder(default = "DimensionValue::Fixed(0)")]
    pub height: DimensionValue,
}

impl SpacerArgs {
    /// Creates a spacer that tries to fill available space in both dimensions.
    pub fn fill_both() -> Self {
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fill { max: None })
            .height(DimensionValue::Fill { max: None })
            .build()
            .unwrap() // build() should not fail with these defaults
    }

    /// Creates a spacer that tries to fill available width.
    pub fn fill_width() -> Self {
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fill { max: None })
            .height(DimensionValue::Fixed(0)) // Default height if only filling width
            .build()
            .unwrap()
    }

    /// Creates a spacer that tries to fill available height.
    pub fn fill_height() -> Self {
        SpacerArgsBuilder::default()
            .width(DimensionValue::Fixed(0)) // Default width if only filling height
            .height(DimensionValue::Fill { max: None })
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
    // Spacer has no children. Its size is determined by its args and parent constraints.
    measure(Box::new(
        move |_node_id, _tree, parent_constraint, _children_node_ids, _metadatas| {
            // 1. Spacer's intrinsic constraint from its arguments
            let spacer_intrinsic_constraint = Constraint::new(args.width, args.height);

            // 2. Merge with parent_constraint to get effective_spacer_constraint
            // This is what the parent wants/allows the spacer to be.
            let effective_spacer_constraint = spacer_intrinsic_constraint.merge(parent_constraint);

            // 3. Calculate final Spacer dimensions based on the effective_spacer_constraint
            // Since Spacer has no content, Wrap resolves to 0.
            // Fill { max: None } relies on the parent providing a concrete size via parent_constraint.
            // If parent_constraint also doesn't provide a concrete size for a Fill {max: None} spacer,
            // it effectively becomes 0.
            let final_spacer_width = match effective_spacer_constraint.width {
                DimensionValue::Fixed(w) => w,
                DimensionValue::Wrap => 0, // Spacer has no content to wrap
                DimensionValue::Fill { max: Some(max_w) } => max_w, // Fills up to this max
                DimensionValue::Fill { max: None } => {
                    // If parent_constraint resolved this Fill to a Fixed value, that value would be here.
                    // If it's still Fill {max: None}, it means parent didn't give a size, so it's 0.
                    // This case is typically handled when a parent (like Row/Column) measures a Fill child:
                    // the parent calculates available space and passes a Fixed constraint to the child.
                    // If Spacer is measured directly with a Fill{None} constraint from parent, it means 0.
                    0
                }
            };

            let final_spacer_height = match effective_spacer_constraint.height {
                DimensionValue::Fixed(h) => h,
                DimensionValue::Wrap => 0, // Spacer has no content to wrap
                DimensionValue::Fill { max: Some(max_h) } => max_h, // Fills up to this max
                DimensionValue::Fill { max: None } => {
                    // Similar logic as width.
                    0
                }
            };

            ComputedData {
                width: final_spacer_width,
                height: final_spacer_height,
            }
        },
    ));
    // Spacer has no children, so the children rendering part is empty.
}
