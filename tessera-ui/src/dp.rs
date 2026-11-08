//! Density-independent layout units.
//!
//! ## Usage
//!
//! Size public UI in Dp; conversions use the active runtime session density.

use std::{
    fmt::Display,
    ops::{Div, Mul},
};

use crate::{Px, RuntimeSession};

/// Sets the positive finite density scale for the active UI session.
///
/// # Examples
///
/// ```
/// use tessera_ui::{RuntimeSession, dp::{scale_factor, set_scale_factor}};
/// let session = RuntimeSession::new();
/// session.run(|| {
///     set_scale_factor(2.0);
///     assert_eq!(scale_factor(), 2.0);
/// });
/// ```
pub fn set_scale_factor(scale_factor: f64) {
    debug_assert!(
        scale_factor.is_finite() && scale_factor > 0.0,
        "set_scale_factor called with a non-positive or non-finite value \
         ({scale_factor}); keeping the previous scale factor. Expected a \
         positive finite value."
    );
    if scale_factor.is_finite() && scale_factor > 0.0 {
        RuntimeSession::current()
            .expect("density changes require an active UI session")
            .set_density(scale_factor);
    }
}

/// Returns the active session density, or 1.0 outside a session.
pub fn scale_factor() -> f64 {
    RuntimeSession::current().map_or(1.0, |session| session.density())
}

/// Density-independent layout units, converted using the active session's scale.
///
/// ```
/// use tessera_ui::{Dp, RuntimeSession, dp::set_scale_factor};
/// let session = RuntimeSession::new();
/// let _scope = session.enter();
/// set_scale_factor(1.5);
/// assert_eq!(Dp(8.0).to_pixels_f64(), 12.0);
/// assert_eq!(Dp::from_pixels_f64(12.0), Dp(8.0));
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
    pub const ZERO: Dp = Dp(0.0);

    /// Creates a new `Dp` instance with the specified value.
    pub const fn new(value: f64) -> Self {
        Dp(value)
    }

    /// Scales to physical pixels using the active session's density.
    pub fn to_pixels_f64(&self) -> f64 {
        let scale_factor = scale_factor();
        self.0 * scale_factor
    }

    /// Converts physical pixels using the active session's density.
    pub fn from_pixels_f64(value: f64) -> Self {
        let scale_factor = scale_factor();
        Dp(value / scale_factor)
    }

    /// Scales to physical pixels, truncating toward zero and saturating to `u32`.
    pub fn to_pixels_u32(&self) -> u32 {
        let scale_factor = scale_factor();
        (self.0 * scale_factor) as u32
    }

    /// Converts physical pixels using the active session's density.
    pub fn from_pixels_u32(value: u32) -> Self {
        let scale_factor = scale_factor();
        Dp((value as f64) / scale_factor)
    }

    /// Scales to physical pixels with `f32` precision.
    pub fn to_pixels_f32(&self) -> f32 {
        let scale_factor = scale_factor();
        (self.0 * scale_factor) as f32
    }

    /// Converts physical pixels using the active session's density.
    pub fn from_pixels_f32(value: f32) -> Self {
        let scale_factor = scale_factor();
        Dp((value as f64) / scale_factor)
    }

    /// Scales to whole physical pixels, truncating the fractional part.
    pub fn to_px(&self) -> Px {
        Px::from_f32(self.to_pixels_f32())
    }
}

impl From<f64> for Dp {
    /// Creates a `Dp` instance from an `f64` value.
    fn from(value: f64) -> Self {
        Dp(value)
    }
}

impl From<Px> for Dp {
    /// Creates a `Dp` instance from a `Px` (physical pixels) value.
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
    use std::panic::catch_unwind;

    use super::*;

    #[test]
    fn density_rejects_invalid_values_without_replacing_the_last_valid_scale() {
        let session = crate::RuntimeSession::new();
        let _scope = session.enter();
        set_scale_factor(2.5);
        assert_eq!(scale_factor(), 2.5);
        assert_eq!(Dp(10.0).to_pixels_f64(), 25.0);
        for invalid in [f64::NAN, 0.0, -1.0, f64::INFINITY, f64::NEG_INFINITY] {
            let rejected = catch_unwind(|| set_scale_factor(invalid));
            assert_eq!(rejected.is_err(), cfg!(debug_assertions));
            assert_eq!(scale_factor(), 2.5);
            assert_eq!(Dp(10.0).to_pixels_f64(), 25.0);
            assert_eq!(Dp(10.0).to_pixels_f32(), 25.0);
            assert_eq!(Dp(10.0).to_px(), Px(25));
        }
    }
}
