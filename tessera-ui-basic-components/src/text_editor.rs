//! A multi-line text editor component.
//!
//! ## Usage
//!
//! Use for text input fields, forms, or any place that requires editable text.
use std::sync::Arc;

use derive_builder::Builder;
use glyphon::{Action as GlyphonAction, Edit};
use parking_lot::RwLock;
use tessera_ui::{
    Color, CursorEventContent, DimensionValue, Dp, ImeRequest, Px, PxPosition, accesskit::Role,
    tessera, winit,
};

use crate::{
    pipelines::write_font_system,
    pos_misc::is_position_in_component,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    text_edit_core::{ClickType, text_edit_core},
};

/// State structure for the text editor, managing text content, cursor, selection, and editing logic.
pub use crate::text_edit_core::TextEditorState;

/// Arguments for configuring the [`text_editor`] component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct TextEditorArgs {
    /// Width constraint for the text editor. Defaults to `Wrap`.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub width: DimensionValue,
    /// Height constraint for the text editor. Defaults to `Wrap`.
    #[builder(default = "DimensionValue::WRAP", setter(into))]
    pub height: DimensionValue,
    /// Called when the text content changes. The closure receives the new text content and returns the updated content.
    #[builder(default = "Arc::new(|_| { String::new() })")]
    pub on_change: Arc<dyn Fn(String) -> String + Send + Sync>,
    /// Minimum width in density-independent pixels. Defaults to 120dp if not specified.
    #[builder(default = "None")]
    pub min_width: Option<Dp>,
    /// Minimum height in density-independent pixels. Defaults to line height + padding if not specified.
    #[builder(default = "None")]
    pub min_height: Option<Dp>,
    /// Background color of the text editor (RGBA). Defaults to light gray.
    #[builder(default = "None")]
    pub background_color: Option<Color>,
    /// Border width in Dp. Defaults to 1.0 Dp.
    #[builder(default = "Dp(1.0)")]
    pub border_width: Dp,
    /// Border color (RGBA). Defaults to gray.
    #[builder(default = "None")]
    pub border_color: Option<Color>,
    /// The shape of the text editor container.
    #[builder(default = "Shape::RoundedRectangle {
                            top_left: Dp(4.0),
                            top_right: Dp(4.0),
                            bottom_right: Dp(4.0),
                            bottom_left: Dp(4.0),
                            g2_k_value: 3.0,
                        }")]
    pub shape: Shape,
    /// Padding inside the text editor. Defaults to 5.0 Dp.
    #[builder(default = "Dp(5.0)")]
    pub padding: Dp,
    /// Border color when focused (RGBA). Defaults to blue.
    #[builder(default = "None")]
    pub focus_border_color: Option<Color>,
    /// Background color when focused (RGBA). Defaults to white.
    #[builder(default = "None")]
    pub focus_background_color: Option<Color>,
    /// Color for text selection highlight (RGBA). Defaults to light blue with transparency.
    #[builder(default = "Some(Color::new(0.5, 0.7, 1.0, 0.4))")]
    pub selection_color: Option<Color>,
    /// Optional label announced by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional description announced by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
}

impl Default for TextEditorArgs {
    fn default() -> Self {
        TextEditorArgsBuilder::default().build().unwrap()
    }
}

/// # text_editor
///
/// Renders a multi-line, editable text field.
///
/// ## Usage
///
/// Create an interactive text editor for forms, note-taking, or other text input scenarios.
///
/// ## Parameters
///
/// - `args` — configures the editor's appearance and layout; see [`TextEditorArgs`].
/// - `state` — an `Arc<RwLock<TextEditorState>>` to manage the editor's content, cursor, and selection.
///
/// ## Examples
///
/// ```
/// use std::sync::Arc;
/// use parking_lot::RwLock;
/// use tessera_ui::Dp;
/// use tessera_ui_basic_components::{
///     text_editor::{text_editor, TextEditorArgsBuilder, TextEditorState},
///     pipelines::write_font_system,
/// };
///
/// // In a real app, you would manage this state.
/// let editor_state = Arc::new(RwLock::new(TextEditorState::new(Dp(14.0), None)));
/// editor_state.write().editor_mut().set_text_reactive(
///     "Initial text",
///     &mut write_font_system(),
///     &glyphon::Attrs::new().family(glyphon::fontdb::Family::SansSerif),
/// );
///
/// text_editor(
///     TextEditorArgsBuilder::default()
///         .padding(Dp(8.0))
///         .build()
///         .unwrap(),
///     editor_state,
/// );
/// ```
#[tessera]
pub fn text_editor(args: impl Into<TextEditorArgs>, state: Arc<RwLock<TextEditorState>>) {
    let editor_args: TextEditorArgs = args.into();
    let on_change = editor_args.on_change.clone();

    // Update the state with the selection color from args
    if let Some(selection_color) = editor_args.selection_color {
        state.write().set_selection_color(selection_color);
    }

    // surface layer - provides visual container and minimum size guarantee
    {
        let state_for_surface = state.clone();
        let args_for_surface = editor_args.clone();
        surface(
            create_surface_args(&args_for_surface, &state_for_surface),
            None, // text editors are not interactive at surface level
            move || {
                // Core layer - handles text rendering and editing logic
                text_edit_core(state_for_surface.clone());
            },
        );
    }

    // Event handling at the outermost layer - can access full surface area

    let args_for_handler = editor_args.clone();
    let state_for_handler = state.clone();
    input_handler(Box::new(move |mut input| {
        let size = input.computed_data; // This is the full surface size
        let cursor_pos_option = input.cursor_position_rel;
        let is_cursor_in_editor = cursor_pos_option
            .map(|pos| is_position_in_component(size, pos))
            .unwrap_or(false);

        // Set text input cursor when hovering
        if is_cursor_in_editor {
            input.requests.cursor_icon = winit::window::CursorIcon::Text;
        }

        // Handle click events - now we have a full clickable area from surface
        if is_cursor_in_editor {
            // Handle mouse pressed events
            let click_events: Vec<_> = input
                .cursor_events
                .iter()
                .filter(|event| matches!(event.content, CursorEventContent::Pressed(_)))
                .collect();

            // Handle mouse released events (end of drag)
            let release_events: Vec<_> = input
                .cursor_events
                .iter()
                .filter(|event| matches!(event.content, CursorEventContent::Released(_)))
                .collect();

            if !click_events.is_empty() {
                // Request focus if not already focused
                if !state_for_handler.read().focus_handler().is_focused() {
                    state_for_handler
                        .write()
                        .focus_handler_mut()
                        .request_focus();
                }

                // Handle cursor positioning for clicks
                if let Some(cursor_pos) = cursor_pos_option {
                    // Calculate the relative position within the text area
                    let padding_px: Px = args_for_handler.padding.into();
                    let border_width_px = Px(args_for_handler.border_width.to_pixels_u32() as i32); // Assuming border_width is integer pixels

                    let text_relative_x_px = cursor_pos.x - padding_px - border_width_px;
                    let text_relative_y_px = cursor_pos.y - padding_px - border_width_px;

                    // Only process if the click is within the text area (non-negative relative coords)
                    if text_relative_x_px >= Px(0) && text_relative_y_px >= Px(0) {
                        let text_relative_pos =
                            PxPosition::new(text_relative_x_px, text_relative_y_px);
                        // Determine click type and handle accordingly
                        let click_type = state_for_handler
                            .write()
                            .handle_click(text_relative_pos, click_events[0].timestamp);

                        match click_type {
                            ClickType::Single => {
                                // Single click: position cursor
                                state_for_handler.write().editor_mut().action(
                                    &mut write_font_system(),
                                    GlyphonAction::Click {
                                        x: text_relative_pos.x.0,
                                        y: text_relative_pos.y.0,
                                    },
                                );
                            }
                            ClickType::Double => {
                                // Double click: select word
                                state_for_handler.write().editor_mut().action(
                                    &mut write_font_system(),
                                    GlyphonAction::DoubleClick {
                                        x: text_relative_pos.x.0,
                                        y: text_relative_pos.y.0,
                                    },
                                );
                            }
                            ClickType::Triple => {
                                // Triple click: select line
                                state_for_handler.write().editor_mut().action(
                                    &mut write_font_system(),
                                    GlyphonAction::TripleClick {
                                        x: text_relative_pos.x.0,
                                        y: text_relative_pos.y.0,
                                    },
                                );
                            }
                        }

                        // Start potential drag operation
                        state_for_handler.write().start_drag();
                    }
                }
            }

            // Handle drag events (mouse move while dragging)
            // This happens every frame when cursor position changes during drag
            if state_for_handler.read().is_dragging()
                && let Some(cursor_pos) = cursor_pos_option
            {
                let padding_px: Px = args_for_handler.padding.into();
                let border_width_px = Px(args_for_handler.border_width.to_pixels_u32() as i32);

                let text_relative_x_px = cursor_pos.x - padding_px - border_width_px;
                let text_relative_y_px = cursor_pos.y - padding_px - border_width_px;

                if text_relative_x_px >= Px(0) && text_relative_y_px >= Px(0) {
                    let current_pos_px = PxPosition::new(text_relative_x_px, text_relative_y_px);
                    let last_pos_px = state_for_handler.read().last_click_position();

                    if last_pos_px != Some(current_pos_px) {
                        // Extend selection by dragging
                        state_for_handler.write().editor_mut().action(
                            &mut write_font_system(),
                            GlyphonAction::Drag {
                                x: current_pos_px.x.0,
                                y: current_pos_px.y.0,
                            },
                        );

                        // Update last position to current position
                        state_for_handler
                            .write()
                            .update_last_click_position(current_pos_px);
                    }
                }
            }

            // Handle mouse release events (end drag)
            if !release_events.is_empty() {
                state_for_handler.write().stop_drag();
            }

            let scroll_events: Vec<_> = input
                .cursor_events
                .iter()
                .filter_map(|event| match &event.content {
                    CursorEventContent::Scroll(scroll_event) => Some(scroll_event),
                    _ => None,
                })
                .collect();

            // Handle scroll events (only when focused and cursor is in editor)
            if state_for_handler.read().focus_handler().is_focused() {
                for scroll_event in scroll_events {
                    // Convert scroll delta to lines
                    let scroll = -scroll_event.delta_y;

                    // Scroll up for positive, down for negative
                    let action = GlyphonAction::Scroll { pixels: scroll };
                    state_for_handler
                        .write()
                        .editor_mut()
                        .action(&mut write_font_system(), action);
                }
            }

            // Only block cursor events when focused to prevent propagation
            if state_for_handler.read().focus_handler().is_focused() {
                input.cursor_events.clear();
            }
        }

        // Handle keyboard events (only when focused)
        if state_for_handler.read().focus_handler().is_focused() {
            // Handle keyboard events
            let is_ctrl = input.key_modifiers.control_key() || input.key_modifiers.super_key();

            // Custom handling for Ctrl+A (Select All)
            let select_all_event_index = input.keyboard_events.iter().position(|key_event| {
                if let winit::keyboard::Key::Character(s) = &key_event.logical_key {
                    is_ctrl
                        && s.to_lowercase() == "a"
                        && key_event.state == winit::event::ElementState::Pressed
                } else {
                    false
                }
            });

            if let Some(_index) = select_all_event_index {
                let mut state = state_for_handler.write();
                let editor = state.editor_mut();
                // Set cursor to the beginning of the document
                editor.set_cursor(glyphon::Cursor::new(0, 0));
                // Set selection to start from the beginning
                editor.set_selection(glyphon::cosmic_text::Selection::Normal(
                    glyphon::Cursor::new(0, 0),
                ));
                // Move cursor to the end, which extends the selection (use BufferEnd for full document)
                editor.action(
                    &mut write_font_system(),
                    GlyphonAction::Motion(glyphon::cosmic_text::Motion::BufferEnd),
                );
            } else {
                // Original logic for other keys
                let mut all_actions = Vec::new();
                {
                    let mut state = state_for_handler.write();
                    for key_event in input.keyboard_events.iter().cloned() {
                        if let Some(actions) = state.map_key_event_to_action(
                            key_event,
                            input.key_modifiers,
                            input.clipboard,
                        ) {
                            all_actions.extend(actions);
                        }
                    }
                }

                if !all_actions.is_empty() {
                    let mut state = state_for_handler.write();
                    for action in all_actions {
                        handle_action(&mut state, action, on_change.clone());
                    }
                }
            }

            // Block all keyboard events to prevent propagation
            input.keyboard_events.clear();

            // Handle IME events
            let ime_events: Vec<_> = input.ime_events.drain(..).collect();
            for event in ime_events {
                let mut state = state_for_handler.write();
                match event {
                    winit::event::Ime::Commit(text) => {
                        // Clear preedit string if it exists
                        if let Some(preedit_text) = state.preedit_string.take() {
                            for _ in 0..preedit_text.chars().count() {
                                handle_action(
                                    &mut state,
                                    GlyphonAction::Backspace,
                                    on_change.clone(),
                                );
                            }
                        }
                        // Insert the committed text
                        for c in text.chars() {
                            handle_action(&mut state, GlyphonAction::Insert(c), on_change.clone());
                        }
                    }
                    winit::event::Ime::Preedit(text, _cursor_offset) => {
                        // Remove the old preedit text if it exists
                        if let Some(old_preedit) = state.preedit_string.take() {
                            for _ in 0..old_preedit.chars().count() {
                                handle_action(
                                    &mut state,
                                    GlyphonAction::Backspace,
                                    on_change.clone(),
                                );
                            }
                        }
                        // Insert the new preedit text
                        for c in text.chars() {
                            handle_action(&mut state, GlyphonAction::Insert(c), on_change.clone());
                        }
                        state.preedit_string = Some(text.to_string());
                    }
                    _ => {}
                }
            }

            // Request IME window
            input.requests.ime_request = Some(ImeRequest::new(size.into()));
        }

        apply_text_editor_accessibility(&mut input, &args_for_handler, &state_for_handler);
    }));
}

fn handle_action(
    state: &mut TextEditorState,
    action: GlyphonAction,
    on_change: Arc<dyn Fn(String) -> String + Send + Sync>,
) {
    // Clone a temporary editor and apply action, waiting for on_change to confirm
    let mut new_editor = state.editor().clone();

    // Make sure new editor own a isolated buffer
    let mut new_buffer = None;
    match new_editor.buffer_ref_mut() {
        glyphon::cosmic_text::BufferRef::Owned(_) => { /* Already owned */ }
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

    new_editor.action(&mut write_font_system(), action);
    let content_after_action = get_editor_content(&new_editor);

    state.editor_mut().action(&mut write_font_system(), action);
    let new_content = on_change(content_after_action);

    // Update editor content
    state.editor_mut().set_text_reactive(
        &new_content,
        &mut write_font_system(),
        &glyphon::Attrs::new().family(glyphon::fontdb::Family::SansSerif),
    );
}

/// Create surface arguments based on editor configuration and state
fn create_surface_args(
    args: &TextEditorArgs,
    state: &Arc<RwLock<TextEditorState>>,
) -> crate::surface::SurfaceArgs {
    let style = if args.border_width.to_pixels_f32() > 0.0 {
        crate::surface::SurfaceStyle::FilledOutlined {
            fill_color: determine_background_color(args, state),
            border_color: determine_border_color(args, state).unwrap(),
            border_width: args.border_width,
        }
    } else {
        crate::surface::SurfaceStyle::Filled {
            color: determine_background_color(args, state),
        }
    };

    SurfaceArgsBuilder::default()
        .style(style)
        .shape(args.shape)
        .padding(args.padding)
        .width(args.width)
        .height(args.height)
        .build()
        .unwrap()
}

/// Determine background color based on focus state
fn determine_background_color(
    args: &TextEditorArgs,
    state: &Arc<RwLock<TextEditorState>>,
) -> Color {
    if state.read().focus_handler().is_focused() {
        args.focus_background_color
            .or(args.background_color)
            .unwrap_or(Color::WHITE) // Default white when focused
    } else {
        args.background_color
            .unwrap_or(Color::new(0.95, 0.95, 0.95, 1.0)) // Default light gray when not focused
    }
}

/// Determine border color based on focus state
fn determine_border_color(
    args: &TextEditorArgs,
    state: &Arc<RwLock<TextEditorState>>,
) -> Option<Color> {
    if state.read().focus_handler().is_focused() {
        args.focus_border_color
            .or(args.border_color)
            .or(Some(Color::new(0.0, 0.5, 1.0, 1.0))) // Default blue focus border
    } else {
        args.border_color.or(Some(Color::new(0.7, 0.7, 0.7, 1.0))) // Default gray border
    }
}

/// Convenience constructors for common use cases
impl TextEditorArgs {
    /// Creates a simple text editor with default styling.
    ///
    /// - Minimum width: 120dp
    /// - Background: white
    /// - Border: 1px gray, rounded rectangle
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple();
    /// ```
    pub fn simple() -> Self {
        TextEditorArgsBuilder::default()
            .min_width(Some(Dp(120.0)))
            .background_color(Some(Color::WHITE))
            .border_width(Dp(1.0))
            .border_color(Some(Color::new(0.7, 0.7, 0.7, 1.0)))
            .shape(Shape::RoundedRectangle {
                top_left: Dp(0.0),
                top_right: Dp(0.0),
                bottom_right: Dp(0.0),
                bottom_left: Dp(0.0),
                g2_k_value: 3.0,
            })
            .build()
            .unwrap()
    }

    /// Creates a text editor with an emphasized border for better visibility.
    ///
    /// - Border: 2px, blue focus border
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::outlined();
    /// ```
    pub fn outlined() -> Self {
        Self::simple()
            .with_border_width(Dp(1.0))
            .with_focus_border_color(Color::new(0.0, 0.5, 1.0, 1.0))
    }

    /// Creates a text editor with no border (minimal style).
    ///
    /// - Border: 0px, square corners
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::minimal();
    /// ```
    pub fn minimal() -> Self {
        TextEditorArgsBuilder::default()
            .min_width(Some(Dp(120.0)))
            .background_color(Some(Color::WHITE))
            .shape(Shape::RoundedRectangle {
                top_left: Dp(0.0),
                top_right: Dp(0.0),
                bottom_right: Dp(0.0),
                bottom_left: Dp(0.0),
                g2_k_value: 3.0,
            })
            .build()
            .unwrap()
    }
}

/// Builder methods for fluent API
impl TextEditorArgs {
    /// Sets the width constraint for the editor.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::{DimensionValue, Px};
    /// let args = TextEditorArgs::simple().with_width(DimensionValue::Fixed(Px(200)));
    /// ```
    pub fn with_width(mut self, width: DimensionValue) -> Self {
        self.width = width;
        self
    }

    /// Sets the height constraint for the editor.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::{DimensionValue, Px};
    /// let args = TextEditorArgs::simple().with_height(DimensionValue::Fixed(Px(100)));
    /// ```
    pub fn with_height(mut self, height: DimensionValue) -> Self {
        self.height = height;
        self
    }

    /// Sets the minimum width in Dp.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::Dp;
    /// let args = TextEditorArgs::simple().with_min_width(Dp(80.0));
    /// ```
    pub fn with_min_width(mut self, min_width: Dp) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// Sets the minimum height in Dp.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::Dp;
    /// let args = TextEditorArgs::simple().with_min_height(Dp(40.0));
    /// ```
    pub fn with_min_height(mut self, min_height: Dp) -> Self {
        self.min_height = Some(min_height);
        self
    }

    /// Sets the background color.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::Color;
    /// let args = TextEditorArgs::simple().with_background_color(Color::WHITE);
    /// ```
    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Sets the border width in pixels.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui::Dp;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    ///
    /// let args = TextEditorArgs::simple().with_border_width(Dp(1.0));
    /// ```
    pub fn with_border_width(mut self, width: Dp) -> Self {
        self.border_width = width;
        self
    }

    /// Sets the border color.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::Color;
    /// let args = TextEditorArgs::simple().with_border_color(Color::BLACK);
    /// ```
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Sets the shape of the editor container.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui::Dp;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui_basic_components::shape_def::Shape;
    /// let args = TextEditorArgs::simple().with_shape(Shape::RoundedRectangle { top_left: Dp(8.0), top_right: Dp(8.0), bottom_right: Dp(8.0), bottom_left: Dp(8.0), g2_k_value: 3.0 });
    /// ```
    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the inner padding in Dp.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::Dp;
    /// let args = TextEditorArgs::simple().with_padding(Dp(12.0));
    /// ```
    pub fn with_padding(mut self, padding: Dp) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the border color when focused.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::Color;
    /// let args = TextEditorArgs::simple().with_focus_border_color(Color::new(0.0, 0.5, 1.0, 1.0));
    /// ```
    pub fn with_focus_border_color(mut self, color: Color) -> Self {
        self.focus_border_color = Some(color);
        self
    }

    /// Sets the background color when focused.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::Color;
    /// let args = TextEditorArgs::simple().with_focus_background_color(Color::WHITE);
    /// ```
    pub fn with_focus_background_color(mut self, color: Color) -> Self {
        self.focus_background_color = Some(color);
        self
    }

    /// Sets the selection highlight color.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::Color;
    /// let args = TextEditorArgs::simple().with_selection_color(Color::new(0.5, 0.7, 1.0, 0.4));
    /// ```
    pub fn with_selection_color(mut self, color: Color) -> Self {
        self.selection_color = Some(color);
        self
    }
}

fn get_editor_content(editor: &glyphon::Editor) -> String {
    editor.with_buffer(|buffer| {
        buffer
            .lines
            .iter()
            .map(|line| line.text().to_string() + line.ending().as_str())
            .collect::<String>()
    })
}

fn apply_text_editor_accessibility(
    input: &mut tessera_ui::InputHandlerInput<'_>,
    args: &TextEditorArgs,
    state: &Arc<RwLock<TextEditorState>>,
) {
    let mut builder = input.accessibility().role(Role::MultilineTextInput);

    if let Some(label) = args.accessibility_label.as_ref() {
        builder = builder.label(label.clone());
    }

    if let Some(description) = args.accessibility_description.as_ref() {
        builder = builder.description(description.clone());
    }

    let current_text = {
        let guard = state.read();
        get_editor_content(guard.editor())
    };
    if !current_text.is_empty() {
        builder = builder.value(current_text);
    }

    builder.focusable().commit();
}
