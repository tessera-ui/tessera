//! Filled and outlined text field specification examples.
//!
//! ## Usage
//!
//! Edit short inputs and compare labels, icons, validation and supporting text.

use tessera_components::{
    alignment::{Alignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    icon_button::{IconButtonColorStyle, icon_button},
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    text_field::{TextFieldBuilder, TextFieldContextMenu, TextFieldLineLimit},
    text_input::TextInputController,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT, BACKGROUND, INK,
    annotations::{self, Geometry, Rect, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn text_fields_spec() {
    article(
        "Text fields",
        "Enter and edit text in filled or outlined fields.",
        vec![
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Filled text field",
                content: RenderSlot::new(|| {
                    detail(false);
                }),
            },
            SpecSection {
                title: "Outlined text field",
                content: RenderSlot::new(|| {
                    detail(true);
                }),
            },
        ],
    ).pages(vec![
        ArticlePage{title:"Overview",intro:Some(RenderSlot::new(field_variants)),sections:vec![SpecSection{title:"Differences from M2",content:RenderSlot::new(|| {prose("Text fields use updated color mappings and dynamic color, with distinct filled and outlined treatments.");gap(Dp(24.0));field_figure(false,false,false,false);gap(Dp(24.0));field_figure(true,false,false,false);})}]},
        ArticlePage{title:"Guidelines",intro:Some(RenderSlot::new(field_variants)),sections:vec![
            SpecSection{title:"Usage",content:RenderSlot::new(|| {prose("Use text fields for contact, payment and other text entry. Both variants provide the same editing functionality.");gap(Dp(24.0));contact_form(false);gap(Dp(24.0));subheading("Outlined text fields".into());prose("Outlined fields have less visual emphasis and can simplify forms containing many inputs.");gap(Dp(24.0));contact_form(true);})},
            SpecSection{title:"Choosing text fields",content:RenderSlot::new(|| {subheading("Choosing text fields".into());prose("Choose the variant that suits the app's style and remains distinct from surrounding content.");gap(Dp(24.0));field_variants();gap(Dp(24.0));subheading("Using both text field variants on the same screen".into());prose("Keep one variant within each region. If both are needed, separate their regions rather than intermixing fields within the same form.");gap(Dp(24.0));contact_form(false);})},
            SpecSection{title:"Anatomy",content:RenderSlot::new(field_guideline_anatomy)},
            SpecSection{title:"Adaptive design",content:RenderSlot::new(|| {prose("Constrain fields on large screens instead of stretching inputs across the full display. Compact forms can use available width.");gap(Dp(24.0));contact_form(true);gap(Dp(24.0));subheading("Density".into());prose("Dense layouts suit information-heavy workflows only when people choose them.");gap(Dp(24.0));subheading("Avoid applying density by default".into());prose("Keep default interaction targets at least 48 × 48. Controls for reverting a density choice must also remain easy to reach.");gap(Dp(24.0));field_figure(true,false,false,true);})},
        ]},
        ArticlePage{title:"Accessibility",intro:None,sections:vec![
            SpecSection{title:"Use cases",content:RenderSlot::new(|| {prose("Let people navigate to a field, enter information, understand supporting and error text, and reach interactive trailing icons.");gap(Dp(24.0));panel(RenderSlot::new(|| {form_field("Password".into(),true,9);}));})},
            SpecSection{title:"Interaction & style",content:RenderSlot::new(|| {prose("Both variants have the same functionality. Clear container boundaries and changes in stroke show interaction. Outlined containers should contrast against their background by at least 3:1.");gap(Dp(24.0));field_figure(false,false,false,true);gap(Dp(24.0));field_figure(true,false,false,true);})},
            SpecSection{title:"Keyboard navigation",content:RenderSlot::new(|| {spec_table(vec!["Key".into(),"Action".into()],vec![vec!["Tab".into(),"Focus an enabled field or interactive trailing icon".into()]]);gap(Dp(24.0));contact_form(true);})},
            SpecSection{title:"Labeling elements",content:RenderSlot::new(field_accessibility_labels)},
        ]},
    ]);
}

#[tessera]
fn sample(
    #[prop(into)] name: String,
    outlined: bool,
    controls: Option<bool>,
    populated: Option<bool>,
    enabled: Option<bool>,
    error: Option<bool>,
    icons: Option<bool>,
    preview_width: Option<Dp>,
) {
    let initial = if populated.unwrap_or(true) {
        "Input"
    } else {
        ""
    };
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        controller.set_text(initial);
        controller
    });
    let value = remember(|| initial.to_owned());
    let active = remember(|| enabled.unwrap_or(true));
    let read_only = remember(|| false);
    let invalid = remember(|| error.unwrap_or(false));
    let icons = remember(|| icons.unwrap_or(false));
    let affixes = remember(|| false);
    let multiline = remember(|| false);
    let secure = remember(|| false);
    let submitted = remember(|| 0_usize);
    let label = remember(|| name.clone());
    let modifier = if let Some(width) = preview_width {
        Modifier::new().width(width)
    } else {
        Modifier::new().fill_max_width()
    };
    column()
        .modifier(modifier.semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(format!("{name} field preview")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if controls.unwrap_or(false) {
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        button()
                            .tonal()
                            .selected(active.get())
                            .accessibility_label(format!("{} enabled", label.get()))
                            .on_click(move || active.set(!active.get()))
                            .child(|| {
                                text().content("Enabled");
                            });
                        button()
                            .tonal()
                            .selected(read_only.get())
                            .accessibility_label(format!("{} read only", label.get()))
                            .on_click(move || read_only.set(!read_only.get()))
                            .child(|| {
                                text().content("Read only");
                            });
                        button()
                            .tonal()
                            .selected(invalid.get())
                            .accessibility_label(format!("{} error", label.get()))
                            .on_click(move || invalid.set(!invalid.get()))
                            .child(|| {
                                text().content("Error");
                            });
                        button()
                            .tonal()
                            .selected(icons.get())
                            .accessibility_label(format!("{} icons", label.get()))
                            .on_click(move || icons.set(!icons.get()))
                            .child(|| {
                                text().content("Icons");
                            });
                        button()
                            .tonal()
                            .selected(affixes.get())
                            .accessibility_label(format!("{} affixes", label.get()))
                            .on_click(move || affixes.set(!affixes.get()))
                            .child(|| {
                                text().content("Prefix and suffix");
                            });
                        button()
                            .tonal()
                            .selected(multiline.get())
                            .accessibility_label(format!("{} multiline", label.get()))
                            .on_click(move || multiline.set(!multiline.get()))
                            .child(|| {
                                text().content("Multiline");
                            });
                        button()
                            .tonal()
                            .selected(secure.get())
                            .accessibility_label(format!("{} secure", label.get()))
                            .on_click(move || secure.set(!secure.get()))
                            .child(|| {
                                text().content("Secure");
                            });
                        button()
                            .text()
                            .accessibility_label(format!("Clear {}", label.get()))
                            .on_click(move || {
                                controller.with_mut(|controller| controller.set_text(""));
                                value.set(String::new());
                            })
                            .child(|| {
                                text().content("Clear");
                            });
                    });
                gap(Dp(24.0));
            }
            boxed()
                .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!("{} container", label.get())),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .children(move || {
                    let mut field = if outlined {
                        TextFieldBuilder::outlined()
                    } else {
                        TextFieldBuilder::filled()
                    };
                    field = field
                        .modifier(Modifier::new().fill_max_width())
                        .min_width(Dp::ZERO)
                        .min_height(Dp(if multiline.get() { 96.0 } else { 56.0 }))
                        .controller(controller)
                        .label("Label")
                        .placeholder("Placeholder")
                        .accessibility_label(format!("{} input", label.get()))
                        .accessibility_description(if invalid.get() {
                            "Please check your input"
                        } else {
                            "Supporting text"
                        })
                        .enabled(active.get())
                        .read_only(read_only.get())
                        .is_error(invalid.get())
                        .line_limit(if multiline.get() {
                            TextFieldLineLimit::MultiLine
                        } else {
                            TextFieldLineLimit::SingleLine
                        })
                        .on_change(move |next: String| {
                            value.set(next.clone());
                            next
                        })
                        .on_submit(move || submitted.set(submitted.get() + 1));
                    if icons.get() {
                        field = field
                            .leading_icon(move || {
                                boxed()
                                    .modifier(Modifier::new().semantics(SemanticsArgs {
                                        role: Some(Role::Group),
                                        label: Some(format!("{} leading icon", label.get())),
                                        merge_descendants: true,
                                        ..Default::default()
                                    }))
                                    .children(|| {
                                        icon().painter(filled::SEARCH_SVG).size(Dp(24.0));
                                    });
                            })
                            .trailing_icon(move || {
                                icon_button()
                                    .standard()
                                    .color_style(IconButtonColorStyle::Inherited)
                                    .icon(filled::CANCEL_SVG)
                                    .accessibility_label(format!("Clear {} icon", label.get()))
                                    .enabled(active.get() && !read_only.get())
                                    .on_click(move || {
                                        controller.with_mut(|controller| controller.set_text(""));
                                        value.set(String::new());
                                    });
                            });
                    }
                    if affixes.get() {
                        field = field
                            .prefix(|| {
                                text().content("$");
                            })
                            .suffix(|| {
                                text().content(".00");
                            });
                    }
                    if secure.get() {
                        field = field
                            .obfuscation_char('•')
                            .context_menu(TextFieldContextMenu::secure_default());
                    }
                    drop(field);
                });
            gap(Dp(4.0));
            row()
                .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                    Dp(16.0),
                    Dp::ZERO,
                    Dp(16.0),
                    Dp::ZERO,
                )))
                .children(move || {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    text()
                        .content(if invalid.get() {
                            "Please check your input"
                        } else {
                            "Supporting text"
                        })
                        .style(type_style(12.0, 16.0))
                        .color(if invalid.get() {
                            scheme.error
                        } else {
                            scheme.on_surface_variant
                        });
                    if controls.unwrap_or(false) {
                        spacer().modifier(Modifier::new().weight(1.0).width(Dp(16.0)));
                        text()
                            .content(format!(
                                "{} characters",
                                value.with(|value| value.chars().count())
                            ))
                            .style(type_style(12.0, 16.0));
                    }
                });
            if controls.unwrap_or(false) {
                gap(Dp(16.0));
                text()
                    .content(format!(
                        "{} value: {}",
                        label.get(),
                        if secure.get() {
                            "•".repeat(value.with(|value| value.chars().count()))
                        } else {
                            value.get()
                        }
                    ))
                    .style(type_style(14.0, 20.0));
                text()
                    .content(format!("{} submissions: {}", label.get(), submitted.get()))
                    .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn detail(outlined: bool) {
    column().modifier(Modifier::new().fill_max_width()).children(move || {

    let title = if outlined {
        "Outlined text field"
    } else {
        "Filled text field"
    };
    panel(RenderSlot::new(move || {
        flow_row()
            .modifier(Modifier::new().fill_max_width().padding_symmetric(Dp::ZERO, Dp(40.0)))
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(80.0))
            .line_spacing(Dp(40.0))
            .children(move || {
                for populated in [false, true] {
                    sample(
                        format!("{} {} anatomy", if outlined { "Outlined" } else { "Filled" }, if populated { "populated" } else { "empty" }),
                        outlined,
                    )
                    .populated(populated)
                    .icons(true)
                    .preview_width(Dp(180.0));
                }
            });
    }));
    gap(Dp(24.0));
    anatomy_legend(if outlined {
        vec!["Enabled container outline", "Leading icon (optional)", "Label text in empty field", "Label text in populated field", "Trailing icon (optional)", "Focused container outline", "Caret", "Input text", "Supporting text (optional)"]
    } else {
        vec!["Container", "Leading icon (optional)", "Label text in empty field", "Label text in populated field", "Trailing icon (optional)", "Focused active indicator", "Caret", "Input text", "Supporting text (optional)", "Enabled active indicator"]
    });
    gap(Dp(112.0));
    subheading(format!("{title} color"));
    gap(Dp(24.0));
    for dark in [false, true] {
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                panel(RenderSlot::new(move || {
                    sample(
                        format!(
                            "{} {}",
                            if dark { "Dark" } else { "Light" },
                            if outlined { "outlined" } else { "filled" }
                        ),
                        outlined,
                    );
                }));
            });
        gap(Dp(24.0));
    }
    gap(Dp(88.0));
    subheading(format!("{title} states"));
    gap(Dp(24.0));
    for populated in [false, true] {
        panel(RenderSlot::new(move || {
            sample(
                format!(
                    "{} {} enabled",
                    title,
                    if populated { "populated" } else { "empty" }
                ),
                outlined,
            )
            .populated(populated);
            gap(Dp(24.0));
            sample(
                format!(
                    "{} {} disabled",
                    title,
                    if populated { "populated" } else { "empty" }
                ),
                outlined,
            )
            .populated(populated)
            .enabled(false);
        }));
        gap(Dp(24.0));
    }
    text().content("Focus or hover the enabled fields to inspect their active outline, label and caret. Disabled fields retain their text and do not accept editing.").style(type_style(16.0,24.0));
    gap(Dp(112.0));
    subheading(format!("{title} error states"));
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        sample(
            format!("{} invalid", if outlined { "Outlined" } else { "Filled" }),
            outlined,
        )
        .error(true);
    }));
    gap(Dp(112.0));
    subheading(format!("{title} measurements"));
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Container height / touch target".into(), "56dp".into()],
            vec!["Unpopulated label alignment".into(), "Vertically centered".into()],
            vec!["Horizontal padding without icons".into(), "16dp".into()],
            vec!["Horizontal padding with icons".into(), "12dp".into()],
            vec!["Icon alignment".into(), "Vertically centered".into()],
            vec!["Icon-to-text spacing".into(), "16dp".into()],
            vec!["Supporting text top padding".into(), "4dp".into()],
            vec!["Supporting text-to-counter spacing".into(), "16dp".into()],
            if outlined {
                vec!["Populated label horizontal padding".into(), "4dp".into()]
            } else {
                vec!["Populated content top/bottom padding".into(), "8dp".into()]
            },
        ],
    );
    gap(Dp(112.0));
    subheading(format!("{title} configurations"));
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        sample(
            format!(
                "Configured {}",
                if outlined { "Outlined" } else { "Filled" }
            ),
            outlined,
        )
        .controls(true);
    }));

    });
}

#[tessera]
fn anatomy_legend(items: Vec<&'static str>) {
    let items = remember(|| items);
    column().children(move || {
        for (index, item) in items.get().into_iter().enumerate() {
            row()
                .modifier(Modifier::new().padding_symmetric(Dp::ZERO, Dp(6.0)))
                .children(move || {
                    surface()
                        .style(INK.into())
                        .shape(Shape::rounded_rectangle(Dp(12.0)))
                        .modifier(Modifier::new().width(Dp(24.0)).height(Dp(32.0)))
                        .child(move || {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(Modifier::new().fill_max_size())
                                .children(move || {
                                    text()
                                        .content((index + 1).to_string())
                                        .style(type_style(14.0, 20.0))
                                        .color(BACKGROUND);
                                });
                        });
                    spacer().modifier(Modifier::new().width(Dp(12.0)));
                    text()
                        .content(item)
                        .style(type_style(16.0, 24.0))
                        .modifier(Modifier::new().weight(1.0));
                });
        }
    });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec![
                "Filled container".into(),
                "Surface container highest".into(),
            ],
            vec!["Input text".into(), "On surface".into()],
            vec![
                "Resting label and icons".into(),
                "On surface variant".into(),
            ],
            vec!["Focused label, outline and caret".into(), "Primary".into()],
            vec!["Error label and indicator".into(), "Error".into()],
            vec!["Outlined resting border".into(), "Outline".into()],
        ],
    );
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn field_figure(outlined: bool, error: bool, multiline: bool, measurements: bool) {
    let geometry = remember(Geometry::new);
    let controller = remember(|| {
        let mut value = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        value.set_text("Input");
        value
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(320.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().width(Dp(280.0)))
                .children(move || {
                    let field = if outlined {
                        TextFieldBuilder::outlined()
                    } else {
                        TextFieldBuilder::filled()
                    };
                    field
                        .modifier(watch(geometry, "field").fill_max_width())
                        .min_width(Dp::ZERO)
                        .min_height(Dp(if multiline { 96.0 } else { 56.0 }))
                        .controller(controller)
                        .label("Label")
                        .accessibility_label("Label")
                        .accessibility_description(if error {
                            "Error message"
                        } else {
                            "Supporting text"
                        })
                        .is_error(error)
                        .line_limit(if multiline {
                            TextFieldLineLimit::MultiLine
                        } else {
                            TextFieldLineLimit::SingleLine
                        })
                        .leading_icon(|| {
                            icon().painter(filled::SEARCH_SVG).size(Dp(24.0));
                        })
                        .trailing_icon(move || {
                            if error {
                                boxed()
                                    .modifier(Modifier::new().semantics(SemanticsArgs {
                                        role: Some(Role::Image),
                                        label: Some("Error".into()),
                                        ..Default::default()
                                    }))
                                    .children(|| {
                                        icon().painter(filled::ERROR_SVG).size(Dp(24.0));
                                    });
                            } else {
                                icon_button()
                                    .standard()
                                    .icon(filled::CANCEL_SVG)
                                    .accessibility_label("Clear illustrated text field")
                                    .on_click(move || {
                                        controller.with_mut(|value| value.set_text(""))
                                    });
                            }
                        });
                    gap(Dp(4.0));
                    text()
                        .content(if error {
                            "Error message"
                        } else {
                            "Supporting text"
                        })
                        .color(if error {
                            scheme.error
                        } else {
                            scheme.on_surface_variant
                        })
                        .style(type_style(12.0, 16.0))
                        .modifier(watch(geometry, "support").padding_symmetric(Dp(16.0), Dp::ZERO));
                });
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    field_annotations(geometry, measurements);
                });
        });
}

#[tessera]
fn field_annotations(geometry: State<Geometry>, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(field), Some(support)) = (
        values.get("figure"),
        relative(&values, "field"),
        relative(&values, "support"),
    ) else {
        return;
    };
    let mut svg = String::new();
    annotations::outline(&mut svg, field);
    if measurements {
        annotations::dimension(
            format!("{:.0}", field.h),
            Dp(field.x - 40.0),
            Dp(field.y + field.h / 2.0 - 8.0),
        );
        annotations::dimension("16".into(), Dp(field.x + 2.0), Dp(field.y - 32.0));
        annotations::dimension(
            "4".into(),
            Dp(field.x + field.w + 20.0),
            Dp(field.y + field.h),
        );
        svg.push_str(&format!(
            "<rect x='{}' y='{}' width='16' height='{}' fill='#ee7da4' fill-opacity='.35'/>",
            field.x, field.y, field.h
        ));
    } else {
        let icon = Rect {
            x: field.x + 12.0,
            y: field.y + (field.h - 24.0) / 2.0,
            w: 24.0,
            h: 24.0,
        };
        annotations::outline(&mut svg, icon);
        for (number, x, y) in [
            (1, field.x - 48.0, field.y - 16.0),
            (2, icon.x - 12.0, field.y - 56.0),
            (3, field.x + 56.0, field.y - 56.0),
            (4, field.x + field.w - 40.0, field.y - 56.0),
            (5, support.x + support.w / 2.0 - 12.0, support.y + 40.0),
        ] {
            annotations::marker(number, Dp(x), Dp(y));
        }
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn field_variants() {
    prose(
        "Text fields let people enter and edit text. Choose filled or outlined fields consistently within a form.",
    );
    gap(Dp(24.0));
    field_figure(false, false, false, false);
    gap(Dp(24.0));
    field_figure(true, false, false, false);
}

#[tessera]
fn contact_form(outlined: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .padding_all(Dp(32.0)),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().width(Dp(320.0)))
                .children(move || {
                    text()
                        .content("Contact information")
                        .style(type_style(24.0, 32.0));
                    gap(Dp(32.0));
                    form_field("Name".into(), outlined, 0);
                    gap(Dp(24.0));
                    form_field("Email".into(), outlined, 5);
                    gap(Dp(24.0));
                    form_field("Phone".into(), outlined, 0);
                });
        });
}

#[tessera]
fn form_field(label: String, outlined: bool, mode: u8) {
    let controller = remember(|| {
        let mut value = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        value.set_text(if mode == 8 {
            "Read-only value"
        } else if mode == 7 {
            "Invalid input"
        } else {
            ""
        });
        value
    });
    let count = remember(|| 0_usize);
    let show = remember(|| false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let label = label.clone();
            if mode == 1 {
                text().content(label.clone()).style(type_style(16.0, 24.0));
                gap(Dp(8.0));
            }
            let field = if outlined {
                TextFieldBuilder::outlined()
            } else {
                TextFieldBuilder::filled()
            };
            let mut field = field
                .modifier(Modifier::new().fill_max_width())
                .min_width(Dp::ZERO)
                .controller(controller)
                .accessibility_label(if mode == 2 {
                    format!("{label}*")
                } else {
                    label.clone()
                })
                .accessibility_description(match mode {
                    2 => "Required",
                    7 => "Enter a valid value",
                    8 => "Read only",
                    _ => "Supporting text",
                })
                .read_only(mode == 8)
                .is_error(mode == 7)
                .on_change(move |value: String| {
                    count.set(value.chars().count());
                    value
                });
            if mode != 1 {
                field = field.label(if mode == 2 {
                    format!("{label}*")
                } else {
                    label
                });
            }
            if mode == 3 {
                field = field
                    .min_height(Dp(112.0))
                    .line_limit(TextFieldLineLimit::MultiLine);
            }
            if mode == 4 {
                field = field.prefix(|| {
                    text()
                        .content("$")
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(Role::Label),
                            label: Some("US dollars".into()),
                            ..Default::default()
                        }));
                });
            }
            if mode == 5 {
                field = field.suffix(|| {
                    text()
                        .content("@gmail.com")
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(Role::Label),
                            label: Some("at gmail dot com".into()),
                            ..Default::default()
                        }));
                });
            }
            if mode == 7 {
                field = field.trailing_icon(|| {
                    boxed()
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(Role::Image),
                            label: Some("Error".into()),
                            ..Default::default()
                        }))
                        .children(|| {
                            icon().painter(filled::ERROR_SVG).size(Dp(24.0));
                        });
                });
            }
            if mode == 9 {
                if !show.get() {
                    field = field
                        .obfuscation_char('•')
                        .context_menu(TextFieldContextMenu::secure_default());
                }
                field = field.trailing_icon(move || {
                    icon_button()
                        .standard()
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
                        .on_click(move || show.set(!show.get()));
                });
            }
            drop(field);
            gap(Dp(4.0));
            row()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .padding_symmetric(Dp(16.0), Dp::ZERO),
                )
                .children(move || {
                    let support = match mode {
                        2 => "* Required",
                        7 => "Enter a valid value",
                        8 => "Read only",
                        _ => "Supporting text",
                    };
                    text()
                        .content(support)
                        .style(type_style(12.0, 16.0))
                        .color(if mode == 7 {
                            scheme.error
                        } else {
                            scheme.on_surface_variant
                        })
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(if mode == 7 { Role::Alert } else { Role::Label }),
                            label: Some(support.into()),
                            ..Default::default()
                        }));
                    if mode == 6 {
                        spacer().modifier(Modifier::new().weight(1.0));
                        text()
                            .content(format!("{} / 100", count.get()))
                            .style(type_style(12.0, 16.0))
                            .modifier(Modifier::new().semantics(SemanticsArgs {
                                role: Some(Role::Label),
                                label: Some(format!("Character count: {} of 100", count.get())),
                                ..Default::default()
                            }));
                    }
                });
        });
}

#[tessera]
fn field_guideline_anatomy() {
    subheading("Filled text field".into());
    field_anatomy_pair(false);
    gap(Dp(24.0));
    subheading("Outlined text field".into());
    field_anatomy_pair(true);
    gap(Dp(24.0));
    subheading("Containers".into());
    prose(
        "Filled containers have rounded top corners and a bottom indicator. Outlined containers have a rounded outline around the field.",
    );
    gap(Dp(24.0));
    field_figure(true, false, false, true);
    gap(Dp(24.0));
    subheading("Label text".into());
    prose(
        "Keep labels short, visible and aligned with input. Do not truncate labels or wrap them across lines.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        form_field("Label".into(), false, 0);
    }));
    gap(Dp(24.0));
    subheading("Adjacent label".into());
    prose(
        "An adjacent label aligns to the leading edge and must name the field for assistive technology.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        form_field("Full name".into(), true, 1);
    }));
    gap(Dp(24.0));
    subheading("Required text indicator".into());
    prose(
        "Append an asterisk to every required label and explain it in supporting text or at the start of the form.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        form_field("Full name".into(), false, 2);
    }));
    gap(Dp(24.0));
    subheading("Input text".into());
    prose(
        "Use a single line for short responses, multiline input for longer answers and a fixed-height text area when substantial text is expected.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        form_field("Description".into(), true, 3);
    }));
    gap(Dp(24.0));
    subheading("Prefix text".into());
    panel(RenderSlot::new(|| {
        form_field("Amount".into(), false, 4);
    }));
    gap(Dp(24.0));
    subheading("Suffix text".into());
    panel(RenderSlot::new(|| {
        form_field("Email".into(), true, 5);
    }));
    gap(Dp(24.0));
    subheading("Supporting text & character counter".into());
    prose("Describe the requested input and show the used/maximum count when a limit exists.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        form_field("Description".into(), false, 6);
    }));
    gap(Dp(24.0));
    subheading("Error text".into());
    prose(
        "Replace supporting text with an actionable error message to avoid shifting surrounding content.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        form_field("Email".into(), true, 7);
    }));
    gap(Dp(24.0));
    subheading("Error icon".into());
    prose("An error icon provides an additional cue alongside color and error text.");
    gap(Dp(24.0));
    field_figure(true, true, false, false);
    gap(Dp(24.0));
    subheading("Icons & images".into());
    prose(
        "Use optional 24dp images or icons to clarify input or expose an actual action, such as clearing entered text.",
    );
    gap(Dp(24.0));
    field_figure(false, false, false, false);
    gap(Dp(24.0));
    subheading("Read-only fields".into());
    prose("Keep read-only fields visually consistent and label their state clearly.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        form_field("Read-only label".into(), false, 8);
        gap(Dp(24.0));
        form_field("Read-only label".into(), true, 8);
    }));
}

#[tessera]
fn field_accessibility_labels() {
    prose(
        "Match the accessible name to the visible label. Name interactive icons by their function and noninteractive error icons Error. Expose supporting text, errors and character counts separately.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        form_field("Password".into(), true, 9);
        gap(Dp(24.0));
        form_field("Amount".into(), true, 4);
        gap(Dp(24.0));
        form_field("Email".into(), true, 5);
        gap(Dp(24.0));
        form_field("Name".into(), false, 2);
        gap(Dp(24.0));
        form_field("Description".into(), false, 6);
        gap(Dp(24.0));
        form_field("Email".into(), true, 7);
    }));
}

#[tessera]
fn field_anatomy_pair(outlined: bool) {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let compact = geometry.with(|value| {
        value
            .get("figure")
            .is_some_and(|bounds| Dp::from_pixels_f32(bounds.width.to_f32()).0 < 536.0)
    });
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(if compact { 528.0 } else { 360.0 }))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            flow_row()
                .main_axis_alignment(MainAxisAlignment::Center)
                .item_spacing(Dp(80.0))
                .line_spacing(Dp(112.0))
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .padding_symmetric(Dp(48.0), Dp::ZERO),
                )
                .children(move || {
                    field_anatomy_item(false, outlined, geometry);
                    field_anatomy_item(true, outlined, geometry);
                });
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    field_pair_annotations(geometry, outlined);
                });
        });
    gap(Dp(24.0));
    anatomy_legend(if outlined {
        vec![
            "Enabled container outline",
            "Leading icon (optional)",
            "Label text in empty field",
            "Label text in populated field",
            "Trailing icon (optional)",
            "Focused container outline",
            "Caret",
            "Input text",
            "Supporting text (optional)",
        ]
    } else {
        vec![
            "Container",
            "Leading icon (optional)",
            "Label text in empty field",
            "Label text in populated field",
            "Trailing icon (optional)",
            "Focused active indicator",
            "Caret",
            "Input text",
            "Supporting text (optional)",
            "Enabled active indicator",
        ]
    });
}

#[tessera]
fn field_anatomy_item(populated: bool, outlined: bool, geometry: State<Geometry>) {
    let controller = remember(|| {
        let mut value = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        value.set_text(if populated { "Input text" } else { "" });
        value
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().width(Dp(180.0)))
        .children(move || {
            let field = if outlined {
                TextFieldBuilder::outlined()
            } else {
                TextFieldBuilder::filled()
            };
            let mut field = field
                .modifier(
                    watch(geometry, if populated { "populated" } else { "empty" }).fill_max_width(),
                )
                .min_width(Dp::ZERO)
                .controller(controller)
                .label("Label text")
                .accessibility_label("Label text")
                .accessibility_description("Supporting text")
                .leading_icon(|| {
                    icon().painter(filled::SEARCH_SVG).size(Dp(24.0));
                });
            if populated {
                field = field.trailing_icon(move || {
                    icon_button()
                        .standard()
                        .icon(filled::CANCEL_SVG)
                        .accessibility_label("Clear anatomy input")
                        .on_click(move || controller.with_mut(|value| value.set_text("")));
                });
            }
            drop(field);
            gap(Dp(4.0));
            text()
                .content("Supporting text")
                .color(scheme.on_surface_variant)
                .style(type_style(12.0, 16.0))
                .modifier(Modifier::new().padding_symmetric(Dp(16.0), Dp::ZERO));
        });
}

#[tessera]
fn field_pair_annotations(geometry: State<Geometry>, outlined: bool) {
    let values = geometry.get();
    let (Some(root), Some(empty), Some(pop)) = (
        values.get("figure"),
        relative(&values, "empty"),
        relative(&values, "populated"),
    ) else {
        return;
    };
    let mut svg = String::new();
    for (number, x, y, tx, ty) in [
        (1, empty.x - 44.0, empty.y, empty.x, empty.y + 16.0),
        (
            2,
            empty.x + 12.0,
            empty.y - 56.0,
            empty.x + 24.0,
            empty.y + 16.0,
        ),
        (
            3,
            empty.x + 80.0,
            empty.y - 56.0,
            empty.x + 92.0,
            empty.y + 28.0,
        ),
        (4, pop.x + 76.0, pop.y - 56.0, pop.x + 88.0, pop.y + 10.0),
        (
            5,
            pop.x + pop.w - 36.0,
            pop.y - 56.0,
            pop.x + pop.w - 24.0,
            pop.y + 16.0,
        ),
        (
            6,
            pop.x + pop.w + 24.0,
            pop.y + pop.h - 16.0,
            pop.x + pop.w,
            pop.y + pop.h,
        ),
        (
            7,
            pop.x + pop.w - 68.0,
            pop.y + pop.h + 32.0,
            pop.x + pop.w - 56.0,
            pop.y + pop.h - 12.0,
        ),
        (
            8,
            pop.x + 68.0,
            pop.y + pop.h + 32.0,
            pop.x + 80.0,
            pop.y + pop.h - 16.0,
        ),
        (
            9,
            empty.x + 78.0,
            empty.y + empty.h + 32.0,
            empty.x + 90.0,
            empty.y + empty.h + 12.0,
        ),
    ] {
        annotations::marker(number, Dp(x), Dp(y));
        if number == 1 || number == 6 {
            annotations::dotted(&mut svg, tx, ty, x - tx, false, "#49454f");
        } else {
            annotations::dotted(&mut svg, tx, ty, y + 16.0 - ty, true, "#49454f");
        }
    }
    if !outlined {
        annotations::marker(10, Dp(empty.x - 44.0), Dp(empty.y + empty.h - 16.0));
        annotations::dotted(
            &mut svg,
            empty.x - 20.0,
            empty.y + empty.h,
            20.0,
            false,
            "#49454f",
        );
    }
    for rect in [
        Rect {
            x: empty.x + 12.0,
            y: empty.y + 16.0,
            w: 24.0,
            h: 24.0,
        },
        Rect {
            x: empty.x + 52.0,
            y: empty.y + 16.0,
            w: 80.0,
            h: 24.0,
        },
        Rect {
            x: pop.x + 52.0,
            y: pop.y + 8.0,
            w: 76.0,
            h: 16.0,
        },
        Rect {
            x: pop.x + 52.0,
            y: pop.y + 24.0,
            w: 76.0,
            h: 24.0,
        },
        Rect {
            x: pop.x + pop.w - 36.0,
            y: pop.y + 16.0,
            w: 24.0,
            h: 24.0,
        },
    ] {
        annotations::outline(&mut svg, rect);
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}
