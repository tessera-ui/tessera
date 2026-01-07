//! Material Design divider primitives.
//!
//! ## Usage
//!
//! Separate sections in lists, menus, and settings screens.
use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, Constraint, DimensionValue, Dp, LayoutInput, LayoutOutput, LayoutSpec,
    MeasurementError, Px, RenderInput, tessera, use_context,
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

/// Arguments for [`horizontal_divider`] and [`vertical_divider`].
#[derive(Clone, Debug, Setters)]
pub struct DividerArgs {
    /// Thickness of the divider line.
    ///
    /// Use `Dp::ZERO` to request a single physical pixel thickness.
    pub thickness: Dp,
    /// Color of the divider line.
    pub color: Color,
}

impl Default for DividerArgs {
    fn default() -> Self {
        Self {
            thickness: DividerDefaults::THICKNESS,
            color: DividerDefaults::color(),
        }
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

impl LayoutSpec for DividerLayout {
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

    fn record(&self, input: &RenderInput<'_>) {
        input
            .metadata_mut()
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
/// - `args` — configures divider thickness and color; see [`DividerArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::divider::DividerArgs;
/// use tessera_ui::{Color, Dp};
///
/// let args = DividerArgs {
///     thickness: Dp::ZERO,
///     color: Color::BLACK,
/// };
/// assert_eq!(args.thickness, Dp::ZERO);
/// ```
#[tessera]
pub fn horizontal_divider(args: impl Into<DividerArgs>) {
    let args: DividerArgs = args.into();
    let thickness_px = resolve_thickness_px(args.thickness);
    let color = args.color;

    layout(DividerLayout {
        thickness: thickness_px,
        color,
        orientation: DividerOrientation::Horizontal,
    });
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
/// - `args` — configures divider thickness and color; see [`DividerArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::divider::DividerArgs;
/// use tessera_ui::{Color, Dp};
///
/// let args = DividerArgs {
///     thickness: Dp(2.0),
///     color: Color::BLACK,
/// };
/// assert_eq!(args.thickness, Dp(2.0));
/// ```
#[tessera]
pub fn vertical_divider(args: impl Into<DividerArgs>) {
    let args: DividerArgs = args.into();
    let thickness_px = resolve_thickness_px(args.thickness);
    let color = args.color;

    layout(DividerLayout {
        thickness: thickness_px,
        color,
        orientation: DividerOrientation::Vertical,
    });
}
