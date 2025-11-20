//! Ripple state — manage ripple animation and hover state for interactive components.
//!
//! ## Usage
//! Provide visual ripple feedback for interactive controls in your app (buttons, surfaces, glass buttons) to indicate clicks and hover interactions.

use std::sync::{Arc, atomic};

/// # RippleState
///
/// Manage ripple animations and hover state for interactive UI components.
/// Recommended use: create a single `RippleState` handle and clone it to share.
///
/// ## Parameters
///
/// - This type has no constructor parameters; create it with [`RippleState::new()`].
///
/// ## Examples
///
/// ```
/// use tessera_ui_basic_components::ripple_state::RippleState;
/// let s = RippleState::new();
/// assert!(!s.is_hovered());
/// s.set_hovered(true);
/// assert!(s.is_hovered());
/// ```
#[derive(Clone)]
pub struct RippleState {
    inner: Arc<RippleStateInner>,
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
            inner: Arc::new(RippleStateInner::new()),
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
        self.inner.start_animation(click_pos);
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
        self.inner.get_animation_progress()
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
        self.inner.set_hovered(hovered);
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
        self.inner.is_hovered()
    }
}

struct RippleStateInner {
    /// Whether the ripple animation is currently active.
    is_animating: atomic::AtomicBool,
    /// The animation start time, stored as milliseconds since the Unix epoch.
    start_time: atomic::AtomicU64,
    /// The X coordinate of the click position, stored as fixed-point (multiplied by 1000).
    click_pos_x: atomic::AtomicI32,
    /// The Y coordinate of the click position, stored as fixed-point (multiplied by 1000).
    click_pos_y: atomic::AtomicI32,
    /// Whether the pointer is currently hovering over the component.
    is_hovered: atomic::AtomicBool,
}

impl RippleStateInner {
    fn new() -> Self {
        Self {
            is_animating: atomic::AtomicBool::new(false),
            start_time: atomic::AtomicU64::new(0),
            click_pos_x: atomic::AtomicI32::new(0),
            click_pos_y: atomic::AtomicI32::new(0),
            is_hovered: atomic::AtomicBool::new(false),
        }
    }

    fn start_animation(&self, click_pos: [f32; 2]) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time earlier than UNIX_EPOCH")
            .as_millis() as u64;

        self.start_time.store(now, atomic::Ordering::SeqCst);
        self.click_pos_x
            .store((click_pos[0] * 1000.0) as i32, atomic::Ordering::SeqCst);
        self.click_pos_y
            .store((click_pos[1] * 1000.0) as i32, atomic::Ordering::SeqCst);
        self.is_animating.store(true, atomic::Ordering::SeqCst);
    }

    fn get_animation_progress(&self) -> Option<(f32, [f32; 2])> {
        let is_animating = self.is_animating.load(atomic::Ordering::SeqCst);

        if !is_animating {
            return None;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time earlier than UNIX_EPOCH")
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

    fn set_hovered(&self, hovered: bool) {
        self.is_hovered.store(hovered, atomic::Ordering::SeqCst);
    }

    fn is_hovered(&self) -> bool {
        self.is_hovered.load(atomic::Ordering::SeqCst)
    }
}

