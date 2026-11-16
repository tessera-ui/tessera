//! Thin separators between related content groups.
//!
//! ## Usage
//!
//! Distinguish sections of a form, toolbar, sidebar, or menu.

use tessera_ui::{
    Color, ComputedData, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode,
    LayoutPolicy, LayoutResult, MeasurementError, Modifier,
    layout::{MeasureScope, layout},
    modifier::ModifierCapabilityExt as _,
    tessera, use_context,
};

use crate::pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualLine};

use super::theme::{Appearance, HigTheme};

#[cfg(test)]
#[path = "separator_tests.rs"]
mod tests;

/// Direction in which a separator extends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeparatorOrientation {
    /// Extend across the available width.
    #[default]
    Horizontal,
    /// Extend along the available height.
    Vertical,
}

#[derive(Clone, Copy, PartialEq)]
struct SeparatorLayout(SeparatorOrientation);

impl LayoutPolicy for SeparatorLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = scope.parent_constraint();
        let (width, height) = match self.0 {
            SeparatorOrientation::Horizontal => (
                parent
                    .width()
                    .resolve_max()
                    .unwrap_or(parent.width().resolve_min()),
                parent.height().clamp(Dp(1.0).into()),
            ),
            SeparatorOrientation::Vertical => (
                parent.width().clamp(Dp(1.0).into()),
                parent
                    .height()
                    .resolve_max()
                    .unwrap_or(parent.height().resolve_min()),
            ),
        };
        Ok(LayoutResult::new(ComputedData { width, height }))
    }
}

#[derive(Clone, Copy, PartialEq)]
struct SeparatorDraw {
    orientation: SeparatorOrientation,
    color: Color,
}

impl DrawModifierNode for SeparatorDraw {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        let vertical = self.orientation == SeparatorOrientation::Vertical;
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                color: self.color,
                radii: [Dp::ZERO; 2],
                insets: [Dp::ZERO; 4],
                clip_radius: None,
                edge_smoothness: 1.0,
                stroke: None,
                line: Some(PerceptualLine {
                    vertical,
                    width: Dp(1.0),
                    offset: if vertical { Dp::ZERO } else { Dp(-1.0) },
                }),
            });
        content.draw(context.render_input);
    }
}

/// # separator
///
/// Visually separates groups of settings, menu items, or toolbar content.
///
/// ## Usage
///
/// Distinguish related sections without adding a focus target or announced control.
///
/// ## Parameters
///
/// - `orientation` — direction to extend; defaults to horizontal.
/// - `modifier` — sizing, placement, clipping, and optional authored semantics.
///
/// ## Examples
///
/// ```
/// use tessera_components::{column::column, experiment::hig::separator::separator};
/// use tessera_foundation::modifier::ModifierExt as _;
/// use tessera_ui::{Dp, Modifier, tessera, testing::UiTest};
/// #[tessera]
/// fn app() {
///     column().children(|| {
///         separator().modifier(Modifier::new().width(Dp(120.0)));
///     });
/// }
/// let mut ui = UiTest::new().content(app);
/// let frame = ui.frame();
/// assert_eq!(frame.node("separator").bounds().width.0, 120);
/// assert_eq!(frame.node("separator").bounds().height.0, 1);
/// ```
#[tessera]
pub fn separator(orientation: Option<SeparatorOrientation>, modifier: Option<Modifier>) {
    let orientation = orientation.unwrap_or_default();
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let color = if dark {
        Color::WHITE.with_alpha(0.12)
    } else {
        Color::BLACK.with_alpha(0.10)
    };
    layout(SeparatorLayout(orientation)).modifier(
        modifier
            .unwrap_or_default()
            .push_draw(SeparatorDraw { orientation, color }),
    );
}
