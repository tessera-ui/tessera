use std::{
    sync::{Arc, atomic},
    time::Instant,
};

use parking_lot::RwLock;
use tessera_ui::tessera;
use tessera_ui_basic_components::text::text;

/// Runtime metrics used by the performance display component.
///
/// This struct is intended to be shared across the application (hence the
/// atomic and lock usage). Fields are intentionally public to allow external
/// systems to update metrics when frames are rendered.
pub struct PerformanceMetrics {
    /// Current FPS value (frames per second), updated periodically.
    pub fps: atomic::AtomicU64,
    /// Instant of the last frame. Protected by a RwLock for occasional writes.
    pub last_frame: RwLock<Instant>,
    /// Instant when FPS was last computed. Protected by a RwLock.
    pub last_fps_update_time: RwLock<Instant>,
    /// Number of frames counted since the last FPS update.
    pub frames_since_last_update: atomic::AtomicU64,
}

impl PerformanceMetrics {
    /// Create a new PerformanceMetrics instance with sensible defaults.
    pub fn new() -> Self {
        Self {
            fps: atomic::AtomicU64::new(0),
            last_frame: RwLock::new(Instant::now()),
            last_fps_update_time: RwLock::new(Instant::now()),
            frames_since_last_update: atomic::AtomicU64::new(0),
        }
    }
}

/// Simple performance display component.
///
/// This component renders the current FPS value as text and uses a lightweight
/// state handler to update timing metrics on each frame tick. It intentionally
/// avoids heavy work in the handler and uses atomics and locks for thread-safe
/// updates.
#[tessera]
pub fn perf_display(metrics: Arc<PerformanceMetrics>) {
    // Display the currently stored FPS value.
    text(format!(
        "FPS: {}",
        metrics.fps.load(atomic::Ordering::SeqCst)
    ));

    // State handler executes on frame ticks; update timing counters here.
    state_handler(Box::new(move |_| {
        let now = Instant::now();

        // Update last_frame timestamp (write lock, occasional use).
        let mut last_frame_guard = metrics.last_frame.write();
        *last_frame_guard = now;

        // Increment frame counter atomically.
        metrics
            .frames_since_last_update
            .fetch_add(1, atomic::Ordering::SeqCst);

        // Compute elapsed time since last FPS update and recalculate FPS if enough time passed.
        let mut last_fps_update_time_guard = metrics.last_fps_update_time.write();
        let elapsed_ms = now.duration_since(*last_fps_update_time_guard).as_millis();

        // Update FPS at most every 100 ms to avoid noisy updates.
        if elapsed_ms >= 100 {
            let frame_count = metrics
                .frames_since_last_update
                .swap(0, atomic::Ordering::SeqCst);
            let new_fps = (frame_count as f64 / (elapsed_ms as f64 / 1000.0)) as u64;
            metrics.fps.store(new_fps, atomic::Ordering::SeqCst);
            *last_fps_update_time_guard = now;
        }
    }));
}
