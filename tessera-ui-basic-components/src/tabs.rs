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

pub struct TabsState {
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
    ripple_states: HashMap<usize, Arc<RippleState>>,
}

impl Default for TabsState {
    fn default() -> Self {
        Self::new(0)
    }
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
            ripple_states: Default::default(),
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

    pub fn active_tab(&self) -> usize {
        self.active_tab
    }

    pub fn prev_active_tab(&self) -> usize {
        self.prev_active_tab
    }
}

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

#[tessera]
pub fn tabs<F>(args: TabsArgs, state: Arc<RwLock<TabsState>>, scope_config: F)
where
    F: FnOnce(&mut TabsScope),
{
    let mut tabs = Vec::new();
    let mut scope = TabsScope { tabs: &mut tabs };
    scope_config(&mut scope);

    let num_tabs = tabs.len();
    let active_tab = state.read().active_tab.min(num_tabs.saturating_sub(1));

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
        let ripple_state = state
            .write()
            .ripple_states
            .entry(index)
            .or_insert_with(|| Arc::new(RippleState::new()))
            .clone();
        let state_clone = state.clone();

        let shape = if index == 0 {
            Shape::RoundedRectangle {
                top_left: 25.0,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
                g2_k_value: 3.0,
            }
        } else if index == titles_count - 1 {
            Shape::RoundedRectangle {
                top_left: 0.0,
                top_right: 25.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
                g2_k_value: 3.0,
            }
        } else {
            Shape::RECTANGLE
        };

        button(
            ButtonArgsBuilder::default()
                .color(color)
                .on_click(Arc::new(move || {
                    state_clone.write().set_active_tab(index);
                }))
                .width(DimensionValue::FILLED)
                .shape(shape)
                .build()
                .unwrap(),
            ripple_state,
            child,
        );
    }

    let scroll_offset = {
        let eased_progress = animation::easing(state.read().progress);
        let offset = state.read().content_scroll_offset.0 as f32
            + (state.read().target_content_scroll_offset.0 - state.read().content_scroll_offset.0)
                as f32
                * eased_progress;
        Px(offset as i32)
    };

    tabs_content_container(scroll_offset, content_closures);

    let state_clone = state.clone();
    state_handler(Box::new(move |_| {
        let last_switch_time = state_clone.read().last_switch_time;
        if let Some(last_switch_time) = last_switch_time {
            let elapsed = last_switch_time.elapsed();
            let fraction = (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
            state_clone.write().progress = fraction;
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
            let target_content_scroll_offset = state.read().target_content_scroll_offset;
            if target_content_scroll_offset != target_offset {
                state.write().content_scroll_offset = target_content_scroll_offset;
                state.write().target_content_scroll_offset = target_offset;
            }

            let (indicator_width, indicator_x) = {
                let active_title_width = title_sizes.get(active_tab).map_or(Px(0), |s| s.width);
                let active_title_x: Px = title_sizes
                    .iter()
                    .take(active_tab)
                    .map(|s| s.width)
                    .fold(Px(0), |acc, w| acc + w);

                state.write().indicator_to_width = active_title_width;
                state.write().indicator_to_x = active_title_x;

                let eased_progress = animation::easing(state.read().progress);
                let width = Px((state.read().indicator_from_width.0 as f32
                    + (state.read().indicator_to_width.0 - state.read().indicator_from_width.0)
                        as f32
                        * eased_progress) as i32);
                let x = Px((state.read().indicator_from_x.0 as f32
                    + (state.read().indicator_to_x.0 - state.read().indicator_from_x.0) as f32
                        * eased_progress) as i32);
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
