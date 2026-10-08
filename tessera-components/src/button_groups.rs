//! Material 3 button groups for related actions.
//!
//! ## Usage
//!
//! Used for grouping related actions.

use std::collections::HashMap;

use tessera_ui::{
    CallbackWith, ComputedData, Dp, LayoutResult, MeasurementError, Modifier, Px, PxPosition,
    RenderSlot, current_frame_nanos, key,
    layout::{MeasureScope, layout},
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::MainAxisAlignment,
    animation::Animatable,
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

    #[cfg(test)]
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
    size: Option<ButtonGroupsSize>,
    style: Option<ButtonGroupsStyle>,
    selection_mode: Option<ButtonGroupsSelectionMode>,
    #[prop(skip_setter)] child_closures: Option<Vec<RenderSlot>>,
    #[prop(skip_setter)] on_click_closures: Option<Vec<CallbackWith<bool>>>,
) {
    let size = size.unwrap_or_default();
    let style = style.unwrap_or_default();
    let selection_mode = selection_mode.unwrap_or_default();
    let child_closures = child_closures.unwrap_or_default();
    let on_click_closures = on_click_closures.unwrap_or_default();
    let state = remember(ButtonGroupsState::default);
    let layout = ButtonGroupsLayout::new(size, style);
    let child_len = child_closures.len();
    row()
        .modifier(Modifier::new().height(layout.container_height))
        .main_axis_alignment(MainAxisAlignment::Start)
        .children(move || {
            for (index, child_closure) in child_closures.iter().cloned().enumerate() {
                key(index, || {
                    let on_click_closure = on_click_closures[index];
                    let item_layout = layout.clone();
                    let between_space = layout.between_space;

                    let actived =
                        state.with(|s| s.item_states.get(&index).is_some_and(|item| item.actived));
                    // The elastic container wraps the whole button so the
                    // expansion applies to the pill itself. Wrapping only the
                    // button content would be swallowed by the button's min
                    // width, leaving the group visually unchanged.
                    elastic_container()
                        .state(state)
                        .index(index)
                        .child(move || {
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
                                    .child(move || child_closure.render());
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
                                    .child(move || child_closure.render());
                            }
                        });
                    if index != child_len - 1 {
                        spacer().modifier(Modifier::new().width(between_space));
                    }
                });
            }
        });
}

#[derive(PartialEq)]
struct ElasticState {
    expended: bool,
    /// Previous sample from the frame callback. `dt` is the gap to the next
    /// one.
    last_frame_nanos: Option<u64>,
    motion: Animatable,
}

impl Default for ElasticState {
    fn default() -> Self {
        Self {
            expended: false,
            last_frame_nanos: None,
            motion: Animatable::easing(0.0, ELASTIC_EASE_DURATION_SECS),
        }
    }
}

const ELASTIC_EASE_DURATION_SECS: f32 = 0.25;
// Natural frequency in rad/s. Tuned so the pop settles in well under half a
// second instead of slowly ringing out.
const ELASTIC_SPRING_STIFFNESS: f32 = 60.0;
const ELASTIC_SPRING_DAMPING: f32 = 0.35;
// Extra width the selected button grows into while expanding.
const ELASTIC_EXPAND_DP: Dp = Dp(12.0);

impl ElasticState {
    fn toggle(&mut self) {
        let frame_nanos = current_frame_nanos();
        self.advance_to(frame_nanos);
        self.expended = !self.expended;
        let value = self.motion.value();
        let carried_velocity = self.motion.velocity();
        if self.expended {
            self.motion =
                Animatable::spring(value, ELASTIC_SPRING_STIFFNESS, ELASTIC_SPRING_DAMPING)
                    .with_velocity(carried_velocity);
            self.motion.set_target(1.0);
        } else {
            self.motion = Animatable::easing(value, ELASTIC_EASE_DURATION_SECS);
            self.motion.set_target(0.0);
        }
        self.last_frame_nanos = Some(frame_nanos);
    }

    fn update(&mut self, frame_nanos: u64) -> f32 {
        self.advance_to(frame_nanos);
        self.motion.value()
    }

    fn advance_to(&mut self, frame_nanos: u64) {
        let Some(last_frame_nanos) = self.last_frame_nanos else {
            self.last_frame_nanos = Some(frame_nanos);
            return;
        };
        if frame_nanos <= last_frame_nanos {
            return;
        }
        let dt = (frame_nanos - last_frame_nanos) as f32 / 1_000_000_000.0;
        self.motion.tick(dt);
        self.last_frame_nanos = Some(frame_nanos);
    }

    fn is_animating(&self) -> bool {
        !self.motion.is_idle()
    }
}

#[tessera]
fn elastic_container(
    state: Option<tessera_ui::State<ButtonGroupsState>>,
    index: Option<usize>,
    child: Option<RenderSlot>,
) {
    let state = state.expect("elastic_container requires state");
    let index = index.unwrap_or(0);
    let child = child.expect("elastic_container requires child content");
    // The frame receiver only bumps this counter; the build below advances the
    // motion. Reading it here makes the component rebuild every delivered
    // frame, which keeps the animation running even when its parent subtree
    // is reused.
    let frame_tick = remember(|| 0_u64);
    let _ = frame_tick.with(|tick| *tick);
    let frame_nanos = current_frame_nanos();
    let needs_tick = state.with(|state| {
        state.item_states.get(&index).is_some_and(|item| {
            item.elastic_state.is_animating()
                && item.elastic_state.last_frame_nanos != Some(frame_nanos)
        })
    });
    let progress = if needs_tick {
        state.with_mut(|state| {
            state
                .item_states
                .get_mut(&index)
                .map_or(0.0, |item| item.elastic_state.update(frame_nanos))
        })
    } else {
        state.with(|state| {
            state
                .item_states
                .get(&index)
                .map_or(0.0, |item| item.elastic_state.motion.value())
        })
    };
    let is_animating = state.with(|state| {
        state
            .item_states
            .get(&index)
            .is_some_and(|item| item.elastic_state.is_animating())
    });
    if is_animating {
        receive_frame_nanos(move |_frame_nanos| {
            frame_tick.with_mut(|tick| *tick = tick.wrapping_add(1));
            let still_animating = state.with(|state| {
                state
                    .item_states
                    .get(&index)
                    .is_some_and(|item| item.elastic_state.is_animating())
            });
            if still_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    layout()
        .policy(ElasticContainerLayout { progress })
        .child(move || {
            child.render();
        });
}

#[derive(Clone, Copy, PartialEq)]
struct ElasticContainerLayout {
    progress: f32,
}

impl tessera_ui::LayoutModifierNode for ElasticContainerLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let child = input.children()[0];
        let child_constraint = input.parent_constraint().without_min();
        let child_size = child.measure(input, &child_constraint)?;
        let additional_width =
            Px::new((ELASTIC_EXPAND_DP.to_pixels_f32() * self.progress).round() as i32);
        result.place_child(child, PxPosition::new(additional_width / 2, Px::ZERO));

        Ok(result.with_size(ComputedData {
            width: child_size.width + additional_width,
            height: child_size.height,
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use tessera_ui::{
        CursorEventContent, Dp, GestureState, Modifier, PointerChange, PressKeyEventType, Px,
        PxPosition, key, layout::layout, remember, tessera, time::Instant,
    };

    use crate::{
        lazy_list::lazy_column,
        modifier::{ModifierExt as _, SemanticsArgs},
        row::row,
        spacer::spacer,
        text::text,
        theme::{MaterialTheme, material_theme},
    };

    use super::{
        Animatable, ButtonGroupsState, ELASTIC_SPRING_DAMPING, ELASTIC_SPRING_STIFFNESS,
        ElasticState, button_groups, elastic_container,
    };

    thread_local! {
        static IDLE_BUILDS: Cell<usize> = const { Cell::new(0) };
        static IDLE_STATE: RefCell<Option<tessera_ui::State<ButtonGroupsState>>> =
            const { RefCell::new(None) };
    }

    #[tessera]
    fn idle_elastic_probe() {
        let state = remember(ButtonGroupsState::default);
        IDLE_STATE.with(|slot| *slot.borrow_mut() = Some(state));
        layout().child(move || {
            IDLE_BUILDS.with(|builds| builds.set(builds.get() + 1));
            let _ = state.with(|state| state.item_states.len());
            elastic_container().state(state).index(0).child(|| {
                elastic_probe_box();
            });
        });
    }

    #[test]
    fn elastic_container_stops_rebuilding_at_rest() {
        IDLE_BUILDS.with(|builds| builds.set(0));
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(|| {
                idle_elastic_probe();
            })
            .viewport_px(200, 100),
        );
        for frame in 0..4 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        let initial_builds = IDLE_BUILDS.with(Cell::get);
        for frame in 4..12 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert_eq!(IDLE_BUILDS.with(Cell::get), initial_builds);

        let state = IDLE_STATE.with(|slot| slot.borrow().expect("button group state"));
        state.with_mut(|state| state.item_state_mut(0).elastic_state.toggle());
        for frame in 12..120 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert!(!state.with(|state| state.item_states[&0].elastic_state.is_animating()));
        let settled_builds = IDLE_BUILDS.with(Cell::get);
        assert!(settled_builds > initial_builds);
        for frame in 120..128 {
            let _ = tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_666_667);
        }
        assert_eq!(IDLE_BUILDS.with(Cell::get), settled_builds);
    }

    #[tessera]
    fn elastic_probe_box() {
        spacer().modifier(Modifier::new().width(Dp(40.0)).height(Dp(20.0)).semantics(
            SemanticsArgs {
                test_tag: Some("probe".to_string()),
                ..Default::default()
            },
        ));
    }

    #[tessera]
    fn elastic_probe() {
        let state = remember(|| {
            let mut state = ButtonGroupsState::default();
            let item = state.item_state_mut(0);
            item.actived = true;
            item.elastic_state.expended = true;
            item.elastic_state.last_frame_nanos = Some(0);
            item.elastic_state.motion =
                Animatable::spring(0.0, ELASTIC_SPRING_STIFFNESS, ELASTIC_SPRING_DAMPING);
            item.elastic_state.motion.set_target(1.0);
            state
        });
        elastic_container().state(state).index(0).child(|| {
            elastic_probe_box();
        });
    }

    #[tessera]
    fn nested_elastic_probe() {
        let state = remember(|| {
            let mut state = ButtonGroupsState::default();
            let item = state.item_state_mut(0);
            item.actived = true;
            item.elastic_state.expended = true;
            item.elastic_state.last_frame_nanos = Some(0);
            item.elastic_state.motion =
                Animatable::spring(0.0, ELASTIC_SPRING_STIFFNESS, ELASTIC_SPRING_DAMPING);
            item.elastic_state.motion.set_target(1.0);
            state
        });
        row().children(move || {
            key(0, || {
                elastic_container().state(state).index(0).child(|| {
                    elastic_probe_box();
                });
            });
        });
    }

    #[test]
    fn nested_elastic_probe_settles_without_jitter() {
        tessera_ui::assert_layout! {
            viewport: (200, 100),
            content: {
                nested_elastic_probe();
            },
            expect: {
                0 => { node("probe").x(0); },
                250_000_000 => {},
                500_000_000 => {},
                750_000_000 => { node("probe").x(6); },
                1_000_000_000 => { node("probe").x(6); },
            }
        }
    }
    #[tessera]
    fn labelled_box(tag: Option<String>, label: Option<String>) {
        let label = label.unwrap_or_default();
        layout()
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(tag.unwrap_or_default()),
                ..Default::default()
            }))
            .child(move || {
                text().content(label.clone());
            });
    }

    #[tessera]
    fn labelled_button_groups_case() {
        material_theme()
            .theme(|| MaterialTheme::default())
            .child(|| {
                button_groups()
                    .child(
                        || {
                            labelled_box()
                                .tag("one".to_string())
                                .label("One".to_string());
                        },
                        |_| {},
                    )
                    .child(
                        || {
                            labelled_box()
                                .tag("two".to_string())
                                .label("Two".to_string());
                        },
                        |_| {},
                    )
                    .child(
                        || {
                            labelled_box()
                                .tag("three".to_string())
                                .label("Three".to_string());
                        },
                        |_| {},
                    );
            });
    }

    #[test]
    fn button_groups_text_content_pops() {
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(|| {
                labelled_button_groups_case();
            })
            .viewport_px(400, 120),
        );
        let first = tessera_ui::testing::pump_layout_test_session(&mut session, 0);
        first.node("one").x(16);
        first.node("one").width(26);
        first.node("two").x(83);
        first.node("two").width(25);
        first.node("three").x(144);

        let now = Instant::now();
        let cursor = Some(PxPosition::new(Px::new(29), Px::new(28)));
        let press = PointerChange::new(
            now,
            1,
            CursorEventContent::Pressed(PressKeyEventType::Left),
            GestureState::TapCandidate,
        );
        let _ = tessera_ui::testing::pump_layout_test_session_with_input(
            &mut session,
            16_666_667,
            vec![press],
            cursor,
        );
        let release = PointerChange::new(
            now,
            1,
            CursorEventContent::Released(PressKeyEventType::Left),
            GestureState::TapCandidate,
        );
        let mut snap = tessera_ui::testing::pump_layout_test_session_with_input(
            &mut session,
            33_333_334,
            vec![release],
            cursor,
        );
        for frame in 3..40u64 {
            snap = tessera_ui::testing::pump_layout_test_session(
                &mut session,
                33_333_334 + 16_666_667 * (frame - 2),
            );
        }
        snap.node("three").x(156);
    }

    #[tessera]
    fn lazy_button_groups_case() {
        material_theme()
            .theme(|| MaterialTheme::default())
            .child(|| {
                lazy_column().item(move || {
                    labelled_button_groups_case();
                });
            });
    }

    #[test]
    fn button_groups_text_content_pops_in_lazy_column() {
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(|| {
                lazy_button_groups_case();
            })
            .viewport_px(400, 120),
        );
        let first = tessera_ui::testing::pump_layout_test_session(&mut session, 0);
        first.node("one").width(26);
        first.node("two").x(83);
        first.node("three").x(144);

        let now = Instant::now();
        let cursor = Some(PxPosition::new(Px::new(29), Px::new(28)));
        let press = PointerChange::new(
            now,
            1,
            CursorEventContent::Pressed(PressKeyEventType::Left),
            GestureState::TapCandidate,
        );
        let _ = tessera_ui::testing::pump_layout_test_session_with_input(
            &mut session,
            16_666_667,
            vec![press],
            cursor,
        );
        let release = PointerChange::new(
            now,
            1,
            CursorEventContent::Released(PressKeyEventType::Left),
            GestureState::TapCandidate,
        );
        let mut snap = tessera_ui::testing::pump_layout_test_session_with_input(
            &mut session,
            33_333_334,
            vec![release],
            cursor,
        );
        for frame in 3..40u64 {
            snap = tessera_ui::testing::pump_layout_test_session(
                &mut session,
                33_333_334 + 16_666_667 * (frame - 2),
            );
        }
        snap.node("three").x(156);
    }

    #[test]
    fn elastic_container_settles_without_jitter() {
        tessera_ui::assert_layout! {
            viewport: (200, 100),
            content: {
                elastic_probe();
            },
            expect: {
                0 => { node("probe").x(0); },
                250_000_000 => {},
                500_000_000 => {},
                750_000_000 => { node("probe").x(6); },
                1_000_000_000 => { node("probe").x(6); },
            }
        }
    }
    #[test]
    fn button_group_state_is_independent_per_item() {
        let mut state = ButtonGroupsState::default();
        state.item_state_mut(0).actived = true;
        state.item_state_mut(1).actived = false;

        assert!(state.item_state(0).is_some_and(|item| item.actived));
        assert!(state.item_state(1).is_some_and(|item| !item.actived));
    }

    #[test]
    fn elastic_expand_keeps_velocity_when_spring_target_changes() {
        let mut elastic = ElasticState {
            expended: true,
            last_frame_nanos: Some(0),
            motion: Animatable::spring(0.0, ELASTIC_SPRING_STIFFNESS, ELASTIC_SPRING_DAMPING),
        };
        elastic.motion.set_target(1.0);

        let step_nanos = 16_666_667_u64;
        for frame in 1..=8 {
            elastic.update(step_nanos * frame);
        }
        let velocity_before = elastic.motion.velocity();
        let value_before = elastic.motion.value();
        assert!(
            velocity_before > 0.05,
            "expected carried expand velocity, got {velocity_before}"
        );

        elastic.motion.set_target(0.0);
        assert_eq!(elastic.motion.velocity(), velocity_before);
        assert_eq!(elastic.motion.value(), value_before);

        elastic.update(step_nanos * 9);
        let velocity_after = elastic.motion.velocity();
        assert!(velocity_after < velocity_before, "got {velocity_after}");
        assert!(velocity_after.abs() > 1.0e-6);
    }

    #[test]
    fn elastic_collapse_eases_without_spring_velocity() {
        let mut elastic = ElasticState::default();
        elastic.toggle();
        assert!(elastic.expended);
        let origin = elastic
            .last_frame_nanos
            .expect("toggle records the frame clock");
        elastic.update(origin.saturating_add(50_000_000));
        assert!(elastic.motion.velocity().abs() > 0.0);

        elastic.toggle();
        assert!(!elastic.expended);
        assert_eq!(elastic.motion.velocity(), 0.0);
        assert_eq!(elastic.motion.target(), 0.0);

        let origin = elastic
            .last_frame_nanos
            .expect("collapse retarget records the frame clock");
        elastic.update(origin.saturating_add(250_000_000));
        assert!((elastic.motion.value() - 0.0).abs() < 1.0e-4);
        assert!(elastic.motion.velocity().abs() < 1.0e-6);
        assert!(!elastic.is_animating());
    }

    #[test]
    fn elastic_expand_settles_promptly() {
        let mut elastic = ElasticState::default();
        elastic.toggle();

        let step_nanos = 16_666_667_u64;
        let mut frame = 0_u64;
        while elastic.is_animating() {
            frame += 1;
            assert!(frame < 120, "elastic pop kept ringing for {frame} frames");
            elastic.update(step_nanos * frame);
        }

        assert!((elastic.motion.value() - 1.0).abs() < 1.0e-4);
    }
}
