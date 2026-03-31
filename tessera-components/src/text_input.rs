//! Core text input component without Material decoration.
//!
//! ## Usage
//!
//! Embed as a bare text input surface when you need to build custom styling.
use glyphon::Action as GlyphonAction;
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, CallbackWith, Color, ComputedData, Dp,
    ImeInput, ImeInputModifierNode, ImeRequest, KeyboardInput, KeyboardInputModifierNode, Modifier,
    PointerInput, PointerInputModifierNode, Px, PxPosition, PxSize, SemanticsModifierNode, State,
    accesskit::{Action, Role},
    layout::layout_primitive,
    modifier::{CursorModifierExt as _, FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context, winit,
};

use crate::{
    gesture_recognizer::{ScrollRecognizer, ScrollResult, TapRecognizer},
    modifier::ModifierExt as _,
    pos_misc::is_position_inside_bounds,
    shape_def::{RoundedCorner, Shape},
    surface::surface,
    text_edit_core::{
        ClickType, ImeEditResult, PlannedImeEvent, RectDef, TextSelection, text_edit_core,
    },
    theme::{MaterialTheme, TextSelectionColors},
};

#[cfg(test)]
use glyphon::Edit;

/// State structure for the text input, managing text content, cursor,
/// selection, and editing logic.
pub use crate::text_edit_core::{
    DisplayTransform, TextEditorController as TextInputController,
    TransformedText as DisplayTransformText,
};

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
        .push_ime_input(TextInputImeModifierNode { args, controller })
}

#[derive(Clone, PartialEq)]
pub(crate) struct TextInputProps {
    /// Whether the editor is enabled for user input.
    pub enabled: bool,
    /// Whether the editor is read-only.
    pub read_only: bool,
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
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        Self {
            enabled: true,
            read_only: false,
            modifier: Modifier::new(),
            on_change: CallbackWith::default_value(),
            on_submit: Callback::noop(),
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
            single_line: false,
            input_transform: None,
            display_transform: None,
            controller: None,
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
/// Renders a multi-line, editable text field.
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
///
/// text_input().padding(Dp(8.0)).initial_text("Hello World");
/// # }
/// # component();
/// ```
#[tessera]
pub fn text_input(
    #[default(true)] enabled: bool,
    read_only: bool,
    modifier: Modifier,
    on_change: Option<CallbackWith<String, String>>,
    on_submit: Option<Callback>,
    min_width: Option<Dp>,
    min_height: Option<Dp>,
    #[default(TextInputProps::default().background_color)] background_color: Option<Color>,
    #[default(TextInputProps::default().border_width)] border_width: Dp,
    #[default(TextInputProps::default().border_color)] border_color: Option<Color>,
    #[default(TextInputProps::default().shape)] shape: Shape,
    #[default(TextInputProps::default().padding)] padding: Dp,
    #[default(TextInputProps::default().focus_border_color)] focus_border_color: Option<Color>,
    focus_border_width: Option<Dp>,
    #[default(TextInputProps::default().focus_background_color)] focus_background_color: Option<
        Color,
    >,
    #[default(TextInputProps::default().selection_color)] selection_color: Option<Color>,
    #[default(TextInputProps::default().text_color)] text_color: Option<Color>,
    #[default(TextInputProps::default().cursor_color)] cursor_color: Option<Color>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    #[prop(into)] initial_text: Option<String>,
    #[default(TextInputProps::default().font_size)] font_size: Dp,
    line_height: Option<Dp>,
    single_line: bool,
    input_transform: Option<CallbackWith<String, String>>,
    display_transform: Option<DisplayTransform>,
    controller: Option<State<TextInputController>>,
) {
    let args = TextInputProps {
        enabled,
        read_only,
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

    layout_primitive().modifier(modifier).child(move || {
        let surface_args = editor_args.clone();
        create_surface_args(&surface_args, &controller).with_child(move || {
            text_input_padded_content()
                .padding(surface_args.padding)
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

    layout_primitive().modifier(modifier).child(move || {
        text_input_padded_content()
            .padding(editor_args.padding)
            .controller(controller);
    });
}

#[tessera]
fn text_input_padded_content(padding: Dp, controller: Option<State<TextInputController>>) {
    let controller = controller.expect("text_input_padded_content requires controller to be set");
    layout_primitive()
        .modifier(Modifier::new().padding_all(padding))
        .child(move || {
            text_edit_core().controller(controller);
        });
}

pub(crate) fn text_input_core(args: &TextInputProps, controller: State<TextInputController>) {
    let mut core_args = args.clone();
    core_args.controller = Some(controller);
    text_input_editor().props(core_args);
}

fn sync_text_input_controller(controller: &State<TextInputController>, args: &TextInputProps) {
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
            input.cursor_position_rel,
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
                let click_type =
                    controller.with_mut(|s| s.handle_click(text_relative_pos, click_timestamp));

                match click_type {
                    ClickType::Single => {
                        if input.key_modifiers.shift_key() {
                            controller.with_mut(|s| {
                                s.extend_selection_to_point(text_relative_pos);
                            });
                        } else {
                            controller.with_mut(|s| {
                                s.apply_pointer_action(GlyphonAction::Click {
                                    x: text_relative_pos.x.0,
                                    y: text_relative_pos.y.0,
                                });
                            });
                        }
                    }
                    ClickType::Double => {
                        controller.with_mut(|s| {
                            s.apply_pointer_action(GlyphonAction::DoubleClick {
                                x: text_relative_pos.x.0,
                                y: text_relative_pos.y.0,
                            });
                        });
                    }
                    ClickType::Triple => {
                        controller.with_mut(|s| {
                            s.apply_pointer_action(GlyphonAction::TripleClick {
                                x: text_relative_pos.x.0,
                                y: text_relative_pos.y.0,
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
                        x: current_pos_px.x.0,
                        y: current_pos_px.y.0,
                    });
                });

                // Update last position to current position
                controller.with_mut(|s| {
                    s.update_last_click_position(current_pos_px);
                });
            }
        }

        let scroll_result = scroll_recognizer.with_mut(|recognizer| {
            recognizer.update(input.pass, input.pointer_changes.as_mut_slice())
        });

        // Handle scroll events (only when focused and cursor is in editor)
        if controller.with(|s| s.focus_handler().is_focused()) && scroll_result.has_scroll() {
            controller.with_mut(|s| {
                if s.single_line() {
                    s.scroll_horizontal_by(single_line_scroll_delta(&scroll_result));
                    return;
                }
                s.scroll_vertical_by(-scroll_result.delta_y);
            });
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    let text_relative_x_px = cursor_pos.x - text_origin.x;
    let text_relative_y_px = cursor_pos.y - text_origin.y;
    let overflow_x = if text_relative_x_px < Px::ZERO {
        text_relative_x_px
    } else if text_relative_x_px > viewport_size.width {
        text_relative_x_px - viewport_size.width
    } else {
        Px::ZERO
    };
    let overflow_y = if text_relative_y_px < Px::ZERO {
        text_relative_y_px
    } else if text_relative_y_px > viewport_size.height {
        text_relative_y_px - viewport_size.height
    } else {
        Px::ZERO
    };
    let clamped_x = text_relative_x_px.max(Px::ZERO).min(viewport_size.width);
    let clamped_y = text_relative_y_px.max(Px::ZERO).min(viewport_size.height);

    DragSelectionPointerPosition {
        position: PxPosition::new(clamped_x + horizontal_scroll, clamped_y + vertical_scroll),
        overflow_x,
        overflow_y,
    }
}

fn text_viewport_size_from_origin(size: ComputedData, text_origin: PxPosition) -> ComputedData {
    ComputedData {
        width: (size.width - text_origin.x - text_origin.x).max(Px::ZERO),
        height: (size.height - text_origin.y - text_origin.y).max(Px::ZERO),
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

fn single_line_scroll_delta(scroll_result: &ScrollResult) -> f32 {
    if scroll_result.delta_x.abs() > f32::EPSILON {
        -scroll_result.delta_x
    } else {
        -scroll_result.delta_y
    }
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

    let ime_events: Vec<_> = input.ime_events.drain(..).collect();
    for event in ime_events {
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

#[cfg(test)]
fn editor_selection(editor: &glyphon::Editor<'_>) -> TextSelection {
    match editor.selection() {
        glyphon::cosmic_text::Selection::None => {
            let cursor = editor.cursor();
            let offset = get_text_offset(editor, cursor);
            TextSelection::collapsed(offset)
        }
        glyphon::cosmic_text::Selection::Normal(anchor)
        | glyphon::cosmic_text::Selection::Line(anchor)
        | glyphon::cosmic_text::Selection::Word(anchor) => TextSelection {
            start: get_text_offset(editor, anchor),
            end: get_text_offset(editor, editor.cursor()),
        },
    }
}

#[cfg(test)]
fn get_text_offset(editor: &glyphon::Editor<'_>, cursor: glyphon::Cursor) -> usize {
    editor.with_buffer(|buffer| {
        let mut offset = 0usize;
        for (line_index, line) in buffer.lines.iter().enumerate() {
            let line_len = line.text().len();
            if line_index == cursor.line {
                return offset + cursor.index.min(line_len);
            }
            offset += line_len + line.ending().as_str().len();
        }
        offset
    })
}

#[cfg(test)]
fn rebase_selection(before: &str, after: &str, selection: TextSelection) -> TextSelection {
    TextSelection {
        start: rebase_offset(before, after, selection.start),
        end: rebase_offset(before, after, selection.end),
    }
}

#[cfg(test)]
fn rebase_range(
    before: &str,
    after: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let start = rebase_offset(before, after, range.start);
    let end = rebase_offset(before, after, range.end);
    start.min(end)..start.max(end)
}

#[cfg(test)]
fn rebase_offset(before: &str, after: &str, offset: usize) -> usize {
    DisplayTransformText::from_strings(before, after.to_string())
        .map_from_raw(offset.min(before.len()))
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
    args: &TextInputProps,
    state: &State<TextInputController>,
) -> crate::surface::SurfaceBuilder {
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

    let mut modifier = args.modifier.clone();
    if args.min_width.is_some() || args.min_height.is_some() {
        modifier = modifier.size_in(args.min_width, None, args.min_height, None);
    }

    surface()
        .style(style)
        .shape(args.shape)
        .block_input(!args.enabled)
        .modifier(modifier)
}

/// Determine background color based on focus state
fn determine_background_color(args: &TextInputProps, state: &State<TextInputController>) -> Color {
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
    args: &TextInputProps,
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

fn determine_border_width(args: &TextInputProps, state: &State<TextInputController>) -> Dp {
    if state.with(|c| c.focus_handler().is_focused()) {
        args.focus_border_width.unwrap_or(args.border_width)
    } else {
        args.border_width
    }
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

/// Builder methods for fluent API
impl TextInputBuilder {
    /// Sets the minimum width in Dp.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputBuilder;
    /// use tessera_ui::Dp;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple().with_min_width(Dp(80.0));
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn with_min_width(self, min_width: Dp) -> Self {
        self.min_width(min_width)
    }

    /// Sets the minimum height in Dp.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputBuilder;
    /// use tessera_ui::Dp;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple().with_min_height(Dp(40.0));
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn with_min_height(self, min_height: Dp) -> Self {
        self.min_height(min_height)
    }

    /// Sets the background color.
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputBuilder;
    /// use tessera_ui::Color;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple().with_background_color(Color::WHITE);
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn with_background_color(self, color: Color) -> Self {
        self.background_color(color)
    }

    /// Sets the border width in pixels.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputBuilder;
    /// use tessera_ui::Dp;
    ///
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple().with_border_width(Dp(1.0));
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn with_border_width(self, width: Dp) -> Self {
        self.border_width(width)
    }

    /// Sets the border color.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputBuilder;
    /// use tessera_ui::Color;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple().with_border_color(Color::BLACK);
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn with_border_color(self, color: Color) -> Self {
        self.border_color(color)
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
    /// use tessera_components::text_input::TextInputBuilder;
    /// use tessera_ui::Dp;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple().with_shape(Shape::RoundedRectangle {
    ///     top_left: RoundedCorner::manual(Dp(8.0), 3.0),
    ///     top_right: RoundedCorner::manual(Dp(8.0), 3.0),
    ///     bottom_right: RoundedCorner::manual(Dp(8.0), 3.0),
    ///     bottom_left: RoundedCorner::manual(Dp(8.0), 3.0),
    /// });
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn with_shape(self, shape: Shape) -> Self {
        self.shape(shape)
    }

    /// Sets the inner padding in Dp.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputBuilder;
    /// use tessera_ui::Dp;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple().with_padding(Dp(12.0));
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn with_padding(self, padding: Dp) -> Self {
        self.padding(padding)
    }

    /// Sets the border color when focused.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputBuilder;
    /// use tessera_ui::Color;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple().with_focus_border_color(Color::new(0.0, 0.5, 1.0, 1.0));
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn with_focus_border_color(self, color: Color) -> Self {
        self.focus_border_color(color)
    }

    /// Sets the background color when focused.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputBuilder;
    /// use tessera_ui::Color;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple().with_focus_background_color(Color::WHITE);
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn with_focus_background_color(self, color: Color) -> Self {
        self.focus_background_color(color)
    }

    /// Sets the selection highlight color.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::text_input::TextInputBuilder;
    /// use tessera_ui::Color;
    /// # use tessera_components::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = TextInputBuilder::simple().with_selection_color(Color::new(0.5, 0.7, 1.0, 0.4));
    /// # });
    /// # }
    /// # component();
    /// ```
    pub fn with_selection_color(self, color: Color) -> Self {
        self.selection_color(color)
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
    accessibility.role = Some(text_input_accessibility_role(args.single_line));
    accessibility.disabled = !args.enabled;
    accessibility.label = args.accessibility_label.clone();
    accessibility.description = args.accessibility_description.clone();

    let current_text = state.with(|c| c.text());
    accessibility.value = (!current_text.is_empty()).then_some(current_text);

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

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use glyphon::{Action as GlyphonAction, Edit as _};
    use tessera_ui::{ComputedData, Px, PxPosition, PxSize, accesskit::Role, winit};

    use crate::text_edit_core::{
        ClickType, ImeComposition, PlannedImeEdit, PlannedImeEvent, RectDef, TextEditorController,
        TextSelection,
    };

    use super::{
        ClipboardShortcutBehavior, DragSelectionPointerPosition, SingleLineKeyBehavior,
        build_ime_request, clipboard_shortcut_for_key, deletion_motion_for_key,
        drag_selection_pointer_position_with_scroll, editor_selection, rebase_offset, rebase_range,
        rebase_selection, should_expose_submit_accessibility_action, single_line_key_behavior,
        text_content_origin_from_values, text_input_accessibility_role,
        text_viewport_size_from_origin,
    };

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct SimulatedImeState {
        text: String,
        selection: TextSelection,
        composition: Option<ImeComposition>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct SimulatedTextInputSession {
        text: String,
        selection: TextSelection,
        composition: Option<ImeComposition>,
        single_line: bool,
        submit_count: usize,
        clipboard: String,
    }

    #[derive(Clone, Copy, Default)]
    struct SimulatedKeyboardPipeline {
        is_ctrl: bool,
        is_shift: bool,
        read_only: bool,
        input_transform: Option<fn(String) -> String>,
        on_change: Option<fn(String) -> String>,
    }

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

    fn apply_ime_event(
        state: &mut SimulatedImeState,
        event: winit::event::Ime,
        input_transform: Option<fn(String) -> String>,
        on_change: Option<fn(String) -> String>,
    ) {
        let PlannedImeEvent::Edit(plan) = plan_ime_event_for_state(
            state.selection.clone(),
            state.composition.clone(),
            false,
            &event,
        )
        .expect("test sequence should only use supported ime events") else {
            panic!("ime sequence should not submit in non single-line simulation");
        };
        let start = plan.replacement_range.start.min(state.text.len());
        let end = plan.replacement_range.end.min(state.text.len()).max(start);
        let mut raw_content_after_replace = state.text.clone();
        raw_content_after_replace.replace_range(start..end, &plan.replacement_text);
        let raw_replaced_range = start..(start + plan.replacement_text.len());

        let transformed_content = if let Some(transform) = input_transform {
            transform(raw_content_after_replace.clone())
        } else {
            raw_content_after_replace.clone()
        };
        let transformed_selection = rebase_selection(
            &raw_content_after_replace,
            &transformed_content,
            plan.selection,
        );
        let transformed_replaced_range = rebase_range(
            &raw_content_after_replace,
            &transformed_content,
            raw_replaced_range,
        );

        let (final_content, final_selection, final_replaced_range) =
            if let Some(on_change) = on_change {
                let new_content = on_change(transformed_content.clone());
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

        state.text = final_content;
        state.selection = final_selection.clone();
        state.composition = plan.composition_range.map(|_| ImeComposition {
            range: final_replaced_range,
            selection: final_selection,
        });
    }

    fn apply_text_input_ime_event(
        state: &mut SimulatedTextInputSession,
        event: winit::event::Ime,
        input_transform: Option<fn(String) -> String>,
        on_change: Option<fn(String) -> String>,
    ) {
        let plan = plan_ime_event_for_state(
            state.selection.clone(),
            state.composition.clone(),
            state.single_line,
            &event,
        )
        .expect("test sequence should only use supported ime events");
        let PlannedImeEvent::Edit(plan) = plan else {
            state.composition = None;
            state.submit_count += 1;
            return;
        };
        let start = plan.replacement_range.start.min(state.text.len());
        let end = plan.replacement_range.end.min(state.text.len()).max(start);
        let mut raw_content_after_replace = state.text.clone();
        raw_content_after_replace.replace_range(start..end, &plan.replacement_text);
        let raw_replaced_range = start..(start + plan.replacement_text.len());

        let transformed_content = if let Some(transform) = input_transform {
            transform(raw_content_after_replace.clone())
        } else {
            raw_content_after_replace.clone()
        };
        let transformed_selection = rebase_selection(
            &raw_content_after_replace,
            &transformed_content,
            plan.selection,
        );
        let transformed_replaced_range = rebase_range(
            &raw_content_after_replace,
            &transformed_content,
            raw_replaced_range,
        );

        let (final_content, final_selection, final_replaced_range) =
            if let Some(on_change) = on_change {
                let new_content = on_change(transformed_content.clone());
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

        state.text = final_content;
        state.selection = final_selection.clone();
        state.composition = plan.composition_range.map(|_| ImeComposition {
            range: final_replaced_range,
            selection: final_selection,
        });
    }

    fn replace_text_range_in_text_input_session(
        state: &mut SimulatedTextInputSession,
        range: std::ops::Range<usize>,
        replacement: &str,
        input_transform: Option<fn(String) -> String>,
        on_change: Option<fn(String) -> String>,
    ) {
        let start = range.start.min(state.text.len());
        let end = range.end.min(state.text.len()).max(start);
        let mut raw_content_after_replace = state.text.clone();
        raw_content_after_replace.replace_range(start..end, replacement);

        let replacement_selection = TextSelection::collapsed(start + replacement.len());
        let transformed_content = if let Some(transform) = input_transform {
            transform(raw_content_after_replace.clone())
        } else {
            raw_content_after_replace.clone()
        };
        let transformed_selection = rebase_selection(
            &raw_content_after_replace,
            &transformed_content,
            replacement_selection,
        );
        let final_content = if let Some(on_change) = on_change {
            on_change(transformed_content.clone())
        } else {
            transformed_content.clone()
        };
        let final_selection = if final_content != transformed_content {
            rebase_selection(&transformed_content, &final_content, transformed_selection)
        } else {
            transformed_selection
        };

        state.text = final_content;
        state.selection = final_selection;
        state.composition = None;
    }

    fn replace_selected_text_in_text_input_session(
        state: &mut SimulatedTextInputSession,
        replacement: &str,
        input_transform: Option<fn(String) -> String>,
        on_change: Option<fn(String) -> String>,
    ) {
        replace_text_range_in_text_input_session(
            state,
            state.selection.ordered_range(),
            replacement,
            input_transform,
            on_change,
        );
    }

    fn selected_text_in_text_input_session(state: &SimulatedTextInputSession) -> Option<String> {
        let selection = state.selection.ordered_range();
        if selection.is_empty() {
            None
        } else {
            Some(state.text[selection].to_string())
        }
    }

    fn deletion_range_in_text_input_session(
        state: &SimulatedTextInputSession,
        motion: glyphon::cosmic_text::Motion,
    ) -> Option<std::ops::Range<usize>> {
        let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
        controller.set_single_line(state.single_line);
        controller.set_text_and_selection(&state.text, state.selection.clone());
        controller.deletion_range_for_motion(motion)
    }

    fn apply_text_input_keyboard_event(
        state: &mut SimulatedTextInputSession,
        key_state: winit::event::ElementState,
        logical_key: winit::keyboard::Key,
        pipeline: SimulatedKeyboardPipeline,
    ) {
        if let Some(behavior) = single_line_key_behavior(state.single_line, key_state, &logical_key)
        {
            match behavior {
                SingleLineKeyBehavior::Submit => {
                    state.submit_count += 1;
                }
                SingleLineKeyBehavior::Propagate => {}
            }
            return;
        }

        if let Some(behavior) =
            clipboard_shortcut_for_key(pipeline.is_ctrl, pipeline.is_shift, key_state, &logical_key)
        {
            match behavior {
                ClipboardShortcutBehavior::Copy => {
                    if let Some(text) = selected_text_in_text_input_session(state) {
                        state.clipboard = text;
                    }
                }
                ClipboardShortcutBehavior::Cut => {
                    if !pipeline.read_only
                        && let Some(text) = selected_text_in_text_input_session(state)
                    {
                        state.clipboard = text;
                        replace_selected_text_in_text_input_session(
                            state,
                            "",
                            pipeline.input_transform,
                            pipeline.on_change,
                        );
                    }
                }
                ClipboardShortcutBehavior::Paste => {
                    if !pipeline.read_only {
                        let text = state.clipboard.clone();
                        replace_selected_text_in_text_input_session(
                            state,
                            &text,
                            pipeline.input_transform,
                            pipeline.on_change,
                        );
                    }
                }
            }
            return;
        }

        if let Some(motion) = deletion_motion_for_key(pipeline.is_ctrl, key_state, &logical_key) {
            if !pipeline.read_only
                && let Some(range) = deletion_range_in_text_input_session(state, motion)
            {
                replace_text_range_in_text_input_session(
                    state,
                    range,
                    "",
                    pipeline.input_transform,
                    pipeline.on_change,
                );
            }
            return;
        }

        let shortcut = if let winit::keyboard::Key::Character(text) = &logical_key {
            if pipeline.is_ctrl && key_state == winit::event::ElementState::Pressed {
                Some(text.to_lowercase())
            } else {
                None
            }
        } else {
            None
        };

        match shortcut.as_deref() {
            Some("a") => {
                state.selection = TextSelection {
                    start: 0,
                    end: state.text.len(),
                };
                return;
            }
            Some("c") => {
                if let Some(text) = selected_text_in_text_input_session(state) {
                    state.clipboard = text;
                }
                return;
            }
            Some("x") => {
                if !pipeline.read_only
                    && let Some(text) = selected_text_in_text_input_session(state)
                {
                    state.clipboard = text;
                    replace_selected_text_in_text_input_session(
                        state,
                        "",
                        pipeline.input_transform,
                        pipeline.on_change,
                    );
                }
                return;
            }
            Some("v") => {
                if !pipeline.read_only {
                    let text = state.clipboard.clone();
                    replace_selected_text_in_text_input_session(
                        state,
                        &text,
                        pipeline.input_transform,
                        pipeline.on_change,
                    );
                }
                return;
            }
            _ => {}
        }

        if key_state != winit::event::ElementState::Pressed {
            return;
        }

        match logical_key {
            winit::keyboard::Key::Character(text) if !pipeline.is_ctrl && !pipeline.read_only => {
                replace_selected_text_in_text_input_session(
                    state,
                    text.as_ref(),
                    pipeline.input_transform,
                    pipeline.on_change,
                );
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter)
                if !state.single_line && !pipeline.read_only =>
            {
                replace_selected_text_in_text_input_session(
                    state,
                    "\n",
                    pipeline.input_transform,
                    pipeline.on_change,
                );
            }
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab)
                if !state.single_line && !pipeline.read_only =>
            {
                replace_selected_text_in_text_input_session(
                    state,
                    "\t",
                    pipeline.input_transform,
                    pipeline.on_change,
                );
            }
            _ => {}
        }
    }

    fn group_digits(value: String) -> String {
        let digits: Vec<char> = value
            .chars()
            .filter(|character| character.is_ascii_digit())
            .collect();
        let mut formatted = String::new();
        for (index, digit) in digits.into_iter().enumerate() {
            if index > 0 && index % 3 == 0 {
                formatted.push(' ');
            }
            formatted.push(digit);
        }
        formatted
    }

    fn keep_prompt_prefix(value: String) -> String {
        if value.starts_with('>') {
            value
        } else {
            format!(">{value}")
        }
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
    fn rebase_offset_shifts_cursor_after_inserted_formatting_prefix() {
        assert_eq!(rebase_offset("1234", "123 4", 4), 5);
    }

    #[test]
    fn rebase_selection_preserves_tail_selection_through_inserted_formatting() {
        assert_eq!(
            rebase_selection("1234", "123 4", TextSelection { start: 3, end: 4 },),
            TextSelection { start: 4, end: 5 }
        );
    }

    #[test]
    fn rebase_selection_collapses_inside_rewritten_region() {
        assert_eq!(
            rebase_selection("abcdef", "abXYef", TextSelection { start: 2, end: 4 },),
            TextSelection { start: 2, end: 4 }
        );
    }

    #[test]
    fn rebase_selection_preserves_backward_direction_through_inserted_formatting() {
        assert_eq!(
            rebase_selection("1234", "123 4", TextSelection { start: 4, end: 3 },),
            TextSelection { start: 5, end: 4 }
        );
    }

    #[test]
    fn editor_selection_preserves_backward_anchor_and_cursor() {
        let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
        controller.set_text_and_selection("hello", TextSelection { start: 4, end: 1 });

        assert_eq!(
            editor_selection(controller.editor()),
            TextSelection { start: 4, end: 1 }
        );
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
    fn ime_sequence_preedit_preedit_commit_tracks_latest_composition_range() {
        let mut state = SimulatedImeState {
            text: String::new(),
            selection: TextSelection::collapsed(0),
            composition: None,
        };

        apply_ime_event(
            &mut state,
            winit::event::Ime::Preedit("abcd".to_string(), Some((2, 2))),
            None,
            None,
        );
        assert_eq!(state.text, "abcd");
        assert_eq!(state.selection, TextSelection::collapsed(2));
        assert_eq!(
            state.composition,
            Some(ImeComposition {
                range: 0..4,
                selection: TextSelection::collapsed(2),
            })
        );

        apply_ime_event(
            &mut state,
            winit::event::Ime::Preedit("abXcd".to_string(), Some((3, 3))),
            None,
            None,
        );
        assert_eq!(state.text, "abXcd");
        assert_eq!(state.selection, TextSelection::collapsed(3));
        assert_eq!(
            state.composition,
            Some(ImeComposition {
                range: 0..5,
                selection: TextSelection::collapsed(3),
            })
        );

        apply_ime_event(
            &mut state,
            winit::event::Ime::Commit("done".to_string()),
            None,
            None,
        );
        assert_eq!(state.text, "done");
        assert_eq!(state.selection, TextSelection::collapsed(4));
        assert_eq!(state.composition, None);
    }

    #[test]
    fn ime_sequence_rebases_composition_range_through_formatter() {
        let mut state = SimulatedImeState {
            text: String::new(),
            selection: TextSelection::collapsed(0),
            composition: None,
        };

        apply_ime_event(
            &mut state,
            winit::event::Ime::Preedit("123456".to_string(), Some((6, 6))),
            Some(group_digits),
            None,
        );
        assert_eq!(state.text, "123 456");
        assert_eq!(state.selection, TextSelection::collapsed(7));
        assert_eq!(
            state.composition,
            Some(ImeComposition {
                range: 0..7,
                selection: TextSelection::collapsed(7),
            })
        );

        apply_ime_event(
            &mut state,
            winit::event::Ime::Preedit("1234567890".to_string(), Some((10, 10))),
            Some(group_digits),
            None,
        );
        assert_eq!(state.text, "123 456 789 0");
        assert_eq!(state.selection, TextSelection::collapsed(13));
        assert_eq!(
            state.composition,
            Some(ImeComposition {
                range: 0..13,
                selection: TextSelection::collapsed(13),
            })
        );
    }

    #[test]
    fn ime_sequence_rebases_composition_range_through_on_change() {
        let mut state = SimulatedImeState {
            text: String::new(),
            selection: TextSelection::collapsed(0),
            composition: None,
        };

        apply_ime_event(
            &mut state,
            winit::event::Ime::Preedit("abcd".to_string(), Some((2, 2))),
            None,
            Some(keep_prompt_prefix),
        );
        assert_eq!(state.text, ">abcd");
        assert_eq!(state.selection, TextSelection::collapsed(3));
        assert_eq!(
            state.composition,
            Some(ImeComposition {
                range: 1..5,
                selection: TextSelection::collapsed(3),
            })
        );

        apply_ime_event(
            &mut state,
            winit::event::Ime::Commit("xy".to_string()),
            None,
            Some(keep_prompt_prefix),
        );
        assert_eq!(state.text, ">xy");
        assert_eq!(state.selection, TextSelection::collapsed(3));
        assert_eq!(state.composition, None);
    }

    #[test]
    fn text_input_ime_sequence_single_line_newline_submit_clears_composition_without_inserting_newline()
     {
        let mut state = SimulatedTextInputSession {
            text: String::new(),
            selection: TextSelection::collapsed(0),
            composition: None,
            single_line: true,
            submit_count: 0,
            clipboard: String::new(),
        };

        apply_text_input_ime_event(
            &mut state,
            winit::event::Ime::Preedit("abcd".to_string(), Some((2, 2))),
            None,
            None,
        );
        assert_eq!(state.text, "abcd");
        assert_eq!(
            state.composition,
            Some(ImeComposition {
                range: 0..4,
                selection: TextSelection::collapsed(2),
            })
        );

        apply_text_input_ime_event(
            &mut state,
            winit::event::Ime::Commit("\n".to_string()),
            None,
            None,
        );
        assert_eq!(state.text, "abcd");
        assert_eq!(state.selection, TextSelection::collapsed(2));
        assert_eq!(state.composition, None);
        assert_eq!(state.submit_count, 1);
    }

    #[test]
    fn text_input_ime_sequence_multi_line_newline_commit_keeps_editing_path() {
        let mut state = SimulatedTextInputSession {
            text: String::new(),
            selection: TextSelection::collapsed(0),
            composition: None,
            single_line: false,
            submit_count: 0,
            clipboard: String::new(),
        };

        apply_text_input_ime_event(
            &mut state,
            winit::event::Ime::Preedit("abcd".to_string(), Some((2, 2))),
            None,
            None,
        );
        apply_text_input_ime_event(
            &mut state,
            winit::event::Ime::Commit("\n".to_string()),
            None,
            None,
        );
        assert_eq!(state.text, "\n");
        assert_eq!(state.selection, TextSelection::collapsed(1));
        assert_eq!(state.composition, None);
        assert_eq!(state.submit_count, 0);
    }

    #[test]
    fn text_input_ime_sequence_single_line_newline_submit_preserves_formatted_text() {
        let mut state = SimulatedTextInputSession {
            text: String::new(),
            selection: TextSelection::collapsed(0),
            composition: None,
            single_line: true,
            submit_count: 0,
            clipboard: String::new(),
        };

        apply_text_input_ime_event(
            &mut state,
            winit::event::Ime::Preedit("123456".to_string(), Some((6, 6))),
            Some(group_digits),
            None,
        );
        assert_eq!(state.text, "123 456");
        assert_eq!(
            state.composition,
            Some(ImeComposition {
                range: 0..7,
                selection: TextSelection::collapsed(7),
            })
        );

        apply_text_input_ime_event(
            &mut state,
            winit::event::Ime::Commit("\r\n".to_string()),
            Some(group_digits),
            None,
        );
        assert_eq!(state.text, "123 456");
        assert_eq!(state.selection, TextSelection::collapsed(7));
        assert_eq!(state.composition, None);
        assert_eq!(state.submit_count, 1);
    }

    #[test]
    fn text_input_keyboard_sequence_single_line_enter_submits_without_editing_text() {
        let mut state = SimulatedTextInputSession {
            text: "hello".to_string(),
            selection: TextSelection::collapsed(5),
            composition: Some(ImeComposition {
                range: 0..5,
                selection: TextSelection::collapsed(5),
            }),
            single_line: true,
            submit_count: 0,
            clipboard: String::new(),
        };

        apply_text_input_keyboard_event(
            &mut state,
            winit::event::ElementState::Pressed,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter),
            SimulatedKeyboardPipeline::default(),
        );
        assert_eq!(state.text, "hello");
        assert_eq!(state.selection, TextSelection::collapsed(5));
        assert_eq!(
            state.composition,
            Some(ImeComposition {
                range: 0..5,
                selection: TextSelection::collapsed(5),
            })
        );
        assert_eq!(state.submit_count, 1);
    }

    #[test]
    fn text_input_keyboard_sequence_ctrl_backspace_deletes_previous_word() {
        let mut state = SimulatedTextInputSession {
            text: "hello world".to_string(),
            selection: TextSelection::collapsed(11),
            composition: None,
            single_line: false,
            submit_count: 0,
            clipboard: String::new(),
        };

        apply_text_input_keyboard_event(
            &mut state,
            winit::event::ElementState::Pressed,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace),
            SimulatedKeyboardPipeline {
                is_ctrl: true,
                ..Default::default()
            },
        );
        assert_eq!(state.text, "hello ");
        assert_eq!(state.selection, TextSelection::collapsed(6));
        assert_eq!(state.clipboard, "");
    }

    #[test]
    fn text_input_keyboard_sequence_ctrl_delete_reformats_grouped_digits() {
        let mut state = SimulatedTextInputSession {
            text: "123 456 789".to_string(),
            selection: TextSelection::collapsed(4),
            composition: None,
            single_line: false,
            submit_count: 0,
            clipboard: String::new(),
        };

        apply_text_input_keyboard_event(
            &mut state,
            winit::event::ElementState::Pressed,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete),
            SimulatedKeyboardPipeline {
                is_ctrl: true,
                input_transform: Some(group_digits),
                ..Default::default()
            },
        );
        assert_eq!(state.text, "123 789");
        assert_eq!(state.selection, TextSelection::collapsed(4));
    }

    #[test]
    fn text_input_keyboard_sequence_shift_delete_cuts_selection_and_ctrl_v_pastes_it_back() {
        let mut state = SimulatedTextInputSession {
            text: "hello world".to_string(),
            selection: TextSelection { start: 6, end: 11 },
            composition: None,
            single_line: false,
            submit_count: 0,
            clipboard: String::new(),
        };

        apply_text_input_keyboard_event(
            &mut state,
            winit::event::ElementState::Pressed,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete),
            SimulatedKeyboardPipeline {
                is_shift: true,
                ..Default::default()
            },
        );
        assert_eq!(state.text, "hello ");
        assert_eq!(state.selection, TextSelection::collapsed(6));
        assert_eq!(state.clipboard, "world");

        apply_text_input_keyboard_event(
            &mut state,
            winit::event::ElementState::Pressed,
            winit::keyboard::Key::Character("v".into()),
            SimulatedKeyboardPipeline {
                is_ctrl: true,
                ..Default::default()
            },
        );
        assert_eq!(state.text, "hello world");
        assert_eq!(state.selection, TextSelection::collapsed(11));
        assert_eq!(state.clipboard, "world");
    }

    #[test]
    fn text_input_keyboard_sequence_character_insert_rebases_through_on_change() {
        let mut state = SimulatedTextInputSession {
            text: "abc".to_string(),
            selection: TextSelection::collapsed(3),
            composition: None,
            single_line: false,
            submit_count: 0,
            clipboard: String::new(),
        };

        apply_text_input_keyboard_event(
            &mut state,
            winit::event::ElementState::Pressed,
            winit::keyboard::Key::Character("x".into()),
            SimulatedKeyboardPipeline {
                on_change: Some(keep_prompt_prefix),
                ..Default::default()
            },
        );
        assert_eq!(state.text, ">abcx");
        assert_eq!(state.selection, TextSelection::collapsed(5));
    }

    #[test]
    fn text_input_keyboard_sequence_multi_line_tab_inserts_tab_character() {
        let mut state = SimulatedTextInputSession {
            text: String::new(),
            selection: TextSelection::collapsed(0),
            composition: None,
            single_line: false,
            submit_count: 0,
            clipboard: String::new(),
        };

        apply_text_input_keyboard_event(
            &mut state,
            winit::event::ElementState::Pressed,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab),
            SimulatedKeyboardPipeline::default(),
        );
        assert_eq!(state.text, "\t");
        assert_eq!(state.selection, TextSelection::collapsed(1));
    }

    #[test]
    fn text_input_keyboard_sequence_ctrl_a_then_ctrl_c_copies_full_selection() {
        let mut state = SimulatedTextInputSession {
            text: "hello world".to_string(),
            selection: TextSelection::collapsed(5),
            composition: None,
            single_line: false,
            submit_count: 0,
            clipboard: String::new(),
        };

        apply_text_input_keyboard_event(
            &mut state,
            winit::event::ElementState::Pressed,
            winit::keyboard::Key::Character("a".into()),
            SimulatedKeyboardPipeline {
                is_ctrl: true,
                ..Default::default()
            },
        );
        apply_text_input_keyboard_event(
            &mut state,
            winit::event::ElementState::Pressed,
            winit::keyboard::Key::Character("c".into()),
            SimulatedKeyboardPipeline {
                is_ctrl: true,
                ..Default::default()
            },
        );
        assert_eq!(
            state.selection,
            TextSelection {
                start: 0,
                end: "hello world".len(),
            }
        );
        assert_eq!(state.clipboard, "hello world");
    }

    #[test]
    fn text_input_keyboard_sequence_read_only_paste_does_not_mutate_text() {
        let mut state = SimulatedTextInputSession {
            text: "hello".to_string(),
            selection: TextSelection::collapsed(5),
            composition: None,
            single_line: false,
            submit_count: 0,
            clipboard: " world".to_string(),
        };

        apply_text_input_keyboard_event(
            &mut state,
            winit::event::ElementState::Pressed,
            winit::keyboard::Key::Character("v".into()),
            SimulatedKeyboardPipeline {
                is_ctrl: true,
                read_only: true,
                ..Default::default()
            },
        );
        assert_eq!(state.text, "hello");
        assert_eq!(state.selection, TextSelection::collapsed(5));
        assert_eq!(state.clipboard, " world");
    }

    #[test]
    fn text_input_pointer_sequence_shift_click_extends_selection_from_existing_cursor() {
        let text = "hello world";
        let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
        controller.set_text_and_selection(text, TextSelection::collapsed(1));
        let target_position = pointer_position_for_offset(text, 5);
        let click_time = Instant::now();

        let click_type = controller.handle_click(target_position, click_time);
        assert!(matches!(click_type, ClickType::Single));
        controller.extend_selection_to_point(target_position);

        assert_eq!(controller.selection(), TextSelection { start: 1, end: 5 });
        assert_eq!(controller.cursor_offset(), 5);
    }

    #[test]
    fn text_input_pointer_sequence_single_click_drag_selects_backward_range() {
        let text = "hello world";
        let mut controller = TextEditorController::new(tessera_ui::Dp(14.0), None);
        controller.set_text_and_selection(text, TextSelection::collapsed(6));
        let start_position = pointer_position_for_offset(text, 6);
        let drag_position = pointer_position_for_offset(text, 1);
        let click_time = Instant::now();

        let click_type = controller.handle_click(start_position, click_time);
        assert!(matches!(click_type, ClickType::Single));
        controller.apply_pointer_action(GlyphonAction::Click {
            x: start_position.x.0,
            y: start_position.y.0,
        });
        controller.start_drag(click_type);
        controller.apply_pointer_action(GlyphonAction::Drag {
            x: drag_position.x.0,
            y: drag_position.y.0,
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

        let first_click_type = controller.handle_click(first_position, first_click_time);
        assert!(matches!(first_click_type, ClickType::Single));
        controller.apply_pointer_action(GlyphonAction::Click {
            x: first_position.x.0,
            y: first_position.y.0,
        });

        let second_click_type = controller.handle_click(first_position, second_click_time);
        assert!(matches!(second_click_type, ClickType::Double));
        controller.apply_pointer_action(GlyphonAction::DoubleClick {
            x: first_position.x.0,
            y: first_position.y.0,
        });
        controller.start_drag(second_click_type);
        controller.apply_pointer_action(GlyphonAction::Drag {
            x: drag_position.x.0,
            y: drag_position.y.0,
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

        let first_click_type = controller.handle_click(line_position, first_click_time);
        assert!(matches!(first_click_type, ClickType::Single));
        controller.apply_pointer_action(GlyphonAction::Click {
            x: line_position.x.0,
            y: line_position.y.0,
        });

        let second_click_type = controller.handle_click(line_position, second_click_time);
        assert!(matches!(second_click_type, ClickType::Double));
        controller.apply_pointer_action(GlyphonAction::DoubleClick {
            x: line_position.x.0,
            y: line_position.y.0,
        });

        let third_click_type = controller.handle_click(line_position, third_click_time);
        assert!(matches!(third_click_type, ClickType::Triple));
        controller.apply_pointer_action(GlyphonAction::TripleClick {
            x: line_position.x.0,
            y: line_position.y.0,
        });
        controller.start_drag(third_click_type);
        controller.apply_pointer_action(GlyphonAction::Drag {
            x: drag_position.x.0,
            y: drag_position.y.0,
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
    fn single_line_submit_plan_clears_existing_composition_via_controller_state() {
        let composition = ImeComposition {
            range: 1..4,
            selection: TextSelection::collapsed(3),
        };

        assert_eq!(
            plan_ime_event_for_state(
                TextSelection::collapsed(3),
                Some(composition),
                true,
                &winit::event::Ime::Commit("\n".to_string()),
            ),
            Some(PlannedImeEvent::Submit)
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
}
