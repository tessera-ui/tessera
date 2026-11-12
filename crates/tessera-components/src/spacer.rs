//! An invisible component for creating empty space in a layout.
//!
//! ## Usage
//!
//! Use to add gaps between components or to create flexible, expanding regions.
use tessera_ui::{
    ComputedData, LayoutResult, MeasurementError, Modifier, layout::MeasureScope, tessera,
};

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
struct SpacerLayout;

impl tessera_ui::LayoutPolicy for SpacerLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData::min_from_constraint(
            input.parent_constraint().as_ref(),
        )))
    }
}

/// # spacer
///
/// Renders an empty, flexible space to influence layout.
///
/// ## Usage
///
/// Add fixed-size gaps or create flexible space that pushes other components
/// apart.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the spacer node.
///
/// ## Examples
///
/// ```
/// use tessera_components::{row::row, spacer::spacer, text::text};
/// use tessera_foundation::modifier::ModifierExt as _;
/// use tessera_ui::Modifier;
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// row().children(|| {
///     text("Left");
///     spacer().modifier(Modifier::new().weight(1.0));
///     text("Right");
/// });
/// # }
/// # tessera_ui::testing::layout_test(component).run();
/// ```
#[tessera]
pub fn spacer(modifier: Option<Modifier>) {
    tessera_ui::layout::layout(SpacerLayout).modifier(modifier.unwrap_or_default());
}
