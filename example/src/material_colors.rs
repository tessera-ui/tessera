//! Material Design 3 color palette constants
//!
//! This module provides Material Design 3 color tokens for consistent theming
//! across the Tessera UI framework example application.
//!
//! Colors are based on the Material Design 3 color system:
//! https://m3.material.io/styles/color/the-color-system/tokens

/// Material Design 3 Core Color Palette
pub mod md_colors {
    /// Primary color - used for key components like buttons, active states
    pub const PRIMARY: [f32; 4] = [0.255, 0.384, 0.686, 1.0]; // Blue

    /// Primary container - used for containers of primary components
    pub const PRIMARY_CONTAINER: [f32; 4] = [0.149, 0.196, 0.267, 1.0];

    /// Secondary color - used for less prominent components
    pub const SECONDARY: [f32; 4] = [0.467, 0.282, 0.573, 1.0]; // Purple

    /// Tertiary color - used for contrasting accents
    pub const TERTIARY: [f32; 4] = [0.047, 0.482, 0.239, 1.0]; // Green

    /// Error color - used for error states and destructive actions
    pub const ERROR: [f32; 4] = [0.725, 0.094, 0.075, 1.0]; // Red

    /// Surface colors for backgrounds and containers (Light theme)
    pub const SURFACE: [f32; 4] = [0.98, 0.98, 1.0, 1.0]; // Light surface
    pub const SURFACE_CONTAINER: [f32; 4] = [0.94, 0.94, 0.97, 1.0]; // Elevated light surface
    pub const SURFACE_VARIANT: [f32; 4] = [0.90, 0.90, 0.94, 1.0]; // Alternative light surface

    /// Outline colors for borders and dividers
    pub const OUTLINE: [f32; 4] = [0.46, 0.46, 0.50, 1.0];

    /// Text colors (for light theme)
    pub const ON_PRIMARY: [u8; 3] = [255, 255, 255]; // White text on primary
    pub const ON_SURFACE: [u8; 3] = [16, 16, 20]; // Dark text on light surface
    pub const ON_SURFACE_VARIANT: [u8; 3] = [73, 69, 79]; // Medium text

    /// Ripple effect color
    pub const RIPPLE: [f32; 3] = [1.0, 1.0, 1.0]; // White ripple for dark surfaces

    /// Transparent versions for overlays
    pub const TERTIARY_TRANSPARENT: [f32; 4] = [0.047, 0.482, 0.239, 0.3];
    pub const SURFACE_CONTAINER_TRANSPARENT: [f32; 4] = [0.94, 0.94, 0.97, 0.9];
}

/// Helper functions for color manipulation
pub mod color_utils {
    /// Convert RGB values (0-255) to normalized RGBA (0.0-1.0)
    pub fn rgb_to_rgba(r: u8, g: u8, b: u8, a: f32) -> [f32; 4] {
        [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a]
    }

    /// Create a transparent version of a color
    pub fn with_alpha(color: [f32; 4], alpha: f32) -> [f32; 4] {
        [color[0], color[1], color[2], alpha]
    }
}
