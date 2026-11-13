//! Ripple state — manage ripple animation and interaction state layers.
//!
//! ## Usage
//!
//! Provide ripple and state-layer feedback for interactive controls such as
//! buttons and surfaces.

use std::time::Duration;

use tessera_ui::{Dp, PxSize, time::Instant};

use crate::md3::theme::MaterialAlpha;

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

#[derive(Clone, PartialEq, Copy, Debug)]
struct RippleAnimationState {
    start: Instant,
    progress: f32,
    center: [f32; 2],
    max_radius: f32,
}

impl RippleAnimationState {
    fn new(center: [f32; 2], max_radius: f32, start: Instant) -> Self {
        Self {
            start,
            progress: 0.0,
            center,
            max_radius,
        }
    }

    fn snapshot(self) -> RippleAnimation {
        let progress = self.progress;
        let eased = ease_out_cubic(progress);
        let radius = self.max_radius * eased;
        let alpha = MaterialAlpha::PRESSED * (1.0 - progress);

        RippleAnimation {
            progress,
            center: self.center,
            radius,
            alpha,
        }
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
    /// use tessera_components::md3::ripple_state::RippleState;
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
    /// * `timestamp` - When the triggering input event occurred; the ripple
    ///   progresses from this time.
    ///
    /// # Example
    /// ```
    /// use tessera_components::md3::ripple_state::RippleState;
    /// let mut state = RippleState::new();
    /// state.start_animation([0.5, 0.5], tessera_ui::time::Instant::now());
    /// ```
    pub fn start_animation(&mut self, click_pos: [f32; 2], timestamp: Instant) {
        self.animation = Some(RippleAnimationState::new(
            [click_pos[0].clamp(0.0, 1.0), click_pos[1].clamp(0.0, 1.0)],
            1.0,
            timestamp,
        ));
    }

    /// Starts a ripple animation using the provided size and spec at the input
    /// event `timestamp`.
    pub fn start_animation_with_spec(
        &mut self,
        click_pos: [f32; 2],
        size: PxSize,
        spec: RippleSpec,
        timestamp: Instant,
    ) {
        let size = [size.width.to_f32(), size.height.to_f32()];
        let center = if spec.bounded { click_pos } else { [0.5, 0.5] };

        let min_dimension = size[0].min(size[1]).max(1.0);
        let max_radius = if let Some(radius) = spec.radius {
            radius.to_pixels_f32() / min_dimension
        } else {
            max_distance_to_corners(center, size) / min_dimension
        };

        self.animation = Some(RippleAnimationState::new(center, max_radius, timestamp));
    }

    /// Marks the ripple as no longer pressed.
    pub fn release(&mut self) {
        self.set_pressed(false);
    }

    /// Sets whether the component is pressed.
    pub fn set_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
    }

    // Drops a ripple whose frame task stops with its removed owner.
    pub(crate) fn cancel_animation(&mut self) {
        self.animation = None;
    }

    /// Returns the running ripple animation, if any.
    pub fn animation(&self) -> Option<RippleAnimation> {
        self.animation.map(RippleAnimationState::snapshot)
    }

    /// Advances the running ripple to the frame time `now` and returns
    /// whether it is still running.
    pub fn advance(&mut self, now: Instant) -> bool {
        let Some(state) = &mut self.animation else {
            return false;
        };
        let elapsed = now.saturating_duration_since(state.start);
        state.progress = elapsed.as_secs_f32() / Self::ANIMATION_DURATION.as_secs_f32();
        if state.progress >= 1.0 {
            self.animation = None;
            return false;
        }
        true
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
    ///
    /// ```
    /// use tessera_components::md3::ripple_state::RippleState;
    /// let mut state = RippleState::new();
    /// state.start_animation([0.5, 0.5], tessera_ui::time::Instant::now());
    /// let (progress, center) = state.get_animation_progress().expect("active ripple");
    /// assert_eq!(progress, 0.0);
    /// assert_eq!(center, [0.5, 0.5]);
    /// ```
    pub fn get_animation_progress(&self) -> Option<(f32, [f32; 2])> {
        self.animation()
            .map(|animation| (animation.progress, animation.center))
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
