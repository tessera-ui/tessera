use tessera_ui::{Dp, Modifier, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgs, column},
    glass_switch::{GlassSwitchArgs, glass_switch},
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgs, scrollable},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};

#[tessera]
#[shard]
pub fn glass_switch_showcase() {
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
    column(
        ColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .cross_axis_alignment(CrossAxisAlignment::Start),
        move |scope| {
            scope.child(|| text("Glass Switch Showcase"));

            scope.child(move || {
                glass_switch(GlassSwitchArgs::default().on_toggle(|value| {
                    println!("Glass Switch toggled to: {}", value);
                }));
            });

            scope.child(|| {
                text(
                    TextArgs::default()
                        .text("Disabled Glass Switch")
                        .size(Dp(16.0)),
                )
            });
            scope.child(|| {
                // Disabled by not providing on_change,
                glass_switch(GlassSwitchArgs::default());
            });
        },
    )
}
