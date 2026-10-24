//! Editable text input with Material outlined styling.
//!
//! ## Usage
//!
//! Collect short-form input when the full text field decoration is not needed.
use glyphon::Action as GlyphonAction;
use tessera_foundation::gesture::{ScrollRecognizer, TapRecognizer};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, CallbackWith, Color, ComputedData,
    CursorEventContent, Dp, ImeInput, ImeInputModifierNode, ImeRequest, KeyboardInput,
    KeyboardInputModifierNode, Modifier, PointerInput, PointerInputModifierNode, Px, PxPosition,
    PxSize, ScrollDeltaUnit, SemanticsModifierNode, State,
    accesskit::{Action, Role},
    clipboard::{ClipboardAction, ClipboardInput, ClipboardInputModifierNode},
    layout::layout,
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    normalize_platform_scroll_delta, remember, tessera, use_context, winit,
};

use crate::{
    modifier::{InteractionState, InteractionVisualState, ModifierExt as _},
    pos_misc::is_position_inside_bounds,
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceStyle, surface},
    text_edit_core::{
        ClickType, ImeEditResult, PlannedImeEvent, RectDef, TextSelection, text_edit_core,
    },
    theme::{MaterialColorScheme, MaterialShapes, MaterialTheme, TextSelectionColors, TextStyle},
};

/// State structure for the text input, managing text content, cursor,
/// selection, and editing logic.
pub use crate::text_edit_core::{
    DisplayTransform, TextEditorController as TextInputController,
    TransformedText as DisplayTransformText,
};

/// Material 3 opacity applied to disabled field text and labels.
pub(crate) const DISABLED_CONTENT_ALPHA: f32 = 0.38;

/// Material 3 opacity applied to the outline of a disabled field.
pub(crate) const DISABLED_OUTLINE_ALPHA: f32 = 0.12;

/// Material 3 opacity applied to the container of a disabled filled field.
pub(crate) const DISABLED_FILL_ALPHA: f32 = 0.04;

struct TextInputPointerModifierNode {
    args: TextInputProps,
    controller: State<TextInputController>,
    tap_recognizer: State<TapRecognizer>,
    scroll_recognizer: State<ScrollRecognizer>,
}

impl PointerInputModifierNode for TextInputPointerModifierNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        handle_text_input(
            &mut input,
            &self.args,
            &self.controller,
            self.tap_recognizer,
            self.scroll_recognizer,
        );
    }
}

struct TextInputKeyboardModifierNode {
    args: TextInputProps,
    controller: State<TextInputController>,
}

impl KeyboardInputModifierNode for TextInputKeyboardModifierNode {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        handle_text_input_keyboard(
            &mut input,
            &self.args,
            &self.controller,
            &self.args.on_change,
            &self.args.input_transform,
        );
    }
}

struct TextInputImeModifierNode {
    args: TextInputProps,
    controller: State<TextInputController>,
}

struct TextInputClipboardModifierNode {
    args: TextInputProps,
    controller: State<TextInputController>,
}

impl ClipboardInputModifierNode for TextInputClipboardModifierNode {
    fn on_clipboard_input(&self, mut input: ClipboardInput<'_>) {
        let controller = self.controller;
        if !self.args.enabled || !controller.with(|c| c.focus_handler().is_focused()) {
            return;
        }
        input.handle();
        match input.action {
            ClipboardAction::Paste(text) => {
                if !self.args.read_only {
                    controller.with_mut(|c| {
                        c.replace_selected_text_with_pipeline(
                            text,
                            self.args.on_change,
                            self.args.input_transform,
                        );
                    });
                }
            }
            ClipboardAction::Copy | ClipboardAction::Cut => {
                if self.args.read_only && matches!(input.action, ClipboardAction::Cut) {
                    return;
                }
                let Some(text) = controller.with(|c| c.selected_text()) else {
                    return;
                };
                if matches!(input.action, ClipboardAction::Cut) {
                    let (version, selection) =
                        controller.with(|c| (c.text_layout_version(), c.selection()));
                    let on_change = self.args.on_change;
                    let input_transform = self.args.input_transform;
                    input.write_text(text, move || {
                        if controller.is_alive()
                            && controller.with(|c| {
                                c.text_layout_version() == version && c.selection() == selection
                            })
                        {
                            controller.with_mut(|c| {
                                c.replace_selected_text_with_pipeline(
                                    "",
                                    on_change,
                                    input_transform,
                                );
                            });
                        }
                    });
                } else {
                    input.write_text(text, || {});
                }
            }
        }
    }
}

impl ImeInputModifierNode for TextInputImeModifierNode {
    fn on_ime_input(&self, mut input: ImeInput<'_>) {
        handle_text_input_ime(
            &mut input,
            &self.args,
            &self.controller,
            &self.args.on_change,
            &self.args.input_transform,
        );
    }
}

fn apply_text_input_input_modifiers(
    base: Modifier,
    args: TextInputProps,
    controller: State<TextInputController>,
    tap_recognizer: State<TapRecognizer>,
    scroll_recognizer: State<ScrollRecognizer>,
) -> Modifier {
    let modifier = if args.enabled {
        base.hover_cursor_icon(winit::window::CursorIcon::Text)
    } else {
        base
    };
    modifier
        .push_semantics(TextInputSemanticsModifierNode {
            args: args.clone(),
            controller,
        })
        .push_pointer_input(TextInputPointerModifierNode {
            args: args.clone(),
            controller,
            tap_recognizer,
            scroll_recognizer,
        })
        .push_keyboard_input(TextInputKeyboardModifierNode {
            args: args.clone(),
            controller,
        })
        .push_clipboard_input(TextInputClipboardModifierNode {
            args: args.clone(),
            controller,
        })
        .push_ime_input(TextInputImeModifierNode { args, controller })
}

#[derive(Clone, PartialEq)]
pub(crate) struct TextInputProps {
    /// Whether the editor is enabled for user input.
    pub enabled: bool,
    pub presentation_state: Option<InteractionVisualState>,
    /// Whether the editor is read-only.
    pub read_only: bool,
    /// Whether the field is displaying an error state.
    pub is_error: bool,
    pub(crate) is_secure: bool,
    /// Optional modifier chain applied to the editor container.
    pub modifier: Modifier,
    /// Called when the text content changes. The closure receives the new text
    /// content and returns the updated content.
    pub on_change: CallbackWith<String, String>,
    /// Called when the user submits a single-line field with the Enter key.
    pub on_submit: Callback,
    /// Minimum width in density-independent pixels. Defaults to 120dp if not
    /// specified.
    pub min_width: Option<Dp>,
    /// Minimum height in density-independent pixels. Defaults to line height +
    /// padding if not specified.
    pub min_height: Option<Dp>,
    /// Background color of the text input (RGBA). Defaults to light gray.
    pub background_color: Option<Color>,
    /// Border width in Dp. Defaults to 1.0 Dp.
    pub border_width: Dp,
    /// Border color (RGBA). Defaults to gray.
    pub border_color: Option<Color>,
    /// The shape of the text input container.
    pub shape: Shape,
    /// Padding inside the text input. Defaults to 5.0 Dp.
    pub padding: Dp,
    /// Border color when focused (RGBA). Defaults to blue.
    pub focus_border_color: Option<Color>,
    /// Border width when focused. Defaults to the unfocused border width.
    pub focus_border_width: Option<Dp>,
    /// Background color when focused (RGBA). Defaults to white.
    pub focus_background_color: Option<Color>,
    /// Color for text selection highlight (RGBA). Defaults to light blue with
    /// transparency.
    pub selection_color: Option<Color>,
    /// Color of the text content. Defaults to the theme on-surface color.
    pub text_color: Option<Color>,
    /// Color of the text cursor. Defaults to the theme primary color.
    pub cursor_color: Option<Color>,
    /// Optional label announced by assistive technologies.
    pub accessibility_label: Option<String>,
    /// Optional description announced by assistive technologies.
    pub accessibility_description: Option<String>,
    /// Initial text content.
    pub initial_text: Option<String>,
    /// Font size in Dp. Defaults to 14.0.
    pub font_size: Dp,
    /// Line height in Dp. Defaults to None (1.2x font size).
    pub line_height: Option<Dp>,
    /// Optional typography override for editable text.
    pub style: Option<TextStyle>,
    /// Whether the editor behaves as a single-line field.
    ///
    /// When `true`, text does not wrap and the internal text buffer uses
    /// horizontal scrolling semantics.
    pub single_line: bool,
    /// Optional transform applied to text changes before on_change.
    pub input_transform: Option<CallbackWith<String, String>>,
    /// Optional transform applied only for display.
    pub display_transform: Option<DisplayTransform>,
    /// Optional external controller for text, cursor, and selection state.
    ///
    /// When this is `None`, `text_input` creates and owns an internal
    /// controller.
    pub controller: Option<State<TextInputController>>,
}

impl Default for TextInputProps {
    fn default() -> Self {
        Self {
            enabled: true,
            presentation_state: None,
            read_only: false,
            is_error: false,
            is_secure: false,
            modifier: Modifier::new(),
            on_change: CallbackWith::default_value(),
            on_submit: Callback::noop(),
            min_width: Some(Dp(280.0)),
            min_height: Some(Dp(56.0)),
            background_color: None,
            border_width: Dp(1.0),
            border_color: None,
            shape: MaterialShapes::default().extra_small,
            padding: Dp(16.0),
            focus_border_color: None,
            focus_border_width: Some(Dp(2.0)),
            focus_background_color: None,
            selection_color: None,
            text_color: None,
            cursor_color: None,
            accessibility_label: None,
            accessibility_description: None,
            initial_text: None,
            font_size: Dp(16.0),
            line_height: Some(Dp(24.0)),
            style: None,
            single_line: false,
            input_transform: None,
            display_transform: None,
            controller: None,
        }
    }
}

/// # text_input
///
/// Renders an editable text field with Material outlined styling and colors
/// resolved from the active theme.
///
/// ## Usage
///
/// Create an interactive text input for forms, note-taking, or other text
/// input scenarios.
///
/// ## Parameters
///
/// - Parameters configure the editor's appearance, layout, transforms, and
///   controller state.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::text_input::text_input;
/// use tessera_ui::Dp;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// text_input().padding(Dp(8.0)).initial_text("Hello World");
/// # });
/// # }
/// # component();
/// ```
impl TextInputBuilder {
    /// Set a display-only transform with explicit offset mapping.
    pub fn display_transform_mapped<F>(mut self, transform: F) -> Self
    where
        F: Fn(&str) -> DisplayTransformText + Send + Sync + 'static,
    {
        self.props.display_transform =
            Some(CallbackWith::new(move |value: String| transform(&value)));
        self
    }
}

/// # text_input
///
/// Renders an editable text field with Material outlined styling and colors
/// resolved from the active theme.
///
/// ## Usage
///
/// Create an interactive text input for forms, note-taking, or other text
/// input scenarios.
///
/// ## Parameters
///
/// - `enabled` — whether the editor is enabled for user input.
/// - `read_only` — whether the editor is read-only.
/// - `is_error` — whether the field is displaying an error state.
/// - `modifier` — optional modifier chain applied to the editor container.
/// - `on_change` — called when text changes.
/// - `on_submit` — called when the user submits a single-line field.
/// - `min_width` — optional minimum width.
/// - `min_height` — optional minimum height.
/// - `background_color` — optional background color.
/// - `border_width` — border width in Dp.
/// - `border_color` — optional border color.
/// - `shape` — container shape.
/// - `padding` — internal padding.
/// - `presentation_state` — optional visual interaction; live interaction takes
///   precedence.
/// - `focus_border_color` — optional focused border color.
/// - `focus_border_width` — optional focused border width.
/// - `focus_background_color` — optional focused background color.
/// - `selection_color` — optional selection highlight color.
/// - `text_color` — optional text color.
/// - `cursor_color` — optional cursor color.
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `initial_text` — optional initial text content.
/// - `font_size` — font size in Dp.
/// - `style` — typography override, including weight and letter spacing.
/// - `line_height` — optional line height in Dp.
/// - `single_line` — whether the editor behaves as a single-line field.
/// - `input_transform` — optional transform applied to text changes before
///   `on_change`.
/// - `display_transform` — optional display-only transform.
/// - `controller` — optional external controller for text, cursor, and
///   selection state.
///
/// ## Examples
/// ```rust
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::text_input::text_input;
/// use tessera_ui::Dp;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// text_input().padding(Dp(8.0)).initial_text("Hello World");
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn text_input(
    enabled: Option<bool>,
    presentation_state: Option<InteractionVisualState>,
    read_only: Option<bool>,
    is_error: Option<bool>,
    modifier: Option<Modifier>,
    on_change: Option<CallbackWith<String, String>>,
    on_submit: Option<Callback>,
    min_width: Option<Dp>,
    min_height: Option<Dp>,
    background_color: Option<Color>,
    border_width: Option<Dp>,
    border_color: Option<Color>,
    shape: Option<Shape>,
    padding: Option<Dp>,
    focus_border_color: Option<Color>,
    focus_border_width: Option<Dp>,
    focus_background_color: Option<Color>,
    selection_color: Option<Color>,
    text_color: Option<Color>,
    cursor_color: Option<Color>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    #[prop(into)] initial_text: Option<String>,
    font_size: Option<Dp>,
    line_height: Option<Dp>,
    style: Option<TextStyle>,
    single_line: Option<bool>,
    input_transform: Option<CallbackWith<String, String>>,
    display_transform: Option<DisplayTransform>,
    controller: Option<State<TextInputController>>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let props_default = TextInputProps::default();
    let enabled = enabled.unwrap_or(true);
    let read_only = read_only.unwrap_or(false);
    let is_error = is_error.unwrap_or(false);
    let modifier = modifier.unwrap_or_default();
    let min_width = min_width.or(props_default.min_width);
    let min_height = min_height.or(props_default.min_height);
    let background_color = background_color.or(Some(Color::TRANSPARENT));
    let border_width = border_width.unwrap_or(props_default.border_width);
    let border_color = border_color.or(Some(scheme.outline));
    let shape = shape.unwrap_or(theme.shapes.extra_small);
    let padding = padding.unwrap_or(props_default.padding);
    let focus_border_color = focus_border_color.or(Some(scheme.primary));
    let focus_border_width = focus_border_width.or(props_default.focus_border_width);
    let focus_background_color = focus_background_color.or(Some(Color::TRANSPARENT));
    let selection_color =
        selection_color.or(Some(TextSelectionColors::from_scheme(&scheme).background));
    let text_color = text_color.or(Some(if enabled {
        scheme.on_surface
    } else {
        scheme.on_surface.with_alpha(DISABLED_CONTENT_ALPHA)
    }));
    let cursor_color = cursor_color.or(Some(if is_error {
        scheme.error
    } else {
        scheme.primary
    }));
    let style = style.unwrap_or(theme.typography.body_large);
    let font_size = font_size.unwrap_or(style.font_size);
    let line_height = line_height.or(style.line_height);
    let single_line = single_line.unwrap_or(false);
    let args = TextInputProps {
        enabled,
        presentation_state,
        read_only,
        is_error,
        is_secure: false,
        modifier,
        on_change: on_change.unwrap_or_else(CallbackWith::default_value),
        on_submit: on_submit.unwrap_or_else(Callback::noop),
        min_width,
        min_height,
        background_color,
        border_width,
        border_color,
        shape,
        padding,
        focus_border_color,
        focus_border_width,
        focus_background_color,
        selection_color,
        text_color,
        cursor_color,
        accessibility_label,
        accessibility_description,
        initial_text,
        font_size,
        line_height,
        style: Some(TextStyle {
            font_size,
            line_height,
            ..style
        }),
        single_line,
        input_transform,
        display_transform,
        controller,
    };
    let controller = args.controller.unwrap_or_else(|| {
        remember(|| {
            let mut c = TextInputController::new(args.font_size, args.line_height);
            if let Some(text) = &args.initial_text {
                c.set_text(text);
            }
            c
        })
    });
    let mut editor_args = args.clone();
    editor_args.controller = Some(controller);

    if !editor_args.enabled {
        controller.with_mut(|c| c.focus_handler_mut().unfocus());
    }
    sync_text_input_controller(&controller, &editor_args);
    let focus = controller.with(|c| *c.focus_handler());
    let tap_recognizer = remember(TapRecognizer::default);
    let scroll_recognizer = remember(ScrollRecognizer::default);
    let modifier = apply_text_input_input_modifiers(
        Modifier::new()
            .focus_requester(focus)
            .focusable()
            .focus_properties(
                tessera_ui::FocusProperties::new()
                    .can_focus(editor_args.enabled)
                    .can_request_focus(editor_args.enabled),
            ),
        editor_args.clone(),
        controller,
        tap_recognizer,
        scroll_recognizer,
    );

    let current = editor_args;
    let editor_args = remember(|| current.clone());
    if editor_args.with(|args| args != &current) {
        editor_args.set(current);
    }
    layout().modifier(modifier).child(move || {
        let args = editor_args.get();
        let text_input_surface = create_surface_args(&args, &controller, &scheme);
        surface()
            .style(text_input_surface.style)
            .shape(text_input_surface.shape)
            .block_input(text_input_surface.block_input)
            .modifier(text_input_surface.modifier)
            .child(move || {
                text_input_padded_content()
                    .props(editor_args)
                    .controller(controller);
            });
    });
}

#[tessera]
fn text_input_editor(props: TextInputProps) {
    let controller = props
        .controller
        .expect("text_input_editor requires controller to be set");
    let editor_args: TextInputProps = props.clone();

    if !editor_args.enabled {
        controller.with_mut(|c| c.focus_handler_mut().unfocus());
    }
    sync_text_input_controller(&controller, &editor_args);
    let focus = controller.with(|c| *c.focus_handler());
    let tap_recognizer = remember(TapRecognizer::default);
    let scroll_recognizer = remember(ScrollRecognizer::default);
    let modifier = apply_text_input_input_modifiers(
        editor_args.modifier.clone().then(
            Modifier::new()
                .focus_requester(focus)
                .focusable()
                .focus_properties(
                    tessera_ui::FocusProperties::new()
                        .can_focus(editor_args.enabled)
                        .can_request_focus(editor_args.enabled),
                ),
        ),
        editor_args.clone(),
        controller,
        tap_recognizer,
        scroll_recognizer,
    );

    let current = editor_args;
    let editor_args = remember(|| current.clone());
    if editor_args.with(|args| args != &current) {
        editor_args.set(current);
    }
    layout().modifier(modifier).child(move || {
        text_input_padded_content()
            .props(editor_args)
            .controller(controller);
    });
}

#[tessera]
fn text_input_padded_content(
    props: Option<State<TextInputProps>>,
    controller: Option<State<TextInputController>>,
) {
    let props = props.expect("text_input_padded_content requires props to be set");
    let padding = props.with(|args| args.padding);
    let controller = controller.expect("text_input_padded_content requires controller to be set");
    layout()
        .modifier(Modifier::new().padding_all(padding))
        .child(move || {
            text_edit_core()
                .controller(controller)
                .presentation_focused(visual_focus(&props.get(), controller));
        });
}

pub(crate) fn visual_focus(args: &TextInputProps, controller: State<TextInputController>) -> bool {
    let mut live = InteractionState::new();
    live.set_focused(controller.with(|c| c.focus_handler().is_focused()));
    live.with_presentation(args.presentation_state, args.enabled)
        .is_focused()
}

pub(crate) fn text_input_core(args: &TextInputProps, controller: State<TextInputController>) {
    let mut core_args = args.clone();
    core_args.controller = Some(controller);
    text_input_editor(core_args);
}

fn sync_text_input_controller(controller: &State<TextInputController>, args: &TextInputProps) {
    if let Some(style) = args.style
        && controller.with(|c| c.text_style()) != style
    {
        controller.with_mut(|c| c.set_text_style(style));
    }

    if let Some(selection_color) = args.selection_color {
        let needs_update = controller.with(|c| c.selection_color() != selection_color);
        if needs_update {
            controller.with_mut(|c| c.set_selection_color(selection_color));
        }
    }
    if let Some(text_color) = args.text_color {
        let needs_update = controller.with(|c| c.text_color() != text_color);
        if needs_update {
            controller.with_mut(|c| c.set_text_color(text_color));
        }
    }
    if let Some(cursor_color) = args.cursor_color {
        let needs_update = controller.with(|c| c.cursor_color() != cursor_color);
        if needs_update {
            controller.with_mut(|c| c.set_cursor_color(cursor_color));
        }
    }
    let display_transform = args.display_transform;
    let needs_display_transform_update =
        controller.with(|c| c.display_transform() != display_transform);
    if needs_display_transform_update {
        controller.with_mut(|c| c.set_display_transform(display_transform));
    }
    let needs_single_line_update = controller.with(|c| c.single_line() != args.single_line);
    if needs_single_line_update {
        controller.with_mut(|c| c.set_single_line(args.single_line));
    }
}

fn handle_text_input(
    input: &mut tessera_ui::PointerInput<'_>,
    args: &TextInputProps,
    controller: &State<TextInputController>,
    tap_recognizer: State<TapRecognizer>,
    scroll_recognizer: State<ScrollRecognizer>,
) {
    if !args.enabled {
        return;
    }
    let size = input.computed_data; // This is the full surface size
    let cursor_pos_option = input.cursor_position_rel;
    let is_cursor_in_editor = cursor_pos_option
        .map(|pos| is_position_inside_bounds(size, pos))
        .unwrap_or(false);
    let tap_result = tap_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            is_cursor_in_editor,
        )
    });

    // Handle click events - now we have a full clickable area from surface
    if is_cursor_in_editor {
        if tap_result.pressed {
            // Request focus if not already focused
            if !controller.with(|s| s.focus_handler().is_focused()) {
                controller.with_mut(|s| {
                    s.focus_handler_mut().request_focus();
                });
            }

            // Handle cursor positioning for clicks
            if let Some(cursor_pos) = cursor_pos_option {
                let text_relative_pos =
                    click_selection_pointer_position(cursor_pos, args, controller, size);
                let Some(click_timestamp) = tap_result.press_timestamp else {
                    return;
                };
                // Determine click type and handle accordingly
                let window_position = input
                    .pointer_changes
                    .iter()
                    .find(|change| {
                        change.timestamp == click_timestamp
                            && matches!(change.content, CursorEventContent::Pressed(_))
                    })
                    .and_then(|change| change.position())
                    .or_else(|| input.cursor_position_abs())
                    .unwrap_or_else(|| input.local_to_window.map_point(cursor_pos));
                let click_type = controller.with_mut(|s| {
                    s.handle_click(text_relative_pos, window_position, click_timestamp)
                });

                match click_type {
                    ClickType::Single => {
                        if input.key_modifiers.shift_key() {
                            controller.with_mut(|s| {
                                s.extend_selection_to_point(text_relative_pos);
                            });
                        } else {
                            controller.with_mut(|s| {
                                s.apply_pointer_action(GlyphonAction::Click {
                                    x: text_relative_pos.x_px().0,
                                    y: text_relative_pos.y_px().0,
                                });
                            });
                        }
                    }
                    ClickType::Double => {
                        controller.with_mut(|s| {
                            s.apply_pointer_action(GlyphonAction::DoubleClick {
                                x: text_relative_pos.x_px().0,
                                y: text_relative_pos.y_px().0,
                            });
                        });
                    }
                    ClickType::Triple => {
                        controller.with_mut(|s| {
                            s.apply_pointer_action(GlyphonAction::TripleClick {
                                x: text_relative_pos.x_px().0,
                                y: text_relative_pos.y_px().0,
                            });
                        });
                    }
                }

                // Start potential drag operation
                controller.with_mut(|s| s.start_drag(click_type));
            }
        }

        // Handle drag events (mouse move while dragging)
        // This happens every frame when cursor position changes during drag
        if controller.with(|s| s.is_dragging())
            && let Some(cursor_pos) = cursor_pos_option
        {
            let drag_target = drag_selection_pointer_position(cursor_pos, args, controller, size);
            auto_scroll_drag_selection(&drag_target, controller);
            let current_pos_px =
                drag_selection_pointer_position(cursor_pos, args, controller, size).position;
            let last_pos_px = controller.with(|s| s.last_click_position());

            if last_pos_px != Some(current_pos_px) {
                controller.with_mut(|s| {
                    s.apply_pointer_action(GlyphonAction::Drag {
                        x: current_pos_px.x_px().0,
                        y: current_pos_px.y_px().0,
                    });
                });

                // Update last position to current position
                controller.with_mut(|s| {
                    s.update_last_click_position(current_pos_px);
                });
            }
        }

        scroll_recognizer.with_mut(|recognizer| {
            recognizer.for_each(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                |_context, event| {
                    let (dx, dy) = normalize_platform_scroll_delta(
                        event.delta_x,
                        event.delta_y,
                        event.unit,
                        event.source,
                    );
                    if controller.with(|s| s.single_line()) {
                        let requested = if dx.abs() > f32::EPSILON { dx } else { dy };
                        if requested.abs() <= f32::EPSILON
                            || !controller.with_mut(|s| s.has_horizontal_overflow())
                        {
                            return;
                        }
                        let consumed =
                            controller.with_mut(|s| s.scroll_horizontal_consumed(-requested));
                        // `consumed` is in editor scroll space; the raw scroll
                        // event uses the opposite sign.
                        let consumed_raw = -consumed;
                        if dx.abs() > f32::EPSILON {
                            event.delta_x = dx - consumed_raw;
                            event.delta_y = dy;
                        } else {
                            event.delta_y = dy - consumed_raw;
                            event.delta_x = dx;
                        }
                    } else {
                        if dy.abs() <= f32::EPSILON
                            || !controller.with_mut(|s| s.has_vertical_overflow())
                        {
                            return;
                        }
                        // The editor engine clamps vertical scroll to the
                        // content extent, so `consumed` is the exact amount
                        // applied and the remainder goes to an ancestor.
                        let consumed = controller.with_mut(|s| s.scroll_vertical_consumed(-dy));
                        event.delta_y = dy + consumed;
                    }
                    // Rewrite the event as already-normalized pixels so an
                    // ancestor scroll container receives exactly the remainder.
                    event.unit = ScrollDeltaUnit::Pixel;
                },
            );
        });
    }

    // Handle mouse release events (end drag), even if the pointer is outside
    // the editor bounds.
    if tap_result.released {
        controller.with_mut(|s| s.stop_drag());
    }

    // Only block cursor events when focused to prevent propagation.
    let should_consume_pointer = controller.with(|s| {
        s.focus_handler().is_focused()
            && (is_cursor_in_editor || s.is_dragging() || tap_result.released)
    });
    if should_consume_pointer {
        input.consume_pointer_changes();
    }

    if args.enabled && !args.read_only && controller.with(|s| s.focus_handler().is_focused()) {
        let computed_data = input.computed_data;
        input
            .ime_session()
            .update(current_ime_request(args, controller, computed_data));
    }
}

fn click_selection_pointer_position(
    cursor_pos: PxPosition,
    args: &TextInputProps,
    controller: &State<TextInputController>,
    size: ComputedData,
) -> PxPosition {
    drag_selection_pointer_position(cursor_pos, args, controller, size).position
}

fn text_content_origin(args: &TextInputProps) -> PxPosition {
    let padding_px: Px = args.padding.into();
    let border_width_px = Px(args.border_width.to_pixels_u32() as i32);
    text_content_origin_from_values(padding_px, border_width_px)
}

fn text_content_origin_from_values(padding_px: Px, border_width_px: Px) -> PxPosition {
    PxPosition::new(padding_px + border_width_px, padding_px + border_width_px)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DragSelectionPointerPosition {
    position: PxPosition,
    overflow_x: Px,
    overflow_y: Px,
}

fn drag_selection_pointer_position(
    cursor_pos: PxPosition,
    args: &TextInputProps,
    controller: &State<TextInputController>,
    size: ComputedData,
) -> DragSelectionPointerPosition {
    let (horizontal_scroll, vertical_scroll) = controller.with(|s| {
        (
            Px(s.scroll_state().horizontal().round() as i32),
            Px(s.scroll_state().vertical().round() as i32),
        )
    });
    let text_origin = text_content_origin(args);
    drag_selection_pointer_position_with_scroll(
        cursor_pos,
        size,
        text_origin,
        horizontal_scroll,
        vertical_scroll,
    )
}

fn drag_selection_pointer_position_with_scroll(
    cursor_pos: PxPosition,
    size: ComputedData,
    text_origin: PxPosition,
    horizontal_scroll: Px,
    vertical_scroll: Px,
) -> DragSelectionPointerPosition {
    let viewport_size = text_viewport_size_from_origin(size, text_origin);
    let viewport_width = viewport_size.width.0 as f32;
    let viewport_height = viewport_size.height.0 as f32;
    let text_relative_x = cursor_pos.x - text_origin.x;
    let text_relative_y = cursor_pos.y - text_origin.y;
    let overflow_x = if text_relative_x < 0.0 {
        Px(text_relative_x.round() as i32)
    } else if text_relative_x > viewport_width {
        Px((text_relative_x - viewport_width).round() as i32)
    } else {
        Px::ZERO
    };
    let overflow_y = if text_relative_y < 0.0 {
        Px(text_relative_y.round() as i32)
    } else if text_relative_y > viewport_height {
        Px((text_relative_y - viewport_height).round() as i32)
    } else {
        Px::ZERO
    };
    let clamped_x = text_relative_x.clamp(0.0, viewport_width);
    let clamped_y = text_relative_y.clamp(0.0, viewport_height);

    DragSelectionPointerPosition {
        position: PxPosition::from_f32_xy(
            clamped_x + horizontal_scroll.0 as f32,
            clamped_y + vertical_scroll.0 as f32,
        ),
        overflow_x,
        overflow_y,
    }
}

fn text_viewport_size_from_origin(size: ComputedData, text_origin: PxPosition) -> ComputedData {
    let width = (size.width.0 as f32 - text_origin.x * 2.0).round() as i32;
    let height = (size.height.0 as f32 - text_origin.y * 2.0).round() as i32;
    ComputedData {
        width: Px(width).max(Px::ZERO),
        height: Px(height).max(Px::ZERO),
    }
}

fn auto_scroll_drag_selection(
    drag_target: &DragSelectionPointerPosition,
    controller: &State<TextInputController>,
) {
    controller.with_mut(|s| {
        if s.single_line() {
            if drag_target.overflow_x != Px::ZERO {
                s.scroll_horizontal_by(drag_target.overflow_x.to_f32());
            }
            return;
        }
        if drag_target.overflow_y != Px::ZERO {
            s.scroll_vertical_by(drag_target.overflow_y.to_f32());
        }
    });
}

fn handle_text_input_keyboard(
    input: &mut tessera_ui::KeyboardInput<'_>,
    args: &TextInputProps,
    controller: &State<TextInputController>,
    on_change: &CallbackWith<String, String>,
    input_transform: &Option<CallbackWith<String, String>>,
) {
    if !args.enabled || !controller.with(|s| s.focus_handler().is_focused()) {
        return;
    }

    let is_ctrl = input.key_modifiers.control_key() || input.key_modifiers.super_key();
    let is_shift = input.key_modifiers.shift_key();
    let mut all_actions = Vec::new();
    let mut should_block_keyboard = false;
    for key_event in input.keyboard_events.iter().cloned() {
        if let Some(behavior) =
            single_line_key_behavior(args.single_line, key_event.state, &key_event.logical_key)
        {
            match behavior {
                SingleLineKeyBehavior::Submit => {
                    if args.on_submit != Callback::noop() {
                        args.on_submit.call();
                        should_block_keyboard = true;
                    }
                }
                SingleLineKeyBehavior::Propagate => {}
            }
            continue;
        }

        if let Some(behavior) =
            clipboard_shortcut_for_key(is_ctrl, is_shift, key_event.state, &key_event.logical_key)
        {
            should_block_keyboard = true;
            #[cfg(not(target_family = "wasm"))]
            match behavior {
                ClipboardShortcutBehavior::Copy => {
                    controller.with(|s| {
                        s.copy_selection_to_clipboard();
                    });
                }
                ClipboardShortcutBehavior::Cut => {
                    if !args.read_only {
                        controller.with_mut(|s| {
                            s.cut_selection_with_pipeline(*on_change, *input_transform);
                        });
                    }
                }
                ClipboardShortcutBehavior::Paste => {
                    if !args.read_only {
                        controller.with_mut(|s| {
                            s.paste_from_clipboard_with_pipeline(*on_change, *input_transform);
                        });
                    }
                }
            }
            #[cfg(target_family = "wasm")]
            let _ = behavior;
            continue;
        }

        if let Some(motion) =
            deletion_motion_for_key(is_ctrl, key_event.state, &key_event.logical_key)
        {
            should_block_keyboard = true;
            if !args.read_only {
                controller.with_mut(|s| {
                    s.delete_motion_with_pipeline(motion, *on_change, *input_transform);
                });
            }
            continue;
        }

        should_block_keyboard = true;
        let shortcut = if let winit::keyboard::Key::Character(s) = &key_event.logical_key {
            if is_ctrl && key_event.state == winit::event::ElementState::Pressed {
                Some(s.to_lowercase())
            } else {
                None
            }
        } else {
            None
        };

        match shortcut.as_deref() {
            Some("a") => {
                controller.with_mut(|s| s.select_all());
                continue;
            }
            Some("c") => {
                controller.with(|s| {
                    s.copy_selection_to_clipboard();
                });
                continue;
            }
            Some("x") => {
                if !args.read_only {
                    controller.with_mut(|s| {
                        s.cut_selection_with_pipeline(*on_change, *input_transform);
                    });
                }
                continue;
            }
            Some("v") => {
                if !args.read_only {
                    controller.with_mut(|s| {
                        s.paste_from_clipboard_with_pipeline(*on_change, *input_transform);
                    });
                }
                continue;
            }
            _ => {}
        }

        controller.with_mut(|s| {
            if let Some(actions) = s.map_key_event_to_action(key_event, input.key_modifiers) {
                all_actions.extend(actions);
            }
        });
    }
    if !all_actions.is_empty() {
        if args.read_only {
            all_actions.retain(|action| !is_editing_action(action));
        }
        for action in all_actions {
            handle_action(controller, action, *on_change, *input_transform);
        }
    }

    if !args.read_only && controller.with(|s| s.focus_handler().is_focused()) {
        let computed_data = input.computed_data;
        input
            .ime_session()
            .update(current_ime_request(args, controller, computed_data));
    }

    if should_block_keyboard {
        input.block_keyboard();
    }
}

fn handle_text_input_ime(
    input: &mut tessera_ui::ImeInput<'_>,
    args: &TextInputProps,
    controller: &State<TextInputController>,
    on_change: &CallbackWith<String, String>,
    input_transform: &Option<CallbackWith<String, String>>,
) {
    if !args.enabled || !controller.with(|s| s.focus_handler().is_focused()) {
        return;
    }
    if args.read_only {
        input.block_ime();
        return;
    }

    for event in input.ime_events.drain(..) {
        let Some(plan) = controller.with(|c| c.plan_ime_event(args.single_line, &event)) else {
            continue;
        };
        match plan {
            PlannedImeEvent::Submit => {
                controller.with_mut(|c| c.clear_composition());
                if args.on_submit != Callback::noop() {
                    args.on_submit.call();
                }
            }
            PlannedImeEvent::Edit(plan) => {
                let result = replace_text_range_with_selection(
                    controller,
                    plan.replacement_range.clone(),
                    &plan.replacement_text,
                    plan.selection.clone(),
                    *on_change,
                    *input_transform,
                );
                controller.with_mut(|c| c.commit_ime_edit(&plan, &result));
            }
        }
    }

    let computed_data = input.computed_data;
    input
        .ime_session()
        .update(current_ime_request(args, controller, computed_data));
    input.block_ime();
}

fn current_ime_request(
    args: &TextInputProps,
    controller: &State<TextInputController>,
    computed_data: ComputedData,
) -> ImeRequest {
    let (ime_rect, selection_range, composition_range) = controller.with(|c| {
        (
            c.current_ime_rect(),
            Some(c.selection().ordered_range()),
            c.composition().map(|composition| composition.range.clone()),
        )
    });
    build_ime_request(
        text_content_origin(args),
        computed_data,
        ime_rect,
        selection_range,
        composition_range,
    )
}

fn build_ime_request(
    text_origin: PxPosition,
    computed_data: ComputedData,
    ime_rect: Option<RectDef>,
    selection_range: Option<std::ops::Range<usize>>,
    composition_range: Option<std::ops::Range<usize>>,
) -> ImeRequest {
    ime_rect
        .map(|ime_rect| {
            let local_position = text_origin + PxPosition::new(ime_rect.x, ime_rect.y);
            ImeRequest::new(PxSize::new(ime_rect.width, ime_rect.height))
                .with_local_position(local_position)
                .with_selection_range(selection_range.clone())
                .with_composition_range(composition_range.clone())
        })
        .unwrap_or_else(|| {
            ImeRequest::new(computed_data.into())
                .with_selection_range(selection_range)
                .with_composition_range(composition_range)
        })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SingleLineKeyBehavior {
    Submit,
    Propagate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClipboardShortcutBehavior {
    Copy,
    Cut,
    Paste,
}

fn deletion_motion_for_key(
    is_ctrl: bool,
    key_state: winit::event::ElementState,
    logical_key: &winit::keyboard::Key,
) -> Option<glyphon::cosmic_text::Motion> {
    if !is_ctrl || key_state != winit::event::ElementState::Pressed {
        return None;
    }

    match logical_key {
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace) => {
            Some(glyphon::cosmic_text::Motion::LeftWord)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete) => {
            Some(glyphon::cosmic_text::Motion::RightWord)
        }
        _ => None,
    }
}

fn clipboard_shortcut_for_key(
    is_ctrl: bool,
    is_shift: bool,
    key_state: winit::event::ElementState,
    logical_key: &winit::keyboard::Key,
) -> Option<ClipboardShortcutBehavior> {
    if key_state != winit::event::ElementState::Pressed {
        return None;
    }

    match logical_key {
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert) if is_ctrl => {
            Some(ClipboardShortcutBehavior::Copy)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert) if is_shift => {
            Some(ClipboardShortcutBehavior::Paste)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete) if is_shift => {
            Some(ClipboardShortcutBehavior::Cut)
        }
        _ => None,
    }
}

fn single_line_key_behavior(
    single_line: bool,
    key_state: winit::event::ElementState,
    logical_key: &winit::keyboard::Key,
) -> Option<SingleLineKeyBehavior> {
    if !single_line || key_state != winit::event::ElementState::Pressed {
        return None;
    }

    match logical_key {
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter) => {
            Some(SingleLineKeyBehavior::Submit)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab) => {
            Some(SingleLineKeyBehavior::Propagate)
        }
        _ => None,
    }
}

pub(crate) fn handle_action(
    state: &State<TextInputController>,
    action: GlyphonAction,
    on_change: CallbackWith<String, String>,
    input_transform: Option<CallbackWith<String, String>>,
) {
    state.with_mut(|c| c.apply_action_with_pipeline(action, on_change, input_transform));
}

fn replace_text_range_with_selection(
    state: &State<TextInputController>,
    range: std::ops::Range<usize>,
    replacement: &str,
    selection: TextSelection,
    on_change: CallbackWith<String, String>,
    input_transform: Option<CallbackWith<String, String>>,
) -> ImeEditResult {
    state.with_mut(|c| {
        c.replace_text_range_with_pipeline(
            range,
            replacement,
            selection,
            on_change,
            input_transform,
        )
    })
}

struct TextInputSemanticsModifierNode {
    args: TextInputProps,
    controller: State<TextInputController>,
}

impl SemanticsModifierNode for TextInputSemanticsModifierNode {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        apply_text_input_semantics(accessibility, action_handler, &self.args, &self.controller);
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

pub(crate) struct TextInputSurfaceArgs {
    pub(crate) style: SurfaceStyle,
    pub(crate) shape: Shape,
    pub(crate) block_input: bool,
    pub(crate) modifier: Modifier,
}

/// Create surface arguments based on editor configuration and state
pub(crate) fn create_surface_args(
    args: &TextInputProps,
    state: &State<TextInputController>,
    scheme: &MaterialColorScheme,
) -> TextInputSurfaceArgs {
    let focused = visual_focus(args, *state);
    let border_width = determine_border_width(args, focused);
    let is_outlined = border_width.to_pixels_f32() > 0.0;
    let style = if is_outlined {
        SurfaceStyle::FilledOutlined {
            fill_color: determine_background_color(args, focused, is_outlined, scheme),
            border_color: determine_border_color(args, focused, scheme),
            border_width,
        }
    } else {
        SurfaceStyle::Filled {
            color: determine_background_color(args, focused, is_outlined, scheme),
        }
    };

    let mut modifier = args.modifier.clone();
    if args.min_width.is_some() || args.min_height.is_some() {
        modifier = modifier.default_min_size(
            args.min_width.unwrap_or(Dp::ZERO),
            args.min_height.unwrap_or(Dp::ZERO),
        );
    }

    TextInputSurfaceArgs {
        style,
        shape: args.shape,
        block_input: !args.enabled,
        modifier,
    }
}

/// Interaction state that selects the Material 3 field colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FieldVisualState {
    focused: bool,
    enabled: bool,
    is_error: bool,
    outlined: bool,
}

/// Material 3 default container fill for the given state.
///
/// Outlined containers stay transparent, including when disabled; a disabled
/// filled container uses the on-surface color at 4%.
fn default_fill_color(scheme: &MaterialColorScheme, state: FieldVisualState) -> Color {
    if !state.enabled && !state.outlined {
        return scheme.on_surface.with_alpha(DISABLED_FILL_ALPHA);
    }
    Color::TRANSPARENT
}

/// Material 3 default outline color for the given state.
///
/// The outline uses `outline` at rest, `primary` while focused, `error` while
/// invalid, and the on-surface color at 12% when disabled.
fn default_outline_color(scheme: &MaterialColorScheme, state: FieldVisualState) -> Color {
    if !state.enabled {
        return scheme.on_surface.with_alpha(DISABLED_OUTLINE_ALPHA);
    }
    if state.is_error {
        return scheme.error;
    }
    if state.focused {
        scheme.primary
    } else {
        scheme.outline
    }
}

/// Resolves the container fill, letting explicit colors win over the Material
/// defaults except while the field is disabled.
fn determine_background_color(
    args: &TextInputProps,
    focused: bool,
    is_outlined: bool,
    scheme: &MaterialColorScheme,
) -> Color {
    let state = FieldVisualState {
        focused,
        enabled: args.enabled,
        is_error: args.is_error,
        outlined: is_outlined,
    };
    if !args.enabled {
        return default_fill_color(scheme, state);
    }
    let explicit = if focused {
        args.focus_background_color.or(args.background_color)
    } else {
        args.background_color
    };
    explicit.unwrap_or_else(|| default_fill_color(scheme, state))
}

/// Resolves the outline color, letting explicit colors win over the Material
/// defaults except while the field is disabled or invalid.
fn determine_border_color(
    args: &TextInputProps,
    focused: bool,
    scheme: &MaterialColorScheme,
) -> Color {
    let state = FieldVisualState {
        focused,
        enabled: args.enabled,
        is_error: args.is_error,
        outlined: true,
    };
    if !args.enabled || args.is_error {
        return default_outline_color(scheme, state);
    }
    if focused {
        args.focus_border_color
            .or(args.border_color)
            .unwrap_or_else(|| default_outline_color(scheme, state))
    } else {
        args.border_color
            .unwrap_or_else(|| default_outline_color(scheme, state))
    }
}

/// Resolves the outline width for the current interaction state.
///
/// Disabled fields keep their resting width instead of the focused one.
fn resolve_border_width(
    border_width: Dp,
    focus_border_width: Option<Dp>,
    enabled: bool,
    focused: bool,
) -> Dp {
    if !enabled {
        return border_width;
    }
    if focused {
        focus_border_width.unwrap_or(border_width)
    } else {
        border_width
    }
}

fn determine_border_width(args: &TextInputProps, focused: bool) -> Dp {
    resolve_border_width(
        args.border_width,
        args.focus_border_width,
        args.enabled,
        focused,
    )
}

/// Convenience constructors for common use cases
impl TextInputBuilder {
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
    /// use tessera_components::text_input::TextInputBuilder;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple();
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn simple() -> Self {
        text_input()
            .min_width(Dp(120.0))
            .background_color(Color::new(0.9, 0.9, 0.9, 1.0))
            .border_width(Dp(1.0))
            .border_color(Color::new(0.7, 0.7, 0.7, 1.0))
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
    /// use tessera_components::text_input::TextInputBuilder;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::outlined();
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn outlined() -> Self {
        Self::simple()
            .border_width(Dp(1.0))
            .focus_border_color(Color::new(0.0, 0.5, 1.0, 1.0))
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
    /// use tessera_components::text_input::TextInputBuilder;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::minimal();
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn minimal() -> Self {
        text_input()
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

fn apply_text_input_semantics(
    accessibility: &mut AccessibilityNode,
    action_handler: &mut Option<AccessibilityActionHandler>,
    args: &TextInputProps,
    state: &State<TextInputController>,
) {
    let focus = state.with(|c| *c.focus_handler());
    let submit_action_enabled = should_expose_submit_accessibility_action(
        args.single_line,
        focus.is_focused(),
        args.on_submit != Callback::noop(),
    );
    accessibility.role = Some(if args.is_secure {
        Role::PasswordInput
    } else {
        text_input_accessibility_role(args.single_line)
    });
    accessibility.disabled = !args.enabled;
    accessibility.label = args.accessibility_label.clone();
    accessibility.description = args.accessibility_description.clone();

    let current_text = state.with(|c| c.text());
    accessibility.value = (!args.is_secure && !current_text.is_empty()).then_some(current_text);

    accessibility.is_editable_text = args.enabled && !args.read_only;
    accessibility.focusable = args.enabled;
    accessibility.actions.clear();
    if args.enabled && submit_action_enabled {
        accessibility.actions.push(Action::Click);
    }

    if args.enabled {
        let on_submit = args.on_submit;
        *action_handler = Some(Box::new(move |action| match action {
            Action::Focus => focus.request_focus(),
            Action::Blur => focus.clear_focus(),
            Action::Click => {
                if focus.is_focused() && on_submit != Callback::noop() {
                    on_submit.call();
                } else {
                    focus.request_focus();
                }
            }
            _ => {}
        }));
    } else {
        *action_handler = None;
    }
}

fn text_input_accessibility_role(single_line: bool) -> Role {
    if single_line {
        Role::TextInput
    } else {
        Role::MultilineTextInput
    }
}

fn should_expose_submit_accessibility_action(
    single_line: bool,
    focused: bool,
    has_submit_handler: bool,
) -> bool {
    single_line && focused && has_submit_handler
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "text_input/clipboard_tests.rs"]
mod clipboard_tests;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use glyphon::{Action as GlyphonAction, Edit as _};
    use tessera_ui::{
        Color, ComputedData, Dp, Px, PxPosition, PxSize, accesskit::Role, time::Instant, winit,
    };

    use crate::text_edit_core::{
        ClickType, ImeComposition, PlannedImeEdit, PlannedImeEvent, RectDef, TextEditorController,
        TextSelection,
    };

    use super::{
        ClipboardShortcutBehavior, DISABLED_FILL_ALPHA, DISABLED_OUTLINE_ALPHA,
        DragSelectionPointerPosition, FieldVisualState, SingleLineKeyBehavior, build_ime_request,
        clipboard_shortcut_for_key, default_fill_color, default_outline_color,
        deletion_motion_for_key, drag_selection_pointer_position_with_scroll, resolve_border_width,
        should_expose_submit_accessibility_action, single_line_key_behavior,
        text_content_origin_from_values, text_input_accessibility_role,
        text_viewport_size_from_origin,
    };

    fn plan_ime_event_for_state(
        selection: TextSelection,
        composition: Option<ImeComposition>,
        single_line: bool,
        event: &winit::event::Ime,
    ) -> Option<PlannedImeEvent> {
        let text_len = selection.ordered_range().end.max(
            composition
                .as_ref()
                .map(|composition| composition.range.end)
                .unwrap_or(0),
        );
        let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
        controller.set_text_and_selection(&"x".repeat(text_len), selection);
        controller.set_composition(composition);
        controller.plan_ime_event(single_line, event)
    }

    fn pointer_position_for_offset(text: &str, offset: usize) -> PxPosition {
        let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
        controller.set_text_and_selection(text, TextSelection::collapsed(offset));
        let (x, y) = controller
            .editor()
            .cursor_position()
            .expect("cursor position should be available");
        PxPosition::new(Px(x), Px(y))
    }

    #[test]
    fn drag_selection_pointer_position_clamps_and_reports_overflow() {
        let text_origin = text_content_origin_from_values(Px(8), Px(1));
        let size = ComputedData {
            width: Px(120),
            height: Px(40),
        };

        let drag_target = drag_selection_pointer_position_with_scroll(
            PxPosition::new(Px(-4), Px(60)),
            size,
            text_origin,
            Px::ZERO,
            Px::ZERO,
        );

        assert_eq!(
            drag_target,
            DragSelectionPointerPosition {
                position: PxPosition::new(Px::ZERO, Px(22)),
                overflow_x: Px(-13),
                overflow_y: Px(29),
            }
        );
    }

    #[test]
    fn drag_selection_pointer_position_applies_vertical_scroll_to_clamped_position() {
        let text_origin = text_content_origin_from_values(Px(8), Px(1));
        let size = ComputedData {
            width: Px(120),
            height: Px(40),
        };

        let drag_target = drag_selection_pointer_position_with_scroll(
            PxPosition::new(Px(20), Px(20)),
            size,
            text_origin,
            Px(5),
            Px(11),
        );

        assert_eq!(
            drag_target,
            DragSelectionPointerPosition {
                position: PxPosition::new(Px(16), Px(22)),
                overflow_x: Px::ZERO,
                overflow_y: Px::ZERO,
            }
        );
    }

    #[test]
    fn click_selection_pointer_position_clamps_inside_text_viewport() {
        let text_origin = text_content_origin_from_values(Px(8), Px(1));
        let size = ComputedData {
            width: Px(120),
            height: Px(40),
        };

        let click_position = drag_selection_pointer_position_with_scroll(
            PxPosition::new(Px(2), Px(4)),
            size,
            text_origin,
            Px::ZERO,
            Px::ZERO,
        )
        .position;

        assert_eq!(click_position, PxPosition::new(Px::ZERO, Px::ZERO));
    }

    #[test]
    fn text_viewport_size_removes_padding_and_border_from_surface() {
        let viewport = text_viewport_size_from_origin(
            ComputedData {
                width: Px(120),
                height: Px(40),
            },
            text_content_origin_from_values(Px(8), Px(1)),
        );

        assert_eq!(viewport.width, Px(102));
        assert_eq!(viewport.height, Px(22));
    }

    #[test]
    fn plan_ime_edit_preedit_reuses_existing_composition_range() {
        let composition = ImeComposition {
            range: 2..5,
            selection: TextSelection::collapsed(5),
        };

        assert_eq!(
            plan_ime_event_for_state(
                TextSelection::collapsed(5),
                Some(composition),
                false,
                &winit::event::Ime::Preedit("xy".to_string(), Some((1, 1))),
            ),
            Some(PlannedImeEvent::Edit(PlannedImeEdit {
                replacement_range: 2..5,
                replacement_text: "xy".to_string(),
                selection: TextSelection::collapsed(3),
                composition_range: Some(2..4),
            }))
        );
    }

    #[test]
    fn plan_ime_edit_empty_preedit_clears_composition_range() {
        let composition = ImeComposition {
            range: 2..5,
            selection: TextSelection::collapsed(5),
        };

        assert_eq!(
            plan_ime_event_for_state(
                TextSelection::collapsed(5),
                Some(composition),
                false,
                &winit::event::Ime::Preedit(String::new(), None),
            ),
            Some(PlannedImeEvent::Edit(PlannedImeEdit {
                replacement_range: 2..5,
                replacement_text: String::new(),
                selection: TextSelection::collapsed(2),
                composition_range: None,
            }))
        );
    }

    #[test]
    fn plan_ime_edit_preedit_without_cursor_offset_preserves_previous_composition_selection() {
        let composition = ImeComposition {
            range: 2..5,
            selection: TextSelection { start: 3, end: 4 },
        };

        assert_eq!(
            plan_ime_event_for_state(
                TextSelection::collapsed(5),
                Some(composition),
                false,
                &winit::event::Ime::Preedit("xyzq".to_string(), None),
            ),
            Some(PlannedImeEvent::Edit(PlannedImeEdit {
                replacement_range: 2..5,
                replacement_text: "xyzq".to_string(),
                selection: TextSelection { start: 3, end: 4 },
                composition_range: Some(2..6),
            }))
        );
    }

    #[test]
    fn plan_ime_edit_commit_replaces_current_composition_range() {
        let composition = ImeComposition {
            range: 3..6,
            selection: TextSelection::collapsed(6),
        };

        assert_eq!(
            plan_ime_event_for_state(
                TextSelection::collapsed(6),
                Some(composition),
                false,
                &winit::event::Ime::Commit("z".to_string()),
            ),
            Some(PlannedImeEvent::Edit(PlannedImeEdit {
                replacement_range: 3..6,
                replacement_text: "z".to_string(),
                selection: TextSelection::collapsed(4),
                composition_range: None,
            }))
        );
    }

    #[test]
    fn plan_ime_edit_commit_without_composition_replaces_current_selection_range() {
        assert_eq!(
            plan_ime_event_for_state(
                TextSelection { start: 6, end: 3 },
                None,
                false,
                &winit::event::Ime::Commit("z".to_string()),
            ),
            Some(PlannedImeEvent::Edit(PlannedImeEdit {
                replacement_range: 3..6,
                replacement_text: "z".to_string(),
                selection: TextSelection::collapsed(4),
                composition_range: None,
            }))
        );
    }

    #[test]
    fn plan_ime_edit_preedit_tracks_full_composition_range_instead_of_caret_range() {
        assert_eq!(
            plan_ime_event_for_state(
                TextSelection::collapsed(1),
                None,
                false,
                &winit::event::Ime::Preedit("abcd".to_string(), Some((2, 2))),
            ),
            Some(PlannedImeEvent::Edit(PlannedImeEdit {
                replacement_range: 1..1,
                replacement_text: "abcd".to_string(),
                selection: TextSelection::collapsed(3),
                composition_range: Some(1..5),
            }))
        );
    }

    #[test]
    fn text_input_pointer_sequence_single_click_drag_selects_backward_range() {
        let text = "hello world";
        let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
        controller.set_text_and_selection(text, TextSelection::collapsed(6));
        let start_position = pointer_position_for_offset(text, 6);
        let drag_position = pointer_position_for_offset(text, 1);
        let click_time = Instant::now();

        let click_type = controller.handle_click(start_position, start_position, click_time);
        assert!(matches!(click_type, ClickType::Single));
        controller.apply_pointer_action(GlyphonAction::Click {
            x: start_position.x_px().0,
            y: start_position.y_px().0,
        });
        controller.start_drag(click_type);
        controller.apply_pointer_action(GlyphonAction::Drag {
            x: drag_position.x_px().0,
            y: drag_position.y_px().0,
        });

        assert_eq!(controller.selection(), TextSelection { start: 6, end: 1 });
        assert_eq!(controller.cursor_offset(), 1);
    }

    #[test]
    fn text_input_pointer_sequence_double_click_drag_selects_word_range() {
        let text = "foo bar baz";
        let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
        controller.set_text_and_selection(text, TextSelection::collapsed(5));
        let first_position = pointer_position_for_offset(text, 5);
        let drag_position = pointer_position_for_offset(text, 9);
        let first_click_time = Instant::now();
        let second_click_time = first_click_time + Duration::from_millis(100);

        let first_click_type =
            controller.handle_click(first_position, first_position, first_click_time);
        assert!(matches!(first_click_type, ClickType::Single));
        controller.apply_pointer_action(GlyphonAction::Click {
            x: first_position.x_px().0,
            y: first_position.y_px().0,
        });

        let second_click_type =
            controller.handle_click(first_position, first_position, second_click_time);
        assert!(matches!(second_click_type, ClickType::Double));
        controller.apply_pointer_action(GlyphonAction::DoubleClick {
            x: first_position.x_px().0,
            y: first_position.y_px().0,
        });
        controller.start_drag(second_click_type);
        controller.apply_pointer_action(GlyphonAction::Drag {
            x: drag_position.x_px().0,
            y: drag_position.y_px().0,
        });

        assert_eq!(controller.selection(), TextSelection { start: 4, end: 11 });
        assert_eq!(controller.cursor_offset(), 11);
    }

    #[test]
    fn text_input_pointer_sequence_triple_click_drag_selects_line_range() {
        let text = "111\n222\n333";
        let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
        controller.set_text_and_selection(text, TextSelection::collapsed(5));
        let line_position = pointer_position_for_offset(text, 5);
        let drag_position = pointer_position_for_offset(text, 9);
        let first_click_time = Instant::now();
        let second_click_time = first_click_time + Duration::from_millis(100);
        let third_click_time = second_click_time + Duration::from_millis(100);

        let first_click_type =
            controller.handle_click(line_position, line_position, first_click_time);
        assert!(matches!(first_click_type, ClickType::Single));
        controller.apply_pointer_action(GlyphonAction::Click {
            x: line_position.x_px().0,
            y: line_position.y_px().0,
        });

        let second_click_type =
            controller.handle_click(line_position, line_position, second_click_time);
        assert!(matches!(second_click_type, ClickType::Double));
        controller.apply_pointer_action(GlyphonAction::DoubleClick {
            x: line_position.x_px().0,
            y: line_position.y_px().0,
        });

        let third_click_type =
            controller.handle_click(line_position, line_position, third_click_time);
        assert!(matches!(third_click_type, ClickType::Triple));
        controller.apply_pointer_action(GlyphonAction::TripleClick {
            x: line_position.x_px().0,
            y: line_position.y_px().0,
        });
        controller.start_drag(third_click_type);
        controller.apply_pointer_action(GlyphonAction::Drag {
            x: drag_position.x_px().0,
            y: drag_position.y_px().0,
        });

        assert_eq!(controller.selection(), TextSelection { start: 4, end: 11 });
        assert_eq!(controller.cursor_offset(), 11);
    }

    #[test]
    fn single_line_enter_uses_submit_behavior() {
        assert_eq!(
            single_line_key_behavior(
                true,
                winit::event::ElementState::Pressed,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
            ),
            Some(SingleLineKeyBehavior::Submit)
        );
    }

    #[test]
    fn single_line_tab_propagates_focus_navigation() {
        assert_eq!(
            single_line_key_behavior(
                true,
                winit::event::ElementState::Pressed,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab),
            ),
            Some(SingleLineKeyBehavior::Propagate)
        );
    }

    #[test]
    fn multi_line_enter_keeps_default_editing_behavior() {
        assert_eq!(
            single_line_key_behavior(
                false,
                winit::event::ElementState::Pressed,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
            ),
            None
        );
    }

    #[test]
    fn released_enter_does_not_trigger_single_line_submit_behavior() {
        assert_eq!(
            single_line_key_behavior(
                true,
                winit::event::ElementState::Released,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
            ),
            None
        );
    }

    #[test]
    fn single_line_ime_commit_newline_uses_submit_behavior() {
        assert_eq!(
            plan_ime_event_for_state(
                TextSelection::collapsed(0),
                None,
                true,
                &winit::event::Ime::Commit("\n".to_string()),
            ),
            Some(PlannedImeEvent::Submit)
        );
        assert_eq!(
            plan_ime_event_for_state(
                TextSelection::collapsed(0),
                None,
                true,
                &winit::event::Ime::Commit("\r\n".to_string()),
            ),
            Some(PlannedImeEvent::Submit)
        );
    }

    #[test]
    fn multi_line_ime_commit_newline_keeps_default_editing_behavior() {
        assert_eq!(
            plan_ime_event_for_state(
                TextSelection::collapsed(0),
                None,
                false,
                &winit::event::Ime::Commit("\n".to_string()),
            ),
            Some(PlannedImeEvent::Edit(PlannedImeEdit {
                replacement_range: 0..0,
                replacement_text: "\n".to_string(),
                selection: TextSelection::collapsed(1),
                composition_range: None,
            }))
        );
    }

    #[test]
    fn single_line_ime_commit_text_does_not_trigger_submit_behavior() {
        assert_eq!(
            plan_ime_event_for_state(
                TextSelection::collapsed(0),
                None,
                true,
                &winit::event::Ime::Commit("done".to_string()),
            ),
            Some(PlannedImeEvent::Edit(PlannedImeEdit {
                replacement_range: 0..0,
                replacement_text: "done".to_string(),
                selection: TextSelection::collapsed(4),
                composition_range: None,
            }))
        );
    }

    #[test]
    fn non_submit_single_line_commit_keeps_edit_plan_instead_of_submit_plan() {
        assert!(matches!(
            plan_ime_event_for_state(
                TextSelection::collapsed(0),
                Some(ImeComposition {
                    range: 1..4,
                    selection: TextSelection::collapsed(3),
                }),
                true,
                &winit::event::Ime::Commit("done".to_string()),
            ),
            Some(PlannedImeEvent::Edit(_))
        ));
    }

    #[test]
    fn ctrl_backspace_uses_left_word_deletion_motion() {
        assert_eq!(
            deletion_motion_for_key(
                true,
                winit::event::ElementState::Pressed,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace),
            ),
            Some(glyphon::cosmic_text::Motion::LeftWord)
        );
    }

    #[test]
    fn ctrl_delete_uses_right_word_deletion_motion() {
        assert_eq!(
            deletion_motion_for_key(
                true,
                winit::event::ElementState::Pressed,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete),
            ),
            Some(glyphon::cosmic_text::Motion::RightWord)
        );
    }

    #[test]
    fn ctrl_insert_uses_copy_clipboard_shortcut_behavior() {
        assert_eq!(
            clipboard_shortcut_for_key(
                true,
                false,
                winit::event::ElementState::Pressed,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert),
            ),
            Some(ClipboardShortcutBehavior::Copy)
        );
    }

    #[test]
    fn shift_insert_uses_paste_clipboard_shortcut_behavior() {
        assert_eq!(
            clipboard_shortcut_for_key(
                false,
                true,
                winit::event::ElementState::Pressed,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert),
            ),
            Some(ClipboardShortcutBehavior::Paste)
        );
    }

    #[test]
    fn shift_delete_uses_cut_clipboard_shortcut_behavior() {
        assert_eq!(
            clipboard_shortcut_for_key(
                false,
                true,
                winit::event::ElementState::Pressed,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete),
            ),
            Some(ClipboardShortcutBehavior::Cut)
        );
    }

    #[test]
    fn released_insert_does_not_trigger_clipboard_shortcut_behavior() {
        assert_eq!(
            clipboard_shortcut_for_key(
                true,
                false,
                winit::event::ElementState::Released,
                &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Insert),
            ),
            None
        );
    }

    #[test]
    fn single_line_accessibility_uses_text_input_role() {
        assert_eq!(text_input_accessibility_role(true), Role::TextInput);
    }

    #[test]
    fn multi_line_accessibility_uses_multiline_role() {
        assert_eq!(
            text_input_accessibility_role(false),
            Role::MultilineTextInput
        );
    }

    #[test]
    fn submit_accessibility_action_requires_focused_single_line_handler() {
        assert!(should_expose_submit_accessibility_action(true, true, true,));
        assert!(!should_expose_submit_accessibility_action(
            false, true, true,
        ));
        assert!(!should_expose_submit_accessibility_action(
            true, false, true,
        ));
        assert!(!should_expose_submit_accessibility_action(
            true, true, false
        ));
    }

    #[test]
    fn build_ime_request_uses_active_ime_rect_when_available() {
        let ime_request = build_ime_request(
            PxPosition::new(Px(9), Px(11)),
            ComputedData {
                width: Px(120),
                height: Px(40),
            },
            Some(RectDef {
                x: Px(3),
                y: Px(5),
                width: Px(7),
                height: Px(13),
            }),
            Some(2..6),
            Some(3..5),
        );

        assert_eq!(ime_request.local_position, PxPosition::new(Px(12), Px(16)));
        assert_eq!(ime_request.size, PxSize::new(Px(7), Px(13)));
        assert_eq!(ime_request.selection_range, Some(2..6));
        assert_eq!(ime_request.composition_range, Some(3..5));
    }

    #[test]
    fn build_ime_request_falls_back_to_component_bounds_without_active_rect() {
        let ime_request = build_ime_request(
            PxPosition::new(Px(9), Px(11)),
            ComputedData {
                width: Px(120),
                height: Px(40),
            },
            None,
            Some(2..6),
            None,
        );

        assert_eq!(ime_request.local_position, PxPosition::ZERO);
        assert_eq!(ime_request.size, PxSize::new(Px(120), Px(40)));
        assert_eq!(ime_request.selection_range, Some(2..6));
        assert_eq!(ime_request.composition_range, None);
    }

    fn scheme() -> crate::theme::MaterialColorScheme {
        crate::theme::MaterialColorScheme::default()
    }

    fn resting_state() -> FieldVisualState {
        FieldVisualState {
            focused: false,
            enabled: true,
            is_error: false,
            outlined: true,
        }
    }

    #[test]
    fn outline_color_follows_the_material_states() {
        let scheme = scheme();
        let resting = resting_state();

        assert_eq!(default_outline_color(&scheme, resting), scheme.outline);
        assert_eq!(
            default_outline_color(
                &scheme,
                FieldVisualState {
                    focused: true,
                    ..resting
                }
            ),
            scheme.primary
        );
        assert_eq!(
            default_outline_color(
                &scheme,
                FieldVisualState {
                    is_error: true,
                    ..resting
                }
            ),
            scheme.error
        );
        assert_eq!(
            default_outline_color(
                &scheme,
                FieldVisualState {
                    enabled: false,
                    focused: true,
                    ..resting
                }
            ),
            scheme.on_surface.with_alpha(DISABLED_OUTLINE_ALPHA)
        );
    }

    #[test]
    fn disabled_fill_depends_on_the_variant() {
        let scheme = scheme();
        let disabled = FieldVisualState {
            enabled: false,
            ..resting_state()
        };

        assert_eq!(
            default_fill_color(
                &scheme,
                FieldVisualState {
                    outlined: false,
                    ..disabled
                }
            ),
            scheme.on_surface.with_alpha(DISABLED_FILL_ALPHA)
        );
        assert_eq!(default_fill_color(&scheme, disabled), Color::TRANSPARENT);
        assert_eq!(
            default_fill_color(&scheme, resting_state()),
            Color::TRANSPARENT
        );
    }

    #[test]
    fn disabled_field_keeps_the_resting_outline_width() {
        assert_eq!(
            resolve_border_width(Dp(1.0), Some(Dp(2.0)), true, true),
            Dp(2.0)
        );
        assert_eq!(
            resolve_border_width(Dp(1.0), Some(Dp(2.0)), false, true),
            Dp(1.0)
        );
    }

    mod scroll_propagation {
        use std::cell::RefCell;

        use tessera_ui::{
            CursorEventContent, Dp, GestureState, Modifier, PointerChange, PressKeyEventType, Px,
            PxPosition, ScrollDeltaUnit, ScrollEventContent, ScrollEventSource, State,
            layout::layout,
            remember, tessera,
            testing::{
                layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
                start_layout_test_session,
            },
            time::Instant,
        };

        use crate::{
            column::column,
            modifier::ModifierExt as _,
            scrollable::ScrollableController,
            spacer::spacer,
            text_field::text_field,
            text_input::text_input,
            theme::{MaterialTheme, material_theme},
        };

        thread_local! {
            static CONTROLLER: RefCell<Option<State<ScrollableController>>> =
                const { RefCell::new(None) };
        }

        #[tessera]
        fn scroll_probe_with_input() {
            let controller = remember(ScrollableController::new);
            CONTROLLER.with(|slot| *slot.borrow_mut() = Some(controller));
            material_theme()
                .theme(|| MaterialTheme::default())
                .child(move || {
                    layout()
                        .modifier(
                            Modifier::new()
                                .width(Dp(200.0))
                                .height(Dp(100.0))
                                .vertical_scroll(controller),
                        )
                        .child(move || {
                            column().children(move || {
                                text_input();
                                spacer().modifier(Modifier::new().height(Dp(400.0)));
                            });
                        });
                });
        }

        #[tessera]
        fn scroll_probe_plain() {
            let controller = remember(ScrollableController::new);
            CONTROLLER.with(|slot| *slot.borrow_mut() = Some(controller));
            material_theme()
                .theme(|| MaterialTheme::default())
                .child(move || {
                    layout()
                        .modifier(
                            Modifier::new()
                                .width(Dp(200.0))
                                .height(Dp(100.0))
                                .vertical_scroll(controller),
                        )
                        .child(move || {
                            column().children(move || {
                                spacer().modifier(Modifier::new().height(Dp(56.0)));
                                spacer().modifier(Modifier::new().height(Dp(400.0)));
                            });
                        });
                });
        }

        fn scrolled_after_wheel(content: impl Fn(), focus_first: bool) -> f32 {
            let mut session = start_layout_test_session(layout_test(content).viewport_px(200, 100));
            let _ = pump_layout_test_session(&mut session, 0);
            let controller =
                CONTROLLER.with(|slot| (*slot.borrow()).expect("scrollable controller"));
            let cursor = Some(PxPosition::new(Px::new(100), Px::new(20)));

            if focus_first {
                let press = PointerChange::new(
                    Instant::now(),
                    1,
                    CursorEventContent::Pressed(PressKeyEventType::Left),
                    GestureState::TapCandidate,
                );
                let _ = pump_layout_test_session_with_input(
                    &mut session,
                    16_666_667,
                    vec![press],
                    cursor,
                );
                let release = PointerChange::new(
                    Instant::now(),
                    1,
                    CursorEventContent::Released(PressKeyEventType::Left),
                    GestureState::TapCandidate,
                );
                let _ = pump_layout_test_session_with_input(
                    &mut session,
                    33_333_334,
                    vec![release],
                    cursor,
                );
            }

            let scroll = PointerChange::new(
                Instant::now(),
                1,
                CursorEventContent::Scroll(ScrollEventContent {
                    delta_x: 0.0,
                    delta_y: -40.0,
                    unit: ScrollDeltaUnit::Pixel,
                    source: ScrollEventSource::Wheel,
                }),
                GestureState::TapCandidate,
            );

            let _ =
                pump_layout_test_session_with_input(&mut session, 50_000_000, vec![scroll], cursor);

            for frame in 4..40u64 {
                let _ = pump_layout_test_session(&mut session, 16_666_667 * frame);
            }

            controller.with(|c| c.child_position().y)
        }

        #[test]
        fn parent_scrollable_scrolls_without_text_input() {
            let offset = scrolled_after_wheel(
                || {
                    scroll_probe_plain();
                },
                false,
            );
            assert!(
                offset != 0.0,
                "plain content should scroll, got y={offset:?}"
            );
        }

        #[test]
        fn wheel_over_unfocused_input_reaches_parent_scrollable() {
            let offset = scrolled_after_wheel(
                || {
                    scroll_probe_with_input();
                },
                false,
            );
            assert!(
                offset != 0.0,
                "wheel over an unfocused text input should scroll the parent, got y={offset:?}"
            );
        }

        #[test]
        fn wheel_over_focused_input_reaches_parent_scrollable() {
            let offset = scrolled_after_wheel(
                || {
                    scroll_probe_with_input();
                },
                true,
            );
            assert!(
                offset != 0.0,
                "wheel over a focused text input should scroll the parent, got y={offset:?}"
            );
        }

        #[tessera]
        fn scroll_probe_with_field() {
            let controller = remember(ScrollableController::new);
            CONTROLLER.with(|slot| *slot.borrow_mut() = Some(controller));
            material_theme()
                .theme(|| MaterialTheme::default())
                .child(move || {
                    layout()
                        .modifier(
                            Modifier::new()
                                .width(Dp(200.0))
                                .height(Dp(100.0))
                                .vertical_scroll(controller),
                        )
                        .child(move || {
                            column().children(move || {
                                text_field();
                                spacer().modifier(Modifier::new().height(Dp(400.0)));
                            });
                        });
                });
        }

        #[test]
        fn wheel_over_text_field_reaches_parent_scrollable() {
            let offset = scrolled_after_wheel(
                || {
                    scroll_probe_with_field();
                },
                false,
            );
            assert!(
                offset != 0.0,
                "wheel over a text field should scroll the parent, got y={offset:?}"
            );
        }
    }
}
