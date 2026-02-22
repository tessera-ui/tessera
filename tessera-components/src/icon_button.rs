//! An interactive button that displays an icon.
//!
//! ## Usage
//!
//! Use for compact actions where an icon is sufficient to convey the meaning.
use derive_setters::Setters;
use tessera_ui::{Callback, Color, Dp, Modifier, tessera, use_context};

use crate::{
    button::{ButtonArgs, ButtonDefaults, button},
    glass_button::{GlassButtonArgs, glass_button},
    icon::{IconArgs, icon},
    modifier::ModifierExt,
    shape_def::Shape,
    theme::MaterialTheme,
};

/// Variations of the icon button as per Material Design 3.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum IconButtonVariant {
    /// Transparent background, no border. Low emphasis.
    #[default]
    Standard,
    /// Filled background (Primary). High emphasis.
    Filled,
    /// Filled background (Secondary Container). Medium emphasis.
    FilledTonal,
    /// Transparent background, with border. Medium emphasis.
    Outlined,
}

/// Arguments for [`icon_button`].
#[derive(PartialEq, Clone, Setters)]
pub struct IconButtonArgs {
    /// The variant of the icon button.
    pub variant: IconButtonVariant,
    /// Icon that will be rendered at the center of the button.
    #[setters(into)]
    pub icon: IconArgs,
    /// The click callback function.
    #[setters(skip)]
    pub on_click: Option<Callback>,
    /// Whether the button is enabled.
    pub enabled: bool,
    /// Optional override for the container color.
    #[setters(strip_option)]
    pub color: Option<Color>,
    /// Optional override for the content (icon) color.
    #[setters(strip_option)]
    pub content_color: Option<Color>,
}

impl IconButtonArgs {
    /// Creates a new icon button configuration with the required icon.
    pub fn new(icon: impl Into<IconArgs>) -> Self {
        Self {
            variant: IconButtonVariant::default(),
            icon: icon.into(),
            on_click: None,
            enabled: true,
            color: None,
            content_color: None,
        }
    }

    /// Sets the on_click handler.
    pub fn on_click(mut self, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Callback::new(on_click));
        self
    }

    /// Sets the on_click handler using a shared callback.
    pub fn on_click_shared(mut self, on_click: impl Into<Callback>) -> Self {
        self.on_click = Some(on_click.into());
        self
    }
}

/// Lifted [`glass_button`] counterpart for icon buttons.
#[derive(PartialEq, Clone, Setters)]
pub struct GlassIconButtonArgs {
    /// Appearance/behavior settings for the underlying [`glass_button`].
    #[setters(into)]
    pub button: GlassButtonArgs,
    /// Icon rendered at the center of the glass button.
    #[setters(into)]
    pub icon: IconArgs,
}

impl GlassIconButtonArgs {
    /// Creates a new glass icon button configuration with the required icon.
    pub fn new(icon: impl Into<IconArgs>) -> Self {
        Self {
            button: GlassButtonArgs::default(),
            icon: icon.into(),
        }
    }
}

/// # icon_button
///
/// Renders a Material Design 3 icon button.
///
/// Specs:
/// - Container: 40dp x 40dp
/// - Icon: 24dp
/// - Shape: Circle
/// - Touch Target: Should ideally be 48dp (currently 40dp visual & touch)
///
/// ## Parameters
///
/// - `args` — configures the button variant, icon, and behavior; see
///   [`IconButtonArgs`].
///
/// ## Examples
///
/// ```no_run
/// use std::sync::Arc;
/// use tessera_components::{
///     icon::IconArgs,
///     icon_button::{IconButtonArgs, IconButtonVariant, icon_button},
///     image_vector::{ImageVectorSource, load_image_vector_from_source},
/// };
///
/// let svg_path = "../assets/emoji_u1f416.svg";
/// let vector_data =
///     load_image_vector_from_source(&ImageVectorSource::Path(svg_path.to_string())).unwrap();
///
/// icon_button(
///     &IconButtonArgs::new(IconArgs::from(vector_data.clone()))
///         .variant(IconButtonVariant::Filled)
///         .on_click(|| println!("Clicked!")),
/// );
/// ```
/// Render an icon button.
#[tessera]
pub fn icon_button(args: &IconButtonArgs) {
    let args: IconButtonArgs = args.clone();
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;

    // Determine colors based on variant
    let (default_container_color, default_content_color, border_width, border_color) =
        match args.variant {
            IconButtonVariant::Filled => (scheme.primary, scheme.on_primary, Dp(0.0), None),
            IconButtonVariant::FilledTonal => (
                scheme.secondary_container,
                scheme.on_secondary_container,
                Dp(0.0),
                None,
            ),
            IconButtonVariant::Outlined => (
                Color::TRANSPARENT,
                scheme.on_surface_variant,
                Dp(1.0),
                Some(scheme.outline),
            ),
            IconButtonVariant::Standard => {
                (Color::TRANSPARENT, scheme.on_surface_variant, Dp(0.0), None)
            }
        };

    // Apply overrides
    let container_color = args.color.unwrap_or(default_container_color);
    let content_color = args.content_color.unwrap_or(default_content_color);

    // Use state-layer + ripple derived from the current content color.
    let ripple_color = content_color;

    // Construct ButtonArgs
    let mut button_args = ButtonArgs::default()
        .modifier(Modifier::new().size(Dp(40.0), Dp(40.0)))
        .padding(Dp(8.0))
        .shape(Shape::rounded_rectangle(Dp(20.0)))
        .color(container_color)
        .content_color(content_color)
        .enabled(args.enabled)
        .disabled_container_color(match args.variant {
            IconButtonVariant::Standard | IconButtonVariant::Outlined => Color::TRANSPARENT,
            IconButtonVariant::Filled | IconButtonVariant::FilledTonal => {
                ButtonDefaults::disabled_container_color(&scheme)
            }
        })
        .disabled_content_color(ButtonDefaults::disabled_content_color(&scheme))
        .disabled_border_color(ButtonDefaults::disabled_border_color(&scheme))
        .ripple_color(ripple_color)
        .border_width(border_width);

    if let Some(bc) = border_color {
        button_args = button_args.border_color(Some(bc));
    }

    if let Some(on_click) = args.on_click {
        button_args = button_args.on_click_shared(on_click);
    }

    // Prepare IconArgs
    let mut icon_args = args.icon;
    icon_args.size = Dp(24.0);
    icon_args.tint = content_color;

    button(&crate::button::ButtonArgs::with_child(
        button_args,
        move || {
            icon(&icon_args.clone());
        },
    ));
}

/// # glass_icon_button
///
/// Renders a button with a glass effect and an icon as its content.
///
/// ## Usage
///
/// Use for prominent icon-based actions in a modern, layered UI.
///
/// ## Parameters
///
/// - `args` — configures the underlying glass button and the icon; see
///   [`GlassIconButtonArgs`].
///
/// ## Examples
///
/// ```no_run
/// use tessera_components::{
///     glass_button::GlassButtonArgs,
///     icon::IconArgs,
///     icon_button::{GlassIconButtonArgs, glass_icon_button},
///     image_vector::{ImageVectorSource, load_image_vector_from_source},
/// };
///
/// let svg_path = "../assets/emoji_u1f416.svg";
/// let vector_data =
///     load_image_vector_from_source(&ImageVectorSource::Path(svg_path.to_string())).unwrap();
///
/// glass_icon_button(
///     &GlassIconButtonArgs::new(IconArgs::from(vector_data))
///         .button(GlassButtonArgs::default().on_click(|| {})),
/// );
/// ```
/// Render a glass icon button.
#[tessera]
pub fn glass_icon_button(args: &GlassIconButtonArgs) {
    let args: GlassIconButtonArgs = args.clone();
    let icon_args = args.icon;

    let button_args = args.button.child(move || {
        icon(&icon_args.clone());
    });
    glass_button(&button_args);
}
