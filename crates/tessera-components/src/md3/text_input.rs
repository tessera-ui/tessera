//! Editable text input with Material outlined styling.
//!
//! ## Usage
//!
//! Collect short-form input when the full text field decoration is not needed.
use tessera_foundation::modifier::{InteractionState, InteractionVisualState, ModifierExt as _};
use tessera_ui::{
    Callback, CallbackWith, Color, Dp, Modifier, State,
    layout::{Stack, layout},
    modifier::FocusModifierExt as _,
    remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    md3::{
        surface::{SurfaceStyle, surface},
        theme::{
            MaterialColorScheme, MaterialShapes, MaterialTheme, TextSelectionColors, TextStyle,
        },
    },
    shape_def::{RoundedCorner, Shape},
    text_edit_core::text_edit_core,
    text_edit_input::{TextEditInput, TextEditInsets, attach_text_edit_input},
};

pub use crate::md3::text_edit_core::{
    DisplayTransform, TextEditorController as TextInputController,
    TransformedText as DisplayTransformText,
};

/// Material 3 opacity applied to disabled field text and labels.
pub(crate) const DISABLED_CONTENT_ALPHA: f32 = 0.38;

/// Material 3 opacity applied to the outline of a disabled field.
pub(crate) const DISABLED_OUTLINE_ALPHA: f32 = 0.12;

/// Material 3 opacity applied to the container of a disabled filled field.
pub(crate) const DISABLED_FILL_ALPHA: f32 = 0.04;

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
    pub min_width: Option<Dp>,
    pub min_height: Option<Dp>,
    pub background_color: Option<Color>,
    pub border_width: Dp,
    pub border_color: Option<Color>,
    pub shape: Shape,
    pub padding: Dp,
    pub focus_border_color: Option<Color>,
    pub focus_border_width: Option<Dp>,
    pub focus_background_color: Option<Color>,
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
    pub font_size: Dp,
    pub line_height: Option<Dp>,
    /// Optional typography override for editable text.
    pub style: Option<TextStyle>,
    /// Whether the editor behaves as a single-line field.
    ///
    /// Text does not wrap and scrolls horizontally.
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

impl TextInputProps {
    fn input(&self) -> TextEditInput {
        TextEditInput {
            enabled: self.enabled,
            read_only: self.read_only,
            is_secure: self.is_secure,
            single_line: self.single_line,
            on_change: self.on_change,
            on_submit: self.on_submit,
            input_transform: self.input_transform,
            accessibility_label: self.accessibility_label.clone(),
            accessibility_description: self.accessibility_description.clone(),
            content_insets: TextEditInsets {
                horizontal: self.padding,
                vertical: self.padding,
                border: self.border_width,
            },
        }
    }
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

impl TextInputBuilder {
    /// Set a display-only transform with explicit offset mapping.
    pub fn display_transform_mapped<F>(&self, transform: F) -> Self
    where
        F: Fn(&str) -> DisplayTransformText + 'static,
    {
        self.props
            .configure_deferred("display_transform", move |props| {
                props.display_transform =
                    Some(CallbackWith::new(move |value: String| transform(&value)))
            });
        Self { props: self.props }
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
/// use tessera_components::md3::text_input::text_input;
/// use tessera_ui::Dp;
/// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// text_input().padding(Dp(8.0)).initial_text("Hello World");
/// # });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
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
    let controller = if let Some(controller) = args.controller {
        controller
    } else {
        remember(|| {
            let mut c = TextInputController::new(args.font_size, args.line_height);
            if let Some(text) = &args.initial_text {
                c.set_text(text);
            }
            c
        })
    };
    let mut editor_args = args.clone();
    editor_args.controller = Some(controller);

    if !editor_args.enabled {
        controller.with(|c| *c.focus_handler()).clear_focus();
    }
    sync_text_input_controller(&controller, &editor_args);
    let focus = controller.with(|c| *c.focus_handler());
    let modifier = attach_text_edit_input(
        Modifier::new()
            .focus_requester(focus)
            .focusable()
            .focus_properties(
                tessera_ui::FocusProperties::new()
                    .can_focus(editor_args.enabled)
                    .can_request_focus(editor_args.enabled),
            ),
        editor_args.input(),
        controller,
    );

    let current = editor_args;
    let editor_args = remember(|| current.clone());
    if editor_args.with(|args| args != &current) {
        editor_args.set(current);
    }
    layout(Stack).modifier(modifier).child(move || {
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
        controller.with(|c| *c.focus_handler()).clear_focus();
    }
    sync_text_input_controller(&controller, &editor_args);
    let focus = controller.with(|c| *c.focus_handler());
    let modifier = attach_text_edit_input(
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
        editor_args.input(),
        controller,
    );

    let current = editor_args;
    let editor_args = remember(|| current.clone());
    if editor_args.with(|args| args != &current) {
        editor_args.set(current);
    }
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(modifier)
        .children(move || {
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
    layout(Stack)
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

#[tessera]
pub(crate) fn text_input_core(args: TextInputProps, controller: State<TextInputController>) {
    let mut core_args = args;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FieldVisualState {
    focused: bool,
    enabled: bool,
    is_error: bool,
    outlined: bool,
}

fn default_fill_color(scheme: &MaterialColorScheme, state: FieldVisualState) -> Color {
    if !state.enabled && !state.outlined {
        return scheme.on_surface.with_alpha(DISABLED_FILL_ALPHA);
    }
    Color::TRANSPARENT
}

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

/// Presets for an already declared text input.
impl TextInputBuilder {
    /// Plain gray-bordered preset.
    ///
    /// # Example
    ///
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::md3::text_input::text_input;
    /// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = text_input().simple();
    /// # });
    /// # }
    /// # tessera_ui::testing::UiTest::new().content(component).frame();
    /// ```
    pub fn simple(&self) -> Self {
        self.min_width(Dp(120.0))
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

    /// [`Self::simple`] preset with a blue focus border.
    ///
    /// # Example
    /// ```
    /// # use tessera_ui::tessera;
    /// # #[tessera]
    /// # fn component() {
    /// use tessera_components::md3::text_input::text_input;
    /// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = text_input().outlined();
    /// # });
    /// # }
    /// # tessera_ui::testing::UiTest::new().content(component).frame();
    /// ```
    pub fn outlined(&self) -> Self {
        self.simple()
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
    /// use tessera_components::md3::text_input::text_input;
    /// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
    /// # material_theme()
    /// #     .theme(|| MaterialTheme::default())
    /// #     .child(|| {
    /// let args = text_input().minimal();
    /// # });
    /// # }
    /// # tessera_ui::testing::UiTest::new().content(component).frame();
    /// ```
    pub fn minimal(&self) -> Self {
        self.min_width(Dp(120.0))
            .background_color(Color::WHITE)
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(0.0), 3.0),
                top_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
            })
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "text_input/clipboard_tests.rs"]
mod clipboard_tests;

#[cfg(test)]
mod tests {
    use tessera_ui::{Color, Dp};

    use super::{
        DISABLED_FILL_ALPHA, DISABLED_OUTLINE_ALPHA, FieldVisualState, default_fill_color,
        default_outline_color, resolve_border_width,
    };

    fn scheme() -> crate::md3::theme::MaterialColorScheme {
        crate::md3::theme::MaterialColorScheme::default()
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
        use crate::md3::modifier::ModifierExt as _;

        use tessera_foundation::modifier::ModifierExt as _;
        use tessera_ui::{
            Dp, Modifier, Px, PxPosition, State, layout::layout, remember, tessera, testing::UiTest,
        };

        use crate::{
            column::column,
            md3::{
                scrollable::ScrollableController,
                text_field::text_field,
                text_input::text_input,
                theme::{MaterialTheme, material_theme},
            },
            spacer::spacer,
        };

        #[derive(Clone, Copy)]
        enum ProbeKind {
            Plain,
            Input,
            Field,
        }

        #[tessera]
        fn scroll_probe_with_input(
            output: tessera_ui::testing::TestProbe<Option<State<ScrollableController>>>,
        ) {
            let controller = remember(ScrollableController::new);
            output.set(Some(controller));
            material_theme()
                .theme(|| MaterialTheme::default())
                .child(move || {
                    layout(tessera_ui::layout::Stack)
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
        fn scroll_probe_plain(
            output: tessera_ui::testing::TestProbe<Option<State<ScrollableController>>>,
        ) {
            let controller = remember(ScrollableController::new);
            output.set(Some(controller));
            material_theme()
                .theme(|| MaterialTheme::default())
                .child(move || {
                    layout(tessera_ui::layout::Stack)
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

        fn scrolled_after_wheel(kind: ProbeKind, focus_first: bool) -> f32 {
            let output = tessera_ui::testing::TestProbe::default();
            let probe = output.clone();
            let mut session = UiTest::new()
                .content(move || match kind {
                    ProbeKind::Plain => {
                        scroll_probe_plain(probe.clone());
                    }
                    ProbeKind::Input => {
                        scroll_probe_with_input(probe.clone());
                    }
                    ProbeKind::Field => {
                        scroll_probe_with_field(probe.clone());
                    }
                })
                .viewport(200, 100);
            session.frame();
            let controller = output.get().expect("scrollable controller");
            let position = PxPosition::new(Px::new(100), Px::new(20));
            if focus_first {
                session.click_at(position);
            }
            session.pointer_scroll(tessera_ui::MOUSE_POINTER_ID, position, 0.0, -40.0);
            session.frames(1);
            session.frames(36);

            controller.with(|c| c.child_position().y)
        }

        #[test]
        fn parent_scrollable_scrolls_without_text_input() {
            let offset = scrolled_after_wheel(ProbeKind::Plain, false);
            assert!(
                offset != 0.0,
                "plain content should scroll, got y={offset:?}"
            );
        }

        #[test]
        fn wheel_over_unfocused_input_reaches_parent_scrollable() {
            let offset = scrolled_after_wheel(ProbeKind::Input, false);
            assert!(
                offset != 0.0,
                "wheel over an unfocused text input should scroll the parent, got y={offset:?}"
            );
        }

        #[test]
        fn wheel_over_focused_input_reaches_parent_scrollable() {
            let offset = scrolled_after_wheel(ProbeKind::Input, true);
            assert!(
                offset != 0.0,
                "wheel over a focused text input should scroll the parent, got y={offset:?}"
            );
        }

        #[tessera]
        fn scroll_probe_with_field(
            output: tessera_ui::testing::TestProbe<Option<State<ScrollableController>>>,
        ) {
            let controller = remember(ScrollableController::new);
            output.set(Some(controller));
            material_theme()
                .theme(|| MaterialTheme::default())
                .child(move || {
                    layout(tessera_ui::layout::Stack)
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
            let offset = scrolled_after_wheel(ProbeKind::Field, false);
            assert!(
                offset != 0.0,
                "wheel over a text field should scroll the parent, got y={offset:?}"
            );
        }
    }
}
