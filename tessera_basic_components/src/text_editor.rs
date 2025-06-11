mod cursor;

use std::{sync::Arc, time::Instant};

use glyphon::{Edit, cosmic_text::BufferRef};
use parking_lot::RwLock;

use tessera::{
    BasicDrawable, ComponentNodeMetaData, ComputedData, DimensionValue, Dp, TextConstraint,
    TextData, measure_node, place_node, winit, write_font_system,
};
use tessera_macros::tessera;

pub struct TextEditorState {
    line_height: u32,
    pub(crate) editor: glyphon::Editor<'static>,
    bink_timer: Instant,
}

impl TextEditorState {
    /// Creates a new `TextEditorState` with the given size, line height, and text constraint.
    pub fn new(size: Dp, line_height: Dp) -> Self {
        let line_height = line_height.to_pixels_u32();
        let mut buffer = glyphon::Buffer::new(
            &mut write_font_system(),
            glyphon::Metrics::new(size.to_pixels_f32(), line_height as f32),
        );
        buffer.set_wrap(&mut write_font_system(), glyphon::Wrap::Glyph);
        let editor = glyphon::Editor::new(buffer);
        Self {
            line_height,
            editor,
            bink_timer: Instant::now(),
        }
    }

    /// Returns the editor instance.
    pub fn line_height(&self) -> u32 {
        self.line_height
    }

    fn text_data(&mut self, constraint: TextConstraint) -> TextData {
        self.editor.with_buffer_mut(|buffer| {
            buffer.set_size(
                &mut write_font_system(),
                constraint.max_width,
                constraint.max_height,
            );
            buffer.shape_until_scroll(&mut write_font_system(), false);
        });

        let text_buffer = match self.editor.buffer_ref() {
            BufferRef::Owned(buffer) => buffer.clone(),
            BufferRef::Borrowed(buffer) => (**buffer).to_owned(),
            BufferRef::Arc(buffer) => (**buffer).clone(),
        };

        TextData::from_buffer(text_buffer)
    }
}

/// A text editor component
#[tessera]
pub fn text_editor(state: Arc<RwLock<TextEditorState>>) {
    {
        let state: Arc<parking_lot::lock_api::RwLock<parking_lot::RawRwLock, TextEditorState>> =
            state.clone();
        measure(Box::new(
            move |node_id, tree, parent_constraint, children_node_ids, metadatas| {
                // Get the pos of the cursor
                let cursor_pos = state
                    .read()
                    .editor
                    .cursor_position()
                    .map(|(x, y)| [x as u32, y as u32])
                    .unwrap_or([0, 0]);
                // Place the cursor node
                let cursor_node_id = children_node_ids[0]; // text editor only has one child, the cursor
                let _ = measure_node(cursor_node_id, parent_constraint, tree, metadatas);
                place_node(cursor_node_id, cursor_pos, metadatas);
                // Get the text constraint from the state
                let max_width: Option<f32> = match parent_constraint.width {
                    DimensionValue::Fixed(w) => Some(w as f32),
                    DimensionValue::Wrap => None,
                    DimensionValue::Fill { max } => max.map(|m| m as f32),
                };
                // Get the max height from the parent constraint
                let max_height: Option<f32> = match parent_constraint.height {
                    DimensionValue::Fixed(h) => Some(h as f32),
                    DimensionValue::Wrap => None,
                    DimensionValue::Fill { max } => max.map(|m| m as f32),
                };
                // Build text data with the current text constraint
                let text_data = state.write().text_data(TextConstraint {
                    max_width,
                    max_height,
                });
                // We use its size as the computed data
                let size = text_data.size;
                // Add text drawable to render
                let drawable = BasicDrawable::Text { data: text_data };
                if let Some(mut metadata) = metadatas.get_mut(&node_id) {
                    metadata.basic_drawable = Some(drawable);
                } else {
                    let default_meta = ComponentNodeMetaData {
                        basic_drawable: Some(drawable),
                        ..Default::default()
                    };
                    metadatas.insert(node_id, default_meta);
                }
                // Return the computed data
                Ok(ComputedData {
                    width: size[0],
                    height: size[1],
                })
            },
        ));
    }

    cursor::cursor(state.read().line_height(), state.read().bink_timer);

    {
        let state = state.clone();
        state_handler(Box::new(move |input| {
            // transform the input event to editor action
            let actions = input
                .keyboard_events
                .iter()
                .cloned()
                .filter_map(map_key_event_to_action)
                .flatten();
            for action in actions {
                // Apply the action to the editor
                state
                    .write()
                    .editor
                    .action(&mut write_font_system(), action);
            }
        }));
    }
}

fn map_key_event_to_action(key_event: winit::event::KeyEvent) -> Option<Vec<glyphon::Action>> {
    match key_event.state {
        winit::event::ElementState::Pressed => {}
        winit::event::ElementState::Released => return None, // We only handle pressed events
    }

    match key_event.logical_key {
        winit::keyboard::Key::Named(named_key) => {
            use glyphon::cosmic_text;
            use winit::keyboard::NamedKey;

            match named_key {
                NamedKey::Backspace => Some(vec![glyphon::Action::Backspace]),
                NamedKey::Delete => Some(vec![glyphon::Action::Delete]),
                NamedKey::Enter => Some(vec![glyphon::Action::Enter]),
                NamedKey::Escape => Some(vec![glyphon::Action::Escape]),
                NamedKey::Tab => Some(vec![glyphon::Action::Insert(' '); 4]),
                NamedKey::ArrowLeft => {
                    Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Left)])
                }
                NamedKey::ArrowRight => {
                    Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Right)])
                }
                NamedKey::ArrowUp => Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Up)]),
                NamedKey::ArrowDown => {
                    Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Down)])
                }
                NamedKey::Space => Some(vec![glyphon::Action::Insert(' ')]),
                _ => None,
            }
        }
        winit::keyboard::Key::Character(input) => Some(
            input
                .chars()
                .map(glyphon::Action::Insert)
                .collect::<Vec<_>>(),
        ),
        _ => None,
    }
}
