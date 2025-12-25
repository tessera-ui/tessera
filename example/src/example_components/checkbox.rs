use tessera_ui::{Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    checkbox::{CheckboxArgs, checkbox},
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    scrollable::{ScrollableArgs, scrollable},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};

#[tessera]
#[shard]
pub fn checkbox_showcase() {
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
                            column(
                                ColumnArgs::default()
                                    .cross_axis_alignment(CrossAxisAlignment::Start),
                                |scope| {
                                    scope.child(|| {
                                        text(
                                            TextArgs::default()
                                                .text("Checkbox Showcase")
                                                .size(Dp(20.0)),
                                        )
                                    });

                                    // Interactive Checkbox,
                                    scope.child(move || {
                                        let is_checked = remember(|| true);
                                        row(
                                            RowArgs::default()
                                                .cross_axis_alignment(CrossAxisAlignment::Center),
                                            |scope| {
                                                scope.child(move || {
                                                    checkbox(
                                                        CheckboxArgs::default()
                                                            .checked(true)
                                                            .on_toggle(move |new_value| {
                                                                is_checked.set(new_value);
                                                            }),
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
