//! Switches for immediately applied settings.
//!
//! ## Usage
//!
//! Turn an application setting on or off without a separate confirmation action.

use tessera_foundation::{
    modifier::{ClickableArgs, InteractionState, ModifierExt as _},
    pointer_gestures::TapGestureConfig,
};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, CallbackWith, Color, Dp,
    DrawModifierContent, DrawModifierContext, DrawModifierNode, FocusRequester, Modifier,
    SemanticsModifierNode,
    accesskit::{Role, Toggled},
    layout::{Stack, layout},
    modifier::{CursorModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::window::CursorIcon,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    modifier::ModifierExt as _,
    pipelines::{
        fluid_glass::{
            FluidGlassCommand, FluidGlassRenderArgs, LensContour, LensOptics, LensShadow,
        },
        perceptual_fill::PerceptualFillCommand,
        shape::command::ShapeCommand,
    },
    shape_def::{ResolvedShape, RoundedCorner, Shape},
};

use super::{
    choice::ChoiceKeyboard,
    theme::{Appearance, HigTheme},
};

#[cfg(test)]
#[path = "switch_tests.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq)]
struct Action {
    value: bool,
    enabled: bool,
    callback: CallbackWith<bool>,
}

struct SwitchSemantics(bool);

impl SemanticsModifierNode for SwitchSemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.toggled = Some(if self.0 {
            Toggled::True
        } else {
            Toggled::False
        });
    }
}

#[derive(Clone, PartialEq)]
struct Fill {
    color: Color,
    radius: Dp,
}

impl DrawModifierNode for Fill {
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
                color: self.color,
                radii: [self.radius; 2],
                insets: [Dp(0.0); 4],
                clip_radius: None,
                line: None,
                edge_smoothness: 1.0,
                stroke: None,
            });
        content.draw(context.render_input);
    }
}

struct ThumbShadow;

impl DrawModifierNode for ThumbShadow {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(ShapeCommand::Shadow {
                color: Color::BLACK.with_alpha(0.28),
                shape: ResolvedShape::Rounded {
                    corner_radii: [Dp(10.0).to_pixels_f32(); 4],
                    corner_g2: [2.0; 4],
                },
                smoothness: Dp(23.25).to_pixels_f32(),
            });
        content.draw(context.render_input);
    }
}

#[tessera]
fn switch_shadow(x: Dp) {
    layout(Stack).modifier(
        Modifier::new()
            .offset(x, Dp(4.0))
            .size(Dp(32.0), Dp(20.0))
            .push_draw(ThumbShadow),
    );
}

#[tessera]
fn switch_thumb(x: Dp, pressed: bool) {
    if pressed {
        layout(Stack).modifier(
            Modifier::new()
                .offset(Dp(x.0 - 9.0), Dp(-3.5))
                .required_size(Dp(50.0), Dp(31.0))
                .push_draw(ThumbLens),
        );
        return;
    }
    layout(Stack).modifier(
        Modifier::new()
            .offset(x, Dp(2.0))
            .size(Dp(32.0), Dp(20.0))
            .push_draw(Fill {
                color: Color::WHITE,
                radius: Dp(10.0),
            }),
    );
}

struct ThumbLens;

impl DrawModifierNode for ThumbLens {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let corner = RoundedCorner::manual(Dp(15.5), 2.03);
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(FluidGlassCommand {
                render: FluidGlassRenderArgs {
                    shape: Shape::RoundedRectangle {
                        top_left: corner,
                        top_right: corner,
                        bottom_right: corner,
                        bottom_left: corner,
                    },
                    pixels_per_dp: Dp(1.0).to_pixels_f32(),
                    lens: Some(LensOptics {
                        contour: Some(LensContour {
                            controls: [
                                [0.0, 22.501842 / 31.0],
                                [3.697153 / 50.0, 12.862425 / 31.0],
                                [12.134089 / 50.0, 6.410123 / 31.0],
                                [20.571026 / 50.0, -0.042180 / 31.0],
                                [29.207634 / 50.0, 0.217691 / 31.0],
                            ],
                            coverage_width: 0.994163,
                        }),
                        edge_width: 6.2,
                        refraction_width: 31.5,
                        refraction_corner_g2: 2.0,
                        inner_refraction_width: 3.1,
                        inner_refraction: -4.65,
                        refraction: 7.75,
                        dispersion_angle: -std::f32::consts::PI / 12.0,
                        shadow: LensShadow {
                            offset: 8.0,
                            width: 4.0,
                            blur: 3.0,
                            opacity: 0.1,
                        },
                        dispersion: 0.4707,
                        transmission: 1.0,
                    }),
                    ..Default::default()
                },
            });
        content.draw(context.render_input);
    }
}

#[tessera]
fn switch_graphic(value: bool, track: Color, pressed: bool, focused: bool) {
    let mut modifier = Modifier::new().size(Dp(54.0), Dp(24.0)).push_draw(Fill {
        color: track,
        radius: Dp(12.0),
    });
    if focused {
        let corner = RoundedCorner::manual(Dp(12.0), 2.0);
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
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(modifier)
        .children(move || {
            let x = Dp(if value { 20.0 } else { 2.0 });
            if !pressed {
                switch_shadow(x);
            }
            switch_thumb(x, pressed);
        });
}

/// # switch
///
/// Requests changes to an immediately applied on/off setting with pointer or keyboard input.
///
/// ## Usage
///
/// Enable notifications, background activity, or other independent application settings.
///
/// ## Parameters
///
/// - `value`, `on_value_change` — controlled setting and requested updates.
/// - `enabled` — input and focus availability; defaults to true.
/// - `window_active` — containing window's active appearance; defaults to true.
/// - `modifier`, `focus_requester` — layout, semantics, and optional external focus handle.
/// - `accessibility_label`, `accessibility_description` — spoken name and instructions.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::switch::switch;
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
/// use tessera_ui::{Modifier, remember, tessera, testing::UiTest};
/// #[tessera]
/// fn app() {
///     let enabled = remember(|| false);
///     switch(enabled.get()).accessibility_label("Notifications")
///         .on_value_change(move |value| enabled.set(value))
///         .modifier(Modifier::new().semantics(SemanticsArgs {
///             test_tag: Some("notifications".into()), ..Default::default()
///         }));
/// }
/// let mut ui = UiTest::new().content(app);
/// ui.frame();
/// ui.node("notifications").click();
/// assert_eq!(ui.accessibility_node("Notifications").1.toggled(), Some(tessera_ui::accesskit::Toggled::True));
/// ```
#[tessera]
pub fn switch(
    value: bool,
    on_value_change: Option<CallbackWith<bool>>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let active = window_active.unwrap_or(true);
    let next = Action {
        value,
        enabled,
        callback: on_value_change.unwrap_or_else(CallbackWith::default_value),
    };
    let action = remember(|| next);
    if action.get() != next {
        action.set(next);
    }
    let activate = Callback::new(move || {
        let action = action.get();
        if action.enabled {
            action.callback.call(!action.value);
        }
    });
    let requester = if let Some(requester) = focus_requester {
        requester
    } else {
        remember(FocusRequester::new).get()
    };
    let interaction = remember(InteractionState::new);
    let held = remember(|| false);
    if !enabled {
        if held.get() {
            held.set(false);
        }
    } else if held.get() && !requester.is_focused() {
        held.set(false);
    }
    let pressed = enabled
        && active
        && (held.get() || interaction.with(|state| state.is_pressed() && state.is_hovered()));
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let track = if value && active {
        if pressed {
            Color::from_rgb(0.0, 0.14412847, 0.7379104)
        } else {
            Color::from_rgb(0.0, 0.19461783, 1.0).with_alpha(if enabled { 1.0 } else { 0.7 })
        }
    } else {
        let tint = if theme.appearance == Appearance::Dark {
            Color::WHITE
        } else {
            Color::BLACK
        };
        tint.with_alpha(match (enabled, value) {
            (true, false) => {
                if pressed {
                    0.20
                } else {
                    0.10
                }
            }
            (true, true) => 0.14,
            (false, false) => 0.05,
            (false, true) => 0.07,
        })
    };
    let focused = enabled && active && requester.is_focus_visible();
    let modifier = modifier
        .unwrap_or_default()
        .clickable(ClickableArgs {
            enabled,
            on_click: activate,
            role: Some(Role::Switch),
            label: accessibility_label,
            description: accessibility_description,
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
        .push_keyboard_input(ChoiceKeyboard {
            enabled,
            held,
            activate,
        })
        .push_semantics(SwitchSemantics(value))
        .hover_cursor_icon(CursorIcon::Default);
    boxed()
        .modifier(modifier)
        .children(move || switch_graphic(value, track, pressed, focused));
}
