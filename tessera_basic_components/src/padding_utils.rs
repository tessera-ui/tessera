use tessera::{DimensionValue, Px};

pub fn remove_padding_from_dimension(dimension: DimensionValue, padding: Px) -> DimensionValue {
    match dimension {
        DimensionValue::Fixed(value) => DimensionValue::Fixed((value - padding * 2).max(Px(0))),
        DimensionValue::Wrap { min, max } => DimensionValue::Wrap {
            min: min.map(|m| (m - padding * 2).max(Px(0))),
            max: max.map(|m| (m - padding * 2).max(Px(0))),
        },
        DimensionValue::Fill { min, max } => DimensionValue::Fill {
            min,
            max: max.map(|m| (m - padding * 2).max(Px(0))),
        },
    }
}
