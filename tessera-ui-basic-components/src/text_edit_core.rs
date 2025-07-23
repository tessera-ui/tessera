//! Core module for text editing logic and state management in Tessera UI.
//!
//! This module provides the foundational structures and functions for building text editing components,
//! including text buffer management, selection and cursor handling, rendering logic, and keyboard event mapping.
//! It is designed to be shared across UI components via `Arc<RwLock<TextEditorState>>`, enabling consistent
//! and efficient text editing experiences.
//!
//! Typical use cases include single-line and multi-line text editors, input fields, and any UI element
//! requiring advanced text manipulation, selection, and IME support.
//!
//! The module integrates with the Tessera component system and rendering pipelines, supporting selection
//! highlighting, cursor blinking, clipboard operations, and extensible keyboard shortcuts.
//!
//! Most applications should interact with [`TextEditorState`] for state management and [`text_edit_core()`]
//! for rendering and layout within a component tree.

mod cursor;

use std::{sync::Arc, time::Instant};

use arboard::Clipboard;
use glyphon::Edit;
use parking_lot::RwLock;
use tessera_ui::{
    Color, ComputedData, DimensionValue, Dp, Px, PxPosition, focus_state::Focus, winit,
};
use tessera_ui_macros::tessera;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    pipelines::{TextCommand, TextConstraint, TextData, write_font_system},
    selection_highlight_rect::selection_highlight_rect,
};

/// Definition of a rectangular selection highlight
#[derive(Clone, Debug)]
/// Defines a rectangular region for text selection highlighting.
///
/// Used internally to represent the geometry of a selection highlight in pixel coordinates.
pub struct RectDef {
    /// The x-coordinate (in pixels) of the rectangle's top-left corner.
    pub x: Px,
    /// The y-coordinate (in pixels) of the rectangle's top-left corner.
    pub y: Px,
    /// The width (in pixels) of the rectangle.
    pub width: Px,
    /// The height (in pixels) of the rectangle.
    pub height: Px,
}

/// Types of mouse clicks
#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents the type of mouse click detected in the editor.
///
/// Used for distinguishing between single, double, and triple click actions.
pub enum ClickType {
    /// A single mouse click.
    Single,
    /// A double mouse click.
    Double,
    /// A triple mouse click.
    Triple,
}

/// Core text editing state, shared between components
/// Core state for text editing, including content, selection, cursor, and interaction state.
///
/// This struct manages the text buffer, selection, cursor position, focus, and user interaction state.
/// It is designed to be shared between UI components via an `Arc<RwLock<TextEditorState>>`.
///
/// # Example
/// ```
/// use std::sync::Arc;
/// use parking_lot::RwLock;
/// use tessera_ui::Dp;
/// use tessera_ui_basic_components::text_edit_core::{TextEditorState, text_edit_core};
///
/// let state = Arc::new(RwLock::new(TextEditorState::new(Dp(16.0), None)));
/// // Use `text_edit_core(state.clone())` inside your component tree.
/// ```
pub struct TextEditorState {
    line_height: Px,
    pub(crate) editor: glyphon::Editor<'static>,
    bink_timer: Instant,
    focus_handler: Focus,
    pub(crate) selection_color: Color,
    pub(crate) current_selection_rects: Vec<RectDef>,
    // Click tracking for double/triple click detection
    last_click_time: Option<Instant>,
    last_click_position: Option<PxPosition>,
    click_count: u32,
    is_dragging: bool,
    // For IME
    pub(crate) preedit_string: Option<String>,
}

impl TextEditorState {
    /// Creates a new `TextEditorState` with the given font size and optional line height.
    ///
    /// # Arguments
    ///
    /// * `size` - Font size in Dp.
    /// * `line_height` - Optional line height in Dp. If `None`, uses 1.2x the font size.
    ///
    /// # Example
    /// ```
    /// use tessera_ui::Dp;
    /// use tessera_ui_basic_components::text_edit_core::TextEditorState;
    /// let state = TextEditorState::new(Dp(16.0), None);
    /// ```
    pub fn new(size: Dp, line_height: Option<Dp>) -> Self {
        Self::with_selection_color(size, line_height, Color::new(0.5, 0.7, 1.0, 0.4))
    }

    /// Creates a new `TextEditorState` with a custom selection highlight color.
    ///
    /// # Arguments
    ///
    /// * `size` - Font size in Dp.
    /// * `line_height` - Optional line height in Dp.
    /// * `selection_color` - Color used for selection highlight.
    pub fn with_selection_color(size: Dp, line_height: Option<Dp>, selection_color: Color) -> Self {
        let final_line_height = line_height.unwrap_or(Dp(size.0 * 1.2));
        let line_height_px: Px = final_line_height.into();
        let mut buffer = glyphon::Buffer::new(
            &mut write_font_system(),
            glyphon::Metrics::new(size.to_pixels_f32(), line_height_px.to_f32()),
        );
        buffer.set_wrap(&mut write_font_system(), glyphon::Wrap::Glyph);
        let editor = glyphon::Editor::new(buffer);
        Self {
            line_height: line_height_px,
            editor,
            bink_timer: Instant::now(),
            focus_handler: Focus::new(),
            selection_color,
            current_selection_rects: Vec::new(),
            last_click_time: None,
            last_click_position: None,
            click_count: 0,
            is_dragging: false,
            preedit_string: None,
        }
    }

    /// Returns the line height in pixels.
    pub fn line_height(&self) -> Px {
        self.line_height
    }

    /// Returns the current text buffer as `TextData`, applying the given layout constraints.
    ///
    /// # Arguments
    ///
    /// * `constraint` - Layout constraints for text rendering.
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

    /// Returns a reference to the internal focus handler.
    pub fn focus_handler(&self) -> &Focus {
        &self.focus_handler
    }

    /// Returns a mutable reference to the internal focus handler.
    pub fn focus_handler_mut(&mut self) -> &mut Focus {
        &mut self.focus_handler
    }

    /// Returns a reference to the underlying `glyphon::Editor`.
    pub fn editor(&self) -> &glyphon::Editor<'static> {
        &self.editor
    }

    /// Returns a mutable reference to the underlying `glyphon::Editor`.
    pub fn editor_mut(&mut self) -> &mut glyphon::Editor<'static> {
        &mut self.editor
    }

    /// Returns the current blink timer instant (for cursor blinking).
    pub fn bink_timer(&self) -> Instant {
        self.bink_timer
    }

    /// Resets the blink timer to the current instant.
    pub fn update_bink_timer(&mut self) {
        self.bink_timer = Instant::now();
    }

    /// Returns the current selection highlight color.
    pub fn selection_color(&self) -> Color {
        self.selection_color
    }

    /// Returns a reference to the current selection rectangles.
    pub fn current_selection_rects(&self) -> &Vec<RectDef> {
        &self.current_selection_rects
    }

    /// Sets the selection highlight color.
    ///
    /// # Arguments
    ///
    /// * `color` - The new selection color.
    pub fn set_selection_color(&mut self, color: Color) {
        self.selection_color = color;
    }

    /// Handles a mouse click event and determines the click type (single, double, triple).
    ///
    /// Used for text selection and word/line selection logic.
    ///
    /// # Arguments
    ///
    /// * `position` - The position of the click in pixels.
    /// * `timestamp` - The time the click occurred.
    ///
    /// # Returns
    ///
    /// The detected [`ClickType`].
    pub fn handle_click(&mut self, position: PxPosition, timestamp: Instant) -> ClickType {
        const DOUBLE_CLICK_TIME_MS: u128 = 500; // 500ms for double click
        const CLICK_DISTANCE_THRESHOLD: Px = Px(5); // 5 pixels tolerance for position

        let click_type = if let (Some(last_time), Some(last_pos)) =
            (self.last_click_time, self.last_click_position)
        {
            let time_diff = timestamp.duration_since(last_time).as_millis();
            let distance = (position.x - last_pos.x).abs() + (position.y - last_pos.y).abs();

            if time_diff <= DOUBLE_CLICK_TIME_MS && distance <= CLICK_DISTANCE_THRESHOLD.abs() {
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

    /// Starts a drag operation (for text selection).
    pub fn start_drag(&mut self) {
        self.is_dragging = true;
    }

    /// Returns `true` if a drag operation is in progress.
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Stops the current drag operation.
    pub fn stop_drag(&mut self) {
        self.is_dragging = false;
    }

    /// Returns the last click position, if any.
    pub fn last_click_position(&self) -> Option<PxPosition> {
        self.last_click_position
    }

    /// Updates the last click position (used for drag tracking).
    ///
    /// # Arguments
    ///
    /// * `position` - The new last click position.
    pub fn update_last_click_position(&mut self, position: PxPosition) {
        self.last_click_position = Some(position);
    }
}

#[tessera]
/// Core text editing component for rendering text, selection, and cursor.
///
/// This component is responsible for rendering the text buffer, selection highlights, and cursor.
/// It does not handle user events directly; instead, it is intended to be used inside a container
/// that manages user interaction and passes state updates via `TextEditorState`.
///
/// # Arguments
///
/// * `state` - Shared state for the text editor, typically wrapped in `Arc<RwLock<...>>`.
///
/// # Example
/// ```
/// use std::sync::Arc;
/// use parking_lot::RwLock;
/// use tessera_ui::Dp;
/// use tessera_ui_basic_components::text_edit_core::{TextEditorState, text_edit_core};
///
/// let state = Arc::new(RwLock::new(TextEditorState::new(Dp(16.0), None)));
/// text_edit_core(state.clone());
/// ```
pub fn text_edit_core(state: Arc<RwLock<TextEditorState>>) {
    // text rendering with constraints from parent container
    {
        let state_clone = state.clone();
        measure(Box::new(move |input| {
            // surface provides constraints that should be respected for text layout
            let max_width_pixels: Option<Px> = match input.parent_constraint.width {
                DimensionValue::Fixed(w) => Some(w),
                DimensionValue::Wrap { max, .. } => max,
                DimensionValue::Fill { max, .. } => max,
            };

            // For proper scrolling behavior, we need to respect height constraints
            // When max height is specified, content should be clipped and scrollable
            let max_height_pixels: Option<Px> = match input.parent_constraint.height {
                DimensionValue::Fixed(h) => Some(h), // Respect explicit fixed heights
                DimensionValue::Wrap { max, .. } => max, // Respect max height for wrapping
                DimensionValue::Fill { max, .. } => max,
            };

            let text_data = state_clone.write().text_data(TextConstraint {
                max_width: max_width_pixels.map(|px| px.to_f32()),
                max_height: max_height_pixels.map(|px| px.to_f32()),
            });

            // Calculate selection rectangles
            let mut selection_rects = Vec::new();
            let selection_bounds = state_clone.read().editor.selection_bounds();
            if let Some((start, end)) = selection_bounds {
                state_clone.read().editor.with_buffer(|buffer| {
                    for run in buffer.layout_runs() {
                        let line_i = run.line_i;
                        let _line_y = run.line_y; // Px
                        let line_top = Px(run.line_top as i32); // Px
                        let line_height = Px(run.line_height as i32); // Px

                        // Highlight selection
                        if line_i >= start.line && line_i <= end.line {
                            let mut range_opt: Option<(Px, Px)> = None;
                            for glyph in run.glyphs.iter() {
                                // Guess x offset based on characters
                                let cluster = &run.text[glyph.start..glyph.end];
                                let total = cluster.grapheme_indices(true).count();
                                let mut c_x = Px(glyph.x as i32);
                                let c_w = Px((glyph.w / total as f32) as i32);
                                for (i, c) in cluster.grapheme_indices(true) {
                                    let c_start = glyph.start + i;
                                    let c_end = glyph.start + i + c.len();
                                    if (start.line != line_i || c_end > start.index)
                                        && (end.line != line_i || c_start < end.index)
                                    {
                                        range_opt = match range_opt.take() {
                                            Some((min_val, max_val)) => Some((
                                                // Renamed to avoid conflict
                                                min_val.min(c_x),
                                                max_val.max(c_x + c_w),
                                            )),
                                            None => Some((c_x, c_x + c_w)),
                                        };
                                    } else if let Some((min_val, max_val)) = range_opt.take() {
                                        // Renamed
                                        selection_rects.push(RectDef {
                                            x: min_val,
                                            y: line_top,
                                            width: (max_val - min_val).max(Px(0)),
                                            height: line_height,
                                        });
                                    }
                                    c_x += c_w;
                                }
                            }

                            if run.glyphs.is_empty() && end.line > line_i {
                                // Highlight all of internal empty lines
                                range_opt =
                                    Some((Px(0), buffer.size().0.map_or(Px(0), |w| Px(w as i32))));
                            }

                            if let Some((mut min_val, mut max_val)) = range_opt.take() {
                                // Renamed
                                if end.line > line_i {
                                    // Draw to end of line
                                    if run.rtl {
                                        min_val = Px(0);
                                    } else {
                                        max_val = buffer.size().0.map_or(Px(0), |w| Px(w as i32));
                                    }
                                }
                                selection_rects.push(RectDef {
                                    x: min_val,
                                    y: line_top,
                                    width: (max_val - min_val).max(Px(0)),
                                    height: line_height,
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
                if let Some(rect_node_id) = input.children_ids.get(i).copied() {
                    let _ = input.measure_child(rect_node_id, input.parent_constraint);
                    input.place_child(rect_node_id, PxPosition::new(rect_def.x, rect_def.y));
                }
            }

            // --- Filter and clip selection rects to visible area ---
            // Only show highlight rects that are (partially) within the visible area
            let visible_x0 = Px(0);
            let visible_y0 = Px(0);
            let visible_x1 = max_width_pixels.unwrap_or(Px(i32::MAX));
            let visible_y1 = max_height_pixels.unwrap_or(Px(i32::MAX));
            selection_rects = selection_rects
                .into_iter()
                .filter_map(|mut rect| {
                    let rect_x1 = rect.x + rect.width;
                    let rect_y1 = rect.y + rect.height;
                    // If completely outside visible area, skip
                    if rect_x1 <= visible_x0
                        || rect.y >= visible_y1
                        || rect.x >= visible_x1
                        || rect_y1 <= visible_y0
                    {
                        None
                    } else {
                        // Clip to visible area
                        let new_x = rect.x.max(visible_x0);
                        let new_y = rect.y.max(visible_y0);
                        let new_x1 = rect_x1.min(visible_x1);
                        let new_y1 = rect_y1.min(visible_y1);
                        rect.x = new_x;
                        rect.y = new_y;
                        rect.width = (new_x1 - new_x).max(Px(0));
                        rect.height = (new_y1 - new_y).max(Px(0));
                        Some(rect)
                    }
                })
                .collect();
            // Write filtered rects to state
            state_clone.write().current_selection_rects = selection_rects;

            // Handle cursor positioning (cursor comes after selection rects)
            if let Some(cursor_pos_raw) = state_clone.read().editor.cursor_position() {
                let cursor_pos = PxPosition::new(Px(cursor_pos_raw.0), Px(cursor_pos_raw.1));
                let cursor_node_index = selection_rects_len;
                if let Some(cursor_node_id) = input.children_ids.get(cursor_node_index).copied() {
                    let _ = input.measure_child(cursor_node_id, input.parent_constraint);
                    input.place_child(cursor_node_id, cursor_pos);
                }
            }

            let drawable = TextCommand {
                data: text_data.clone(),
            };
            input.metadata_mut().push_draw_command(drawable);

            // Return constrained size - respect maximum height to prevent overflow
            let constrained_height = if let Some(max_h) = max_height_pixels {
                text_data.size[1].min(max_h.abs())
            } else {
                text_data.size[1]
            };

            Ok(ComputedData {
                width: text_data.size[0].into(),
                height: constrained_height.into(),
            })
        }));
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
/// Maps a keyboard event to a list of text editing actions for the editor.
///
/// This function translates keyboard input (including modifiers) into editing actions
/// such as character insertion, deletion, navigation, and clipboard operations.
///
/// # Arguments
///
/// * `key_event` - The keyboard event to map.
/// * `key_modifiers` - The current keyboard modifier state.
/// * `editor` - Reference to the editor for clipboard operations.
///
/// # Returns
///
/// An optional vector of `glyphon::Action` to be applied to the editor.
pub fn map_key_event_to_action(
    key_event: winit::event::KeyEvent,
    key_modifiers: winit::keyboard::ModifiersState,
    editor: &glyphon::Editor,
) -> Option<Vec<glyphon::Action>> {
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
                NamedKey::Home => Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Home)]),
                NamedKey::End => Some(vec![glyphon::Action::Motion(cosmic_text::Motion::End)]),
                NamedKey::Space => Some(vec![glyphon::Action::Insert(' ')]),
                _ => None,
            }
        }
        winit::keyboard::Key::Character(s) => {
            let is_ctrl = key_modifiers.control_key() || key_modifiers.super_key();
            if is_ctrl {
                match s.to_lowercase().as_str() {
                    "c" => {
                        if let Some(text) = editor.copy_selection() {
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(text);
                            }
                        }
                        return None;
                    }
                    "v" => {
                        if let Ok(mut clipboard) = Clipboard::new() {
                            if let Ok(text) = clipboard.get_text() {
                                return Some(text.chars().map(glyphon::Action::Insert).collect());
                            }
                        }
                        return None;
                    }
                    "x" => {
                        if let Some(text) = editor.copy_selection() {
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let _ = clipboard.set_text(text);
                            }
                            // Use Backspace action to delete selection
                            return Some(vec![glyphon::Action::Backspace]);
                        }
                        return None;
                    }
                    _ => {}
                }
            }
            Some(s.chars().map(glyphon::Action::Insert).collect::<Vec<_>>())
        }
        _ => None,
    }
}
