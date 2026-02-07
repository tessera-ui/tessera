mod app;
mod example_components;

use tessera_ui::{
    EntryPoint,
    renderer::{TesseraConfig, WindowConfig},
};

use crate::app::app;

#[tessera_ui::entry]
pub fn run() -> EntryPoint {
    let config = TesseraConfig {
        window: WindowConfig {
            decorations: false,
            ..Default::default()
        },
        ..Default::default()
    };
    EntryPoint::new(app)
        .package(tessera_components::ComponentsPackage)
        .config(config)
}
