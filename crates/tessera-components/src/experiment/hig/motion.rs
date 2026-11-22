use std::time::Duration;

use tessera_ui::{Color, Dp, State, time::Instant};

use crate::{
    animation::{Motion, animate_to, cubic_bezier_easing},
    shape_def::{RoundedCorner, Shape},
};

#[cfg(test)]
#[path = "motion_tests.rs"]
mod tests;

/// Duration of a selection change in checkboxes, radio buttons, and segments.
pub(super) const SELECTION: Duration = Duration::from_millis(150);
/// Duration of a disclosure triangle turning a quarter.
pub(super) const DISCLOSURE: Duration = Duration::from_millis(200);
/// Duration of a keyboard focus ring fading in.
pub(super) const FOCUS_RING: Duration = Duration::from_millis(200);

/// A value eased between targets; an interrupted change continues from the current value and
/// takes time in proportion to the remaining distance.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Eased {
    value: f32,
    from: f32,
    to: f32,
    start: Option<Instant>,
    duration: Duration,
}

impl Eased {
    pub(super) fn new(value: f32, duration: Duration) -> Self {
        Self {
            value,
            from: value,
            to: value,
            start: None,
            duration,
        }
    }

    pub(super) fn value(&self) -> f32 {
        self.value
    }
}

impl Motion for Eased {
    type Target = f32;

    fn is_settled_at(&self, target: &f32) -> bool {
        self.start.is_none() && self.value == *target
    }

    fn retarget(&mut self, target: f32, now: Instant) {
        self.advance(now);
        self.from = self.value;
        self.to = target;
        self.start = Some(now);
    }

    fn advance(&mut self, now: Instant) -> bool {
        let Some(start) = self.start else {
            return false;
        };
        let span = self.duration.as_secs_f32() * (self.to - self.from).abs().min(1.0);
        let progress = if span > 0.0 {
            now.saturating_duration_since(start).as_secs_f32() / span
        } else {
            1.0
        };
        if progress >= 1.0 {
            self.value = self.to;
            self.start = None;
        } else {
            let eased = cubic_bezier_easing(progress, 0.42, 0.0, 0.58, 1.0);
            self.value = self.from + (self.to - self.from) * eased;
        }
        self.start.is_some()
    }
}

/// Blends two colors component-wise.
pub(super) fn mix(from: Color, to: Color, fraction: f32) -> Color {
    let lerp = |a: f32, b: f32| a + (b - a) * fraction;
    Color::new(
        lerp(from.r, to.r),
        lerp(from.g, to.g),
        lerp(from.b, to.b),
        lerp(from.a, to.a),
    )
}

/// Fades a focus ring in when focus arrives, and resets it once focus leaves. Callers hide
/// the ring at once when unfocused.
pub(super) async fn focus_ring(ring: State<Eased>, focused: bool) {
    if focused {
        animate_to(ring, 1.0).await;
    } else if ring.is_alive() && ring.with(Eased::value) > 0.0 {
        ring.set(Eased::new(0.0, FOCUS_RING));
    }
}

/// Color of a focus ring at `fade` of its full opacity.
pub(super) fn focus_ring_color(fade: f32) -> Color {
    Color::from_rgb(0.0, 0.19461783, 1.0).with_alpha(0.6 * fade)
}

/// A rounded rectangle with continuous corners of `radius`.
pub(super) fn ring_shape(radius: Dp) -> Shape {
    let corner = RoundedCorner::manual(radius, 2.0);
    Shape::RoundedRectangle {
        top_left: corner,
        top_right: corner,
        bottom_right: corner,
        bottom_left: corner,
    }
}
