mod cursor;

use std::{sync::Arc, time::Instant};

use glyphon::Edit;
use parking_lot::RwLock;

use tessera::{
    BasicDrawable, ComponentNodeMetaData, ComputedData, DimensionValue, Dp, TextConstraint,
    TextData, focus_state::Focus, measure_node, place_node, winit, write_font_system,
};
use tessera_macros::tessera;

/// Core text editing state, shared between components
pub struct TextEditorState {
    line_height: u32,
    pub(crate) editor: glyphon::Editor<'static>,
    bink_timer: Instant,
    focus_handler: Focus,
}

impl TextEditorState {
    pub fn new(size: Dp, line_height: Dp) -> Self {
        let line_height_pixels = line_height.to_pixels_u32();
        let mut buffer = glyphon::Buffer::new(
            &mut write_font_system(),
            glyphon::Metrics::new(size.to_pixels_f32(), line_height_pixels as f32),
        );
        buffer.set_wrap(&mut write_font_system(), glyphon::Wrap::Glyph);
        let editor = glyphon::Editor::new(buffer);
        Self {
            line_height: line_height_pixels,
            editor,
            bink_timer: Instant::now(),
            focus_handler: Focus::new(),
        }
    }

    pub fn line_height(&self) -> u32 {
        self.line_height
    }

    pub fn text_data(&mut self, constraint: TextConstraint) -> TextData {
        self.editor.with_buffer_mut(|buffer| {
            buffer.set_size(
                &mut write_font_system(),
                constraint.max_width,
                constraint.max_height,
            );
            buffer.shape_until_scroll(&mut write_font_system(), false);
        });

        let text_buffer = match self.editor.buffer_ref() {
            glyphon::cosmic_text::BufferRef::Owned(buffer) => buffer.clone(),
            glyphon::cosmic_text::BufferRef::Borrowed(buffer) => (**buffer).to_owned(),
            glyphon::cosmic_text::BufferRef::Arc(buffer) => (**buffer).clone(),
        };

        TextData::from_buffer(text_buffer)
    }

    pub fn focus_handler(&self) -> &Focus {
        &self.focus_handler
    }

    pub fn focus_handler_mut(&mut self) -> &mut Focus {
        &mut self.focus_handler
    }

    pub fn editor(&self) -> &glyphon::Editor<'static> {
        &self.editor
    }

    pub fn editor_mut(&mut self) -> &mut glyphon::Editor<'static> {
        &mut self.editor
    }

    pub fn bink_timer(&self) -> Instant {
        self.bink_timer
    }

    pub fn update_bink_timer(&mut self) {
        self.bink_timer = Instant::now();
    }
}

/// Core text editing component - handles text rendering and cursor, no events
///
/// This component is designed to be used inside a container (like surface) that
/// provides the proper size constraints and handles user interaction events.
#[tessera]
pub fn text_edit_core(state: Arc<RwLock<TextEditorState>>) {
    // Text rendering with constraints from parent container
    {
        let state_clone = state.clone();
        measure(Box::new(
            move |node_id, tree, parent_constraint, children_node_ids, metadatas| {
                // Surface provides constraints that should be respected for text layout
                let max_width_pixels: Option<f32> = match parent_constraint.width {
                    DimensionValue::Fixed(w) => Some(w as f32),
                    DimensionValue::Wrap { max, .. } => max.map(|m| m as f32),
                    DimensionValue::Fill { max, .. } => max.map(|m| m as f32),
                };

                // Critical insight: For text editors, we should let text flow naturally and
                // let the surface adjust, rather than cramming text into small spaces
                let max_height_pixels: Option<f32> = match parent_constraint.height {
                    DimensionValue::Fixed(h) => Some(h as f32), // Respect explicit fixed heights
                    DimensionValue::Wrap { .. } => None,        // Let text determine natural height
                    DimensionValue::Fill { max, .. } => max.map(|m| m as f32),
                };

                let text_data = state_clone.write().text_data(TextConstraint {
                    max_width: max_width_pixels,
                    max_height: max_height_pixels,
                });

                // Handle cursor positioning
                let cursor_pos = state_clone
                    .read()
                    .editor
                    .cursor_position()
                    .map(|(x, y)| [x as u32, y as u32])
                    .unwrap_or([0, 0]);

                if let Some(cursor_node_id) = children_node_ids.first().copied() {
                    let _ = measure_node(cursor_node_id, parent_constraint, tree, metadatas);
                    place_node(cursor_node_id, cursor_pos, metadatas);
                }

                let drawable = BasicDrawable::Text {
                    data: text_data.clone(),
                };
                if let Some(mut metadata) = metadatas.get_mut(&node_id) {
                    metadata.basic_drawable = Some(drawable);
                } else {
                    metadatas.insert(
                        node_id,
                        ComponentNodeMetaData {
                            basic_drawable: Some(drawable),
                            ..Default::default()
                        },
                    );
                }

                // Return actual text size - container will handle minimum constraints
                Ok(ComputedData {
                    width: text_data.size[0],
                    height: text_data.size[1],
                })
            },
        ));
    }

    // Cursor rendering (only when focused)
    if state.read().focus_handler().is_focused() {
        cursor::cursor(state.read().line_height(), state.read().bink_timer());
    }
}

/// Map keyboard events to text editing actions
pub fn map_key_event_to_action(key_event: winit::event::KeyEvent) -> Option<Vec<glyphon::Action>> {
    match key_event.state {
        winit::event::ElementState::Pressed => {}
        winit::event::ElementState::Released => return None,
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
