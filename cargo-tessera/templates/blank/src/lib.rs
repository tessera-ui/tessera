use tessera_ui::{Renderer, tessera};
use tracing::error;
use tracing_subscriber::EnvFilter;

#[cfg(target_os = "android")]
use tessera_ui::winit::platform::android::activity::AndroidApp;

#[tessera]
fn app() {
    // Empty application
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    init_tracing_android();
    Renderer::run(
        app,
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
        android_app.clone(),
    )
    .unwrap_or_else(|err| error!("App failed to run: {err}"));
}

#[cfg(not(target_os = "android"))]
pub fn desktop_main() {
    init_tracing_desktop();
    Renderer::run(app, |app| {
        tessera_ui_basic_components::pipelines::register_pipelines(app);
    })
    .unwrap_or_else(|err| error!("App failed to run: {err}"));
}

#[cfg(target_os = "android")]
fn init_tracing_android() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();
}

#[cfg(not(target_os = "android"))]
fn init_tracing_desktop() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("off,tessera_ui=info"))
        .unwrap();
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
fn main() {}
