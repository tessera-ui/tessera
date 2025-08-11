mod app;
mod cal;

use log::error;
use tessera_ui::{Renderer, router::router_root};

use crate::app::AppDestination;

#[cfg(target_os = "android")]
use tessera_ui::winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    use android_logger::Config;
    use log::{LevelFilter, error, info};

    android_logger::init_once(Config::default().with_max_level(LevelFilter::Info));

    Renderer::run(
        || router_root(AppDestination {}),
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
            let background_pipeline = app::pipelines::background::BackgroundPipeline::new(
                &app.gpu,
                &app.config,
                app.sample_count,
            );
            app.drawer.pipeline_registry.register(background_pipeline);
        },
        android_app.clone(),
    )
    .unwrap_or_else(|err| error!("App failed to run: {}", err));
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
fn main() {}

#[cfg(not(target_os = "android"))]
pub fn desktop_main() -> anyhow::Result<()> {
    use tessera_ui::renderer::TesseraConfig;

    let _logger = flexi_logger::Logger::try_with_env_or_str("info")?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;

    Renderer::run_with_config(
        || router_root(AppDestination {}),
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
            let background_pipeline = app::pipelines::background::BackgroundPipeline::new(
                &app.gpu,
                &app.config,
                app.sample_count,
            );
            app.drawer.pipeline_registry.register(background_pipeline);
        },
        TesseraConfig {
            window_title: "Calculator".to_string(),
            sample_count: 1,
        },
    )
    .unwrap_or_else(|e| error!("App failed to run: {e}"));
    Ok(())
}
