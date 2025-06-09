mod app;

use std::sync::{Arc, atomic::AtomicU64};

use log::error;
use tessera::Renderer;
#[cfg(target_os = "android")]
use tessera::winit::platform::android::activity::AndroidApp;

use app::app;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    use android_logger::Config;
    use log::{LevelFilter, info};

    android_logger::init_once(Config::default().with_max_level(LevelFilter::Info));
    info!("Starting Android app...");
    let value = Arc::new(AtomicU64::new(0));
    Renderer::run(|| app(value.clone()), android_app.clone())
        .unwrap_or_else(|err| error!("App failed to run: {}", err));
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
fn main() {}

#[cfg(not(target_os = "android"))]
fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    let value = Arc::new(AtomicU64::new(0));
    Renderer::run(|| app(value.clone()))
}
