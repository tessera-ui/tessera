use tessera_components::{
    modifier::ModifierExt as _,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Modifier, tessera};

#[tessera]
pub fn app() {
    material_theme().theme(MaterialTheme::default).child(|| {
        surface()
            .modifier(Modifier::new().fill_max_size())
            .child(|| {
                text().content("Hello Tessera!");
            });
    });
}
