//! Material Design 3 navigation bar for primary app destinations.
//!
//! ## Usage
//!
//! Use for bottom navigation between a small set of top-level destinations.
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, tessera};

use crate::{
    RippleState, ShadowProps,
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    animation,
    boxed::{BoxedArgsBuilder, boxed},
    column::{ColumnArgsBuilder, column},
    material_color::{MaterialColorScheme, global_material_scheme},
    row::{RowArgsBuilder, row},
    shape_def::Shape,
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
    text::{TextArgsBuilder, text},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(300);
const CONTAINER_HEIGHT: Dp = Dp(80.0);
const ITEM_PADDING: Dp = Dp(12.0);
const LABEL_TEXT_SIZE: Dp = Dp(16.0);
const LABEL_SPACING: Dp = Dp(4.0);
const INDICATOR_WIDTH: Dp = Dp(56.0);
const INDICATOR_HEIGHT: Dp = Dp(32.0);
const DIVIDER_HEIGHT: Dp = Dp(1.0);
const UNSELECTED_LABEL_ALPHA: f32 = 0.8;

fn interpolate_color(from: Color, to: Color, progress: f32) -> Color {
    Color {
        r: from.r + (to.r - from.r) * progress,
        g: from.g + (to.g - from.g) * progress,
        b: from.b + (to.b - from.b) * progress,
        a: from.a + (to.a - from.a) * progress,
    }
}

/// Controls label visibility for a navigation bar item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationBarLabelBehavior {
    /// Always render the label.
    AlwaysShow,
    /// Fade the label in only when the item is selected.
    SelectedOnly,
}

/// Item configuration for [`navigation_bar`].
#[derive(Clone, Builder)]
#[builder(pattern = "owned")]
pub struct NavigationBarItem {
    /// Text label shown under the icon.
    #[builder(setter(into))]
    pub label: String,
    /// Optional icon rendered above the label.
    #[builder(default, setter(strip_option))]
    pub icon: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Callback invoked after selection changes to this item.
    #[builder(default = "Arc::new(|| {})")]
    pub on_click: Arc<dyn Fn() + Send + Sync>,
    /// Whether the label is always visible or only appears when selected.
    #[builder(default = "NavigationBarLabelBehavior::AlwaysShow")]
    pub label_behavior: NavigationBarLabelBehavior,
}

/// # navigation_bar
///
/// Material navigation bar with active indicator and icon/label pairs.
///
/// ## Usage
///
/// Place at the bottom of the app to switch between 3–5 primary destinations.
///
/// ## Parameters
///
/// - `state` — see [`NavigationBarState`] to track which destination is active.
/// - `scope_config` — closure that registers items via [`NavigationBarScope`].
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::navigation_bar::{
///     NavigationBarItemBuilder, NavigationBarState, navigation_bar,
/// };
///
/// let state = NavigationBarState::new(0);
/// navigation_bar(state.clone(), |scope| {
///     scope.item(
///         NavigationBarItemBuilder::default()
///             .label("Home")
///             .build()
///             .unwrap(),
///     );
///     scope.item(
///         NavigationBarItemBuilder::default()
///             .label("Search")
///             .build()
///             .unwrap(),
///     );
/// });
/// assert_eq!(state.selected(), 0);
/// state.select(1);
/// assert_eq!(state.selected(), 1);
/// assert_eq!(state.previous_selected(), 0);
/// ```
#[tessera]
pub fn navigation_bar<F>(state: NavigationBarState, scope_config: F)
where
    F: FnOnce(&mut NavigationBarScope),
{
    let mut items = Vec::new();
    {
        let mut scope = NavigationBarScope { items: &mut items };
        scope_config(&mut scope);
    }

    let scheme = global_material_scheme();
    let container_shadow = ShadowProps {
        color: scheme.shadow.with_alpha(0.16),
        offset: [0.0, 3.0],
        smoothness: 10.0,
    };

    let animation_progress = state.animation_progress().unwrap_or(1.0);
    let selected_index = state.selected();
    let previous_index = state.previous_selected();

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(CONTAINER_HEIGHT)
            .style(scheme.surface.into())
            .shadow(container_shadow)
            .block_input(true)
            .build()
            .expect("SurfaceArgsBuilder failed with required fields set"),
        None,
        move || {
            let separator_color = scheme.outline_variant.with_alpha(0.12);
            column(
                ColumnArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .build()
                    .expect("ColumnArgsBuilder failed with required fields set"),
                move |column_scope| {
                    column_scope.child(move || {
                        surface(
                            SurfaceArgsBuilder::default()
                                .width(DimensionValue::FILLED)
                                .height(DIVIDER_HEIGHT)
                                .style(separator_color.into())
                                .build()
                                .expect("SurfaceArgsBuilder failed for divider"),
                            None,
                            || {},
                        );
                    });

                    column_scope.child_weighted(
                        move || {
                            row(
                                RowArgsBuilder::default()
                                    .width(DimensionValue::FILLED)
                                    .height(DimensionValue::FILLED)
                                    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .build()
                                    .expect("RowArgsBuilder failed with required fields set"),
                                move |row_scope| {
                                    for (index, item) in items.into_iter().enumerate() {
                                        let state_clone = state.clone();
                                        let scheme_for_item = scheme.clone();
                                        row_scope.child_weighted(
                                            move || {
                                                render_navigation_item(
                                                    &state_clone,
                                                    index,
                                                    item,
                                                    selected_index,
                                                    previous_index,
                                                    animation_progress,
                                                    scheme_for_item,
                                                );
                                            },
                                            1.0,
                                        );
                                    }
                                },
                            );
                        },
                        1.0,
                    );
                },
            );
        },
    );
}

fn render_navigation_item(
    state: &NavigationBarState,
    index: usize,
    item: NavigationBarItem,
    selected_index: usize,
    previous_index: usize,
    animation_progress: f32,
    scheme: MaterialColorScheme,
) {
    let is_selected = index == selected_index;
    let was_selected = index == previous_index && selected_index != previous_index;
    let selection_fraction = if is_selected {
        animation_progress
    } else if was_selected {
        1.0 - animation_progress
    } else {
        0.0
    };

    let indicator_alpha = selection_fraction;
    let content_color = interpolate_color(
        scheme.on_surface_variant,
        scheme.on_secondary_container,
        selection_fraction,
    );
    let ripple_color = interpolate_color(
        scheme.on_surface_variant.with_alpha(0.12),
        scheme.on_secondary_container.with_alpha(0.12),
        selection_fraction,
    );

    let label_alpha = match item.label_behavior {
        NavigationBarLabelBehavior::AlwaysShow => {
            selection_fraction + (1.0 - selection_fraction) * UNSELECTED_LABEL_ALPHA
        }
        NavigationBarLabelBehavior::SelectedOnly => selection_fraction,
    };
    let label_color = content_color.with_alpha(content_color.a * label_alpha);

    let label_text = item.label.clone();
    let icon_closure = item.icon.clone();
    let indicator_color = scheme.secondary_container.with_alpha(indicator_alpha);

    let ripple_state = state.ripple_state(index);
    let on_click = item.on_click.clone();
    let state_for_click = state.clone();
    let icon_only_indicator_color = indicator_color;

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .style(SurfaceStyle::Filled {
                color: Color::TRANSPARENT,
            })
            .shape(Shape::RECTANGLE)
            .padding(ITEM_PADDING)
            .ripple_color(ripple_color)
            .hover_style(None)
            .accessibility_label(label_text.clone())
            .on_click(Arc::new(move || {
                if index != state_for_click.selected() {
                    state_for_click.set_selected(index);
                    on_click();
                }
            }))
            .build()
            .expect("SurfaceArgsBuilder failed with required fields set"),
        Some(ripple_state),
        move || {
            let label_for_text = label_text.clone();
            let label_color_for_text = label_color;
            boxed(
                BoxedArgsBuilder::default()
                    .alignment(Alignment::Center)
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .build()
                    .expect("BoxedArgsBuilder failed for item container"),
                move |container| {
                    container.child(move || {
                        column(
                            ColumnArgsBuilder::default()
                                .width(DimensionValue::WRAP)
                                .height(DimensionValue::WRAP)
                                .main_axis_alignment(MainAxisAlignment::Center)
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .build()
                                .expect("ColumnArgsBuilder failed with required fields set"),
                            move |column_scope| {
                                let label_for_text = label_for_text.clone();
                                let label_color = label_color_for_text;
                                let has_icon = icon_closure.is_some();
                                let icon_closure_for_stack = icon_closure.clone();
                                column_scope.child(move || {
                                    boxed(
                                        BoxedArgsBuilder::default()
                                        .alignment(Alignment::Center)
                                        .build()
                                        .expect("BoxedArgsBuilder failed for icon stack"),
                                    move |icon_stack| {
                                        let indicator_color = icon_only_indicator_color;
                                        icon_stack.child(move || {
                                            surface(
                                                SurfaceArgsBuilder::default()
                                                    .style(SurfaceStyle::Filled {
                                                        color: indicator_color,
                                                    })
                                                    .shape(Shape::capsule())
                                                    .width(INDICATOR_WIDTH)
                                                    .height(INDICATOR_HEIGHT)
                                                    .build()
                                                    .expect("SurfaceArgsBuilder failed for indicator"),
                                                None,
                                                || {},
                                            );
                                        });

                                        if let Some(draw_icon) = icon_closure_for_stack.clone()
                                        {
                                            icon_stack.child(move || {
                                                draw_icon();
                                                });
                                            }
                                        },
                                    );
                                });

                                if !label_for_text.is_empty() {
                                    if has_icon {
                                        column_scope.child(move || {
                                            spacer(
                                                SpacerArgsBuilder::default()
                                                    .height(LABEL_SPACING)
                                                    .build()
                                                    .expect(
                                                        "SpacerArgsBuilder failed with required fields set",
                                                    ),
                                            );
                                        });
                                    }
                                    let label = label_for_text.clone();
                                    column_scope.child(move || {
                                        text(
                                            TextArgsBuilder::default()
                                                .text(label)
                                                .color(label_color)
                                                .size(LABEL_TEXT_SIZE)
                                                .build()
                                                .expect("TextArgsBuilder failed with required fields set"),
                                        );
                                    });
                                }
                            },
                        );
                    });
                },
            );
        },
    );
}

/// Holds selection & per-item ripple state for the navigation bar.
///
/// `selected` tracks the currently active item index, while `ripple_states` lazily initializes
/// per-item ripple data on first access.
struct NavigationBarStateInner {
    selected: usize,
    previous_selected: usize,
    ripple_states: HashMap<usize, RippleState>,
    anim_start_time: Option<Instant>,
}

impl NavigationBarStateInner {
    fn new(selected: usize) -> Self {
        Self {
            selected,
            previous_selected: selected,
            ripple_states: HashMap::new(),
            anim_start_time: None,
        }
    }

    fn set_selected(&mut self, index: usize) {
        if self.selected != index {
            self.previous_selected = self.selected;
            self.selected = index;
            self.anim_start_time = Some(Instant::now());
        }
    }

    fn animation_progress(&mut self) -> Option<f32> {
        if let Some(start_time) = self.anim_start_time {
            let elapsed = start_time.elapsed();
            if elapsed < ANIMATION_DURATION {
                Some(animation::easing(
                    elapsed.as_secs_f32() / ANIMATION_DURATION.as_secs_f32(),
                ))
            } else {
                self.anim_start_time = None;
                None
            }
        } else {
            None
        }
    }

    fn ripple_state(&mut self, index: usize) -> RippleState {
        self.ripple_states.entry(index).or_default().clone()
    }
}

/// Shared state for the `navigation_bar` component.
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::navigation_bar::NavigationBarState;
///
/// let state = NavigationBarState::new(0);
/// assert_eq!(state.selected(), 0);
/// state.select(2);
/// assert_eq!(state.selected(), 2);
/// assert_eq!(state.previous_selected(), 0);
/// ```
#[derive(Clone)]
pub struct NavigationBarState {
    inner: Arc<RwLock<NavigationBarStateInner>>,
}

impl NavigationBarState {
    /// Create a new state with an initial selected index.
    pub fn new(selected: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(NavigationBarStateInner::new(selected))),
        }
    }

    /// Returns the index of the currently selected navigation item.
    pub fn selected(&self) -> usize {
        self.inner.read().selected
    }

    /// Returns the index of the previously selected navigation item.
    pub fn previous_selected(&self) -> usize {
        self.inner.read().previous_selected
    }

    /// Programmatically select an item by index.
    pub fn select(&self, index: usize) {
        self.inner.write().set_selected(index);
    }

    fn set_selected(&self, index: usize) {
        self.inner.write().set_selected(index);
    }

    fn animation_progress(&self) -> Option<f32> {
        self.inner.write().animation_progress()
    }

    fn ripple_state(&self, index: usize) -> RippleState {
        self.inner.write().ripple_state(index)
    }
}

impl Default for NavigationBarState {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Scope passed to the closure for defining children of the NavigationBar.
pub struct NavigationBarScope<'a> {
    items: &'a mut Vec<NavigationBarItem>,
}

impl<'a> NavigationBarScope<'a> {
    /// Add a navigation item to the bar.
    pub fn item<I>(&mut self, item: I)
    where
        I: Into<NavigationBarItem>,
    {
        self.items.push(item.into());
    }
}
