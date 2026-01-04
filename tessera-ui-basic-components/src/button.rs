//! Material-styled button components.
//!
//! ## Usage
//!
//! Trigger actions, submit forms, or navigate.
use std::sync::Arc;

use derive_setters::Setters;
use tessera_ui::{Color, Dp, Modifier, accesskit::Role, tessera, use_context};

use crate::{
    alignment::Alignment,
    modifier::ModifierExt,
    shape_def::Shape,
    surface::{SurfaceArgs, surface},
    theme::{
        ContentColor, MaterialAlpha, MaterialColorScheme, MaterialTheme, content_color_for,
        provide_text_style,
    },
};

/// Material Design 3 defaults for [`button`].
pub struct ButtonDefaults;

impl ButtonDefaults {
    /// Default pressed alpha used for ripple feedback.
    pub const PRESSED_ALPHA: f32 = MaterialAlpha::PRESSED;
    /// Default disabled container alpha.
    pub const DISABLED_CONTAINER_ALPHA: f32 = MaterialAlpha::DISABLED_CONTAINER;
    /// Default disabled content alpha.
    pub const DISABLED_CONTENT_ALPHA: f32 = MaterialAlpha::DISABLED_CONTENT;
    /// Disabled container opacity used by filled/elevated buttons.
    pub const FILLED_DISABLED_CONTAINER_ALPHA: f32 = 0.1;
    /// Disabled content opacity used by most buttons.
    pub const DISABLED_LABEL_ALPHA: f32 = 0.38;
    /// Minimum width for buttons (Material default = 58dp).
    pub const MIN_WIDTH: Dp = Dp(58.0);
    /// Minimum height for buttons (Material default = 40dp).
    pub const MIN_HEIGHT: Dp = Dp(40.0);
    /// Horizontal padding used inside buttons.
    pub const CONTENT_HORIZONTAL_PADDING: Dp = Dp(16.0);
    /// Vertical padding used inside buttons.
    pub const CONTENT_VERTICAL_PADDING: Dp = Dp(8.0);

    /// Default disabled container color for filled buttons.
    pub fn disabled_container_color(scheme: &MaterialColorScheme) -> Color {
        scheme
            .on_surface
            .with_alpha(Self::FILLED_DISABLED_CONTAINER_ALPHA)
    }

    /// Default disabled content color for filled buttons.
    pub fn disabled_content_color(scheme: &MaterialColorScheme) -> Color {
        scheme
            .on_surface_variant
            .with_alpha(Self::DISABLED_LABEL_ALPHA)
    }

    /// Default disabled border color for outlined buttons.
    pub fn disabled_border_color(scheme: &MaterialColorScheme) -> Color {
        scheme
            .outline_variant
            .with_alpha(Self::FILLED_DISABLED_CONTAINER_ALPHA)
    }
}

/// Arguments for the `button` component.
#[derive(Clone, Setters)]
pub struct ButtonArgs {
    /// Whether the button is enabled for user interaction.
    pub enabled: bool,
    /// Optional modifier chain applied to the button subtree.
    pub modifier: Modifier,
    /// The fill color of the button (RGBA).
    pub color: Color,
    /// Optional explicit content color override for descendants.
    ///
    /// When `None`, the button derives its content color from the theme.
    #[setters(strip_option)]
    pub content_color: Option<Color>,
    /// The shape of the button.
    pub shape: Shape,
    /// The padding of the button.
    pub padding: Dp,
    /// The click callback function
    #[setters(skip)]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// The ripple color (RGB) for the button.
    pub ripple_color: Color,
    /// Width of the border. If > 0, an outline will be drawn.
    pub border_width: Dp,
    /// Optional color for the border (RGBA). If None and border_width > 0,
    /// `color` will be used.
    pub border_color: Option<Color>,
    /// Optional shadow elevation hint forwarded to the underlying surface.
    #[setters(strip_option)]
    pub elevation: Option<Dp>,
    /// Tonal elevation forwarded to the underlying surface.
    pub tonal_elevation: Dp,
    /// Container color used when `enabled=false`.
    pub disabled_container_color: Color,
    /// Content color used when `enabled=false`.
    pub disabled_content_color: Color,
    /// Border color used when `enabled=false` and an outline is drawn.
    pub disabled_border_color: Color,
    /// Optional label announced by assistive technologies (e.g., screen
    /// readers).
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional longer description or hint for assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
}

impl ButtonArgs {
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

impl Default for ButtonArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        Self {
            enabled: true,
            modifier: Modifier::new(),
            color: scheme.primary,
            content_color: None,
            shape: Shape::capsule(),
            padding: ButtonDefaults::CONTENT_VERTICAL_PADDING,
            on_click: Some(Arc::new(|| {})),
            ripple_color: scheme.on_primary,
            border_width: Dp(0.0),
            border_color: None,
            elevation: None,
            tonal_elevation: Dp(0.0),
            disabled_container_color: ButtonDefaults::disabled_container_color(&scheme),
            disabled_content_color: ButtonDefaults::disabled_content_color(&scheme),
            disabled_border_color: ButtonDefaults::disabled_border_color(&scheme),
            accessibility_label: None,
            accessibility_description: None,
        }
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
///     text::{TextArgs, text},
/// };
/// # use tessera_ui_basic_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme(|| MaterialTheme::default(), || {
/// button(ButtonArgs::filled(|| {}), || {
///     text(TextArgs::default().text("Click Me"));
/// });
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn button(args: impl Into<ButtonArgs>, child: impl FnOnce() + Send + Sync + 'static) {
    let button_args: ButtonArgs = args.into();
    let typography = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .typography;

    // Create interactive surface for button
    surface(create_surface_args(&button_args), move || {
        Modifier::new()
            .padding_all(button_args.padding)
            .run(move || {
                provide_text_style(typography.label_large, child);
            });
    });
}

/// Create surface arguments based on button configuration
fn create_surface_args(args: &ButtonArgs) -> crate::surface::SurfaceArgs {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let inherited_content_color = use_context::<ContentColor>()
        .map(|c| c.get().current)
        .unwrap_or(ContentColor::default().current);

    let container_color = if args.enabled {
        args.color
    } else {
        args.disabled_container_color
    };

    let content_color = if args.enabled {
        args.content_color.unwrap_or_else(|| {
            content_color_for(args.color, &scheme).unwrap_or(inherited_content_color)
        })
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

    let mut surface_args = SurfaceArgs::default();

    if let Some(elevation) = args.elevation {
        surface_args = surface_args.elevation(elevation);
    }

    // Set on_click handler if available
    if args.enabled
        && let Some(on_click) = args.on_click.clone()
    {
        surface_args = surface_args.on_click_shared(on_click);
    }

    if let Some(label) = args.accessibility_label.clone() {
        surface_args = surface_args.accessibility_label(label);
    }

    if let Some(description) = args.accessibility_description.clone() {
        surface_args = surface_args.accessibility_description(description);
    }

    surface_args
        .style(style)
        .shape(args.shape)
        .modifier(args.modifier.size_in(
            Some(ButtonDefaults::MIN_WIDTH),
            None,
            Some(ButtonDefaults::MIN_HEIGHT),
            None,
        ))
        .ripple_color(args.ripple_color)
        .content_alignment(Alignment::Center)
        .content_color(content_color)
        .enabled(args.enabled)
        .tonal_elevation(args.tonal_elevation)
        .accessibility_role(Role::Button)
        .accessibility_focusable(true)
}

/// Convenience constructors for standard Material Design 3 button styles
impl ButtonArgs {
    /// Create a standard "Filled" button (High emphasis).
    /// Uses Primary color for container and OnPrimary for content.
    pub fn filled(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        ButtonArgs::default()
            .color(scheme.primary)
            .content_color(scheme.on_primary)
            .ripple_color(scheme.on_primary)
            .disabled_container_color(
                scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::FILLED_DISABLED_CONTAINER_ALPHA),
            )
            .disabled_content_color(
                scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
            )
            .on_click(on_click)
    }

    /// Create an "Elevated" button (Medium emphasis).
    /// Uses Surface color (or SurfaceContainerLow if available) with a shadow.
    pub fn elevated(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        ButtonArgs::default()
            .color(scheme.surface_container_low)
            .content_color(scheme.primary)
            .ripple_color(scheme.primary)
            .elevation(Dp(1.0))
            .disabled_container_color(
                scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::FILLED_DISABLED_CONTAINER_ALPHA),
            )
            .disabled_content_color(
                scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
            )
            .on_click(on_click)
    }

    /// Create a "Tonal" button (Medium emphasis).
    /// Uses SecondaryContainer color for container and OnSecondaryContainer for
    /// content.
    pub fn tonal(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        ButtonArgs::default()
            .color(scheme.secondary_container)
            .content_color(scheme.on_secondary_container)
            .ripple_color(scheme.on_secondary_container)
            .disabled_container_color(
                scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::DISABLED_CONTAINER_ALPHA),
            )
            .disabled_content_color(
                scheme
                    .on_surface
                    .with_alpha(ButtonDefaults::DISABLED_CONTENT_ALPHA),
            )
            .on_click(on_click)
    }

    /// Create an "Outlined" button (Medium emphasis).
    /// Transparent container with an Outline border.
    pub fn outlined(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        ButtonArgs::default()
            .color(Color::TRANSPARENT)
            .content_color(scheme.on_surface_variant)
            .disabled_container_color(Color::TRANSPARENT)
            .ripple_color(scheme.on_surface_variant)
            .border_width(Dp(1.0))
            .border_color(Some(scheme.outline_variant))
            .disabled_border_color(
                scheme
                    .outline_variant
                    .with_alpha(ButtonDefaults::FILLED_DISABLED_CONTAINER_ALPHA),
            )
            .disabled_content_color(
                scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
            )
            .on_click(on_click)
    }

    /// Create a "Text" button (Low emphasis).
    /// Transparent container and no border.
    pub fn text(on_click: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        ButtonArgs::default()
            .color(Color::TRANSPARENT)
            .content_color(scheme.primary)
            .disabled_container_color(Color::TRANSPARENT)
            .ripple_color(scheme.primary)
            .disabled_content_color(
                scheme
                    .on_surface_variant
                    .with_alpha(ButtonDefaults::DISABLED_LABEL_ALPHA),
            )
            .on_click(on_click)
    }
}

/// Builder methods for fluent API
impl ButtonArgs {
    /// Sets the fill color for the button.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
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
