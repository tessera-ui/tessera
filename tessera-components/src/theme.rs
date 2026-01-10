//! Material theme primitives for color, typography, and shape.
//!
//! ## Usage
//!
//! Provide app-wide defaults for Material components.

use material_color_utilities::{
    dynamiccolor::{DynamicSchemeBuilder, MaterialDynamicColors, SpecVersion, Variant},
    hct::Hct,
};
use tessera_ui::{Color, Dp, provide_context, tessera};

use crate::shape_def::Shape;

const DEFAULT_COLOR: Color = Color::from_rgb(0.4039, 0.3137, 0.6431); // #6750A4

/// Ambient content color used by text and icons when no explicit tint is
/// provided.
#[derive(Clone, Copy, Debug)]
pub struct ContentColor {
    /// Current content color used by text/icons when no explicit tint is
    /// provided.
    pub current: Color,
}

impl Default for ContentColor {
    fn default() -> Self {
        ContentColor {
            current: Color::BLACK,
        }
    }
}

/// Standard Material 3 alpha values used for state layers and disabled content.
pub struct MaterialAlpha;

impl MaterialAlpha {
    /// Alpha for hover state layers.
    pub const HOVER: f32 = 0.08;
    /// Alpha for pressed state layers.
    pub const PRESSED: f32 = 0.1;
    /// Alpha for focused state layers.
    pub const FOCUSED: f32 = 0.1;
    /// Alpha for dragged state layers.
    pub const DRAGGED: f32 = 0.16;
    /// Alpha for disabled containers (e.g., filled controls).
    pub const DISABLED_CONTAINER: f32 = 0.12;
    /// Alpha for disabled content (text/icons) placed on disabled containers.
    pub const DISABLED_CONTENT: f32 = 0.38;
}

/// Colors used for text selection highlights and handles.
#[derive(Clone, Copy, Debug)]
pub struct TextSelectionColors {
    /// Color of the selection handle (when rendered).
    pub handle: Color,
    /// Background color of the selection highlight.
    pub background: Color,
}

impl TextSelectionColors {
    /// Creates selection colors based on the current Material scheme.
    pub fn from_scheme(scheme: &MaterialColorScheme) -> Self {
        Self {
            handle: scheme.primary,
            background: scheme.primary.with_alpha(0.35),
        }
    }
}

/// Returns the matching content color for a background color in the scheme.
///
/// When the background color does not match a known scheme color, this returns
/// `None`.
pub fn content_color_for(background: Color, scheme: &MaterialColorScheme) -> Option<Color> {
    scheme.content_color_for(background)
}

/// A simple text style used by components to derive default font size and line
/// height.
#[derive(Clone, Copy, Debug)]
pub struct TextStyle {
    /// Font size in density-independent pixels (dp).
    pub font_size: Dp,
    /// Optional line height override in density-independent pixels (dp).
    pub line_height: Option<Dp>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: Dp(16.0),
            line_height: Some(Dp(24.0)),
        }
    }
}

/// Provides a text style to descendants for the duration of `child`.
pub fn provide_text_style(style: TextStyle, child: impl FnOnce()) {
    provide_context(|| style, child);
}

/// Material typography scale used by components to resolve default text styles.
#[derive(Clone, Copy, Debug)]
pub struct MaterialTypography {
    /// Large display text.
    pub display_large: TextStyle,
    /// Medium display text.
    pub display_medium: TextStyle,
    /// Small display text.
    pub display_small: TextStyle,
    /// Large headline text.
    pub headline_large: TextStyle,
    /// Medium headline text.
    pub headline_medium: TextStyle,
    /// Small headline text.
    pub headline_small: TextStyle,
    /// Large title text.
    pub title_large: TextStyle,
    /// Medium title text.
    pub title_medium: TextStyle,
    /// Small title text.
    pub title_small: TextStyle,
    /// Large body text.
    pub body_large: TextStyle,
    /// Medium body text.
    pub body_medium: TextStyle,
    /// Small body text.
    pub body_small: TextStyle,
    /// Large label text.
    pub label_large: TextStyle,
    /// Medium label text.
    pub label_medium: TextStyle,
    /// Small label text.
    pub label_small: TextStyle,
}

impl Default for MaterialTypography {
    fn default() -> Self {
        Self {
            display_large: TextStyle {
                font_size: Dp(57.0),
                line_height: Some(Dp(64.0)),
            },
            display_medium: TextStyle {
                font_size: Dp(45.0),
                line_height: Some(Dp(52.0)),
            },
            display_small: TextStyle {
                font_size: Dp(36.0),
                line_height: Some(Dp(44.0)),
            },
            headline_large: TextStyle {
                font_size: Dp(32.0),
                line_height: Some(Dp(40.0)),
            },
            headline_medium: TextStyle {
                font_size: Dp(28.0),
                line_height: Some(Dp(36.0)),
            },
            headline_small: TextStyle {
                font_size: Dp(24.0),
                line_height: Some(Dp(32.0)),
            },
            title_large: TextStyle {
                font_size: Dp(22.0),
                line_height: Some(Dp(28.0)),
            },
            title_medium: TextStyle {
                font_size: Dp(16.0),
                line_height: Some(Dp(24.0)),
            },
            title_small: TextStyle {
                font_size: Dp(14.0),
                line_height: Some(Dp(20.0)),
            },
            body_large: TextStyle {
                font_size: Dp(16.0),
                line_height: Some(Dp(24.0)),
            },
            body_medium: TextStyle {
                font_size: Dp(14.0),
                line_height: Some(Dp(20.0)),
            },
            body_small: TextStyle {
                font_size: Dp(12.0),
                line_height: Some(Dp(16.0)),
            },
            label_large: TextStyle {
                font_size: Dp(14.0),
                line_height: Some(Dp(20.0)),
            },
            label_medium: TextStyle {
                font_size: Dp(12.0),
                line_height: Some(Dp(16.0)),
            },
            label_small: TextStyle {
                font_size: Dp(11.0),
                line_height: Some(Dp(16.0)),
            },
        }
    }
}

/// Material shape scale used by components to resolve default container shapes.
#[derive(Clone, Copy, Debug)]
pub struct MaterialShapes {
    /// Extra small container shape.
    pub extra_small: Shape,
    /// Small container shape.
    pub small: Shape,
    /// Medium container shape.
    pub medium: Shape,
    /// Large container shape.
    pub large: Shape,
    /// Extra large container shape.
    pub extra_large: Shape,
}

impl Default for MaterialShapes {
    fn default() -> Self {
        Self {
            extra_small: Shape::rounded_rectangle(Dp(4.0)),
            small: Shape::rounded_rectangle(Dp(8.0)),
            medium: Shape::rounded_rectangle(Dp(12.0)),
            large: Shape::rounded_rectangle(Dp(16.0)),
            extra_large: Shape::rounded_rectangle(Dp(28.0)),
        }
    }
}

/// Material theme container holding the three primary Material 3 theme
/// primitives.
#[derive(Clone, Debug, Default)]
pub struct MaterialTheme {
    /// Color scheme used by Material components.
    pub color_scheme: MaterialColorScheme,
    /// Typography scale used by text-based components.
    pub typography: MaterialTypography,
    /// Shape scale used by container components.
    pub shapes: MaterialShapes,
}

impl MaterialTheme {
    /// Create a theme from an explicit color scheme, using default typography
    /// and shapes.
    pub fn from_color_scheme(color_scheme: MaterialColorScheme) -> Self {
        Self {
            color_scheme,
            ..Self::default()
        }
    }

    /// Create a theme from a seed color.
    pub fn from_seed(seed: Color, is_dark: bool) -> Self {
        Self::from_color_scheme(scheme_from_seed(seed, is_dark))
    }
}

/// # material_theme
///
/// Provides a Material theme context (color scheme, typography, shapes) to
/// descendants.
///
/// ## Usage
///
/// Wrap your app (or a subtree) to configure defaults for Material components.
///
/// ## Parameters
///
/// - `theme` — initializer closure for theme configuration; see
///   [`MaterialTheme`].
/// - `child` — subtree that consumes the theme.
///
/// ## Examples
///
/// ```
/// use tessera_components::theme::{
///     MaterialColorScheme, MaterialTheme, MaterialTypography, material_theme,
/// };
/// use tessera_ui::{Color, tessera};
///
/// #[tessera]
/// fn app() {
///     let scheme = MaterialColorScheme::light_from_seed(Color::from_rgb(0.4, 0.3, 0.6));
///     let typography = MaterialTypography::default();
///
///     material_theme(
///         || MaterialTheme {
///             color_scheme: scheme,
///             typography,
///             ..MaterialTheme::default()
///         },
///         || {
///             // Your UI here.
///         },
///     );
/// }
/// ```
#[tessera]
pub fn material_theme(theme: impl FnOnce() -> MaterialTheme, child: impl FnOnce()) {
    provide_context(theme, child);
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
    /// Tint color used by tonal elevation overlays on surfaces.
    pub surface_tint: Color,
    /// Brighter surface color used by expressive surface roles.
    pub surface_bright: Color,
    /// Dimmer surface color used by expressive surface roles.
    pub surface_dim: Color,
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
    /// Fixed primary color role that maintains the same tone in light/dark
    /// themes.
    pub primary_fixed: Color,
    /// Fixed primary dim color role that maintains the same tone in light/dark
    /// themes.
    pub primary_fixed_dim: Color,
    /// Content color used on `primary_fixed` and `primary_fixed_dim`.
    pub on_primary_fixed: Color,
    /// Lower-emphasis content color used on `primary_fixed` roles.
    pub on_primary_fixed_variant: Color,
    /// Fixed secondary color role that maintains the same tone in light/dark
    /// themes.
    pub secondary_fixed: Color,
    /// Fixed secondary dim color role that maintains the same tone in
    /// light/dark themes.
    pub secondary_fixed_dim: Color,
    /// Content color used on `secondary_fixed` and `secondary_fixed_dim`.
    pub on_secondary_fixed: Color,
    /// Lower-emphasis content color used on `secondary_fixed` roles.
    pub on_secondary_fixed_variant: Color,
    /// Fixed tertiary color role that maintains the same tone in light/dark
    /// themes.
    pub tertiary_fixed: Color,
    /// Fixed tertiary dim color role that maintains the same tone in light/dark
    /// themes.
    pub tertiary_fixed_dim: Color,
    /// Content color used on `tertiary_fixed` and `tertiary_fixed_dim`.
    pub on_tertiary_fixed: Color,
    /// Lower-emphasis content color used on `tertiary_fixed` roles.
    pub on_tertiary_fixed_variant: Color,
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

    /// Returns the matching content color for a background color in this
    /// scheme.
    ///
    /// When the background color does not match a known scheme color, this
    /// returns `None`.
    pub fn content_color_for(&self, background: Color) -> Option<Color> {
        if background == self.primary {
            Some(self.on_primary)
        } else if background == self.secondary {
            Some(self.on_secondary)
        } else if background == self.tertiary {
            Some(self.on_tertiary)
        } else if background == self.background {
            Some(self.on_background)
        } else if background == self.error {
            Some(self.on_error)
        } else if background == self.primary_container {
            Some(self.on_primary_container)
        } else if background == self.secondary_container {
            Some(self.on_secondary_container)
        } else if background == self.tertiary_container {
            Some(self.on_tertiary_container)
        } else if background == self.error_container {
            Some(self.on_error_container)
        } else if background == self.inverse_surface {
            Some(self.inverse_on_surface)
        } else if background == self.surface {
            Some(self.on_surface)
        } else if background == self.surface_variant {
            Some(self.on_surface_variant)
        } else if background == self.surface_bright
            || background == self.surface_container
            || background == self.surface_container_high
            || background == self.surface_container_highest
            || background == self.surface_container_low
            || background == self.surface_container_lowest
            || background == self.surface_dim
        {
            Some(self.on_surface)
        } else if background == self.primary_fixed || background == self.primary_fixed_dim {
            Some(self.on_primary_fixed)
        } else if background == self.secondary_fixed || background == self.secondary_fixed_dim {
            Some(self.on_secondary_fixed)
        } else if background == self.tertiary_fixed || background == self.tertiary_fixed_dim {
            Some(self.on_tertiary_fixed)
        } else {
            None
        }
    }

    /// Computes the surface tonal color at an elevation.
    pub fn surface_color_at_elevation(&self, elevation: Dp) -> Color {
        if elevation.0 == 0.0 {
            return self.surface;
        }
        let alpha = ((4.5 * (elevation.0 + 1.0).ln()) + 2.0) / 100.0;
        self.surface.blend_over(self.surface_tint, alpha as f32)
    }

    /// Returns `surface_color_at_elevation` for the scheme surface, otherwise
    /// returns `background`.
    pub fn surface_color_at_elevation_for(&self, background: Color, elevation: Dp) -> Color {
        if background == self.surface {
            self.surface_color_at_elevation(elevation)
        } else {
            background
        }
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
        surface_tint: argb_to_color(dynamic_colors.surface_tint().get_argb(&scheme)),
        inverse_surface: argb_to_color(dynamic_colors.inverse_surface().get_argb(&scheme)),
        inverse_on_surface: argb_to_color(dynamic_colors.inverse_on_surface().get_argb(&scheme)),
        inverse_primary: argb_to_color(dynamic_colors.inverse_primary().get_argb(&scheme)),
        surface_bright: argb_to_color(dynamic_colors.surface_bright().get_argb(&scheme)),
        surface_dim: argb_to_color(dynamic_colors.surface_dim().get_argb(&scheme)),
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
        primary_fixed: argb_to_color(dynamic_colors.primary_fixed().get_argb(&scheme)),
        primary_fixed_dim: argb_to_color(dynamic_colors.primary_fixed_dim().get_argb(&scheme)),
        on_primary_fixed: argb_to_color(dynamic_colors.on_primary_fixed().get_argb(&scheme)),
        on_primary_fixed_variant: argb_to_color(
            dynamic_colors.on_primary_fixed_variant().get_argb(&scheme),
        ),
        secondary_fixed: argb_to_color(dynamic_colors.secondary_fixed().get_argb(&scheme)),
        secondary_fixed_dim: argb_to_color(dynamic_colors.secondary_fixed_dim().get_argb(&scheme)),
        on_secondary_fixed: argb_to_color(dynamic_colors.on_secondary_fixed().get_argb(&scheme)),
        on_secondary_fixed_variant: argb_to_color(
            dynamic_colors
                .on_secondary_fixed_variant()
                .get_argb(&scheme),
        ),
        tertiary_fixed: argb_to_color(dynamic_colors.tertiary_fixed().get_argb(&scheme)),
        tertiary_fixed_dim: argb_to_color(dynamic_colors.tertiary_fixed_dim().get_argb(&scheme)),
        on_tertiary_fixed: argb_to_color(dynamic_colors.on_tertiary_fixed().get_argb(&scheme)),
        on_tertiary_fixed_variant: argb_to_color(
            dynamic_colors.on_tertiary_fixed_variant().get_argb(&scheme),
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
