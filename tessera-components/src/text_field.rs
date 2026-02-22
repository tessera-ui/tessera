//! Material text fields for filled, outlined, and secure input.
//!
//! ## Usage
//!
//! Collect short-form inputs like names, passwords, or search queries.
use derive_setters::Setters;
use glyphon::{
    Action as GlyphonAction, Cursor, Edit,
    cosmic_text::{self, Selection},
};
use tessera_platform::clipboard;
use tessera_ui::{
    CallbackWith, Color, ComputedData, Constraint, CursorEventContent, DimensionValue, Dp,
    LayoutInput, LayoutOutput, LayoutSpec, MeasurementError, Modifier, PressKeyEventType, Px,
    PxPosition, RenderSlot, State, provide_context, remember, tessera, use_context, winit,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::{BoxedArgs, boxed},
    divider::{DividerArgs, horizontal_divider},
    menus::{
        MenuAnchor, MenuController, MenuItemArgs, MenuPlacement, MenuProviderArgs, MenuScope,
        menu_provider,
    },
    modifier::{ModifierExt as _, Padding},
    pipelines::text::pipeline::write_font_system,
    pos_misc::is_position_inside_bounds,
    row::{RowArgs, row},
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    text::{TextArgs, text},
    text_edit_core::DisplayTransform,
    text_input::{
        TextInputArgs, TextInputController, create_surface_args, handle_action, text_input_core,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextFieldLineLimit {
    /// Restrict input to a single line.
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
#[derive(PartialEq, Clone, Setters)]
pub struct TextFieldArgs {
    /// Whether the text field is enabled for user input.
    pub enabled: bool,
    /// Whether the text field is read-only.
    pub read_only: bool,
    /// Optional modifier chain applied to the field container.
    pub modifier: Modifier,
    /// Called when the text content changes. The closure receives the new text
    /// content and returns the updated content.
    #[setters(skip)]
    pub on_change: CallbackWith<String, String>,
    /// Minimum width in density-independent pixels.
    #[setters(strip_option)]
    pub min_width: Option<Dp>,
    /// Minimum height in density-independent pixels.
    #[setters(strip_option)]
    pub min_height: Option<Dp>,
    /// Background color of the text field.
    #[setters(strip_option)]
    pub background_color: Option<Color>,
    /// Border width in Dp.
    pub border_width: Dp,
    /// Border color of the text field.
    #[setters(strip_option)]
    pub border_color: Option<Color>,
    /// The shape of the text field container.
    pub shape: Shape,
    /// Padding inside the text field.
    pub padding: Dp,
    /// Border color when focused.
    #[setters(strip_option)]
    pub focus_border_color: Option<Color>,
    /// Border width when focused.
    #[setters(strip_option)]
    pub focus_border_width: Option<Dp>,
    /// Background color when focused.
    #[setters(strip_option)]
    pub focus_background_color: Option<Color>,
    /// Color for text selection highlight.
    #[setters(strip_option)]
    pub selection_color: Option<Color>,
    /// Color of the text content.
    #[setters(strip_option)]
    pub text_color: Option<Color>,
    /// Color of the text cursor.
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
    /// Font size in Dp.
    pub font_size: Dp,
    /// Line height in Dp.
    #[setters(strip_option)]
    pub line_height: Option<Dp>,
    /// Optional label text shown inside the field and floated when focused or
    /// non-empty.
    #[setters(strip_option, into)]
    pub label: Option<String>,
    /// Optional placeholder text shown when input is empty (and the label is
    /// floating, if any).
    #[setters(strip_option, into)]
    pub placeholder: Option<String>,
    /// Optional leading icon shown before the input text.
    #[setters(skip)]
    pub leading_icon: Option<RenderSlot>,
    /// Optional trailing icon shown after the input text.
    #[setters(skip)]
    pub trailing_icon: Option<RenderSlot>,
    /// Optional prefix content shown before the input text.
    #[setters(skip)]
    pub prefix: Option<RenderSlot>,
    /// Optional suffix content shown after the input text.
    #[setters(skip)]
    pub suffix: Option<RenderSlot>,
    /// Whether to show the filled-style indicator line.
    pub show_indicator: bool,
    /// Input line limit policy.
    #[setters(skip)]
    pub line_limit: TextFieldLineLimit,
    /// Context menu configuration.
    #[setters(skip)]
    pub context_menu: TextFieldContextMenu,
    /// Optional transform applied to text changes before on_change.
    #[setters(skip)]
    pub input_transform: Option<CallbackWith<String, String>>,
    /// Optional obfuscation character for secure fields.
    #[setters(skip)]
    pub obfuscation_char: Option<char>,
    /// Optional transform applied only for display.
    #[setters(skip)]
    pub display_transform: Option<DisplayTransform>,
    /// Optional external controller for text, cursor, and selection state.
    ///
    /// When this is `None`, `text_field` creates and owns an internal
    /// controller.
    #[setters(skip)]
    pub controller: Option<State<TextInputController>>,
}

impl TextFieldArgs {
    /// Set the text change handler.
    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: Fn(String) -> String + Send + Sync + 'static,
    {
        self.on_change = CallbackWith::new(on_change);
        self
    }

    /// Set the text change handler using a shared callback.
    pub fn on_change_shared(mut self, on_change: CallbackWith<String, String>) -> Self {
        self.on_change = on_change;
        self
    }

    /// Set an input transform applied before the change handler.
    pub fn input_transform<F>(mut self, transform: F) -> Self
    where
        F: Fn(String) -> String + Send + Sync + 'static,
    {
        self.input_transform = Some(CallbackWith::new(transform));
        self
    }

    /// Set an input transform using a shared callback.
    pub fn input_transform_shared(mut self, transform: CallbackWith<String, String>) -> Self {
        self.input_transform = Some(transform);
        self
    }

    /// Set the input line limit policy.
    pub fn line_limit(mut self, line_limit: TextFieldLineLimit) -> Self {
        self.line_limit = line_limit;
        self
    }

    /// Set the context menu policy.
    pub fn context_menu(mut self, context_menu: TextFieldContextMenu) -> Self {
        self.context_menu = context_menu;
        self
    }

    /// Sets an external text input controller.
    pub fn controller(mut self, controller: State<TextInputController>) -> Self {
        self.controller = Some(controller);
        self
    }

    /// Set the obfuscation character for secure fields.
    pub fn obfuscation_char(mut self, obfuscation_char: char) -> Self {
        self.obfuscation_char = Some(obfuscation_char);
        self
    }

    /// Set the leading icon slot.
    pub fn leading_icon<F>(mut self, leading_icon: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.leading_icon = Some(RenderSlot::new(leading_icon));
        self
    }

    /// Set the leading icon slot using a shared callback.
    pub fn leading_icon_shared(mut self, leading_icon: RenderSlot) -> Self {
        self.leading_icon = Some(leading_icon);
        self
    }

    /// Set the trailing icon slot.
    pub fn trailing_icon<F>(mut self, trailing_icon: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.trailing_icon = Some(RenderSlot::new(trailing_icon));
        self
    }

    /// Set the trailing icon slot using a shared callback.
    pub fn trailing_icon_shared(mut self, trailing_icon: RenderSlot) -> Self {
        self.trailing_icon = Some(trailing_icon);
        self
    }

    /// Set the prefix content slot.
    pub fn prefix<F>(mut self, prefix: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.prefix = Some(RenderSlot::new(prefix));
        self
    }

    /// Set the prefix content slot using a shared callback.
    pub fn prefix_shared(mut self, prefix: RenderSlot) -> Self {
        self.prefix = Some(prefix);
        self
    }

    /// Set the suffix content slot.
    pub fn suffix<F>(mut self, suffix: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.suffix = Some(RenderSlot::new(suffix));
        self
    }

    /// Set the suffix content slot using a shared callback.
    pub fn suffix_shared(mut self, suffix: RenderSlot) -> Self {
        self.suffix = Some(suffix);
        self
    }

    /// Creates filled text field defaults.
    pub fn filled() -> Self {
        Self::default()
    }

    /// Creates outlined text field defaults.
    pub fn outlined() -> Self {
        let theme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get();
        let scheme = theme.color_scheme;
        let selection_colors = TextSelectionColors::from_scheme(&scheme);
        Self {
            background_color: Some(Color::TRANSPARENT),
            border_width: TextFieldDefaults::OUTLINED_BORDER_WIDTH,
            border_color: Some(scheme.outline),
            shape: theme.shapes.extra_small,
            focus_border_color: Some(scheme.primary),
            focus_border_width: Some(TextFieldDefaults::OUTLINED_FOCUS_BORDER_WIDTH),
            focus_background_color: Some(Color::TRANSPARENT),
            selection_color: Some(selection_colors.background),
            text_color: Some(scheme.on_surface),
            cursor_color: Some(scheme.primary),
            font_size: theme.typography.body_large.font_size,
            line_height: theme.typography.body_large.line_height,
            ..Self::default()
        }
    }

    /// Creates secure text field defaults.
    pub fn secure() -> Self {
        Self::default()
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

impl Default for TextFieldArgs {
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
            on_change: CallbackWith::new(|value| value),
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
    args: &TextFieldArgs,
    input_transform: Option<CallbackWith<String, String>>,
    display_transform: Option<DisplayTransform>,
) -> TextInputArgs {
    TextInputArgs {
        enabled: args.enabled,
        read_only: args.read_only,
        modifier: args.modifier.clone(),
        on_change: args.on_change.clone(),
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
        input_transform,
        display_transform,
        controller: None,
    }
}

fn editor_content_len(controller: &State<TextInputController>) -> usize {
    controller.with(|c| {
        c.editor().with_buffer(|buffer| {
            buffer
                .lines
                .iter()
                .map(|line| line.text().len() + line.ending().as_str().len())
                .sum()
        })
    })
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
    args: &TextFieldArgs,
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

fn resolve_border_width(args: &TextFieldArgs, focused: bool) -> Dp {
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
#[derive(PartialEq, Clone)]
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

impl LayoutSpec for OutlinedFloatingLabelLayout {
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
fn outlined_floating_label_node(args: &OutlinedFloatingLabelArgs) {
    let args = args.clone();

    layout(OutlinedFloatingLabelLayout {
        label_offset: PxPosition::new(args.label_offset_x.into(), args.label_offset_y.into()),
        notch_padding: args.notch_padding.into(),
        notch_vertical_padding: args.notch_vertical_padding.into(),
    });

    spacer(&crate::spacer::SpacerArgs::new(
        Modifier::new().background(args.notch_fill_color),
    ));

    let mut label_args = TextArgs::default()
        .text(&args.label_text)
        .color(args.label_color)
        .size(args.label_font_size);
    label_args = label_args.line_height(args.label_line_height);
    text(&crate::text::TextArgs::from(&label_args));
}

fn render_text_field(
    args: TextFieldArgs,
    controller: State<TextInputController>,
    editor: TextInputArgs,
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
        surface(&crate::surface::SurfaceArgs::with_child(
            surface_args,
            move || {
                let leading_icon = leading_icon.clone();
                let prefix = prefix.clone();
                let core_args = core_args.clone();
                let placeholder_text = placeholder_text.clone();
                let label_text = label_text.clone();
                let suffix = suffix.clone();
                let trailing_icon = trailing_icon.clone();
                boxed(BoxedArgs::default(), move |scope| {
                    scope.child(move || {
                        let leading_icon = leading_icon.clone();
                        let prefix = prefix.clone();
                        let core_args = core_args.clone();
                        let placeholder_text = placeholder_text.clone();
                        let label_text = label_text.clone();
                        let suffix = suffix.clone();
                        let trailing_icon = trailing_icon.clone();
                        let row_modifier = RowArgs::default()
                            .modifier
                            .padding(Padding::all(content_padding));
                        row(
                            RowArgs::default()
                                .modifier(row_modifier)
                                .cross_axis_alignment(CrossAxisAlignment::Center),
                            move |row_scope| {
                                if let Some(leading_icon) = leading_icon.as_ref() {
                                    let leading_icon = leading_icon.clone();
                                    row_scope.child(move || {
                                        provide_context(
                                            || ContentColor {
                                                current: content_color,
                                            },
                                            || {
                                                leading_icon.render();
                                            },
                                        );
                                    });
                                    let spacing = TextFieldDefaults::ICON_TEXT_PADDING;
                                    row_scope.child(move || {
                                        spacer(&crate::spacer::SpacerArgs::new(
                                            Modifier::new().width(spacing),
                                        ));
                                    });
                                }

                                if let Some(prefix) = prefix.as_ref() {
                                    let prefix = prefix.clone();
                                    row_scope.child(move || {
                                        provide_context(
                                            || ContentColor {
                                                current: content_color,
                                            },
                                            || {
                                                prefix.render();
                                            },
                                        );
                                    });
                                    let spacing = TextFieldDefaults::PREFIX_SUFFIX_PADDING;
                                    row_scope.child(move || {
                                        spacer(&crate::spacer::SpacerArgs::new(
                                            Modifier::new().width(spacing),
                                        ));
                                    });
                                }

                                row_scope.child_weighted(
                                    move || {
                                        let core_args = core_args.clone();
                                        let placeholder_text = placeholder_text.clone();
                                        let label_text = label_text.clone();
                                        boxed(BoxedArgs::default(), move |box_scope| {
                                            box_scope.child(move || {
                                                text_input_core(&core_args.clone(), controller);
                                            });

                                            if show_placeholder
                                                && let Some(placeholder_text) =
                                                    placeholder_text.as_ref()
                                            {
                                                let placeholder_text = placeholder_text.clone();
                                                box_scope.child_with_alignment(
                                                    Alignment::TopStart,
                                                    move || {
                                                        let (font_size, line_height) =
                                                            placeholder_style;
                                                        let mut args = TextArgs::default()
                                                            .text(placeholder_text.clone())
                                                            .color(placeholder_color)
                                                            .size(font_size);
                                                        if let Some(line_height) = line_height {
                                                            args = args.line_height(line_height);
                                                        }
                                                        text(&crate::text::TextArgs::from(&args));
                                                    },
                                                );
                                            }

                                            if let Some(label_text) = label_text.as_ref() {
                                                let label_text = label_text.clone();
                                                if label_should_float {
                                                    if is_outlined {
                                                        let floating_args =
                                                            OutlinedFloatingLabelArgs {
                                                                label_text: label_text.clone(),
                                                                label_color,
                                                                label_font_size:
                                                                    label_floating_font_size,
                                                                label_line_height:
                                                                    label_floating_line_height,
                                                                label_offset_x:
                                                                    floating_label_offset_x,
                                                                label_offset_y:
                                                                    floating_label_offset_y,
                                                                notch_fill_color,
                                                                notch_padding,
                                                                notch_vertical_padding,
                                                            };
                                                        box_scope.child_with_alignment(
                                                            Alignment::TopStart,
                                                            move || {
                                                                let args = floating_args.clone();
                                                                outlined_floating_label_node(&args);
                                                            },
                                                        );
                                                    } else {
                                                        box_scope.child_with_alignment(
                                                            Alignment::TopStart,
                                                            move || {
                                                                let mut args = TextArgs::default()
                                                                    .text(label_text.clone())
                                                                    .color(label_color)
                                                                    .size(label_floating_font_size)
                                                                    .modifier(
                                                                        Modifier::new().offset(
                                                                            floating_label_offset_x,
                                                                            floating_label_offset_y,
                                                                        ),
                                                                    );
                                                                args = args.line_height(
                                                                    label_floating_line_height,
                                                                );
                                                                text(&crate::text::TextArgs::from(
                                                                    &args,
                                                                ));
                                                            },
                                                        );
                                                    }
                                                } else {
                                                    box_scope.child_with_alignment(
                                                        Alignment::TopStart,
                                                        move || {
                                                            let mut args = TextArgs::default()
                                                                .text(label_text.clone())
                                                                .color(label_color)
                                                                .size(label_resting_font_size);
                                                            args = args.line_height(
                                                                label_resting_line_height,
                                                            );
                                                            text(&crate::text::TextArgs::from(
                                                                &args,
                                                            ));
                                                        },
                                                    );
                                                }
                                            }
                                        });
                                    },
                                    1.0,
                                );

                                if let Some(suffix) = suffix.as_ref() {
                                    let suffix = suffix.clone();
                                    let spacing = TextFieldDefaults::PREFIX_SUFFIX_PADDING;
                                    row_scope.child(move || {
                                        spacer(&crate::spacer::SpacerArgs::new(
                                            Modifier::new().width(spacing),
                                        ));
                                    });
                                    row_scope.child(move || {
                                        provide_context(
                                            || ContentColor {
                                                current: content_color,
                                            },
                                            || {
                                                suffix.render();
                                            },
                                        );
                                    });
                                }

                                if let Some(trailing_icon) = trailing_icon.as_ref() {
                                    let trailing_icon = trailing_icon.clone();
                                    let spacing = TextFieldDefaults::ICON_TEXT_PADDING;
                                    row_scope.child(move || {
                                        spacer(&crate::spacer::SpacerArgs::new(
                                            Modifier::new().width(spacing),
                                        ));
                                    });
                                    row_scope.child(move || {
                                        provide_context(
                                            || ContentColor {
                                                current: content_color,
                                            },
                                            || {
                                                trailing_icon.render();
                                            },
                                        );
                                    });
                                }
                            },
                        );
                    });

                    if show_indicator {
                        scope.child_with_alignment(Alignment::BottomStart, move || {
                            horizontal_divider(
                                &DividerArgs::default()
                                    .thickness(indicator_thickness)
                                    .color(indicator_color),
                            );
                        });
                    }
                });
            },
        ));
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
            let selected =
                controller.with_mut(|c| c.with_editor_mut(|editor| editor.copy_selection()));
            if let Some(text) = selected {
                clipboard::set_text(&text);
            }
        }
        TextFieldMenuAction::Cut => {
            if read_only {
                return;
            }
            let selected =
                controller.with_mut(|c| c.with_editor_mut(|editor| editor.copy_selection()));
            if let Some(text) = selected {
                clipboard::set_text(&text);
                handle_action(
                    &controller,
                    GlyphonAction::Backspace,
                    on_change.clone(),
                    input_transform.clone(),
                );
            }
        }
        TextFieldMenuAction::Paste => {
            if read_only {
                return;
            }
            let Some(text) = clipboard::get_text() else {
                return;
            };
            for ch in text.chars() {
                handle_action(
                    &controller,
                    GlyphonAction::Insert(ch),
                    on_change.clone(),
                    input_transform.clone(),
                );
            }
        }
        TextFieldMenuAction::SelectAll => {
            controller.with_mut(|c| {
                c.with_editor_mut(|editor| {
                    editor.set_cursor(Cursor::new(0, 0));
                    editor.set_selection(Selection::Normal(Cursor::new(0, 0)));
                    editor.action(
                        &mut write_font_system(),
                        GlyphonAction::Motion(cosmic_text::Motion::BufferEnd),
                    );
                });
            });
        }
    }
}

fn menu_item(
    scope: &mut MenuScope<'_, '_>,
    label: &str,
    enabled: bool,
    action_state: State<Option<TextFieldMenuAction>>,
    action: TextFieldMenuAction,
) {
    scope.menu_item(
        &MenuItemArgs::default()
            .label(label)
            .enabled(enabled)
            .on_click(move || {
                action_state.set(Some(action));
            }),
    );
}

fn configure_text_field_menu(
    scope: &mut MenuScope<'_, '_>,
    controller: State<TextInputController>,
    menu: TextFieldContextMenu,
    action_state: State<Option<TextFieldMenuAction>>,
) {
    if !menu.enabled {
        return;
    }

    let selection_available = controller.with(|c| c.editor().selection_bounds().is_some());

    if menu.allow_cut {
        menu_item(
            scope,
            "Cut",
            selection_available,
            action_state,
            TextFieldMenuAction::Cut,
        );
    }
    if menu.allow_copy {
        menu_item(
            scope,
            "Copy",
            selection_available,
            action_state,
            TextFieldMenuAction::Copy,
        );
    }
    if menu.allow_paste {
        menu_item(
            scope,
            "Paste",
            true,
            action_state,
            TextFieldMenuAction::Paste,
        );
    }
    if menu.allow_select_all {
        menu_item(
            scope,
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
/// - `args` — configures the field styling, transforms, and behavior; see
///   [`TextFieldArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::text_field::{TextFieldArgs, text_field};
/// # use tessera_components::theme::{MaterialTheme, material_theme};
/// # let args = tessera_components::theme::MaterialThemeProviderArgs::new(|| MaterialTheme::default(), || {
/// let args = TextFieldArgs::default().input_transform(|value| value.to_uppercase());
/// assert_eq!(
///     args.input_transform
///         .as_ref()
///         .unwrap()
///         .call("hello".to_string()),
///     "HELLO"
/// );
/// text_field(&args);
/// # });
/// # material_theme(&args);
/// # }
/// # component();
/// ```
#[tessera]
pub fn text_field(args: &TextFieldArgs) {
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
    let input_transform = merge_input_transforms(args.line_limit, args.input_transform.clone());
    let display_transform = args.obfuscation_char.map(|mask| {
        CallbackWith::new(move |value: String| obfuscate_text(&value, mask)) as DisplayTransform
    });
    let editor_args = build_editor_args(&args, input_transform.clone(), display_transform);
    let menu_policy = resolve_text_field_menu_policy(args.context_menu, enabled, read_only);
    let on_change = args.on_change.clone();
    let render_args = args.clone();

    if menu_policy.enabled {
        let render_args_for_menu = render_args.clone();
        let editor_args_for_menu = editor_args.clone();
        let menu_args = MenuProviderArgs::default()
            .placement(MenuPlacement::BelowStart)
            .offset([Dp(0.0), Dp(4.0)])
            .controller(menu_controller)
            .main_content(move || {
                render_text_field(
                    render_args_for_menu.clone(),
                    controller,
                    editor_args_for_menu.clone(),
                );
            })
            .menu_content(move |menu_scope| {
                configure_text_field_menu(menu_scope, controller, menu_policy, action_state);
            });
        menu_provider(&menu_args);
    } else {
        render_text_field(render_args, controller, editor_args);
    }

    input_handler(move |mut input| {
        let cursor_pos = input.cursor_position_rel;
        if menu_policy.enabled {
            if let Some(action) = action_state.with_mut(|state| state.take()) {
                apply_menu_action(
                    action,
                    controller,
                    on_change.clone(),
                    input_transform.clone(),
                    enabled,
                    read_only,
                );
            }

            let has_right_click = input.cursor_events.iter().any(|event| {
                matches!(
                    event.content,
                    CursorEventContent::Released(PressKeyEventType::Right)
                )
            });

            if has_right_click
                && let Some(cursor_pos) = cursor_pos
                && is_position_inside_bounds(input.computed_data, cursor_pos)
            {
                menu_controller.with_mut(|menu| {
                    menu.open_at(MenuAnchor::at(cursor_pos));
                });
            }
        }

        let is_inside = cursor_pos
            .map(|pos| is_position_inside_bounds(input.computed_data, pos))
            .unwrap_or(false);

        if enabled {
            if is_inside {
                input.requests.cursor_icon = winit::window::CursorIcon::Text;
            }

            let has_left_click = input.cursor_events.iter().any(|event| {
                matches!(
                    event.content,
                    CursorEventContent::Pressed(PressKeyEventType::Left)
                )
            });

            if has_left_click && cursor_pos.is_some() && is_inside {
                controller.with_mut(|c| c.focus_handler_mut().request_focus());
                input.cursor_events.clear();
            }
        }

        if is_inside {
            input.block_cursor();
        }
    });
}
