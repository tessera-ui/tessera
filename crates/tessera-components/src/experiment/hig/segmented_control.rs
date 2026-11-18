//! Segmented controls for related choices or actions.
//!
//! ## Usage
//!
//! Switch between related views, time ranges, or presentation modes.

use tessera_foundation::{
    modifier::{ClickableArgs, InteractionState, ModifierExt as _, SemanticsArgs},
    pointer_gestures::TapGestureConfig,
};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, Callback, CallbackWith, Color,
    ComputedData, Constraint, CursorEventContent, Dp, DrawModifierContent, DrawModifierContext,
    DrawModifierNode, FloatRect, FocusDirection, FocusRequester, FocusState, FocusTraversalPolicy,
    GestureState, LayoutPolicy, LayoutResult, MeasurementError, Modifier, PointerEventPass,
    PointerId, PointerInput, PointerInputModifierNode, PressKeyEventType, Px, PxPosition,
    SemanticsModifierNode, State,
    accesskit::{Action as AxAction, Role, Toggled},
    focus, key,
    layout::{MeasureScope, layout},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, remember_with_key, tessera, use_context,
    winit::window::CursorIcon,
};

use crate::{
    modifier::ModifierExt as _,
    pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualStroke},
    shape_def::{RoundedCorner, Shape},
};

use super::{
    choice::ChoiceKeyboard,
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "segmented_control_tests.rs"]
mod tests;

/// One labeled option in a [`segmented_control`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Segment {
    /// Visible title and accessible name.
    pub label: String,
    /// Whether the option accepts input and keyboard focus.
    pub enabled: bool,
}

impl Segment {
    /// Creates an enabled option.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
        }
    }
}

/// How a segmented control responds to activation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SegmentTracking {
    /// Request one selection among the segments.
    #[default]
    SelectOne,
    /// Toggle independent selections, including an empty selection.
    SelectAny,
    /// Invoke a segment action without keeping a selection.
    Momentary,
}

#[derive(Clone, Copy, PartialEq)]
struct SegmentsLayout;

impl LayoutPolicy for SegmentsLayout {
    fn measure_eq(&self, _: &Self) -> bool {
        true
    }

    fn placement_eq(&self, _: &Self) -> bool {
        true
    }

    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = scope.children();
        let mut widths = Vec::with_capacity(children.len());
        let mut natural = Px::ZERO;
        for child in &children {
            let width = child.probe_intrinsic(scope, &Constraint::NONE)?.width;
            widths.push(width);
            natural += width;
        }
        let parent = *scope.parent_constraint().as_ref();
        let width = parent.width.clamp(natural);
        let height = parent.height.clamp(Dp(24.0).into());
        let mut result = LayoutResult::new(ComputedData { width, height });
        let mut prefix = 0.0;
        let mut left = Px::ZERO;
        for (index, child) in children.iter().enumerate() {
            prefix += widths[index].to_f32();
            let right = if index + 1 == children.len() {
                width
            } else if width >= natural {
                Px((prefix
                    + (width - natural).to_f32() * (index + 1) as f32 / children.len() as f32)
                    .round() as i32)
            } else if natural > Px::ZERO {
                Px((prefix * width.to_f32() / natural.to_f32()).round() as i32)
            } else {
                Px::ZERO
            };
            child.measure(
                scope,
                &Constraint::new(
                    AxisConstraint::exact(right - left),
                    AxisConstraint::exact(height),
                ),
            )?;
            result.place_child(*child, PxPosition::new(left, Px::ZERO));
            left = right;
        }
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct SegmentFill {
    color: Color,
    divider: Option<Color>,
    joined: [bool; 2],
    accent: bool,
}

#[derive(Clone, Copy, PartialEq)]
struct TrackingPress {
    pointer: PointerId,
    target: Option<usize>,
}

struct MultiPointer {
    press: State<Option<TrackingPress>>,
    bounds: State<Vec<Option<FloatRect>>>,
    enabled: Vec<bool>,
    activate: CallbackWith<usize>,
}

impl PointerInputModifierNode for MultiPointer {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        let mut press = self.press.get();
        let position = input.cursor_position_abs();
        for change in input.pointer_changes.iter_mut() {
            if press.is_some_and(|press| press.pointer != change.pointer_id) {
                continue;
            }
            if change.is_consumed() {
                press = None;
                continue;
            }
            let target = || {
                position.and_then(|position| {
                    self.bounds.with(|bounds| {
                        bounds
                            .iter()
                            .zip(&self.enabled)
                            .position(|(bounds, enabled)| {
                                *enabled && bounds.is_some_and(|bounds| bounds.contains(position))
                            })
                    })
                })
            };
            match change.content {
                CursorEventContent::Pressed(PressKeyEventType::Left) if press.is_none() => {
                    if let Some(target) = target()
                        && change.claim_tap_press()
                    {
                        press = Some(TrackingPress {
                            pointer: change.pointer_id,
                            target: Some(target),
                        });
                    }
                }
                CursorEventContent::Moved(_) if press.is_some() => {
                    press.as_mut().unwrap().target =
                        if change.gesture_state == GestureState::Dragged {
                            None
                        } else {
                            target()
                        };
                }
                CursorEventContent::Released(PressKeyEventType::Left) if press.take().is_some() => {
                    if change.gesture_state == GestureState::TapCandidate
                        && let Some(target) = target()
                    {
                        change.consume();
                        self.activate.call(target);
                    }
                }
                CursorEventContent::Cancelled(PressKeyEventType::Left)
                | CursorEventContent::Scroll(_) => {
                    press = None;
                }
                _ => {}
            }
        }
        if self.press.is_alive() && self.press.get() != press {
            self.press.set(press);
        }
    }
}

#[derive(Clone, PartialEq)]
struct SegmentFocusRing {
    appearance: Appearance,
    radius: Dp,
}

impl DrawModifierNode for SegmentFocusRing {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        content.draw(context.render_input);
        let mut metadata = context.render_input.metadata_mut();
        let Some(size) = metadata.computed_data() else {
            return;
        };
        if size.width <= Px::ZERO || size.height <= Px::ZERO {
            return;
        }
        let outset = Dp(3.0).to_px();
        let color = match self.appearance {
            Appearance::Light => Color::new(0.0, 0.133, 0.92, 0.5),
            Appearance::Dark => Color::new(0.009, 0.41, 1.0, 0.5),
        };
        let fragment = metadata.fragment_mut();
        let index = fragment.push_draw_command(PerceptualFillCommand {
            color,
            radii: [self.radius; 2],
            insets: [Dp::ZERO; 4],
            clip_radius: None,
            edge_smoothness: 1.0,
            stroke: None,
            path: None,
            line: None,
            outline_width: Dp(4.0),
            snap_to_pixels: false,
        });
        // Expand only this paint operation, not the segment's input or layout bounds.
        let op = &mut fragment.ops_mut()[index as usize];
        op.position_override = Some(PxPosition::new(-outset, -outset));
        op.size_override = Some(tessera_ui::PxSize::new(
            size.width + outset * 2,
            size.height + outset * 2,
        ));
    }
}

impl DrawModifierNode for SegmentFill {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let size = context
            .render_input
            .metadata_mut()
            .computed_data()
            .expect("measured segment");
        // Extend only the fill math past joined edges; the raster bounds stay local.
        let [left, right] = self
            .joined
            .map(|joined| if joined { Dp(6.0) } else { Dp::ZERO });
        let stroke = self.divider.map(|color| {
            let x = Dp(0.5 + left.0).to_pixels_f32()
                / (size.width.to_f32() + Dp(left.0 + right.0).to_pixels_f32()).max(1.0);
            let y = if self.joined[0] {
                0.0
            } else {
                Dp(5.5).to_pixels_f32() / size.height.to_f32().max(1.0)
            };
            PerceptualStroke {
                raster_mask: false,
                color,
                width: Dp(1.0),
                points: [[x, y], [x, 0.5], [x, 1.0 - y]],
            }
        });
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                outline_width: Dp::ZERO,
                snap_to_pixels: false,
                path: None,
                color: self.color,
                radii: [Dp(if self.accent { 5.65 } else { 6.0 }); 2],
                insets: [Dp::ZERO, Dp(-right.0), Dp::ZERO, Dp(-left.0)],
                clip_radius: None,
                line: None,
                edge_smoothness: if self.accent { 1.10 } else { 1.0 },
                stroke,
            });
        content.draw(context.render_input);
    }
}

struct SegmentSemantics {
    selected: Option<bool>,
    activate: Option<Callback>,
}

#[derive(Clone, Copy, PartialEq)]
struct SegmentLayout;

impl LayoutPolicy for SegmentLayout {
    fn measure_eq(&self, _: &Self) -> bool {
        true
    }

    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let padding: Px = Dp(12.0).into();
        let child = scope.children()[0];
        let size = child.measure(
            scope,
            &Constraint::new(
                AxisConstraint::new(
                    Px::ZERO,
                    parent
                        .width
                        .resolve_max()
                        .map(|max| (max - padding * 2).max(Px::ZERO)),
                ),
                parent.height.without_min(),
            ),
        )?;
        let width = parent.width.clamp(size.width + padding * 2);
        let height = parent.height.clamp(size.height);
        let mut result = LayoutResult::new(ComputedData { width, height });
        // Center labels on physical pixels instead of between raster phases.
        result.place_child(
            child,
            PxPosition::from_f32_xy(
                ((width - size.width).to_f32() * 0.5).round(),
                ((height - size.height).to_f32() * 0.5).round(),
            ),
        );
        Ok(result)
    }
}

impl SemanticsModifierNode for SegmentSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        handler: &mut Option<AccessibilityActionHandler>,
    ) {
        node.merge_descendants = false;
        node.toggled = self.selected.map(|selected| {
            if selected {
                Toggled::True
            } else {
                Toggled::False
            }
        });
        if let Some(activate) = self.activate {
            node.actions.push(AxAction::Click);
            *handler = Some(Box::new(move |action| {
                if action == AxAction::Click {
                    activate.call();
                }
            }));
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Action {
    index: usize,
    enabled: bool,
    callback: CallbackWith<usize>,
}

#[tessera]
fn segment_item(
    index: usize,
    item: Segment,
    tracking: SegmentTracking,
    selected: bool,
    divider: bool,
    joined: [bool; 2],
    active: bool,
    callback: CallbackWith<usize>,
    multi_pointer: Option<(State<Option<TrackingPress>>, State<Vec<Option<FloatRect>>>)>,
) {
    let choosing = tracking == SegmentTracking::SelectOne;
    let multiple = tracking == SegmentTracking::SelectAny;
    let next = Action {
        index,
        enabled: item.enabled,
        callback,
    };
    let action = remember(|| next);
    if action.get() != next {
        action.set(next);
    }
    let activate = Callback::new(move || {
        let action = action.get();
        if action.enabled {
            action.callback.call(action.index);
        }
    });
    let requester = remember(FocusRequester::new).get();
    let interaction = remember(InteractionState::new);
    let held = remember(|| false);
    if !item.enabled {
        if held.get() {
            held.set(false);
        }
    } else if held.get() && !requester.is_focused() {
        held.set(false);
    }
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let tint = if theme.appearance == Appearance::Dark {
        Color::WHITE
    } else {
        Color::BLACK
    };
    let pointed = multi_pointer
        .and_then(|(press, _)| press.with(|press| press.and_then(|press| press.target)));
    let pointer_pressed = if multiple {
        pointed == Some(index)
    } else {
        interaction.with(|state| state.is_pressed() && state.is_hovered())
    };
    let pressed = item.enabled && active && (held.get() || pointer_pressed);
    let accent = multiple && selected && active;
    let color = if accent {
        match (theme.appearance, item.enabled, pressed) {
            (Appearance::Light, false, _) => Color::from_rgb(0.05448, 0.29614, 0.85499),
            (Appearance::Dark, false, _) => Color::from_rgb(0.006995, 0.13563, 0.52712),
            (Appearance::Light, true, false) => Color::from_rgb(0.0, 0.18448, 0.94731),
            (Appearance::Light, true, true) => Color::from_rgb(0.0, 0.14703, 0.75294),
            (Appearance::Dark, true, false) => Color::from_rgb(0.001821, 0.20508, 1.0),
            (Appearance::Dark, true, true) => Color::from_rgb(0.012983, 0.25415, 1.0),
        }
    } else if selected {
        tint.with_alpha(if !item.enabled {
            0.0625
        } else if pressed {
            0.20
        } else {
            0.125
        })
    } else if pressed && !choosing {
        tint.with_alpha(if theme.appearance == Appearance::Dark {
            0.165
        } else {
            0.16
        })
    } else {
        tint.with_alpha(if pressed { 0.125 } else { 0.0 })
    };
    let ink = if accent {
        // Dimming the white ink keeps the same raster mask as the enabled label.
        Color::WHITE.with_alpha(if item.enabled { 1.0 } else { 0.5 })
    } else if item.enabled {
        theme.label_color()
    } else {
        tint.with_alpha(if theme.appearance == Appearance::Dark {
            0.25
        } else {
            0.26
        })
    };
    let mut modifier = if let Some((_, bounds)) = multi_pointer {
        let mut modifier = Modifier::new()
            .on_globally_positioned_precise(move |rect| {
                bounds.with_mut(|bounds| {
                    if bounds.len() <= index {
                        bounds.resize(index + 1, None);
                    }
                    bounds[index] = Some(rect);
                });
            })
            .semantics(SemanticsArgs {
                role: Some(Role::Button),
                label: Some(item.label.clone()),
                disabled: !item.enabled,
                focusable: item.enabled,
                ..Default::default()
            });
        if item.enabled {
            modifier = modifier.focusable().focus_requester(requester);
        }
        modifier
    } else {
        Modifier::new().clickable(ClickableArgs {
            enabled: item.enabled,
            on_click: activate,
            role: Some(if choosing {
                Role::RadioButton
            } else {
                Role::Button
            }),
            label: Some(item.label.clone()),
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
    };
    if item.enabled && choosing {
        modifier =
            modifier
                .preferred_tab_stop(selected)
                .on_focus_changed(move |state: FocusState| {
                    if state.is_focused()
                        && matches!(
                            focus::focus_change_direction(),
                            Some(FocusDirection::Left | FocusDirection::Right)
                        )
                    {
                        activate.call();
                    }
                });
    }
    let divider = if joined[0] {
        Some(if active {
            match theme.appearance {
                Appearance::Light => Color::from_rgb(0.0, 0.10224, 0.73791),
                Appearance::Dark => Color::from_rgb(0.01444, 0.32314, 1.0),
            }
        } else {
            tint.with_alpha(if item.enabled { 0.42 } else { 0.41 })
        })
    } else if divider
        && !(multiple
            && (pressed
                || pointed
                    .zip(index.checked_sub(1))
                    .is_some_and(|(pointed, previous)| pointed == previous)))
    {
        Some(tint.with_alpha(0.25))
    } else {
        None
    };
    modifier = modifier
        .push_keyboard_input(ChoiceKeyboard {
            enabled: item.enabled,
            held,
            activate,
        })
        .push_semantics(SegmentSemantics {
            selected: (tracking != SegmentTracking::Momentary).then_some(selected),
            activate: (multiple && item.enabled).then_some(activate),
        })
        .hover_cursor_icon(CursorIcon::Default)
        .height(Dp(24.0))
        .push_draw(SegmentFill {
            color,
            divider,
            joined,
            accent: multiple && selected,
        });
    if item.enabled && active && requester.is_focus_visible() && !choosing {
        modifier = modifier.push_draw(SegmentFocusRing {
            appearance: theme.appearance,
            radius: Dp(if multiple { 9.75 } else { 9.0 }),
        });
    } else if item.enabled && active && requester.is_focus_visible() {
        let corner = RoundedCorner::manual(Dp(6.0), 2.0);
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
    layout(SegmentLayout).modifier(modifier).child(move || {
        text(item.label.clone())
            .style(TextStyle::default())
            .color(ink)
            .single_line(true)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                hidden: true,
                ..Default::default()
            }));
    });
}

/// # segmented_control
///
/// Groups related labeled options or commands in one control.
///
/// ## Usage
///
/// Switch a content view or offer a pair of navigation commands.
///
/// ## Parameters
///
/// - `segments` — options in display order, including individual availability.
/// - `tracking` — select one option by default, toggle independent choices, or invoke momentary commands.
/// - `selected` — controlled selected index in select-one mode.
/// - `selected_indices` — controlled selected indices in select-any mode; empty by default.
/// - `on_select` — index requested on activation; toggle that index in select-any mode.
/// - `enabled`, `window_active` — input availability and active window appearance; both default to true.
/// - `modifier` — layout and semantics; extra width is distributed evenly between segments.
/// - `accessibility_label`, `accessibility_description` — spoken group name and instructions.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::segmented_control::{Segment, segmented_control};
/// use tessera_ui::{PxPosition, remember, tessera, testing::UiTest};
/// #[tessera]
/// fn app() {
///     let selected = remember(|| 0);
///     segmented_control(vec![Segment::new("Day"), Segment::new("Week")])
///         .selected(selected.get()).on_select(move |index| selected.set(index));
/// }
/// let mut ui = UiTest::new().content(app);
/// ui.frame();
/// let (_, week) = ui.accessibility_node("Week");
/// let bounds = week.bounds().unwrap();
/// ui.click_at(PxPosition::from_f32_xy(((bounds.x0 + bounds.x1) / 2.0) as f32, ((bounds.y0 + bounds.y1) / 2.0) as f32));
/// assert_eq!(ui.accessibility_node("Week").1.toggled(), Some(tessera_ui::accesskit::Toggled::True));
/// ```
#[tessera]
pub fn segmented_control(
    segments: Vec<Segment>,
    tracking: Option<SegmentTracking>,
    selected: Option<usize>,
    selected_indices: Option<Vec<usize>>,
    on_select: Option<CallbackWith<usize>>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let tracking = tracking.unwrap_or_default();
    let choosing = tracking == SegmentTracking::SelectOne;
    let multiple = tracking == SegmentTracking::SelectAny;
    let selected_indices = selected_indices.unwrap_or_default();
    let enabled = enabled.unwrap_or(true);
    let active = window_active.unwrap_or(true);
    let callback = on_select.unwrap_or_else(CallbackWith::default_value);
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let tint = if theme.appearance == Appearance::Dark {
        Color::WHITE
    } else {
        Color::BLACK
    };
    let mut modifier = modifier
        .unwrap_or_default()
        .height(Dp(24.0))
        .focus_group(None)
        .focus_traversal_policy(FocusTraversalPolicy::horizontal().single_tab_stop(true))
        .semantics(SemanticsArgs {
            role: Some(if choosing {
                Role::RadioGroup
            } else {
                Role::Group
            }),
            label: accessibility_label,
            description: accessibility_description,
            disabled: !enabled,
            merge_descendants: true,
            ..Default::default()
        })
        .push_draw(SegmentFill {
            color: tint.with_alpha(0.075),
            divider: None,
            joined: [false; 2],
            accent: false,
        });
    let multi_pointer = if multiple {
        let press = remember_with_key((segments.clone(), enabled), || None);
        let bounds = remember(Vec::new);
        modifier = modifier.push_pointer_input(MultiPointer {
            press,
            bounds,
            enabled: segments
                .iter()
                .map(|item| enabled && item.enabled)
                .collect(),
            activate: callback,
        });
        Some((press, bounds))
    } else {
        None
    };
    let is_selected = move |index| match tracking {
        SegmentTracking::SelectOne => selected == Some(index),
        SegmentTracking::SelectAny => selected_indices.contains(&index),
        SegmentTracking::Momentary => false,
    };
    let segment_count = segments.len();
    layout(SegmentsLayout).modifier(modifier).child(move || {
        for (index, mut item) in segments.clone().into_iter().enumerate() {
            item.enabled &= enabled;
            let selected = is_selected(index);
            let divider = index > 0 && !is_selected(index - 1) && !selected;
            let joined = if multiple && selected {
                [
                    index > 0 && is_selected(index - 1),
                    index + 1 < segment_count && is_selected(index + 1),
                ]
            } else {
                [false; 2]
            };
            key((index, tracking), move || {
                segment_item(
                    index, item, tracking, selected, divider, joined, active, callback,
                )
                .multi_pointer_optional(multi_pointer);
            });
        }
    });
}
