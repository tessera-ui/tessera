use tessera_ui::{AxisConstraint, Px};

/// Removes symmetric padding from both bounds, clamping each at zero.
pub fn remove_padding_from_constraint(constraint: AxisConstraint, padding: Px) -> AxisConstraint {
    AxisConstraint::new(
        (constraint.min - padding * 2).max(Px::ZERO),
        constraint
            .max
            .map(|value| (value - padding * 2).max(Px::ZERO)),
    )
}
