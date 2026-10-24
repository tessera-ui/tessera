//! Material text fields for filled, outlined, and secure input.
//!
//! ## Usage
//!
//! Collect short-form inputs like names, passwords, or search queries.
use glyphon::Action as GlyphonAction;
use tessera_foundation::gesture::{ScrollRecognizer, TapRecognizer, TapSettings};
use tessera_ui::{
    AxisConstraint, Callback, CallbackWith, Color, ComputedData, Constraint, Dp, LayoutResult,
    MeasurementError, Modifier, PressKeyEventType, Px, PxPosition, PxSize, RenderSlot,
    ScrollDeltaUnit, State,
    layout::{MeasureScope, layout},
    modifier::{CursorModifierExt as _, FocusModifierExt as _},
    normalize_platform_scroll_delta, provide_context, remember, tessera, use_context, winit,
};

#[cfg(test)]
use tessera_ui::PxRect;

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    divider::horizontal_divider,
    menus::{MenuAnchor, MenuController, MenuPlacement, menu_item, menu_provider},
    modifier::{InteractionVisualState, ModifierExt as _, Padding, with_pointer_input},
    pos_misc::is_position_inside_bounds,
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    text_edit_core::DisplayTransform,
    text_input::{
        DISABLED_CONTENT_ALPHA, DisplayTransformText, TextInputController, TextInputProps,
        create_surface_args, text_input_core, visual_focus,
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
    pub presentation_state: Option<InteractionVisualState>,
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
            presentation_state: None,
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
        presentation_state: args.presentation_state,
        read_only: args.read_only,
        is_error: args.is_error,
        is_secure: args.obfuscation_char.is_some(),
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

#[derive(Default)]
struct FieldGeometry {
    outer: Option<tessera_ui::LayoutCoordinates>,
    editor: Option<tessera_ui::LayoutCoordinates>,
}

fn field_click_pointer_position(
    cursor_pos: PxPosition,
    geometry: State<FieldGeometry>,
    controller: State<TextInputController>,
) -> Option<PxPosition> {
    let (outer, editor) = geometry.with(|geometry| (geometry.outer, geometry.editor));
    let (outer, editor) = (outer?, editor?);
    let editor_cursor = editor
        .local_to_window
        .inverse_point(outer.local_to_window.map_point(cursor_pos));
    let (horizontal, vertical) = controller.with(|controller| {
        (
            controller.scroll_state().horizontal(),
            controller.scroll_state().vertical(),
        )
    });
    Some(PxPosition::from_f32_xy(
        editor_cursor.x.clamp(0.0, editor.local_size.width.to_f32()) + horizontal,
        editor_cursor
            .y
            .clamp(0.0, editor.local_size.height.to_f32())
            + vertical,
    ))
}

#[derive(Clone, PartialEq)]
struct OutlinedFloatingLabelLayout {
    label_offset: PxPosition,
}

impl tessera_ui::LayoutModifierNode for OutlinedFloatingLabelLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        debug_assert_eq!(
            children.len(),
            1,
            "OutlinedFloatingLabel expects exactly one child"
        );
        let label = children[0];

        let parent_constraint = input.parent_constraint().without_min();
        label.measure(input, &parent_constraint)?;
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
    outline: Option<State<OutlineGeometry>>,
) {
    let typography = use_context::<MaterialTheme>().unwrap().get().typography;
    let label_text = label_text.unwrap_or_default();
    let label_color = label_color.unwrap_or(Color::TRANSPARENT);
    let label_font_size = label_font_size.unwrap_or(Dp(0.0));
    let label_line_height = label_line_height.unwrap_or(Dp(0.0));
    let label_offset_x = label_offset_x.unwrap_or(Dp(0.0));
    let label_offset_y = label_offset_y.unwrap_or(Dp(0.0));
    let outline = outline.expect("floating outlined label requires outline geometry");
    layout()
        .policy(OutlinedFloatingLabelLayout {
            label_offset: PxPosition::new(label_offset_x.into(), label_offset_y.into()),
        })
        .child(move || {
            text()
                .content(label_text.clone())
                .modifier(Modifier::new().on_layout_coordinates(
                    move |coordinates: tessera_ui::LayoutCoordinates| {
                        if outline.with(|outline| outline.label_width)
                            != coordinates.local_size.width
                        {
                            outline.with_mut(|outline| {
                                outline.label_width = coordinates.local_size.width
                            });
                        }
                    },
                ))
                .color(label_color)
                .style(TextStyle {
                    font_size: label_font_size,
                    line_height: Some(label_line_height),
                    ..typography.body_small
                });
        });
}

#[derive(Default, Clone, Copy, PartialEq)]
struct OutlineGeometry {
    size: PxSize,
    label_width: Px,
}

#[derive(Clone, PartialEq)]
struct OutlineSlicesLayout {
    size: PxSize,
    left: Px,
    right: Px,
    band: Px,
}

impl tessera_ui::LayoutModifierNode for OutlineSlicesLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        if input.is_intrinsic() {
            return Ok(result);
        }
        let slices = [
            (
                PxPosition::new(Px::ZERO, self.band),
                PxSize::new(
                    self.size.width,
                    Px((self.size.height.0 - self.band.0).max(0)),
                ),
            ),
            (PxPosition::ZERO, PxSize::new(self.left, self.band)),
            (
                PxPosition::new(self.right, Px::ZERO),
                PxSize::new(Px((self.size.width.0 - self.right.0).max(0)), self.band),
            ),
        ];
        for (child, (position, size)) in input.children().into_iter().zip(slices) {
            if size.width <= Px::ZERO || size.height <= Px::ZERO {
                continue;
            }
            child.measure(input, &Constraint::exact(size.width, size.height))?;
            result.place_child(child, position);
        }
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct OutlineSliceLayout {
    size: PxSize,
    origin: PxPosition,
}

impl tessera_ui::LayoutModifierNode for OutlineSliceLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let Some(child) = input.children().first().copied() else {
            return Ok(LayoutResult::default());
        };
        child.measure(input, &Constraint::exact(self.size.width, self.size.height))?;
        let mut result = LayoutResult::new(ComputedData {
            width: input.parent_constraint().width().clamp(Px::ZERO),
            height: input.parent_constraint().height().clamp(Px::ZERO),
        });
        result.place_child(
            child,
            PxPosition::from_f32_xy(-self.origin.x, -self.origin.y),
        );
        Ok(result)
    }
}

#[tessera]
fn outlined_border(
    geometry: OutlineGeometry,
    padding: Dp,
    width: Dp,
    color: Color,
    shape: Shape,
    floating: bool,
) {
    let size = geometry.size;
    let left = if floating {
        Px(
            (padding.to_px().0 - TextFieldDefaults::OUTLINED_LABEL_PADDING.to_px().0)
                .clamp(0, size.width.0),
        )
    } else {
        size.width
    };
    let right = if floating {
        Px((padding.to_px().0
            + geometry.label_width.0
            + TextFieldDefaults::OUTLINED_LABEL_PADDING.to_px().0)
            .clamp(left.0, size.width.0))
    } else {
        size.width
    };
    let band = Px((width.to_pixels_f32().ceil() as i32 + 1).min(size.height.0));
    layout()
        .policy(OutlineSlicesLayout {
            size,
            left,
            right,
            band,
        })
        .child(move || {
            for origin in [
                PxPosition::new(Px::ZERO, band),
                PxPosition::ZERO,
                PxPosition::new(right, Px::ZERO),
            ] {
                layout()
                    .modifier(Modifier::new().clip_shape(Shape::RECTANGLE))
                    .policy(OutlineSliceLayout { size, origin })
                    .child(move || {
                        surface()
                            .style(SurfaceStyle::Outlined { color, width })
                            .shape(shape)
                            .block_input(false);
                    });
            }
        });
}

type FieldContentData = (TextFieldProps, TextInputProps, MaterialTheme, bool, bool);

#[derive(Clone, PartialEq)]
struct FieldBodyHeight;

impl tessera_ui::LayoutModifierNode for FieldBodyHeight {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let Some(child) = input.children().first().copied() else {
            return Ok(LayoutResult::default());
        };
        let natural_constraints =
            Constraint::new(input.parent_constraint().width(), AxisConstraint::NONE);
        let natural = child.probe_intrinsic(input, &natural_constraints)?;
        let height = input.parent_constraint().height().clamp(natural.height);
        let constraints = Constraint::new(
            input.parent_constraint().width(),
            AxisConstraint::exact(height),
        );
        let measured = child.measure(input, &constraints)?;
        let mut result = LayoutResult::new(measured.size());
        result.place_child(child, PxPosition::ZERO);
        Ok(result)
    }
}

#[derive(Clone, PartialEq)]
struct FieldDecorationLayout {
    single_line: bool,
    line_height: Px,
}

impl tessera_ui::LayoutModifierNode for FieldDecorationLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let Some(child) = input.children().first().copied() else {
            return Ok(LayoutResult::default());
        };
        let constraints = if self.single_line {
            Constraint::new(
                AxisConstraint::NONE,
                AxisConstraint::exact(self.line_height),
            )
        } else {
            input.parent_constraint().without_min()
        };
        let measured = child.measure(input, &constraints)?;
        let mut result = LayoutResult::new(ComputedData {
            width: input.parent_constraint().width().clamp(measured.width),
            height: input.parent_constraint().height().clamp(measured.height),
        });
        result.place_child(child, PxPosition::ZERO);
        Ok(result)
    }
}

#[tessera]
fn field_decoration(content: String, color: Color, style: TextStyle, single_line: bool) {
    let modifier = if single_line {
        Modifier::new().clip_to_bounds()
    } else {
        Modifier::new()
    };
    layout()
        .modifier(modifier)
        .policy(FieldDecorationLayout {
            single_line,
            line_height: style
                .line_height
                .unwrap_or(Dp(style.font_size.0 * 1.2))
                .to_px(),
        })
        .child(move || {
            text().content(content.clone()).color(color).style(style);
        });
}

#[tessera]
fn text_field_content(
    args: Option<TextFieldProps>,
    controller: Option<State<TextInputController>>,
    editor: Option<TextInputProps>,
    geometry: Option<State<FieldGeometry>>,
) {
    let args = args.expect("text_field_content requires args");
    let controller = controller.expect("text_field_content requires controller");
    let editor = editor.expect("text_field_content requires editor");
    let geometry = geometry.expect("text_field_content requires geometry");
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let focused = visual_focus(&editor, controller);
    let empty = editor_content_len(&controller) == 0;
    let surface_args = create_surface_args(&editor, &controller, &theme.color_scheme);
    let outline = remember(OutlineGeometry::default);
    let border = match surface_args.style {
        SurfaceStyle::FilledOutlined {
            border_color,
            border_width,
            ..
        } => Some((border_color, border_width)),
        SurfaceStyle::Outlined { color, width } => Some((color, width)),
        _ => None,
    };
    let surface_style = match surface_args.style {
        SurfaceStyle::FilledOutlined { fill_color, .. } => {
            SurfaceStyle::Filled { color: fill_color }
        }
        SurfaceStyle::Outlined { .. } => SurfaceStyle::Filled {
            color: Color::TRANSPARENT,
        },
        ref style => style.clone(),
    };
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
        .style(surface_style)
        .shape(surface_args.shape)
        .block_input(!args.enabled)
        .modifier(surface_args.modifier.on_layout_coordinates(
            move |coordinates: tessera_ui::LayoutCoordinates| {
                let size = coordinates.local_size;
                if outline.with(|outline| outline.size) != size {
                    outline.with_mut(|outline| outline.size = size);
                }
            },
        ))
        .content_color(color)
        .child(move || {
            boxed().children(move || {
                text_field_body(data, controller, geometry, outline);
                if let Some((color, width)) = border {
                    let args = data.with(|data| data.0.clone());
                    outlined_border(
                        outline.get(),
                        args.padding,
                        width,
                        color,
                        args.shape,
                        args.label.is_some() && (focused || !empty),
                    );
                }
            });
        });
}

#[tessera]
fn text_field_body(
    data: State<FieldContentData>,
    controller: State<TextInputController>,
    geometry: State<FieldGeometry>,
    outline: State<OutlineGeometry>,
) {
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
    layout().policy(FieldBodyHeight).child(move || {
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
                        spacer().modifier(
                            Modifier::new().width(TextFieldDefaults::PREFIX_SUFFIX_PADDING),
                        );
                    }
                    boxed()
                        .modifier(Modifier::new().weight(1.0))
                        .children(move || {
                            let (args, mut editor, theme, focused, empty) = data.get();
                            editor.modifier = Modifier::new()
                                .fill_max_size()
                                .on_layout_coordinates(move |bounds| {
                                    if geometry.with(|geometry| geometry.editor) != Some(bounds) {
                                        geometry
                                            .with_mut(|geometry| geometry.editor = Some(bounds));
                                    }
                                });
                            editor.padding = Dp::ZERO;
                            editor.border_width = Dp::ZERO;
                            editor.focus_border_width = Some(Dp::ZERO);
                            text_input_core(&editor, controller);
                            let floating = args.label.is_some() && (focused || !empty);
                            if empty
                                && (args.label.is_none() || floating)
                                && let Some(placeholder) = args.placeholder
                            {
                                field_decoration(
                                    placeholder,
                                    resolve_placeholder_color(&theme.color_scheme, args.enabled),
                                    placeholder_text_style(&theme),
                                    matches!(args.line_limit, TextFieldLineLimit::SingleLine),
                                );
                            }
                            if !floating && let Some(label) = args.label {
                                field_decoration(
                                    label,
                                    resolve_label_color(
                                        &theme.color_scheme,
                                        focused,
                                        args.is_error,
                                        args.enabled,
                                    ),
                                    label_resting_text_style(&theme),
                                    true,
                                );
                            }
                        });
                    let (args, _, _, _, _) = data.get();
                    if affixes && let Some(suffix) = args.suffix {
                        spacer().modifier(
                            Modifier::new().width(TextFieldDefaults::PREFIX_SUFFIX_PADDING),
                        );
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
                    outlined_floating_label()
                        .label_text(label)
                        .label_color(color)
                        .label_font_size(label_style.font_size)
                        .label_line_height(line_height)
                        .label_offset_x(padding)
                        .label_offset_y(Dp(-line_height.0 * 0.5))
                        .outline(outline);
                } else {
                    text()
                        .content(label)
                        .style(label_style)
                        .color(color)
                        .modifier(
                            Modifier::new()
                                .offset(left, TextFieldDefaults::FLOATING_LABEL_Y_OFFSET),
                        );
                }
            }
            if args.show_indicator && !outlined {
                let (color, thickness) = resolve_indicator_style(
                    &theme.color_scheme,
                    focused,
                    args.is_error,
                    args.enabled,
                );
                horizontal_divider()
                    .modifier(Modifier::new().align(Alignment::BottomStart))
                    .thickness(thickness)
                    .color(color);
            }
        });
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
/// - `presentation_state` — optional visual interaction without changing input
///   or focus.
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
        presentation_state,
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
    let geometry = remember(FieldGeometry::default);

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
    let modifier_base = modifier_base.on_layout_coordinates(move |bounds| {
        if geometry.with(|geometry| geometry.outer) != Some(bounds) {
            geometry.with_mut(|geometry| geometry.outer = Some(bounds));
        }
    });
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
                    if let Some(click_position) =
                        field_click_pointer_position(cursor_pos, geometry, controller)
                    {
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
                        .editor(editor_args.clone())
                        .geometry(geometry);
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
                .editor(editor_args.clone())
                .geometry(geometry);
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

#[cfg(all(test, not(target_family = "wasm")))]
mod native_field_regressions {
    use std::{
        cell::{Cell, RefCell},
        time::Duration,
    };

    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, accesskit::Role,
        winit::event::Ime,
    };

    use super::*;

    #[derive(Clone, Copy)]
    enum Case {
        Filled,
        Outlined,
        Multiline,
        Affixes,
        Secure,
        ExplicitHeight,
    }

    thread_local! {
        static NOTCH_CASE: Cell<(bool, bool)> = const { Cell::new((false, true)) };
        static CASE: Cell<Case> = const {Cell::new(Case::Filled)};
        static EDITOR: Cell<Option<State<TextInputController>>> = const {Cell::new(None)};
        static BOUNDS: Cell<Option<PxRect>> = const {Cell::new(None)};
        static CHANGES: RefCell<Vec<String>> = const {RefCell::new(Vec::new())};
    }

    #[tessera]
    fn fixture() {
        let case = CASE.get();
        let controller = remember(|| TextInputController::new(Dp(16.0), Some(Dp(24.0))));
        let changes = remember(|| 0_usize);
        EDITOR.set(Some(controller));
        crate::theme::material_theme().child(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    crate::column::column()
                        .modifier(Modifier::new().width(Dp(180.0)))
                        .children(move || {
                            let mut field = if matches!(case, Case::Outlined) {
                                TextFieldBuilder::outlined()
                            } else {
                                TextFieldBuilder::filled()
                            };
                            let mut field_modifier = Modifier::new()
                                .fill_max_width()
                                .on_globally_positioned(|rect| BOUNDS.set(Some(rect)));
                            if matches!(case, Case::ExplicitHeight) {
                                field_modifier = field_modifier.height(Dp(96.0));
                            }
                            field = field
                                .controller(controller)
                                .modifier(field_modifier)
                                .min_width(Dp::ZERO)
                                .min_height(Dp(if matches!(case, Case::Multiline) {
                                    96.0
                                } else {
                                    56.0
                                }))
                                .label("Label")
                                .placeholder("Placeholder text longer than the available input")
                                .line_limit(if matches!(case, Case::Multiline) {
                                    TextFieldLineLimit::MultiLine
                                } else {
                                    TextFieldLineLimit::SingleLine
                                })
                                .accessibility_label("Regression field")
                                .leading_icon(|| {
                                    boxed()
                                        .modifier(Modifier::new().semantics(
                                            crate::modifier::SemanticsArgs {
                                                role: Some(Role::Image),
                                                label: Some("Leading search decoration".into()),
                                                ..Default::default()
                                            },
                                        ))
                                        .children(|| {
                                            crate::icon::icon()
                                                .vector(crate::material_icons::filled::SEARCH_SVG)
                                                .size(Dp(24.0));
                                        });
                                })
                                .trailing_icon(|| {
                                    crate::icon::icon()
                                        .vector(crate::material_icons::filled::CANCEL_SVG)
                                        .size(Dp(24.0));
                                })
                                .on_change(move |value: String| {
                                    CHANGES.with_borrow_mut(|values| values.push(value.clone()));
                                    changes.set(changes.get() + 1);
                                    value
                                });
                            if matches!(case, Case::Affixes) {
                                field = field
                                    .prefix(|| {
                                        text().content("$").accessibility_label("Dollar prefix");
                                    })
                                    .suffix(|| {
                                        text().content("USD");
                                    });
                            }
                            if matches!(case, Case::Secure) {
                                field = field
                                    .obfuscation_char('•')
                                    .context_menu(TextFieldContextMenu::secure_default());
                            }
                            drop(field);
                            text().content(format!("Changes: {}", changes.get()));
                        });
                });
        });
    }

    #[tessera]
    fn notch_fixture() {
        let (dark, enabled) = NOTCH_CASE.get();
        crate::theme::material_theme()
            .theme(move || MaterialTheme::from_seed(Color::from_rgb(0.4, 0.3, 0.6), dark))
            .child(move || {
                surface()
                    .style(SurfaceStyle::Filled {
                        color: Color::from_rgb(0.21, 0.55, 0.31),
                    })
                    .modifier(Modifier::new().fill_max_size())
                    .child(move || {
                        boxed()
                            .alignment(Alignment::TopStart)
                            .modifier(Modifier::new().fill_max_size())
                            .children(move || {
                                let scheme =
                                    use_context::<MaterialTheme>().unwrap().get().color_scheme;
                                for (x, label) in [(24.5, true), (244.5, false)] {
                                    if !label {
                                        surface()
                                            .modifier(
                                                Modifier::new()
                                                    .offset(Dp(x), Dp(40.5))
                                                    .size(Dp(180.0), Dp(56.0)),
                                            )
                                            .shape(MaterialShapes::default().extra_small)
                                            .style(SurfaceStyle::Outlined {
                                                color: if enabled {
                                                    scheme.outline
                                                } else {
                                                    scheme.on_surface.with_alpha(
                                                        crate::text_input::DISABLED_OUTLINE_ALPHA,
                                                    )
                                                },
                                                width: Dp(1.0),
                                            });
                                        continue;
                                    }
                                    let controller = remember(|| {
                                        let mut controller =
                                            TextInputController::new(Dp(16.0), Some(Dp(24.0)));
                                        controller.set_text("Input");
                                        controller
                                    });
                                    if label {
                                        EDITOR.set(Some(controller));
                                    }
                                    let mut field = TextFieldBuilder::outlined()
                                        .shape(MaterialShapes::default().extra_small)
                                        .min_width(Dp::ZERO)
                                        .controller(controller)
                                        .enabled(enabled)
                                        .modifier(
                                            Modifier::new()
                                                .offset(Dp(x), Dp(40.5))
                                                .width(Dp(180.0))
                                                .on_globally_positioned(move |bounds| {
                                                    if label {
                                                        BOUNDS.set(Some(bounds));
                                                    }
                                                }),
                                        );
                                    if label {
                                        field = field.label("Label");
                                    }
                                    drop(field);
                                }
                            });
                    });
            });
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for outlined field pixels"]
    fn outlined_label_notch_preserves_parent_pixels_and_fractional_stroke_opacity() {
        for dark in [false, true] {
            for enabled in [false, true] {
                NOTCH_CASE.set((dark, enabled));
                let mut renderer = HeadlessRenderer::new(
                    Box::new(notch_fixture),
                    vec![Box::new(crate::TesseraComponents)],
                    HeadlessConfig {
                        width: 460,
                        height: 160,
                        sample_count: 1,
                        frame_time: Duration::from_nanos(16_666_667),
                        ..Default::default()
                    },
                )
                .unwrap();
                renderer.render_frames(7);
                for focused in [false, true] {
                    if focused {
                        if !enabled {
                            break;
                        }
                        EDITOR
                            .get()
                            .unwrap()
                            .with(|editor| editor.focus_handler().request_focus());
                        renderer.render_frames(4);
                    }
                    let pixels = renderer.read_pixels().unwrap();
                    let pixel = |x: usize, y: usize| -> [u8; 4] {
                        pixels[(y * 460 + x) * 4..][..4].try_into().unwrap()
                    };
                    let bounds = BOUNDS.get().unwrap();
                    assert_eq!(bounds.height, Dp(56.0).to_px());
                    let x = bounds.x.0 as usize;
                    let y = bounds.y.0 as usize;
                    let background = pixel(2, 2);
                    let tree = renderer.accessibility_snapshot().unwrap();
                    let label = tree
                        .nodes
                        .iter()
                        .find(|(_, node)| {
                            node.role() == Role::Label && node.label() == Some("Label")
                        })
                        .unwrap()
                        .1
                        .bounds()
                        .unwrap();
                    let gap_x = label.x1.ceil() as usize + 1;
                    assert_eq!(
                        pixel(gap_x, y),
                        background,
                        "top stroke must stop at floating label: dark={dark}, enabled={enabled}, focused={focused}"
                    );
                    assert_eq!(
                        pixel(label.x0.floor() as usize - 1, y - 3),
                        background,
                        "transparent notch must preserve arbitrary parent background"
                    );
                    assert_ne!(
                        pixel(x + 120, y),
                        background,
                        "upper stroke outside notch remains painted: dark={dark}, enabled={enabled}, focused={focused}, bounds={bounds:?}, samples={:?}, reference={:?}",
                        [
                            pixel(x + 120, y - 1),
                            pixel(x + 120, y),
                            pixel(x + 120, y + 1)
                        ],
                        [
                            pixel(x + 340, y - 1),
                            pixel(x + 340, y),
                            pixel(x + 340, y + 1)
                        ]
                    );
                    if !focused {
                        assert_eq!(
                            pixel(x + 120, y),
                            pixel(x + 340, y),
                            "unfocused upper stroke must match the unsplit native reference"
                        );
                        for dy in 0..12 {
                            for dx in 0..6 {
                                assert_eq!(
                                    pixel(x + dx, y + dy),
                                    pixel(x + 220 + dx, y + dy),
                                    "fractional clipped outline must match unsplit native stroke at {dx},{dy}; dark={dark}, enabled={enabled}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn renderer(case: Case) -> HeadlessRenderer {
        CASE.set(case);
        EDITOR.set(None);
        BOUNDS.set(None);
        CHANGES.with_borrow_mut(Vec::clear);
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 400,
                height: 320,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .expect("native field renderer");
        renderer.render_frames(5);
        renderer
    }

    fn click(renderer: &mut HeadlessRenderer, label: &str, role: Role) {
        let bounds = renderer
            .accessibility_snapshot()
            .expect("field semantics")
            .nodes
            .iter()
            .find(|(_, node)| node.role() == role && node.label() == Some(label))
            .unwrap_or_else(|| panic!("missing {label}"))
            .1
            .bounds()
            .expect("decoration bounds");
        let position = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(4);
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for actual text shaping and field rendering"]
    fn narrow_single_line_fields_keep_one_line_decorations_and_queued_commits() {
        for case in [Case::Filled, Case::Outlined] {
            let mut renderer = renderer(case);
            let editor = EDITOR.get().expect("editor");
            assert_eq!(
                BOUNDS.get().expect("resting bounds").height,
                Dp(56.0).to_px()
            );
            editor.with(|editor| editor.focus_handler().request_focus());
            renderer.render_frames(4);
            assert_eq!(
                BOUNDS.get().expect("focused bounds").height,
                Dp(56.0).to_px()
            );
            let tree = renderer.accessibility_snapshot().expect("placeholder tree");
            let placeholder = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Label
                        && node.label() == Some("Placeholder text longer than the available input")
                })
                .expect("native placeholder")
                .1
                .bounds()
                .expect("placeholder bounds");
            assert_eq!(placeholder.height(), 24.0);
            for character in ["a", "l", "p", "h", "a"] {
                renderer.inject_ime_event(Ime::Commit(character.into()));
            }
            renderer.render_frames(4);
            assert_eq!(editor.with(|editor| editor.text()), "alpha");
            assert_eq!(
                CHANGES.with_borrow(|values| values.clone()),
                ["a", "al", "alp", "alph", "alpha"]
            );
            renderer.inject_ime_event(Ime::Commit(" beta".into()));
            renderer.render_frames(3);
            assert_eq!(editor.with(|editor| editor.text()), "alpha beta");
            let long = "AAAAAAAAAAAAAAAAAAAAAAAA zzzzzzzzzzzzzzzzzzzzzzzz";
            editor.with_mut(|editor| {
                editor.set_text_and_selection(
                    long,
                    crate::text_edit_core::TextSelection::collapsed(long.len()),
                );
                editor.focus_handler_mut().unfocus();
            });
            renderer.render_frames(4);
            assert!(editor.with(|editor| editor.scroll_state().horizontal()) > 0.0);
            let end_pixels = renderer.read_pixels().unwrap();
            let input = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| node.role() == Role::TextInput)
                .unwrap()
                .1
                .bounds()
                .unwrap();
            editor.with_mut(|editor| {
                editor.set_text_and_selection(
                    long,
                    crate::text_edit_core::TextSelection::collapsed(0),
                );
            });
            renderer.render_frames(4);
            assert_eq!(
                editor.with(|editor| editor.scroll_state().horizontal()),
                0.0
            );
            assert_eq!(editor.with(|editor| editor.text()), long);
            let start_pixels = renderer.read_pixels().unwrap();
            let mut changed_glyph_pixels = 0;
            for y in input.y0.ceil() as usize..input.y1.floor() as usize {
                for x in (input.x0.ceil() as usize + 3)..(input.x1.floor() as usize - 3) {
                    let offset = (y * 400 + x) * 4;
                    changed_glyph_pixels += usize::from(
                        end_pixels[offset..offset + 4] != start_pixels[offset..offset + 4],
                    );
                }
            }
            assert!(
                changed_glyph_pixels > 30,
                "horizontal editor scrolling must move the visible glyphs, not only its caret"
            );
        }
        let mut tall = renderer(Case::ExplicitHeight);
        assert_eq!(BOUNDS.get().unwrap().height, Dp(96.0).to_px());
        EDITOR
            .get()
            .unwrap()
            .with(|editor| editor.focus_handler().request_focus());
        tall.render_frames(4);
        assert_eq!(
            BOUNDS.get().unwrap().height,
            Dp(96.0).to_px(),
            "explicit caller height remains authoritative"
        );
        drop(tall);
        let mut renderer = renderer(Case::Multiline);
        EDITOR
            .get()
            .unwrap()
            .with(|editor| editor.focus_handler().request_focus());
        renderer.render_frames(4);
        assert!(BOUNDS.get().expect("multiline bounds").height >= Dp(96.0).to_px());
        let tree = renderer.accessibility_snapshot().unwrap();
        let placeholder = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::Label
                    && node.label() == Some("Placeholder text longer than the available input")
            })
            .expect("multiline placeholder")
            .1
            .bounds()
            .unwrap();
        assert!(
            placeholder.height() > 24.0,
            "multiline placeholders retain wrapping"
        );
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for actual glyph caret placement"]
    fn decorated_field_fallback_clicks_use_the_measured_editor_origin() {
        let mut renderer = renderer(Case::Affixes);
        let editor = EDITOR.get().unwrap();
        let text = "iii";
        editor.with_mut(|editor| {
            editor.set_text_and_selection(
                text,
                crate::text_edit_core::TextSelection {
                    start: text.len(),
                    end: text.len(),
                },
            )
        });
        renderer.render_frames(4);
        click(&mut renderer, "Leading search decoration", Role::Image);
        assert_eq!(editor.with(|editor| editor.selection().start), 0);
        editor.with_mut(|editor| {
            editor.set_text_and_selection(
                text,
                crate::text_edit_core::TextSelection {
                    start: text.len(),
                    end: text.len(),
                },
            )
        });
        renderer.render_frames(3);
        click(&mut renderer, "Dollar prefix", Role::Label);
        assert_eq!(editor.with(|editor| editor.selection().start), 0);
    }
    #[test]
    #[ignore = "Requires a native GPU adapter for an actual protected text field"]
    fn obfuscated_field_uses_password_semantics_without_publishing_plaintext() {
        let mut renderer = renderer(Case::Secure);
        let editor = EDITOR.get().unwrap();
        editor.with(|editor| editor.focus_handler().request_focus());
        renderer.render_frames(3);
        renderer.inject_ime_event(Ime::Commit("secret".into()));
        renderer.render_frames(4);
        assert_eq!(editor.with(|editor| editor.text()), "secret");
        let tree = renderer.accessibility_snapshot().unwrap();
        let node = &tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.label() == Some("Regression field") && node.role() == Role::PasswordInput
            })
            .expect("actual protected editor semantics")
            .1;
        assert_eq!(
            node.value(),
            None,
            "protected text is not exposed through the accessibility value"
        );
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.value() == Some("secret"))
        );
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod presentation_regressions {
    use std::{cell::Cell, time::Duration};

    use tessera_ui::{HeadlessConfig, HeadlessRenderer, State};

    use super::*;

    type Presentation = (Option<InteractionVisualState>, bool);

    thread_local! {
        static PRESENTATION: Cell<Option<State<Presentation>>> = const { Cell::new(None) };
        static EDITORS: Cell<[Option<State<TextInputController>>; 2]> = const { Cell::new([None; 2]) };
        static CHANGES: Cell<usize> = const { Cell::new(0) };
    }

    #[tessera]
    fn fixture() {
        let presentation = remember(|| (Some(InteractionVisualState::Focus), true));
        PRESENTATION.set(Some(presentation));
        crate::theme::material_theme().child(move || {
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    for index in 0..2 {
                        let editor =
                            remember(|| TextInputController::new(Dp(16.0), Some(Dp(24.0))));
                        EDITORS.with(|editors| {
                            let mut values = editors.get();
                            values[index] = Some(editor);
                            editors.set(values);
                        });
                        let (visual, enabled) = presentation.get();
                        TextFieldBuilder::outlined()
                            .controller(editor)
                            .presentation_state_optional(visual)
                            .enabled(enabled)
                            .label("Label")
                            .accessibility_label(format!("Preview {index}"))
                            .modifier(
                                Modifier::new()
                                    .offset(Dp(20.0 + index as f64 * 300.0), Dp(20.0))
                                    .width(Dp(280.0)),
                            )
                            .on_change(|value: String| {
                                CHANGES.set(CHANGES.get() + 1);
                                value
                            });
                    }
                });
        });
    }

    fn crop(pixels: &[u8], index: usize) -> Vec<u8> {
        let mut result = Vec::new();
        for y in 15..85 {
            let start = (y * 620 + 15 + index * 300) * 4;
            result.extend_from_slice(&pixels[start..start + 290 * 4]);
        }
        result
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn focus_presentation_draws_static_native_fields_without_taking_focus() {
        crate::test_utils::run_isolated_test(
            "text_field::presentation_regressions::focus_presentation_draws_static_native_fields_without_taking_focus",
            || {
                CHANGES.set(0);
                let mut renderer = HeadlessRenderer::new(
                    Box::new(fixture),
                    vec![Box::new(crate::TesseraComponents)],
                    HeadlessConfig {
                        width: 620,
                        height: 120,
                        frame_time: Duration::from_millis(100),
                        ..Default::default()
                    },
                )
                .unwrap();
                renderer.render_frames(4);
                let editors = EDITORS.get().map(Option::unwrap);
                assert!(
                    editors
                        .iter()
                        .all(|editor| !editor.with(|editor| editor.focus_handler().is_focused()))
                );
                let semantics = renderer.accessibility_snapshot().unwrap();
                let field_ids: Vec<_> = semantics
                    .nodes
                    .iter()
                    .filter(|(_, node)| node.role() == tessera_ui::accesskit::Role::TextInput)
                    .map(|(id, _)| *id)
                    .collect();
                assert_eq!(field_ids.len(), 2);
                assert!(!field_ids.contains(&semantics.focus));
                let preview = renderer.read_pixels().unwrap();
                assert_eq!(crop(&preview, 0), crop(&preview, 1));
                renderer.render_frames(6);
                assert_eq!(
                    renderer.read_pixels().unwrap(),
                    preview,
                    "preview carets stay visible across blink phases"
                );
                assert_eq!(renderer.diagnostics().paint_nodes_recorded, 0);
                let presentation = PRESENTATION.get().unwrap();
                presentation.set((None, true));
                renderer.render_frames(3);
                assert_ne!(
                    crop(&renderer.read_pixels().unwrap(), 0),
                    crop(&preview, 0),
                    "native resting decoration differs from focused presentation"
                );
                let rest = renderer.read_pixels().unwrap();
                presentation.set((Some(InteractionVisualState::Focus), true));
                renderer.render_frames(3);
                assert_eq!(
                    renderer.read_pixels().unwrap(),
                    preview,
                    "retained props restore the static focus presentation"
                );
                presentation.set((None, true));
                renderer.render_frames(3);
                assert_eq!(
                    renderer.read_pixels().unwrap(),
                    rest,
                    "retained props clear the static focus presentation again"
                );
                assert!(
                    editors
                        .iter()
                        .all(|editor| !editor.with(|editor| editor.focus_handler().is_focused()))
                );

                presentation.set((Some(InteractionVisualState::Focus), false));
                renderer.render_frames(3);
                let disabled = renderer.read_pixels().unwrap();
                presentation.set((None, false));
                renderer.render_frames(3);
                assert_eq!(
                    renderer.read_pixels().unwrap(),
                    disabled,
                    "disabled style takes precedence over presentation"
                );
                presentation.set((Some(InteractionVisualState::Focus), true));
                renderer.render_frames(3);
                editors[0].with(|editor| editor.focus_handler().request_focus());
                editors[0].with_mut(|editor| editor.set_text(""));
                renderer.render_frames(2);
                assert!(editors[0].with(|editor| editor.focus_handler().is_focused()));
                let focused = renderer.read_pixels().unwrap();
                assert_eq!(
                    crop(&focused, 0),
                    crop(&focused, 1),
                    "actual focus and presentation share the native focused appearance"
                );
                renderer.render_frames(4);
                let blink_off = renderer.read_pixels().unwrap();
                assert_ne!(
                    crop(&blink_off, 0),
                    crop(&blink_off, 1),
                    "only the actual focus caret blinks"
                );
                assert_eq!(renderer.diagnostics().measure_node_calls, 1);
                assert_eq!(renderer.diagnostics().geometry_nodes_updated, 0);
                assert_eq!(CHANGES.get(), 0);
            },
        );
    }
}
