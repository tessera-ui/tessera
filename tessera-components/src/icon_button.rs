//! An interactive button that displays an icon.
//!
//! ## Usage
//!
//! Use for compact actions where an icon is sufficient to convey the meaning.
use tessera_ui::{Callback, Color, Dp, Modifier, RenderSlot, tessera, use_context};

use crate::{
    button::{ButtonDefaults, button},
    modifier::ModifierExt as _,
    painter::Painter,
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

impl IconButtonBuilder {
    /// Sets the icon content using any supported icon source.
    pub fn icon(mut self, icon: impl Into<Painter>) -> Self {
        self.props.icon = Some(icon.into());
        self
    }

    /// Applies the standard icon button preset.
    pub fn standard(self) -> Self {
        self.variant(IconButtonVariant::Standard)
    }

    /// Applies the filled icon button preset.
    pub fn filled(self) -> Self {
        self.variant(IconButtonVariant::Filled)
    }

    /// Applies the filled tonal icon button preset.
    pub fn filled_tonal(self) -> Self {
        self.variant(IconButtonVariant::FilledTonal)
    }

    /// Applies the outlined icon button preset.
    pub fn outlined(self) -> Self {
        self.variant(IconButtonVariant::Outlined)
    }
}

/// # icon_button
///
/// Renders a Material Design 3 icon button.
///
/// ## Usage
///
/// Use for compact icon-only actions.
///
/// ## Parameters
///
/// - `variant` — optional icon button visual variant.
/// - `icon` — optional icon content shown at the center.
/// - `on_click` — optional click callback.
/// - `enabled` — optional enabled flag.
/// - `color` — optional container color override.
/// - `content_color` — optional icon color override.
///
/// ## Examples
///
/// ```
/// use tessera_components::{icon_button::icon_button, material_icons::filled};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn component() {
///     icon_button()
///         .icon(filled::STAR_SVG)
///         .filled()
///         .on_click(|| println!("Clicked!"));
/// }
/// ```
#[tessera]
pub fn icon_button(
    variant: Option<IconButtonVariant>,
    #[prop(skip_setter)] icon: Option<Painter>,
    on_click: Option<Callback>,
    enabled: Option<bool>,
    color: Option<Color>,
    content_color: Option<Color>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let variant = variant.unwrap_or_default();
    let enabled = enabled.unwrap_or(true);

    let (default_container_color, default_content_color, border_width, border_color) = match variant
    {
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

    let container_color = color.unwrap_or(default_container_color);
    let content_color = content_color.unwrap_or(default_content_color);
    let ripple_color = content_color;

    let child = icon.map(|icon| {
        RenderSlot::new(move || {
            crate::icon::icon()
                .painter(icon.clone())
                .size(Dp(24.0))
                .tint(content_color);
        })
    });

    button()
        .modifier(Modifier::new().size(Dp(40.0), Dp(40.0)))
        .padding(Dp(8.0))
        .shape(Shape::rounded_rectangle(Dp(20.0)))
        .color(container_color)
        .content_color(content_color)
        .enabled(enabled)
        .disabled_container_color(match variant {
            IconButtonVariant::Standard | IconButtonVariant::Outlined => Color::TRANSPARENT,
            IconButtonVariant::Filled | IconButtonVariant::FilledTonal => {
                ButtonDefaults::disabled_container_color(&scheme)
            }
        })
        .disabled_content_color(ButtonDefaults::disabled_content_color(&scheme))
        .disabled_border_color(ButtonDefaults::disabled_border_color(&scheme))
        .ripple_color(ripple_color)
        .border_width(border_width)
        .border_color_optional(border_color)
        .on_click_optional(on_click)
        .child_optional(child);
}
