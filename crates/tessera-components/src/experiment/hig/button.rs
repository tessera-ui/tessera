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
    AccessibilityActionHandler, AccessibilityNode, Callback, Color, Dp, DrawModifierContent,
    DrawModifierContext, DrawModifierNode, FocusRequester, KeyboardInput,
    KeyboardInputModifierNode, Modifier, RenderSlotWith, SemanticsModifierNode, State,
    accesskit::{Role, Toggled},
    launch, launch_once,
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
    alignment::{Alignment, CrossAxisAlignment},
    animation::run_frames,
    boxed::boxed,
    column::column,
    modifier::ModifierExt as _,
    pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualStroke},
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
};

use super::{
    control_group::ControlGroupContext,
    glass::{GlassAppearance, GlassMaterial, glass},
    link::link_color,
    menu::{MenuContext, menu_item},
    motion::{Eased, FOCUS_RING, focus_ring, focus_ring_color, mix},
    text::text,
    theme::{Appearance, ControlSize, HigTheme, TextStyle},
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

fn button_shape(radius: Dp) -> Shape {
    let corner = RoundedCorner::manual(radius, 2.0);
    Shape::RoundedRectangle {
        top_left: corner,
        top_right: corner,
        bottom_right: corner,
        bottom_left: corner,
    }
}

fn srgb(r: u8, g: u8, b: u8) -> Color {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    Color::from_rgb(channel(r), channel(g), channel(b))
}

struct ButtonToggleSemantics(Option<bool>);

impl SemanticsModifierNode for ButtonToggleSemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        if let Some(on) = self.0 {
            node.toggled = Some(if on { Toggled::True } else { Toggled::False });
        }
    }
}

/// The appearance of a [`button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    /// The platform default, a bordered push button on macOS.
    #[default]
    Automatic,
    /// A rounded push button with a gray fill.
    Bordered,
    /// A rounded push button filled with the accent color.
    BorderedProminent,
    /// The title alone, with the push button's padding.
    Borderless,
    /// The title alone, dimmed while pressed.
    Plain,
    /// The title in the link color.
    Link,
    /// A compact button for accessory bars that shows a fill on hover.
    AccessoryBar,
    /// An action in an accessory bar, such as adding or removing an item.
    AccessoryBarAction,
    /// A capsule of glass.
    Glass,
    /// A capsule filled with the accent color.
    GlassProminent,
}

/// The purpose of a [`button`], which can change its appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonRole {
    /// Cancels the current operation.
    Cancel,
    /// Deletes data or performs another action that cannot be undone; shown in red.
    Destructive,
}

/// A symbol after a button's title that shows it opens a menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ButtonIndicator {
    /// A down chevron, for buttons that pull down a menu of commands.
    PullDown,
    /// Up and down chevrons, for buttons that pop up a menu of choices.
    PopUp,
}

struct MenuButtonSemantics(bool);

impl SemanticsModifierNode for MenuButtonSemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.has_popup = Some(tessera_ui::accesskit::HasPopup::Menu);
        node.expanded = Some(self.0);
    }
}

#[derive(Clone, PartialEq)]
struct Chevron {
    points: [[f32; 2]; 3],
    color: Color,
}

impl DrawModifierNode for Chevron {
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
                color: Color::TRANSPARENT,
                radii: [Dp::ZERO; 2],
                insets: [Dp::ZERO; 4],
                clip_radius: None,
                line: None,
                edge_smoothness: 1.0,
                stroke: Some(PerceptualStroke {
                    raster_mask: true,
                    color: self.color,
                    width: Dp(1.5),
                    points: self.points,
                }),
            });
        content.draw(context.render_input);
    }
}

#[tessera]
fn chevron(points: [[f32; 2]; 3], color: Color) {
    layout(Stack).modifier(
        Modifier::new()
            .size(Dp(8.0), Dp(5.0))
            .push_draw(Chevron { points, color }),
    );
}

const DOWN: [[f32; 2]; 3] = [[0.1, 0.15], [0.5, 0.85], [0.9, 0.15]];
const UP: [[f32; 2]; 3] = [[0.1, 0.85], [0.5, 0.15], [0.9, 0.85]];

#[tessera]
fn indicator_glyph(indicator: ButtonIndicator, color: Color) {
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || match indicator {
            ButtonIndicator::PullDown => {
                chevron(DOWN, color);
            }
            ButtonIndicator::PopUp => {
                chevron(UP, color);
                spacer().modifier(Modifier::new().height(Dp(2.0)));
                chevron(DOWN, color);
            }
        });
}

#[derive(Clone, Copy)]
struct SizeMetrics {
    height: f64,
    font: f64,
    line: f64,
    padding: f64,
    radius: f64,
}

fn size_metrics(size: ControlSize) -> SizeMetrics {
    let (height, font, line, padding, radius) = match size {
        ControlSize::Mini => (16.0, 9.0, 11.0, 6.0, 4.0),
        ControlSize::Small => (20.0, 11.0, 14.0, 8.0, 5.0),
        ControlSize::Regular => (24.0, 13.0, 16.0, 12.0, 6.0),
        ControlSize::Large => (32.0, 13.0, 16.0, 14.0, 8.0),
        ControlSize::ExtraLarge => (40.0, 15.0, 19.0, 16.0, 10.0),
    };
    SizeMetrics {
        height,
        font,
        line,
        padding,
        radius,
    }
}

fn system_red(dark: bool) -> Color {
    if dark {
        srgb(255, 66, 69)
    } else {
        srgb(255, 56, 60)
    }
}

#[derive(Clone, PartialEq)]
struct ButtonBackground(Color, Dp);

/// A divider just past the trailing edge; the group clips the last one.
#[derive(Clone, PartialEq)]
struct TrailingDivider(Color);

impl DrawModifierNode for TrailingDivider {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let mut metadata = context.render_input.metadata_mut();
        if let Some(size) = metadata.computed_data() {
            let width = Dp::from_pixels_f32(size.width.to_f32());
            let line = Dp::from_pixels_f32(1.0);
            metadata
                .fragment_mut()
                .push_draw_command(PerceptualFillCommand {
                    outline_width: Dp::ZERO,
                    snap_to_pixels: true,
                    path: None,
                    color: self.0,
                    radii: [Dp::ZERO; 2],
                    insets: [Dp(5.0), Dp(-line.0), Dp(5.0), width],
                    clip_radius: None,
                    line: None,
                    edge_smoothness: 1.0,
                    stroke: None,
                });
        }
        content.draw(context.render_input);
    }
}

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
                radii: [self.1; 2],
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
/// - `style` — push button, title-only, link, accessory bar, or glass appearance; defaults to
///   [`ButtonStyle::Automatic`].
/// - `role` — cancel or destructive purpose; destructive buttons are red.
/// - `control_size` — height, font, and padding; defaults to regular.
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
    style: Option<ButtonStyle>,
    role: Option<ButtonRole>,
    control_size: Option<ControlSize>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    interaction_state: Option<State<InteractionState>>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    if use_context::<MenuContext>().is_some() {
        menu_item(label)
            .on_click_optional(on_click)
            .enabled_optional(enabled)
            .accessibility_label_optional(accessibility_label)
            .accessibility_description_optional(accessibility_description);
        return;
    }
    push_button(label)
        .on_click_optional(on_click)
        .enabled_optional(enabled)
        .default_action_optional(default_action)
        .scope_optional(scope)
        .style_optional(style)
        .role_optional(role)
        .control_size_optional(control_size)
        .window_active_optional(window_active)
        .modifier_optional(modifier)
        .interaction_state_optional(interaction_state)
        .focus_requester_optional(focus_requester)
        .accessibility_label_optional(accessibility_label)
        .accessibility_description_optional(accessibility_description);
}

/// The push button behind [`button`] and the button style of
/// [`toggle`](super::toggle::toggle). `toggled` publishes an on/off state and highlights the
/// button while on.
#[tessera]
pub(super) fn push_button(
    #[prop(into)] label: String,
    on_click: Option<Callback>,
    enabled: Option<bool>,
    default_action: Option<bool>,
    toggled: Option<bool>,
    indicator: Option<ButtonIndicator>,
    expanded: Option<bool>,
    scope: Option<ButtonScope>,
    style: Option<ButtonStyle>,
    role: Option<ButtonRole>,
    control_size: Option<ControlSize>,
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
    let ring = remember(|| Eased::new(0.0, FOCUS_RING));
    launch(focused, focus_ring(ring, focused));
    let ring = if focused {
        ring.with(Eased::value)
    } else {
        0.0
    };
    let style = style.unwrap_or_default();
    let metrics = size_metrics(control_size.unwrap_or_default());
    let destructive = role == Some(ButtonRole::Destructive);
    let hovered = interaction.with(InteractionState::is_hovered);
    let highlighted = default_action
        || toggled == Some(true)
        || matches!(
            style,
            ButtonStyle::BorderedProminent | ButtonStyle::GlassProminent
        );
    let accent = if destructive {
        system_red(dark)
    } else {
        Color::from_rgb(0.0, 0.19461783, 1.0)
    };
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let bordered = matches!(
        style,
        ButtonStyle::Automatic | ButtonStyle::Bordered | ButtonStyle::BorderedProminent
    ) || highlighted;
    let fill = if !enabled && toggled == Some(true) && active {
        if dark {
            srgb(15, 98, 187)
        } else {
            srgb(72, 154, 244)
        }
    } else if enabled && highlighted && active {
        if pressed {
            mix(accent, Color::BLACK, 0.3)
        } else {
            accent
        }
    } else if bordered {
        tint.with_alpha(match (dark, enabled, pressed && active) {
            (false, false, _) => 0.04,
            (false, true, true) => 0.16,
            (false, true, false) => 0.075,
            (true, false, _) => 0.035,
            (true, true, true) => 0.16,
            (true, true, false) => 0.08,
        })
    } else if matches!(
        style,
        ButtonStyle::AccessoryBar | ButtonStyle::AccessoryBarAction
    ) && enabled
    {
        tint.with_alpha(if pressed {
            0.12
        } else if hovered {
            0.06
        } else {
            0.0
        })
    } else {
        Color::TRANSPARENT
    };
    let on = toggled == Some(true) && active;
    let filled = enabled && highlighted && active;
    let ink = if filled || on {
        Color::WHITE.with_alpha(if filled { 1.0 } else { 0.5 })
    } else if !enabled {
        tint.with_alpha(if dark { 0.25 } else { 0.26 })
    } else if style == ButtonStyle::Link {
        link_color(theme.appearance)
    } else if destructive {
        system_red(dark)
    } else if dark {
        Color::WHITE.with_alpha(0.85)
    } else {
        Color::BLACK.with_alpha(0.85)
    };
    let ink = if pressed
        && matches!(
            style,
            ButtonStyle::Plain | ButtonStyle::Borderless | ButtonStyle::Link
        ) {
        ink.with_alpha(ink.a * 0.5)
    } else {
        ink
    };
    let grouped = use_context::<ControlGroupContext>().is_some();
    let fill = if grouped {
        tint.with_alpha(if pressed && enabled { 0.08 } else { 0.0 })
    } else {
        fill
    };
    let capsule = matches!(style, ButtonStyle::Glass | ButtonStyle::GlassProminent);
    let radius = Dp(if capsule {
        metrics.height * 0.5
    } else {
        metrics.radius
    });
    let bare = matches!(style, ButtonStyle::Plain | ButtonStyle::Link);
    let glass_surface = style == ButtonStyle::Glass && !filled;
    let shape = button_shape(radius);
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
        .hover_cursor_icon(if style == ButtonStyle::Link {
            CursorIcon::Pointer
        } else {
            CursorIcon::Default
        });
    if !bare {
        modifier = modifier.default_min_size(Dp(metrics.height + 4.0), Dp(metrics.height));
    }
    if ring > 0.0 {
        modifier = modifier.border_with_shape(Dp(2.0), focus_ring_color(ring), shape);
    }
    modifier = modifier.push_semantics(ButtonToggleSemantics(toggled));
    if let Some(expanded) = expanded {
        modifier = modifier.push_semantics(MenuButtonSemantics(expanded));
    }
    if grouped {
        modifier = modifier
            .push_draw(ButtonBackground(fill, Dp::ZERO))
            .push_draw(TrailingDivider(tint.with_alpha(0.1)));
    } else if !glass_surface {
        modifier = modifier.push_draw(ButtonBackground(fill, radius));
    }
    let insets = if bare {
        Padding::default()
    } else {
        Padding::symmetric(
            Dp(metrics.padding),
            Dp(((metrics.height - metrics.line) * 0.5).max(0.0)),
        )
    };
    let title = ButtonTitle {
        label,
        indicator,
        ink,
        font: Dp(metrics.font),
        line: Dp(metrics.line),
    };
    if glass_surface {
        boxed()
            .alignment(Alignment::Center)
            .modifier(modifier)
            .children(move || glass_title(title.clone(), radius, insets, dark));
    } else {
        boxed()
            .alignment(Alignment::Center)
            .modifier(modifier.padding(insets))
            .children(move || button_title(title.clone()));
    }
}

#[derive(Clone, PartialEq)]
struct ButtonTitle {
    label: String,
    indicator: Option<ButtonIndicator>,
    ink: Color,
    font: Dp,
    line: Dp,
}

#[tessera]
fn button_title(title: ButtonTitle) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            title_text(title.clone());
            if let Some(indicator) = title.indicator {
                spacer().modifier(Modifier::new().width(Dp(6.0)));
                indicator_glyph(indicator, title.ink);
            }
        });
}

#[tessera]
fn title_text(title: ButtonTitle) {
    text(title.label)
        .style(TextStyle {
            font_size: title.font,
            line_height: title.line,
            ..Default::default()
        })
        .color(title.ink)
        .single_line(true)
        .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        }));
}

#[tessera]
fn glass_title(title: ButtonTitle, radius: Dp, insets: Padding, dark: bool) {
    glass()
        .material(GlassMaterial::Regular)
        .appearance(if dark {
            GlassAppearance::Dark
        } else {
            GlassAppearance::Light
        })
        .corner_radius(radius)
        .padding(Dp::ZERO)
        .block_input(false)
        .child(move || {
            let title = title.clone();
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().padding(insets))
                .children(move || button_title(title.clone()));
        });
}
