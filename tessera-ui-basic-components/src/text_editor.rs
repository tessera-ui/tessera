//! Multi-line text editor component for the Tessera UI framework.
//!
//! This module provides a robust, customizable multi-line text editor designed for integration into Tessera-based applications.
//! It features a two-layer architecture: a surface layer for visual container and click area, and a core layer for text rendering and editing logic.
//!
//! # Features
//! - Unicode multi-line text editing
//! - Full cursor and selection management (mouse, keyboard, drag, double/triple click)
//! - IME/preedit support for CJK and complex input
//! - Customizable appearance (background, border, shape, padding, selection color)
//! - Focus management and event handling
//! - Scroll support via mouse wheel or keyboard
//!
//! # Usage
//! The editor state is managed externally via [`TextEditorState`] (typically wrapped in `Arc<RwLock<...>>`).
//! The [`text_editor`] component can be configured using [`TextEditorArgs`] for layout and appearance customization.
//!
//! Typical use cases include form inputs, code editors, chat boxes, and any scenario requiring rich text input within a Tessera UI application.
use std::sync::Arc;

use derive_builder::Builder;
use glyphon::{Action, Edit};
use parking_lot::RwLock;
use tessera_ui::{
    Color, CursorEventContent, DimensionValue, Dp, ImeRequest, Px, PxPosition, tessera, winit,
};

use crate::{
    pipelines::write_font_system,
    pos_misc::is_position_in_component,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    text_edit_core::{ClickType, text_edit_core},
};

/// State structure for the text editor, managing text content, cursor, selection, and editing logic.
///
/// This is a re-export of [`TextEditorState`] from the core text editing module.
/// It encapsulates all stateful aspects of the editor, including text buffer, cursor position, selection range,
/// focus handling, and IME/preedit support. The state should be wrapped in `Arc<RwLock<...>>` for safe sharing between UI and event handlers.
///
/// # Example
///
/// ```
/// use tessera_ui_basic_components::text_editor::TextEditorState;
/// use tessera_ui::Dp;
/// let state = TextEditorState::new(Dp(14.0), None);
/// ```
pub use crate::text_edit_core::TextEditorState;

/// Arguments for the `text_editor` component.
///
/// # Example
/// ```
/// use tessera_ui_basic_components::text_editor::{TextEditorArgs, TextEditorArgsBuilder, TextEditorState};
/// use tessera_ui::{Dp, DimensionValue, Px};
/// use std::sync::Arc;
/// use parking_lot::RwLock;
///
/// // Create a text editor with a fixed width and height.
/// let editor_args_fixed = TextEditorArgsBuilder::default()
///     .width(Some(DimensionValue::Fixed(Px(200)))) // pixels
///     .height(Some(DimensionValue::Fixed(Px(100)))) // pixels
///     .build()
///     .unwrap();
///
/// // Create a text editor that fills available width up to 500px, with a min width of 50px
/// let editor_args_fill_wrap = TextEditorArgsBuilder::default()
///     .width(Some(DimensionValue::Fill { min: Some(Px(50)), max: Some(Px(500)) })) // pixels
///     .height(Some(DimensionValue::Wrap { min: None, max: None }))
///     .build()
///     .unwrap();
///
/// // Create the editor state
/// let editor_state = Arc::new(RwLock::new(TextEditorState::new(Dp(10.0), None)));
///
/// // text_editor(editor_args_fixed, editor_state.clone());
/// // text_editor(editor_args_fill_wrap, editor_state.clone());
/// ```
#[derive(Debug, Default, Builder, Clone)]
#[builder(pattern = "owned")]
/// Arguments for configuring the [`text_editor`] component.
///
/// `TextEditorArgs` provides flexible options for layout, appearance, and interaction of the text editor.
/// All fields are optional and have sensible defaults. Use the builder pattern or convenience methods for construction.
///
/// # Fields
/// - `width`, `height`: Optional constraints for the editor's size (logical pixels or fill/wrap).
/// - `min_width`, `min_height`: Minimum size in density-independent pixels (Dp).
/// - `background_color`, `focus_background_color`: Editor background color (normal/focused).
/// - `border_width`, `border_color`, `focus_border_color`: Border styling (width and color, normal/focused).
/// - `shape`: Shape of the editor container (e.g., rounded rectangle).
/// - `padding`: Inner padding (Dp).
/// - `selection_color`: Highlight color for selected text.
///
/// # Example
/// ```
/// use tessera_ui_basic_components::text_editor::{TextEditorArgs, TextEditorArgsBuilder};
/// use tessera_ui::{Dp, DimensionValue, Px};
///
/// let args = TextEditorArgsBuilder::default()
///     .width(Some(DimensionValue::Fixed(Px(300))))
///     .height(Some(DimensionValue::Fill { min: Some(Px(50)), max: Some(Px(500)) }))
///     .background_color(Some(tessera_ui::Color::WHITE))
///     .padding(Dp(8.0))
///     .build()
///     .unwrap();
/// ```
pub struct TextEditorArgs {
    /// Optional width constraint for the text editor. Values are in logical pixels or fill/wrap.
    #[builder(default = "None")]
    pub width: Option<DimensionValue>,
    /// Optional height constraint for the text editor. Values are in logical pixels or fill/wrap.
    #[builder(default = "None")]
    pub height: Option<DimensionValue>,
    /// Minimum width in density-independent pixels. Defaults to 120dp if not specified.
    #[builder(default = "None")]
    pub min_width: Option<Dp>,
    /// Minimum height in density-independent pixels. Defaults to line height + padding if not specified.
    #[builder(default = "None")]
    pub min_height: Option<Dp>,
    /// Background color of the text editor (RGBA). Defaults to light gray.
    #[builder(default = "None")]
    pub background_color: Option<Color>,
    /// Border width in pixels. Defaults to 1.0.
    #[builder(default = "1.0")]
    pub border_width: f32,
    /// Border color (RGBA). Defaults to gray.
    #[builder(default = "None")]
    pub border_color: Option<Color>,
    /// The shape of the text editor container.
    #[builder(default = "                Shape::RoundedRectangle {
                    top_left: 4.0,
                    top_right: 4.0,
                    bottom_right: 4.0,
                    bottom_left: 4.0,
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
}

/// A text editor component with two-layer architecture:
/// - surface layer: provides visual container, minimum size, and click area
/// - Core layer: handles text rendering and editing logic
///
/// This design solves the issue where empty text editors had zero width and couldn't be clicked.
///
/// # Example
///
/// ```
/// use tessera_ui_basic_components::text_editor::{text_editor, TextEditorArgs, TextEditorArgsBuilder, TextEditorState};
/// use tessera_ui::{Dp, DimensionValue, Px};
/// use std::sync::Arc;
/// use parking_lot::RwLock;
///
/// let args = TextEditorArgsBuilder::default()
///     .width(Some(DimensionValue::Fixed(Px(300))))
///     .height(Some(DimensionValue::Fill { min: Some(Px(50)), max: Some(Px(500)) }))
///     .build()
///     .unwrap();
///
/// let state = Arc::new(RwLock::new(TextEditorState::new(Dp(12.0), None)));
/// // text_editor(args, state);
/// ```
/// Multi-line text editor component with full state management, cursor, selection, and IME support.
///
/// The `text_editor` component provides a robust, customizable multi-line text editing area.
/// It supports keyboard and mouse input, selection, cursor movement, IME/preedit, and scroll handling.
/// State is managed externally via [`TextEditorState`] (typically wrapped in `Arc<RwLock<...>>`).
///
/// # Features
/// - Multi-line text editing with Unicode support
/// - Full cursor and selection management (mouse, keyboard, drag, double/triple click)
/// - IME/preedit support for CJK and complex input
/// - Customizable appearance (background, border, shape, padding, selection color)
/// - Focus management and event handling
/// - Scroll via mouse wheel or keyboard
///
/// # Parameters
/// - `args`: Editor configuration, see [`TextEditorArgs`].
/// - `state`: Shared editor state, see [`TextEditorState`].
///
/// # Example
/// ```
/// use tessera_ui_basic_components::text_editor::{text_editor, TextEditorArgs, TextEditorArgsBuilder, TextEditorState};
/// use tessera_ui::{Dp, DimensionValue, Px};
/// use std::sync::Arc;
/// use parking_lot::RwLock;
///
/// let args = TextEditorArgsBuilder::default()
///     .width(Some(DimensionValue::Fixed(Px(300))))
///     .height(Some(DimensionValue::Fill { min: Some(Px(50)), max: Some(Px(500)) }))
///     .build()
///     .unwrap();
///
/// let state = Arc::new(RwLock::new(TextEditorState::new(Dp(12.0), None)));
/// text_editor(args, state);
/// ```
#[tessera]
pub fn text_editor(args: impl Into<TextEditorArgs>, state: Arc<RwLock<TextEditorState>>) {
    let editor_args: TextEditorArgs = args.into();

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

    let state_for_handler = state.clone();
    state_handler(Box::new(move |input| {
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
                    let padding_px: Px = editor_args.padding.into();
                    let border_width_px = Px(editor_args.border_width as i32); // Assuming border_width is integer pixels

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
                                    Action::Click {
                                        x: text_relative_pos.x.0,
                                        y: text_relative_pos.y.0,
                                    },
                                );
                            }
                            ClickType::Double => {
                                // Double click: select word
                                state_for_handler.write().editor_mut().action(
                                    &mut write_font_system(),
                                    Action::DoubleClick {
                                        x: text_relative_pos.x.0,
                                        y: text_relative_pos.y.0,
                                    },
                                );
                            }
                            ClickType::Triple => {
                                // Triple click: select line
                                state_for_handler.write().editor_mut().action(
                                    &mut write_font_system(),
                                    Action::TripleClick {
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
                let padding_px: Px = editor_args.padding.into();
                let border_width_px = Px(editor_args.border_width as i32);

                let text_relative_x_px = cursor_pos.x - padding_px - border_width_px;
                let text_relative_y_px = cursor_pos.y - padding_px - border_width_px;

                if text_relative_x_px >= Px(0) && text_relative_y_px >= Px(0) {
                    let current_pos_px = PxPosition::new(text_relative_x_px, text_relative_y_px);
                    let last_pos_px = state_for_handler.read().last_click_position();

                    if last_pos_px != Some(current_pos_px) {
                        // Extend selection by dragging
                        state_for_handler.write().editor_mut().action(
                            &mut write_font_system(),
                            Action::Drag {
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
                    let action = glyphon::Action::Scroll { pixels: scroll };
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
                    glyphon::Action::Motion(glyphon::cosmic_text::Motion::BufferEnd),
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
                        state.editor_mut().action(&mut write_font_system(), action);
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
                                state
                                    .editor_mut()
                                    .action(&mut write_font_system(), glyphon::Action::Backspace);
                            }
                        }
                        // Insert the committed text
                        for c in text.chars() {
                            state
                                .editor_mut()
                                .action(&mut write_font_system(), glyphon::Action::Insert(c));
                        }
                    }
                    winit::event::Ime::Preedit(text, _cursor_offset) => {
                        // Remove the old preedit text if it exists
                        if let Some(old_preedit) = state.preedit_string.take() {
                            for _ in 0..old_preedit.chars().count() {
                                state
                                    .editor_mut()
                                    .action(&mut write_font_system(), glyphon::Action::Backspace);
                            }
                        }
                        // Insert the new preedit text
                        for c in text.chars() {
                            state
                                .editor_mut()
                                .action(&mut write_font_system(), glyphon::Action::Insert(c));
                        }
                        state.preedit_string = Some(text.to_string());
                    }
                    _ => {}
                }
            }

            // Request IME window
            input.requests.ime_request = Some(ImeRequest::new(size.into()));
        }
    }));
}

/// Create surface arguments based on editor configuration and state
fn create_surface_args(
    args: &TextEditorArgs,
    state: &Arc<RwLock<TextEditorState>>,
) -> crate::surface::SurfaceArgs {
    let mut builder = SurfaceArgsBuilder::default();

    // Set width if available
    if let Some(width) = args.width {
        builder = builder.width(width);
    } else {
        // Use default with minimum
        builder = builder.width(DimensionValue::Wrap {
            min: args.min_width.map(|dp| dp.into()).or(Some(Px(120))), // Default minimum width 120px
            max: None,
        });
    }

    // Set height if available
    if let Some(height) = args.height {
        builder = builder.height(height);
    } else {
        // Use line height as basis with some padding
        let line_height_px = state.read().line_height();
        let padding_px: Px = args.padding.into();
        let min_height_px = args
            .min_height
            .map(|dp| dp.into())
            .unwrap_or(line_height_px + padding_px * 2 + Px(10)); // +10 for comfortable spacing
        builder = builder.height(DimensionValue::Wrap {
            min: Some(min_height_px),
            max: None,
        });
    }

    builder
        .color(determine_background_color(args, state))
        .border_width(determine_border_width(args, state))
        .border_color(determine_border_color(args, state))
        .shape(args.shape)
        .padding(args.padding)
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

/// Determine border width
fn determine_border_width(args: &TextEditorArgs, _state: &Arc<RwLock<TextEditorState>>) -> f32 {
    args.border_width
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
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple();
    /// ```
    pub fn simple() -> Self {
        TextEditorArgsBuilder::default()
            .min_width(Some(Dp(120.0)))
            .background_color(Some(Color::WHITE))
            .border_width(1.0)
            .border_color(Some(Color::new(0.7, 0.7, 0.7, 1.0)))
            .shape(Shape::RoundedRectangle {
                top_left: 4.0,
                top_right: 4.0,
                bottom_right: 4.0,
                bottom_left: 4.0,
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
            .with_border_width(2.0)
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
            .border_width(0.0)
            .shape(Shape::RoundedRectangle {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
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
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::{DimensionValue, Px};
    /// let args = TextEditorArgs::simple().with_width(DimensionValue::Fixed(Px(200)));
    /// ```
    pub fn with_width(mut self, width: DimensionValue) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the height constraint for the editor.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui::{DimensionValue, Px};
    /// let args = TextEditorArgs::simple().with_height(DimensionValue::Fixed(Px(100)));
    /// ```
    pub fn with_height(mut self, height: DimensionValue) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the minimum width in Dp.
    ///
    /// # Example
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
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_border_width(2.0);
    /// ```
    pub fn with_border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Sets the border color.
    ///
    /// # Example
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
    /// ```
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// use tessera_ui_basic_components::shape_def::Shape;
    /// let args = TextEditorArgs::simple().with_shape(Shape::RoundedRectangle { top_left: 8.0, top_right: 8.0, bottom_right: 8.0, bottom_left: 8.0, g2_k_value: 3.0 });
    /// ```
    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the inner padding in Dp.
    ///
    /// # Example
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
