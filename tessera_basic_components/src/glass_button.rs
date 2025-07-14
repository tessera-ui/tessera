use derive_builder::Builder;
use std::sync::Arc;
use tessera::{Color, DimensionValue, Dp};
use tessera_macros::tessera;

use crate::fluid_glass::{FluidGlassArgsBuilder, fluid_glass};
use crate::surface::{RippleState, SurfaceArgsBuilder, surface};
use crate::{boxed::BoxedArgsBuilder, boxed_ui};

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
    #[builder(default = "Color::new(1.0, 1.0, 1.0, 0.5)")]
    pub highlight_color: Color,
    #[builder(default = "Color::new(0.0, 0.0, 0.0, 0.5)")]
    pub inner_shadow_color: Color,
    #[builder(default = "25.0")]
    pub corner_radius: f32,
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
    #[builder(default = "0.2")]
    pub eccentric_factor: f32,
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
    pub contrast: Option<f32>,
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
        .highlight_color(args.highlight_color)
        .inner_shadow_color(args.inner_shadow_color)
        .corner_radius(args.corner_radius)
        .blur_radius(args.blur_radius)
        .g2_k_value(args.g2_k_value)
        .dispersion_height(args.dispersion_height)
        .chroma_multiplier(args.chroma_multiplier)
        .refraction_height(args.refraction_height)
        .refraction_amount(args.refraction_amount)
        .eccentric_factor(args.eccentric_factor)
        .highlight_size(args.highlight_size)
        .highlight_smoothing(args.highlight_smoothing)
        .inner_shadow_radius(args.inner_shadow_radius)
        .inner_shadow_smoothing(args.inner_shadow_smoothing)
        .noise_amount(args.noise_amount)
        .noise_scale(args.noise_scale)
        .time(args.time)
        .build()
        .unwrap();

    let surface_args = SurfaceArgsBuilder::default()
        .color(Color::new(0.0, 0.0, 0.0, 0.0)) // Transparent surface
        .width(DimensionValue::Wrap {
            min: None,
            max: None,
        })
        .height(DimensionValue::Wrap {
            min: None,
            max: None,
        })
        .padding(args.padding)
        .on_click(args.on_click.clone())
        .ripple_color(args.ripple_color)
        .corner_radius(args.corner_radius)
        .build()
        .unwrap();

    let boxed_args = BoxedArgsBuilder::default()
        .width(DimensionValue::Wrap {
            min: None,
            max: None,
        })
        .height(DimensionValue::Wrap {
            min: None,
            max: None,
        })
        .build()
        .unwrap();

    surface(surface_args, Some(ripple_state), move || {
        boxed_ui!(boxed_args, move || fluid_glass(glass_args), child);
    });
}
