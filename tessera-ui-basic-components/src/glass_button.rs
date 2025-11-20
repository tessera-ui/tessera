//! An interactive button with a glassmorphic background.
//!
//! ## Usage
//!
//! Use for visually distinctive actions in layered or modern UIs.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{Color, DimensionValue, Dp, tessera};

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
    /// The ripple color (RGB) for the button.
    #[builder(default = "Color::from_rgb(1.0, 1.0, 1.0)")]
    pub ripple_color: Color,
    /// The padding of the button.
    #[builder(default = "Dp(12.0)")]
    pub padding: Dp,
    /// Explicit width behavior for the button. Defaults to `WRAP`.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub width: DimensionValue,
    /// Explicit height behavior for the button. Defaults to `WRAP`.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub height: DimensionValue,
    /// Tint color applied to the glass surface.
    #[builder(default = "Color::new(0.5, 0.5, 0.5, 0.1)")]
    pub tint_color: Color,
    /// Shape used for the button background.
    #[builder(
        default = "Shape::RoundedRectangle { top_left: Dp(25.0), top_right: Dp(25.0), bottom_right: Dp(25.0), bottom_left: Dp(25.0), g2_k_value: 3.0 }"
    )]
    pub shape: Shape,
    /// Blur radius applied to the captured background.
    #[builder(default = "Dp(0.0)")]
    pub blur_radius: Dp,
    /// Virtual height of the chromatic dispersion effect.
    #[builder(default = "Dp(25.0)")]
    pub dispersion_height: Dp,
    /// Multiplier controlling the strength of chromatic aberration.
    #[builder(default = "1.1")]
    pub chroma_multiplier: f32,
    /// Virtual height used when calculating refraction distortion.
    #[builder(default = "Dp(24.0)")]
    pub refraction_height: Dp,
    /// Amount of refraction to apply to the background.
    #[builder(default = "32.0")]
    pub refraction_amount: f32,
    /// Strength of the grain/noise applied across the surface.
    #[builder(default = "0.0")]
    pub noise_amount: f32,
    /// Scale factor for the generated noise texture.
    #[builder(default = "1.0")]
    pub noise_scale: f32,
    /// Time value for animating noise or other procedural effects.
    #[builder(default = "0.0")]
    pub time: f32,
    /// Optional contrast adjustment applied to the glass rendering.
    #[builder(default, setter(strip_option))]
    pub contrast: Option<f32>,
    /// Optional outline configuration for the glass shape.
    #[builder(default, setter(strip_option))]
    pub border: Option<GlassBorder>,
    /// Optional label announced by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional longer description for assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
    /// Whether the button should remain focusable even when no click handler is provided.
    #[builder(default)]
    pub accessibility_focusable: bool,
}

/// Convenience constructors for common glass button styles
impl GlassButtonArgs {
    /// Create a primary glass button with default blue tint
    pub fn primary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.2, 0.5, 0.8, 0.2)) // Blue tint
            .border(GlassBorder::new(Dp(1.0).into()))
            .build()
            .expect("builder construction failed")
    }

    /// Create a secondary glass button with gray tint
    pub fn secondary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.6, 0.6, 0.6, 0.2)) // Gray tint
            .border(GlassBorder::new(Dp(1.0).into()))
            .build()
            .expect("builder construction failed")
    }

    /// Create a success glass button with green tint
    pub fn success(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.1, 0.7, 0.3, 0.2)) // Green tint
            .border(GlassBorder::new(Dp(1.0).into()))
            .build()
            .expect("builder construction failed")
    }

    /// Create a danger glass button with red tint
    pub fn danger(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        GlassButtonArgsBuilder::default()
            .on_click(on_click)
            .tint_color(Color::new(0.8, 0.2, 0.2, 0.2)) // Red tint
            .border(GlassBorder::new(Dp(1.0).into()))
            .build()
            .expect("builder construction failed")
    }
}

/// # glass_button
///
/// Renders an interactive button with a customizable glass effect and ripple animation.
///
/// ## Usage
///
/// Use as a primary action button where a modern, layered look is desired.
///
/// ## Parameters
///
/// - `args` — configures the button's glass appearance and `on_click` handler; see [`GlassButtonArgs`].
/// - `ripple_state` — a clonable [`RippleState`] to manage the ripple animation.
/// - `child` — a closure that renders the button's content (e.g., text or an icon).
///
/// ## Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::Color;
/// use tessera_ui_basic_components::{
///     glass_button::{glass_button, GlassButtonArgs},
///     ripple_state::RippleState,
///     text::{text, TextArgsBuilder},
/// };
///
/// let ripple_state = RippleState::new();
///
/// glass_button(
///     GlassButtonArgs {
///         on_click: Some(Arc::new(|| println!("Button clicked!"))),
///         tint_color: Color::new(0.2, 0.3, 0.8, 0.3),
///         ..Default::default()
///     },
///     ripple_state,
///     || text(TextArgsBuilder::default().text("Click Me".to_string()).build().expect("builder construction failed")),
/// );
/// ```
#[tessera]
pub fn glass_button(
    args: impl Into<GlassButtonArgs>,
    ripple_state: RippleState,
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
        .width(args.width)
        .height(args.height)
        .time(args.time)
        .padding(args.padding);

    if let Some(on_click) = args.on_click {
        glass_args = glass_args.on_click(on_click);
    }

    if let Some(border) = args.border {
        glass_args = glass_args.border(border);
    }

    if let Some(label) = args.accessibility_label {
        glass_args = glass_args.accessibility_label(label);
    }

    if let Some(description) = args.accessibility_description {
        glass_args = glass_args.accessibility_description(description);
    }

    if args.accessibility_focusable {
        glass_args = glass_args.accessibility_focusable(true);
    }

    let glass_args = glass_args.build().expect("builder construction failed");

    fluid_glass(glass_args, Some(ripple_state), child);
}
