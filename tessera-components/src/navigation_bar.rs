//! Material Design 3 navigation bar for primary app destinations.
//!
//! ## Usage
//!
//! Use for bottom navigation between a small set of top-level destinations.
use std::time::{Duration, Instant};

use derive_setters::Setters;
use tessera_ui::{
    Callback, Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier, Px,
    PxPosition, PxSize, RenderSlot, State,
    accesskit::Role,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    animation,
    column::{ColumnArgs, column},
    modifier::{InteractionState, ModifierExt, PointerEventContext, SelectableArgs},
    ripple_state::{RippleSpec, RippleState},
    row::{RowArgs, row},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
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

#[derive(Clone, PartialEq)]
struct NavigationBarItemContentArgs {
    item: NavigationBarItem,
    is_selected: bool,
    was_selected: bool,
    animation_progress: f32,
    interaction_state: State<InteractionState>,
    ripple_state: State<RippleState>,
}

#[derive(Clone, PartialEq)]
struct NavigationBarItemArgs {
    controller: State<NavigationBarController>,
    index: usize,
    item: NavigationBarItem,
    selected_index: usize,
    previous_index: usize,
    animation_progress: f32,
}

#[derive(Clone, PartialEq)]
struct NavigationBarComposeArgs {
    items: Vec<NavigationBarItem>,
}

#[derive(Clone, PartialEq)]
struct NavigationBarRenderArgs {
    controller: State<NavigationBarController>,
    items: Vec<NavigationBarItem>,
}

fn interpolate_color(from: Color, to: Color, progress: f32) -> Color {
    Color {
        r: from.r + (to.r - from.r) * progress,
        g: from.g + (to.g - from.g) * progress,
        b: from.b + (to.b - from.b) * progress,
        a: from.a + (to.a - from.a) * progress,
    }
}

#[tessera]
fn navigation_bar_item_content_node(args: &NavigationBarItemContentArgs) {
    let item = args.item.clone();
    let is_selected = args.is_selected;
    let was_selected = args.was_selected;
    let animation_progress = args.animation_progress;
    let interaction_state = args.interaction_state;
    let ripple_state = args.ripple_state;

    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

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

    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
            .style(SurfaceStyle::Filled {
                color: indicator_color,
            })
            .shape(Shape::capsule())
            .modifier(Modifier::new().constrain(
                Some(DimensionValue::Fixed(animated_indicator_width_px)),
                Some(DimensionValue::Fixed(INDICATOR_HEIGHT.to_px())),
            ))
            .show_state_layer(false)
            .show_ripple(false),
        || {},
    ));

    let indicator_args = SurfaceArgs::default()
        .style(SurfaceStyle::Filled {
            color: Color::TRANSPARENT,
        })
        .shape(Shape::capsule())
        .modifier(Modifier::new().size(INDICATOR_WIDTH, INDICATOR_HEIGHT))
        .enabled(true)
        .interaction_state(interaction_state)
        .ripple_color(ripple_color);
    surface(&crate::surface::SurfaceArgs::with_child(
        indicator_args,
        move || {
            surface(&crate::surface::SurfaceArgs::with_child(
                SurfaceArgs::default()
                    .style(SurfaceStyle::Filled {
                        color: Color::TRANSPARENT,
                    })
                    .shape(Shape::capsule())
                    .modifier(Modifier::new().size(INDICATOR_WIDTH, INDICATOR_HEIGHT))
                    .enabled(true)
                    .ripple_color(ripple_color)
                    .show_state_layer(false)
                    .ripple_state(ripple_state),
                || {},
            ));
        },
    ));

    if let Some(draw_icon) = item.icon {
        provide_context(
            || ContentColor {
                current: icon_color,
            },
            || {
                draw_icon.render();
            },
        );
    }

    if has_label {
        let label = item.label.clone();
        provide_text_style(typography.label_medium, move || {
            text(&crate::text::TextArgs::from(
                &TextArgs::default().text(label.clone()).color(label_color),
            ));
        });
    }

    layout(NavigationBarItemLayout {
        selection_fraction,
        always_show_label,
        has_label,
        has_icon,
    });
}

#[derive(Clone, PartialEq)]
struct NavigationBarItemLayout {
    selection_fraction: f32,
    always_show_label: bool,
    has_label: bool,
    has_icon: bool,
}

impl LayoutSpec for NavigationBarItemLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let parent_width = match input.parent_constraint().width() {
            DimensionValue::Fixed(v) => v,
            DimensionValue::Wrap { max, .. } => max.unwrap_or(Px::ZERO),
            DimensionValue::Fill { max, .. } => max.unwrap_or(Px::ZERO),
        };

        let min_height = CONTAINER_HEIGHT.to_px();
        let parent_height = match input.parent_constraint().height() {
            DimensionValue::Fixed(v) => v.max(min_height),
            DimensionValue::Wrap { min, .. } => min.unwrap_or(min_height).max(min_height),
            DimensionValue::Fill { min, .. } => min.unwrap_or(min_height).max(min_height),
        };

        let indicator_background_id = input.children_ids()[0];
        let indicator_ripple_id = input.children_ids()[1];
        let mut child_index = 2;

        let icon_id = if self.has_icon {
            let id = input.children_ids()[child_index];
            child_index += 1;
            Some(id)
        } else {
            None
        };

        let label_id = if self.has_label {
            let id = input.children_ids()[child_index];
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
        let indicator_ripple_size = input.measure_child(indicator_ripple_id, &child_constraint)?;

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

        if !self.has_label {
            let ripple_x = (width - indicator_ripple_size.width) / 2;
            let ripple_y = (height - indicator_ripple_size.height) / 2;
            let indicator_x = (width - indicator_size.width) / 2;
            let indicator_y = (height - indicator_size.height) / 2;
            output.place_child(
                indicator_background_id,
                PxPosition::new(indicator_x, indicator_y),
            );
            output.place_child(indicator_ripple_id, PxPosition::new(ripple_x, ripple_y));

            if let (Some(icon_id), Some(icon_size)) = (icon_id, icon_size) {
                let icon_x = (width - icon_size.width) / 2;
                let icon_y = (height - icon_size.height) / 2;
                output.place_child(icon_id, PxPosition::new(icon_x, icon_y));
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
        let unselected_icon_y = if self.always_show_label {
            selected_icon_y
        } else {
            (height - icon_size.height) / 2
        };

        let icon_distance = unselected_icon_y - selected_icon_y;
        let offset =
            Px(((icon_distance.0 as f32) * (1.0 - self.selection_fraction)).round() as i32);

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

        output.place_child(
            indicator_background_id,
            PxPosition::new(indicator_x, Px(indicator_y.0 + offset.0)),
        );
        output.place_child(
            indicator_ripple_id,
            PxPosition::new(ripple_x, Px(ripple_y.0 + offset.0)),
        );

        if let Some(icon_id) = icon_id {
            output.place_child(icon_id, PxPosition::new(icon_x, Px(icon_y.0 + offset.0)));
        }

        if (self.always_show_label || self.selection_fraction != 0.0)
            && let Some(label_id) = label_id
        {
            output.place_child(label_id, PxPosition::new(label_x, Px(label_y.0 + offset.0)));
        }

        Ok(ComputedData { width, height })
    }
}

#[tessera]
fn navigation_bar_item_node(args: &NavigationBarItemArgs) {
    let controller = args.controller;
    let index = args.index;
    let item = args.item.clone();
    let selected_index = args.selected_index;
    let previous_index = args.previous_index;
    let animation_progress = args.animation_progress;

    let interaction_state = remember(InteractionState::new);
    let ripple_state = remember(RippleState::new);

    let is_selected = index == selected_index;
    let was_selected = index == previous_index && selected_index != previous_index;
    let label = item.label.clone();

    let ripple_state_for_press = ripple_state;
    let on_press = move |ctx: PointerEventContext| {
        let spec = RippleSpec {
            bounded: true,
            radius: None,
        };
        ripple_state_for_press.with_mut(|state| {
            state.start_animation_with_spec(
                ctx.normalized_pos,
                PxSize::new(INDICATOR_WIDTH.to_px(), INDICATOR_HEIGHT.to_px()),
                spec,
            );
        });
    };
    let ripple_state_for_release = ripple_state;
    let on_release = move |_ctx: PointerEventContext| {
        ripple_state_for_release.with_mut(|state| state.release());
    };

    let on_click_item = item.on_click.clone();
    let controller_for_click = controller;
    let on_click = move || {
        controller_for_click.with_mut(|c| c.set_selected(index));
        on_click_item.call();
    };

    let selectable_args = SelectableArgs::new(is_selected, on_click)
        .enabled(true)
        .role(Role::Tab)
        .label(label)
        .interaction_state(interaction_state)
        .on_press(on_press)
        .on_release(on_release);

    Modifier::new().selectable(selectable_args).run({
        let item = item.clone();
        move || {
            let content_args = NavigationBarItemContentArgs {
                item: item.clone(),
                is_selected,
                was_selected,
                animation_progress,
                interaction_state,
                ripple_state,
            };
            navigation_bar_item_content_node(&content_args);
        }
    });
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
#[derive(Clone, PartialEq, Setters)]
pub struct NavigationBarItem {
    /// Text label shown under the icon.
    #[setters(into)]
    pub label: String,
    /// Optional icon rendered above the label.
    #[setters(skip)]
    pub icon: Option<RenderSlot>,
    /// Callback invoked after selection changes to this item.
    #[setters(skip)]
    pub on_click: Callback,
    /// Whether the label is always visible or only appears when selected.
    pub label_behavior: NavigationBarLabelBehavior,
}

impl NavigationBarItem {
    /// Creates a navigation item with the required label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            on_click: Callback::new(|| {}),
            label_behavior: NavigationBarLabelBehavior::AlwaysShow,
        }
    }

    /// Set the icon drawing callback.
    pub fn icon<F>(mut self, icon: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.icon = Some(RenderSlot::new(icon));
        self
    }

    /// Set the icon drawing callback using a shared callback.
    pub fn icon_shared(mut self, icon: impl Into<RenderSlot>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Callback::new(on_click);
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: impl Into<Callback>) -> Self {
        self.on_click = on_click.into();
        self
    }
}

impl Default for NavigationBarItem {
    fn default() -> Self {
        Self::new("")
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
/// use tessera_components::navigation_bar::{NavigationBarItem, navigation_bar};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn demo() {
///     navigation_bar(|scope| {
///         scope.item(NavigationBarItem::new("Home"));
///         scope.item(NavigationBarItem::new("Search"));
///     });
/// }
/// ```
pub fn navigation_bar<F>(scope_config: F)
where
    F: FnOnce(&mut NavigationBarScope),
{
    let mut items = Vec::new();
    {
        let mut scope = NavigationBarScope { items: &mut items };
        scope_config(&mut scope);
    }
    let render_args = NavigationBarComposeArgs { items };
    navigation_bar_node(&render_args);
}

#[tessera]
fn navigation_bar_node(args: &NavigationBarComposeArgs) {
    let controller = remember(|| NavigationBarController::new(0));
    let render_args = NavigationBarRenderArgs {
        controller,
        items: args.items.clone(),
    };
    navigation_bar_render_node(&render_args);
}

#[tessera]
fn navigation_bar_render_node(args: &NavigationBarRenderArgs) {
    let controller = args.controller;
    let items = args.items.clone();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;

    let animation_progress = controller
        .with_mut(|c| c.animation_progress())
        .unwrap_or(1.0);
    if controller.with(|c| c.is_animating()) {
        let controller_for_frame = controller;
        receive_frame_nanos(move |_| {
            let is_animating = controller_for_frame.with_mut(|controller| {
                let _ = controller.animation_progress();
                controller.is_animating()
            });
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let selected_index = controller.with(|c| c.selected());
    let previous_index = controller.with(|c| c.previous_selected());

    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_width().height(CONTAINER_HEIGHT))
            .style(scheme.surface_container.into())
            .elevation(Dp(3.0))
            .block_input(true),
        move || {
            let items = items.clone();
            let separator_color = scheme.outline_variant.with_alpha(0.12);
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size())
                    .cross_axis_alignment(CrossAxisAlignment::Stretch),
                move |column_scope| {
                    column_scope.child(move || {
                        surface(&crate::surface::SurfaceArgs::with_child(
                            SurfaceArgs::default()
                                .modifier(Modifier::new().fill_max_width().height(DIVIDER_HEIGHT))
                                .style(separator_color.into()),
                            || {},
                        ));
                    });

                    column_scope.child_weighted(
                        move || {
                            let items = items.clone();
                            row(
                                RowArgs::default()
                                    .modifier(Modifier::new().fill_max_size())
                                    .main_axis_alignment(MainAxisAlignment::Start)
                                    .cross_axis_alignment(CrossAxisAlignment::Center),
                                move |row_scope| {
                                    let last_index = items.len().saturating_sub(1);
                                    for (index, item) in items.iter().cloned().enumerate() {
                                        row_scope.child_weighted(
                                            move || {
                                                let item_args = NavigationBarItemArgs {
                                                    controller,
                                                    index,
                                                    item: item.clone(),
                                                    selected_index,
                                                    previous_index,
                                                    animation_progress,
                                                };
                                                navigation_bar_item_node(&item_args);
                                            },
                                            1.0,
                                        );

                                        if index != last_index {
                                            row_scope.child(|| {
                                                spacer(&crate::spacer::SpacerArgs::new(
                                                    Modifier::new().width(ITEM_HORIZONTAL_SPACING),
                                                ));
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
    ));
}

/// Controller for the `navigation_bar` component.
#[derive(Clone, PartialEq)]
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

    fn is_animating(&self) -> bool {
        self.anim_start_time.is_some()
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
