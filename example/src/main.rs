use tessera::Renderer;
use tessera_basic_components::{
    column::column,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
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
            &|| {
                text(
                    TextArgsBuilder::default()
                        .text("Hello, World!".to_string())
                        .build()
                        .unwrap(),
                )
            },
            &|| {
                text(
                    TextArgsBuilder::default()
                        .text("This is a simple example of using Tessera.".to_string())
                        .build()
                        .unwrap(),
                )
            },
            &|| {
                text(
                    TextArgsBuilder::default()
                        .text("You can create complex UIs with ease.".to_string())
                        .build()
                        .unwrap(),
                )
            },
            &|| {
                text(
                    TextArgsBuilder::default()
                        .text("Enjoy building with Tessera!".to_string())
                        .build()
                        .unwrap(),
                )
            },
        ])
    });
}
