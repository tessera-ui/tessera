//! Animation mapping for UI components.
///
/// Cubic ease-in-out mapping (smooth start and end).
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

/// Calculates the spring animation value based on physics parameters.
///
/// # Parameters
///
/// - `progress`: Linear time progress in [0.0, 1.0].
/// - `stiffness`: Controls speed. Try **15.0** for UI.
/// - `damping`: Controls bounciness [0.0, 1.0). Try **0.5**.
///   - Lower (0.2) = Very bouncy (oscillation).
///   - Higher (0.9) = Little to no bounce.
///
/// # Returns
///
/// A value that starts at 0.0, overshoots 1.0, and settles at 1.0.
pub fn spring(progress: f32, stiffness: f32, damping: f32) -> f32 {
    let t = progress.clamp(0.0, 1.0);

    // Boundary checks to avoid expensive math at start/end
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }

    // Ensure damping is < 1.0 to guarantee the "bounce" math works.
    // If it is >= 1.0, it becomes critically damped (no overshoot),
    // which requires a different formula. We clamp to 0.999 here.
    let zeta = damping.clamp(0.0, 0.999);

    // Natural Angular Frequency (Speed)
    let omega = stiffness;

    // Damped Angular Frequency
    // ωd = ωn * sqrt(1 - ζ^2)
    let omega_d = omega * (1.0 - zeta * zeta).sqrt();

    // Decay constant
    // decay = ζ * ωn
    let decay = zeta * omega;

    // Calculate oscillation
    // f(t) = 1 - e^(-decay * t) * (cos(ωd * t) + (sin(ωd * t) * decay / ωd))
    let oscillation = (omega_d * t).cos() + (omega_d * t).sin() * (decay / omega_d);

    1.0 - (-decay * t).exp() * oscillation
}
