use tessera_ui::{Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgs, scrollable},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    text_editor::{TextEditorArgs, TextEditorController, text_editor_with_controller},
};

#[tessera]
#[shard]
pub fn text_editor_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_size()),
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
    let editor_state = remember(|| TextEditorController::new(Dp(22.0), None));

    column(
        ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
        move |scope| {
            scope.child(|| {
                text(
                    TextArgs::default()
                        .text("Text Editor Showcase")
                        .size(Dp(20.0)),
                )
            });

            scope.child(|| spacer(Modifier::new().height(Dp(10.0))));

            scope.child(move || {
                text_editor_with_controller(
                    TextEditorArgs::default()
                        .modifier(Modifier::new().fill_max_width().height(Dp(200.0)))
                        .on_change(move |v| v),
                    editor_state,
                );
            });
        },
    )
}
