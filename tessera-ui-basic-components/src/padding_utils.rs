use tessera_ui::{DimensionValue, Px};

/// Subtract symmetric padding from an optional `Px`, clamped at zero.
/// Extracted to module scope to reduce nested-function complexity.
fn sub_opt_px(value: Option<Px>, padding: Px) -> Option<Px> {
    value.map(|v| (v - padding * 2).max(Px(0)))
}

/// Removes symmetric padding from a `DimensionValue`.
///
/// Subtracts twice the padding from fixed/min/max values and ensures the result
/// never goes below zero. Useful to compute inner dimensions for padded components.
///
/// # Arguments
///
/// * `dimension` - The `DimensionValue` to adjust.
/// * `padding` - The padding value applied on each side.
///
/// # Returns
///
/// A new `DimensionValue` with symmetric padding removed.
pub fn remove_padding_from_dimension(dimension: DimensionValue, padding: Px) -> DimensionValue {
    match dimension {
        DimensionValue::Fixed(value) => DimensionValue::Fixed((value - padding * 2).max(Px(0))),
        DimensionValue::Wrap { min, max } => DimensionValue::Wrap {
            min: sub_opt_px(min, padding),
            max: sub_opt_px(max, padding),
        },
        DimensionValue::Fill { min, max } => DimensionValue::Fill {
            min,
            max: sub_opt_px(max, padding),
        },
    }
}
