use std::time::Duration;

use tessera_foundation::{
    modifier::{ClickableArgs, InteractionState, ModifierExt as _},
    pointer_gestures::TapGestureConfig,
};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, CallbackWith, Color, Dp,
    DrawModifierContent, DrawModifierContext, DrawModifierNode, FocusRequester, Modifier,
    SemanticsModifierNode,
    accesskit::{Role, Toggled},
    launch,
    layout::{Stack, layout},
    modifier::{CursorModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera,
    time::Instant,
    use_context,
    winit::window::CursorIcon,
};

use crate::{
    alignment::Alignment,
    animation::{Animatable, FrameDelta, Motion, animate_to, cubic_bezier_easing},
    boxed::boxed,
    modifier::ModifierExt as _,
    pipelines::{
        liquid_glass::{LiquidGlassElement, lens::LiquidLensCommand},
        perceptual_fill::PerceptualFillCommand,
        shape::command::ShapeCommand,
    },
    shape_def::{ResolvedShape, RoundedCorner, Shape},
};

use super::{
    choice::ChoiceKeyboard,
    motion::mix,
    theme::{Appearance, HigTheme},
};

#[cfg(test)]
#[path = "switch_tests.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq)]
struct Action {
    value: bool,
    mixed: bool,
    enabled: bool,
    callback: CallbackWith<bool>,
}

struct SwitchSemantics {
    value: bool,
    mixed: bool,
}

impl SemanticsModifierNode for SwitchSemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.toggled = Some(if self.mixed {
            Toggled::Mixed
        } else if self.value {
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

/// Time for a pressed thumb to lift into a lens.
const LIFT: Duration = Duration::from_millis(200);
/// Angular frequency and damping ratio of the thumb's release and travel.
const RELEASE: (f32, f32) = (13.49, 0.71);

fn release_spring(value: f32) -> Animatable {
    Animatable::spring(value, RELEASE.0, RELEASE.1)
}

#[derive(Clone, Copy, PartialEq)]
struct Lifting {
    start: Instant,
    from: f32,
}

impl Lifting {
    fn at(self, now: Instant) -> f32 {
        let progress = now.saturating_duration_since(self.start).as_secs_f32() / LIFT.as_secs_f32();
        self.from + (1.0 - self.from) * cubic_bezier_easing(progress, 0.42, 0.0, 0.58, 1.0)
    }

    fn done(self, now: Instant) -> bool {
        now.saturating_duration_since(self.start) >= LIFT
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ThumbTarget {
    lifted: bool,
    x: f32,
}

/// The thumb's lift (0 at rest, 1 as a lens) and leading edge in points.
#[derive(Clone, Copy, PartialEq)]
struct ThumbMotion {
    lift: Animatable,
    lifting: Option<Lifting>,
    x: Animatable,
    delta: FrameDelta,
}

impl ThumbMotion {
    fn new(x: f32) -> Self {
        Self {
            lift: release_spring(0.0),
            lifting: None,
            x: release_spring(x),
            delta: FrameDelta::default(),
        }
    }

    fn lift(&self) -> f32 {
        self.lift.value().clamp(0.0, 1.0)
    }
}

impl Motion for ThumbMotion {
    type Target = ThumbTarget;

    fn is_settled_at(&self, target: &ThumbTarget) -> bool {
        self.lifting.is_none()
            && self.lift.target() == if target.lifted { 1.0 } else { 0.0 }
            && self.lift.is_idle()
            && self.x.target() == target.x
            && self.x.is_idle()
    }

    fn retarget(&mut self, target: ThumbTarget, now: Instant) {
        self.advance(now);
        if target.lifted {
            if self.lifting.is_none() && self.lift.target() != 1.0 {
                self.lifting = Some(Lifting {
                    start: now,
                    from: self.lift(),
                });
            }
        } else if let Some(lifting) = self.lifting.take() {
            // An interrupted lift releases from its current height and speed.
            let value = lifting.at(now);
            let velocity = (lifting.at(now + Duration::from_millis(1)) - value) * 1000.0;
            self.lift = release_spring(value).with_velocity(velocity);
            self.lift.set_target(0.0);
        } else {
            self.lift.set_target(0.0);
        }
        self.x.set_target(target.x);
    }

    fn advance(&mut self, now: Instant) -> bool {
        let dt = self.delta.step(now);
        if let Some(lifting) = self.lifting {
            if lifting.done(now) {
                self.lifting = None;
                self.lift = release_spring(1.0);
            } else {
                self.lift = release_spring(lifting.at(now));
                self.lift.set_target(1.0);
            }
        } else {
            self.lift.tick(dt);
        }
        self.x.tick(dt);
        self.lifting.is_some() || !self.lift.is_idle() || !self.x.is_idle()
    }
}

#[derive(Clone, PartialEq)]
struct ThumbShadow(f32);

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
                color: Color::BLACK.with_alpha(0.28 * self.0),
                shape: ResolvedShape::Rounded {
                    corner_radii: [Dp(10.0).to_pixels_f32(); 4],
                    corner_g2: [2.0; 4],
                },
                smoothness: Dp(23.25).to_pixels_f32(),
            });
        content.draw(context.render_input);
    }
}

/// Width, height, and top-left corner of the thumb at leading edge `x` and lift `lift`; it
/// grows about its centre from a 32 x 20 capsule into a 50 x 31 lens.
fn thumb_frame(x: f32, lift: f32) -> (Dp, Dp, Dp, Dp) {
    let width = 32.0 + 18.0 * lift;
    let height = 20.0 + 11.0 * lift;
    (
        Dp(f64::from(width)),
        Dp(f64::from(height)),
        Dp(f64::from(x + 16.0 - width * 0.5)),
        Dp(f64::from(12.0 - height * 0.5)),
    )
}

#[tessera]
fn switch_shadow(x: f32, lift: f32) {
    let (width, height, left, top) = thumb_frame(x, lift);
    layout(Stack).modifier(
        Modifier::new()
            .offset(left, Dp(top.0 + 2.0))
            .required_size(width, height)
            .push_draw(ThumbShadow(1.0 - lift)),
    );
}

#[tessera]
fn switch_thumb(x: f32, lift: f32) {
    let (width, height, left, top) = thumb_frame(x, lift);
    let modifier = Modifier::new()
        .offset(left, top)
        .required_size(width, height);
    let modifier = if lift > 0.0 {
        modifier.push_draw(ThumbLens {
            width: width.0 as f32,
            height: height.0 as f32,
            opacity: lift,
        })
    } else {
        modifier
    };
    layout(Stack).modifier(if lift < 1.0 {
        modifier.push_draw(Fill {
            color: Color::WHITE.with_alpha(1.0 - lift),
            radius: Dp(height.0 * 0.5),
        })
    } else {
        modifier
    });
}

#[derive(Clone, PartialEq)]
struct ThumbLens {
    width: f32,
    height: f32,
    opacity: f32,
}

impl DrawModifierNode for ThumbLens {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(LiquidLensCommand {
                element: LiquidGlassElement {
                    x: 0.0,
                    y: 0.0,
                    width: self.width,
                    height: self.height,
                    radius: self.height * 0.5,
                },
                pixels_per_point: Dp(1.0).to_pixels_f32(),
                opacity: self.opacity,
            });
        content.draw(context.render_input);
    }
}

/// The mixed-state mark: a 7 x 2 capsule on the leading half of the track.
#[tessera]
fn switch_dash() {
    layout(Stack).modifier(
        Modifier::new()
            .offset(Dp(8.5), Dp(11.0))
            .size(Dp(7.0), Dp(2.0))
            .push_draw(Fill {
                color: Color::WHITE,
                radius: Dp(1.0),
            }),
    );
}

#[tessera]
fn switch_graphic(mixed: bool, track: Color, x: f32, lift: f32, focused: bool) {
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
            if mixed {
                switch_dash();
            }
            if lift < 1.0 {
                switch_shadow(x, lift);
            }
            switch_thumb(x, lift);
        });
}

/// The switch style of [`toggle`](super::toggle::toggle). A mixed switch shows its thumb on
/// the trailing side with a dash on the track; activating it turns it on.
#[tessera]
pub(super) fn switch(
    value: bool,
    mixed: Option<bool>,
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
    let mixed = mixed.unwrap_or(false);
    let value = value || mixed;
    let next = Action {
        value,
        mixed,
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
            action.callback.call(action.mixed || !action.value);
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
    let pressed = enabled && active && (held.get() || interaction.with(|state| state.is_pressed()));
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let rest_x = if value { 20.0 } else { 2.0 };
    let motion = remember(|| ThumbMotion::new(rest_x));
    launch(
        (pressed, value),
        animate_to(
            motion,
            ThumbTarget {
                lifted: pressed,
                x: rest_x,
            },
        ),
    );
    let (x, lift) = motion.with(|motion| (motion.x.value(), motion.lift()));
    // The track follows the thumb between its off and on colors, and the lift between the
    // resting and pressed colors.
    let colors = |on: bool| {
        mix(
            track_color(on, false, enabled, active, dark),
            track_color(on, true, enabled, active, dark),
            lift,
        )
    };
    let track = mix(
        colors(false),
        colors(true),
        ((x - 2.0) / 18.0).clamp(0.0, 1.0),
    );
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
        .push_semantics(SwitchSemantics { value, mixed })
        .hover_cursor_icon(CursorIcon::Default);
    boxed()
        .modifier(modifier)
        .children(move || switch_graphic(mixed, track, x, lift, focused));
}

fn track_color(value: bool, pressed: bool, enabled: bool, active: bool, dark: bool) -> Color {
    let pressed = pressed && enabled && active;
    if value && active {
        if pressed {
            if dark {
                Color::from_rgb(0.012983031, 0.2541521, 1.0)
            } else {
                Color::from_rgb(0.0, 0.14412847, 0.7379104)
            }
        } else {
            Color::from_rgb(0.0, 0.19461783, 1.0).with_alpha(if enabled { 1.0 } else { 0.7 })
        }
    } else {
        let tint = if dark { Color::WHITE } else { Color::BLACK };
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
    }
}
