use tessera_components::{
    alignment::CrossAxisAlignment, icon::icon, lazy_list::lazy_column, modifier::ModifierExt,
    painter::try_painter_asset, spacer::spacer, text::text, theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{Dp, Modifier, RenderSlot, remember_with_key, use_context};

use crate::res;

#[shard]
pub fn home_page() {
    let theme = use_context::<MaterialTheme>().unwrap();
    let logo_asset = if theme.with(|theme| theme.color_scheme.is_dark) {
        res::LOGO_DARK_PNG
    } else {
        res::LOGO_PNG
    };
    let logo_painter = remember_with_key(logo_asset, move || {
        try_painter_asset(logo_asset).expect("logo painter should decode successfully")
    });

    lazy_column()
        .modifier(Modifier::new().fill_max_size())
        .estimated_item_size(Dp(120.0))
        .content_padding(Dp(16.0))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .item({
            let content = RenderSlot::new(move || {
                text()
                    .content("Hello From Tessera!")
                    .style(theme.with(|t| t.typography.headline_large));
                });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(|| {
                spacer().modifier(Modifier::new().height(Dp(5.0)));
                });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(move || {
                icon().size(Dp(200.0)).painter(logo_painter.get());
                });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(|| {
                spacer().modifier(Modifier::new().height(Dp(5.0)));
                });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(|| {
                text().content(r#"This is a sample application for the Tessera UI framework!

Tessera is a modern UI framework based on Rust, aiming to provide high performance, ease of use, and cross-platform support.

The purpose of this sample application is to showcase Tessera's capabilities and some of its components."#);
            });
            move || content.render()
        });
}
