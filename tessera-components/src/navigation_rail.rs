//! Material Design 3 navigation rail with collapsed and expanded layouts.
//!
//! ## Usage
//!
//! Use for primary destinations on wide layouts with a collapsible side rail.
use std::time::Duration;

use tessera_foundation::gesture::TapRecognizer;
use tessera_ui::{
    AxisConstraint, Callback, Color, ComputedData, Constraint, Dp, FocusTraversalPolicy,
    LayoutResult, MeasurementError, Modifier, Px, PxPosition, PxRect, PxSize, RenderSlot, State,
    accesskit::Role,
    current_frame_nanos,
    layout::{MeasureScope, layout},
    modifier::FocusModifierExt as _,
    provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    animation,
    badge::BadgeDefaults,
    column::column,
    modifier::{InteractionState, ModifierExt, Padding, PointerEventContext, SelectableArgs},
    ripple_state::{RippleSpec, RippleState},
    row::row,
    shape_def::Shape,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{ContentColor, MaterialTheme},
};

#[cfg(test)]
#[path = "navigation_destination_tests.rs"]
mod destination_tests;

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

#[derive(Clone, PartialEq)]
struct NavigationRailCompositionContext {
    icon_position: NavigationRailIconPosition,
    indicator_start_width: Dp,
    item_min_height: Dp,
    item_spacing: Dp,
}

#[tessera]
fn navigation_rail_item_view_content(
    item: Option<NavigationRailItemDefinition>,
    icon_position: Option<NavigationRailIconPosition>,
    selection_fraction: Option<f32>,
    item_min_height: Option<Dp>,
    interaction_state: Option<State<InteractionState>>,
    ripple_state: Option<State<RippleState>>,
    indicator_geometry: Option<State<RailRippleGeometry>>,
) {
    let item = item.unwrap_or_default();
    let icon_position = icon_position.unwrap_or_default();
    let selection_fraction = selection_fraction.unwrap_or(0.0);
    let item_min_height = item_min_height.unwrap_or(Dp(0.0));
    let interaction_state = interaction_state.expect("interaction_state must be set");
    let ripple_state = ripple_state.expect("ripple_state must be set");
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

    let has_label = !item.label.is_empty();
    let has_icon = item.icon.is_some();
    let has_badge = item.badge.is_some();

    let icon_color = interpolate_color(
        scheme.on_surface_variant,
        scheme.on_secondary_container,
        selection_fraction,
    );
    let label_color = interpolate_color(
        scheme.on_surface_variant,
        if icon_position == NavigationRailIconPosition::Start {
            scheme.on_secondary_container
        } else {
            scheme.secondary
        },
        selection_fraction,
    );
    let ripple_color = icon_color;
    let indicator_color = scheme.secondary_container.with_alpha(selection_fraction);

    layout()
        .policy(NavigationRailItemLayout {
            icon_position,
            has_label,
            has_icon,
            has_badge,
            selection_fraction,
            item_min_height,
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
                .modifier(Modifier::new().on_globally_positioned(
                    move |bounds: tessera_ui::PxRect| {
                        if let Some(indicator_geometry) = indicator_geometry {
                            if indicator_geometry.with(|geometry| geometry.indicator)
                                != Some(bounds)
                            {
                                indicator_geometry
                                    .with_mut(|geometry| geometry.indicator = Some(bounds));
                            }
                        }
                    },
                ))
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
                let label_style = match icon_position {
                    NavigationRailIconPosition::Top => typography.label_medium,
                    NavigationRailIconPosition::Start => typography.label_large,
                };
                provide_context(
                    || label_style,
                    move || {
                        text()
                            .content(label.clone())
                            .color(label_color)
                            .style(label_style);
                    },
                );
            }
            if let Some(badge) = item.badge {
                crate::boxed::boxed().children_shared(badge);
            }
        });
}

#[derive(Clone, PartialEq)]
struct NavigationRailItemLayout {
    icon_position: NavigationRailIconPosition,
    has_label: bool,
    has_icon: bool,
    has_badge: bool,
    selection_fraction: f32,
    item_min_height: Dp,
}

impl tessera_ui::LayoutModifierNode for NavigationRailItemLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let parent_width = input
            .parent_constraint()
            .width()
            .resolve_max()
            .unwrap_or(Px::ZERO);
        let min_height = self.item_min_height.to_px();

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
            child_index += 1;
            Some(id)
        } else {
            None
        };

        let child_constraint = Constraint::NONE;
        let icon_size = icon_id
            .map(|id| id.measure(input, &child_constraint))
            .transpose()?
            .map(|size| size.size())
            .unwrap_or(ComputedData::ZERO);
        let badge_id = self.has_badge.then(|| children[child_index]);
        let badge_size = badge_id
            .map(|id| id.measure(input, &child_constraint))
            .transpose()?
            .map(|size| size.size())
            .unwrap_or(ComputedData::ZERO);
        let label_size = label_id
            .map(|id| {
                id.measure(
                    input,
                    &Constraint::new(
                        AxisConstraint::new(
                            Px::ZERO,
                            Some(
                                (parent_width
                                    - if self.icon_position == NavigationRailIconPosition::Start {
                                        ITEM_HORIZONTAL_PADDING.to_px() * 2
                                            + START_ICON_INDICATOR_HORIZONTAL_PADDING.to_px() * 2
                                            + icon_size.width
                                            + if self.has_icon {
                                                START_ICON_TO_LABEL_PADDING.to_px()
                                            } else {
                                                Px::ZERO
                                            }
                                            + badge_size.width
                                            + if self.has_badge {
                                                Dp(8.0).to_px()
                                            } else {
                                                Px::ZERO
                                            }
                                    } else {
                                        Dp(8.0).to_px()
                                    })
                                .max(Px::ZERO),
                            ),
                        ),
                        AxisConstraint::NONE,
                    ),
                )
            })
            .transpose()?
            .map(|size| size.size())
            .unwrap_or(ComputedData::ZERO);

        let expanded = self.icon_position == NavigationRailIconPosition::Start;
        let label_gap = if self.has_icon && self.has_label {
            START_ICON_TO_LABEL_PADDING.to_px()
        } else {
            Px::ZERO
        };
        let badge_gap = if self.has_badge && (self.has_icon || self.has_label) {
            Dp(8.0).to_px()
        } else {
            Px::ZERO
        };
        let content_width =
            icon_size.width + label_gap + label_size.width + badge_gap + badge_size.width;
        let base_width = if expanded {
            (content_width + START_ICON_INDICATOR_HORIZONTAL_PADDING.to_px() * 2)
                .min((parent_width - ITEM_HORIZONTAL_PADDING.to_px() * 2).max(Px::ZERO))
        } else {
            INDICATOR_TOP_WIDTH.to_px()
        };
        let indicator_height = if expanded {
            INDICATOR_START_HEIGHT.to_px().max(
                icon_size
                    .height
                    .max(label_size.height)
                    .max(badge_size.height)
                    + START_ICON_INDICATOR_HORIZONTAL_PADDING.to_px() * 2,
            )
        } else {
            INDICATOR_TOP_HEIGHT.to_px()
        };
        let animated_width =
            Px((base_width.0 as f32 * self.selection_fraction.clamp(0.0, 1.0)).round() as i32);
        let indicator_size = indicator_background
            .measure(
                input,
                &Constraint::new(
                    AxisConstraint::exact(animated_width),
                    AxisConstraint::exact(indicator_height),
                ),
            )?
            .size();
        let indicator_ripple_size = indicator_ripple
            .measure(
                input,
                &Constraint::new(
                    AxisConstraint::exact(base_width),
                    AxisConstraint::exact(indicator_height),
                ),
            )?
            .size();
        let width = parent_width;
        let content_height = if expanded {
            icon_size
                .height
                .max(label_size.height)
                .max(badge_size.height)
                + START_ICON_INDICATOR_HORIZONTAL_PADDING.to_px() * 2
        } else if self.has_label {
            icon_size.height
                + label_size.height
                + TOP_ICON_INDICATOR_VERTICAL_PADDING.to_px() * 3
                + INDICATOR_TOP_TO_LABEL_PADDING.to_px()
        } else {
            icon_size.height.max(INDICATOR_TOP_HEIGHT.to_px())
        };
        let height = input
            .parent_constraint()
            .height()
            .clamp(min_height.max(content_height));

        match self.icon_position {
            NavigationRailIconPosition::Start => {
                let horizontal_padding = ITEM_HORIZONTAL_PADDING.to_px();
                let ripple_x = horizontal_padding;
                let ripple_y = (height - indicator_ripple_size.height) / 2;
                result.place_child(indicator_ripple, PxPosition::new(ripple_x, ripple_y));

                let indicator_x =
                    ripple_x + (indicator_ripple_size.width - indicator_size.width) / 2;
                let indicator_y = (height - indicator_size.height) / 2;
                result.place_child(
                    indicator_background,
                    PxPosition::new(indicator_x, indicator_y),
                );

                let content_x =
                    horizontal_padding + START_ICON_INDICATOR_HORIZONTAL_PADDING.to_px();

                if let Some(icon_id) = icon_id {
                    let icon_y = (height - icon_size.height) / 2;
                    result.place_child(icon_id, PxPosition::new(content_x, icon_y));
                }

                if let Some(label_id) = label_id {
                    let label_y = (height - label_size.height) / 2;
                    let label_x = if self.has_icon {
                        content_x + icon_size.width + START_ICON_TO_LABEL_PADDING.to_px()
                    } else {
                        content_x
                    };
                    result.place_child(label_id, PxPosition::new(label_x, label_y));
                }

                if let Some(badge_id) = badge_id {
                    let badge_x =
                        content_x + icon_size.width + label_gap + label_size.width + badge_gap;
                    result.place_child(
                        badge_id,
                        PxPosition::new(badge_x, (height - badge_size.height) / 2),
                    );
                }

                return Ok(result.with_size(ComputedData { width, height }));
            }
            NavigationRailIconPosition::Top => {}
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

            if let Some(icon_id) = icon_id {
                let icon_x = (width - icon_size.width) / 2;
                let icon_y = (height - icon_size.height) / 2;
                result.place_child(icon_id, PxPosition::new(icon_x, icon_y));
            }

            if let Some(badge_id) = badge_id {
                let large = badge_size.width > BadgeDefaults::SIZE.to_px();
                let dx = if large {
                    BadgeDefaults::WITH_CONTENT_HORIZONTAL_OFFSET
                } else {
                    BadgeDefaults::OFFSET
                }
                .to_px();
                let dy = if large {
                    BadgeDefaults::WITH_CONTENT_VERTICAL_OFFSET
                } else {
                    BadgeDefaults::OFFSET
                }
                .to_px();
                result.place_child(
                    badge_id,
                    PxPosition::new(
                        (width + icon_size.width) / 2 - dx,
                        (height - icon_size.height) / 2 - badge_size.height + dy,
                    ),
                );
            }

            return Ok(result.with_size(ComputedData { width, height }));
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

        result.place_child(
            indicator_background,
            PxPosition::new(indicator_x, indicator_y),
        );
        result.place_child(indicator_ripple, PxPosition::new(ripple_x, ripple_y));

        if let Some(icon_id) = icon_id {
            result.place_child(icon_id, PxPosition::new(icon_x, icon_y));
        }

        if let Some(label_id) = label_id {
            result.place_child(label_id, PxPosition::new(label_x, label_y));
        }

        if let Some(badge_id) = badge_id {
            let large = badge_size.width > BadgeDefaults::SIZE.to_px();
            let dx = if large {
                BadgeDefaults::WITH_CONTENT_HORIZONTAL_OFFSET
            } else {
                BadgeDefaults::OFFSET
            }
            .to_px();
            let dy = if large {
                BadgeDefaults::WITH_CONTENT_VERTICAL_OFFSET
            } else {
                BadgeDefaults::OFFSET
            }
            .to_px();
            result.place_child(
                badge_id,
                PxPosition::new(
                    icon_x + icon_size.width - dx,
                    icon_y - badge_size.height + dy,
                ),
            );
        }

        Ok(result.with_size(ComputedData { width, height }))
    }
}

#[tessera]
fn navigation_rail_item_view(
    item: Option<NavigationRailItemDefinition>,
    selected: Option<bool>,
    selection_fraction: Option<f32>,
    icon_position: Option<NavigationRailIconPosition>,
    indicator_start_width: Option<Dp>,
    item_min_height: Option<Dp>,
) {
    let item = item.unwrap_or_default();
    let selected = selected.unwrap_or(false);
    let selection_fraction = selection_fraction.unwrap_or(0.0);
    let icon_position = icon_position.unwrap_or_default();
    let indicator_start_width = indicator_start_width.unwrap_or(Dp(0.0));
    let item_min_height = item_min_height.unwrap_or(Dp(0.0));
    let interaction_state = remember(InteractionState::new);
    let ripple_state = remember(RippleState::new);
    let tap_recognizer = remember(TapRecognizer::default);

    let label = item.label.clone();
    let description = item.accessibility_description.clone();
    let on_click = item.on_click;
    let ripple_size = match icon_position {
        NavigationRailIconPosition::Top => {
            PxSize::new(INDICATOR_TOP_WIDTH.to_px(), INDICATOR_TOP_HEIGHT.to_px())
        }
        NavigationRailIconPosition::Start => PxSize::new(
            indicator_start_width.to_px(),
            INDICATOR_START_HEIGHT.to_px(),
        ),
    };

    let indicator_geometry = remember(RailRippleGeometry::default);
    let on_press = {
        move |ctx: PointerEventContext| {
            let spec = RippleSpec {
                bounded: true,
                radius: None,
            };
            ripple_state.with_mut(|state| {
                let geometry = indicator_geometry.get();
                let (position, size) = if let (Some(target), Some(indicator)) =
                    (geometry.target, geometry.indicator)
                {
                    (
                        [
                            (target.x.to_f32() + ctx.normalized_pos[0] * ctx.size.width.to_f32()
                                - indicator.x.to_f32())
                                / indicator.width.to_f32().max(1.0),
                            (target.y.to_f32() + ctx.normalized_pos[1] * ctx.size.height.to_f32()
                                - indicator.y.to_f32())
                                / indicator.height.to_f32().max(1.0),
                        ],
                        PxSize::new(indicator.width, indicator.height),
                    )
                } else {
                    (ctx.normalized_pos, ripple_size)
                };
                state.start_animation_with_spec(position, size, spec);
            });
        }
    };
    let on_release = {
        move |_ctx: PointerEventContext| {
            ripple_state.with_mut(|state| state.release());
        }
    };

    let selectable_args = SelectableArgs {
        selected,
        on_click,
        enabled: true,
        role: Some(Role::Tab),
        label: Some(label),
        description,
        interaction_state: Some(interaction_state),
        on_press: Some(on_press.into()),
        on_release: Some(on_release.into()),
        tap_recognizer: Some(tap_recognizer),
        ..Default::default()
    };

    let modifier = Modifier::new()
        .selectable_with(selectable_args)
        .on_globally_positioned(move |bounds: PxRect| {
            if indicator_geometry.with(|geometry| geometry.target) != Some(bounds) {
                indicator_geometry.with_mut(|geometry| geometry.target = Some(bounds));
            }
        });
    layout().modifier(modifier).child({
        let item = item.clone();
        move || {
            navigation_rail_item_view_content()
                .item(item.clone())
                .icon_position(icon_position)
                .selection_fraction(selection_fraction)
                .item_min_height(item_min_height)
                .interaction_state(interaction_state)
                .ripple_state(ripple_state)
                .indicator_geometry(indicator_geometry);
        }
    });
}

#[derive(Clone, Copy, Default, PartialEq)]
struct RailRippleGeometry {
    target: Option<PxRect>,
    indicator: Option<PxRect>,
}

#[derive(Clone, PartialEq, Default)]
struct NavigationRailItemDefinition {
    label: String,
    icon: Option<RenderSlot>,
    badge: Option<RenderSlot>,
    accessibility_description: Option<String>,
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
/// - `badge` — optional update marker, overlapping a collapsed icon or
///   following an expanded label.
/// - `accessibility_description` — additional announcement such as a
///   notification count.
/// - `selected` — whether this item is the active destination.
/// - `on_click` — callback invoked after the item becomes selected.
///
/// ## Examples
///
/// ```
/// use tessera_components::navigation_rail::{navigation_rail, navigation_rail_item};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let selected = remember(|| 0usize);
///     navigation_rail().content(move || {
///         navigation_rail_item()
///             .label("Home")
///             .selected(selected.get() == 0)
///             .on_click(move || selected.set(0));
///     });
/// }
/// ```
#[tessera]
pub fn navigation_rail_item(
    #[prop(into)] label: Option<String>,
    icon: Option<RenderSlot>,
    on_click: Option<Callback>,
    selected: Option<bool>,
    badge: Option<RenderSlot>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let label = label.unwrap_or_default();
    let on_click = on_click.unwrap_or_default();
    let selected = selected.unwrap_or(false);
    let composition = use_context::<NavigationRailCompositionContext>()
        .expect("navigation_rail_item must be used inside navigation_rail")
        .get();

    // Each item owns its own selection animation, keyed by call position, so
    // there is no shared index registry that could drift across rebuilds.
    let animation = remember(|| NavigationRailItemAnimation::new(selected));
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

    let item_spacing = composition.item_spacing;
    layout()
        .modifier(Modifier::new().padding(Padding::new(Dp::ZERO, Dp::ZERO, Dp::ZERO, item_spacing)))
        .child(move || {
            let item = NavigationRailItemDefinition {
                label: label.clone(),
                icon,
                badge,
                accessibility_description: accessibility_description.clone(),
                on_click,
            };
            navigation_rail_item_view()
                .item(item)
                .selected(selected)
                .selection_fraction(selection_fraction)
                .icon_position(composition.icon_position)
                .indicator_start_width(composition.indicator_start_width)
                .item_min_height(composition.item_min_height);
        });
}

/// # navigation_rail
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
/// - `expanded` — whether the rail renders in expanded (icon-start) mode.
/// - `content` — item declarations rendered in the rail.
/// - `header` — optional header rendered above items.
///
/// ## Examples
///
/// ```
/// use tessera_components::navigation_rail::{navigation_rail, navigation_rail_item};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let selected = remember(|| 0usize);
///     navigation_rail().expanded(true).content(move || {
///         navigation_rail_item()
///             .label("Home")
///             .selected(selected.get() == 0)
///             .on_click(move || selected.set(0));
///     });
/// }
/// ```
#[tessera]
pub fn navigation_rail(
    expanded: Option<bool>,
    content: Option<RenderSlot>,
    header: Option<RenderSlot>,
) {
    let content = content.unwrap_or_else(RenderSlot::empty);
    let expanded = expanded.unwrap_or(false);
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let controller = remember(NavigationRailExpansion::default);
    let frame_nanos = current_frame_nanos();
    controller.with_mut(|c| c.set_expanded(expanded));
    let expand_fraction = controller.with_mut(|c| {
        c.update(frame_nanos);
        c.expand_fraction()
    });
    if controller.with(|c| c.is_animating()) {
        receive_frame_nanos(move |frame_nanos| {
            let is_animating = controller.with_mut(|c| {
                c.update(frame_nanos);
                c.is_animating()
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
    let composition = NavigationRailCompositionContext {
        icon_position,
        indicator_start_width,
        item_min_height,
        item_spacing,
    };

    let modifier = Modifier::new()
        .focus_group()
        .focus_traversal_policy(FocusTraversalPolicy::vertical().wrap(true));
    layout().modifier(modifier).child({
        let composition = composition.clone();
        move || {
            let composition = composition.clone();
            surface()
                .modifier(Modifier::new().fill_max_height().width(container_width))
                .style(scheme.surface.into())
                .block_input(true)
                .child(move || {
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
                                layout().modifier(Modifier::new().height(HEADER_BOTTOM_PADDING));
                            }

                            let provided_context = content_context.clone();
                            let content_context = content_context.clone();
                            provide_context(
                                move || provided_context.clone(),
                                move || {
                                    let current =
                                        use_context::<NavigationRailCompositionContext>().unwrap();
                                    if current.get() != content_context {
                                        current.set(content_context.clone());
                                    }
                                    content.render();
                                },
                            );
                        });
                });
        }
    });
}

/// Per-item selection animation for a navigation rail destination.
#[derive(Clone, PartialEq)]
struct NavigationRailItemAnimation {
    selected: bool,
    progress: f32,
    start_progress: f32,
    last_change_frame_nanos: Option<u64>,
}

impl NavigationRailItemAnimation {
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

    fn update(&mut self, frame_nanos: u64) {
        if let Some(start_frame_nanos) = self.last_change_frame_nanos {
            let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
            let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
            let fraction = if animation_nanos == 0 {
                1.0
            } else {
                (elapsed_nanos as f32 / animation_nanos as f32).min(1.0)
            };
            let target = if self.selected { 1.0 } else { 0.0 };
            let eased = animation::easing(fraction);
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

/// Expansion animation state for the navigation rail container.
#[derive(Clone, PartialEq, Default)]
struct NavigationRailExpansion {
    expanded: bool,
    start_frame_nanos: Option<u64>,
}

impl NavigationRailExpansion {
    fn set_expanded(&mut self, expanded: bool) {
        if self.expanded != expanded {
            self.expanded = expanded;
            self.start_frame_nanos = Some(current_frame_nanos());
        }
    }

    fn update(&mut self, frame_nanos: u64) {
        if let Some(start_frame_nanos) = self.start_frame_nanos {
            let elapsed_nanos = frame_nanos.saturating_sub(start_frame_nanos);
            let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
            if elapsed_nanos >= animation_nanos {
                self.start_frame_nanos = None;
            }
        }
    }

    fn expand_fraction(&self) -> f32 {
        let raw = match self.start_frame_nanos {
            None => 1.0,
            Some(start_frame_nanos) => {
                let elapsed_nanos = current_frame_nanos().saturating_sub(start_frame_nanos);
                let animation_nanos = ANIMATION_DURATION.as_nanos().min(u64::MAX as u128) as u64;
                if elapsed_nanos >= animation_nanos {
                    1.0
                } else {
                    elapsed_nanos as f32 / animation_nanos as f32
                }
            }
        };
        let eased = animation::easing(raw);
        if self.expanded { eased } else { 1.0 - eased }
    }

    fn is_animating(&self) -> bool {
        self.start_frame_nanos.is_some()
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

    use super::{navigation_rail, navigation_rail_item};

    #[tessera]
    fn navigation_rail_row() {
        let selected = remember(|| 0usize);
        column().children(move || {
            navigation_rail().content(move || {
                navigation_rail_item()
                    .label("Home")
                    .selected(selected.get() == 0)
                    .on_click(move || selected.set(0));
                navigation_rail_item()
                    .label("Search")
                    .selected(selected.get() == 1)
                    .on_click(move || selected.set(1));
                navigation_rail_item()
                    .label("Settings")
                    .selected(selected.get() == 2)
                    .on_click(move || selected.set(2));
            });
            if selected.get() == 1 {
                spacer().modifier(Modifier::new().width(Dp(10.0)).height(Dp(10.0)).semantics(
                    SemanticsArgs {
                        test_tag: Some("rail-search-active".to_string()),
                        ..Default::default()
                    },
                ));
            }
        });
    }

    #[tessera]
    fn navigation_rail_click_probe() {
        material_theme()
            .theme(MaterialTheme::default)
            .child(move || {
                lazy_column().item(move || {
                    navigation_rail_row();
                });
            });
    }

    #[test]
    fn clicking_middle_item_selects_middle_item() {
        let mut session = tessera_ui::testing::start_layout_test_session(
            tessera_ui::testing::layout_test(|| {
                navigation_rail_click_probe();
            })
            .viewport_px(200, 400),
        );
        let _ = tessera_ui::testing::pump_layout_test_session(&mut session, 0);

        // Collapsed rail items stack vertically at the 64dp minimum height with
        // 4dp spacing after a 44dp top padding, so the middle item center is
        // near y = 144.
        let now = tessera_ui::time::Instant::now();
        let cursor = Some(PxPosition::new(Px::new(48), Px::new(144)));

        // Move the pointer over the middle item first so it rebuilds in
        // isolation inside `lazy_column`; the per-item selection must still
        // resolve to the middle destination after the click.
        let moved = PointerChange::new(
            now,
            1,
            CursorEventContent::Moved(PxPosition::new(Px::new(48), Px::new(144))),
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

        snapshot.node("rail-search-active").exists();
    }
}
