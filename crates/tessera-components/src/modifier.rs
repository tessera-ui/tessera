//! Drawing modifier extensions.
//!
//! ## Usage
//!
//! Attach backgrounds and borders to any subtree.

mod input;
#[cfg(test)]
mod insets_tests;
mod visual;

use tessera_ui::{Color, Dp, Modifier, modifier::ModifierCapabilityExt as _};

use crate::shape_def::Shape;

use visual::{BackgroundModifierNode, BorderModifierNode};

pub(crate) use input::{with_keyboard_input, with_pointer_input};

/// Extensions for drawing around component subtrees.
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
}
