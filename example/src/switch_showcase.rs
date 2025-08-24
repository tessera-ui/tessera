use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{DimensionValue, Dp, tessera};
use tessera_ui_basic_components::{
    column::ColumnArgsBuilder,
    column_ui,
    row::RowArgsBuilder,
    row_ui,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    switch::{SwitchArgsBuilder, SwitchState, switch},
    text::{TextArgsBuilder, text},
};

use crate::{material_colors::md_colors, misc::create_spacer};

#[tessera]
pub fn switch_showcase(state: Arc<Mutex<SwitchState>>) {
    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER)
            .padding(Dp(24.0))
            .shape(Shape::RoundedRectangle {
                top_left: 25.0,
                top_right: 25.0,
                bottom_right: 25.0,
                bottom_left: 25.0,
                g2_k_value: 3.0,
            })
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
                            .size(tessera_ui::Dp(24.0))
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
                                    .on_toggle(Arc::new(move |on| {
                                        if on {
                                            println!("Switch toggled to OFF");
                                        } else {
                                            println!("Switch toggled to ON");
                                        }
                                    }))
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
