mod display_screen;
mod keyboard;
pub mod pipelines;

use parking_lot::RwLock;
use tessera_ui::{shard, tessera};
use tessera_ui_basic_components::column::{ColumnArgsBuilder, column};

use crate::CalStyle;

use display_screen::display_screen;
use keyboard::keyboard;
use pipelines::background::background;

struct AppState {
    expr: RwLock<String>,
    result: RwLock<f64>,
    interpreter: RwLock<rsc::Interpreter<f64>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            expr: String::from("1 + 1").into(),
            result: 0.0.into(),
            interpreter: rsc::Interpreter::new().into(),
        }
    }
}

#[tessera]
#[shard]
pub fn app(#[state] state: AppState, style: CalStyle) {
    background(
        || {
            column(
                ColumnArgsBuilder::default()
                    .width(tessera_ui::DimensionValue::FILLED)
                    .height(tessera_ui::DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                |scope| {
                    scope.child({
                        let state = state.clone();
                        move || {
                            display_screen(state, style);
                        }
                    });
                    scope.child_weighted(
                        {
                            let state = state.clone();
                            move || {
                                keyboard(state, style);
                            }
                        },
                        1.0,
                    );
                },
            );
        },
        style,
    );
}
