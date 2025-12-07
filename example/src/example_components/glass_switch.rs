use std::sync::Arc;

use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgsBuilder, column},
    glass_switch::{GlassSwitchArgsBuilder, glass_switch},
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn glass_switch_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
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
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Switch Showcase"));

            scope.child(move || {
                glass_switch(
                    GlassSwitchArgsBuilder::default()
                        .on_toggle(Arc::new(|value| {
                            println!("Glass Switch toggled to: {}", value);
                        }))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Disabled Glass Switch")
                        .size(Dp(16.0))
                        .build()
                        .unwrap(),
                )
            });
            scope.child(|| {
                // Disabled by not providing on_change
                glass_switch(GlassSwitchArgsBuilder::default().build().unwrap());
            });
        },
    )
}
