//! This module defines the [`RippleState`] struct, which manages the state for ripple animations in interactive UI components.
//!
//! Currently, two foundational components use it to display ripple animations: [`crate::surface::surface`] and [`crate::fluid_glass::fluid_glass`].
//!
//! Other components composed from those, such as [`crate::button::button`], also leverage it to provide ripple effects.

use std::sync::atomic;

/// `RippleState` manages the animation and hover state for ripple effects in interactive UI components.
/// It is designed to be shared across components using `Arc<RippleState>`, enabling coordinated animation and hover feedback.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use tessera_ui_basic_components::ripple_state::RippleState;
///
/// // Create a new ripple state and share it with a button or surface
/// let ripple_state = Arc::new(RippleState::new());
///
/// // Start a ripple animation at a given position (e.g., on mouse click)
/// ripple_state.start_animation([0.5, 0.5]);
///
/// // In your component's render or animation loop:
/// if let Some((progress, center)) = ripple_state.get_animation_progress() {
///     // Use progress (0.0..1.0) and center ([f32; 2]) to drive the ripple effect
/// }
///
/// // Set hover state on pointer enter/leave
/// ripple_state.set_hovered(true);
/// ```
pub struct RippleState {
    /// Whether the ripple animation is currently active.
    pub is_animating: atomic::AtomicBool,
    /// The animation start time, stored as milliseconds since the Unix epoch.
    pub start_time: atomic::AtomicU64,
    /// The X coordinate of the click position, stored as fixed-point (multiplied by 1000).
    pub click_pos_x: atomic::AtomicI32,
    /// The Y coordinate of the click position, stored as fixed-point (multiplied by 1000).
    pub click_pos_y: atomic::AtomicI32,
    /// Whether the pointer is currently hovering over the component.
    pub is_hovered: atomic::AtomicBool,
}

impl Default for RippleState {
    /// Creates a new `RippleState` with all fields initialized to their default values.
    fn default() -> Self {
        Self::new()
    }
}

impl RippleState {
    /// Creates a new `RippleState` with default values.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::ripple_state::RippleState;
    /// let state = RippleState::new();
    /// ```
    pub fn new() -> Self {
        Self {
            is_animating: atomic::AtomicBool::new(false),
            start_time: atomic::AtomicU64::new(0),
            click_pos_x: atomic::AtomicI32::new(0),
            click_pos_y: atomic::AtomicI32::new(0),
            is_hovered: atomic::AtomicBool::new(false),
        }
    }

    /// Starts a new ripple animation from the given click position.
    ///
    /// # Arguments
    ///
    /// * `click_pos` - The normalized `[x, y]` position (typically in the range [0.0, 1.0]) where the ripple originates.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::ripple_state::RippleState;
    /// let state = RippleState::new();
    /// state.start_animation([0.5, 0.5]);
    /// ```
    pub fn start_animation(&self, click_pos: [f32; 2]) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.start_time.store(now, atomic::Ordering::SeqCst);
        self.click_pos_x
            .store((click_pos[0] * 1000.0) as i32, atomic::Ordering::SeqCst);
        self.click_pos_y
            .store((click_pos[1] * 1000.0) as i32, atomic::Ordering::SeqCst);
        self.is_animating.store(true, atomic::Ordering::SeqCst);
    }

    /// Returns the current progress of the ripple animation and the origin position.
    ///
    /// Returns `Some((progress, [x, y]))` if the animation is active, where:
    /// - `progress` is a value in `[0.0, 1.0)` representing the animation progress.
    /// - `[x, y]` is the normalized origin of the ripple.
    ///
    /// Returns `None` if the animation is not active or has completed.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::ripple_state::RippleState;
    /// let state = RippleState::new();
    /// state.start_animation([0.5, 0.5]);
    /// if let Some((progress, center)) = state.get_animation_progress() {
    ///     // Use progress and center for rendering
    /// }
    /// ```
    pub fn get_animation_progress(&self) -> Option<(f32, [f32; 2])> {
        let is_animating = self.is_animating.load(atomic::Ordering::SeqCst);

        if !is_animating {
            return None;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let start = self.start_time.load(atomic::Ordering::SeqCst);
        let elapsed_ms = now.saturating_sub(start);
        let progress = (elapsed_ms as f32) / 600.0; // 600ms animation

        if progress >= 1.0 {
            self.is_animating.store(false, atomic::Ordering::SeqCst);
            return None;
        }

        let click_pos = [
            self.click_pos_x.load(atomic::Ordering::SeqCst) as f32 / 1000.0,
            self.click_pos_y.load(atomic::Ordering::SeqCst) as f32 / 1000.0,
        ];

        Some((progress, click_pos))
    }

    /// Sets the hover state for the ripple.
    ///
    /// # Arguments
    ///
    /// * `hovered` - `true` if the pointer is over the component, `false` otherwise.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::ripple_state::RippleState;
    /// let state = RippleState::new();
    /// state.set_hovered(true);
    /// ```
    pub fn set_hovered(&self, hovered: bool) {
        self.is_hovered.store(hovered, atomic::Ordering::SeqCst);
    }

    /// Returns whether the pointer is currently hovering over the component.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::ripple_state::RippleState;
    /// let state = RippleState::new();
    /// let hovered = state.is_hovered();
    /// ```
    pub fn is_hovered(&self) -> bool {
        self.is_hovered.load(atomic::Ordering::SeqCst)
    }
}
