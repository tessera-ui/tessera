//! Material 3 button groups for related actions.
//!
//! ## Usage
//!
//! Group related actions or connected choices.

use std::collections::HashMap;

use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, CallbackWith, ComputedData,
    Constraint, Dp, FocusTraversalPolicy, LayoutResult, MeasurementError, Modifier, Px, PxPosition,
    RenderSlot, SemanticsModifierNode, State,
    accesskit::{Orientation, Role},
    current_frame_nanos, key,
    layout::{MeasureScope, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    animation::shape_motion::ShapeMotion,
    button::{ButtonShape, ButtonSize, button},
    icon::IconSize,
    modifier::{InteractionState, ModifierExt, Padding},
    shape_def::{RoundedCorner, Shape},
};

use width_motion::{PressWidthMotion, WidthData, WidthObserver, redistribute_widths};

#[cfg(all(test, not(target_family = "wasm")))]
mod native_tests;
#[cfg(all(test, not(target_family = "wasm")))]
mod width_tests;

mod width_motion;

#[cfg(test)]
thread_local! {
    static GROUP_ITEM_BUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// According to the Material 3 spec, the [`button_groups`] component supports
/// two styles: `Standard` and `Connected`.
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum ButtonGroupsStyle {
    /// Buttons have spacing between them and do not need to be the same width.
    #[default]
    Standard,
    /// Buttons use 2dp gaps and equal widths with rounded outer ends.
    Connected,
}

/// According to the Material 3 spec, the [`button_groups`] component supports
/// two selection modes: `Single` and `Multiple`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonGroupsSelectionMode {
    /// Only one button can be selected at a time.
    #[default]
    Single,
    /// Multiple buttons can be selected at the same time.
    Multiple,
}

/// According to the Material 3 spec, the [`button_groups`] component supports
/// a series of sizes.
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum ButtonGroupsSize {
    /// Extra small size.
    ExtraSmall,
    /// Small size.
    Small,
    /// Medium size.
    #[default]
    Medium,
    /// Large size.
    Large,
    /// Extra large size.
    ExtraLarge,
}

impl ButtonGroupsBuilder {
    /// Add a child item and click callback.
    pub fn child<F, C>(mut self, child: F, on_click: C) -> Self
    where
        F: Fn() + Send + Sync + 'static,
        C: Fn(bool) + Send + Sync + 'static,
    {
        self.props
            .child_closures
            .get_or_insert_with(Vec::new)
            .push(RenderSlot::new(child));
        self.props
            .on_click_closures
            .get_or_insert_with(Vec::new)
            .push(CallbackWith::new(on_click));
        self
    }

    /// Add a child item and click callback using shared callbacks.
    pub fn child_shared(
        mut self,
        child: impl Into<RenderSlot>,
        on_click: impl Into<CallbackWith<bool>>,
    ) -> Self {
        self.props
            .child_closures
            .get_or_insert_with(Vec::new)
            .push(child.into());
        self.props
            .on_click_closures
            .get_or_insert_with(Vec::new)
            .push(on_click.into());
        self
    }
}

#[derive(Clone, PartialEq)]
struct ButtonGroupsLayout {
    container_height: Dp,
    between_space: Dp,
    active_button_shape: Shape,
    inactive_button_shape: Shape,
    inactive_button_shape_start: Shape,
    inactive_button_shape_end: Shape,
    outer_corner: RoundedCorner,
}

impl ButtonGroupsLayout {
    fn new(size: ButtonGroupsSize, style: ButtonGroupsStyle, shape: ButtonShape) -> Self {
        let square = |radius| {
            let corner = RoundedCorner::manual(radius, 2.0);
            Shape::RoundedRectangle {
                top_left: corner,
                top_right: corner,
                bottom_right: corner,
                bottom_left: corner,
            }
        };
        let container_height = match size {
            ButtonGroupsSize::ExtraSmall => Dp(32.0),
            ButtonGroupsSize::Small => Dp(40.0),
            ButtonGroupsSize::Medium => Dp(56.0),
            ButtonGroupsSize::Large => Dp(96.0),
            ButtonGroupsSize::ExtraLarge => Dp(136.0),
        };
        let between_space = match style {
            ButtonGroupsStyle::Standard => match size {
                ButtonGroupsSize::ExtraSmall => Dp(18.0),
                ButtonGroupsSize::Small => Dp(12.0),
                _ => Dp(8.0),
            },
            ButtonGroupsStyle::Connected => Dp(2.0),
        };
        let active_button_shape = match style {
            ButtonGroupsStyle::Standard if shape == ButtonShape::Round => {
                square(ButtonSize::from(size).square_radius())
            }
            ButtonGroupsStyle::Standard => Shape::CAPSULE,
            ButtonGroupsStyle::Connected => Shape::CAPSULE,
        };
        let inner_radius = match size {
            ButtonGroupsSize::Large => Dp(16.0),
            ButtonGroupsSize::ExtraLarge => Dp(20.0),
            _ => Dp(8.0),
        };
        let outer_corner = if shape == ButtonShape::Round {
            RoundedCorner::Capsule
        } else {
            RoundedCorner::manual(inner_radius, 2.0)
        };
        let inactive_button_shape = match style {
            ButtonGroupsStyle::Standard if shape == ButtonShape::Square => {
                square(ButtonSize::from(size).square_radius())
            }
            ButtonGroupsStyle::Standard => Shape::CAPSULE,
            ButtonGroupsStyle::Connected => square(inner_radius),
        };
        let inactive_button_shape_start = match style {
            ButtonGroupsStyle::Standard => inactive_button_shape,
            ButtonGroupsStyle::Connected => Shape::RoundedRectangle {
                top_left: outer_corner,
                top_right: RoundedCorner::manual(inner_radius, 2.0),
                bottom_right: RoundedCorner::manual(inner_radius, 2.0),
                bottom_left: outer_corner,
            },
        };
        let inactive_button_shape_end = match style {
            ButtonGroupsStyle::Standard => inactive_button_shape,
            ButtonGroupsStyle::Connected => Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(inner_radius, 2.0),
                top_right: outer_corner,
                bottom_right: outer_corner,
                bottom_left: RoundedCorner::manual(inner_radius, 2.0),
            },
        };
        Self {
            container_height,
            between_space,
            active_button_shape,
            inactive_button_shape,
            inactive_button_shape_start,
            inactive_button_shape_end,
            outer_corner,
        }
    }
}

impl From<ButtonGroupsSize> for ButtonSize {
    fn from(size: ButtonGroupsSize) -> Self {
        match size {
            ButtonGroupsSize::ExtraSmall => Self::ExtraSmall,
            ButtonGroupsSize::Small => Self::Small,
            ButtonGroupsSize::Medium => Self::Medium,
            ButtonGroupsSize::Large => Self::Large,
            ButtonGroupsSize::ExtraLarge => Self::ExtraLarge,
        }
    }
}

#[derive(PartialEq, Default)]
struct ButtonItemState {
    actived: bool,
}

struct ButtonGroupsSemantics;

impl SemanticsModifierNode for ButtonGroupsSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        _handler: &mut Option<AccessibilityActionHandler>,
    ) {
        node.role = Some(Role::Group);
        node.orientation = Some(Orientation::Horizontal);
        node.merge_descendants = true;
    }
}

/// Selection shared with a button group.
///
/// Item indices follow visual order. Single-selection groups retain the first
/// selected item when multiple selections are supplied.
#[derive(PartialEq, Default)]
pub struct ButtonGroupsState {
    item_states: HashMap<usize, ButtonItemState>,
}

impl ButtonGroupsState {
    /// Creates an empty selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the item at `index` is selected.
    pub fn is_selected(&self, index: usize) -> bool {
        self.item_states
            .get(&index)
            .is_some_and(|item| item.actived)
    }

    /// Updates an item's selection without invoking its click callback.
    pub fn set_selected(&mut self, index: usize, selected: bool) {
        if self.is_selected(index) != selected {
            let item = self.item_state_mut(index);
            item.actived = selected;
        }
    }

    /// Clears selection without invoking click callbacks.
    /// Groups requiring selection restore the first enabled item after an empty
    /// update.
    pub fn clear(&mut self) {
        for item in self.item_states.values_mut() {
            if item.actived {
                item.actived = false;
            }
        }
    }

    fn item_state_mut(&mut self, index: usize) -> &mut ButtonItemState {
        self.item_states.entry(index).or_default()
    }
}

/// # button_groups
///
/// Button groups organize buttons and add interactions between them.
///
/// ## Usage
///
/// Used for grouping related actions.
///
/// ## Parameters
///
/// - `size` — size of the button group.
/// - `style` — visual style of the button group.
/// - `shape_style` — round or square resting shapes; defaults to round.
/// - `expanded_ratio` — pressed width expansion in standard groups, clamped to
///   0–1; defaults to 0.15. Neighbors limit expansion to their content padding.
///   Non-finite values use the default.
/// - `selection_mode` — single or multiple selection; switching to single
///   selection retains the first selected item in visual order.
/// - `modifier` — group sizing, placement, and semantics.
/// - `state` — optional external selection; omitted groups remember their own.
/// - `enabled` — whether the group accepts input.
/// - `disabled_indices` — items that retain selection but cannot be activated.
/// - `selection_required` — prevents removing the final selection; an empty
///   selection chooses the first enabled item when available.
/// - `child_closures` — per-item content builders.
/// - `on_click_closures` — per-item click handlers receiving the new active
///   state after selection has been updated.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     button_groups::{ButtonGroupsState, button_groups},
///     text::text,
/// };
///
/// let mut selection = ButtonGroupsState::new();
/// selection.set_selected(0, true);
/// assert!(selection.is_selected(0));
/// selection.clear();
/// assert!(!selection.is_selected(0));
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn demo() {
/// button_groups().child(
///     || {
///         text().content("Button 1");
///     },
///     |_| {},
/// );
/// # }
/// ```
#[tessera]
pub fn button_groups(
    size: Option<ButtonGroupsSize>,
    style: Option<ButtonGroupsStyle>,
    shape_style: Option<ButtonShape>,
    expanded_ratio: Option<f32>,
    selection_mode: Option<ButtonGroupsSelectionMode>,
    modifier: Option<Modifier>,
    state: Option<State<ButtonGroupsState>>,
    enabled: Option<bool>,
    disabled_indices: Option<Vec<usize>>,
    selection_required: Option<bool>,
    #[prop(skip_setter)] child_closures: Option<Vec<RenderSlot>>,
    #[prop(skip_setter)] on_click_closures: Option<Vec<CallbackWith<bool>>>,
) {
    let size = size.unwrap_or_default();
    let style = style.unwrap_or_default();
    let shape_style = shape_style.unwrap_or_default();
    let expanded_ratio = expanded_ratio
        .filter(|value| value.is_finite())
        .unwrap_or(0.15)
        .clamp(0.0, 1.0);
    let selection_mode = selection_mode.unwrap_or_default();
    let enabled = enabled.unwrap_or(true);
    let disabled_indices = disabled_indices.unwrap_or_default();
    let selection_required = selection_required.unwrap_or(false);
    let child_closures = child_closures.unwrap_or_default();
    let on_click_closures = on_click_closures.unwrap_or_default();
    let state = if let Some(state) = state {
        state
    } else {
        remember(ButtonGroupsState::default)
    };
    let metrics = ButtonGroupsLayout::new(size, style, shape_style);
    let child_len = child_closures.len();
    let policy = (selection_mode, selection_required);
    let selection_policy = remember(|| policy);
    if selection_policy.get() != policy {
        selection_policy.set(policy);
    }
    if state.with(|state| state.item_states.keys().any(|index| *index >= child_len)) {
        state.with_mut(|state| state.item_states.retain(|index, _| *index < child_len));
    }
    if selection_mode == ButtonGroupsSelectionMode::Single
        && state.with(|state| {
            state
                .item_states
                .values()
                .filter(|item| item.actived)
                .count()
                > 1
        })
    {
        state.with_mut(|state| {
            let selected = state
                .item_states
                .iter()
                .filter(|(_, item)| item.actived)
                .map(|(index, _)| *index)
                .min();
            for (index, item) in &mut state.item_states {
                if item.actived && Some(*index) != selected {
                    item.actived = false;
                }
            }
        });
    }
    if selection_required
        && enabled
        && !state.with(|state| state.item_states.values().any(|item| item.actived))
        && let Some(index) = (0..child_len).find(|index| !disabled_indices.contains(index))
    {
        state.with_mut(|state| state.set_selected(index, true));
    }
    layout()
        .modifier(
            modifier
                .unwrap_or_default()
                .focus_group()
                .focus_traversal_policy(FocusTraversalPolicy::horizontal().wrap(true))
                .push_semantics(ButtonGroupsSemantics)
                .size_in(
                    None,
                    None,
                    Some(Dp(metrics.container_height.0.max(48.0))),
                    None,
                ),
        )
        .policy(ButtonGroupsRowLayout {
            connected: style == ButtonGroupsStyle::Connected,
            spacing: metrics.between_space.into(),
            expanded_ratio,
        })
        .child(move || {
            for (index, child_closure) in child_closures.iter().cloned().enumerate() {
                key(index, || {
                    let on_click_closure = on_click_closures[index];
                    let item_layout = metrics.clone();
                    let enabled = enabled && !disabled_indices.contains(&index);

                    let interaction = remember(InteractionState::new);
                    let width_motion = remember(PressWidthMotion::default);
                    let item = RenderSlot::new(move || {
                        let actived = state.with(|state| state.is_selected(index));
                        let pressed = enabled && interaction.with(InteractionState::is_pressed);
                        let resting_shape = if actived {
                            item_layout.active_button_shape
                        } else if child_len == 1 && style == ButtonGroupsStyle::Connected {
                            let corner = item_layout.outer_corner;
                            Shape::RoundedRectangle {
                                top_left: corner,
                                top_right: corner,
                                bottom_right: corner,
                                bottom_left: corner,
                            }
                        } else if index == 0 {
                            item_layout.inactive_button_shape_start
                        } else if index == child_len - 1 {
                            item_layout.inactive_button_shape_end
                        } else {
                            item_layout.inactive_button_shape
                        };
                        let shape = if pressed {
                            if style == ButtonGroupsStyle::Connected && child_len > 1 {
                                let radius = match size {
                                    ButtonGroupsSize::Large => Dp(12.0),
                                    ButtonGroupsSize::ExtraLarge => Dp(16.0),
                                    _ => Dp(4.0),
                                };
                                let inner = RoundedCorner::manual(radius, 2.0);
                                Shape::RoundedRectangle {
                                    top_left: if index == 0 {
                                        item_layout.outer_corner
                                    } else {
                                        inner
                                    },
                                    top_right: if index + 1 == child_len {
                                        item_layout.outer_corner
                                    } else {
                                        inner
                                    },
                                    bottom_right: if index + 1 == child_len {
                                        item_layout.outer_corner
                                    } else {
                                        inner
                                    },
                                    bottom_left: if index == 0 {
                                        item_layout.outer_corner
                                    } else {
                                        inner
                                    },
                                }
                            } else {
                                let corner = RoundedCorner::manual(
                                    ButtonSize::from(size).pressed_radius(),
                                    2.0,
                                );
                                Shape::RoundedRectangle {
                                    top_left: corner,
                                    top_right: corner,
                                    bottom_right: corner,
                                    bottom_left: corner,
                                }
                            }
                        } else {
                            resting_shape
                        };
                        let motion = remember(|| ShapeMotion::new(shape));
                        let mut next = motion.get();
                        next.retarget(shape, enabled, current_frame_nanos());
                        if next != motion.get() {
                            motion.set(next);
                        }
                        if next.is_animating() {
                            receive_frame_nanos(move |frame| {
                                let mut next = motion.get();
                                next.advance(frame);
                                if next != motion.get() {
                                    motion.set(next);
                                }
                                if next.is_animating() {
                                    tessera_ui::FrameNanosControl::Continue
                                } else {
                                    tessera_ui::FrameNanosControl::Stop
                                }
                            });
                        }
                        button()
                            .filled()
                            .size(size.into())
                            .content_padding(Padding::default())
                            .selected(actived)
                            .enabled(enabled)
                            .accessibility_role(Role::CheckBox)
                            .interaction_state(interaction)
                            .on_click(move || {
                                let mut next = width_motion.get();
                                next.pulse(current_frame_nanos());
                                if next != width_motion.get() {
                                    width_motion.set(next);
                                }
                                let active = state.with(|state| {
                                    state
                                        .item_states
                                        .get(&index)
                                        .is_some_and(|item| item.actived)
                                });
                                let (mode, required) = selection_policy.get();
                                if active
                                    && required
                                    && state.with(|state| {
                                        state
                                            .item_states
                                            .values()
                                            .filter(|item| item.actived)
                                            .count()
                                            == 1
                                    })
                                {
                                    return;
                                }
                                state.with_mut(|s| {
                                    if !active && mode == ButtonGroupsSelectionMode::Single {
                                        for (other_index, item) in &mut s.item_states {
                                            if *other_index != index && item.actived {
                                                item.actived = false;
                                            }
                                        }
                                    }

                                    let item = s.item_state_mut(index);
                                    item.actived = !active;
                                });
                                on_click_closure.call(!active);
                            })
                            .shape(motion.get().shape())
                            .child(move || {
                                let icon_size = ButtonSize::from(size).icon_size();
                                provide_context(
                                    || IconSize(icon_size),
                                    move || {
                                        let inherited = use_context::<IconSize>()
                                            .expect("button group icon size");
                                        if inherited.get().0 != icon_size {
                                            inherited.set(IconSize(icon_size));
                                        }
                                        layout()
                                            .policy(GroupContentLayout {
                                                padding: ButtonSize::from(size).content_padding(),
                                                compress: style == ButtonGroupsStyle::Standard,
                                            })
                                            .child_shared(child_closure);
                                    },
                                );
                            });
                    });
                    group_item()
                        .interaction_state(interaction)
                        .width_motion(width_motion)
                        .enabled(enabled && expanded_ratio > 0.0 && child_len > 1)
                        .compression_limit(ButtonSize::from(size).content_padding().right)
                        .connected(style == ButtonGroupsStyle::Connected)
                        .child_shared(item);
                });
            }
        });
}

#[tessera]
fn group_item(
    interaction_state: Option<State<InteractionState>>,
    width_motion: Option<State<PressWidthMotion>>,
    enabled: Option<bool>,
    compression_limit: Option<Dp>,
    connected: Option<bool>,
    child: Option<RenderSlot>,
) {
    #[cfg(test)]
    GROUP_ITEM_BUILDS.set(GROUP_ITEM_BUILDS.get() + 1);
    let interaction = interaction_state.expect("group item interaction");
    let motion = width_motion.expect("group item width motion");
    let child = child.expect("group item content");
    let mut next = motion.get();
    next.update(
        interaction.with(InteractionState::is_pressed),
        enabled.unwrap_or(true) && !connected.unwrap_or(false),
        current_frame_nanos(),
    );
    if next != motion.get() {
        motion.set(next);
    }
    if next.is_animating() {
        receive_frame_nanos(move |frame| {
            let mut next = motion.get();
            next.advance(frame);
            if next != motion.get() {
                motion.set(next);
            }
            if next.is_animating() {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    layout()
        .modifier(
            Modifier::new()
                .push_parent_data(WidthData {
                    progress: motion.get().value(),
                    compression_limit: compression_limit.unwrap_or_default().into(),
                })
                .push_pointer_input(WidthObserver {
                    interaction,
                    motion,
                }),
        )
        .policy(GroupItemLayout)
        .child(move || {
            child.render();
        });
}

#[derive(Clone, PartialEq)]
struct GroupItemLayout;

impl tessera_ui::LayoutModifierNode for GroupItemLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let child = input.children()[0];
        let constraint = *input.parent_constraint().as_ref();
        let size = child.measure(input, &constraint)?.size();
        let mut result = LayoutResult::new(size);
        result.place_child(child, PxPosition::ZERO);
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct GroupContentLayout {
    padding: Padding,
    compress: bool,
}

impl tessera_ui::LayoutModifierNode for GroupContentLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *input.parent_constraint().as_ref();
        let children = input.children();
        let mut horizontal = Px::from(self.padding.left) + Px::from(self.padding.right);
        let top = Px::from(self.padding.top);
        let vertical = top + Px::from(self.padding.bottom);
        if self.compress {
            let mut natural_width = Px::ZERO;
            for child in &children {
                natural_width =
                    natural_width.max(child.probe_intrinsic(input, &Constraint::NONE)?.width);
            }
            if let Some(maximum) = parent.width.resolve_max() {
                horizontal = horizontal.min((maximum - natural_width).max(Px::ZERO));
            }
        }
        let constraint = Constraint::new(
            AxisConstraint::new(
                Px::ZERO,
                parent
                    .width
                    .resolve_max()
                    .map(|width| (width - horizontal).max(Px::ZERO)),
            ),
            AxisConstraint::new(
                Px::ZERO,
                parent
                    .height
                    .resolve_max()
                    .map(|height| (height - vertical).max(Px::ZERO)),
            ),
        );
        let mut content = ComputedData {
            width: Px::ZERO,
            height: Px::ZERO,
        };
        for child in &children {
            let size = child.measure(input, &constraint)?;
            content.width = content.width.max(size.width);
            content.height = content.height.max(size.height);
        }
        let size = ComputedData {
            width: parent.width.clamp(content.width + horizontal),
            height: parent.height.clamp(content.height + vertical),
        };
        let left = if self.compress {
            horizontal.0 as f32 / 2.0
        } else {
            Px::from(self.padding.left).0 as f32
        };
        let mut result = LayoutResult::new(size);
        for child in children {
            result.place_child(child, PxPosition::from_f32_xy(left, top.0 as f32));
        }
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct ButtonGroupsRowLayout {
    connected: bool,
    spacing: Px,
    expanded_ratio: f32,
}

impl tessera_ui::LayoutModifierNode for ButtonGroupsRowLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let parent = *input.parent_constraint().as_ref();
        let mut result = LayoutResult::default();
        if children.is_empty() {
            return Ok(result.with_size(ComputedData {
                width: parent.width.clamp(Px::ZERO),
                height: parent.height.clamp(Px::ZERO),
            }));
        }
        let count = children.len() as i32;
        let spacing = if count > 1 {
            self.spacing
                .min(parent.width.resolve_max().unwrap_or(Px::MAX) / (count - 1))
        } else {
            Px::ZERO
        };
        let gaps = spacing * (count - 1);
        let child_constraint =
            Constraint::new(parent.width.without_min(), parent.height.without_min());
        let mut sizes = Vec::with_capacity(children.len());
        for child in &children {
            sizes.push(child.measure(input, &child_constraint)?.size());
        }
        if self.connected {
            let natural = sizes
                .iter()
                .map(|size| size.width)
                .max()
                .unwrap_or(Px::ZERO)
                * count
                + gaps;
            let budget = (parent.width.clamp(natural) - gaps).max(Px::ZERO);
            let width = budget / count;
            let remainder = budget.0 % count;
            for (index, child) in children.iter().enumerate() {
                let width = width + Px(i32::from((index as i32) < remainder));
                sizes[index] = child
                    .measure(
                        input,
                        &Constraint::new(AxisConstraint::exact(width), child_constraint.height),
                    )?
                    .size();
            }
        } else {
            let data: Vec<_> = children
                .iter()
                .map(|child| child.parent_data::<WidthData>(input).unwrap_or_default())
                .collect();
            let mut widths: Vec<_> = sizes.iter().map(|size| size.width).collect();
            redistribute_widths(&mut widths, &data, self.expanded_ratio);
            for (index, child) in children.iter().enumerate() {
                if widths[index] != sizes[index].width {
                    sizes[index] = child
                        .measure(
                            input,
                            &Constraint::new(
                                AxisConstraint::exact(widths[index]),
                                child_constraint.height,
                            ),
                        )?
                        .size();
                }
            }
        }
        let height = sizes
            .iter()
            .map(|size| size.height)
            .max()
            .unwrap_or(Px::ZERO);
        if self.connected {
            let height = parent.height.clamp(height);
            for (index, child) in children.iter().enumerate() {
                if sizes[index].height == height {
                    continue;
                }
                sizes[index] = child
                    .measure(input, &Constraint::new(sizes[index].width, height))?
                    .size();
            }
        }
        let size = ComputedData {
            width: parent.width.clamp(
                sizes
                    .iter()
                    .map(|size| size.width)
                    .fold(gaps, |total, width| total + width),
            ),
            height: parent.height.clamp(height),
        };
        let mut x = Px::ZERO;
        for (child, child_size) in children.iter().zip(sizes) {
            result.place_child(
                *child,
                PxPosition::from_f32_xy(
                    x.0 as f32,
                    (size.height.0 - child_size.height.0) as f32 / 2.0,
                ),
            );
            x += child_size.width + spacing;
        }
        Ok(result.with_size(size))
    }
}

#[cfg(test)]
mod tests;
