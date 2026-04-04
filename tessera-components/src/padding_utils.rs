use tessera_ui::{AxisConstraint, Px};

/// Subtract symmetric padding from an optional `Px`, clamped at zero.
/// Extracted to module scope to reduce nested-function complexity.
fn sub_opt_px(value: Option<Px>, padding: Px) -> Option<Px> {
    value.map(|v| (v - padding * 2).max(Px(0)))
}

/// Removes symmetric padding from an axis constraint interval.
///
/// Subtracts twice the padding from the lower and upper bounds and ensures the
/// result never goes below zero. Useful to compute inner constraints for
/// padded components.
///
/// # Arguments
///
/// * `constraint` - The axis constraint to adjust.
/// * `padding` - The padding value applied on each side.
///
/// # Returns
///
/// A new interval with symmetric padding removed.
pub fn remove_padding_from_constraint(constraint: AxisConstraint, padding: Px) -> AxisConstraint {
    AxisConstraint::new(
        (constraint.min - padding * 2).max(Px::ZERO),
        sub_opt_px(constraint.max, padding),
    )
}
