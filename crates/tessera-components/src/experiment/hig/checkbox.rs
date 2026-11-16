//! Checkboxes with optional mixed-state selection.
//!
//! ## Usage
//!
//! Enable settings independently or represent a partially selected group.

use tessera_foundation::{
    gesture::{TapRecognizer, TapSettings},
    modifier::{
        ClickableArgs, InteractionState, ModifierExt as _, Padding, PointerEventContext,
        SemanticsArgs,
    },
};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, CallbackWith, Color, Dp,
    DrawModifierContent, DrawModifierContext, DrawModifierNode, FocusRequester, KeyboardInput,
    KeyboardInputModifierNode, Modifier, SemanticsModifierNode, State,
    accesskit::Toggled,
    layout::{Stack, layout},
    modifier::{CursorModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
        window::CursorIcon,
    },
};

use crate::{
    alignment::CrossAxisAlignment,
    modifier::{ModifierExt as _, apply_pointer_clickable_modifier},
    pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualStroke},
    row::row,
    shape_def::{RoundedCorner, Shape},
};

use super::{
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "checkbox_tests.rs"]
mod tests;

/// The selection represented by a checkbox.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckboxState {
    /// No selection.
    #[default]
    Off,
    /// Complete selection.
    On,
    /// Partial selection; requires `allows_mixed`.
    Mixed,
}

impl CheckboxState {
    fn normalized(self, allows_mixed: bool) -> Self {
        if self == Self::Mixed && !allows_mixed {
            Self::On
        } else {
            self
        }
    }

    fn next(self, allows_mixed: bool) -> Self {
        match self.normalized(allows_mixed) {
            Self::Off if allows_mixed => Self::Mixed,
            Self::Off | Self::Mixed => Self::On,
            Self::On => Self::Off,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Action {
    value: CheckboxState,
    allows_mixed: bool,
    enabled: bool,
    callback: CallbackWith<CheckboxState>,
}

struct CheckboxSemantics(CheckboxState);

impl SemanticsModifierNode for CheckboxSemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.toggled = Some(match self.0 {
            CheckboxState::Off => Toggled::False,
            CheckboxState::On => Toggled::True,
            CheckboxState::Mixed => Toggled::Mixed,
        });
    }
}

struct CheckboxKeyboard {
    enabled: bool,
    held: State<bool>,
    activate: Callback,
}

impl KeyboardInputModifierNode for CheckboxKeyboard {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        if !self.enabled {
            return;
        }
        let modified = input.key_modifiers.control_key()
            || input.key_modifiers.alt_key()
            || input.key_modifiers.super_key();
        if modified && self.held.get() {
            self.held.set(false);
        }
        input.keyboard_events.retain(|event| {
            if event.logical_key != Key::Named(NamedKey::Space) || modified {
                return true;
            }
            match event.state {
                ElementState::Pressed => {
                    if !self.held.get() && !event.repeat {
                        self.held.set(true);
                    }
                }
                ElementState::Released => {
                    if self.held.get() {
                        self.held.set(false);
                        self.activate.call();
                    }
                }
            }
            false
        });
    }
}

#[derive(Clone, PartialEq)]
struct Glyph(PerceptualFillCommand);

impl DrawModifierNode for Glyph {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(self.0.clone());
        content.draw(context.render_input);
    }
}

#[tessera]
fn checkbox_glyph(command: PerceptualFillCommand, focused: bool) {
    let mut modifier = Modifier::new().size(Dp(16.0), Dp(16.0));
    if focused {
        let corner = RoundedCorner::manual(Dp(5.0), 2.0);
        modifier = modifier.border_with_shape(
            Dp(2.0),
            Color::from_rgb(0.0, 0.19461783, 1.0).with_alpha(0.6),
            Shape::RoundedRectangle {
                top_left: corner,
                top_right: corner,
                bottom_right: corner,
                bottom_left: corner,
            },
        );
    }
    layout(Stack).modifier(modifier.push_draw(Glyph(command)));
}

/// # checkbox
///
/// Reports selection changes from a labeled checkbox in settings, forms, and selection groups.
///
/// ## Usage
///
/// Let users enable independent options or cycle through partial group selection.
///
/// ## Parameters
///
/// - `value`, `on_value_change` — controlled selection and requested updates.
/// - `label` — optional visible title and default accessible name.
/// - `allows_mixed` — cycles Off → Mixed → On → Off; defaults to false, treating Mixed as On.
/// - `enabled` — input and focus availability; defaults to true.
/// - `window_active` — containing window's active appearance; defaults to true.
/// - `modifier`, `focus_requester` — layout, semantics, and optional external focus handle.
/// - `accessibility_label`, `accessibility_description` — spoken content overrides.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::checkbox::{CheckboxState, checkbox};
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
/// use tessera_ui::{Modifier, State, remember, tessera, testing::{TestProbe, UiTest}};
/// #[tessera]
/// fn app(probe: TestProbe<Option<State<CheckboxState>>>) {
///     let selected = remember(|| CheckboxState::Off);
///     probe.set(Some(selected));
///     checkbox(selected.get()).label("Notifications")
///         .on_value_change(move |next| selected.set(next))
///         .modifier(Modifier::new().semantics(SemanticsArgs {
///             test_tag: Some("notifications".into()), ..Default::default()
///         }));
/// }
/// let probe = TestProbe::default();
/// let mut ui = UiTest::new().content(|| app(probe.clone()));
/// ui.frame();
/// ui.node("notifications").click();
/// assert_eq!(probe.get().unwrap().get(), CheckboxState::On);
/// ```
#[tessera]
pub fn checkbox(
    value: CheckboxState,
    on_value_change: Option<CallbackWith<CheckboxState>>,
    #[prop(into)] label: Option<String>,
    allows_mixed: Option<bool>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let allows_mixed = allows_mixed.unwrap_or(false);
    let value = value.normalized(allows_mixed);
    let active = window_active.unwrap_or(true);
    let callback = on_value_change.unwrap_or_else(CallbackWith::default_value);
    let action = remember(|| Action {
        value,
        allows_mixed,
        enabled,
        callback,
    });
    let next = Action {
        value,
        allows_mixed,
        enabled,
        callback,
    };
    if action.get() != next {
        action.set(next);
    }
    let activate = Callback::new(move || {
        let action = action.get();
        if action.enabled {
            action.callback.call(action.value.next(action.allows_mixed));
        }
    });
    let requester = if let Some(requester) = focus_requester {
        requester
    } else {
        remember(FocusRequester::new).get()
    };
    let interaction = remember(InteractionState::new);
    let held = remember(|| false);
    let pointer_down = remember(|| false);
    let tap = remember(|| {
        TapRecognizer::new(TapSettings {
            slop_px: f32::INFINITY,
            ..Default::default()
        })
    });
    if !enabled {
        if held.get() {
            held.set(false);
        }
        if pointer_down.get() {
            pointer_down.set(false);
            tap.with_mut(TapRecognizer::cancel);
        }
    } else if held.get() && !requester.is_focused() {
        held.set(false);
    }
    let pressed =
        held.get() || (pointer_down.get() && interaction.with(InteractionState::is_hovered));
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let dark = theme.appearance == Appearance::Dark;
    let selected = value != CheckboxState::Off;
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let fill = if enabled && active && selected {
        if pressed {
            Color::from_rgb(0.0, 0.18, 0.60)
        } else {
            Color::from_rgb(0.0, 0.19461783, 1.0)
        }
    } else {
        tint.with_alpha(match (enabled, selected, pressed && active) {
            (false, false, _) => 0.05,
            (false, true, _) => 0.07,
            (true, _, true) => 0.20,
            (true, false, false) => 0.10,
            (true, true, false) => 0.14,
        })
    };
    let ink = if enabled {
        theme.label_color()
    } else {
        tint.with_alpha(if dark { 0.25 } else { 0.26 })
    };
    let mark_color = if enabled && active { Color::WHITE } else { ink };
    let stroke = selected.then_some(PerceptualStroke {
        color: mark_color,
        width: Dp(2.04),
        points: if value == CheckboxState::Mixed {
            [[0.35, 0.50], [0.50, 0.50], [0.65, 0.50]]
        } else {
            [[0.253, 0.548], [0.434, 0.750], [0.734, 0.298]]
        },
    });
    let label = label.unwrap_or_default();
    let modifier = apply_pointer_clickable_modifier(
        modifier.unwrap_or_default(),
        ClickableArgs {
            enabled,
            on_click: activate,
            label: Some(accessibility_label.unwrap_or_else(|| label.clone())),
            description: accessibility_description,
            role: Some(tessera_ui::accesskit::Role::CheckBox),
            focus_requester: Some(requester),
            interaction_state: Some(interaction),
            tap_recognizer: Some(tap),
            on_press: Some(CallbackWith::new(move |_: PointerEventContext| {
                pointer_down.set(true)
            })),
            on_release: Some(CallbackWith::new(move |_: PointerEventContext| {
                pointer_down.set(false)
            })),
            ..Default::default()
        },
    )
    .push_keyboard_input(CheckboxKeyboard {
        enabled,
        held,
        activate,
    })
    .push_semantics(CheckboxSemantics(value))
    .hover_cursor_icon(CursorIcon::Default);
    let focused = enabled && active && requester.is_focus_visible();
    row()
        .modifier(modifier)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            checkbox_glyph(
                PerceptualFillCommand {
                    color: fill,
                    radius: Dp(5.0),
                    stroke: stroke.clone(),
                },
                focused,
            );
            if !label.is_empty() {
                text(label.clone())
                    .style(TextStyle::default())
                    .color(ink)
                    .single_line(true)
                    .modifier(
                        Modifier::new()
                            .padding(Padding {
                                left: Dp(6.0),
                                ..Default::default()
                            })
                            .semantics(SemanticsArgs {
                                hidden: true,
                                ..Default::default()
                            }),
                    );
            }
        });
}
