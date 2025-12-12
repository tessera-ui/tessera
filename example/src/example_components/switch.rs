use std::sync::Arc;

use tessera_ui::{DimensionValue, Dp, remember, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgsBuilder, column},
    icon::{IconArgsBuilder, icon},
    material_icons::round::check_icon,
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    switch::{
        SwitchArgsBuilder, SwitchController, switch, switch_with_child_and_controller,
        switch_with_controller,
    },
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn switch_showcase() {
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
            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Switch Showcase")
                        .size(Dp(20.0))
                        .build()
                        .unwrap(),
                )
            });

            scope.child(move || {
                let controller = remember(|| SwitchController::new(false));
                if controller.with(|c| c.is_checked()) {
                    switch_with_child_and_controller(
                        SwitchArgsBuilder::default()
                            .on_toggle(Arc::new(|value| {
                                println!("Switch toggled to: {}", value);
                            }))
                            .build()
                            .unwrap(),
                        controller,
                        move || {
                            icon(
                                IconArgsBuilder::default()
                                    .content(check_icon())
                                    .size(Dp(16.0))
                                    .build()
                                    .unwrap(),
                            );
                        },
                    );
                } else {
                    switch_with_controller(
                        SwitchArgsBuilder::default()
                            .on_toggle(Arc::new(|value| {
                                println!("Switch toggled to: {}", value);
                            }))
                            .build()
                            .unwrap(),
                        controller,
                    );
                }
            });

            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Disabled Switch")
                        .size(Dp(16.0))
                        .build()
                        .unwrap(),
                )
            });
            scope.child(move || {
                // Disabled by not providing on_change
                switch(SwitchArgsBuilder::default().build().unwrap());
            });
        },
    )
}
