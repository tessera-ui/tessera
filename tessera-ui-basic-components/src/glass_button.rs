use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{Color, DimensionValue, Dp};
use tessera_ui_macros::tessera;

use crate::{
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    ripple_state::RippleState,
    shape_def::Shape,
};

/// Arguments for the `glass_button` component.
#[derive(Builder, Clone, Default)]
#[builder(pattern = "owned", setter(into, strip_option), default)]
pub struct GlassButtonArgs {
    /// The click callback function
    #[builder(setter(strip_option, into = false))]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,

    // Ripple effect properties
    /// The ripple color (RGB) for the button.
    #[builder(default = "Color::from_rgb(1.0, 1.0, 1.0)")]
    pub ripple_color: Color,

    // Layout properties
    /// The padding of the button.
    #[builder(default = "Dp(12.0)")]
    pub padding: Dp,
    /// Optional explicit width behavior for the button.
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    /// Optional explicit height behavior for the button.
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,

    // Glass visual properties
    #[builder(default = "Color::new(0.5, 0.5, 0.5, 0.1)")]
    pub tint_color: Color,
    #[builder(default = "Shape::RoundedRectangle { corner_radius: 25.0 }")]
    pub shape: Shape,
    #[builder(default = "0.0")]
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
    #[builder(default = "0.02")]
    pub noise_amount: f32,
    #[builder(default = "1.0")]
    pub noise_scale: f32,
    #[builder(default = "0.0")]
    pub time: f32,
    #[builder(default, setter(strip_option))]
    pub contrast: Option<f32>,
}

/// Convenience constructors for common glass button styles
impl GlassButtonArgs {
    /// Create a primary glass button with default blue tint
    pub fn primary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.2, 0.5, 0.8, 0.2)) // Blue tint
            .build()
            .unwrap()
    }

    /// Create a secondary glass button with gray tint
    pub fn secondary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.6, 0.6, 0.6, 0.2)) // Gray tint
            .build()
            .unwrap()
    }

    /// Create a success glass button with green tint
    pub fn success(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.1, 0.7, 0.3, 0.2)) // Green tint
            .build()
            .unwrap()
    }

    /// Create a danger glass button with red tint
    pub fn danger(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.8, 0.2, 0.2, 0.2)) // Red tint
            .build()
            .unwrap()
    }
}

/// An interactive button with a fluid glass background and a ripple effect.
///
/// This component is a composite of `fluid_glass` for the visuals and a transparent
/// `surface` for interaction handling and the ripple animation.
#[tessera]
pub fn glass_button(
    args: impl Into<GlassButtonArgs>,
    ripple_state: Arc<RippleState>,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    let args: GlassButtonArgs = args.into();

    let mut glass_args_builder = FluidGlassArgsBuilder::default();
    if let Some((progress, center)) = ripple_state.get_animation_progress() {
        let ripple_alpha = (1.0 - progress) * 0.3; // Fade out
        glass_args_builder = glass_args_builder
            .ripple_center(center)
            .ripple_radius(progress)
            .ripple_alpha(ripple_alpha)
            .ripple_strength(progress);
    }

    if let Some(width) = args.width {
        glass_args_builder = glass_args_builder.width(width);
    }
    if let Some(height) = args.height {
        glass_args_builder = glass_args_builder.height(height);
    }
    if let Some(contrast) = args.contrast {
        glass_args_builder = glass_args_builder.contrast(contrast);
    }

    let glass_args = glass_args_builder
        .tint_color(args.tint_color)
        .shape(args.shape)
        .blur_radius(args.blur_radius)
        .g2_k_value(args.g2_k_value)
        .dispersion_height(args.dispersion_height)
        .chroma_multiplier(args.chroma_multiplier)
        .refraction_height(args.refraction_height)
        .refraction_amount(args.refraction_amount)
        .noise_amount(args.noise_amount)
        .noise_scale(args.noise_scale)
        .time(args.time)
        .padding(args.padding);
    let glass_args = if let Some(on_click) = args.on_click {
        glass_args.on_click(on_click).build().unwrap()
    } else {
        glass_args.build().unwrap()
    };

    fluid_glass(glass_args, Some(ripple_state), child);
}
