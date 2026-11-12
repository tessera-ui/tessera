mod app;
#[cfg(all(target_family = "wasm", feature = "md3-diagnostics"))]
mod md3_diagnostics;
mod pages;
#[cfg(all(test, not(target_family = "wasm")))]
mod render_precision_tests;
pub mod res;

use tessera_ui::EntryPoint;

use pages::custom_shader::CustomShaderPackage;

#[cfg(not(all(target_family = "wasm", feature = "md3-diagnostics")))]
use app::app;

#[cfg(target_family = "wasm")]
use tessera_ui::renderer::WebConfig;
#[cfg(target_family = "wasm")]
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[tessera_ui::entry]
pub fn run() -> EntryPoint {
    #[cfg(all(target_family = "wasm", feature = "md3-diagnostics"))]
    let entry = md3_diagnostics::entry();
    #[cfg(not(all(target_family = "wasm", feature = "md3-diagnostics")))]
    let entry = EntryPoint::new(app);
    let mut config = res::application();
    config.sample_count = 4;
    config.window = config.window.with_app_titlebar();
    #[cfg(target_family = "wasm")]
    {
        config.web = WebConfig::default().with_canvas_id(env!("CARGO_CRATE_NAME"));
    }
    entry
        .config(config)
        .package(tessera_components::ComponentsPackage)
        .package(res::i18n())
        .package(pages::FONTS)
        .package(CustomShaderPackage)
}

#[cfg(all(target_family = "wasm", not(feature = "md3-diagnostics")))]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run()
        .run_web()
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

#[cfg(all(target_family = "wasm", feature = "md3-diagnostics"))]
#[wasm_bindgen]
/// Starts the diagnostic application and returns observations bound to its lifetime.
pub fn md3_start_diagnostics() -> Result<md3_diagnostics::Md3Diagnostics, JsValue> {
    let entry = run();
    let observations = md3_diagnostics::Md3Diagnostics::bind(&entry);
    entry
        .run_web()
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    Ok(observations)
}

#[cfg(test)]
struct TestPipelines;

#[cfg(test)]
impl tessera_ui::render::RenderModule for TestPipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::render::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[cfg(test)]
use tessera_ui::testing::{text_matches, visible_text};

#[cfg(test)]
fn has_label(node: &tessera_ui::accesskit::Node, expected: &str) -> bool {
    tessera_ui::testing::text_matches(node.label(), &res::I18N.text(expected))
}
