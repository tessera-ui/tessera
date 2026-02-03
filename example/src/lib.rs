mod app;
mod example_components;

use tessera_ui::EntryPoint;

use crate::app::app;

#[tessera_ui::entry]
pub fn run() -> EntryPoint {
    EntryPoint::new(app).package(tessera_components::ComponentsPackage)
}
