//! Interaction modifiers for click, toggle, and selection handling.
//!
//! ## Usage
//!
//! Attach pointer and keyboard handling with accessibility actions to subtrees.

use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, BuildModifierNode, Callback, CallbackWith,
    FocusRequester, FocusState, KeyboardInput, KeyboardInputModifierNode, Modifier,
    ModifierBuildContext, PointerChange, PointerDown, PointerInput, PointerInputModifierNode,
    PointerInputScope, PxPosition, SemanticsModifierNode, State,
    accesskit::{self, Action, Toggled},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    winit::window::CursorIcon,
};

use crate::pointer_gestures::{TapGesture, TapGestureConfig, detect_tap_gestures};

use super::{ClickableArgs, InteractionState, PointerEventContext, SelectableArgs, ToggleableArgs};

type PressCallback = CallbackWith<PointerEventContext, ()>;

fn with_focus_state(
    base: Modifier,
    state: Option<State<InteractionState>>,
    focus: State<(bool, u64)>,
    cancel_press_on_blur: bool,
) -> Modifier {
    base.on_focus_changed(move |next: FocusState| {
        let focused = next.has_focus();
        if focus.with(|(current, _)| *current) != focused {
            focus.with_mut(|(current, epoch)| {
                *current = focused;
                if !focused && cancel_press_on_blur {
                    *epoch = epoch.wrapping_add(1);
                }
            });
        }
        if let Some(state) = state {
            state.with_mut(|state| {
                state.set_focused(focused);
                if !focused && cancel_press_on_blur {
                    state.release();
                }
            });
        }
    })
}

fn with_hover_state(
    base: Modifier,
    state: Option<State<InteractionState>>,
    enabled: bool,
) -> Modifier {
    let Some(state) = state else {
        return base;
    };
    base.push_pointer_input(HoverState { state, enabled })
}

struct HoverState {
    state: State<InteractionState>,
    enabled: bool,
}

impl PointerInputModifierNode for HoverState {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        let hovered = self.enabled
            && input.cursor_position_rel.is_some_and(|position| {
                position.x >= 0.0
                    && position.y >= 0.0
                    && position.x < input.computed_data.width.to_f32()
                    && position.y < input.computed_data.height.to_f32()
            });
        if self.state.with(InteractionState::is_hovered) != hovered {
            self.state.with_mut(|state| state.set_hovered(hovered));
        }
    }
}

fn has_keyboard_activation_event(
    keyboard_events: &[tessera_ui::KeyboardEvent],
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

fn async_pointer_context(down: &PointerDown, change: &PointerChange) -> PointerEventContext {
    let size = down.size;
    let position = change
        .position()
        .map(|position| down.local_to_window.inverse_point(position))
        .or(down.local_position)
        .unwrap_or(PxPosition::ZERO);
    PointerEventContext {
        normalized_pos: [
            (position.x / size.width.to_f32().max(1.0)).clamp(0.0, 1.0),
            (position.y / size.height.to_f32().max(1.0)).clamp(0.0, 1.0),
        ],
        size,
        timestamp: change.timestamp,
    }
}

async fn async_clickable_detector(
    mut scope: PointerInputScope,
    mut on_click: impl FnMut(),
    on_long_click: Option<Callback>,
    click_enabled: bool,
    enabled: bool,
    block_input: bool,
    on_press: Option<PressCallback>,
    on_release: Option<PressCallback>,
    interaction_state: Option<State<InteractionState>>,
    focus_requester: FocusRequester,
) {
    if !enabled {
        return;
    }
    detect_tap_gestures(
        &mut scope,
        TapGestureConfig {
            consume_down: block_input,
            consume_up: block_input,
            long_press: on_long_click.map(|_| std::time::Duration::from_millis(500)),
            ..Default::default()
        },
        |gesture| match gesture {
            TapGesture::Pressed(down) => {
                if let Some(state) = interaction_state {
                    focus_requester.request_focus();
                    state.with_mut(|state| state.set_pressed(true));
                }
                if let Some(callback) = on_press {
                    callback.call(async_pointer_context(&down, &down.change));
                }
            }
            TapGesture::Released(down, up) => {
                if let Some(state) = interaction_state {
                    state.with_mut(InteractionState::release);
                }
                if let Some(callback) = on_release {
                    callback.call(async_pointer_context(&down, &up));
                }
            }
            TapGesture::Tapped(_, _) if click_enabled => on_click(),
            TapGesture::LongPressed(_) => {
                if let Some(callback) = on_long_click {
                    callback.call();
                }
            }
            TapGesture::Cancelled => {
                if let Some(state) = interaction_state
                    && state.is_alive()
                {
                    state.with_mut(InteractionState::release);
                }
            }
            _ => {}
        },
    )
    .await;
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

struct ClickableKeyboardModifierNode {
    on_click: Callback,
    on_long_click: Option<Callback>,
    click_enabled: bool,
    enabled: bool,
    focus_requester: FocusRequester,
}

impl KeyboardInputModifierNode for ClickableKeyboardModifierNode {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        if !self.enabled {
            return;
        }
        let context_menu = !input.key_modifiers.control_key() && !input.key_modifiers.alt_key() && !input.key_modifiers.super_key()
            && input.keyboard_events.iter().any(|event| {
                event.state == tessera_ui::winit::event::ElementState::Pressed
                    && matches!(&event.logical_key, tessera_ui::winit::keyboard::Key::Named(key)
                        if *key == tessera_ui::winit::keyboard::NamedKey::ContextMenu
                            || (*key == tessera_ui::winit::keyboard::NamedKey::F10 && input.key_modifiers.shift_key()))
            });
        if context_menu && let Some(callback) = self.on_long_click {
            self.focus_requester.request_focus();
            callback.call();
            input.block_keyboard();
            return;
        }
        if !self.click_enabled
            || !has_keyboard_activation_event(input.keyboard_events, input.key_modifiers)
        {
            return;
        }

        self.focus_requester.request_focus();
        self.on_click.call();
        input.block_keyboard();
    }
}

struct ClickableSemanticsModifierNode {
    on_click: Callback,
    on_long_click: Option<Callback>,
    click_enabled: bool,
    enabled: bool,
    role: Option<accesskit::Role>,
    label: Option<String>,
    description: Option<String>,
    focus_requester: FocusRequester,
}

impl SemanticsModifierNode for ClickableSemanticsModifierNode {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        accessibility.role = Some(self.role.unwrap_or(accesskit::Role::Button));
        accessibility.label = self.label.clone();
        accessibility.description = self.description.clone();
        accessibility.focusable = self.enabled;
        accessibility.disabled = !self.enabled;
        accessibility.actions.clear();
        if self.enabled {
            if self.click_enabled {
                accessibility.actions.push(Action::Click);
            }
            if self.on_long_click.is_some() {
                accessibility.actions.push(Action::ShowContextMenu);
            }
            let on_click = self.on_click;
            let on_long_click = self.on_long_click;
            let click_enabled = self.click_enabled;
            let focus_requester = self.focus_requester;
            *action_handler = Some(Box::new(move |action| {
                if action == Action::Click && click_enabled {
                    focus_requester.request_focus();
                    on_click.call();
                } else if action == Action::ShowContextMenu
                    && let Some(callback) = on_long_click
                {
                    focus_requester.request_focus();
                    callback.call();
                }
            }));
        } else {
            *action_handler = None;
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Activation {
    Toggle {
        value: bool,
        on_value_change: CallbackWith<bool, ()>,
    },
    Click(Callback),
}

impl Activation {
    fn activate(self) {
        match self {
            Self::Toggle {
                value,
                on_value_change,
            } => on_value_change.call(!value),
            Self::Click(on_click) => on_click.call(),
        }
    }
}

struct ActivationKeyboardModifierNode {
    activation: Activation,
    enabled: bool,
    focus_requester: FocusRequester,
}

impl KeyboardInputModifierNode for ActivationKeyboardModifierNode {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        if !self.enabled
            || !has_keyboard_activation_event(input.keyboard_events, input.key_modifiers)
        {
            return;
        }

        self.focus_requester.request_focus();
        self.activation.activate();
        input.block_keyboard();
    }
}

struct ToggleableSemanticsModifierNode {
    value: bool,
    on_value_change: CallbackWith<bool, ()>,
    enabled: bool,
    role: Option<accesskit::Role>,
    label: Option<String>,
    description: Option<String>,
    focus_requester: FocusRequester,
}

impl SemanticsModifierNode for ToggleableSemanticsModifierNode {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        accessibility.role = Some(self.role.unwrap_or(accesskit::Role::CheckBox));
        accessibility.label = self.label.clone();
        accessibility.description = self.description.clone();
        accessibility.focusable = self.enabled;
        accessibility.toggled = Some(if self.value {
            Toggled::True
        } else {
            Toggled::False
        });
        accessibility.disabled = !self.enabled;
        accessibility.actions.clear();
        if self.enabled {
            accessibility.actions.push(Action::Click);
            let on_value_change = self.on_value_change;
            let value = self.value;
            let focus_requester = self.focus_requester;
            *action_handler = Some(Box::new(move |action| {
                if action == Action::Click {
                    focus_requester.request_focus();
                    on_value_change.call(!value);
                }
            }));
        } else {
            *action_handler = None;
        }
    }
}

struct SelectableSemanticsModifierNode {
    selected: bool,
    on_click: Callback,
    enabled: bool,
    role: Option<accesskit::Role>,
    label: Option<String>,
    description: Option<String>,
    focus_requester: FocusRequester,
}

impl SemanticsModifierNode for SelectableSemanticsModifierNode {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        let role = self.role.unwrap_or(accesskit::Role::Button);
        accessibility.role = Some(role);
        accessibility.label = self.label.clone();
        accessibility.description = self.description.clone();
        accessibility.focusable = self.enabled;
        if role == accesskit::Role::RadioButton {
            accessibility.toggled = Some(if self.selected {
                Toggled::True
            } else {
                Toggled::False
            });
            accessibility.selected = None;
        } else {
            accessibility.selected = Some(self.selected);
            accessibility.toggled = None;
        }
        accessibility.disabled = !self.enabled;
        accessibility.actions.clear();
        if self.enabled {
            accessibility.actions.push(Action::Click);
            let on_click = self.on_click;
            let focus_requester = self.focus_requester;
            *action_handler = Some(Box::new(move |action| {
                if action == Action::Click {
                    focus_requester.request_focus();
                    on_click.call();
                }
            }));
        } else {
            *action_handler = None;
        }
    }
}

fn apply_combined_clickable_resolved(
    base: Modifier,
    args: ClickableArgs,
    on_long_click: Option<Callback>,
    click_enabled: bool,
    focus: State<(bool, u64)>,
) -> Modifier {
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
        focus_requester,
        focus_properties,
    } = args;
    let focus_requester = focus_requester.unwrap_or_else(FocusRequester::new);
    let participates_in_focus =
        enabled && (role.is_some() || label.is_some() || description.is_some());

    let mut modifier = base;
    if participates_in_focus {
        modifier = modifier.focus_requester(focus_requester).focusable();
        if let Some(focus_properties) = focus_properties {
            modifier = modifier.focus_properties(focus_properties);
        }
    } else if !enabled && let Some(interaction_state) = interaction_state {
        reset_disabled_interaction_state(interaction_state);
    }
    if enabled {
        modifier = modifier.hover_cursor_icon(CursorIcon::Pointer);
    }
    modifier = with_focus_state(modifier, interaction_state, focus, on_long_click.is_some());

    with_hover_state(modifier, interaction_state, enabled)
        .push_semantics(ClickableSemanticsModifierNode {
            on_long_click,
            click_enabled,
            on_click,
            enabled,
            role,
            label: label.clone(),
            description: description.clone(),
            focus_requester,
        })
        .push_keyboard_input(ClickableKeyboardModifierNode {
            on_long_click,
            click_enabled,
            on_click,
            enabled,
            focus_requester,
        })
        .pointer_input(
            (
                enabled,
                click_enabled,
                block_input,
                on_click,
                on_long_click,
                on_press,
                on_release,
                interaction_state,
                focus.with(|(_, epoch)| *epoch),
            ),
            move |scope| {
                async_clickable_detector(
                    scope,
                    move || on_click.call(),
                    on_long_click,
                    click_enabled,
                    enabled,
                    block_input,
                    on_press,
                    on_release,
                    interaction_state,
                    focus_requester,
                )
            },
        )
}

fn apply_toggleable_resolved(
    base: Modifier,
    args: ToggleableArgs,
    focus: State<(bool, u64)>,
) -> Modifier {
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
        focus_requester,
    } = args;
    let activation = Activation::Toggle {
        value,
        on_value_change,
    };
    let focus_requester = focus_requester.unwrap_or_else(FocusRequester::new);

    let mut modifier = base;
    if enabled {
        modifier = modifier
            .focus_requester(focus_requester)
            .focusable()
            .hover_cursor_icon(CursorIcon::Pointer);
    } else if let Some(interaction_state) = interaction_state {
        reset_disabled_interaction_state(interaction_state);
    }
    modifier = with_focus_state(modifier, interaction_state, focus, false);

    with_hover_state(modifier, interaction_state, enabled)
        .push_semantics(ToggleableSemanticsModifierNode {
            value,
            on_value_change,
            enabled,
            role,
            label: label.clone(),
            description: description.clone(),
            focus_requester,
        })
        .push_keyboard_input(ActivationKeyboardModifierNode {
            activation,
            enabled,
            focus_requester,
        })
        .pointer_input(
            (
                activation,
                enabled,
                on_press,
                on_release,
                interaction_state,
                focus.with(|(_, epoch)| *epoch),
            ),
            move |scope| {
                async_clickable_detector(
                    scope,
                    move || activation.activate(),
                    None,
                    true,
                    enabled,
                    false,
                    on_press,
                    on_release,
                    interaction_state,
                    focus_requester,
                )
            },
        )
}

fn apply_selectable_resolved(
    base: Modifier,
    args: SelectableArgs,
    focus: State<(bool, u64)>,
) -> Modifier {
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
        focus_requester,
    } = args;
    let activation = Activation::Click(on_click);
    let focus_requester = focus_requester.unwrap_or_else(FocusRequester::new);

    let mut modifier = base;
    if enabled {
        modifier = modifier
            .focus_requester(focus_requester)
            .focusable()
            .hover_cursor_icon(CursorIcon::Pointer);
    } else if let Some(interaction_state) = interaction_state {
        reset_disabled_interaction_state(interaction_state);
    }
    modifier = with_focus_state(modifier, interaction_state, focus, false);

    with_hover_state(modifier, interaction_state, enabled)
        .push_semantics(SelectableSemanticsModifierNode {
            selected,
            on_click,
            enabled,
            role,
            label: label.clone(),
            description: description.clone(),
            focus_requester,
        })
        .push_keyboard_input(ActivationKeyboardModifierNode {
            activation,
            enabled,
            focus_requester,
        })
        .pointer_input(
            (
                activation,
                enabled,
                on_press,
                on_release,
                interaction_state,
                focus.with(|(_, epoch)| *epoch),
            ),
            move |scope| {
                async_clickable_detector(
                    scope,
                    move || activation.activate(),
                    None,
                    true,
                    enabled,
                    false,
                    on_press,
                    on_release,
                    interaction_state,
                    focus_requester,
                )
            },
        )
}

pub(super) struct ClickableBuild {
    pub args: ClickableArgs,
    pub on_long_click: Option<Callback>,
    pub click_enabled: bool,
}

impl BuildModifierNode for ClickableBuild {
    fn materialize(&self, context: &mut ModifierBuildContext) -> Option<Modifier> {
        let mut args = self.args.clone();
        args.focus_requester = Some(
            args.focus_requester
                .unwrap_or_else(|| context.state(FocusRequester::new).get()),
        );
        Some(apply_combined_clickable_resolved(
            Modifier::new(),
            args,
            self.on_long_click,
            self.click_enabled,
            context.state(|| (false, 0)),
        ))
    }
}

pub(super) struct ToggleableBuild(pub ToggleableArgs);

impl BuildModifierNode for ToggleableBuild {
    fn materialize(&self, context: &mut ModifierBuildContext) -> Option<Modifier> {
        let mut args = self.0.clone();
        args.focus_requester = Some(
            args.focus_requester
                .unwrap_or_else(|| context.state(FocusRequester::new).get()),
        );
        Some(apply_toggleable_resolved(
            Modifier::new(),
            args,
            context.state(|| (false, 0)),
        ))
    }
}

pub(super) struct SelectableBuild(pub SelectableArgs);

impl BuildModifierNode for SelectableBuild {
    fn materialize(&self, context: &mut ModifierBuildContext) -> Option<Modifier> {
        let mut args = self.0.clone();
        args.focus_requester = Some(
            args.focus_requester
                .unwrap_or_else(|| context.state(FocusRequester::new).get()),
        );
        Some(apply_selectable_resolved(
            Modifier::new(),
            args,
            context.state(|| (false, 0)),
        ))
    }
}

#[cfg(test)]
#[path = "activation_tests.rs"]
mod tests;
