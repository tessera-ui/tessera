use tessera_components::{
    modifier::ModifierExt,
    surface::{SurfaceArgs, surface},
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Modifier, tessera};

#[tessera]
pub fn app() {
    material_theme(MaterialTheme::default, || {
        surface(
            SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
            || {
                text("Hello Tessera!");
            },
        );
    });
}
