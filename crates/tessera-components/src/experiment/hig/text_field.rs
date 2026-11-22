//! Editable text fields, secure fields, search fields, and text editors.
//!
//! ## Usage
//!
//! Collect names, passwords, search terms, and longer notes in forms and settings.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Callback, CallbackWith, Color, CursorEventContent, Dp, DrawModifierContent,
    DrawModifierContext, DrawModifierNode, Modifier, PointerInput, PointerInputModifierNode, Px,
    PxPosition, PxSize, State, launch,
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    experiment::hig::{
        motion::{Eased, FOCUS_RING, focus_ring},
        text::text,
        theme::{Appearance, HigTheme, TextStyle},
    },
    modifier::ModifierExt as _,
    pipelines::perceptual_fill::{PerceptualFillCommand, PerceptualStroke},
    shape_def::Shape,
    text_edit_core::{TextStyle as EditorTextStyle, text_edit_core},
    text_edit_input::{TextEditInput, TextEditInsets, attach_text_edit_input},
};

pub use crate::text_edit_core::TextEditorController as TextFieldController;

#[cfg(test)]
#[path = "text_field_tests.rs"]
mod tests;

const HORIZONTAL_PADDING: Dp = Dp(6.0);
const VERTICAL_PADDING: Dp = Dp(4.0);
const BEZEL_WIDTH: Dp = Dp(1.0);
const FOCUS_OUTSET: Dp = Dp(3.0);

#[derive(Clone, PartialEq)]
struct FieldFocusRing(Option<Color>, Dp);

impl DrawModifierNode for FieldFocusRing {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        content.draw(ctx.render_input);
        let Some(color) = self.0 else {
            return;
        };
        let mut metadata = ctx.render_input.metadata_mut();
        let Some(size) = metadata.computed_data() else {
            return;
        };
        if size.width <= Px::ZERO || size.height <= Px::ZERO {
            return;
        }
        let outset = FOCUS_OUTSET.to_px();
        let fragment = metadata.fragment_mut();
        let index = fragment.push_draw_command(PerceptualFillCommand {
            color,
            radii: [self.1; 2],
            insets: [Dp::ZERO; 4],
            clip_radius: None,
            edge_smoothness: 1.0,
            stroke: None,
            path: None,
            line: None,
            outline_width: Dp(4.0),
            snap_to_pixels: false,
        });
        let op = &mut fragment.ops_mut()[index as usize];
        op.position_override = Some(PxPosition::new(-outset, -outset));
        op.size_override = Some(PxSize::new(
            size.width + outset * 2,
            size.height + outset * 2,
        ));
    }
}

/// The appearance of a [`text_field`] or [`secure_field`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextFieldStyle {
    /// The platform default, a rounded border on macOS.
    #[default]
    Automatic,
    /// A field with a rounded border.
    RoundedBorder,
    /// A field with a square border.
    SquareBorder,
    /// The text alone, without a border, background, or focus ring.
    Plain,
}

/// The appearance of a [`text_editor`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextEditorStyle {
    /// The platform default, text on the text background color on macOS.
    #[default]
    Automatic,
    /// The text alone, without a background.
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldKind {
    Rounded,
    Square,
    Plain,
    Search,
    Editor { background: bool },
}

impl FieldKind {
    fn from_style(style: TextFieldStyle) -> Self {
        match style {
            TextFieldStyle::Automatic | TextFieldStyle::RoundedBorder => Self::Rounded,
            TextFieldStyle::SquareBorder => Self::Square,
            TextFieldStyle::Plain => Self::Plain,
        }
    }

    fn radius(self) -> Dp {
        match self {
            Self::Rounded => Dp(6.0),
            Self::Search => Dp(12.0),
            _ => Dp::ZERO,
        }
    }

    fn bezel(self) -> bool {
        matches!(self, Self::Rounded | Self::Square | Self::Search)
    }

    fn single_line(self) -> bool {
        !matches!(self, Self::Editor { .. })
    }
}

/// Bullet shown for each character of a secure field.
const BULLET: char = '\u{2022}';

/// Applies an edit of the bullets shown by a secure field to its real text. `selection` is the
/// selected character range before the edit; characters that are not bullets were typed or
/// pasted at that position.
fn apply_secure_edit(
    real: &str,
    shown_before: &str,
    shown_after: &str,
    selection: (usize, usize),
) -> String {
    let real: Vec<char> = real.chars().collect();
    let before = shown_before.chars().count();
    let after: Vec<char> = shown_after.chars().collect();
    let typed: Vec<char> = after.iter().copied().filter(|c| *c != BULLET).collect();
    let removed = (before + typed.len()).saturating_sub(after.len());
    let (start, end) = (selection.0.min(selection.1), selection.0.max(selection.1));
    let (from, to) = if let Some(at) = after.iter().position(|c| *c != BULLET) {
        (at, at + removed)
    } else if end > start {
        (start, end)
    } else if start >= removed {
        (start - removed, start)
    } else {
        (start, start + removed)
    };
    let (from, to) = (from.min(real.len()), to.min(real.len()));
    real[..from]
        .iter()
        .chain(&typed)
        .chain(&real[to..])
        .collect()
}

fn bullets(text: &str) -> String {
    std::iter::repeat_n(BULLET, text.chars().count()).collect()
}

#[derive(Clone, PartialEq)]
struct FieldBezel(Color, Dp);

impl DrawModifierNode for FieldBezel {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        {
            let mut metadata = ctx.render_input.metadata_mut();
            if let Some(size) = metadata.computed_data() {
                // The bezel extends past the field without expanding its hit target.
                let outset = Px::from(BEZEL_WIDTH);
                let fragment = metadata.fragment_mut();
                let index = fragment.push_draw_command(PerceptualFillCommand {
                    color: self.0,
                    radii: [self.1; 2],
                    insets: [Dp::ZERO; 4],
                    clip_radius: None,
                    edge_smoothness: 1.0,
                    stroke: None,
                    path: None,
                    line: None,
                    outline_width: BEZEL_WIDTH,
                    snap_to_pixels: false,
                });
                let op = &mut fragment.ops_mut()[index as usize];
                op.position_override = Some(PxPosition::new(-outset, -outset));
                op.size_override = Some(PxSize::new(
                    size.width + outset + outset,
                    size.height + outset + outset,
                ));
            }
        }
        content.draw(ctx.render_input);
    }
}

struct FieldColors {
    fill: Color,
    outline: Color,
    text: Color,
    placeholder: Color,
    selection: Color,
}

fn field_colors(appearance: Appearance, enabled: bool) -> FieldColors {
    match appearance {
        Appearance::Light => FieldColors {
            fill: Color::WHITE,
            outline: gray(0.831),
            text: gray(if enabled { 0.0194 } else { 0.521 }),
            placeholder: gray(0.216),
            selection: gray(0.716),
        },
        Appearance::Dark => FieldColors {
            fill: gray(0.00857),
            outline: gray(0.0222),
            text: gray(if enabled { 0.725 } else { 0.082 }),
            placeholder: gray(0.305),
            selection: gray(0.0612),
        },
    }
}

fn gray(linear: f32) -> Color {
    Color::from_rgb(linear, linear, linear)
}

fn srgb(r: u8, g: u8, b: u8) -> Color {
    let [r, g, b] = [r, g, b].map(|value| {
        let value = f32::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    });
    Color::from_rgb(r, g, b)
}

/// # text_field
///
/// Provides a compact, editable single-line field for names, search terms, and settings.
///
/// ## Usage
///
/// Collect one-line values in forms, search interfaces, and preference panes.
///
/// ## Parameters
///
/// - `placeholder` — optional hint shown while the field is empty.
/// - `initial_text` — initial value, used when the field owns its controller.
/// - `on_change` — optional value transformation and change callback.
/// - `on_submit` — callback invoked when the user submits with Enter.
/// - `style` — rounded, square, or no border; defaults to [`TextFieldStyle::Automatic`].
/// - `enabled`, `read_only` — whether the field accepts focus and text edits.
/// - `window_active` — whether to show active selection, caret, and focus visuals;
///   defaults to true.
/// - `modifier` — layout and node behavior.
/// - `controller` — optional external editor state.
/// - `accessibility_label`, `accessibility_description` — spoken name and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::text_field::text_field;
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let name = remember(String::new);
/// text_field()
///     .placeholder("Full name")
///     .on_change(move |value: String| {
///         name.set(value.clone());
///         value
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn text_field(
    #[prop(into)] placeholder: Option<String>,
    #[prop(into)] initial_text: Option<String>,
    on_change: Option<CallbackWith<String, String>>,
    on_submit: Option<Callback>,
    style: Option<TextFieldStyle>,
    enabled: Option<bool>,
    read_only: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    controller: Option<State<TextFieldController>>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    field(FieldKind::from_style(style.unwrap_or_default()), false)
        .placeholder_optional(placeholder)
        .initial_text_optional(initial_text)
        .on_change_optional(on_change)
        .on_submit_optional(on_submit)
        .enabled_optional(enabled)
        .read_only_optional(read_only)
        .window_active_optional(window_active)
        .modifier_optional(modifier)
        .controller_optional(controller)
        .accessibility_label_optional(accessibility_label)
        .accessibility_description_optional(accessibility_description);
}

/// # secure_field
///
/// Collects a password or other secret, showing a bullet for each character.
///
/// ## Usage
///
/// Ask for passwords, passcodes, and keys in sign-in and settings forms.
///
/// ## Parameters
///
/// - `placeholder` — optional hint shown while the field is empty.
/// - `on_change` — receives the entered secret and returns the value to keep.
/// - `on_submit` — callback invoked when the user submits with Enter.
/// - `style` — rounded, square, or no border; defaults to [`TextFieldStyle::Automatic`].
/// - `enabled` — whether the field accepts focus and edits.
/// - `window_active` — whether to show active selection, caret, and focus visuals.
/// - `modifier` — layout and node behavior.
/// - `accessibility_label`, `accessibility_description` — spoken name and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::text_field::secure_field;
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let password = remember(String::new);
/// secure_field().placeholder("Password").on_change(move |value: String| {
///     password.set(value.clone());
///     value
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn secure_field(
    #[prop(into)] placeholder: Option<String>,
    on_change: Option<CallbackWith<String, String>>,
    on_submit: Option<Callback>,
    style: Option<TextFieldStyle>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    field(FieldKind::from_style(style.unwrap_or_default()), true)
        .placeholder_optional(placeholder)
        .on_change_optional(on_change)
        .on_submit_optional(on_submit)
        .enabled_optional(enabled)
        .window_active_optional(window_active)
        .modifier_optional(modifier)
        .accessibility_label_optional(accessibility_label)
        .accessibility_description_optional(accessibility_description);
}

/// # search_field
///
/// Collects a search query in a rounded field with a search glyph and a clear button.
///
/// ## Usage
///
/// Filter a list or search a document from a toolbar or the top of a view.
///
/// ## Parameters
///
/// - `placeholder` — hint shown while the field is empty; defaults to "Search".
/// - `on_change` — receives each edit of the query and returns the value to keep.
/// - `on_submit` — callback invoked when the user submits with Enter.
/// - `enabled` — whether the field accepts focus and edits.
/// - `modifier` — layout and node behavior.
/// - `controller` — optional external editor state.
/// - `accessibility_label` — spoken name; defaults to the placeholder.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::text_field::search_field;
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let query = remember(String::new);
/// search_field().on_change(move |value: String| {
///     query.set(value.clone());
///     value
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn search_field(
    #[prop(into)] placeholder: Option<String>,
    on_change: Option<CallbackWith<String, String>>,
    on_submit: Option<Callback>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    controller: Option<State<TextFieldController>>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    field(FieldKind::Search, false)
        .placeholder(placeholder.unwrap_or_else(|| "Search".to_owned()))
        .on_change_optional(on_change)
        .on_submit_optional(on_submit)
        .enabled_optional(enabled)
        .modifier_optional(modifier)
        .controller_optional(controller)
        .accessibility_label_optional(accessibility_label);
}

/// # text_editor
///
/// Edits multiple lines of plain text.
///
/// ## Usage
///
/// Write notes, descriptions, and other longer text in a form or inspector.
///
/// ## Parameters
///
/// - `initial_text` — initial value, used when the editor owns its controller.
/// - `on_change` — optional value transformation and change callback.
/// - `style` — with or without the text background; defaults to
///   [`TextEditorStyle::Automatic`].
/// - `enabled`, `read_only` — whether the editor accepts focus and edits.
/// - `window_active` — whether to show active selection and caret visuals.
/// - `modifier` — size and node behavior; the editor is at least 80pt tall.
/// - `controller` — optional external editor state.
/// - `accessibility_label`, `accessibility_description` — spoken name and context.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::text_field::text_editor;
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let notes = remember(String::new);
/// text_editor()
///     .accessibility_label("Notes")
///     .on_change(move |value: String| {
///         notes.set(value.clone());
///         value
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn text_editor(
    #[prop(into)] initial_text: Option<String>,
    on_change: Option<CallbackWith<String, String>>,
    style: Option<TextEditorStyle>,
    enabled: Option<bool>,
    read_only: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    controller: Option<State<TextFieldController>>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    field(
        FieldKind::Editor {
            background: style.unwrap_or_default() == TextEditorStyle::Automatic,
        },
        false,
    )
    .initial_text_optional(initial_text)
    .on_change_optional(on_change)
    .enabled_optional(enabled)
    .read_only_optional(read_only)
    .window_active_optional(window_active)
    .modifier_optional(modifier)
    .controller_optional(controller)
    .accessibility_label(accessibility_label.unwrap_or_else(|| "Text editor".to_owned()))
    .accessibility_description_optional(accessibility_description);
}

#[tessera]
fn field(
    kind: FieldKind,
    secure: bool,
    #[prop(into)] placeholder: Option<String>,
    #[prop(into)] initial_text: Option<String>,
    on_change: Option<CallbackWith<String, String>>,
    on_submit: Option<Callback>,
    enabled: Option<bool>,
    read_only: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    controller: Option<State<TextFieldController>>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let single_line = kind.single_line();
    let enabled = enabled.unwrap_or(true);
    let read_only = read_only.unwrap_or(false);
    let window_active = window_active.unwrap_or(true);
    let style = use_context::<TextStyle>()
        .map(|style| style.get())
        .unwrap_or_default();
    let appearance = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance)
        .unwrap_or_default();
    let colors = field_colors(appearance, enabled);
    let style = EditorTextStyle {
        font_size: style.font_size,
        line_height: Some(style.line_height),
        font_weight: style.font_weight,
        letter_spacing: style.letter_spacing,
        font_family: style.font_family,
    };
    let controller = if let Some(controller) = controller {
        controller
    } else {
        remember(|| {
            let mut controller = TextFieldController::new(style.font_size, style.line_height);
            controller.set_single_line(single_line);
            if let Some(initial_text) = initial_text.as_deref() {
                controller.set_text(initial_text);
            }
            controller.set_text_raster_style(style, true);
            controller
        })
    };
    if controller.with(TextFieldController::single_line) != single_line {
        controller.with_mut(|editor| editor.set_single_line(single_line));
    }
    if controller.with(|editor| editor.text_style() != style || editor.native_ink().is_none()) {
        controller.with_mut(|editor| editor.set_text_raster_style(style, true));
    }
    if controller.with(|editor| editor.text_color()) != colors.text {
        controller.with_mut(|editor| editor.set_text_color(colors.text));
    }
    let active_selection = if window_active {
        match appearance {
            Appearance::Light => Color::from_rgb(0.4508, 0.6795, 1.0),
            Appearance::Dark => Color::from_rgb(0.0497, 0.125, 0.2582),
        }
    } else {
        colors.selection
    };
    let cursor_color = srgb(0, 122, 255);
    if controller.with(|editor| editor.cursor_color()) != cursor_color {
        controller.with_mut(|editor| editor.set_cursor_color(cursor_color));
    }
    if controller.with(|editor| editor.selection_color()) != active_selection {
        controller.with_mut(|editor| editor.set_selection_color(active_selection));
    }
    if !enabled {
        controller
            .with(|editor| *editor.focus_handler())
            .clear_focus();
    }
    let focus = controller.with(|editor| *editor.focus_handler());
    let focused = kind.bezel() && enabled && window_active && focus.is_focused();
    let ring = remember(|| Eased::new(0.0, FOCUS_RING));
    launch(focused, focus_ring(ring, focused));
    let ring = if focused {
        ring.with(Eased::value)
    } else {
        0.0
    };
    let focus_ring = (ring > 0.0).then(|| {
        match appearance {
            Appearance::Light => srgb(1, 103, 245),
            Appearance::Dark => srgb(26, 170, 254),
        }
        .with_alpha(0.5 * ring)
    });
    let on_change = on_change.unwrap_or_else(CallbackWith::identity);
    // A secure field edits bullets and keeps the real text beside them. The editor is being
    // written while it reports a change, so the shown text and selection are tracked here.
    let secret = remember(String::new);
    let shown = remember(|| (String::new(), (0usize, 0usize)));
    if secure {
        let current = controller.with(|editor| {
            let selection = editor.selection();
            let width = BULLET.len_utf8();
            (
                editor.text().to_string(),
                (selection.start / width, selection.end / width),
            )
        });
        if shown.get() != current {
            shown.set(current);
        }
    }
    let on_change = if secure {
        CallbackWith::new(move |next: String| {
            let (before, selection) = shown.get();
            let real = apply_secure_edit(&secret.get(), &before, &next, selection);
            let kept = on_change.call(real);
            let masked = bullets(&kept);
            let typed = next.chars().filter(|c| *c != BULLET).count();
            let caret = (selection.0.min(selection.1) + typed).min(masked.chars().count());
            shown.set((masked.clone(), (caret, caret)));
            secret.set(kept);
            masked
        })
    } else {
        on_change
    };
    let horizontal = match kind {
        FieldKind::Plain => Dp::ZERO,
        FieldKind::Search => Dp(22.0),
        _ => HORIZONTAL_PADDING,
    };
    let input = TextEditInput {
        enabled,
        read_only,
        is_secure: secure,
        single_line,
        on_change,
        on_submit: on_submit.unwrap_or_else(Callback::noop),
        input_transform: None,
        accessibility_label: Some(accessibility_label.unwrap_or_else(|| {
            placeholder
                .clone()
                .unwrap_or_else(|| "Text field".to_owned())
        })),
        accessibility_description,
        content_insets: TextEditInsets {
            horizontal,
            vertical: VERTICAL_PADDING,
            border: Dp::ZERO,
        },
    };
    let radius = kind.radius();
    let mut modifier = modifier
        .unwrap_or_default()
        .default_min_size(Dp(160.0), if single_line { Dp(24.0) } else { Dp(80.0) });
    if kind.bezel() {
        modifier = modifier
            .push_draw(FieldFocusRing(focus_ring, Dp(radius.0 + 2.0)))
            .push_draw(FieldBezel(colors.outline, Dp(radius.0 + 1.0)));
    }
    if kind != FieldKind::Plain && kind != (FieldKind::Editor { background: false }) {
        modifier = modifier.background_with_shape(colors.fill, Shape::rounded_rectangle(radius));
    }
    let modifier = modifier.then(
        Modifier::new()
            .focus_requester(focus)
            .focusable()
            .focus_properties(
                tessera_ui::FocusProperties::new()
                    .can_focus(enabled)
                    .can_request_focus(enabled),
            ),
    );
    let placeholder_visible = controller.with(|editor| editor.text().is_empty());
    let placeholder_color = colors.placeholder;
    let modifier = if kind == FieldKind::Search {
        let clear = (!placeholder_visible && enabled).then(|| {
            Callback::new(move || {
                let kept = on_change.call(String::new());
                controller.with_mut(|editor| editor.set_text(&kept));
            })
        });
        modifier
            .push_pointer_input(ClearPress(clear))
            .push_draw(SearchDecorations {
                color: placeholder_color,
                clear: clear.is_some(),
            })
    } else {
        modifier
    };
    let modifier = attach_text_edit_input(modifier, input, controller)
        .padding_symmetric(horizontal, VERTICAL_PADDING);
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(modifier)
        .children(move || {
            if placeholder_visible && let Some(placeholder) = placeholder.as_deref() {
                text(placeholder)
                    .color(placeholder_color)
                    .single_line(true)
                    .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
                        hidden: true,
                        ..Default::default()
                    }));
            }
            text_edit_core()
                .controller(controller)
                .window_active(window_active)
                .selected_text_color(match appearance {
                    Appearance::Light => Color::BLACK,
                    Appearance::Dark => Color::WHITE,
                });
        });
}

/// Clears a search field when the press lands on its trailing clear button.
struct ClearPress(Option<Callback>);

impl PointerInputModifierNode for ClearPress {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        let Some(clear) = self.0 else {
            return;
        };
        let Some(position) = input.cursor_position_rel else {
            return;
        };
        let width = input.computed_data.width.to_f32();
        eprintln!(
            "DBG clear pass {:?} pos {:?} width {}",
            input.pass, position, width
        );
        if position.x < width - Dp(22.0).to_pixels_f32() {
            return;
        }
        let mut pressed = false;
        for change in input.pointer_changes.iter_mut() {
            if !change.is_consumed() && matches!(change.content, CursorEventContent::Pressed(_)) {
                change.consume();
                pressed = true;
            }
        }
        if pressed {
            clear.call();
        }
    }
}

/// The search glyph at the leading edge and, with text, the clear button at the trailing edge.
#[derive(Clone, PartialEq)]
struct SearchDecorations {
    color: Color,
    clear: bool,
}

fn decoration(color: Color, area: [Dp; 4]) -> PerceptualFillCommand {
    PerceptualFillCommand {
        outline_width: Dp::ZERO,
        snap_to_pixels: false,
        path: None,
        color,
        radii: [Dp::ZERO; 2],
        insets: area,
        clip_radius: None,
        line: None,
        edge_smoothness: 1.0,
        stroke: None,
    }
}

fn line(color: Color, width: Dp, points: [[f32; 2]; 3], area: [Dp; 4]) -> PerceptualFillCommand {
    PerceptualFillCommand {
        stroke: Some(PerceptualStroke {
            raster_mask: false,
            color,
            width,
            points,
        }),
        ..decoration(Color::TRANSPARENT, area)
    }
}

impl DrawModifierNode for SearchDecorations {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        let mut metadata = ctx.render_input.metadata_mut();
        if let Some(size) = metadata.computed_data() {
            let width = Dp::from_pixels_f32(size.width.to_f32()).0;
            let height = Dp::from_pixels_f32(size.height.to_f32()).0;
            let top = (height - 12.0) * 0.5;
            let bottom = height - top - 12.0;
            let fragment = metadata.fragment_mut();
            // Insets are top, right, bottom, left.
            fragment.push_draw_command(PerceptualFillCommand {
                outline_width: Dp(1.5),
                radii: [Dp(4.5); 2],
                ..decoration(
                    self.color,
                    [Dp(top), Dp(width - 16.0), Dp(bottom + 3.0), Dp(7.0)],
                )
            });
            fragment.push_draw_command(line(
                self.color,
                Dp(1.5),
                [[0.0, 0.0], [0.5, 0.5], [1.0, 1.0]],
                [Dp(top + 8.0), Dp(width - 19.0), Dp(bottom), Dp(15.0)],
            ));
            if self.clear {
                fragment.push_draw_command(PerceptualFillCommand {
                    radii: [Dp(6.0); 2],
                    ..decoration(self.color, [Dp(top), Dp(6.0), Dp(bottom), Dp(width - 18.0)])
                });
                let cross = [Dp(top + 3.5), Dp(9.5), Dp(bottom + 3.5), Dp(width - 14.5)];
                fragment.push_draw_command(line(
                    Color::WHITE,
                    Dp(1.3),
                    [[0.0, 0.0], [0.5, 0.5], [1.0, 1.0]],
                    cross,
                ));
                fragment.push_draw_command(line(
                    Color::WHITE,
                    Dp(1.3),
                    [[1.0, 0.0], [0.5, 0.5], [0.0, 1.0]],
                    cross,
                ));
            }
        }
        content.draw(ctx.render_input);
    }
}
