use tessera_components::{
    column::column, icon::icon, modifier::ModifierExt, spacer::spacer, text::text,
    theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{Dp, Modifier, use_context};

use crate::res;

#[shard]
pub fn home() {
    let theme = use_context::<MaterialTheme>().unwrap();

    column()
        .modifier(Modifier::new().padding_all(Dp(16.0)))
        .children(move || {
            text()
                .content("Hello From Tessera!")
                .style(theme.with(|t| t.typography.headline_large));

            spacer().modifier(Modifier::new().height(Dp(5.0)));

            icon()
                .size(Dp(200.0))
                .try_painter_asset(res::LOGO_PNG)
                .expect("bundled logo should load");

            spacer().modifier(Modifier::new().height(Dp(5.0)));

            text().content(
                r#"This is a sample application for the Tessera UI framework!

Tessera is a modern UI framework based on Rust, aiming to provide high performance, ease of use, and cross-platform support.

The purpose of this sample application is to showcase Tessera's capabilities and some of its components.
                "#,
            );
        });
}
