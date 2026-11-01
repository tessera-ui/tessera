mod app;
#[cfg(all(target_family = "wasm", feature = "md3-diagnostics"))]
mod md3_diagnostics;
mod pages;
#[cfg(all(test, not(target_family = "wasm")))]
mod render_precision_tests;
pub mod res;

use tessera_ui::{
    EntryPoint,
    renderer::{TesseraConfig, WindowConfig},
};

#[cfg(not(all(target_family = "wasm", feature = "md3-diagnostics")))]
use app::app;
use pages::custom_shader::CustomShaderPackage;

#[cfg(target_family = "wasm")]
use tessera_ui::renderer::WebConfig;
#[cfg(target_family = "wasm")]
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[tessera_ui::entry]
pub fn run() -> EntryPoint {
    #[cfg(all(target_family = "wasm", feature = "md3-diagnostics"))]
    let entry = EntryPoint::new(md3_diagnostics::diagnostic_app);
    #[cfg(not(all(target_family = "wasm", feature = "md3-diagnostics")))]
    let entry = EntryPoint::new(app);
    entry
        .config(TesseraConfig {
            sample_count: 4,
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
        .package(CustomShaderPackage)
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run()
        .run_web()
        .map_err(|err| JsValue::from_str(&err.to_string()))
}
