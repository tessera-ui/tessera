use tessera_components::{
    boxed::boxed, button::button, fluid_glass::fluid_glass, lazy_list::lazy_column,
    modifier::ModifierExt, slider::slider, text::text, theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{Dp, Modifier, Px, remember, use_context};

const MAX_BLUR_RADIUS: Dp = Dp(20.0);

#[shard]
pub fn glass_components_page() {
    let theme = use_context::<MaterialTheme>().unwrap();
    let offset = remember(|| (Px::ZERO, Px::ZERO));
    let blur_radius_factor = remember(|| 0.0);

    lazy_column()
        .modifier(Modifier::new().fill_max_size())
        .content_padding(Dp(16.0))
        .item_spacing(Dp(8.0))
        .item(move || {
            text()
                .content("Glass Components")
                .style(theme.with(|t| t.typography.headline_large));
        })
        .item(|| {
            text().content("Dragable glass");
        })
        .item(move || {
            boxed()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    text().content("The quick brown fox jumps over the lazy dog.".repeat(10));

                    fluid_glass()
                        .blur_radius(MAX_BLUR_RADIUS * blur_radius_factor.get())
                        .modifier(
                            Modifier::new()
                                .size(Dp(50.0), Dp(50.0))
                                .offset(offset.get().0.to_dp(), offset.get().1.to_dp())
                                .draggable(
                                    move |delta: tessera_components::modifier::DragDelta| {
                                        offset.set((
                                            offset.get().0 + delta.x,
                                            offset.get().1 + delta.y,
                                        ));
                                    },
                                ),
                        );
                });
        })
        .item(move || {
            button()
                .filled()
                .on_click(move || {
                    offset.set((Px::ZERO, Px::ZERO));
                })
                .child(move || {
                    text().content("Reset");
                });
        })
        .item(|| {
            text().content("Blur radius");
        })
        .item(move || {
            slider()
                .value(blur_radius_factor.get())
                .on_change(move |p| blur_radius_factor.set(p));
        });
}
