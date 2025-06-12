mod cursor;

use std::{sync::Arc, time::Instant};

use glyphon::Edit;
use parking_lot::RwLock;
use unicode_segmentation::UnicodeSegmentation;

use crate::selection_highlight_rect::selection_highlight_rect;
use tessera::{
    BasicDrawable, ComponentNodeMetaData, ComputedData, DimensionValue, Dp, TextConstraint,
    TextData, focus_state::Focus, measure_node, place_node, winit, write_font_system,
};
use tessera_macros::tessera;

/// Definition of a rectangular selection highlight
#[derive(Clone, Debug)]
pub struct RectDef {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Types of mouse clicks
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClickType {
    Single,
    Double,
    Triple,
}

/// Core text editing state, shared between components
pub struct TextEditorState {
    line_height: u32,
    pub(crate) editor: glyphon::Editor<'static>,
    bink_timer: Instant,
    focus_handler: Focus,
    pub(crate) selection_color: [f32; 4],
    pub(crate) current_selection_rects: Vec<RectDef>,
    // Click tracking for double/triple click detection
    last_click_time: Option<Instant>,
    last_click_position: Option<[i32; 2]>,
    click_count: u32,
    is_dragging: bool,
}

impl TextEditorState {
    pub fn new(size: Dp, line_height: Dp) -> Self {
        Self::with_selection_color(size, line_height, [0.5, 0.7, 1.0, 0.4])
    }

    pub fn with_selection_color(size: Dp, line_height: Dp, selection_color: [f32; 4]) -> Self {
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
            selection_color,
            current_selection_rects: Vec::new(),
            last_click_time: None,
            last_click_position: None,
            click_count: 0,
            is_dragging: false,
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

    pub fn selection_color(&self) -> [f32; 4] {
        self.selection_color
    }

    pub fn current_selection_rects(&self) -> &Vec<RectDef> {
        &self.current_selection_rects
    }

    pub fn set_selection_color(&mut self, color: [f32; 4]) {
        self.selection_color = color;
    }

    /// Handle a click event and determine the click type (single, double, triple)
    pub fn handle_click(&mut self, position: [i32; 2], timestamp: Instant) -> ClickType {
        const DOUBLE_CLICK_TIME_MS: u128 = 500; // 500ms for double click
        const CLICK_DISTANCE_THRESHOLD: i32 = 5; // 5 pixels tolerance for position

        let click_type = if let (Some(last_time), Some(last_pos)) =
            (self.last_click_time, self.last_click_position)
        {
            let time_diff = timestamp.duration_since(last_time).as_millis();
            let distance = (position[0] - last_pos[0]).abs() + (position[1] - last_pos[1]).abs();

            if time_diff <= DOUBLE_CLICK_TIME_MS && distance <= CLICK_DISTANCE_THRESHOLD {
                self.click_count += 1;
                match self.click_count {
                    2 => ClickType::Double,
                    3 => {
                        self.click_count = 0; // Reset after triple click
                        ClickType::Triple
                    }
                    _ => ClickType::Single,
                }
            } else {
                self.click_count = 1;
                ClickType::Single
            }
        } else {
            self.click_count = 1;
            ClickType::Single
        };

        self.last_click_time = Some(timestamp);
        self.last_click_position = Some(position);
        self.is_dragging = false;

        click_type
    }

    /// Start drag operation
    pub fn start_drag(&mut self) {
        self.is_dragging = true;
    }

    /// Check if currently dragging
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Stop drag operation
    pub fn stop_drag(&mut self) {
        self.is_dragging = false;
    }

    /// Get last click position
    pub fn last_click_position(&self) -> Option<[i32; 2]> {
        self.last_click_position
    }

    /// Update last click position (for drag tracking)
    pub fn update_last_click_position(&mut self, position: [i32; 2]) {
        self.last_click_position = Some(position);
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

                // Calculate selection rectangles
                let mut selection_rects = Vec::new();
                let selection_bounds = state_clone.read().editor.selection_bounds();
                if let Some((start, end)) = selection_bounds {
                    state_clone.read().editor.with_buffer(|buffer| {
                        for run in buffer.layout_runs() {
                            let line_i = run.line_i;
                            let _line_y = run.line_y;
                            let line_top = run.line_top;
                            let line_height = run.line_height;

                            // Highlight selection
                            if line_i >= start.line && line_i <= end.line {
                                let mut range_opt = None;
                                for glyph in run.glyphs.iter() {
                                    // Guess x offset based on characters
                                    let cluster = &run.text[glyph.start..glyph.end];
                                    let total = cluster.grapheme_indices(true).count();
                                    let mut c_x = glyph.x;
                                    let c_w = glyph.w / total as f32;
                                    for (i, c) in cluster.grapheme_indices(true) {
                                        let c_start = glyph.start + i;
                                        let c_end = glyph.start + i + c.len();
                                        if (start.line != line_i || c_end > start.index)
                                            && (end.line != line_i || c_start < end.index)
                                        {
                                            range_opt = match range_opt.take() {
                                                Some((min, max)) => Some((
                                                    std::cmp::min(min, c_x as i32),
                                                    std::cmp::max(max, (c_x + c_w) as i32),
                                                )),
                                                None => Some((c_x as i32, (c_x + c_w) as i32)),
                                            };
                                        } else if let Some((min, max)) = range_opt.take() {
                                            selection_rects.push(RectDef {
                                                x: min,
                                                y: line_top as i32,
                                                width: std::cmp::max(0, max - min) as u32,
                                                height: line_height as u32,
                                            });
                                        }
                                        c_x += c_w;
                                    }
                                }

                                if run.glyphs.is_empty() && end.line > line_i {
                                    // Highlight all of internal empty lines
                                    range_opt = Some((0, buffer.size().0.unwrap_or(0.0) as i32));
                                }

                                if let Some((mut min, mut max)) = range_opt.take() {
                                    if end.line > line_i {
                                        // Draw to end of line
                                        if run.rtl {
                                            min = 0;
                                        } else {
                                            max = buffer.size().0.unwrap_or(0.0) as i32;
                                        }
                                    }
                                    selection_rects.push(RectDef {
                                        x: min,
                                        y: line_top as i32,
                                        width: std::cmp::max(0, max - min) as u32,
                                        height: line_height as u32,
                                    });
                                }
                            }
                        }
                    });
                }

                // Record length before moving
                let selection_rects_len = selection_rects.len();

                // Handle selection rectangle positioning
                for (i, rect_def) in selection_rects.iter().enumerate() {
                    if let Some(rect_node_id) = children_node_ids.get(i).copied() {
                        let _ = measure_node(rect_node_id, parent_constraint, tree, metadatas);
                        place_node(
                            rect_node_id,
                            [rect_def.x as u32, rect_def.y as u32],
                            metadatas,
                        );
                    }
                }

                // Store calculated selection rectangles
                state_clone.write().current_selection_rects = selection_rects;

                // Handle cursor positioning (cursor comes after selection rects)
                let cursor_pos = state_clone
                    .read()
                    .editor
                    .cursor_position()
                    .map(|(x, y)| [x as u32, y as u32])
                    .unwrap_or([0, 0]);

                let cursor_node_index = selection_rects_len;
                if let Some(cursor_node_id) = children_node_ids.get(cursor_node_index).copied() {
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

    // Selection highlighting
    {
        let (rect_definitions, color_for_selection) = {
            let guard = state.read();
            (guard.current_selection_rects.clone(), guard.selection_color)
        };

        for def in rect_definitions {
            selection_highlight_rect(def.width, def.height, color_for_selection);
        }
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
