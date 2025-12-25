//! An invisible component for creating empty space in a layout.
//!
//! ## Usage
//!
//! Use to add gaps between components or to create flexible, expanding regions.
use tessera_ui::{ComputedData, Constraint, MeasurementError, Modifier, tessera};

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
/// - `modifier` — configures sizing and layout constraints for this spacer.
///
/// ## Examples
///
/// ```
/// use tessera_ui::Modifier;
/// use tessera_ui_basic_components::{
///     row::{RowArgs, row},
///     spacer::spacer,
///     text::{TextArgs, text},
/// };
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// row(RowArgs::default(), |scope| {
///     scope.child(|| text(TextArgs::default().text("Left")));
///     // Use weight to let the spacer expand and push the trailing content.
///     scope.child_weighted(|| spacer(Modifier::new()), 1.0);
///     scope.child(|| text(TextArgs::default().text("Right")));
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn spacer(modifier: Modifier) {
    modifier.run(spacer_inner);
}

#[tessera]
fn spacer_inner() {
    measure(Box::new(
        |input| -> Result<ComputedData, MeasurementError> {
            let constraint = Constraint::new(
                input.parent_constraint.width(),
                input.parent_constraint.height(),
            );
            Ok(ComputedData::min_from_constraint(&constraint))
        },
    ));
}
