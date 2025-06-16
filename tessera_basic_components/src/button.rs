use derive_builder::Builder;
use std::sync::Arc;
use tessera::{DimensionValue, Dp};
use tessera_macros::tessera;

use crate::surface::{RippleState, SurfaceArgsBuilder, surface};

/// Arguments for the `button` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ButtonArgs {
    /// The fill color of the button (RGBA).
    #[builder(default = "[0.2, 0.5, 0.8, 1.0]")]
    pub color: [f32; 4],
    /// The corner radius of the button.
    #[builder(default = "8.0")]
    pub corner_radius: f32,
    /// The padding of the button.
    #[builder(default = "Dp(12.0)")]
    pub padding: Dp,
    /// Optional explicit width behavior for the button.
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    /// Optional explicit height behavior for the button.
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,
    /// The click callback function
    pub on_click: Arc<dyn Fn() + Send + Sync>,
    /// The ripple color (RGB) for the button.
    #[builder(default = "[1.0, 1.0, 1.0]")]
    pub ripple_color: [f32; 3],
    /// Width of the border. If > 0, an outline will be drawn.
    #[builder(default = "0.0")]
    pub border_width: f32,
    /// Optional color for the border (RGBA). If None and border_width > 0, `color` will be used.
    #[builder(default)]
    pub border_color: Option<[f32; 4]>,
}

impl std::fmt::Debug for ButtonArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ButtonArgs")
            .field("color", &self.color)
            .field("corner_radius", &self.corner_radius)
            .field("padding", &self.padding)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("on_click", &"<callback>")
            .field("ripple_color", &self.ripple_color)
            .field("border_width", &self.border_width)
            .field("border_color", &self.border_color)
            .finish()
    }
}

impl Default for ButtonArgs {
    fn default() -> Self {
        ButtonArgsBuilder::default()
            .on_click(Arc::new(|| {}))
            .build()
            .unwrap()
    }
}

/// Interactive button component that wraps custom child components with ripple effect.
///
/// # Example
/// ```no_run
/// use tessera_basic_components::button::{button, ButtonArgsBuilder};
/// use tessera_basic_components::text::{text, TextArgsBuilder};
/// use tessera_basic_components::surface::RippleState;
/// use tessera::Dp;
/// use std::sync::Arc;
///
/// let ripple_state = Arc::new(RippleState::new());
/// let args = ButtonArgsBuilder::default()
///     .color([0.1, 0.7, 0.3, 1.0]) // Green button
///     .padding(Dp(16.0))
///     .on_click(Arc::new(|| println!("Button clicked!")))
///     .build()
///     .unwrap();
///
/// button(args, ripple_state, || {
///     text(TextArgsBuilder::default()
///         .text("Click me!".to_string())
///         .color([255, 255, 255])
///         .build()
///         .unwrap());
/// });
/// ```
#[tessera]
pub fn button(args: impl Into<ButtonArgs>, ripple_state: Arc<RippleState>, child: impl FnOnce()) {
    let button_args: ButtonArgs = args.into();

    // Create interactive surface for button
    surface(create_surface_args(&button_args), Some(ripple_state), child);
}

/// Create surface arguments based on button configuration
fn create_surface_args(args: &ButtonArgs) -> crate::surface::SurfaceArgs {
    let mut builder = SurfaceArgsBuilder::default();

    // Set width if available
    if let Some(width) = args.width {
        builder = builder.width(width);
    }

    // Set height if available
    if let Some(height) = args.height {
        builder = builder.height(height);
    }

    builder
        .color(args.color)
        .corner_radius(args.corner_radius)
        .padding(args.padding)
        .border_width(args.border_width)
        .border_color(args.border_color)
        .ripple_color(args.ripple_color)
        .on_click(Some(args.on_click.clone()))
        .build()
        .unwrap()
}

/// Convenience constructors for common button styles
impl ButtonArgs {
    /// Create a primary button with default blue styling
    pub fn primary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color([0.2, 0.5, 0.8, 1.0]) // Blue
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a secondary button with gray styling
    pub fn secondary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color([0.6, 0.6, 0.6, 1.0]) // Gray
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a success button with green styling
    pub fn success(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color([0.1, 0.7, 0.3, 1.0]) // Green
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a danger button with red styling
    pub fn danger(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color([0.8, 0.2, 0.2, 1.0]) // Red
            .on_click(on_click)
            .build()
            .unwrap()
    }
}

/// Builder methods for fluent API
impl ButtonArgs {
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_padding(mut self, padding: Dp) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_corner_radius(mut self, corner_radius: f32) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn with_width(mut self, width: DimensionValue) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: DimensionValue) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_ripple_color(mut self, ripple_color: [f32; 3]) -> Self {
        self.ripple_color = ripple_color;
        self
    }

    pub fn with_border(mut self, width: f32, color: Option<[f32; 4]>) -> Self {
        self.border_width = width;
        self.border_color = color;
        self
    }
}
