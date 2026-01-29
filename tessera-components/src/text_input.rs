//! Core text input component without Material decoration.
//!
//! ## Usage
//!
//! Embed as a bare text input surface when you need to build custom styling.
use std::sync::Arc;

use derive_setters::Setters;
use glyphon::{Action as GlyphonAction, Edit};
use tessera_ui::{
    Color, CursorEventContent, Dp, ImeRequest, Modifier, Px, PxPosition, State, accesskit::Role,
    remember, tessera, use_context, winit,
};

use crate::{
    modifier::ModifierExt,
    pipelines::text::pipeline::write_font_system,
    pos_misc::is_position_in_component,
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgs, surface},
    text_edit_core::{ClickType, text_edit_core},
    theme::{MaterialTheme, TextSelectionColors},
};

/// State structure for the text input, managing text content, cursor,
/// selection, and editing logic.
pub use crate::text_edit_core::{DisplayTransform, TextEditorController as TextInputController};

/// Arguments for configuring the [`text_input`] component.
#[derive(Clone, Setters)]
pub struct TextInputArgs {
    /// Whether the editor is enabled for user input.
    pub enabled: bool,
    /// Whether the editor is read-only.
    pub read_only: bool,
    /// Optional modifier chain applied to the editor container.
    pub modifier: Modifier,
    /// Called when the text content changes. The closure receives the new text
    /// content and returns the updated content.
    #[setters(skip)]
    pub on_change: Arc<dyn Fn(String) -> String + Send + Sync>,
    /// Minimum width in density-independent pixels. Defaults to 120dp if not
    /// specified.
    #[setters(strip_option)]
    pub min_width: Option<Dp>,
    /// Minimum height in density-independent pixels. Defaults to line height +
    /// padding if not specified.
    #[setters(strip_option)]
    pub min_height: Option<Dp>,
    /// Background color of the text input (RGBA). Defaults to light gray.
    #[setters(strip_option)]
    pub background_color: Option<Color>,
    /// Border width in Dp. Defaults to 1.0 Dp.
    pub border_width: Dp,
    /// Border color (RGBA). Defaults to gray.
    #[setters(strip_option)]
    pub border_color: Option<Color>,
    /// The shape of the text input container.
    pub shape: Shape,
    /// Padding inside the text input. Defaults to 5.0 Dp.
    pub padding: Dp,
    /// Border color when focused (RGBA). Defaults to blue.
    #[setters(strip_option)]
    pub focus_border_color: Option<Color>,
    /// Border width when focused. Defaults to the unfocused border width.
    #[setters(strip_option)]
    pub focus_border_width: Option<Dp>,
    /// Background color when focused (RGBA). Defaults to white.
    #[setters(strip_option)]
    pub focus_background_color: Option<Color>,
    /// Color for text selection highlight (RGBA). Defaults to light blue with
    /// transparency.
    #[setters(strip_option)]
    pub selection_color: Option<Color>,
    /// Color of the text content. Defaults to the theme on-surface color.
    #[setters(strip_option)]
    pub text_color: Option<Color>,
    /// Color of the text cursor. Defaults to the theme primary color.
    #[setters(strip_option)]
    pub cursor_color: Option<Color>,
    /// Optional label announced by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_label: Option<String>,
    /// Optional description announced by assistive technologies.
    #[setters(strip_option, into)]
    pub accessibility_description: Option<String>,
    /// Initial text content.
    #[setters(strip_option, into)]
    pub initial_text: Option<String>,
    /// Font size in Dp. Defaults to 14.0.
    pub font_size: Dp,
    /// Line height in Dp. Defaults to None (1.2x font size).
    #[setters(strip_option)]
    pub line_height: Option<Dp>,
    /// Optional transform applied to text changes before on_change.
    #[setters(skip)]
    pub input_transform: Option<Arc<dyn Fn(String) -> String + Send + Sync>>,
    /// Optional transform applied only for display.
    #[setters(skip)]
    pub display_transform: Option<DisplayTransform>,
}

impl TextInputArgs {
    /// Set the text change handler.
    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: Fn(String) -> String + Send + Sync + 'static,
    {
        self.on_change = Arc::new(on_change);
        self
    }

    /// Set the text change handler using a shared callback.
    pub fn on_change_shared(
        mut self,
        on_change: Arc<dyn Fn(String) -> String + Send + Sync>,
    ) -> Self {
        self.on_change = on_change;
        self
    }

    /// Set an input transform applied before the change handler.
    pub fn input_transform<F>(mut self, transform: F) -> Self
    where
        F: Fn(String) -> String + Send + Sync + 'static,
    {
        self.input_transform = Some(Arc::new(transform));
        self
    }

    /// Set an input transform using a shared callback.
    pub fn input_transform_shared(
        mut self,
        transform: Arc<dyn Fn(String) -> String + Send + Sync>,
    ) -> Self {
        self.input_transform = Some(transform);
        self
    }

    /// Set a display-only transform applied at render time.
    pub fn display_transform<F>(mut self, transform: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.display_transform = Some(Arc::new(transform));
        self
    }

    /// Set a display-only transform using a shared callback.
    pub fn display_transform_shared(mut self, transform: DisplayTransform) -> Self {
        self.display_transform = Some(transform);
        self
    }
}

impl Default for TextInputArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        Self {
            enabled: true,
            read_only: false,
            modifier: Modifier::new(),
            on_change: Arc::new(|_| String::new()),
            min_width: None,
            min_height: None,
            background_color: Some(scheme.surface_variant),
            border_width: Dp(1.0),
            border_color: Some(scheme.outline_variant),
            shape: Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(4.0), 3.0),
                top_right: RoundedCorner::manual(Dp(4.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(4.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(4.0), 3.0),
            },
            padding: Dp(5.0),
            focus_border_color: Some(scheme.primary),
            focus_border_width: None,
            focus_background_color: Some(scheme.surface),
            selection_color: Some(TextSelectionColors::from_scheme(&scheme).background),
            text_color: Some(scheme.on_surface),
            cursor_color: Some(scheme.primary),
            accessibility_label: None,
            accessibility_description: None,
            initial_text: None,
            font_size: Dp(14.0),
            line_height: None,
            input_transform: None,
            display_transform: None,
        }
    }
}

/// # text_input
///
/// Renders a multi-line, editable text field.
///
/// ## Usage
///
/// Create an interactive text input for forms, note-taking, or other text
/// input scenarios.
///
/// ## Parameters
///
/// - `args` — configures the editor's appearance and layout; see
///   [`TextInputArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::text_input::{TextInputArgs, text_input};
/// use tessera_ui::Dp;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme(|| MaterialTheme::default(), || {
/// text_input(
///     TextInputArgs::default()
///         .padding(Dp(8.0))
///         .initial_text("Hello World"),
/// );
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn text_input(args: impl Into<TextInputArgs>) {
    let args: TextInputArgs = args.into();
    let controller = remember(|| {
        let mut c = TextInputController::new(args.font_size, args.line_height);
        if let Some(text) = &args.initial_text {
            c.set_text(text);
        }
        c
    });

    text_input_with_controller(args, controller);
}

/// # text_input_with_controller
///
/// Renders a multi-line, editable text field with an external controller.
///
/// ## Usage
///
/// Use this component when you need to control the text input state
/// externally, for example to synchronize text with other components or to
/// programmatically modify the text content or selection.
///
/// ## Parameters
///
/// - `args` — configures the editor's appearance and layout; see
///   [`TextInputArgs`].
/// - `controller` — a `TextInputController` to manage the editor's content,
///   cursor, and selection.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::{
///     text::write_font_system,
///     text_input::{TextInputArgs, TextInputController, text_input_with_controller},
/// };
/// use tessera_ui::Dp;
/// use tessera_ui::remember;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme(|| MaterialTheme::default(), || {
/// let controller = remember(|| TextInputController::new(Dp(14.0), None));
/// controller.with_mut(|c| c.set_text("Initial text"));
///
/// text_input_with_controller(TextInputArgs::default().padding(Dp(8.0)), controller);
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn text_input_with_controller(
    args: impl Into<TextInputArgs>,
    controller: State<TextInputController>,
) {
    let editor_args: TextInputArgs = args.into();
    if !editor_args.enabled {
        controller.with_mut(|c| c.focus_handler_mut().unfocus());
    }
    let on_change = editor_args.on_change.clone();
    let input_transform = editor_args.input_transform.clone();

    sync_text_input_controller(&controller, &editor_args);

    // surface layer - provides visual container and minimum size guarantee
    {
        let surface_args = editor_args.clone();
        surface(create_surface_args(&surface_args, &controller), move || {
            // Core layer - handles text rendering and editing logic
            let padding = surface_args.padding;
            Modifier::new().padding_all(padding).run(move || {
                text_edit_core(controller);
            });
        });
    }

    // Event handling at the outermost layer - can access full surface area

    let handler_args = editor_args.clone();
    input_handler(move |mut input| {
        handle_text_input(
            &mut input,
            &handler_args,
            &controller,
            &on_change,
            &input_transform,
        );
    });
}

#[tessera]
pub(crate) fn text_input_core_with_controller(
    args: impl Into<TextInputArgs>,
    controller: State<TextInputController>,
) {
    let editor_args: TextInputArgs = args.into();
    if !editor_args.enabled {
        controller.with_mut(|c| c.focus_handler_mut().unfocus());
    }
    let on_change = editor_args.on_change.clone();
    let input_transform = editor_args.input_transform.clone();
    sync_text_input_controller(&controller, &editor_args);

    let handler_args = editor_args.clone();
    let modifier = editor_args.modifier;
    let padding = editor_args.padding;
    modifier.run(move || {
        Modifier::new().padding_all(padding).run(move || {
            text_edit_core(controller);
        });
    });

    input_handler(move |mut input| {
        handle_text_input(
            &mut input,
            &handler_args,
            &controller,
            &on_change,
            &input_transform,
        );
    });
}

fn sync_text_input_controller(controller: &State<TextInputController>, args: &TextInputArgs) {
    if let Some(selection_color) = args.selection_color {
        controller.with_mut(|c| c.set_selection_color(selection_color));
    }
    if let Some(text_color) = args.text_color {
        controller.with_mut(|c| c.set_text_color(text_color));
    }
    if let Some(cursor_color) = args.cursor_color {
        controller.with_mut(|c| c.set_cursor_color(cursor_color));
    }
    controller.with_mut(|c| c.set_display_transform(args.display_transform.clone()));
}

fn handle_text_input(
    input: &mut tessera_ui::InputHandlerInput<'_>,
    args: &TextInputArgs,
    controller: &State<TextInputController>,
    on_change: &Arc<dyn Fn(String) -> String + Send + Sync>,
    input_transform: &Option<Arc<dyn Fn(String) -> String + Send + Sync>>,
) {
    if !args.enabled {
        apply_text_input_accessibility(input, args, controller);
        return;
    }
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
                let padding_px: Px = args.padding.into();
                let border_width_px = Px(args.border_width.to_pixels_u32() as i32); // Assuming border_width is integer pixels

                let text_relative_x_px = cursor_pos.x - padding_px - border_width_px;
                let text_relative_y_px = cursor_pos.y - padding_px - border_width_px;

                // Only process if the click is within the text area (non-negative relative
                // coords)
                if text_relative_x_px >= Px(0) && text_relative_y_px >= Px(0) {
                    let text_relative_pos = PxPosition::new(text_relative_x_px, text_relative_y_px);
                    // Determine click type and handle accordingly
                    let click_type = controller
                        .with_mut(|s| s.handle_click(text_relative_pos, click_events[0].timestamp));

                    match click_type {
                        ClickType::Single => {
                            // Single click: position cursor
                            controller.with_mut(|s| {
                                s.with_editor_mut(|editor| {
                                    editor.action(
                                        &mut write_font_system(),
                                        GlyphonAction::Click {
                                            x: text_relative_pos.x.0,
                                            y: text_relative_pos.y.0,
                                        },
                                    );
                                });
                            });
                        }
                        ClickType::Double => {
                            // Double click: select word
                            controller.with_mut(|s| {
                                s.with_editor_mut(|editor| {
                                    editor.action(
                                        &mut write_font_system(),
                                        GlyphonAction::DoubleClick {
                                            x: text_relative_pos.x.0,
                                            y: text_relative_pos.y.0,
                                        },
                                    );
                                });
                            });
                        }
                        ClickType::Triple => {
                            // Triple click: select line
                            controller.with_mut(|s| {
                                s.with_editor_mut(|editor| {
                                    editor.action(
                                        &mut write_font_system(),
                                        GlyphonAction::TripleClick {
                                            x: text_relative_pos.x.0,
                                            y: text_relative_pos.y.0,
                                        },
                                    );
                                });
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
            let padding_px: Px = args.padding.into();
            let border_width_px = Px(args.border_width.to_pixels_u32() as i32);

            let text_relative_x_px = cursor_pos.x - padding_px - border_width_px;
            let text_relative_y_px = cursor_pos.y - padding_px - border_width_px;

            if text_relative_x_px >= Px(0) && text_relative_y_px >= Px(0) {
                let current_pos_px = PxPosition::new(text_relative_x_px, text_relative_y_px);
                let last_pos_px = controller.with(|s| s.last_click_position());

                if last_pos_px != Some(current_pos_px) {
                    // Extend selection by dragging
                    controller.with_mut(|s| {
                        s.with_editor_mut(|editor| {
                            editor.action(
                                &mut write_font_system(),
                                GlyphonAction::Drag {
                                    x: current_pos_px.x.0,
                                    y: current_pos_px.y.0,
                                },
                            );
                        });
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
                    s.with_editor_mut(|editor| {
                        editor.action(&mut write_font_system(), action);
                    });
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
                s.with_editor_mut(|editor| {
                    // Set cursor to the beginning of the document
                    editor.set_cursor(glyphon::Cursor::new(0, 0));
                    // Set selection to start from the beginning
                    editor.set_selection(glyphon::cosmic_text::Selection::Normal(
                        glyphon::Cursor::new(0, 0),
                    ));
                    // Move cursor to the end, which extends the selection (use BufferEnd for
                    // full document)
                    editor.action(
                        &mut write_font_system(),
                        GlyphonAction::Motion(glyphon::cosmic_text::Motion::BufferEnd),
                    );
                });
            });
        } else {
            // Original logic for other keys
            let mut all_actions = Vec::new();
            controller.with_mut(|s| {
                for key_event in input.keyboard_events.iter().cloned() {
                    if let Some(actions) = s.map_key_event_to_action(key_event, input.key_modifiers)
                    {
                        all_actions.extend(actions);
                    }
                }
            });

            if !all_actions.is_empty() {
                if args.read_only {
                    all_actions.retain(|action| !is_editing_action(action));
                }
                for action in all_actions {
                    handle_action(
                        controller,
                        action,
                        on_change.clone(),
                        input_transform.clone(),
                    );
                }
            }
        }

        // Block all keyboard events to prevent propagation
        input.keyboard_events.clear();

        if !args.read_only {
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
                                    controller,
                                    GlyphonAction::Backspace,
                                    on_change.clone(),
                                    input_transform.clone(),
                                );
                            }
                        }
                        // Insert the committed text
                        for c in text.chars() {
                            handle_action(
                                controller,
                                GlyphonAction::Insert(c),
                                on_change.clone(),
                                input_transform.clone(),
                            );
                        }
                    }
                    winit::event::Ime::Preedit(text, _cursor_offset) => {
                        // Remove the old preedit text if it exists
                        let old_preedit = controller.with_mut(|s| s.preedit_string.take());

                        if let Some(old_preedit) = old_preedit {
                            for _ in 0..old_preedit.chars().count() {
                                handle_action(
                                    controller,
                                    GlyphonAction::Backspace,
                                    on_change.clone(),
                                    input_transform.clone(),
                                );
                            }
                        }
                        // Insert the new preedit text
                        for c in text.chars() {
                            handle_action(
                                controller,
                                GlyphonAction::Insert(c),
                                on_change.clone(),
                                input_transform.clone(),
                            );
                        }
                        controller.with_mut(|c| c.preedit_string = Some(text.to_string()));
                    }
                    _ => {}
                }
            }

            // Request IME window
            input.requests.ime_request = Some(ImeRequest::new(size.into()));
        } else {
            input.ime_events.clear();
        }
    }

    apply_text_input_accessibility(input, args, controller);
}

pub(crate) fn handle_action(
    state: &State<TextInputController>,
    action: GlyphonAction,
    on_change: Arc<dyn Fn(String) -> String + Send + Sync>,
    input_transform: Option<Arc<dyn Fn(String) -> String + Send + Sync>>,
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
    let raw_content_after_action = get_editor_content(&new_editor);
    let transformed_content = if let Some(transform) = input_transform.as_ref() {
        transform(raw_content_after_action.clone())
    } else {
        raw_content_after_action.clone()
    };

    state.with_mut(|c| c.with_editor_mut(|editor| editor.action(&mut write_font_system(), action)));
    let new_content = on_change(transformed_content.clone());

    if new_content != transformed_content {
        state.with_mut(|c| c.set_text(&new_content));
    } else if transformed_content != raw_content_after_action {
        state.with_mut(|c| c.set_text(&transformed_content));
    }
}

fn is_editing_action(action: &GlyphonAction) -> bool {
    matches!(
        action,
        GlyphonAction::Insert(_)
            | GlyphonAction::Backspace
            | GlyphonAction::Delete
            | GlyphonAction::Enter
    )
}

/// Create surface arguments based on editor configuration and state
pub(crate) fn create_surface_args(
    args: &TextInputArgs,
    state: &State<TextInputController>,
) -> crate::surface::SurfaceArgs {
    let border_width = determine_border_width(args, state);
    let style = if border_width.to_pixels_f32() > 0.0 {
        crate::surface::SurfaceStyle::FilledOutlined {
            fill_color: determine_background_color(args, state),
            border_color: determine_border_color(args, state)
                .expect("Border color should exist when border width is positive"),
            border_width,
        }
    } else {
        crate::surface::SurfaceStyle::Filled {
            color: determine_background_color(args, state),
        }
    };

    let mut modifier = args.modifier;
    if args.min_width.is_some() || args.min_height.is_some() {
        modifier = modifier.size_in(args.min_width, None, args.min_height, None);
    }

    SurfaceArgs::default()
        .style(style)
        .shape(args.shape)
        .block_input(true)
        .modifier(modifier)
}

/// Determine background color based on focus state
fn determine_background_color(args: &TextInputArgs, state: &State<TextInputController>) -> Color {
    if state.with(|c| c.focus_handler().is_focused()) {
        args.focus_background_color
            .or(args.background_color)
            .unwrap_or(
                use_context::<MaterialTheme>()
                    .expect("MaterialTheme must be provided")
                    .get()
                    .color_scheme
                    .surface,
            )
    } else {
        args.background_color.unwrap_or(
            use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .color_scheme
                .surface_variant,
        )
    }
}

/// Determine border color based on focus state
fn determine_border_color(
    args: &TextInputArgs,
    state: &State<TextInputController>,
) -> Option<Color> {
    if state.with(|c| c.focus_handler().is_focused()) {
        args.focus_border_color.or(args.border_color).or(Some(
            use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .color_scheme
                .primary,
        ))
    } else {
        args.border_color.or(Some(
            use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get()
                .color_scheme
                .outline_variant,
        ))
    }
}

fn determine_border_width(args: &TextInputArgs, state: &State<TextInputController>) -> Dp {
    if state.with(|c| c.focus_handler().is_focused()) {
        args.focus_border_width.unwrap_or(args.border_width)
    } else {
        args.border_width
    }
}

/// Convenience constructors for common use cases
impl TextInputArgs {
    /// Creates a simple text input with default styling.
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
    /// use tessera_components::text_input::TextInputArgs;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple();
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn simple() -> Self {
        TextInputArgs::default()
            .min_width(Dp(120.0))
            .background_color(
                use_context::<MaterialTheme>()
                    .expect("MaterialTheme must be provided")
                    .get()
                    .color_scheme
                    .surface_variant,
            )
            .border_width(Dp(1.0))
            .border_color(
                use_context::<MaterialTheme>()
                    .expect("MaterialTheme must be provided")
                    .get()
                    .color_scheme
                    .outline_variant,
            )
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                top_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
            })
    }

    /// Creates a text input with an emphasized border for better visibility.
    ///
    /// - Border: 2px, blue focus border
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputArgs;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::outlined();
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn outlined() -> Self {
        Self::simple()
            .with_border_width(Dp(1.0))
            .with_focus_border_color(Color::new(0.0, 0.5, 1.0, 1.0))
    }

    /// Creates a text input with no border (minimal style).
    ///
    /// - Border: 0px, square corners
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputArgs;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::minimal();
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn minimal() -> Self {
        TextInputArgs::default()
            .min_width(Dp(120.0))
            .background_color(Color::WHITE)
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                top_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
            })
    }
}

/// Builder methods for fluent API
impl TextInputArgs {
    /// Sets the minimum width in Dp.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputArgs;
    /// use tessera_ui::Dp;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple().with_min_width(Dp(80.0));
    /// # });
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
    /// use tessera_components::text_input::TextInputArgs;
    /// use tessera_ui::Dp;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple().with_min_height(Dp(40.0));
    /// # });
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
    /// use tessera_components::text_input::TextInputArgs;
    /// use tessera_ui::Color;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple().with_background_color(Color::WHITE);
    /// # });
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
    /// use tessera_components::text_input::TextInputArgs;
    /// use tessera_ui::Dp;
    ///
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple().with_border_width(Dp(1.0));
    /// # });
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
    /// use tessera_components::text_input::TextInputArgs;
    /// use tessera_ui::Color;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple().with_border_color(Color::BLACK);
    /// # });
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
    /// use tessera_components::shape_def::{RoundedCorner, Shape};
    /// use tessera_components::text_input::TextInputArgs;
    /// use tessera_ui::Dp;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple().with_shape(Shape::RoundedRectangle {
    ///     top_left: RoundedCorner::manual(Dp(8.0), 3.0),
    ///     top_right: RoundedCorner::manual(Dp(8.0), 3.0),
    ///     bottom_right: RoundedCorner::manual(Dp(8.0), 3.0),
    ///     bottom_left: RoundedCorner::manual(Dp(8.0), 3.0),
    /// });
    /// # });
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
    /// use tessera_components::text_input::TextInputArgs;
    /// use tessera_ui::Dp;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple().with_padding(Dp(12.0));
    /// # });
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
    /// use tessera_components::text_input::TextInputArgs;
    /// use tessera_ui::Color;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple().with_focus_border_color(Color::new(0.0, 0.5, 1.0, 1.0));
    /// # });
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
    /// use tessera_components::text_input::TextInputArgs;
    /// use tessera_ui::Color;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple().with_focus_background_color(Color::WHITE);
    /// # });
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
    /// use tessera_components::text_input::TextInputArgs;
    /// use tessera_ui::Color;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme(|| MaterialTheme::default(), || {
    /// let args = TextInputArgs::simple().with_selection_color(Color::new(0.5, 0.7, 1.0, 0.4));
    /// # });
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

fn apply_text_input_accessibility(
    input: &mut tessera_ui::InputHandlerInput<'_>,
    args: &TextInputArgs,
    state: &State<TextInputController>,
) {
    let mut builder = input.accessibility().role(Role::MultilineTextInput);
    if !args.enabled {
        builder = builder.disabled();
    }

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

    builder = builder.editable_text(args.enabled && !args.read_only);
    if args.enabled {
        builder = builder.focusable();
    }
    builder.commit();
}
