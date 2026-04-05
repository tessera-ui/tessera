//! Interaction modifiers for click, toggle, and selection handling.
//!
//! ## Usage
//!
//! Attach pointer/keyboard handling with accessibility and ripple feedback to
//! subtrees.
//!
//! Pointer and keyboard handlers in this module focus on interaction state and
//! gesture flow. Accessibility is attached through semantics modifier nodes,
//! hover cursors use dedicated cursor modifiers, and window actions use
//! explicit window-action helpers.

use tessera_foundation::{
    gesture::{LongPressRecognizer, TapRecognizer},
    modifier::{
        ClickableArgs, InteractionState, PointerEventContext, SelectableArgs, ToggleableArgs,
    },
};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, CallbackWith, ComputedData,
    FocusRequester, FocusState, KeyboardInput, KeyboardInputModifierNode, Modifier, PointerInput,
    PointerInputModifierNode, PxPosition, PxSize, SemanticsModifierNode, State, WindowAction,
    accesskit::{self, Action, Toggled},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    remember,
    winit::window::CursorIcon,
};

use crate::pos_misc::is_position_in_rect;

type PressCallback = CallbackWith<PointerEventContext, ()>;

struct ClosurePointerInputModifierNode<F> {
    handler: F,
}

impl<F> PointerInputModifierNode for ClosurePointerInputModifierNode<F>
where
    F: for<'a> Fn(PointerInput<'a>) + Send + Sync + 'static,
{
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        (self.handler)(input);
    }
}

struct ClosureKeyboardInputModifierNode<F> {
    handler: F,
}

impl<F> KeyboardInputModifierNode for ClosureKeyboardInputModifierNode<F>
where
    F: for<'a> Fn(KeyboardInput<'a>) + Send + Sync + 'static,
{
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        (self.handler)(input);
    }
}

pub(crate) fn with_pointer_input<F>(base: Modifier, handler: F) -> Modifier
where
    F: for<'a> Fn(PointerInput<'a>) + Send + Sync + 'static,
{
    base.push_pointer_input(ClosurePointerInputModifierNode { handler })
}

pub(crate) fn with_keyboard_input<F>(base: Modifier, handler: F) -> Modifier
where
    F: for<'a> Fn(KeyboardInput<'a>) + Send + Sync + 'static,
{
    base.push_keyboard_input(ClosureKeyboardInputModifierNode { handler })
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

struct ClickablePointerModifierNode {
    tap_recognizer: State<TapRecognizer>,
    long_press_recognizer: State<LongPressRecognizer>,
    on_click: Callback,
    enabled: bool,
    block_input: bool,
    on_press: Option<PressCallback>,
    on_release: Option<PressCallback>,
    interaction_state: Option<State<InteractionState>>,
    focus_requester: FocusRequester,
}

impl PointerInputModifierNode for ClickablePointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
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

        let tap_result = self.tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });
        let long_press_result = self.long_press_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });
        let tapped = tap_result.tapped && !long_press_result.triggered;

        let Some(interaction_state) = self.interaction_state else {
            if !self.enabled {
                return;
            }

            if tapped {
                self.on_click.call();
            }
            if self.block_input && within_bounds {
                input.block_all();
            }
            return;
        };

        if self.enabled {
            let hover_changed = interaction_state.with(|state| state.is_hovered() != within_bounds);
            if hover_changed {
                interaction_state.with_mut(|state| state.set_hovered(within_bounds));
            }
        } else {
            let should_reset =
                interaction_state.with(|state| state.is_pressed() || state.is_hovered());
            if should_reset {
                interaction_state.with_mut(|state| {
                    state.release();
                    state.set_hovered(false);
                });
            }
            return;
        }

        let context = pointer_context(input.cursor_position_rel, input.computed_data);

        if tap_result.pressed {
            self.focus_requester.request_focus();
            if let Some(on_press) = self.on_press.as_ref() {
                on_press.call(context);
            }
            let press_changed = interaction_state.with(|state| !state.is_pressed());
            if press_changed {
                interaction_state.with_mut(|state| state.set_pressed(true));
            }
        }

        if tap_result.released {
            let was_pressed = interaction_state.with(|state| state.is_pressed());
            if was_pressed {
                interaction_state.with_mut(|state| state.release());
            }
            if let Some(on_release) = self.on_release.as_ref() {
                on_release.call(context);
            }
        }

        if tapped {
            self.on_click.call();
        }

        if !within_bounds {
            let should_reset =
                interaction_state.with(|state| state.is_pressed() || state.is_hovered());
            if should_reset {
                interaction_state.with_mut(|state| {
                    state.release();
                    state.set_hovered(false);
                });
            }
        }

        if self.block_input && within_bounds {
            input.block_all();
        }
    }
}

struct ClickableKeyboardModifierNode {
    on_click: Callback,
    enabled: bool,
    focus_requester: FocusRequester,
}

impl KeyboardInputModifierNode for ClickableKeyboardModifierNode {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        if !self.enabled
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

struct ToggleablePointerModifierNode {
    tap_recognizer: State<TapRecognizer>,
    value: bool,
    on_value_change: CallbackWith<bool, ()>,
    enabled: bool,
    interaction_state: Option<State<InteractionState>>,
    on_press: Option<PressCallback>,
    on_release: Option<PressCallback>,
    focus_requester: FocusRequester,
}

impl PointerInputModifierNode for ToggleablePointerModifierNode {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
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

        let tap_result = self.tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });

        let Some(interaction_state) = self.interaction_state else {
            return;
        };

        if self.enabled {
            let hover_changed = interaction_state.with(|state| state.is_hovered() != within_bounds);
            if hover_changed {
                interaction_state.with_mut(|state| state.set_hovered(within_bounds));
            }
        } else {
            let should_reset = interaction_state
                .with(|state| state.is_pressed() || state.is_hovered() || state.is_focused());
            if should_reset {
                interaction_state.with_mut(|state| {
                    state.release();
                    state.set_hovered(false);
                    state.set_focused(false);
                });
            }
            return;
        }

        let context = pointer_context(input.cursor_position_rel, input.computed_data);

        if tap_result.pressed {
            self.focus_requester.request_focus();
            if let Some(on_press) = self.on_press.as_ref() {
                on_press.call(context);
            }
            let press_changed = interaction_state.with(|state| !state.is_pressed());
            if press_changed {
                interaction_state.with_mut(|state| state.set_pressed(true));
            }
        }

        if tap_result.released {
            let was_pressed = interaction_state.with(|state| state.is_pressed());
            if was_pressed {
                interaction_state.with_mut(|state| state.release());
            }
            if let Some(on_release) = self.on_release.as_ref() {
                on_release.call(context);
            }
        }

        if tap_result.tapped {
            self.on_value_change.call(!self.value);
        }

        if !within_bounds {
            let should_reset =
                interaction_state.with(|state| state.is_pressed() || state.is_hovered());
            if should_reset {
                interaction_state.with_mut(|state| {
                    state.release();
                    state.set_hovered(false);
                });
            }
        }
    }
}

struct ToggleableKeyboardModifierNode {
    value: bool,
    on_value_change: CallbackWith<bool, ()>,
    enabled: bool,
    focus_requester: FocusRequester,
}

impl KeyboardInputModifierNode for ToggleableKeyboardModifierNode {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        if !self.enabled
            || !has_keyboard_activation_event(input.keyboard_events, input.key_modifiers)
        {
            return;
        }

        self.focus_requester.request_focus();
        self.on_value_change.call(!self.value);
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

struct SelectablePointerModifierNode {
    tap_recognizer: State<TapRecognizer>,
    on_click: Callback,
    enabled: bool,
    interaction_state: Option<State<InteractionState>>,
    on_press: Option<PressCallback>,
    on_release: Option<PressCallback>,
    focus_requester: FocusRequester,
}

impl PointerInputModifierNode for SelectablePointerModifierNode {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
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

        let tap_result = self.tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });

        let Some(interaction_state) = self.interaction_state else {
            return;
        };

        if self.enabled {
            let hover_changed = interaction_state.with(|state| state.is_hovered() != within_bounds);
            if hover_changed {
                interaction_state.with_mut(|state| state.set_hovered(within_bounds));
            }
        } else {
            let should_reset = interaction_state
                .with(|state| state.is_pressed() || state.is_hovered() || state.is_focused());
            if should_reset {
                interaction_state.with_mut(|state| {
                    state.release();
                    state.set_hovered(false);
                    state.set_focused(false);
                });
            }
            return;
        }

        let context = pointer_context(input.cursor_position_rel, input.computed_data);

        if tap_result.pressed {
            self.focus_requester.request_focus();
            if let Some(on_press) = self.on_press.as_ref() {
                on_press.call(context);
            }
            let press_changed = interaction_state.with(|state| !state.is_pressed());
            if press_changed {
                interaction_state.with_mut(|state| state.set_pressed(true));
            }
        }

        if tap_result.released {
            let was_pressed = interaction_state.with(|state| state.is_pressed());
            if was_pressed {
                interaction_state.with_mut(|state| state.release());
            }
            if let Some(on_release) = self.on_release.as_ref() {
                on_release.call(context);
            }
        }

        if tap_result.tapped {
            self.on_click.call();
        }

        if !within_bounds {
            let should_reset =
                interaction_state.with(|state| state.is_pressed() || state.is_hovered());
            if should_reset {
                interaction_state.with_mut(|state| {
                    state.release();
                    state.set_hovered(false);
                });
            }
        }
    }
}

struct SelectableKeyboardModifierNode {
    on_click: Callback,
    enabled: bool,
    focus_requester: FocusRequester,
}

impl KeyboardInputModifierNode for SelectableKeyboardModifierNode {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        if !self.enabled
            || !has_keyboard_activation_event(input.keyboard_events, input.key_modifiers)
        {
            return;
        }

        self.focus_requester.request_focus();
        self.on_click.call();
        input.block_keyboard();
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
        accessibility.role = Some(self.role.unwrap_or(accesskit::Role::Button));
        accessibility.label = self.label.clone();
        accessibility.description = self.description.clone();
        accessibility.focusable = self.enabled;
        accessibility.toggled = Some(if self.selected {
            Toggled::True
        } else {
            Toggled::False
        });
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

struct WindowDragRegionPointerModifierNode {
    tap_recognizer: State<TapRecognizer>,
}

impl PointerInputModifierNode for WindowDragRegionPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
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

        let tap_result = self.tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                within_bounds,
            )
        });

        if tap_result.pressed {
            input.drag_window();
        }
        input.block_all();
    }
}

struct WindowActionPointerModifierNode {
    action: WindowAction,
    tap_recognizer: State<TapRecognizer>,
}

impl PointerInputModifierNode for WindowActionPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
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

        let is_drag_action = matches!(self.action, WindowAction::DragWindow);
        let tap_result = self.tap_recognizer.with_mut(|recognizer| {
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
            match self.action {
                WindowAction::DragWindow => input.drag_window(),
                WindowAction::Minimize => input.minimize_window(),
                WindowAction::Maximize => input.maximize_window(),
                WindowAction::ToggleMaximize => input.toggle_maximize_window(),
                WindowAction::Close => input.close_window(),
            }
        }

        if requested || within_bounds {
            input.block_all();
        }
    }
}

struct BlockTouchPropagationPointerModifierNode;

impl PointerInputModifierNode for BlockTouchPropagationPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
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
    }
}

pub(crate) fn apply_clickable_modifier(base: Modifier, args: ClickableArgs) -> Modifier {
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
    let focus_requester_state = remember(FocusRequester::new);
    let tap_recognizer = remember(TapRecognizer::default);
    let long_press_recognizer = remember(LongPressRecognizer::default);
    let focus_requester = focus_requester.unwrap_or_else(|| focus_requester_state.get());
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
    if let Some(interaction_state) = interaction_state {
        modifier = modifier.on_focus_changed(move |focus_state: FocusState| {
            interaction_state.with_mut(|state| state.set_focused(focus_state.has_focus()));
        });
    }

    modifier
        .push_semantics(ClickableSemanticsModifierNode {
            on_click,
            enabled,
            role,
            label: label.clone(),
            description: description.clone(),
            focus_requester,
        })
        .push_keyboard_input(ClickableKeyboardModifierNode {
            on_click,
            enabled,
            focus_requester,
        })
        .push_pointer_input(ClickablePointerModifierNode {
            tap_recognizer,
            long_press_recognizer,
            on_click,
            enabled,
            block_input,
            on_press,
            on_release,
            interaction_state,
            focus_requester,
        })
}

pub(crate) fn apply_toggleable_modifier(base: Modifier, args: ToggleableArgs) -> Modifier {
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
    let focus_requester_state = remember(FocusRequester::new);
    let tap_recognizer = remember(TapRecognizer::default);
    let focus_requester = focus_requester.unwrap_or_else(|| focus_requester_state.get());

    let mut modifier = base;
    if enabled {
        modifier = modifier
            .focus_requester(focus_requester)
            .focusable()
            .hover_cursor_icon(CursorIcon::Pointer);
    } else if let Some(interaction_state) = interaction_state {
        reset_disabled_interaction_state(interaction_state);
    }
    if let Some(interaction_state) = interaction_state {
        modifier = modifier.on_focus_changed(move |focus_state: FocusState| {
            interaction_state.with_mut(|state| state.set_focused(focus_state.has_focus()));
        });
    }

    modifier
        .push_semantics(ToggleableSemanticsModifierNode {
            value,
            on_value_change,
            enabled,
            role,
            label: label.clone(),
            description: description.clone(),
            focus_requester,
        })
        .push_keyboard_input(ToggleableKeyboardModifierNode {
            value,
            on_value_change,
            enabled,
            focus_requester,
        })
        .push_pointer_input(ToggleablePointerModifierNode {
            tap_recognizer,
            value,
            on_value_change,
            enabled,
            interaction_state,
            on_press,
            on_release,
            focus_requester,
        })
}

pub(crate) fn apply_selectable_modifier(base: Modifier, args: SelectableArgs) -> Modifier {
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
    let focus_requester_state = remember(FocusRequester::new);
    let tap_recognizer = remember(TapRecognizer::default);
    let focus_requester = focus_requester.unwrap_or_else(|| focus_requester_state.get());

    let mut modifier = base;
    if enabled {
        modifier = modifier
            .focus_requester(focus_requester)
            .focusable()
            .hover_cursor_icon(CursorIcon::Pointer);
    } else if let Some(interaction_state) = interaction_state {
        reset_disabled_interaction_state(interaction_state);
    }
    if let Some(interaction_state) = interaction_state {
        modifier = modifier.on_focus_changed(move |focus_state: FocusState| {
            interaction_state.with_mut(|state| state.set_focused(focus_state.has_focus()));
        });
    }

    modifier
        .push_semantics(SelectableSemanticsModifierNode {
            selected,
            on_click,
            enabled,
            role,
            label: label.clone(),
            description: description.clone(),
            focus_requester,
        })
        .push_keyboard_input(SelectableKeyboardModifierNode {
            on_click,
            enabled,
            focus_requester,
        })
        .push_pointer_input(SelectablePointerModifierNode {
            tap_recognizer,
            on_click,
            enabled,
            interaction_state,
            on_press,
            on_release,
            focus_requester,
        })
}

pub(crate) fn apply_window_drag_region_modifier(base: Modifier) -> Modifier {
    let tap_recognizer = remember(TapRecognizer::default);
    base.push_pointer_input(WindowDragRegionPointerModifierNode { tap_recognizer })
}

pub(crate) fn apply_window_action_modifier(base: Modifier, action: WindowAction) -> Modifier {
    let tap_recognizer = remember(TapRecognizer::default);
    base.hover_cursor_icon(CursorIcon::Pointer)
        .push_pointer_input(WindowActionPointerModifierNode {
            action,
            tap_recognizer,
        })
}

pub(crate) fn apply_block_touch_propagation_modifier(base: Modifier) -> Modifier {
    base.push_pointer_input(BlockTouchPropagationPointerModifierNode)
}
