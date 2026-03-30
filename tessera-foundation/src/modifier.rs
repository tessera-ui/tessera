//! Shared foundational modifier extensions.
//!
//! ## Usage
//!
//! Attach reusable layout and semantics behavior to any component subtree.

mod interaction;
mod layout;
mod semantics;

use tessera_ui::{DimensionValue, Dp, Modifier, Px, modifier::ModifierCapabilityExt as _};

use crate::alignment::Alignment;

use layout::{
    AlignmentParentDataModifierNode, ConstraintModifierNode, MinimumInteractiveModifierNode,
    OffsetModifierNode, PaddingModifierNode, WeightParentDataModifierNode,
};

pub use interaction::{
    ClickableArgs, InteractionState, PointerEventContext, SelectableArgs, ToggleableArgs,
};
pub use layout::{
    AlignmentParentData, MinimumInteractiveComponentEnforcement, Padding, WeightParentData,
};
pub use semantics::SemanticsArgs;

/// Shared modifier extensions that are not tied to a specific design system.
pub trait ModifierExt {
    /// Adds padding around the content.
    fn padding(self, padding: Padding) -> Modifier;

    /// Adds symmetric padding on all edges.
    fn padding_all(self, padding: Dp) -> Modifier;

    /// Adds symmetric padding for horizontal and vertical edges.
    fn padding_symmetric(self, horizontal: Dp, vertical: Dp) -> Modifier;

    /// Offsets the content without affecting layout size.
    fn offset(self, x: Dp, y: Dp) -> Modifier;

    /// Constrains the content to an exact size when possible.
    fn size(self, width: Dp, height: Dp) -> Modifier;

    /// Constrains the content to an exact width when possible.
    fn width(self, width: Dp) -> Modifier;

    /// Constrains the content to an exact height when possible.
    fn height(self, height: Dp) -> Modifier;

    /// Constrains the content size within optional min/max bounds.
    fn size_in(
        self,
        min_width: Option<Dp>,
        max_width: Option<Dp>,
        min_height: Option<Dp>,
        max_height: Option<Dp>,
    ) -> Modifier;

    /// Applies explicit width/height `DimensionValue` constraints.
    fn constrain(self, width: Option<DimensionValue>, height: Option<DimensionValue>) -> Modifier;

    /// Fills the available width within parent bounds.
    fn fill_max_width(self) -> Modifier;

    /// Fills the available height within parent bounds.
    fn fill_max_height(self) -> Modifier;

    /// Fills the available size within parent bounds.
    fn fill_max_size(self) -> Modifier;

    /// Enforces a minimum interactive size by expanding and centering content.
    fn minimum_interactive_component_size(self) -> Modifier;

    /// Provides weighted parent data for row and column layouts.
    fn weight(self, weight: f32) -> Modifier;

    /// Provides alignment parent data for layered boxed layouts.
    fn align(self, alignment: Alignment) -> Modifier;

    /// Attaches accessibility semantics metadata to this subtree.
    fn semantics(self, args: SemanticsArgs) -> Modifier;

    /// Clears descendant semantics and applies the provided metadata.
    fn clear_and_set_semantics(self, args: SemanticsArgs) -> Modifier;
}

impl ModifierExt for Modifier {
    fn padding(self, padding: Padding) -> Modifier {
        self.push_layout(PaddingModifierNode { padding })
    }

    fn padding_all(self, padding: Dp) -> Modifier {
        self.padding(Padding::all(padding))
    }

    fn padding_symmetric(self, horizontal: Dp, vertical: Dp) -> Modifier {
        self.padding(Padding::symmetric(horizontal, vertical))
    }

    fn offset(self, x: Dp, y: Dp) -> Modifier {
        self.push_layout(OffsetModifierNode { x, y })
    }

    fn size(self, width: Dp, height: Dp) -> Modifier {
        let width_px: Px = width.into();
        let height_px: Px = height.into();
        self.push_layout(ConstraintModifierNode {
            width_override: Some(DimensionValue::Wrap {
                min: Some(width_px),
                max: Some(width_px),
            }),
            height_override: Some(DimensionValue::Wrap {
                min: Some(height_px),
                max: Some(height_px),
            }),
        })
    }

    fn width(self, width: Dp) -> Modifier {
        let width_px: Px = width.into();
        self.push_layout(ConstraintModifierNode {
            width_override: Some(DimensionValue::Wrap {
                min: Some(width_px),
                max: Some(width_px),
            }),
            height_override: None,
        })
    }

    fn height(self, height: Dp) -> Modifier {
        let height_px: Px = height.into();
        self.push_layout(ConstraintModifierNode {
            width_override: None,
            height_override: Some(DimensionValue::Wrap {
                min: Some(height_px),
                max: Some(height_px),
            }),
        })
    }

    fn size_in(
        self,
        min_width: Option<Dp>,
        max_width: Option<Dp>,
        min_height: Option<Dp>,
        max_height: Option<Dp>,
    ) -> Modifier {
        let width = DimensionValue::Wrap {
            min: min_width.map(Into::into),
            max: max_width.map(Into::into),
        };
        let height = DimensionValue::Wrap {
            min: min_height.map(Into::into),
            max: max_height.map(Into::into),
        };
        self.push_layout(ConstraintModifierNode {
            width_override: Some(width),
            height_override: Some(height),
        })
    }

    fn constrain(self, width: Option<DimensionValue>, height: Option<DimensionValue>) -> Modifier {
        self.push_layout(ConstraintModifierNode {
            width_override: width,
            height_override: height,
        })
    }

    fn fill_max_width(self) -> Modifier {
        self.push_layout(ConstraintModifierNode {
            width_override: Some(DimensionValue::Fill {
                min: None,
                max: None,
            }),
            height_override: None,
        })
    }

    fn fill_max_height(self) -> Modifier {
        self.push_layout(ConstraintModifierNode {
            width_override: None,
            height_override: Some(DimensionValue::Fill {
                min: None,
                max: None,
            }),
        })
    }

    fn fill_max_size(self) -> Modifier {
        self.push_layout(ConstraintModifierNode {
            width_override: Some(DimensionValue::Fill {
                min: None,
                max: None,
            }),
            height_override: Some(DimensionValue::Fill {
                min: None,
                max: None,
            }),
        })
    }

    fn minimum_interactive_component_size(self) -> Modifier {
        self.push_layout(MinimumInteractiveModifierNode)
    }

    fn weight(self, weight: f32) -> Modifier {
        self.push_parent_data(WeightParentDataModifierNode { weight })
    }

    fn align(self, alignment: Alignment) -> Modifier {
        self.push_parent_data(AlignmentParentDataModifierNode { alignment })
    }

    fn semantics(self, args: SemanticsArgs) -> Modifier {
        semantics::apply_semantics_modifier(self, args)
    }

    fn clear_and_set_semantics(self, mut args: SemanticsArgs) -> Modifier {
        args.merge_descendants = false;
        semantics::apply_semantics_modifier(self, args)
    }
}
