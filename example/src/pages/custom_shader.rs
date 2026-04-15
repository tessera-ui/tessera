use tessera_components::{
    column::column, modifier::ModifierExt, spacer::spacer, text::text, theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{Dp, Modifier, use_context};

#[shard]
pub fn custom_shader_page() {
    let theme = use_context::<MaterialTheme>().unwrap();

    column()
        .modifier(Modifier::new().padding_all(Dp(16.0)))
        .children(move || {
            text()
                .content("Custom Shader")
                .style(theme.with(|t| t.typography.headline_large));

            spacer().modifier(Modifier::new().height(Dp(8.0)));

            text().content("Custom WGSL effects.");
        });
}
