//! Provide theme contexts
use material_color_utilities::{
    dynamiccolor::{DynamicSchemeBuilder, MaterialDynamicColors, SpecVersion, Variant},
    hct::Hct,
};
use tessera_ui::{Color, provide_context, tessera};

const DEFAULT_COLOR: Color = Color::from_rgb(0.4039, 0.3137, 0.6431); // #6750A4

/// Ambient content color used by text and icons when no explicit tint is provided.
#[derive(Clone, Copy, Debug)]
pub struct ContentColor {
    /// Current content color used by text/icons when no explicit tint is provided.
    pub current: Color,
}

impl Default for ContentColor {
    fn default() -> Self {
        ContentColor {
            current: Color::BLACK,
        }
    }
}

/// Maps a container color to an appropriate foreground color from the scheme.
pub fn content_color_for(container: Color, scheme: &MaterialColorScheme) -> Color {
    if container == scheme.primary {
        scheme.on_primary
    } else if container == scheme.primary_container {
        scheme.on_primary_container
    } else if container == scheme.secondary {
        scheme.on_secondary
    } else if container == scheme.secondary_container {
        scheme.on_secondary_container
    } else if container == scheme.tertiary {
        scheme.on_tertiary
    } else if container == scheme.tertiary_container {
        scheme.on_tertiary_container
    } else if container == scheme.error {
        scheme.on_error
    } else if container == scheme.error_container {
        scheme.on_error_container
    } else if container == scheme.surface_variant {
        scheme.on_surface_variant
    } else if container == scheme.inverse_surface {
        scheme.inverse_on_surface
    } else {
        scheme.on_surface
    }
}

/// Provides a Material theme to descendants.
#[tessera]
pub fn material_theme_provider(scheme: MaterialColorScheme, child: impl FnOnce()) {
    let content_color = ContentColor {
        current: scheme.on_surface,
    };
    provide_context(scheme, || {
        provide_context(content_color, child);
    });
}

/// Generates a Material theme from a seed color and provides it to descendants.
#[tessera]
pub fn material_theme_from_seed(seed: Color, is_dark: bool, child: impl FnOnce()) {
    let scheme = scheme_from_seed(seed, is_dark);
    material_theme_provider(scheme, child);
}

/// A Material Design color scheme, which can be light or dark,
/// produced from a seed color.
#[derive(Clone, Debug)]
pub struct MaterialColorScheme {
    /// Indicates if the scheme is dark mode (`true`) or light mode (`false`).
    pub is_dark: bool,
    /// The primary color of the scheme.
    pub primary: Color,
    /// Color used for content on top of `primary`.
    pub on_primary: Color,
    /// A container color for `primary`.
    pub primary_container: Color,
    /// Color used for content on top of `primary_container`.
    pub on_primary_container: Color,
    /// The secondary color of the scheme.
    pub secondary: Color,
    /// Color used for content on top of `secondary`.
    pub on_secondary: Color,
    /// A container color for `secondary`.
    pub secondary_container: Color,
    /// Color used for content on top of `secondary_container`.
    pub on_secondary_container: Color,
    /// The tertiary color of the scheme.
    pub tertiary: Color,
    /// Color used for content on top of `tertiary`.
    pub on_tertiary: Color,
    /// A container color for `tertiary`.
    pub tertiary_container: Color,
    /// Color used for content on top of `tertiary_container`.
    pub on_tertiary_container: Color,
    /// The error color of the scheme.
    pub error: Color,
    /// Color used for content on top of `error`.
    pub on_error: Color,
    /// A container color for `error`.
    pub error_container: Color,
    /// Color used for content on top of `error_container`.
    pub on_error_container: Color,
    /// The background color of the scheme.
    pub background: Color,
    /// Color used for content on top of `background`.
    pub on_background: Color,
    /// The surface color of the scheme.
    pub surface: Color,
    /// Color used for content on top of `surface`.
    pub on_surface: Color,
    /// A variant of the surface color.
    pub surface_variant: Color,
    /// Color used for content on top of `surface_variant`.
    pub on_surface_variant: Color,
    /// The outline color.
    pub outline: Color,
    /// A variant of the outline color.
    pub outline_variant: Color,
    /// The shadow color.
    pub shadow: Color,
    /// The scrim color.
    pub scrim: Color,
    /// An inverse of the surface color.
    pub inverse_surface: Color,
    /// Color used for content on top of `inverse_surface`.
    pub inverse_on_surface: Color,
    /// An inverse of the primary color.
    pub inverse_primary: Color,
    /// A container color for surfaces.
    pub surface_container: Color,
    /// A high container color for surfaces.
    pub surface_container_high: Color,
    /// A low container color for surfaces.
    pub surface_container_highest: Color,
    /// A low container color for surfaces.
    pub surface_container_low: Color,
    /// A lowest container color for surfaces.
    pub surface_container_lowest: Color,
}

impl MaterialColorScheme {
    /// Generates a light color scheme derived from the provided seed color.
    pub fn light_from_seed(seed: Color) -> Self {
        scheme_from_seed(seed, false)
    }

    /// Generates a dark color scheme derived from the provided seed color.
    pub fn dark_from_seed(seed: Color) -> Self {
        scheme_from_seed(seed, true)
    }
}

impl Default for MaterialColorScheme {
    fn default() -> Self {
        MaterialColorScheme::light_from_seed(DEFAULT_COLOR)
    }
}

fn scheme_from_seed(seed: Color, is_dark: bool) -> MaterialColorScheme {
    let scheme = DynamicSchemeBuilder::default()
        .source_color_hct(Hct::from_int(color_to_argb(seed)))
        .variant(Variant::TonalSpot)
        .spec_version(SpecVersion::Spec2025)
        .is_dark(is_dark)
        .build();
    let dynamic_colors = MaterialDynamicColors::new();

    MaterialColorScheme {
        is_dark,
        primary: argb_to_color(dynamic_colors.primary().get_argb(&scheme)),
        on_primary: argb_to_color(dynamic_colors.on_primary().get_argb(&scheme)),
        primary_container: argb_to_color(dynamic_colors.primary_container().get_argb(&scheme)),
        on_primary_container: argb_to_color(
            dynamic_colors.on_primary_container().get_argb(&scheme),
        ),
        secondary: argb_to_color(dynamic_colors.secondary().get_argb(&scheme)),
        on_secondary: argb_to_color(dynamic_colors.on_secondary().get_argb(&scheme)),
        secondary_container: argb_to_color(dynamic_colors.secondary_container().get_argb(&scheme)),
        on_secondary_container: argb_to_color(
            dynamic_colors.on_secondary_container().get_argb(&scheme),
        ),
        tertiary: argb_to_color(dynamic_colors.tertiary().get_argb(&scheme)),
        on_tertiary: argb_to_color(dynamic_colors.on_tertiary().get_argb(&scheme)),
        tertiary_container: argb_to_color(dynamic_colors.tertiary_container().get_argb(&scheme)),
        on_tertiary_container: argb_to_color(
            dynamic_colors.on_tertiary_container().get_argb(&scheme),
        ),
        error: argb_to_color(dynamic_colors.error().get_argb(&scheme)),
        on_error: argb_to_color(dynamic_colors.on_error().get_argb(&scheme)),
        error_container: argb_to_color(dynamic_colors.error_container().get_argb(&scheme)),
        on_error_container: argb_to_color(dynamic_colors.on_error_container().get_argb(&scheme)),
        background: argb_to_color(dynamic_colors.background().get_argb(&scheme)),
        on_background: argb_to_color(dynamic_colors.on_background().get_argb(&scheme)),
        surface: argb_to_color(dynamic_colors.surface().get_argb(&scheme)),
        on_surface: argb_to_color(dynamic_colors.on_surface().get_argb(&scheme)),
        surface_variant: argb_to_color(dynamic_colors.surface_variant().get_argb(&scheme)),
        on_surface_variant: argb_to_color(dynamic_colors.on_surface_variant().get_argb(&scheme)),
        outline: argb_to_color(dynamic_colors.outline().get_argb(&scheme)),
        outline_variant: argb_to_color(dynamic_colors.outline_variant().get_argb(&scheme)),
        shadow: argb_to_color(dynamic_colors.shadow().get_argb(&scheme)),
        scrim: argb_to_color(dynamic_colors.scrim().get_argb(&scheme)),
        inverse_surface: argb_to_color(dynamic_colors.inverse_surface().get_argb(&scheme)),
        inverse_on_surface: argb_to_color(dynamic_colors.inverse_on_surface().get_argb(&scheme)),
        inverse_primary: argb_to_color(dynamic_colors.inverse_primary().get_argb(&scheme)),
        surface_container: argb_to_color(dynamic_colors.surface_container().get_argb(&scheme)),
        surface_container_high: argb_to_color(
            dynamic_colors.surface_container_high().get_argb(&scheme),
        ),
        surface_container_highest: argb_to_color(
            dynamic_colors.surface_container_highest().get_argb(&scheme),
        ),
        surface_container_low: argb_to_color(
            dynamic_colors.surface_container_low().get_argb(&scheme),
        ),
        surface_container_lowest: argb_to_color(
            dynamic_colors.surface_container_lowest().get_argb(&scheme),
        ),
    }
}

fn linear_to_srgb_channel(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_to_linear_channel(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn color_to_argb(color: Color) -> u32 {
    let r = (linear_to_srgb_channel(color.r) * 255.0 + 0.5) as u32;
    let g = (linear_to_srgb_channel(color.g) * 255.0 + 0.5) as u32;
    let b = (linear_to_srgb_channel(color.b) * 255.0 + 0.5) as u32;
    let a = (color.a.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
    (a << 24) | (r << 16) | (g << 8) | b
}

fn argb_to_color(argb: u32) -> Color {
    let a = ((argb >> 24) & 0xFF) as f32 / 255.0;
    let r_srgb = ((argb >> 16) & 0xFF) as f32 / 255.0;
    let g_srgb = ((argb >> 8) & 0xFF) as f32 / 255.0;
    let b_srgb = (argb & 0xFF) as f32 / 255.0;
    Color::new(
        srgb_to_linear_channel(r_srgb),
        srgb_to_linear_channel(g_srgb),
        srgb_to_linear_channel(b_srgb),
        a,
    )
}
