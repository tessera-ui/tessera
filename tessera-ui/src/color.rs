//! Linear sRGB colors.
//!
//! ## Usage
//!
//! Define straight RGBA colors and convert encoded assets explicitly.

use bytemuck::{Pod, Zeroable};

/// Straight RGBA components in linear sRGB, normally in the range `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)] // Ensures C-compatible memory layout for WGPU
pub struct Color {
    /// Red component (0.0 to 1.0)
    pub r: f32,
    /// Green component (0.0 to 1.0)
    pub g: f32,
    /// Blue component (0.0 to 1.0)
    pub b: f32,
    /// Alpha (transparency) component (0.0 = fully transparent, 1.0 = fully
    /// opaque)
    pub a: f32,
}

impl Color {
    /// Fully transparent color (0, 0, 0, 0).
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Pure black color (0, 0, 0, 1).
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// Pure white color (1, 1, 1, 1).
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);

    /// Pure red color (1, 0, 0, 1).
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);

    /// Pure green color (0, 1, 0, 1).
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);

    /// Pure blue color (0, 0, 1, 1).
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);

    /// Gray color (0.12, 0.12, 0.12, 1).
    pub const GRAY: Self = Self::new(0.12, 0.12, 0.12, 1.0);

    /// Alias for [`Color::GRAY`].
    pub const GREY: Self = Self::GRAY;

    /// Teal color (0, 0.5, 0.5, 1).
    pub const TEAL: Self = Self::new(0.0, 0.5, 0.5, 1.0);

    /// Orange color (1, 0.5, 0, 1).
    pub const ORANGE: Self = Self::new(1.0, 0.5, 0.0, 1.0);

    /// Creates a new `Color` from four `f32` values (red, green, blue, alpha).
    #[inline]
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a new opaque `Color` from three `f32` values (red, green, blue).
    #[inline]
    #[must_use]
    pub const fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Normalizes byte RGBA components to `[0, 1]` without sRGB decoding.
    #[inline]
    #[must_use]
    pub const fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Normalizes byte RGB components without sRGB decoding; alpha is 1.
    #[inline]
    #[must_use]
    pub const fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba_u8(r, g, b, 255)
    }

    /// Converts the color to an array of `[f32; 4]`.
    #[inline]
    #[must_use]
    pub const fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Sets the alpha (transparency) component of the color.
    #[inline]
    #[must_use]
    pub const fn with_alpha(self, alpha: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }

    /// Composites `overlay` over this base color using straight RGBA values.
    ///
    /// `overlay_alpha` scales the overlay's own alpha. Transparent colors do
    /// not darken the result, and a fully transparent result returns
    /// transparent black.
    ///
    /// # Parameters
    ///
    /// * `overlay` - The color to be composited on top of `self`.
    /// * `overlay_alpha` - Opacity of the overlay in range `[0.0, 1.0]`.
    ///
    /// # Returns
    ///
    /// A new `Color` representing the blended result.
    #[inline]
    #[must_use]
    pub fn blend_over(self, overlay: Color, overlay_alpha: f32) -> Self {
        let source_alpha = overlay.a * overlay_alpha.clamp(0.0, 1.0);
        let destination_alpha = self.a * (1.0 - source_alpha);
        let alpha = source_alpha + destination_alpha;
        if alpha <= 0.0 {
            return Self::TRANSPARENT;
        }
        Self::new(
            overlay.r.mul_add(source_alpha, self.r * destination_alpha) / alpha,
            overlay.g.mul_add(source_alpha, self.g * destination_alpha) / alpha,
            overlay.b.mul_add(source_alpha, self.b * destination_alpha) / alpha,
            alpha,
        )
    }
    /// Interpolates linear components, clamping `t` to `[0, 1]`.
    #[inline]
    #[must_use]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: (other.r - self.r).mul_add(t, self.r),
            g: (other.g - self.g).mul_add(t, self.g),
            b: (other.b - self.b).mul_add(t, self.b),
            a: (other.a - self.a).mul_add(t, self.a),
        }
    }
}

/// The default color is fully transparent.
impl Default for Color {
    #[inline]
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

/// Converts from a 4-element `f32` array `[r, g, b, a]` to a `Color`.
impl From<[f32; 4]> for Color {
    #[inline]
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self { r, g, b, a }
    }
}

/// Converts from a `Color` to a 4-element `f32` array `[r, g, b, a]`.
impl From<Color> for [f32; 4] {
    #[inline]
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

/// Converts from a 3-element `f32` array `[r, g, b]` to an opaque `Color`.
impl From<[f32; 3]> for Color {
    #[inline]
    fn from([r, g, b]: [f32; 3]) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

/// Converts from a 4-element `u8` array `[r, g, b, a]` to a `Color`.
impl From<[u8; 4]> for Color {
    #[inline]
    fn from([r, g, b, a]: [u8; 4]) -> Self {
        Self::from_rgba_u8(r, g, b, a)
    }
}

/// Converts from a 3-element `u8` array `[r, g, b]` to an opaque `Color`.
impl From<[u8; 3]> for Color {
    #[inline]
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::from_rgb_u8(r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    fn assert_color(actual: Color, expected: Color) {
        for (actual, expected) in actual.to_array().into_iter().zip(expected.to_array()) {
            assert!(
                (actual - expected).abs() <= 1.0e-6,
                "{actual} != {expected}"
            );
        }
    }

    #[test]
    fn state_layer_over_a_transparent_container_keeps_its_rgb() {
        let layer = Color::new(0.2, 0.4, 0.8, 1.0);
        assert_color(
            Color::TRANSPARENT.blend_over(layer, 0.1),
            layer.with_alpha(0.1),
        );
        assert_color(
            Color::new(1.0, 0.0, 0.0, 0.0).blend_over(layer, 0.1),
            layer.with_alpha(0.1),
        );
    }

    #[test]
    fn source_over_combines_both_alphas_instead_of_interpolating_them() {
        assert_color(
            Color::RED
                .with_alpha(0.5)
                .blend_over(Color::BLUE.with_alpha(0.5), 0.5),
            Color::new(0.6, 0.0, 0.4, 0.625),
        );
        assert_color(
            Color::RED.blend_over(Color::BLUE.with_alpha(0.5), 0.5),
            Color::new(0.75, 0.0, 0.25, 1.0),
        );
    }

    #[test]
    fn transparent_and_opaque_source_endpoints_are_preserved() {
        let base = Color::RED.with_alpha(0.4);
        assert_color(base.blend_over(Color::BLUE, 0.0), base);
        assert_color(base.blend_over(Color::BLUE.with_alpha(0.0), 1.0), base);
        assert_color(base.blend_over(Color::BLUE, 1.0), Color::BLUE);
        assert_color(
            Color::TRANSPARENT.blend_over(Color::TRANSPARENT, 1.0),
            Color::TRANSPARENT,
        );
    }
}
