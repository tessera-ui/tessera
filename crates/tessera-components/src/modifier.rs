//! Material modifier extensions for basic components.
//!
//! ## Usage
//!
//! Attach backgrounds, borders, shadows, scrolling, and ripple interactions to any subtree.

#[cfg(test)]
mod insets_tests;
mod interaction;
mod shadow;
mod visual;

use tessera_foundation::modifier::{ClickableArgs, SelectableArgs, ToggleableArgs};
use tessera_ui::{Color, Dp, Modifier, State, modifier::ModifierCapabilityExt as _};

use crate::{scrollable::ScrollableController, shape_def::Shape, theme::MaterialColorScheme};

use interaction::{apply_clickable_modifier, apply_selectable_modifier, apply_toggleable_modifier};
use visual::{BackgroundModifierNode, BorderModifierNode};

pub use shadow::ShadowArgs;

pub(crate) use interaction::{
    DisabledToggleSemantics, apply_combined_clickable_modifier, with_keyboard_input,
    with_pointer_input,
};

/// Material extensions for composing drawing, scrolling, and ripple
/// interaction behavior around component subtrees.
pub trait ModifierExt {
    /// Makes this node a vertical scroll viewport driven by `controller`.
    fn vertical_scroll(self, controller: State<ScrollableController>) -> Modifier;

    /// Makes this node a horizontal scroll viewport driven by `controller`.
    fn horizontal_scroll(self, controller: State<ScrollableController>) -> Modifier;

    /// Makes this node a scroll viewport on the selected axes.
    fn scroll(
        self,
        controller: State<ScrollableController>,
        vertical: bool,
        horizontal: bool,
    ) -> Modifier;

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

    /// Adds a shadow with advanced configuration options.
    fn shadow(self, scheme: &MaterialColorScheme, args: &ShadowArgs) -> Modifier;

    /// Makes the subtree clickable with optional ripple feedback and an
    /// accessibility click action.
    fn clickable(self, args: ClickableArgs) -> Modifier;

    /// Makes the subtree toggleable with optional ripple/state-layer feedback.
    fn toggleable(self, args: ToggleableArgs) -> Modifier;

    /// Makes the subtree selectable with optional ripple/state-layer feedback.
    fn selectable(self, args: SelectableArgs) -> Modifier;
}

impl ModifierExt for Modifier {
    fn vertical_scroll(self, controller: State<ScrollableController>) -> Modifier {
        crate::scrollable::apply_scroll_modifier(self, controller, true, false)
    }

    fn horizontal_scroll(self, controller: State<ScrollableController>) -> Modifier {
        crate::scrollable::apply_scroll_modifier(self, controller, false, true)
    }

    fn scroll(
        self,
        controller: State<ScrollableController>,
        vertical: bool,
        horizontal: bool,
    ) -> Modifier {
        crate::scrollable::apply_scroll_modifier(self, controller, vertical, horizontal)
    }

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

    fn shadow(self, scheme: &MaterialColorScheme, args: &ShadowArgs) -> Modifier {
        shadow::apply_shadow_modifier(self, scheme, args.clone())
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
