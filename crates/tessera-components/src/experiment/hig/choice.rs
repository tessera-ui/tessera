use tessera_foundation::{
    modifier::{ClickableArgs, InteractionState, ModifierExt as _, Padding, SemanticsArgs},
    pointer_gestures::TapGestureConfig,
};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, Color, Dp, DrawModifierContent,
    DrawModifierContext, DrawModifierNode, FocusRequester, KeyboardInput,
    KeyboardInputModifierNode, Modifier, SemanticsModifierNode, State,
    accesskit::{Role, Toggled},
    launch,
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
    animation::animate_to,
    modifier::ModifierExt as _,
    pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualStroke},
    row::row,
};

use super::{
    motion::{Eased, FOCUS_RING, SELECTION, focus_ring, focus_ring_color, mix, ring_shape},
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum ChoiceMark {
    Check,
    Dash,
    Dot,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct ChoiceStyle {
    pub role: Role,
    pub mark: Option<ChoiceMark>,
    /// Space between the glyph and the label.
    pub label_gap: Dp,
    /// Height of the line the label is centred in.
    pub label_line_height: Dp,
}

struct ChoiceSemantics(Option<ChoiceMark>);

impl SemanticsModifierNode for ChoiceSemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.toggled = Some(match self.0 {
            None => Toggled::False,
            Some(ChoiceMark::Dash) => Toggled::Mixed,
            Some(_) => Toggled::True,
        });
    }
}

pub(super) struct ChoiceKeyboard {
    pub enabled: bool,
    pub held: State<bool>,
    pub activate: Callback,
}

impl KeyboardInputModifierNode for ChoiceKeyboard {
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
fn choice_glyph(command: PerceptualFillCommand, ring: f32) {
    let mut modifier = Modifier::new().size(Dp(16.0), Dp(16.0));
    if ring > 0.0 {
        modifier = modifier.border_with_shape(
            Dp(2.0),
            focus_ring_color(ring),
            ring_shape(command.radii[0]),
        );
    }
    layout(Stack).modifier(modifier.push_draw(Glyph(command)));
}

#[tessera]
pub(super) fn choice_control(
    style: ChoiceStyle,
    activate: Callback,
    label: String,
    enabled: bool,
    active: bool,
    modifier: Modifier,
    requester: FocusRequester,
    focus_modifier: Option<Modifier>,
    accessibility_label: Option<String>,
    description: Option<String>,
) {
    let interaction = remember(InteractionState::new);
    let held = remember(|| false);
    if !enabled {
        if held.get() {
            held.set(false);
        }
    } else if held.get() && !requester.is_focused() {
        held.set(false);
    }
    let pressed = held.get() || interaction.with(|state| state.is_pressed() && state.is_hovered());
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let dark = theme.appearance == Appearance::Dark;
    let selected = style.mark.is_some();
    let target = if selected { 1.0 } else { 0.0 };
    let selection = remember(|| Eased::new(target, SELECTION));
    launch(selected, animate_to(selection, target));
    let progress = selection.with(Eased::value);
    // The last mark keeps drawing while it fades out.
    let shown_mark = remember(|| style.mark);
    if style.mark.is_some() && shown_mark.get() != style.mark {
        shown_mark.set(style.mark);
    }
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let fill_for = |selected: bool| {
        if enabled && active && selected {
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
        }
    };
    let fill = mix(fill_for(false), fill_for(true), progress);
    let ink = if enabled {
        theme.label_color()
    } else {
        tint.with_alpha(if dark { 0.25 } else { 0.26 })
    };
    let mark_color = if enabled && active { Color::WHITE } else { ink };
    // Marks fade in while growing from 60% about the glyph centre; the dot grows from nothing.
    let scale = 0.6 + 0.4 * progress;
    let stroke = shown_mark
        .get()
        .filter(|_| progress > 0.0)
        .map(|mark| PerceptualStroke {
            raster_mask: false,
            color: mark_color.with_alpha(mark_color.a * progress),
            width: Dp(if mark == ChoiceMark::Dot {
                5.0 * f64::from(progress)
            } else {
                2.04
            }),
            points: match mark {
                ChoiceMark::Check => [[0.253, 0.548], [0.434, 0.750], [0.734, 0.298]],
                ChoiceMark::Dash => [[0.35, 0.50], [0.50, 0.50], [0.65, 0.50]],
                ChoiceMark::Dot => [[0.50, 0.50]; 3],
            }
            .map(|[x, y]| [0.5 + (x - 0.5) * scale, 0.5 + (y - 0.5) * scale]),
        });
    let modifier = modifier
        .clickable(ClickableArgs {
            enabled,
            on_click: activate,
            label: Some(accessibility_label.unwrap_or_else(|| label.clone())),
            description,
            role: Some(style.role),
            focus_requester: Some(requester),
            interaction_state: Some(interaction),
            keyboard_activation: false,
            tap_config: TapGestureConfig {
                slop_px: f32::INFINITY,
                cancel_on_exit: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .then(focus_modifier.unwrap_or_default())
        .push_keyboard_input(ChoiceKeyboard {
            enabled,
            held,
            activate,
        })
        .push_semantics(ChoiceSemantics(style.mark))
        .hover_cursor_icon(CursorIcon::Default);
    let focused = enabled && active && requester.is_focus_visible();
    let ring = remember(|| Eased::new(0.0, FOCUS_RING));
    launch(focused, focus_ring(ring, focused));
    let ring = if focused {
        ring.with(Eased::value)
    } else {
        0.0
    };
    row()
        .modifier(modifier)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            choice_glyph(
                PerceptualFillCommand {
                    outline_width: Dp::ZERO,
                    snap_to_pixels: false,
                    path: None,
                    color: fill,
                    radii: [Dp(if style.role == Role::RadioButton {
                        8.0
                    } else {
                        5.0
                    }); 2],
                    stroke: stroke.clone(),
                    insets: [Dp(0.0); 4],
                    clip_radius: None,
                    line: None,
                    edge_smoothness: 1.0,
                },
                ring,
            );
            if !label.is_empty() {
                text(label.clone())
                    .style(TextStyle {
                        line_height: style.label_line_height,
                        ..Default::default()
                    })
                    .color(ink)
                    .single_line(true)
                    .modifier(
                        Modifier::new()
                            .padding(Padding {
                                left: style.label_gap,
                                ..Default::default()
                            })
                            .clear_and_set_semantics(SemanticsArgs {
                                hidden: true,
                                ..Default::default()
                            }),
                    );
            }
        });
}
