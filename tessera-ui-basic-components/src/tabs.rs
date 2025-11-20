//! A component for creating a tab-based layout.
//!
//! ## Usage
//!
//! Use to organize content into separate pages that can be switched between.
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Px, PxPosition,
    place_node, tessera,
};

use crate::{
    RippleState, animation,
    button::{ButtonArgsBuilder, button},
    shape_def::Shape,
    surface::{SurfaceArgs, surface},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(250);

fn clamp_wrap(min: Option<Px>, max: Option<Px>, measure: Px) -> Px {
    min.unwrap_or(Px(0))
        .max(measure)
        .min(max.unwrap_or(Px::MAX))
}

fn fill_value(min: Option<Px>, max: Option<Px>, measure: Px) -> Px {
    max.expect("Seems that you are trying to fill an infinite dimension, which is not allowed")
        .max(measure)
        .max(min.unwrap_or(Px(0)))
}

fn resolve_dimension(dim: DimensionValue, measure: Px) -> Px {
    match dim {
        DimensionValue::Fixed(v) => v,
        DimensionValue::Wrap { min, max } => clamp_wrap(min, max, measure),
        DimensionValue::Fill { min, max } => fill_value(min, max, measure),
    }
}

/// Holds the mutable state used by the [`tabs`] component.
///
/// Clone this handle to share it across UI parts. The state tracks the
/// active tab index, previous index, animation progress and cached values used to animate the
/// indicator and content scrolling. The component mutates parts of this state when a tab is
/// switched; callers may also read the active tab via [`TabsState::active_tab`].
struct TabsStateInner {
    active_tab: usize,
    prev_active_tab: usize,
    progress: f32,
    last_switch_time: Option<Instant>,
    indicator_from_width: Px,
    indicator_to_width: Px,
    indicator_from_x: Px,
    indicator_to_x: Px,
    content_scroll_offset: Px,
    target_content_scroll_offset: Px,
    ripple_states: HashMap<usize, RippleState>,
}

impl TabsStateInner {
    fn new(initial_tab: usize) -> Self {
        Self {
            active_tab: initial_tab,
            prev_active_tab: initial_tab,
            progress: 1.0,
            last_switch_time: None,
            indicator_from_width: Px(0),
            indicator_to_width: Px(0),
            indicator_from_x: Px(0),
            indicator_to_x: Px(0),
            content_scroll_offset: Px(0),
            target_content_scroll_offset: Px(0),
            ripple_states: Default::default(),
        }
    }

    /// Set the active tab index and initiate the transition animation.
    ///
    /// If the requested index equals the current active tab this is a no-op.
    /// Otherwise the method updates cached indicator/content positions and resets the animation
    /// progress so the component will animate to the new active tab.
    fn set_active_tab(&mut self, index: usize) {
        if self.active_tab != index {
            self.prev_active_tab = self.active_tab;
            self.active_tab = index;
            self.last_switch_time = Some(Instant::now());
            let eased_progress = animation::easing(self.progress);
            self.indicator_from_width = Px((self.indicator_from_width.0 as f32
                + (self.indicator_to_width.0 - self.indicator_from_width.0) as f32 * eased_progress)
                as i32);
            self.indicator_from_x = Px((self.indicator_from_x.0 as f32
                + (self.indicator_to_x.0 - self.indicator_from_x.0) as f32 * eased_progress)
                as i32);
            self.content_scroll_offset = Px((self.content_scroll_offset.0 as f32
                + (self.target_content_scroll_offset.0 - self.content_scroll_offset.0) as f32
                    * eased_progress) as i32);
            self.progress = 0.0;
        }
    }

    fn ripple_state(&mut self, index: usize) -> RippleState {
        self.ripple_states.entry(index).or_default().clone()
    }
}

#[derive(Clone)]
pub struct TabsState {
    inner: Arc<RwLock<TabsStateInner>>,
}

impl TabsState {
    /// Create a new state with the specified initial active tab.
    pub fn new(initial_tab: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(TabsStateInner::new(initial_tab))),
        }
    }

    pub fn set_active_tab(&self, index: usize) {
        self.inner.write().set_active_tab(index);
    }

    /// Returns the currently active tab index.
    pub fn active_tab(&self) -> usize {
        self.inner.read().active_tab
    }

    /// Returns the previously active tab index (useful during animated transitions).
    pub fn prev_active_tab(&self) -> usize {
        self.inner.read().prev_active_tab
    }

    pub fn last_switch_time(&self) -> Option<Instant> {
        self.inner.read().last_switch_time
    }

    pub fn set_progress(&self, progress: f32) {
        self.inner.write().progress = progress;
    }

    pub fn progress(&self) -> f32 {
        self.inner.read().progress
    }

    pub fn content_offsets(&self) -> (Px, Px) {
        let inner = self.inner.read();
        (
            inner.content_scroll_offset,
            inner.target_content_scroll_offset,
        )
    }

    pub fn update_content_offsets(&self, current: Px, target: Px) {
        let mut inner = self.inner.write();
        inner.content_scroll_offset = current;
        inner.target_content_scroll_offset = target;
    }

    pub fn set_indicator_targets(&self, width: Px, x: Px) {
        let mut inner = self.inner.write();
        inner.indicator_to_width = width;
        inner.indicator_to_x = x;
    }

    pub fn indicator_metrics(&self) -> (Px, Px, Px, Px) {
        let inner = self.inner.read();
        (
            inner.indicator_from_width,
            inner.indicator_to_width,
            inner.indicator_from_x,
            inner.indicator_to_x,
        )
    }

    pub fn ripple_state(&self, index: usize) -> RippleState {
        self.inner.write().ripple_state(index)
    }
}

impl Default for TabsState {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Configuration arguments for the [`tabs`] component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TabsArgs {
    #[builder(default = "Color::new(0.4745, 0.5255, 0.7961, 1.0)")]
    pub indicator_color: Color,
    #[builder(default = "DimensionValue::FILLED")]
    pub width: DimensionValue,
    #[builder(default = "DimensionValue::Wrap { min: None, max: None }")]
    pub height: DimensionValue,
}

impl Default for TabsArgs {
    fn default() -> Self {
        TabsArgsBuilder::default().build().expect("builder construction failed")
    }
}

pub struct TabDef {
    title: Box<dyn FnOnce() + Send + Sync>,
    content: Box<dyn FnOnce() + Send + Sync>,
}

pub struct TabsScope<'a> {
    tabs: &'a mut Vec<TabDef>,
}

impl<'a> TabsScope<'a> {
    pub fn child<F1, F2>(&mut self, title: F1, content: F2)
    where
        F1: FnOnce() + Send + Sync + 'static,
        F2: FnOnce() + Send + Sync + 'static,
    {
        self.tabs.push(TabDef {
            title: Box::new(title),
            content: Box::new(content),
        });
    }
}

#[tessera]
fn tabs_content_container(scroll_offset: Px, children: Vec<Box<dyn FnOnce() + Send + Sync>>) {
    for child in children {
        child();
    }

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            input.enable_clipping();

            let mut max_height = Px(0);
            let container_width = resolve_dimension(input.parent_constraint.width, Px(0));

            for &child_id in input.children_ids.iter() {
                let child_constraint = Constraint::new(
                    DimensionValue::Fixed(container_width),
                    DimensionValue::Wrap {
                        min: None,
                        max: None,
                    },
                );
                let child_size = input.measure_child(child_id, &child_constraint)?;
                max_height = max_height.max(child_size.height);
            }

            let mut current_x = scroll_offset;
            for &child_id in input.children_ids.iter() {
                place_node(child_id, PxPosition::new(current_x, Px(0)), input.metadatas);
                current_x += container_width;
            }

            Ok(ComputedData {
                width: container_width,
                height: max_height,
            })
        },
    ));
}

/// # tabs
///
/// Renders a set of tabs with corresponding content pages.
///
/// ## Usage
///
/// Display a row of tab titles and a content area that switches between different views.
///
/// ## Parameters
///
/// - `args` — configures the tabs' layout and indicator color; see [`TabsArgs`].
/// - `state` — a clonable [`TabsState`] to manage the active tab and animation.
/// - `scope_config` — a closure that receives a [`TabsScope`] for defining each tab's title and content.
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::{
///     tabs::{tabs, TabsArgsBuilder, TabsState},
///     text::{text, TextArgsBuilder},
/// };
///
/// // In a real app, you would manage this state.
/// let tabs_state = TabsState::new(0);
///
/// tabs(
///     TabsArgsBuilder::default().build().expect("builder construction failed"),
///     tabs_state,
///     |scope| {
///         scope.child(
///             || text(TextArgsBuilder::default().text("Tab 1".to_string()).build().expect("builder construction failed")),
///             || text(TextArgsBuilder::default().text("Content for Tab 1").build().expect("builder construction failed"))
///         );
///         scope.child(
///             || text(TextArgsBuilder::default().text("Tab 2".to_string()).build().expect("builder construction failed")),
///             || text(TextArgsBuilder::default().text("Content for Tab 2").build().expect("builder construction failed"))
///         );
///     },
/// );
/// ```
#[tessera]
pub fn tabs<F>(args: TabsArgs, state: TabsState, scope_config: F)
where
    F: FnOnce(&mut TabsScope),
{
    let mut tabs = Vec::new();
    let mut scope = TabsScope { tabs: &mut tabs };
    scope_config(&mut scope);

    let num_tabs = tabs.len();
    let active_tab = state.active_tab().min(num_tabs.saturating_sub(1));

    let (title_closures, content_closures): (Vec<_>, Vec<_>) =
        tabs.into_iter().map(|def| (def.title, def.content)).unzip();

    surface(
        SurfaceArgs {
            style: args.indicator_color.into(),
            width: DimensionValue::FILLED,
            height: DimensionValue::FILLED,
            ..Default::default()
        },
        None,
        || {},
    );

    let titles_count = title_closures.len();
    for (index, child) in title_closures.into_iter().enumerate() {
        let color = if index == active_tab {
            Color::new(0.9, 0.9, 0.9, 1.0) // Active tab color
        } else {
            Color::TRANSPARENT
        };
        let ripple_state = state.ripple_state(index);
        let state_clone = state.clone();

        let shape = if index == 0 {
            Shape::RoundedRectangle {
                top_left: Dp(25.0),
                top_right: Dp(0.0),
                bottom_right: Dp(0.0),
                bottom_left: Dp(0.0),
                g2_k_value: 3.0,
            }
        } else if index == titles_count - 1 {
            Shape::RoundedRectangle {
                top_left: Dp(0.0),
                top_right: Dp(25.0),
                bottom_right: Dp(0.0),
                bottom_left: Dp(0.0),
                g2_k_value: 3.0,
            }
        } else {
            Shape::RECTANGLE
        };

        button(
            ButtonArgsBuilder::default()
                .color(color)
                .on_click({
                    let state_clone = state_clone.clone();
                    Arc::new(move || {
                        state_clone.set_active_tab(index);
                    })
                })
                .width(DimensionValue::FILLED)
                .shape(shape)
                .build().expect("builder construction failed"),
            ripple_state,
            child,
        );
    }

    let scroll_offset = {
        let eased_progress = animation::easing(state.progress());
        let (content_offset, target_offset) = state.content_offsets();
        let offset =
            content_offset.0 as f32 + (target_offset.0 - content_offset.0) as f32 * eased_progress;
        Px(offset as i32)
    };

    tabs_content_container(scroll_offset, content_closures);

    let state_clone = state.clone();
    input_handler(Box::new(move |_| {
        if let Some(last_switch_time) = state_clone.last_switch_time() {
            let elapsed = last_switch_time.elapsed();
            let fraction = (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
            state_clone.set_progress(fraction);
        }
    }));

    let tabs_args = args.clone();

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            let tabs_intrinsic_constraint = Constraint::new(tabs_args.width, tabs_args.height);
            let tabs_effective_constraint =
                tabs_intrinsic_constraint.merge(input.parent_constraint);

            let tab_effective_width = Constraint {
                width: {
                    match tabs_effective_constraint.width {
                        DimensionValue::Fixed(v) => DimensionValue::Fixed(v / num_tabs as i32),
                        DimensionValue::Wrap { min, max } => {
                            let max = max.map(|v| v / num_tabs as i32);
                            DimensionValue::Wrap { min, max }
                        }
                        DimensionValue::Fill { min, max } => {
                            let max = max.map(|v| v / num_tabs as i32);
                            DimensionValue::Fill { min, max }
                        }
                    }
                },
                height: tabs_effective_constraint.height,
            };

            let indicator_id = input.children_ids[0];
            let title_ids = &input.children_ids[1..=num_tabs];
            let content_container_id = input.children_ids[num_tabs + 1];

            let title_constraints: Vec<_> = title_ids
                .iter()
                .map(|&id| (id, tab_effective_width))
                .collect();
            let title_results = input.measure_children(title_constraints)?;

            let mut title_sizes = Vec::with_capacity(num_tabs);
            let mut titles_total_width = Px(0);
            let mut titles_max_height = Px(0);
            for &title_id in title_ids {
                if let Some(result) = title_results.get(&title_id) {
                    title_sizes.push(*result);
                    titles_total_width += result.width;
                    titles_max_height = titles_max_height.max(result.height);
                }
            }

            let content_container_constraint = Constraint::new(
                DimensionValue::Fill {
                    min: None,
                    max: Some(titles_total_width),
                },
                DimensionValue::Wrap {
                    min: None,
                    max: None,
                },
            );
            let content_container_size =
                input.measure_child(content_container_id, &content_container_constraint)?;

            let final_width = titles_total_width;
            let target_offset = -Px(active_tab as i32 * final_width.0);
            let (_, target_content_scroll_offset) = state.content_offsets();
            if target_content_scroll_offset != target_offset {
                state.update_content_offsets(target_content_scroll_offset, target_offset);
            }

            let (indicator_width, indicator_x) = {
                let active_title_width = title_sizes.get(active_tab).map_or(Px(0), |s| s.width);
                let active_title_x: Px = title_sizes
                    .iter()
                    .take(active_tab)
                    .map(|s| s.width)
                    .fold(Px(0), |acc, w| acc + w);

                state.set_indicator_targets(active_title_width, active_title_x);

                let (from_width, to_width, from_x, to_x) = state.indicator_metrics();
                let eased_progress = animation::easing(state.progress());
                let width = Px((from_width.0 as f32
                    + (to_width.0 - from_width.0) as f32 * eased_progress)
                    as i32);
                let x = Px((from_x.0 as f32 + (to_x.0 - from_x.0) as f32 * eased_progress) as i32);
                (width, x)
            };

            let indicator_height = Dp(2.0).into();
            let indicator_constraint = Constraint::new(
                DimensionValue::Fixed(indicator_width),
                DimensionValue::Fixed(indicator_height),
            );
            let _ = input.measure_child(indicator_id, &indicator_constraint)?;

            let final_width = titles_total_width;
            let final_height = titles_max_height + content_container_size.height;

            let mut current_x = Px(0);
            for (i, &title_id) in title_ids.iter().enumerate() {
                place_node(title_id, PxPosition::new(current_x, Px(0)), input.metadatas);
                if let Some(title_size) = title_sizes.get(i) {
                    current_x += title_size.width;
                }
            }

            place_node(
                indicator_id,
                PxPosition::new(indicator_x, titles_max_height),
                input.metadatas,
            );

            place_node(
                content_container_id,
                PxPosition::new(Px(0), titles_max_height),
                input.metadatas,
            );

            Ok(ComputedData {
                width: final_width,
                height: final_height,
            })
        },
    ));
}


