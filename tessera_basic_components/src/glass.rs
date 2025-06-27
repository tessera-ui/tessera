use crate::pipelines::GlassCommand;
use derive_builder::Builder;
use tessera::{ComputedData, Constraint, DimensionValue, Px};
use tessera_macros::tessera;

#[derive(Builder, Clone, Default)]
#[builder(pattern = "owned")]
pub struct GlassArgs {
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,
}

#[tessera]
pub fn glass(args: GlassArgs) {
    let args_measure_clone = args.clone();

    measure(Box::new(move |input| {
        let glass_intrinsic_width = args_measure_clone.width.unwrap_or(DimensionValue::Wrap {
            min: None,
            max: None,
        });
        let glass_intrinsic_height = args_measure_clone.height.unwrap_or(DimensionValue::Wrap {
            min: None,
            max: None,
        });

        let glass_intrinsic_constraint =
            Constraint::new(glass_intrinsic_width, glass_intrinsic_height);

        let effective_glass_constraint = glass_intrinsic_constraint.merge(input.parent_constraint);

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.basic_drawable = Some(Box::new(GlassCommand));
        }

        let width = match effective_glass_constraint.width {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min.unwrap_or(Px(0)).min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect("Seems that you are trying to fill an infinite width, which is not allowed")
                .max(min.unwrap_or(Px(0))),
        };
        let height = match effective_glass_constraint.height {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min.unwrap_or(Px(0)).min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect(
                    "Seems that you are trying to fill an infinite height, which is not allowed",
                )
                .max(min.unwrap_or(Px(0))),
        };
        Ok(ComputedData { width, height })
    }));
}
