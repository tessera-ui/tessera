use tessera_components::{
    alignment::CrossAxisAlignment,
    checkbox::{CheckboxArgs, checkbox},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, retain, shard, tessera};
#[shard]
pub fn checkbox_showcase() {
    checkbox_showcase_node();
}

#[tessera]
fn checkbox_showcase_node() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            let controller = retain(LazyListController::new);
            lazy_column(
                &LazyColumnArgs::default()
                    .modifier(Modifier::new().fill_max_width())
                    .content_padding(Dp(16.0))
                    .controller(controller)
                    .content(move |scope| {
                        scope.item(|| {
                            text(&TextArgs::default().text("Checkbox Showcase").size(Dp(20.0)))
                        });

                        // Interactive Checkbox,
                        scope.item(move || {
                            let is_checked = remember(|| true);
                            row(
                                RowArgs::default().cross_axis_alignment(CrossAxisAlignment::Center),
                                |scope| {
                                    scope.child(move || {
                                        checkbox(&CheckboxArgs::default().checked(true).on_toggle(
                                            move |new_value| {
                                                is_checked.set(new_value);
                                            },
                                        ));
                                    });
                                    scope.child(move || {
                                        let checked_str = if is_checked.get() {
                                            "Checked"
                                        } else {
                                            "Unchecked"
                                        };
                                        text(&TextArgs::from(format!("State: {}", checked_str)));
                                    });
                                },
                            );
                        });
                    }),
            )
        },
    ));
}
