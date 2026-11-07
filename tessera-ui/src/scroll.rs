//! Platform scroll delta normalization helpers.
//!
//! ## Usage
//!
//! Normalize wheel deltas before applying them inside scrollable containers.

use crate::{Dp, ScrollDeltaUnit, ScrollEventContent, ScrollEventSource};

/// Platform-specific wheel scroll normalization parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlatformScrollConfig {
    /// Density-independent distance consumed for one wheel line step.
    pub wheel_line_step: Dp,
}

impl PlatformScrollConfig {
    /// Returns the default configuration for the current compilation target.
    pub fn current() -> Self {
        // Browser line units differ from the wheel ticks supplied by desktop
        // backends. Desktop defaults match Chromium's conversion for each OS.
        let step = if cfg!(target_family = "wasm") {
            16.0
        } else if cfg!(target_os = "windows") {
            100.0
        } else if cfg!(target_os = "linux") {
            53.0
        } else {
            40.0
        };
        Self {
            wheel_line_step: Dp(step),
        }
    }

    /// Converts a raw platform scroll delta into physical scroll pixels.
    pub fn normalize_scroll_delta(
        self,
        delta_x: f32,
        delta_y: f32,
        unit: ScrollDeltaUnit,
        source: ScrollEventSource,
    ) -> (f32, f32) {
        let multiplier = match (source, unit) {
            (ScrollEventSource::Wheel, ScrollDeltaUnit::Line) => {
                self.wheel_line_step.to_pixels_f64() as f32
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

/// Converts a raw platform scroll delta into physical scroll pixels.
pub fn normalize_platform_scroll_delta(
    delta_x: f32,
    delta_y: f32,
    unit: ScrollDeltaUnit,
    source: ScrollEventSource,
) -> (f32, f32) {
    platform_scroll_config().normalize_scroll_delta(delta_x, delta_y, unit, source)
}

pub(crate) fn is_discrete_wheel(event: &ScrollEventContent) -> bool {
    event.source == ScrollEventSource::Wheel
        && (event.unit == ScrollDeltaUnit::Line || is_wheel_step(event.delta_x, event.delta_y))
}

pub(crate) fn is_discrete_platform_wheel(event: &ScrollEventContent) -> bool {
    match event.unit {
        // Browser line units and AppKit non-precise wheel ticks are discrete.
        ScrollDeltaUnit::Line if cfg!(any(target_family = "wasm", target_os = "macos")) => true,
        // Windows legacy messages and X11 may report fractional ticks for
        // high-resolution devices. Match the browser's pixel classification.
        ScrollDeltaUnit::Line => {
            let (x, y) = event.pixel_delta();
            is_wheel_step(x, y)
        }
        // Native pixel deltas already identify continuous input and may carry
        // OS-generated momentum. The Web does not expose the wheel device kind.
        ScrollDeltaUnit::Pixel => cfg!(target_family = "wasm") && is_discrete_wheel(event),
    }
}

fn is_wheel_step(x: f32, y: f32) -> bool {
    let threshold = Dp(40.0).to_pixels_f64() as f32;
    x.abs().max(y.abs()) >= threshold
}

#[cfg(test)]
mod tests {
    use super::{PlatformScrollConfig, is_discrete_platform_wheel, is_discrete_wheel};
    use crate::{Dp, ScrollDeltaUnit, ScrollEventContent, ScrollEventSource};

    #[test]
    fn wheel_line_delta_is_scaled_by_platform_step() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
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
    fn wheel_classification_uses_logical_distance_and_native_precision() {
        let session = crate::RuntimeSession::new();
        session.run(|| {
            crate::dp::set_scale_factor(2.0);
            let mut event = ScrollEventContent {
                delta_x: 0.0,
                delta_y: -70.0,
                unit: ScrollDeltaUnit::Pixel,
                source: ScrollEventSource::Wheel,
            };
            assert!(!is_discrete_wheel(&event));
            event.delta_y = -120.0;
            assert!(is_discrete_wheel(&event));
            assert_eq!(
                is_discrete_platform_wheel(&event),
                cfg!(target_family = "wasm")
            );
        });
    }

    #[test]
    fn wheel_pixel_delta_is_left_untouched() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
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
