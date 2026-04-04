//! Material Design 3 navigation rail with collapsed and expanded layouts.
//!
//! ## Usage
//!
//! Use for primary destinations on wide layouts with a collapsible side rail.
use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, FocusTraversalPolicy,
    MeasurementError, Modifier, Px, PxPosition, PxSize, RenderSlot, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{LayoutInput, LayoutOutput, LayoutPolicy, layout_primitive},
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    animation,
    column::column,
    modifier::{InteractionState, ModifierExt, Padding, PointerEventContext, SelectableArgs},
    ripple_state::{RippleSpec, RippleState},
    row::row,
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    text::text,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum NavigationRailIconPosition {
    #[default]
    Top,
    Start,
}

#[derive(Clone)]
struct NavigationRailCompositionContext {
    controller: State<NavigationRailController>,
    selected_index: usize,
    previous_index: usize,
    selection_progress: f32,
    icon_position: NavigationRailIconPosition,
    indicator_start_width: Dp,
    item_min_height: Dp,
    item_spacing: Dp,
    next_index: Arc<Mutex<usize>>,
}

#[tessera]
fn navigation_rail_item_view_content(
    item: NavigationRailItemDefinition,
    icon_position: NavigationRailIconPosition,
    is_selected: bool,
    was_selected: bool,
    selection_progress: f32,
    indicator_start_width: Dp,
    item_min_height: Dp,
    interaction_state: Option<State<InteractionState>>,
    ripple_state: Option<State<RippleState>>,
) {
    let interaction_state = interaction_state.expect("interaction_state must be set");
    let ripple_state = ripple_state.expect("ripple_state must be set");
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

    layout_primitive()
        .layout_policy(NavigationRailItemLayout {
            icon_position,
            has_label,
            has_icon,
            item_min_height,
        })
        .child(move || {
            surface()
                .style(SurfaceStyle::Filled {
                    color: indicator_color,
                })
                .shape(Shape::CAPSULE)
                .modifier(Modifier::new().constrain(
                    Some(AxisConstraint::exact(animated_indicator_width_px)),
                    Some(AxisConstraint::exact(indicator_height.to_px())),
                ))
                .show_state_layer(false)
                .show_ripple(false)
                .with_child(|| {});

            surface()
                .style(SurfaceStyle::Filled {
                    color: Color::TRANSPARENT,
                })
                .shape(Shape::CAPSULE)
                .modifier(Modifier::new().size(indicator_base_width, indicator_height))
                .enabled(true)
                .interaction_state(interaction_state)
                .ripple_color(ripple_color)
                .with_child(move || {
                    surface()
                        .style(SurfaceStyle::Filled {
                            color: Color::TRANSPARENT,
                        })
                        .shape(Shape::CAPSULE)
                        .modifier(Modifier::new().size(indicator_base_width, indicator_height))
                        .enabled(true)
                        .ripple_color(ripple_color)
                        .show_state_layer(false)
                        .ripple_state(ripple_state)
                        .with_child(|| {});
                });

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
                    text().content(label.clone()).color(label_color);
                });
            }
        });
}

#[derive(Clone, PartialEq)]
struct NavigationRailItemLayout {
    icon_position: NavigationRailIconPosition,
    has_label: bool,
    has_icon: bool,
    item_min_height: Dp,
}

impl LayoutPolicy for NavigationRailItemLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let parent_width = input
            .parent_constraint()
            .width()
            .resolve_max()
            .unwrap_or(Px::ZERO);
        let min_height = self.item_min_height.to_px();
        let parent_height = input.parent_constraint().height().clamp(min_height);

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

        let child_constraint = Constraint::NONE;
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

#[tessera]
fn navigation_rail_item_view(
    controller: Option<State<NavigationRailController>>,
    index: usize,
    item: NavigationRailItemDefinition,
    selected_index: usize,
    previous_index: usize,
    selection_progress: f32,
    icon_position: NavigationRailIconPosition,
    indicator_start_width: Dp,
    item_min_height: Dp,
) {
    let controller = controller.expect("controller must be set");
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

    let on_click_item = item.on_click;
    let on_click = {
        move || {
            controller.with_mut(|c| c.set_selected(index));
            on_click_item.call();
        }
    };

    let selectable_args = SelectableArgs {
        selected: is_selected,
        on_click: on_click.into(),
        enabled: true,
        role: Some(Role::Tab),
        label: Some(label),
        interaction_state: Some(interaction_state),
        on_press: Some(on_press.into()),
        on_release: Some(on_release.into()),
        ..Default::default()
    };

    let modifier = Modifier::new().selectable_with(selectable_args);
    layout_primitive().modifier(modifier).child({
        let item = item.clone();
        move || {
            navigation_rail_item_view_content()
                .item(item.clone())
                .icon_position(icon_position)
                .is_selected(is_selected)
                .was_selected(was_selected)
                .selection_progress(selection_progress)
                .indicator_start_width(indicator_start_width)
                .item_min_height(item_min_height)
                .interaction_state(interaction_state)
                .ripple_state(ripple_state);
        }
    });
}

#[derive(Clone, PartialEq, Default)]
struct NavigationRailItemDefinition {
    label: String,
    icon: Option<RenderSlot>,
    on_click: Callback,
}

/// # navigation_rail_item
///
/// Renders a single destination inside [`navigation_rail`].
///
/// ## Usage
///
/// Declare one primary destination in a navigation rail content slot.
///
/// ## Parameters
///
/// - `label` — text label shown next to or below the icon.
/// - `icon` — optional icon rendered for the item.
/// - `on_click` — callback invoked after the item becomes selected.
///
/// ## Examples
///
/// ```
/// use tessera_components::navigation_rail::{navigation_rail, navigation_rail_item};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn demo() {
///     navigation_rail().content(|| {
///         navigation_rail_item().label("Home");
///     });
/// }
/// ```
#[tessera]
pub fn navigation_rail_item(
    #[prop(into)] label: String,
    icon: Option<RenderSlot>,
    on_click: Callback,
) {
    let composition = use_context::<NavigationRailCompositionContext>()
        .expect("navigation_rail_item must be used inside navigation_rail")
        .get();
    let index = {
        let mut next_index = composition.next_index.lock();
        let index = *next_index;
        *next_index += 1;
        index
    };

    layout_primitive()
        .modifier(Modifier::new().padding(Padding::new(
            Dp::ZERO,
            Dp::ZERO,
            Dp::ZERO,
            composition.item_spacing,
        )))
        .child(move || {
            let item = NavigationRailItemDefinition {
                label: label.clone(),
                icon,
                on_click,
            };
            navigation_rail_item_view()
                .controller(composition.controller)
                .index(index)
                .item(item)
                .selected_index(composition.selected_index)
                .previous_index(composition.previous_index)
                .selection_progress(composition.selection_progress)
                .icon_position(composition.icon_position)
                .indicator_start_width(composition.indicator_start_width)
                .item_min_height(composition.item_min_height);
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
/// - `controller` — optional external controller.
/// - `content` — item declarations rendered in the rail.
/// - `header` — optional header rendered above items.
///
/// ## Examples
///
/// ```
/// use tessera_components::navigation_rail::{
///     NavigationRailController, NavigationRailValue, navigation_rail, navigation_rail_item,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let controller = remember(|| NavigationRailController::new(0));
///     controller.with_mut(|c| c.set_value(NavigationRailValue::Expanded));
///     assert!(controller.with(|c| c.is_expanded()));
///
///     navigation_rail().controller(controller).content(|| {
///         navigation_rail_item().label("Home");
///     });
/// }
/// ```
#[tessera]
pub fn navigation_rail(
    controller: Option<State<NavigationRailController>>,
    content: RenderSlot,
    header: Option<RenderSlot>,
) {
    let controller = controller.unwrap_or_else(|| remember(|| NavigationRailController::new(0)));
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let frame_nanos = current_frame_nanos();
    let selection_progress = controller
        .with(|c| c.selection_animation_progress(frame_nanos))
        .unwrap_or(1.0);
    let selected_index = controller.with(|c| c.selected());
    let previous_index = controller.with(|c| c.previous_selected());
    let expand_fraction = controller.with(|c| c.expand_fraction(frame_nanos));
    if controller.with(|c| c.is_animating(frame_nanos)) {
        receive_frame_nanos(move |frame_nanos| {
            let is_animating = controller.with(|controller| controller.is_animating(frame_nanos));
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
    let composition = NavigationRailCompositionContext {
        controller,
        selected_index,
        previous_index,
        selection_progress,
        icon_position,
        indicator_start_width,
        item_min_height,
        item_spacing,
        next_index: Arc::new(Mutex::new(0)),
    };

    let modifier = Modifier::new()
        .focus_group()
        .focus_traversal_policy(FocusTraversalPolicy::vertical().wrap(true));
    layout_primitive().modifier(modifier).child({
        let composition = composition.clone();
        move || {
            let composition = composition.clone();
            surface()
                .modifier(Modifier::new().fill_max_height().width(container_width))
                .style(scheme.surface.into())
                .block_input(true)
                .with_child(move || {
                    let content_context = composition.clone();
                    column()
                        .modifier(Modifier::new().fill_max_size().padding(Padding::new(
                            Dp::ZERO,
                            TOP_PADDING,
                            Dp::ZERO,
                            Dp::ZERO,
                        )))
                        .main_axis_alignment(MainAxisAlignment::Start)
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .children(move || {
                            if let Some(header) = header {
                                row()
                                    .modifier(Modifier::new().padding(Padding::new(
                                        ITEM_HORIZONTAL_PADDING,
                                        Dp::ZERO,
                                        Dp::ZERO,
                                        Dp::ZERO,
                                    )))
                                    .children(move || {
                                        header.render();
                                    });
                                layout_primitive()
                                    .modifier(Modifier::new().height(HEADER_BOTTOM_PADDING));
                            }

                            let provided_context = content_context.clone();
                            provide_context(
                                move || provided_context.clone(),
                                move || {
                                    content.render();
                                },
                            );
                        });
                });
        }
    });
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

    fn selection_animation_progress(&self, frame_nanos: u64) -> Option<f32> {
        if let Some(start_frame_nanos) = self.selection_start_frame_nanos {
            let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
            let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
            if elapsed_nanos < animation_nanos {
                Some(animation::easing(
                    elapsed_nanos as f32 / animation_nanos as f32,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn expand_fraction(&self, frame_nanos: u64) -> f32 {
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
