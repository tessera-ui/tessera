use tessera_ui::{DimensionValue, Renderer, tessera};
use tessera_ui_basic_components::{
    surface::{SurfaceArgs, surface},
    text::text,
};

#[tessera]
fn app() {
    surface(
        SurfaceArgs {
            width: DimensionValue::FILLED,
            height: DimensionValue::FILLED,
            ..Default::default()
        },
        None,
        || {
            text("Hello World!");
        },
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Renderer::run(app, |app| {
        tessera_ui_basic_components::pipelines::register_pipelines(app);
    })?;
    Ok(())
}
