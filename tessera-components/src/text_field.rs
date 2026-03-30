//! Material text fields for filled, outlined, and secure input.
//!
//! ## Usage
//!
//! Collect short-form inputs like names, passwords, or search queries.
use glyphon::Action as GlyphonAction;
use tessera_ui::{
    Callback, CallbackWith, Color, ComputedData, Constraint, DimensionValue, Dp, LayoutInput,
    LayoutOutput, LayoutPolicy, MeasurementError, Modifier, PressKeyEventType, Px, PxPosition,
    RenderSlot, State, layout::layout_primitive, modifier::CursorModifierExt as _, provide_context,
    remember, tessera, use_context, winit,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    divider::horizontal_divider,
    gesture_recognizer::{TapRecognizer, TapSettings},
    menus::{MenuAnchor, MenuController, MenuPlacement, menu_item, menu_provider},
    modifier::{ModifierExt as _, Padding, with_pointer_input},
    pos_misc::is_position_inside_bounds,
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    text::text,
    text_edit_core::DisplayTransform,
    text_input::{
        DisplayTransformText, TextInputController, TextInputProps, create_surface_args,
        text_input_core,
    },
    theme::{ContentColor, MaterialColorScheme, MaterialTheme, TextSelectionColors},
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
    /// Default vertical spacing between label and input text.
    pub const LABEL_BOTTOM_PADDING: Dp = Dp(4.0);
    /// Vertical offset from the container top for floating labels.
    pub const FLOATING_LABEL_Y_OFFSET: Dp = Dp(4.0);
    /// Horizontal padding applied around outlined floating labels.
    pub const OUTLINED_LABEL_PADDING: Dp = Dp(2.0);
    /// Extra height applied to outlined label notches.
    pub const OUTLINED_NOTCH_VERTICAL_PADDING: Dp = Dp(2.0);
    /// Horizontal padding between icon slots and the text content.
    pub const ICON_TEXT_PADDING: Dp = Dp(12.0);
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
#[derive(Clone)]
struct TextFieldProps {
    /// Whether the text field is enabled for user input.
    pub enabled: bool,
    /// Whether the text field is read-only.
    pub read_only: bool,
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
    /// Optional label announced by assistive technologies.
    pub accessibility_label: Option<String>,
    /// Optional description announced by assistive technologies.
    pub accessibility_description: Option<String>,
    /// Initial text content.
    pub initial_text: Option<String>,
    /// Font size in Dp.
    pub font_size: Dp,
    /// Line height in Dp.
    pub line_height: Option<Dp>,
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
    /// Set the input line limit policy.
    pub fn line_limit(mut self, line_limit: TextFieldLineLimit) -> Self {
        self.props.line_limit = line_limit;
        self
    }

    /// Set the context menu policy.
    pub fn context_menu(mut self, context_menu: TextFieldContextMenu) -> Self {
        self.props.context_menu = context_menu;
        self
    }

    /// Sets an external text input controller.
    pub fn controller(mut self, controller: State<TextInputController>) -> Self {
        self.props.controller = Some(controller);
        self
    }

    /// Set the obfuscation character for secure fields.
    pub fn obfuscation_char(mut self, obfuscation_char: char) -> Self {
        self.props.obfuscation_char = Some(obfuscation_char);
        self
    }

    /// Creates filled text field defaults.
    pub fn filled() -> Self {
        text_field()
    }

    /// Creates outlined text field defaults.
    pub fn outlined() -> Self {
        let theme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get();
        let scheme = theme.color_scheme;
        let selection_colors = TextSelectionColors::from_scheme(&scheme);
        if let Some(line_height) = theme.typography.body_large.line_height {
            text_field()
                .background_color(Color::TRANSPARENT)
                .border_width(TextFieldDefaults::OUTLINED_BORDER_WIDTH)
                .border_color(scheme.outline)
                .shape(theme.shapes.extra_small)
                .focus_border_color(scheme.primary)
                .focus_border_width(TextFieldDefaults::OUTLINED_FOCUS_BORDER_WIDTH)
                .focus_background_color(Color::TRANSPARENT)
                .selection_color(selection_colors.background)
                .text_color(scheme.on_surface)
                .cursor_color(scheme.primary)
                .font_size(theme.typography.body_large.font_size)
                .line_height(line_height)
        } else {
            text_field()
                .background_color(Color::TRANSPARENT)
                .border_width(TextFieldDefaults::OUTLINED_BORDER_WIDTH)
                .border_color(scheme.outline)
                .shape(theme.shapes.extra_small)
                .focus_border_color(scheme.primary)
                .focus_border_width(TextFieldDefaults::OUTLINED_FOCUS_BORDER_WIDTH)
                .focus_background_color(Color::TRANSPARENT)
                .selection_color(selection_colors.background)
                .text_color(scheme.on_surface)
                .cursor_color(scheme.primary)
                .font_size(theme.typography.body_large.font_size)
        }
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
        let theme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get();
        let shape = filled_container_shape(&theme);
        let scheme = theme.color_scheme;
        let selection_colors = TextSelectionColors::from_scheme(&scheme);
        Self {
            enabled: true,
            read_only: false,
            modifier: Modifier::new(),
            on_change: CallbackWith::identity(),
            on_submit: Callback::noop(),
            min_width: Some(TextFieldDefaults::MIN_WIDTH),
            min_height: Some(TextFieldDefaults::MIN_HEIGHT),
            background_color: Some(scheme.surface_container_highest),
            border_width: Dp(0.0),
            border_color: None,
            shape,
            padding: TextFieldDefaults::CONTENT_PADDING,
            focus_border_color: Some(scheme.primary),
            focus_border_width: None,
            focus_background_color: Some(scheme.surface_container_highest),
            selection_color: Some(selection_colors.background),
            text_color: Some(scheme.on_surface),
            cursor_color: Some(scheme.primary),
            accessibility_label: None,
            accessibility_description: None,
            initial_text: None,
            font_size: theme.typography.body_large.font_size,
            line_height: theme.typography.body_large.line_height,
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

fn filled_container_shape(theme: &MaterialTheme) -> Shape {
    match theme.shapes.extra_small {
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
        accessibility_label: args.accessibility_label.clone(),
        accessibility_description: args.accessibility_description.clone(),
        initial_text: args.initial_text.clone(),
        font_size: args.font_size,
        line_height: args.line_height,
        single_line: matches!(args.line_limit, TextFieldLineLimit::SingleLine),
        input_transform,
        display_transform,
        controller: None,
    }
}

fn editor_content_len(controller: &State<TextInputController>) -> usize {
    controller.with(|c| c.text().len())
}

fn label_floating_text_style(theme: &MaterialTheme) -> (Dp, Option<Dp>) {
    let style = theme.typography.body_small;
    (style.font_size, style.line_height)
}

fn label_resting_text_style(theme: &MaterialTheme) -> (Dp, Option<Dp>) {
    let style = theme.typography.body_large;
    (style.font_size, style.line_height)
}

fn placeholder_text_style(theme: &MaterialTheme) -> (Dp, Option<Dp>) {
    let style = theme.typography.body_large;
    (style.font_size, style.line_height)
}

fn resolve_label_color(scheme: &MaterialColorScheme, focused: bool) -> Color {
    if focused {
        scheme.primary
    } else {
        scheme.on_surface_variant
    }
}

fn resolve_placeholder_color(scheme: &MaterialColorScheme) -> Color {
    scheme.on_surface_variant.with_alpha(0.7)
}

fn resolve_indicator_style(scheme: &MaterialColorScheme, focused: bool) -> (Color, Dp) {
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
    let (horizontal_scroll, vertical_scroll) = controller.with(|s| {
        (
            Px(s.scroll_state().horizontal().round() as i32),
            Px(s.scroll_state().vertical().round() as i32),
        )
    });
    let viewport_width = (size.width - text_origin.x - text_origin.x).max(Px::ZERO);
    let viewport_height = (size.height - text_origin.y - text_origin.y).max(Px::ZERO);
    let text_relative_x = (cursor_pos.x - text_origin.x)
        .max(Px::ZERO)
        .min(viewport_width);
    let text_relative_y = (cursor_pos.y - text_origin.y)
        .max(Px::ZERO)
        .min(viewport_height);
    PxPosition::new(
        text_relative_x + horizontal_scroll,
        text_relative_y + vertical_scroll,
    )
}
#[derive(Clone)]
struct OutlinedFloatingLabelArgs {
    label_text: String,
    label_color: Color,
    label_font_size: Dp,
    label_line_height: Dp,
    label_offset_x: Dp,
    label_offset_y: Dp,
    notch_fill_color: Color,
    notch_padding: Dp,
    notch_vertical_padding: Dp,
}

#[derive(Clone, PartialEq)]
struct OutlinedFloatingLabelLayout {
    label_offset: PxPosition,
    notch_padding: Px,
    notch_vertical_padding: Px,
}

impl LayoutPolicy for OutlinedFloatingLabelLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        debug_assert_eq!(
            input.children_ids().len(),
            2,
            "OutlinedFloatingLabel expects exactly two children"
        );
        let notch_id = input.children_ids()[0];
        let label_id = input.children_ids()[1];

        let parent_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );
        let label_measurement = input.measure_child(label_id, &parent_constraint)?;
        let notch_width = label_measurement.width + self.notch_padding * 2;
        let notch_height = label_measurement.height + self.notch_vertical_padding * 2;
        let notch_constraint = Constraint::new(
            DimensionValue::Fixed(notch_width),
            DimensionValue::Fixed(notch_height),
        );
        let _ = input.measure_child(notch_id, &notch_constraint)?;

        let notch_position = PxPosition::new(
            self.label_offset.x - self.notch_padding,
            self.label_offset.y - self.notch_vertical_padding,
        );
        output.place_child(notch_id, notch_position);
        output.place_child(label_id, self.label_offset);

        Ok(ComputedData {
            width: Px(0),
            height: Px(0),
        })
    }
}

#[tessera]
fn outlined_floating_label(
    label_text: String,
    label_color: Color,
    label_font_size: Dp,
    label_line_height: Dp,
    label_offset_x: Dp,
    label_offset_y: Dp,
    notch_fill_color: Color,
    notch_padding: Dp,
    notch_vertical_padding: Dp,
) {
    layout_primitive()
        .layout_policy(OutlinedFloatingLabelLayout {
            label_offset: PxPosition::new(label_offset_x.into(), label_offset_y.into()),
            notch_padding: notch_padding.into(),
            notch_vertical_padding: notch_vertical_padding.into(),
        })
        .child(move || {
            spacer().modifier(Modifier::new().background(notch_fill_color));
            text()
                .content(label_text.clone())
                .color(label_color)
                .size(label_font_size)
                .line_height(label_line_height);
        });
}

fn render_text_field(
    args: TextFieldProps,
    controller: State<TextInputController>,
    editor: TextInputProps,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let focused = controller.with(|c| c.focus_handler().is_focused());
    let is_empty = editor_content_len(&controller) == 0;
    let border_width = resolve_border_width(&args, focused);
    let show_indicator = args.show_indicator && border_width == Dp(0.0);

    let label_text = args.label.clone();
    let placeholder_text = args.placeholder.clone();
    let has_label = label_text.is_some();
    let label_should_float = has_label && (focused || !is_empty);
    let show_placeholder =
        is_empty && placeholder_text.is_some() && (!has_label || label_should_float);
    let leading_icon = args.leading_icon.clone();
    let trailing_icon = args.trailing_icon.clone();
    let prefix = args.prefix.clone();
    let suffix = args.suffix.clone();
    let content_padding = args.padding;
    let (label_floating_font_size, label_floating_line_height) = {
        let (font_size, line_height) = label_floating_text_style(&theme);
        let line_height = line_height.unwrap_or(Dp(font_size.0 * 1.2));
        (font_size, line_height)
    };
    let (label_resting_font_size, label_resting_line_height) = {
        let (font_size, line_height) = label_resting_text_style(&theme);
        let line_height = line_height.unwrap_or(Dp(font_size.0 * 1.2));
        (font_size, line_height)
    };
    let placeholder_style = placeholder_text_style(&theme);
    let scheme = theme.color_scheme;
    let label_color = resolve_label_color(&scheme, focused);
    let placeholder_color = resolve_placeholder_color(&scheme);
    let (indicator_color, indicator_thickness) = resolve_indicator_style(&scheme, focused);
    let content_color = args.text_color.unwrap_or(scheme.on_surface);
    let container_color = resolve_container_color(&args, &scheme, focused);
    let is_outlined = border_width.0 > 0.0;
    let notch_fill_color = if container_color.a <= 0.0 {
        // Use a solid fallback so the notch masks the outline on transparent
        // containers.
        scheme.surface
    } else {
        container_color
    };
    let floating_label_offset_x = Dp(0.0);
    let floating_label_offset_y = if is_outlined {
        Dp(-content_padding.0 - (label_floating_line_height.0 * 0.5))
    } else {
        Dp(TextFieldDefaults::FLOATING_LABEL_Y_OFFSET.0 - content_padding.0)
    };
    let notch_padding = TextFieldDefaults::OUTLINED_LABEL_PADDING;
    let notch_vertical_padding = TextFieldDefaults::OUTLINED_NOTCH_VERTICAL_PADDING;

    let render_editor = move || {
        let mut core_args = editor.clone();
        core_args.modifier = Modifier::new().fill_max_size();
        core_args.padding = Dp(0.0);
        core_args.border_width = Dp(0.0);
        core_args.focus_border_width = Some(Dp(0.0));

        let surface_args = create_surface_args(&editor, &controller)
            .content_color(content_color)
            .block_input(!args.enabled);
        surface_args.with_child(move || {
            let leading_icon = leading_icon.clone();
            let prefix = prefix.clone();
            let core_args = core_args.clone();
            let placeholder_text = placeholder_text.clone();
            let label_text = label_text.clone();
            let suffix = suffix.clone();
            let trailing_icon = trailing_icon.clone();
            boxed().children(move || {
                {
                    let leading_icon = leading_icon.clone();
                    let prefix = prefix.clone();
                    let core_args = core_args.clone();
                    let placeholder_text = placeholder_text.clone();
                    let label_text = label_text.clone();
                    let suffix = suffix.clone();
                    let trailing_icon = trailing_icon.clone();
                    let row_modifier = Modifier::new()
                        .fill_max_height()
                        .padding(Padding::all(content_padding));
                    row()
                        .modifier(row_modifier)
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            if let Some(leading_icon) = leading_icon.as_ref() {
                                let leading_icon = leading_icon.clone();
                                {
                                    provide_context(
                                        || ContentColor {
                                            current: content_color,
                                        },
                                        || {
                                            leading_icon.render();
                                        },
                                    );
                                };
                                let spacing = TextFieldDefaults::ICON_TEXT_PADDING;
                                {
                                    spacer().modifier(Modifier::new().width(spacing));
                                };
                            }

                            if let Some(prefix) = prefix.as_ref() {
                                let prefix = prefix.clone();
                                {
                                    provide_context(
                                        || ContentColor {
                                            current: content_color,
                                        },
                                        || {
                                            prefix.render();
                                        },
                                    );
                                };
                                let spacing = TextFieldDefaults::PREFIX_SUFFIX_PADDING;
                                {
                                    spacer().modifier(Modifier::new().width(spacing));
                                };
                            }

                            let core_args_for_box = core_args.clone();
                            let placeholder_text_for_box = placeholder_text.clone();
                            let label_text_for_box = label_text.clone();
                            boxed()
                                .modifier(Modifier::new().weight(1.0))
                                .children(move || {
                                    let core_args = core_args_for_box.clone();
                                    let placeholder_text = placeholder_text_for_box.clone();
                                    let label_text = label_text_for_box.clone();
                                    text_input_core(&core_args.clone(), controller);

                                    if show_placeholder
                                        && let Some(placeholder_text) = placeholder_text.as_ref()
                                    {
                                        let placeholder_text = placeholder_text.clone();
                                        layout_primitive()
                                            .modifier(Modifier::new().align(Alignment::TopStart))
                                            .child(move || {
                                                let (font_size, line_height) = placeholder_style;
                                                if let Some(line_height) = line_height {
                                                    text()
                                                        .content(placeholder_text.clone())
                                                        .color(placeholder_color)
                                                        .size(font_size)
                                                        .line_height(line_height);
                                                } else {
                                                    text()
                                                        .content(placeholder_text.clone())
                                                        .color(placeholder_color)
                                                        .size(font_size);
                                                }
                                            });
                                    }

                                    if let Some(label_text) = label_text.as_ref() {
                                        let label_text = label_text.clone();
                                        if label_should_float {
                                            if is_outlined {
                                                let floating_args = OutlinedFloatingLabelArgs {
                                                    label_text: label_text.clone(),
                                                    label_color,
                                                    label_font_size: label_floating_font_size,
                                                    label_line_height: label_floating_line_height,
                                                    label_offset_x: floating_label_offset_x,
                                                    label_offset_y: floating_label_offset_y,
                                                    notch_fill_color,
                                                    notch_padding,
                                                    notch_vertical_padding,
                                                };
                                                layout_primitive()
                                                    .modifier(
                                                        Modifier::new().align(Alignment::TopStart),
                                                    )
                                                    .child(move || {
                                                        let args = floating_args.clone();
                                                        outlined_floating_label()
                                                            .label_text(args.label_text)
                                                            .label_color(args.label_color)
                                                            .label_font_size(args.label_font_size)
                                                            .label_line_height(
                                                                args.label_line_height,
                                                            )
                                                            .label_offset_x(args.label_offset_x)
                                                            .label_offset_y(args.label_offset_y)
                                                            .notch_fill_color(args.notch_fill_color)
                                                            .notch_padding(args.notch_padding)
                                                            .notch_vertical_padding(
                                                                args.notch_vertical_padding,
                                                            );
                                                    });
                                            } else {
                                                layout_primitive()
                                                    .modifier(
                                                        Modifier::new().align(Alignment::TopStart),
                                                    )
                                                    .child(move || {
                                                        text()
                                                            .content(label_text.clone())
                                                            .color(label_color)
                                                            .size(label_floating_font_size)
                                                            .modifier(Modifier::new().offset(
                                                                floating_label_offset_x,
                                                                floating_label_offset_y,
                                                            ))
                                                            .line_height(
                                                                label_floating_line_height,
                                                            );
                                                    });
                                            }
                                        } else {
                                            layout_primitive()
                                                .modifier(
                                                    Modifier::new().align(Alignment::TopStart),
                                                )
                                                .child(move || {
                                                    text()
                                                        .content(label_text.clone())
                                                        .color(label_color)
                                                        .size(label_resting_font_size)
                                                        .line_height(label_resting_line_height);
                                                });
                                        }
                                    }
                                });

                            if let Some(suffix) = suffix.as_ref() {
                                let suffix = suffix.clone();
                                let spacing = TextFieldDefaults::PREFIX_SUFFIX_PADDING;
                                {
                                    spacer().modifier(Modifier::new().width(spacing));
                                };
                                {
                                    provide_context(
                                        || ContentColor {
                                            current: content_color,
                                        },
                                        || {
                                            suffix.render();
                                        },
                                    );
                                };
                            }

                            if let Some(trailing_icon) = trailing_icon.as_ref() {
                                let trailing_icon = trailing_icon.clone();
                                let spacing = TextFieldDefaults::ICON_TEXT_PADDING;
                                {
                                    spacer().modifier(Modifier::new().width(spacing));
                                };
                                {
                                    provide_context(
                                        || ContentColor {
                                            current: content_color,
                                        },
                                        || {
                                            trailing_icon.render();
                                        },
                                    );
                                };
                            }
                        });
                };

                if show_indicator {
                    layout_primitive()
                        .modifier(Modifier::new().align(Alignment::BottomStart))
                        .child(move || {
                            horizontal_divider()
                                .thickness(indicator_thickness)
                                .color(indicator_color);
                        });
                }
            });
        });
    };

    render_editor();
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

fn add_menu_item(
    label: &str,
    enabled: bool,
    action_state: State<Option<TextFieldMenuAction>>,
    action: TextFieldMenuAction,
) {
    let label = label.to_string();
    menu_item().label(label).enabled(enabled).on_click(move || {
        action_state.set(Some(action));
    });
}

fn configure_text_field_menu(
    controller: State<TextInputController>,
    menu: TextFieldContextMenu,
    action_state: State<Option<TextFieldMenuAction>>,
) {
    if !menu.enabled {
        return;
    }

    let selection_available = controller.with(|c| c.has_selection());

    if menu.allow_cut {
        add_menu_item(
            "Cut",
            selection_available,
            action_state,
            TextFieldMenuAction::Cut,
        );
    }
    if menu.allow_copy {
        add_menu_item(
            "Copy",
            selection_available,
            action_state,
            TextFieldMenuAction::Copy,
        );
    }
    if menu.allow_paste {
        add_menu_item("Paste", true, action_state, TextFieldMenuAction::Paste);
    }
    if menu.allow_select_all {
        add_menu_item(
            "Select All",
            true,
            action_state,
            TextFieldMenuAction::SelectAll,
        );
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
/// - `accessibility_label` — optional accessibility label.
/// - `accessibility_description` — optional accessibility description.
/// - `initial_text` — optional initial text content.
/// - `font_size` — font size in Dp.
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
    #[default(true)] enabled: bool,
    read_only: bool,
    modifier: Modifier,
    on_change: Option<CallbackWith<String, String>>,
    on_submit: Option<Callback>,
    #[default(TextFieldProps::default().min_width)] min_width: Option<Dp>,
    #[default(TextFieldProps::default().min_height)] min_height: Option<Dp>,
    #[default(TextFieldProps::default().background_color)] background_color: Option<Color>,
    border_width: Dp,
    border_color: Option<Color>,
    #[default(TextFieldProps::default().shape)] shape: Shape,
    #[default(TextFieldProps::default().padding)] padding: Dp,
    #[default(TextFieldProps::default().focus_border_color)] focus_border_color: Option<Color>,
    focus_border_width: Option<Dp>,
    #[default(TextFieldProps::default().focus_background_color)] focus_background_color: Option<
        Color,
    >,
    #[default(TextFieldProps::default().selection_color)] selection_color: Option<Color>,
    #[default(TextFieldProps::default().text_color)] text_color: Option<Color>,
    #[default(TextFieldProps::default().cursor_color)] cursor_color: Option<Color>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
    #[prop(into)] initial_text: Option<String>,
    #[default(TextFieldProps::default().font_size)] font_size: Dp,
    #[default(TextFieldProps::default().line_height)] line_height: Option<Dp>,
    #[prop(into)] label: Option<String>,
    #[prop(into)] placeholder: Option<String>,
    leading_icon: Option<RenderSlot>,
    trailing_icon: Option<RenderSlot>,
    prefix: Option<RenderSlot>,
    suffix: Option<RenderSlot>,
    #[default(true)] show_indicator: bool,
    #[prop(skip_setter)] line_limit: TextFieldLineLimit,
    #[prop(skip_setter)] context_menu: TextFieldContextMenu,
    input_transform: Option<CallbackWith<String, String>>,
    #[prop(skip_setter)] obfuscation_char: Option<char>,
    #[prop(skip_setter)] display_transform: Option<DisplayTransform>,
    #[prop(skip_setter)] controller: Option<State<TextInputController>>,
) {
    let args = TextFieldProps {
        enabled,
        read_only,
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
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                is_inside,
            )
        });
        let focus_tap_result = focus_tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                is_inside,
            )
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
                                x: click_position.x.0,
                                y: click_position.y.0,
                            });
                        }
                    });
                }
            }
        }

        if is_inside {
            input.block_cursor();
        }
    });

    layout_primitive().modifier(modifier).child(move || {
        if menu_policy.enabled {
            let render_args = render_args.clone();
            let editor_args = editor_args.clone();
            menu_provider()
                .placement(MenuPlacement::BelowStart)
                .offset([Dp(0.0), Dp(4.0)])
                .controller(menu_controller)
                .main_content(move || {
                    render_text_field(render_args.clone(), controller, editor_args.clone());
                })
                .menu_content(move || {
                    configure_text_field_menu(controller, menu_policy, action_state);
                });
        } else {
            render_text_field(render_args.clone(), controller, editor_args.clone());
        }
    });
}
