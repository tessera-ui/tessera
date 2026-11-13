use tessera_components::md3::{
    surface::surface,
    theme::{MaterialTheme, material_theme},
};
use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{Modifier, tessera};

#[tessera]
pub fn app() {
    material_theme()
        .theme(|| MaterialTheme::default())
        .child(|| {
            surface().modifier(Modifier::new().fill_max_size());
        });
}
