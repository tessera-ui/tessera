use derive_builder::Builder;
use tessera::renderer::DrawCommand;
use tessera::{ComputedData, Constraint, DimensionValue, Px, PxPosition, measure_node, place_node};
use tessera_macros::tessera;

use crate::pipelines::{blur::command::BlurCommand, contrast::ContrastCommand, mean::MeanCommand};

/// Arguments for the `fluid_glass` component, providing extensive control over its appearance.
///
/// This struct uses the builder pattern for easy construction.
#[derive(Builder, Clone)]
#[builder(build_fn(validate = "Self::validate"), pattern = "owned", setter(into))]
pub struct FluidGlassArgs {
    /// The tint color of the glass.
    /// The alpha channel uniquely and directly controls the tint strength.
    /// `A=0.0` means no tint (100% background visibility).
    /// `A=1.0` means full tint (100% color visibility).
    #[builder(default = "[0.5, 0.5, 0.5, 0.1]")]
    pub tint_color: [f32; 4],
    /// The color of the highlight along the top edge of the glass.
    /// Format is `[R, G, B, A]`. Defaults to a semi-transparent white.
    #[builder(default = "[1.0, 1.0, 1.0, 0.5]")]
    pub highlight_color: [f32; 4],
    /// The color of the inner shadow, which adds depth to the component.
    /// Format is `[R, G, B, A]`. Defaults to a semi-transparent black.
    #[builder(default = "[0.0, 0.0, 0.0, 0.5]")]
    pub inner_shadow_color: [f32; 4],
    /// The radius of the component's corners.
    #[builder(default = "25.0")]
    pub corner_radius: f32,
    /// The radius for the background blur effect. A value of `0.0` disables the blur.
    #[builder(default = "0.0")]
    pub blur_radius: f32,
    /// The G2 K-value, influencing the dispersion effect's shape.
    #[builder(default = "3.0")]
    pub g2_k_value: f32,
    /// The height of the chromatic dispersion effect.
    #[builder(default = "25.0")]
    pub dispersion_height: f32,
    /// Multiplier for the chromatic aberration, enhancing the color separation effect.
    #[builder(default = "1.2")]
    pub chroma_multiplier: f32,
    /// The height of the refraction effect, simulating light bending through the glass.
    #[builder(default = "24.0")]
    pub refraction_height: f32,
    /// The amount of refraction to apply.
    #[builder(default = "32.0")]
    pub refraction_amount: f32,
    /// Controls the shape and eccentricity of the highlight.
    #[builder(default = "0.2")]
    pub eccentric_factor: f32,
    /// The size of the highlight at the top of the component.
    #[builder(default = "0.4")]
    pub highlight_size: f32,
    /// The smoothness of the highlight's falloff.
    #[builder(default = "2.0")]
    pub highlight_smoothing: f32,
    /// The radius of the inner shadow.
    #[builder(default = "32.0")]
    pub inner_shadow_radius: f32,
    /// The smoothness of the inner shadow's falloff.
    #[builder(default = "2.0")]
    pub inner_shadow_smoothing: f32,
    /// The amount of noise to apply over the surface, adding texture.
    #[builder(default = "0.02")]
    pub noise_amount: f32,
    /// The scale of the noise pattern.
    #[builder(default = "1.0")]
    pub noise_scale: f32,
    /// A time value, typically used to animate the noise or other effects.
    #[builder(default = "0.0")]
    pub time: f32,
    /// The contrast adjustment factor.
    #[builder(default, setter(strip_option))]
    pub contrast: Option<f32>,
    /// The optional width of the component, defined as a `DimensionValue`.
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    /// The optional height of the component, defined as a `DimensionValue`.
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
    fn barrier(&self) -> Option<tessera::BarrierRequirement> {
        // Fluid glass aquires the scene texture, so it needs to sample the background
        Some(tessera::BarrierRequirement::SampleBackground)
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
                input.compute_resource_manager.clone(),
                input.gpu,
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

        if args.blur_radius > 0.0 {
            let blur_command = BlurCommand {
                radius: args.blur_radius,
                direction: (1.0, 0.0), // Horizontal
            };
            let blur_command2 = BlurCommand {
                radius: args.blur_radius,
                direction: (0.0, 1.0), // Vertical
            };
            if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
                metadata.push_compute_command(blur_command);
                metadata.push_compute_command(blur_command2);
            }
        }

        if let Some(contrast_value) = args.contrast {
            let mean_command =
                MeanCommand::new(input.gpu, &mut input.compute_resource_manager.write());
            let contrast_command =
                ContrastCommand::new(contrast_value, mean_command.result_buffer_ref());
            if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
                metadata.push_compute_command(mean_command);
                metadata.push_compute_command(contrast_command);
            }
        }

        let drawable = FluidGlassCommand {
            args: args_measure_clone.clone(),
        };

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.push_draw_command(drawable);
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
