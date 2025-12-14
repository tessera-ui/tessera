//! Material-styled button components.
//!
//! ## Usage
//!
//! Trigger actions, submit forms, or navigate.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{Color, DimensionValue, Dp, accesskit::Role, tessera, use_context};

use crate::{
    ShadowProps,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    theme::{
        MaterialAlpha, MaterialColorScheme, MaterialTheme, content_color_for, provide_text_style,
    },
};

/// Material Design 3 defaults for [`button`].
pub struct ButtonDefaults;

impl ButtonDefaults {
    /// Default hover alpha used for button state layers.
    pub const HOVER_ALPHA: f32 = MaterialAlpha::HOVER;
    /// Default pressed alpha used for ripple feedback.
    pub const PRESSED_ALPHA: f32 = MaterialAlpha::PRESSED;
    /// Default disabled container alpha.
    pub const DISABLED_CONTAINER_ALPHA: f32 = MaterialAlpha::DISABLED_CONTAINER;
    /// Default disabled content alpha.
    pub const DISABLED_CONTENT_ALPHA: f32 = MaterialAlpha::DISABLED_CONTENT;

    /// Default disabled container color for most buttons.
    pub fn disabled_container_color(scheme: &MaterialColorScheme) -> Color {
        scheme.on_surface.with_alpha(Self::DISABLED_CONTAINER_ALPHA)
    }

    /// Default disabled content color for most buttons.
    pub fn disabled_content_color(scheme: &MaterialColorScheme) -> Color {
        scheme.on_surface.with_alpha(Self::DISABLED_CONTENT_ALPHA)
    }

    /// Default disabled border color for outlined buttons.
    pub fn disabled_border_color(scheme: &MaterialColorScheme) -> Color {
        scheme.on_surface.with_alpha(Self::DISABLED_CONTAINER_ALPHA)
    }

    /// Returns a state-layer hover color computed from a container + overlay
    /// color.
    pub fn hover_color(container: Color, overlay: Color) -> Color {
        container.blend_over(overlay, Self::HOVER_ALPHA)
    }
}

/// Arguments for the `button` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ButtonArgs {
    /// Whether the button is enabled for user interaction.
    #[builder(default = "true")]
    pub enabled: bool,
    /// The fill color of the button (RGBA).
    #[builder(default = "use_context::<MaterialTheme>().get().color_scheme.primary")]
    pub color: Color,
    /// Optional explicit content color override for descendants.
    ///
    /// When `None`, the button derives its content color from the theme.
    #[builder(default, setter(strip_option))]
    pub content_color: Option<Color>,
    /// The hover color of the button (RGBA). If None, no hover effect is
    /// applied.
    #[builder(
        default = "{ let scheme = use_context::<MaterialTheme>().get().color_scheme; Some(ButtonDefaults::hover_color(scheme.primary, scheme.on_primary)) }"
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
    #[builder(default, setter(custom, strip_option))]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// The ripple color (RGB) for the button.
    #[builder(
        default = "use_context::<MaterialTheme>().get().color_scheme.on_primary.with_alpha(ButtonDefaults::PRESSED_ALPHA)"
    )]
    pub ripple_color: Color,
    /// Width of the border. If > 0, an outline will be drawn.
    #[builder(default = "Dp(0.0)")]
    pub border_width: Dp,
    /// Optional color for the border (RGBA). If None and border_width > 0,
    /// `color` will be used.
    #[builder(default)]
    pub border_color: Option<Color>,
    /// Shadow of the button. If None, no shadow is applied.
    #[builder(default, setter(strip_option))]
    pub shadow: Option<ShadowProps>,
    /// Optional shadow elevation hint forwarded to the underlying surface.
    #[builder(default, setter(strip_option))]
    pub shadow_elevation: Option<Dp>,
    /// Tonal elevation forwarded to the underlying surface.
    #[builder(default = "Dp(0.0)")]
    pub tonal_elevation: Dp,
    /// Container color used when `enabled=false`.
    #[builder(
        default = "ButtonDefaults::disabled_container_color(&use_context::<MaterialTheme>().get().color_scheme)"
    )]
    pub disabled_container_color: Color,
    /// Content color used when `enabled=false`.
    #[builder(
        default = "ButtonDefaults::disabled_content_color(&use_context::<MaterialTheme>().get().color_scheme)"
    )]
    pub disabled_content_color: Color,
    /// Border color used when `enabled=false` and an outline is drawn.
    #[builder(
        default = "ButtonDefaults::disabled_border_color(&use_context::<MaterialTheme>().get().color_scheme)"
    )]
    pub disabled_border_color: Color,
    /// Optional label announced by assistive technologies (e.g., screen
    /// readers).
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional longer description or hint for assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
}

impl ButtonArgsBuilder {
    /// Set the click handler.
    pub fn on_click<F>(mut self, on_click: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Some(Arc::new(on_click)));
        self
    }

    /// Set the click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_click = Some(Some(on_click));
        self
    }
}

impl Default for ButtonArgs {
    fn default() -> Self {
        ButtonArgsBuilder::default()
            .on_click(|| {})
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
/// - `args` — configures the button's appearance and `on_click` handler; see
///   [`ButtonArgs`].
/// - `child` — a closure that renders the button's content (e.g., text or an
///   icon).
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui_basic_components::{
///     button::{ButtonArgs, button},
///     text::{TextArgsBuilder, text},
/// };
///
/// button(ButtonArgs::filled(|| {}), || {
///     text(
///         TextArgsBuilder::default()
///             .text("Click Me".to_string())
///             .build()
///             .expect("builder construction failed"),
///     );
/// });
/// # }
/// # component();
/// ```
#[tessera]
pub fn button(args: impl Into<ButtonArgs>, child: impl FnOnce()) {
    let button_args: ButtonArgs = args.into();
    let typography = use_context::<MaterialTheme>().get().typography;

    // Create interactive surface for button
    surface(create_surface_args(&button_args), || {
        provide_text_style(typography.label_large, child)
    });
}

/// Create surface arguments based on button configuration
fn create_surface_args(args: &ButtonArgs) -> crate::surface::SurfaceArgs {
    let scheme = use_context::<MaterialTheme>().get().color_scheme;

    let container_color = if args.enabled {
        args.color
    } else {
        args.disabled_container_color
    };

    let content_color = if args.enabled {
        args.content_color
            .unwrap_or_else(|| content_color_for(args.color, &scheme))
    } else {
        args.disabled_content_color
    };

    let style = if args.border_width.to_pixels_f32() > 0.0 {
        let border_color = if args.enabled {
            args.border_color.unwrap_or(container_color)
        } else {
            args.disabled_border_color
        };
        crate::surface::SurfaceStyle::FilledOutlined {
            fill_color: container_color,
            border_color,
            border_width: args.border_width,
        }
    } else {
        crate::surface::SurfaceStyle::Filled {
            color: container_color,
        }
    };

    let hover_style = if args.enabled
        && args.on_click.is_some()
        && let Some(hover_color) = args.hover_color
    {
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
    if let Some(shadow_elevation) = args.shadow_elevation {
        builder = builder.shadow_elevation(shadow_elevation);
    }

    // Set on_click handler if available
    if args.enabled
        && let Some(on_click) = args.on_click.clone()
    {
        builder = builder.on_click_shared(on_click);
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
        .content_color(content_color)
        .enabled(args.enabled)
        .tonal_elevation(args.tonal_elevation)
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
    pub fn filled(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        ButtonArgsBuilder::default()
            .color(scheme.primary)
            .content_color(scheme.on_primary)
            .hover_color(Some(ButtonDefaults::hover_color(
                scheme.primary,
                scheme.on_primary,
            )))
            .ripple_color(scheme.on_primary.with_alpha(ButtonDefaults::PRESSED_ALPHA))
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for filled button")
    }

    /// Create an "Elevated" button (Medium emphasis).
    /// Uses Surface color (or SurfaceContainerLow if available) with a shadow.
    pub fn elevated(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        ButtonArgsBuilder::default()
            .color(scheme.surface_container_low)
            .content_color(scheme.primary)
            .hover_color(Some(ButtonDefaults::hover_color(
                scheme.surface,
                scheme.primary,
            )))
            .ripple_color(scheme.primary.with_alpha(ButtonDefaults::PRESSED_ALPHA))
            .shadow_elevation(Dp(1.0))
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for elevated button")
    }

    /// Create a "Tonal" button (Medium emphasis).
    /// Uses SecondaryContainer color for container and OnSecondaryContainer for
    /// content.
    pub fn tonal(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        ButtonArgsBuilder::default()
            .color(scheme.secondary_container)
            .content_color(scheme.on_secondary_container)
            .hover_color(Some(ButtonDefaults::hover_color(
                scheme.secondary_container,
                scheme.on_secondary_container,
            )))
            .ripple_color(
                scheme
                    .on_secondary_container
                    .with_alpha(ButtonDefaults::PRESSED_ALPHA),
            )
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for tonal button")
    }

    /// Create an "Outlined" button (Medium emphasis).
    /// Transparent container with an Outline border.
    pub fn outlined(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        ButtonArgsBuilder::default()
            .color(Color::TRANSPARENT)
            .content_color(scheme.primary)
            .disabled_container_color(Color::TRANSPARENT)
            .hover_color(Some(ButtonDefaults::hover_color(
                Color::TRANSPARENT,
                scheme.primary,
            )))
            .ripple_color(scheme.primary.with_alpha(ButtonDefaults::PRESSED_ALPHA))
            .border_width(Dp(1.0))
            .border_color(Some(scheme.outline))
            .on_click(on_click)
            .build()
            .expect("ButtonArgsBuilder failed for outlined button")
    }

    /// Create a "Text" button (Low emphasis).
    /// Transparent container and no border.
    pub fn text(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>().get().color_scheme;
        ButtonArgsBuilder::default()
            .color(Color::TRANSPARENT)
            .content_color(scheme.primary)
            .disabled_container_color(Color::TRANSPARENT)
            .hover_color(Some(ButtonDefaults::hover_color(
                Color::TRANSPARENT,
                scheme.primary,
            )))
            .ripple_color(scheme.primary.with_alpha(ButtonDefaults::PRESSED_ALPHA))
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
