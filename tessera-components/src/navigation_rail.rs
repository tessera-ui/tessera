//! Material Design 3 navigation rail with collapsed and expanded layouts.
//!
//! ## Usage
//!
//! Use for primary destinations on wide layouts with a collapsible side rail.
use std::time::Duration;

use derive_setters::Setters;
use tessera_ui::{
    Callback, Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier, Px,
    PxPosition, PxSize, RenderSlot, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{LayoutInput, LayoutOutput, LayoutSpec},
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    animation,
    column::{ColumnArgs, column},
    modifier::{InteractionState, ModifierExt, Padding, PointerEventContext, SelectableArgs},
    ripple_state::{RippleSpec, RippleState},
    row::{RowArgs, row},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
    theme::{ContentColor, MaterialTheme, provide_text_style},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(300);
const COLLAPSED_WIDTH: Dp = Dp(96.0);
const EXPANDED_MIN_WIDTH: Dp = Dp(220.0);
const TOP_PADDING: Dp = Dp(44.0);
const HEADER_BOTTOM_PADDING: Dp = Dp(40.0);
const ITEM_HORIZONTAL_PADDING: Dp = Dp(20.0);
const ITEM_VERTICAL_SPACING_COLLAPSED: Dp = Dp(4.0);
const ITEM_VERTICAL_SPACING_EXPANDED: Dp = Dp(0.0);
const TOP_ICON_ITEM_MIN_HEIGHT: Dp = Dp(64.0);
const START_ICON_ITEM_MIN_HEIGHT: Dp = Dp(56.0);
const INDICATOR_TOP_WIDTH: Dp = Dp(56.0);
const INDICATOR_TOP_HEIGHT: Dp = Dp(32.0);
const INDICATOR_START_HEIGHT: Dp = Dp(56.0);
const INDICATOR_TOP_TO_LABEL_PADDING: Dp = Dp(4.0);
const START_ICON_TO_LABEL_PADDING: Dp = Dp(8.0);
const TOP_ICON_INDICATOR_VERTICAL_PADDING: Dp = Dp(4.0);
const START_ICON_INDICATOR_HORIZONTAL_PADDING: Dp = Dp(16.0);

fn interpolate_color(from: Color, to: Color, progress: f32) -> Color {
    Color {
        r: from.r + (to.r - from.r) * progress,
        g: from.g + (to.g - from.g) * progress,
        b: from.b + (to.b - from.b) * progress,
        a: from.a + (to.a - from.a) * progress,
    }
}

fn lerp_dp(from: Dp, to: Dp, progress: f32) -> Dp {
    let t = f64::from(progress.clamp(0.0, 1.0));
    Dp(from.0 + (to.0 - from.0) * t)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NavigationRailIconPosition {
    Top,
    Start,
}

#[derive(Clone, PartialEq)]
struct NavigationRailItemContentArgs {
    item: NavigationRailItem,
    icon_position: NavigationRailIconPosition,
    is_selected: bool,
    was_selected: bool,
    selection_progress: f32,
    indicator_start_width: Dp,
    item_min_height: Dp,
    interaction_state: State<InteractionState>,
    ripple_state: State<RippleState>,
}

#[tessera]
fn navigation_rail_item_content_node(args: &NavigationRailItemContentArgs) {
    let args = args.clone();
    let NavigationRailItemContentArgs {
        item,
        icon_position,
        is_selected,
        was_selected,
        selection_progress,
        indicator_start_width,
        item_min_height,
        interaction_state,
        ripple_state,
    } = args;
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

    let selection_fraction = if is_selected {
        selection_progress
    } else if was_selected {
        1.0 - selection_progress
    } else {
        0.0
    };

    let has_label = !item.label.is_empty();
    let has_icon = item.icon.is_some();

    let icon_color = interpolate_color(
        scheme.on_surface_variant,
        scheme.on_secondary_container,
        selection_fraction,
    );
    let label_color = interpolate_color(
        scheme.on_surface_variant,
        scheme.secondary,
        selection_fraction,
    );
    let ripple_color = icon_color;
    let indicator_color = scheme.secondary_container.with_alpha(selection_fraction);

    let indicator_base_width = match icon_position {
        NavigationRailIconPosition::Top => INDICATOR_TOP_WIDTH,
        NavigationRailIconPosition::Start => indicator_start_width,
    };
    let indicator_height = match icon_position {
        NavigationRailIconPosition::Top => INDICATOR_TOP_HEIGHT,
        NavigationRailIconPosition::Start => INDICATOR_START_HEIGHT,
    };
    let indicator_width_px = indicator_base_width.to_px();
    let animated_indicator_width_px = Px(((indicator_width_px.0 as f32)
        * selection_fraction.max(0.0))
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
                Some(DimensionValue::Fixed(indicator_height.to_px())),
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
        .modifier(Modifier::new().size(indicator_base_width, indicator_height))
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
                    .modifier(Modifier::new().size(indicator_base_width, indicator_height))
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
        let label_style = match icon_position {
            NavigationRailIconPosition::Top => typography.label_medium,
            NavigationRailIconPosition::Start => typography.label_large,
        };
        provide_text_style(label_style, move || {
            text(&crate::text::TextArgs::from(
                &TextArgs::default().text(&label).color(label_color),
            ));
        });
    }

    layout(NavigationRailItemLayout {
        icon_position,
        has_label,
        has_icon,
        item_min_height,
    });
}

#[derive(Clone, PartialEq)]
struct NavigationRailItemLayout {
    icon_position: NavigationRailIconPosition,
    has_label: bool,
    has_icon: bool,
    item_min_height: Dp,
}

impl LayoutSpec for NavigationRailItemLayout {
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
        let min_height = self.item_min_height.to_px();
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

        let child_constraint = Constraint::new(DimensionValue::WRAP, DimensionValue::WRAP);
        let children_to_measure: Vec<_> = input
            .children_ids()
            .iter()
            .map(|&child_id| (child_id, child_constraint))
            .collect();
        let children_results = input.measure_children(children_to_measure)?;

        let indicator_size = children_results
            .get(&indicator_background_id)
            .unwrap_or(&ComputedData::ZERO);
        let indicator_ripple_size = children_results
            .get(&indicator_ripple_id)
            .unwrap_or(&ComputedData::ZERO);
        let icon_size = icon_id
            .and_then(|id| children_results.get(&id))
            .unwrap_or(&ComputedData::ZERO);
        let label_size = label_id
            .and_then(|id| children_results.get(&id))
            .unwrap_or(&ComputedData::ZERO);

        let width = parent_width;
        let height = parent_height;

        match self.icon_position {
            NavigationRailIconPosition::Start => {
                let horizontal_padding = ITEM_HORIZONTAL_PADDING.to_px();
                let ripple_x = horizontal_padding;
                let ripple_y = (height - indicator_ripple_size.height) / 2;
                output.place_child(indicator_ripple_id, PxPosition::new(ripple_x, ripple_y));

                let indicator_x =
                    ripple_x + (indicator_ripple_size.width - indicator_size.width) / 2;
                let indicator_y = (height - indicator_size.height) / 2;
                output.place_child(
                    indicator_background_id,
                    PxPosition::new(indicator_x, indicator_y),
                );

                let content_x =
                    horizontal_padding + START_ICON_INDICATOR_HORIZONTAL_PADDING.to_px();

                if let Some(icon_id) = icon_id {
                    let icon_y = (height - icon_size.height) / 2;
                    output.place_child(icon_id, PxPosition::new(content_x, icon_y));
                }

                if let Some(label_id) = label_id {
                    let label_y = (height - label_size.height) / 2;
                    let label_x = if self.has_icon {
                        content_x + icon_size.width + START_ICON_TO_LABEL_PADDING.to_px()
                    } else {
                        content_x
                    };
                    output.place_child(label_id, PxPosition::new(label_x, label_y));
                }

                return Ok(ComputedData { width, height });
            }
            NavigationRailIconPosition::Top => {}
        }

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

            if let Some(icon_id) = icon_id {
                let icon_x = (width - icon_size.width) / 2;
                let icon_y = (height - icon_size.height) / 2;
                output.place_child(icon_id, PxPosition::new(icon_x, icon_y));
            }

            return Ok(ComputedData { width, height });
        }

        let indicator_vertical_padding_px = TOP_ICON_INDICATOR_VERTICAL_PADDING.to_px();
        let content_height = icon_size.height
            + indicator_vertical_padding_px
            + INDICATOR_TOP_TO_LABEL_PADDING.to_px()
            + label_size.height;
        let content_vertical_padding =
            ((height - content_height) / 2).max(indicator_vertical_padding_px);

        let icon_x = (width - icon_size.width) / 2;
        let label_x = (width - label_size.width) / 2;
        let indicator_x = (width - indicator_size.width) / 2;
        let ripple_x = (width - indicator_ripple_size.width) / 2;

        let indicator_y = content_vertical_padding - indicator_vertical_padding_px;
        let icon_y = content_vertical_padding;
        let ripple_y = indicator_y;
        let label_y = content_vertical_padding
            + icon_size.height
            + indicator_vertical_padding_px
            + INDICATOR_TOP_TO_LABEL_PADDING.to_px();

        output.place_child(
            indicator_background_id,
            PxPosition::new(indicator_x, indicator_y),
        );
        output.place_child(indicator_ripple_id, PxPosition::new(ripple_x, ripple_y));

        if let Some(icon_id) = icon_id {
            output.place_child(icon_id, PxPosition::new(icon_x, icon_y));
        }

        if let Some(label_id) = label_id {
            output.place_child(label_id, PxPosition::new(label_x, label_y));
        }

        Ok(ComputedData { width, height })
    }
}

#[derive(Clone, PartialEq)]
struct NavigationRailItemArgs {
    controller: State<NavigationRailController>,
    index: usize,
    item: NavigationRailItem,
    selected_index: usize,
    previous_index: usize,
    selection_progress: f32,
    icon_position: NavigationRailIconPosition,
    indicator_start_width: Dp,
    item_min_height: Dp,
}

#[derive(Clone, PartialEq)]
struct NavigationRailComposeArgs {
    controller: State<NavigationRailController>,
    items: Vec<NavigationRailItem>,
    header: Option<RenderSlot>,
}

#[derive(Clone, PartialEq)]
struct NavigationRailRenderArgs {
    controller: State<NavigationRailController>,
    items: Vec<NavigationRailItem>,
    header: Option<RenderSlot>,
}

#[tessera]
fn navigation_rail_item_node(args: &NavigationRailItemArgs) {
    let args = args.clone();
    let NavigationRailItemArgs {
        controller,
        index,
        item,
        selected_index,
        previous_index,
        selection_progress,
        icon_position,
        indicator_start_width,
        item_min_height,
    } = args;
    let interaction_state = remember(InteractionState::new);
    let ripple_state = remember(RippleState::new);

    let is_selected = index == selected_index;
    let was_selected = index == previous_index && selected_index != previous_index;
    let label = item.label.clone();
    let ripple_size = match icon_position {
        NavigationRailIconPosition::Top => {
            PxSize::new(INDICATOR_TOP_WIDTH.to_px(), INDICATOR_TOP_HEIGHT.to_px())
        }
        NavigationRailIconPosition::Start => PxSize::new(
            indicator_start_width.to_px(),
            INDICATOR_START_HEIGHT.to_px(),
        ),
    };

    let on_press = {
        move |ctx: PointerEventContext| {
            let spec = RippleSpec {
                bounded: true,
                radius: None,
            };
            ripple_state.with_mut(|state| {
                state.start_animation_with_spec(ctx.normalized_pos, ripple_size, spec);
            });
        }
    };
    let on_release = {
        move |_ctx: PointerEventContext| {
            ripple_state.with_mut(|state| state.release());
        }
    };

    let on_click_item = item.on_click.clone();
    let on_click = {
        move || {
            controller.with_mut(|c| c.set_selected(index));
            on_click_item.call();
        }
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
            let content_args = NavigationRailItemContentArgs {
                item: item.clone(),
                icon_position,
                is_selected,
                was_selected,
                selection_progress,
                indicator_start_width,
                item_min_height,
                interaction_state,
                ripple_state,
            };
            navigation_rail_item_content_node(&content_args);
        }
    });
}

/// Collapsed or expanded mode for a navigation rail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationRailValue {
    /// Compact, icon-top layout.
    Collapsed,
    /// Expanded, icon-start layout.
    Expanded,
}

impl NavigationRailValue {
    fn is_expanded(self) -> bool {
        matches!(self, Self::Expanded)
    }
}

/// Item configuration for [`navigation_rail`].
#[derive(Clone, PartialEq, Setters)]
pub struct NavigationRailItem {
    /// Text label shown next to or below the icon.
    #[setters(into)]
    pub label: String,
    /// Optional icon rendered for the item.
    #[setters(skip)]
    pub icon: Option<RenderSlot>,
    /// Callback invoked after selection changes to this item.
    #[setters(skip)]
    pub on_click: Callback,
}

impl NavigationRailItem {
    /// Creates a navigation item with the required label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            on_click: Callback::new(|| {}),
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

impl Default for NavigationRailItem {
    fn default() -> Self {
        Self::new("")
    }
}

/// # navigation_rail
///
/// Navigation rail that switches between collapsed and expanded layouts for
/// primary destinations.
///
/// ## Usage
///
/// Use for tablet and desktop layouts with 3-7 primary destinations.
///
/// ## Parameters
///
/// - `scope_config` — closure that registers items and an optional header via
///   [`NavigationRailScope`].
///
/// ## Examples
///
/// ```
/// use tessera_components::navigation_rail::{
///     NavigationRailController, NavigationRailItem, NavigationRailValue, navigation_rail,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(|| NavigationRailController::new(0));
///     controller.with_mut(|c| c.set_value(NavigationRailValue::Expanded));
///     assert!(controller.with(|c| c.is_expanded()));
///
///     let item = NavigationRailItem::new("Home");
///     assert_eq!(item.label, "Home");
///
///     navigation_rail(|scope| {
///         scope.item(item);
///     });
/// }
/// ```
pub fn navigation_rail<F>(scope_config: F)
where
    F: FnOnce(&mut NavigationRailScope),
{
    let controller = remember(|| NavigationRailController::new(0));
    let mut items = Vec::new();
    let mut header: Option<RenderSlot> = None;
    {
        let mut scope = NavigationRailScope {
            controller,
            items: &mut items,
            header: &mut header,
        };
        scope_config(&mut scope);
    }
    let render_args = NavigationRailComposeArgs {
        controller,
        items,
        header,
    };
    navigation_rail_node(&render_args);
}

#[tessera]
fn navigation_rail_node(args: &NavigationRailComposeArgs) {
    let render_args = NavigationRailRenderArgs {
        controller: args.controller,
        items: args.items.clone(),
        header: args.header.clone(),
    };
    navigation_rail_render_node(&render_args);
}

#[tessera]
fn navigation_rail_render_node(args: &NavigationRailRenderArgs) {
    let controller = args.controller;
    let items = args.items.clone();
    let header = args.header.clone();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let frame_nanos = current_frame_nanos();
    let selection_progress = controller
        .with_mut(|c| c.selection_animation_progress(frame_nanos))
        .unwrap_or(1.0);
    let selected_index = controller.with(|c| c.selected());
    let previous_index = controller.with(|c| c.previous_selected());
    let expand_fraction = controller.with_mut(|c| c.expand_fraction(frame_nanos));
    if controller.with(|c| c.is_animating(frame_nanos)) {
        let controller_for_frame = controller;
        receive_frame_nanos(move |frame_nanos| {
            let is_animating = controller_for_frame.with_mut(|controller| {
                let _ = controller.selection_animation_progress(frame_nanos);
                let _ = controller.expand_fraction(frame_nanos);
                controller.is_animating(frame_nanos)
            });
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    let icon_position = if expand_fraction >= 0.5 {
        NavigationRailIconPosition::Start
    } else {
        NavigationRailIconPosition::Top
    };

    let container_width = lerp_dp(COLLAPSED_WIDTH, EXPANDED_MIN_WIDTH, expand_fraction);
    let item_min_height = match icon_position {
        NavigationRailIconPosition::Top => TOP_ICON_ITEM_MIN_HEIGHT,
        NavigationRailIconPosition::Start => START_ICON_ITEM_MIN_HEIGHT,
    };
    let item_spacing = match icon_position {
        NavigationRailIconPosition::Top => ITEM_VERTICAL_SPACING_COLLAPSED,
        NavigationRailIconPosition::Start => ITEM_VERTICAL_SPACING_EXPANDED,
    };
    let indicator_start_width =
        Dp((container_width.0 - ITEM_HORIZONTAL_PADDING.0 * 2.0).max(INDICATOR_TOP_WIDTH.0));

    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_height().width(container_width))
            .style(scheme.surface.into())
            .block_input(true),
        move || {
            let header = header.clone();
            let items = items.clone();
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size().padding(Padding::new(
                        Dp::ZERO,
                        TOP_PADDING,
                        Dp::ZERO,
                        Dp::ZERO,
                    )))
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .cross_axis_alignment(CrossAxisAlignment::Start),
                move |column_scope| {
                    if let Some(header) = header.clone() {
                        column_scope.child(move || {
                            let header = header.clone();
                            row(
                                RowArgs::default().modifier(Modifier::new().padding(Padding::new(
                                    ITEM_HORIZONTAL_PADDING,
                                    Dp::ZERO,
                                    Dp::ZERO,
                                    Dp::ZERO,
                                ))),
                                move |row_scope| {
                                    row_scope.child(move || {
                                        header.render();
                                    });
                                },
                            );
                        });
                        column_scope.child(move || {
                            spacer(&crate::spacer::SpacerArgs::new(
                                Modifier::new().height(HEADER_BOTTOM_PADDING),
                            ));
                        });
                    }

                    let last_index = items.len().saturating_sub(1);
                    for (index, item) in items.iter().cloned().enumerate() {
                        column_scope.child(move || {
                            let item_args = NavigationRailItemArgs {
                                controller,
                                index,
                                item: item.clone(),
                                selected_index,
                                previous_index,
                                selection_progress,
                                icon_position,
                                indicator_start_width,
                                item_min_height,
                            };
                            navigation_rail_item_node(&item_args);
                        });

                        if index != last_index && item_spacing.0 > 0.0 {
                            let spacing = item_spacing;
                            column_scope.child(move || {
                                spacer(&crate::spacer::SpacerArgs::new(
                                    Modifier::new().height(spacing),
                                ));
                            });
                        }
                    }
                },
            );
        },
    ));
}

/// Controller for the `navigation_rail` component.
#[derive(Clone, PartialEq)]
pub struct NavigationRailController {
    selected: usize,
    previous_selected: usize,
    selection_start_frame_nanos: Option<u64>,
    expanded: bool,
    expand_start_frame_nanos: Option<u64>,
}

impl NavigationRailController {
    /// Create a new controller with the initial selected index.
    pub fn new(selected: usize) -> Self {
        Self::new_with_value(selected, NavigationRailValue::Collapsed)
    }

    /// Create a controller with an initial selection and expansion value.
    pub fn new_with_value(selected: usize, value: NavigationRailValue) -> Self {
        Self {
            selected,
            previous_selected: selected,
            selection_start_frame_nanos: None,
            expanded: value.is_expanded(),
            expand_start_frame_nanos: None,
        }
    }

    /// Returns the index of the currently selected navigation item.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Returns whether the rail is expanded.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Returns the current expansion value.
    pub fn value(&self) -> NavigationRailValue {
        if self.expanded {
            NavigationRailValue::Expanded
        } else {
            NavigationRailValue::Collapsed
        }
    }

    /// Programmatically select an item by index.
    pub fn select(&mut self, index: usize) {
        self.set_selected(index);
    }

    /// Expand the rail.
    pub fn expand(&mut self) {
        self.set_expanded(true);
    }

    /// Collapse the rail.
    pub fn collapse(&mut self) {
        self.set_expanded(false);
    }

    /// Toggle between collapsed and expanded states.
    pub fn toggle(&mut self) {
        self.set_expanded(!self.expanded);
    }

    /// Set the expansion value.
    pub fn set_value(&mut self, value: NavigationRailValue) {
        self.set_expanded(value.is_expanded());
    }

    fn set_selected(&mut self, index: usize) {
        if self.selected != index {
            self.previous_selected = self.selected;
            self.selected = index;
            self.selection_start_frame_nanos = Some(current_frame_nanos());
        }
    }

    fn set_expanded(&mut self, expanded: bool) {
        if self.expanded != expanded {
            self.expanded = expanded;
            let now_nanos = current_frame_nanos();
            if let Some(old_start_frame_nanos) = self.expand_start_frame_nanos {
                let elapsed_nanos = now_nanos.saturating_sub(old_start_frame_nanos);
                let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
                if elapsed_nanos < animation_nanos {
                    self.expand_start_frame_nanos =
                        Some(now_nanos.saturating_add(animation_nanos - elapsed_nanos));
                    return;
                }
            }
            self.expand_start_frame_nanos = Some(now_nanos);
        }
    }

    fn selection_animation_progress(&mut self, frame_nanos: u64) -> Option<f32> {
        if let Some(start_frame_nanos) = self.selection_start_frame_nanos {
            let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
            let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
            if elapsed_nanos < animation_nanos {
                Some(animation::easing(
                    elapsed_nanos as f32 / animation_nanos as f32,
                ))
            } else {
                self.selection_start_frame_nanos = None;
                None
            }
        } else {
            None
        }
    }

    fn expand_fraction(&mut self, frame_nanos: u64) -> f32 {
        let progress = calc_progress_from_timer(self.expand_start_frame_nanos, frame_nanos);
        if self.expanded {
            progress
        } else {
            1.0 - progress
        }
    }

    fn previous_selected(&self) -> usize {
        self.previous_selected
    }

    fn is_animating(&self, frame_nanos: u64) -> bool {
        let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
        self.selection_start_frame_nanos
            .is_some_and(|start_frame_nanos| {
                frame_nanos.saturating_sub(start_frame_nanos) < animation_nanos
            })
            || self
                .expand_start_frame_nanos
                .is_some_and(|start_frame_nanos| {
                    frame_nanos.saturating_sub(start_frame_nanos) < animation_nanos
                })
    }
}

impl Default for NavigationRailController {
    fn default() -> Self {
        Self::new(0)
    }
}

fn calc_progress_from_timer(animation_start_frame_nanos: Option<u64>, frame_nanos: u64) -> f32 {
    let raw = match animation_start_frame_nanos {
        None => 1.0,
        Some(start_frame_nanos) => {
            let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
            let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
            if elapsed_nanos >= animation_nanos {
                1.0
            } else {
                elapsed_nanos as f32 / animation_nanos as f32
            }
        }
    };
    animation::easing(raw)
}

/// Scope passed to the closure for defining children of the NavigationRail.
pub struct NavigationRailScope<'a> {
    controller: State<NavigationRailController>,
    items: &'a mut Vec<NavigationRailItem>,
    header: &'a mut Option<RenderSlot>,
}

impl<'a> NavigationRailScope<'a> {
    /// Returns the controller for expanded/collapsed and selection state.
    pub fn controller(&self) -> State<NavigationRailController> {
        self.controller
    }

    /// Set an optional header above the rail items.
    pub fn header<F>(&mut self, header: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        *self.header = Some(RenderSlot::new(header));
    }

    /// Add a navigation item to the rail.
    pub fn item<I>(&mut self, item: I)
    where
        I: Into<NavigationRailItem>,
    {
        self.items.push(item.into());
    }
}
