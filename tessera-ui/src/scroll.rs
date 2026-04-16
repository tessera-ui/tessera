//! Platform scroll delta normalization helpers.
//!
//! ## Usage
//!
//! Normalize wheel deltas before applying them inside scrollable containers.

use crate::{Dp, Px, ScrollDeltaUnit, ScrollEventSource};

/// Platform-specific wheel scroll normalization parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlatformScrollConfig {
    /// Density-independent distance consumed for one wheel line step.
    pub wheel_line_step: Dp,
}

impl PlatformScrollConfig {
    /// Returns the default configuration for the current compilation target.
    pub fn current() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                wheel_line_step: Dp(16.0),
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                wheel_line_step: Dp(40.0),
            }
        }
    }

    /// Converts a raw platform scroll delta into logical scroll pixels.
    pub fn normalize_scroll_delta(
        self,
        delta_x: f32,
        delta_y: f32,
        unit: ScrollDeltaUnit,
        source: ScrollEventSource,
    ) -> (f32, f32) {
        let multiplier = match (source, unit) {
            (ScrollEventSource::Wheel, ScrollDeltaUnit::Line) => {
                Px::from(self.wheel_line_step).to_f32()
            }
            _ => 1.0,
        };
        (delta_x * multiplier, delta_y * multiplier)
    }
}

impl Default for PlatformScrollConfig {
    fn default() -> Self {
        Self::current()
    }
}

/// Returns the platform scroll configuration for the current target.
pub fn platform_scroll_config() -> PlatformScrollConfig {
    PlatformScrollConfig::current()
}

/// Converts a raw platform scroll delta into logical scroll pixels.
pub fn normalize_platform_scroll_delta(
    delta_x: f32,
    delta_y: f32,
    unit: ScrollDeltaUnit,
    source: ScrollEventSource,
) -> (f32, f32) {
    platform_scroll_config().normalize_scroll_delta(delta_x, delta_y, unit, source)
}

#[cfg(test)]
mod tests {
    use super::PlatformScrollConfig;
    use crate::{Dp, ScrollDeltaUnit, ScrollEventSource};

    #[test]
    fn wheel_line_delta_is_scaled_by_platform_step() {
        let config = PlatformScrollConfig {
            wheel_line_step: Dp(24.0),
        };
        let (x, y) = config.normalize_scroll_delta(
            1.0,
            -2.0,
            ScrollDeltaUnit::Line,
            ScrollEventSource::Wheel,
        );
        assert_eq!((x, y), (24.0, -48.0));
    }

    #[test]
    fn wheel_pixel_delta_is_left_untouched() {
        let config = PlatformScrollConfig {
            wheel_line_step: Dp(24.0),
        };
        let (x, y) = config.normalize_scroll_delta(
            3.0,
            -5.0,
            ScrollDeltaUnit::Pixel,
            ScrollEventSource::Wheel,
        );
        assert_eq!((x, y), (3.0, -5.0));
    }
}
