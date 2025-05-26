use tessera::Renderer;
use tessera_basic_components::{
    column::column,
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_macros::tessera;

fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    Renderer::run(app)
}

#[tessera]
fn app() {
    let args = SurfaceArgsBuilder::default().build().unwrap();
    surface(args, || {
        column([
            &|| text("Hello"),
            &|| text("This is a simple example of using Tessera."),
            &|| text("You can create complex UIs with ease."),
            &|| text("You can create complex UIs with ease."),
        ])
    });
}
