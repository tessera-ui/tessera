//! Material Design divider primitives.
//!
//! ## Usage
//!
//! Separate sections in lists, menus, and settings screens.
use tessera_ui::{
    Color, ComputedData, Dp, LayoutResult, MeasurementError, Modifier, Px,
    layout::{MeasureScope, layout},
    tessera, use_context,
};

use crate::{pipelines::simple_rect::command::SimpleRectCommand, theme::MaterialTheme};

fn resolve_thickness_px(thickness: Dp) -> Px {
    if thickness == Dp::ZERO {
        Px(1)
    } else {
        thickness.to_px().max(Px::ZERO)
    }
}

/// Default values for divider components.
pub struct DividerDefaults;

impl DividerDefaults {
    /// Default divider thickness.
    pub const THICKNESS: Dp = Dp(1.0);

    /// Default divider color.
    pub fn color() -> Color {
        Color::new(0.0, 0.0, 0.0, 0.0)
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

impl tessera_ui::LayoutModifierNode for DividerLayout {
    fn measure_eq(&self, other: &Self) -> bool {
        self.thickness == other.thickness && self.orientation == other.orientation
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.measure_eq(other)
    }

    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let (width, height) = match self.orientation {
            DividerOrientation::Horizontal => (
                input
                    .parent_constraint()
                    .width()
                    .resolve_max()
                    .unwrap_or(input.parent_constraint().width().resolve_min()),
                input.parent_constraint().height().clamp(self.thickness),
            ),
            DividerOrientation::Vertical => (
                input.parent_constraint().width().clamp(self.thickness),
                input
                    .parent_constraint()
                    .height()
                    .resolve_max()
                    .unwrap_or(input.parent_constraint().height().resolve_min()),
            ),
        };

        Ok(LayoutResult::new(ComputedData { width, height }))
    }
}

impl tessera_ui::DrawModifierNode for DividerLayout {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let input = &mut *ctx.render_input;
        input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(SimpleRectCommand { color: self.color });
        content.draw(input);
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
/// - `modifier` — sizing, insets, placement, and semantics.
/// - `thickness` — optional line thickness; zero draws one physical pixel, and
///   negative values resolve to zero.
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
pub fn horizontal_divider(modifier: Option<Modifier>, thickness: Option<Dp>, color: Option<Color>) {
    let thickness_px = resolve_thickness_px(thickness.unwrap_or(DividerDefaults::THICKNESS));
    let color = if let Some(color) = color {
        color
    } else {
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme
            .outline_variant
    };

    let policy = DividerLayout {
        thickness: thickness_px,
        color,
        orientation: DividerOrientation::Horizontal,
    };
    layout()
        .modifier(modifier.unwrap_or_default())
        .policy(policy)
        .draw(policy);
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
/// - `modifier` — sizing, insets, placement, and semantics.
/// - `thickness` — optional line thickness; zero draws one physical pixel, and
///   negative values resolve to zero.
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
pub fn vertical_divider(modifier: Option<Modifier>, thickness: Option<Dp>, color: Option<Color>) {
    let thickness_px = resolve_thickness_px(thickness.unwrap_or(DividerDefaults::THICKNESS));
    let color = if let Some(color) = color {
        color
    } else {
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme
            .outline_variant
    };

    let policy = DividerLayout {
        thickness: thickness_px,
        color,
        orientation: DividerOrientation::Vertical,
    };
    layout()
        .modifier(modifier.unwrap_or_default())
        .policy(policy)
        .draw(policy);
}

#[cfg(test)]
mod tests;

#[cfg(all(test, not(target_family = "wasm")))]
mod drawing_tests;
