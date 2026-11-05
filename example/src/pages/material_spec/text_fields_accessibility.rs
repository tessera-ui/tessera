//! Text field accessibility labeling diagrams.
//!
//! ## Usage
//!
//! Inspect accessible field names, actions, errors and supporting text.

use tessera_components::{
    boxed::boxed,
    column::column,
    icon_button::{IconButtonColorStyle, icon_button},
    image::{TryIntoImageData, image},
    material_icons::filled,
    menus::{MenuAnchor, MenuController, menu_item, menu_provider},
    modifier::{InteractionVisualState, ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    spacer::spacer,
    text::text,
    text_field::{TextFieldContextMenu, text_field},
    text_input::TextInputController,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, PxPosition, PxSize, RenderSlot, accesskit::Role, remember, tessera,
    use_context,
};

use crate::res::I18N;

use super::{gap, srgb, type_style};

#[tessera]
pub(super) fn labeling() {
    paragraph("ui-the-accessible-name-matches-the-visible-text-field-label");
    gap(Dp(24.0));
    labeling_figure(0);
    gap(Dp(24.0));
    paragraph(
        "ui-name-interactive-trailing-icons-by-their-action-the-password-action-changes-between-show-p",
    );
    gap(Dp(24.0));
    labeling_figure(1);
    for (kind, description) in [
        (
            2,
            "ui-give-affixes-a-meaningful-accessible-name-such-as-euro-or-at-gmail-dot-com",
        ),
        (
            3,
            "ui-expose-error-messages-with-an-alert-role-and-describe-how-to-correct-the-input",
        ),
        (
            4,
            "ui-name-the-character-counter-and-announce-the-number-of-characters-entered-and-the-limit",
        ),
        (
            5,
            "ui-supporting-text-has-its-own-accessible-name-and-is-associated-with-the-field",
        ),
        (
            6,
            "ui-include-the-asterisk-in-a-required-field-s-accessible-name-and-explain-it-in-supporting-te",
        ),
    ] {
        gap(Dp(24.0));
        paragraph(description);
        gap(Dp(24.0));
        labeling_figure(kind);
    }
}

#[tessera]
fn paragraph(#[prop(into)] value: String) {
    text()
        .content(I18N.text(value))
        .style(type_style(16.0, 24.0));
}

#[tessera]
pub(super) fn labeling_figure(kind: usize) {
    super::text_fields::field_canvas(
        676.0,
        300.0,
        RenderSlot::new(move || {
            let scale = 1.0;
            let theme = use_context::<MaterialTheme>().unwrap().get();
            let scheme = theme.color_scheme.clone();
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(300.0 * scale))
                        .clip_to_bounds()
                        .background(scheme.surface_container_low),
                )
                .children(move || {
                    let theme = theme.clone();
                    material_theme()
                        .theme(move || {
                            let mut theme = theme.clone();
                            for style in [
                                &mut theme.typography.body_large,
                                &mut theme.typography.body_small,
                            ] {
                                style.font_size = Dp(style.font_size.0 * scale);
                                style.line_height =
                                    style.line_height.map(|height| Dp(height.0 * scale));
                                style.letter_spacing = Dp(style.letter_spacing.0 * scale);
                            }
                            theme
                        })
                        .child(move || match kind {
                            0 => {
                                email_diagram(scale);
                            }
                            1 => {
                                password_diagram(scale);
                            }
                            2 => {
                                affixes_diagram(scale);
                            }
                            3 => {
                                error_diagram(scale);
                            }
                            4 => {
                                counter_diagram(scale);
                            }
                            5 => {
                                supporting_diagram(scale);
                            }
                            _ => {
                                required_diagram(scale);
                            }
                        });
                });
        }),
    );
}

#[tessera]
fn rectangle(x: f64, y: f64, width: f64, height: f64, color: Color, scale: f64) {
    spacer().modifier(
        Modifier::new()
            .offset(Dp(x * scale), Dp(y * scale))
            .size(Dp(width * scale), Dp(height * scale))
            .background(color),
    );
}

#[tessera]
fn highlight(x: f64, y: f64, width: f64, height: f64, radius: f64, scale: f64) {
    spacer().modifier(
        Modifier::new()
            .offset(Dp(x * scale), Dp(y * scale))
            .size(Dp(width * scale), Dp(height * scale))
            .border_with_shape(
                Dp(4.0 * scale),
                srgb(132, 69, 143),
                Shape::rounded_rectangle(Dp(radius * scale)),
            ),
    );
}

#[tessera]
fn callout(x: f64, y: f64, width: f64, height: f64, label: String, role: String, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let mut emphasis = type_style(16.0 * scale, 20.0 * scale);
    emphasis.font_weight = 700;
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(width * scale), Dp(height * scale))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(12.0 * scale)))
                .border_with_shape(
                    Dp(4.0 * scale),
                    srgb(132, 69, 143),
                    Shape::rounded_rectangle(Dp(12.0 * scale)),
                ),
        )
        .children(move || {
            let label = label.clone();
            let role = role.clone();
            column()
                .modifier(
                    Modifier::new()
                        .offset(Dp(16.0 * scale), Dp(16.0 * scale))
                        .width(Dp((width - 32.0) * scale)),
                )
                .children(move || {
                    text()
                        .content(I18N.text("ui-label"))
                        .style(type_style(14.0 * scale, 18.0 * scale));
                    text()
                        .content(I18N.text(&(label)))
                        .style(emphasis.clone())
                        .color(srgb(132, 69, 143));
                    gap(Dp(12.0 * scale));
                    text()
                        .content(I18N.text("ui-role"))
                        .style(type_style(14.0 * scale, 18.0 * scale));
                    text()
                        .content(I18N.text(&(role)))
                        .style(emphasis)
                        .color(srgb(132, 69, 143));
                });
        });
}

#[tessera]
fn email_diagram(scale: f64) {
    let photo = remember(|| {
        include_bytes!("../../../assets/material_spec/text-field-accessibility-email.png")
            .as_slice()
            .try_into_image_data()
            .expect("official text field photo")
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    rectangle(0.0, 0.0, 307.0, 300.0, scheme.secondary_container, scale);
    rectangle(307.0, 0.0, 8.0, 300.0, scheme.outline_variant, scale);
    rectangle(315.0, 0.0, 361.0, 300.0, scheme.surface, scale);
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(339.0 * scale), Dp::ZERO)
                .size(Dp(337.0 * scale), Dp(103.0 * scale))
                .clip_to_bounds()
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(I18N.text("ui-food-photograph")),
                    ..Default::default()
                }),
        )
        .children(move || {
            // Clip to the photograph so all field and annotation pixels stay
            // native.
            image().raster(photo.get()).modifier(
                Modifier::new()
                    .offset(Dp(-339.0 * scale), Dp::ZERO)
                    .required_size(Dp(676.0 * scale), Dp(300.0 * scale)),
            );
        });
    editable_field(340.0, 128.0, 327.0, "ui-email".into(), "".into(), scale);
    editable_field(
        340.0,
        223.0,
        327.0,
        "ui-phone-number".into(),
        "".into(),
        scale,
    );
    rectangle(270.0, 155.0, 66.0, 5.0, srgb(132, 69, 143), scale);
    highlight(334.0, 122.0, 339.0, 66.0, 6.0, scale);
    callout(
        40.0,
        102.0,
        230.0,
        116.0,
        "ui-ui-text-email".into(),
        "ui-textbox".into(),
        scale,
    );
}

#[tessera]
fn editable_field(x: f64, y: f64, width: f64, label: String, initial: String, scale: f64) {
    text_field()
        .outlined()
        .label(I18N.text(&(label)))
        .initial_text(initial)
        .accessibility_label(I18N.text(label))
        .min_width(Dp::ZERO)
        .min_height(Dp(56.0 * scale))
        .font_size(Dp(16.0 * scale))
        .line_height(Dp(24.0 * scale))
        .padding(Dp(16.0 * scale))
        .border_width(Dp(scale))
        .focus_border_width(Dp(3.0 * scale))
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(width * scale), Dp(56.0 * scale)),
        );
}

#[tessera]
fn password_diagram(scale: f64) {
    let show = remember(|| false);
    let controller =
        remember(|| TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale))));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    rectangle(0.0, 0.0, 378.0, 300.0, scheme.surface, scale);
    rectangle(378.0, 0.0, 8.0, 300.0, scheme.outline_variant, scale);
    editable_field(
        26.5,
        41.5,
        327.0,
        "ui-label".into(),
        "ui-input".into(),
        scale,
    );
    dropdown_field(scale);
    let field = text_field()
        .outlined()
        .label(I18N.text("ui-password"))
        .accessibility_label(I18N.text("ui-password"))
        .controller(controller)
        .min_width(Dp::ZERO)
        .min_height(Dp(56.0 * scale))
        .font_size(Dp(16.0 * scale))
        .line_height(Dp(24.0 * scale))
        .padding(Dp(16.0 * scale))
        .border_width(Dp(scale))
        .focus_border_width(Dp(3.0 * scale))
        .modifier(
            Modifier::new()
                .offset(Dp(26.5 * scale), Dp(122.0 * scale))
                .size(Dp(327.0 * scale), Dp(56.0 * scale)),
        )
        .trailing_icon(move || {
            icon_button()
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(if show.get() {
                    filled::VISIBILITY_OFF_SVG
                } else {
                    filled::VISIBILITY_SVG
                })
                .accessibility_label(I18N.text(if show.get() {
                    "ui-hide-password"
                } else {
                    "ui-show-password"
                }))
                .on_click(move || {
                    show.set(!show.get());
                    controller.with(|controller| controller.request_focus());
                });
        });
    if !show.get() {
        field
            .obfuscation_char('•')
            .context_menu(TextFieldContextMenu::secure_default());
    }
    rectangle(350.0, 150.0, 63.0, 5.0, srgb(132, 69, 143), scale);
    highlight(310.0, 130.0, 40.0, 40.0, 6.0, scale);
    callout(
        413.0,
        92.0,
        237.0,
        116.0,
        if show.get() {
            "ui-hide-password-5934".into()
        } else {
            "ui-show-password-5935".into()
        },
        "ui-button".into(),
        scale,
    );
}

#[tessera]
fn dropdown_field(scale: f64) {
    let menu = remember(MenuController::new);
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale)));
        controller.set_text(&I18N.text("ui-input"));
        controller
    });
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(26.5 * scale), Dp(202.5 * scale))
                .size(Dp(327.0 * scale), Dp(56.0 * scale)),
        )
        .children(move || {
            menu_provider()
                .controller(menu)
                .anchor(MenuAnchor::new(
                    PxPosition::new(Dp::ZERO.into(), Dp::ZERO.into()),
                    PxSize::new(Dp(327.0 * scale).into(), Dp(56.0 * scale).into()),
                ))
                .modifier(Modifier::new().size(Dp(327.0 * scale), Dp(56.0 * scale)))
                .main_content(move || {
                    text_field()
                        .outlined()
                        .controller(controller)
                        .label(I18N.text("ui-label"))
                        .accessibility_label(I18N.text("ui-label-options"))
                        .min_width(Dp::ZERO)
                        .min_height(Dp(56.0 * scale))
                        .font_size(Dp(16.0 * scale))
                        .line_height(Dp(24.0 * scale))
                        .padding(Dp(16.0 * scale))
                        .border_width(Dp(scale))
                        .focus_border_width(Dp(3.0 * scale))
                        .modifier(Modifier::new().size(Dp(327.0 * scale), Dp(56.0 * scale)))
                        .trailing_icon(move || {
                            icon_button()
                                .standard()
                                .color_style(IconButtonColorStyle::Inherited)
                                .icon(filled::ARROW_DROP_DOWN_SVG)
                                .accessibility_label(I18N.text("ui-choose-label-option"))
                                .on_click(move || menu.with_mut(MenuController::toggle));
                        });
                })
                .menu_content(move |menu| {
                    menu_item()
                        .menu(menu)
                        .label(I18N.text("ui-personal"))
                        .on_click(move || {
                            controller.with_mut(|controller| {
                                controller.set_text(&I18N.text("ui-personal"))
                            });
                        });
                    menu_item()
                        .menu(menu)
                        .label(I18N.text("ui-work"))
                        .on_click(move || {
                            controller
                                .with_mut(|controller| controller.set_text(&I18N.text("ui-work")));
                        });
                });
        });
}

#[tessera]
fn reference_photo(source: usize, x: f64, y: f64, width: f64, height: f64, scale: f64) {
    let photo = remember(|| {
        let bytes: &[u8] = match source {
            2 => {
                include_bytes!("../../../assets/material_spec/text-field-accessibility-affixes.png")
            }
            3 => include_bytes!("../../../assets/material_spec/text-field-accessibility-error.png"),
            _ => include_bytes!(
                "../../../assets/material_spec/text-field-accessibility-required.png"
            ),
        };
        bytes
            .try_into_image_data()
            .expect("official text field photo")
    });
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(width * scale), Dp(height * scale))
                .clip_to_bounds()
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(I18N.text(if source == 6 {
                        "ui-flower-photograph"
                    } else {
                        "ui-food-photograph"
                    })),
                    ..Default::default()
                }),
        )
        .children(move || {
            // Only the original photograph lies inside this clip rectangle.
            image().raster(photo.get()).modifier(
                Modifier::new()
                    .offset(Dp(-x * scale), Dp(-y * scale))
                    .required_size(Dp(676.0 * scale), Dp(300.0 * scale)),
            );
        });
}

#[tessera]
fn affixes_diagram(scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    rectangle(0.0, 0.0, 676.0, 300.0, scheme.secondary_container, scale);
    rectangle(148.0, 0.0, 380.0, 300.0, scheme.outline_variant, scale);
    rectangle(156.0, 0.0, 364.0, 300.0, scheme.surface, scale);
    reference_photo(2, 180.0, 0.0, 316.0, 64.0, scale);
    labeled_field(0, 180.0, 88.0, 316.0, scale);
    labeled_field(1, 180.0, 172.0, 316.0, scale);
    editable_field(
        180.0,
        256.0,
        316.0,
        "ui-phone-number".into(),
        "".into(),
        scale,
    );
    rectangle(138.0, 114.0, 48.0, 5.0, srgb(132, 69, 143), scale);
    rectangle(491.0, 197.0, 48.0, 5.0, srgb(132, 69, 143), scale);
    highlight(186.0, 96.0, 40.0, 40.0, 6.0, scale);
    highlight(382.0, 180.0, 109.0, 40.0, 6.0, scale);
    callout(
        16.0,
        52.0,
        122.0,
        136.0,
        "ui-ui-text-euro".into(),
        "ui-no-role".into(),
        scale,
    );
    callout(
        539.0,
        102.0,
        122.0,
        156.0,
        "ui-ui-text-at-gmail-dot-com".into(),
        "ui-no-role".into(),
        scale,
    );
}

#[tessera]
fn error_diagram(scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(339.0 * scale), Dp(-30.0 * scale))
                .size(Dp(400.0 * scale), Dp(288.0 * scale))
                .background_with_shape(
                    scheme.outline_variant,
                    Shape::rounded_rectangle(Dp(26.0 * scale)),
                ),
        )
        .children(move || {
            spacer().modifier(
                Modifier::new()
                    .offset(Dp(8.0 * scale), Dp::ZERO)
                    .size(Dp(392.0 * scale), Dp(280.0 * scale))
                    .background_with_shape(
                        scheme.surface,
                        Shape::rounded_rectangle(Dp(20.0 * scale)),
                    ),
            );
        });
    reference_photo(3, 371.0, 0.0, 305.0, 85.0, scale);
    labeled_field(2, 371.0, 109.0, 327.0, scale);
    spacer().modifier(
        Modifier::new()
            .offset(Dp(501.0 * scale), Dp(236.0 * scale))
            .size(Dp(110.0 * scale), Dp(4.0 * scale))
            .background_with_shape(scheme.on_surface, Shape::CAPSULE),
    );
    rectangle(304.0, 148.0, 63.0, 4.0, srgb(132, 69, 143), scale);
    highlight(367.0, 105.0, 339.0, 86.0, 6.0, scale);
    callout(
        26.0,
        92.0,
        278.0,
        116.0,
        "ui-ui-text-not-a-valid-zip-code".into(),
        "ui-alert".into(),
        scale,
    );
}

#[tessera]
fn counter_diagram(scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    rectangle(0.0, 0.0, 676.0, 300.0, scheme.secondary_container, scale);
    rectangle(0.0, 0.0, 213.0, 300.0, scheme.surface, scale);
    rectangle(213.0, 0.0, 8.0, 300.0, scheme.outline_variant, scale);
    editable_field(
        -138.5,
        -5.5,
        327.0,
        "ui-previous-input".into(),
        "".into(),
        scale,
    );
    boxed()
        .modifier(Modifier::new().fill_max_size().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(I18N.text("ui-character-count-labeling-example")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            labeled_field(3, -140.0, 90.0, 327.0, scale);
        });
    editable_field(
        -138.5,
        190.5,
        327.0,
        "ui-next-input".into(),
        "".into(),
        scale,
    );
    rectangle(187.0, 154.0, 62.0, 5.0, srgb(132, 69, 143), scale);
    highlight(139.0, 133.0, 47.0, 48.0, 6.0, scale);
}

#[tessera]
fn supporting_diagram(scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    rectangle(0.0, 0.0, 369.0, 300.0, scheme.surface, scale);
    rectangle(369.0, 0.0, 8.0, 300.0, scheme.outline_variant, scale);
    editable_field(
        17.5,
        19.5,
        326.5,
        "ui-phone-number".into(),
        "".into(),
        scale,
    );
    labeled_field(4, 16.0, 119.5, 327.0, scale);
    editable_field(
        16.0,
        219.5,
        327.0,
        "ui-label".into(),
        "ui-input".into(),
        scale,
    );
    rectangle(348.0, 150.0, 68.0, 5.0, srgb(132, 69, 143), scale);
    highlight(12.0, 114.5, 336.0, 85.5, 6.0, scale);
    callout(
        417.0,
        82.0,
        233.0,
        134.0,
        "ui-ui-text-please-use-the-company-email-address".into(),
        "ui-no-role".into(),
        scale,
    );
}

#[tessera]
fn required_diagram(scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    rectangle(0.0, 0.0, 330.0, 300.0, scheme.secondary_container, scale);
    rectangle(330.0, 0.0, 8.0, 300.0, scheme.outline_variant, scale);
    rectangle(338.0, 0.0, 338.0, 300.0, scheme.surface, scale);
    reference_photo(6, 362.0, 0.0, 314.0, 83.0, scale);
    labeled_field(5, 362.0, 107.5, 327.0, scale);
    editable_field(
        362.0,
        203.5,
        327.0,
        "ui-phone-number".into(),
        "".into(),
        scale,
    );
    editable_field(
        362.0,
        283.5,
        327.0,
        "ui-additional-input".into(),
        "".into(),
        scale,
    );
    rectangle(284.0, 146.0, 74.0, 5.0, srgb(132, 69, 143), scale);
    highlight(358.0, 103.0, 339.0, 84.0, 6.0, scale);
    callout(
        54.0,
        92.0,
        230.0,
        116.0,
        "ui-ui-text-username".into(),
        "ui-textbox".into(),
        scale,
    );
}

#[tessera]
fn labeled_field(kind: usize, x: f64, y: f64, width: f64, scale: f64) {
    let initial = if kind == 2 {
        "01"
    } else if kind == 3 {
        "ui-input"
    } else {
        ""
    };
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale)));
        controller.set_text(&I18N.text(initial));
        controller
    });
    let count = remember(|| controller.with(|controller| controller.text().chars().count()));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let description = match kind {
        2 => "ui-not-a-valid-zip-code",
        4 => "ui-please-use-the-company-email-address",
        5 => "ui-required",
        _ => "",
    };
    let name = match kind {
        0 => "ui-amount",
        1 | 4 => "ui-email",
        2 => "ui-zip-code",
        3 => "ui-label",
        _ => "Username*",
    };
    let field = text_field()
        .outlined()
        .controller(controller)
        .accessibility_label(I18N.text(name))
        .accessibility_description(I18N.text(description))
        .min_width(Dp::ZERO)
        .min_height(Dp(56.0 * scale))
        .font_size(Dp(16.0 * scale))
        .line_height(Dp(24.0 * scale))
        .padding(Dp(16.0 * scale))
        .border_width(Dp(scale))
        .focus_border_width(Dp(3.0 * scale))
        .is_error(kind == 2)
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(width * scale), Dp(56.0 * scale)),
        )
        .on_change(move |next: String| {
            count.set(next.chars().count());
            next
        });
    if kind >= 2 {
        field.label(I18N.text(name));
    }
    if kind == 1 {
        field.placeholder(I18N.text("ui-email")).suffix(move || {
            text()
                .content(I18N.text("@gmail.com"))
                .style(type_style(16.0 * scale, 24.0 * scale))
                .accessibility_label(I18N.text("ui-at-gmail-dot-com-5953"));
        });
    }
    if kind == 0 {
        field.prefix(move || {
            text()
                .content(I18N.text("€"))
                .style(type_style(16.0 * scale, 24.0 * scale))
                .accessibility_label(I18N.text("ui-euro"));
        });
    }
    if kind == 2 || kind == 3 {
        field.presentation_state(InteractionVisualState::Focus);
    }
    if kind == 3 {
        text()
            .content(format!("{}/20", count.get()))
            .style(type_style(12.0 * scale, 16.0 * scale))
            .accessibility_label(I18N.format(
                "ui-character-count-value-of-20-characters-entered",
                [("v0", (count.get()).into())],
            ))
            .modifier(
                Modifier::new().offset(Dp((x + width - 37.0) * scale), Dp((y + 62.0) * scale)),
            );
        callout(
            249.0,
            96.0,
            316.0,
            136.0,
            I18N.format(
                "ui-ui-text-character-count-value-of-20-characters-entered",
                [("v0", (count.get()).into())],
            ),
            "ui-no-role".into(),
            scale,
        );
    } else if !description.is_empty() {
        text()
            .content(I18N.text(if kind == 5 { "*required" } else { description }))
            .style(type_style(12.0 * scale, 16.0 * scale))
            .color(if kind == 2 {
                scheme.error
            } else {
                scheme.on_surface_variant
            })
            .modifier(
                Modifier::new()
                    .offset(
                        Dp((x + if kind == 2 { 0.0 } else { 16.0 }) * scale),
                        Dp((y + 60.0) * scale),
                    )
                    .semantics(SemanticsArgs {
                        role: if kind == 2 { Some(Role::Alert) } else { None },
                        label: Some(I18N.text(if kind == 5 { "*required" } else { description })),
                        ..Default::default()
                    }),
            );
    }
}
