use std::sync::OnceLock;

use parking_lot::RwLock;

use crate::Px;

pub static SCALE_FACTOR: OnceLock<RwLock<f64>> = OnceLock::new();

/// Density-independent pixels (dp) for UI scaling.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Dp(pub f64);

impl Dp {
    /// Creates a new `Dp` instance.
    pub const fn new(value: f64) -> Self {
        Dp(value)
    }

    /// Returns the value in pixels.
    pub fn to_pixels_f64(&self) -> f64 {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        self.0 * scale_factor
    }

    /// Get dp from pixels in f64
    pub fn from_pixels_f64(value: f64) -> Self {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        Dp(value / scale_factor)
    }

    /// Returns the value in pixels as u32.
    pub fn to_pixels_u32(&self) -> u32 {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        (self.0 * scale_factor) as u32
    }

    /// Get dp from pixels in u32
    pub fn from_pixels_u32(value: u32) -> Self {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        Dp((value as f64) / scale_factor)
    }

    /// Returns the value in pixels as f32.
    pub fn to_pixels_f32(&self) -> f32 {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        (self.0 * scale_factor) as f32
    }

    /// Get dp from pixels in f32
    pub fn from_pixels_f32(value: f32) -> Self {
        let scale_factor = SCALE_FACTOR.get().map(|lock| *lock.read()).unwrap_or(1.0);
        Dp((value as f64) / scale_factor)
    }
}

impl From<f64> for Dp {
    fn from(value: f64) -> Self {
        Dp::new(value)
    }
}

impl From<Px> for Dp {
    fn from(px: Px) -> Self {
        Dp::from_pixels_f64(px.to_dp().0)
    }
}
