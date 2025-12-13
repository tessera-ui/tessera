use std::sync::Arc;

use tessera_ui::{DimensionValue, Dp, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
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
    let editor_state = remember(|| TextEditorController::new(Dp(22.0), None));

    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
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

            scope.child(|| spacer(Dp(10.0)));

            scope.child(move || {
                text_editor_with_controller(
                    TextEditorArgsBuilder::default()
                        .width(DimensionValue::FILLED)
                        .height(Dp(200.0))
                        .on_change(Arc::new(move |v| v))
                        .build()
                        .unwrap(),
                    editor_state,
                );
            });
        },
    )
}
