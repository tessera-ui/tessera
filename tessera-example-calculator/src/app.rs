mod display_screen;
mod keyboard;
pub mod pipelines;

use tessera_ui::{Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    modifier::ModifierExt as _,
};

use crate::CalStyle;

use display_screen::display_screen;
use keyboard::keyboard;
use pipelines::background::background;

struct AppState {
    expr: String,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            expr: String::from("1 + 1"),
        }
    }
}

#[tessera]
#[shard]
pub fn app(style: CalStyle) {
    let state = remember(AppState::default);
    background(
        move || {
            column(
                ColumnArgsBuilder::default()
                    .modifier(Modifier::new().fill_max_size())
                    .build()
                    .unwrap(),
                |scope| {
                    scope.child(move || {
                        display_screen(state, style);
                    });
                    scope.child_weighted(
                        {
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
