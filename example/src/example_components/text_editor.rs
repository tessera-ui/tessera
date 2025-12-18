use std::sync::Arc;

use tessera_ui::{Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgsBuilder, scrollable},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
    text_editor::{TextEditorArgsBuilder, TextEditorController, text_editor_with_controller},
};

#[tessera]
#[shard]
pub fn text_editor_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .modifier(Modifier::new().fill_max_size())
                    .build()
                    .unwrap(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0)))
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
    let editor_state = remember(|| TextEditorController::new(Dp(22.0), None));

    column(
        ColumnArgsBuilder::default()
            .modifier(Modifier::new().fill_max_width())
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Text Editor Showcase")
                        .size(Dp(20.0))
                        .build()
                        .unwrap(),
                )
            });

            scope.child(|| spacer(Modifier::new().height(Dp(10.0))));

            scope.child(move || {
                text_editor_with_controller(
                    TextEditorArgsBuilder::default()
                        .modifier(Modifier::new().fill_max_width().height(Dp(200.0)))
                        .on_change(Arc::new(move |v| v))
                        .build()
                        .unwrap(),
                    editor_state,
                );
            });
        },
    )
}
