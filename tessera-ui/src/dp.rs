//! # Density-Independent Pixels (Dp)
//!
//! This module provides the [`Dp`] type for representing density-independent pixels,
//! a fundamental unit for UI scaling in the Tessera framework.
//!
//! ## Overview
//!
//! Density-independent pixels (dp) are a virtual pixel unit that provides consistent
//! visual sizing across different screen densities. Unlike physical pixels, dp units
//! automatically scale based on the device's screen density, ensuring that UI elements
//! appear at the same physical size regardless of the display's pixel density.
//!
//! ## Scale Factor
//!
//! The conversion between dp and physical pixels is controlled by a global scale factor
//! stored in [`SCALE_FACTOR`]. This factor is typically set based on the device's DPI
//! (dots per inch) and user preferences.
//!
//! ## Usage
//!
//! ```
//! use tessera_ui::Dp;
//!
//! // Create a dp value
//! let padding = Dp(16.0);
//!
//! // Convert to pixels for rendering
//! let pixels = padding.to_pixels_f32();
//!
//! // Create from pixel values
//! let dp_from_pixels = Dp::from_pixels_f32(48.0);
//! ```
//!
//! ## Relationship with Px
//!
//! The [`Dp`] type works closely with the [`Px`] type (physical pixels). You can
//! convert between them using the provided methods, with the conversion automatically
//! applying the current scale factor.

use std::sync::OnceLock;

use parking_lot::RwLock;

use crate::Px;

/// Global scale factor for converting between density-independent pixels and physical pixels.
///
/// This static variable holds the current scale factor used for dp-to-pixel conversions.
/// It's typically initialized once during application startup based on the device's
/// screen density and user scaling preferences.
///
/// The scale factor represents how many physical pixels correspond to one dp unit.
/// For example:
/// - Scale factor of 1.0: 1 dp = 1 pixel (standard density)
/// - Scale factor of 2.0: 1 dp = 2 pixels (high density)
/// - Scale factor of 0.75: 1 dp = 0.75 pixels (low density)
///
/// # Thread Safety
///
/// This variable uses `OnceLock<RwLock<f64>>` to ensure thread-safe access while
/// allowing the scale factor to be updated during runtime if needed.
pub static SCALE_FACTOR: OnceLock<RwLock<f64>> = OnceLock::new();

/// Density-independent pixels (dp) for UI scaling.
///
/// `Dp` represents a length measurement that remains visually consistent across
/// different screen densities. This is essential for creating UIs that look the
/// same physical size on devices with varying pixel densities.
///
/// ## Design Philosophy
///
/// The dp unit is inspired by Android's density-independent pixel system and
/// provides a device-agnostic way to specify UI dimensions. When you specify
/// a button height of `Dp(48.0)`, it will appear roughly the same physical
/// size on a low-DPI laptop screen and a high-DPI mobile device.
///
/// ## Internal Representation
///
/// The `Dp` struct wraps a single `f64` value representing the dp measurement.
/// This value is converted to physical pixels using the global [`SCALE_FACTOR`]
/// when rendering operations require pixel-precise measurements.
///
/// ## Examples
///
/// ```
/// use tessera_ui::Dp;
///
/// // Common UI measurements in dp
/// let small_padding = Dp(8.0);
/// let medium_padding = Dp(16.0);
/// let button_height = Dp(48.0);
/// let large_spacing = Dp(32.0);
///
/// // Convert to pixels for rendering
/// let pixels = button_height.to_pixels_f32();
/// // Result depends on the current scale factor
/// ```
///
/// ## Arithmetic Operations
///
/// While `Dp` doesn't implement arithmetic operators directly, you can perform
/// operations on the inner value:
///
/// ```
/// use tessera_ui::Dp;
///
/// let base_size = Dp(16.0);
/// let double_size = Dp(base_size.0 * 2.0);
/// let half_size = Dp(base_size.0 / 2.0);
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Dp(pub f64);

impl Dp {
    /// Creates a new `Dp` instance with the specified value.
    ///
    /// This is a const function, allowing `Dp` values to be created at compile time.
    ///
    /// # Arguments
    ///
    /// * `value` - The dp value as a floating-point number
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Dp;
    ///
    /// const BUTTON_HEIGHT: Dp = Dp::new(48.0);
    /// let padding = Dp::new(16.0);
    /// ```
    pub const fn new(value: f64) -> Self {
        Dp(value)
    }

    /// Converts this dp value to physical pixels as an `f64`.
    ///
    /// This method applies the current global scale factor to convert density-independent
    /// pixels to physical pixels. The scale factor is read from [`SCALE_FACTOR`].
    ///
    /// # Returns
    ///
    /// The equivalent value in physical pixels as a 64-bit floating-point number.
    /// If the scale factor hasn't been initialized, defaults to 1.0 (no scaling).
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Dp;
    ///
    /// let dp_value = Dp(24.0);
    /// let pixels = dp_value.to_pixels_f64();
    /// // Result depends on the current scale factor
    /// ```
    pub fn to_pixels_f64(&self) -> f64 {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        self.0 * scale_factor
    }

    /// Creates a `Dp` value from physical pixels specified as an `f64`.
    ///
    /// This method performs the inverse conversion of [`to_pixels_f64`](Self::to_pixels_f64),
    /// converting physical pixels back to density-independent pixels using the current
    /// global scale factor.
    ///
    /// # Arguments
    ///
    /// * `value` - The pixel value as a 64-bit floating-point number
    ///
    /// # Returns
    ///
    /// A new `Dp` instance representing the equivalent dp value.
    /// If the scale factor hasn't been initialized, defaults to 1.0 (no scaling).
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Dp;
    ///
    /// // Convert 96 pixels to dp (assuming 2.0 scale factor = 48 dp)
    /// let dp_value = Dp::from_pixels_f64(96.0);
    /// ```
    pub fn from_pixels_f64(value: f64) -> Self {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        Dp(value / scale_factor)
    }

    /// Converts this dp value to physical pixels as a `u32`.
    ///
    /// This method applies the current global scale factor and truncates the result
    /// to an unsigned 32-bit integer. This is commonly used for rendering operations
    /// that require integer pixel coordinates.
    ///
    /// # Returns
    ///
    /// The equivalent value in physical pixels as an unsigned 32-bit integer.
    /// The result is truncated (not rounded) from the floating-point calculation.
    /// If the scale factor hasn't been initialized, defaults to 1.0 (no scaling).
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Dp;
    ///
    /// let dp_value = Dp(24.5);
    /// let pixels = dp_value.to_pixels_u32();
    /// // With scale factor 2.0: 24.5 * 2.0 = 49.0 -> 49u32
    /// ```
    ///
    /// # Note
    ///
    /// Values are truncated, not rounded. For more precise control over rounding
    /// behavior, use [`to_pixels_f64`](Self::to_pixels_f64) and apply your preferred
    /// rounding method.
    pub fn to_pixels_u32(&self) -> u32 {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        (self.0 * scale_factor) as u32
    }

    /// Creates a `Dp` value from physical pixels specified as a `u32`.
    ///
    /// This method converts an unsigned 32-bit integer pixel value to density-independent
    /// pixels using the current global scale factor. The integer is first converted to
    /// `f64` for the calculation.
    ///
    /// # Arguments
    ///
    /// * `value` - The pixel value as an unsigned 32-bit integer
    ///
    /// # Returns
    ///
    /// A new `Dp` instance representing the equivalent dp value.
    /// If the scale factor hasn't been initialized, defaults to 1.0 (no scaling).
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Dp;
    ///
    /// // Convert 96 pixels to dp (assuming 2.0 scale factor = 48.0 dp)
    /// let dp_value = Dp::from_pixels_u32(96);
    /// ```
    pub fn from_pixels_u32(value: u32) -> Self {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        Dp((value as f64) / scale_factor)
    }

    /// Converts this dp value to physical pixels as an `f32`.
    ///
    /// This method applies the current global scale factor and converts the result
    /// to a 32-bit floating-point number. This is commonly used for graphics APIs
    /// that work with `f32` coordinates.
    ///
    /// # Returns
    ///
    /// The equivalent value in physical pixels as a 32-bit floating-point number.
    /// If the scale factor hasn't been initialized, defaults to 1.0 (no scaling).
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Dp;
    ///
    /// let dp_value = Dp(24.0);
    /// let pixels = dp_value.to_pixels_f32();
    /// // With scale factor 1.5: 24.0 * 1.5 = 36.0f32
    /// ```
    ///
    /// # Precision Note
    ///
    /// Converting from `f64` to `f32` may result in precision loss for very large
    /// or very precise values. For maximum precision, use [`to_pixels_f64`](Self::to_pixels_f64).
    pub fn to_pixels_f32(&self) -> f32 {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        (self.0 * scale_factor) as f32
    }

    /// Creates a `Dp` value from physical pixels specified as an `f32`.
    ///
    /// This method converts a 32-bit floating-point pixel value to density-independent
    /// pixels using the current global scale factor. The `f32` value is first converted
    /// to `f64` for internal calculations.
    ///
    /// # Arguments
    ///
    /// * `value` - The pixel value as a 32-bit floating-point number
    ///
    /// # Returns
    ///
    /// A new `Dp` instance representing the equivalent dp value.
    /// If the scale factor hasn't been initialized, defaults to 1.0 (no scaling).
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Dp;
    ///
    /// // Convert 36.0 pixels to dp (assuming 1.5 scale factor = 24.0 dp)
    /// let dp_value = Dp::from_pixels_f32(36.0);
    /// ```
    pub fn from_pixels_f32(value: f32) -> Self {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        Dp((value as f64) / scale_factor)
    }

    /// Converts this `Dp` value to a `Px` (physical pixels) value.
    ///
    /// This method provides a convenient way to convert between the two pixel
    /// types used in the Tessera framework. It applies the current scale factor
    /// and creates a `Px` instance from the result.
    ///
    /// # Returns
    ///
    /// A new `Px` instance representing the equivalent physical pixel value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Dp;
    ///
    /// let dp_value = Dp(24.0);
    /// let px_value = dp_value.to_px();
    /// // px_value now contains the scaled pixel equivalent
    /// ```
    ///
    /// # See Also
    ///
    /// * [`Px::to_dp`] - For the inverse conversion
    /// * [`to_pixels_f32`](Self::to_pixels_f32) - For direct `f32` pixel conversion
    pub fn to_px(&self) -> Px {
        Px::from_f32(self.to_pixels_f32())
    }
}

impl From<f64> for Dp {
    /// Creates a `Dp` instance from an `f64` value.
    ///
    /// This implementation allows for convenient conversion from floating-point
    /// numbers to `Dp` values using the `into()` method or direct assignment
    /// in contexts where type coercion occurs.
    ///
    /// # Arguments
    ///
    /// * `value` - The dp value as a 64-bit floating-point number
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Dp;
    ///
    /// let dp1: Dp = 24.0.into();
    /// let dp2 = Dp::from(16.0);
    ///
    /// // In function calls that expect Dp
    /// fn set_padding(padding: Dp) { /* ... */ }
    /// set_padding(8.0.into());
    /// ```
    fn from(value: f64) -> Self {
        Dp::new(value)
    }
}

impl From<Px> for Dp {
    /// Creates a `Dp` instance from a `Px` (physical pixels) value.
    ///
    /// This implementation enables seamless conversion between the two pixel
    /// types used in the Tessera framework. The conversion applies the inverse
    /// of the current scale factor to convert physical pixels back to
    /// density-independent pixels.
    ///
    /// # Arguments
    ///
    /// * `px` - A `Px` instance representing physical pixels
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::{Dp, Px};
    ///
    /// let px_value = Px::from_f32(48.0);
    /// let dp_value: Dp = px_value.into();
    ///
    /// // Or using From::from
    /// let dp_value2 = Dp::from(px_value);
    /// ```
    ///
    /// # See Also
    ///
    /// * [`to_px`](Self::to_px) - For the inverse conversion
    /// * [`from_pixels_f64`](Self::from_pixels_f64) - For direct pixel-to-dp conversion
    fn from(px: Px) -> Self {
        Dp::from_pixels_f64(px.to_dp().0)
    }
}
