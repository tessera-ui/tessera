//! Fluid glass effect module for Tessera UI Basic Components.
//!
//! This module provides the core implementation for the "fluid glass" (frosted/distorted glass) visual effect,
//! including parameter structures, builder patterns, and the main `fluid_glass` component.
//! It enables highly customizable backgrounds with blur, tint, chromatic dispersion, noise, and ripple effects,
//! suitable for creating modern, layered user interfaces with enhanced depth and focus.
//! Typical use cases include backgrounds for buttons, sliders, switches, and other interactive UI elements
//! where a dynamic, visually appealing glass-like surface is desired.

use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp, PressKeyEventType, Px,
    PxPosition, renderer::DrawCommand, winit::window::CursorIcon,
};
use tessera_ui_macros::tessera;

use crate::{
    padding_utils::remove_padding_from_dimension,
    pipelines::{blur::command::BlurCommand, contrast::ContrastCommand, mean::MeanCommand},
    pos_misc::is_position_in_component,
    ripple_state::RippleState,
    shape_def::Shape,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct GlassBorder {
    pub width: Px,
}

impl GlassBorder {
    pub fn new(width: Px) -> Self {
        Self { width }
    }
}

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
    #[builder(default = "Color::TRANSPARENT")]
    pub tint_color: Color,
    /// The shape of the component, an enum that can be `RoundedRectangle` or `Ellipse`.
    #[builder(default = "Shape::RoundedRectangle { corner_radius: 25.0, g2_k_value: 3.0 }")]
    pub shape: Shape,
    /// The radius for the background blur effect. A value of `0.0` disables the blur.
    #[builder(default = "0.0")]
    pub blur_radius: f32,
    /// The height of the chromatic dispersion effect.
    #[builder(default = "25.0")]
    pub dispersion_height: f32,
    /// Multiplier for the chromatic aberration, enhancing the color separation effect.
    #[builder(default = "1.0")]
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

    #[builder(default = "Dp(0.0)")]
    pub padding: Dp,

    // Ripple effect properties
    #[builder(default, setter(strip_option))]
    pub ripple_center: Option<[f32; 2]>,
    #[builder(default, setter(strip_option))]
    pub ripple_radius: Option<f32>,
    #[builder(default, setter(strip_option))]
    pub ripple_alpha: Option<f32>,
    #[builder(default, setter(strip_option))]
    pub ripple_strength: Option<f32>,

    #[builder(default, setter(strip_option, into = false))]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,

    #[builder(default = "Some(GlassBorder { width: Dp(1.0).into() })")]
    pub border: Option<GlassBorder>,

    /// Whether to block input events on the glass surface.
    /// When `true`, the surface will consume all input events, preventing interaction with underlying components.
    #[builder(default = "false")]
    pub block_input: bool,
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
    fn barrier(&self) -> Option<tessera_ui::BarrierRequirement> {
        // Fluid glass aquires the scene texture, so it needs to sample the background
        Some(tessera_ui::BarrierRequirement::SampleBackground)
    }
}

#[tessera]
/// Creates a fluid glass effect component, which serves as a dynamic and visually appealing background.
///
/// The `fluid_glass` component simulates the look of frosted or distorted glass with a fluid,
/// animated texture. It can be used to create modern, layered user interfaces where the background
/// content is blurred and stylized, enhancing depth and focus. The effect is highly customizable
/// through `FluidGlassArgs`.
///
/// # Example
///
/// ```
/// use tessera_ui_basic_components::{
///     fluid_glass::{fluid_glass, FluidGlassArgs},
///     text::text,
/// };
///
/// fluid_glass(FluidGlassArgs::default(), None, || {
///     text("Content on glass".to_string());
/// });
/// ```
///
/// # Arguments
///
/// * `args` - A `FluidGlassArgs` struct that specifies the appearance and behavior of the glass
///   effect. This includes properties like tint color, shape, blur radius, and noise level.
///   The builder pattern is recommended for constructing the arguments.
///
/// * `ripple_state` - An optional `Arc<RippleState>` to enable and manage a ripple effect on user
///   interaction, such as a click. When `None`, no ripple effect is applied.
///
/// * `child` - A closure that defines the child components to be rendered on top of the glass surface.
///   These children will be contained within the bounds of the `fluid_glass` component.
pub fn fluid_glass(
    mut args: FluidGlassArgs,
    ripple_state: Option<Arc<RippleState>>,
    child: impl FnOnce(),
) {
    if let Some(ripple_state) = &ripple_state {
        if let Some((progress, center)) = ripple_state.get_animation_progress() {
            args.ripple_center = Some(center);
            args.ripple_radius = Some(progress);
            args.ripple_alpha = Some((1.0 - progress) * 0.3);
            args.ripple_strength = Some(progress);
        }
    }
    (child)();
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

        let child_constraint = Constraint::new(
            remove_padding_from_dimension(
                effective_glass_constraint.width,
                args_measure_clone.padding.into(),
            ),
            remove_padding_from_dimension(
                effective_glass_constraint.height,
                args_measure_clone.padding.into(),
            ),
        );

        let child_measurement = if !input.children_ids.is_empty() {
            let child_measurement =
                input.measure_child(input.children_ids[0], &child_constraint)?;
            input.place_child(
                input.children_ids[0],
                PxPosition {
                    x: args.padding.into(),
                    y: args.padding.into(),
                },
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
            let mut metadata = input.metadata_mut();
            metadata.push_compute_command(blur_command);
            metadata.push_compute_command(blur_command2);
        }

        if let Some(contrast_value) = args.contrast {
            let mean_command =
                MeanCommand::new(input.gpu, &mut input.compute_resource_manager.write());
            let contrast_command =
                ContrastCommand::new(contrast_value, mean_command.result_buffer_ref());
            let mut metadata = input.metadata_mut();
            metadata.push_compute_command(mean_command);
            metadata.push_compute_command(contrast_command);
        }

        let drawable = FluidGlassCommand {
            args: args_measure_clone.clone(),
        };

        input.metadata_mut().push_draw_command(drawable);

        let padding_px: Px = args_measure_clone.padding.into();
        let min_width = child_measurement.width + padding_px * 2;
        let min_height = child_measurement.height + padding_px * 2;
        let width = match effective_glass_constraint.width {
            DimensionValue::Fixed(value) => value,
            DimensionValue::Wrap { min, max } => min
                .unwrap_or(Px(0))
                .max(min_width)
                .min(max.unwrap_or(Px::MAX)),
            DimensionValue::Fill { min, max } => max
                .expect("Seems that you are trying to fill an infinite width, which is not allowed")
                .max(min_width)
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

    if let Some(on_click) = args.on_click {
        let ripple_state = ripple_state.clone();
        state_handler(Box::new(move |mut input| {
            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position_rel;
            let is_cursor_in = cursor_pos_option
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);

            if is_cursor_in {
                input.requests.cursor_icon = CursorIcon::Pointer;
            }

            if is_cursor_in {
                if let Some(_event) = input.cursor_events.iter().find(|e| {
                    matches!(
                        e.content,
                        CursorEventContent::Pressed(PressKeyEventType::Left)
                    )
                }) {
                    if let Some(ripple_state) = &ripple_state {
                        if let Some(pos) = input.cursor_position_rel {
                            let size = input.computed_data;
                            let normalized_pos = [
                                pos.x.to_f32() / size.width.to_f32(),
                                pos.y.to_f32() / size.height.to_f32(),
                            ];
                            ripple_state.start_animation(normalized_pos);
                        }
                    }
                    on_click();
                }

                if args.block_input {
                    // Consume all input events to prevent interaction with underlying components
                    input.block_all();
                }
            }
        }));
    } else if args.block_input {
        state_handler(Box::new(move |mut input| {
            let size = input.computed_data;
            let cursor_pos_option = input.cursor_position_rel;
            let is_cursor_in = cursor_pos_option
                .map(|pos| is_position_in_component(size, pos))
                .unwrap_or(false);

            if is_cursor_in {
                // Consume all input events to prevent interaction with underlying components
                input.block_all();
            }
        }));
    }
}
