use tessera_components::{
    alignment::CrossAxisAlignment, column::column, icon::icon, lazy_list::lazy_column,
    modifier::ModifierExt, row::row, spacer::spacer, text::text, theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{Dp, Modifier, use_context};

use crate::res::{self, I18N};

#[shard]
pub fn home_page() {
    let theme = use_context::<MaterialTheme>().unwrap();
    let logo_asset = if theme.with(|theme| theme.color_scheme.is_dark) {
        res::LOGO_DARK_PNG
    } else {
        res::LOGO_PNG
    };
    lazy_column()
        .modifier(Modifier::new().fill_max_size())
        .estimated_item_size(Dp(120.0))
        .content_padding(Dp(24.0))
        .item_spacing(Dp(24.0))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .item(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    icon().size(Dp(96.0)).painter(logo_asset);
                    spacer().modifier(Modifier::new().width(Dp(24.0)));
                    column().children(move || {
                        text(I18N.text("ui-hello-from-tessera"))
                            .style(theme.with(|t| t.typography.headline_large))
                            .color(theme.with(|t| t.color_scheme.on_surface));
                        spacer().modifier(Modifier::new().height(Dp(8.0)));
                        text(I18N.text("ui-tessera-tagline"))
                            .style(theme.with(|t| t.typography.title_medium))
                            .color(theme.with(|t| t.color_scheme.on_surface_variant));
                    });
                });
        })
        .item(move || {
            text(I18N.text(
                "ui-this-is-a-sample-application-for-the-tessera-ui-framework-tessera-is-a-modern-ui-framework",
            ))
            .style(theme.with(|t| t.typography.body_large))
            .color(theme.with(|t| t.color_scheme.on_surface_variant))
            .modifier(Modifier::new().size_in(None, Some(Dp(720.0)), None, None));
        });
}
