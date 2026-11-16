//! Continuous horizontal sliders.
//!
//! ## Usage
//!
//! Adjust volume, intensity, or another bounded numeric setting.

use std::ops::RangeInclusive;

use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, CallbackWith, Color,
    ComputedData, Constraint, CursorEventContent, Dp, DrawModifierContent, DrawModifierContext,
    DrawModifierNode, FocusRequester, KeyboardInput, KeyboardInputModifierNode, LayoutPolicy,
    LayoutResult, MeasurementError, Modifier, PointerEventPass, PointerId, PointerInput,
    PointerInputModifierNode, PressKeyEventType, Px, PxPosition, SemanticsModifierNode, State,
    accesskit::{Action, Role},
    layout::{MeasureScope, Stack, layout},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
        window::CursorIcon,
    },
};

use crate::{
    modifier::ModifierExt as _,
    pipelines::{perceptual_fill::PerceptualFillCommand, shape::command::ShapeCommand},
    shape_def::{ResolvedShape, RoundedCorner, Shape},
};

use super::theme::{Appearance, HigTheme};

#[cfg(test)]
#[path = "slider_tests.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq)]
struct Value {
    current: f64,
    min: f64,
    max: f64,
    enabled: bool,
    callback: CallbackWith<f64>,
}

impl Value {
    fn fraction(self) -> f64 {
        (self.current - self.min) / (self.max - self.min)
    }

    fn request(self, value: f64) {
        let value = value.clamp(self.min, self.max);
        if self.enabled && value != self.current {
            self.callback.call(value);
        }
    }

    fn adjust(self, direction: f64) {
        self.request(self.current + (self.max - self.min) * 0.05 * direction);
    }
}

#[derive(Clone, Copy)]
struct Drag {
    pointer: PointerId,
    offset: f32,
}

struct SliderInput {
    value: State<Value>,
    drag: State<Option<Drag>>,
    focus: FocusRequester,
}

impl PointerInputModifierNode for SliderInput {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        let value = self.value.get();
        if !value.enabled {
            return;
        }
        let width = input.computed_data.width.to_f32();
        let height = input.computed_data.height.to_f32();
        let thumb = Dp(20.0).to_pixels_f32().min(width);
        let travel = (width - thumb).max(0.0);
        for change in input.pointer_changes.iter_mut() {
            let position = change
                .position()
                .map(|point| input.local_to_window.inverse_point(point));
            let mut drag = self.drag.get();
            if let CursorEventContent::Pressed(PressKeyEventType::Left) = change.content {
                if drag.is_some() || change.is_consumed() {
                    continue;
                }
                let Some(point) = position else {
                    continue;
                };
                if point.x < 0.0 || point.x >= width || point.y < 0.0 || point.y >= height {
                    continue;
                }
                if !change.claim_tap_press() {
                    continue;
                }
                self.focus.request_focus();
                let left = travel * value.fraction() as f32;
                let offset = if point.x >= left && point.x <= left + thumb {
                    point.x - left - thumb * 0.5
                } else {
                    0.0
                };
                drag = Some(Drag {
                    pointer: change.pointer_id,
                    offset,
                });
                self.drag.set(drag);
            }
            let Some(held) = drag.filter(|held| held.pointer == change.pointer_id) else {
                continue;
            };
            if matches!(change.content, CursorEventContent::Cancelled(_)) || change.is_consumed() {
                self.drag.set(None);
                continue;
            }
            if matches!(
                change.content,
                CursorEventContent::Pressed(PressKeyEventType::Left)
                    | CursorEventContent::Moved(_)
                    | CursorEventContent::Released(PressKeyEventType::Left)
            ) {
                if travel > 0.0
                    && let Some(point) = position
                {
                    let fraction =
                        ((point.x - held.offset - thumb * 0.5) / travel).clamp(0.0, 1.0) as f64;
                    value.request(value.min + fraction * (value.max - value.min));
                }
                change.consume();
            }
            if matches!(
                change.content,
                CursorEventContent::Released(PressKeyEventType::Left)
            ) {
                self.drag.set(None);
            }
        }
    }
}

struct SliderKeyboard(State<Value>);

impl KeyboardInputModifierNode for SliderKeyboard {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        let value = self.0.get();
        if !value.enabled
            || input.key_modifiers.control_key()
            || input.key_modifiers.alt_key()
            || input.key_modifiers.super_key()
        {
            return;
        }
        input.keyboard_events.retain(|event| {
            let value = self.0.get();
            let step = (value.max - value.min) * 0.05;
            let requested = match event.logical_key {
                Key::Named(NamedKey::ArrowRight | NamedKey::ArrowUp) => value.current + step,
                Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowDown) => value.current - step,
                Key::Named(NamedKey::PageUp) => value.max,
                Key::Named(NamedKey::PageDown) => value.min,
                _ => return true,
            };
            if event.state == ElementState::Pressed {
                value.request(requested);
            }
            false
        });
    }
}

struct SliderSemantics {
    value: State<Value>,
    label: Option<String>,
    description: Option<String>,
}

impl SemanticsModifierNode for SliderSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        handler: &mut Option<AccessibilityActionHandler>,
    ) {
        let value = self.value.get();
        node.role = Some(Role::Slider);
        node.label = self.label.clone();
        node.description = self.description.clone();
        node.numeric_value = Some(value.current);
        node.min_numeric_value = Some(value.min);
        node.max_numeric_value = Some(value.max);
        node.numeric_value_step = Some((value.max - value.min) * 0.05);
        node.numeric_value_jump = Some(value.max - value.min);
        node.disabled = !value.enabled;
        node.focusable = value.enabled;
        node.merge_descendants = false;
        node.actions.clear();
        if value.enabled {
            node.actions.extend([Action::Increment, Action::Decrement]);
            let state = self.value;
            *handler = Some(Box::new(move |action| match action {
                Action::Increment => state.get().adjust(1.0),
                Action::Decrement => state.get().adjust(-1.0),
                _ => {}
            }));
        } else {
            *handler = None;
        }
    }
}

#[derive(Clone, PartialEq)]
struct Fill(Color, Dp, f32);

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
                color: self.0,
                radii: [self.1; 2],
                insets: [Dp(0.0); 4],
                clip_radius: None,
                line: None,
                edge_smoothness: self.2,
                stroke: None,
            });
        content.draw(context.render_input);
    }
}

struct Shadow;

impl DrawModifierNode for Shadow {
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
                    corner_radii: [Dp(8.0).to_pixels_f32(); 4],
                    corner_g2: [2.0; 4],
                },
                smoothness: Dp(23.25).to_pixels_f32(),
            });
        content.draw(context.render_input);
    }
}

#[derive(Clone, Copy, PartialEq)]
struct SliderLayout(f64);

impl LayoutPolicy for SliderLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self == other
    }
    fn placement_eq(&self, other: &Self) -> bool {
        self == other
    }

    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.clamp(Dp(160.0).into());
        let height = parent.height.clamp(Dp(16.0).into());
        let thumb = Px::from(Dp(20.0)).min(width);
        let thumb_height = Px::from(Dp(16.0)).min(height);
        let track = Px::from(Dp(6.0)).min(height);
        let x = (width - thumb).to_f32() * self.0 as f32;
        let mut result = LayoutResult::new(ComputedData { width, height });
        for (index, child) in scope.children().iter().enumerate() {
            let (w, h, left, top) = match index {
                0 => (width, track, 0.0, (height - track).to_f32() * 0.5),
                1 => (
                    Px((x + thumb.to_f32() * 0.5).round() as i32),
                    track,
                    0.0,
                    (height - track).to_f32() * 0.5,
                ),
                _ => (
                    thumb,
                    thumb_height,
                    x,
                    (height - thumb_height).to_f32() * 0.5
                        + if index == 2 {
                            Dp(2.0).to_pixels_f32()
                        } else {
                            0.0
                        },
                ),
            };
            child.measure(
                scope,
                &Constraint::new(AxisConstraint::exact(w), AxisConstraint::exact(h)),
            )?;
            result.place_child(*child, PxPosition::from_f32_xy(left, top));
        }
        Ok(result)
    }
}

#[tessera]
fn slider_fill(color: Color) {
    layout(Stack).modifier(Modifier::new().push_draw(Fill(color, Dp(3.0), 1.0)));
}

#[tessera]
fn slider_shadow() {
    layout(Stack).modifier(Modifier::new().push_draw(Shadow));
}

#[tessera]
fn slider_thumb(pressed: bool, focused: bool) {
    let mut modifier = Modifier::new().push_draw(Fill(
        if pressed {
            Color::from_rgb(0.90, 0.90, 0.90)
        } else {
            Color::WHITE
        },
        Dp(8.0),
        1.3,
    ));
    if focused {
        let corner = RoundedCorner::manual(Dp(8.0), 2.0);
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
    layout(Stack).modifier(modifier);
}

/// # slider
///
/// Requests continuous numeric changes for bounded settings such as volume or intensity.
///
/// ## Usage
///
/// Adjust a setting by dragging its thumb or using directional keys.
///
/// ## Parameters
///
/// - `value`, `on_value_change` — controlled value and requested updates.
/// - `range` — finite increasing limits, defaulting to `0.0..=1.0`; values are clamped for display.
/// - `enabled`, `window_active` — input availability and active appearance; default to true.
/// - `modifier`, `focus_requester` — layout, semantics, and optional external focus handle.
/// - `accessibility_label`, `accessibility_description` — spoken name and instructions.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::slider::slider;
/// use tessera_ui::{remember, tessera, testing::UiTest, accesskit::Action};
/// #[tessera]
/// fn app() {
///     let volume = remember(|| 0.5);
///     slider(volume.get()).accessibility_label("Volume")
///         .on_value_change(move |value| volume.set(value));
/// }
/// let mut ui = UiTest::new().content(app);
/// ui.frame();
/// let (id, _) = ui.accessibility_node("Volume");
/// assert!(ui.accessibility_action(id, Action::Increment));
/// ui.frame();
/// assert_eq!(ui.accessibility_node("Volume").1.numeric_value(), Some(0.55));
/// ```
#[tessera]
pub fn slider(
    value: f64,
    range: Option<RangeInclusive<f64>>,
    on_value_change: Option<CallbackWith<f64>>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let range = range.unwrap_or(0.0..=1.0);
    let (min, max) = (*range.start(), *range.end());
    assert!(
        min.is_finite() && max.is_finite() && min < max && (max - min).is_finite(),
        "slider range must be finite and increasing"
    );
    assert!(value.is_finite(), "slider value must be finite");
    let enabled = enabled.unwrap_or(true);
    let active = window_active.unwrap_or(true);
    let next = Value {
        current: value.clamp(min, max),
        min,
        max,
        enabled,
        callback: on_value_change.unwrap_or_else(CallbackWith::default_value),
    };
    let state = remember(|| next);
    if state.get() != next {
        state.set(next);
    }
    let drag = remember(|| None::<Drag>);
    if !enabled && drag.get().is_some() {
        drag.set(None);
    }
    let requester = if let Some(requester) = focus_requester {
        requester
    } else {
        remember(FocusRequester::new).get()
    };
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let tint = if theme.appearance == Appearance::Dark {
        Color::WHITE
    } else {
        Color::BLACK
    };
    let fill = if active && enabled {
        Color::from_rgb(0.0, 0.19461783, 1.0)
    } else {
        tint.with_alpha(if enabled { 0.14 } else { 0.07 })
    };
    let mut modifier = modifier
        .unwrap_or_default()
        .push_pointer_input(SliderInput {
            value: state,
            drag,
            focus: requester,
        })
        .push_keyboard_input(SliderKeyboard(state))
        .push_semantics(SliderSemantics {
            value: state,
            label: accessibility_label,
            description: accessibility_description,
        })
        .hover_cursor_icon(CursorIcon::Default);
    if enabled {
        modifier = modifier.focusable().focus_requester(requester);
    }
    let focused = enabled && active && requester.is_focus_visible();
    let pressed = enabled && active && drag.get().is_some();
    layout(SliderLayout(next.fraction()))
        .modifier(modifier)
        .child(move || {
            slider_fill(tint.with_alpha(0.10));
            slider_fill(fill);
            slider_shadow();
            slider_thumb(pressed, focused);
        });
}
