//! Ripple state — manage ripple animation and interaction state layers.
//!
//! ## Usage
//!
//! Provide ripple and state-layer feedback for interactive controls such as
//! buttons and surfaces.

use std::time::{Duration, Instant};

use tessera_ui::{Dp, PxSize};

use crate::theme::MaterialAlpha;

/// Controls how the ripple should be drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RippleSpec {
    /// If true, the ripple originates from the pointer position and is clipped
    /// by the component bounds.
    ///
    /// If false, the ripple originates from the component center.
    pub bounded: bool,
    /// Optional explicit ripple radius. When `None`, the radius is derived from
    /// the component size.
    pub radius: Option<Dp>,
}

impl Default for RippleSpec {
    fn default() -> Self {
        Self {
            bounded: true,
            radius: None,
        }
    }
}

/// A snapshot of the currently running ripple animation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RippleAnimation {
    /// Current animation progress in `[0.0, 1.0)`.
    pub progress: f32,
    /// Ripple origin in normalized `[x, y]` coordinates (0.0..=1.0), relative
    /// to the component bounds.
    pub center: [f32; 2],
    /// Ripple radius in normalized coordinates, relative to the minimum
    /// component dimension.
    pub radius: f32,
    /// Alpha applied to the ripple wave before blending (0.0..=1.0).
    pub alpha: f32,
}

#[derive(Clone, Copy, Debug)]
struct RippleAnimationState {
    start: Instant,
    center: [f32; 2],
    max_radius: f32,
}

impl RippleAnimationState {
    fn animation_at(self, now: Instant) -> Option<RippleAnimation> {
        let elapsed = now.saturating_duration_since(self.start);
        let progress =
            (elapsed.as_secs_f32() / RippleState::ANIMATION_DURATION.as_secs_f32()).clamp(0.0, 1.0);
        if progress >= 1.0 {
            return None;
        }

        let eased = ease_out_cubic(progress);
        let radius = self.max_radius * eased;
        let alpha = MaterialAlpha::PRESSED * (1.0 - progress);

        Some(RippleAnimation {
            progress,
            center: self.center,
            radius,
            alpha,
        })
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// # RippleState
///
/// Manage ripple animations and hover state for interactive UI components.
/// Use with `remember` to create persistent state across frames.
///
/// ## Parameters
///
/// - This type has no constructor parameters; create it with
///   [`RippleState::new()`].
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::ripple_state::RippleState;
/// let mut s = RippleState::new();
/// assert!(!s.is_hovered());
/// s.set_hovered(true);
/// assert!(s.is_hovered());
/// ```
pub struct RippleState {
    animation: Option<RippleAnimationState>,
    is_hovered: bool,
    is_focused: bool,
    is_dragged: bool,
    is_pressed: bool,
}

impl Default for RippleState {
    /// Creates a new `RippleState` with all fields initialized to their default
    /// values.
    fn default() -> Self {
        Self::new()
    }
}

impl RippleState {
    /// The default duration of the ripple animation.
    pub const ANIMATION_DURATION: Duration = Duration::from_millis(300);

    /// Creates a new `RippleState` with default values.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::ripple_state::RippleState;
    /// let state = RippleState::new();
    /// ```
    pub fn new() -> Self {
        Self {
            animation: None,
            is_hovered: false,
            is_focused: false,
            is_dragged: false,
            is_pressed: false,
        }
    }

    /// Starts a new ripple animation from the given click position.
    ///
    /// # Arguments
    ///
    /// * `click_pos` - The normalized `[x, y]` position in 0.0..=1.0 where the
    ///   ripple originates.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::ripple_state::RippleState;
    /// let mut state = RippleState::new();
    /// state.start_animation([0.5, 0.5]);
    /// ```
    pub fn start_animation(&mut self, click_pos: [f32; 2]) {
        self.animation = Some(RippleAnimationState {
            start: Instant::now(),
            center: [click_pos[0].clamp(0.0, 1.0), click_pos[1].clamp(0.0, 1.0)],
            max_radius: 1.0,
        });
    }

    /// Starts a ripple animation using the provided size and spec.
    pub fn start_animation_with_spec(
        &mut self,
        click_pos: [f32; 2],
        size: PxSize,
        spec: RippleSpec,
    ) {
        let now = Instant::now();
        let size = [size.width.to_f32(), size.height.to_f32()];
        let center = if spec.bounded {
            [click_pos[0].clamp(0.0, 1.0), click_pos[1].clamp(0.0, 1.0)]
        } else {
            [0.5, 0.5]
        };

        let min_dimension = size[0].min(size[1]).max(1.0);
        let max_radius = if let Some(radius) = spec.radius {
            radius.to_pixels_f32() / min_dimension
        } else {
            max_distance_to_corners(center, size) / min_dimension
        };

        self.animation = Some(RippleAnimationState {
            start: now,
            center,
            max_radius,
        });
    }

    /// Marks the ripple as no longer pressed.
    pub fn release(&mut self) {
        self.set_pressed(false);
    }

    /// Sets whether the component is pressed.
    pub fn set_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
    }

    /// Returns the current ripple animation snapshot.
    pub fn animation(&mut self) -> Option<RippleAnimation> {
        self.animation_at(Instant::now())
    }

    /// Returns the current ripple animation snapshot at `now`.
    pub fn animation_at(&mut self, now: Instant) -> Option<RippleAnimation> {
        let state = self.animation?;
        match state.animation_at(now) {
            Some(animation) => Some(animation),
            None => {
                self.animation = None;
                None
            }
        }
    }

    /// Returns the state-layer alpha derived from the current interactions.
    pub fn state_layer_alpha(&self) -> f32 {
        if self.is_dragged {
            MaterialAlpha::DRAGGED
        } else if self.is_pressed {
            MaterialAlpha::PRESSED
        } else if self.is_focused {
            MaterialAlpha::FOCUSED
        } else if self.is_hovered {
            MaterialAlpha::HOVER
        } else {
            0.0
        }
    }

    /// Sets whether the component is hovered.
    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    /// Returns whether the component is hovered.
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    /// Sets whether the component is focused.
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Returns whether the component is focused.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Sets whether the component is dragged.
    pub fn set_dragged(&mut self, dragged: bool) {
        self.is_dragged = dragged;
    }

    /// Returns whether the component is dragged.
    pub fn is_dragged(&self) -> bool {
        self.is_dragged
    }

    /// Returns whether the component is pressed.
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns the current progress of the ripple animation and the origin
    /// position.
    ///
    /// Returns `Some((progress, [x, y]))` if the animation is active, where:
    /// - `progress` is a value in `[0.0, 1.0)` representing the animation
    ///   progress.
    /// - `[x, y]` is the normalized origin of the ripple in 0.0..=1.0.
    ///
    /// Returns `None` if the animation is not active or has completed.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::ripple_state::RippleState;
    /// let mut state = RippleState::new();
    /// state.start_animation([0.5, 0.5]);
    /// if let Some((progress, center)) = state.get_animation_progress() {
    ///     // Use progress and center for rendering
    /// }
    /// ```
    pub fn get_animation_progress(&mut self) -> Option<(f32, [f32; 2])> {
        self.animation()
            .map(|animation| (animation.progress, animation.center))
    }
}

fn max_distance_to_corners(center: [f32; 2], size: [f32; 2]) -> f32 {
    let origin = [center[0] * size[0], center[1] * size[1]];
    let corners = [
        [0.0, 0.0],
        [size[0], 0.0],
        [size[0], size[1]],
        [0.0, size[1]],
    ];

    corners
        .into_iter()
        .map(|corner| {
            let dx = corner[0] - origin[0];
            let dy = corner[1] - origin[1];
            (dx * dx + dy * dy).sqrt()
        })
        .fold(0.0_f32, f32::max)
}
