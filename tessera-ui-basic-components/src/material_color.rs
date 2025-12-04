//! Material Design color utilities for HCT and dynamic scheme generation.
//! ## Usage Generate Material-compliant dynamic palettes for consistent UI theming.

use std::sync::OnceLock;

use material_color_utilities::{
    dynamiccolor::{DynamicSchemeBuilder, MaterialDynamicColors, SpecVersion, Variant},
    hct::Hct,
};
use parking_lot::RwLock;
use tessera_ui::Color;

const DEFAULT_COLOR: Color = Color::from_rgb(0.4039, 0.3137, 0.6431); // #6750A4

static GLOBAL_SCHEME: OnceLock<RwLock<MaterialColorScheme>> = OnceLock::new();

/// Returns the global Material Design 3 color scheme.
///
/// If no scheme has been set, it initializes a default light scheme
/// with a seed color of #6750A4.
pub fn global_material_scheme() -> MaterialColorScheme {
    GLOBAL_SCHEME
        .get_or_init(|| RwLock::new(MaterialColorScheme::light_from_seed(DEFAULT_COLOR)))
        .read()
        .clone()
}

/// Sets the global Material Design 3 color scheme.
///
/// The scheme is generated based on the provided `seed` color and `is_dark` flag.
pub fn set_global_material_scheme(seed: Color, is_dark: bool) {
    let scheme = if is_dark {
        MaterialColorScheme::dark_from_seed(seed)
    } else {
        MaterialColorScheme::light_from_seed(seed)
    };

    GLOBAL_SCHEME
        .get_or_init(|| RwLock::new(scheme.clone()))
        .write()
        .clone_from(&scheme);
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
    }
}

/// Blends two colors, `overlay` drawn over `base`, using the provided `overlay_alpha`.
///
/// The `overlay_alpha` parameter controls the opacity of the `overlay` color,
/// ranging from 0.0 (fully transparent) to 1.0 (fully opaque).
pub fn blend_over(base: Color, overlay: Color, overlay_alpha: f32) -> Color {
    let alpha = overlay_alpha.clamp(0.0, 1.0);
    let r = overlay.r * alpha + base.r * (1.0 - alpha);
    let g = overlay.g * alpha + base.g * (1.0 - alpha);
    let b = overlay.b * alpha + base.b * (1.0 - alpha);
    let a = overlay.a * alpha + base.a * (1.0 - alpha);
    Color::new(r, g, b, a)
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
