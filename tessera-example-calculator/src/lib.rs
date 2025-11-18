mod app;
mod cal;

use tessera_ui::{Renderer, router::router_root};
use tracing::error;

use crate::app::AppDestination;

#[cfg(not(target_os = "android"))]
use clap::Parser;

#[cfg(target_os = "android")]
use tessera_ui::winit::platform::android::activity::AndroidApp;

#[cfg(not(target_os = "android"))]
#[derive(Parser, Debug, Clone, Copy, clap::ValueEnum)]
enum CalStyle {
    Glass,
    Material,
}

#[cfg(target_os = "android")]
#[derive(Clone, Copy, Debug)]
enum CalStyle {
    Glass,
    Material,
}

#[cfg(not(target_os = "android"))]
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_enum, default_value_t = CalStyle::Glass)]
    style: CalStyle,
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    // Bridge any remaining `log` crate usage into `tracing`
    tracing_log::LogTracer::init().ok();

    // Initialize tracing subscriber for Android (EnvFilter still honored)
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting Android app...");

    Renderer::run(
        || {
            router_root(AppDestination {
                style: CalStyle::Glass,
            })
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
            let background_pipeline = app::pipelines::background::BackgroundPipeline::new(
                &app.gpu,
                &app.config,
                app.sample_count,
            );
            app.register_draw_pipeline(background_pipeline);
        },
        android_app.clone(),
    )
    .unwrap_or_else(|err| tracing::error!("App failed to run: {}", err));
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
fn main() {}

#[cfg(not(target_os = "android"))]
pub fn desktop_main() -> anyhow::Result<()> {
    use tessera_ui::renderer::TesseraConfig;
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(tracing::Level::INFO)
        .init();

    Renderer::run_with_config(
        move || router_root(AppDestination { style: cli.style }),
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
            let background_pipeline = app::pipelines::background::BackgroundPipeline::new(
                &app.gpu,
                app.pipeline_cache.as_ref(),
                &app.config,
                app.sample_count,
            );
            app.register_draw_pipeline(background_pipeline);
        },
        TesseraConfig {
            window_title: "Calculator".to_string(),
            sample_count: 1,
        },
    )
    .unwrap_or_else(|e| error!("App failed to run: {e}"));
    Ok(())
}
