use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
    text_editor::{TextEditorArgsBuilder, TextEditorStateHandle, text_editor},
};

#[derive(Clone)]
struct TextEditorShowcaseState {
    scrollable_state: ScrollableState,
    editor_state: TextEditorStateHandle,
}

impl Default for TextEditorShowcaseState {
    fn default() -> Self {
        Self {
            scrollable_state: Default::default(),
            editor_state: TextEditorStateHandle::new(Dp(22.0), None),
        }
    }
}

#[tessera]
#[shard]
pub fn text_editor_showcase(#[state] state: TextEditorShowcaseState) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .style(Color::WHITE.into())
            .build()
            .unwrap(),
        None,
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                state.scrollable_state.clone(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .style(Color::WHITE.into())
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        None,
                        move || {
                            test_content(state);
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content(state: Arc<TextEditorShowcaseState>) {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        |scope| {
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
                text_editor(
                    TextEditorArgsBuilder::default()
                        .width(DimensionValue::FILLED)
                        .height(Dp(200.0))
                        .on_change(Arc::new(move |new_value| new_value))
                        .build()
                        .unwrap(),
                    state.editor_state.clone(),
                );
            });
        },
    )
}
