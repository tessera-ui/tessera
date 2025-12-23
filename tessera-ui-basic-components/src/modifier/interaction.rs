//! Interaction modifiers for click, toggle, and selection handling.
//!
//! ## Usage
//!
//! Attach pointer/keyboard handling with accessibility and ripple feedback to
//! subtrees.

use std::{mem, sync::Arc};

use tessera_ui::{
    ComputedData, CursorEventContent, GestureState, PressKeyEventType, PxPosition, PxSize, State,
    accesskit::{self, Action, Toggled},
    tessera,
    winit::window::CursorIcon,
};

use crate::{
    pos_misc::is_position_in_rect,
    ripple_state::{RippleSpec, RippleState},
};

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
pub(crate) fn modifier_clickable<F>(args: ClickableArgs, child: F)
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
pub(crate) fn modifier_block_touch_propagation<F>(child: F)
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

#[tessera]
pub(crate) fn modifier_toggleable<F>(args: ToggleableArgs, child: F)
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
pub(crate) fn modifier_selectable<F>(args: SelectableArgs, child: F)
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
