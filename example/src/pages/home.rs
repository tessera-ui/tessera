use tessera_components::{
    alignment::CrossAxisAlignment, icon::icon, lazy_list::lazy_column, modifier::ModifierExt,
    painter::remember_painter_asset, spacer::spacer, text::text, theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{Dp, Modifier, use_context};

use crate::res;

#[shard]
pub fn home_page() {
    let theme = use_context::<MaterialTheme>().unwrap();
    let logo_painter = remember_painter_asset(res::LOGO_PNG);

    lazy_column()
        .modifier(Modifier::new().fill_max_size())
        .estimated_item_size(Dp(120.0))
        .content_padding(Dp(16.0))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .item(move || {
            text()
                .content("Hello From Tessera!")
                .style(theme.with(|t| t.typography.headline_large));
        })
        .item(|| {
            spacer().modifier(Modifier::new().height(Dp(5.0)));
        })
        .item(move || {
            icon().size(Dp(200.0)).painter(logo_painter.get());
        })
        .item(|| {
            spacer().modifier(Modifier::new().height(Dp(5.0)));
        })
        .item(|| {
            text().content(r#"This is a sample application for the Tessera UI framework!

Tessera is a modern UI framework based on Rust, aiming to provide high performance, ease of use, and cross-platform support.

The purpose of this sample application is to showcase Tessera's capabilities and some of its components."#);
        });
}
