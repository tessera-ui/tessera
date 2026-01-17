//! Core module for text editing logic and state management in Tessera UI.
//!
//! This module provides the foundational structures and functions for building
//! text editing components, including text buffer management, selection and
//! cursor handling, rendering logic, and keyboard event mapping. It is designed
//! to be shared across UI components via the `TextEditorController` wrapper,
//! enabling consistent and thread-safe access to editor state.
//! and efficient text editing experiences.
//!
//! Typical use cases include single-line and multi-line text editors, input
//! fields, and any UI element requiring advanced text manipulation, selection,
//! and IME support.
//!
//! The module integrates with the Tessera component system and rendering
//! pipelines, supporting selection highlighting, cursor blinking, clipboard
//! operations, and extensible keyboard shortcuts.
//!
//! Most applications should interact with [`TextEditorController`] for state
//! management and [`text_edit_core()`] for rendering and layout within a
//! component tree.

mod cursor;

use std::{sync::Arc, time::Instant};

use glyphon::{
    Cursor, Edit,
    cosmic_text::{self, Selection},
};
use tessera_platform::clipboard;
use tessera_ui::{
    Color, ComputedData, DimensionValue, Dp, MeasurementError, Px, PxPosition, State,
    focus_state::Focus,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    tessera, winit,
};
use winit::keyboard::NamedKey;

use crate::{
    pipelines::text::{
        command::{TextCommand, TextConstraint},
        pipeline::{TextData, write_font_system},
    },
    selection_highlight_rect::selection_highlight_rect,
    text_edit_core::cursor::CURSOR_WIDRH,
};

/// Display-only text transform applied to the text content before rendering
/// (e.g., masking or formatting without changing the underlying buffer).
pub type DisplayTransform = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// Definition of a rectangular selection highlight
#[derive(Clone, Debug)]
/// Defines a rectangular region for text selection highlighting.
///
/// Used internally to represent the geometry of a selection highlight in pixel
/// coordinates.
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
/// Core state for text editing, including content, selection, cursor, and
/// interaction state.
///
/// This struct manages the text buffer, selection, cursor position, focus, and
/// user interaction state. It is designed to be shared between UI components
/// via a `TextEditorController`.
pub struct TextEditorController {
    line_height: Px,
    pub(crate) editor: glyphon::Editor<'static>,
    blink_timer: Instant,
    focus_handler: Focus,
    pub(crate) selection_color: Color,
    pub(crate) text_color: Color,
    pub(crate) cursor_color: Color,
    pub(crate) current_selection_rects: Vec<RectDef>,
    current_text_data: Option<TextData>,
    current_layout_buffer: Option<glyphon::Buffer>,
    display_transform: Option<DisplayTransform>,
    layout_version: u64,
    // Click tracking for double/triple click detection
    last_click_time: Option<Instant>,
    last_click_position: Option<PxPosition>,
    click_count: u32,
    is_dragging: bool,
    // For IME
    pub(crate) preedit_string: Option<String>,
}

impl TextEditorController {
    /// Creates a new `TextEditorController` with the given font size and
    /// optional line height.
    ///
    /// # Arguments
    ///
    /// * `size` - Font size in Dp.
    /// * `line_height` - Optional line height in Dp. If `None`, uses 1.2x the
    ///   font size.
    pub fn new(size: Dp, line_height: Option<Dp>) -> Self {
        Self::with_selection_color(size, line_height, Color::new(0.5, 0.7, 1.0, 0.4))
    }

    /// Creates a new `TextEditorController` with a custom selection highlight
    /// color.
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
        let text_color = Color::BLACK;
        let cursor_color = Color::BLACK;
        Self {
            line_height: line_height_px,
            editor,
            blink_timer: Instant::now(),
            focus_handler: Focus::new(),
            selection_color,
            text_color,
            cursor_color,
            current_selection_rects: Vec::new(),
            current_text_data: None,
            current_layout_buffer: None,
            display_transform: None,
            layout_version: 0,
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

    // Returns the current text buffer as `TextData`, applying the given layout
    // constraints.
    fn text_data(&mut self, constraint: TextConstraint) -> TextData {
        self.editor.with_buffer_mut(|buffer| {
            buffer.set_size(
                &mut write_font_system(),
                constraint.max_width,
                constraint.max_height,
            );
            buffer.shape_until_scroll(&mut write_font_system(), false);
        });

        let text_buffer = if let Some(transform) = self.display_transform.as_ref() {
            let metrics = self.editor.with_buffer(|buffer| buffer.metrics());
            let content = editor_content(&self.editor);
            let display_text = transform(&content);
            build_display_buffer(
                &display_text,
                self.text_color,
                metrics.font_size,
                metrics.line_height,
                &constraint,
            )
        } else {
            match self.editor.buffer_ref() {
                glyphon::cosmic_text::BufferRef::Owned(buffer) => buffer.clone(),
                glyphon::cosmic_text::BufferRef::Borrowed(buffer) => (**buffer).to_owned(),
                glyphon::cosmic_text::BufferRef::Arc(buffer) => (**buffer).clone(),
            }
        };

        let text_data = TextData::from_buffer(text_buffer.clone());
        self.current_layout_buffer = Some(text_buffer);
        self.current_text_data = Some(text_data.clone());
        text_data
    }

    // Returns a reference to the internal focus handler.
    pub(crate) fn focus_handler(&self) -> &Focus {
        &self.focus_handler
    }

    // Returns a mutable reference to the internal focus handler.
    pub(crate) fn focus_handler_mut(&mut self) -> &mut Focus {
        &mut self.focus_handler
    }

    /// Returns a reference to the underlying `glyphon::Editor`.
    pub fn editor(&self) -> &glyphon::Editor<'static> {
        &self.editor
    }

    /// Mutates the underlying `glyphon::Editor` and refreshes layout state.
    pub fn with_editor_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut glyphon::Editor<'static>) -> R,
    {
        let result = f(&mut self.editor);
        self.bump_layout_version();
        result
    }

    // Returns the current blink timer instant (for cursor blinking).
    fn blink_timer(&self) -> Instant {
        self.blink_timer
    }

    /// Returns the current selection highlight color.
    pub fn selection_color(&self) -> Color {
        self.selection_color
    }

    /// Sets the selection highlight color.
    ///
    /// # Arguments
    ///
    /// * `color` - The new selection color.
    pub fn set_selection_color(&mut self, color: Color) {
        self.selection_color = color;
    }

    /// Returns the current text color.
    pub fn text_color(&self) -> Color {
        self.text_color
    }

    /// Sets the text color used by the editor.
    pub fn set_text_color(&mut self, color: Color) {
        if self.text_color == color {
            return;
        }
        self.text_color = color;
        let current_text = editor_content(&self.editor);
        self.set_text(&current_text);
    }

    /// Returns the cursor color.
    pub fn cursor_color(&self) -> Color {
        self.cursor_color
    }

    /// Sets the cursor color used by the editor.
    pub fn set_cursor_color(&mut self, color: Color) {
        self.cursor_color = color;
    }

    /// Sets a display transform applied when rendering text.
    pub fn set_display_transform(&mut self, transform: Option<DisplayTransform>) {
        let should_update = match (&self.display_transform, &transform) {
            (None, None) => false,
            (Some(current), Some(next)) => !Arc::ptr_eq(current, next),
            _ => true,
        };
        if should_update {
            self.display_transform = transform;
            self.bump_layout_version();
        }
    }

    /// Returns whether a display transform is active.
    pub fn display_transform_active(&self) -> bool {
        self.display_transform.is_some()
    }

    // Handles a mouse click event and determines the click type (single,
    // double, triple).
    pub(crate) fn handle_click(&mut self, position: PxPosition, timestamp: Instant) -> ClickType {
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

    // Starts a drag operation (for text selection).
    pub(crate) fn start_drag(&mut self) {
        self.is_dragging = true;
    }

    /// Returns `true` if a drag operation is in progress.
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    // Stops the current drag operation.
    pub(crate) fn stop_drag(&mut self) {
        self.is_dragging = false;
    }

    // Returns the last click position, if any.
    pub(crate) fn last_click_position(&self) -> Option<PxPosition> {
        self.last_click_position
    }

    // Updates the last click position (used for drag tracking).
    pub(crate) fn update_last_click_position(&mut self, position: PxPosition) {
        self.last_click_position = Some(position);
    }

    /// Map keyboard events to text editing actions
    /// Maps a keyboard event to a list of text editing actions for the editor.
    ///
    /// This function translates keyboard input (including modifiers) into
    /// editing actions such as character insertion, deletion, navigation,
    /// and clipboard operations.
    ///
    /// # Arguments
    ///
    /// * `key_event` - The keyboard event to map.
    /// * `key_modifiers` - The current keyboard modifier state.
    ///
    /// # Returns
    ///
    /// An optional vector of `glyphon::Action` to be applied to the editor.
    pub fn map_key_event_to_action(
        &mut self,
        key_event: winit::event::KeyEvent,
        key_modifiers: winit::keyboard::ModifiersState,
    ) -> Option<Vec<glyphon::Action>> {
        let editor = &mut self.editor;

        match key_event.state {
            winit::event::ElementState::Pressed => {}
            winit::event::ElementState::Released => return None,
        }

        match key_event.logical_key {
            winit::keyboard::Key::Named(named_key) => match named_key {
                NamedKey::Backspace => Some(vec![glyphon::Action::Backspace]),
                NamedKey::Delete => Some(vec![glyphon::Action::Delete]),
                NamedKey::Enter => Some(vec![glyphon::Action::Enter]),
                NamedKey::Escape => Some(vec![glyphon::Action::Escape]),
                NamedKey::Tab => Some(vec![glyphon::Action::Insert(' '); 4]),
                NamedKey::ArrowLeft => {
                    if key_modifiers.control_key() {
                        editor.set_selection(Selection::None);

                        Some(vec![glyphon::Action::Motion(cosmic_text::Motion::LeftWord)])
                    } else {
                        // if we have selected text, we need to clear it and not perform any action
                        if editor.selection_bounds().is_some() {
                            editor.set_selection(Selection::None);

                            return None;
                        }

                        Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Left)])
                    }
                }
                NamedKey::ArrowRight => {
                    if key_modifiers.control_key() {
                        editor.set_selection(Selection::None);

                        Some(vec![glyphon::Action::Motion(
                            cosmic_text::Motion::RightWord,
                        )])
                    } else {
                        if editor.selection_bounds().is_some() {
                            editor.set_selection(Selection::None);

                            return None;
                        }

                        Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Right)])
                    }
                }
                NamedKey::ArrowUp => {
                    // if we are on the first line, we move the cursor to the beginning of the line
                    if editor.cursor().line == 0 {
                        editor.set_cursor(Cursor::new(0, 0));

                        return None;
                    }

                    Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Up)])
                }
                NamedKey::ArrowDown => {
                    let last_line_index =
                        editor.with_buffer(|buffer| buffer.lines.len().saturating_sub(1));

                    // if we are on the last line, we move the cursor to the end of the line
                    if editor.cursor().line >= last_line_index {
                        let last_col =
                            editor.with_buffer(|buffer| buffer.lines[last_line_index].text().len());

                        editor.set_cursor(Cursor::new(last_line_index, last_col));
                        return None;
                    }

                    Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Down)])
                }
                NamedKey::Home => Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Home)]),
                NamedKey::End => Some(vec![glyphon::Action::Motion(cosmic_text::Motion::End)]),
                NamedKey::Space => Some(vec![glyphon::Action::Insert(' ')]),
                _ => None,
            },

            winit::keyboard::Key::Character(s) => {
                let is_ctrl = key_modifiers.control_key() || key_modifiers.super_key();
                if is_ctrl {
                    match s.to_lowercase().as_str() {
                        "c" => {
                            if let Some(text) = editor.copy_selection() {
                                clipboard::set_text(&text);
                            }
                            return None;
                        }
                        "v" => {
                            if let Some(text) = clipboard::get_text() {
                                return Some(text.chars().map(glyphon::Action::Insert).collect());
                            }

                            return None;
                        }
                        "x" => {
                            if let Some(text) = editor.copy_selection() {
                                clipboard::set_text(&text);
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

    /// Sets the entire text content of the editor, preserving cursor position
    /// as much as possible.
    ///
    /// # Arguments
    ///
    /// - `text` - The new text content to set in the editor.
    pub fn set_text(&mut self, text: &str) {
        let old_cursor = self.editor.cursor();

        self.editor.with_buffer_mut(|buffer| {
            let color = glyphon::Color::rgba(
                (self.text_color.r * 255.0) as u8,
                (self.text_color.g * 255.0) as u8,
                (self.text_color.b * 255.0) as u8,
                (self.text_color.a * 255.0) as u8,
            );
            buffer.set_text(
                &mut write_font_system(),
                text,
                &glyphon::Attrs::new()
                    .family(glyphon::fontdb::Family::SansSerif)
                    .color(color),
                glyphon::Shaping::Advanced,
                None,
            );
            buffer.set_redraw(true);
        });

        let new_cursor = self.editor.with_buffer(|buffer| {
            let new_num_lines = buffer.lines.len();

            if old_cursor.line < new_num_lines {
                let line = &buffer.lines[old_cursor.line];
                let new_line_len = line.text().len();

                if old_cursor.index <= new_line_len {
                    old_cursor
                } else {
                    glyphon::Cursor::new(old_cursor.line, new_line_len)
                }
            } else {
                let last_line_index = new_num_lines.saturating_sub(1);
                let last_line_len = buffer
                    .lines
                    .get(last_line_index)
                    .map_or(0, |l| l.text().len());
                glyphon::Cursor::new(last_line_index, last_line_len)
            }
        });

        self.editor.set_cursor(new_cursor);
        self.editor
            .set_selection(glyphon::cosmic_text::Selection::None);
        self.bump_layout_version();
    }

    pub(crate) fn layout_version(&self) -> u64 {
        self.layout_version
    }

    pub(crate) fn bump_layout_version(&mut self) {
        self.layout_version = self.layout_version.wrapping_add(1);
    }
}

/// Compute selection rectangles for the given editor.
fn compute_selection_rects(editor: &glyphon::Editor) -> Vec<RectDef> {
    let mut selection_rects: Vec<RectDef> = Vec::new();
    let (selection_start, selection_end) = editor.selection_bounds().unwrap_or_default();

    editor.with_buffer(|buffer| {
        for run in buffer.layout_runs() {
            let line_top = Px(run.line_top as i32);
            let line_height = Px(run.line_height as i32);

            if let Some((x, w)) = run.highlight(selection_start, selection_end) {
                selection_rects.push(RectDef {
                    x: Px(x as i32),
                    y: line_top,
                    width: Px(w as i32),
                    height: line_height,
                });
            }
        }
    });

    selection_rects
}

fn editor_content(editor: &glyphon::Editor) -> String {
    editor.with_buffer(|buffer| {
        buffer
            .lines
            .iter()
            .map(|line| line.text().to_string() + line.ending().as_str())
            .collect::<String>()
    })
}

fn glyphon_color(color: Color) -> glyphon::Color {
    glyphon::Color::rgba(
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8,
        (color.a * 255.0) as u8,
    )
}

fn build_display_buffer(
    text: &str,
    color: Color,
    font_size: f32,
    line_height: f32,
    constraint: &TextConstraint,
) -> glyphon::Buffer {
    let mut buffer = glyphon::Buffer::new(
        &mut write_font_system(),
        glyphon::Metrics::new(font_size, line_height),
    );
    buffer.set_wrap(&mut write_font_system(), glyphon::Wrap::Glyph);
    buffer.set_size(
        &mut write_font_system(),
        constraint.max_width,
        constraint.max_height,
    );
    buffer.set_text(
        &mut write_font_system(),
        text,
        &glyphon::Attrs::new()
            .family(glyphon::fontdb::Family::SansSerif)
            .color(glyphon_color(color)),
        glyphon::Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut write_font_system(), false);
    buffer
}

fn build_display_editor(
    buffer: glyphon::Buffer,
    cursor: Cursor,
    selection: Option<(Cursor, Cursor)>,
) -> glyphon::Editor<'static> {
    let mut editor = glyphon::Editor::new(buffer);
    if let Some((start, end)) = selection {
        editor.set_selection(Selection::Normal(start));
        editor.set_cursor(end);
    } else {
        editor.set_cursor(cursor);
        editor.set_selection(Selection::None);
    }
    editor
}

/// Clip rects to visible area and drop those fully outside.
fn clip_and_take_visible(rects: Vec<RectDef>, visible_x1: Px, visible_y1: Px) -> Vec<RectDef> {
    let visible_x0 = Px(0);
    let visible_y0 = Px(0);

    rects
        .into_iter()
        .filter_map(|mut rect| {
            let rect_x1 = rect.x + rect.width;
            let rect_y1 = rect.y + rect.height;
            if rect_x1 <= visible_x0
                || rect.y >= visible_y1
                || rect.x >= visible_x1
                || rect_y1 <= visible_y0
            {
                None
            } else {
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
        .collect()
}

#[derive(Clone)]
struct TextEditLayout {
    controller: State<TextEditorController>,
    layout_version: u64,
}

impl PartialEq for TextEditLayout {
    fn eq(&self, other: &Self) -> bool {
        self.layout_version == other.layout_version
    }
}

impl LayoutSpec for TextEditLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let max_width_pixels: Option<Px> = match input.parent_constraint().width() {
            DimensionValue::Fixed(w) => Some(w),
            DimensionValue::Wrap { max, .. } => max,
            DimensionValue::Fill { max, .. } => max,
        };

        let max_height_pixels: Option<Px> = match input.parent_constraint().height() {
            DimensionValue::Fixed(h) => Some(h),
            DimensionValue::Wrap { max, .. } => max,
            DimensionValue::Fill { max, .. } => max,
        };

        let text_data = self.controller.with_mut(|c| {
            c.text_data(TextConstraint {
                max_width: max_width_pixels.map(|px| px.to_f32()),
                max_height: max_height_pixels.map(|px| px.to_f32()),
            })
        });

        let (mut selection_rects, cursor_pos_raw) = self.controller.with(|c| {
            if c.display_transform_active()
                && let Some(buffer) = c.current_layout_buffer.clone()
            {
                let cursor = c.editor().cursor();
                let selection = c.editor().selection_bounds();
                let display_editor = build_display_editor(buffer, cursor, selection);
                (
                    compute_selection_rects(&display_editor),
                    display_editor.cursor_position(),
                )
            } else {
                (
                    compute_selection_rects(c.editor()),
                    c.editor().cursor_position(),
                )
            }
        });

        let selection_rects_len = selection_rects.len();

        for (i, rect_def) in selection_rects.iter().enumerate() {
            if let Some(rect_node_id) = input.children_ids().get(i).copied() {
                input.measure_child_in_parent_constraint(rect_node_id)?;
                output.place_child(rect_node_id, PxPosition::new(rect_def.x, rect_def.y));
            }
        }

        let visible_x1 = max_width_pixels.unwrap_or(Px(i32::MAX));
        let visible_y1 = max_height_pixels.unwrap_or(Px(i32::MAX));
        selection_rects = clip_and_take_visible(selection_rects, visible_x1, visible_y1);
        self.controller
            .with_mut(|c| c.current_selection_rects = selection_rects.clone());

        if let Some(cursor_pos_raw) = cursor_pos_raw {
            let cursor_pos = PxPosition::new(Px(cursor_pos_raw.0), Px(cursor_pos_raw.1));
            let cursor_node_index = selection_rects_len;
            if let Some(cursor_node_id) = input.children_ids().get(cursor_node_index).copied() {
                input.measure_child_in_parent_constraint(cursor_node_id)?;
                output.place_child(cursor_node_id, cursor_pos);
            }
        }

        let constrained_height = if let Some(max_h) = max_height_pixels {
            text_data.size[1].min(max_h.abs())
        } else {
            text_data.size[1]
        };

        Ok(ComputedData {
            width: Px::from(text_data.size[0]) + CURSOR_WIDRH.to_px(),
            height: constrained_height.into(),
        })
    }

    fn record(&self, input: &RenderInput<'_>) {
        let mut metadata = input.metadata_mut();
        metadata.clips_children = true;
        if let Some(text_data) = self.controller.with(|c| c.current_text_data.clone()) {
            let drawable = TextCommand {
                data: text_data,
                offset: PxPosition::ZERO,
            };
            metadata.fragment_mut().push_draw_command(drawable);
        }
    }
}

/// Core text editing component for rendering text, selection, and cursor.
///
/// This component is responsible for rendering the text buffer, selection
/// highlights, and cursor. It does not handle user events directly; instead, it
/// is intended to be used inside a container that manages user interaction and
/// passes state updates via `TextEditorController`.
///
/// # Arguments
///
/// * `controller` - Shared controller for the text editor.
#[tessera]
pub fn text_edit_core(controller: State<TextEditorController>) {
    // text rendering with constraints from parent container
    let layout_version = controller.with(|c| c.layout_version());
    layout(TextEditLayout {
        controller,
        layout_version,
    });

    // Selection highlighting
    {
        let (rect_definitions, color_for_selection) =
            controller.with(|c| (c.current_selection_rects.clone(), c.selection_color));

        for def in rect_definitions {
            selection_highlight_rect(def.width, def.height, color_for_selection);
        }
    }

    // Cursor rendering (only when focused)
    if controller.with(|c| c.focus_handler().is_focused()) {
        let (line_height, blink_timer, cursor_color) =
            controller.with(|c| (c.line_height(), c.blink_timer(), c.cursor_color()));
        cursor::cursor(line_height, blink_timer, cursor_color);
    }
}
