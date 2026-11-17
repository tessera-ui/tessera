//! Steppers for bounded numeric settings.
//!
//! ## Usage
//!
//! Increase or decrease a quantity by a fixed increment.

use std::{
    ops::RangeInclusive,
    time::{Duration, Instant},
};

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, CallbackWith, Color,
    ComputedData, Constraint, CursorEventContent, Dp, DrawModifierContent, DrawModifierContext,
    DrawModifierNode, FocusRequester, KeyboardInput, KeyboardInputModifierNode, LayoutPolicy,
    LayoutResult, MeasurementError, Modifier, PointerEventPass, PointerId, PointerInput,
    PointerInputModifierNode, PressKeyEventType, Px, PxPosition, SemanticsModifierNode, State,
    accesskit::{Action, Role},
    launch,
    layout::{MeasureScope, Stack, layout},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
        window::CursorIcon,
    },
    with_frame_time,
};

use crate::{
    modifier::ModifierExt as _,
    pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualStroke},
    shape_def::{RoundedCorner, Shape},
};

use super::theme::{Appearance, HigTheme};

#[cfg(test)]
#[path = "stepper_tests.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq)]
struct Value {
    current: f64,
    min: f64,
    max: f64,
    increment: f64,
    wraps: bool,
    enabled: bool,
    callback: CallbackWith<f64>,
}

impl Value {
    fn next(self, up: bool) -> f64 {
        let next = self.current + if up { self.increment } else { -self.increment };
        if self.wraps && next > self.max {
            self.min
        } else if self.wraps && next < self.min {
            self.max
        } else {
            next.clamp(self.min, self.max)
        }
    }

    fn available(self, up: bool) -> bool {
        self.enabled && self.next(up) != self.current
    }

    fn step(self, up: bool) {
        if self.available(up) {
            self.callback.call(self.next(up));
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Held {
    pointer: PointerId,
    up: bool,
    inside: bool,
    start: Instant,
}

struct StepperPointer {
    value: State<Value>,
    held: State<Option<Held>>,
    focus: FocusRequester,
}

impl PointerInputModifierNode for StepperPointer {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main || !self.value.get().enabled {
            return;
        }
        for change in input.pointer_changes.iter_mut() {
            let point = change
                .position()
                .map(|point| input.local_to_window.inverse_point(point));
            let width = input.computed_data.width.to_f32();
            let height = input.computed_data.height.to_f32();
            let half = point
                .filter(|point| {
                    point.x >= 0.0 && point.x < width && point.y >= 0.0 && point.y < height
                })
                .map(|point| point.y < height * 0.5);
            if matches!(
                change.content,
                CursorEventContent::Pressed(PressKeyEventType::Left)
            ) && self.held.get().is_none()
                && !change.is_consumed()
            {
                let Some(up) = half else {
                    continue;
                };
                if !self.value.get().available(up) || !change.claim_tap_press() {
                    continue;
                }
                self.focus.request_focus();
                self.held.set(Some(Held {
                    pointer: change.pointer_id,
                    up,
                    inside: true,
                    start: change.timestamp,
                }));
                self.value.get().step(up);
                change.consume();
                continue;
            }
            let Some(mut held) = self
                .held
                .get()
                .filter(|held| held.pointer == change.pointer_id)
            else {
                continue;
            };
            if change.is_consumed()
                || matches!(
                    change.content,
                    CursorEventContent::Cancelled(PressKeyEventType::Left)
                )
            {
                self.held.set(None);
            } else if matches!(
                change.content,
                CursorEventContent::Released(PressKeyEventType::Left)
            ) {
                self.held.set(None);
                change.consume();
            } else if matches!(change.content, CursorEventContent::Moved(_)) {
                let inside = half == Some(held.up);
                if held.inside != inside {
                    held.inside = inside;
                    self.held.set(Some(held));
                }
                change.consume();
            }
        }
    }
}

struct StepperKeyboard(State<Value>);

impl KeyboardInputModifierNode for StepperKeyboard {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        if !self.0.get().enabled
            || input.key_modifiers.control_key()
            || input.key_modifiers.alt_key()
            || input.key_modifiers.super_key()
        {
            return;
        }
        input.keyboard_events.retain(|event| {
            let up = match event.logical_key {
                Key::Named(NamedKey::ArrowUp) => true,
                Key::Named(NamedKey::ArrowDown) => false,
                _ => return true,
            };
            if event.state == ElementState::Pressed {
                self.0.get().step(up);
            }
            false
        });
    }
}

struct StepperSemantics {
    value: State<Value>,
    label: Option<String>,
    description: Option<String>,
}

impl SemanticsModifierNode for StepperSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        handler: &mut Option<AccessibilityActionHandler>,
    ) {
        let value = self.value.get();
        node.role = Some(Role::SpinButton);
        node.label = self.label.clone();
        node.description = self.description.clone();
        node.numeric_value = Some(value.current);
        node.min_numeric_value = Some(value.min);
        node.max_numeric_value = Some(value.max);
        node.numeric_value_step = Some(value.increment);
        node.disabled = !value.enabled;
        node.focusable = value.enabled;
        node.merge_descendants = false;
        node.actions.clear();
        if value.available(true) {
            node.actions.push(Action::Increment);
        }
        if value.available(false) {
            node.actions.push(Action::Decrement);
        }
        let state = self.value;
        *handler = value.enabled.then(|| {
            Box::new(move |action| match action {
                Action::Increment => state.get().step(true),
                Action::Decrement => state.get().step(false),
                _ => {}
            }) as AccessibilityActionHandler
        });
    }
}

#[derive(Clone, PartialEq)]
struct HalfFill {
    fill: Color,
    ink: Color,
    up: bool,
}

impl DrawModifierNode for HalfFill {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let points = if self.up {
            [[0.27, 0.7166667], [0.51, 0.3083333], [0.7525, 0.7166667]]
        } else {
            [[0.27, 0.3125], [0.51, 0.725], [0.7525, 0.3125]]
        };
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                outline_width: Dp::ZERO,
                snap_to_pixels: false,
                path: None,
                color: self.fill,
                radii: [Dp(0.0); 2],
                insets: [Dp(0.0); 4],
                clip_radius: None,
                line: None,
                edge_smoothness: 1.0,
                stroke: Some(PerceptualStroke {
                    raster_mask: false,
                    color: self.ink,
                    width: Dp(1.7),
                    points,
                }),
            });
        let y = if self.up { 1.0 } else { 0.0 };
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                outline_width: Dp::ZERO,
                snap_to_pixels: false,
                path: None,
                color: Color::TRANSPARENT,
                radii: [Dp(0.0); 2],
                insets: [Dp(0.0); 4],
                clip_radius: None,
                line: None,
                edge_smoothness: 1.0,
                stroke: Some(PerceptualStroke {
                    raster_mask: false,
                    color: self.ink.with_alpha(1.0 / 7.0),
                    width: Dp(1.0),
                    points: [[0.175, y], [0.5, y], [0.825, y]],
                }),
            });
        content.draw(context.render_input);
    }
}

#[derive(Clone, Copy, PartialEq)]
struct StepperLayout;

impl LayoutPolicy for StepperLayout {
    fn measure_eq(&self, _: &Self) -> bool {
        true
    }
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.clamp(Dp(20.0).into());
        let height = parent.height.clamp(Dp(24.0).into());
        let half = Px(height.0 / 2);
        let mut result = LayoutResult::new(ComputedData { width, height });
        for (index, child) in scope.children().iter().enumerate() {
            let h = if index == 0 { half } else { height - half };
            child.measure(
                scope,
                &Constraint::new(AxisConstraint::exact(width), AxisConstraint::exact(h)),
            )?;
            result.place_child(
                *child,
                PxPosition::new(Px::ZERO, if index == 0 { Px::ZERO } else { half }),
            );
        }
        Ok(result)
    }
}

#[tessera]
fn stepper_half(up: bool, enabled: bool, pressed: bool, dark: bool) {
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    layout(Stack).modifier(Modifier::new().push_draw(HalfFill {
        fill: tint.with_alpha(if pressed {
            0.18
        } else if enabled {
            0.075
        } else {
            0.04
        }),
        ink: tint.with_alpha(if dark { 0.55 } else { 0.5 }),
        up,
    }));
}

/// # stepper
///
/// Requests fixed numeric increments for quantities and bounded settings.
///
/// ## Usage
///
/// Adjust a quantity with up/down buttons or directional keys.
///
/// ## Parameters
///
/// - `value`, `on_value_change` — controlled value and requested updates.
/// - `range`, `increment` — finite increasing limits and positive step; default to `0.0..=59.0` and `1.0`.
/// - `wraps` — requests the opposite endpoint on overflow; defaults to true.
/// - `autorepeat` — repeats held pointer input after a delay; defaults to true.
/// - `enabled`, `window_active` — input availability and active appearance; default to true.
/// - `modifier`, `focus_requester` — layout, semantics, and optional external focus handle.
/// - `accessibility_label`, `accessibility_description` — spoken name and instructions.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::stepper::stepper;
/// use tessera_ui::{remember, tessera, testing::UiTest, accesskit::Action};
/// #[tessera]
/// fn app() {
///     let quantity = remember(|| 0.0);
///     stepper(quantity.get()).accessibility_label("Quantity")
///         .on_value_change(move |value| quantity.set(value));
/// }
/// let mut ui = UiTest::new().content(app);
/// ui.frame();
/// let (id, _) = ui.accessibility_node("Quantity");
/// assert!(ui.accessibility_action(id, Action::Increment));
/// ui.frame();
/// assert_eq!(ui.accessibility_node("Quantity").1.numeric_value(), Some(1.0));
/// assert!(ui.accessibility_action(id, Action::Decrement));
/// ui.frame();
/// assert!(ui.accessibility_action(id, Action::Decrement));
/// ui.frame();
/// assert_eq!(ui.accessibility_node("Quantity").1.numeric_value(), Some(59.0));
/// ```
#[tessera]
pub fn stepper(
    value: f64,
    range: Option<RangeInclusive<f64>>,
    increment: Option<f64>,
    wraps: Option<bool>,
    autorepeat: Option<bool>,
    on_value_change: Option<CallbackWith<f64>>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let range = range.unwrap_or(0.0..=59.0);
    let (min, max) = (*range.start(), *range.end());
    let increment = increment.unwrap_or(1.0);
    assert!(
        min.is_finite() && max.is_finite() && min < max,
        "stepper range must be finite and increasing"
    );
    assert!(
        increment.is_finite() && increment > 0.0 && value.is_finite(),
        "stepper value must be finite and increment must be positive and finite"
    );
    let enabled = enabled.unwrap_or(true);
    let active = window_active.unwrap_or(true);
    let repeat = autorepeat.unwrap_or(true);
    let next = Value {
        current: value.clamp(min, max),
        min,
        max,
        increment,
        wraps: wraps.unwrap_or(true),
        enabled,
        callback: on_value_change.unwrap_or_else(CallbackWith::default_value),
    };
    let state = remember(|| next);
    if state.get() != next {
        state.set(next);
    }
    let held = remember(|| None::<Held>);
    if !enabled && held.get().is_some() {
        held.set(None);
    }
    let requester = if let Some(requester) = focus_requester {
        requester
    } else {
        remember(FocusRequester::new).get()
    };
    let press = held.get();
    launch((press, repeat), async move {
        let Some(press) = press.filter(|press| press.inside && repeat) else {
            return;
        };
        let mut deadline = press.start + Duration::from_millis(500);
        loop {
            let running = with_frame_time(|now| {
                if held.get() != Some(press) || !state.get().enabled {
                    return false;
                }
                if now >= deadline {
                    state.get().step(press.up);
                    deadline = now + Duration::from_millis(100);
                }
                state.get().available(press.up)
            })
            .await;
            if !running {
                break;
            }
        }
    });
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let corner = RoundedCorner::manual(Dp(6.0), 2.0);
    let shape = Shape::RoundedRectangle {
        top_left: corner,
        top_right: corner,
        bottom_right: corner,
        bottom_left: corner,
    };
    let mut modifier = modifier
        .unwrap_or_default()
        .push_pointer_input(StepperPointer {
            value: state,
            held,
            focus: requester,
        })
        .push_keyboard_input(StepperKeyboard(state))
        .push_semantics(StepperSemantics {
            value: state,
            label: accessibility_label,
            description: accessibility_description,
        })
        .hover_cursor_icon(CursorIcon::Default);
    if enabled {
        modifier = modifier
            .focusable()
            .focus_requester(requester)
            .on_focus_changed(move |focus: tessera_ui::FocusState| {
                if !focus.is_focused() && held.get().is_some() {
                    held.set(None);
                }
            });
    }
    if enabled && active && requester.is_focus_visible() {
        modifier = modifier.border_with_shape(
            Dp(2.0),
            Color::from_rgb(0.0, 0.19461783, 1.0).with_alpha(0.6),
            shape,
        );
    }
    layout(StepperLayout)
        .modifier(modifier.clip_shape(shape))
        .child(move || {
            stepper_half(
                true,
                next.available(true),
                active && press.is_some_and(|press| press.up && press.inside),
                dark,
            );
            stepper_half(
                false,
                next.available(false),
                active && press.is_some_and(|press| !press.up && press.inside),
                dark,
            );
        });
}
