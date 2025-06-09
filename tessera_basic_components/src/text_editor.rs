use std::sync::Arc;

use glyphon::{Edit, cosmic_text::BufferRef};
use parking_lot::RwLock;

use tessera::{
    BasicDrawable, ComputedData, DimensionValue, Dp, TextConstraint, TextData, write_font_system,
};
use tessera_macros::tessera;

pub struct TextEditorState {
    text_constraint: TextConstraint,
    editor: glyphon::Editor<'static>,
}

impl TextEditorState {
    /// Creates a new `TextEditorState` with the given size, line height, and text constraint.
    pub fn new(size: Dp, line_height: Dp, text_constraint: TextConstraint) -> Self {
        let buffer = glyphon::Buffer::new(
            &mut write_font_system(),
            glyphon::Metrics::new(size.to_pixels_f32(), line_height.to_pixels_f32()),
        );
        let editor = glyphon::Editor::new(buffer);
        Self {
            editor,
            text_constraint,
        }
    }

    fn text_data(&mut self, new_constraint: TextConstraint) -> TextData {
        let mut text_buffer = match self.editor.buffer_ref() {
            BufferRef::Owned(buffer) => buffer.clone(),
            BufferRef::Borrowed(buffer) => (**buffer).to_owned(),
            BufferRef::Arc(buffer) => (**buffer).clone(),
        };

        // we need to resize buffer if the text constraint has changed
        if new_constraint != self.text_constraint {
            // If the text constraint has changed
            // we need to resize the text buffer
            // get global font system
            let font_system = &mut write_font_system();
            // extract the new width and height limit
            let width = new_constraint.max_width;
            let height = new_constraint.max_height;
            // Update the text buffer size
            text_buffer.set_wrap(font_system, glyphon::Wrap::Glyph);
            text_buffer.set_size(font_system, width, height);
            text_buffer.shape_until_scroll(font_system, false);
            // and save the new constraint
            self.text_constraint = new_constraint;
        }

        TextData::from_buffer(text_buffer)
    }
}

/// A text editor component
#[tessera]
pub fn text_editor(state: Arc<RwLock<TextEditorState>>) {
    measure(Box::new(
        move |node_id, _, parent_constraint, _, metadatas| {
            let max_width: Option<f32> = match parent_constraint.width {
                DimensionValue::Fixed(w) => Some(w as f32),
                DimensionValue::Wrap => None,
                DimensionValue::Fill { max } => max.map(|m| m as f32),
            };

            let max_height: Option<f32> = match parent_constraint.height {
                DimensionValue::Fixed(h) => Some(h as f32),
                DimensionValue::Wrap => None,
                DimensionValue::Fill { max } => max.map(|m| m as f32),
            };

            let text_data = state.write().text_data(TextConstraint {
                max_width,
                max_height,
            });
            let size = text_data.size;
            let drawable = BasicDrawable::Text { data: text_data };
            if let Some(mut metadata) = metadatas.get_mut(&node_id) {
                metadata.basic_drawable = Some(drawable);
            }
            Ok(ComputedData {
                width: size[0],
                height: size[1],
            })
        },
    ));
}
