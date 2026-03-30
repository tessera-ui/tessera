//! Modifier extensions for basic components.
//!
//! ## Usage
//!
//! Attach layout and drawing behavior like padding and opacity to any subtree.

mod interaction;
mod shadow;
mod visual;

use tessera_foundation::modifier::ModifierExt as FoundationModifierExt;
use tessera_ui::{
    Callback, CallbackWith, Color, DimensionValue, Dp, Modifier, WindowAction,
    modifier::ModifierCapabilityExt as _, use_context,
};

pub use tessera_foundation::modifier::{
    ClickableArgs, InteractionState, MinimumInteractiveComponentEnforcement, Padding,
    PointerEventContext, SelectableArgs, SemanticsArgs, ToggleableArgs,
};

pub(crate) use tessera_foundation::modifier::{AlignmentParentData, WeightParentData};

use crate::{alignment::Alignment, shape_def::Shape};

use interaction::{
    apply_block_touch_propagation_modifier, apply_clickable_modifier, apply_selectable_modifier,
    apply_toggleable_modifier, apply_window_action_modifier, apply_window_drag_region_modifier,
};
use visual::{AlphaModifierNode, BackgroundModifierNode, BorderModifierNode, ClipModifierNode};

pub use shadow::ShadowArgs;

pub(crate) use interaction::{with_keyboard_input, with_pointer_input};

/// Extensions for composing reusable wrapper behavior around component
/// subtrees.
pub trait ModifierExt {
    /// Adds padding around the content.
    fn padding(self, padding: Padding) -> Modifier;

    /// Adds symmetric padding on all edges.
    fn padding_all(self, padding: Dp) -> Modifier;

    /// Adds symmetric padding for horizontal and vertical edges.
    fn padding_symmetric(self, horizontal: Dp, vertical: Dp) -> Modifier;

    /// Offsets the content without affecting layout size.
    fn offset(self, x: Dp, y: Dp) -> Modifier;

    /// Multiplies the opacity of the subtree by `alpha`.
    fn alpha(self, alpha: f32) -> Modifier;

    /// Clips descendants to this modifier's bounds.
    fn clip_to_bounds(self) -> Modifier;

    /// Draws a background behind the subtree.
    fn background(self, color: Color) -> Modifier;

    /// Draws a background behind the subtree using a custom shape.
    fn background_with_shape(self, color: Color, shape: Shape) -> Modifier;

    /// Draws a border stroke above the subtree.
    fn border(self, width: Dp, color: Color) -> Modifier;

    /// Draws a border stroke above the subtree using a custom shape.
    fn border_with_shape(self, width: Dp, color: Color, shape: Shape) -> Modifier;

    /// Adds a shadow with advanced configuration options.
    fn shadow(self, args: &ShadowArgs) -> Modifier;

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

    /// Prevents cursor events from propagating to components behind this
    /// subtree.
    fn block_touch_propagation(self) -> Modifier;

    /// Attaches accessibility semantics metadata to this subtree.
    fn semantics(self, args: SemanticsArgs) -> Modifier;

    /// Clears descendant semantics and applies the provided metadata.
    fn clear_and_set_semantics(self, args: SemanticsArgs) -> Modifier;

    /// Makes the subtree clickable with optional ripple feedback and an
    /// accessibility click action.
    fn clickable<C>(self, on_click: C) -> Modifier
    where
        C: Into<Callback>;

    /// Makes the subtree clickable with advanced configuration options.
    fn clickable_with(self, args: ClickableArgs) -> Modifier;

    /// Makes the subtree toggleable with optional ripple/state-layer feedback.
    fn toggleable<C>(self, value: bool, on_value_change: C) -> Modifier
    where
        C: Into<CallbackWith<bool, ()>>;

    /// Makes the subtree toggleable with advanced configuration options.
    fn toggleable_with(self, args: ToggleableArgs) -> Modifier;

    /// Makes the subtree selectable with optional ripple/state-layer feedback.
    fn selectable<C>(self, selected: bool, on_click: C) -> Modifier
    where
        C: Into<Callback>;

    /// Makes the subtree selectable with advanced configuration options.
    fn selectable_with(self, args: SelectableArgs) -> Modifier;

    /// Marks this subtree as a draggable window region.
    fn window_drag_region(self) -> Modifier;

    /// Requests a window action when tapped.
    fn window_action(self, action: WindowAction) -> Modifier;
}

impl ModifierExt for Modifier {
    fn padding(self, padding: Padding) -> Modifier {
        FoundationModifierExt::padding(self, padding)
    }

    fn padding_all(self, padding: Dp) -> Modifier {
        FoundationModifierExt::padding(self, Padding::all(padding))
    }

    fn padding_symmetric(self, horizontal: Dp, vertical: Dp) -> Modifier {
        FoundationModifierExt::padding(self, Padding::symmetric(horizontal, vertical))
    }

    fn offset(self, x: Dp, y: Dp) -> Modifier {
        FoundationModifierExt::offset(self, x, y)
    }

    fn alpha(self, alpha: f32) -> Modifier {
        let alpha = alpha.clamp(0.0, 1.0);
        if (alpha - 1.0).abs() <= f32::EPSILON {
            return self;
        }

        self.push_draw(AlphaModifierNode { alpha })
    }

    fn clip_to_bounds(self) -> Modifier {
        self.push_draw(ClipModifierNode)
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

    fn shadow(self, args: &ShadowArgs) -> Modifier {
        shadow::apply_shadow_modifier(self, args.clone())
    }

    fn size(self, width: Dp, height: Dp) -> Modifier {
        FoundationModifierExt::size(self, width, height)
    }

    fn width(self, width: Dp) -> Modifier {
        FoundationModifierExt::width(self, width)
    }

    fn height(self, height: Dp) -> Modifier {
        FoundationModifierExt::height(self, height)
    }

    fn size_in(
        self,
        min_width: Option<Dp>,
        max_width: Option<Dp>,
        min_height: Option<Dp>,
        max_height: Option<Dp>,
    ) -> Modifier {
        FoundationModifierExt::size_in(self, min_width, max_width, min_height, max_height)
    }

    fn constrain(self, width: Option<DimensionValue>, height: Option<DimensionValue>) -> Modifier {
        FoundationModifierExt::constrain(self, width, height)
    }

    fn fill_max_width(self) -> Modifier {
        FoundationModifierExt::fill_max_width(self)
    }

    fn fill_max_height(self) -> Modifier {
        FoundationModifierExt::fill_max_height(self)
    }

    fn fill_max_size(self) -> Modifier {
        FoundationModifierExt::fill_max_size(self)
    }

    fn minimum_interactive_component_size(self) -> Modifier {
        if !use_context::<MinimumInteractiveComponentEnforcement>()
            .map(|e| e.get().enabled)
            .unwrap_or_else(|| MinimumInteractiveComponentEnforcement::default().enabled)
        {
            return self;
        }

        FoundationModifierExt::minimum_interactive_component_size(self)
    }

    fn weight(self, weight: f32) -> Modifier {
        FoundationModifierExt::weight(self, weight)
    }

    fn align(self, alignment: Alignment) -> Modifier {
        FoundationModifierExt::align(self, alignment)
    }

    fn block_touch_propagation(self) -> Modifier {
        apply_block_touch_propagation_modifier(self)
    }

    fn semantics(self, args: SemanticsArgs) -> Modifier {
        FoundationModifierExt::semantics(self, args)
    }

    fn clear_and_set_semantics(self, mut args: SemanticsArgs) -> Modifier {
        args.merge_descendants = false;
        FoundationModifierExt::semantics(self, args)
    }

    fn clickable<C>(self, on_click: C) -> Modifier
    where
        C: Into<Callback>,
    {
        self.clickable_with(ClickableArgs {
            on_click: on_click.into(),
            ..Default::default()
        })
    }

    fn clickable_with(self, args: ClickableArgs) -> Modifier {
        apply_clickable_modifier(self, args)
    }

    fn toggleable<C>(self, value: bool, on_value_change: C) -> Modifier
    where
        C: Into<CallbackWith<bool, ()>>,
    {
        self.toggleable_with(ToggleableArgs {
            value,
            on_value_change: on_value_change.into(),
            ..Default::default()
        })
    }

    fn toggleable_with(self, args: ToggleableArgs) -> Modifier {
        apply_toggleable_modifier(self, args)
    }

    fn selectable<C>(self, selected: bool, on_click: C) -> Modifier
    where
        C: Into<Callback>,
    {
        self.selectable_with(SelectableArgs {
            selected,
            on_click: on_click.into(),
            ..Default::default()
        })
    }

    fn selectable_with(self, args: SelectableArgs) -> Modifier {
        apply_selectable_modifier(self, args)
    }

    fn window_drag_region(self) -> Modifier {
        apply_window_drag_region_modifier(self)
    }

    fn window_action(self, action: WindowAction) -> Modifier {
        apply_window_action_modifier(self, action)
    }
}
