//! An invisible component for creating empty space in a layout.
//!
//! ## Usage
//!
//! Use to add gaps between components or to create flexible, expanding regions.
use tessera_ui::{
    ComputedData, Constraint, LayoutPolicy, LayoutResult, MeasurementError, Modifier,
    layout::{MeasureScope, layout},
    tessera,
};

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
struct SpacerLayout;

impl LayoutPolicy for SpacerLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );
        Ok(LayoutResult::new(ComputedData::min_from_constraint(
            &constraint,
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
/// - `args` — props for this component; see [`SpacerArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::{modifier::ModifierExt as _, row::row, spacer::spacer, text::text};
/// use tessera_ui::Modifier;
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// row().children(|| {
///     text().content("Left");
///     spacer().modifier(Modifier::new().weight(1.0));
///     text().content("Right");
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn spacer(modifier: Modifier) {
    layout().modifier(modifier).layout_policy(SpacerLayout);
}
