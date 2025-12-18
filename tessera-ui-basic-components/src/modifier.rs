//! Modifier extensions for basic components.
//!
//! ## Usage
//!
//! Attach layout and drawing behavior like padding and opacity to any subtree.

use std::{mem, sync::Arc};

use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, Modifier, Px,
    GestureState, PressKeyEventType, PxPosition, PxSize, State, accesskit,
    accesskit::{Action, Toggled},
    tessera, use_context,
    winit::window::CursorIcon,
};

use crate::{
    pipelines::shape::command::ShapeCommand,
    pos_misc::is_position_in_rect,
    ripple_state::{RippleSpec, RippleState},
    shape_def::{ResolvedShape, Shape},
    ShadowProps,
};

/// Controls whether minimum interactive size wrappers are enforced.
#[derive(Clone, Copy, Debug)]
pub struct MinimumInteractiveComponentEnforcement {
    /// When true, `minimum_interactive_component_size` expands to the minimum
    /// size.
    pub enabled: bool,
}

impl Default for MinimumInteractiveComponentEnforcement {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Padding values in density-independent pixels.
#[derive(Clone, Copy, Debug, Default)]
pub struct Padding {
    /// Left padding.
    pub left: Dp,
    /// Top padding.
    pub top: Dp,
    /// Right padding.
    pub right: Dp,
    /// Bottom padding.
    pub bottom: Dp,
}

impl Padding {
    /// Creates symmetric padding on all edges.
    pub const fn all(value: Dp) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    /// Creates symmetric padding for horizontal and vertical edges.
    pub const fn symmetric(horizontal: Dp, vertical: Dp) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }

    /// Creates padding with explicit edges.
    pub const fn only(left: Dp, top: Dp, right: Dp, bottom: Dp) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
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

    /// Draws a shadow behind the subtree.
    fn shadow(self, shadow: ShadowProps) -> Modifier;

    /// Draws a shadow behind the subtree using a custom shape.
    fn shadow_with_shape(self, shadow: ShadowProps, shape: Shape) -> Modifier;

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

    /// Makes the subtree clickable with optional ripple feedback and an
    /// accessibility click action.
    fn clickable(
        self,
        on_click: Arc<dyn Fn() + Send + Sync>,
        enabled: bool,
        role: Option<accesskit::Role>,
        label: Option<String>,
        description: Option<String>,
        interaction_state: Option<State<RippleState>>,
        ripple_spec: Option<RippleSpec>,
        ripple_size: Option<PxSize>,
    ) -> Modifier;

    /// Makes the subtree toggleable with optional ripple/state-layer feedback.
    fn toggleable(
        self,
        value: bool,
        on_value_change: Arc<dyn Fn(bool) + Send + Sync>,
        enabled: bool,
        role: Option<accesskit::Role>,
        label: Option<String>,
        description: Option<String>,
        interaction_state: Option<State<RippleState>>,
        ripple_spec: Option<RippleSpec>,
        ripple_size: Option<PxSize>,
    ) -> Modifier;

    /// Makes the subtree selectable with optional ripple/state-layer feedback.
    fn selectable(
        self,
        selected: bool,
        on_click: Arc<dyn Fn() + Send + Sync>,
        enabled: bool,
        role: Option<accesskit::Role>,
        label: Option<String>,
        description: Option<String>,
        interaction_state: Option<State<RippleState>>,
        ripple_spec: Option<RippleSpec>,
        ripple_size: Option<PxSize>,
    ) -> Modifier;
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

    fn shadow(self, shadow: ShadowProps) -> Modifier {
        self.shadow_with_shape(shadow, Shape::RECTANGLE)
    }

    fn shadow_with_shape(self, shadow: ShadowProps, shape: Shape) -> Modifier {
        self.push_wrapper(move |child| {
            move || {
                modifier_shadow(shadow, shape, || {
                    child();
                });
            }
        })
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

    fn clickable(
        self,
        on_click: Arc<dyn Fn() + Send + Sync>,
        enabled: bool,
        role: Option<accesskit::Role>,
        label: Option<String>,
        description: Option<String>,
        interaction_state: Option<State<RippleState>>,
        ripple_spec: Option<RippleSpec>,
        ripple_size: Option<PxSize>,
    ) -> Modifier {
        self.push_wrapper(move |child| {
            let on_click = on_click.clone();
            let label = label.clone();
            let description = description.clone();
            move || {
                modifier_clickable(
                    on_click,
                    enabled,
                    role,
                    label,
                    description,
                    interaction_state,
                    ripple_spec,
                    ripple_size,
                    || {
                        child();
                    },
                );
            }
        })
    }

    fn toggleable(
        self,
        value: bool,
        on_value_change: Arc<dyn Fn(bool) + Send + Sync>,
        enabled: bool,
        role: Option<accesskit::Role>,
        label: Option<String>,
        description: Option<String>,
        interaction_state: Option<State<RippleState>>,
        ripple_spec: Option<RippleSpec>,
        ripple_size: Option<PxSize>,
    ) -> Modifier {
        self.push_wrapper(move |child| {
            let on_value_change = on_value_change.clone();
            let label = label.clone();
            let description = description.clone();
            move || {
                modifier_toggleable(
                    value,
                    on_value_change,
                    enabled,
                    role,
                    label,
                    description,
                    interaction_state,
                    ripple_spec,
                    ripple_size,
                    || {
                        child();
                    },
                );
            }
        })
    }

    fn selectable(
        self,
        selected: bool,
        on_click: Arc<dyn Fn() + Send + Sync>,
        enabled: bool,
        role: Option<accesskit::Role>,
        label: Option<String>,
        description: Option<String>,
        interaction_state: Option<State<RippleState>>,
        ripple_spec: Option<RippleSpec>,
        ripple_size: Option<PxSize>,
    ) -> Modifier {
        self.push_wrapper(move |child| {
            let on_click = on_click.clone();
            let label = label.clone();
            let description = description.clone();
            move || {
                modifier_selectable(
                    selected,
                    on_click,
                    enabled,
                    role,
                    label,
                    description,
                    interaction_state,
                    ripple_spec,
                    ripple_size,
                    || {
                        child();
                    },
                );
            }
        })
    }
}

fn subtract_opt_px(value: Option<Px>, subtract: Px) -> Option<Px> {
    value.map(|v| (v - subtract).max(Px(0)))
}

fn shrink_dimension(dimension: DimensionValue, before: Px, after: Px) -> DimensionValue {
    let subtract = before + after;
    match dimension {
        DimensionValue::Fixed(value) => DimensionValue::Fixed((value - subtract).max(Px(0))),
        DimensionValue::Wrap { min, max } => DimensionValue::Wrap {
            min: subtract_opt_px(min, subtract),
            max: subtract_opt_px(max, subtract),
        },
        DimensionValue::Fill { min, max } => DimensionValue::Fill {
            min: subtract_opt_px(min, subtract),
            max: subtract_opt_px(max, subtract),
        },
    }
}

fn resolve_dimension(dimension: DimensionValue, content: Px, axis: &'static str) -> Px {
    match dimension {
        DimensionValue::Fixed(value) => value,
        DimensionValue::Wrap { min, max } => {
            let mut value = content;
            if let Some(min_value) = min {
                value = value.max(min_value);
            }
            if let Some(max_value) = max {
                value = value.min(max_value);
            }
            value
        }
        DimensionValue::Fill { min, max } => {
            let Some(max_value) = max else {
                panic!(
                    "Seems that you are trying to fill an infinite dimension, which is not allowed\naxis = {axis}\nconstraint = {dimension:?}"
                );
            };
            let mut value = max_value;
            if let Some(min_value) = min {
                value = value.max(min_value);
            }
            value
        }
    }
}

#[tessera]
fn modifier_padding<F>(padding: Padding, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_padding expects exactly one child");

        let left_px: Px = padding.left.into();
        let top_px: Px = padding.top.into();
        let right_px: Px = padding.right.into();
        let bottom_px: Px = padding.bottom.into();

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_constraint = Constraint::new(
            shrink_dimension(parent_constraint.width, left_px, right_px),
            shrink_dimension(parent_constraint.height, top_px, bottom_px),
        );

        let child_measurements = input.measure_children(vec![(child_id, child_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let content_width = child_measurement.width + left_px + right_px;
        let content_height = child_measurement.height + top_px + bottom_px;

        let final_width = resolve_dimension(parent_constraint.width, content_width, "width");
        let final_height = resolve_dimension(parent_constraint.height, content_height, "height");

        input.place_child(child_id, PxPosition::new(left_px, top_px));

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
}

#[tessera]
fn modifier_offset<F>(x: Dp, y: Dp, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_offset expects exactly one child");

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        input.place_child(child_id, PxPosition::new(x.into(), y.into()));

        Ok(child_measurement)
    }));

    child();
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
fn modifier_clickable<F>(
    on_click: Arc<dyn Fn() + Send + Sync>,
    enabled: bool,
    role: Option<accesskit::Role>,
    label: Option<String>,
    description: Option<String>,
    interaction_state: Option<State<RippleState>>,
    ripple_spec: Option<RippleSpec>,
    ripple_size: Option<PxSize>,
    child: F,
) where
    F: FnOnce(),
{
    child();

    let role = role.unwrap_or(accesskit::Role::Button);
    input_handler(Box::new(move |input| {
        let mut cursor_events = Vec::new();
        mem::swap(&mut cursor_events, input.cursor_events);

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
fn modifier_toggleable<F>(
    value: bool,
    on_value_change: Arc<dyn Fn(bool) + Send + Sync>,
    enabled: bool,
    role: Option<accesskit::Role>,
    label: Option<String>,
    description: Option<String>,
    interaction_state: Option<State<RippleState>>,
    ripple_spec: Option<RippleSpec>,
    ripple_size: Option<PxSize>,
    child: F,
) where
    F: FnOnce(),
{
    child();

    let role = role.unwrap_or(accesskit::Role::CheckBox);
    input_handler(Box::new(move |input| {
        let mut cursor_events = Vec::new();
        mem::swap(&mut cursor_events, input.cursor_events);

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
        builder = builder.toggled(if value {
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
        let size = ripple_size.unwrap_or(PxSize::new(input.computed_data.width, input.computed_data.height));
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
fn modifier_selectable<F>(
    selected: bool,
    on_click: Arc<dyn Fn() + Send + Sync>,
    enabled: bool,
    role: Option<accesskit::Role>,
    label: Option<String>,
    description: Option<String>,
    interaction_state: Option<State<RippleState>>,
    ripple_spec: Option<RippleSpec>,
    ripple_size: Option<PxSize>,
    child: F,
) where
    F: FnOnce(),
{
    child();

    let role = role.unwrap_or(accesskit::Role::Button);
    input_handler(Box::new(move |input| {
        let mut cursor_events = Vec::new();
        mem::swap(&mut cursor_events, input.cursor_events);

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
        let size = ripple_size.unwrap_or(PxSize::new(input.computed_data.width, input.computed_data.height));
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

fn shape_shadow_command(shadow: ShadowProps, shape: Shape, size: PxSize) -> ShapeCommand {
    let color = Color::TRANSPARENT;
    match shape.resolve_for_size(size) {
        ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } => ShapeCommand::Rect {
            color,
            corner_radii,
            corner_g2,
            shadow: Some(shadow),
        },
        ResolvedShape::Ellipse => ShapeCommand::Ellipse {
            color,
            shadow: Some(shadow),
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
fn modifier_shadow<F>(shadow: ShadowProps, shape: Shape, child: F)
where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_shadow expects exactly one child");

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
            .push_draw_command(shape_shadow_command(shadow, shape, size));

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

#[tessera]
fn modifier_constraints<F>(
    width_override: Option<DimensionValue>,
    height_override: Option<DimensionValue>,
    child: F,
) where
    F: FnOnce(),
{
    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_constraints expects exactly one child");

        let parent_width = input.parent_constraint.width();
        let parent_height = input.parent_constraint.height();
        let constraint = Constraint::new(
            width_override.unwrap_or(parent_width),
            height_override.unwrap_or(parent_height),
        )
        .merge(input.parent_constraint);

        let child_measurements = input.measure_children(vec![(child_id, constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let final_width = resolve_dimension(constraint.width, child_measurement.width, "width");
        let final_height = resolve_dimension(constraint.height, child_measurement.height, "height");

        input.place_child(child_id, PxPosition::ZERO);

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
}

#[tessera]
fn modifier_minimum_interactive_size<F>(child: F)
where
    F: FnOnce(),
{
    const MIN_SIZE: Dp = Dp(48.0);

    measure(Box::new(move |input| {
        let child_id = input
            .children_ids
            .first()
            .copied()
            .expect("modifier_minimum_interactive_size expects exactly one child");

        let parent_constraint = Constraint::new(
            input.parent_constraint.width(),
            input.parent_constraint.height(),
        );
        let child_measurements = input.measure_children(vec![(child_id, parent_constraint)])?;
        let child_measurement = *child_measurements
            .get(&child_id)
            .expect("Child measurement missing");

        let min_px: Px = MIN_SIZE.into();
        let content_width = child_measurement.width.max(min_px);
        let content_height = child_measurement.height.max(min_px);

        let final_width = resolve_dimension(parent_constraint.width, content_width, "width");
        let final_height = resolve_dimension(parent_constraint.height, content_height, "height");

        let x = ((final_width - child_measurement.width) / 2).max(Px(0));
        let y = ((final_height - child_measurement.height) / 2).max(Px(0));
        input.place_child(child_id, PxPosition::new(x, y));

        Ok(ComputedData {
            width: final_width,
            height: final_height,
        })
    }));

    child();
}
