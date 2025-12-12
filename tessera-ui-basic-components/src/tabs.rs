//! A component for creating a tab-based layout.
//!
//! ## Usage
//!
//! Use to organize content into separate pages that can be switched between.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Px, PxPosition, State,
    remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    animation,
    boxed::{BoxedArgs, boxed},
    button::{ButtonArgsBuilder, button},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgs, surface},
    theme::MaterialColorScheme,
};

const ANIMATION_DURATION: Duration = Duration::from_millis(300);

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

fn clamp_px(value: Px, min: Px, max: Option<Px>) -> Px {
    let clamped_max = max.unwrap_or(value);
    Px(value.0.max(min.0).min(clamped_max.0))
}

fn resolve_dimension(dim: DimensionValue, measure: Px) -> Px {
    match dim {
        DimensionValue::Fixed(v) => v,
        DimensionValue::Wrap { min, max } => clamp_wrap(min, max, measure),
        DimensionValue::Fill { min, max } => fill_value(min, max, measure),
    }
}

fn blend_state_layer(base: Color, layer: Color, opacity: f32) -> Color {
    let opacity = opacity.clamp(0.0, 1.0);
    Color {
        r: base.r * (1.0 - opacity) + layer.r * opacity,
        g: base.g * (1.0 - opacity) + layer.g * opacity,
        b: base.b * (1.0 - opacity) + layer.b * opacity,
        a: base.a,
    }
}

/// Controller for the `tabs` component.
///
/// Tracks the active tab index, previous index, animation progress and cached
/// values used to animate the indicator and content scrolling.
#[derive(Clone)]
pub struct TabsController {
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
}

impl TabsController {
    /// Create a new state with the specified initial active tab.
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

    /// Set the active tab index and initiate the transition animation.
    ///
    /// If the requested index equals the current active tab this is a no-op.
    /// Otherwise the method updates cached indicator/content positions and
    /// resets the animation progress so the component will animate to the
    /// new active tab.
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

    /// Returns the currently active tab index.
    pub fn active_tab(&self) -> usize {
        self.active_tab
    }

    fn last_switch_time(&self) -> Option<Instant> {
        self.last_switch_time
    }

    fn set_progress(&mut self, progress: f32) {
        self.progress = progress;
    }

    fn progress(&self) -> f32 {
        self.progress
    }

    fn content_offsets(&self) -> (Px, Px) {
        (
            self.content_scroll_offset,
            self.target_content_scroll_offset,
        )
    }

    fn update_content_offsets(&mut self, current: Px, target: Px) {
        self.content_scroll_offset = current;
        self.target_content_scroll_offset = target;
    }

    fn set_indicator_targets(&mut self, width: Px, x: Px) {
        self.indicator_to_width = width;
        self.indicator_to_x = x;
    }

    fn indicator_metrics(&self) -> (Px, Px, Px, Px) {
        (
            self.indicator_from_width,
            self.indicator_to_width,
            self.indicator_from_x,
            self.indicator_to_x,
        )
    }
}

impl Default for TabsController {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Configuration arguments for the [`tabs`] component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TabsArgs {
    /// Initial active tab index (0-based). Ignored if a controller is provided
    /// with its own state.
    #[builder(default = "0")]
    pub initial_active_tab: usize,
    /// Color of the active tab indicator.
    #[builder(default = "use_context::<MaterialColorScheme>().get().primary")]
    // Material primary tone
    pub indicator_color: Color,
    /// Background color for the tab row container.
    #[builder(default = "use_context::<MaterialColorScheme>().get().surface")]
    pub container_color: Color,
    /// Color applied to active tab titles (Material on-surface).
    #[builder(default = "use_context::<MaterialColorScheme>().get().on_surface")]
    pub active_content_color: Color,
    /// Color applied to inactive tab titles (Material on-surface-variant).
    #[builder(default = "use_context::<MaterialColorScheme>().get().on_surface_variant")]
    pub inactive_content_color: Color,
    /// Height of the indicator bar in density-independent pixels.
    #[builder(default = "Dp(3.0)")]
    pub indicator_height: Dp,
    /// Minimum width for the indicator bar.
    #[builder(default = "Dp(24.0)")]
    pub indicator_min_width: Dp,
    /// Optional maximum width for the indicator bar.
    #[builder(default = "Some(Dp(64.0))")]
    pub indicator_max_width: Option<Dp>,
    /// Minimum height for a tab (Material spec uses 48dp).
    #[builder(default = "Dp(48.0)")]
    pub min_tab_height: Dp,
    /// Internal padding for each tab, applied symmetrically.
    #[builder(default = "Dp(12.0)")]
    pub tab_padding: Dp,
    /// Color used for hover/pressed state layers.
    #[builder(default = "use_context::<MaterialColorScheme>().get().on_surface")]
    pub state_layer_color: Color,
    /// Opacity applied to the state layer on hover.
    #[builder(default = "0.08")]
    pub hover_state_layer_opacity: f32,
    /// Width behavior for the entire tabs container.
    #[builder(default = "DimensionValue::FILLED")]
    pub width: DimensionValue,
    /// Height behavior for the tabs container.
    #[builder(default = "DimensionValue::Wrap { min: None, max: None }")]
    pub height: DimensionValue,
}

impl Default for TabsArgs {
    fn default() -> Self {
        TabsArgsBuilder::default()
            .build()
            .expect("builder construction failed")
    }
}

struct TabDef {
    title: TabTitle,
    content: Box<dyn FnOnce() + Send + Sync>,
}

enum TabTitle {
    Custom(Box<dyn FnOnce() + Send + Sync>),
    Themed(Box<dyn FnOnce(Color) + Send + Sync>),
}

/// Scope passed to tab configuration closures.
pub struct TabsScope<'a> {
    tabs: &'a mut Vec<TabDef>,
}

impl<'a> TabsScope<'a> {
    /// Adds a tab with its title and content builders.
    pub fn child<F1, F2>(&mut self, title: F1, content: F2)
    where
        F1: FnOnce() + Send + Sync + 'static,
        F2: FnOnce() + Send + Sync + 'static,
    {
        self.tabs.push(TabDef {
            title: TabTitle::Custom(Box::new(title)),
            content: Box::new(content),
        });
    }

    /// Adds a tab whose title closure receives the resolved content color
    /// (active/inactive).
    pub fn child_with_color<F1, F2>(&mut self, title: F1, content: F2)
    where
        F1: FnOnce(Color) + Send + Sync + 'static,
        F2: FnOnce() + Send + Sync + 'static,
    {
        self.tabs.push(TabDef {
            title: TabTitle::Themed(Box::new(title)),
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
                input.place_child(child_id, PxPosition::new(current_x, Px(0)));
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
/// Display a row of tab titles and a content area that switches between
/// different views.
///
/// ## Parameters
///
/// - `args` — configures the tabs' layout, initial active tab, and indicator
///   color; see [`TabsArgs`].
/// - `scope_config` — a closure that receives a [`TabsScope`] for defining each
///   tab's title and content. Use [`TabsScope::child_with_color`] to let the
///   component supply Material-compliant active/inactive colors.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{Dp, tessera};
/// use tessera_ui_basic_components::{
///     tabs::{TabsArgsBuilder, tabs},
///     text::{TextArgsBuilder, text},
/// };
///
/// #[tessera]
/// fn demo() {
///     tabs(
///         TabsArgsBuilder::default()
///             .initial_active_tab(1)
///             .build()
///             .expect("builder construction failed"),
///         |scope| {
///             scope.child_with_color(
///                 |color| {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Flights".to_string())
///                             .color(color)
///                             .size(Dp(14.0))
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///                 || {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Content for Flights")
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///             );
///             scope.child_with_color(
///                 |color| {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Hotel".to_string())
///                             .color(color)
///                             .size(Dp(14.0))
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///                 || {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Content for Hotel")
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///             );
///         },
///     );
/// }
/// ```
#[tessera]
pub fn tabs<F>(args: TabsArgs, scope_config: F)
where
    F: FnOnce(&mut TabsScope),
{
    let controller = remember(|| TabsController::new(args.initial_active_tab));
    tabs_with_controller(args, controller, scope_config);
}

/// # tabs_with_controller
///
/// Controlled variant that accepts an explicit controller.
///
/// ## Usage
///
/// Use when you need to synchronize active tab selection across components or
/// restore selection after remounts.
///
/// ## Parameters
///
/// - `args` — configures the tabs' layout and indicator color; see
///   [`TabsArgs`].
/// - `controller` — a [`TabsController`] storing the active tab index and
///   animation progress.
/// - `scope_config` — a closure that receives a [`TabsScope`] for defining each
///   tab's title and content.
///
/// ## Examples
///
/// ```
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::{
///     tabs::{TabsArgsBuilder, TabsController, tabs_with_controller},
///     text::{TextArgsBuilder, text},
/// };
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(|| TabsController::new(0));
///     tabs_with_controller(
///         TabsArgsBuilder::default()
///             .build()
///             .expect("builder construction failed"),
///         controller,
///         |scope| {
///             scope.child_with_color(
///                 |color| {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("A".to_string())
///                             .color(color)
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///                 || {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Tab A")
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///             );
///             scope.child_with_color(
///                 |color| {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("B".to_string())
///                             .color(color)
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///                 || {
///                     text(
///                         TextArgsBuilder::default()
///                             .text("Tab B")
///                             .build()
///                             .expect("builder construction failed"),
///                     )
///                 },
///             );
///         },
///     );
/// }
/// ```
#[tessera]
pub fn tabs_with_controller<F>(args: TabsArgs, controller: State<TabsController>, scope_config: F)
where
    F: FnOnce(&mut TabsScope),
{
    let mut tabs = Vec::new();
    let mut scope = TabsScope { tabs: &mut tabs };
    scope_config(&mut scope);

    let num_tabs = tabs.len();
    if num_tabs == 0 {
        return;
    }
    let active_tab = controller
        .with(|c| c.active_tab())
        .min(num_tabs.saturating_sub(1));

    let (title_closures, content_closures): (Vec<_>, Vec<_>) =
        tabs.into_iter().map(|def| (def.title, def.content)).unzip();

    surface(
        SurfaceArgs {
            style: args.indicator_color.into(),
            width: DimensionValue::FILLED,
            height: DimensionValue::FILLED,
            shape: Shape::RoundedRectangle {
                top_left: RoundedCorner::Capsule,
                top_right: RoundedCorner::Capsule,
                bottom_right: RoundedCorner::ZERO,
                bottom_left: RoundedCorner::ZERO,
            },
            ..Default::default()
        },
        || {},
    );

    let hover_color = blend_state_layer(
        args.container_color,
        args.state_layer_color,
        args.hover_state_layer_opacity,
    );

    for (index, child) in title_closures.into_iter().enumerate() {
        let label_color = if index == active_tab {
            args.active_content_color
        } else {
            args.inactive_content_color
        };

        button(
            ButtonArgsBuilder::default()
                .color(args.container_color)
                .hover_color(Some(hover_color))
                .padding(args.tab_padding)
                .ripple_color(args.state_layer_color)
                .on_click(Arc::new(move || {
                    controller.with_mut(|c| c.set_active_tab(index));
                }))
                .width(DimensionValue::FILLED)
                .shape(Shape::RECTANGLE)
                .build()
                .expect("builder construction failed"),
            move || {
                boxed(
                    BoxedArgs {
                        alignment: Alignment::Center,
                        width: DimensionValue::FILLED,
                        ..Default::default()
                    },
                    |scope| {
                        scope.child(move || match child {
                            TabTitle::Custom(render) => render(),
                            TabTitle::Themed(render) => render(label_color),
                        });
                    },
                );
            },
        );
    }

    let scroll_offset = controller.with(|c| {
        let eased_progress = animation::easing(c.progress());
        let (content_offset, target_offset) = c.content_offsets();
        let offset =
            content_offset.0 as f32 + (target_offset.0 - content_offset.0) as f32 * eased_progress;
        Px(offset as i32)
    });

    tabs_content_container(scroll_offset, content_closures);

    input_handler(Box::new(move |_| {
        if let Some(last_switch_time) = controller.with(|c| c.last_switch_time()) {
            let elapsed = last_switch_time.elapsed();
            let fraction = (elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32()).min(1.0);
            controller.with_mut(|c| c.set_progress(fraction));
        }
    }));

    let tabs_args = args.clone();
    let controller_for_measure = controller;

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
                    min: Some(titles_total_width),
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
            let page_width = content_container_size.width;
            let target_offset = -Px(active_tab as i32 * page_width.0);
            let (_, target_content_scroll_offset) =
                controller_for_measure.with(|c| c.content_offsets());
            if target_content_scroll_offset != target_offset {
                controller_for_measure.with_mut(|c| {
                    c.update_content_offsets(target_content_scroll_offset, target_offset)
                });
            }

            let (indicator_width, indicator_x) = {
                let active_title_width = title_sizes.get(active_tab).map_or(Px(0), |s| s.width);
                let active_title_x: Px = title_sizes
                    .iter()
                    .take(active_tab)
                    .map(|s| s.width)
                    .fold(Px(0), |acc, w| acc + w);

                let clamped_width = clamp_px(
                    active_title_width,
                    tabs_args.indicator_min_width.into(),
                    tabs_args.indicator_max_width.map(|v| v.into()),
                );
                let centered_x = active_title_x + Px((active_title_width.0 - clamped_width.0) / 2);

                controller_for_measure
                    .with_mut(|c| c.set_indicator_targets(clamped_width, centered_x));

                let (from_width, to_width, from_x, to_x) =
                    controller_for_measure.with(|c| c.indicator_metrics());
                let eased_progress =
                    animation::easing(controller_for_measure.with(|c| c.progress()));
                let width = Px((from_width.0 as f32
                    + (to_width.0 - from_width.0) as f32 * eased_progress)
                    as i32);
                let x = Px((from_x.0 as f32 + (to_x.0 - from_x.0) as f32 * eased_progress) as i32);
                (width, x)
            };

            let indicator_height: Px = tabs_args.indicator_height.into();
            let indicator_constraint = Constraint::new(
                DimensionValue::Fixed(indicator_width),
                DimensionValue::Fixed(indicator_height),
            );
            let _ = input.measure_child(indicator_id, &indicator_constraint)?;

            let tab_bar_height =
                (titles_max_height + indicator_height).max(tabs_args.min_tab_height.into());
            let final_height = tab_bar_height + content_container_size.height;
            let title_offset_y = (tab_bar_height - indicator_height - titles_max_height).max(Px(0));

            let mut current_x = Px(0);
            for (i, &title_id) in title_ids.iter().enumerate() {
                input.place_child(title_id, PxPosition::new(current_x, title_offset_y));
                if let Some(title_size) = title_sizes.get(i) {
                    current_x += title_size.width;
                }
            }

            input.place_child(
                indicator_id,
                PxPosition::new(indicator_x, tab_bar_height - indicator_height),
            );

            input.place_child(content_container_id, PxPosition::new(Px(0), tab_bar_height));

            Ok(ComputedData {
                width: final_width,
                height: final_height,
            })
        },
    ));
}
