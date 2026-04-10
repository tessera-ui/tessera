//! Core module for text editing logic and state management in Tessera UI.
//!
//! This module provides the foundational structures and functions for building
//! text editing components, including text buffer management, selection and
//! cursor handling, rendering logic, and keyboard event mapping. It is designed
//! to be shared across UI components via the `TextEditorController` wrapper,
//! enabling consistent access to editor state.
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

use std::ops::Range;

use glyphon::{
    Cursor, Edit,
    cosmic_text::{self, Selection},
};
use tessera_platform::clipboard;
use tessera_ui::{
    CallbackWith, Color, ComputedData, Dp, FocusRequester, LayoutResult, MeasurementError, Px,
    PxPosition, State, current_frame_nanos,
    layout::{LayoutPolicy, MeasureScope, RenderInput, RenderPolicy, layout},
    receive_frame_nanos, tessera,
    time::Instant,
    winit,
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

/// Display-only text transform output with offset mapping between raw and
/// transformed content.
#[derive(Clone, PartialEq)]
pub struct TransformedText {
    text: String,
    raw_boundaries: Vec<usize>,
    transformed_boundaries: Vec<usize>,
    raw_to_transformed: Vec<usize>,
    transformed_to_raw: Vec<usize>,
}

impl TransformedText {
    /// Creates a transformed text model using heuristics to derive offset
    /// mapping from the raw and transformed strings.
    pub fn from_strings(raw: &str, text: String) -> Self {
        let raw_boundaries = char_boundary_offsets(raw);
        let transformed_boundaries = char_boundary_offsets(&text);

        let raw_to_transformed = if raw == text {
            raw_boundaries.clone()
        } else if raw_boundaries.len() == transformed_boundaries.len() {
            transformed_boundaries.clone()
        } else if let Some(mapping) = subsequence_boundary_mapping(raw, &text) {
            mapping
        } else {
            raw_boundaries
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    let transformed_index =
                        index.min(transformed_boundaries.len().saturating_sub(1));
                    transformed_boundaries[transformed_index]
                })
                .collect()
        };

        let transformed_to_raw = transformed_boundaries
            .iter()
            .map(|&transformed_offset| {
                let raw_index = raw_to_transformed
                    .partition_point(|&mapped_offset| mapped_offset <= transformed_offset)
                    .saturating_sub(1);
                raw_boundaries[raw_index]
            })
            .collect();

        Self {
            text,
            raw_boundaries,
            transformed_boundaries,
            raw_to_transformed,
            transformed_to_raw,
        }
    }

    /// Creates a transformed text model without changing visible content.
    pub fn identity(text: String) -> Self {
        let boundaries = char_boundary_offsets(&text);
        Self {
            text,
            raw_boundaries: boundaries.clone(),
            transformed_boundaries: boundaries.clone(),
            raw_to_transformed: boundaries.clone(),
            transformed_to_raw: boundaries,
        }
    }

    fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn map_from_raw(&self, raw_offset: usize) -> usize {
        let index = boundary_index_for_offset(&self.raw_boundaries, raw_offset);
        self.raw_to_transformed[index]
    }

    #[allow(dead_code)]
    fn map_to_raw(&self, transformed_offset: usize) -> usize {
        let index = boundary_index_for_offset(&self.transformed_boundaries, transformed_offset);
        self.transformed_to_raw[index]
    }

    fn map_to_raw_forward(&self, transformed_offset: usize) -> usize {
        let index = self
            .raw_to_transformed
            .partition_point(|&mapped_offset| mapped_offset < transformed_offset)
            .min(self.raw_boundaries.len().saturating_sub(1));
        self.raw_boundaries[index]
    }
}

/// Display-only text transform applied to the text content before rendering
/// (e.g., masking or formatting without changing the underlying buffer).
pub type DisplayTransform = CallbackWith<String, TransformedText>;

type CachedLayout = (Vec<RectDef>, Vec<RectDef>, Option<RectDef>, ComputedData);

#[derive(Clone)]
struct RawEditorSnapshot {
    buffer: glyphon::Buffer,
    cursor: Cursor,
    selection: Option<(Cursor, Cursor)>,
}

struct DerivedGeometryInput {
    scroll_horizontal: Px,
    cursor_offset: usize,
    selection: Option<TextSelection>,
    composition_range: Option<Range<usize>>,
    raw_editor: RawEditorSnapshot,
    raw_composition: Option<(Cursor, Cursor)>,
}

struct PointerActionContext<'a> {
    raw_editor: RawEditorSnapshot,
    raw_text: &'a str,
    cursor_offset: usize,
    selection: Option<TextSelection>,
    drag_selection_mode: DragSelectionMode,
    drag_origin_selection: Option<Range<usize>>,
    action: glyphon::Action,
}

fn char_boundary_offsets(text: &str) -> Vec<usize> {
    let mut boundaries = Vec::with_capacity(text.chars().count() + 1);
    boundaries.push(0);
    for (index, character) in text.char_indices() {
        boundaries.push(index + character.len_utf8());
    }
    boundaries
}

fn subsequence_boundary_mapping(raw: &str, transformed: &str) -> Option<Vec<usize>> {
    let raw_chars: Vec<char> = raw.chars().collect();
    let transformed_chars: Vec<char> = transformed.chars().collect();
    let transformed_boundaries = char_boundary_offsets(transformed);
    if raw_chars.is_empty() {
        return Some(vec![transformed.len()]);
    }
    let mut matched_indices = Vec::with_capacity(raw_chars.len());

    let mut transformed_index = 0usize;
    for raw_char in raw_chars {
        while transformed_index < transformed_chars.len()
            && transformed_chars[transformed_index] != raw_char
        {
            transformed_index += 1;
        }
        if transformed_index >= transformed_chars.len() {
            return None;
        }
        matched_indices.push(transformed_index);
        transformed_index += 1;
    }

    let mut mapping = Vec::with_capacity(matched_indices.len() + 1);
    mapping.push(transformed_boundaries[matched_indices[0]]);
    for matched_index in matched_indices.iter().skip(1) {
        mapping.push(transformed_boundaries[*matched_index]);
    }
    mapping.push(transformed.len());

    Some(mapping)
}

fn boundary_index_for_offset(boundaries: &[usize], offset: usize) -> usize {
    boundaries
        .partition_point(|&boundary| boundary <= offset)
        .saturating_sub(1)
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn next_word_boundary(text: &str, offset: usize) -> usize {
    let boundaries = char_boundary_offsets(text);
    let characters: Vec<char> = text.chars().collect();
    let mut index = boundary_index_for_offset(&boundaries, offset.min(text.len()));

    if index >= characters.len() {
        return text.len();
    }

    if !is_word_character(characters[index]) {
        while index < characters.len() && !is_word_character(characters[index]) {
            index += 1;
        }
    }
    while index < characters.len() && is_word_character(characters[index]) {
        index += 1;
    }

    boundaries[index.min(boundaries.len().saturating_sub(1))]
}

fn previous_word_boundary(text: &str, offset: usize) -> usize {
    let boundaries = char_boundary_offsets(text);
    let characters: Vec<char> = text.chars().collect();
    let mut index = boundary_index_for_offset(&boundaries, offset.min(text.len()));

    while index > 0 && !is_word_character(characters[index - 1]) {
        index -= 1;
    }
    while index > 0 && is_word_character(characters[index - 1]) {
        index -= 1;
    }

    boundaries[index]
}

fn word_range_at_offset(text: &str, offset: usize) -> Option<Range<usize>> {
    let boundaries = char_boundary_offsets(text);
    let characters: Vec<char> = text.chars().collect();
    if characters.is_empty() {
        return None;
    }

    let mut index = boundary_index_for_offset(&boundaries, offset.min(text.len()));
    if index >= characters.len() {
        index = characters.len().saturating_sub(1);
    }

    if !is_word_character(characters[index]) {
        let next_index = (index..characters.len()).find(|&i| is_word_character(characters[i]));
        let previous_index = (0..index).rev().find(|&i| is_word_character(characters[i]));
        index = next_index.or(previous_index)?;
    }

    let mut start = index;
    while start > 0 && is_word_character(characters[start - 1]) {
        start -= 1;
    }

    let mut end = index + 1;
    while end < characters.len() && is_word_character(characters[end]) {
        end += 1;
    }

    Some(boundaries[start]..boundaries[end])
}

fn line_range_at_offset(text: &str, offset: usize) -> Range<usize> {
    let clamped_offset = offset.min(text.len());
    let start = text[..clamped_offset]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let end = text[clamped_offset..]
        .find('\n')
        .map_or(text.len(), |index| clamped_offset + index);
    start..end
}

fn paragraph_start_offsets(text: &str) -> Vec<usize> {
    if text.is_empty() {
        return vec![0];
    }

    let mut starts = Vec::new();
    let mut offset = 0usize;
    let mut previous_blank = true;
    for segment in text.split_inclusive('\n') {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        let blank = line.trim().is_empty();
        if !blank && previous_blank {
            starts.push(offset);
        }
        offset += segment.len();
        previous_blank = blank;
    }

    if starts.is_empty() {
        starts.push(0);
    }

    starts
}

fn previous_paragraph_boundary(text: &str, offset: usize) -> usize {
    let starts = paragraph_start_offsets(text);
    let clamped_offset = offset.min(text.len());
    let index = starts.partition_point(|&start| start < clamped_offset);
    if index == 0 { 0 } else { starts[index - 1] }
}

fn next_paragraph_boundary(text: &str, offset: usize) -> usize {
    let starts = paragraph_start_offsets(text);
    let clamped_offset = offset.min(text.len());
    let index = starts.partition_point(|&start| start <= clamped_offset);
    starts.get(index).copied().unwrap_or(text.len())
}

fn extend_selection_range(origin: &Range<usize>, target: &Range<usize>) -> TextSelection {
    if target.start >= origin.end {
        TextSelection {
            start: origin.start,
            end: target.end,
        }
    } else if target.end <= origin.start {
        TextSelection {
            start: origin.end,
            end: target.start,
        }
    } else {
        TextSelection {
            start: origin.start,
            end: origin.end,
        }
    }
}

/// Definition of a rectangular selection highlight
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Copy, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum DragSelectionMode {
    Character,
    Word,
    Line,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct TextScrollState {
    scroll: glyphon::cosmic_text::Scroll,
}

impl TextScrollState {
    fn from_scroll(scroll: glyphon::cosmic_text::Scroll) -> Self {
        Self { scroll }
    }

    fn as_scroll(self) -> glyphon::cosmic_text::Scroll {
        self.scroll
    }

    pub(crate) fn horizontal(&self) -> f32 {
        self.scroll.horizontal
    }

    pub(crate) fn vertical(&self) -> f32 {
        self.scroll.vertical
    }

    fn reset_vertical(self) -> Self {
        let mut scroll = self.scroll;
        scroll.line = 0;
        scroll.vertical = 0.0;
        Self { scroll }
    }
}

#[derive(Clone, Default)]
struct TextLayoutSnapshot {
    text_cache_key: Option<TextLayoutCacheKey>,
    geometry_cache_key: Option<TextLayoutCacheKey>,
    text_data: Option<TextData>,
    buffer: Option<glyphon::Buffer>,
    transformed_text: Option<TransformedText>,
    selection_rects: Vec<RectDef>,
    composition_rects: Vec<RectDef>,
    cursor_rect: Option<RectDef>,
    ime_rect: Option<RectDef>,
    computed_data: Option<ComputedData>,
}

struct DerivedLayoutGeometry {
    selection_rects: Vec<RectDef>,
    composition_rects: Vec<RectDef>,
    cursor_position: Option<(i32, i32)>,
    scroll_horizontal: Px,
}

struct PreviewEditResult {
    text: String,
    selection: TextSelection,
}

struct PreviewReplaceResult {
    text: String,
    replaced_range: Range<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct TextLayoutCacheKey {
    layout_version: u64,
    max_width_bits: Option<u32>,
    max_height_bits: Option<u32>,
}

impl TextLayoutCacheKey {
    fn new(layout_version: u64, constraint: &TextConstraint) -> Self {
        Self {
            layout_version,
            max_width_bits: constraint.max_width.map(f32::to_bits),
            max_height_bits: constraint.max_height.map(f32::to_bits),
        }
    }
}

struct TextEditState {
    editor: glyphon::Editor<'static>,
    display_transform: Option<DisplayTransform>,
    text_color: Color,
    cursor_color: Color,
    single_line: bool,
}

impl TextEditState {
    fn editor(&self) -> &glyphon::Editor<'static> {
        &self.editor
    }

    fn with_editor_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut glyphon::Editor<'static>) -> R,
    {
        f(&mut self.editor)
    }

    fn text(&self) -> String {
        editor_content(&self.editor)
    }

    fn text_color(&self) -> Color {
        self.text_color
    }

    fn set_text_color(&mut self, color: Color) -> bool {
        if self.text_color == color {
            return false;
        }
        self.text_color = color;
        true
    }

    fn cursor_color(&self) -> Color {
        self.cursor_color
    }

    fn set_cursor_color(&mut self, color: Color) {
        self.cursor_color = color;
    }

    fn display_transform(&self) -> Option<DisplayTransform> {
        self.display_transform
    }

    fn display_transform_ref(&self) -> Option<&DisplayTransform> {
        self.display_transform.as_ref()
    }

    fn set_display_transform(&mut self, transform: Option<DisplayTransform>) -> bool {
        let should_update = match (&self.display_transform, &transform) {
            (None, None) => false,
            (Some(current), Some(next)) => current != next,
            _ => true,
        };
        if should_update {
            self.display_transform = transform;
        }
        should_update
    }

    fn single_line(&self) -> bool {
        self.single_line
    }

    fn set_single_line(&mut self, single_line: bool) -> bool {
        if self.single_line == single_line {
            return false;
        }
        self.single_line = single_line;
        true
    }

    fn cursor(&self) -> Cursor {
        self.editor.cursor()
    }

    fn cursor_offset(&self) -> usize {
        self.cursor_to_text_offset(self.cursor())
    }

    fn selection(&self) -> TextSelection {
        match self.editor.selection() {
            Selection::None => TextSelection::collapsed(self.cursor_offset()),
            Selection::Normal(anchor) | Selection::Line(anchor) | Selection::Word(anchor) => {
                TextSelection {
                    start: self
                        .editor
                        .with_buffer(|buffer| cursor_to_text_offset_in_buffer(buffer, anchor)),
                    end: self.cursor_offset(),
                }
            }
        }
    }

    fn selected_text(&self) -> Option<String> {
        let selection = self.selection().ordered_range();
        if selection.is_empty() {
            return None;
        }
        let text = self.text();
        Some(text[selection].to_string())
    }

    fn clone_owned_editor(&self) -> glyphon::Editor<'static> {
        let mut new_editor = self.editor.clone();
        let mut new_buffer = None;
        match new_editor.buffer_ref_mut() {
            glyphon::cosmic_text::BufferRef::Owned(_) => {}
            glyphon::cosmic_text::BufferRef::Borrowed(buffer) => {
                new_buffer = Some(buffer.clone());
            }
            glyphon::cosmic_text::BufferRef::Arc(buffer) => {
                new_buffer = Some((**buffer).clone());
            }
        }
        if let Some(buffer) = new_buffer {
            *new_editor.buffer_ref_mut() = glyphon::cosmic_text::BufferRef::Owned(buffer);
        }
        new_editor
    }

    fn preview_action_result(&self, action: glyphon::Action) -> PreviewEditResult {
        let mut editor = self.clone_owned_editor();
        editor.action(&mut write_font_system(), action);
        PreviewEditResult {
            text: editor_content(&editor),
            selection: editor_selection(&editor),
        }
    }

    fn preview_replace_result(
        &self,
        range: Range<usize>,
        replacement: &str,
    ) -> PreviewReplaceResult {
        let current_text = self.text();
        let start = range.start.min(current_text.len());
        let end = range.end.min(current_text.len()).max(start);
        let mut text = current_text;
        text.replace_range(start..end, replacement);
        PreviewReplaceResult {
            text,
            replaced_range: start..(start + replacement.len()),
        }
    }

    fn apply_action(&mut self, action: glyphon::Action) {
        self.editor.action(&mut write_font_system(), action);
    }

    fn set_cursor_and_selection(&mut self, cursor: Cursor, selection: Option<Selection>) {
        self.editor.set_cursor(cursor);
        self.editor
            .set_selection(selection.unwrap_or(Selection::None));
    }

    fn set_cursor_and_selection_offsets(
        &mut self,
        cursor_offset: usize,
        selection: Option<TextSelection>,
    ) {
        let cursor = self.text_offset_to_cursor(cursor_offset);
        let editor_selection = if let Some(selection) = selection
            && !selection.is_collapsed()
        {
            Some(Selection::Normal(
                self.text_offset_to_cursor(selection.start),
            ))
        } else {
            None
        };
        self.set_cursor_and_selection(cursor, editor_selection);
    }

    fn cursor_line(&self) -> usize {
        self.cursor().line
    }

    fn scroll(&self) -> glyphon::cosmic_text::Scroll {
        self.editor.with_buffer(|buffer| buffer.scroll())
    }

    fn metrics_and_scroll(&self) -> (glyphon::Metrics, glyphon::cosmic_text::Scroll) {
        self.editor
            .with_buffer(|buffer| (buffer.metrics(), buffer.scroll()))
    }

    fn last_line_index(&self) -> usize {
        self.editor
            .with_buffer(|buffer| buffer.lines.len().saturating_sub(1))
    }

    fn line_len(&self, line_index: usize) -> usize {
        self.editor
            .with_buffer(|buffer| buffer.lines[line_index].text().len())
    }

    fn first_line_start_cursor(&self) -> Cursor {
        Cursor::new(0, 0)
    }

    fn last_line_end_cursor(&self) -> Cursor {
        let last_line_index = self.last_line_index();
        Cursor::new(last_line_index, self.line_len(last_line_index))
    }

    fn cursor_to_text_offset(&self, cursor: Cursor) -> usize {
        self.editor
            .with_buffer(|buffer| cursor_to_text_offset_in_buffer(buffer, cursor))
    }

    fn text_offset_to_cursor(&self, offset: usize) -> Cursor {
        self.editor
            .with_buffer(|buffer| text_offset_to_cursor_in_buffer(buffer, offset))
    }

    fn set_wrap(&mut self, wrap: glyphon::Wrap) {
        self.editor.with_buffer_mut(|buffer| {
            buffer.set_wrap(&mut write_font_system(), wrap);
        });
    }

    fn set_scroll(&mut self, scroll: glyphon::cosmic_text::Scroll) {
        self.editor.with_buffer_mut(|buffer| {
            buffer.set_scroll(scroll);
        });
    }

    fn sync_size_and_shape_until_scroll(&mut self, constraint: &TextConstraint) {
        self.editor.with_buffer_mut(|buffer| {
            buffer.set_size(
                &mut write_font_system(),
                constraint.max_width,
                constraint.max_height,
            );
            buffer.shape_until_scroll(&mut write_font_system(), false);
        });
    }

    fn scroll_horizontal_by(&mut self, delta: f32) -> bool {
        self.editor.with_buffer_mut(|buffer| {
            let viewport_width = buffer.size().0.unwrap_or(0.0);
            let content_width = buffer
                .layout_runs()
                .fold(0.0f32, |width, run| width.max(run.line_w));
            let max_horizontal = (content_width - viewport_width).max(0.0);
            let mut scroll = buffer.scroll();
            let next_horizontal = (scroll.horizontal + delta).clamp(0.0, max_horizontal);
            if (next_horizontal - scroll.horizontal).abs() <= f32::EPSILON {
                return false;
            }
            scroll.horizontal = next_horizontal;
            buffer.set_scroll(scroll);
            true
        })
    }

    fn scroll_vertical_by(&mut self, delta: f32) {
        self.editor.action(
            &mut write_font_system(),
            glyphon::Action::Scroll { pixels: delta },
        );
    }

    fn shape_until_cursor(&mut self) -> glyphon::cosmic_text::Scroll {
        let cursor = self.cursor();
        self.editor.with_buffer_mut(|buffer| {
            buffer.shape_until_cursor(&mut write_font_system(), cursor, false);
        });
        self.scroll()
    }

    fn motion_target_offset(&mut self, motion: cosmic_text::Motion) -> Option<usize> {
        let cursor = self.cursor();
        let next_cursor = self.editor.with_buffer_mut(|buffer| {
            buffer
                .cursor_motion(&mut write_font_system(), cursor, None, motion)
                .map(|(next_cursor, _)| next_cursor)
        })?;
        Some(self.cursor_to_text_offset(next_cursor))
    }

    fn buffer_clone(&self) -> glyphon::Buffer {
        match self.editor.buffer_ref() {
            glyphon::cosmic_text::BufferRef::Owned(buffer) => buffer.clone(),
            glyphon::cosmic_text::BufferRef::Borrowed(buffer) => (**buffer).to_owned(),
            glyphon::cosmic_text::BufferRef::Arc(buffer) => (**buffer).clone(),
        }
    }

    fn set_text_and_selection(&mut self, text: &str, selection: TextSelection) {
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

        let start = self.editor.with_buffer(|buffer| {
            text_offset_to_cursor_in_buffer(buffer, selection.start.min(text.len()))
        });
        let end = self.editor.with_buffer(|buffer| {
            text_offset_to_cursor_in_buffer(buffer, selection.end.min(text.len()))
        });
        let editor_selection = if selection.is_collapsed() {
            None
        } else {
            Some(Selection::Normal(start))
        };
        self.set_cursor_and_selection(end, editor_selection);
    }

    fn select_all(&mut self) {
        let end = self.text().len();
        let start_cursor = self.text_offset_to_cursor(0);
        let end_cursor = self.text_offset_to_cursor(end);
        self.set_cursor_and_selection(end_cursor, Some(Selection::Normal(start_cursor)));
    }
}

struct TextSelectionState {
    selection_color: Color,
    last_click_time: Option<Instant>,
    last_click_position: Option<PxPosition>,
    click_count: u32,
    is_dragging: bool,
    drag_selection_mode: DragSelectionMode,
    drag_origin_selection: Option<Range<usize>>,
}

impl TextSelectionState {
    fn selection_color(&self) -> Color {
        self.selection_color
    }

    fn set_selection_color(&mut self, color: Color) {
        self.selection_color = color;
    }

    fn handle_click(&mut self, position: PxPosition, timestamp: Instant) -> ClickType {
        const DOUBLE_CLICK_TIME_MS: u128 = 500;
        const CLICK_DISTANCE_THRESHOLD: Px = Px(5);

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
                        self.click_count = 0;
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
        self.drag_selection_mode = DragSelectionMode::Character;
        self.drag_origin_selection = None;

        click_type
    }

    fn start_drag(&mut self, click_type: ClickType, origin_selection: Option<Range<usize>>) {
        self.is_dragging = true;
        match click_type {
            ClickType::Single => {
                self.drag_selection_mode = DragSelectionMode::Character;
                self.drag_origin_selection = None;
            }
            ClickType::Double => {
                self.drag_selection_mode = DragSelectionMode::Word;
                self.drag_origin_selection = origin_selection;
            }
            ClickType::Triple => {
                self.drag_selection_mode = DragSelectionMode::Line;
                self.drag_origin_selection = origin_selection;
            }
        }
    }

    fn stop_drag(&mut self) {
        self.is_dragging = false;
        self.drag_selection_mode = DragSelectionMode::Character;
        self.drag_origin_selection = None;
    }

    fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    fn drag_selection_mode(&self) -> DragSelectionMode {
        self.drag_selection_mode
    }

    fn drag_origin_selection(&self) -> Option<Range<usize>> {
        self.drag_origin_selection.clone()
    }

    fn last_click_position(&self) -> Option<PxPosition> {
        self.last_click_position
    }

    fn last_click_time(&self) -> Option<Instant> {
        self.last_click_time
    }

    fn update_last_click_position(&mut self, position: PxPosition) {
        self.last_click_position = Some(position);
    }

    fn anchor_offset(&self, selection: TextSelection, cursor_offset: usize) -> usize {
        if selection.is_collapsed() {
            cursor_offset
        } else {
            selection.start
        }
    }

    fn drag_origin_for_click(
        &self,
        click_type: ClickType,
        selection: TextSelection,
    ) -> Option<Range<usize>> {
        match click_type {
            ClickType::Single => Some(selection.ordered_range()),
            ClickType::Double | ClickType::Triple => Some(selection.ordered_range()),
        }
    }

    fn collapse_target(&self, selection: TextSelection, collapse_to_end: bool) -> Option<usize> {
        if selection.is_collapsed() {
            return None;
        }
        let ordered = selection.ordered_range();
        Some(if collapse_to_end {
            ordered.end
        } else {
            ordered.start
        })
    }

    fn extended_selection(
        &self,
        selection: TextSelection,
        cursor_offset: usize,
        next_offset: usize,
    ) -> Option<TextSelection> {
        let anchor_offset = self.anchor_offset(selection, cursor_offset);
        (next_offset != anchor_offset).then_some(TextSelection {
            start: anchor_offset,
            end: next_offset,
        })
    }

    fn selection_cursor_range(
        &self,
        selection: TextSelection,
        edit_state: &TextEditState,
    ) -> Option<(Cursor, Cursor)> {
        (!selection.is_collapsed()).then(|| {
            (
                edit_state.text_offset_to_cursor(selection.start),
                edit_state.text_offset_to_cursor(selection.end),
            )
        })
    }

    fn deletion_range(
        &self,
        selection: TextSelection,
        cursor_offset: usize,
        target_offset: Option<usize>,
    ) -> Option<Range<usize>> {
        let selection = selection.ordered_range();
        if !selection.is_empty() {
            return Some(selection);
        }

        let target_offset = target_offset?;
        (target_offset != cursor_offset)
            .then_some(cursor_offset.min(target_offset)..cursor_offset.max(target_offset))
    }
}

struct TextScrollControllerState {
    state: TextScrollState,
}

impl TextScrollControllerState {
    fn new(scroll: glyphon::cosmic_text::Scroll) -> Self {
        Self {
            state: TextScrollState::from_scroll(scroll),
        }
    }

    fn as_scroll(&self) -> glyphon::cosmic_text::Scroll {
        self.state.as_scroll()
    }

    fn state(&self) -> TextScrollState {
        self.state
    }

    fn horizontal(&self) -> f32 {
        self.state.horizontal()
    }

    fn sync_from_scroll(&mut self, scroll: glyphon::cosmic_text::Scroll) -> bool {
        if self.state.as_scroll() == scroll {
            return false;
        }
        self.state = TextScrollState::from_scroll(scroll);
        true
    }

    fn sync_from_edit_state(&mut self, edit_state: &TextEditState) -> bool {
        self.sync_from_scroll(edit_state.scroll())
    }

    fn reset_vertical(&mut self) {
        self.state = self.state.reset_vertical();
    }
}

struct TextImeState {
    composition: Option<ImeComposition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlannedImeEdit {
    pub(crate) replacement_range: Range<usize>,
    pub(crate) replacement_text: String,
    pub(crate) selection: TextSelection,
    pub(crate) composition_range: Option<Range<usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PlannedImeEvent {
    Submit,
    Edit(PlannedImeEdit),
}

impl TextImeState {
    fn composition(&self) -> Option<&ImeComposition> {
        self.composition.as_ref()
    }

    #[cfg(test)]
    fn set_composition(&mut self, composition: Option<ImeComposition>) {
        self.composition = composition;
    }

    fn clear(&mut self) {
        self.composition = None;
    }

    fn composition_range(&self) -> Option<Range<usize>> {
        self.composition
            .as_ref()
            .map(|composition| composition.range.clone())
    }

    fn composition_cursor_range(&self, edit_state: &TextEditState) -> Option<(Cursor, Cursor)> {
        self.composition_range().map(|range| {
            (
                edit_state.text_offset_to_cursor(range.start),
                edit_state.text_offset_to_cursor(range.end),
            )
        })
    }

    fn plan_event(
        &self,
        selection: TextSelection,
        single_line: bool,
        event: &winit::event::Ime,
    ) -> Option<PlannedImeEvent> {
        if single_line
            && matches!(
                event,
                winit::event::Ime::Commit(text) if matches!(text.as_str(), "\n" | "\r" | "\r\n")
            )
        {
            return Some(PlannedImeEvent::Submit);
        }

        let replacement_range = self
            .composition
            .as_ref()
            .map(|composition| composition.range.clone())
            .unwrap_or_else(|| selection.ordered_range());

        match event {
            winit::event::Ime::Commit(text) => Some(PlannedImeEvent::Edit(PlannedImeEdit {
                replacement_range: replacement_range.clone(),
                replacement_text: text.clone(),
                selection: TextSelection::collapsed(replacement_range.start + text.len()),
                composition_range: None,
            })),
            winit::event::Ime::Preedit(text, cursor_offset) => {
                if text.is_empty() {
                    Some(PlannedImeEvent::Edit(PlannedImeEdit {
                        replacement_range: replacement_range.clone(),
                        replacement_text: String::new(),
                        selection: TextSelection::collapsed(replacement_range.start),
                        composition_range: None,
                    }))
                } else {
                    let selection = composition_selection(
                        replacement_range.start,
                        text,
                        *cursor_offset,
                        self.composition
                            .as_ref()
                            .map(|composition| composition.selection.clone()),
                    );
                    Some(PlannedImeEvent::Edit(PlannedImeEdit {
                        replacement_range: replacement_range.clone(),
                        replacement_text: text.clone(),
                        selection: selection.clone(),
                        composition_range: Some(
                            replacement_range.start..replacement_range.start + text.len(),
                        ),
                    }))
                }
            }
            _ => None,
        }
    }

    fn commit_edit_result(&mut self, plan: &PlannedImeEdit, result: &ImeEditResult) {
        self.composition = plan.composition_range.as_ref().map(|_| ImeComposition {
            range: result.replaced_range.clone(),
            selection: result.selection.clone(),
        });
    }
}

struct TextLayoutState {
    snapshot: TextLayoutSnapshot,
    layout_version: u64,
    text_layout_version: u64,
}

enum LayoutInvalidation {
    Geometry,
    Text,
}

enum PointerActionOutcome {
    Selection(TextSelection),
    Cursor {
        cursor_offset: usize,
        selection: Option<TextSelection>,
    },
    ApplyAction(glyphon::Action),
    Ignored,
}

impl TextLayoutState {
    fn new() -> Self {
        Self {
            snapshot: TextLayoutSnapshot::default(),
            layout_version: 0,
            text_layout_version: 0,
        }
    }

    fn update_geometry(
        &mut self,
        cache_key: TextLayoutCacheKey,
        selection_rects: Vec<RectDef>,
        composition_rects: Vec<RectDef>,
        cursor_rect: Option<RectDef>,
        ime_rect: Option<RectDef>,
        computed_data: ComputedData,
    ) {
        self.snapshot.geometry_cache_key = Some(cache_key);
        self.snapshot.selection_rects = selection_rects;
        self.snapshot.composition_rects = composition_rects;
        self.snapshot.cursor_rect = cursor_rect;
        self.snapshot.ime_rect = ime_rect;
        self.snapshot.computed_data = Some(computed_data);
    }

    fn update_geometry_for_constraint(
        &mut self,
        constraint: &TextConstraint,
        selection_rects: Vec<RectDef>,
        composition_rects: Vec<RectDef>,
        cursor_rect: Option<RectDef>,
        ime_rect: Option<RectDef>,
        computed_data: ComputedData,
    ) {
        self.update_geometry(
            self.geometry_cache_key(constraint),
            selection_rects,
            composition_rects,
            cursor_rect,
            ime_rect,
            computed_data,
        );
    }

    fn selection_rects(&self) -> &[RectDef] {
        &self.snapshot.selection_rects
    }

    fn text_data(&self) -> Option<TextData> {
        self.snapshot.text_data.clone()
    }

    #[cfg(test)]
    fn cache_key(&self) -> Option<TextLayoutCacheKey> {
        self.snapshot.text_cache_key
    }

    fn composition_rects(&self) -> &[RectDef] {
        &self.snapshot.composition_rects
    }

    fn ime_rect(&self) -> Option<RectDef> {
        self.snapshot.ime_rect.clone()
    }

    fn cached_geometry(&self, cache_key: TextLayoutCacheKey) -> Option<CachedLayout> {
        if self.snapshot.geometry_cache_key != Some(cache_key) {
            return None;
        }
        Some((
            self.snapshot.selection_rects.clone(),
            self.snapshot.composition_rects.clone(),
            self.snapshot.cursor_rect.clone(),
            self.snapshot.computed_data?,
        ))
    }

    fn cached_geometry_for_constraint(&self, constraint: &TextConstraint) -> Option<CachedLayout> {
        self.cached_geometry(self.geometry_cache_key(constraint))
    }

    fn reset_geometry_cache(&mut self) {
        self.snapshot.geometry_cache_key = None;
        self.snapshot.selection_rects.clear();
        self.snapshot.composition_rects.clear();
        self.snapshot.cursor_rect = None;
        self.snapshot.ime_rect = None;
        self.snapshot.computed_data = None;
    }

    fn sync_snapshot_scroll(&mut self, scroll: glyphon::cosmic_text::Scroll) {
        if let Some(buffer) = self.snapshot.buffer.as_mut()
            && buffer.scroll() != scroll
        {
            buffer.set_scroll(scroll);
            buffer.shape_until_scroll(&mut write_font_system(), false);
        }
    }

    fn geometry_cache_key(&self, constraint: &TextConstraint) -> TextLayoutCacheKey {
        TextLayoutCacheKey::new(self.layout_version, constraint)
    }

    fn text_cache_key(&self, constraint: &TextConstraint) -> TextLayoutCacheKey {
        TextLayoutCacheKey::new(self.text_layout_version, constraint)
    }

    fn cached_text_data(&self, cache_key: TextLayoutCacheKey) -> Option<TextData> {
        (self.snapshot.text_cache_key == Some(cache_key))
            .then(|| self.snapshot.text_data.clone())
            .flatten()
    }

    fn cached_text_data_for_constraint(
        &mut self,
        constraint: &TextConstraint,
        scroll: glyphon::cosmic_text::Scroll,
    ) -> Option<TextData> {
        let cache_key = self.text_cache_key(constraint);
        let text_data = self.cached_text_data(cache_key)?;
        self.sync_snapshot_scroll(scroll);
        Some(text_data)
    }

    fn set_text_snapshot(
        &mut self,
        cache_key: TextLayoutCacheKey,
        buffer: glyphon::Buffer,
        transformed_text: Option<TransformedText>,
        text_data: TextData,
    ) {
        self.snapshot.text_cache_key = Some(cache_key);
        self.snapshot.buffer = Some(buffer);
        self.snapshot.transformed_text = transformed_text;
        self.snapshot.text_data = Some(text_data);
        self.reset_geometry_cache();
    }

    fn set_text_snapshot_for_constraint(
        &mut self,
        constraint: &TextConstraint,
        buffer: glyphon::Buffer,
        transformed_text: Option<TransformedText>,
        text_data: TextData,
    ) {
        self.set_text_snapshot(
            self.text_cache_key(constraint),
            buffer,
            transformed_text,
            text_data,
        );
    }

    fn buffer(&self) -> Option<glyphon::Buffer> {
        self.snapshot.buffer.clone()
    }

    fn buffer_or(&self, fallback_buffer: glyphon::Buffer) -> glyphon::Buffer {
        self.buffer().unwrap_or(fallback_buffer)
    }

    fn transformed_text(&self) -> Option<&TransformedText> {
        self.snapshot.transformed_text.as_ref()
    }

    fn cloned_transformed_text(&self) -> Option<TransformedText> {
        self.snapshot.transformed_text.clone()
    }

    fn layout_version(&self) -> u64 {
        self.layout_version
    }

    #[cfg(test)]
    fn text_layout_version(&self) -> u64 {
        self.text_layout_version
    }

    fn bump_layout_version(&mut self) {
        self.layout_version = self.layout_version.wrapping_add(1);
    }

    fn bump_text_layout_version(&mut self) {
        self.text_layout_version = self.text_layout_version.wrapping_add(1);
        self.bump_layout_version();
    }

    fn invalidate(&mut self, invalidation: LayoutInvalidation) {
        match invalidation {
            LayoutInvalidation::Geometry => self.bump_layout_version(),
            LayoutInvalidation::Text => self.bump_text_layout_version(),
        }
    }

    fn display_editor(
        &self,
        cursor_offset: usize,
        selection: Option<TextSelection>,
    ) -> Option<(glyphon::Editor<'static>, TransformedText)> {
        let buffer = self.buffer()?;
        let transformed_text = self.cloned_transformed_text()?;
        let cursor =
            text_offset_to_cursor_in_buffer(&buffer, transformed_text.map_from_raw(cursor_offset));
        let selection = selection.map(|selection| {
            let ordered = selection.ordered_range();
            let start = text_offset_to_cursor_in_buffer(
                &buffer,
                transformed_text.map_from_raw(ordered.start),
            );
            let end = text_offset_to_cursor_in_buffer(
                &buffer,
                transformed_text.map_from_raw(ordered.end),
            );
            if selection.start <= selection.end {
                (start, end)
            } else {
                (end, start)
            }
        });
        Some((
            build_display_editor(buffer, cursor, selection),
            transformed_text,
        ))
    }

    fn raw_layout_editor(
        &self,
        fallback_buffer: glyphon::Buffer,
        cursor: Cursor,
        selection: Option<(Cursor, Cursor)>,
    ) -> glyphon::Editor<'static> {
        build_display_editor(self.buffer_or(fallback_buffer), cursor, selection)
    }

    fn derived_geometry(&self, input: DerivedGeometryInput) -> Option<DerivedLayoutGeometry> {
        let DerivedGeometryInput {
            scroll_horizontal,
            cursor_offset,
            selection,
            composition_range,
            raw_editor,
            raw_composition,
        } = input;

        if let Some((display_editor, transformed_text)) =
            self.display_editor(cursor_offset, selection)
        {
            let composition_rects = display_editor.with_buffer(|buffer| {
                composition_range
                    .clone()
                    .map(|composition| {
                        compute_transformed_composition_rects(
                            buffer,
                            &transformed_text,
                            composition,
                        )
                    })
                    .unwrap_or_default()
            });
            return Some(DerivedLayoutGeometry {
                selection_rects: compute_selection_rects(&display_editor),
                composition_rects,
                cursor_position: display_editor.cursor_position(),
                scroll_horizontal,
            });
        }

        let raw_editor =
            self.raw_layout_editor(raw_editor.buffer, raw_editor.cursor, raw_editor.selection);
        let composition_rects = raw_editor
            .with_buffer(|buffer| compute_composition_rects_for_range(buffer, raw_composition));
        Some(DerivedLayoutGeometry {
            selection_rects: compute_selection_rects(&raw_editor),
            composition_rects,
            cursor_position: raw_editor.cursor_position(),
            scroll_horizontal,
        })
    }

    fn display_hit_offset(
        &self,
        cursor_offset: usize,
        selection: Option<TextSelection>,
        x: i32,
        y: i32,
    ) -> Option<(TransformedText, usize)> {
        let (editor, transformed_text) = self.display_editor(cursor_offset, selection)?;
        let hit_offset = editor_hit_offset(&editor, x, y)?;
        Some((transformed_text, hit_offset))
    }

    fn raw_hit_offset(
        &self,
        fallback_buffer: glyphon::Buffer,
        cursor: Cursor,
        selection: Option<(Cursor, Cursor)>,
        x: i32,
        y: i32,
    ) -> Option<usize> {
        let editor = self.raw_layout_editor(fallback_buffer, cursor, selection);
        editor_hit_offset(&editor, x, y)
    }

    fn display_motion_offsets(
        &self,
        cursor_offset: usize,
        selection: Option<TextSelection>,
        motion: cosmic_text::Motion,
    ) -> Option<(TransformedText, usize, usize)> {
        let (mut editor, transformed_text) = self.display_editor(cursor_offset, selection)?;
        let current_offset = editor_cursor_offset(&editor);
        let next_offset = editor_motion_offset(&mut editor, motion)?;
        Some((transformed_text, current_offset, next_offset))
    }

    fn paragraph_motion_offset(&self, text: &str, cursor_offset: usize, forward: bool) -> usize {
        if let Some(transformed_text) = self.transformed_text() {
            let display_offset = transformed_text.map_from_raw(cursor_offset);
            let next_display_offset = if forward {
                next_paragraph_boundary(transformed_text.text(), display_offset)
            } else {
                previous_paragraph_boundary(transformed_text.text(), display_offset)
            };
            if next_display_offset > display_offset {
                transformed_text.map_to_raw_forward(next_display_offset)
            } else {
                transformed_text.map_to_raw(next_display_offset)
            }
        } else if forward {
            next_paragraph_boundary(text, cursor_offset)
        } else {
            previous_paragraph_boundary(text, cursor_offset)
        }
    }

    fn transformed_motion_target_offset(
        &self,
        cursor_offset: usize,
        selection: Option<TextSelection>,
        motion: cosmic_text::Motion,
    ) -> Option<usize> {
        if let Some(transformed_text) = self.transformed_text() {
            let display_offset = transformed_text.map_from_raw(cursor_offset);
            let next_display_offset = match motion {
                cosmic_text::Motion::LeftWord => {
                    previous_word_boundary(transformed_text.text(), display_offset)
                }
                cosmic_text::Motion::RightWord => {
                    next_word_boundary(transformed_text.text(), display_offset)
                }
                _ => {
                    let (transformed_text, display_offset, next_display_offset) =
                        self.display_motion_offsets(cursor_offset, selection, motion)?;
                    let moving_forward = next_display_offset > display_offset;
                    return Some(if moving_forward {
                        transformed_text.map_to_raw_forward(next_display_offset)
                    } else {
                        transformed_text.map_to_raw(next_display_offset)
                    });
                }
            };
            let moving_forward = next_display_offset > display_offset;
            return Some(if moving_forward {
                transformed_text.map_to_raw_forward(next_display_offset)
            } else {
                transformed_text.map_to_raw(next_display_offset)
            });
        }

        let (transformed_text, _, display_offset) =
            self.display_motion_offsets(cursor_offset, selection, motion)?;
        Some(transformed_text.map_to_raw(display_offset))
    }

    fn display_drag_selection(
        &self,
        origin: Range<usize>,
        cursor_offset: usize,
        selection: Option<TextSelection>,
        mode: DragSelectionMode,
        x: i32,
        y: i32,
    ) -> Option<TextSelection> {
        let (transformed_text, hit_offset) =
            self.display_hit_offset(cursor_offset, selection, x, y)?;
        if matches!(mode, DragSelectionMode::Character) {
            let anchor = origin.start;
            return (hit_offset != transformed_text.map_from_raw(anchor)).then(|| TextSelection {
                start: anchor,
                end: transformed_text.map_to_raw(hit_offset),
            });
        }
        let display_range = match mode {
            DragSelectionMode::Word => word_range_at_offset(transformed_text.text(), hit_offset)?,
            DragSelectionMode::Line => line_range_at_offset(transformed_text.text(), hit_offset),
            DragSelectionMode::Character => unreachable!(),
        };
        let target = transformed_text.map_to_raw(display_range.start)
            ..transformed_text.map_to_raw_forward(display_range.end);
        Some(extend_selection_range(&origin, &target))
    }

    fn display_pointer_selection(
        &self,
        cursor_offset: usize,
        selection: Option<TextSelection>,
        action: glyphon::Action,
    ) -> Option<TextSelection> {
        let (x, y) = match action {
            glyphon::Action::DoubleClick { x, y } | glyphon::Action::TripleClick { x, y } => (x, y),
            _ => return None,
        };
        let (transformed_text, hit_offset) =
            self.display_hit_offset(cursor_offset, selection, x, y)?;
        let display_range = match action {
            glyphon::Action::DoubleClick { .. } => {
                word_range_at_offset(transformed_text.text(), hit_offset)?
            }
            glyphon::Action::TripleClick { .. } => {
                line_range_at_offset(transformed_text.text(), hit_offset)
            }
            _ => return None,
        };
        Some(TextSelection {
            start: transformed_text.map_to_raw(display_range.start),
            end: transformed_text.map_to_raw_forward(display_range.end),
        })
    }

    fn raw_drag_selection(
        &self,
        origin: Range<usize>,
        raw_editor: &RawEditorSnapshot,
        text: &str,
        mode: DragSelectionMode,
        x: i32,
        y: i32,
    ) -> Option<TextSelection> {
        let hit_offset = self.raw_hit_offset(
            raw_editor.buffer.clone(),
            raw_editor.cursor,
            raw_editor.selection,
            x,
            y,
        )?;
        if matches!(mode, DragSelectionMode::Character) {
            return (hit_offset != origin.start).then_some(TextSelection {
                start: origin.start,
                end: hit_offset,
            });
        }
        let target = match mode {
            DragSelectionMode::Word => word_range_at_offset(text, hit_offset)?,
            DragSelectionMode::Line => line_range_at_offset(text, hit_offset),
            DragSelectionMode::Character => unreachable!(),
        };
        Some(extend_selection_range(&origin, &target))
    }

    fn raw_pointer_selection(
        &self,
        raw_editor: &RawEditorSnapshot,
        text: &str,
        action: glyphon::Action,
    ) -> Option<TextSelection> {
        let (x, y) = match action {
            glyphon::Action::DoubleClick { x, y } | glyphon::Action::TripleClick { x, y } => (x, y),
            _ => return None,
        };
        let hit_offset = self.raw_hit_offset(
            raw_editor.buffer.clone(),
            raw_editor.cursor,
            raw_editor.selection,
            x,
            y,
        )?;
        let target = match action {
            glyphon::Action::DoubleClick { .. } => word_range_at_offset(text, hit_offset)?,
            glyphon::Action::TripleClick { .. } => line_range_at_offset(text, hit_offset),
            _ => return None,
        };
        Some(TextSelection {
            start: target.start,
            end: target.end,
        })
    }

    fn display_action_result(
        &self,
        cursor_offset: usize,
        selection: Option<TextSelection>,
        action: glyphon::Action,
    ) -> Option<(usize, Option<TextSelection>)> {
        let (mut editor, transformed_text) = self.display_editor(cursor_offset, selection)?;
        editor.action(&mut write_font_system(), action);
        let cursor_offset = transformed_text.map_to_raw(editor_cursor_offset(&editor));
        let selection = Self::display_selection_to_raw(&editor, &transformed_text);
        Some((cursor_offset, selection))
    }

    fn display_selection_to_raw(
        display_editor: &glyphon::Editor<'_>,
        transformed_text: &TransformedText,
    ) -> Option<TextSelection> {
        let anchor = match display_editor.selection() {
            Selection::None => return None,
            Selection::Normal(anchor) | Selection::Line(anchor) | Selection::Word(anchor) => anchor,
        };
        let anchor_offset =
            display_editor.with_buffer(|buffer| cursor_to_text_offset_in_buffer(buffer, anchor));
        let cursor_offset = editor_cursor_offset(display_editor);
        Some(TextSelection {
            start: transformed_text.map_to_raw(anchor_offset),
            end: if cursor_offset >= anchor_offset {
                transformed_text.map_to_raw_forward(cursor_offset)
            } else {
                transformed_text.map_to_raw(cursor_offset)
            },
        })
    }

    fn pointer_action_outcome(&self, context: PointerActionContext<'_>) -> PointerActionOutcome {
        let PointerActionContext {
            raw_editor,
            raw_text,
            cursor_offset,
            selection,
            drag_selection_mode,
            drag_origin_selection,
            action,
        } = context;

        if self.cloned_transformed_text().is_some() {
            if let glyphon::Action::Drag { x, y } = action
                && let Some(origin) = drag_origin_selection
                && let Some(selection) = self.display_drag_selection(
                    origin,
                    cursor_offset,
                    selection.clone(),
                    drag_selection_mode,
                    x,
                    y,
                )
            {
                return PointerActionOutcome::Selection(selection);
            }

            if let Some(selection) =
                self.display_pointer_selection(cursor_offset, selection.clone(), action)
            {
                return PointerActionOutcome::Selection(selection);
            }

            if matches!(
                action,
                glyphon::Action::DoubleClick { .. } | glyphon::Action::TripleClick { .. }
            ) {
                return PointerActionOutcome::Ignored;
            }

            return self
                .display_action_result(cursor_offset, selection, action)
                .map(|(cursor_offset, selection)| PointerActionOutcome::Cursor {
                    cursor_offset,
                    selection,
                })
                .unwrap_or(PointerActionOutcome::Ignored);
        }

        if let glyphon::Action::Drag { x, y } = action
            && let Some(origin) = drag_origin_selection
            && let Some(selection) =
                self.raw_drag_selection(origin, &raw_editor, raw_text, drag_selection_mode, x, y)
        {
            return PointerActionOutcome::Selection(selection);
        }

        if let Some(selection) = self.raw_pointer_selection(&raw_editor, raw_text, action) {
            return PointerActionOutcome::Selection(selection);
        }

        PointerActionOutcome::ApplyAction(action)
    }

    #[cfg(test)]
    fn set_snapshot_buffer(
        &mut self,
        buffer: glyphon::Buffer,
        transformed_text: Option<TransformedText>,
    ) {
        self.snapshot.buffer = Some(buffer);
        self.snapshot.transformed_text = transformed_text;
    }
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
    blink_start_frame_nanos: u64,
    current_frame_nanos: u64,
    focus_handler: FocusRequester,
    edit_state: TextEditState,
    selection_state: TextSelectionState,
    scroll_state: TextScrollControllerState,
    ime_state: TextImeState,
    layout_state: TextLayoutState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextSelection {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl TextSelection {
    pub(crate) fn collapsed(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    pub(crate) fn ordered_range(&self) -> Range<usize> {
        self.start.min(self.end)..self.start.max(self.end)
    }

    pub(crate) fn is_collapsed(&self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ImeComposition {
    pub(crate) range: Range<usize>,
    pub(crate) selection: TextSelection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ImeEditResult {
    pub(crate) selection: TextSelection,
    pub(crate) replaced_range: Range<usize>,
}

fn composition_selection(
    replacement_start: usize,
    replacement_text: &str,
    cursor_offset: Option<(usize, usize)>,
    previous_selection: Option<TextSelection>,
) -> TextSelection {
    let replacement_end = replacement_start + replacement_text.len();
    if let Some((start, end)) = cursor_offset {
        TextSelection {
            start: (replacement_start + start).min(replacement_end),
            end: (replacement_start + end).min(replacement_end),
        }
    } else if let Some(previous_selection) = previous_selection {
        TextSelection {
            start: previous_selection
                .start
                .clamp(replacement_start, replacement_end),
            end: previous_selection
                .end
                .clamp(replacement_start, replacement_end),
        }
    } else {
        TextSelection::collapsed(replacement_end)
    }
}

impl TextEditorController {
    fn update_layout_geometry(
        &mut self,
        constraint: &TextConstraint,
        selection_rects: Vec<RectDef>,
        composition_rects: Vec<RectDef>,
        cursor_rect: Option<RectDef>,
        ime_rect: Option<RectDef>,
        computed_data: ComputedData,
    ) {
        self.layout_state.update_geometry_for_constraint(
            constraint,
            selection_rects,
            composition_rects,
            cursor_rect,
            ime_rect,
            computed_data,
        );
    }

    pub(crate) fn current_selection_rects(&self) -> &[RectDef] {
        self.layout_state.selection_rects()
    }

    pub(crate) fn current_composition_rects(&self) -> &[RectDef] {
        self.layout_state.composition_rects()
    }

    pub(crate) fn current_ime_rect(&self) -> Option<RectDef> {
        self.layout_state.ime_rect()
    }

    pub(crate) fn current_text_data(&self) -> Option<TextData> {
        self.layout_state.text_data()
    }

    fn cached_layout(&self, constraint: &TextConstraint) -> Option<CachedLayout> {
        self.layout_state.cached_geometry_for_constraint(constraint)
    }

    fn sync_layout_snapshot_scroll(&mut self) {
        self.layout_state
            .sync_snapshot_scroll(self.scroll_state.as_scroll());
    }

    fn sync_scroll_state_from_editor(&mut self) {
        self.scroll_state.sync_from_edit_state(&self.edit_state);
    }

    #[cfg(test)]
    fn set_layout_snapshot_buffer(
        &mut self,
        buffer: glyphon::Buffer,
        transformed_text: Option<TransformedText>,
    ) {
        self.layout_state
            .set_snapshot_buffer(buffer, transformed_text);
    }

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
        let frame_nanos = current_frame_nanos();
        let mut buffer = glyphon::Buffer::new(
            &mut write_font_system(),
            glyphon::Metrics::new(size.to_pixels_f32(), line_height_px.to_f32()),
        );
        buffer.set_wrap(&mut write_font_system(), glyphon::Wrap::Glyph);
        let scroll_state = TextScrollControllerState::new(buffer.scroll());
        let editor = glyphon::Editor::new(buffer);
        let text_color = Color::BLACK;
        let cursor_color = Color::BLACK;
        Self {
            line_height: line_height_px,
            blink_start_frame_nanos: frame_nanos,
            current_frame_nanos: frame_nanos,
            focus_handler: FocusRequester::new(),
            edit_state: TextEditState {
                editor,
                display_transform: None,
                text_color,
                cursor_color,
                single_line: false,
            },
            selection_state: TextSelectionState {
                selection_color,
                last_click_time: None,
                last_click_position: None,
                click_count: 0,
                is_dragging: false,
                drag_selection_mode: DragSelectionMode::Character,
                drag_origin_selection: None,
            },
            scroll_state,
            ime_state: TextImeState { composition: None },
            layout_state: TextLayoutState::new(),
        }
    }

    /// Returns the line height in pixels.
    pub fn line_height(&self) -> Px {
        self.line_height
    }

    // Returns the current text buffer as `TextData`, applying the given layout
    // constraints.
    fn text_data(&mut self, constraint: TextConstraint) -> TextData {
        if let Some(text_data) = self
            .layout_state
            .cached_text_data_for_constraint(&constraint, self.scroll_state.as_scroll())
        {
            return text_data;
        }

        self.edit_state
            .sync_size_and_shape_until_scroll(&constraint);

        let (text_buffer, transformed_text) =
            if let Some(transform) = self.edit_state.display_transform_ref() {
                let (metrics, scroll) = self.edit_state.metrics_and_scroll();
                let content = self.edit_state.text();
                let transformed_text = transform.call(content);
                let text_buffer = build_display_buffer(
                    transformed_text.text(),
                    self.edit_state.text_color(),
                    metrics.font_size,
                    metrics.line_height,
                    &constraint,
                    self.wrap_mode(),
                    scroll,
                );
                (text_buffer, Some(transformed_text))
            } else {
                (self.edit_state.buffer_clone(), None)
            };

        let text_data = TextData::from_buffer(text_buffer.clone());
        self.layout_state.set_text_snapshot_for_constraint(
            &constraint,
            text_buffer,
            transformed_text,
            text_data.clone(),
        );
        text_data
    }

    // Returns a reference to the internal focus handler.
    pub(crate) fn focus_handler(&self) -> &FocusRequester {
        &self.focus_handler
    }

    // Returns a mutable reference to the internal focus handler.
    pub(crate) fn focus_handler_mut(&mut self) -> &mut FocusRequester {
        &mut self.focus_handler
    }

    /// Returns a reference to the underlying `glyphon::Editor`.
    pub fn editor(&self) -> &glyphon::Editor<'static> {
        self.edit_state.editor()
    }

    /// Mutates the underlying `glyphon::Editor` and refreshes layout state.
    pub fn with_editor_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut glyphon::Editor<'static>) -> R,
    {
        let result = self.edit_state.with_editor_mut(f);
        self.reset_cursor_blink();
        self.ensure_cursor_visible();
        self.invalidate_text_layout();
        result
    }

    // Returns the cursor blink start timestamp.
    fn blink_start_frame_nanos(&self) -> u64 {
        self.blink_start_frame_nanos
    }

    // Returns the latest frame timestamp observed by this controller.
    fn current_frame_nanos(&self) -> u64 {
        self.current_frame_nanos
    }

    // Updates the latest frame timestamp used by cursor blink rendering.
    fn update_frame_nanos(&mut self, frame_nanos: u64) {
        self.current_frame_nanos = frame_nanos;
    }

    fn reset_cursor_blink(&mut self) {
        let frame_nanos = self.current_frame_nanos.max(current_frame_nanos());
        self.blink_start_frame_nanos = frame_nanos;
        self.current_frame_nanos = frame_nanos;
    }

    /// Returns the current selection highlight color.
    pub fn selection_color(&self) -> Color {
        self.selection_state.selection_color()
    }

    /// Sets the selection highlight color.
    ///
    /// # Arguments
    ///
    /// * `color` - The new selection color.
    pub fn set_selection_color(&mut self, color: Color) {
        self.selection_state.set_selection_color(color);
    }

    /// Returns the current text color.
    pub fn text_color(&self) -> Color {
        self.edit_state.text_color()
    }

    /// Sets the text color used by the editor.
    pub fn set_text_color(&mut self, color: Color) {
        if !self.edit_state.set_text_color(color) {
            return;
        }
        let current_text = self.text();
        let selection = self.selection();
        self.set_text_and_selection(&current_text, selection);
    }

    /// Returns the cursor color.
    pub fn cursor_color(&self) -> Color {
        self.edit_state.cursor_color()
    }

    /// Sets the cursor color used by the editor.
    pub fn set_cursor_color(&mut self, color: Color) {
        self.edit_state.set_cursor_color(color);
    }

    /// Sets a display transform applied when rendering text.
    pub fn set_display_transform(&mut self, transform: Option<DisplayTransform>) {
        if self.edit_state.set_display_transform(transform) {
            self.invalidate_text_layout();
        }
    }

    pub(crate) fn single_line(&self) -> bool {
        self.edit_state.single_line()
    }

    pub(crate) fn set_single_line(&mut self, single_line: bool) {
        if !self.edit_state.set_single_line(single_line) {
            return;
        }
        let wrap = self.wrap_mode();
        self.edit_state.set_wrap(wrap);
        if single_line {
            self.scroll_state.reset_vertical();
            self.edit_state.set_scroll(self.scroll_state.as_scroll());
        }
        if !single_line {
            self.sync_scroll_state_from_editor();
        }
        self.ensure_cursor_visible();
        self.invalidate_text_layout();
    }

    #[cfg(test)]
    pub(crate) fn scroll(&self) -> glyphon::cosmic_text::Scroll {
        self.scroll_state.as_scroll()
    }

    pub(crate) fn scroll_state(&self) -> TextScrollState {
        self.scroll_state.state()
    }

    pub(crate) fn scroll_horizontal_by(&mut self, delta: f32) {
        let changed = self.edit_state.scroll_horizontal_by(delta);
        if changed {
            self.sync_scroll_state_from_editor();
            self.sync_layout_snapshot_scroll();
            self.invalidate_layout_geometry();
        }
    }

    pub(crate) fn scroll_vertical_by(&mut self, delta: f32) {
        let scroll_before = self.scroll_state.as_scroll();
        self.edit_state.scroll_vertical_by(delta);
        self.sync_scroll_state_from_editor();
        if self.scroll_state.as_scroll() != scroll_before {
            self.sync_layout_snapshot_scroll();
            self.invalidate_layout_geometry();
        }
    }

    /// Returns whether a display transform is active.
    pub fn display_transform_active(&self) -> bool {
        self.edit_state.display_transform().is_some()
    }

    // Returns the current display transform handle for equality checks before
    // mutating state.
    pub(crate) fn display_transform(&self) -> Option<DisplayTransform> {
        self.edit_state.display_transform()
    }

    // Handles a mouse click event and determines the click type (single,
    // double, triple).
    pub(crate) fn handle_click(&mut self, position: PxPosition, timestamp: Instant) -> ClickType {
        self.selection_state.handle_click(position, timestamp)
    }

    // Starts a drag operation (for text selection).
    pub(crate) fn start_drag(&mut self, click_type: ClickType) {
        let origin_selection = self
            .selection_state
            .drag_origin_for_click(click_type, self.selection());
        self.selection_state
            .start_drag(click_type, origin_selection);
    }

    /// Returns `true` if a drag operation is in progress.
    pub fn is_dragging(&self) -> bool {
        self.selection_state.is_dragging()
    }

    // Stops the current drag operation.
    pub(crate) fn stop_drag(&mut self) {
        self.selection_state.stop_drag();
    }

    // Returns the last click position, if any.
    pub(crate) fn last_click_position(&self) -> Option<PxPosition> {
        self.selection_state.last_click_position()
    }

    pub(crate) fn last_click_time(&self) -> Option<Instant> {
        self.selection_state.last_click_time()
    }

    // Updates the last click position (used for drag tracking).
    pub(crate) fn update_last_click_position(&mut self, position: PxPosition) {
        self.selection_state.update_last_click_position(position);
    }

    fn ensure_cursor_visible(&mut self) {
        let scroll_after = self.edit_state.shape_until_cursor();
        if self.scroll_state.sync_from_scroll(scroll_after) {
            self.sync_layout_snapshot_scroll();
        }
    }

    fn current_editor_buffer_clone(&self) -> glyphon::Buffer {
        self.edit_state.buffer_clone()
    }

    #[cfg(test)]
    fn current_raw_layout_editor(&self) -> Option<glyphon::Editor<'static>> {
        Some(self.layout_state.raw_layout_editor(
            self.current_editor_buffer_clone(),
            self.edit_state.cursor(),
            self.selection_cursor_range(),
        ))
    }

    fn current_layout_geometry(&self) -> Option<DerivedLayoutGeometry> {
        self.layout_state.derived_geometry(DerivedGeometryInput {
            scroll_horizontal: Px(self.scroll_state.horizontal().round() as i32),
            cursor_offset: self.cursor_offset(),
            selection: self.has_selection().then(|| self.selection()),
            composition_range: self.ime_state.composition_range(),
            raw_editor: RawEditorSnapshot {
                buffer: self.current_editor_buffer_clone(),
                cursor: self.edit_state.cursor(),
                selection: self.selection_cursor_range(),
            },
            raw_composition: self.ime_state.composition_cursor_range(&self.edit_state),
        })
    }

    fn apply_cursor_and_selection_offsets(
        &mut self,
        cursor_offset: usize,
        selection: Option<TextSelection>,
    ) {
        self.edit_state
            .set_cursor_and_selection_offsets(cursor_offset, selection);
        self.reset_cursor_blink();
        self.ensure_cursor_visible();
        self.invalidate_layout_geometry();
    }

    pub(crate) fn apply_pointer_action(&mut self, action: glyphon::Action) {
        let text = self.text();
        match self
            .layout_state
            .pointer_action_outcome(PointerActionContext {
                raw_editor: RawEditorSnapshot {
                    buffer: self.current_editor_buffer_clone(),
                    cursor: self.edit_state.cursor(),
                    selection: self.selection_cursor_range(),
                },
                raw_text: &text,
                cursor_offset: self.cursor_offset(),
                selection: self.has_selection().then(|| self.selection()),
                drag_selection_mode: self.selection_state.drag_selection_mode(),
                drag_origin_selection: self.selection_state.drag_origin_selection(),
                action,
            }) {
            PointerActionOutcome::Selection(selection) => {
                self.apply_cursor_and_selection_offsets(selection.end, Some(selection));
            }
            PointerActionOutcome::Cursor {
                cursor_offset,
                selection,
            } => {
                self.apply_cursor_and_selection_offsets(cursor_offset, selection);
            }
            PointerActionOutcome::ApplyAction(action) => {
                self.edit_state.apply_action(action);
                self.reset_cursor_blink();
                self.ensure_cursor_visible();
                self.invalidate_text_layout();
            }
            PointerActionOutcome::Ignored => {}
        }
    }

    fn motion_target_offset(&mut self, motion: cosmic_text::Motion) -> Option<usize> {
        if let Some(next_offset) = self.layout_state.transformed_motion_target_offset(
            self.cursor_offset(),
            self.has_selection().then(|| self.selection()),
            motion,
        ) {
            return Some(next_offset);
        }

        self.edit_state.motion_target_offset(motion)
    }

    fn paragraph_motion_offset(&self, forward: bool) -> usize {
        self.layout_state
            .paragraph_motion_offset(&self.text(), self.cursor_offset(), forward)
    }

    fn move_cursor_with_motion(&mut self, motion: cosmic_text::Motion) -> bool {
        let Some(next_offset) = self.motion_target_offset(motion) else {
            return false;
        };
        self.apply_cursor_and_selection_offsets(next_offset, None);
        true
    }

    fn move_cursor_with_paragraph_motion(&mut self, forward: bool) -> bool {
        let next_offset = self.paragraph_motion_offset(forward);
        self.apply_cursor_and_selection_offsets(next_offset, None);
        true
    }

    fn collapse_selection(&mut self, collapse_to_end: bool) -> bool {
        let selection = self.selection();
        let Some(target) = self
            .selection_state
            .collapse_target(selection, collapse_to_end)
        else {
            return false;
        };

        self.edit_state
            .set_cursor_and_selection(self.text_offset_to_cursor(target), None);
        self.reset_cursor_blink();
        self.ensure_cursor_visible();
        self.invalidate_layout_geometry();
        true
    }

    fn extend_selection_with_motion(&mut self, motion: cosmic_text::Motion) -> bool {
        if let Some(next_offset) = self.motion_target_offset(motion) {
            let selection = self.selection_state.extended_selection(
                self.selection(),
                self.cursor_offset(),
                next_offset,
            );
            self.apply_cursor_and_selection_offsets(next_offset, selection);
            return true;
        }
        false
    }

    fn extend_selection_with_paragraph_motion(&mut self, forward: bool) -> bool {
        let next_offset = self.paragraph_motion_offset(forward);
        let selection = self.selection_state.extended_selection(
            self.selection(),
            self.cursor_offset(),
            next_offset,
        );
        self.apply_cursor_and_selection_offsets(next_offset, selection);
        true
    }

    pub(crate) fn deletion_range_for_motion(
        &mut self,
        motion: cosmic_text::Motion,
    ) -> Option<std::ops::Range<usize>> {
        let target_offset = self.motion_target_offset(motion);
        self.selection_state
            .deletion_range(self.selection(), self.cursor_offset(), target_offset)
    }

    pub(crate) fn extend_selection_to_point(&mut self, position: PxPosition) -> bool {
        let selection = self.selection();
        let cursor_offset = self.cursor_offset();
        if let Some((transformed_text, next_offset)) = self.layout_state.display_hit_offset(
            cursor_offset,
            self.has_selection().then(|| selection.clone()),
            position.x.0,
            position.y.0,
        ) {
            let next_offset = transformed_text.map_to_raw(next_offset);
            let next_selection = self.selection_state.extended_selection(
                selection.clone(),
                cursor_offset,
                next_offset,
            );
            self.apply_cursor_and_selection_offsets(next_offset, next_selection);
            return true;
        }

        let Some(next_offset) = self.layout_state.raw_hit_offset(
            self.current_editor_buffer_clone(),
            self.edit_state.cursor(),
            self.selection_cursor_range(),
            position.x.0,
            position.y.0,
        ) else {
            return false;
        };
        let next_selection =
            self.selection_state
                .extended_selection(selection, cursor_offset, next_offset);
        self.apply_cursor_and_selection_offsets(next_offset, next_selection);
        true
    }

    /// Map keyboard events to text editing actions
    /// Maps a keyboard event to a list of text editing actions for the editor.
    ///
    /// This function translates keyboard input (including modifiers) into
    /// editing actions such as character insertion, deletion, and navigation.
    ///
    /// Clipboard shortcuts are handled at a higher layer so they stay on the
    /// unified edit pipeline instead of bypassing controller state.
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
        let is_ctrl = key_modifiers.control_key() || key_modifiers.super_key();
        let is_shift = key_modifiers.shift_key();

        match key_event.state {
            winit::event::ElementState::Pressed => {}
            winit::event::ElementState::Released => return None,
        }

        match key_event.logical_key {
            winit::keyboard::Key::Named(named_key) => {
                self.handle_named_key(named_key, is_ctrl, is_shift)
            }

            winit::keyboard::Key::Character(s) => {
                if is_ctrl {
                    return None;
                }
                Some(s.chars().map(glyphon::Action::Insert).collect::<Vec<_>>())
            }
            _ => None,
        }
    }

    fn handle_named_key(
        &mut self,
        named_key: NamedKey,
        is_ctrl: bool,
        is_shift: bool,
    ) -> Option<Vec<glyphon::Action>> {
        match named_key {
            NamedKey::Backspace => Some(vec![glyphon::Action::Backspace]),
            NamedKey::Delete => Some(vec![glyphon::Action::Delete]),
            NamedKey::Enter => Some(vec![glyphon::Action::Enter]),
            NamedKey::Escape => Some(vec![glyphon::Action::Escape]),
            NamedKey::Tab => Some(vec![glyphon::Action::Insert('\t')]),
            NamedKey::ArrowLeft => {
                let motion = if is_ctrl {
                    cosmic_text::Motion::LeftWord
                } else {
                    cosmic_text::Motion::Left
                };
                if is_shift {
                    self.extend_selection_with_motion(motion);
                    None
                } else {
                    (!self.collapse_selection(false) && !self.move_cursor_with_motion(motion))
                        .then(|| vec![glyphon::Action::Motion(motion)])
                }
            }
            NamedKey::ArrowRight => {
                let motion = if is_ctrl {
                    cosmic_text::Motion::RightWord
                } else {
                    cosmic_text::Motion::Right
                };
                if is_shift {
                    self.extend_selection_with_motion(motion);
                    None
                } else {
                    (!self.collapse_selection(true) && !self.move_cursor_with_motion(motion))
                        .then(|| vec![glyphon::Action::Motion(motion)])
                }
            }
            NamedKey::ArrowUp => {
                if is_ctrl {
                    if is_shift {
                        self.extend_selection_with_paragraph_motion(false);
                    } else {
                        let _ = self.collapse_selection(false)
                            || self.move_cursor_with_paragraph_motion(false);
                    }
                    return None;
                }
                if is_shift {
                    self.extend_selection_with_motion(cosmic_text::Motion::Up);
                    return None;
                }
                if self.collapse_selection(false) {
                    return None;
                }
                if self.move_cursor_with_motion(cosmic_text::Motion::Up) {
                    return None;
                }
                // if we are on the first line, we move the cursor to the beginning of the line
                if self.edit_state.cursor_line() == 0 {
                    self.edit_state
                        .set_cursor_and_selection(self.edit_state.first_line_start_cursor(), None);
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();

                    return None;
                }

                Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Up)])
            }
            NamedKey::ArrowDown => {
                if is_ctrl {
                    if is_shift {
                        self.extend_selection_with_paragraph_motion(true);
                    } else {
                        let _ = self.collapse_selection(true)
                            || self.move_cursor_with_paragraph_motion(true);
                    }
                    return None;
                }
                if is_shift {
                    self.extend_selection_with_motion(cosmic_text::Motion::Down);
                    return None;
                }
                if self.collapse_selection(true) {
                    return None;
                }
                if self.move_cursor_with_motion(cosmic_text::Motion::Down) {
                    return None;
                }
                let last_line_index = self.edit_state.last_line_index();

                // if we are on the last line, we move the cursor to the end of the line
                if self.edit_state.cursor_line() >= last_line_index {
                    self.edit_state
                        .set_cursor_and_selection(self.edit_state.last_line_end_cursor(), None);
                    self.reset_cursor_blink();
                    self.ensure_cursor_visible();
                    return None;
                }

                Some(vec![glyphon::Action::Motion(cosmic_text::Motion::Down)])
            }
            NamedKey::PageUp => {
                if is_shift {
                    self.extend_selection_with_motion(cosmic_text::Motion::PageUp);
                    None
                } else {
                    (!self.collapse_selection(false)
                        && !self.move_cursor_with_motion(cosmic_text::Motion::PageUp))
                    .then(|| vec![glyphon::Action::Motion(cosmic_text::Motion::PageUp)])
                }
            }
            NamedKey::PageDown => {
                if is_shift {
                    self.extend_selection_with_motion(cosmic_text::Motion::PageDown);
                    None
                } else {
                    (!self.collapse_selection(true)
                        && !self.move_cursor_with_motion(cosmic_text::Motion::PageDown))
                    .then(|| vec![glyphon::Action::Motion(cosmic_text::Motion::PageDown)])
                }
            }
            NamedKey::Home => {
                let motion = if is_ctrl {
                    cosmic_text::Motion::BufferStart
                } else {
                    cosmic_text::Motion::Home
                };
                if is_shift {
                    self.extend_selection_with_motion(motion);
                    None
                } else if self.move_cursor_with_motion(motion) {
                    None
                } else {
                    Some(vec![glyphon::Action::Motion(motion)])
                }
            }
            NamedKey::End => {
                let motion = if is_ctrl {
                    cosmic_text::Motion::BufferEnd
                } else {
                    cosmic_text::Motion::End
                };
                if is_shift {
                    self.extend_selection_with_motion(motion);
                    None
                } else if self.move_cursor_with_motion(motion) {
                    None
                } else {
                    Some(vec![glyphon::Action::Motion(motion)])
                }
            }
            NamedKey::Space => Some(vec![glyphon::Action::Insert(' ')]),
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
        let cursor = self.cursor_offset();
        self.set_text_and_selection(text, TextSelection::collapsed(cursor));
    }

    pub(crate) fn text(&self) -> String {
        self.edit_state.text()
    }

    pub(crate) fn cursor_offset(&self) -> usize {
        self.edit_state.cursor_offset()
    }

    pub(crate) fn selection(&self) -> TextSelection {
        self.edit_state.selection()
    }

    fn selection_cursor_range(&self) -> Option<(Cursor, Cursor)> {
        self.selection_state
            .selection_cursor_range(self.selection(), &self.edit_state)
    }

    pub(crate) fn composition(&self) -> Option<&ImeComposition> {
        self.ime_state.composition()
    }

    #[cfg(test)]
    pub(crate) fn set_composition(&mut self, composition: Option<ImeComposition>) {
        self.ime_state.set_composition(composition);
    }

    pub(crate) fn clear_composition(&mut self) {
        self.ime_state.clear();
    }

    pub(crate) fn plan_ime_event(
        &self,
        single_line: bool,
        event: &winit::event::Ime,
    ) -> Option<PlannedImeEvent> {
        self.ime_state
            .plan_event(self.selection(), single_line, event)
    }

    pub(crate) fn commit_ime_edit(&mut self, plan: &PlannedImeEdit, result: &ImeEditResult) {
        self.ime_state.commit_edit_result(plan, result);
    }

    pub(crate) fn selected_text(&self) -> Option<String> {
        self.edit_state.selected_text()
    }

    pub(crate) fn has_selection(&self) -> bool {
        !self.selection().is_collapsed()
    }

    pub(crate) fn select_all(&mut self) {
        self.edit_state.select_all();
        self.reset_cursor_blink();
        self.ensure_cursor_visible();
        self.invalidate_layout_geometry();
    }

    pub(crate) fn apply_action_with_pipeline(
        &mut self,
        action: glyphon::Action,
        on_change: CallbackWith<String, String>,
        input_transform: Option<CallbackWith<String, String>>,
    ) {
        let preview = self.edit_state.preview_action_result(action);
        let raw_content_after_action = preview.text;
        let selection_after_action = preview.selection;
        let transformed_content = if let Some(transform) = input_transform.as_ref() {
            transform.call(raw_content_after_action.clone())
        } else {
            raw_content_after_action.clone()
        };
        let transformed_selection = rebase_selection(
            &raw_content_after_action,
            &transformed_content,
            selection_after_action.clone(),
        );
        let raw_action_matches_final = transformed_content == raw_content_after_action;

        let new_content = on_change.call(transformed_content.clone());
        let (final_content, final_selection) = if new_content != transformed_content {
            (
                new_content.clone(),
                rebase_selection(&transformed_content, &new_content, transformed_selection),
            )
        } else {
            (transformed_content, transformed_selection)
        };

        self.clear_composition();
        if raw_action_matches_final && final_selection == selection_after_action {
            self.edit_state.apply_action(action);
            self.reset_cursor_blink();
            self.ensure_cursor_visible();
            self.invalidate_text_layout();
        } else {
            self.set_text_and_selection(&final_content, final_selection);
        }
    }

    pub(crate) fn replace_text_range_with_pipeline(
        &mut self,
        range: Range<usize>,
        replacement: &str,
        selection: TextSelection,
        on_change: CallbackWith<String, String>,
        input_transform: Option<CallbackWith<String, String>>,
    ) -> ImeEditResult {
        let preview = self.edit_state.preview_replace_result(range, replacement);
        let raw_content_after_replace = preview.text;
        let raw_replaced_range = preview.replaced_range;

        let transformed_content = if let Some(transform) = input_transform.as_ref() {
            transform.call(raw_content_after_replace.clone())
        } else {
            raw_content_after_replace.clone()
        };
        let transformed_selection =
            rebase_selection(&raw_content_after_replace, &transformed_content, selection);
        let transformed_replaced_range = rebase_range(
            &raw_content_after_replace,
            &transformed_content,
            raw_replaced_range,
        );
        let new_content = on_change.call(transformed_content.clone());
        let (final_content, final_selection, final_replaced_range) =
            if new_content != transformed_content {
                (
                    new_content.clone(),
                    rebase_selection(&transformed_content, &new_content, transformed_selection),
                    rebase_range(
                        &transformed_content,
                        &new_content,
                        transformed_replaced_range,
                    ),
                )
            } else {
                (
                    transformed_content,
                    transformed_selection,
                    transformed_replaced_range,
                )
            };

        self.set_text_and_selection(&final_content, final_selection.clone());
        ImeEditResult {
            selection: final_selection,
            replaced_range: final_replaced_range,
        }
    }

    pub(crate) fn replace_selected_text_with_pipeline(
        &mut self,
        replacement: &str,
        on_change: CallbackWith<String, String>,
        input_transform: Option<CallbackWith<String, String>>,
    ) -> TextSelection {
        let selection = self.selection().ordered_range();
        self.replace_text_range_with_pipeline(
            selection.clone(),
            replacement,
            TextSelection::collapsed(selection.start + replacement.len()),
            on_change,
            input_transform,
        )
        .selection
    }

    pub(crate) fn copy_selection_to_clipboard(&self) -> bool {
        if let Some(text) = self.selected_text() {
            clipboard::set_text(&text);
            true
        } else {
            false
        }
    }

    pub(crate) fn cut_selection_with_pipeline(
        &mut self,
        on_change: CallbackWith<String, String>,
        input_transform: Option<CallbackWith<String, String>>,
    ) -> bool {
        if !self.copy_selection_to_clipboard() {
            return false;
        }
        self.replace_selected_text_with_pipeline("", on_change, input_transform);
        true
    }

    pub(crate) fn paste_from_clipboard_with_pipeline(
        &mut self,
        on_change: CallbackWith<String, String>,
        input_transform: Option<CallbackWith<String, String>>,
    ) -> bool {
        let Some(text) = clipboard::get_text() else {
            return false;
        };
        self.replace_selected_text_with_pipeline(&text, on_change, input_transform);
        true
    }

    pub(crate) fn delete_motion_with_pipeline(
        &mut self,
        motion: cosmic_text::Motion,
        on_change: CallbackWith<String, String>,
        input_transform: Option<CallbackWith<String, String>>,
    ) -> bool {
        let Some(range) = self.deletion_range_for_motion(motion) else {
            return false;
        };
        self.replace_text_range_with_pipeline(
            range.clone(),
            "",
            TextSelection::collapsed(range.start),
            on_change,
            input_transform,
        );
        true
    }

    pub(crate) fn set_text_and_selection(&mut self, text: &str, selection: TextSelection) {
        self.edit_state.set_text_and_selection(text, selection);
        self.reset_cursor_blink();
        self.ensure_cursor_visible();
        self.ime_state.clear();
        self.invalidate_text_layout();
    }

    #[cfg(test)]
    fn cursor_to_text_offset(&self, cursor: Cursor) -> usize {
        self.edit_state.cursor_to_text_offset(cursor)
    }

    fn text_offset_to_cursor(&self, offset: usize) -> Cursor {
        self.edit_state.text_offset_to_cursor(offset)
    }

    fn wrap_mode(&self) -> glyphon::Wrap {
        if self.edit_state.single_line() {
            glyphon::Wrap::None
        } else {
            glyphon::Wrap::Glyph
        }
    }

    pub(crate) fn layout_version(&self) -> u64 {
        self.layout_state.layout_version()
    }

    #[cfg(test)]
    fn text_layout_version(&self) -> u64 {
        self.layout_state.text_layout_version()
    }

    #[cfg(test)]
    fn text_cache_key(&self) -> Option<TextLayoutCacheKey> {
        self.layout_state.cache_key()
    }

    #[cfg(test)]
    fn snapshot_buffer_scroll(&self) -> Option<glyphon::cosmic_text::Scroll> {
        self.layout_state
            .snapshot
            .buffer
            .as_ref()
            .map(|buffer| buffer.scroll())
    }

    #[cfg(test)]
    fn snapshot_buffer(&self) -> Option<glyphon::Buffer> {
        self.layout_state.buffer()
    }

    pub(crate) fn invalidate_layout_geometry(&mut self) {
        self.layout_state.invalidate(LayoutInvalidation::Geometry);
    }

    pub(crate) fn invalidate_text_layout(&mut self) {
        self.layout_state.invalidate(LayoutInvalidation::Text);
    }
}

fn compute_selection_rects_for_range(
    buffer: &glyphon::Buffer,
    selection: Option<(Cursor, Cursor)>,
) -> Vec<RectDef> {
    let Some((selection_start, selection_end)) = selection else {
        return Vec::new();
    };
    compute_range_rects(buffer, selection_start, selection_end)
}

/// Compute selection rectangles for the given editor.
fn compute_selection_rects(editor: &glyphon::Editor) -> Vec<RectDef> {
    let selection = match editor.selection() {
        Selection::None => None,
        Selection::Normal(anchor) | Selection::Line(anchor) | Selection::Word(anchor) => {
            Some((anchor, editor.cursor()))
        }
    };

    editor.with_buffer(|buffer| compute_selection_rects_for_range(buffer, selection))
}

fn editor_cursor_offset(editor: &glyphon::Editor<'_>) -> usize {
    editor.with_buffer(|buffer| cursor_to_text_offset_in_buffer(buffer, editor.cursor()))
}

fn fallback_buffer_hit(buffer: &glyphon::Buffer, x: f32, y: f32) -> Option<Cursor> {
    let mut closest_line_y = None::<(f32, f32)>;

    for run in buffer.layout_runs() {
        let line_top = run.line_top;
        let line_bottom = line_top + run.line_height;
        let distance = if y < line_top {
            line_top - y
        } else if y >= line_bottom {
            y - line_bottom
        } else {
            0.0
        };
        let line_mid_y = line_top + (run.line_height * 0.5);

        match closest_line_y {
            Some((best_distance, _)) if best_distance <= distance => {}
            _ => {
                closest_line_y = Some((distance, line_mid_y));
            }
        }
    }

    let (_, line_mid_y) = closest_line_y?;
    buffer.hit(x, line_mid_y)
}

fn editor_hit_offset(editor: &glyphon::Editor<'_>, x: i32, y: i32) -> Option<usize> {
    let cursor = editor.with_buffer(|buffer| {
        buffer
            .hit(x as f32, y as f32)
            .or_else(|| fallback_buffer_hit(buffer, x as f32, y as f32))
    })?;
    Some(editor.with_buffer(|buffer| cursor_to_text_offset_in_buffer(buffer, cursor)))
}

fn editor_motion_offset(
    editor: &mut glyphon::Editor<'_>,
    motion: cosmic_text::Motion,
) -> Option<usize> {
    let cursor = editor.cursor();
    let next_cursor = editor.with_buffer_mut(|buffer| {
        buffer
            .cursor_motion(&mut write_font_system(), cursor, None, motion)
            .map(|(next_cursor, _)| next_cursor)
    })?;
    Some(editor.with_buffer(|buffer| cursor_to_text_offset_in_buffer(buffer, next_cursor)))
}

fn cursor_to_text_offset_in_buffer(buffer: &glyphon::Buffer, cursor: Cursor) -> usize {
    let mut offset = 0usize;
    for (line_index, line) in buffer.lines.iter().enumerate() {
        let line_len = line.text().len();
        if line_index == cursor.line {
            return offset + cursor.index.min(line_len);
        }
        offset += line_len + line.ending().as_str().len();
    }
    offset
}

fn text_offset_to_cursor_in_buffer(buffer: &glyphon::Buffer, offset: usize) -> Cursor {
    if buffer.lines.is_empty() {
        return Cursor::new(0, 0);
    }

    let mut remaining = offset;
    for (line_index, line) in buffer.lines.iter().enumerate() {
        let line_len = line.text().len();
        if remaining <= line_len {
            return Cursor::new(line_index, remaining);
        }

        let line_total_len = line_len + line.ending().as_str().len();
        if remaining < line_total_len {
            return Cursor::new(line_index, line_len);
        }
        remaining = remaining.saturating_sub(line_total_len);
    }

    let last_line_index = buffer.lines.len().saturating_sub(1);
    let last_line_len = buffer.lines[last_line_index].text().len();
    Cursor::new(last_line_index, last_line_len)
}

fn compute_range_rects(
    buffer: &glyphon::Buffer,
    range_start: Cursor,
    range_end: Cursor,
) -> Vec<RectDef> {
    let mut rects = Vec::new();
    let (range_start, range_end) = if cursor_to_text_offset_in_buffer(buffer, range_start)
        <= cursor_to_text_offset_in_buffer(buffer, range_end)
    {
        (range_start, range_end)
    } else {
        (range_end, range_start)
    };

    for run in buffer.layout_runs() {
        let line_top = Px(run.line_top as i32);
        let line_height = Px(run.line_height as i32);

        if let Some((x, w)) = run.highlight(range_start, range_end) {
            rects.push(RectDef {
                x: Px(x as i32),
                y: line_top,
                width: Px(w as i32),
                height: line_height,
            });
        }
    }

    rects
}

fn composition_underline_rects(rects: Vec<RectDef>) -> Vec<RectDef> {
    const UNDERLINE_HEIGHT: Px = Px(2);

    rects
        .into_iter()
        .map(|mut rect| {
            let height = UNDERLINE_HEIGHT.min(rect.height.max(Px(1)));
            rect.y += rect.height - height;
            rect.height = height;
            rect
        })
        .collect()
}

fn compute_transformed_composition_rects(
    buffer: &glyphon::Buffer,
    transformed_text: &TransformedText,
    raw_range: Range<usize>,
) -> Vec<RectDef> {
    let start =
        text_offset_to_cursor_in_buffer(buffer, transformed_text.map_from_raw(raw_range.start));
    let end = text_offset_to_cursor_in_buffer(buffer, transformed_text.map_from_raw(raw_range.end));
    composition_underline_rects(compute_range_rects(buffer, start, end))
}

fn compute_composition_rects_for_range(
    buffer: &glyphon::Buffer,
    composition: Option<(Cursor, Cursor)>,
) -> Vec<RectDef> {
    let Some((start, end)) = composition else {
        return Vec::new();
    };
    composition_underline_rects(compute_range_rects(buffer, start, end))
}

fn rect_union(rects: &[RectDef]) -> Option<RectDef> {
    let mut iter = rects.iter();
    let first = iter.next()?.clone();
    Some(iter.fold(first, |acc, rect| {
        let left = acc.x.min(rect.x);
        let top = acc.y.min(rect.y);
        let right = (acc.x + acc.width).max(rect.x + rect.width);
        let bottom = (acc.y + acc.height).max(rect.y + rect.height);
        RectDef {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        }
    }))
}

fn active_ime_rect(
    selection_rects: &[RectDef],
    composition_rects: &[RectDef],
    cursor_rect: Option<RectDef>,
) -> Option<RectDef> {
    rect_union(composition_rects)
        .or_else(|| rect_union(selection_rects))
        .or(cursor_rect)
}

fn apply_horizontal_scroll_offset(rects: &mut [RectDef], scroll_horizontal: Px) {
    if scroll_horizontal == Px::ZERO {
        return;
    }

    for rect in rects {
        rect.x -= scroll_horizontal;
    }
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

fn editor_selection(editor: &glyphon::Editor<'_>) -> TextSelection {
    match editor.selection() {
        glyphon::cosmic_text::Selection::None => {
            let cursor = editor.cursor();
            let offset =
                editor.with_buffer(|buffer| cursor_to_text_offset_in_buffer(buffer, cursor));
            TextSelection::collapsed(offset)
        }
        glyphon::cosmic_text::Selection::Normal(anchor)
        | glyphon::cosmic_text::Selection::Line(anchor)
        | glyphon::cosmic_text::Selection::Word(anchor) => TextSelection {
            start: editor.with_buffer(|buffer| cursor_to_text_offset_in_buffer(buffer, anchor)),
            end: editor
                .with_buffer(|buffer| cursor_to_text_offset_in_buffer(buffer, editor.cursor())),
        },
    }
}

fn rebase_selection(before: &str, after: &str, selection: TextSelection) -> TextSelection {
    TextSelection {
        start: rebase_offset(before, after, selection.start),
        end: rebase_offset(before, after, selection.end),
    }
}

fn rebase_range(before: &str, after: &str, range: Range<usize>) -> Range<usize> {
    let start = rebase_offset(before, after, range.start);
    let end = rebase_offset(before, after, range.end);
    start.min(end)..start.max(end)
}

fn rebase_offset(before: &str, after: &str, offset: usize) -> usize {
    TransformedText::from_strings(before, after.to_string()).map_from_raw(offset.min(before.len()))
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
    wrap: glyphon::Wrap,
    scroll: glyphon::cosmic_text::Scroll,
) -> glyphon::Buffer {
    let mut buffer = glyphon::Buffer::new(
        &mut write_font_system(),
        glyphon::Metrics::new(font_size, line_height),
    );
    buffer.set_wrap(&mut write_font_system(), wrap);
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
    buffer.set_scroll(scroll);
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

impl LayoutPolicy for TextEditLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let max_width_pixels: Option<Px> = input.parent_constraint().width().resolve_max();
        let max_height_pixels: Option<Px> = input.parent_constraint().height().resolve_max();

        let text_constraint = TextConstraint {
            max_width: max_width_pixels.map(|px: Px| px.to_f32()),
            max_height: max_height_pixels.map(|px: Px| px.to_f32()),
        };
        let text_data = self
            .controller
            .with_mut(|c| c.text_data(text_constraint.clone()));

        let place_rect_children =
            |rects: &[RectDef], child_offset: usize, result: &mut LayoutResult| {
                let child_constraint = input.parent_constraint().without_min();
                for (index, rect_def) in rects.iter().enumerate() {
                    if let Some(rect_node) = children.get(child_offset + index).copied() {
                        rect_node.measure(&child_constraint)?;
                        result.place_child(rect_node, PxPosition::new(rect_def.x, rect_def.y));
                    }
                }
                Ok::<(), MeasurementError>(())
            };

        if let Some((selection_rects, composition_rects, cursor_rect, computed_data)) =
            self.controller.with(|c| c.cached_layout(&text_constraint))
        {
            place_rect_children(&selection_rects, 0, &mut result)?;
            place_rect_children(&composition_rects, selection_rects.len(), &mut result)?;

            if let Some(cursor_rect) = cursor_rect {
                let cursor_node_index = selection_rects.len() + composition_rects.len();
                if let Some(cursor_node) = children.get(cursor_node_index).copied() {
                    let child_constraint = input.parent_constraint().without_min();
                    cursor_node.measure(&child_constraint)?;
                    result.place_child(cursor_node, PxPosition::new(cursor_rect.x, cursor_rect.y));
                }
            }

            return Ok(result.with_size(computed_data));
        }

        let line_height = self.controller.with(|c| c.line_height());
        let DerivedLayoutGeometry {
            mut selection_rects,
            mut composition_rects,
            cursor_position: cursor_pos_raw,
            scroll_horizontal,
        } = self.controller.with(|c| {
            c.current_layout_geometry()
                .expect("layout geometry should be available")
        });

        apply_horizontal_scroll_offset(&mut selection_rects, scroll_horizontal);
        apply_horizontal_scroll_offset(&mut composition_rects, scroll_horizontal);
        let selection_rects_len = selection_rects.len();
        let composition_rects_len = composition_rects.len();

        place_rect_children(&selection_rects, 0, &mut result)?;
        place_rect_children(&composition_rects, selection_rects_len, &mut result)?;

        let visible_x1 = max_width_pixels.unwrap_or(Px(i32::MAX));
        let visible_y1 = max_height_pixels.unwrap_or(Px(i32::MAX));
        selection_rects = clip_and_take_visible(selection_rects, visible_x1, visible_y1);
        composition_rects = clip_and_take_visible(composition_rects, visible_x1, visible_y1);
        let cursor_rect = cursor_pos_raw.map(|cursor_pos_raw| RectDef {
            x: Px(cursor_pos_raw.0) - scroll_horizontal,
            y: Px(cursor_pos_raw.1),
            width: CURSOR_WIDRH.to_px(),
            height: line_height,
        });
        let ime_rect = active_ime_rect(&selection_rects, &composition_rects, cursor_rect.clone());
        let constrained_height = if let Some(max_h) = max_height_pixels {
            text_data.size[1].min(max_h.abs())
        } else {
            text_data.size[1]
        };
        let measured_width = Px::from(text_data.size[0]) + CURSOR_WIDRH.to_px();
        let constrained_width = if self.controller.with(|c| c.single_line()) {
            if let Some(max_width) = max_width_pixels {
                measured_width.min(Px::from(max_width.abs()))
            } else {
                measured_width
            }
        } else {
            measured_width
        };
        let computed_data = ComputedData {
            width: constrained_width,
            height: constrained_height.into(),
        };
        self.controller.with_mut(|c| {
            c.update_layout_geometry(
                &text_constraint,
                selection_rects.clone(),
                composition_rects.clone(),
                cursor_rect.clone(),
                ime_rect,
                computed_data,
            );
        });

        if let Some(cursor_rect) = cursor_rect {
            let cursor_node_index = selection_rects_len + composition_rects_len;
            if let Some(cursor_node) = children.get(cursor_node_index).copied() {
                let child_constraint = input.parent_constraint().without_min();
                cursor_node.measure(&child_constraint)?;
                result.place_child(cursor_node, PxPosition::new(cursor_rect.x, cursor_rect.y));
            }
        }

        Ok(result.with_size(computed_data))
    }
}

impl RenderPolicy for TextEditLayout {
    fn record(&self, input: &mut RenderInput<'_>) {
        let mut metadata = input.metadata_mut();
        metadata.set_clips_children(true);
        if let Some(text_data) = self.controller.with(|c| c.current_text_data()) {
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
pub fn text_edit_core(controller: Option<State<TextEditorController>>) {
    let controller = controller.expect("text_edit_core requires a controller");
    let layout_version = controller.with(|c| c.layout_version());
    let policy = TextEditLayout {
        controller,
        layout_version,
    };
    layout()
        .layout_policy(policy.clone())
        .render_policy(policy)
        .child(move || {
            {
                let (rect_definitions, color_for_selection) = controller
                    .with(|c| (c.current_selection_rects().to_vec(), c.selection_color()));

                for def in rect_definitions {
                    selection_highlight_rect()
                        .width(def.width)
                        .height(def.height)
                        .color(color_for_selection);
                }
            }

            {
                let (rect_definitions, color_for_composition) =
                    controller.with(|c| (c.current_composition_rects().to_vec(), c.cursor_color()));

                for def in rect_definitions {
                    selection_highlight_rect()
                        .width(def.width)
                        .height(def.height)
                        .color(color_for_composition);
                }
            }

            if controller.with(|c| c.focus_handler().is_focused()) {
                let frame_nanos = current_frame_nanos();
                controller.with_mut(|controller| controller.update_frame_nanos(frame_nanos));
                receive_frame_nanos(move |frame_nanos| {
                    let is_focused = controller.with_mut(|controller| {
                        controller.update_frame_nanos(frame_nanos);
                        controller.focus_handler().is_focused()
                    });
                    if is_focused {
                        tessera_ui::FrameNanosControl::Continue
                    } else {
                        tessera_ui::FrameNanosControl::Stop
                    }
                });
                let (line_height, blink_start_frame_nanos, frame_nanos, cursor_color) = controller
                    .with(|c| {
                        (
                            c.line_height(),
                            c.blink_start_frame_nanos(),
                            c.current_frame_nanos(),
                            c.cursor_color(),
                        )
                    });
                cursor::cursor(
                    line_height,
                    blink_start_frame_nanos,
                    frame_nanos,
                    cursor_color,
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use super::{
        ClickType, RectDef, TextEditorController, TextLayoutCacheKey, TextSelection,
        TransformedText, active_ime_rect, build_display_buffer, build_display_editor,
        compute_transformed_composition_rects, text_offset_to_cursor_in_buffer, write_font_system,
    };
    use crate::pipelines::text::command::TextConstraint;
    use glyphon::{Action as GlyphonAction, Edit as _, cosmic_text::Motion};
    use tessera_ui::winit::keyboard::NamedKey;
    use tessera_ui::{ComputedData, Dp, Px};

    fn controller_with_text(text: &str) -> TextEditorController {
        let mut controller = TextEditorController::new(Dp(14.0), None);
        controller.set_text(text);
        controller
    }

    fn prepare_transformed_layout(
        controller: &mut TextEditorController,
        transformed: TransformedText,
    ) -> TransformedText {
        prepare_transformed_layout_with_height(controller, transformed, 24.0)
    }

    fn prepare_transformed_layout_with_height(
        controller: &mut TextEditorController,
        transformed: TransformedText,
        max_height: f32,
    ) -> TransformedText {
        let constraint = TextConstraint {
            max_width: Some(240.0),
            max_height: Some(max_height),
        };
        let (metrics, scroll) = controller
            .editor()
            .with_buffer(|buffer| (buffer.metrics(), buffer.scroll()));
        let buffer = build_display_buffer(
            transformed.text(),
            controller.text_color(),
            metrics.font_size,
            metrics.line_height,
            &constraint,
            controller.wrap_mode(),
            scroll,
        );
        controller.set_layout_snapshot_buffer(buffer, Some(transformed.clone()));
        transformed
    }

    #[test]
    fn text_data_reuses_cached_snapshot_for_same_layout_key() {
        let mut controller = controller_with_text("hello");
        let constraint = TextConstraint {
            max_width: Some(240.0),
            max_height: Some(80.0),
        };
        let text_cache_key = TextLayoutCacheKey::new(controller.text_layout_version(), &constraint);

        controller.text_data(constraint.clone());
        controller.update_layout_geometry(
            &constraint,
            vec![RectDef {
                x: Px(1),
                y: Px(2),
                width: Px(3),
                height: Px(4),
            }],
            Vec::new(),
            Some(RectDef {
                x: Px(5),
                y: Px(6),
                width: Px(1),
                height: Px(7),
            }),
            None,
            ComputedData {
                width: Px(11),
                height: Px(22),
            },
        );

        controller.text_data(constraint.clone());
        assert_eq!(
            controller
                .cached_layout(&constraint)
                .map(|(_, _, _, size)| size),
            Some(ComputedData {
                width: Px(11),
                height: Px(22),
            })
        );
        assert!(matches!(
            controller.text_cache_key(),
            Some(key) if key == text_cache_key
        ));

        controller.invalidate_layout_geometry();
        controller.text_data(constraint.clone());
        assert_eq!(controller.cached_layout(&constraint), None);
        assert!(matches!(
            controller.text_cache_key(),
            Some(key) if key == text_cache_key
        ));
    }

    #[test]
    fn scroll_horizontal_by_reuses_text_layout_cache_and_syncs_snapshot_scroll() {
        let mut controller =
            controller_with_text("hello world hello world hello world hello world");
        controller.set_single_line(true);
        let constraint = TextConstraint {
            max_width: Some(60.0),
            max_height: Some(40.0),
        };

        controller.text_data(constraint.clone());
        let text_layout_version = controller.text_layout_version();
        let text_cache_key = TextLayoutCacheKey::new(text_layout_version, &constraint);

        controller.scroll_horizontal_by(40.0);

        assert!(controller.scroll().horizontal > 0.0);
        assert_eq!(controller.text_layout_version(), text_layout_version);
        assert!(matches!(
            controller.text_cache_key(),
            Some(key) if key == text_cache_key
        ));
        assert_eq!(
            controller
                .snapshot_buffer_scroll()
                .expect("layout snapshot buffer should exist"),
            controller.scroll()
        );

        controller.text_data(constraint);
        assert!(matches!(
            controller.text_cache_key(),
            Some(key) if key == text_cache_key
        ));
    }

    #[test]
    fn ensure_cursor_visible_keeps_scroll_state_in_sync_with_editor_scroll() {
        let text = "hello world hello world hello world hello world";
        let mut controller = controller_with_text(text);
        controller.set_single_line(true);
        controller.text_data(TextConstraint {
            max_width: Some(60.0),
            max_height: Some(40.0),
        });
        controller.set_text_and_selection(text, TextSelection::collapsed(text.len()));

        assert!(controller.scroll_state().horizontal() > 0.0);
        assert_eq!(
            controller.scroll_state().as_scroll(),
            controller.editor().with_buffer(|buffer| buffer.scroll())
        );
    }

    #[test]
    fn scroll_vertical_by_reuses_text_layout_cache_and_syncs_scroll_state() {
        let mut controller = controller_with_text("111\n222\n333\n444\n555\n666\n777\n888");
        let constraint = TextConstraint {
            max_width: Some(120.0),
            max_height: Some(20.0),
        };

        controller.text_data(constraint.clone());
        let text_layout_version = controller.text_layout_version();
        let text_cache_key = TextLayoutCacheKey::new(text_layout_version, &constraint);

        controller.scroll_vertical_by(20.0);

        assert!(controller.scroll_state().vertical() > 0.0);
        assert_eq!(controller.text_layout_version(), text_layout_version);
        assert!(matches!(
            controller.text_cache_key(),
            Some(key) if key == text_cache_key
        ));
        assert_eq!(
            controller.scroll_state().as_scroll(),
            controller.editor().with_buffer(|buffer| buffer.scroll())
        );
    }

    #[test]
    fn selection_cursor_range_preserves_backward_selection_direction() {
        let mut controller = controller_with_text("hello");
        controller.set_text_and_selection("hello", TextSelection { start: 4, end: 1 });

        let (anchor, cursor) = controller
            .selection_cursor_range()
            .expect("selection cursor range should exist");

        assert_eq!(controller.cursor_to_text_offset(anchor), 4);
        assert_eq!(controller.cursor_to_text_offset(cursor), 1);
    }

    #[test]
    fn current_raw_layout_editor_preserves_backward_selection_direction() {
        let mut controller = controller_with_text("hello");
        controller.set_text_and_selection("hello", TextSelection { start: 4, end: 1 });
        controller.text_data(TextConstraint {
            max_width: Some(120.0),
            max_height: Some(40.0),
        });

        let raw_editor = controller
            .current_raw_layout_editor()
            .expect("raw layout editor should exist");
        let anchor = match raw_editor.selection() {
            glyphon::cosmic_text::Selection::Normal(anchor)
            | glyphon::cosmic_text::Selection::Line(anchor)
            | glyphon::cosmic_text::Selection::Word(anchor) => anchor,
            glyphon::cosmic_text::Selection::None => {
                panic!("raw layout editor should preserve selection")
            }
        };
        let cursor = raw_editor.cursor();

        raw_editor.with_buffer(|buffer| {
            assert_eq!(super::cursor_to_text_offset_in_buffer(buffer, anchor), 4);
            assert_eq!(super::cursor_to_text_offset_in_buffer(buffer, cursor), 1);
        });
    }

    #[test]
    fn current_layout_geometry_renders_backward_selection_rects() {
        let mut controller = controller_with_text("hello");
        controller.set_text_and_selection("hello", TextSelection { start: 4, end: 1 });

        let geometry = controller
            .current_layout_geometry()
            .expect("layout geometry should be available");

        assert!(!geometry.selection_rects.is_empty());
    }

    #[test]
    fn collapse_selection_left_moves_cursor_to_selection_start() {
        let mut controller = controller_with_text("hello");
        controller.set_text_and_selection("hello", TextSelection { start: 1, end: 4 });

        assert!(controller.collapse_selection(false));
        assert_eq!(controller.cursor_offset(), 1);
        assert!(controller.editor().selection_bounds().is_none());
    }

    #[test]
    fn collapse_selection_right_moves_cursor_to_selection_end() {
        let mut controller = controller_with_text("hello");
        controller.set_text_and_selection("hello", TextSelection { start: 1, end: 4 });

        assert!(controller.collapse_selection(true));
        assert_eq!(controller.cursor_offset(), 4);
        assert!(controller.editor().selection_bounds().is_none());
    }

    #[test]
    fn handle_named_key_arrow_up_collapses_selection_to_start() {
        let mut controller = controller_with_text("111\n222\n333");
        controller.set_text_and_selection("111\n222\n333", TextSelection { start: 8, end: 4 });

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowUp, false, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 4);
        assert_eq!(controller.selection(), TextSelection::collapsed(4));
    }

    #[test]
    fn handle_named_key_arrow_down_collapses_selection_to_end() {
        let mut controller = controller_with_text("111\n222\n333");
        controller.set_text_and_selection("111\n222\n333", TextSelection { start: 8, end: 4 });

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowDown, false, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 8);
        assert_eq!(controller.selection(), TextSelection::collapsed(8));
    }

    #[test]
    fn handle_named_key_page_up_collapses_selection_to_start() {
        let mut controller = controller_with_text("111\n222\n333\n444");
        controller
            .set_text_and_selection("111\n222\n333\n444", TextSelection { start: 10, end: 4 });

        assert_eq!(
            controller.handle_named_key(NamedKey::PageUp, false, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 4);
        assert_eq!(controller.selection(), TextSelection::collapsed(4));
    }

    #[test]
    fn handle_named_key_page_down_collapses_selection_to_end() {
        let mut controller = controller_with_text("111\n222\n333\n444");
        controller
            .set_text_and_selection("111\n222\n333\n444", TextSelection { start: 10, end: 4 });

        assert_eq!(
            controller.handle_named_key(NamedKey::PageDown, false, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 10);
        assert_eq!(controller.selection(), TextSelection::collapsed(10));
    }

    #[test]
    fn handle_named_key_ctrl_arrow_down_moves_to_next_paragraph_start() {
        let mut controller = controller_with_text("111\n\n222\n\n333");
        controller.set_text_and_selection("111\n\n222\n\n333", TextSelection::collapsed(1));

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowDown, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 5);

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowDown, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 10);
    }

    #[test]
    fn handle_named_key_ctrl_arrow_up_moves_to_previous_paragraph_start() {
        let mut controller = controller_with_text("111\n\n222\n\n333");
        controller.set_text_and_selection("111\n\n222\n\n333", TextSelection::collapsed(10));

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowUp, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 5);

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowUp, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 0);
    }

    #[test]
    fn handle_named_key_ctrl_arrow_up_collapses_selection_to_start() {
        let mut controller = controller_with_text("111\n\n222\n\n333");
        controller.set_text_and_selection("111\n\n222\n\n333", TextSelection { start: 9, end: 5 });

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowUp, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 5);
        assert_eq!(controller.selection(), TextSelection::collapsed(5));
    }

    #[test]
    fn handle_named_key_ctrl_arrow_down_collapses_selection_to_end() {
        let mut controller = controller_with_text("111\n\n222\n\n333");
        controller.set_text_and_selection("111\n\n222\n\n333", TextSelection { start: 9, end: 5 });

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowDown, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 9);
        assert_eq!(controller.selection(), TextSelection::collapsed(9));
    }

    #[test]
    fn handle_named_key_ctrl_shift_arrow_down_extends_to_next_paragraph_start() {
        let mut controller = controller_with_text("111\n\n222\n\n333");
        controller.set_text_and_selection("111\n\n222\n\n333", TextSelection::collapsed(1));

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowDown, true, true),
            None
        );
        assert_eq!(controller.selection(), TextSelection { start: 1, end: 5 });
    }

    #[test]
    fn handle_named_key_ctrl_shift_arrow_up_extends_to_previous_paragraph_start() {
        let mut controller = controller_with_text("111\n\n222\n\n333");
        controller.set_text_and_selection("111\n\n222\n\n333", TextSelection::collapsed(10));

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowUp, true, true),
            None
        );
        assert_eq!(controller.selection(), TextSelection { start: 10, end: 5 });
    }

    #[test]
    fn handle_named_key_ctrl_arrow_down_uses_transformed_paragraph_boundaries() {
        let mut controller = controller_with_text("123456\n\nabcdef\n\nghijkl");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings(
                "123456\n\nabcdef\n\nghijkl",
                "123 456\n\nabc def\n\nghi jkl".to_string(),
            ),
        );
        controller
            .set_text_and_selection("123456\n\nabcdef\n\nghijkl", TextSelection::collapsed(4));

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowDown, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 8);

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowDown, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 16);
    }

    #[test]
    fn handle_named_key_ctrl_arrow_up_uses_transformed_paragraph_boundaries() {
        let mut controller = controller_with_text("123456\n\nabcdef\n\nghijkl");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings(
                "123456\n\nabcdef\n\nghijkl",
                "123 456\n\nabc def\n\nghi jkl".to_string(),
            ),
        );
        controller
            .set_text_and_selection("123456\n\nabcdef\n\nghijkl", TextSelection::collapsed(16));

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowUp, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 8);

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowUp, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 0);
    }

    #[test]
    fn handle_named_key_ctrl_shift_arrow_up_uses_transformed_paragraph_boundaries() {
        let mut controller = controller_with_text("123456\n\nabcdef\n\nghijkl");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings(
                "123456\n\nabcdef\n\nghijkl",
                "123 456\n\nabc def\n\nghi jkl".to_string(),
            ),
        );
        controller
            .set_text_and_selection("123456\n\nabcdef\n\nghijkl", TextSelection::collapsed(16));

        assert_eq!(
            controller.handle_named_key(NamedKey::ArrowUp, true, true),
            None
        );
        assert_eq!(controller.selection(), TextSelection { start: 16, end: 8 });
    }

    #[test]
    fn handle_named_key_ctrl_home_uses_transformed_buffer_start() {
        let mut controller = controller_with_text("123456\nabcdef");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("123456\nabcdef", "123 456\nabc def".to_string()),
        );
        controller.set_text_and_selection("123456\nabcdef", TextSelection::collapsed(11));

        assert_eq!(
            controller.handle_named_key(NamedKey::Home, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 0);
        assert_eq!(controller.selection(), TextSelection::collapsed(0));
    }

    #[test]
    fn handle_named_key_ctrl_end_uses_transformed_buffer_end() {
        let mut controller = controller_with_text("123456\nabcdef");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("123456\nabcdef", "123 456\nabc def".to_string()),
        );
        controller.set_text_and_selection("123456\nabcdef", TextSelection::collapsed(1));

        assert_eq!(
            controller.handle_named_key(NamedKey::End, true, false),
            None
        );
        assert_eq!(controller.cursor_offset(), 13);
        assert_eq!(controller.selection(), TextSelection::collapsed(13));
    }

    #[test]
    fn handle_named_key_ctrl_shift_home_uses_transformed_buffer_start() {
        let mut controller = controller_with_text("123456\nabcdef");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("123456\nabcdef", "123 456\nabc def".to_string()),
        );
        controller.set_text_and_selection("123456\nabcdef", TextSelection::collapsed(11));

        assert_eq!(
            controller.handle_named_key(NamedKey::Home, true, true),
            None
        );
        assert_eq!(controller.selection(), TextSelection { start: 11, end: 0 });
    }

    #[test]
    fn handle_named_key_ctrl_shift_end_uses_transformed_buffer_end() {
        let mut controller = controller_with_text("123456\nabcdef");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("123456\nabcdef", "123 456\nabc def".to_string()),
        );
        controller.set_text_and_selection("123456\nabcdef", TextSelection::collapsed(1));

        assert_eq!(controller.handle_named_key(NamedKey::End, true, true), None);
        assert_eq!(controller.selection(), TextSelection { start: 1, end: 13 });
    }

    #[test]
    fn extend_selection_with_motion_creates_selection_from_cursor() {
        let mut controller = controller_with_text("hello");
        controller.set_text_and_selection("hello", TextSelection::collapsed(1));

        assert!(controller.extend_selection_with_motion(Motion::Right));
        assert_eq!(controller.selection(), TextSelection { start: 1, end: 2 });
    }

    #[test]
    fn extend_selection_with_motion_clears_when_returning_to_anchor() {
        let mut controller = controller_with_text("hello");
        controller.set_text_and_selection("hello", TextSelection::collapsed(1));

        assert!(controller.extend_selection_with_motion(Motion::Right));
        assert!(controller.extend_selection_with_motion(Motion::Left));
        assert_eq!(controller.selection(), TextSelection::collapsed(1));
        assert!(controller.editor().selection_bounds().is_none());
    }

    #[test]
    fn extend_selection_with_word_motion_reaches_next_word_boundary() {
        let mut controller = controller_with_text("hello world");
        controller.set_text_and_selection("hello world", TextSelection::collapsed(0));

        assert!(controller.extend_selection_with_motion(Motion::RightWord));
        assert_eq!(controller.selection(), TextSelection { start: 0, end: 5 });
    }

    #[test]
    fn extend_selection_to_point_selects_from_existing_cursor_anchor() {
        let mut controller = controller_with_text("hello");
        controller.set_text_and_selection("hello", TextSelection::collapsed(1));

        assert!(
            controller.extend_selection_to_point(tessera_ui::PxPosition::new(
                tessera_ui::Px(40),
                tessera_ui::Px(0),
            ))
        );
        assert!(controller.selection().end > controller.selection().start);
        assert_eq!(controller.selection().start, 1);
    }

    #[test]
    fn extend_selection_to_point_clears_when_pointing_back_to_anchor() {
        let mut controller = controller_with_text("hello");
        controller.set_text_and_selection("hello", TextSelection::collapsed(1));
        let anchor_pos = controller
            .editor()
            .cursor_position()
            .expect("cursor position should be available for shaped text");

        assert!(
            controller.extend_selection_to_point(tessera_ui::PxPosition::new(
                tessera_ui::Px(40),
                tessera_ui::Px(0),
            ))
        );
        assert!(
            controller.extend_selection_to_point(tessera_ui::PxPosition::new(
                tessera_ui::Px(anchor_pos.0),
                tessera_ui::Px(anchor_pos.1),
            ))
        );
        assert!(controller.editor().selection_bounds().is_none());
    }

    #[test]
    fn set_text_and_selection_keeps_single_line_caret_in_view() {
        let text = "hello world hello world";
        let mut controller = controller_with_text(text);
        controller.with_editor_mut(|editor| {
            editor.with_buffer_mut(|buffer| {
                buffer.set_size(&mut write_font_system(), Some(40.0), Some(20.0))
            })
        });
        controller.set_single_line(true);
        controller.set_text_and_selection(text, TextSelection::collapsed(text.len()));

        assert!(controller.scroll().horizontal > 0.0);
    }

    #[test]
    fn set_text_and_selection_resets_cursor_blink_phase() {
        let mut controller = controller_with_text("hello");
        controller.update_frame_nanos(123);
        controller.blink_start_frame_nanos = 0;

        controller.set_text_and_selection("hello!", TextSelection::collapsed(6));

        assert_eq!(controller.blink_start_frame_nanos(), 123);
        assert_eq!(controller.current_frame_nanos(), 123);
    }

    #[test]
    fn select_all_marks_full_text_selection() {
        let mut controller = controller_with_text("hello");

        controller.select_all();

        assert!(controller.has_selection());
        assert_eq!(controller.selection(), TextSelection { start: 0, end: 5 });
    }

    #[test]
    fn apply_pointer_action_maps_raw_double_click_to_word_selection() {
        let mut controller = controller_with_text("foo bar");
        controller.set_text_and_selection("foo bar", TextSelection::collapsed(5));
        let (x, y) = controller
            .editor()
            .cursor_position()
            .expect("raw cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::DoubleClick { x, y });

        assert_eq!(controller.selection(), TextSelection { start: 4, end: 7 });
        assert_eq!(controller.cursor_offset(), 7);
    }

    #[test]
    fn apply_pointer_action_maps_raw_double_click_drag_to_word_selection() {
        let mut controller = controller_with_text("foo bar baz");
        controller.set_text_and_selection("foo bar baz", TextSelection::collapsed(5));
        let (start_x, start_y) = controller
            .editor()
            .cursor_position()
            .expect("raw cursor position should be available");
        controller.set_text_and_selection("foo bar baz", TextSelection::collapsed(9));
        let (end_x, end_y) = controller
            .editor()
            .cursor_position()
            .expect("raw cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::DoubleClick {
            x: start_x,
            y: start_y,
        });
        controller.start_drag(ClickType::Double);
        controller.apply_pointer_action(GlyphonAction::Drag { x: end_x, y: end_y });

        assert_eq!(controller.selection(), TextSelection { start: 4, end: 11 });
        assert_eq!(controller.cursor_offset(), 11);
    }

    #[test]
    fn apply_pointer_action_maps_raw_double_click_drag_to_backward_word_selection() {
        let mut controller = controller_with_text("foo bar baz");
        controller.set_text_and_selection("foo bar baz", TextSelection::collapsed(5));
        let (start_x, start_y) = controller
            .editor()
            .cursor_position()
            .expect("raw cursor position should be available");
        controller.set_text_and_selection("foo bar baz", TextSelection::collapsed(1));
        let (end_x, end_y) = controller
            .editor()
            .cursor_position()
            .expect("raw cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::DoubleClick {
            x: start_x,
            y: start_y,
        });
        controller.start_drag(ClickType::Double);
        controller.apply_pointer_action(GlyphonAction::Drag { x: end_x, y: end_y });

        assert_eq!(controller.selection(), TextSelection { start: 7, end: 0 });
        assert_eq!(controller.cursor_offset(), 0);
    }

    #[test]
    fn apply_pointer_action_maps_raw_triple_click_to_line_selection() {
        let mut controller = controller_with_text("111\n222\n333");
        controller.set_text_and_selection("111\n222\n333", TextSelection::collapsed(5));
        let (x, y) = controller
            .editor()
            .cursor_position()
            .expect("raw cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::TripleClick { x, y });

        assert_eq!(controller.selection(), TextSelection { start: 4, end: 7 });
        assert_eq!(controller.cursor_offset(), 7);
    }

    #[test]
    fn apply_pointer_action_maps_raw_triple_click_drag_to_line_selection() {
        let mut controller = controller_with_text("111\n222\n333");
        controller.set_text_and_selection("111\n222\n333", TextSelection::collapsed(5));
        let (start_x, start_y) = controller
            .editor()
            .cursor_position()
            .expect("raw cursor position should be available");
        controller.set_text_and_selection("111\n222\n333", TextSelection::collapsed(9));
        let (end_x, end_y) = controller
            .editor()
            .cursor_position()
            .expect("raw cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::TripleClick {
            x: start_x,
            y: start_y,
        });
        controller.start_drag(ClickType::Triple);
        controller.apply_pointer_action(GlyphonAction::Drag { x: end_x, y: end_y });

        assert_eq!(controller.selection(), TextSelection { start: 4, end: 11 });
        assert_eq!(controller.cursor_offset(), 11);
    }

    #[test]
    fn apply_pointer_action_maps_raw_triple_click_drag_to_backward_line_selection() {
        let mut controller = controller_with_text("111\n222\n333");
        controller.set_text_and_selection("111\n222\n333", TextSelection::collapsed(5));
        let (start_x, start_y) = controller
            .editor()
            .cursor_position()
            .expect("raw cursor position should be available");
        controller.set_text_and_selection("111\n222\n333", TextSelection::collapsed(1));
        let (end_x, end_y) = controller
            .editor()
            .cursor_position()
            .expect("raw cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::TripleClick {
            x: start_x,
            y: start_y,
        });
        controller.start_drag(ClickType::Triple);
        controller.apply_pointer_action(GlyphonAction::Drag { x: end_x, y: end_y });

        assert_eq!(controller.selection(), TextSelection { start: 7, end: 0 });
        assert_eq!(controller.cursor_offset(), 0);
    }

    #[test]
    fn transformed_text_maps_inserted_formatting_offsets() {
        let transformed = TransformedText::from_strings("1234567890", "123 456 7890".to_string());

        assert_eq!(transformed.map_from_raw(3), 4);
        assert_eq!(transformed.map_from_raw(6), 8);
        assert_eq!(transformed.map_to_raw(4), 3);
        assert_eq!(transformed.map_to_raw(8), 6);
    }

    #[test]
    fn transformed_text_maps_same_char_count_with_multibyte_mask() {
        let transformed = TransformedText::from_strings("abcd", "••••".to_string());

        assert_eq!(transformed.map_from_raw(1), 3);
        assert_eq!(transformed.map_from_raw(4), 12);
        assert_eq!(transformed.map_to_raw(3), 1);
        assert_eq!(transformed.map_to_raw(12), 4);
    }

    #[test]
    fn apply_pointer_action_maps_transformed_click_to_raw_cursor_offset() {
        let mut controller = controller_with_text("1234567890");
        let transformed = prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("1234567890", "123 456 7890".to_string()),
        );
        let buffer = controller
            .snapshot_buffer()
            .expect("transformed layout buffer should exist");
        let display_offset = 4;
        let display_editor = build_display_editor(
            buffer.clone(),
            text_offset_to_cursor_in_buffer(&buffer, display_offset),
            None,
        );
        let (x, y) = display_editor
            .cursor_position()
            .expect("transformed cursor position should be available");

        controller.set_layout_snapshot_buffer(buffer, Some(transformed));
        controller.apply_pointer_action(GlyphonAction::Click { x, y });

        assert_eq!(controller.cursor_offset(), 3);
    }

    #[test]
    fn apply_pointer_action_click_below_single_line_moves_cursor_to_nearest_offset() {
        let mut controller = controller_with_text("hello");
        let constraint = TextConstraint {
            max_width: Some(240.0),
            max_height: Some(80.0),
        };
        controller.text_data(constraint);
        let buffer = controller
            .snapshot_buffer()
            .expect("raw layout buffer should exist");
        let editor = build_display_editor(
            buffer.clone(),
            text_offset_to_cursor_in_buffer(&buffer, "hello".len()),
            None,
        );
        let (x, y) = editor
            .cursor_position()
            .expect("cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::Click { x, y: y + 24 });

        assert_eq!(controller.cursor_offset(), "hello".len());
    }

    #[test]
    fn apply_pointer_action_maps_transformed_double_click_to_raw_word_selection() {
        let mut controller = controller_with_text("1234567890");
        let transformed = prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("1234567890", "123 456 7890".to_string()),
        );
        let buffer = controller
            .snapshot_buffer()
            .expect("transformed layout buffer should exist");
        let display_offset = transformed.map_from_raw(4);
        let display_editor = build_display_editor(
            buffer,
            text_offset_to_cursor_in_buffer(
                &controller
                    .snapshot_buffer()
                    .expect("transformed layout buffer should exist"),
                display_offset,
            ),
            None,
        );
        let (x, y) = display_editor
            .cursor_position()
            .expect("transformed cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::DoubleClick { x, y });

        assert_eq!(controller.selection(), TextSelection { start: 3, end: 6 });
        assert_eq!(controller.cursor_offset(), 6);
    }

    #[test]
    fn apply_pointer_action_maps_transformed_double_click_drag_to_raw_word_selection() {
        let mut controller = controller_with_text("1234567890");
        let transformed = prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("1234567890", "123 456 7890".to_string()),
        );
        let buffer = controller
            .snapshot_buffer()
            .expect("transformed layout buffer should exist");
        let start_editor = build_display_editor(
            buffer.clone(),
            text_offset_to_cursor_in_buffer(&buffer, transformed.map_from_raw(4)),
            None,
        );
        let end_editor = build_display_editor(
            buffer,
            text_offset_to_cursor_in_buffer(
                &controller
                    .snapshot_buffer()
                    .expect("transformed layout buffer should exist"),
                transformed.map_from_raw(8),
            ),
            None,
        );
        let (start_x, start_y) = start_editor
            .cursor_position()
            .expect("transformed cursor position should be available");
        let (end_x, end_y) = end_editor
            .cursor_position()
            .expect("transformed cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::DoubleClick {
            x: start_x,
            y: start_y,
        });
        controller.start_drag(ClickType::Double);
        controller.apply_pointer_action(GlyphonAction::Drag { x: end_x, y: end_y });

        assert_eq!(controller.selection(), TextSelection { start: 3, end: 10 });
        assert_eq!(controller.cursor_offset(), 10);
    }

    #[test]
    fn apply_pointer_action_maps_transformed_double_click_drag_to_backward_word_selection() {
        let mut controller = controller_with_text("1234567890");
        let transformed = prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("1234567890", "123 456 7890".to_string()),
        );
        let buffer = controller
            .snapshot_buffer()
            .expect("transformed layout buffer should exist");
        let start_editor = build_display_editor(
            buffer.clone(),
            text_offset_to_cursor_in_buffer(&buffer, transformed.map_from_raw(4)),
            None,
        );
        let end_editor = build_display_editor(
            buffer,
            text_offset_to_cursor_in_buffer(
                &controller
                    .snapshot_buffer()
                    .expect("transformed layout buffer should exist"),
                transformed.map_from_raw(1),
            ),
            None,
        );
        let (start_x, start_y) = start_editor
            .cursor_position()
            .expect("transformed cursor position should be available");
        let (end_x, end_y) = end_editor
            .cursor_position()
            .expect("transformed cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::DoubleClick {
            x: start_x,
            y: start_y,
        });
        controller.start_drag(ClickType::Double);
        controller.apply_pointer_action(GlyphonAction::Drag { x: end_x, y: end_y });

        assert_eq!(controller.selection(), TextSelection { start: 6, end: 0 });
        assert_eq!(controller.cursor_offset(), 0);
    }

    #[test]
    fn apply_pointer_action_preserves_backward_transformed_drag_selection_direction() {
        let mut controller = controller_with_text("1234567890");
        let transformed = prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("1234567890", "123 456 7890".to_string()),
        );
        let buffer = controller
            .snapshot_buffer()
            .expect("transformed layout buffer should exist");
        let start_editor = build_display_editor(
            buffer.clone(),
            text_offset_to_cursor_in_buffer(&buffer, transformed.map_from_raw(6)),
            None,
        );
        let end_editor = build_display_editor(
            buffer,
            text_offset_to_cursor_in_buffer(
                &controller
                    .snapshot_buffer()
                    .expect("transformed layout buffer should exist"),
                transformed.map_from_raw(3),
            ),
            None,
        );
        let (start_x, start_y) = start_editor
            .cursor_position()
            .expect("transformed cursor position should be available");
        let (end_x, end_y) = end_editor
            .cursor_position()
            .expect("transformed cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::Click {
            x: start_x,
            y: start_y,
        });
        controller.apply_pointer_action(GlyphonAction::Drag { x: end_x, y: end_y });

        assert_eq!(controller.selection(), TextSelection { start: 6, end: 3 });
        assert_eq!(controller.cursor_offset(), 3);
    }

    #[test]
    fn apply_pointer_action_maps_transformed_triple_click_to_raw_line_selection() {
        let mut controller = controller_with_text("123456\nabcdef");
        let transformed =
            TransformedText::from_strings("123456\nabcdef", "123 456\nabc def".to_string());
        let constraint = TextConstraint {
            max_width: Some(240.0),
            max_height: Some(80.0),
        };
        let (metrics, scroll) = controller
            .editor()
            .with_buffer(|buffer| (buffer.metrics(), buffer.scroll()));
        let buffer = build_display_buffer(
            transformed.text(),
            controller.text_color(),
            metrics.font_size,
            metrics.line_height,
            &constraint,
            controller.wrap_mode(),
            scroll,
        );
        controller.set_layout_snapshot_buffer(buffer.clone(), Some(transformed.clone()));

        let display_editor = build_display_editor(
            buffer,
            text_offset_to_cursor_in_buffer(
                &controller
                    .snapshot_buffer()
                    .expect("transformed layout buffer should exist"),
                transformed.map_from_raw(9),
            ),
            None,
        );
        let (x, y) = display_editor
            .cursor_position()
            .expect("transformed cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::TripleClick { x, y });

        assert_eq!(controller.selection(), TextSelection { start: 7, end: 13 });
        assert_eq!(controller.cursor_offset(), 13);
    }

    #[test]
    fn apply_pointer_action_maps_transformed_triple_click_drag_to_raw_line_selection() {
        let mut controller = controller_with_text("111\n222\n333");
        let transformed = prepare_transformed_layout_with_height(
            &mut controller,
            TransformedText::from_strings("111\n222\n333", "1 11\n2 22\n3 33".to_string()),
            80.0,
        );
        let buffer = controller
            .snapshot_buffer()
            .expect("transformed layout buffer should exist");
        let start_editor = build_display_editor(
            buffer.clone(),
            text_offset_to_cursor_in_buffer(&buffer, transformed.map_from_raw(5)),
            None,
        );
        let end_editor = build_display_editor(
            buffer,
            text_offset_to_cursor_in_buffer(
                &controller
                    .snapshot_buffer()
                    .expect("transformed layout buffer should exist"),
                transformed.map_from_raw(9),
            ),
            None,
        );
        let (start_x, start_y) = start_editor
            .cursor_position()
            .expect("transformed cursor position should be available");
        let (end_x, end_y) = end_editor
            .cursor_position()
            .expect("transformed cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::TripleClick {
            x: start_x,
            y: start_y,
        });
        controller.start_drag(ClickType::Triple);
        controller.apply_pointer_action(GlyphonAction::Drag { x: end_x, y: end_y });

        assert_eq!(controller.selection(), TextSelection { start: 4, end: 11 });
        assert_eq!(controller.cursor_offset(), 11);
    }

    #[test]
    fn apply_pointer_action_maps_transformed_triple_click_drag_to_backward_line_selection() {
        let mut controller = controller_with_text("111\n222\n333");
        let transformed = prepare_transformed_layout_with_height(
            &mut controller,
            TransformedText::from_strings("111\n222\n333", "1 11\n2 22\n3 33".to_string()),
            80.0,
        );
        let buffer = controller
            .snapshot_buffer()
            .expect("transformed layout buffer should exist");
        let start_editor = build_display_editor(
            buffer.clone(),
            text_offset_to_cursor_in_buffer(&buffer, transformed.map_from_raw(5)),
            None,
        );
        let end_editor = build_display_editor(
            buffer,
            text_offset_to_cursor_in_buffer(
                &controller
                    .snapshot_buffer()
                    .expect("transformed layout buffer should exist"),
                transformed.map_from_raw(1),
            ),
            None,
        );
        let (start_x, start_y) = start_editor
            .cursor_position()
            .expect("transformed cursor position should be available");
        let (end_x, end_y) = end_editor
            .cursor_position()
            .expect("transformed cursor position should be available");

        controller.apply_pointer_action(GlyphonAction::TripleClick {
            x: start_x,
            y: start_y,
        });
        controller.start_drag(ClickType::Triple);
        controller.apply_pointer_action(GlyphonAction::Drag { x: end_x, y: end_y });

        assert_eq!(controller.selection(), TextSelection { start: 7, end: 0 });
        assert_eq!(controller.cursor_offset(), 0);
    }

    #[test]
    fn move_cursor_with_motion_uses_transformed_vertical_offsets() {
        let mut controller = controller_with_text("123456\nabcdef");
        let transformed =
            TransformedText::from_strings("123456\nabcdef", "123 456\nabc def".to_string());
        let constraint = TextConstraint {
            max_width: Some(240.0),
            max_height: Some(80.0),
        };
        let (metrics, scroll) = controller
            .editor()
            .with_buffer(|buffer| (buffer.metrics(), buffer.scroll()));
        let buffer = build_display_buffer(
            transformed.text(),
            controller.text_color(),
            metrics.font_size,
            metrics.line_height,
            &constraint,
            controller.wrap_mode(),
            scroll,
        );
        controller.set_layout_snapshot_buffer(buffer, Some(transformed));
        controller.set_text_and_selection("123456\nabcdef", TextSelection::collapsed(4));

        assert!(controller.move_cursor_with_motion(Motion::Down));
        assert_eq!(controller.cursor_offset(), 11);

        assert!(controller.move_cursor_with_motion(Motion::Up));
        assert_eq!(controller.cursor_offset(), 4);
    }

    #[test]
    fn move_cursor_with_motion_uses_transformed_line_boundaries() {
        let mut controller = controller_with_text("123456\nabcdef");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("123456\nabcdef", "123 456\nabc def".to_string()),
        );
        controller.set_text_and_selection("123456\nabcdef", TextSelection::collapsed(4));

        assert!(controller.move_cursor_with_motion(Motion::Home));
        assert_eq!(controller.cursor_offset(), 0);

        assert!(controller.move_cursor_with_motion(Motion::End));
        assert_eq!(controller.cursor_offset(), 6);
    }

    #[test]
    fn extend_selection_with_motion_uses_transformed_line_boundaries() {
        let mut controller = controller_with_text("123456\nabcdef");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("123456\nabcdef", "123 456\nabc def".to_string()),
        );
        controller.set_text_and_selection("123456\nabcdef", TextSelection::collapsed(4));

        assert!(controller.extend_selection_with_motion(Motion::Home));
        assert_eq!(controller.selection(), TextSelection { start: 4, end: 0 });

        controller.set_text_and_selection("123456\nabcdef", TextSelection::collapsed(4));
        assert!(controller.extend_selection_with_motion(Motion::End));
        assert_eq!(controller.selection(), TextSelection { start: 4, end: 6 });
    }

    #[test]
    fn move_cursor_with_motion_uses_transformed_page_vertical_offsets() {
        let mut controller = controller_with_text("111\n222\n333\n444");
        prepare_transformed_layout_with_height(
            &mut controller,
            TransformedText::from_strings(
                "111\n222\n333\n444",
                "1 11\n2 22\n3 33\n4 44".to_string(),
            ),
            40.0,
        );
        controller.set_text_and_selection("111\n222\n333\n444", TextSelection::collapsed(1));

        assert!(controller.move_cursor_with_motion(Motion::PageDown));
        assert_eq!(controller.cursor_offset(), 9);

        assert!(controller.move_cursor_with_motion(Motion::PageUp));
        assert_eq!(controller.cursor_offset(), 1);
    }

    #[test]
    fn extend_selection_with_motion_uses_transformed_page_vertical_offsets() {
        let mut controller = controller_with_text("111\n222\n333\n444");
        prepare_transformed_layout_with_height(
            &mut controller,
            TransformedText::from_strings(
                "111\n222\n333\n444",
                "1 11\n2 22\n3 33\n4 44".to_string(),
            ),
            40.0,
        );
        controller.set_text_and_selection("111\n222\n333\n444", TextSelection::collapsed(1));

        assert!(controller.extend_selection_with_motion(Motion::PageDown));
        assert_eq!(controller.selection(), TextSelection { start: 1, end: 9 });
    }

    #[test]
    fn move_cursor_with_motion_uses_transformed_word_boundaries() {
        let mut controller = controller_with_text("1234567890");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("1234567890", "123 456 7890".to_string()),
        );
        controller.set_text_and_selection("1234567890", TextSelection::collapsed(0));

        assert!(controller.move_cursor_with_motion(Motion::RightWord));
        assert_eq!(controller.cursor_offset(), 3);

        assert!(controller.move_cursor_with_motion(Motion::RightWord));
        assert_eq!(controller.cursor_offset(), 6);
    }

    #[test]
    fn extend_selection_with_motion_uses_transformed_word_boundaries() {
        let mut controller = controller_with_text("1234567890");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("1234567890", "123 456 7890".to_string()),
        );
        controller.set_text_and_selection("1234567890", TextSelection::collapsed(0));

        assert!(controller.extend_selection_with_motion(Motion::RightWord));
        assert_eq!(controller.selection(), TextSelection { start: 0, end: 3 });
    }

    #[test]
    fn deletion_range_for_motion_uses_transformed_word_boundaries() {
        let mut controller = controller_with_text("1234567890");
        prepare_transformed_layout(
            &mut controller,
            TransformedText::from_strings("1234567890", "123 456 7890".to_string()),
        );
        controller.set_text_and_selection("1234567890", TextSelection::collapsed(6));

        assert_eq!(
            controller.deletion_range_for_motion(Motion::LeftWord),
            Some(3..6)
        );
        assert_eq!(
            controller.deletion_range_for_motion(Motion::RightWord),
            Some(6..10)
        );
    }

    #[test]
    fn deletion_range_for_motion_prefers_existing_selection() {
        let mut controller = controller_with_text("hello world");
        controller.set_text_and_selection("hello world", TextSelection { start: 8, end: 2 });

        assert_eq!(
            controller.deletion_range_for_motion(Motion::LeftWord),
            Some(2..8)
        );
    }

    #[test]
    fn transformed_composition_rects_follow_mapped_display_offsets() {
        let transformed = TransformedText::from_strings("1234567890", "123 456 7890".to_string());
        let constraint = TextConstraint {
            max_width: Some(240.0),
            max_height: Some(24.0),
        };
        let buffer = build_display_buffer(
            transformed.text(),
            tessera_ui::Color::BLACK,
            14.0,
            18.0,
            &constraint,
            glyphon::Wrap::None,
            glyphon::cosmic_text::Scroll::default(),
        );
        let expected_start = text_offset_to_cursor_in_buffer(&buffer, transformed.map_from_raw(3));
        let expected_x = build_display_editor(buffer.clone(), expected_start, None)
            .cursor_position()
            .expect("cursor position should be available")
            .0;

        let rects = compute_transformed_composition_rects(&buffer, &transformed, 3..6);

        assert!(!rects.is_empty());
        assert_eq!(rects[0].x, tessera_ui::Px(expected_x));
    }

    #[test]
    fn active_ime_rect_prefers_composition_over_selection_and_cursor() {
        let selection_rects = vec![super::RectDef {
            x: tessera_ui::Px(4),
            y: tessera_ui::Px(8),
            width: tessera_ui::Px(20),
            height: tessera_ui::Px(18),
        }];
        let composition_rects = vec![super::RectDef {
            x: tessera_ui::Px(12),
            y: tessera_ui::Px(10),
            width: tessera_ui::Px(30),
            height: tessera_ui::Px(2),
        }];
        let cursor_rect = Some(super::RectDef {
            x: tessera_ui::Px(2),
            y: tessera_ui::Px(8),
            width: tessera_ui::Px(1),
            height: tessera_ui::Px(18),
        });

        assert_eq!(
            active_ime_rect(&selection_rects, &composition_rects, cursor_rect),
            composition_rects.into_iter().next()
        );
    }

    #[test]
    fn active_ime_rect_unions_selection_rects_before_cursor_fallback() {
        let selection_rects = vec![
            super::RectDef {
                x: tessera_ui::Px(4),
                y: tessera_ui::Px(8),
                width: tessera_ui::Px(20),
                height: tessera_ui::Px(18),
            },
            super::RectDef {
                x: tessera_ui::Px(30),
                y: tessera_ui::Px(8),
                width: tessera_ui::Px(10),
                height: tessera_ui::Px(18),
            },
        ];

        assert_eq!(
            active_ime_rect(&selection_rects, &[], None),
            Some(super::RectDef {
                x: tessera_ui::Px(4),
                y: tessera_ui::Px(8),
                width: tessera_ui::Px(36),
                height: tessera_ui::Px(18),
            })
        );
    }
}
