//! Material text fields for filled, outlined, and secure input.
//!
//! ## Usage
//!
//! Collect short-form inputs like names, passwords, or search queries.
use glyphon::Action as GlyphonAction;
use tessera_foundation::gesture::{ScrollRecognizer, TapRecognizer, TapSettings};
use tessera_ui::{
    Callback, CallbackWith, Color, ComputedData, Constraint, Dp, LayoutResult, MeasurementError,
    Modifier, PressKeyEventType, Px, PxPosition, RenderSlot, ScrollDeltaUnit, State,
    layout::{MeasureScope, layout},
    modifier::CursorModifierExt as _,
    normalize_platform_scroll_delta, provide_context, remember, tessera, use_context, winit,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    divider::horizontal_divider,
    menus::{MenuAnchor, MenuController, MenuPlacement, menu_item, menu_provider},
    modifier::{ModifierExt as _, Padding, with_pointer_input},
    pos_misc::is_position_inside_bounds,
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    text::text,
    text_edit_core::DisplayTransform,
    text_input::{
        DISABLED_CONTENT_ALPHA, DISABLED_FILL_ALPHA, DisplayTransformText, TextInputController,
        TextInputProps, create_surface_args, text_input_core,
    },
    theme::{
        ContentColor, MaterialColorScheme, MaterialShapes, MaterialTheme, TextSelectionColors,
        TextStyle,
    },
};

/// Defaults for Material text fields.
pub struct TextFieldDefaults;

impl TextFieldDefaults {
    /// Default minimum width for text fields.
    pub const MIN_WIDTH: Dp = Dp(280.0);
    /// Default minimum height for text fields.
    pub const MIN_HEIGHT: Dp = Dp(56.0);
    /// Default padding applied on all sides.
    pub const CONTENT_PADDING: Dp = Dp(16.0);
    /// Default outline border width for outlined fields.
    pub const OUTLINED_BORDER_WIDTH: Dp = Dp(1.0);
    /// Default focused border width for outlined fields.
    pub const OUTLINED_FOCUS_BORDER_WIDTH: Dp = Dp(2.0);
    /// Default unfocused indicator thickness for filled fields.
    pub const INDICATOR_THICKNESS: Dp = Dp(1.0);
    /// Default focused indicator thickness for filled fields.
    pub const INDICATOR_FOCUSED_THICKNESS: Dp = Dp(2.0);
    /// Suggested vertical spacing for custom label arrangements.
    pub const LABEL_BOTTOM_PADDING: Dp = Dp(4.0);
    /// Vertical offset from the container top for floating labels.
    pub const FLOATING_LABEL_Y_OFFSET: Dp = Dp(8.0);
    /// Horizontal padding applied around outlined floating labels.
    pub const OUTLINED_LABEL_PADDING: Dp = Dp(2.0);
    /// Extra height applied to outlined label notches.
    pub const OUTLINED_NOTCH_VERTICAL_PADDING: Dp = Dp(2.0);
    /// Horizontal padding between icon slots and the text content.
    pub const ICON_TEXT_PADDING: Dp = Dp(16.0);
    /// Horizontal padding between prefix/suffix and the input text.
    pub const PREFIX_SUFFIX_PADDING: Dp = Dp(2.0);
    /// Default obfuscation character for secure fields.
    pub const OBFUSCATION_CHAR: char = '*';
}

/// Line limit behavior for a text field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextFieldLineLimit {
    /// Restrict input to a single line.
    #[default]
    SingleLine,
    /// Allow multiple lines of input.
    MultiLine,
}

/// Controls which context menu actions are available.
#[derive(Clone, PartialEq, Copy, Debug)]
pub struct TextFieldContextMenu {
    /// Whether the context menu is enabled.
    pub enabled: bool,
    /// Whether the Cut action is available.
    pub allow_cut: bool,
    /// Whether the Copy action is available.
    pub allow_copy: bool,
    /// Whether the Paste action is available.
    pub allow_paste: bool,
    /// Whether the Select All action is available.
    pub allow_select_all: bool,
}

impl TextFieldContextMenu {
    /// Disables the context menu.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            allow_cut: false,
            allow_copy: false,
            allow_paste: false,
            allow_select_all: false,
        }
    }

    /// Default policy for secure fields (no cut/copy).
    pub fn secure_default() -> Self {
        Self {
            enabled: true,
            allow_cut: false,
            allow_copy: false,
            allow_paste: true,
            allow_select_all: true,
        }
    }
}

impl Default for TextFieldContextMenu {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_cut: true,
            allow_copy: true,
            allow_paste: true,
            allow_select_all: true,
        }
    }
}

/// Arguments for configuring a Material text field.
#[derive(Clone, PartialEq)]
struct TextFieldProps {
    /// Whether the text field is enabled for user input.
    pub enabled: bool,
    /// Whether the text field is read-only.
    pub read_only: bool,
    /// Whether the field is displaying an error state.
    pub is_error: bool,
    /// Optional modifier chain applied to the field container.
    pub modifier: Modifier,
    /// Called when the text content changes. The closure receives the new text
    /// content and returns the updated content.
    pub on_change: CallbackWith<String, String>,
    /// Called when the user submits a single-line field with the Enter key.
    pub on_submit: Callback,
    /// Minimum width in density-independent pixels.
    pub min_width: Option<Dp>,
    /// Minimum height in density-independent pixels.
    pub min_height: Option<Dp>,
    /// Background color of the text field.
    pub background_color: Option<Color>,
    /// Border width in Dp.
    pub border_width: Dp,
    /// Border color of the text field.
    pub border_color: Option<Color>,
    /// The shape of the text field container.
    pub shape: Shape,
    /// Padding inside the text field.
    pub padding: Dp,
    /// Border color when focused.
    pub focus_border_color: Option<Color>,
    /// Border width when focused.
    pub focus_border_width: Option<Dp>,
    /// Background color when focused.
    pub focus_background_color: Option<Color>,
    /// Color for text selection highlight.
    pub selection_color: Option<Color>,
    /// Color of the text content.
    pub text_color: Option<Color>,
    /// Color of the text cursor.
    pub cursor_color: Option<Color>,
    /// Accessible name, defaulting to the visible label.
    pub accessibility_label: Option<String>,
    /// Optional description announced by assistive technologies.
    pub accessibility_description: Option<String>,
    /// Initial text content.
    pub initial_text: Option<String>,
    /// Font size in Dp.
    pub font_size: Dp,
    /// Line height in Dp.
    pub line_height: Option<Dp>,
    /// Optional typography override for editable text.
    pub style: Option<TextStyle>,
    /// Optional label text shown inside the field and floated when focused or
    /// non-empty.
    pub label: Option<String>,
    /// Optional placeholder text shown when input is empty (and the label is
    /// floating, if any).
    pub placeholder: Option<String>,
    /// Optional leading icon shown before the input text.
    pub leading_icon: Option<RenderSlot>,
    /// Optional trailing icon shown after the input text.
    pub trailing_icon: Option<RenderSlot>,
    /// Optional prefix content shown before the input text.
    pub prefix: Option<RenderSlot>,
    /// Optional suffix content shown after the input text.
    pub suffix: Option<RenderSlot>,
    /// Whether to show the filled-style indicator line.
    pub show_indicator: bool,
    /// Input line limit policy.
    pub line_limit: TextFieldLineLimit,
    /// Context menu configuration.
    pub context_menu: TextFieldContextMenu,
    /// Optional transform applied to text changes before on_change.
    pub input_transform: Option<CallbackWith<String, String>>,
    /// Optional obfuscation character for secure fields.
    pub obfuscation_char: Option<char>,
    /// Optional transform applied only for display.
    pub display_transform: Option<DisplayTransform>,
    /// Optional external controller for text, cursor, and selection state.
    ///
    /// When this is `None`, `text_field` creates and owns an internal
    /// controller.
    pub controller: Option<State<TextInputController>>,
}

impl TextFieldBuilder {
    /// Creates filled text field defaults.
    pub fn filled() -> Self {
        text_field()
    }

    /// Creates outlined text field defaults.
    pub fn outlined() -> Self {
        text_field()
            .background_color(Color::TRANSPARENT)
            .border_width(TextFieldDefaults::OUTLINED_BORDER_WIDTH)
            .focus_border_width(TextFieldDefaults::OUTLINED_FOCUS_BORDER_WIDTH)
            .focus_background_color(Color::TRANSPARENT)
    }

    /// Creates secure text field defaults.
    pub fn secure() -> Self {
        text_field()
            .obfuscation_char(TextFieldDefaults::OBFUSCATION_CHAR)
            .line_limit(TextFieldLineLimit::SingleLine)
            .context_menu(TextFieldContextMenu::secure_default())
    }

    /// Creates outlined secure text field defaults.
    pub fn outlined_secure() -> Self {
        Self::outlined()
            .obfuscation_char(TextFieldDefaults::OBFUSCATION_CHAR)
            .line_limit(TextFieldLineLimit::SingleLine)
            .context_menu(TextFieldContextMenu::secure_default())
    }
}

impl Default for TextFieldProps {
    fn default() -> Self {
        let shape = filled_container_shape(&MaterialShapes::default());
        Self {
            enabled: true,
            read_only: false,
            is_error: false,
            modifier: Modifier::new(),
            on_change: CallbackWith::identity(),
            on_submit: Callback::noop(),
            min_width: Some(TextFieldDefaults::MIN_WIDTH),
            min_height: Some(TextFieldDefaults::MIN_HEIGHT),
            background_color: None,
            border_width: Dp(0.0),
            border_color: None,
            shape,
            padding: TextFieldDefaults::CONTENT_PADDING,
            focus_border_color: None,
            focus_border_width: None,
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
            label: None,
            placeholder: None,
            leading_icon: None,
            trailing_icon: None,
            prefix: None,
            suffix: None,
            show_indicator: true,
            line_limit: TextFieldLineLimit::SingleLine,
            context_menu: TextFieldContextMenu::default(),
            input_transform: None,
            obfuscation_char: None,
            display_transform: None,
            controller: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextFieldMenuAction {
    Cut,
    Copy,
    Paste,
    SelectAll,
}

fn obfuscate_text(value: &str, mask: char) -> String {
    value
        .chars()
        .map(|ch| if ch == '\n' || ch == '\r' { ch } else { mask })
        .collect()
}

fn filled_container_shape(shapes: &MaterialShapes) -> Shape {
    match shapes.extra_small {
        Shape::RoundedRectangle {
            top_left,
            top_right,
            ..
        } => Shape::RoundedRectangle {
            top_left,
            top_right,
            bottom_right: RoundedCorner::ZERO,
            bottom_left: RoundedCorner::ZERO,
        },
        other => other,
    }
}

fn apply_single_line(value: String) -> String {
    value.replace(['\n', '\r'], "")
}

fn merge_input_transforms(
    line_limit: TextFieldLineLimit,
    input_transform: Option<CallbackWith<String, String>>,
) -> Option<CallbackWith<String, String>> {
    let line_transform = match line_limit {
        TextFieldLineLimit::SingleLine => Some(CallbackWith::new(apply_single_line)),
        TextFieldLineLimit::MultiLine => None,
    };

    match (line_transform, input_transform) {
        (None, None) => None,
        (Some(transform), None) => Some(transform),
        (None, Some(transform)) => Some(transform),
        (Some(line_transform), Some(input_transform)) => Some(CallbackWith::new(move |value| {
            let value = input_transform.call(value);
            line_transform.call(value)
        })),
    }
}

fn build_editor_args(
    args: &TextFieldProps,
    input_transform: Option<CallbackWith<String, String>>,
    display_transform: Option<DisplayTransform>,
) -> TextInputProps {
    TextInputProps {
        enabled: args.enabled,
        read_only: args.read_only,
        is_error: args.is_error,
        modifier: args.modifier.clone(),
        on_change: args.on_change,
        on_submit: args.on_submit,
        min_width: args.min_width,
        min_height: args.min_height,
        background_color: args.background_color,
        border_width: args.border_width,
        border_color: args.border_color,
        shape: args.shape,
        padding: args.padding,
        focus_border_color: args.focus_border_color,
        focus_border_width: args.focus_border_width,
        focus_background_color: args.focus_background_color,
        selection_color: args.selection_color,
        text_color: args.text_color,
        cursor_color: args.cursor_color,
        accessibility_label: args
            .accessibility_label
            .clone()
            .or_else(|| args.label.clone().filter(|label| !label.is_empty())),
        accessibility_description: args.accessibility_description.clone(),
        initial_text: args.initial_text.clone(),
        font_size: args.font_size,
        line_height: args.line_height,
        style: args.style,
        single_line: matches!(args.line_limit, TextFieldLineLimit::SingleLine),
        input_transform,
        display_transform,
        controller: None,
    }
}

fn editor_content_len(controller: &State<TextInputController>) -> usize {
    controller.with(|c| c.text().len())
}

fn label_floating_text_style(theme: &MaterialTheme) -> TextStyle {
    theme.typography.body_small
}

fn label_resting_text_style(theme: &MaterialTheme) -> TextStyle {
    theme.typography.body_large
}

fn placeholder_text_style(theme: &MaterialTheme) -> TextStyle {
    theme.typography.body_large
}

fn resolve_label_color(
    scheme: &MaterialColorScheme,
    focused: bool,
    is_error: bool,
    enabled: bool,
) -> Color {
    if !enabled {
        return scheme.on_surface.with_alpha(DISABLED_CONTENT_ALPHA);
    }
    if is_error {
        return scheme.error;
    }
    if focused {
        scheme.primary
    } else {
        scheme.on_surface_variant
    }
}

fn resolve_placeholder_color(scheme: &MaterialColorScheme, enabled: bool) -> Color {
    if !enabled {
        return scheme.on_surface.with_alpha(DISABLED_CONTENT_ALPHA);
    }
    scheme.on_surface_variant.with_alpha(0.7)
}

fn resolve_indicator_style(
    scheme: &MaterialColorScheme,
    focused: bool,
    is_error: bool,
    enabled: bool,
) -> (Color, Dp) {
    if !enabled {
        return (
            scheme.on_surface.with_alpha(DISABLED_CONTENT_ALPHA),
            TextFieldDefaults::INDICATOR_THICKNESS,
        );
    }
    if is_error {
        let thickness = if focused {
            TextFieldDefaults::INDICATOR_FOCUSED_THICKNESS
        } else {
            TextFieldDefaults::INDICATOR_THICKNESS
        };
        return (scheme.error, thickness);
    }
    if focused {
        (
            scheme.primary,
            TextFieldDefaults::INDICATOR_FOCUSED_THICKNESS,
        )
    } else {
        (
            scheme.on_surface_variant,
            TextFieldDefaults::INDICATOR_THICKNESS,
        )
    }
}

fn resolve_container_color(
    args: &TextFieldProps,
    scheme: &MaterialColorScheme,
    focused: bool,
) -> Color {
    if !args.enabled {
        return scheme.on_surface.with_alpha(DISABLED_FILL_ALPHA);
    }
    if focused {
        args.focus_background_color
            .or(args.background_color)
            .unwrap_or(scheme.surface)
    } else {
        args.background_color.unwrap_or(scheme.surface_variant)
    }
}

fn resolve_border_width(args: &TextFieldProps, focused: bool) -> Dp {
    if focused {
        args.focus_border_width.unwrap_or(args.border_width)
    } else {
        args.border_width
    }
}

fn resolve_text_field_menu_policy(
    mut menu: TextFieldContextMenu,
    enabled: bool,
    read_only: bool,
) -> TextFieldContextMenu {
    if !enabled {
        return TextFieldContextMenu::disabled();
    }
    if !menu.enabled {
        return menu;
    }
    if read_only {
        menu.allow_cut = false;
        menu.allow_paste = false;
    }
    menu
}

fn field_text_content_origin(args: &TextFieldProps, focused: bool) -> PxPosition {
    let padding_px: Px = args.padding.into();
    let border_width_px = Px(resolve_border_width(args, focused).to_pixels_u32() as i32);
    PxPosition::new(padding_px + border_width_px, padding_px + border_width_px)
}

fn field_click_pointer_position(
    cursor_pos: PxPosition,
    size: ComputedData,
    args: &TextFieldProps,
    controller: &State<TextInputController>,
) -> PxPosition {
    let text_origin =
        field_text_content_origin(args, controller.with(|c| c.focus_handler().is_focused()));
    let (horizontal_scroll, vertical_scroll) =
        controller.with(|s| (s.scroll_state().horizontal(), s.scroll_state().vertical()));
    let viewport_width = (size.width.0 as f32 - text_origin.x - text_origin.x).max(0.0);
    let viewport_height = (size.height.0 as f32 - text_origin.y - text_origin.y).max(0.0);
    let text_relative_x = (cursor_pos.x - text_origin.x).max(0.0).min(viewport_width);
    let text_relative_y = (cursor_pos.y - text_origin.y).max(0.0).min(viewport_height);
    PxPosition::from_f32_xy(
        text_relative_x + horizontal_scroll,
        text_relative_y + vertical_scroll,
    )
}
#[derive(Clone, PartialEq)]
struct OutlinedFloatingLabelLayout {
    label_offset: PxPosition,
    notch_padding: Px,
    notch_vertical_padding: Px,
}

impl tessera_ui::LayoutModifierNode for OutlinedFloatingLabelLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        debug_assert_eq!(
            children.len(),
            2,
            "OutlinedFloatingLabel expects exactly two children"
        );
        let notch = children[0];
        let label = children[1];

        let parent_constraint = input.parent_constraint().without_min();
        let label_measurement = label.measure(input, &parent_constraint)?;
        let notch_width = label_measurement.width + self.notch_padding * 2;
        let notch_height = label_measurement.height + self.notch_vertical_padding * 2;
        let notch_constraint = Constraint::exact(notch_width, notch_height);
        let _ = notch.measure(input, &notch_constraint)?;

        let notch_position = PxPosition::from_f32_xy(
            self.label_offset.x - self.notch_padding.0 as f32,
            self.label_offset.y - self.notch_vertical_padding.0 as f32,
        );
        result.place_child(notch, notch_position);
        result.place_child(label, self.label_offset);

        Ok(result.with_size(ComputedData {
            width: Px(0),
            height: Px(0),
        }))
    }
}

#[tessera]
fn outlined_floating_label(
    label_text: Option<String>,
    label_color: Option<Color>,
    label_font_size: Option<Dp>,
    label_line_height: Option<Dp>,
    label_offset_x: Option<Dp>,
    label_offset_y: Option<Dp>,
    notch_fill_color: Option<Color>,
    notch_padding: Option<Dp>,
    notch_vertical_padding: Option<Dp>,
) {
    let typography = use_context::<MaterialTheme>().unwrap().get().typography;
    let label_text = label_text.unwrap_or_default();
    let label_color = label_color.unwrap_or(Color::TRANSPARENT);
    let label_font_size = label_font_size.unwrap_or(Dp(0.0));
    let label_line_height = label_line_height.unwrap_or(Dp(0.0));
    let label_offset_x = label_offset_x.unwrap_or(Dp(0.0));
    let label_offset_y = label_offset_y.unwrap_or(Dp(0.0));
    let notch_fill_color = notch_fill_color.unwrap_or(Color::TRANSPARENT);
    let notch_padding = notch_padding.unwrap_or(Dp(0.0));
    let notch_vertical_padding = notch_vertical_padding.unwrap_or(Dp(0.0));
    layout()
        .policy(OutlinedFloatingLabelLayout {
            label_offset: PxPosition::new(label_offset_x.into(), label_offset_y.into()),
            notch_padding: notch_padding.into(),
            notch_vertical_padding: notch_vertical_padding.into(),
        })
        .child(move || {
            spacer().modifier(Modifier::new().background(notch_fill_color));
            text()
                .content(label_text.clone())
                .color(label_color)
                .style(TextStyle {
                    font_size: label_font_size,
                    line_height: Some(label_line_height),
                    ..typography.body_small
                });
        });
}

type FieldContentData = (TextFieldProps, TextInputProps, MaterialTheme, bool, bool);

#[tessera]
fn text_field_content(
    args: Option<TextFieldProps>,
    controller: Option<State<TextInputController>>,
    editor: Option<TextInputProps>,
) {
    let args = args.expect("text_field_content requires args");
    let controller = controller.expect("text_field_content requires controller");
    let editor = editor.expect("text_field_content requires editor");
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let focused = controller.with(|controller| controller.focus_handler().is_focused());
    let empty = editor_content_len(&controller) == 0;
    let surface_args = create_surface_args(&editor, &controller, &theme.color_scheme);
    let color = args.text_color.unwrap_or(if args.enabled {
        theme.color_scheme.on_surface
    } else {
        theme
            .color_scheme
            .on_surface
            .with_alpha(DISABLED_CONTENT_ALPHA)
    });
    let next = (args.clone(), editor, theme, focused, empty);
    let data = remember(|| next.clone());
    if data.with(|current| current != &next) {
        data.set(next);
    }
    surface()
        .style(surface_args.style)
        .shape(surface_args.shape)
        .block_input(!args.enabled)
        .modifier(surface_args.modifier)
        .content_color(color)
        .child(move || {
            text_field_body(data, controller);
        });
}

#[tessera]
fn text_field_body(data: State<FieldContentData>, controller: State<TextInputController>) {
    let (args, _, theme, focused, empty) = data.get();
    let outlined = resolve_border_width(&args, focused).0 > 0.0;
    let floating = args.label.is_some() && (focused || !empty);
    let affixes = args.label.is_none() || floating;
    let padding = args.padding;
    let icon_inset = Dp(36.0 + TextFieldDefaults::ICON_TEXT_PADDING.0);
    let left = if args.leading_icon.is_some() {
        icon_inset
    } else {
        padding
    };
    let right = if args.trailing_icon.is_some() {
        icon_inset
    } else {
        padding
    };
    let label_style = label_floating_text_style(&theme);
    let line_height = label_style
        .line_height
        .unwrap_or(Dp(label_style.font_size.0 * 1.2));
    let top = if !outlined && floating {
        Dp(padding
            .0
            .max(TextFieldDefaults::FLOATING_LABEL_Y_OFFSET.0 + line_height.0))
    } else {
        padding
    };
    let bottom = Dp((padding.0 * 2.0 - top.0).max(0.0));
    let decoration = if args.enabled {
        theme.color_scheme.on_surface_variant
    } else {
        theme
            .color_scheme
            .on_surface
            .with_alpha(DISABLED_CONTENT_ALPHA)
    };
    boxed().children(move || {
        row()
            .modifier(
                Modifier::new()
                    .fill_max_height()
                    .padding(Padding::new(left, top, right, bottom)),
            )
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                let (args, _, _, _, _) = data.get();
                if affixes && let Some(prefix) = args.prefix {
                    text_field_slot(prefix, decoration);
                    spacer()
                        .modifier(Modifier::new().width(TextFieldDefaults::PREFIX_SUFFIX_PADDING));
                }
                boxed()
                    .modifier(Modifier::new().weight(1.0))
                    .children(move || {
                        let (args, mut editor, theme, focused, empty) = data.get();
                        editor.modifier = Modifier::new().fill_max_size();
                        editor.padding = Dp::ZERO;
                        editor.border_width = Dp::ZERO;
                        editor.focus_border_width = Some(Dp::ZERO);
                        text_input_core(&editor, controller);
                        let floating = args.label.is_some() && (focused || !empty);
                        if empty
                            && (args.label.is_none() || floating)
                            && let Some(placeholder) = args.placeholder
                        {
                            text()
                                .content(placeholder)
                                .color(resolve_placeholder_color(&theme.color_scheme, args.enabled))
                                .style(placeholder_text_style(&theme));
                        }
                        if !floating && let Some(label) = args.label {
                            text()
                                .content(label)
                                .color(resolve_label_color(
                                    &theme.color_scheme,
                                    focused,
                                    args.is_error,
                                    args.enabled,
                                ))
                                .style(label_resting_text_style(&theme));
                        }
                    });
                let (args, _, _, _, _) = data.get();
                if affixes && let Some(suffix) = args.suffix {
                    spacer()
                        .modifier(Modifier::new().width(TextFieldDefaults::PREFIX_SUFFIX_PADDING));
                    text_field_slot(suffix, decoration);
                }
            });
        let (args, _, theme, focused, _) = data.get();
        if let Some(content) = args.leading_icon {
            boxed()
                .modifier(
                    Modifier::new()
                        .align(Alignment::CenterStart)
                        .width(Dp(48.0))
                        .fill_max_height(),
                )
                .alignment(Alignment::Center)
                .children(move || {
                    text_field_slot(content, decoration);
                });
        }
        if let Some(content) = args.trailing_icon {
            let color = if args.is_error && args.enabled {
                theme.color_scheme.error
            } else {
                decoration
            };
            boxed()
                .modifier(
                    Modifier::new()
                        .align(Alignment::CenterEnd)
                        .width(Dp(48.0))
                        .fill_max_height(),
                )
                .alignment(Alignment::Center)
                .children(move || {
                    text_field_slot(content, color);
                });
        }
        if floating && let Some(label) = args.label.clone() {
            let color =
                resolve_label_color(&theme.color_scheme, focused, args.is_error, args.enabled);
            if outlined {
                let container = resolve_container_color(&args, &theme.color_scheme, focused);
                outlined_floating_label()
                    .label_text(label)
                    .label_color(color)
                    .label_font_size(label_style.font_size)
                    .label_line_height(line_height)
                    .label_offset_x(padding)
                    .label_offset_y(Dp(-line_height.0 * 0.5))
                    .notch_fill_color(if container.a <= 0.0 {
                        theme.color_scheme.surface
                    } else {
                        container
                    })
                    .notch_padding(TextFieldDefaults::OUTLINED_LABEL_PADDING)
                    .notch_vertical_padding(TextFieldDefaults::OUTLINED_NOTCH_VERTICAL_PADDING);
            } else {
                text()
                    .content(label)
                    .style(label_style)
                    .color(color)
                    .modifier(
                        Modifier::new().offset(left, TextFieldDefaults::FLOATING_LABEL_Y_OFFSET),
                    );
            }
        }
        if args.show_indicator && !outlined {
            let (color, thickness) =
                resolve_indicator_style(&theme.color_scheme, focused, args.is_error, args.enabled);
            horizontal_divider()
                .modifier(Modifier::new().align(Alignment::BottomStart))
                .thickness(thickness)
                .color(color);
        }
    });
}

#[tessera]
fn text_field_slot(content: RenderSlot, color: Color) {
    let next = (content, color);
    let data = remember(|| next);
    if data.get() != next {
        data.set(next);
    }
    provide_context(
        || ContentColor { current: color },
        || {
            let color = data.with(|(_, color)| *color);
            let context = use_context::<ContentColor>().expect("text field content color");
            if context.get().current != color {
                context.set(ContentColor { current: color });
            }
            data.with(|(content, _)| *content).render();
        },
    );
}

fn apply_menu_action(
    action: TextFieldMenuAction,
    controller: State<TextInputController>,
    on_change: CallbackWith<String, String>,
    input_transform: Option<CallbackWith<String, String>>,
    enabled: bool,
    read_only: bool,
) {
    if !enabled {
        return;
    }
    match action {
        TextFieldMenuAction::Copy => {
            controller.with(|c| {
                c.copy_selection_to_clipboard();
            });
        }
        TextFieldMenuAction::Cut => {
            if read_only {
                return;
            }
            controller.with_mut(|c| {
                c.cut_selection_with_pipeline(on_change, input_transform);
            });
        }
        TextFieldMenuAction::Paste => {
            if read_only {
                return;
            }
            controller.with_mut(|c| {
                c.paste_from_clipboard_with_pipeline(on_change, input_transform);
            });
        }
        TextFieldMenuAction::SelectAll => {
            controller.with_mut(|c| c.select_all());
        }
    }
}

#[tessera]
fn text_field_menu_content(
    controller: Option<State<TextInputController>>,
    menu: Option<TextFieldContextMenu>,
    action_state: Option<State<Option<TextFieldMenuAction>>>,
) {
    let controller = controller.expect("text_field_menu_content requires controller to be set");
    let menu = menu.expect("text_field_menu_content requires menu policy to be set");
    let action_state =
        action_state.expect("text_field_menu_content requires action state to be set");
    if !menu.enabled {
        return;
    }

    let selection_available = controller.with(|c| c.has_selection());

    if menu.allow_cut {
        menu_item()
            .label("Cut")
            .enabled(selection_available)
            .on_click(move || {
                action_state.set(Some(TextFieldMenuAction::Cut));
            });
    }
    if menu.allow_copy {
        menu_item()
            .label("Copy")
            .enabled(selection_available)
            .on_click(move || {
                action_state.set(Some(TextFieldMenuAction::Copy));
            });
    }
    if menu.allow_paste {
        menu_item().label("Paste").enabled(true).on_click(move || {
            action_state.set(Some(TextFieldMenuAction::Paste));
        });
    }
    if menu.allow_select_all {
        menu_item()
            .label("Select All")
            .enabled(true)
            .on_click(move || {
                action_state.set(Some(TextFieldMenuAction::SelectAll));
            });
    }
}

/// # text_field
///
/// Render a Material text field for short-form input with optional secure
/// obfuscation.
///
/// ## Usage
///
/// Capture names, email addresses, or passwords with Material defaults.
///
/// ## Parameters
///
/// - `enabled` — whether the field accepts user input.
/// - `read_only` — whether the field is read-only.
/// - `is_error` — whether the field is displaying an error state.
/// - `modifier` — optional modifier chain applied to the field container.
/// - `on_change` — optional text change callback.
/// - `on_submit` — optional submit callback for single-line fields.
/// - `min_width` — optional minimum width.
/// - `min_height` — optional minimum height.
/// - `background_color` — optional container background color.
/// - `border_width` — base border width.
/// - `border_color` — optional border color.
/// - `shape` — field container shape.
/// - `padding` — content padding.
/// - `focus_border_color` — optional focused border color.
/// - `focus_border_width` — optional focused border width.
/// - `focus_background_color` — optional focused background color.
/// - `selection_color` — optional text selection highlight color.
/// - `text_color` — optional text color.
/// - `cursor_color` — optional cursor color.
/// - `accessibility_label` — accessible name; defaults to the visible label.
/// - `accessibility_description` — optional accessibility description.
/// - `initial_text` — optional initial text content.
/// - `font_size` — font size in Dp.
/// - `style` — typography override, including weight and letter spacing.
/// - `line_height` — optional line height in Dp.
/// - `label` — optional label text.
/// - `placeholder` — optional placeholder text.
/// - `leading_icon` — optional leading icon slot.
/// - `trailing_icon` — optional trailing icon slot.
/// - `prefix` — optional prefix slot.
/// - `suffix` — optional suffix slot.
/// - `show_indicator` — whether filled-style indicator is rendered.
/// - `line_limit` — text line limit policy.
/// - `context_menu` — context menu policy.
/// - `input_transform` — optional input transform callback.
/// - `obfuscation_char` — optional obfuscation character.
/// - `display_transform` — optional display transform.
/// - `controller` — optional external text input controller.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::text_field::text_field;
/// use tessera_ui::CallbackWith;
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// let transform = CallbackWith::new(|value: String| value.to_uppercase());
/// assert_eq!(transform.call("hello".to_string()), "HELLO");
/// text_field().input_transform_shared(transform);
/// # });
/// # }
/// # component();
/// ```
#[tessera]
pub fn text_field(
    enabled: Option<bool>,
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
    #[prop(into)] label: Option<String>,
    #[prop(into)] placeholder: Option<String>,
    leading_icon: Option<RenderSlot>,
    trailing_icon: Option<RenderSlot>,
    prefix: Option<RenderSlot>,
    suffix: Option<RenderSlot>,
    show_indicator: Option<bool>,
    line_limit: Option<TextFieldLineLimit>,
    context_menu: Option<TextFieldContextMenu>,
    input_transform: Option<CallbackWith<String, String>>,
    obfuscation_char: Option<char>,
    #[prop(skip_setter)] display_transform: Option<DisplayTransform>,
    controller: Option<State<TextInputController>>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let scheme = theme.color_scheme;
    let selection_colors = TextSelectionColors::from_scheme(&scheme);
    let enabled = enabled.unwrap_or(true);
    let read_only = read_only.unwrap_or(false);
    let is_error = is_error.unwrap_or(false);
    let modifier = modifier.unwrap_or_default();
    let min_width = min_width.or(Some(TextFieldDefaults::MIN_WIDTH));
    let min_height = min_height.or(Some(TextFieldDefaults::MIN_HEIGHT));
    let background_color = background_color.or(Some(scheme.surface_container_highest));
    let border_width = border_width.unwrap_or(TextFieldProps::default().border_width);
    let shape = shape.unwrap_or_else(|| {
        if border_width.0 > 0.0 {
            theme.shapes.extra_small
        } else {
            filled_container_shape(&theme.shapes)
        }
    });
    let padding = padding.unwrap_or(TextFieldProps::default().padding);
    let focus_border_color = focus_border_color.or(Some(scheme.primary));
    let focus_background_color = focus_background_color.or(Some(scheme.surface_container_highest));
    let selection_color = selection_color.or(Some(selection_colors.background));
    let text_color = text_color.or(Some(scheme.on_surface));
    let cursor_color = cursor_color.or(Some(scheme.primary));
    let style = style.unwrap_or(theme.typography.body_large);
    let font_size = font_size.unwrap_or(style.font_size);
    let line_height = line_height.or(style.line_height);
    let show_indicator = show_indicator.unwrap_or(true);
    let line_limit = line_limit.unwrap_or(TextFieldProps::default().line_limit);
    let context_menu = context_menu.unwrap_or(TextFieldProps::default().context_menu);
    let args = TextFieldProps {
        enabled,
        read_only,
        is_error,
        modifier,
        on_change: on_change.unwrap_or_else(CallbackWith::identity),
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
        label,
        placeholder,
        leading_icon,
        trailing_icon,
        prefix,
        suffix,
        show_indicator,
        line_limit,
        context_menu,
        input_transform,
        obfuscation_char,
        display_transform,
        controller,
    };
    let controller = args.controller.unwrap_or_else(|| {
        remember(|| {
            let mut controller = TextInputController::new(args.font_size, args.line_height);
            if let Some(text) = &args.initial_text {
                controller.set_text(text);
            }
            controller
        })
    });
    let mut args = args.clone();
    args.controller = Some(controller);

    let enabled = args.enabled;
    let read_only = args.read_only;
    let menu_controller = remember(MenuController::new);
    let action_state = remember(|| None::<TextFieldMenuAction>);
    let input_transform = merge_input_transforms(args.line_limit, args.input_transform);
    let display_transform = if let Some(mask) = args.obfuscation_char {
        Some(CallbackWith::new(move |value: String| {
            DisplayTransformText::from_strings(&value, obfuscate_text(&value, mask))
        }) as DisplayTransform)
    } else {
        args.display_transform
    };
    let editor_args = build_editor_args(&args, input_transform, display_transform);
    let menu_policy = resolve_text_field_menu_policy(args.context_menu, enabled, read_only);
    let on_change = args.on_change;
    let render_args = args.clone();
    let focus_tap_recognizer = remember(TapRecognizer::default);
    let scroll_recognizer = remember(ScrollRecognizer::default);
    let context_menu_tap_recognizer = remember(|| {
        TapRecognizer::new(TapSettings {
            button: PressKeyEventType::Right,
            ..Default::default()
        })
    });
    let modifier_base = if enabled {
        Modifier::new().hover_cursor_icon(winit::window::CursorIcon::Text)
    } else {
        Modifier::new()
    };
    let modifier = with_pointer_input(modifier_base, move |mut input| {
        let cursor_pos = input.cursor_position_rel;
        let is_inside = cursor_pos
            .map(|pos| is_position_inside_bounds(input.computed_data, pos))
            .unwrap_or(false);
        let context_menu_tap_result = context_menu_tap_recognizer.with_mut(|recognizer| {
            recognizer.update(input.pass, input.pointer_changes.as_mut_slice(), is_inside)
        });
        let focus_tap_result = focus_tap_recognizer.with_mut(|recognizer| {
            recognizer.update(input.pass, input.pointer_changes.as_mut_slice(), is_inside)
        });

        if menu_policy.enabled {
            if let Some(action) = action_state.with_mut(|state| state.take()) {
                apply_menu_action(
                    action,
                    controller,
                    on_change,
                    input_transform,
                    enabled,
                    read_only,
                );
            }

            if context_menu_tap_result.tapped
                && let Some(cursor_pos) = cursor_pos
            {
                menu_controller.with_mut(|menu| {
                    menu.open_at(MenuAnchor::at(cursor_pos));
                });
            }
        }

        if enabled {
            if focus_tap_result.pressed
                && let Some(cursor_pos) = cursor_pos
                && is_inside
            {
                let press_timestamp = focus_tap_result.press_timestamp;
                controller.with_mut(|c| c.focus_handler_mut().request_focus());
                let inner_handled_click = press_timestamp
                    .map(|timestamp| controller.with(|c| c.last_click_time() == Some(timestamp)))
                    .unwrap_or(false);
                if !inner_handled_click {
                    let click_position = field_click_pointer_position(
                        cursor_pos,
                        input.computed_data,
                        &args,
                        &controller,
                    );
                    controller.with_mut(|c| {
                        if input.key_modifiers.shift_key() {
                            c.extend_selection_to_point(click_position);
                        } else {
                            c.apply_pointer_action(GlyphonAction::Click {
                                x: click_position.x_px().0,
                                y: click_position.y_px().0,
                            });
                        }
                    });
                }
            }
        }

        if enabled && is_inside {
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
                        if controller.with(|c| c.single_line()) {
                            let requested = if dx.abs() > f32::EPSILON { dx } else { dy };
                            if requested.abs() <= f32::EPSILON
                                || !controller.with_mut(|c| c.has_horizontal_overflow())
                            {
                                return;
                            }
                            let consumed =
                                controller.with_mut(|c| c.scroll_horizontal_consumed(-requested));
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
                                || !controller.with_mut(|c| c.has_vertical_overflow())
                            {
                                return;
                            }
                            // The editor engine clamps vertical scroll to the
                            // content extent, so `consumed` is the exact amount
                            // applied and the remainder goes to an ancestor.
                            let consumed = controller.with_mut(|c| c.scroll_vertical_consumed(-dy));
                            event.delta_y = dy + consumed;
                        }
                        event.unit = ScrollDeltaUnit::Pixel;
                    },
                );
            });
        }

        if is_inside {
            input.block_cursor();
        }
    });

    layout().modifier(modifier).child(move || {
        if menu_policy.enabled {
            let render_args = render_args.clone();
            let editor_args = editor_args.clone();
            menu_provider()
                .placement(MenuPlacement::BelowStart)
                .offset([Dp(0.0), Dp(4.0)])
                .controller(menu_controller)
                .main_content(move || {
                    text_field_content()
                        .args(render_args.clone())
                        .controller(controller)
                        .editor(editor_args.clone());
                })
                .menu_content(move || {
                    text_field_menu_content()
                        .controller(controller)
                        .menu(menu_policy)
                        .action_state(action_state);
                });
        } else {
            text_field_content()
                .args(render_args.clone())
                .controller(controller)
                .editor(editor_args.clone());
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::theme::MaterialColorScheme;

    use super::{
        TextFieldDefaults, resolve_indicator_style, resolve_label_color, resolve_placeholder_color,
    };

    fn scheme() -> MaterialColorScheme {
        MaterialColorScheme::default()
    }

    #[test]
    fn label_color_follows_the_material_states() {
        let scheme = scheme();

        assert_eq!(
            resolve_label_color(&scheme, false, false, true),
            scheme.on_surface_variant
        );
        assert_eq!(
            resolve_label_color(&scheme, true, false, true),
            scheme.primary
        );
        assert_eq!(
            resolve_label_color(&scheme, false, true, true),
            scheme.error
        );
        assert_eq!(
            resolve_label_color(&scheme, false, false, false),
            scheme
                .on_surface
                .with_alpha(crate::text_input::DISABLED_CONTENT_ALPHA)
        );
    }

    #[test]
    fn placeholder_color_dims_when_disabled() {
        let scheme = scheme();

        assert_eq!(
            resolve_placeholder_color(&scheme, true),
            scheme.on_surface_variant.with_alpha(0.7)
        );
        assert_eq!(
            resolve_placeholder_color(&scheme, false),
            scheme
                .on_surface
                .with_alpha(crate::text_input::DISABLED_CONTENT_ALPHA)
        );
    }

    #[test]
    fn indicator_style_follows_the_material_states() {
        let scheme = scheme();

        assert_eq!(
            resolve_indicator_style(&scheme, false, false, true),
            (
                scheme.on_surface_variant,
                TextFieldDefaults::INDICATOR_THICKNESS
            )
        );
        assert_eq!(
            resolve_indicator_style(&scheme, true, false, true),
            (
                scheme.primary,
                TextFieldDefaults::INDICATOR_FOCUSED_THICKNESS
            )
        );
        assert_eq!(
            resolve_indicator_style(&scheme, true, true, true),
            (scheme.error, TextFieldDefaults::INDICATOR_FOCUSED_THICKNESS)
        );
        assert_eq!(
            resolve_indicator_style(&scheme, true, false, false),
            (
                scheme
                    .on_surface
                    .with_alpha(crate::text_input::DISABLED_CONTENT_ALPHA),
                TextFieldDefaults::INDICATOR_THICKNESS
            )
        );
    }
    thread_local! {
        static FIELD_CONTROLLER: std::cell::Cell<Option<tessera_ui::State<crate::text_input::TextInputController>>> = const { std::cell::Cell::new(None) };
    }

    #[tessera_ui::tessera]
    fn pointer_focus_fixture() {
        let controller = tessera_ui::remember(|| {
            crate::text_input::TextInputController::new(
                tessera_ui::Dp(16.0),
                Some(tessera_ui::Dp(24.0)),
            )
        });
        FIELD_CONTROLLER.set(Some(controller));
        crate::theme::material_theme().child(move || {
            crate::column::column().children(move || {
                super::TextFieldBuilder::outlined()
                    .label("Date")
                    .controller(controller);
            });
        });
    }

    #[test]
    fn clicking_an_empty_outlined_field_focuses_its_editor() {
        use tessera_ui::{
            CursorEventContent, GestureState, PointerChange, PressKeyEventType, PxPosition,
            testing::{
                layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
                start_layout_test_session,
            },
        };
        let mut session = start_layout_test_session(
            layout_test(|| {
                pointer_focus_fixture();
            })
            .viewport_px(400, 200),
        );
        pump_layout_test_session(&mut session, 0);
        for (index, content) in [
            CursorEventContent::Pressed(PressKeyEventType::Left),
            CursorEventContent::Released(PressKeyEventType::Left),
        ]
        .into_iter()
        .enumerate()
        {
            pump_layout_test_session_with_input(
                &mut session,
                (index as u64 + 1) * 16_666_667,
                vec![PointerChange::new(
                    tessera_ui::time::Instant::now(),
                    1,
                    content,
                    GestureState::TapCandidate,
                )],
                Some(PxPosition::from_f32_xy(100.0, 28.0)),
            );
        }
        pump_layout_test_session(&mut session, 50_000_001);
        assert!(
            FIELD_CONTROLLER
                .get()
                .unwrap()
                .with(|c| c.focus_handler().is_focused())
        );
    }
}
