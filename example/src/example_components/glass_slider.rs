use std::sync::Arc;

use tessera_ui::{Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    glass_slider::{GlassSliderArgsBuilder, GlassSliderController, glass_slider_with_controller},
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn glass_slider_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .modifier(Modifier::new().fill_max_width())
                    .build()
                    .unwrap(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0)))
                            .build()
                            .unwrap(),
                        move || {
                            test_content();
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content() {
    let value = remember(|| 0.5);
    let slider_controller = remember(GlassSliderController::new);

    column(
        ColumnArgsBuilder::default()
            .modifier(Modifier::new().fill_max_width())
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Slider Showcase"));
            scope.child(move || {
                let on_change = Arc::new(move |new_value| {
                    value.set(new_value);
                });
                glass_slider_with_controller(
                    GlassSliderArgsBuilder::default()
                        .value(value.get())
                        .on_change(on_change)
                        .width(Dp(250.0))
                        .build()
                        .unwrap(),
                    slider_controller,
                );
            });

            scope.child(move || {
                text(
                    TextArgsBuilder::default()
                        .text(format!("Value: {:.2}", value.get()))
                        .size(Dp(16.0))
                        .build()
                        .unwrap(),
                );
            });
        },
    )
}
