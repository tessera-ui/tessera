//! An interactive button that displays an icon.
//!
//! ## Usage
//!
//! Use for compact actions where an icon is sufficient to convey the meaning.
use tessera_ui::{Callback, Color, Dp, Modifier, tessera, use_context};

use crate::{
    button::{ButtonDefaults, button},
    glass_button::glass_button,
    icon::icon,
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

impl GlassIconButtonBuilder {
    /// Sets the icon content using any supported icon source.
    pub fn icon(mut self, icon: impl Into<Painter>) -> Self {
        self.props.icon = Some(icon.into());
        self
    }
}

fn render_icon_content(content: Painter, tint: Color) {
    icon().painter(content).size(Dp(24.0)).tint(tint);
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

    let mut builder = button()
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
        .border_width(border_width);

    if let Some(border_color) = border_color {
        builder = builder.border_color(border_color);
    }
    if let Some(on_click) = on_click {
        builder = builder.on_click_shared(on_click);
    }
    if let Some(icon) = icon {
        builder = builder.child(move || render_icon_content(icon.clone(), content_color));
    }

    drop(builder);
}

/// # glass_icon_button
///
/// Renders a glass button with an icon as its content.
///
/// ## Usage
///
/// Use for prominent icon-based actions in a layered UI.
///
/// ## Parameters
///
/// - `icon` — optional icon content shown at the center.
/// - `modifier` — modifier chain applied to the glass button.
/// - `on_click` — optional click callback.
/// - `padding` — optional inner padding.
/// - `tint_color` — optional glass tint color.
/// - `shape` — optional shape override.
/// - `blur_radius` — optional blur radius.
/// - `noise_amount` — optional noise amount.
/// - `noise_scale` — optional noise scale.
/// - `time` — optional animated time input.
/// - `contrast` — optional contrast override.
/// - `border` — optional glass border override.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `accessibility_focusable` — optional accessibility focusable flag.
/// - `content_color` — optional icon tint override.
///
/// ## Examples
///
/// ```
/// use tessera_components::{icon_button::glass_icon_button, material_icons::filled};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn component() {
///     glass_icon_button()
///         .icon(filled::STAR_SVG)
///         .on_click(|| {})
///         .tint_color(tessera_ui::Color::new(0.2, 0.5, 0.8, 0.2));
/// }
/// ```
#[tessera]
pub fn glass_icon_button(
    #[prop(skip_setter)] icon: Option<Painter>,
    modifier: Modifier,
    on_click: Option<Callback>,
    padding: Option<Dp>,
    tint_color: Option<Color>,
    shape: Option<Shape>,
    blur_radius: Option<Dp>,
    noise_amount: Option<f32>,
    noise_scale: Option<f32>,
    time: Option<f32>,
    contrast: Option<f32>,
    border: Option<crate::fluid_glass::GlassBorder>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    accessibility_focusable: Option<bool>,
    content_color: Option<Color>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let content_color = content_color.unwrap_or(scheme.on_surface);

    let mut builder = glass_button()
        .modifier(modifier)
        .padding(padding.unwrap_or(Dp(12.0)))
        .tint_color(tint_color.unwrap_or(Color::new(0.5, 0.5, 0.5, 0.1)))
        .shape(shape.unwrap_or(Shape::rounded_rectangle(Dp(25.0))))
        .blur_radius(blur_radius.unwrap_or(Dp(0.0)))
        .noise_amount(noise_amount.unwrap_or(0.0))
        .noise_scale(noise_scale.unwrap_or(1.0))
        .time(time.unwrap_or(0.0));

    if let Some(on_click) = on_click {
        builder = builder.on_click_shared(on_click);
    }
    if let Some(contrast) = contrast {
        builder = builder.contrast(contrast);
    }
    if let Some(border) = border {
        builder = builder.border(border);
    }
    if let Some(label) = accessibility_label {
        builder = builder.accessibility_label(label);
    }
    if let Some(description) = accessibility_description {
        builder = builder.accessibility_description(description);
    }
    if accessibility_focusable.unwrap_or(false) {
        builder = builder.accessibility_focusable(true);
    }
    if let Some(icon) = icon {
        builder = builder.child(move || render_icon_content(icon.clone(), content_color));
    }

    drop(builder);
}
