use tessera_components::{
    column::column, modifier::ModifierExt, spacer::spacer, text::text, theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{Dp, Modifier, use_context};

#[shard]
pub fn glass_components_page() {
    let theme = use_context::<MaterialTheme>().unwrap();

    column()
        .modifier(Modifier::new().padding_all(Dp(16.0)))
        .children(move || {
            text()
                .content("Glass Components")
                .style(theme.with(|t| t.typography.headline_large));

            spacer().modifier(Modifier::new().height(Dp(8.0)));

            text().content("Glassmorphism components powered by the fluid glass pipeline, as a showcase of the capabilities of complex effects in Tessera.");
        });
}
