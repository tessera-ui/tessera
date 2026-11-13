//! Drawing and stateful interaction modifier extensions.
//!
//! ## Usage
//!
//! Attach backgrounds, borders, and clickable, toggleable, or selectable behavior to any subtree.

#[cfg(test)]
mod insets_tests;
mod interaction;
mod visual;

use tessera_foundation::modifier::{ClickableArgs, SelectableArgs, ToggleableArgs};
use tessera_ui::{Color, Dp, Modifier, modifier::ModifierCapabilityExt as _};

use crate::shape_def::Shape;

use interaction::{apply_clickable_modifier, apply_selectable_modifier, apply_toggleable_modifier};
use visual::{BackgroundModifierNode, BorderModifierNode};

pub(crate) use interaction::{
    DisabledToggleSemantics, apply_combined_clickable_modifier, with_keyboard_input,
    with_pointer_input,
};

/// Extensions for drawing and stateful interaction around component subtrees.
pub trait ModifierExt {
    /// Draws a background behind the subtree.
    fn background(self, color: Color) -> Modifier;

    /// Draws a background behind the subtree using a custom shape.
    fn background_with_shape(self, color: Color, shape: Shape) -> Modifier;

    /// Draws a background scaled around its center while preserving layout and
    /// input bounds.
    ///
    /// Both scale axes must be finite and in `0..=1`. A zero axis draws
    /// nothing.
    fn background_with_scaled_shape(self, color: Color, shape: Shape, scale: [f32; 2]) -> Modifier;

    /// Draws a border stroke above the subtree.
    fn border(self, width: Dp, color: Color) -> Modifier;

    /// Draws a border stroke above the subtree using a custom shape.
    fn border_with_shape(self, width: Dp, color: Color, shape: Shape) -> Modifier;

    /// Makes the subtree clickable with an accessibility click action.
    fn clickable(self, args: ClickableArgs) -> Modifier;

    /// Makes the subtree toggleable.
    fn toggleable(self, args: ToggleableArgs) -> Modifier;

    /// Makes the subtree selectable.
    fn selectable(self, args: SelectableArgs) -> Modifier;
}

impl ModifierExt for Modifier {
    fn background(self, color: Color) -> Modifier {
        self.background_with_shape(color, Shape::RECTANGLE)
    }

    fn background_with_shape(self, color: Color, shape: Shape) -> Modifier {
        if color.a <= 0.0 {
            return self;
        }

        self.push_draw(BackgroundModifierNode { color, shape })
    }

    fn background_with_scaled_shape(self, color: Color, shape: Shape, scale: [f32; 2]) -> Modifier {
        assert!(
            scale
                .iter()
                .all(|value| value.is_finite() && (0.0..=1.0).contains(value)),
            "background scales must be finite and in 0..=1",
        );
        self.push_draw(visual::ScaledBackgroundModifierNode {
            color,
            shape,
            scale,
        })
    }

    fn border(self, width: Dp, color: Color) -> Modifier {
        self.border_with_shape(width, color, Shape::RECTANGLE)
    }

    fn border_with_shape(self, width: Dp, color: Color, shape: Shape) -> Modifier {
        if width.0 <= 0.0 || color.a <= 0.0 {
            return self;
        }

        self.push_draw(BorderModifierNode {
            width,
            color,
            shape,
        })
    }

    fn clickable(self, args: ClickableArgs) -> Modifier {
        apply_clickable_modifier(self, args)
    }

    fn toggleable(self, args: ToggleableArgs) -> Modifier {
        apply_toggleable_modifier(self, args)
    }

    fn selectable(self, args: SelectableArgs) -> Modifier {
        apply_selectable_modifier(self, args)
    }
}
