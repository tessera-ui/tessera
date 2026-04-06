//! Material 3 button groups for related actions.
//!
//! ## Usage
//!
//! Used for grouping related actions.

use std::collections::HashMap;

use tessera_ui::{
    CallbackWith, ComputedData, Dp, LayoutPolicy, LayoutResult, MeasurementError, Modifier, Px,
    PxPosition, RenderSlot, current_frame_nanos,
    layout::{MeasureScope, layout},
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::MainAxisAlignment,
    animation,
    button::button,
    modifier::ModifierExt,
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    theme::MaterialTheme,
};

/// According to the Material 3 spec, the [`button_groups`] component supports
/// two styles: `Standard` and `Connected`.
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum ButtonGroupsStyle {
    /// Buttons have spacing between them and do not need to be the same width.
    #[default]
    Standard,
    /// Buttons are adjacent with no spacing, and each button must be the same
    /// width.
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
        self.props.child_closures.push(RenderSlot::new(child));
        self.props
            .on_click_closures
            .push(CallbackWith::new(on_click));
        self
    }

    /// Add a child item and click callback using shared callbacks.
    pub fn child_shared(
        mut self,
        child: impl Into<RenderSlot>,
        on_click: impl Into<CallbackWith<bool>>,
    ) -> Self {
        self.props.child_closures.push(child.into());
        self.props.on_click_closures.push(on_click.into());
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
}

impl ButtonGroupsLayout {
    fn new(size: ButtonGroupsSize, style: ButtonGroupsStyle) -> Self {
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
            ButtonGroupsStyle::Standard => Shape::rounded_rectangle(Dp(16.0)),
            ButtonGroupsStyle::Connected => Shape::CAPSULE,
        };
        let inactive_button_shape = match style {
            ButtonGroupsStyle::Standard => Shape::CAPSULE,
            ButtonGroupsStyle::Connected => Shape::rounded_rectangle(Dp(16.0)),
        };
        let inactive_button_shape_start = match style {
            ButtonGroupsStyle::Standard => active_button_shape,
            ButtonGroupsStyle::Connected => Shape::RoundedRectangle {
                top_left: RoundedCorner::Capsule,
                top_right: RoundedCorner::manual(Dp(16.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(16.0), 3.0),
                bottom_left: RoundedCorner::Capsule,
            },
        };
        let inactive_button_shape_end = match style {
            ButtonGroupsStyle::Standard => active_button_shape,
            ButtonGroupsStyle::Connected => Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(16.0), 3.0),
                top_right: RoundedCorner::Capsule,
                bottom_right: RoundedCorner::Capsule,
                bottom_left: RoundedCorner::manual(Dp(16.0), 3.0),
            },
        };
        Self {
            container_height,
            between_space,
            active_button_shape,
            inactive_button_shape,
            inactive_button_shape_start,
            inactive_button_shape_end,
        }
    }
}

#[derive(PartialEq, Default)]
struct ButtonItemState {
    actived: bool,
    elastic_state: ElasticState,
}

#[derive(PartialEq, Default)]
struct ButtonGroupsState {
    item_states: HashMap<usize, ButtonItemState>,
}

impl ButtonGroupsState {
    fn item_state_mut(&mut self, index: usize) -> &mut ButtonItemState {
        self.item_states.entry(index).or_default()
    }

    fn item_state(&self, index: usize) -> Option<&ButtonItemState> {
        self.item_states.get(&index)
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
/// - `selection_mode` — selection mode of the button group.
/// - `child_closures` — per-item content builders.
/// - `on_click_closures` — per-item click handlers receiving the new active
///   state.
///
/// ## Examples
///
/// ```
/// use tessera_components::{button_groups::button_groups, text::text};
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
    size: ButtonGroupsSize,
    style: ButtonGroupsStyle,
    selection_mode: ButtonGroupsSelectionMode,
    #[prop(skip_setter)] child_closures: Vec<RenderSlot>,
    #[prop(skip_setter)] on_click_closures: Vec<CallbackWith<bool>>,
) {
    let state = remember(ButtonGroupsState::default);
    let layout = ButtonGroupsLayout::new(size, style);
    let child_len = child_closures.len();
    row()
        .modifier(Modifier::new().height(layout.container_height))
        .main_axis_alignment(MainAxisAlignment::Start)
        .children(move || {
            for (index, child_closure) in child_closures.iter().cloned().enumerate() {
                let on_click_closure = on_click_closures[index];
                let item_layout = layout.clone();
                let between_space = layout.between_space;

                let actived =
                    state.with(|s| s.item_states.get(&index).is_some_and(|item| item.actived));
                if actived {
                    button()
                        .filled()
                        .on_click(move || {
                            on_click_closure.call(false);
                            state.with_mut(|s| {
                                let item = s.item_state_mut(index);
                                item.actived = false;
                                item.elastic_state.toggle();
                            });
                        })
                        .shape(item_layout.active_button_shape)
                        .with_child(move || {
                            elastic_container()
                                .state(state)
                                .index(index)
                                .child(move || child_closure.render());
                        });
                } else {
                    let scheme = use_context::<MaterialTheme>()
                        .expect("MaterialTheme must be provided")
                        .get()
                        .color_scheme;
                    let shape = if index == 0 {
                        item_layout.inactive_button_shape_start
                    } else if index == child_len - 1 {
                        item_layout.inactive_button_shape_end
                    } else {
                        item_layout.inactive_button_shape
                    };
                    button()
                        .filled()
                        .on_click(move || {
                            on_click_closure.call(true);
                            state.with_mut(|s| {
                                if selection_mode == ButtonGroupsSelectionMode::Single {
                                    for (other_index, item) in &mut s.item_states {
                                        if *other_index != index && item.actived {
                                            item.actived = false;
                                            item.elastic_state.toggle();
                                        }
                                    }
                                }

                                let item = s.item_state_mut(index);
                                item.actived = true;
                                item.elastic_state.toggle();
                            });
                        })
                        .color(scheme.secondary_container)
                        .shape(shape)
                        .with_child(move || {
                            elastic_container()
                                .state(state)
                                .index(index)
                                .child(move || child_closure.render());
                        });
                }
                if index != child_len - 1 {
                    spacer().modifier(Modifier::new().width(between_space));
                }
            }
        });
}

#[derive(PartialEq)]
struct ElasticState {
    expended: bool,
    last_toggle_frame_nanos: Option<u64>,
    start_progress: f32,
}

impl Default for ElasticState {
    fn default() -> Self {
        Self {
            expended: false,
            last_toggle_frame_nanos: None,
            start_progress: 0.0,
        }
    }
}

impl ElasticState {
    fn toggle(&mut self) {
        let frame_nanos = current_frame_nanos();
        let current_visual_progress = self.calculate_current_progress(frame_nanos);
        self.expended = !self.expended;
        self.last_toggle_frame_nanos = Some(frame_nanos);
        self.start_progress = current_visual_progress;
    }

    fn update(&self, frame_nanos: u64) -> f32 {
        let current_progress = self.calculate_current_progress(frame_nanos);
        if self.expended {
            animation::spring(current_progress, 15.0, 0.35)
        } else {
            animation::easing(current_progress)
        }
    }

    fn calculate_current_progress(&self, frame_nanos: u64) -> f32 {
        let Some(last_toggle_frame_nanos) = self.last_toggle_frame_nanos else {
            return if self.expended { 1.0 } else { 0.0 };
        };

        let elapsed_nanos = frame_nanos.saturating_sub(last_toggle_frame_nanos);
        let elapsed = elapsed_nanos as f32 / 1_000_000_000.0;
        let duration = 0.25;
        let t = (elapsed / duration).clamp(0.0, 1.0);
        let start = self.start_progress;
        let target = if self.expended { 1.0 } else { 0.0 };

        start + (target - start) * t
    }

    fn is_animating(&self, frame_nanos: u64) -> bool {
        self.last_toggle_frame_nanos
            .is_some_and(|last_toggle_frame_nanos| {
                frame_nanos.saturating_sub(last_toggle_frame_nanos) < 250_000_000
            })
    }
}

#[tessera]
fn elastic_container(
    state: Option<tessera_ui::State<ButtonGroupsState>>,
    index: usize,
    child: Option<RenderSlot>,
) {
    let state = state.expect("elastic_container requires state");
    let child = child.expect("elastic_container requires child content");
    let frame_tick = remember(|| 0_u64);
    let _ = frame_tick.with(|tick| *tick);
    let frame_nanos = current_frame_nanos();

    child.render();
    let progress = state.with(|state| {
        state
            .item_state(index)
            .map_or(0.0, |item| item.elastic_state.update(frame_nanos))
    });

    let should_schedule_frame = state.with(|s| {
        s.item_states
            .get(&index)
            .is_some_and(|item| item.elastic_state.is_animating(frame_nanos))
    });
    if should_schedule_frame {
        receive_frame_nanos(move |frame_nanos| {
            frame_tick.with_mut(|tick| *tick = tick.wrapping_add(1));
            let is_animating = state.with(|state| {
                state
                    .item_states
                    .get(&index)
                    .is_some_and(|item| item.elastic_state.is_animating(frame_nanos))
            });
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    layout()
        .layout_policy(ElasticContainerLayout { progress })
        .child(move || {
            child.render();
        });
}

#[derive(Clone, Copy, PartialEq)]
struct ElasticContainerLayout {
    progress: f32,
}

impl LayoutPolicy for ElasticContainerLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let child = input.children()[0];
        let child_size = child.measure_in_parent_constraint(input.parent_constraint())?;
        let additional_width = child_size.width.mul_f32(0.15 * self.progress);
        result.place_child(child, PxPosition::new(additional_width / 2, Px::ZERO));

        Ok(result.with_size(ComputedData {
            width: child_size.width + additional_width,
            height: child_size.height,
        }))
    }
}
