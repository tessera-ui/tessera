//! Color utilities for the Tessera UI framework.
//!
//! This module provides the [`Color`] struct and related utilities for working with colors
//! in the linear sRGB color space. The color representation is optimized for GPU rendering
//! and shader compatibility.
//!
//! # Color Space
//!
//! All colors are represented in the linear sRGB color space, which is the standard for
//! modern graphics rendering. This ensures consistent color reproduction across different
//! devices and platforms.
//!
//! # Usage
//!
//! ```
//! use tessera::Color;
//!
//! // Create colors using predefined constants
//! let red = Color::RED;
//! let transparent = Color::TRANSPARENT;
//!
//! // Create colors from f32 values (0.0 to 1.0)
//! let custom_color = Color::new(0.5, 0.3, 0.8, 1.0);
//! let opaque_color = Color::from_rgb(0.2, 0.7, 0.4);
//!
//! // Create colors from u8 values (0 to 255)
//! let from_bytes = Color::from_rgba_u8(128, 64, 192, 255);
//! let from_rgb_bytes = Color::from_rgb_u8(100, 150, 200);
//!
//! // Convert from arrays
//! let from_array: Color = [0.1, 0.2, 0.3, 0.4].into();
//! let to_array: [f32; 4] = custom_color.into();
//! ```

use bytemuck::{Pod, Zeroable};

/// A color in the linear sRGB color space with an alpha component.
///
/// This struct represents a color using four floating-point components: red, green, blue,
/// and alpha (transparency). Values are typically in the range `[0.0, 1.0]`, where:
/// - `0.0` represents no intensity (black for RGB, fully transparent for alpha)
/// - `1.0` represents full intensity (full color for RGB, fully opaque for alpha)
///
/// The struct is designed to be GPU-friendly with a C-compatible memory layout,
/// making it suitable for direct use in shaders and graphics pipelines.
///
/// # Memory Layout
///
/// The struct uses `#[repr(C)]` to ensure a predictable memory layout that matches
/// the expected format for GPU buffers and shader uniforms.
///
/// # Examples
///
/// ```
/// use tessera::Color;
///
/// // Using predefined colors
/// let red = Color::RED;
/// let white = Color::WHITE;
/// let transparent = Color::TRANSPARENT;
///
/// // Creating custom colors
/// let purple = Color::new(0.5, 0.0, 0.5, 1.0);
/// let semi_transparent_blue = Color::new(0.0, 0.0, 1.0, 0.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)] // Ensures C-compatible memory layout for WGPU
pub struct Color {
    /// Red component (0.0 to 1.0)
    pub r: f32,
    /// Green component (0.0 to 1.0)
    pub g: f32,
    /// Blue component (0.0 to 1.0)
    pub b: f32,
    /// Alpha (transparency) component (0.0 = fully transparent, 1.0 = fully opaque)
    pub a: f32,
}

impl Color {
    // --- Common Colors ---

    /// Fully transparent color (0, 0, 0, 0).
    ///
    /// This color is completely invisible and is often used as a default
    /// or for creating transparent backgrounds.
    pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);

    /// Pure black color (0, 0, 0, 1).
    ///
    /// Represents the absence of all light, fully opaque.
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);

    /// Pure white color (1, 1, 1, 1).
    ///
    /// Represents the presence of all light at full intensity, fully opaque.
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);

    /// Pure red color (1, 0, 0, 1).
    ///
    /// Full intensity red with no green or blue components, fully opaque.
    pub const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);

    /// Pure green color (0, 1, 0, 1).
    ///
    /// Full intensity green with no red or blue components, fully opaque.
    pub const GREEN: Color = Color::new(0.0, 1.0, 0.0, 1.0);

    /// Pure blue color (0, 0, 1, 1).
    ///
    /// Full intensity blue with no red or green components, fully opaque.
    pub const BLUE: Color = Color::new(0.0, 0.0, 1.0, 1.0);

    /// Creates a new `Color` from four `f32` values (red, green, blue, alpha).
    ///
    /// # Parameters
    ///
    /// * `r` - Red component, typically in range [0.0, 1.0]
    /// * `g` - Green component, typically in range [0.0, 1.0]
    /// * `b` - Blue component, typically in range [0.0, 1.0]
    /// * `a` - Alpha (transparency) component, typically in range [0.0, 1.0]
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera::Color;
    ///
    /// let red = Color::new(1.0, 0.0, 0.0, 1.0);
    /// let semi_transparent_blue = Color::new(0.0, 0.0, 1.0, 0.5);
    /// let custom_color = Color::new(0.3, 0.7, 0.2, 0.8);
    /// ```
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a new opaque `Color` from three `f32` values (red, green, blue).
    ///
    /// The alpha component is automatically set to 1.0 (fully opaque).
    ///
    /// # Parameters
    ///
    /// * `r` - Red component, typically in range [0.0, 1.0]
    /// * `g` - Green component, typically in range [0.0, 1.0]
    /// * `b` - Blue component, typically in range [0.0, 1.0]
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera::Color;
    ///
    /// let purple = Color::from_rgb(0.5, 0.0, 0.5);
    /// let orange = Color::from_rgb(1.0, 0.5, 0.0);
    /// ```
    #[inline]
    pub const fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Creates a new `Color` from four `u8` values (red, green, blue, alpha).
    ///
    /// This is convenient for working with traditional 8-bit color values
    /// commonly used in image formats and color pickers.
    ///
    /// # Parameters
    ///
    /// * `r` - Red component in range [0, 255]
    /// * `g` - Green component in range [0, 255]
    /// * `b` - Blue component in range [0, 255]
    /// * `a` - Alpha component in range [0, 255] (0 = transparent, 255 = opaque)
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera::Color;
    ///
    /// let red = Color::from_rgba_u8(255, 0, 0, 255);
    /// let semi_transparent_blue = Color::from_rgba_u8(0, 0, 255, 128);
    /// let custom_color = Color::from_rgba_u8(76, 178, 51, 204);
    /// ```
    #[inline]
    pub fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Creates a new opaque `Color` from three `u8` values (red, green, blue).
    ///
    /// The alpha component is automatically set to 255 (fully opaque).
    /// This is convenient for working with traditional RGB color values.
    ///
    /// # Parameters
    ///
    /// * `r` - Red component in range [0, 255]
    /// * `g` - Green component in range [0, 255]
    /// * `b` - Blue component in range [0, 255]
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera::Color;
    ///
    /// let purple = Color::from_rgb_u8(128, 0, 128);
    /// let orange = Color::from_rgb_u8(255, 165, 0);
    /// let dark_green = Color::from_rgb_u8(0, 100, 0);
    /// ```
    #[inline]
    pub fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba_u8(r, g, b, 255)
    }

    /// Converts the color to an array of `[f32; 4]`.
    ///
    /// This is useful for interfacing with graphics APIs and shaders that
    /// expect color data in array format.
    ///
    /// # Returns
    ///
    /// An array `[r, g, b, a]` where each component is an `f32` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera::Color;
    ///
    /// let color = Color::new(0.5, 0.3, 0.8, 1.0);
    /// let array = color.to_array();
    /// assert_eq!(array, [0.5, 0.3, 0.8, 1.0]);
    /// ```
    #[inline]
    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Sets the alpha (transparency) component of the color.
    ///
    /// # Returns
    ///
    /// A new `Color` instance with the updated alpha value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera::Color;
    ///
    /// let color = Color::new(0.5, 0.3, 0.8, 1.0);
    /// let semi_transparent_color = color.alpha(0.5);
    ///
    /// assert_eq!(semi_transparent_color.a, 0.5);
    /// ```
    #[inline]
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }
}

/// The default color is fully transparent.
///
/// This implementation returns [`Color::TRANSPARENT`], which is often
/// the most sensible default for UI elements that may not have an
/// explicit color specified.
///
/// # Examples
///
/// ```
/// use tessera::Color;
///
/// let default_color = Color::default();
/// assert_eq!(default_color, Color::TRANSPARENT);
/// ```
impl Default for Color {
    #[inline]
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

// --- From Conversions ---

/// Converts from a 4-element `f32` array `[r, g, b, a]` to a `Color`.
///
/// # Examples
///
/// ```
/// use tessera::Color;
///
/// let color: Color = [0.5, 0.3, 0.8, 1.0].into();
/// assert_eq!(color, Color::new(0.5, 0.3, 0.8, 1.0));
/// ```
impl From<[f32; 4]> for Color {
    #[inline]
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self { r, g, b, a }
    }
}

/// Converts from a `Color` to a 4-element `f32` array `[r, g, b, a]`.
///
/// # Examples
///
/// ```
/// use tessera::Color;
///
/// let color = Color::new(0.5, 0.3, 0.8, 1.0);
/// let array: [f32; 4] = color.into();
/// assert_eq!(array, [0.5, 0.3, 0.8, 1.0]);
/// ```
impl From<Color> for [f32; 4] {
    #[inline]
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

/// Converts from a 3-element `f32` array `[r, g, b]` to an opaque `Color`.
///
/// The alpha component is automatically set to 1.0 (fully opaque).
///
/// # Examples
///
/// ```
/// use tessera::Color;
///
/// let color: Color = [0.5, 0.3, 0.8].into();
/// assert_eq!(color, Color::new(0.5, 0.3, 0.8, 1.0));
/// ```
impl From<[f32; 3]> for Color {
    #[inline]
    fn from([r, g, b]: [f32; 3]) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

/// Converts from a 4-element `u8` array `[r, g, b, a]` to a `Color`.
///
/// Each component is converted from the range [0, 255] to [0.0, 1.0].
///
/// # Examples
///
/// ```
/// use tessera::Color;
///
/// let color: Color = [255, 128, 64, 255].into();
/// assert_eq!(color, Color::from_rgba_u8(255, 128, 64, 255));
/// ```
impl From<[u8; 4]> for Color {
    #[inline]
    fn from([r, g, b, a]: [u8; 4]) -> Self {
        Self::from_rgba_u8(r, g, b, a)
    }
}

/// Converts from a 3-element `u8` array `[r, g, b]` to an opaque `Color`.
///
/// Each component is converted from the range [0, 255] to [0.0, 1.0].
/// The alpha component is automatically set to 1.0 (fully opaque).
///
/// # Examples
///
/// ```
/// use tessera::Color;
///
/// let color: Color = [255, 128, 64].into();
/// assert_eq!(color, Color::from_rgb_u8(255, 128, 64));
/// ```
impl From<[u8; 3]> for Color {
    #[inline]
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::from_rgb_u8(r, g, b)
    }
}
