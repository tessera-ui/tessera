//! # Density-Independent Pixels (Dp)
//!
//! This module provides the [`Dp`] type for representing density-independent
//! pixels, a fundamental unit for UI scaling in the Tessera framework.
//!
//! ## Overview
//!
//! Density-independent pixels (dp) are a virtual pixel unit that provides
//! consistent visual sizing across different screen densities. Unlike physical
//! pixels, dp units automatically scale based on the device's screen density,
//! ensuring that UI elements appear at the same physical size regardless of the
//! display's pixel density.
//!
//! ## Scale Factor
//!
//! The conversion between dp and physical pixels is controlled by a global
//! scale factor returned by [`scale_factor`]. This factor is typically set
//! based on the device's DPI (dots per inch) and user preferences, and can be
//! updated at runtime. It is a single process-global value, so a multi-window
//! application cannot customize it per window; see [`set_scale_factor`].
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
//! The [`Dp`] type works closely with the [`Px`] type (physical pixels). You
//! can convert between them using the provided methods, with the conversion
//! automatically applying the current scale factor.

use std::{
    fmt::Display,
    ops::{Div, Mul},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::Px;

/// Bit pattern of `1.0`, used until [`set_scale_factor`] stores another value.
const DEFAULT_SCALE_FACTOR_BITS: u64 = 1.0f64.to_bits();

/// Global scale factor for converting between density-independent pixels and
/// physical pixels, stored as `f64::to_bits`.
///
/// Uninitialized reads return `1.0`. This is a single **process-global** value
/// shared by every window; see [`set_scale_factor`] for the multi-window
/// assumption and for the fact that only positive finite factors are ever
/// stored.
pub(crate) static SCALE_FACTOR: AtomicU64 = AtomicU64::new(DEFAULT_SCALE_FACTOR_BITS);

/// Sets the global scale factor used to convert between dp and physical pixels.
///
/// Unlike a one-shot initialization, this can be called again when the display
/// scale changes. Only *positive finite* values are honored: any `NaN`, `±inf`,
/// or a value `<= 0.0` leaves the previous factor in place and trips a
/// `debug_assert!` so the misuse is flagged during development. This keeps a
/// bad DPI report from silently poisoning every `Dp`↔`Px` conversion. In
/// release builds the setter simply ignores the invalid input instead of
/// panicking, and it deliberately returns `()` so existing call sites need no
/// `Result` handling.
///
/// # Process-global, single-factor semantics
///
/// The factor is one **process-global** value shared by every window (see
/// [`SCALE_FACTOR`]). Callers such as the renderer set it from each window's
/// DPI, so a multi-window application currently cannot give different windows
/// different factors: the last writer wins for the whole process. This
/// limitation is only documented here; no per-window factor is provided.
///
/// # Examples
///
/// ```
/// use tessera_ui::dp::{scale_factor, set_scale_factor};
///
/// set_scale_factor(2.0);
/// assert_eq!(scale_factor(), 2.0);
/// set_scale_factor(1.5);
/// assert_eq!(scale_factor(), 1.5);
/// ```
pub fn set_scale_factor(scale_factor: f64) {
    debug_assert!(
        scale_factor.is_finite() && scale_factor > 0.0,
        "set_scale_factor called with a non-positive or non-finite value \
         ({scale_factor}); keeping the previous scale factor. Expected a \
         positive finite value."
    );
    if scale_factor.is_finite() && scale_factor > 0.0 {
        SCALE_FACTOR.store(scale_factor.to_bits(), Ordering::Relaxed);
    }
}

/// Returns the current global scale factor.
///
/// The value is `1.0` until a positive finite factor is stored via
/// [`set_scale_factor`]. Because that setter ignores non-positive or non-finite
/// input, the value returned here is always positive and finite (or the initial
/// `1.0`).
///
/// # Examples
///
/// ```
/// use tessera_ui::dp::scale_factor;
///
/// let _ = scale_factor();
/// ```
pub fn scale_factor() -> f64 {
    f64::from_bits(SCALE_FACTOR.load(Ordering::Relaxed))
}

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
/// This value is converted to physical pixels using the global [`scale_factor`]
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

impl Display for Dp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}dp", self.0)
    }
}

impl Dp {
    /// A constant representing zero density-independent pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// use tessera_ui::Dp;
    ///
    /// let zero_dp = Dp::ZERO;
    /// assert_eq!(zero_dp, Dp(0.0));
    /// ```
    pub const ZERO: Dp = Dp(0.0);

    /// Creates a new `Dp` instance with the specified value.
    ///
    /// This is a const function, allowing `Dp` values to be created at compile
    /// time.
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
    /// const BUTTON_HEIGHT: Dp = Dp(48.0);
    /// let padding = Dp(16.0);
    /// ```
    pub const fn new(value: f64) -> Self {
        Dp(value)
    }

    /// Converts this dp value to physical pixels as an `f64`.
    ///
    /// This method applies the current global scale factor to convert
    /// density-independent pixels to physical pixels. The scale factor is
    /// read from [`scale_factor`].
    ///
    /// # Returns
    ///
    /// The equivalent value in physical pixels as a 64-bit floating-point
    /// number. If the scale factor hasn't been initialized, defaults to 1.0
    /// (no scaling).
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
        let scale_factor = scale_factor();
        self.0 * scale_factor
    }

    /// Creates a `Dp` value from physical pixels specified as an `f64`.
    ///
    /// This method performs the inverse conversion of
    /// [`to_pixels_f64`](Self::to_pixels_f64), converting physical pixels
    /// back to density-independent pixels using the current global scale
    /// factor.
    ///
    /// # Arguments
    ///
    /// * `value` - The pixel value as a 64-bit floating-point number
    ///
    /// # Returns
    ///
    /// A new `Dp` instance representing the equivalent dp value.
    /// If the scale factor hasn't been initialized, defaults to 1.0 (no
    /// scaling).
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
        let scale_factor = scale_factor();
        Dp(value / scale_factor)
    }

    /// Converts this dp value to physical pixels as a `u32`.
    ///
    /// This method applies the current global scale factor and truncates the
    /// result to an unsigned 32-bit integer. This is commonly used for
    /// rendering operations that require integer pixel coordinates.
    ///
    /// # Returns
    ///
    /// The equivalent value in physical pixels as an unsigned 32-bit integer.
    /// The result is truncated (not rounded) from the floating-point
    /// calculation. If the scale factor hasn't been initialized, defaults
    /// to 1.0 (no scaling).
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
    /// Values are truncated, not rounded. For more precise control over
    /// rounding behavior, use [`to_pixels_f64`](Self::to_pixels_f64) and
    /// apply your preferred rounding method.
    pub fn to_pixels_u32(&self) -> u32 {
        let scale_factor = scale_factor();
        (self.0 * scale_factor) as u32
    }

    /// Creates a `Dp` value from physical pixels specified as a `u32`.
    ///
    /// This method converts an unsigned 32-bit integer pixel value to
    /// density-independent pixels using the current global scale factor.
    /// The integer is first converted to `f64` for the calculation.
    ///
    /// # Arguments
    ///
    /// * `value` - The pixel value as an unsigned 32-bit integer
    ///
    /// # Returns
    ///
    /// A new `Dp` instance representing the equivalent dp value.
    /// If the scale factor hasn't been initialized, defaults to 1.0 (no
    /// scaling).
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
        let scale_factor = scale_factor();
        Dp((value as f64) / scale_factor)
    }

    /// Converts this dp value to physical pixels as an `f32`.
    ///
    /// This method applies the current global scale factor and converts the
    /// result to a 32-bit floating-point number. This is commonly used for
    /// graphics APIs that work with `f32` coordinates.
    ///
    /// # Returns
    ///
    /// The equivalent value in physical pixels as a 32-bit floating-point
    /// number. If the scale factor hasn't been initialized, defaults to 1.0
    /// (no scaling).
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
    /// Converting from `f64` to `f32` may result in precision loss for very
    /// large or very precise values. For maximum precision, use
    /// [`to_pixels_f64`](Self::to_pixels_f64).
    pub fn to_pixels_f32(&self) -> f32 {
        let scale_factor = scale_factor();
        (self.0 * scale_factor) as f32
    }

    /// Creates a `Dp` value from physical pixels specified as an `f32`.
    ///
    /// This method converts a 32-bit floating-point pixel value to
    /// density-independent pixels using the current global scale factor.
    /// The `f32` value is first converted to `f64` for internal
    /// calculations.
    ///
    /// # Arguments
    ///
    /// * `value` - The pixel value as a 32-bit floating-point number
    ///
    /// # Returns
    ///
    /// A new `Dp` instance representing the equivalent dp value.
    /// If the scale factor hasn't been initialized, defaults to 1.0 (no
    /// scaling).
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
        let scale_factor = scale_factor();
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
    /// * [`to_pixels_f32`](Self::to_pixels_f32) - For direct `f32` pixel
    ///   conversion
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
    /// fn set_padding(padding: Dp) { /* ... */
    /// }
    /// set_padding(8.0.into());
    /// ```
    fn from(value: f64) -> Self {
        Dp(value)
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
    /// * [`from_pixels_f64`](Self::from_pixels_f64) - For direct pixel-to-dp
    ///   conversion
    fn from(px: Px) -> Self {
        px.to_dp()
    }
}

impl Mul<f32> for Dp {
    type Output = Dp;

    fn mul(self, rhs: f32) -> Self::Output {
        Dp(self.0 * (rhs as f64))
    }
}

impl Div<f32> for Dp {
    type Output = Dp;

    fn div(self, rhs: f32) -> Self::Output {
        Dp(self.0 / (rhs as f64))
    }
}

impl Mul<f64> for Dp {
    type Output = Dp;

    fn mul(self, rhs: f64) -> Self::Output {
        Dp(self.0 * rhs)
    }
}

impl Div<f64> for Dp {
    type Output = Dp;

    fn div(self, rhs: f64) -> Self::Output {
        Dp(self.0 / rhs)
    }
}

#[cfg(test)]
mod set_scale_factor_tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::{Mutex, MutexGuard};

    /// Serializes every test that touches the process-global [`SCALE_FACTOR`].
    ///
    /// The factor is shared by the whole test binary, so tests that mutate it
    /// must not run concurrently. The guard also restores the value that was
    /// current on entry (even if the test panics).
    static SCALE_FACTOR_TEST_LOCK: Mutex<()> = Mutex::new(());

    /// RAII guard holding the module-wide lock and restoring the original scale
    /// factor on drop.
    struct ScaleFactorGuard {
        // Held until the guard is dropped so the restore below is race-free.
        _lock: MutexGuard<'static, ()>,
        original: f64,
    }

    impl Drop for ScaleFactorGuard {
        fn drop(&mut self) {
            super::set_scale_factor(self.original);
        }
    }

    fn guard() -> ScaleFactorGuard {
        let lock = SCALE_FACTOR_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let original = scale_factor();
        ScaleFactorGuard {
            _lock: lock,
            original,
        }
    }

    /// Calls [`set_scale_factor`], containing the `debug_assert!` panic that
    /// flags invalid input in debug builds (the profile `cargo test` uses).
    fn set_scale_factor_expecting_rejection(value: f64) {
        // Silence the default panic hook for the sampled window so catching the
        // debug assertion does not print a scary backtrace into the test log.
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let caught = catch_unwind(AssertUnwindSafe(|| set_scale_factor(value)));
        std::panic::set_hook(previous_hook);

        if cfg!(debug_assertions) {
            assert!(
                caught.is_err(),
                "invalid factor {value} should trip the debug_assert!"
            );
        }
    }

    #[test]
    fn set_scale_factor_accepts_positive_finite_values() {
        let _guard = guard();
        set_scale_factor(2.5);
        assert_eq!(scale_factor(), 2.5);
        assert_eq!(Dp(10.0).to_pixels_f64(), 25.0);
    }

    #[test]
    fn set_scale_factor_rejects_nan_keeping_previous_factor() {
        let _guard = guard();
        set_scale_factor(2.0);
        set_scale_factor_expecting_rejection(f64::NAN);

        assert_eq!(scale_factor(), 2.0, "NaN must not replace the valid factor");
        assert_eq!(Dp(10.0).to_pixels_f64(), 20.0);
        assert_eq!(Dp(10.0).to_pixels_f32(), 20.0);
        assert_eq!(Dp(10.0).to_px(), Px::from_f32(20.0));
    }

    #[test]
    fn set_scale_factor_rejects_zero_keeping_previous_factor() {
        let _guard = guard();
        set_scale_factor(2.0);
        set_scale_factor_expecting_rejection(0.0);

        assert_eq!(scale_factor(), 2.0, "0.0 must not replace the valid factor");
        assert_eq!(Dp(10.0).to_pixels_f64(), 20.0);
        assert_eq!(Dp(10.0).to_px(), Px::from_f32(20.0));
    }

    #[test]
    fn set_scale_factor_rejects_negative_keeping_previous_factor() {
        let _guard = guard();
        set_scale_factor(2.0);
        set_scale_factor_expecting_rejection(-1.0);

        assert_eq!(
            scale_factor(),
            2.0,
            "-1.0 must not replace the valid factor"
        );
        assert_eq!(Dp(10.0).to_pixels_f64(), 20.0);
        assert_eq!(Dp(10.0).to_px(), Px::from_f32(20.0));
    }

    #[test]
    fn set_scale_factor_rejects_non_finite_keeping_previous_factor() {
        let _guard = guard();
        set_scale_factor(2.0);
        set_scale_factor_expecting_rejection(f64::INFINITY);
        set_scale_factor_expecting_rejection(f64::NEG_INFINITY);

        assert_eq!(
            scale_factor(),
            2.0,
            "±inf must not replace the valid factor"
        );
        assert_eq!(Dp(10.0).to_pixels_f64(), 20.0);
        assert_eq!(Dp(10.0).to_px(), Px::from_f32(20.0));
    }
}
