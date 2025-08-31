use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::Mutex;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Px, PxPosition,
    place_node, tessera,
};

use crate::{
    animation,
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

pub struct TabsState {
    pub active_tab: usize,
    pub prev_active_tab: usize,
    progress: f32,
    last_switch_time: Option<Instant>,
    indicator_from_width: Px,
    indicator_to_width: Px,
    indicator_from_x: Px,
    indicator_to_x: Px,
    content_scroll_offset: Px,
    target_content_scroll_offset: Px,
}

impl TabsState {
    pub fn new(initial_tab: usize) -> Self {
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
        }
    }

    pub fn set_active_tab(&mut self, index: usize) {
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
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TabsArgs {
    #[builder(default = "0")]
    pub active_tab: usize,
    #[builder(default = "Color::new(0.4745, 0.5255, 0.7961, 1.0)")]
    pub indicator_color: Color,
    #[builder(default)]
    pub state: Option<Arc<Mutex<TabsState>>>,
    #[builder(default = "DimensionValue::FILLED")]
    pub width: DimensionValue,
    #[builder(default = "DimensionValue::Wrap { min: None, max: None }")]
    pub height: DimensionValue,
}

impl Default for TabsArgs {
    fn default() -> Self {
        TabsArgsBuilder::default().build().unwrap()
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
    pub fn tab<F1, F2>(&mut self, title: F1, content: F2)
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
fn tabs_content_container<F>(scroll_offset: Px, children: F)
where
    F: FnOnce(),
{
    children();

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

#[tessera]
pub fn tabs<F>(args: TabsArgs, scope_config: F)
where
    F: FnOnce(&mut TabsScope),
{
    let mut tabs = Vec::new();
    let mut scope = TabsScope { tabs: &mut tabs };
    scope_config(&mut scope);

    let num_tabs = tabs.len();
    let active_tab = if let Some(state) = &args.state {
        state.lock().active_tab
    } else {
        args.active_tab
    }
    .min(num_tabs.saturating_sub(1));

    let (title_closures, content_closures): (Vec<_>, Vec<_>) =
        tabs.into_iter().map(|def| (def.title, def.content)).unzip();

    surface(
        SurfaceArgs {
            color: args.indicator_color,
            width: Some(DimensionValue::FILLED),
            height: Some(DimensionValue::FILLED),
            ..Default::default()
        },
        None,
        || {},
    );

    for title in title_closures {
        title();
    }

    let scroll_offset = if let Some(state) = &args.state {
        let state = state.lock();
        let eased_progress = animation::easing(state.progress);
        let offset = state.content_scroll_offset.0 as f32
            + (state.target_content_scroll_offset.0 - state.content_scroll_offset.0) as f32
                * eased_progress;
        Px(offset as i32)
    } else {
        Px(0)
    };

    tabs_content_container(scroll_offset, move || {
        for content in content_closures {
            content();
        }
    });

    if let Some(state) = &args.state {
        let state = state.clone();
        state_handler(Box::new(move |_| {
            let mut state = state.lock();
            if let Some(last_switch_time) = state.last_switch_time {
                let elapsed = last_switch_time.elapsed();
                let fraction = (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
                state.progress = fraction;
            }
        }));
    }

    let state = args.state.clone();
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

            if let Some(state) = &state {
                let mut state = state.lock();
                let final_width = titles_total_width;
                let target_offset = -Px(active_tab as i32 * final_width.0);
                if state.target_content_scroll_offset != target_offset {
                    state.content_scroll_offset = state.target_content_scroll_offset;
                    state.target_content_scroll_offset = target_offset;
                }
            }

            let (indicator_width, indicator_x) = if let Some(state) = &state {
                let mut state = state.lock();
                let active_title_width = title_sizes.get(active_tab).map_or(Px(0), |s| s.width);
                let active_title_x: Px = title_sizes
                    .iter()
                    .take(active_tab)
                    .map(|s| s.width)
                    .fold(Px(0), |acc, w| acc + w);

                state.indicator_to_width = active_title_width;
                state.indicator_to_x = active_title_x;

                let eased_progress = animation::easing(state.progress);
                let width = Px((state.indicator_from_width.0 as f32
                    + (state.indicator_to_width.0 - state.indicator_from_width.0) as f32
                        * eased_progress) as i32);
                let x = Px((state.indicator_from_x.0 as f32
                    + (state.indicator_to_x.0 - state.indicator_from_x.0) as f32 * eased_progress)
                    as i32);
                (width, x)
            } else {
                let active_title_width = title_sizes.get(active_tab).map_or(Px(0), |s| s.width);
                let active_title_x: Px = title_sizes
                    .iter()
                    .take(active_tab)
                    .map(|s| s.width)
                    .fold(Px(0), |acc, w| acc + w);
                (active_title_width, active_title_x)
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
