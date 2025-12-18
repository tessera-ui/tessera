use std::sync::Arc;

use tessera_ui::{Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    checkbox::{CheckboxArgsBuilder, checkbox},
    column::{ColumnArgsBuilder, column},
    modifier::ModifierExt as _,
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn checkbox_showcase() {
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
                            column(
                                ColumnArgsBuilder::default()
                                    .cross_axis_alignment(CrossAxisAlignment::Start)
                                    .build()
                                    .unwrap(),
                                |scope| {
                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Checkbox Showcase")
                                                .size(Dp(20.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    // Interactive Checkbox
                                    scope.child(move || {
                                        let is_checked = remember(|| true);
                                        row(
                                            RowArgsBuilder::default()
                                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                                .build()
                                                .unwrap(),
                                            |scope| {
                                                scope.child(move || {
                                                    let on_toggle = Arc::new({
                                                        move |new_value| {
                                                            is_checked.set(new_value);
                                                        }
                                                    });
                                                    checkbox(
                                                        CheckboxArgsBuilder::default()
                                                            .checked(true)
                                                            .on_toggle(on_toggle)
                                                            .build()
                                                            .unwrap(),
                                                    );
                                                });
                                                scope.child(move || {
                                                    let checked_str = if is_checked.get() {
                                                        "Checked"
                                                    } else {
                                                        "Unchecked"
                                                    };
                                                    text(format!("State: {}", checked_str));
                                                });
                                            },
                                        );
                                    });
                                },
                            )
                        },
                    );
                },
            )
        },
    );
}
