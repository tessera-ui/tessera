//! An interactive button with a glassmorphic background.
//!
//! ## Usage
//!
//! Use for visually distinctive actions in layered or modern UIs.
use std::sync::Arc;

use derive_setters::Setters;
use tessera_ui::{Color, Dp, Modifier, tessera};

use crate::{
    fluid_glass::{FluidGlassArgs, GlassBorder, fluid_glass},
    shape_def::{RoundedCorner, Shape},
};

/// Arguments for the `glass_button` component.
#[derive(Clone, Setters)]
#[setters(into)]
pub struct GlassButtonArgs {
    /// Optional modifier chain applied to the button node.
    pub modifier: Modifier,
    /// The click callback function
    #[setters(skip)]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// The ripple color (RGB) for the button.
    pub ripple_color: Color,
    /// The padding of the button.
    pub padding: Dp,
    /// Tint color applied to the glass surface.
    pub tint_color: Color,
    /// Shape used for the button background.
    pub shape: Shape,
    /// Blur radius applied to the captured background.
    pub blur_radius: Dp,
    /// Virtual height of the chromatic dispersion effect.
    pub dispersion_height: Dp,
    /// Multiplier controlling the strength of chromatic aberration.
    pub chroma_multiplier: f32,
    /// Virtual height used when calculating refraction distortion.
    pub refraction_height: Dp,
    /// Amount of refraction to apply to the background.
    pub refraction_amount: f32,
    /// Strength of the grain/noise applied across the surface.
    pub noise_amount: f32,
    /// Scale factor for the generated noise texture.
    pub noise_scale: f32,
    /// Time value for animating noise or other procedural effects.
    pub time: f32,
    /// Optional contrast adjustment applied to the glass rendering.
    #[setters(strip_option)]
    pub contrast: Option<f32>,
    /// Optional outline configuration for the glass shape.
    #[setters(strip_option)]
    pub border: Option<GlassBorder>,
    /// Optional label announced by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional longer description for assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
    /// Whether the button should remain focusable even when no click handler is
    /// provided.
    pub accessibility_focusable: bool,
}

impl GlassButtonArgs {
    /// Set the click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

impl Default for GlassButtonArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new(),
            on_click: None,
            ripple_color: Color::from_rgb(1.0, 1.0, 1.0),
            padding: Dp(12.0),
            tint_color: Color::new(0.5, 0.5, 0.5, 0.1),
            shape: Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(25.0), 3.0),
                top_right: RoundedCorner::manual(Dp(25.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(25.0), 3.0),
            },
            blur_radius: Dp(0.0),
            dispersion_height: Dp(25.0),
            chroma_multiplier: 1.1,
            refraction_height: Dp(24.0),
            refraction_amount: 32.0,
            noise_amount: 0.0,
            noise_scale: 1.0,
            time: 0.0,
            contrast: None,
            border: None,
            accessibility_label: None,
            accessibility_description: None,
            accessibility_focusable: false,
        }
    }
}

/// Convenience constructors for common glass button styles
impl GlassButtonArgs {
    /// Create a primary glass button with default blue tint
    pub fn primary(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        GlassButtonArgs::default()
            .on_click(on_click)
            .tint_color(Color::new(0.2, 0.5, 0.8, 0.2)) // Blue tint
            .border(GlassBorder::new(Dp(1.0).into()))
    }

    /// Create a secondary glass button with gray tint
    pub fn secondary(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        GlassButtonArgs::default()
            .on_click(on_click)
            .tint_color(Color::new(0.6, 0.6, 0.6, 0.2)) // Gray tint
            .border(GlassBorder::new(Dp(1.0).into()))
    }

    /// Create a success glass button with green tint
    pub fn success(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        GlassButtonArgs::default()
            .on_click(on_click)
            .tint_color(Color::new(0.1, 0.7, 0.3, 0.2)) // Green tint
            .border(GlassBorder::new(Dp(1.0).into()))
    }

    /// Create a danger glass button with red tint
    pub fn danger(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        GlassButtonArgs::default()
            .on_click(on_click)
            .tint_color(Color::new(0.8, 0.2, 0.2, 0.2)) // Red tint
            .border(GlassBorder::new(Dp(1.0).into()))
    }
}

/// # glass_button
///
/// Renders an interactive button with a customizable glass effect and ripple
/// animation.
///
/// ## Usage
///
/// Use as a primary action button where a modern, layered look is desired.
///
/// ## Parameters
///
/// - `args` — configures the button's glass appearance and `on_click` handler;
///   see [`GlassButtonArgs`].
/// - `child` — a closure that renders the button's content (e.g., text or an
///   icon).
///
/// ## Examples
///
/// ```
/// use tessera_ui::Color;
/// use tessera_ui_basic_components::{
///     glass_button::{GlassButtonArgs, glass_button},
///     text::{TextArgs, text},
/// };
///
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// glass_button(
///     GlassButtonArgs::default()
///         .on_click(|| println!("Button clicked!"))
///         .tint_color(Color::new(0.2, 0.3, 0.8, 0.3)),
///     || text(TextArgs::default().text("Click Me")),
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn glass_button(
    args: impl Into<GlassButtonArgs>,
    child: impl FnOnce() + Send + Sync + 'static,
) {
    let args: GlassButtonArgs = args.into();

    let mut glass_args = FluidGlassArgs::default();
    if let Some(contrast) = args.contrast {
        glass_args = glass_args.contrast(contrast);
    }

    let mut glass_args = glass_args
        .modifier(args.modifier)
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
        glass_args = glass_args.on_click_shared(on_click);
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

    fluid_glass(glass_args, child);
}
