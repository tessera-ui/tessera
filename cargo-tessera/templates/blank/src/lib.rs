mod app;
pub mod res;

use tessera_ui::EntryPoint;

use app::app;

#[tessera_ui::entry]
pub fn run() -> EntryPoint {
    EntryPoint::new(app).package(tessera_components::ComponentsPackage)
}
