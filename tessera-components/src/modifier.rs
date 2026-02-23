//! Modifier extensions for basic components.
//!
//! ## Usage
//!
//! Attach layout and drawing behavior like padding and opacity to any subtree.

mod interaction;
mod layout;
mod semantics;
mod shadow;
mod visual;

use tessera_ui::{
    Color, DimensionValue, Dp, Modifier, ModifierChild, Px, RenderSlot, WindowAction, use_context,
};

use crate::shape_def::Shape;

use interaction::{
    modifier_block_touch_propagation, modifier_clickable, modifier_selectable, modifier_toggleable,
    modifier_window_action, modifier_window_drag_region,
};
use layout::{
    modifier_constraints, modifier_minimum_interactive_size, modifier_offset, modifier_padding,
};
use visual::{modifier_alpha, modifier_background, modifier_border, modifier_clip_to_bounds};

pub use interaction::{
    ClickableArgs, InteractionState, PointerEventContext, SelectableArgs, ToggleableArgs,
};
pub use layout::{MinimumInteractiveComponentEnforcement, Padding};
pub use semantics::SemanticsArgs;
pub use shadow::ShadowArgs;

fn replayable_modifier_child(child: ModifierChild) -> RenderSlot {
    RenderSlot::new(child)
}

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

    /// Prevents cursor events from propagating to components behind this
    /// subtree.
    fn block_touch_propagation(self) -> Modifier;

    /// Attaches accessibility semantics metadata to this subtree.
    fn semantics(self, args: SemanticsArgs) -> Modifier;

    /// Clears descendant semantics and applies the provided metadata.
    fn clear_and_set_semantics(self, args: SemanticsArgs) -> Modifier;

    /// Makes the subtree clickable with optional ripple feedback and an
    /// accessibility click action.
    fn clickable(self, args: ClickableArgs) -> Modifier;

    /// Makes the subtree toggleable with optional ripple/state-layer feedback.
    fn toggleable(self, args: ToggleableArgs) -> Modifier;

    /// Makes the subtree selectable with optional ripple/state-layer feedback.
    fn selectable(self, args: SelectableArgs) -> Modifier;

    /// Marks this subtree as a draggable window region.
    fn window_drag_region(self) -> Modifier;

    /// Requests a window action when tapped.
    fn window_action(self, action: WindowAction) -> Modifier;
}

impl ModifierExt for Modifier {
    fn padding(self, padding: Padding) -> Modifier {
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_padding(padding, child.clone());
            }
        })
    }

    fn padding_all(self, padding: Dp) -> Modifier {
        self.padding(Padding::all(padding))
    }

    fn padding_symmetric(self, horizontal: Dp, vertical: Dp) -> Modifier {
        self.padding(Padding::symmetric(horizontal, vertical))
    }

    fn offset(self, x: Dp, y: Dp) -> Modifier {
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_offset(x, y, child.clone());
            }
        })
    }

    fn alpha(self, alpha: f32) -> Modifier {
        let alpha = alpha.clamp(0.0, 1.0);
        if (alpha - 1.0).abs() <= f32::EPSILON {
            return self;
        }

        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_alpha(alpha, child.clone());
            }
        })
    }

    fn clip_to_bounds(self) -> Modifier {
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_clip_to_bounds(child.clone());
            }
        })
    }

    fn background(self, color: Color) -> Modifier {
        self.background_with_shape(color, Shape::RECTANGLE)
    }

    fn background_with_shape(self, color: Color, shape: Shape) -> Modifier {
        if color.a <= 0.0 {
            return self;
        }

        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_background(color, shape, child.clone());
            }
        })
    }

    fn border(self, width: Dp, color: Color) -> Modifier {
        self.border_with_shape(width, color, Shape::RECTANGLE)
    }

    fn border_with_shape(self, width: Dp, color: Color, shape: Shape) -> Modifier {
        if width.0 <= 0.0 || color.a <= 0.0 {
            return self;
        }

        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_border(width, color, shape, child.clone());
            }
        })
    }

    fn shadow(self, args: &ShadowArgs) -> Modifier {
        shadow::apply_shadow_modifier(self, args.clone())
    }

    fn size(self, width: Dp, height: Dp) -> Modifier {
        let width_px: Px = width.into();
        let height_px: Px = height.into();
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_constraints(
                    Some(DimensionValue::Wrap {
                        min: Some(width_px),
                        max: Some(width_px),
                    }),
                    Some(DimensionValue::Wrap {
                        min: Some(height_px),
                        max: Some(height_px),
                    }),
                    child.clone(),
                );
            }
        })
    }

    fn width(self, width: Dp) -> Modifier {
        let width_px: Px = width.into();
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_constraints(
                    Some(DimensionValue::Wrap {
                        min: Some(width_px),
                        max: Some(width_px),
                    }),
                    None,
                    child.clone(),
                );
            }
        })
    }

    fn height(self, height: Dp) -> Modifier {
        let height_px: Px = height.into();
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_constraints(
                    None,
                    Some(DimensionValue::Wrap {
                        min: Some(height_px),
                        max: Some(height_px),
                    }),
                    child.clone(),
                );
            }
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
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_constraints(Some(width), Some(height), child.clone());
            }
        })
    }

    fn constrain(self, width: Option<DimensionValue>, height: Option<DimensionValue>) -> Modifier {
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_constraints(width, height, child.clone());
            }
        })
    }

    fn fill_max_width(self) -> Modifier {
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_constraints(Some(DimensionValue::FILLED), None, child.clone());
            }
        })
    }

    fn fill_max_height(self) -> Modifier {
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_constraints(None, Some(DimensionValue::FILLED), child.clone());
            }
        })
    }

    fn fill_max_size(self) -> Modifier {
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_constraints(
                    Some(DimensionValue::FILLED),
                    Some(DimensionValue::FILLED),
                    child.clone(),
                );
            }
        })
    }

    fn minimum_interactive_component_size(self) -> Modifier {
        if !use_context::<MinimumInteractiveComponentEnforcement>()
            .map(|e| e.get().enabled)
            .unwrap_or_else(|| MinimumInteractiveComponentEnforcement::default().enabled)
        {
            return self;
        }

        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_minimum_interactive_size(child.clone());
            }
        })
    }

    fn block_touch_propagation(self) -> Modifier {
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_block_touch_propagation(child.clone());
            }
        })
    }

    fn semantics(self, args: SemanticsArgs) -> Modifier {
        self.push_wrapper(move |child| {
            let args = args.clone();
            let child = replayable_modifier_child(child);
            move || {
                semantics::modifier_semantics(args.clone(), child.clone());
            }
        })
    }

    fn clear_and_set_semantics(self, mut args: SemanticsArgs) -> Modifier {
        args.merge_descendants = false;
        self.semantics(args)
    }

    fn clickable(self, args: ClickableArgs) -> Modifier {
        self.push_wrapper(move |child| {
            let args = args.clone();
            let child = replayable_modifier_child(child);
            move || {
                modifier_clickable(args.clone(), child.clone());
            }
        })
    }

    fn toggleable(self, args: ToggleableArgs) -> Modifier {
        self.push_wrapper(move |child| {
            let args = args.clone();
            let child = replayable_modifier_child(child);
            move || {
                modifier_toggleable(args.clone(), child.clone());
            }
        })
    }

    fn selectable(self, args: SelectableArgs) -> Modifier {
        self.push_wrapper(move |child| {
            let args = args.clone();
            let child = replayable_modifier_child(child);
            move || {
                modifier_selectable(args.clone(), child.clone());
            }
        })
    }

    fn window_drag_region(self) -> Modifier {
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_window_drag_region(child.clone());
            }
        })
    }

    fn window_action(self, action: WindowAction) -> Modifier {
        self.push_wrapper(move |child| {
            let child = replayable_modifier_child(child);
            move || {
                modifier_window_action(action, child.clone());
            }
        })
    }
}
