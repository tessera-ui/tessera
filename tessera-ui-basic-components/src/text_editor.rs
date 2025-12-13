//! A multi-line text editor component.
//!
//! ## Usage
//!
//! Use for text input fields, forms, or any place that requires editable text.
use std::sync::Arc;

use derive_builder::Builder;
use glyphon::{Action as GlyphonAction, Edit};
use tessera_ui::{
    Color, CursorEventContent, DimensionValue, Dp, ImeRequest, Px, PxPosition, State,
    accesskit::Role, remember, tessera, use_context, winit,
};

use crate::{
    pipelines::text::pipeline::write_font_system,
    pos_misc::is_position_in_component,
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgsBuilder, surface},
    text_edit_core::{ClickType, text_edit_core},
    theme::MaterialColorScheme,
};

/// State structure for the text editor, managing text content, cursor,
/// selection, and editing logic.
pub use crate::text_edit_core::TextEditorController;

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
    /// Called when the text content changes. The closure receives the new text
    /// content and returns the updated content.
    #[builder(default = "Arc::new(|_| { String::new() })")]
    pub on_change: Arc<dyn Fn(String) -> String + Send + Sync>,
    /// Minimum width in density-independent pixels. Defaults to 120dp if not
    /// specified.
    #[builder(default = "None")]
    pub min_width: Option<Dp>,
    /// Minimum height in density-independent pixels. Defaults to line height +
    /// padding if not specified.
    #[builder(default = "None")]
    pub min_height: Option<Dp>,
    /// Background color of the text editor (RGBA). Defaults to light gray.
    #[builder(default = "Some(use_context::<MaterialColorScheme>().get().surface_variant)")]
    pub background_color: Option<Color>,
    /// Border width in Dp. Defaults to 1.0 Dp.
    #[builder(default = "Dp(1.0)")]
    pub border_width: Dp,
    /// Border color (RGBA). Defaults to gray.
    #[builder(default = "Some(use_context::<MaterialColorScheme>().get().outline_variant)")]
    pub border_color: Option<Color>,
    /// The shape of the text editor container.
    #[builder(default = "Shape::RoundedRectangle {
                            top_left: RoundedCorner::manual(Dp(4.0), 3.0),
                            top_right: RoundedCorner::manual(Dp(4.0), 3.0),
                            bottom_right: RoundedCorner::manual(Dp(4.0), 3.0),
                            bottom_left: RoundedCorner::manual(Dp(4.0), 3.0),
                        }")]
    pub shape: Shape,
    /// Padding inside the text editor. Defaults to 5.0 Dp.
    #[builder(default = "Dp(5.0)")]
    pub padding: Dp,
    /// Border color when focused (RGBA). Defaults to blue.
    #[builder(default = "Some(use_context::<MaterialColorScheme>().get().primary)")]
    pub focus_border_color: Option<Color>,
    /// Background color when focused (RGBA). Defaults to white.
    #[builder(default = "Some(use_context::<MaterialColorScheme>().get().surface)")]
    pub focus_background_color: Option<Color>,
    /// Color for text selection highlight (RGBA). Defaults to light blue with
    /// transparency.
    #[builder(
        default = "Some(use_context::<MaterialColorScheme>().get().primary.with_alpha(0.35))"
    )]
    pub selection_color: Option<Color>,
    /// Optional label announced by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_label: Option<String>,
    /// Optional description announced by assistive technologies.
    #[builder(default, setter(strip_option, into))]
    pub accessibility_description: Option<String>,
    /// Initial text content.
    #[builder(default, setter(strip_option, into))]
    pub initial_text: Option<String>,
    /// Font size in Dp. Defaults to 14.0.
    #[builder(default = "Dp(14.0)")]
    pub font_size: Dp,
    /// Line height in Dp. Defaults to None (1.2x font size).
    #[builder(default = "None")]
    pub line_height: Option<Dp>,
}

impl Default for TextEditorArgs {
    fn default() -> Self {
        TextEditorArgsBuilder::default()
            .build()
            .expect("builder construction failed")
    }
}

/// # text_editor
///
/// Renders a multi-line, editable text field.
///
/// # Usage
///
/// Create an interactive text editor for forms, note-taking, or other text
/// input scenarios.
///
/// # Parameters
///
/// - `args` — configures the editor's appearance and layout; see
///   [`TextEditorArgs`].
///
/// # Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::Dp;
/// use tessera_ui_basic_components::text_editor::{TextEditorArgsBuilder, text_editor};
///
/// text_editor(
///     TextEditorArgsBuilder::default()
///         .padding(Dp(8.0))
///         .initial_text("Hello World")
///         .build()
///         .unwrap(),
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn text_editor(args: impl Into<TextEditorArgs>) {
    let args: TextEditorArgs = args.into();
    let controller = remember(|| {
        let mut c = TextEditorController::new(args.font_size, args.line_height);
        if let Some(text) = &args.initial_text {
            c.editor_mut().set_text_reactive(
                text,
                &mut write_font_system(),
                &glyphon::Attrs::new().family(glyphon::fontdb::Family::SansSerif),
            );
        }
        c
    });

    text_editor_with_controller(args, controller);
}

/// # text_editor_with_controller
///
/// Renders a multi-line, editable text field with an external controller.
///
/// # Usage
///
/// Use this component when you need to control the text editor state
/// externally, for example to synchronize text with other components or to
/// programmatically modify the text content or selection.
///
/// # Parameters
///
/// - `args` — configures the editor's appearance and layout; see
///   [`TextEditorArgs`].
/// - `controller` — a `TextEditorController` to manage the editor's content,
///   cursor, and selection.
///
/// # Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::Dp;
/// use tessera_ui::remember;
/// use tessera_ui_basic_components::{
///     text::write_font_system,
///     text_editor::{TextEditorArgsBuilder, TextEditorController, text_editor_with_controller},
/// };
///
/// let controller = remember(|| TextEditorController::new(Dp(14.0), None));
/// controller.with_mut(|c| {
///     c.editor_mut().set_text_reactive(
///         "Initial text",
///         &mut write_font_system(),
///         &glyphon::Attrs::new().family(glyphon::fontdb::Family::SansSerif),
///     );
/// });
///
/// text_editor_with_controller(
///     TextEditorArgsBuilder::default()
///         .padding(Dp(8.0))
///         .build()
///         .unwrap(),
///     controller,
/// );
/// # }
/// # component();
/// ```
#[tessera]
pub fn text_editor_with_controller(
    args: impl Into<TextEditorArgs>,
    controller: State<TextEditorController>,
) {
    let editor_args: TextEditorArgs = args.into();
    let on_change = editor_args.on_change.clone();

    // Update the state with the selection color from args
    if let Some(selection_color) = editor_args.selection_color {
        controller.with_mut(|c| c.set_selection_color(selection_color));
    }

    // surface layer - provides visual container and minimum size guarantee
    {
        let args_for_surface = editor_args.clone();
        surface(
            create_surface_args(&args_for_surface, &controller),
            move || {
                // Core layer - handles text rendering and editing logic
                text_edit_core(controller);
            },
        );
    }

    // Event handling at the outermost layer - can access full surface area

    let args_for_handler = editor_args.clone();
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
                if !controller.with(|s| s.focus_handler().is_focused()) {
                    controller.with_mut(|s| {
                        s.focus_handler_mut().request_focus();
                    });
                }

                // Handle cursor positioning for clicks
                if let Some(cursor_pos) = cursor_pos_option {
                    // Calculate the relative position within the text area
                    let padding_px: Px = args_for_handler.padding.into();
                    let border_width_px = Px(args_for_handler.border_width.to_pixels_u32() as i32); // Assuming border_width is integer pixels

                    let text_relative_x_px = cursor_pos.x - padding_px - border_width_px;
                    let text_relative_y_px = cursor_pos.y - padding_px - border_width_px;

                    // Only process if the click is within the text area (non-negative relative
                    // coords)
                    if text_relative_x_px >= Px(0) && text_relative_y_px >= Px(0) {
                        let text_relative_pos =
                            PxPosition::new(text_relative_x_px, text_relative_y_px);
                        // Determine click type and handle accordingly
                        let click_type = controller.with_mut(|s| {
                            s.handle_click(text_relative_pos, click_events[0].timestamp)
                        });

                        match click_type {
                            ClickType::Single => {
                                // Single click: position cursor
                                controller.with_mut(|s| {
                                    s.editor_mut().action(
                                        &mut write_font_system(),
                                        GlyphonAction::Click {
                                            x: text_relative_pos.x.0,
                                            y: text_relative_pos.y.0,
                                        },
                                    );
                                });
                            }
                            ClickType::Double => {
                                // Double click: select word
                                controller.with_mut(|s| {
                                    s.editor_mut().action(
                                        &mut write_font_system(),
                                        GlyphonAction::DoubleClick {
                                            x: text_relative_pos.x.0,
                                            y: text_relative_pos.y.0,
                                        },
                                    );
                                });
                            }
                            ClickType::Triple => {
                                // Triple click: select line
                                controller.with_mut(|s| {
                                    s.editor_mut().action(
                                        &mut write_font_system(),
                                        GlyphonAction::TripleClick {
                                            x: text_relative_pos.x.0,
                                            y: text_relative_pos.y.0,
                                        },
                                    );
                                });
                            }
                        }

                        // Start potential drag operation
                        controller.with_mut(|s| s.start_drag());
                    }
                }
            }

            // Handle drag events (mouse move while dragging)
            // This happens every frame when cursor position changes during drag
            if controller.with(|s| s.is_dragging())
                && let Some(cursor_pos) = cursor_pos_option
            {
                let padding_px: Px = args_for_handler.padding.into();
                let border_width_px = Px(args_for_handler.border_width.to_pixels_u32() as i32);

                let text_relative_x_px = cursor_pos.x - padding_px - border_width_px;
                let text_relative_y_px = cursor_pos.y - padding_px - border_width_px;

                if text_relative_x_px >= Px(0) && text_relative_y_px >= Px(0) {
                    let current_pos_px = PxPosition::new(text_relative_x_px, text_relative_y_px);
                    let last_pos_px = controller.with(|s| s.last_click_position());

                    if last_pos_px != Some(current_pos_px) {
                        // Extend selection by dragging
                        controller.with_mut(|s| {
                            s.editor_mut().action(
                                &mut write_font_system(),
                                GlyphonAction::Drag {
                                    x: current_pos_px.x.0,
                                    y: current_pos_px.y.0,
                                },
                            );
                        });

                        // Update last position to current position
                        controller.with_mut(|s| {
                            s.update_last_click_position(current_pos_px);
                        });
                    }
                }
            }

            // Handle mouse release events (end drag)
            if !release_events.is_empty() {
                controller.with_mut(|s| s.stop_drag());
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
            if controller.with(|s| s.focus_handler().is_focused()) {
                for scroll_event in scroll_events {
                    // Convert scroll delta to lines
                    let scroll = -scroll_event.delta_y;

                    // Scroll up for positive, down for negative
                    let action = GlyphonAction::Scroll { pixels: scroll };
                    controller.with_mut(|s| {
                        s.editor_mut().action(&mut write_font_system(), action);
                    });
                }
            }

            // Only block cursor events when focused to prevent propagation
            if controller.with(|s| s.focus_handler().is_focused()) {
                input.cursor_events.clear();
            }
        }

        // Handle keyboard events (only when focused)
        if controller.with(|s| s.focus_handler().is_focused()) {
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
                controller.with_mut(|s| {
                    let editor = s.editor_mut();
                    // Set cursor to the beginning of the document
                    editor.set_cursor(glyphon::Cursor::new(0, 0));
                    // Set selection to start from the beginning
                    editor.set_selection(glyphon::cosmic_text::Selection::Normal(
                        glyphon::Cursor::new(0, 0),
                    ));
                    // Move cursor to the end, which extends the selection (use BufferEnd for full
                    // document)
                    editor.action(
                        &mut write_font_system(),
                        GlyphonAction::Motion(glyphon::cosmic_text::Motion::BufferEnd),
                    );
                });
            } else {
                // Original logic for other keys
                let mut all_actions = Vec::new();
                controller.with_mut(|s| {
                    for key_event in input.keyboard_events.iter().cloned() {
                        if let Some(actions) = s.map_key_event_to_action(
                            key_event,
                            input.key_modifiers,
                            input.clipboard,
                        ) {
                            all_actions.extend(actions);
                        }
                    }
                });

                if !all_actions.is_empty() {
                    for action in all_actions {
                        handle_action(&controller, action, on_change.clone());
                    }
                }
            }

            // Block all keyboard events to prevent propagation
            input.keyboard_events.clear();

            // Handle IME events
            let ime_events: Vec<_> = input.ime_events.drain(..).collect();
            for event in ime_events {
                match event {
                    winit::event::Ime::Commit(text) => {
                        // Clear preedit string if it exists
                        let preedit_text = controller.with_mut(|s| s.preedit_string.take());

                        if let Some(preedit_text) = preedit_text {
                            for _ in 0..preedit_text.chars().count() {
                                handle_action(
                                    &controller,
                                    GlyphonAction::Backspace,
                                    on_change.clone(),
                                );
                            }
                        }
                        // Insert the committed text
                        for c in text.chars() {
                            handle_action(&controller, GlyphonAction::Insert(c), on_change.clone());
                        }
                    }
                    winit::event::Ime::Preedit(text, _cursor_offset) => {
                        // Remove the old preedit text if it exists
                        let old_preedit = controller.with_mut(|s| s.preedit_string.take());

                        if let Some(old_preedit) = old_preedit {
                            for _ in 0..old_preedit.chars().count() {
                                handle_action(
                                    &controller,
                                    GlyphonAction::Backspace,
                                    on_change.clone(),
                                );
                            }
                        }
                        // Insert the new preedit text
                        for c in text.chars() {
                            handle_action(&controller, GlyphonAction::Insert(c), on_change.clone());
                        }
                        controller.with_mut(|c| c.preedit_string = Some(text.to_string()));
                    }
                    _ => {}
                }
            }

            // Request IME window
            input.requests.ime_request = Some(ImeRequest::new(size.into()));
        }

        apply_text_editor_accessibility(&mut input, &args_for_handler, &controller);
    }));
}

fn handle_action(
    state: &State<TextEditorController>,
    action: GlyphonAction,
    on_change: Arc<dyn Fn(String) -> String + Send + Sync>,
) {
    // Clone a temporary editor and apply action, waiting for on_change to confirm
    let mut new_editor = state.with(|c| c.editor().clone());

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

    state.with_mut(|c| c.editor_mut().action(&mut write_font_system(), action));
    let new_content = on_change(content_after_action);

    // Update editor content
    state.with_mut(|c| {
        c.editor_mut().set_text_reactive(
            &new_content,
            &mut write_font_system(),
            &glyphon::Attrs::new().family(glyphon::fontdb::Family::SansSerif),
        )
    });
}

/// Create surface arguments based on editor configuration and state
fn create_surface_args(
    args: &TextEditorArgs,
    state: &State<TextEditorController>,
) -> crate::surface::SurfaceArgs {
    let style = if args.border_width.to_pixels_f32() > 0.0 {
        crate::surface::SurfaceStyle::FilledOutlined {
            fill_color: determine_background_color(args, state),
            border_color: determine_border_color(args, state)
                .expect("Border color should exist when border width is positive"),
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
        .expect("builder construction failed")
}

/// Determine background color based on focus state
fn determine_background_color(args: &TextEditorArgs, state: &State<TextEditorController>) -> Color {
    if state.with(|c| c.focus_handler().is_focused()) {
        args.focus_background_color
            .or(args.background_color)
            .unwrap_or(use_context::<MaterialColorScheme>().get().surface)
    } else {
        args.background_color
            .unwrap_or(use_context::<MaterialColorScheme>().get().surface_variant)
    }
}

/// Determine border color based on focus state
fn determine_border_color(
    args: &TextEditorArgs,
    state: &State<TextEditorController>,
) -> Option<Color> {
    if state.with(|c| c.focus_handler().is_focused()) {
        args.focus_border_color
            .or(args.border_color)
            .or(Some(use_context::<MaterialColorScheme>().get().primary))
    } else {
        args.border_color.or(Some(
            use_context::<MaterialColorScheme>().get().outline_variant,
        ))
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple();
    /// # }
    /// # component();
    /// ```
    pub fn simple() -> Self {
        TextEditorArgsBuilder::default()
            .min_width(Some(Dp(120.0)))
            .background_color(Some(
                use_context::<MaterialColorScheme>().get().surface_variant,
            ))
            .border_width(Dp(1.0))
            .border_color(Some(
                use_context::<MaterialColorScheme>().get().outline_variant,
            ))
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                top_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
            })
            .build()
            .expect("builder construction failed")
    }

    /// Creates a text editor with an emphasized border for better visibility.
    ///
    /// - Border: 2px, blue focus border
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::outlined();
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::minimal();
    /// # }
    /// # component();
    /// ```
    pub fn minimal() -> Self {
        TextEditorArgsBuilder::default()
            .min_width(Some(Dp(120.0)))
            .background_color(Some(Color::WHITE))
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                top_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
            })
            .build()
            .expect("builder construction failed")
    }
}

/// Builder methods for fluent API
impl TextEditorArgs {
    /// Sets the width constraint for the editor.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::{DimensionValue, Px};
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_width(DimensionValue::Fixed(Px(200)));
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::{DimensionValue, Px};
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_height(DimensionValue::Fixed(Px(100)));
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::Dp;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_min_width(Dp(80.0));
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::Dp;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_min_height(Dp(40.0));
    /// # }
    /// # component();
    /// ```
    pub fn with_min_height(mut self, min_height: Dp) -> Self {
        self.min_height = Some(min_height);
        self
    }

    /// Sets the background color.
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::Color;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_background_color(Color::WHITE);
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::Dp;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    ///
    /// let args = TextEditorArgs::simple().with_border_width(Dp(1.0));
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::Color;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_border_color(Color::BLACK);
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::Dp;
    /// use tessera_ui_basic_components::shape_def::{RoundedCorner, Shape};
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_shape(Shape::RoundedRectangle {
    ///     top_left: RoundedCorner::manual(Dp(8.0), 3.0),
    ///     top_right: RoundedCorner::manual(Dp(8.0), 3.0),
    ///     bottom_right: RoundedCorner::manual(Dp(8.0), 3.0),
    ///     bottom_left: RoundedCorner::manual(Dp(8.0), 3.0),
    /// });
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::Dp;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_padding(Dp(12.0));
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::Color;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_focus_border_color(Color::new(0.0, 0.5, 1.0, 1.0));
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::Color;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_focus_background_color(Color::WHITE);
    /// # }
    /// # component();
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
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_ui::Color;
    /// use tessera_ui_basic_components::text_editor::TextEditorArgs;
    /// let args = TextEditorArgs::simple().with_selection_color(Color::new(0.5, 0.7, 1.0, 0.4));
    /// # }
    /// # component();
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
    state: &State<TextEditorController>,
) {
    let mut builder = input.accessibility().role(Role::MultilineTextInput);

    if let Some(label) = args.accessibility_label.as_ref() {
        builder = builder.label(label.clone());
    }

    if let Some(description) = args.accessibility_description.as_ref() {
        builder = builder.description(description.clone());
    }

    let current_text = state.with(|c| get_editor_content(c.editor()));
    if !current_text.is_empty() {
        builder = builder.value(current_text);
    }

    builder.focusable().commit();
}
