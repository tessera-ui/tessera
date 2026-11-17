//! Push buttons for actions and default form submission.
//!
//! ## Usage
//!
//! Confirm changes, cancel operations, and invoke commands with pointer or keyboard input.

use std::time::Duration;

use tessera_foundation::modifier::{
    ClickableArgs, InteractionState, ModifierExt as _, Padding, SemanticsArgs,
};
use tessera_ui::{
    Callback, Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode,
    FocusRequester, KeyboardInput, KeyboardInputModifierNode, Modifier, RenderSlotWith, State,
    accesskit::Role,
    launch_once,
    layout::{Stack, layout},
    modifier::{CursorModifierExt as _, ModifierCapabilityExt as _},
    remember, remember_with_key, tessera,
    time::Instant,
    use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
        window::CursorIcon,
    },
};

use crate::{
    alignment::Alignment,
    animation::run_frames,
    boxed::boxed,
    modifier::ModifierExt as _,
    pipelines::perceptual_fill::PerceptualFillCommand,
    shape_def::{RoundedCorner, Shape},
};

use super::{
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "button_tests.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq)]
struct DefaultAction {
    callback: Callback,
    pressed: State<bool>,
    pulse: State<Option<Instant>>,
}

impl DefaultAction {
    fn release(self) {
        if self.pressed.is_alive() {
            self.pressed.set(false);
        }
    }
}

fn cancel_key(held: State<Option<(NamedKey, DefaultAction)>>) {
    if let Some((_, action)) = held.get() {
        held.set(None);
        action.release();
    }
}

/// A form-local Return-key target supplied by [`button_scope`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ButtonScope {
    target: State<Option<State<Option<DefaultAction>>>>,
}

impl ButtonScope {
    fn action(self) -> Option<DefaultAction> {
        self.target
            .get()
            .filter(|state| state.is_alive())
            .and_then(|state| state.get())
    }
}

struct DefaultRegistration {
    scope: ButtonScope,
    action: State<Option<DefaultAction>>,
}

impl Drop for DefaultRegistration {
    fn drop(&mut self) {
        if self.scope.target.is_alive() && self.scope.target.get() == Some(self.action) {
            self.scope.target.set(None);
        }
    }
}

struct ButtonKeyboard {
    enabled: bool,
    callback: Callback,
    pressed: State<bool>,
    pulse: State<Option<Instant>>,
    scope: Option<ButtonScope>,
    held: State<Option<(NamedKey, DefaultAction)>>,
    scope_only: bool,
}

impl KeyboardInputModifierNode for ButtonKeyboard {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        if !self.enabled {
            return;
        }
        let modified = input.key_modifiers.control_key()
            || input.key_modifiers.alt_key()
            || input.key_modifiers.super_key();
        if modified {
            cancel_key(self.held);
        }
        input.keyboard_events.retain(|event| {
            let Key::Named(key) = event.logical_key else {
                return true;
            };
            if key != NamedKey::Enter && (key != NamedKey::Space || self.scope_only) {
                return true;
            }
            if modified {
                return true;
            }
            let action = if key == NamedKey::Enter && self.scope.is_some() {
                self.scope.and_then(ButtonScope::action)
            } else {
                Some(DefaultAction {
                    callback: self.callback,
                    pressed: self.pressed,
                    pulse: self.pulse,
                })
            };
            let Some(action) = action else {
                cancel_key(self.held);
                return self.scope_only;
            };
            if key == NamedKey::Enter {
                if event.state == ElementState::Pressed {
                    action.pulse.set(Some(input.timestamp));
                    action.callback.call();
                }
                return false;
            }
            match event.state {
                ElementState::Pressed => {
                    if self.held.get().is_none() {
                        self.held.set(Some((key, action)));
                        action.pressed.set(true);
                    }
                }
                ElementState::Released => {
                    if let Some((held_key, previous)) = self.held.get()
                        && held_key == key
                    {
                        self.held.set(None);
                        previous.release();
                        if previous == action {
                            action.callback.call();
                        }
                    }
                }
            }
            false
        });
    }
}

/// # button_scope
///
/// Routes unhandled Return input within focused form content to its enabled default button.
///
/// ## Usage
///
/// Share one scope between a form and its primary confirmation action.
///
/// ## Parameters
///
/// - `child` — content receiving the explicit scope; arrange multiple controls in a row or column.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::button::{button, button_scope};
/// use tessera_ui::{Modifier, tessera, testing::UiTest};
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
/// #[tessera]
/// fn app() {
///     button_scope().child(|scope| {
///         button("Save").scope(scope).default_action(true)
///             .modifier(Modifier::new().semantics(SemanticsArgs {
///                 test_tag: Some("primary".into()), ..Default::default()
///             }));
///     });
/// }
/// let frame = UiTest::new().content(app).frame();
/// assert!(frame.node("primary").bounds().height.0 >= 24);
/// ```
#[tessera]
pub fn button_scope(child: Option<RenderSlotWith<ButtonScope>>) {
    let scope = ButtonScope {
        target: remember(|| None),
    };
    let pressed = remember(|| false);
    let pulse = remember(|| None);
    let held = remember(|| None);
    layout(Stack)
        .modifier(Modifier::new().push_keyboard_input(ButtonKeyboard {
            enabled: true,
            callback: Callback::noop(),
            pressed,
            pulse,
            scope: Some(scope),
            held,
            scope_only: true,
        }))
        .child(move || {
            if let Some(child) = child {
                child.render(scope);
            }
        });
}

fn button_shape() -> Shape {
    let corner = RoundedCorner::manual(Dp(6.0), 2.0);
    Shape::RoundedRectangle {
        top_left: corner,
        top_right: corner,
        bottom_right: corner,
        bottom_left: corner,
    }
}

#[derive(Clone, PartialEq)]
struct ButtonBackground(Color);

impl DrawModifierNode for ButtonBackground {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                outline_width: Dp::ZERO,
                snap_to_pixels: false,
                path: None,
                color: self.0,
                radii: [Dp(6.0); 2],
                insets: [Dp(0.0); 4],
                clip_radius: None,
                line: None,
                edge_smoothness: 1.0,
                stroke: None,
            });
        content.draw(context.render_input);
    }
}

/// # button
///
/// Invokes an action on pointer or Space release, or when a form's default Return key is pressed.
///
/// ## Usage
///
/// Submit forms, confirm choices, and invoke application commands.
///
/// ## Parameters
///
/// - `label` — visible title and default accessible name.
/// - `on_click` — action invoked once per completed activation.
/// - `enabled` — disables input, focus traversal, and accessibility activation when false.
/// - `default_action`, `scope` — primary action and its explicit Return-key scope.
/// - `window_active` — whether the containing window is active; defaults to true.
/// - `modifier` — caller layout and semantics.
/// - `interaction_state`, `focus_requester` — optional external interaction and focus handles.
/// - `accessibility_label`, `accessibility_description` — spoken content overrides.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::button::button;
/// use tessera_ui::{Modifier, tessera, testing::UiTest};
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
/// #[tessera]
/// fn app() {
///     button("Save").modifier(Modifier::new().semantics(SemanticsArgs {
///         test_tag: Some("save".into()), ..Default::default()
///     }));
/// }
/// let frame = UiTest::new().content(app).frame();
/// assert!(frame.node("save").bounds().height.0 >= 24);
/// ```
#[tessera]
pub fn button(
    #[prop(into)] label: String,
    on_click: Option<Callback>,
    enabled: Option<bool>,
    default_action: Option<bool>,
    scope: Option<ButtonScope>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    interaction_state: Option<State<InteractionState>>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let default_action = default_action.unwrap_or(false);
    let active = window_active.unwrap_or(true);
    let interaction = if let Some(state) = interaction_state {
        state
    } else {
        remember(InteractionState::new)
    };
    let requester = if let Some(requester) = focus_requester {
        requester
    } else {
        remember(FocusRequester::new).get()
    };
    let held = remember(|| None::<(NamedKey, DefaultAction)>);
    let keyboard_pressed = remember(|| false);
    let pulse = remember(|| None::<Instant>);
    let callback = on_click.unwrap_or_else(Callback::noop);
    let registration = remember(|| None::<DefaultAction>);
    if let Some(scope) = scope {
        remember_with_key(scope, move || DefaultRegistration {
            scope,
            action: registration,
        });
        let next = (enabled && default_action).then_some(DefaultAction {
            callback,
            pressed: keyboard_pressed,
            pulse,
        });
        if registration.get() != next {
            registration.set(next);
        }
        if enabled && default_action && scope.target.get() != Some(registration) {
            assert!(
                scope.action().is_none(),
                "a button scope supports one enabled default action"
            );
            scope.target.set(Some(registration));
        } else if (!enabled || !default_action) && scope.target.get() == Some(registration) {
            scope.target.set(None);
        }
    }
    if !enabled {
        cancel_key(held);
        if keyboard_pressed.get() {
            keyboard_pressed.set(false);
        }
        if pulse.get().is_some() {
            pulse.set(None);
        }
    }
    if held.get().is_some() && !requester.is_focused() {
        cancel_key(held);
    }
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let dark = theme.appearance == Appearance::Dark;
    let pulsing = pulse.get().is_some();
    if pulsing {
        launch_once(run_frames(pulse, |pulse, now| {
            if pulse.is_some_and(|start| {
                now.saturating_duration_since(start) >= Duration::from_millis(100)
            }) {
                *pulse = None;
            }
            pulse.is_some()
        }));
    }
    let pressed = enabled
        && (pulsing || keyboard_pressed.get() || interaction.with(InteractionState::is_pressed));
    let focused = enabled && active && requester.is_focus_visible();
    let fill = if enabled && default_action && active {
        if pressed {
            Color::from_rgb(0.0, 0.18, 0.60)
        } else {
            Color::from_rgb(0.0, 0.19461783, 1.0)
        }
    } else {
        let tint = if dark { Color::WHITE } else { Color::BLACK };
        tint.with_alpha(match (dark, enabled, pressed && active) {
            (false, false, _) => 0.04,
            (false, true, true) => 0.16,
            (false, true, false) => 0.075,
            (true, false, _) => 0.035,
            (true, true, true) => 0.16,
            (true, true, false) => 0.08,
        })
    };
    let ink = if default_action && enabled && active || dark {
        Color::WHITE
    } else {
        Color::BLACK
    };
    let ink = ink.with_alpha(if enabled && default_action && active {
        1.0
    } else if enabled {
        0.85
    } else if dark {
        0.25
    } else {
        0.26
    });
    let shape = button_shape();
    let mut modifier = modifier
        .unwrap_or_default()
        .push_keyboard_preview_input(ButtonKeyboard {
            enabled,
            callback,
            pressed: keyboard_pressed,
            pulse,
            scope,
            held,
            scope_only: false,
        })
        .clickable(ClickableArgs {
            on_click: callback,
            enabled,
            role: Some(Role::Button),
            label: Some(accessibility_label.unwrap_or_else(|| label.clone())),
            description: accessibility_description,
            interaction_state: Some(interaction),
            focus_requester: Some(requester),
            ..Default::default()
        })
        .hover_cursor_icon(CursorIcon::Default)
        .default_min_size(Dp(28.0), Dp(24.0));
    if focused {
        modifier = modifier.border_with_shape(
            Dp(2.0),
            Color::from_rgb(0.0, 0.19461783, 1.0).with_alpha(0.6),
            shape,
        );
    }
    modifier = modifier
        .push_draw(ButtonBackground(fill))
        .padding(Padding::symmetric(Dp(12.0), Dp(4.0)));
    boxed()
        .alignment(Alignment::Center)
        .modifier(modifier)
        .children(move || {
            text(label.clone())
                .style(TextStyle::default())
                .color(ink)
                .single_line(true)
                .modifier(Modifier::new().semantics(SemanticsArgs {
                    hidden: true,
                    ..Default::default()
                }));
        });
}
