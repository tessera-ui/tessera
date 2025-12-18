//! Material Design 3 navigation bar for primary app destinations.
//!
//! ## Usage
//!
//! Use for bottom navigation between a small set of top-level destinations.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, GestureState,
    MeasurementError, Modifier, PressKeyEventType, Px, PxPosition, PxSize, State, accesskit::Role,
    provide_context, remember, tessera, use_context, winit::window::CursorIcon,
};

use crate::{
    ShadowProps,
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    animation,
    column::{ColumnArgsBuilder, column},
    modifier::ModifierExt,
    pos_misc::is_position_in_component,
    ripple_state::{RippleSpec, RippleState},
    row::{RowArgsBuilder, row},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
    text::{TextArgsBuilder, text},
    theme::{ContentColor, MaterialTheme, provide_text_style},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(300);
const CONTAINER_HEIGHT: Dp = Dp(80.0);
const INDICATOR_WIDTH: Dp = Dp(56.0);
const INDICATOR_HEIGHT: Dp = Dp(32.0);
const DIVIDER_HEIGHT: Dp = Dp(1.0);
const ITEM_HORIZONTAL_SPACING: Dp = Dp(8.0);
const INDICATOR_TO_LABEL_PADDING: Dp = Dp(4.0);
const INDICATOR_VERTICAL_PADDING: Dp = Dp(4.0);
const INDICATOR_VERTICAL_OFFSET: Dp = Dp(12.0);

fn interpolate_color(from: Color, to: Color, progress: f32) -> Color {
    Color {
        r: from.r + (to.r - from.r) * progress,
        g: from.g + (to.g - from.g) * progress,
        b: from.b + (to.b - from.b) * progress,
        a: from.a + (to.a - from.a) * progress,
    }
}

#[tessera]
fn navigation_bar_item(
    controller: State<NavigationBarController>,
    index: usize,
    item: NavigationBarItem,
    selected_index: usize,
    previous_index: usize,
    animation_progress: f32,
) {
    let theme = use_context::<MaterialTheme>().get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

    let interaction_state = remember(RippleState::new);

    let is_selected = index == selected_index;
    let was_selected = index == previous_index && selected_index != previous_index;
    let selection_fraction = if is_selected {
        animation_progress
    } else if was_selected {
        1.0 - animation_progress
    } else {
        0.0
    };

    let always_show_label = matches!(item.label_behavior, NavigationBarLabelBehavior::AlwaysShow);
    let has_label = !item.label.is_empty();
    let has_icon = item.icon.is_some();

    let indicator_alpha = selection_fraction;
    let icon_color = interpolate_color(
        scheme.on_surface_variant,
        scheme.on_secondary_container,
        selection_fraction,
    );
    let ripple_color = icon_color;

    let label_alpha = if always_show_label {
        1.0
    } else {
        selection_fraction
    };
    let label_color_base = interpolate_color(
        scheme.on_surface_variant,
        scheme.secondary,
        selection_fraction,
    );
    let label_color = label_color_base.with_alpha(label_color_base.a * label_alpha);

    let indicator_color = scheme.secondary_container.with_alpha(indicator_alpha);

    let size_animation_progress = selection_fraction.max(0.0);
    let indicator_width_px = INDICATOR_WIDTH.to_px();
    let animated_indicator_width_px = Px(((indicator_width_px.0 as f32) * size_animation_progress)
        .round()
        .max(0.0) as i32);

    surface(
        SurfaceArgsBuilder::default()
            .style(SurfaceStyle::Filled {
                color: indicator_color,
            })
            .shape(Shape::capsule())
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(animated_indicator_width_px)),
                Some(DimensionValue::Fixed(INDICATOR_HEIGHT.to_px())),
            ))
            .show_state_layer(false)
            .show_ripple(false)
            .build()
            .expect("builder construction failed"),
        || {},
    );

    surface(
        SurfaceArgsBuilder::default()
            .style(SurfaceStyle::Filled {
                color: Color::TRANSPARENT,
            })
            .shape(Shape::capsule())
            .modifier(Modifier::new().size(INDICATOR_WIDTH, INDICATOR_HEIGHT))
            .enabled(true)
            .interaction_state(interaction_state)
            .ripple_color(ripple_color)
            .build()
            .expect("builder construction failed"),
        || {},
    );

    if let Some(draw_icon) = item.icon {
        provide_context(
            ContentColor {
                current: icon_color,
            },
            || {
                draw_icon();
            },
        );
    }

    if has_label {
        let label = item.label.clone();
        provide_text_style(typography.label_medium, move || {
            text(
                TextArgsBuilder::default()
                    .text(label)
                    .color(label_color)
                    .build()
                    .expect("builder construction failed"),
            );
        });
    }

    let label_for_accessibility = item.label.clone();
    let on_click = item.on_click;

    input_handler(Box::new(move |input| {
        let size = input.computed_data;
        let cursor_pos_option = input.cursor_position_rel;
        let is_cursor_in_item = cursor_pos_option
            .map(|pos| is_position_in_component(size, pos))
            .unwrap_or(false);

        interaction_state.with_mut(|s| s.set_hovered(is_cursor_in_item));

        if input.cursor_events.iter().any(|event| {
            matches!(
                event.content,
                CursorEventContent::Released(PressKeyEventType::Left)
            )
        }) {
            interaction_state.with_mut(|s| s.release());
        }

        if is_cursor_in_item {
            input.requests.cursor_icon = CursorIcon::Pointer;
        }

        if is_cursor_in_item {
            let pressed = input.cursor_events.iter().any(|event| {
                matches!(
                    event.content,
                    CursorEventContent::Pressed(PressKeyEventType::Left)
                )
            });

            if pressed {
                if let Some(cursor_pos) = cursor_pos_option {
                    let item_width_px = size.width.to_f32().max(1.0);
                    let indicator_width_px = INDICATOR_WIDTH.to_px().to_f32().max(1.0);
                    let indicator_height_px = INDICATOR_HEIGHT.to_px().to_f32().max(1.0);

                    let delta_x = (item_width_px - indicator_width_px) / 2.0;
                    let delta_y = INDICATOR_VERTICAL_OFFSET.to_px().to_f32();

                    let normalized_x = (cursor_pos.x.to_f32() - delta_x) / indicator_width_px;
                    let normalized_y = (cursor_pos.y.to_f32() - delta_y) / indicator_height_px;

                    let spec = RippleSpec {
                        bounded: true,
                        radius: None,
                    };

                    interaction_state.with_mut(|s| {
                        s.start_animation_with_spec(
                            [normalized_x, normalized_y],
                            PxSize::new(INDICATOR_WIDTH.to_px(), INDICATOR_HEIGHT.to_px()),
                            spec,
                        );
                        s.set_pressed(true);
                    });
                }
            }

            let released = input.cursor_events.iter().any(|event| {
                event.gesture_state == GestureState::TapCandidate
                    && matches!(
                        event.content,
                        CursorEventContent::Released(PressKeyEventType::Left)
                    )
            });

            if released {
                if index != controller.with(|c| c.selected()) {
                    controller.with_mut(|c| c.set_selected(index));
                    on_click();
                }
            }
        }

        input
            .accessibility()
            .role(Role::Tab)
            .label(label_for_accessibility.clone())
            .commit();
    }));

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            let parent_width = match input.parent_constraint.width() {
                DimensionValue::Fixed(v) => v,
                DimensionValue::Wrap { max, .. } => max.unwrap_or(Px::ZERO),
                DimensionValue::Fill { max, .. } => max.unwrap_or(Px::ZERO),
            };

            let min_height = CONTAINER_HEIGHT.to_px();
            let parent_height = match input.parent_constraint.height() {
                DimensionValue::Fixed(v) => v.max(min_height),
                DimensionValue::Wrap { min, .. } => min.unwrap_or(min_height).max(min_height),
                DimensionValue::Fill { min, .. } => min.unwrap_or(min_height).max(min_height),
            };

            let indicator_background_id = input.children_ids[0];
            let indicator_ripple_id = input.children_ids[1];
            let mut child_index = 2;

            let icon_id = if has_icon {
                let id = input.children_ids[child_index];
                child_index += 1;
                Some(id)
            } else {
                None
            };

            let label_id = if has_label {
                let id = input.children_ids[child_index];
                Some(id)
            } else {
                None
            };

            let child_constraint = Constraint::new(
                DimensionValue::Wrap {
                    min: None,
                    max: None,
                },
                DimensionValue::Wrap {
                    min: None,
                    max: None,
                },
            );

            let indicator_size = input.measure_child(indicator_background_id, &child_constraint)?;
            let indicator_ripple_size =
                input.measure_child(indicator_ripple_id, &child_constraint)?;

            let icon_size = if let Some(icon_id) = icon_id {
                Some(input.measure_child(icon_id, &child_constraint)?)
            } else {
                None
            };

            let label_size = if let Some(label_id) = label_id {
                Some(input.measure_child(label_id, &child_constraint)?)
            } else {
                None
            };

            let width = parent_width;
            let height = parent_height;

            if !has_label {
                let ripple_x = (width - indicator_ripple_size.width) / 2;
                let ripple_y = (height - indicator_ripple_size.height) / 2;
                let indicator_x = (width - indicator_size.width) / 2;
                let indicator_y = (height - indicator_size.height) / 2;
                input.place_child(
                    indicator_background_id,
                    PxPosition::new(indicator_x, indicator_y),
                );
                input.place_child(indicator_ripple_id, PxPosition::new(ripple_x, ripple_y));

                if let (Some(icon_id), Some(icon_size)) = (icon_id, icon_size) {
                    let icon_x = (width - icon_size.width) / 2;
                    let icon_y = (height - icon_size.height) / 2;
                    input.place_child(icon_id, PxPosition::new(icon_x, icon_y));
                }

                return Ok(ComputedData { width, height });
            }

            let icon_size = icon_size.unwrap_or(ComputedData {
                width: Px::ZERO,
                height: Px::ZERO,
            });
            let label_size = label_size.unwrap_or(ComputedData {
                width: Px::ZERO,
                height: Px::ZERO,
            });

            let indicator_vertical_padding_px = INDICATOR_VERTICAL_PADDING.to_px();
            let content_height = icon_size.height
                + indicator_vertical_padding_px
                + INDICATOR_TO_LABEL_PADDING.to_px()
                + label_size.height;

            let content_vertical_padding =
                ((height - content_height) / 2).max(indicator_vertical_padding_px);
            let selected_icon_y = content_vertical_padding;
            let unselected_icon_y = if always_show_label {
                selected_icon_y
            } else {
                (height - icon_size.height) / 2
            };

            let icon_distance = unselected_icon_y - selected_icon_y;
            let offset = Px(((icon_distance.0 as f32) * (1.0 - selection_fraction)).round() as i32);

            let icon_x = (width - icon_size.width) / 2;
            let label_x = (width - label_size.width) / 2;
            let ripple_x = (width - indicator_ripple_size.width) / 2;
            let indicator_x = (width - indicator_size.width) / 2;

            let ripple_y = selected_icon_y - indicator_vertical_padding_px;
            let indicator_y = selected_icon_y - indicator_vertical_padding_px;
            let icon_y = selected_icon_y;
            let label_y = selected_icon_y
                + icon_size.height
                + indicator_vertical_padding_px
                + INDICATOR_TO_LABEL_PADDING.to_px();

            input.place_child(
                indicator_background_id,
                PxPosition::new(indicator_x, Px(indicator_y.0 + offset.0)),
            );
            input.place_child(
                indicator_ripple_id,
                PxPosition::new(ripple_x, Px(ripple_y.0 + offset.0)),
            );

            if let Some(icon_id) = icon_id {
                input.place_child(icon_id, PxPosition::new(icon_x, Px(icon_y.0 + offset.0)));
            }

            if always_show_label || selection_fraction != 0.0 {
                if let Some(label_id) = label_id {
                    input.place_child(label_id, PxPosition::new(label_x, Px(label_y.0 + offset.0)));
                }
            }

            Ok(ComputedData { width, height })
        },
    ));
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
    #[builder(default, setter(custom, strip_option))]
    pub icon: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Callback invoked after selection changes to this item.
    #[builder(default = "Arc::new(|| {})", setter(custom))]
    pub on_click: Arc<dyn Fn() + Send + Sync>,
    /// Whether the label is always visible or only appears when selected.
    #[builder(default = "NavigationBarLabelBehavior::AlwaysShow")]
    pub label_behavior: NavigationBarLabelBehavior,
}

impl NavigationBarItemBuilder {
    /// Set the icon drawing callback.
    pub fn icon<F>(mut self, icon: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.icon = Some(Some(Arc::new(icon)));
        self
    }

    /// Set the icon drawing callback using a shared callback.
    pub fn icon_shared(mut self, icon: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.icon = Some(Some(icon));
        self
    }

    /// Set the click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_click = Some(on_click);
        self
    }
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
/// - `scope_config` — closure that registers items via [`NavigationBarScope`].
///
/// ## Examples
///
/// ```
/// use tessera_ui::tessera;
/// use tessera_ui_basic_components::navigation_bar::{NavigationBarItemBuilder, navigation_bar};
///
/// #[tessera]
/// fn demo() {
///     navigation_bar(|scope| {
///         scope.item(
///             NavigationBarItemBuilder::default()
///                 .label("Home")
///                 .build()
///                 .unwrap(),
///         );
///         scope.item(
///             NavigationBarItemBuilder::default()
///                 .label("Search")
///                 .build()
///                 .unwrap(),
///         );
///     });
/// }
/// ```
#[tessera]
pub fn navigation_bar<F>(scope_config: F)
where
    F: FnOnce(&mut NavigationBarScope),
{
    let controller = remember(|| NavigationBarController::new(0));
    navigation_bar_with_controller(controller, scope_config);
}

/// # navigation_bar_with_controller
///
/// Controlled variant that accepts an explicit controller.
///
/// ## Parameters
///
/// - `controller` — explicit controller to manage selection.
/// - `scope_config` — closure that registers items via [`NavigationBarScope`].
///
/// ## Examples
///
/// ```
/// use tessera_ui::{remember, tessera};
/// use tessera_ui_basic_components::navigation_bar::{
///     NavigationBarController, NavigationBarItemBuilder, navigation_bar_with_controller,
/// };
///
/// #[tessera]
/// fn controlled_demo() {
///     let controller = remember(|| NavigationBarController::new(0));
///     navigation_bar_with_controller(controller, |scope| {
///         scope.item(
///             NavigationBarItemBuilder::default()
///                 .label("Home")
///                 .build()
///                 .unwrap(),
///         );
///         scope.item(
///             NavigationBarItemBuilder::default()
///                 .label("Search")
///                 .build()
///                 .unwrap(),
///         );
///     });
/// }
/// ```
#[tessera]
pub fn navigation_bar_with_controller<F>(
    controller: State<NavigationBarController>,
    scope_config: F,
) where
    F: FnOnce(&mut NavigationBarScope),
{
    let mut items = Vec::new();
    {
        let mut scope = NavigationBarScope { items: &mut items };
        scope_config(&mut scope);
    }
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let container_shadow = ShadowProps {
        color: scheme.shadow.with_alpha(0.16),
        offset: [0.0, 3.0],
        smoothness: 10.0,
    };

    let animation_progress = controller
        .with_mut(|c| c.animation_progress())
        .unwrap_or(1.0);
    let selected_index = controller.with(|c| c.selected());
    let previous_index = controller.with(|c| c.previous_selected());

    surface(
        SurfaceArgsBuilder::default()
            .modifier(Modifier::new().fill_max_width().height(CONTAINER_HEIGHT))
            .style(scheme.surface_container.into())
            .shadow(container_shadow)
            .block_input(true)
            .build()
            .expect("SurfaceArgsBuilder failed with required fields set"),
        move || {
            let separator_color = scheme.outline_variant.with_alpha(0.12);
            column(
                ColumnArgsBuilder::default()
                    .modifier(Modifier::new().fill_max_size())
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .build()
                    .expect("ColumnArgsBuilder failed with required fields set"),
                move |column_scope| {
                    column_scope.child(move || {
                        surface(
                            SurfaceArgsBuilder::default()
                                .modifier(Modifier::new().fill_max_width().height(DIVIDER_HEIGHT))
                                .style(separator_color.into())
                                .build()
                                .expect("SurfaceArgsBuilder failed for divider"),
                            || {},
                        );
                    });

                    column_scope.child_weighted(
                        move || {
                            row(
                                RowArgsBuilder::default()
                                    .modifier(Modifier::new().fill_max_size())
                                    .main_axis_alignment(MainAxisAlignment::Start)
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .build()
                                    .expect("RowArgsBuilder failed with required fields set"),
                                move |row_scope| {
                                    let last_index = items.len().saturating_sub(1);
                                    for (index, item) in items.into_iter().enumerate() {
                                        row_scope.child_weighted(
                                            move || {
                                                navigation_bar_item(
                                                    controller,
                                                    index,
                                                    item,
                                                    selected_index,
                                                    previous_index,
                                                    animation_progress,
                                                );
                                            },
                                            1.0,
                                        );

                                        if index != last_index {
                                            row_scope.child(|| {
                                                spacer(
                                                    Modifier::new().width(ITEM_HORIZONTAL_SPACING),
                                                );
                                            });
                                        }
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

/// Controller for the `navigation_bar` component.
#[derive(Clone)]
pub struct NavigationBarController {
    selected: usize,
    previous_selected: usize,
    anim_start_time: Option<Instant>,
}

impl NavigationBarController {
    /// Create a new state with an initial selected index.
    pub fn new(selected: usize) -> Self {
        Self {
            selected,
            previous_selected: selected,
            anim_start_time: None,
        }
    }

    /// Returns the index of the currently selected navigation item.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Returns the index of the previously selected navigation item.
    pub(crate) fn previous_selected(&self) -> usize {
        self.previous_selected
    }

    /// Programmatically select an item by index.
    pub fn select(&mut self, index: usize) {
        self.set_selected(index);
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
}

impl Default for NavigationBarController {
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
