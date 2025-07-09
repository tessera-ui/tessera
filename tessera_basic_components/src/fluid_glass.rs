use derive_builder::Builder;
use std::any::Any;
use tessera::renderer::{DrawCommand, RenderRequirement};
use tessera::{
    ComponentNodeMetaData, ComputedData, Constraint, DimensionValue, Px, PxPosition, measure_node,
    place_node,
};
use tessera_macros::tessera;

#[derive(Builder, Clone)]
#[builder(build_fn(validate = "Self::validate"), pattern = "owned", setter(into))]
pub struct FluidGlassArgs {
    #[builder(default = "[1.0, 0.0, 0.0, 0.1]")]
    pub bleed_color: [f32; 4],
    #[builder(default = "[1.0, 1.0, 1.0, 0.5]")]
    pub highlight_color: [f32; 4],
    #[builder(default = "[0.0, 0.0, 0.0, 0.5]")]
    pub inner_shadow_color: [f32; 4],
    #[builder(default = "25.0")]
    pub corner_radius: f32,
    #[builder(default = "0.1")]
    pub blur_radius: f32,
    #[builder(default = "3.0")]
    pub g2_k_value: f32,
    #[builder(default = "25.0")]
    pub dispersion_height: f32,
    #[builder(default = "1.2")]
    pub chroma_multiplier: f32,
    #[builder(default = "24.0")]
    pub refraction_height: f32,
    #[builder(default = "32.0")]
    pub refraction_amount: f32,
    #[builder(default = "0.2")]
    pub eccentric_factor: f32,
    #[builder(default = "0.5")]
    pub bleed_amount: f32,
    #[builder(default = "0.4")]
    pub highlight_size: f32,
    #[builder(default = "2.0")]
    pub highlight_smoothing: f32,
    #[builder(default = "32.0")]
    pub inner_shadow_radius: f32,
    #[builder(default = "2.0")]
    pub inner_shadow_smoothing: f32,
    #[builder(default = "0.02")]
    pub noise_amount: f32,
    #[builder(default = "1.0")]
    pub noise_scale: f32,
    #[builder(default = "0.0")]
    pub time: f32,
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,
}

impl FluidGlassArgsBuilder {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

// Manual implementation of Default because derive_builder's default conflicts with our specific defaults
impl Default for FluidGlassArgs {
    fn default() -> Self {
        FluidGlassArgsBuilder::default().build().unwrap()
    }
}

#[derive(Clone)]
pub struct FluidGlassCommand {
    pub args: FluidGlassArgs,
}

impl DrawCommand for FluidGlassCommand {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn requirement(&self) -> RenderRequirement {
        RenderRequirement::SamplesBackground
    }
}

#[tessera]
pub fn fluid_glass(args: FluidGlassArgs) {
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

        let child_measurement = if !input.children_ids.is_empty() {
            let child_measurement = measure_node(
                input.children_ids[0],
                &effective_glass_constraint,
                input.tree,
                input.metadatas,
            )?;
            place_node(
                input.children_ids[0],
                PxPosition { x: Px(0), y: Px(0) },
                input.metadatas,
            );
            child_measurement
        } else {
            ComputedData {
                width: Px(0),
                height: Px(0),
            }
        };

        let drawable = FluidGlassCommand {
            args: args_measure_clone.clone(),
        };

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.basic_drawable = Some(Box::new(drawable));
        } else {
            input.metadatas.insert(
                input.current_node_id,
                ComponentNodeMetaData {
                    basic_drawable: Some(Box::new(drawable)),
                    ..Default::default()
                },
            );
        }

        let min_width = child_measurement.width;
        let min_height = child_measurement.height;
        let width = match effective_glass_constraint.width {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(min_width)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect("Seems that you are trying to fill an infinite width, which is not allowed")
                .max(min_height)
                .max(min.unwrap_or(Px(0))),
        };
        let height = match effective_glass_constraint.height {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(min_height)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect(
                    "Seems that you are trying to fill an infinite height, which is not allowed",
                )
                .max(min_height)
                .max(min.unwrap_or(Px(0))),
        };
        Ok(ComputedData { width, height })
    }));
}
