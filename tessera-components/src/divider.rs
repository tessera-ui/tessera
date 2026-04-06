//! Material Design divider primitives.
//!
//! ## Usage
//!
//! Separate sections in lists, menus, and settings screens.
use tessera_ui::{
    AxisConstraint, Color, ComputedData, Dp, LayoutPolicy, LayoutResult, MeasurementError, Px,
    RenderInput, RenderPolicy,
    layout::{MeasureScope, layout},
    tessera, use_context,
};

use crate::{pipelines::simple_rect::command::SimpleRectCommand, theme::MaterialTheme};

fn resolve_thickness_px(thickness: Dp) -> Px {
    if thickness == Dp::ZERO {
        Px(1)
    } else {
        thickness.to_px()
    }
}

fn resolve_dimension(axis: AxisConstraint, measure: Px) -> Px {
    axis.clamp(measure)
}

/// Default values for divider components.
pub struct DividerDefaults;

impl DividerDefaults {
    /// Default divider thickness.
    pub const THICKNESS: Dp = Dp(1.0);

    /// Default divider color.
    pub fn color() -> Color {
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme
            .outline_variant
    }
}

#[derive(Clone, Copy, PartialEq)]
enum DividerOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq)]
struct DividerLayout {
    thickness: Px,
    color: Color,
    orientation: DividerOrientation,
}

impl LayoutPolicy for DividerLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let (width, height) = match self.orientation {
            DividerOrientation::Horizontal => (
                input
                    .parent_constraint()
                    .width()
                    .resolve_max()
                    .expect("horizontal_divider requires a bounded width"),
                resolve_dimension(
                    AxisConstraint::exact(self.thickness)
                        .intersect(input.parent_constraint().height()),
                    self.thickness,
                ),
            ),
            DividerOrientation::Vertical => (
                resolve_dimension(
                    AxisConstraint::exact(self.thickness)
                        .intersect(input.parent_constraint().width()),
                    self.thickness,
                ),
                input
                    .parent_constraint()
                    .height()
                    .resolve_max()
                    .expect("vertical_divider requires a bounded height"),
            ),
        };

        Ok(LayoutResult::new(ComputedData { width, height }))
    }
}

impl RenderPolicy for DividerLayout {
    fn record(&self, input: &RenderInput<'_>) {
        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(SimpleRectCommand { color: self.color });
    }
}

/// # horizontal_divider
///
/// Renders a horizontal divider line that fills the available width.
///
/// ## Usage
///
/// Separate list groups or content sections within a screen.
///
/// ## Parameters
///
/// - `thickness` — optional line thickness in density-independent pixels.
/// - `color` — optional line color override.
///
/// ## Examples
///
/// ```
/// use tessera_components::divider::horizontal_divider;
/// use tessera_ui::{Color, Dp};
///
/// horizontal_divider().thickness(Dp::ZERO).color(Color::BLACK);
/// ```
#[tessera]
pub fn horizontal_divider(thickness: Option<Dp>, color: Option<Color>) {
    let thickness_px = resolve_thickness_px(thickness.unwrap_or(DividerDefaults::THICKNESS));
    let color = color.unwrap_or_else(DividerDefaults::color);

    let policy = DividerLayout {
        thickness: thickness_px,
        color,
        orientation: DividerOrientation::Horizontal,
    };
    layout().layout_policy(policy).render_policy(policy);
}

/// # vertical_divider
///
/// Renders a vertical divider line that fills the available height.
///
/// ## Usage
///
/// Separate side-by-side content regions.
///
/// ## Parameters
///
/// - `thickness` — optional line thickness in density-independent pixels.
/// - `color` — optional line color override.
///
/// ## Examples
///
/// ```
/// use tessera_components::divider::vertical_divider;
/// use tessera_ui::{Color, Dp};
///
/// vertical_divider().thickness(Dp(2.0)).color(Color::BLACK);
/// ```
#[tessera]
pub fn vertical_divider(thickness: Option<Dp>, color: Option<Color>) {
    let thickness_px = resolve_thickness_px(thickness.unwrap_or(DividerDefaults::THICKNESS));
    let color = color.unwrap_or_else(DividerDefaults::color);

    let policy = DividerLayout {
        thickness: thickness_px,
        color,
        orientation: DividerOrientation::Vertical,
    };
    layout().layout_policy(policy).render_policy(policy);
}
