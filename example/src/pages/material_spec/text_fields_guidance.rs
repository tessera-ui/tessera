//! Do and don't examples for text field guidelines and accessibility advice.
//!
//! ## Usage
//!
//! Contrast recommended and discouraged text field layouts, labels, errors and
//! outlines.

use tessera_components::{
    alignment::CrossAxisAlignment,
    boxed::boxed,
    column::column,
    modifier::{InteractionVisualState, ModifierExt as _, Padding},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    text_field::{TextFieldLineLimit, text_field},
    theme::MaterialTheme,
};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, tessera, use_context};

use super::{gap, panel, type_style};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const CAUTION_COLOR: Color = Color::from_rgb(0.722, 0.420, 0.0);

#[tessera]
fn verdict(tone: u8, #[prop(into)] note: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (mark, color) = match tone {
        0 => ("Do", DO_COLOR),
        1 => ("Don't", DONT_COLOR),
        _ => ("Caution", CAUTION_COLOR),
    };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed().modifier(
                Modifier::new()
                    .fill_max_width()
                    .height(Dp(4.0))
                    .background_with_shape(color, Shape::rounded_rectangle(Dp(2.0))),
            );
            gap(Dp(12.0));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    text()
                        .content(mark)
                        .style(type_style(14.0, 20.0))
                        .color(color);
                });
            gap(Dp(4.0));
            text()
                .content(note.clone())
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn example(tone: u8, #[prop(into)] note: String, content: RenderSlot) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            panel(content.clone());
            gap(Dp(12.0));
            verdict(tone, note.clone());
        });
}

#[tessera]
fn pair(left: RenderSlot, right: RenderSlot) {
    row()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let left = left.clone();
            let right = right.clone();
            column()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    left.render();
                });
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            column()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    right.render();
                });
        });
}

#[tessera]
fn field(
    #[prop(into)] label: String,
    outlined: bool,
    value: Option<String>,
    error: Option<bool>,
    focused: Option<bool>,
    outline: Option<Color>,
) {
    let base = if outlined {
        text_field().outlined()
    } else {
        text_field().filled()
    };
    base.label(label.clone())
        .accessibility_label(label.clone())
        .min_width(Dp::ZERO)
        .is_error(error.unwrap_or(false))
        .modifier(Modifier::new().fill_max_width());
    if let Some(value) = value {
        base.initial_text(value);
    }
    if focused.unwrap_or(false) {
        base.presentation_state(InteractionVisualState::Focus);
    }
    if let Some(color) = outline {
        base.border_color(color);
    }
}

#[tessera]
fn support(#[prop(into)] value: String, error: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(value)
        .color(if error {
            scheme.error
        } else {
            scheme.on_surface_variant
        })
        .style(type_style(12.0, 16.0))
        .modifier(Modifier::new().padding(Padding::symmetric(Dp(16.0), Dp(4.0))));
}

#[tessera]
fn form(outlined: bool, mixed: bool) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            field("First name", outlined).value("Odette".to_owned());
            gap(Dp(16.0));
            field("Last name", if mixed { !outlined } else { outlined });
            gap(Dp(16.0));
            field("Email", outlined);
        });
}

/// Separate filled and outlined regions instead of mixing them in one form.
#[tessera]
pub(super) fn variant_regions() {
    pair(
        RenderSlot::new(|| {
            example(
                0,
                "When using both variants of text fields in a UI, separate them by region",
                RenderSlot::new(|| {
                    form(false, false);
                    gap(Dp(24.0));
                    field("Promo code", true).value("SPRING".to_owned());
                }),
            );
        }),
        RenderSlot::new(|| {
            example(
                1,
                "When using both variants of text fields, don't use both next to each other or within the same form",
                RenderSlot::new(|| {
                    form(false, true);
                }),
            );
        }),
    );
}

/// Show labels that are truncated or wrapped instead of short and fully
/// visible.
#[tessera]
pub(super) fn label_misuse() {
    pair(
        RenderSlot::new(|| {
            example(
                1,
                "Don't truncate label text. Keep it short, clear, and fully visible.",
                RenderSlot::new(|| {
                    boxed()
                        .modifier(Modifier::new().width(Dp(200.0)))
                        .children(|| {
                            field("Enter the name printed on your card…", true);
                        });
                }),
            );
        }),
        RenderSlot::new(|| {
            example(
                1,
                "Label text shouldn't take up multiple lines",
                RenderSlot::new(|| {
                    column()
                        .modifier(Modifier::new().width(Dp(200.0)))
                        .children(|| {
                            text()
                                .content("Enter the name printed\non your card")
                                .style(type_style(12.0, 16.0))
                                .modifier(
                                    Modifier::new().padding(Padding::symmetric(Dp(16.0), Dp(4.0))),
                                );
                            field("", true);
                        });
                }),
            );
        }),
    );
}

/// Replace supporting text with error text instead of stacking both.
#[tessera]
pub(super) fn error_swap() {
    pair(
        RenderSlot::new(|| {
            example(
                0,
                "Swap supporting text with error text",
                RenderSlot::new(|| {
                    field("Password", true)
                        .value("pass".to_owned())
                        .error(true)
                        .focused(true);
                    support("Must be at least 8 characters", true);
                }),
            );
        }),
        RenderSlot::new(|| {
            example(
                1,
                "Don't add error text in addition to supporting text, as their appearance will shift content",
                RenderSlot::new(|| {
                    field("Password", true)
                        .value("pass".to_owned())
                        .error(true)
                        .focused(true);
                    support("Use 8 or more characters", false);
                    support("Must be at least 8 characters", true);
                }),
            );
        }),
    );
    gap(Dp(24.0));
    example(
        2,
        "Long errors can wrap to multiple lines if there isn't enough space to clearly describe the error. In this case, ensure padding between text fields is sufficient to prevent multi-lined errors from bumping layout content.",
        RenderSlot::new(|| {
            boxed()
                .modifier(Modifier::new().width(Dp(280.0)))
                .children(|| {
                    column().children(|| {
                        field("Email", true).value("odette@".to_owned()).error(true);
                        support(
                            "Enter an email address in the format name@example.com",
                            true,
                        );
                        gap(Dp(24.0));
                        field("Password", true);
                    });
                });
        }),
    );
}

/// Compare an outline that meets 3:1 contrast with one that does not.
#[tessera]
pub(super) fn outline_contrast() {
    pair(
        RenderSlot::new(|| {
            example(
                0,
                "Make sure the container outline has a minimum contrast of 3:1 to the background",
                RenderSlot::new(|| {
                    field("Email", true);
                }),
            );
        }),
        RenderSlot::new(|| {
            example(
                1,
                "Don't choose colors that won't pass Material's minimum contrast of 3:1",
                RenderSlot::new(|| {
                    field("Email", true).outline(Color::from_rgb(0.86, 0.85, 0.88));
                }),
            );
        }),
    );
}

/// Editable single-line, multi-line and fixed-height text area inputs.
#[tessera]
pub(super) fn input_modes() {
    input_mode(0);
    gap(Dp(24.0));
    input_mode(1);
    gap(Dp(24.0));
    input_mode(2);
}

#[tessera]
fn input_mode(mode: u8) {
    let (label, note, value) = match mode {
        0 => (
            "Single-line field",
            "In single-line fields, as the cursor reaches the right field edge, text longer than the input line automatically scrolls left.",
            "A response that is longer than the input line keeps scrolling horizontally as you type",
        ),
        1 => (
            "Multi-line field",
            "In multi-line fields, overflow text wraps onto a new line and the field expands, shifting elements below it downward.",
            "This field starts as a single line. Keep typing and it grows to fit each new line.",
        ),
        _ => (
            "Text area",
            "Text areas are a fixed height. Overflow text wraps onto a new line and scrolls vertically when the cursor reaches the bottom of the field.",
            "Text areas suit longer responses.\nThis example keeps a fixed height.\nAdd more lines to scroll inside it.\nLine four.\nLine five.\nLine six.",
        ),
    };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            panel(RenderSlot::new(move || {
                let base = text_field()
                    .outlined()
                    .label(label)
                    .accessibility_label(label)
                    .initial_text(value)
                    .min_width(Dp::ZERO)
                    .line_limit(if mode == 0 {
                        TextFieldLineLimit::SingleLine
                    } else {
                        TextFieldLineLimit::MultiLine
                    });
                if mode == 2 {
                    base.min_height(Dp(120.0))
                        .modifier(Modifier::new().width(Dp(320.0)).height(Dp(120.0)))
                } else {
                    base.modifier(Modifier::new().width(Dp(320.0)))
                };
            }));
            gap(Dp(12.0));
            text()
                .content(note)
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0));
        });
}
