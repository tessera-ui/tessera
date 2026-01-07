use tessera_components::theme::{MaterialTheme, material_theme};
use tessera_ui::tessera;

#[tessera]
pub fn app() {
    material_theme(MaterialTheme::default, || {
        // Your app code goes here
    });
}
