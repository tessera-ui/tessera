mod app;

use std::sync::Arc;

use tessera::Renderer;
#[cfg(target_os = "android")]
use tessera::winit::platform::android::activity::AndroidApp;

use app::{AppState, app};

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    use android_logger::Config;
    use log::{LevelFilter, error, info};

    android_logger::init_once(Config::default().with_max_level(LevelFilter::Info));
    info!("Starting Android app...");
    let app_state = Arc::new(AppState::new());
    Renderer::run(|| app(app_state.clone()), android_app.clone())
        .unwrap_or_else(|err| error!("App failed to run: {}", err));
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
fn main() {}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use log::error;

    let _logger = flexi_logger::Logger::try_with_env()?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;
    let app_state = Arc::new(AppState::new());
    Renderer::run(|| app(app_state.clone()))
        .unwrap_or_else(|e| error!("App failed to run: {}", e));
    Ok(())
}
