use tessera::Renderer;
use tessera_basic_components::text::{TextArgsBuilder, text};
use tessera_macros::tessera;

fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    Renderer::run(app)
}

#[tessera]
fn app() {
    let text_args = TextArgsBuilder::default()
        .text("Hello, World!".to_string())
        .build()
        .unwrap();
    text(text_args);
}
