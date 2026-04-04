//! Material Design badge primitives.
//!
//! ## Usage
//!
//! Highlight counts or status markers on top of icons and other UI elements.

use tessera_ui::{
    AxisConstraint, Color, ComputedData, Constraint, Dp, LayoutInput, LayoutOutput, LayoutPolicy,
    MeasurementError, Px, PxPosition, PxSize, RenderInput, RenderPolicy, RenderSlot,
    layout::layout_primitive, provide_context, tessera, use_context,
};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    pipelines::shape::command::ShapeCommand,
    row::row,
    shape_def::{ResolvedShape, Shape},
    theme::{ContentColor, MaterialTheme, content_color_for, provide_text_style},
};

fn resolve_dimension(axis: AxisConstraint, measure: Px) -> Px {
    axis.clamp(measure)
}

fn dimension_max(axis: AxisConstraint) -> Option<Px> {
    axis.resolve_max()
}

fn relax_min_constraint(axis: AxisConstraint) -> AxisConstraint {
    axis.without_min()
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
struct BadgedBoxLayout;

impl LayoutPolicy for BadgedBoxLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        debug_assert_eq!(
            input.children_ids().len(),
            2,
            "badged_box expects exactly two children: anchor and badge",
        );

        let parent_constraint = *input.parent_constraint().as_ref();

        let badge_constraint = Constraint::new(
            input.parent_constraint().width(),
            relax_min_constraint(input.parent_constraint().height()),
        );

        let anchor_id = input.children_ids()[0];
        let badge_id = input.children_ids()[1];

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

        output.place_child(anchor_id, PxPosition::new(Px(0), Px(0)));

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

        output.place_child(badge_id, PxPosition::new(badge_x, badge_y));

        Ok(ComputedData {
            width: anchor.width,
            height: anchor.height,
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
struct BadgeLayout {
    container_color: Color,
}

impl LayoutPolicy for BadgeLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let size_px = BadgeDefaults::SIZE.to_px();
        let intrinsic = Constraint::new(
            AxisConstraint::new(size_px, None),
            AxisConstraint::new(size_px, None),
        );
        let effective = Constraint::new(
            intrinsic.width.intersect(input.parent_constraint().width()),
            intrinsic
                .height
                .intersect(input.parent_constraint().height()),
        );

        let width = resolve_dimension(effective.width, size_px);
        let height = resolve_dimension(effective.height, size_px);

        Ok(ComputedData { width, height })
    }
}

impl RenderPolicy for BadgeLayout {
    fn record(&self, input: &RenderInput<'_>) {
        let mut metadata = input.metadata_mut();
        let size = metadata
            .computed_data()
            .expect("badge must have computed size before record");

        let ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } = BadgeDefaults::SHAPE.resolve_for_size(PxSize::new(size.width, size.height))
        else {
            unreachable!("badge shape must resolve to a rounded rectangle");
        };

        metadata
            .fragment_mut()
            .push_draw_command(ShapeCommand::Rect {
                color: self.container_color,
                corner_radii,
                corner_g2,
            });
    }
}

#[derive(Clone, Copy, PartialEq)]
struct BadgeWithContentLayout {
    container_color: Color,
    padding_px: Px,
}

impl LayoutPolicy for BadgeWithContentLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        debug_assert_eq!(
            input.children_ids().len(),
            1,
            "badge_with_content expects a single row child",
        );

        let min_size_px = BadgeDefaults::LARGE_SIZE.to_px();
        let intrinsic = Constraint::new(
            AxisConstraint::new(min_size_px, None),
            AxisConstraint::new(min_size_px, None),
        );
        let effective = Constraint::new(
            intrinsic.width.intersect(input.parent_constraint().width()),
            intrinsic
                .height
                .intersect(input.parent_constraint().height()),
        );

        let max_width =
            dimension_max(effective.width).map(|v| (v - self.padding_px * 2).max(Px(0)));
        let max_height = dimension_max(effective.height);

        let child_constraint = Constraint::new(
            AxisConstraint::new(Px::ZERO, max_width),
            AxisConstraint::new(Px::ZERO, max_height),
        );

        let row_id = input.children_ids()[0];
        let row_data = input.measure_child(row_id, &child_constraint)?;

        let measured_width = (row_data.width + self.padding_px * 2).max(min_size_px);
        let measured_height = row_data.height.max(min_size_px);

        let width = resolve_dimension(effective.width, measured_width);
        let height = resolve_dimension(effective.height, measured_height);

        let x = (width - row_data.width).max(Px(0)) / 2;
        let y = (height - row_data.height).max(Px(0)) / 2;
        output.place_child(row_id, PxPosition::new(x, y));

        Ok(ComputedData { width, height })
    }
}

impl RenderPolicy for BadgeWithContentLayout {
    fn record(&self, input: &RenderInput<'_>) {
        let mut metadata = input.metadata_mut();
        let size = metadata
            .computed_data()
            .expect("badge_with_content must have computed size before record");

        let ResolvedShape::Rounded {
            corner_radii,
            corner_g2,
        } = BadgeDefaults::SHAPE.resolve_for_size(PxSize::new(size.width, size.height))
        else {
            unreachable!("badge shape must resolve to a rounded rectangle");
        };

        metadata
            .fragment_mut()
            .push_draw_command(ShapeCommand::Rect {
                color: self.container_color,
                corner_radii,
                corner_g2,
            });
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
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme
            .error
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
/// - `badge` — badge slot rendered on top of content.
/// - `content` — anchor content slot.
///
/// ## Examples
///
/// ```rust
/// use tessera_components::badge::BadgeDefaults;
/// use tessera_ui::Dp;
/// assert_eq!(BadgeDefaults::OFFSET, Dp(6.0));
/// ```
#[tessera]
pub fn badged_box(badge: Option<RenderSlot>, content: Option<RenderSlot>) {
    layout_primitive()
        .layout_policy(BadgedBoxLayout)
        .child(move || {
            content.unwrap_or_else(RenderSlot::empty).render();
            badge.unwrap_or_else(RenderSlot::empty).render();
        });
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
/// - `container_color` — optional background color of the badge.
/// - `content_color` — optional preferred content color for descendants.
///
/// ## Examples
///
/// ```rust
/// use tessera_components::badge::badge;
/// use tessera_ui::Color;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// badge().content_color(Color::WHITE);
/// #     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn badge(container_color: Option<Color>, content_color: Option<Color>) {
    let _ = content_color;
    let container_color = container_color.unwrap_or_else(BadgeDefaults::container_color);
    let policy = BadgeLayout { container_color };
    layout_primitive()
        .layout_policy(policy)
        .render_policy(policy);
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
/// - `container_color` — optional background color of the badge.
/// - `content_color` — optional preferred content color for descendants.
/// - `content` — optional content slot rendered inside the badge.
///
/// ## Examples
///
/// ```rust
/// use tessera_components::badge::badge_with_content;
/// use tessera_ui::Color;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// badge_with_content()
///     .content_color(Color::WHITE)
///     .content(|| {});
/// #     });
/// # }
/// # component();
/// ```
#[tessera]
pub fn badge_with_content(
    container_color: Option<Color>,
    content_color: Option<Color>,
    content: Option<RenderSlot>,
) {
    let content = content.unwrap_or_else(RenderSlot::empty);
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

    let container_color = container_color.unwrap_or_else(BadgeDefaults::container_color);
    let content_color = content_color.unwrap_or_else(|| {
        content_color_for(container_color, &scheme).unwrap_or(
            use_context::<ContentColor>()
                .map(|c| c.get().current)
                .unwrap_or(ContentColor::default().current),
        )
    });

    let padding_px = BadgeDefaults::WITH_CONTENT_HORIZONTAL_PADDING.to_px();
    let policy = BadgeWithContentLayout {
        container_color,
        padding_px,
    };
    layout_primitive()
        .layout_policy(policy)
        .render_policy(policy)
        .child(move || {
            provide_context(
                || ContentColor {
                    current: content_color,
                },
                || {
                    provide_text_style(typography.label_small, move || {
                        row()
                            .main_axis_alignment(MainAxisAlignment::Center)
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children_shared(content);
                    });
                },
            );
        });
}
