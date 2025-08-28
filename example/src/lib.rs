mod app;
mod app_state;
mod component_showcase;
mod content_section;
mod interactive_demo;
mod material_colors;
mod misc;
mod performance_display;
mod switch_showcase;
mod text_editors;

use std::sync::Arc;

use tessera_ui::Renderer;
use tracing::error;

use app::app;
use app_state::AppState;

#[cfg(target_os = "android")]
use tessera_ui::winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    // Initialize tracing subscriber for Android (EnvFilter still honored)
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(tracing::Level::INFO)
        .init();

    let app_state = Arc::new(AppState::new());
    Renderer::run(
        || app(app_state.clone()),
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
        android_app.clone(),
    )
    .unwrap_or_else(|err| error!("App failed to run: {}", err));
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
fn main() {}

#[cfg(not(target_os = "android"))]
pub fn desktop_main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("off,tessera_ui=info"))
        .unwrap();
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let app_state = Arc::new(AppState::new());

    Renderer::run(
        || app(app_state.clone()),
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )
    .unwrap_or_else(|e| error!("App failed to run: {e}"));
    Ok(())
}
