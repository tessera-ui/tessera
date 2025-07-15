//! Material Design 3 color palette constants
//!
//! This module provides Material Design 3 color tokens for consistent theming
//! across the Tessera UI framework example application.
//!
//! Colors are based on the Material Design 3 color system:
//! <https://m3.material.io/styles/color/the-color-system/tokens>

/// Material Design 3 Core Color Palette
pub mod md_colors {
    use tessera_ui::Color;
    /// Primary color - used for key components like buttons, active states
    pub const PRIMARY: Color = Color::new(0.255, 0.384, 0.686, 1.0); // Blue

    /// Primary container - used for containers of primary components
    pub const PRIMARY_CONTAINER: Color = Color::new(0.149, 0.196, 0.267, 1.0);

    /// Secondary color - used for less prominent components
    pub const SECONDARY: Color = Color::new(0.467, 0.282, 0.573, 1.0); // Purple

    /// Tertiary color - used for contrasting accents
    pub const TERTIARY: Color = Color::new(0.047, 0.482, 0.239, 1.0); // Green

    /// Error color - used for error states and destructive actions
    pub const ERROR: Color = Color::new(0.725, 0.094, 0.075, 1.0); // Red

    /// Surface color for backgrounds and containers (Light theme).
    pub const SURFACE: Color = Color::new(0.98, 0.98, 1.0, 1.0); // Light surface
    /// Surface container color for elevated surfaces (Light theme).
    pub const SURFACE_CONTAINER: Color = Color::new(0.94, 0.94, 0.97, 1.0); // Elevated light surface
    /// Surface variant color for alternative surfaces (Light theme).
    pub const SURFACE_VARIANT: Color = Color::new(0.90, 0.90, 0.94, 1.0); // Alternative light surface

    /// Outline color for borders and dividers.
    pub const OUTLINE: Color = Color::new(0.46, 0.46, 0.50, 1.0);

    /// Text color for components on top of a surface color (light theme).
    pub const ON_SURFACE: Color = Color::new(0.0627, 0.0627, 0.0784, 1.0); // Dark text on light surface
    /// Variant of the text color for components on top of a surface color (light theme).
    pub const ON_SURFACE_VARIANT: Color = Color::new(0.286, 0.270, 0.309, 1.0); // Medium text

    /// Ripple effect color.
    pub const RIPPLE: Color = Color::new(1.0, 1.0, 1.0, 1.0); // White ripple for dark surfaces

    /// Transparent version of the tertiary color for overlays.
    pub const TERTIARY_TRANSPARENT: Color = Color::new(0.047, 0.482, 0.239, 0.3);
    /// Transparent version of the surface container color for overlays.
    pub const SURFACE_CONTAINER_TRANSPARENT: Color = Color::new(0.94, 0.94, 0.97, 0.9);
}
