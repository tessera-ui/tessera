//! Material Design badge primitives.
//!
//! ## Usage
//!
//! Highlight counts or status markers on top of icons and other UI elements.

use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Px, PxPosition, PxSize,
    provide_context, tessera, use_context,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    pipelines::shape::command::ShapeCommand,
    row::{RowArgs, RowScope, row},
    shape_def::{ResolvedShape, Shape},
    theme::{ContentColor, MaterialTheme, content_color_for, provide_text_style},
};

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

fn dimension_max(dim: DimensionValue) -> Option<Px> {
    match dim {
        DimensionValue::Fixed(v) => Some(v),
        DimensionValue::Wrap { max, .. } | DimensionValue::Fill { max, .. } => max,
    }
}

fn relax_min_constraint(dim: DimensionValue) -> DimensionValue {
    match dim {
        DimensionValue::Fixed(v) => DimensionValue::Wrap {
            min: Some(Px(0)),
            max: Some(v),
        },
        DimensionValue::Wrap { max, .. } => DimensionValue::Wrap {
            min: Some(Px(0)),
            max,
        },
        DimensionValue::Fill { max, .. } => DimensionValue::Fill {
            min: Some(Px(0)),
            max,
        },
    }
}

/// Default values for [`badge`], [`badge_with_content`], and [`badged_box`].
pub struct BadgeDefaults;

impl BadgeDefaults {
    /// Default badge size when it has no content.
    pub const SIZE: Dp = Dp(6.0);
    /// Default badge size when it has content.
    pub const LARGE_SIZE: Dp = Dp(16.0);

    /// Default badge shape.
    pub const SHAPE: Shape = Shape::capsule();

    /// Horizontal padding for badges with content.
    pub const WITH_CONTENT_HORIZONTAL_PADDING: Dp = Dp(4.0);

    /// Horizontal offset for badges with content relative to the anchor.
    pub const WITH_CONTENT_HORIZONTAL_OFFSET: Dp = Dp(12.0);
    /// Vertical offset for badges with content relative to the anchor.
    pub const WITH_CONTENT_VERTICAL_OFFSET: Dp = Dp(14.0);

    /// Offset for badges without content relative to the anchor.
    pub const OFFSET: Dp = Dp(6.0);

    /// Default container color for a badge.
    pub fn container_color() -> Color {
        use_context::<MaterialTheme>().get().color_scheme.error
    }
}

/// Arguments for [`badge`] and [`badge_with_content`].
#[derive(Clone, Debug, Setters)]
pub struct BadgeArgs {
    /// Background color of the badge.
    pub container_color: Color,
    /// Preferred content color for badge descendants.
    ///
    /// When `None`, the badge derives a matching content color from the theme.
    #[setters(strip_option)]
    pub content_color: Option<Color>,
}

impl Default for BadgeArgs {
    fn default() -> Self {
        Self {
            container_color: BadgeDefaults::container_color(),
            content_color: None,
        }
    }
}

/// # badged_box
///
/// Positions a badge relative to an anchor element.
///
/// ## Usage
///
/// Display counts or status indicators on top of icons in navigation or
/// toolbars.
///
/// ## Parameters
///
/// - `badge` — draws the badge content, typically [`badge`] or
///   [`badge_with_content`].
/// - `content` — draws the anchor the badge should be positioned against.
///
/// ## Examples
///
/// ```
/// use tessera_ui::Dp;
/// use tessera_ui_basic_components::badge::BadgeDefaults;
/// assert_eq!(BadgeDefaults::OFFSET, Dp(6.0));
/// ```
#[tessera]
pub fn badged_box<F1, F2>(badge: F1, content: F2)
where
    F1: FnOnce() + Send + Sync + 'static,
    F2: FnOnce() + Send + Sync + 'static,
{
    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            debug_assert_eq!(
                input.children_ids.len(),
                2,
                "badged_box expects exactly two children: anchor and badge",
            );

            let parent_constraint = Constraint::new(
                input.parent_constraint.width(),
                input.parent_constraint.height(),
            );

            let badge_constraint = Constraint::new(
                input.parent_constraint.width(),
                relax_min_constraint(input.parent_constraint.height()),
            );

            let anchor_id = input.children_ids[0];
            let badge_id = input.children_ids[1];

            let to_measure = vec![(badge_id, badge_constraint), (anchor_id, parent_constraint)];

            let results = input.measure_children(to_measure)?;
            let anchor = results
                .get(&anchor_id)
                .copied()
                .expect("badged_box anchor must be measured");
            let badge_data = results
                .get(&badge_id)
                .copied()
                .expect("badged_box badge must be measured");

            input.place_child(anchor_id, PxPosition::new(Px(0), Px(0)));

            let badge_size_px = BadgeDefaults::SIZE.to_px();
            let has_content = badge_data.width > badge_size_px;

            let horizontal_offset = if has_content {
                BadgeDefaults::WITH_CONTENT_HORIZONTAL_OFFSET
            } else {
                BadgeDefaults::OFFSET
            }
            .to_px();

            let vertical_offset = if has_content {
                BadgeDefaults::WITH_CONTENT_VERTICAL_OFFSET
            } else {
                BadgeDefaults::OFFSET
            }
            .to_px();

            let badge_x = anchor.width - horizontal_offset;
            let badge_y = -badge_data.height + vertical_offset;

            input.place_child(badge_id, PxPosition::new(badge_x, badge_y));

            Ok(ComputedData {
                width: anchor.width,
                height: anchor.height,
            })
        },
    ));

    content();
    badge();
}

/// # badge
///
/// Renders an icon-only badge.
///
/// ## Usage
///
/// Mark an icon as having new activity without showing a numeric count.
///
/// ## Parameters
///
/// - `args` — configures badge colors; see [`BadgeArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_ui::Color;
/// use tessera_ui_basic_components::badge::BadgeArgs;
///
/// let args = BadgeArgs::default()
///     .container_color(Color::RED)
///     .content_color(Color::WHITE);
/// assert_eq!(args.container_color, Color::RED);
/// ```
#[tessera]
pub fn badge(args: impl Into<BadgeArgs>) {
    let args: BadgeArgs = args.into();
    let container_color = args.container_color;

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            let size_px = BadgeDefaults::SIZE.to_px();
            let intrinsic = Constraint::new(
                DimensionValue::Wrap {
                    min: Some(size_px),
                    max: None,
                },
                DimensionValue::Wrap {
                    min: Some(size_px),
                    max: None,
                },
            );
            let effective = intrinsic.merge(input.parent_constraint);

            let width = resolve_dimension(effective.width, size_px);
            let height = resolve_dimension(effective.height, size_px);

            let ResolvedShape::Rounded {
                corner_radii,
                corner_g2,
            } = BadgeDefaults::SHAPE.resolve_for_size(PxSize::new(width, height))
            else {
                unreachable!("badge shape must resolve to a rounded rectangle");
            };

            input.metadata_mut().push_draw_command(ShapeCommand::Rect {
                color: container_color,
                corner_radii,
                corner_g2,
                shadow: None,
            });

            Ok(ComputedData { width, height })
        },
    ));
}

/// # badge_with_content
///
/// Renders a badge that contains short content, such as a number.
///
/// ## Usage
///
/// Display compact counts (for example, unread messages) on top of navigation
/// icons.
///
/// ## Parameters
///
/// - `args` — configures badge colors; see [`BadgeArgs`].
/// - `content` — adds children inside the badge using a [`RowScope`].
///
/// ## Examples
///
/// ```
/// use tessera_ui::Color;
/// use tessera_ui_basic_components::badge::BadgeArgs;
///
/// let args = BadgeArgs::default()
///     .container_color(Color::RED)
///     .content_color(Color::WHITE);
/// assert_eq!(args.content_color, Some(Color::WHITE));
/// ```
#[tessera]
pub fn badge_with_content<F>(args: impl Into<BadgeArgs>, content: F)
where
    F: FnOnce(&mut RowScope),
{
    let args: BadgeArgs = args.into();
    let theme = use_context::<MaterialTheme>().get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

    let container_color = args.container_color;
    let content_color = args.content_color.unwrap_or_else(|| {
        content_color_for(container_color, &scheme)
            .unwrap_or(use_context::<ContentColor>().get().current)
    });

    let padding_px = BadgeDefaults::WITH_CONTENT_HORIZONTAL_PADDING.to_px();

    measure(Box::new(
        move |input| -> Result<ComputedData, MeasurementError> {
            debug_assert_eq!(
                input.children_ids.len(),
                1,
                "badge_with_content expects a single row child",
            );

            let min_size_px = BadgeDefaults::LARGE_SIZE.to_px();
            let intrinsic = Constraint::new(
                DimensionValue::Wrap {
                    min: Some(min_size_px),
                    max: None,
                },
                DimensionValue::Wrap {
                    min: Some(min_size_px),
                    max: None,
                },
            );
            let effective = intrinsic.merge(input.parent_constraint);

            let max_width = dimension_max(effective.width).map(|v| (v - padding_px * 2).max(Px(0)));
            let max_height = dimension_max(effective.height);

            let child_constraint = Constraint::new(
                DimensionValue::Wrap {
                    min: None,
                    max: max_width,
                },
                DimensionValue::Wrap {
                    min: None,
                    max: max_height,
                },
            );

            let row_id = input.children_ids[0];
            let row_data = input.measure_child(row_id, &child_constraint)?;

            let measured_width = (row_data.width + padding_px * 2).max(min_size_px);
            let measured_height = row_data.height.max(min_size_px);

            let width = resolve_dimension(effective.width, measured_width);
            let height = resolve_dimension(effective.height, measured_height);

            let ResolvedShape::Rounded {
                corner_radii,
                corner_g2,
            } = BadgeDefaults::SHAPE.resolve_for_size(PxSize::new(width, height))
            else {
                unreachable!("badge shape must resolve to a rounded rectangle");
            };

            input.metadata_mut().push_draw_command(ShapeCommand::Rect {
                color: container_color,
                corner_radii,
                corner_g2,
                shadow: None,
            });

            let x = (width - row_data.width).max(Px(0)) / 2;
            let y = (height - row_data.height).max(Px(0)) / 2;
            input.place_child(row_id, PxPosition::new(x, y));

            Ok(ComputedData { width, height })
        },
    ));

    provide_context(
        ContentColor {
            current: content_color,
        },
        || {
            provide_text_style(typography.label_small, || {
                row(
                    RowArgs::default()
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .cross_axis_alignment(CrossAxisAlignment::Center),
                    content,
                );
            });
        },
    );
}
