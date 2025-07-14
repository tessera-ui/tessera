use std::{
    sync::{Arc, atomic},
    time::Instant,
};

use parking_lot::RwLock;
use tessera_basic_components::text::text;
use tessera_macros::tessera;

pub struct PerformanceMetrics {
    pub fps: atomic::AtomicU64,
    pub last_frame: RwLock<Instant>,
    pub last_fps_update_time: RwLock<Instant>,
    pub frames_since_last_update: atomic::AtomicU64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            fps: atomic::AtomicU64::new(0),
            last_frame: RwLock::new(Instant::now()),
            last_fps_update_time: RwLock::new(Instant::now()),
            frames_since_last_update: atomic::AtomicU64::new(0),
        }
    }
}

/// Performance display component showing FPS
#[tessera]
pub fn perf_display(metrics: Arc<PerformanceMetrics>) {
    text(format!(
        "FPS: {}",
        metrics.fps.load(atomic::Ordering::SeqCst)
    ));
    state_handler(Box::new(move |_| {
        let now = Instant::now();
        let mut last_frame_guard = metrics.last_frame.write();
        *last_frame_guard = now;

        metrics
            .frames_since_last_update
            .fetch_add(1, atomic::Ordering::SeqCst);

        let mut last_fps_update_time_guard = metrics.last_fps_update_time.write();
        let elapsed_ms = now.duration_since(*last_fps_update_time_guard).as_millis();

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
