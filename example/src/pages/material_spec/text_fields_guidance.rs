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
    md3::{
        text::text,
        text_field::{TextFieldLineLimit, text_field},
        theme::MaterialTheme,
    },
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
};
use tessera_foundation::modifier::{InteractionVisualState, ModifierExt as _, Padding};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, tessera, use_context};

use crate::res::I18N;

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
        _ => ("ui-caution", CAUTION_COLOR),
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
                    text(I18N.text(mark))
                        .style(type_style(14.0, 20.0))
                        .color(color);
                });
            gap(Dp(4.0));
            text(I18N.text(&(note)))
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
    base.label(I18N.text(&(label)))
        .accessibility_label(I18N.text(&(label)))
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
    text(I18N.text(value))
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
            field("ui-first-name-5955", outlined).value("ui-odette".to_owned());
            gap(Dp(16.0));
            field(
                "ui-last-name-5957",
                if mixed { !outlined } else { outlined },
            );
            gap(Dp(16.0));
            field("ui-email", outlined);
        });
}

/// Separate filled and outlined regions instead of mixing them in one form.
#[tessera]
pub(super) fn variant_regions() {
    pair(
        RenderSlot::new(|| {
            example(
                0,
                "ui-when-using-both-variants-of-text-fields-in-a-ui-separate-them-by-region",
                RenderSlot::new(|| {
                    form(false, false);
                    gap(Dp(24.0));
                    field("ui-promo-code", true).value("SPRING".to_owned());
                }),
            );
        }),
        RenderSlot::new(|| {
            example(
                1,
                "ui-when-using-both-variants-of-text-fields-don-t-use-both-next-to-each-other-or-within-the-sa",
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
                "ui-don-t-truncate-label-text-keep-it-short-clear-and-fully-visible",
                RenderSlot::new(|| {
                    boxed()
                        .modifier(Modifier::new().width(Dp(200.0)))
                        .children(|| {
                            field("ui-enter-the-name-printed-on-your-card", true);
                        });
                }),
            );
        }),
        RenderSlot::new(move || {
            example(
                1,
                "ui-label-text-shouldn-t-take-up-multiple-lines",
                RenderSlot::new(move || {
                    column()
                        .modifier(Modifier::new().width(Dp(200.0)))
                        .children(move || {
                            text(I18N.text("ui-enter-the-name-printed-on-your-card-5964"))
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
                "ui-swap-supporting-text-with-error-text",
                RenderSlot::new(|| {
                    field("ui-password", true)
                        .value("pass".to_owned())
                        .error(true)
                        .focused(true);
                    support("ui-must-be-at-least-8-characters", true);
                }),
            );
        }),
        RenderSlot::new(|| {
            example(
                1,
                "ui-don-t-add-error-text-in-addition-to-supporting-text-as-their-appearance-will-shift-content",
                RenderSlot::new(|| {
                    field("ui-password", true)
                        .value("pass".to_owned())
                        .error(true)
                        .focused(true);
                    support("ui-use-8-or-more-characters", false);
                    support("ui-must-be-at-least-8-characters", true);
                }),
            );
        }),
    );
    gap(Dp(24.0));
    example(
        2,
        "ui-long-errors-can-wrap-to-multiple-lines-if-there-isn-t-enough-space-to-clearly-describe-the",
        RenderSlot::new(|| {
            boxed()
                .modifier(Modifier::new().width(Dp(280.0)))
                .children(|| {
                    column().children(|| {
                        field("ui-email", true)
                            .value("odette@".to_owned())
                            .error(true);
                        support(
                            "ui-enter-an-email-address-in-the-format-name-example-com",
                            true,
                        );
                        gap(Dp(24.0));
                        field("ui-password", true);
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
                "ui-make-sure-the-container-outline-has-a-minimum-contrast-of-3-1-to-the-background",
                RenderSlot::new(|| {
                    field("ui-email", true);
                }),
            );
        }),
        RenderSlot::new(|| {
            example(
                1,
                "ui-don-t-choose-colors-that-won-t-pass-material-s-minimum-contrast-of-3-1",
                RenderSlot::new(|| {
                    field("ui-email", true).outline(Color::from_rgb(0.86, 0.85, 0.88));
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
            "ui-single-line-field",
            "ui-in-single-line-fields-as-the-cursor-reaches-the-right-field-edge-text-longer-than-the-inpu",
            "ui-a-response-that-is-longer-than-the-input-line-keeps-scrolling-horizontally-as-you-type",
        ),
        1 => (
            "ui-multi-line-field",
            "ui-in-multi-line-fields-overflow-text-wraps-onto-a-new-line-and-the-field-expands-shifting-el",
            "ui-this-field-starts-as-a-single-line-keep-typing-and-it-grows-to-fit-each-new-line",
        ),
        _ => (
            "ui-text-area",
            "ui-text-areas-are-a-fixed-height-overflow-text-wraps-onto-a-new-line-and-scrolls-vertically-w",
            "ui-text-areas-suit-longer-responses-this-example-keeps-a-fixed-height-add-more-lines-to-scrol",
        ),
    };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            panel(RenderSlot::new(move || {
                let base = text_field()
                    .outlined()
                    .label(I18N.text(label))
                    .accessibility_label(I18N.text(label))
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
            text(I18N.text(note))
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0));
        });
}
