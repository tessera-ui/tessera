//! Animation mapping for UI components.

/// Quadratic ease-in-out mapping.
/// Input: linear progress in [0.0, 1.0].
/// Output: eased progress in [0.0, 1.0].
pub(crate) fn easing(progress: f32) -> f32 {
    // Cubic ease-in-out
    let t = progress.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}
