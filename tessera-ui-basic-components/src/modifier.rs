//! Modifier extensions for basic components.
//!
//! ## Usage
//!
//! Attach layout and drawing behavior like padding and opacity to any subtree.

use std::{mem, sync::Arc};

use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, GestureState,
    Modifier, PressKeyEventType, Px, PxPosition, PxSize, State,
    accesskit::{self, Action, Toggled},
    tessera, use_context,
    winit::window::CursorIcon,
};

use crate::{
    pipelines::shape::command::ShapeCommand,
    pos_misc::is_position_in_rect,
    ripple_state::{RippleSpec, RippleState},
    shape_def::{ResolvedShape, Shape},
};

use layout::{
    modifier_constraints, modifier_minimum_interactive_size, modifier_offset, modifier_padding,
    resolve_dimension,
};

pub use layout::{MinimumInteractiveComponentEnforcement, Padding};
pub use shadow::ShadowArgs;

mod layout;
mod shadow;

/// Argument structs for complex interactive modifiers to improve call-site
/// readability.
///
/// Use these builders to configure `clickable`, `toggleable` and `selectable`
/// modifiers without long positional parameter lists.
/// Arguments for the `clickable` modifier.
#[derive(Clone)]
pub struct ClickableArgs {
    /// Callback invoked when the element is clicked.
    pub on_click: Arc<dyn Fn() + Send + Sync>,
    /// Whether the element is enabled for interaction.
    pub enabled: bool,
    /// Optional accessibility role (defaults to `Button`).
    pub role: Option<accesskit::Role>,
    /// Optional accessibility label.
    pub label: Option<String>,
    /// Optional accessibility description.
    pub description: Option<String>,
    /// Optional external ripple/interaction state.
    pub interaction_state: Option<State<RippleState>>,
    /// Optional ripple customization spec.
    pub ripple_spec: Option<RippleSpec>,
    /// Optional explicit ripple size.
    pub ripple_size: Option<PxSize>,
}

impl ClickableArgs {
    /// Create a new `ClickableArgs` with the required `on_click` handler.
    pub fn new(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self {
            on_click,
            enabled: true,
            role: None,
            label: None,
            description: None,
            interaction_state: None,
            ripple_spec: None,
            ripple_size: None,
        }
    }

    /// Set whether the control is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the accessibility role.
    pub fn role(mut self, role: accesskit::Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Set an accessibility label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set an accessibility description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Attach an external ripple/interaction `State`.
    pub fn interaction_state(mut self, state: State<RippleState>) -> Self {
        self.interaction_state = Some(state);
        self
    }

    /// Provide a custom ripple spec.
    pub fn ripple_spec(mut self, spec: RippleSpec) -> Self {
        self.ripple_spec = Some(spec);
        self
    }

    /// Provide an explicit ripple size.
    pub fn ripple_size(mut self, size: PxSize) -> Self {
        self.ripple_size = Some(size);
        self
    }
}

/// Arguments for the `toggleable` modifier.
#[derive(Clone)]
pub struct ToggleableArgs {
    /// Current boolean value.
    pub value: bool,
    /// Callback invoked with the new value when changed.
    pub on_value_change: Arc<dyn Fn(bool) + Send + Sync>,
    /// Whether the control is enabled for interaction.
    pub enabled: bool,
    /// Optional accessibility role (defaults to `CheckBox` or similar).
    pub role: Option<accesskit::Role>,
    /// Optional accessibility label.
    pub label: Option<String>,
    /// Optional accessibility description.
    pub description: Option<String>,
    /// Optional external ripple/interaction state.
    pub interaction_state: Option<State<RippleState>>,
    /// Optional ripple customization spec.
    pub ripple_spec: Option<RippleSpec>,
    /// Optional explicit ripple size.
    pub ripple_size: Option<PxSize>,
}

impl ToggleableArgs {
    /// Create a new `ToggleableArgs` with the required `value` and
    /// `on_value_change`.
    pub fn new(value: bool, on_value_change: Arc<dyn Fn(bool) + Send + Sync>) -> Self {
        Self {
            value,
            on_value_change,
            enabled: true,
            role: None,
            label: None,
            description: None,
            interaction_state: None,
            ripple_spec: None,
            ripple_size: None,
        }
    }

    /// Set whether the control is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the accessibility role.
    pub fn role(mut self, role: accesskit::Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Set an accessibility label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set an accessibility description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Attach an external ripple/interaction `State`.
    pub fn interaction_state(mut self, state: State<RippleState>) -> Self {
        self.interaction_state = Some(state);
        self
    }

    /// Provide a custom ripple spec.
    pub fn ripple_spec(mut self, spec: RippleSpec) -> Self {
        self.ripple_spec = Some(spec);
        self
    }

    /// Provide an explicit ripple size.
    pub fn ripple_size(mut self, size: PxSize) -> Self {
        self.ripple_size = Some(size);
        self
    }
}

/// Arguments for the `selectable` modifier.
#[derive(Clone)]
pub struct SelectableArgs {
    /// Whether the item is selected.
    pub selected: bool,
    /// Callback invoked when the selectable is activated.
    pub on_click: Arc<dyn Fn() + Send + Sync>,
    /// Whether the element is enabled for interaction.
    pub enabled: bool,
    /// Optional accessibility role (defaults to `Button` or specific role).
    pub role: Option<accesskit::Role>,
    /// Optional accessibility label.
    pub label: Option<String>,
    /// Optional accessibility description.
    pub description: Option<String>,
    /// Optional external ripple/interaction state.
    pub interaction_state: Option<State<RippleState>>,
    /// Optional ripple customization spec.
    pub ripple_spec: Option<RippleSpec>,
    /// Optional explicit ripple size.
    pub ripple_size: Option<PxSize>,
}

impl SelectableArgs {
    /// Create a new `SelectableArgs` with the required `selected` and
    /// `on_click`.
    pub fn new(selected: bool, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self {
            selected,
            on_click,
            enabled: true,
            role: None,
            label: None,
            description: None,
            interaction_state: None,
            ripple_spec: None,
            ripple_size: None,
        }
    }

    /// Set whether the control is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the accessibility role.
    pub fn role(mut self, role: accesskit::Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Set an accessibility label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set an accessibility description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Attach an external ripple/interaction `State`.
    pub fn interaction_state(mut self, state: State<RippleState>) -> Self {
        self.interaction_state = Some(state);
        self
    }

    /// Provide a custom ripple spec.
    pub fn ripple_spec(mut self, spec: RippleSpec) -> Self {
        self.ripple_spec = Some(spec);
        self
    }

    /// Provide an explicit ripple size.
    pub fn ripple_size(mut self, size: PxSize) -> Self {
        self.ripple_size = Some(size);
        self
    }
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
    fn shadow(self, args: impl Into<ShadowArgs>) -> Modifier;

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

    /// Makes the subtree clickable with optional ripple feedback and an
    /// accessibility click action.
    fn clickable(self, args: ClickableArgs) -> Modifier;

    /// Makes the subtree toggleable with optional ripple/state-layer feedback.
    fn toggleable(self, args: ToggleableArgs) -> Modifier;

    /// Makes the subtree selectable with optional ripple/state-layer feedback.
    fn selectable(self, args: SelectableArgs) -> Modifier;
}

impl ModifierExt for Modifier {
    fn padding(self, padding: Padding) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_padding(padding, || {
                    child();
                });
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
            move || {
                modifier_offset(x, y, || {
                    child();
                });
            }
        })
    }

    fn alpha(self, alpha: f32) -> Modifier {
        let alpha = alpha.clamp(0.0, 1.0);
        if (alpha - 1.0).abs() <= f32::EPSILON {
            return self;
        }

        self.push_wrapper(move |child| {
            move || {
                modifier_alpha(alpha, || {
                    child();
                });
            }
        })
    }

    fn clip_to_bounds(self) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_clip_to_bounds(|| {
                    child();
                });
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
            move || {
                modifier_background(color, shape, || {
                    child();
                });
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
            move || {
                modifier_border(width, color, shape, || {
                    child();
                });
            }
        })
    }

    fn shadow(self, args: impl Into<ShadowArgs>) -> Modifier {
        shadow::apply_shadow_modifier(self, args.into())
    }

    fn size(self, width: Dp, height: Dp) -> Modifier {
        let width_px: Px = width.into();
        let height_px: Px = height.into();
        self.push_wrapper(move |child| {
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
                    || {
                        child();
                    },
                );
            }
        })
    }

    fn width(self, width: Dp) -> Modifier {
        let width_px: Px = width.into();
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(
                    Some(DimensionValue::Wrap {
                        min: Some(width_px),
                        max: Some(width_px),
                    }),
                    None,
                    || {
                        child();
                    },
                );
            }
        })
    }

    fn height(self, height: Dp) -> Modifier {
        let height_px: Px = height.into();
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(
                    None,
                    Some(DimensionValue::Wrap {
                        min: Some(height_px),
                        max: Some(height_px),
                    }),
                    || {
                        child();
                    },
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
            move || {
                modifier_constraints(Some(width), Some(height), || {
                    child();
                });
            }
        })
    }

    fn constrain(self, width: Option<DimensionValue>, height: Option<DimensionValue>) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(width, height, || {
                    child();
                });
            }
        })
    }

    fn fill_max_width(self) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(Some(DimensionValue::FILLED), None, || {
                    child();
                });
            }
        })
    }

    fn fill_max_height(self) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(None, Some(DimensionValue::FILLED), || {
                    child();
                });
            }
        })
    }

    fn fill_max_size(self) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_constraints(
                    Some(DimensionValue::FILLED),
                    Some(DimensionValue::FILLED),
                    || {
                        child();
                    },
                );
            }
        })
    }

    fn minimum_interactive_component_size(self) -> Modifier {
        if !use_context::<MinimumInteractiveComponentEnforcement>()
            .get()
            .enabled
        {
            return self;
        }

        self.push_wrapper(move |child| {
            move || {
                modifier_minimum_interactive_size(|| {
                    child();
                });
            }
        })
    }

    fn block_touch_propagation(self) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_block_touch_propagation(|| {
                    child();
                });
            }
        })
    }

    fn clickable(self, args: ClickableArgs) -> Modifier {
        self.push_wrapper(move |child| {
            let args = args.clone();
            move || {
                modifier_clickable(args, || {
                    child();
                });
            }
        })
    }

    fn toggleable(self, args: ToggleableArgs) -> Modifier {
        self.push_wrapper(move |child| {
            let args = args.clone();
            move || {
                modifier_toggleable(args, || {
                    child();
                });
            }
        })
    }

    fn selectable(self, args: SelectableArgs) -> Modifier {
        self.push_wrapper(move |child| {
            let args = args.clone();
            move || {
                modifier_selectable(args, || {
                    child();
                });
            }
        })
    }
}

#[tessera]
fn modifier_alpha<F>(alpha: f32, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_alpha expects exactly one child");

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let final_width =
            resolve_dimension(parent_constraint.width, child_measurement.width, "width");
        let final_height =
            resolve_dimension(parent_constraint.height, child_measurement.height, "height");

        input.place_child(child_id, PxPosition::ZERO);
        input.multiply_opacity(alpha);

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
}

#[tessera]
fn modifier_clip_to_bounds<F>(child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        input.enable_clipping();

        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_clip_to_bounds expects exactly one child");

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let final_width =
            resolve_dimension(parent_constraint.width, child_measurement.width, "width");
        let final_height =
            resolve_dimension(parent_constraint.height, child_measurement.height, "height");

        input.place_child(child_id, PxPosition::ZERO);

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
}

#[tessera]
fn modifier_clickable<F>(args: ClickableArgs, child: F)
where
    F: FnOnce(),
{
    let ClickableArgs {
        on_click,
        enabled,
        role,
        label,
        description,
        interaction_state,
        ripple_spec,
        ripple_size,
    } = args;

    child();

    let role = role.unwrap_or(accesskit::Role::Button);
    input_handler(Box::new(move |input| {
        let mut cursor_events = Vec::new();
        mem::swap(&mut cursor_events, input.cursor_events);

        let mut unhandled_events = Vec::new();
        let mut processed_events = Vec::new();

        for event in cursor_events {
            if matches!(
                event.content,
                CursorEventContent::Pressed(PressKeyEventType::Left)
                    | CursorEventContent::Released(PressKeyEventType::Left)
            ) {
                processed_events.push(event);
            } else {
                unhandled_events.push(event);
            }
        }

        input.cursor_events.extend(unhandled_events);
        let cursor_events = processed_events;

        let within_bounds = input
            .cursor_position_rel
            .map(|pos| {
                is_position_in_rect(
                    pos,
                    PxPosition::ZERO,
                    input.computed_data.width,
                    input.computed_data.height,
                )
            })
            .unwrap_or(false);

        if enabled && within_bounds {
            input.requests.cursor_icon = CursorIcon::Pointer;
        }

        let mut builder = input.accessibility().role(role);
        if let Some(label) = label.as_ref() {
            builder = builder.label(label.clone());
        }
        if let Some(description) = description.as_ref() {
            builder = builder.description(description.clone());
        }
        builder = if enabled {
            builder.action(Action::Click).focusable()
        } else {
            builder.disabled()
        };
        builder.commit();

        if enabled {
            let on_click_action = on_click.clone();
            input.set_accessibility_action_handler(move |action| {
                if action == Action::Click {
                    on_click_action();
                }
            });
        }

        let Some(interaction_state) = interaction_state else {
            if !enabled {
                return;
            }

            for event in cursor_events.iter() {
                if within_bounds
                    && event.gesture_state == GestureState::TapCandidate
                    && matches!(
                        event.content,
                        CursorEventContent::Released(PressKeyEventType::Left)
                    )
                {
                    on_click();
                }
            }
            return;
        };

        if enabled {
            interaction_state.with_mut(|s| s.set_hovered(within_bounds));
        } else {
            interaction_state.with_mut(|s| {
                s.release();
                s.set_hovered(false);
            });
            return;
        }

        let spec = ripple_spec.unwrap_or(RippleSpec {
            bounded: true,
            radius: None,
        });
        let size = ripple_size.unwrap_or(PxSize::new(
            input.computed_data.width,
            input.computed_data.height,
        ));
        let click_pos = normalized_click_position(input.cursor_position_rel, input.computed_data);

        for event in cursor_events.iter() {
            if within_bounds
                && matches!(
                    event.content,
                    CursorEventContent::Pressed(PressKeyEventType::Left)
                )
            {
                interaction_state.with_mut(|s| {
                    s.start_animation_with_spec(click_pos, size, spec);
                    s.set_pressed(true);
                });
            }

            if matches!(
                event.content,
                CursorEventContent::Released(PressKeyEventType::Left)
            ) {
                interaction_state.with_mut(|s| s.release());
            }

            if within_bounds
                && event.gesture_state == GestureState::TapCandidate
                && matches!(
                    event.content,
                    CursorEventContent::Released(PressKeyEventType::Left)
                )
            {
                on_click();
            }
        }

        if !within_bounds {
            interaction_state.with_mut(|s| {
                s.release();
                s.set_hovered(false);
            });
        }
    }));
}

#[tessera]
fn modifier_block_touch_propagation<F>(child: F)
where
    F: FnOnce(),
{
    child();

    input_handler(Box::new(move |mut input| {
        let within_bounds = input
            .cursor_position_rel
            .map(|pos| {
                is_position_in_rect(
                    pos,
                    PxPosition::ZERO,
                    input.computed_data.width,
                    input.computed_data.height,
                )
            })
            .unwrap_or(false);

        if within_bounds {
            input.block_cursor();
        }
    }));
}

fn normalized_click_position(position: Option<PxPosition>, size: ComputedData) -> [f32; 2] {
    let Some(position) = position else {
        return [0.5, 0.5];
    };
    let width = size.width.to_f32().max(1.0);
    let height = size.height.to_f32().max(1.0);
    let x = (position.x.to_f32() / width).clamp(0.0, 1.0);
    let y = (position.y.to_f32() / height).clamp(0.0, 1.0);
    [x, y]
}

#[tessera]
fn modifier_toggleable<F>(args: ToggleableArgs, child: F)
where
    F: FnOnce(),
{
    let ToggleableArgs {
        value,
        on_value_change,
        enabled,
        role,
        label,
        description,
        interaction_state,
        ripple_spec,
        ripple_size,
    } = args;

    child();

    let role = role.unwrap_or(accesskit::Role::CheckBox);
    input_handler(Box::new(move |input| {
        let mut cursor_events = Vec::new();
        mem::swap(&mut cursor_events, input.cursor_events);

        let mut unhandled_events = Vec::new();
        let mut processed_events = Vec::new();

        for event in cursor_events {
            if matches!(
                event.content,
                CursorEventContent::Pressed(PressKeyEventType::Left)
                    | CursorEventContent::Released(PressKeyEventType::Left)
            ) {
                processed_events.push(event);
            } else {
                unhandled_events.push(event);
            }
        }

        input.cursor_events.extend(unhandled_events);
        let cursor_events = processed_events;

        let within_bounds = input
            .cursor_position_rel
            .map(|pos| {
                is_position_in_rect(
                    pos,
                    PxPosition::ZERO,
                    input.computed_data.width,
                    input.computed_data.height,
                )
            })
            .unwrap_or(false);

        if enabled && within_bounds {
            input.requests.cursor_icon = CursorIcon::Pointer;
        }

        let mut builder = input.accessibility().role(role);
        if let Some(label) = label.as_ref() {
            builder = builder.label(label.clone());
        }
        if let Some(description) = description.as_ref() {
            builder = builder.description(description.clone());
        }
        builder = builder.toggled(if value { Toggled::True } else { Toggled::False });

        builder = if enabled {
            builder.action(Action::Click).focusable()
        } else {
            builder.disabled()
        };
        builder.commit();

        if enabled {
            let on_value_change = on_value_change.clone();
            input.set_accessibility_action_handler(move |action| {
                if action == Action::Click {
                    on_value_change(!value);
                }
            });
        }

        let Some(interaction_state) = interaction_state else {
            return;
        };

        if enabled {
            interaction_state.with_mut(|s| s.set_hovered(within_bounds));
        } else {
            interaction_state.with_mut(|s| {
                s.release();
                s.set_hovered(false);
            });
            return;
        }

        let spec = ripple_spec.unwrap_or(RippleSpec {
            bounded: true,
            radius: None,
        });
        let size = ripple_size.unwrap_or(PxSize::new(
            input.computed_data.width,
            input.computed_data.height,
        ));
        let click_pos = normalized_click_position(input.cursor_position_rel, input.computed_data);

        for event in cursor_events.iter() {
            if within_bounds
                && matches!(
                    event.content,
                    CursorEventContent::Pressed(PressKeyEventType::Left)
                )
            {
                interaction_state.with_mut(|s| {
                    s.start_animation_with_spec(click_pos, size, spec);
                    s.set_pressed(true);
                });
            }

            if matches!(
                event.content,
                CursorEventContent::Released(PressKeyEventType::Left)
            ) {
                interaction_state.with_mut(|s| s.release());
            }

            if within_bounds
                && event.gesture_state == GestureState::TapCandidate
                && matches!(
                    event.content,
                    CursorEventContent::Released(PressKeyEventType::Left)
                )
            {
                on_value_change(!value);
            }
        }

        if !within_bounds {
            interaction_state.with_mut(|s| {
                s.release();
                s.set_hovered(false);
            });
        }
    }));
}

#[tessera]
fn modifier_selectable<F>(args: SelectableArgs, child: F)
where
    F: FnOnce(),
{
    let SelectableArgs {
        selected,
        on_click,
        enabled,
        role,
        label,
        description,
        interaction_state,
        ripple_spec,
        ripple_size,
    } = args;

    child();

    let role = role.unwrap_or(accesskit::Role::Button);
    input_handler(Box::new(move |input| {
        let mut cursor_events = Vec::new();
        mem::swap(&mut cursor_events, input.cursor_events);

        let mut unhandled_events = Vec::new();
        let mut processed_events = Vec::new();

        for event in cursor_events {
            if matches!(
                event.content,
                CursorEventContent::Pressed(PressKeyEventType::Left)
                    | CursorEventContent::Released(PressKeyEventType::Left)
            ) {
                processed_events.push(event);
            } else {
                unhandled_events.push(event);
            }
        }

        input.cursor_events.extend(unhandled_events);
        let cursor_events = processed_events;

        let within_bounds = input
            .cursor_position_rel
            .map(|pos| {
                is_position_in_rect(
                    pos,
                    PxPosition::ZERO,
                    input.computed_data.width,
                    input.computed_data.height,
                )
            })
            .unwrap_or(false);

        if enabled && within_bounds {
            input.requests.cursor_icon = CursorIcon::Pointer;
        }

        let mut builder = input.accessibility().role(role);
        if let Some(label) = label.as_ref() {
            builder = builder.label(label.clone());
        }
        if let Some(description) = description.as_ref() {
            builder = builder.description(description.clone());
        }
        builder = builder.toggled(if selected {
            Toggled::True
        } else {
            Toggled::False
        });

        builder = if enabled {
            builder.action(Action::Click).focusable()
        } else {
            builder.disabled()
        };
        builder.commit();

        if enabled {
            let on_click = on_click.clone();
            input.set_accessibility_action_handler(move |action| {
                if action == Action::Click {
                    on_click();
                }
            });
        }

        let Some(interaction_state) = interaction_state else {
            return;
        };

        if enabled {
            interaction_state.with_mut(|s| s.set_hovered(within_bounds));
        } else {
            interaction_state.with_mut(|s| {
                s.release();
                s.set_hovered(false);
            });
            return;
        }

        let spec = ripple_spec.unwrap_or(RippleSpec {
            bounded: true,
            radius: None,
        });
        let size = ripple_size.unwrap_or(PxSize::new(
            input.computed_data.width,
            input.computed_data.height,
        ));
        let click_pos = normalized_click_position(input.cursor_position_rel, input.computed_data);

        for event in cursor_events.iter() {
            if within_bounds
                && matches!(
                    event.content,
                    CursorEventContent::Pressed(PressKeyEventType::Left)
                )
            {
                interaction_state.with_mut(|s| {
                    s.start_animation_with_spec(click_pos, size, spec);
                    s.set_pressed(true);
                });
            }

            if matches!(
                event.content,
                CursorEventContent::Released(PressKeyEventType::Left)
            ) {
                interaction_state.with_mut(|s| s.release());
            }

            if within_bounds
                && event.gesture_state == GestureState::TapCandidate
                && matches!(
                    event.content,
                    CursorEventContent::Released(PressKeyEventType::Left)
                )
            {
                on_click();
            }
        }

        if !within_bounds {
            interaction_state.with_mut(|s| {
                s.release();
                s.set_hovered(false);
            });
        }
    }));
}

fn shape_background_command(color: Color, shape: Shape, size: PxSize) -> ShapeCommand {
    match shape.resolve_for_size(size) {
        ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => ShapeCommand::Rect {
            color,
            corner_radii,
            corner_g2,
            shadow: None,
        },
        ResolvedShape::Ellipse => ShapeCommand::Ellipse {
            color,
            shadow: None,
        },
    }
}

fn shape_border_command(color: Color, width: Dp, shape: Shape, size: PxSize) -> ShapeCommand {
    let border_width = width.to_pixels_f32();
    match shape.resolve_for_size(size) {
        ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => ShapeCommand::OutlinedRect {
            color,
            corner_radii,
            corner_g2,
            shadow: None,
            border_width,
        },
        ResolvedShape::Ellipse => ShapeCommand::OutlinedEllipse {
            color,
            shadow: None,
            border_width,
        },
    }
}

#[tessera]
fn modifier_background<F>(color: Color, shape: Shape, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_background expects exactly one child");

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let final_width =
            resolve_dimension(parent_constraint.width, child_measurement.width, "width");
        let final_height =
            resolve_dimension(parent_constraint.height, child_measurement.height, "height");
        let size = PxSize::new(final_width, final_height);

        input
            .metadata_mut()
            .push_draw_command(shape_background_command(color, shape, size));

        input.place_child(child_id, PxPosition::ZERO);

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
}

#[tessera]
fn modifier_border_overlay(width: Dp, color: Color, shape: Shape) {
    measure(Box::new(move |input| {
        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let final_width = resolve_dimension(parent_constraint.width, Px(0), "width");
        let final_height = resolve_dimension(parent_constraint.height, Px(0), "height");
        let size = PxSize::new(final_width, final_height);

        input
            .metadata_mut()
            .push_draw_command(shape_border_command(color, width, shape, size));

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));
}

#[tessera]
fn modifier_border<F>(width: Dp, color: Color, shape: Shape, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let content_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_border expects exactly two children");
        let overlay_id = input
            .children_ids
            .get(1)
            .copied()
            .expect("modifier_border expects exactly two children");

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(content_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&content_id)
            .expect("Child measurement missing");

        let final_width =
            resolve_dimension(parent_constraint.width, child_measurement.width, "width");
        let final_height =
            resolve_dimension(parent_constraint.height, child_measurement.height, "height");

        input.place_child(content_id, PxPosition::ZERO);

        let overlay_constraint = Constraint::new(
            DimensionValue::Fixed(final_width),
            DimensionValue::Fixed(final_height),
        );
        let overlay_measurements =
            input.measure_children(vec![(overlay_id, overlay_constraint)])?;
        overlay_measurements
            .get(&overlay_id)
            .expect("Overlay measurement missing");

        input.place_child(overlay_id, PxPosition::ZERO);

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
    modifier_border_overlay(width, color, shape);
}
