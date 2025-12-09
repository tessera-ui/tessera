//! An interactive button component.
//!
//! ## Usage
//!
//! Use for triggering actions, submitting forms, or navigation.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{Color, DimensionValue, Dp, accesskit::Role, tessera, use_context};

use crate::{
    ShadowProps,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    theme::MaterialColorScheme,
};

/// Arguments for the `button` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ButtonArgs {
    /// The fill color of the button (RGBA).
    #[builder(default = "use_context::<MaterialColorScheme>().primary")]
    pub color: Color,
    /// The hover color of the button (RGBA). If None, no hover effect is applied.
    #[builder(
        default = "Some(use_context::<MaterialColorScheme>().primary.blend_over(use_context::<MaterialColorScheme>().on_primary, 0.08))"
    )]
    pub hover_color: Option<Color>,
    /// The shape of the button.
    #[builder(default = "Shape::rounded_rectangle(Dp(20.0))")]
    pub shape: Shape,
    /// The padding of the button.
    #[builder(default = "Dp(10.0)")]
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
    #[builder(default = "use_context::<MaterialColorScheme>().on_primary.with_alpha(0.12)")]
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
/// - `child` — a closure that renders the button's content (e.g., text or an icon).
///
/// ## Examples
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui::Color;
/// use tessera_ui_basic_components::{
///     button::{ButtonArgsBuilder, button},
///     text::{TextArgsBuilder, text},
/// };
///
/// let args = ButtonArgsBuilder::default()
///     .on_click(Arc::new(|| {}))
///     .build()
///     .unwrap();
///
/// button(args, || {
///     text(
///         TextArgsBuilder::default()
///             .text("Click Me".to_string())
///             .build()
///             .expect("builder construction failed"),
///     );
/// });
/// ```
#[tessera]
pub fn button(args: impl Into<ButtonArgs>, child: impl FnOnce()) {
    let button_args: ButtonArgs = args.into();

    // Create interactive surface for button
    surface(create_surface_args(&button_args), child);
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

/// Convenience constructors for standard Material Design 3 button styles
impl ButtonArgs {
    /// Create a standard "Filled" button (High emphasis).
    /// Uses Primary color for container and OnPrimary for content.
    pub fn filled(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        let scheme = use_context::<MaterialColorScheme>();
        ButtonArgsBuilder::default()
            .color(scheme.primary)
            .hover_color(Some(scheme.primary.blend_over(scheme.on_primary, 0.08)))
            .ripple_color(scheme.on_primary.with_alpha(0.12))
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for filled button")
    }

    /// Create an "Elevated" button (Medium emphasis).
    /// Uses Surface color (or SurfaceContainerLow if available) with a shadow.
    pub fn elevated(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        let scheme = use_context::<MaterialColorScheme>();
        ButtonArgsBuilder::default()
            .color(scheme.surface)
            .hover_color(Some(scheme.surface.blend_over(scheme.primary, 0.08)))
            .ripple_color(scheme.primary.with_alpha(0.12))
            .shadow(ShadowProps::default())
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for elevated button")
    }

    /// Create a "Tonal" button (Medium emphasis).
    /// Uses SecondaryContainer color for container and OnSecondaryContainer for content.
    pub fn tonal(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        let scheme = use_context::<MaterialColorScheme>();
        ButtonArgsBuilder::default()
            .color(scheme.secondary_container)
            .hover_color(Some(
                scheme
                    .secondary_container
                    .blend_over(scheme.on_secondary_container, 0.08),
            ))
            .ripple_color(scheme.on_secondary_container.with_alpha(0.12))
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for tonal button")
    }

    /// Create an "Outlined" button (Medium emphasis).
    /// Transparent container with an Outline border.
    pub fn outlined(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        let scheme = use_context::<MaterialColorScheme>();
        ButtonArgsBuilder::default()
            .color(Color::TRANSPARENT)
            .hover_color(Some(Color::TRANSPARENT.blend_over(scheme.primary, 0.08)))
            .ripple_color(scheme.primary.with_alpha(0.12))
            .border_width(Dp(1.0))
            .border_color(Some(scheme.outline))
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for outlined button")
    }

    /// Create a "Text" button (Low emphasis).
    /// Transparent container and no border.
    pub fn text(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        let scheme = use_context::<MaterialColorScheme>();
        ButtonArgsBuilder::default()
            .color(Color::TRANSPARENT)
            .hover_color(Some(Color::TRANSPARENT.blend_over(scheme.primary, 0.08)))
            .ripple_color(scheme.primary.with_alpha(0.12))
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for text button")
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
