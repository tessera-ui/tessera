use tessera_ui::{Color, Dp, Renderer, tessera};
use tessera_ui_basic_components::{
    surface::{surface, SurfaceArgs},
    text::{text, TextArgsBuilder},
};

#[tessera]
fn app() {
    // Empty application
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Renderer::run(app, |app| {
        tessera_ui_basic_components::pipelines::register_pipelines(app);
    })?;
    Ok(())
}
