//! Interaction modifiers for click, toggle, and selection handling.
//!
//! ## Usage
//!
//! Attach pointer/keyboard handling with accessibility and ripple feedback to
//! subtrees.

use tessera_ui::{
    Callback, CallbackWith, ComputedData, FocusProperties, FocusRequester, FocusState, Modifier,
    Prop, PxPosition, PxSize, RenderSlot, State, WindowAction,
    accesskit::{self, Action, Toggled},
    modifier::FocusModifierExt as _,
    remember, tessera,
    winit::window::CursorIcon,
};

use crate::{
    gesture_recognizer::{LongPressRecognizer, TapRecognizer},
    pos_misc::is_position_in_rect,
    theme::MaterialAlpha,
};

/// Context for pointer press/release callbacks.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct PointerEventContext {
    /// Pointer position normalized to `[0.0, 1.0]` within the element bounds.
    pub normalized_pos: [f32; 2],
    /// The element size in pixels.
    pub size: PxSize,
}

type PressCallback = CallbackWith<PointerEventContext, ()>;

/// Arguments for the `clickable` modifier.
#[derive(Clone, Prop)]
#[prop(skip_setter)]
pub struct ClickableArgs {
    /// Callback invoked when the element is clicked.
    pub on_click: Callback,
    /// Whether the element is enabled for interaction.
    pub enabled: bool,
    /// Whether to block input propagation when within bounds. Defaults to true
    /// to match Compose's behavior of consuming click gestures.
    pub block_input: bool,
    /// Optional press callback with normalized position and element size.
    pub on_press: Option<PressCallback>,
    /// Optional release callback with normalized position and element size.
    pub on_release: Option<PressCallback>,
    /// Optional accessibility role (defaults to `Button`).
    pub role: Option<accesskit::Role>,
    /// Optional accessibility label.
    pub label: Option<String>,
    /// Optional accessibility description.
    pub description: Option<String>,
    /// Optional external interaction state (hover/pressed/focus).
    pub interaction_state: Option<State<InteractionState>>,
    /// Optional externally managed focus requester for this clickable target.
    pub focus_requester: Option<FocusRequester>,
    /// Optional focus properties applied to the clickable target.
    pub focus_properties: Option<FocusProperties>,
}

impl ClickableArgs {
    /// Create a new `ClickableArgs` with the required `on_click` handler.
    pub fn new(on_click: impl Into<Callback>) -> Self {
        Self {
            on_click: on_click.into(),
            enabled: true,
            block_input: true,
            on_press: None,
            on_release: None,
            role: None,
            label: None,
            description: None,
            interaction_state: None,
            focus_requester: None,
            focus_properties: None,
        }
    }

    /// Set whether the control is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set whether to block input propagation when hovered.
    pub fn block_input(mut self, block_input: bool) -> Self {
        self.block_input = block_input;
        self
    }

    /// Set a press callback.
    pub fn on_press(mut self, on_press: impl Into<PressCallback>) -> Self {
        self.on_press = Some(on_press.into());
        self
    }

    /// Set a release callback.
    pub fn on_release(mut self, on_release: impl Into<PressCallback>) -> Self {
        self.on_release = Some(on_release.into());
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

    /// Attach an external interaction `State`.
    pub fn interaction_state(mut self, state: State<InteractionState>) -> Self {
        self.interaction_state = Some(state);
        self
    }

    /// Attach an external focus requester.
    pub fn focus_requester(mut self, requester: FocusRequester) -> Self {
        self.focus_requester = Some(requester);
        self
    }

    /// Attach explicit focus properties.
    pub fn focus_properties(mut self, properties: FocusProperties) -> Self {
        self.focus_properties = Some(properties);
        self
    }
}

/// Arguments for the `toggleable` modifier.
#[derive(Clone, Prop)]
#[prop(skip_setter)]
pub struct ToggleableArgs {
    /// Current boolean value.
    pub value: bool,
    /// Callback invoked with the new value when changed.
    pub on_value_change: CallbackWith<bool, ()>,
    /// Whether the control is enabled for interaction.
    pub enabled: bool,
    /// Optional accessibility role (defaults to `CheckBox` or similar).
    pub role: Option<accesskit::Role>,
    /// Optional accessibility label.
    pub label: Option<String>,
    /// Optional accessibility description.
    pub description: Option<String>,
    /// Optional external interaction state (hover/press/focus).
    pub interaction_state: Option<State<InteractionState>>,
    /// Optional press callback with normalized position and element size.
    pub on_press: Option<PressCallback>,
    /// Optional release callback with normalized position and element size.
    pub on_release: Option<PressCallback>,
    /// Optional externally managed focus requester for this toggleable target.
    pub focus_requester: Option<FocusRequester>,
}

impl ToggleableArgs {
    /// Create a new `ToggleableArgs` with the required `value` and
    /// `on_value_change`.
    pub fn new(value: bool, on_value_change: impl Into<CallbackWith<bool, ()>>) -> Self {
        Self {
            value,
            on_value_change: on_value_change.into(),
            enabled: true,
            role: None,
            label: None,
            description: None,
            interaction_state: None,
            on_press: None,
            on_release: None,
            focus_requester: None,
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

    /// Attach an external interaction `State`.
    pub fn interaction_state(mut self, state: State<InteractionState>) -> Self {
        self.interaction_state = Some(state);
        self
    }

    /// Set a press callback.
    pub fn on_press(mut self, on_press: impl Into<PressCallback>) -> Self {
        self.on_press = Some(on_press.into());
        self
    }

    /// Set a release callback.
    pub fn on_release(mut self, on_release: impl Into<PressCallback>) -> Self {
        self.on_release = Some(on_release.into());
        self
    }

    /// Attach an external focus requester.
    pub fn focus_requester(mut self, requester: FocusRequester) -> Self {
        self.focus_requester = Some(requester);
        self
    }
}

/// Arguments for the `selectable` modifier.
#[derive(Clone, Prop)]
#[prop(skip_setter)]
pub struct SelectableArgs {
    /// Whether the item is selected.
    pub selected: bool,
    /// Callback invoked when the selectable is activated.
    pub on_click: Callback,
    /// Whether the element is enabled for interaction.
    pub enabled: bool,
    /// Optional accessibility role (defaults to `Button`).
    pub role: Option<accesskit::Role>,
    /// Optional accessibility label.
    pub label: Option<String>,
    /// Optional accessibility description.
    pub description: Option<String>,
    /// Optional external interaction state (hover/press/focus).
    pub interaction_state: Option<State<InteractionState>>,
    /// Optional press callback with normalized position and element size.
    pub on_press: Option<PressCallback>,
    /// Optional release callback with normalized position and element size.
    pub on_release: Option<PressCallback>,
    /// Optional externally managed focus requester for this selectable target.
    pub focus_requester: Option<FocusRequester>,
}

impl SelectableArgs {
    /// Create a new `SelectableArgs` with the required `selected` and
    /// `on_click`.
    pub fn new(selected: bool, on_click: impl Into<Callback>) -> Self {
        Self {
            selected,
            on_click: on_click.into(),
            enabled: true,
            role: None,
            label: None,
            description: None,
            interaction_state: None,
            on_press: None,
            on_release: None,
            focus_requester: None,
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

    /// Attach an external interaction `State`.
    pub fn interaction_state(mut self, state: State<InteractionState>) -> Self {
        self.interaction_state = Some(state);
        self
    }

    /// Set a press callback.
    pub fn on_press(mut self, on_press: impl Into<PressCallback>) -> Self {
        self.on_press = Some(on_press.into());
        self
    }

    /// Set a release callback.
    pub fn on_release(mut self, on_release: impl Into<PressCallback>) -> Self {
        self.on_release = Some(on_release.into());
        self
    }

    /// Attach an external focus requester.
    pub fn focus_requester(mut self, requester: FocusRequester) -> Self {
        self.focus_requester = Some(requester);
        self
    }
}

fn pointer_context(position: Option<PxPosition>, size: ComputedData) -> PointerEventContext {
    let Some(position) = position else {
        return PointerEventContext {
            normalized_pos: [0.5, 0.5],
            size: PxSize::new(size.width, size.height),
        };
    };
    let width = size.width.to_f32().max(1.0);
    let height = size.height.to_f32().max(1.0);
    let x = (position.x.to_f32() / width).clamp(0.0, 1.0);
    let y = (position.y.to_f32() / height).clamp(0.0, 1.0);
    PointerEventContext {
        normalized_pos: [x, y],
        size: PxSize::new(size.width, size.height),
    }
}

fn has_keyboard_activation_event(
    keyboard_events: &[tessera_ui::winit::event::KeyEvent],
    modifiers: tessera_ui::winit::keyboard::ModifiersState,
) -> bool {
    if modifiers.control_key() || modifiers.alt_key() || modifiers.super_key() {
        return false;
    }

    keyboard_events.iter().any(|event| {
        event.state == tessera_ui::winit::event::ElementState::Pressed
            && matches!(
                &event.logical_key,
                tessera_ui::winit::keyboard::Key::Named(
                    tessera_ui::winit::keyboard::NamedKey::Enter
                        | tessera_ui::winit::keyboard::NamedKey::Space
                )
            )
    })
}

fn reset_disabled_interaction_state(state: State<InteractionState>) {
    let should_reset = state.with(|interaction| {
        interaction.is_hovered()
            || interaction.is_pressed()
            || interaction.is_dragged()
            || interaction.is_focused()
    });
    if should_reset {
        state.with_mut(|interaction| {
            interaction.release();
            interaction.set_hovered(false);
            interaction.set_dragged(false);
            interaction.set_focused(false);
        });
    }
}

#[derive(Clone, Prop)]
struct ModifierClickableArgs {
    clickable: ClickableArgs,
    focus_requester: Option<FocusRequester>,
    child: RenderSlot,
}

pub(crate) fn modifier_clickable(args: ClickableArgs, child: RenderSlot) {
    let render_args = ModifierClickableArgs {
        clickable: args,
        focus_requester: None,
        child,
    };
    modifier_clickable_node(&render_args);
}

#[tessera]
fn modifier_clickable_node(args: &ModifierClickableArgs) {
    let ClickableArgs {
        on_click,
        enabled,
        block_input,
        on_press,
        on_release,
        role,
        label,
        description,
        interaction_state,
        focus_requester: provided_focus_requester,
        focus_properties: provided_focus_properties,
    } = args.clickable.clone();
    let tap_recognizer = remember(TapRecognizer::default);
    let long_press_recognizer = remember(LongPressRecognizer::default);
    let focus_requester = provided_focus_requester
        .unwrap_or_else(|| args.focus_requester.unwrap_or_else(focus_requester));
    let participates_in_focus =
        enabled && (role.is_some() || label.is_some() || description.is_some());
    let mut modifier = Modifier::new();

    if participates_in_focus {
        modifier = modifier.focus_requester(focus_requester).focusable();
        if let Some(properties) = provided_focus_properties {
            modifier = modifier.focus_properties(properties);
        }
    } else if !enabled && let Some(interaction_state) = interaction_state {
        reset_disabled_interaction_state(interaction_state);
    }
    if let Some(interaction_state) = interaction_state {
        modifier = modifier.on_focus_changed(move |focus_state: FocusState| {
            interaction_state.with_mut(|state| state.set_focused(focus_state.has_focus()));
        });
    }

    let child = args.child.clone();
    modifier.run(move || child.render());

    {
        let on_click = on_click.clone();
        keyboard_input_handler(move |mut input| {
            if !enabled
                || !has_keyboard_activation_event(input.keyboard_events, input.key_modifiers)
            {
                return;
            }

            focus_requester.request_focus();
            on_click.call();
            input.block_keyboard();
        });
    }

    let role = role.unwrap_or(accesskit::Role::Button);
    pointer_input_handler(move |mut input| {
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

        let tap_result = tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });
        let long_press_result = long_press_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });
        let tapped = tap_result.tapped && !long_press_result.triggered;

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
            let focus_requester = focus_requester;
            input.set_accessibility_action_handler(move |action| {
                if action == Action::Click {
                    focus_requester.request_focus();
                    on_click_action.call();
                }
            });
        }

        let Some(interaction_state) = interaction_state else {
            if !enabled {
                return;
            }

            if tapped {
                on_click.call();
            }
            if block_input && within_bounds {
                input.block_all();
            }
            return;
        };

        if enabled {
            let hover_changed = interaction_state.with(|s| s.is_hovered() != within_bounds);
            if hover_changed {
                interaction_state.with_mut(|s| s.set_hovered(within_bounds));
            }
        } else {
            let should_reset = interaction_state.with(|s| s.is_pressed() || s.is_hovered());
            if should_reset {
                interaction_state.with_mut(|s| {
                    s.release();
                    s.set_hovered(false);
                });
            }
            return;
        }

        let context = pointer_context(input.cursor_position_rel, input.computed_data);

        if tap_result.pressed {
            focus_requester.request_focus();
            if let Some(on_press) = on_press.as_ref() {
                on_press.call(context);
            }
            let press_changed = interaction_state.with(|s| !s.is_pressed());
            if press_changed {
                interaction_state.with_mut(|s| s.set_pressed(true));
            }
        }

        if tap_result.released {
            let was_pressed = interaction_state.with(|s| s.is_pressed());
            if was_pressed {
                interaction_state.with_mut(|s| s.release());
            }
            if let Some(on_release) = on_release.as_ref() {
                on_release.call(context);
            }
        }

        if tapped {
            on_click.call();
        }

        if !within_bounds {
            let should_reset = interaction_state.with(|s| s.is_pressed() || s.is_hovered());
            if should_reset {
                interaction_state.with_mut(|s| {
                    s.release();
                    s.set_hovered(false);
                });
            }
        }

        if block_input && within_bounds {
            input.block_all();
        }
    });
}

#[derive(Clone, Prop)]
struct ModifierWindowDragRegionArgs {
    child: RenderSlot,
}

pub(crate) fn modifier_window_drag_region(child: RenderSlot) {
    let args = ModifierWindowDragRegionArgs { child };
    modifier_window_drag_region_node(&args);
}

#[tessera]
fn modifier_window_drag_region_node(args: &ModifierWindowDragRegionArgs) {
    let tap_recognizer = remember(TapRecognizer::default);
    args.child.render();

    pointer_input_handler(move |mut input| {
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

        if !within_bounds {
            return;
        }

        let tap_result = tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });

        if tap_result.pressed {
            input.request_window_action(WindowAction::DragWindow);
        }
        input.block_all();
    });
}

#[derive(Clone, Prop)]
struct ModifierWindowActionArgs {
    action: WindowAction,
    child: RenderSlot,
}

pub(crate) fn modifier_window_action(action: WindowAction, child: RenderSlot) {
    let args = ModifierWindowActionArgs { action, child };
    modifier_window_action_node(&args);
}

#[tessera]
fn modifier_window_action_node(args: &ModifierWindowActionArgs) {
    let action = args.action;
    let tap_recognizer = remember(TapRecognizer::default);
    args.child.render();

    pointer_input_handler(move |mut input| {
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
            input.requests.cursor_icon = CursorIcon::Pointer;
        }

        let is_drag_action = matches!(action, WindowAction::DragWindow);
        let tap_result = tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });
        let requested = if is_drag_action {
            tap_result.pressed
        } else {
            tap_result.tapped
        };

        if requested && within_bounds {
            input.request_window_action(action);
        }

        if requested || within_bounds {
            input.block_all();
        }
    });
}

#[derive(Clone, Prop)]
struct ModifierBlockTouchPropagationArgs {
    child: RenderSlot,
}

pub(crate) fn modifier_block_touch_propagation(child: RenderSlot) {
    let args = ModifierBlockTouchPropagationArgs { child };
    modifier_block_touch_propagation_node(&args);
}

#[tessera]
fn modifier_block_touch_propagation_node(args: &ModifierBlockTouchPropagationArgs) {
    args.child.render();

    // Block after descendants so overlay/content wrappers do not swallow child
    // interactions.
    pointer_input_handler(move |mut input| {
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
    });
}

#[derive(Clone, Prop)]
struct ModifierToggleableArgs {
    toggleable: ToggleableArgs,
    focus_requester: Option<FocusRequester>,
    child: RenderSlot,
}

pub(crate) fn modifier_toggleable(args: ToggleableArgs, child: RenderSlot) {
    let render_args = ModifierToggleableArgs {
        toggleable: args,
        focus_requester: None,
        child,
    };
    modifier_toggleable_node(&render_args);
}

#[tessera]
fn modifier_toggleable_node(args: &ModifierToggleableArgs) {
    let ToggleableArgs {
        value,
        on_value_change,
        enabled,
        role,
        label,
        description,
        interaction_state,
        on_press,
        on_release,
        focus_requester: provided_focus_requester,
    } = args.toggleable.clone();
    let tap_recognizer = remember(TapRecognizer::default);
    let focus_requester = provided_focus_requester
        .unwrap_or_else(|| args.focus_requester.unwrap_or_else(focus_requester));
    let mut modifier = Modifier::new();

    if enabled {
        modifier = modifier.focus_requester(focus_requester).focusable();
    } else if let Some(interaction_state) = interaction_state {
        reset_disabled_interaction_state(interaction_state);
    }
    if let Some(interaction_state) = interaction_state {
        modifier = modifier.on_focus_changed(move |focus_state: FocusState| {
            interaction_state.with_mut(|state| state.set_focused(focus_state.has_focus()));
        });
    }

    let child = args.child.clone();
    modifier.run(move || child.render());

    {
        let on_value_change = on_value_change.clone();
        keyboard_input_handler(move |mut input| {
            if !enabled
                || !has_keyboard_activation_event(input.keyboard_events, input.key_modifiers)
            {
                return;
            }

            focus_requester.request_focus();
            on_value_change.call(!value);
            input.block_keyboard();
        });
    }

    let role = role.unwrap_or(accesskit::Role::CheckBox);
    pointer_input_handler(move |input| {
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

        let tap_result = tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });

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
            let focus_requester = focus_requester;
            input.set_accessibility_action_handler(move |action| {
                if action == Action::Click {
                    focus_requester.request_focus();
                    on_value_change.call(!value);
                }
            });
        }

        let Some(interaction_state) = interaction_state else {
            return;
        };

        if enabled {
            let hover_changed = interaction_state.with(|s| s.is_hovered() != within_bounds);
            if hover_changed {
                interaction_state.with_mut(|s| s.set_hovered(within_bounds));
            }
        } else {
            let should_reset =
                interaction_state.with(|s| s.is_pressed() || s.is_hovered() || s.is_focused());
            if should_reset {
                interaction_state.with_mut(|s| {
                    s.release();
                    s.set_hovered(false);
                    s.set_focused(false);
                });
            }
            return;
        }

        let context = pointer_context(input.cursor_position_rel, input.computed_data);

        if tap_result.pressed {
            focus_requester.request_focus();
            if let Some(on_press) = on_press.as_ref() {
                on_press.call(context);
            }
            let press_changed = interaction_state.with(|s| !s.is_pressed());
            if press_changed {
                interaction_state.with_mut(|s| s.set_pressed(true));
            }
        }

        if tap_result.released {
            let was_pressed = interaction_state.with(|s| s.is_pressed());
            if was_pressed {
                interaction_state.with_mut(|s| s.release());
            }
            if let Some(on_release) = on_release.as_ref() {
                on_release.call(context);
            }
        }

        if tap_result.tapped {
            on_value_change.call(!value);
        }

        if !within_bounds {
            let should_reset = interaction_state.with(|s| s.is_pressed() || s.is_hovered());
            if should_reset {
                interaction_state.with_mut(|s| {
                    s.release();
                    s.set_hovered(false);
                });
            }
        }
    });
}

#[derive(Clone, Prop)]
struct ModifierSelectableArgs {
    selectable: SelectableArgs,
    focus_requester: Option<FocusRequester>,
    child: RenderSlot,
}

pub(crate) fn modifier_selectable(args: SelectableArgs, child: RenderSlot) {
    let render_args = ModifierSelectableArgs {
        selectable: args,
        focus_requester: None,
        child,
    };
    modifier_selectable_node(&render_args);
}

#[tessera]
fn modifier_selectable_node(args: &ModifierSelectableArgs) {
    let SelectableArgs {
        selected,
        on_click,
        enabled,
        role,
        label,
        description,
        interaction_state,
        on_press,
        on_release,
        focus_requester: provided_focus_requester,
    } = args.selectable.clone();
    let tap_recognizer = remember(TapRecognizer::default);
    let focus_requester = provided_focus_requester
        .unwrap_or_else(|| args.focus_requester.unwrap_or_else(focus_requester));
    let mut modifier = Modifier::new();

    if enabled {
        modifier = modifier.focus_requester(focus_requester).focusable();
    } else if let Some(interaction_state) = interaction_state {
        reset_disabled_interaction_state(interaction_state);
    }
    if let Some(interaction_state) = interaction_state {
        modifier = modifier.on_focus_changed(move |focus_state: FocusState| {
            interaction_state.with_mut(|state| state.set_focused(focus_state.has_focus()));
        });
    }

    let child = args.child.clone();
    modifier.run(move || child.render());

    {
        let on_click = on_click.clone();
        keyboard_input_handler(move |mut input| {
            if !enabled
                || !has_keyboard_activation_event(input.keyboard_events, input.key_modifiers)
            {
                return;
            }

            focus_requester.request_focus();
            on_click.call();
            input.block_keyboard();
        });
    }

    let role = role.unwrap_or(accesskit::Role::Button);
    pointer_input_handler(move |input| {
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

        let tap_result = tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });

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
            let focus_requester = focus_requester;
            input.set_accessibility_action_handler(move |action| {
                if action == Action::Click {
                    focus_requester.request_focus();
                    on_click.call();
                }
            });
        }

        let Some(interaction_state) = interaction_state else {
            return;
        };

        if enabled {
            let hover_changed = interaction_state.with(|s| s.is_hovered() != within_bounds);
            if hover_changed {
                interaction_state.with_mut(|s| s.set_hovered(within_bounds));
            }
        } else {
            let should_reset =
                interaction_state.with(|s| s.is_pressed() || s.is_hovered() || s.is_focused());
            if should_reset {
                interaction_state.with_mut(|s| {
                    s.release();
                    s.set_hovered(false);
                    s.set_focused(false);
                });
            }
            return;
        }

        let context = pointer_context(input.cursor_position_rel, input.computed_data);

        if tap_result.pressed {
            focus_requester.request_focus();
            if let Some(on_press) = on_press.as_ref() {
                on_press.call(context);
            }
            let press_changed = interaction_state.with(|s| !s.is_pressed());
            if press_changed {
                interaction_state.with_mut(|s| s.set_pressed(true));
            }
        }

        if tap_result.released {
            let was_pressed = interaction_state.with(|s| s.is_pressed());
            if was_pressed {
                interaction_state.with_mut(|s| s.release());
            }
            if let Some(on_release) = on_release.as_ref() {
                on_release.call(context);
            }
        }

        if tap_result.tapped {
            on_click.call();
        }

        if !within_bounds {
            let should_reset = interaction_state.with(|s| s.is_pressed() || s.is_hovered());
            if should_reset {
                interaction_state.with_mut(|s| {
                    s.release();
                    s.set_hovered(false);
                });
            }
        }
    });
}

/// Tracks basic interaction flags and derives state-layer alpha.
#[derive(Clone, PartialEq, Copy, Debug, Default)]
pub struct InteractionState {
    is_hovered: bool,
    is_focused: bool,
    is_dragged: bool,
    is_pressed: bool,
}

impl InteractionState {
    /// Creates a new interaction state with all flags cleared.
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks the component as no longer pressed.
    pub fn release(&mut self) {
        self.set_pressed(false);
    }

    /// Sets whether the component is hovered.
    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    /// Returns whether the component is hovered.
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    /// Sets whether the component is focused.
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Returns whether the component is focused.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Sets whether the component is dragged.
    pub fn set_dragged(&mut self, dragged: bool) {
        self.is_dragged = dragged;
    }

    /// Returns whether the component is dragged.
    pub fn is_dragged(&self) -> bool {
        self.is_dragged
    }

    /// Sets whether the component is pressed.
    pub fn set_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
    }

    /// Returns whether the component is pressed.
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns the state-layer alpha derived from the current interactions.
    pub fn state_layer_alpha(&self) -> f32 {
        if self.is_dragged {
            MaterialAlpha::DRAGGED
        } else if self.is_pressed {
            MaterialAlpha::PRESSED
        } else if self.is_focused {
            MaterialAlpha::FOCUSED
        } else if self.is_hovered {
            MaterialAlpha::HOVER
        } else {
            0.0
        }
    }
}
