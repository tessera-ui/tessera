mod app;

use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Instant,
};

use parking_lot::RwLock;

use tessera::Renderer;
#[cfg(target_os = "android")]
use tessera::winit::platform::android::activity::AndroidApp;

use app::app;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    use android_logger::Config;
    use log::{LevelFilter, error, info};

    android_logger::init_once(Config::default().with_max_level(LevelFilter::Info));
    info!("Starting Android app...");
    let value = Arc::new(AtomicU64::new(0));
    let fps = Arc::new(AtomicU64::new(0));
    let last_frame = Arc::new(RwLock::new(Instant::now()));
    Renderer::run(
        || app(value.clone(), last_frame.clone(), fps.clone()),
        android_app.clone(),
    )
    .unwrap_or_else(|err| error!("App failed to run: {}", err));
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
fn main() {}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    let value = Arc::new(AtomicU64::new(0));
    let fps = Arc::new(AtomicU64::new(0));
    let last_frame = Arc::new(RwLock::new(Instant::now()));
    Renderer::run(|| app(value.clone(), last_frame.clone(), fps.clone()))
}
