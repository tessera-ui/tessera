//! Material Design divider primitives.
//!
//! ## Usage
//!
//! Separate sections in lists, menus, and settings screens.
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, LayoutInput, LayoutOutput, LayoutPolicy,
    MeasurementError, Px, RenderInput, RenderPolicy, layout::layout_primitive, tessera,
    use_context,
};

use crate::{pipelines::simple_rect::command::SimpleRectCommand, theme::MaterialTheme};

fn resolve_thickness_px(thickness: Dp) -> Px {
    if thickness == Dp::ZERO {
        Px(1)
    } else {
        thickness.to_px()
    }
}

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
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let intrinsic = match self.orientation {
            DividerOrientation::Horizontal => Constraint::new(
                DimensionValue::FILLED,
                DimensionValue::Fixed(self.thickness),
            ),
            DividerOrientation::Vertical => Constraint::new(
                DimensionValue::Fixed(self.thickness),
                DimensionValue::FILLED,
            ),
        };
        let effective = intrinsic.merge(input.parent_constraint());

        let (width, height) = match self.orientation {
            DividerOrientation::Horizontal => (
                resolve_dimension(effective.width, Px(0)),
                resolve_dimension(effective.height, self.thickness),
            ),
            DividerOrientation::Vertical => (
                resolve_dimension(effective.width, self.thickness),
                resolve_dimension(effective.height, Px(0)),
            ),
        };

        Ok(ComputedData { width, height })
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
    layout_primitive()
        .layout_policy(policy)
        .render_policy(policy);
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
    layout_primitive()
        .layout_policy(policy)
        .render_policy(policy);
}
