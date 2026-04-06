//! Material Design 3 navigation bar for primary app destinations.
//!
//! ## Usage
//!
//! Use for bottom navigation between a small set of top-level destinations.
use std::{sync::Arc, time::Duration};

use parking_lot::Mutex;
use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, FocusTraversalPolicy,
    LayoutResult, MeasurementError, Modifier, Px, PxPosition, PxSize, RenderSlot, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{LayoutPolicy, MeasureScope, layout},
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    animation,
    column::column,
    modifier::{InteractionState, ModifierExt, PointerEventContext, SelectableArgs},
    ripple_state::{RippleSpec, RippleState},
    row::row,
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{ContentColor, MaterialTheme, provide_text_style},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(300);
const CONTAINER_HEIGHT: Dp = Dp(80.0);
const INDICATOR_WIDTH: Dp = Dp(56.0);
const INDICATOR_HEIGHT: Dp = Dp(32.0);
const DIVIDER_HEIGHT: Dp = Dp(1.0);
const INDICATOR_TO_LABEL_PADDING: Dp = Dp(4.0);
const INDICATOR_VERTICAL_PADDING: Dp = Dp(4.0);

fn interpolate_color(from: Color, to: Color, progress: f32) -> Color {
    Color {
        r: from.r + (to.r - from.r) * progress,
        g: from.g + (to.g - from.g) * progress,
        b: from.b + (to.b - from.b) * progress,
        a: from.a + (to.a - from.a) * progress,
    }
}

/// Controls label visibility for a navigation bar item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationBarLabelBehavior {
    /// Always render the label.
    #[default]
    AlwaysShow,
    /// Fade the label in only when the item is selected.
    SelectedOnly,
}

#[derive(Clone)]
struct NavigationBarCompositionContext {
    controller: State<NavigationBarController>,
    selected_index: usize,
    previous_index: usize,
    animation_progress: f32,
    next_index: Arc<Mutex<usize>>,
}

#[tessera]
fn navigation_bar_item_view_content(
    item: NavigationBarItemDefinition,
    is_selected: bool,
    was_selected: bool,
    animation_progress: f32,
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

    layout()
        .layout_policy(NavigationBarItemLayout {
            selection_fraction,
            always_show_label,
            has_label,
            has_icon,
        })
        .child(move || {
            surface()
                .style(SurfaceStyle::Filled {
                    color: indicator_color,
                })
                .shape(Shape::CAPSULE)
                .modifier(Modifier::new().constrain(
                    Some(AxisConstraint::exact(animated_indicator_width_px)),
                    Some(AxisConstraint::exact(INDICATOR_HEIGHT.to_px())),
                ))
                .show_state_layer(false)
                .show_ripple(false)
                .with_child(|| {});

            surface()
                .style(SurfaceStyle::Filled {
                    color: Color::TRANSPARENT,
                })
                .shape(Shape::CAPSULE)
                .modifier(Modifier::new().size(INDICATOR_WIDTH, INDICATOR_HEIGHT))
                .enabled(true)
                .interaction_state(interaction_state)
                .ripple_color(ripple_color)
                .with_child(move || {
                    surface()
                        .style(SurfaceStyle::Filled {
                            color: Color::TRANSPARENT,
                        })
                        .shape(Shape::CAPSULE)
                        .modifier(Modifier::new().size(INDICATOR_WIDTH, INDICATOR_HEIGHT))
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
                provide_text_style(typography.label_medium, move || {
                    text().content(label.clone()).color(label_color);
                });
            }
        });
}

#[derive(Clone, PartialEq)]
struct NavigationBarItemLayout {
    selection_fraction: f32,
    always_show_label: bool,
    has_label: bool,
    has_icon: bool,
}

impl LayoutPolicy for NavigationBarItemLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let parent_width = input
            .parent_constraint()
            .width()
            .resolve_max()
            .unwrap_or(Px::ZERO);

        let min_height = CONTAINER_HEIGHT.to_px();
        let parent_height = input.parent_constraint().height().clamp(min_height);

        let indicator_background = children[0];
        let indicator_ripple = children[1];
        let mut child_index = 2;

        let icon_id = if self.has_icon {
            let id = children[child_index];
            child_index += 1;
            Some(id)
        } else {
            None
        };

        let label_id = if self.has_label {
            let id = children[child_index];
            Some(id)
        } else {
            None
        };

        let child_constraint = Constraint::NONE;

        let indicator_size = indicator_background.measure(&child_constraint)?;
        let indicator_ripple_size = indicator_ripple.measure(&child_constraint)?;

        let icon_size = if let Some(icon_id) = icon_id {
            Some(icon_id.measure(&child_constraint)?)
        } else {
            None
        };

        let label_size = if let Some(label_id) = label_id {
            Some(label_id.measure(&child_constraint)?)
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
            result.place_child(
                indicator_background,
                PxPosition::new(indicator_x, indicator_y),
            );
            result.place_child(indicator_ripple, PxPosition::new(ripple_x, ripple_y));

            if let (Some(icon_id), Some(icon_size)) = (icon_id, icon_size) {
                let icon_x = (width - icon_size.width) / 2;
                let icon_y = (height - icon_size.height) / 2;
                result.place_child(icon_id, PxPosition::new(icon_x, icon_y));
            }

            return Ok(result.with_size(ComputedData { width, height }));
        }

        let icon_size = icon_size.map(|size| size.size()).unwrap_or(ComputedData {
            width: Px::ZERO,
            height: Px::ZERO,
        });
        let label_size = label_size.map(|size| size.size()).unwrap_or(ComputedData {
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

        result.place_child(
            indicator_background,
            PxPosition::new(indicator_x, Px(indicator_y.0 + offset.0)),
        );
        result.place_child(
            indicator_ripple,
            PxPosition::new(ripple_x, Px(ripple_y.0 + offset.0)),
        );

        if let Some(icon_id) = icon_id {
            result.place_child(icon_id, PxPosition::new(icon_x, Px(icon_y.0 + offset.0)));
        }

        if (self.always_show_label || self.selection_fraction != 0.0)
            && let Some(label_id) = label_id
        {
            result.place_child(label_id, PxPosition::new(label_x, Px(label_y.0 + offset.0)));
        }

        Ok(result.with_size(ComputedData { width, height }))
    }
}

#[tessera]
fn navigation_bar_item_view(
    controller: Option<State<NavigationBarController>>,
    index: usize,
    item: NavigationBarItemDefinition,
    selected_index: usize,
    previous_index: usize,
    animation_progress: f32,
) {
    let controller = controller.expect("controller must be set");
    let interaction_state = remember(InteractionState::new);
    let ripple_state = remember(RippleState::new);

    let is_selected = index == selected_index;
    let was_selected = index == previous_index && selected_index != previous_index;
    let label = item.label.clone();

    let on_press = move |ctx: PointerEventContext| {
        let spec = RippleSpec {
            bounded: true,
            radius: None,
        };
        ripple_state.with_mut(|state| {
            state.start_animation_with_spec(
                ctx.normalized_pos,
                PxSize::new(INDICATOR_WIDTH.to_px(), INDICATOR_HEIGHT.to_px()),
                spec,
            );
        });
    };
    let on_release = move |_ctx: PointerEventContext| {
        ripple_state.with_mut(|state| state.release());
    };

    let on_click_item = item.on_click;
    let on_click = move || {
        controller.with_mut(|c| c.set_selected(index));
        on_click_item.call();
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
    layout().modifier(modifier).child({
        let item = item.clone();
        move || {
            navigation_bar_item_view_content()
                .item(item.clone())
                .is_selected(is_selected)
                .was_selected(was_selected)
                .animation_progress(animation_progress)
                .interaction_state(interaction_state)
                .ripple_state(ripple_state);
        }
    });
}

#[derive(Clone, PartialEq, Default)]
struct NavigationBarItemDefinition {
    label: String,
    icon: Option<RenderSlot>,
    on_click: Callback,
    label_behavior: NavigationBarLabelBehavior,
}

/// # navigation_bar_item
///
/// Renders a single destination inside [`navigation_bar`].
///
/// ## Usage
///
/// Declare one primary destination in a navigation bar content slot.
///
/// ## Parameters
///
/// - `label` — text label shown under the icon.
/// - `icon` — optional icon rendered above the label.
/// - `on_click` — callback invoked after the item becomes selected.
/// - `label_behavior` — whether the label is always shown or only when
///   selected.
///
/// ## Examples
///
/// ```
/// use tessera_components::navigation_bar::{navigation_bar, navigation_bar_item};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn demo() {
///     navigation_bar().content(|| {
///         navigation_bar_item().label("Home");
///     });
/// }
/// ```
#[tessera]
pub fn navigation_bar_item(
    #[prop(into)] label: String,
    icon: Option<RenderSlot>,
    on_click: Callback,
    label_behavior: NavigationBarLabelBehavior,
) {
    let composition = use_context::<NavigationBarCompositionContext>()
        .expect("navigation_bar_item must be used inside navigation_bar")
        .get();
    let index = {
        let mut next_index = composition.next_index.lock();
        let index = *next_index;
        *next_index += 1;
        index
    };

    layout()
        .modifier(Modifier::new().weight(1.0))
        .child(move || {
            let item = NavigationBarItemDefinition {
                label: label.clone(),
                icon,
                on_click,
                label_behavior,
            };
            navigation_bar_item_view()
                .controller(composition.controller)
                .index(index)
                .item(item)
                .selected_index(composition.selected_index)
                .previous_index(composition.previous_index)
                .animation_progress(composition.animation_progress);
        });
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
/// - `controller` — optional external controller.
/// - `content` — item declarations rendered inside the bar.
///
/// ## Examples
///
/// ```
/// use tessera_components::navigation_bar::{navigation_bar, navigation_bar_item};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn demo() {
///     navigation_bar().content(|| {
///         navigation_bar_item().label("Home");
///         navigation_bar_item().label("Search");
///     });
/// }
/// ```
#[tessera]
pub fn navigation_bar(controller: Option<State<NavigationBarController>>, content: RenderSlot) {
    let controller = controller.unwrap_or_else(|| remember(|| NavigationBarController::new(0)));
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let frame_nanos = current_frame_nanos();

    let animation_progress = controller
        .with(|c| c.animation_progress(frame_nanos))
        .unwrap_or(1.0);
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
    let selected_index = controller.with(|c| c.selected());
    let previous_index = controller.with(|c| c.previous_selected());
    let composition = NavigationBarCompositionContext {
        controller,
        selected_index,
        previous_index,
        animation_progress,
        next_index: Arc::new(Mutex::new(0)),
    };

    let modifier = Modifier::new()
        .focus_group()
        .focus_traversal_policy(FocusTraversalPolicy::horizontal().wrap(true));
    layout().modifier(modifier).child({
        let composition = composition.clone();
        move || {
            let composition = composition.clone();
            surface()
                .modifier(Modifier::new().fill_max_width().height(CONTAINER_HEIGHT))
                .style(scheme.surface_container.into())
                .elevation(Dp(3.0))
                .block_input(true)
                .with_child(move || {
                    let composition = composition.clone();
                    let separator_color = scheme.outline_variant.with_alpha(0.12);
                    column()
                        .modifier(Modifier::new().fill_max_size())
                        .cross_axis_alignment(CrossAxisAlignment::Stretch)
                        .children(move || {
                            {
                                surface()
                                    .modifier(
                                        Modifier::new().fill_max_width().height(DIVIDER_HEIGHT),
                                    )
                                    .style(separator_color.into())
                                    .with_child(|| {});
                            };

                            let content_context = composition.clone();
                            row()
                                .modifier(Modifier::new().fill_max_size().weight(1.0))
                                .main_axis_alignment(MainAxisAlignment::Start)
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    let provided_context = content_context.clone();
                                    provide_context(
                                        move || provided_context.clone(),
                                        move || {
                                            content.render();
                                        },
                                    );
                                });
                        });
                });
        }
    });
}

/// Controller for the `navigation_bar` component.
#[derive(Clone, PartialEq)]
pub struct NavigationBarController {
    selected: usize,
    previous_selected: usize,
    animation_start_frame_nanos: Option<u64>,
}

impl NavigationBarController {
    /// Create a new state with an initial selected index.
    pub fn new(selected: usize) -> Self {
        Self {
            selected,
            previous_selected: selected,
            animation_start_frame_nanos: None,
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
            self.animation_start_frame_nanos = Some(current_frame_nanos());
        }
    }

    fn animation_progress(&self, frame_nanos: u64) -> Option<f32> {
        if let Some(start_frame_nanos) = self.animation_start_frame_nanos {
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

    fn is_animating(&self, frame_nanos: u64) -> bool {
        self.animation_start_frame_nanos
            .is_some_and(|start_frame_nanos| {
                let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
                let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
                elapsed_nanos < animation_nanos
            })
    }
}

impl Default for NavigationBarController {
    fn default() -> Self {
        Self::new(0)
    }
}
