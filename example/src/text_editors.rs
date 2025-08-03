use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui_basic_components::text_editor::TextEditorState;

pub struct TextEditorsState {
    pub editor_state: Arc<RwLock<TextEditorState>>,
}

impl TextEditorsState {
    pub fn new() -> Self {
        Self {
            editor_state: Arc::new(RwLock::new(TextEditorState::new(50.0.into(), None))),
        }
    }
}
