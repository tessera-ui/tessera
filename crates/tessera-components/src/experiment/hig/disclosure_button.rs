//! Compact disclosure buttons for expandable content.
//!
//! ## Usage
//!
//! Reveal or hide details in inspectors, settings, and hierarchical lists.

use tessera_foundation::{
    gesture::{TapRecognizer, TapSettings},
    modifier::{ClickableArgs, InteractionState, ModifierExt as _, PointerEventContext},
};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, CallbackWith, Color, Dp,
    DrawModifierContent, DrawModifierContext, DrawModifierNode, FocusRequester, Modifier,
    SemanticsModifierNode, State,
    accesskit::{Action, Role},
    layout::{Stack, layout},
    modifier::{CursorModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::window::CursorIcon,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    modifier::{ModifierExt as _, apply_pointer_clickable_modifier},
    pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualStroke},
    shape_def::{RoundedCorner, Shape},
};

use super::{
    choice::ChoiceKeyboard,
    theme::{Appearance, HigTheme},
};

#[cfg(test)]
#[path = "disclosure_button_tests.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq)]
struct Value {
    expanded: bool,
    enabled: bool,
    callback: CallbackWith<bool>,
}

impl Value {
    fn request(self, expanded: bool) {
        if self.enabled && self.expanded != expanded {
            self.callback.call(expanded);
        }
    }
}

struct DisclosureSemantics(State<Value>);

impl SemanticsModifierNode for DisclosureSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        handler: &mut Option<AccessibilityActionHandler>,
    ) {
        let value = self.0.get();
        node.expanded = Some(value.expanded);
        if value.enabled {
            node.actions.push(if value.expanded {
                Action::Collapse
            } else {
                Action::Expand
            });
            let previous = handler.take();
            let state = self.0;
            *handler = Some(Box::new(move |action| match action {
                Action::Expand => state.get().request(true),
                Action::Collapse => state.get().request(false),
                _ => {
                    if let Some(previous) = &previous {
                        previous(action);
                    }
                }
            }));
        }
    }
}

#[derive(Clone, PartialEq)]
struct Chevron {
    expanded: bool,
    color: Color,
}

impl DrawModifierNode for Chevron {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let points = if self.expanded {
            [
                [2.743 / 13.0, 5.152 / 13.0],
                [6.039 / 13.0, 8.552 / 13.0],
                [9.350 / 13.0, 5.173 / 13.0],
            ]
        } else {
            [
                [5.374 / 13.0, 2.893 / 13.0],
                [8.721 / 13.0, 6.167 / 13.0],
                [5.361 / 13.0, 9.489 / 13.0],
            ]
        };
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                color: Color::TRANSPARENT,
                radii: [Dp::ZERO; 2],
                insets: [Dp::ZERO; 4],
                clip_radius: None,
                edge_smoothness: 1.0,
                line: None,
                stroke: Some(PerceptualStroke {
                    raster_mask: true,
                    color: self.color,
                    width: Dp(1.465),
                    points,
                }),
            });
        content.draw(context.render_input);
    }
}

#[tessera]
fn disclosure_glyph(expanded: bool, color: Color, focused: bool) {
    let mut modifier = Modifier::new().size(Dp(13.0), Dp(13.0));
    if focused {
        let corner = RoundedCorner::manual(Dp(3.0), 2.0);
        modifier = modifier.border_with_shape(
            Dp(2.0),
            Color::from_rgb(0.0, 0.19461783, 1.0).with_alpha(0.6),
            Shape::RoundedRectangle {
                top_left: corner,
                top_right: corner,
                bottom_left: corner,
                bottom_right: corner,
            },
        );
    }
    layout(Stack).modifier(modifier.push_draw(Chevron { expanded, color }));
}

/// # disclosure_button
///
/// Requests expansion changes from a compact chevron beside optional details.
///
/// ## Usage
///
/// Reveal inspector sections, advanced settings, or hierarchical children.
///
/// ## Parameters
///
/// - `expanded`, `on_expanded_change` — controlled disclosure state and requested updates.
/// - `enabled` — input and focus availability; defaults to true.
/// - `window_active` — active-window appearance; defaults to true.
/// - `modifier`, `focus_requester` — layout, semantics, and optional external focus handle.
/// - `accessibility_label`, `accessibility_description` — spoken name and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::disclosure_button::disclosure_button;
/// use tessera_ui::{remember, tessera, testing::UiTest};
/// #[tessera]
/// fn app() {
///     let expanded = remember(|| false);
///     disclosure_button(expanded.get()).accessibility_label("Advanced settings")
///         .on_expanded_change(move |next| expanded.set(next));
/// }
/// let mut ui = UiTest::new().content(app);
/// ui.frame();
/// let (id, _) = ui.accessibility_node("Advanced settings");
/// assert!(ui.accessibility_action(id, tessera_ui::accesskit::Action::Click));
/// ui.frame();
/// assert_eq!(ui.accessibility_node("Advanced settings").1.is_expanded(), Some(true));
/// ```
#[tessera]
pub fn disclosure_button(
    expanded: bool,
    on_expanded_change: Option<CallbackWith<bool>>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let active = window_active.unwrap_or(true);
    let next = Value {
        expanded,
        enabled,
        callback: on_expanded_change.unwrap_or_else(CallbackWith::default_value),
    };
    let value = remember(|| next);
    if value.get() != next {
        value.set(next);
    }
    let activate = Callback::new(move || {
        let current = value.get();
        current.request(!current.expanded);
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
    let pressed = enabled
        && active
        && (held.get() || (pointer_down.get() && interaction.with(InteractionState::is_hovered)));
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let color = if !enabled || !active {
        let gray = match (dark, enabled) {
            (true, true) => 0.2158605,
            (true, false) => 0.05126946,
            (false, true) => 0.5209956,
            (false, false) => 0.7835378,
        };
        Color::from_rgb(gray, gray, gray)
    } else {
        let tint = if dark { Color::WHITE } else { Color::BLACK };
        tint.with_alpha(match (dark, pressed) {
            (true, true) => 1.0,
            (true, false) | (false, true) => 0.85,
            (false, false) => 0.55,
        })
    };
    let modifier = apply_pointer_clickable_modifier(
        modifier.unwrap_or_default(),
        ClickableArgs {
            enabled,
            on_click: activate,
            role: Some(Role::DisclosureTriangle),
            label: accessibility_label,
            description: accessibility_description,
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
    .push_keyboard_input(ChoiceKeyboard {
        enabled,
        held,
        activate,
    })
    .push_semantics(DisclosureSemantics(value))
    .hover_cursor_icon(CursorIcon::Default);
    boxed()
        .alignment(Alignment::Center)
        .modifier(modifier)
        .children(move || {
            disclosure_glyph(
                expanded,
                color,
                enabled && active && requester.is_focus_visible(),
            );
        });
}
