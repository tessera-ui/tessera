//! An interactive button component.
//!
//! ## Usage
//!
//! Use for triggering actions, submitting forms, or navigation.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{Color, DimensionValue, Dp, accesskit::Role, tessera};

use crate::{
    pipelines::ShadowProps,
    ripple_state::RippleState,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

/// Arguments for the `button` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ButtonArgs {
    /// The fill color of the button (RGBA).
    #[builder(default = "Color::new(0.2, 0.5, 0.8, 1.0)")]
    pub color: Color,
    /// The hover color of the button (RGBA). If None, no hover effect is applied.
    #[builder(default)]
    pub hover_color: Option<Color>,
    /// The shape of the button.
    #[builder(
        default = "Shape::RoundedRectangle { top_left: Dp(25.0), top_right: Dp(25.0), bottom_right: Dp(25.0), bottom_left: Dp(25.0), g2_k_value: 3.0 }"
    )]
    pub shape: Shape,
    /// The padding of the button.
    #[builder(default = "Dp(12.0)")]
    pub padding: Dp,
    /// Optional explicit width behavior for the button.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub width: DimensionValue,
    /// Optional explicit height behavior for the button.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub height: DimensionValue,
    /// The click callback function
    #[builder(default, setter(strip_option))]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// The ripple color (RGB) for the button.
    #[builder(default = "Color::from_rgb(1.0, 1.0, 1.0)")]
    pub ripple_color: Color,
    /// Width of the border. If > 0, an outline will be drawn.
    #[builder(default = "Dp(0.0)")]
    pub border_width: Dp,
    /// Optional color for the border (RGBA). If None and border_width > 0, `color` will be used.
    #[builder(default)]
    pub border_color: Option<Color>,
    /// Shadow of the button. If None, no shadow is applied.
    #[builder(default, setter(strip_option))]
    pub shadow: Option<ShadowProps>,
    /// Optional label announced by assistive technologies (e.g., screen readers).
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional longer description or hint for assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
}

impl Default for ButtonArgs {
    fn default() -> Self {
        ButtonArgsBuilder::default()
            .on_click(Arc::new(|| {}))
            .build()
            .expect("ButtonArgsBuilder default build should succeed")
    }
}

/// # button
///
/// Provides a clickable button with customizable style and ripple feedback.
///
/// ## Usage
///
/// Use to trigger an action when the user clicks or taps.
///
/// ## Parameters
///
/// - `args` — configures the button's appearance and `on_click` handler; see [`ButtonArgs`].
/// - `ripple_state` — a clonable [`RippleState`] to manage the ripple animation.
/// - `child` — a closure that renders the button's content (e.g., text or an icon).
///
/// ## Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::Color;
/// use tessera_ui_basic_components::{
///     button::{button, ButtonArgsBuilder},
///     ripple_state::RippleState,
///     text::{text, TextArgsBuilder},
/// };
///
/// let ripple = RippleState::new();
/// let args = ButtonArgsBuilder::default()
///     .on_click(Arc::new(|| {}))
///     .build()
///     .unwrap();
///
/// button(args, ripple, || {
///     text(TextArgsBuilder::default().text("Click Me".to_string()).build().expect("builder construction failed"));
/// });
/// ```
#[tessera]
pub fn button(args: impl Into<ButtonArgs>, ripple_state: RippleState, child: impl FnOnce()) {
    let button_args: ButtonArgs = args.into();

    // Create interactive surface for button
    surface(create_surface_args(&button_args), Some(ripple_state), child);
}

/// Create surface arguments based on button configuration
fn create_surface_args(args: &ButtonArgs) -> crate::surface::SurfaceArgs {
    let style = if args.border_width.to_pixels_f32() > 0.0 {
        crate::surface::SurfaceStyle::FilledOutlined {
            fill_color: args.color,
            border_color: args.border_color.unwrap_or(args.color),
            border_width: args.border_width,
        }
    } else {
        crate::surface::SurfaceStyle::Filled { color: args.color }
    };

    let hover_style = if let Some(hover_color) = args.hover_color {
        let style = if args.border_width.to_pixels_f32() > 0.0 {
            crate::surface::SurfaceStyle::FilledOutlined {
                fill_color: hover_color,
                border_color: args.border_color.unwrap_or(hover_color),
                border_width: args.border_width,
            }
        } else {
            crate::surface::SurfaceStyle::Filled { color: hover_color }
        };
        Some(style)
    } else {
        None
    };

    let mut builder = SurfaceArgsBuilder::default();

    // Set shadow if available
    if let Some(shadow) = args.shadow {
        builder = builder.shadow(shadow);
    }

    // Set on_click handler if available
    if let Some(on_click) = args.on_click.clone() {
        builder = builder.on_click(on_click);
    }

    if let Some(label) = args.accessibility_label.clone() {
        builder = builder.accessibility_label(label);
    }

    if let Some(description) = args.accessibility_description.clone() {
        builder = builder.accessibility_description(description);
    }

    builder
        .style(style)
        .hover_style(hover_style)
        .shape(args.shape)
        .padding(args.padding)
        .ripple_color(args.ripple_color)
        .width(args.width)
        .height(args.height)
        .accessibility_role(Role::Button)
        .accessibility_focusable(true)
        .build()
        .expect("SurfaceArgsBuilder failed with required button fields set")
}

/// Convenience constructors for common button styles
impl ButtonArgs {
    /// Create a primary button with default blue styling
    pub fn primary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color(Color::new(0.2, 0.5, 0.8, 1.0)) // Blue
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for primary button")
    }

    /// Create a secondary button with gray styling
    pub fn secondary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color(Color::new(0.6, 0.6, 0.6, 1.0)) // Gray
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for secondary button")
    }

    /// Create a success button with green styling
    pub fn success(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color(Color::new(0.1, 0.7, 0.3, 1.0)) // Green
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for success button")
    }

    /// Create a danger button with red styling
    pub fn danger(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color(Color::new(0.8, 0.2, 0.2, 1.0)) // Red
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for danger button")
    }
}

/// Builder methods for fluent API
impl ButtonArgs {
    /// Sets the fill color for the button.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the hover color applied when the pointer is over the button.
    pub fn with_hover_color(mut self, hover_color: Color) -> Self {
        self.hover_color = Some(hover_color);
        self
    }

    /// Updates the padding inside the button.
    pub fn with_padding(mut self, padding: Dp) -> Self {
        self.padding = padding;
        self
    }

    /// Overrides the button's shape.
    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets a fixed or flexible width constraint.
    pub fn with_width(mut self, width: DimensionValue) -> Self {
        self.width = width;
        self
    }

    /// Sets a fixed or flexible height constraint.
    pub fn with_height(mut self, height: DimensionValue) -> Self {
        self.height = height;
        self
    }

    /// Adjusts the ripple color tint.
    pub fn with_ripple_color(mut self, ripple_color: Color) -> Self {
        self.ripple_color = ripple_color;
        self
    }

    /// Configures the border width and optional color.
    pub fn with_border(mut self, width: Dp, color: Option<Color>) -> Self {
        self.border_width = width;
        self.border_color = color;
        self
    }
}
