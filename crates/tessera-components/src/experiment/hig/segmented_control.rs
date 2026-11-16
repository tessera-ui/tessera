//! Segmented controls for mutually exclusive choices.
//!
//! ## Usage
//!
//! Switch between related views, time ranges, or presentation modes.

use tessera_foundation::{
    gesture::{TapRecognizer, TapSettings},
    modifier::{
        ClickableArgs, InteractionState, ModifierExt as _, PointerEventContext, SemanticsArgs,
    },
};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, Callback, CallbackWith, Color,
    ComputedData, Constraint, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode,
    FocusDirection, FocusRequester, FocusState, FocusTraversalPolicy, LayoutPolicy, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, SemanticsModifierNode,
    accesskit::{Role, Toggled},
    focus, key,
    layout::{MeasureScope, layout},
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::window::CursorIcon,
};

use crate::{
    modifier::{ModifierExt as _, apply_pointer_clickable_modifier},
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
#[derive(Clone, Debug, PartialEq)]
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
        let stroke = self.divider.map(|color| {
            let x = Dp(0.5).to_pixels_f32() / size.width.to_f32().max(1.0);
            let y = Dp(5.5).to_pixels_f32() / size.height.to_f32().max(1.0);
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
                color: self.color,
                radii: [Dp(6.0); 2],
                insets: [Dp(0.0); 4],
                clip_radius: None,
                line: None,
                edge_smoothness: 1.0,
                stroke,
            });
        content.draw(context.render_input);
    }
}

struct SegmentSemantics(bool);

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
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        node.merge_descendants = false;
        node.toggled = Some(if self.0 {
            Toggled::True
        } else {
            Toggled::False
        });
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
    selected: bool,
    divider: bool,
    active: bool,
    callback: CallbackWith<usize>,
) {
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
    let pointer_down = remember(|| false);
    let tap = remember(|| {
        TapRecognizer::new(TapSettings {
            slop_px: f32::INFINITY,
            ..Default::default()
        })
    });
    if !item.enabled {
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
    let theme = use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let tint = if theme.appearance == Appearance::Dark {
        Color::WHITE
    } else {
        Color::BLACK
    };
    let pressed = item.enabled
        && active
        && (held.get() || (pointer_down.get() && interaction.with(InteractionState::is_hovered)));
    let color = if selected {
        tint.with_alpha(if !item.enabled {
            0.0625
        } else if pressed {
            0.20
        } else {
            0.125
        })
    } else {
        tint.with_alpha(if pressed { 0.125 } else { 0.0 })
    };
    let ink = if item.enabled {
        theme.label_color()
    } else {
        tint.with_alpha(if theme.appearance == Appearance::Dark {
            0.25
        } else {
            0.26
        })
    };
    let mut modifier = apply_pointer_clickable_modifier(
        Modifier::new(),
        ClickableArgs {
            enabled: item.enabled,
            on_click: activate,
            role: Some(Role::RadioButton),
            label: Some(item.label.clone()),
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
    );
    if item.enabled {
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
    modifier = modifier
        .push_keyboard_input(ChoiceKeyboard {
            enabled: item.enabled,
            held,
            activate,
        })
        .push_semantics(SegmentSemantics(selected))
        .hover_cursor_icon(CursorIcon::Default)
        .height(Dp(24.0))
        .push_draw(SegmentFill {
            color,
            divider: divider.then_some(tint.with_alpha(0.25)),
        });
    if item.enabled && active && requester.is_focus_visible() {
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
/// Requests one selection among related labeled options for view modes and filters.
///
/// ## Usage
///
/// Switch a content view or choose a time range without opening a menu.
///
/// ## Parameters
///
/// - `segments` — options in display order, including individual availability.
/// - `selected` — controlled selected index; absent or out-of-range values display no selection.
/// - `on_select` — requested index on activation, including repeated selection.
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
    selected: Option<usize>,
    on_select: Option<CallbackWith<usize>>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let selected = selected.filter(|index| *index < segments.len());
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
    let modifier = modifier
        .unwrap_or_default()
        .height(Dp(24.0))
        .focus_group(None)
        .focus_traversal_policy(FocusTraversalPolicy::horizontal().single_tab_stop(true))
        .semantics(SemanticsArgs {
            role: Some(Role::RadioGroup),
            label: accessibility_label,
            description: accessibility_description,
            disabled: !enabled,
            merge_descendants: true,
            ..Default::default()
        })
        .push_draw(SegmentFill {
            color: tint.with_alpha(0.075),
            divider: None,
        });
    layout(SegmentsLayout).modifier(modifier).child(move || {
        for (index, mut item) in segments.clone().into_iter().enumerate() {
            item.enabled &= enabled;
            key(index, move || {
                segment_item(
                    index,
                    item,
                    selected == Some(index),
                    index > 0 && selected != Some(index - 1) && selected != Some(index),
                    active,
                    callback,
                );
            });
        }
    });
}
