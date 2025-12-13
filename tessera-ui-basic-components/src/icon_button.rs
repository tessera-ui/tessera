//! An interactive button that displays an icon.
//!
//! ## Usage
//!
//! Use for compact actions where an icon is sufficient to convey the meaning.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{Color, Dp, tessera, use_context};

use crate::{
    button::{ButtonArgsBuilder, button},
    glass_button::{GlassButtonArgs, glass_button},
    icon::{IconArgs, icon},
    shape_def::Shape,
    theme::MaterialColorScheme,
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
#[derive(Clone, Builder)]
#[builder(pattern = "owned")]
pub struct IconButtonArgs {
    /// The variant of the icon button.
    #[builder(default)]
    pub variant: IconButtonVariant,
    /// Icon that will be rendered at the center of the button.
    #[builder(setter(into))]
    pub icon: IconArgs,
    /// The click callback function.
    #[builder(default, setter(custom))]
    pub on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Whether the button is enabled.
    #[builder(default = "true")]
    pub enabled: bool,
    /// Optional override for the container color.
    #[builder(default, setter(strip_option))]
    pub color: Option<Color>,
    /// Optional override for the content (icon) color.
    #[builder(default, setter(strip_option))]
    pub content_color: Option<Color>,
}

impl IconButtonArgsBuilder {
    /// Sets the on_click handler.
    pub fn on_click(mut self, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_click = Some(Some(Arc::new(on_click)));
        self
    }
}

/// Lifted [`glass_button`] counterpart for icon buttons.
#[derive(Clone, Builder)]
#[builder(pattern = "owned")]
pub struct GlassIconButtonArgs {
    /// Appearance/behavior settings for the underlying [`glass_button`].
    #[builder(default = "GlassButtonArgs::default()", setter(custom))]
    pub button: GlassButtonArgs,
    /// Icon rendered at the center of the glass button.
    #[builder(setter(into))]
    pub icon: IconArgs,
}

impl GlassIconButtonArgsBuilder {
    /// Override the [`GlassButtonArgs`] using either a ready instance or a
    /// builder-produced value.
    pub fn button(mut self, button: impl Into<GlassButtonArgs>) -> Self {
        self.button = Some(button.into());
        self
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
/// use tessera_ui_basic_components::{
///     icon::IconArgsBuilder,
///     icon_button::{IconButtonArgsBuilder, IconButtonVariant, icon_button},
///     image_vector::{ImageVectorSource, load_image_vector_from_source},
/// };
///
/// let svg_path = "../assets/emoji_u1f416.svg";
/// let vector_data =
///     load_image_vector_from_source(&ImageVectorSource::Path(svg_path.to_string())).unwrap();
///
/// icon_button(
///     IconButtonArgsBuilder::default()
///         .variant(IconButtonVariant::Filled)
///         .on_click(|| println!("Clicked!"))
///         .icon(
///             IconArgsBuilder::default()
///                 .content(vector_data.clone())
///                 .build()
///                 .expect("builder construction failed"),
///         )
///         .build()
///         .unwrap(),
/// );
/// ```
#[tessera]
pub fn icon_button(args: impl Into<IconButtonArgs>) {
    let args: IconButtonArgs = args.into();
    let scheme = use_context::<MaterialColorScheme>().get();

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

    // Determine hover and ripple colors
    let hover_overlay_color = content_color;

    let hover_color = if args.variant == IconButtonVariant::Standard
        || args.variant == IconButtonVariant::Outlined
    {
        Some(hover_overlay_color.with_alpha(0.08))
    } else {
        Some(container_color.blend_over(hover_overlay_color, 0.08))
    };

    let ripple_color = hover_overlay_color.with_alpha(0.12);

    // Construct ButtonArgs
    let mut button_builder = ButtonArgsBuilder::default()
        .width(Dp(40.0))
        .height(Dp(40.0))
        .padding(Dp(8.0))
        .shape(Shape::rounded_rectangle(Dp(20.0)))
        .color(container_color)
        .hover_color(hover_color)
        .ripple_color(ripple_color)
        .border_width(border_width);

    if let Some(bc) = border_color {
        button_builder = button_builder.border_color(Some(bc));
    }

    if let Some(on_click) = args.on_click {
        if args.enabled {
            button_builder = button_builder.on_click_shared(on_click);
        }
    }

    // Prepare IconArgs
    let mut icon_args = args.icon;
    icon_args.size = Dp(24.0);
    icon_args.tint = content_color;

    button(
        button_builder
            .build()
            .expect("failed to build icon button args"),
        move || {
            icon(icon_args);
        },
    );
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
/// use tessera_ui_basic_components::{
///     glass_button::GlassButtonArgsBuilder,
///     icon::IconArgsBuilder,
///     icon_button::{GlassIconButtonArgsBuilder, glass_icon_button},
///     image_vector::{ImageVectorSource, load_image_vector_from_source},
/// };
///
/// let svg_path = "../assets/emoji_u1f416.svg";
/// let vector_data =
///     load_image_vector_from_source(&ImageVectorSource::Path(svg_path.to_string())).unwrap();
///
/// glass_icon_button(
///     GlassIconButtonArgsBuilder::default()
///         .button(
///             GlassButtonArgsBuilder::default()
///                 .on_click(|| {})
///                 .build()
///                 .unwrap(),
///         )
///         .icon(
///             IconArgsBuilder::default()
///                 .content(vector_data)
///                 .build()
///                 .expect("builder construction failed"),
///         )
///         .build()
///         .unwrap(),
/// );
/// ```
#[tessera]
pub fn glass_icon_button(args: impl Into<GlassIconButtonArgs>) {
    let args: GlassIconButtonArgs = args.into();
    let icon_args = args.icon.clone();

    glass_button(args.button, move || {
        icon(icon_args.clone());
    });
}
