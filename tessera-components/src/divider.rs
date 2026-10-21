//! Material Design divider primitives.
//!
//! ## Usage
//!
//! Separate sections in lists, menus, and settings screens.
use std::sync::OnceLock;

use tessera_ui::{
    Color, ComputedData, Dp, LayoutResult, MeasurementError, Modifier, Px,
    layout::{MeasureScope, layout},
    tessera, use_context,
};

use crate::{
    pipelines::divider::DividerCommand,
    theme::{MaterialColorScheme, MaterialTheme},
};

fn resolve_thickness_px(thickness: Dp) -> f32 {
    let thickness = if thickness.0.is_finite() {
        thickness
    } else {
        DividerDefaults::THICKNESS
    };
    if thickness == Dp::ZERO {
        1.0
    } else {
        thickness.to_pixels_f32().max(0.0)
    }
}

/// Default values for divider components.
pub struct DividerDefaults;

impl DividerDefaults {
    /// Default divider thickness.
    pub const THICKNESS: Dp = Dp(1.0);

    /// Divider color from the default light palette.
    ///
    /// Omit the component color to inherit the active Material theme.
    pub fn color() -> Color {
        static COLOR: OnceLock<Color> = OnceLock::new();
        *COLOR.get_or_init(|| MaterialColorScheme::default().outline_variant)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum DividerOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq)]
struct DividerLayout {
    thickness: f32,
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
                input
                    .parent_constraint()
                    .height()
                    .clamp(Px(self.thickness.round() as i32)),
            ),
            DividerOrientation::Vertical => (
                input
                    .parent_constraint()
                    .width()
                    .clamp(Px(self.thickness.round() as i32)),
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
        let size = input
            .metadata_mut()
            .computed_data()
            .expect("divider must be measured");
        let vertical = self.orientation == DividerOrientation::Vertical;
        let cross = if vertical { size.width } else { size.height };
        let stroke = if cross < Px(self.thickness.round() as i32) {
            self.thickness.min(cross.to_f32())
        } else {
            self.thickness
        };
        if size.width > Px::ZERO && size.height > Px::ZERO && stroke > 0.0 {
            input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(DividerCommand {
                    color: self.color,
                    stroke,
                    vertical,
                });
        }
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
///   negative values resolve to zero. Non-finite values use the default.
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
///   negative values resolve to zero. Non-finite values use the default.
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
