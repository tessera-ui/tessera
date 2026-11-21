//! Editable text fields for short-form input.
//!
//! ## Usage
//!
//! Collect names, search terms, and other single-line values in forms and settings.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Callback, CallbackWith, Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode,
    Modifier, Px, PxPosition, PxSize, State,
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    experiment::hig::{
        text::text,
        theme::{Appearance, HigTheme, TextStyle},
    },
    modifier::ModifierExt as _,
    pipelines::perceptual_fill::PerceptualFillCommand,
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
const CORNER_RADIUS: Dp = Dp(7.0);
const BEZEL_WIDTH: Dp = Dp(1.0);
const FOCUS_OUTSET: Dp = Dp(3.0);

#[derive(Clone, PartialEq)]
struct FieldFocusRing(Option<Color>);

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
            radii: [Dp(8.0); 2],
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

#[derive(Clone, PartialEq)]
struct FieldBezel(Color);

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
                    radii: [CORNER_RADIUS; 2],
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
/// use tessera_ui::{tessera, testing::UiTest};
///
/// #[tessera]
/// fn app() {
///     text_field()
///         .placeholder("Search settings")
///         .accessibility_label("Search settings field");
/// }
/// let frame = UiTest::new().content(app).frame();
/// assert!(frame.node("Search settings field").bounds().height.0 >= 24);
/// ```
#[tessera]
pub fn text_field(
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
            controller.set_single_line(true);
            if let Some(initial_text) = initial_text.as_deref() {
                controller.set_text(initial_text);
            }
            controller.set_text_raster_style(style, true);
            controller
        })
    };
    if !controller.with(TextFieldController::single_line) {
        controller.with_mut(|editor| editor.set_single_line(true));
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
    let focus_ring = (enabled && window_active && focus.is_focused()).then(|| {
        match appearance {
            Appearance::Light => srgb(1, 103, 245),
            Appearance::Dark => srgb(26, 170, 254),
        }
        .with_alpha(0.5)
    });
    let input = TextEditInput {
        enabled,
        read_only,
        is_secure: false,
        single_line: true,
        on_change: on_change.unwrap_or_else(CallbackWith::identity),
        on_submit: on_submit.unwrap_or_else(Callback::noop),
        input_transform: None,
        accessibility_label: Some(accessibility_label.unwrap_or_else(|| {
            placeholder
                .clone()
                .unwrap_or_else(|| "Text field".to_owned())
        })),
        accessibility_description,
        content_insets: TextEditInsets {
            horizontal: HORIZONTAL_PADDING,
            vertical: VERTICAL_PADDING,
            border: Dp::ZERO,
        },
    };
    let modifier = modifier
        .unwrap_or_default()
        .default_min_size(Dp(160.0), Dp(24.0))
        .push_draw(FieldFocusRing(focus_ring))
        .push_draw(FieldBezel(colors.outline))
        .background_with_shape(colors.fill, Shape::rounded_rectangle(Dp(6.0)))
        .then(
            Modifier::new()
                .focus_requester(focus)
                .focusable()
                .focus_properties(
                    tessera_ui::FocusProperties::new()
                        .can_focus(enabled)
                        .can_request_focus(enabled),
                ),
        );
    let modifier = attach_text_edit_input(modifier, input, controller)
        .padding_symmetric(HORIZONTAL_PADDING, VERTICAL_PADDING);
    let placeholder_visible = controller.with(|editor| editor.text().is_empty());
    let placeholder_color = colors.placeholder;
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
