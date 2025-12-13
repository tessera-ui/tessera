//! Ripple state — manage ripple animation and hover state for interactive
//! components.
//!
//! ## Usage
//! Provide visual ripple feedback for interactive controls in your app
//! (buttons, surfaces, glass buttons) to indicate clicks and hover
//! interactions.

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
    /// Whether the ripple animation is currently active.
    is_animating: bool,
    /// The animation start time, stored as milliseconds since the Unix epoch.
    start_time_ms: u64,
    /// The normalized origin of the ripple.
    click_pos: [f32; 2],
    /// Whether the pointer is currently hovering over the component.
    is_hovered: bool,
}

impl Default for RippleState {
    /// Creates a new `RippleState` with all fields initialized to their default
    /// values.
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
            is_animating: false,
            start_time_ms: 0,
            click_pos: [0.0, 0.0],
            is_hovered: false,
        }
    }

    /// Starts a new ripple animation from the given click position.
    ///
    /// # Arguments
    ///
    /// * `click_pos` - The normalized `[x, y]` position (typically in the range
    ///   [0.0, 1.0]) where the ripple originates.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::ripple_state::RippleState;
    /// let mut state = RippleState::new();
    /// state.start_animation([0.5, 0.5]);
    /// ```
    pub fn start_animation(&mut self, click_pos: [f32; 2]) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time earlier than UNIX_EPOCH")
            .as_millis() as u64;

        self.start_time_ms = now;
        self.click_pos = click_pos;
        self.is_animating = true;
    }

    /// Returns the current progress of the ripple animation and the origin
    /// position.
    ///
    /// Returns `Some((progress, [x, y]))` if the animation is active, where:
    /// - `progress` is a value in `[0.0, 1.0)` representing the animation
    ///   progress.
    /// - `[x, y]` is the normalized origin of the ripple.
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
        if !self.is_animating {
            return None;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time earlier than UNIX_EPOCH")
            .as_millis() as u64;
        let elapsed_ms = now.saturating_sub(self.start_time_ms);
        let progress = (elapsed_ms as f32) / 600.0; // 600ms animation

        if progress >= 1.0 {
            self.is_animating = false;
            return None;
        }

        Some((progress, self.click_pos))
    }

    /// Sets the hover state for the ripple.
    ///
    /// # Arguments
    ///
    /// * `hovered` - `true` if the pointer is over the component, `false`
    ///   otherwise.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::ripple_state::RippleState;
    /// let mut state = RippleState::new();
    /// state.set_hovered(true);
    /// ```
    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
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
        self.is_hovered
    }
}
