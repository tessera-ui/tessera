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
    text_field::{TextFieldBuilder, TextFieldContextMenu},
    text_input::TextInputController,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, PxPosition, PxSize, RenderSlot, accesskit::Role, remember, tessera,
    use_context,
};

use super::{gap, srgb, type_style};

#[tessera]
pub(super) fn labeling() {
    paragraph("The accessible name matches the visible text field label.");
    gap(Dp(24.0));
    labeling_figure(0);
    gap(Dp(24.0));
    paragraph(
        "Name interactive trailing icons by their action. The password action changes between Show password and Hide password.",
    );
    gap(Dp(24.0));
    labeling_figure(1);
    for (kind, description) in [
        (
            2,
            "Give affixes a meaningful accessible name, such as Euro or At gmail dot com.",
        ),
        (
            3,
            "Expose error messages with an alert role and describe how to correct the input.",
        ),
        (
            4,
            "Name the character counter and announce the number of characters entered and the limit.",
        ),
        (
            5,
            "Supporting text has its own accessible name and is associated with the field.",
        ),
        (
            6,
            "Include the asterisk in a required field's accessible name and explain it in supporting text.",
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
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn labeling_figure(kind: usize) {
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
                        .content("Label")
                        .style(type_style(14.0 * scale, 18.0 * scale));
                    text()
                        .content(label.clone())
                        .style(emphasis.clone())
                        .color(srgb(132, 69, 143));
                    gap(Dp(12.0 * scale));
                    text()
                        .content("Role")
                        .style(type_style(14.0 * scale, 18.0 * scale));
                    text()
                        .content(role.clone())
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
                    label: Some("Food photograph".into()),
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
    editable_field(340.0, 128.0, 327.0, "Email".into(), "".into(), scale);
    editable_field(340.0, 223.0, 327.0, "Phone number".into(), "".into(), scale);
    rectangle(270.0, 155.0, 66.0, 5.0, srgb(132, 69, 143), scale);
    highlight(334.0, 122.0, 339.0, 66.0, 6.0, scale);
    callout(
        40.0,
        102.0,
        230.0,
        116.0,
        "UI text (“Email”)".into(),
        "Textbox".into(),
        scale,
    );
}

#[tessera]
fn editable_field(x: f64, y: f64, width: f64, label: String, initial: String, scale: f64) {
    TextFieldBuilder::outlined()
        .label(label.clone())
        .initial_text(initial)
        .accessibility_label(label)
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
    editable_field(26.5, 41.5, 327.0, "Label".into(), "Input".into(), scale);
    dropdown_field(scale);
    let mut field = TextFieldBuilder::outlined()
        .label("Password")
        .accessibility_label("Password")
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
        .trailing_icon_shared(RenderSlot::new(move || {
            icon_button()
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(if show.get() {
                    filled::VISIBILITY_OFF_SVG
                } else {
                    filled::VISIBILITY_SVG
                })
                .accessibility_label(if show.get() {
                    "Hide password"
                } else {
                    "Show password"
                })
                .on_click(move || {
                    show.set(!show.get());
                    controller.with(|controller| controller.request_focus());
                });
        }));
    if !show.get() {
        field = field
            .obfuscation_char('•')
            .context_menu(TextFieldContextMenu::secure_default());
    }
    drop(field);
    rectangle(350.0, 150.0, 63.0, 5.0, srgb(132, 69, 143), scale);
    highlight(310.0, 130.0, 40.0, 40.0, 6.0, scale);
    callout(
        413.0,
        92.0,
        237.0,
        116.0,
        if show.get() {
            "Hide Password".into()
        } else {
            "Show Password".into()
        },
        "Button".into(),
        scale,
    );
}

#[tessera]
fn dropdown_field(scale: f64) {
    let menu = remember(MenuController::new);
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale)));
        controller.set_text("Input");
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
                    TextFieldBuilder::outlined()
                        .controller(controller)
                        .label("Label")
                        .accessibility_label("Label options")
                        .min_width(Dp::ZERO)
                        .min_height(Dp(56.0 * scale))
                        .font_size(Dp(16.0 * scale))
                        .line_height(Dp(24.0 * scale))
                        .padding(Dp(16.0 * scale))
                        .border_width(Dp(scale))
                        .focus_border_width(Dp(3.0 * scale))
                        .modifier(Modifier::new().size(Dp(327.0 * scale), Dp(56.0 * scale)))
                        .trailing_icon_shared(RenderSlot::new(move || {
                            icon_button()
                                .standard()
                                .color_style(IconButtonColorStyle::Inherited)
                                .icon(filled::ARROW_DROP_DOWN_SVG)
                                .accessibility_label("Choose label option")
                                .on_click(move || menu.with_mut(MenuController::toggle));
                        }));
                })
                .menu_content(move || {
                    menu_item().label("Personal").on_click(move || {
                        controller.with_mut(|controller| controller.set_text("Personal"));
                    });
                    menu_item().label("Work").on_click(move || {
                        controller.with_mut(|controller| controller.set_text("Work"));
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
                    label: Some(
                        if source == 6 {
                            "Flower photograph"
                        } else {
                            "Food photograph"
                        }
                        .into(),
                    ),
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
    editable_field(180.0, 256.0, 316.0, "Phone number".into(), "".into(), scale);
    rectangle(138.0, 114.0, 48.0, 5.0, srgb(132, 69, 143), scale);
    rectangle(491.0, 197.0, 48.0, 5.0, srgb(132, 69, 143), scale);
    highlight(186.0, 96.0, 40.0, 40.0, 6.0, scale);
    highlight(382.0, 180.0, 109.0, 40.0, 6.0, scale);
    callout(
        16.0,
        52.0,
        122.0,
        136.0,
        "UI text\n(“Euro”)".into(),
        "[No role]".into(),
        scale,
    );
    callout(
        539.0,
        102.0,
        122.0,
        156.0,
        "UI text (“At\ngmail dot\ncom”)".into(),
        "[No role]".into(),
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
        "UI text (“Not a valid ZIP code”)".into(),
        "Alert".into(),
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
        "Previous input".into(),
        "".into(),
        scale,
    );
    labeled_field(3, -140.0, 90.0, 327.0, scale);
    editable_field(-138.5, 190.5, 327.0, "Next input".into(), "".into(), scale);
    rectangle(187.0, 154.0, 62.0, 5.0, srgb(132, 69, 143), scale);
    highlight(139.0, 133.0, 47.0, 48.0, 6.0, scale);
}

#[tessera]
fn supporting_diagram(scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    rectangle(0.0, 0.0, 369.0, 300.0, scheme.surface, scale);
    rectangle(369.0, 0.0, 8.0, 300.0, scheme.outline_variant, scale);
    editable_field(17.5, 19.5, 326.5, "Phone number".into(), "".into(), scale);
    labeled_field(4, 16.0, 119.5, 327.0, scale);
    editable_field(16.0, 219.5, 327.0, "Label".into(), "Input".into(), scale);
    rectangle(348.0, 150.0, 68.0, 5.0, srgb(132, 69, 143), scale);
    highlight(12.0, 114.5, 336.0, 85.5, 6.0, scale);
    callout(
        417.0,
        82.0,
        233.0,
        134.0,
        "UI text (“Please use the\ncompany email address”)".into(),
        "[No role]".into(),
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
    editable_field(362.0, 203.5, 327.0, "Phone number".into(), "".into(), scale);
    editable_field(
        362.0,
        283.5,
        327.0,
        "Additional input".into(),
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
        "UI text (“Username*”)".into(),
        "Textbox".into(),
        scale,
    );
}

#[tessera]
fn labeled_field(kind: usize, x: f64, y: f64, width: f64, scale: f64) {
    let initial = if kind == 2 {
        "01"
    } else if kind == 3 {
        "Input"
    } else {
        ""
    };
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale)));
        controller.set_text(initial);
        controller
    });
    let count = remember(|| initial.chars().count());
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let description = match kind {
        2 => "Not a valid ZIP code",
        4 => "Please use the company email address",
        5 => "Required",
        _ => "",
    };
    let name = match kind {
        0 => "Amount",
        1 | 4 => "Email",
        2 => "ZIP code",
        3 => "Character count example",
        _ => "Username*",
    };
    let mut field = TextFieldBuilder::outlined()
        .controller(controller)
        .accessibility_label(name)
        .accessibility_description(description)
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
        field = field.label(if kind == 3 { "Label" } else { name });
    }
    if kind == 1 {
        field = field
            .placeholder("Email")
            .suffix_shared(RenderSlot::new(move || {
                text()
                    .content("@gmail.com")
                    .style(type_style(16.0 * scale, 24.0 * scale))
                    .accessibility_label("At gmail dot com");
            }));
    }
    if kind == 0 {
        field = field.prefix_shared(RenderSlot::new(move || {
            text()
                .content("€")
                .style(type_style(16.0 * scale, 24.0 * scale))
                .accessibility_label("Euro");
        }));
    }
    if kind == 2 || kind == 3 {
        field = field.presentation_state(InteractionVisualState::Focus);
    }
    drop(field);
    if kind == 3 {
        text()
            .content(format!("{}/20", count.get()))
            .style(type_style(12.0 * scale, 16.0 * scale))
            .accessibility_label(format!(
                "Character count, {} of 20 characters entered",
                count.get()
            ))
            .modifier(
                Modifier::new().offset(Dp((x + width - 37.0) * scale), Dp((y + 62.0) * scale)),
            );
        callout(
            249.0,
            96.0,
            316.0,
            136.0,
            format!(
                "UI text (“Character count, {} of 20\ncharacters entered”)",
                count.get()
            ),
            "[No role]".into(),
            scale,
        );
    } else if !description.is_empty() {
        text()
            .content(if kind == 5 { "*required" } else { description })
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
                        label: Some(if kind == 5 { "*required" } else { description }.into()),
                        ..Default::default()
                    }),
            );
    }
}
