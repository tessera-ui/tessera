//! Provides an interactive button component with a glassmorphic (glass-like) background and ripple effect.
//!
//! This module defines `glass_button`, a highly customizable button for modern UI applications.
//! It combines advanced glass visual effects with interactive feedback, supporting primary, secondary,
//! success, and danger styles. Typical use cases include visually distinctive action buttons in
//! glassmorphic or layered interfaces, where both aesthetics and user feedback are important.
//!
//! The component is suitable for dashboards, dialogs, toolbars, and any context requiring
//! a visually appealing, interactive button with a translucent, layered look.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{Color, DimensionValue, Dp};
use tessera_ui_macros::tessera;

use crate::{
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
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
    #[builder(default = "Shape::RoundedRectangle { corner_radius: 25.0, g2_k_value: 3.0 }")]
    pub shape: Shape,
    #[builder(default = "0.0")]
    pub blur_radius: f32,
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
    #[builder(default, setter(strip_option))]
    pub border: Option<GlassBorder>,
}

/// Convenience constructors for common glass button styles
impl GlassButtonArgs {
    /// Create a primary glass button with default blue tint
    pub fn primary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.2, 0.5, 0.8, 0.2)) // Blue tint
            .border(GlassBorder::new(Dp(2.0), Color::new(0.2, 0.5, 0.8, 0.5)))
            .build()
            .unwrap()
    }

    /// Create a secondary glass button with gray tint
    pub fn secondary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.6, 0.6, 0.6, 0.2)) // Gray tint
            .border(GlassBorder::new(Dp(2.0), Color::new(0.6, 0.6, 0.6, 0.5)))
            .build()
            .unwrap()
    }

    /// Create a success glass button with green tint
    pub fn success(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.1, 0.7, 0.3, 0.2)) // Green tint
            .border(GlassBorder::new(Dp(2.0), Color::new(0.1, 0.7, 0.3, 0.5)))
            .build()
            .unwrap()
    }

    /// Create a danger glass button with red tint
    pub fn danger(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.8, 0.2, 0.2, 0.2)) // Red tint
            .border(GlassBorder::new(Dp(2.0), Color::new(0.8, 0.2, 0.2, 0.5)))
            .build()
            .unwrap()
    }
}

/// An interactive button with a fluid, glass-like background and a ripple effect on click.
///
/// This component combines a `fluid_glass` background for advanced visual effects with a
/// ripple animation to provide clear user feedback. It is highly customizable, allowing
/// control over the glass appearance, layout, and interaction behavior.
///
/// # Arguments
///
/// * `args` - A struct that provides detailed configuration for the button's appearance
///   and behavior. See [`GlassButtonArgs`] for more details.
/// * `ripple_state` - The state manager for the ripple animation. It should be created
///   once and shared across recompositions.
/// * `child` - A closure that defines the content displayed inside the button, such as text
///   or an icon.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::Color;
/// use tessera_ui_basic_components::{
///     glass_button::{glass_button, GlassButtonArgs},
///     ripple_state::RippleState,
///     text::text,
/// };
///
/// let ripple_state = Arc::new(RippleState::new());
/// glass_button(
///     GlassButtonArgs {
///         on_click: Some(Arc::new(|| { /* Handle click */ })),
///         tint_color: Color::new(0.3, 0.2, 0.5, 0.4),
///         ..Default::default()
///     },
///     ripple_state,
///     || text("Click Me".to_string()),
/// );
/// ```
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

    let mut glass_args = glass_args_builder
        .tint_color(args.tint_color)
        .shape(args.shape)
        .blur_radius(args.blur_radius)
        .dispersion_height(args.dispersion_height)
        .chroma_multiplier(args.chroma_multiplier)
        .refraction_height(args.refraction_height)
        .refraction_amount(args.refraction_amount)
        .noise_amount(args.noise_amount)
        .noise_scale(args.noise_scale)
        .time(args.time)
        .padding(args.padding);

    if let Some(on_click) = args.on_click {
        glass_args = glass_args.on_click(on_click);
    }

    if let Some(border) = args.border {
        glass_args = glass_args.border(border);
    }

    let glass_args = glass_args.build().unwrap();

    fluid_glass(glass_args, Some(ripple_state), child);
}
