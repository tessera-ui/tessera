use std::sync::atomic;

/// State for managing ripple animation and hover effects
pub struct RippleState {
    pub is_animating: atomic::AtomicBool,
    pub start_time: atomic::AtomicU64, // Store as u64 millis since epoch
    pub click_pos_x: atomic::AtomicI32, // Store as fixed-point * 1000
    pub click_pos_y: atomic::AtomicI32, // Store as fixed-point * 1000
    pub is_hovered: atomic::AtomicBool, // Track hover state
}

impl Default for RippleState {
    fn default() -> Self {
        Self::new()
    }
}

impl RippleState {
    pub fn new() -> Self {
        Self {
            is_animating: atomic::AtomicBool::new(false),
            start_time: atomic::AtomicU64::new(0),
            click_pos_x: atomic::AtomicI32::new(0),
            click_pos_y: atomic::AtomicI32::new(0),
            is_hovered: atomic::AtomicBool::new(false),
        }
    }

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

    /// Set hover state
    pub fn set_hovered(&self, hovered: bool) {
        self.is_hovered.store(hovered, atomic::Ordering::SeqCst);
    }

    /// Get hover state
    pub fn is_hovered(&self) -> bool {
        self.is_hovered.load(atomic::Ordering::SeqCst)
    }
}
