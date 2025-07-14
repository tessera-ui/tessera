use std::sync::Arc;

use parking_lot::Mutex;
use tessera::{DimensionValue, Dp};
use tessera_basic_components::{
    column::ColumnArgsBuilder,
    column_ui,
    row::RowArgsBuilder,
    row_ui,
    surface::{SurfaceArgsBuilder, surface},
    switch::{SwitchArgsBuilder, SwitchState, switch},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

use crate::{material_colors::md_colors, misc::create_spacer};

#[tessera]
pub fn switch_showcase(state: Arc<Mutex<SwitchState>>) {
    let on_toggle = {
        let state = state.clone();
        Arc::new(move |_| {
            state.lock().toggle();
        })
    };

    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER)
            .corner_radius(25.0)
            .padding(Dp(24.0))
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None,
        move || {
            column_ui!(
                ColumnArgsBuilder::default().build().unwrap(),
                || {
                    text(
                        TextArgsBuilder::default()
                            .text("Switch Component".to_string())
                            .size(tessera::Dp(24.0))
                            .color(md_colors::ON_SURFACE)
                            .build()
                            .unwrap(),
                    )
                },
                || (create_spacer(12))(),
                move || {
                    row_ui!(
                        RowArgsBuilder::default().build().unwrap(),
                        || text("Off"),
                        || (create_spacer(16))(),
                        move || {
                            let checked = state.lock().checked;
                            switch(
                                SwitchArgsBuilder::default()
                                    .state(Some(state.clone()))
                                    .checked(checked)
                                    .on_toggle(on_toggle.clone())
                                    .build()
                                    .unwrap(),
                            )
                        },
                        || (create_spacer(16))(),
                        || text("On"),
                    )
                }
            )
        },
    )
}
