//! Segmented controls for mutually exclusive choices or related actions.
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

/// How a segmented control responds to activation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SegmentTracking {
    /// Request one selection among the segments.
    #[default]
    SelectOne,
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
}

#[derive(Clone, PartialEq)]
struct MomentaryFocusRing(Appearance);

impl DrawModifierNode for MomentaryFocusRing {
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
        let color = match self.0 {
            Appearance::Light => Color::new(0.0, 0.133, 0.92, 0.5),
            Appearance::Dark => Color::new(0.009, 0.41, 1.0, 0.5),
        };
        let fragment = metadata.fragment_mut();
        let index = fragment.push_draw_command(PerceptualFillCommand {
            color,
            radii: [Dp(9.0); 2],
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
                path: None,
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

struct SegmentSemantics(Option<bool>);

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
        node.toggled = self.0.map(|selected| {
            if selected {
                Toggled::True
            } else {
                Toggled::False
            }
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
    tracking: SegmentTracking,
    selected: bool,
    divider: bool,
    active: bool,
    callback: CallbackWith<usize>,
) {
    let choosing = tracking == SegmentTracking::SelectOne;
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
    let pressed = item.enabled
        && active
        && (held.get() || interaction.with(|state| state.is_pressed() && state.is_hovered()));
    let color = if selected {
        tint.with_alpha(if !item.enabled {
            0.0625
        } else if pressed {
            0.20
        } else {
            0.125
        })
    } else if pressed && !choosing {
        // A momentary press darkens or lightens the resting plate without retaining selection.
        tint.with_alpha(if theme.appearance == Appearance::Dark {
            0.165
        } else {
            0.16
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
    let mut modifier = Modifier::new().clickable(ClickableArgs {
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
    });
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
    modifier = modifier
        .push_keyboard_input(ChoiceKeyboard {
            enabled: item.enabled,
            held,
            activate,
        })
        .push_semantics(SegmentSemantics(choosing.then_some(selected)))
        .hover_cursor_icon(CursorIcon::Default)
        .height(Dp(24.0))
        .push_draw(SegmentFill {
            color,
            divider: divider.then_some(tint.with_alpha(0.25)),
        });
    if item.enabled && active && requester.is_focus_visible() && !choosing {
        modifier = modifier.push_draw(MomentaryFocusRing(theme.appearance));
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
/// - `tracking` — select one option by default, or invoke momentary commands.
/// - `selected` — controlled selected index in select-one mode; ignored in momentary mode.
/// - `on_select` — index requested on activation, including repeated activation.
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
    on_select: Option<CallbackWith<usize>>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let tracking = tracking.unwrap_or_default();
    let choosing = tracking == SegmentTracking::SelectOne;
    let selected = selected.filter(|index| choosing && *index < segments.len());
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
        });
    layout(SegmentsLayout).modifier(modifier).child(move || {
        for (index, mut item) in segments.clone().into_iter().enumerate() {
            item.enabled &= enabled;
            key((index, tracking), move || {
                segment_item(
                    index,
                    item,
                    tracking,
                    selected == Some(index),
                    index > 0 && selected != Some(index - 1) && selected != Some(index),
                    active,
                    callback,
                );
            });
        }
    });
}
