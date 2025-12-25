use tessera_ui::{Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgs, column},
    glass_slider::{GlassSliderArgs, GlassSliderController, glass_slider_with_controller},
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgs, scrollable},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};

#[tessera]
#[shard]
pub fn glass_slider_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_width()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
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
        ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
        move |scope| {
            scope.child(|| text("Glass Slider Showcase"));
            scope.child(move || {
                glass_slider_with_controller(
                    GlassSliderArgs::default()
                        .value(value.get())
                        .on_change(move |new_value| {
                            value.set(new_value);
                        })
                        .modifier(Modifier::new().width(Dp(250.0))),
                    slider_controller,
                );
            });

            scope.child(move || {
                text(
                    TextArgs::default()
                        .text(format!("Value: {:.2}", value.get()))
                        .size(Dp(16.0)),
                );
            });
        },
    )
}
