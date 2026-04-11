mod app;
pub mod res;

use tessera_ui::{
    EntryPoint,
    renderer::{TesseraConfig, WindowConfig},
};

use app::app;

#[cfg(target_family = "wasm")]
use tessera_ui::renderer::WebConfig;
#[cfg(target_family = "wasm")]
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[tessera_ui::entry]
pub fn run() -> EntryPoint {
    EntryPoint::new(app)
        .config(TesseraConfig {
            window: WindowConfig {
                decorations: false,
                ..Default::default()
            },
            #[cfg(target_family = "wasm")]
            web: WebConfig::default().with_canvas_id(env!("CARGO_CRATE_NAME")),
            ..Default::default()
        })
        .package(tessera_components::ComponentsPackage)
        .package(tessera_platform::PlatformPackage)
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run()
        .run_web()
        .map_err(|err| JsValue::from_str(&err.to_string()))
}
