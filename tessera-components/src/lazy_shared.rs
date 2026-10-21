//! Shared measurements and viewport bounds for lazy scrolling components.
//!
//! ## Usage
//!
//! Display long lists and grids in resizable windows on different displays.

use tessera_ui::Px;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeasuredMain {
    pixels: Px,
    logical: f64,
}

impl MeasuredMain {
    pub(crate) fn new(pixels: Px, density: f64) -> Self {
        Self {
            pixels,
            logical: f64::from(pixels.0) / density,
        }
    }

    pub(crate) fn pixels(self) -> Px {
        self.pixels
    }

    pub(crate) fn rebase(&mut self, density: f64) {
        self.pixels = Px((self.logical * density) as i32);
    }
}

/// Clamps the main-axis size a lazy component reports to its parent.
///
/// The reported size never exceeds the viewport, and is further limited by the
/// parent constraint for the main axis, falling back to `fallback_limit` when
/// the parent does not constrain that axis.
pub(crate) fn clamp_reported_main(
    total_main: Px,
    viewport_span: Px,
    constraint_max: Option<Px>,
    fallback_limit: Option<Px>,
) -> Px {
    let viewport = viewport_span.max(Px::ZERO);
    let mut report = total_main.min(viewport);
    if let Some(max_value) = constraint_max.or(fallback_limit) {
        report = report.min(max_value.max(Px::ZERO));
    }
    report
}
