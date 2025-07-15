mod app;
mod background;
mod logo;

use std::sync::Arc;

use log::error;
use tessera_ui::Renderer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize a logger to see output
    let _logger = flexi_logger::Logger::try_with_env()?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;

    let app_state = Arc::new(app::AppState::new());

    // 2. Run the Tessera application using the standard helper
    Renderer::run(
        // The root component of our application
        move || app::app(app_state.clone()),
        // A closure to register all necessary rendering pipelines
        |renderer| {
            // Register pipelines from the basic components crate
            tessera_ui_basic_components::pipelines::register_pipelines(renderer);

            // Register our custom crystal pipeline
            let crystal_pipeline =
                logo::CrystalPipeline::new(&renderer.gpu, &renderer.config, renderer.sample_count);
            renderer.drawer.pipeline_registry.register(crystal_pipeline);

            let background_pipeline = background::BackgroundPipeline::new(
                &renderer.gpu,
                &renderer.config,
                renderer.sample_count,
            );
            renderer
                .drawer
                .pipeline_registry
                .register(background_pipeline);
        },
    )
    .unwrap_or_else(|e| error!("App failed to run: {e}"));

    Ok(())
}
