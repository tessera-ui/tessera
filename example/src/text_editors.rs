use parking_lot::RwLock;
use std::sync::Arc;
use tessera::{DimensionValue, Px};
use tessera_basic_components::text_editor::{text_editor, TextEditorArgsBuilder, TextEditorState};
use tessera_macros::tessera;

use crate::material_colors::md_colors;

pub struct TextEditorsState {
    pub editor_state: Arc<RwLock<TextEditorState>>,
    pub editor_state_2: Arc<RwLock<TextEditorState>>,
}

impl TextEditorsState {
    pub fn new() -> Self {
        Self {
            editor_state: Arc::new(RwLock::new(TextEditorState::new(50.0.into(), None))),
            editor_state_2: Arc::new(RwLock::new(TextEditorState::new(50.0.into(), None))),
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
            .selection_color(Some([
                md_colors::PRIMARY[0],
                md_colors::PRIMARY[1],
                md_colors::PRIMARY[2],
                0.4,
            ])) // Material Design primary color with transparency
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
