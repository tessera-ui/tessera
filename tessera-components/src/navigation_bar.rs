//! Material Design 3 navigation bar for primary app destinations.
//!
//! ## Usage
//!
//! Use for bottom navigation between a small set of top-level destinations.
use std::time::Duration;

use tessera_foundation::gesture::TapRecognizer;
use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, FocusTraversalPolicy,
    LayoutResult, MeasurementError, Modifier, Px, PxPosition, PxSize, RenderSlot, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{MeasureScope, layout},
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    animation,
    modifier::{InteractionState, ModifierExt, PointerEventContext, SelectableArgs},
    ripple_state::{RippleSpec, RippleState},
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{ContentColor, MaterialTheme},
};

const ANIMATION_DURATION: Duration = Duration::from_millis(300);
const CONTAINER_HEIGHT: Dp = Dp(64.0);
const INDICATOR_WIDTH: Dp = Dp(56.0);
const INDICATOR_HEIGHT: Dp = Dp(32.0);
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

/// Position of the icon relative to the destination label.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationBarIconPosition {
    /// Switch from top to start at a container width of 600dp.
    #[default]
    Auto,
    /// Place the icon above the label on compact screens.
    Top,
    /// Place the icon before the label on wider screens.
    Start,
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

#[tessera]
fn navigation_bar_item_view_content(
    item: Option<NavigationBarItemDefinition>,
    selection_fraction: Option<f32>,
    interaction_state: Option<State<InteractionState>>,
    ripple_state: Option<State<RippleState>>,
) {
    let item = item.unwrap_or_default();
    let selection_fraction = selection_fraction.unwrap_or(0.0);
    let interaction_state = interaction_state.expect("interaction_state must be set");
    let ripple_state = ripple_state.expect("ripple_state must be set");
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

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
        if item.icon_position == NavigationBarIconPosition::Start {
            scheme.on_secondary_container
        } else {
            scheme.secondary
        },
        selection_fraction,
    );
    let label_color = label_color_base.with_alpha(label_color_base.a * label_alpha);

    let indicator_color = scheme.secondary_container.with_alpha(indicator_alpha);

    layout()
        .policy(NavigationBarItemLayout {
            selection_fraction,
            always_show_label,
            has_label,
            has_icon,
            icon_position: item.icon_position,
        })
        .child(move || {
            surface()
                .style(SurfaceStyle::Filled {
                    color: indicator_color,
                })
                .shape(Shape::CAPSULE)
                .show_state_layer(false)
                .show_ripple(false)
                .child(|| {});

            surface()
                .style(SurfaceStyle::Filled {
                    color: Color::TRANSPARENT,
                })
                .shape(Shape::CAPSULE)
                .enabled(true)
                .interaction_state(interaction_state)
                .ripple_color(ripple_color)
                .child(move || {
                    surface()
                        .style(SurfaceStyle::Filled {
                            color: Color::TRANSPARENT,
                        })
                        .shape(Shape::CAPSULE)
                        .modifier(Modifier::new().fill_max_size())
                        .enabled(true)
                        .ripple_color(ripple_color)
                        .show_state_layer(false)
                        .ripple_state(ripple_state)
                        .child(|| {});
                });

            if let Some(draw_icon) = item.icon {
                provide_context(
                    || ContentColor {
                        current: icon_color,
                    },
                    || {
                        let color = use_context::<ContentColor>().unwrap();
                        if color.with(|value| value.current) != icon_color {
                            color.set(ContentColor {
                                current: icon_color,
                            });
                        }
                        draw_icon.render();
                    },
                );
            }

            if has_label {
                let label = item.label.clone();
                provide_context(
                    || typography.label_medium,
                    move || {
                        text()
                            .content(label.clone())
                            .color(label_color)
                            .style(typography.label_medium);
                    },
                );
            }
        });
}

#[derive(Clone, PartialEq)]
struct NavigationBarItemLayout {
    selection_fraction: f32,
    always_show_label: bool,
    has_label: bool,
    has_icon: bool,
    icon_position: NavigationBarIconPosition,
}

impl tessera_ui::LayoutModifierNode for NavigationBarItemLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let parent_width = input
            .parent_constraint()
            .width()
            .resolve_max()
            .unwrap_or(Px::ZERO);

        let min_height = CONTAINER_HEIGHT.to_px();

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

        let icon_size = if let Some(icon_id) = icon_id {
            Some(icon_id.measure(input, &child_constraint)?)
        } else {
            None
        };

        let horizontal = self.icon_position == NavigationBarIconPosition::Start && self.has_label;
        let label_size = if let Some(label_id) = label_id {
            let reserved = if horizontal {
                icon_size.map_or(Px::ZERO, |size| size.width) + Dp(36.0).to_px()
            } else {
                Dp(8.0).to_px()
            };
            Some(label_id.measure(
                input,
                &Constraint::new(
                    AxisConstraint::new(Px::ZERO, Some((parent_width - reserved).max(Px::ZERO))),
                    AxisConstraint::NONE,
                ),
            )?)
        } else {
            None
        };

        let width = parent_width;
        let icon_height = icon_size.map_or(Px::ZERO, |size| size.height);
        let label_height = label_size.map_or(Px::ZERO, |size| size.height);
        let natural_height = if horizontal {
            icon_height.max(label_height) + Dp(40.0).to_px()
        } else if self.has_label {
            icon_height
                + INDICATOR_VERTICAL_PADDING.to_px() * 2
                + INDICATOR_TO_LABEL_PADDING.to_px()
                + label_height
                + Dp(12.0).to_px()
        } else {
            INDICATOR_HEIGHT.to_px() + Dp(12.0).to_px()
        };
        let height = input
            .parent_constraint()
            .height()
            .clamp(min_height.max(natural_height));
        let indicator_width = if horizontal {
            icon_size.map_or(Px::ZERO, |size| size.width)
                + label_size.map_or(Px::ZERO, |size| size.width)
                + Dp(36.0).to_px()
        } else {
            INDICATOR_WIDTH.to_px()
        };
        let indicator_height = if horizontal {
            Dp(40.0).to_px().max(label_height + Dp(16.0).to_px())
        } else {
            INDICATOR_HEIGHT.to_px()
        };
        let animated_width =
            Px((indicator_width.to_f32() * self.selection_fraction).round() as i32);
        let indicator_size = indicator_background.measure(
            input,
            &Constraint::new(
                AxisConstraint::exact(animated_width),
                AxisConstraint::exact(indicator_height),
            ),
        )?;
        let indicator_ripple_size = indicator_ripple.measure(
            input,
            &Constraint::new(
                AxisConstraint::exact(indicator_width),
                AxisConstraint::exact(indicator_height),
            ),
        )?;
        if horizontal {
            let x = (width - indicator_width) / 2;
            let y = (height - indicator_height) / 2;
            result.place_child(
                indicator_background,
                PxPosition::new((width - animated_width) / 2, y),
            );
            result.place_child(indicator_ripple, PxPosition::new(x, y));
            let icon_width = icon_size.map_or(Px::ZERO, |size| size.width);
            if let (Some(id), Some(size)) = (icon_id, icon_size) {
                result.place_child(
                    id,
                    PxPosition::new(x + Dp(16.0).to_px(), (height - size.height) / 2),
                );
            }
            if let (Some(id), Some(size)) = (label_id, label_size) {
                result.place_child(
                    id,
                    PxPosition::new(
                        x + Dp(20.0).to_px() + icon_width,
                        (height - size.height) / 2,
                    ),
                );
            }
            return Ok(result.with_size(ComputedData { width, height }));
        }

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
            + indicator_vertical_padding_px * 2
            + INDICATOR_TO_LABEL_PADDING.to_px()
            + label_size.height;

        let content_vertical_padding =
            ((height - content_height) / 2).max(indicator_vertical_padding_px);
        let selected_icon_y = content_vertical_padding + indicator_vertical_padding_px;
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
    item: Option<NavigationBarItemDefinition>,
    selected: Option<bool>,
    selection_fraction: Option<f32>,
    on_click: Option<Callback>,
) {
    let item = item.unwrap_or_default();
    let selected = selected.unwrap_or(false);
    let selection_fraction = selection_fraction.unwrap_or(0.0);
    let on_click = on_click.unwrap_or_default();
    let interaction_state = remember(InteractionState::new);
    let ripple_state = remember(RippleState::new);
    let tap_recognizer = remember(TapRecognizer::default);

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

    let selectable_args = SelectableArgs {
        selected,
        on_click,
        enabled: true,
        role: Some(Role::Tab),
        label: Some(label),
        description: item.accessibility_description.clone(),
        interaction_state: Some(interaction_state),
        on_press: Some(on_press.into()),
        on_release: Some(on_release.into()),
        tap_recognizer: Some(tap_recognizer),
        ..Default::default()
    };

    let modifier = Modifier::new().selectable_with(selectable_args);
    layout().modifier(modifier).child({
        let item = item.clone();
        move || {
            navigation_bar_item_view_content()
                .item(item.clone())
                .selection_fraction(selection_fraction)
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
    accessibility_description: Option<String>,
    label_behavior: NavigationBarLabelBehavior,
    icon_position: NavigationBarIconPosition,
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
/// - `selected` — whether this item is the active destination.
/// - `icon_position` — place the icon above or before the label.
/// - `on_click` — callback invoked after the item becomes selected.
/// - `accessibility_description` — additional announcement such as an unread
///   count.
/// - `label_behavior` — whether the label is always shown or only when
///   selected.
///
/// ## Examples
///
/// ```
/// use tessera_components::navigation_bar::{navigation_bar, navigation_bar_item};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let selected = remember(|| 0usize);
///     navigation_bar().content(move || {
///         navigation_bar_item()
///             .label("Home")
///             .selected(selected.get() == 0)
///             .on_click(move || selected.set(0));
///     });
/// }
/// ```
#[tessera]
pub fn navigation_bar_item(
    #[prop(into)] label: Option<String>,
    icon: Option<RenderSlot>,
    on_click: Option<Callback>,
    label_behavior: Option<NavigationBarLabelBehavior>,
    selected: Option<bool>,
    icon_position: Option<NavigationBarIconPosition>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let icon_position = icon_position
        .filter(|position| *position != NavigationBarIconPosition::Auto)
        .or_else(|| use_context::<NavigationBarIconPosition>().map(|position| position.get()))
        .unwrap_or(NavigationBarIconPosition::Top);
    let label = label.unwrap_or_default();
    let on_click = on_click.unwrap_or_default();
    let label_behavior = label_behavior.unwrap_or_default();
    let selected = selected.unwrap_or(false);

    // Each item owns its own selection animation. State is keyed by the item's
    // call position, so it stays stable across rebuilds without any shared
    // index registry.
    let animation = remember(|| NavigationBarItemAnimation::new(selected));
    let frame_nanos = current_frame_nanos();
    animation.with_mut(|state| state.set_selected(selected));
    let selection_fraction = animation.with_mut(|state| {
        state.update(frame_nanos);
        state.progress()
    });
    if animation.with(|state| state.is_animating()) {
        receive_frame_nanos(move |frame_nanos| {
            let is_animating = animation.with_mut(|state| {
                state.update(frame_nanos);
                state.is_animating()
            });
            if is_animating {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    layout()
        .modifier(Modifier::new().weight(1.0))
        .child(move || {
            let item = NavigationBarItemDefinition {
                label: label.clone(),
                icon,
                on_click,
                accessibility_description: accessibility_description.clone(),
                label_behavior,
                icon_position,
            };
            navigation_bar_item_view()
                .item(item)
                .selected(selected)
                .selection_fraction(selection_fraction)
                .on_click_shared(on_click);
        });
}

/// # navigation_bar
///
/// Material navigation bar with active indicator and icon/label pairs.
///
/// ## Usage
///
/// Place at the bottom of the app to switch between 3–5 primary destinations.
/// ## Parameters
///
/// - `content` — item declarations rendered inside the bar.
/// - `icon_position` — icon placement; defaults to adapting at 600dp.
///
/// ## Examples
///
/// ```
/// use tessera_components::navigation_bar::{navigation_bar, navigation_bar_item};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let selected = remember(|| 0usize);
///     navigation_bar().content(move || {
///         navigation_bar_item()
///             .label("Home")
///             .selected(selected.get() == 0)
///             .on_click(move || selected.set(0));
///         navigation_bar_item()
///             .label("Search")
///             .selected(selected.get() == 1)
///             .on_click(move || selected.set(1));
///     });
/// }
/// ```
#[tessera]
pub fn navigation_bar(
    content: Option<RenderSlot>,
    icon_position: Option<NavigationBarIconPosition>,
) {
    let icon_position = icon_position.unwrap_or_default();
    let wide = remember(|| false);
    let resolved_position = if icon_position == NavigationBarIconPosition::Auto {
        if wide.get() {
            NavigationBarIconPosition::Start
        } else {
            NavigationBarIconPosition::Top
        }
    } else {
        icon_position
    };
    let content = content.unwrap_or_else(RenderSlot::empty);
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;

    let modifier = Modifier::new()
        .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
            let value = bounds.width.to_f32() >= Dp(600.0).to_pixels_f32();
            if value != wide.get() {
                wide.set(value);
            }
        })
        .focus_group()
        .focus_traversal_policy(FocusTraversalPolicy::horizontal().wrap(true));
    layout().modifier(modifier).child({
        move || {
            surface()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .default_min_size(Dp::ZERO, CONTAINER_HEIGHT),
                )
                .style(scheme.surface_container.into())
                .block_input(true)
                .child(move || {
                    provide_context(
                        || resolved_position,
                        || {
                            let position = use_context::<NavigationBarIconPosition>().unwrap();
                            if position.get() != resolved_position {
                                position.set(resolved_position);
                            }
                            layout()
                                .policy(NavigationBarLayout {
                                    centered: resolved_position == NavigationBarIconPosition::Start,
                                })
                                .child(move || {
                                    content.render();
                                });
                        },
                    );
                });
        }
    });
}

#[derive(Clone, PartialEq)]
struct NavigationBarLayout {
    centered: bool,
}

impl tessera_ui::LayoutModifierNode for NavigationBarLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let width = input
            .parent_constraint()
            .width()
            .resolve_max()
            .unwrap_or(Px::ZERO);
        let minimum_height = CONTAINER_HEIGHT.to_px();
        let mut height = input.parent_constraint().height().clamp(minimum_height);
        let count = children.len();
        let padding = if self.centered && count <= 6 {
            Px((width.to_f32() * ((100.0 - 10.0 * (count as f32 + 3.0)) / 200.0)).round() as i32)
        } else {
            Px::ZERO
        };
        let available = width - padding * 2;
        let mut result = LayoutResult::default();
        let mut measured = Vec::with_capacity(count);
        let maximum_height = input.parent_constraint().height().resolve_max();
        for (index, child) in children.into_iter().enumerate() {
            let start = Px(available.0 * index as i32 / count as i32);
            let end = Px(available.0 * (index + 1) as i32 / count as i32);
            let size = child.measure(
                input,
                &Constraint::new(
                    AxisConstraint::exact(end - start),
                    AxisConstraint::new(minimum_height, maximum_height),
                ),
            )?;
            height = height.max(size.height);
            measured.push((child, start, end, size.height));
        }
        for (child, start, end, child_height) in measured {
            if child_height != height {
                child.measure(
                    input,
                    &Constraint::new(
                        AxisConstraint::exact(end - start),
                        AxisConstraint::exact(height),
                    ),
                )?;
            }
            result.place_child(child, PxPosition::new(padding + start, Px::ZERO));
        }
        Ok(result.with_size(ComputedData { width, height }))
    }
}

/// Per-item selection animation for a navigation bar destination.
///
/// Each item owns one of these; there is no shared index registry, so the
/// animation is keyed purely by the item's own call position.
#[derive(Clone, PartialEq)]
struct NavigationBarItemAnimation {
    selected: bool,
    progress: f32,
    start_progress: f32,
    last_change_frame_nanos: Option<u64>,
}

impl NavigationBarItemAnimation {
    fn new(selected: bool) -> Self {
        let progress = if selected { 1.0 } else { 0.0 };
        Self {
            selected,
            progress,
            start_progress: progress,
            last_change_frame_nanos: None,
        }
    }

    fn set_selected(&mut self, selected: bool) {
        if self.selected != selected {
            self.selected = selected;
            self.start_progress = self.progress;
            self.last_change_frame_nanos = Some(current_frame_nanos());
        }
    }

    fn target(&self) -> f32 {
        if self.selected { 1.0 } else { 0.0 }
    }

    fn update(&mut self, frame_nanos: u64) {
        if let Some(start_frame_nanos) = self.last_change_frame_nanos {
            let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
            let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
            let fraction = if animation_nanos == 0 {
                1.0
            } else {
                (elapsed_nanos as f32 / animation_nanos as f32).min(1.0)
            };
            let eased = animation::easing(fraction);
            let target = self.target();
            self.progress = self.start_progress + (target - self.start_progress) * eased;
            if fraction >= 1.0 {
                self.last_change_frame_nanos = None;
                self.progress = target;
                self.start_progress = target;
            }
        }
    }

    fn progress(&self) -> f32 {
        self.progress
    }

    fn is_animating(&self) -> bool {
        self.last_change_frame_nanos.is_some()
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::{
        CursorEventContent, Dp, GestureState, Modifier, PointerChange, PressKeyEventType, Px,
        PxPosition, remember, tessera,
    };

    use crate::{
        column::column,
        lazy_list::lazy_column,
        modifier::{ModifierExt as _, SemanticsArgs},
        spacer::spacer,
        theme::{MaterialTheme, material_theme},
    };

    use super::{navigation_bar, navigation_bar_item};

    #[tessera]
    fn navigation_bar_row() {
        let selected = remember(|| 0usize);
        column().children(move || {
            navigation_bar().content(move || {
                navigation_bar_item()
                    .label("Home")
                    .selected(selected.get() == 0)
                    .on_click(move || selected.set(0));
                navigation_bar_item()
                    .label("Search")
                    .selected(selected.get() == 1)
                    .on_click(move || selected.set(1));
                navigation_bar_item()
                    .label("Settings")
                    .selected(selected.get() == 2)
                    .on_click(move || selected.set(2));
            });
            if selected.get() == 1 {
                spacer().modifier(Modifier::new().width(Dp(10.0)).height(Dp(10.0)).semantics(
                    SemanticsArgs {
                        test_tag: Some("nav-search-active".to_string()),
                        ..Default::default()
                    },
                ));
            }
        });
    }

    #[tessera]
    fn navigation_bar_click_probe() {
        material_theme()
            .theme(MaterialTheme::default)
            .child(move || {
                lazy_column().item(move || {
                    navigation_bar_row();
                });
            });
    }

    #[test]
    fn clicking_middle_item_selects_middle_item() {
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(|| {
                navigation_bar_click_probe();
            })
            .viewport_px(300, 200),
        );
        let _ = tessera_ui::testing::pump_layout_test_session(&mut session, 0);

        // Three equal items span the 300px viewport, so the middle item covers
        // x in [100, 200] and its center is at x = 150.
        let now = tessera_ui::time::Instant::now();
        let cursor = Some(PxPosition::new(Px::new(150), Px::new(40)));

        // Move the pointer over the middle item first so it rebuilds in
        // isolation inside `lazy_column`; the per-item selection must still
        // resolve to the middle destination after the click.
        let moved = PointerChange::new(
            now,
            1,
            CursorEventContent::Moved(PxPosition::new(Px::new(150), Px::new(40))),
            GestureState::TapCandidate,
        );
        let _ = tessera_ui::testing::pump_layout_test_session_with_input(
            &mut session,
            16_666_667,
            vec![moved],
            cursor,
        );

        let press = PointerChange::new(
            now,
            1,
            CursorEventContent::Pressed(PressKeyEventType::Left),
            GestureState::TapCandidate,
        );
        let _ = tessera_ui::testing::pump_layout_test_session_with_input(
            &mut session,
            33_333_334,
            vec![press],
            cursor,
        );
        let release = PointerChange::new(
            now,
            1,
            CursorEventContent::Released(PressKeyEventType::Left),
            GestureState::TapCandidate,
        );
        let mut snapshot = tessera_ui::testing::pump_layout_test_session_with_input(
            &mut session,
            50_000_001,
            vec![release],
            cursor,
        );
        for frame in 3..40u64 {
            snapshot = tessera_ui::testing::pump_layout_test_session(
                &mut session,
                50_000_001 + 16_666_667 * (frame - 2),
            );
        }

        snapshot.node("nav-search-active").exists();
    }
}
