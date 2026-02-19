//! An invisible component for creating empty space in a layout.
//!
//! ## Usage
//!
//! Use to add gaps between components or to create flexible, expanding regions.
use tessera_ui::{
    ComputedData, Constraint, LayoutInput, LayoutOutput, LayoutSpec, MeasurementError, Modifier,
    tessera,
};

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
struct SpacerLayout;

impl LayoutSpec for SpacerLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );
        Ok(ComputedData::min_from_constraint(&constraint))
    }
}

#[derive(Clone, PartialEq)]
/// Props for [`spacer`].
pub struct SpacerArgs {
    modifier: Modifier,
}

impl SpacerArgs {
    /// Creates spacer component props.
    pub fn new(modifier: Modifier) -> Self {
        Self { modifier }
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
/// use tessera_components::{
///     row::{RowArgs, row},
///     spacer::{SpacerArgs, spacer},
///     text::{TextArgs, text},
/// };
/// use tessera_ui::Modifier;
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// row(RowArgs::default(), |scope| {
///     scope.child(|| text(&TextArgs::default().text("Left")));
///     // Use weight to let the spacer expand and push the trailing content.
///     scope.child_weighted(|| spacer(&SpacerArgs::new(Modifier::new())), 1.0);
///     scope.child(|| text(&TextArgs::default().text("Right")));
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn spacer(args: &SpacerArgs) {
    let modifier = args.modifier.clone();
    modifier.run(|| {
        layout(SpacerLayout);
    });
}
