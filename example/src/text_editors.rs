use parking_lot::RwLock;
use std::sync::Arc;
use tessera::{DimensionValue, Px};
use tessera_basic_components::text_editor::{TextEditorArgsBuilder, TextEditorState, text_editor};
use tessera_macros::tessera;

pub struct TextEditorsState {
    pub editor_state: Arc<RwLock<TextEditorState>>,
    pub editor_state_2: Arc<RwLock<TextEditorState>>,
}

impl TextEditorsState {
    pub fn new() -> Self {
        Self {
            editor_state: Arc::new(RwLock::new(TextEditorState::new(50.0.into(), 50.0.into()))),
            editor_state_2: Arc::new(RwLock::new(TextEditorState::new(50.0.into(), 50.0.into()))),
        }
    }
}

/// First text editor with custom selection color
#[tessera]
pub fn text_editor_1(state: Arc<RwLock<TextEditorState>>) {
    text_editor(
        TextEditorArgsBuilder::default()
            .height(Some(DimensionValue::Fixed(Px(120))))
            .width(Some(DimensionValue::Fill {
                min: None,
                max: None,
            }))
            .selection_color(Some([0.3, 0.8, 0.4, 0.5])) // Custom green selection with 50% transparency
            .build()
            .unwrap(),
        state,
    );
}

/// Second text editor with default selection color
#[tessera]
pub fn text_editor_2(state: Arc<RwLock<TextEditorState>>) {
    text_editor(
        TextEditorArgsBuilder::default()
            .height(Some(DimensionValue::Fixed(Px(100))))
            .width(Some(DimensionValue::Fill {
                min: None,
                max: None,
            }))
            .build()
            .unwrap(),
        state,
    );
}
