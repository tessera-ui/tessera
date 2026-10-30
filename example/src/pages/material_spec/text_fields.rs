//! Filled and outlined text field specification examples.
//!
//! ## Usage
//!
//! Edit short inputs and compare labels, icons, validation and supporting text.

use std::fmt::Write as _;

use tessera_components::{
    alignment::{Alignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    icon_button::{IconButtonColorStyle, icon_button},
    material_icons::filled,
    menus::{MenuAnchor, MenuController, menu_item, menu_provider},
    modifier::{InteractionVisualState, ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    text::text,
    text_field::{TextFieldBuilder, TextFieldContextMenu, TextFieldLineLimit},
    text_input::TextInputController,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, FocusState, LayoutCoordinates, Modifier, PxPosition, PxSize, RenderSlot, State,
    accesskit::Role, modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use super::{
    ACCENT, BACKGROUND, INK,
    annotations::{self, Rect, SpaceGeometry, watch_in_space},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, srgb, subheading, type_style,
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
        ArticlePage{title:"Overview",intro:Some(RenderSlot::new(overview_variants)),sections:vec![SpecSection{title:"Differences from M2",content:RenderSlot::new(|| {prose("Text fields use updated color mappings and dynamic color, with distinct filled and outlined treatments.");gap(Dp(24.0));field_variant_hero(false);})}]},
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
    anatomy_geometry: Option<State<SpaceGeometry>>,
) {
    let initial = if populated.unwrap_or(true) {
        if anatomy_geometry.is_some() {
            "Input text"
        } else {
            "Input"
        }
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
                            .enabled(active.get() && !read_only.get())
                            .on_click(move || {
                                controller.with(|controller| controller.request_focus());
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
                        .modifier(if let Some(geometry) = anatomy_geometry {
                            watch_in_space(
                                geometry,
                                if populated.unwrap_or(true) {
                                    "populated"
                                } else {
                                    "empty"
                                },
                            )
                            .fill_max_width()
                        } else {
                            Modifier::new().fill_max_width()
                        })
                        .min_width(Dp::ZERO)
                        .min_height(Dp(if multiline.get() { 96.0 } else { 56.0 }))
                        .controller(controller)
                        .label(if anatomy_geometry.is_some() {
                            "Label text"
                        } else {
                            "Label"
                        })
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
                    if anatomy_geometry.is_some() && populated.unwrap_or(true) {
                        field = field.presentation_state(InteractionVisualState::Focus);
                    }
                    if icons.get() {
                        field = field.leading_icon_shared(RenderSlot::new(move || {
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
                        }));
                        if value.with(|value| !value.is_empty()) {
                            field = field.trailing_icon_shared(RenderSlot::new(move || {
                                icon_button()
                                    .standard()
                                    .color_style(IconButtonColorStyle::Inherited)
                                    .icon(filled::CANCEL_SVG)
                                    .accessibility_label(format!("Clear {} icon", label.get()))
                                    .enabled(active.get() && !read_only.get())
                                    .on_click(move || {
                                        controller.with(|controller| controller.request_focus());
                                        controller.with_mut(|controller| controller.set_text(""));
                                        value.set(String::new());
                                    });
                            }));
                        }
                    }
                    if affixes.get() {
                        field = field
                            .prefix_shared(RenderSlot::new(|| {
                                text().content("$");
                            }))
                            .suffix_shared(RenderSlot::new(|| {
                                text().content(".00");
                            }));
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
                        })
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(if invalid.get() {
                                Role::Alert
                            } else {
                                Role::Label
                            }),
                            ..Default::default()
                        }));
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
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let title = if outlined {
                "Outlined text field"
            } else {
                "Filled text field"
            };
            specs_anatomy(outlined);
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
            gap(Dp(112.0));
            subheading(format!("{title} color"));
            gap(Dp(24.0));
            for dark in [false, true] {
                field_color_scheme(outlined, dark);
            }
            gap(Dp(24.0));
            anatomy_legend(if outlined {
                vec![
                    "Outline",
                    "On surface variant",
                    "On surface variant",
                    "Primary",
                    "On surface variant",
                    "Primary",
                    "Primary",
                    "On surface",
                    "On surface variant",
                ]
            } else {
                vec![
                    "Surface container highest",
                    "On surface variant",
                    "On surface variant",
                    "Primary",
                    "On surface variant",
                    "Primary",
                    "Primary",
                    "On surface",
                    "On surface variant",
                    "On surface",
                ]
            });
            gap(Dp(88.0));
            subheading(format!("{title} states"));
            gap(Dp(24.0));
            for dark in [false, true] {
                field_state_matrix(outlined, false, dark);
            }
            gap(Dp(24.0));
            anatomy_legend(vec![
                "Enabled, empty",
                "Focused, empty",
                "Hovered, empty",
                "Disabled, empty",
                "Enabled, populated",
                "Focused, populated",
                "Hovered, populated",
                "Disabled, populated",
            ]);
            gap(Dp(112.0));
            subheading(format!("{title} error states"));
            gap(Dp(24.0));
            for dark in [false, true] {
                field_state_matrix(outlined, true, dark);
            }
            gap(Dp(24.0));
            anatomy_legend(vec![
                "Enabled error, empty",
                "Focused error, empty",
                "Hovered error, empty",
                "Enabled error, populated",
                "Focused error, populated",
                "Hovered error, populated",
            ]);
            gap(Dp(112.0));
            subheading(format!("{title} measurements"));
            gap(Dp(24.0));
            for variant in 0..3 {
                field_measurements(outlined, variant);
                gap(Dp(24.0));
            }
            spec_table(
                vec!["Attribute".into(), "Value".into()],
                if outlined {
                    vec![
                        vec!["Container height".into(), "56dp".into()],
                        vec!["Left/right padding without icons".into(), "16dp".into()],
                        vec!["Left/right padding with icons".into(), "12dp".into()],
                        vec!["Padding between icons and text".into(), "16dp".into()],
                        vec!["Icon alignment".into(), "Vertically centered".into()],
                        vec![
                            "Supporting text and character counter top padding".into(),
                            "4dp".into(),
                        ],
                        vec![
                            "Padding between supporting text and character counter".into(),
                            "16dp".into(),
                        ],
                        vec!["Label alignment".into(), "Vertically centered".into()],
                        vec![
                            "Left/right padding populated label text".into(),
                            "4dp".into(),
                        ],
                        vec!["Target size".into(), "56dp".into()],
                    ]
                } else {
                    vec![
                        vec!["Default container height".into(), "56dp".into()],
                        vec![
                            "Label alignment (unpopulated)".into(),
                            "Vertically centered".into(),
                        ],
                        vec!["Top/bottom padding".into(), "8dp".into()],
                        vec!["Left/right padding without icons".into(), "16dp".into()],
                        vec!["Left/right padding with icons".into(), "12dp".into()],
                        vec!["Icon alignment".into(), "Vertically centered".into()],
                        vec!["Padding between icons and text".into(), "16dp".into()],
                        vec![
                            "Supporting text and character counter top padding".into(),
                            "4dp".into(),
                        ],
                        vec![
                            "Padding between supporting text and character counter".into(),
                            "16dp".into(),
                        ],
                        vec!["Target size".into(), "56dp".into()],
                    ]
                },
            );
            gap(Dp(112.0));
            subheading(format!("{title} configurations"));
            gap(Dp(24.0));
            field_configurations(outlined);
            gap(Dp(24.0));
            anatomy_legend(vec![
                "Supporting text",
                "Trailing clear icon",
                "Leading icon",
                "Leading and trailing icons",
                "Prefix",
                "Suffix",
                "Multiline input",
            ]);
            gap(Dp(48.0));
            subheading("Try the configuration options".into());
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
    field_canvas(
        676.0,
        320.0,
        RenderSlot::new(move || {
            let geometry = remember(SpaceGeometry::new);
            let controller = remember(|| {
                let mut value = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
                value.set_text("Input");
                value
            });
            let value = remember(|| "Input".to_owned());
            let populated = value.with(|value| !value.is_empty());
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch_in_space(geometry, "figure")
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
                            let mut field = field
                                .modifier(watch_in_space(geometry, "field").fill_max_width())
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
                                .on_change(move |next: String| {
                                    value.set(next.clone());
                                    next
                                })
                                .line_limit(if multiline {
                                    TextFieldLineLimit::MultiLine
                                } else {
                                    TextFieldLineLimit::SingleLine
                                })
                                .leading_icon_shared(RenderSlot::new(|| {
                                    icon().painter(filled::SEARCH_SVG).size(Dp(24.0));
                                }));
                            if error || populated {
                                field = field.trailing_icon_shared(RenderSlot::new(move || {
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
                                    } else if populated {
                                        icon_button()
                                            .standard()
                                            .icon(filled::CANCEL_SVG)
                                            .accessibility_label("Clear illustrated text field")
                                            .on_click(move || {
                                                controller
                                                    .with(|controller| controller.request_focus());
                                                controller
                                                    .with_mut(|controller| controller.set_text(""));
                                                value.set(String::new());
                                            });
                                    }
                                }));
                            }
                            drop(field);
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
                                .modifier(
                                    watch_in_space(geometry, "support")
                                        .padding_symmetric(Dp(16.0), Dp::ZERO)
                                        .semantics(SemanticsArgs {
                                            role: Some(if error {
                                                Role::Alert
                                            } else {
                                                Role::Label
                                            }),
                                            ..Default::default()
                                        }),
                                );
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            field_annotations(geometry, measurements);
                        });
                });
        }),
    );
}

#[tessera]
fn field_annotations(geometry: State<SpaceGeometry>, measurements: bool) {
    field_annotation_geometry(geometry.with(annotations::space_geometry), measurements);
}

#[tessera]
fn field_annotation_geometry(
    values: std::collections::BTreeMap<&'static str, Rect>,
    measurements: bool,
) {
    let (Some(root), Some(field), Some(support)) = (
        values.get("figure"),
        values.get("field").copied(),
        values.get("support").copied(),
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
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn overview_variants() {
    field_variant_hero(true);
}

#[tessera]
fn field_variant_hero(numbered: bool) {
    field_canvas(
        if numbered { 1025.0 } else { 1024.0 },
        if numbered { 300.0 } else { 238.0 },
        RenderSlot::new(move || {
            let theme = use_context::<MaterialTheme>().unwrap().get();
            let scale = 1.0;
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(if numbered { 300.0 } else { 238.0 } * scale))
                        .background_with_shape(
                            theme.color_scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
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
                        .child(move || {
                            overview_variant(false, scale, numbered);
                            overview_variant(true, scale, numbered);
                        });
                });
        }),
    );
}

#[tessera]
fn overview_variant(outlined: bool, scale: f64, numbered: bool) {
    let title = if outlined { "Outlined" } else { "Filled" };
    let x = if outlined { 552.0 } else { 192.0 };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(91.0 * scale))
                .width(Dp(280.0 * scale)),
        )
        .children(move || {
            let field = if outlined {
                TextFieldBuilder::outlined()
            } else {
                TextFieldBuilder::filled()
            };
            let mut field = field
                .label("Text field")
                .initial_text(title)
                .accessibility_label(format!("Overview {title} text field"))
                .accessibility_description("Supporting text")
                .min_width(Dp::ZERO)
                .min_height(Dp(56.0 * scale))
                .padding(Dp(16.0 * scale))
                .font_size(Dp(16.0 * scale))
                .line_height(Dp(24.0 * scale))
                .modifier(
                    Modifier::new()
                        .width(Dp(280.0 * scale))
                        .height(Dp(56.0 * scale)),
                );
            if outlined {
                field = field
                    .border_width(Dp(scale))
                    .focus_border_width(Dp(2.0 * scale));
            }
            drop(field);
            gap(Dp(4.0 * scale));
            text()
                .content("Supporting text")
                .style(type_style(12.0 * scale, 16.0 * scale))
                .color(scheme.on_surface_variant)
                .modifier(Modifier::new().padding_symmetric(Dp(16.0 * scale), Dp::ZERO));
        });
    if numbered {
        boxed()
            .alignment(Alignment::Center)
            .modifier(
                Modifier::new()
                    .offset(Dp((x + 120.0) * scale), Dp(189.0 * scale))
                    .size(Dp(40.0 * scale), Dp(40.0 * scale))
                    .background_with_shape(
                        scheme.on_surface,
                        Shape::rounded_rectangle(Dp(20.0 * scale)),
                    )
                    .semantics(SemanticsArgs {
                        role: Some(Role::Note),
                        label: Some(format!(
                            "{} {title} text field",
                            if outlined { 2 } else { 1 }
                        )),
                        ..Default::default()
                    }),
            )
            .children(move || {
                text()
                    .content(if outlined { "2" } else { "1" })
                    .style(type_style(16.0 * scale, 24.0 * scale))
                    .color(scheme.surface);
            });
    }
}

#[tessera]
fn field_variants() {
    prose(
        "Text fields let people enter and edit text. Choose filled or outlined fields consistently within a form.",
    );
    gap(Dp(24.0));
    field_variant_hero(false);
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
    let overflow = mode == 6 && count.get() > 100;
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
                .accessibility_description(if overflow {
                    "Use 100 characters or fewer"
                } else {
                    match mode {
                        2 => "Required",
                        7 => "Enter a valid value",
                        8 => "Read only",
                        _ => "Supporting text",
                    }
                })
                .read_only(mode == 8)
                .is_error(mode == 7 || overflow)
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
                field = field.prefix_shared(RenderSlot::new(|| {
                    text()
                        .content("$")
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(Role::Label),
                            label: Some("US dollars".into()),
                            ..Default::default()
                        }));
                }));
            }
            if mode == 5 {
                field = field.suffix_shared(RenderSlot::new(|| {
                    text()
                        .content("@gmail.com")
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(Role::Label),
                            label: Some("at gmail dot com".into()),
                            ..Default::default()
                        }));
                }));
            }
            if mode == 7 {
                field = field.trailing_icon_shared(RenderSlot::new(|| {
                    boxed()
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(Role::Image),
                            label: Some("Error".into()),
                            ..Default::default()
                        }))
                        .children(|| {
                            icon().painter(filled::ERROR_SVG).size(Dp(24.0));
                        });
                }));
            }
            if mode == 9 {
                if !show.get() {
                    field = field
                        .obfuscation_char('•')
                        .context_menu(TextFieldContextMenu::secure_default());
                }
                field = field.trailing_icon_shared(RenderSlot::new(move || {
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
                }));
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
                    let support = if overflow {
                        "Use 100 characters or fewer"
                    } else {
                        match mode {
                            2 => "* Required",
                            7 => "Enter a valid value",
                            8 => "Read only",
                            _ => "Supporting text",
                        }
                    };
                    text()
                        .content(support)
                        .style(type_style(12.0, 16.0))
                        .color(if mode == 7 || overflow {
                            scheme.error
                        } else {
                            scheme.on_surface_variant
                        })
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(if mode == 7 || overflow {
                                Role::Alert
                            } else {
                                Role::Label
                            }),
                            label: Some(support.into()),
                            ..Default::default()
                        }));
                    if mode == 6 {
                        spacer().modifier(Modifier::new().weight(1.0));
                        text()
                            .content(format!("{} / 100", count.get()))
                            .color(if overflow {
                                scheme.error
                            } else {
                                scheme.on_surface_variant
                            })
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
    field_container_diagram();
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
    field_error_icon_diagram();
    gap(Dp(24.0));
    subheading("Icons & images".into());
    prose(
        "Use optional 24dp images or icons to clarify input or expose an actual action, such as clearing entered text.",
    );
    gap(Dp(24.0));
    field_icons_diagram();
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
fn field_diagram_theme(scale: f64, child: RenderSlot) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    material_theme()
        .theme(move || {
            let mut theme = theme.clone();
            for style in [
                &mut theme.typography.body_large,
                &mut theme.typography.body_small,
            ] {
                style.font_size = Dp(style.font_size.0 * scale);
                style.line_height = style.line_height.map(|height| Dp(height.0 * scale));
                style.letter_spacing = Dp(style.letter_spacing.0 * scale);
            }
            theme
        })
        .child_shared(child);
}

#[tessera]
fn field_container_diagram() {
    field_canvas(
        676.0,
        425.0,
        RenderSlot::new(move || {
            let geometry = remember(SpaceGeometry::new);
            let scale = 1.0;
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .modifier(
                    watch_in_space(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(425.0 * scale))
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(198.0 * scale), Dp(127.0 * scale))
                                .size(Dp(280.0 * scale), Dp(56.0 * scale))
                                .background_with_shape(
                                    scheme.surface_container_highest,
                                    Shape::RoundedRectangle {
                                        top_left: RoundedCorner::new(Dp(4.0 * scale)),
                                        top_right: RoundedCorner::new(Dp(4.0 * scale)),
                                        bottom_left: RoundedCorner::ZERO,
                                        bottom_right: RoundedCorner::ZERO,
                                    },
                                )
                                .semantics(SemanticsArgs {
                                    role: Some(Role::Group),
                                    label: Some("Filled text field container".into()),
                                    ..Default::default()
                                }),
                        )
                        .children(move || {
                            spacer().modifier(
                                Modifier::new()
                                    .offset(Dp::ZERO, Dp(53.0 * scale))
                                    .size(Dp(280.0 * scale), Dp(3.0 * scale))
                                    .background(scheme.primary),
                            );
                        });
                    spacer().modifier(
                        Modifier::new()
                            .offset(Dp(198.0 * scale), Dp(243.0 * scale))
                            .size(Dp(280.0 * scale), Dp(56.0 * scale))
                            .border_with_shape(
                                Dp(3.0 * scale),
                                scheme.primary,
                                Shape::rounded_rectangle(Dp(4.0 * scale)),
                            )
                            .semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some("Outlined text field container".into()),
                                ..Default::default()
                            }),
                    );
                });
        }),
    );
}

#[tessera]
fn field_error_icon_diagram() {
    field_canvas(
        676.0,
        300.0,
        RenderSlot::new(move || {
            let geometry = remember(SpaceGeometry::new);
            let scale = 1.0;
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .modifier(
                    watch_in_space(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(300.0 * scale))
                        .clip_to_bounds()
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(124.0 * scale), Dp::ZERO)
                                .size(Dp(428.0 * scale), Dp(300.0 * scale))
                                .background(scheme.outline_variant),
                        )
                        .children(move || {
                            spacer().modifier(
                                Modifier::new()
                                    .offset(Dp(8.0 * scale), Dp::ZERO)
                                    .size(Dp(412.0 * scale), Dp(300.0 * scale))
                                    .background(scheme.surface),
                            );
                        });
                    field_diagram_theme(
                        scale,
                        RenderSlot::new(move || {
                            field_error_icon_example(false, scale);
                            field_error_icon_example(true, scale);
                        }),
                    );
                });
        }),
    );
}

#[tessera]
fn field_error_icon_example(password: bool, scale: f64) {
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale)));
        controller.set_text(if password { "Password" } else { "Sam" });
        controller
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let support = if password {
        "Include an uppercase letter, a lowercase letter, and a numeric character"
    } else {
        "Use at least 6 characters"
    };
    column()
        .modifier(
            Modifier::new()
                .offset(
                    Dp(156.0 * scale),
                    Dp(if password { 184.0 } else { 63.0 } * scale),
                )
                .width(Dp(364.0 * scale)),
        )
        .children(move || {
            let mut field = TextFieldBuilder::outlined()
                .controller(controller)
                .is_error(true)
                .presentation_state_optional(if password {
                    None
                } else {
                    Some(InteractionVisualState::Focus)
                })
                .min_width(Dp::ZERO)
                .padding(Dp(16.0 * scale))
                .font_size(Dp(16.0 * scale))
                .line_height(Dp(24.0 * scale))
                .min_height(Dp(56.0 * scale))
                .border_width(Dp(if password { 1.0 } else { 3.0 } * scale))
                .focus_border_width(Dp(3.0 * scale))
                .modifier(
                    Modifier::new()
                        .width(Dp(364.0 * scale))
                        .height(Dp(56.0 * scale)),
                )
                .accessibility_label(if password { "Password" } else { "Username" })
                .accessibility_description(support)
                .trailing_icon_shared(RenderSlot::new(move || {
                    boxed()
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(Role::Image),
                            label: Some("Error".into()),
                            ..Default::default()
                        }))
                        .children(move || {
                            icon()
                                .painter(filled::ERROR_SVG)
                                .size(Dp(24.0 * scale))
                                .tint(scheme.error);
                        });
                }));
            if password {
                field = field
                    .obfuscation_char('•')
                    .text_color(scheme.error)
                    .context_menu(TextFieldContextMenu::secure_default());
            } else {
                field = field.label("Username");
            }
            drop(field);
            gap(Dp(4.0 * scale));
            text()
                .content(support)
                .style(type_style(12.0 * scale, 16.0 * scale))
                .color(scheme.error)
                .modifier(
                    Modifier::new()
                        .padding_symmetric(Dp(16.0 * scale), Dp::ZERO)
                        .semantics(SemanticsArgs {
                            role: Some(Role::Alert),
                            label: Some(support.into()),
                            ..Default::default()
                        }),
                );
        });
}

#[tessera]
fn field_icons_diagram() {
    field_canvas(
        676.0,
        320.0,
        RenderSlot::new(move || {
            let geometry = remember(SpaceGeometry::new);
            let scale = 1.0;
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .modifier(
                    watch_in_space(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(320.0 * scale))
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    field_diagram_theme(
                        scale,
                        RenderSlot::new(move || {
                            field_icon_example(1, scale);
                            field_icon_example(2, scale);
                            field_icon_example(3, scale);
                            field_icon_example(4, scale);
                            field_icon_example(5, scale);
                            field_icon_example(6, scale);
                        }),
                    );
                });
        }),
    );
}

#[tessera]
fn field_icon_example(kind: usize, scale: f64) {
    let column = if kind <= 3 { 0.0 } else { 1.0 };
    let row = (kind - 1) % 3;
    let x = 111.0 + column * 284.0;
    let y = 36.0 + row as f64 * 96.0;
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale)));
        controller.set_text(match kind {
            2 => "Input",
            3 => "Input text",
            _ => "",
        });
        controller
    });
    let value = remember(|| match kind {
        2 => "Input".to_owned(),
        3 => "Input text".to_owned(),
        _ => String::new(),
    });
    let menu = remember(MenuController::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(180.0 * scale), Dp(56.0 * scale)),
        )
        .children(move || {
            let content = RenderSlot::new(move || {
                field_icon_input(kind, scale, controller, value, menu);
            });
            if kind == 5 {
                menu_provider()
                    .controller(menu)
                    .anchor(MenuAnchor::new(
                        PxPosition::new(Dp::ZERO.into(), Dp::ZERO.into()),
                        PxSize::new(Dp(180.0 * scale).into(), Dp(56.0 * scale).into()),
                    ))
                    .modifier(Modifier::new().size(Dp(180.0 * scale), Dp(56.0 * scale)))
                    .main_content_shared(content)
                    .menu_content(move || {
                        menu_item().label("Personal").on_click(move || {
                            controller.with_mut(|controller| controller.set_text("Personal"));
                            value.set("Personal".into());
                        });
                        menu_item().label("Work").on_click(move || {
                            controller.with_mut(|controller| controller.set_text("Work"));
                            value.set("Work".into());
                        });
                    });
            } else {
                content.render();
            }
        });
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp((x - 39.5) * scale), Dp((y + 14.0) * scale))
                .size(Dp(24.0 * scale), Dp(24.0 * scale))
                .background_with_shape(
                    scheme.on_surface,
                    Shape::rounded_rectangle(Dp(12.0 * scale)),
                )
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Text field icon example {kind}")),
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(kind.to_string())
                .style(type_style(16.0 * scale, 24.0 * scale))
                .color(scheme.surface);
        });
}

#[tessera]
fn field_icon_input(
    kind: usize,
    scale: f64,
    controller: State<TextInputController>,
    value: State<String>,
    menu: State<MenuController>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let mut field = TextFieldBuilder::filled()
        .controller(controller)
        .min_width(Dp::ZERO)
        .min_height(Dp(56.0 * scale))
        .font_size(Dp(16.0 * scale))
        .line_height(Dp(24.0 * scale))
        .padding(Dp(16.0 * scale))
        .label(if kind == 2 { "Focused" } else { "Label text" })
        .accessibility_label(match kind {
            1 => "Calendar icon example",
            2 => "Error icon example",
            3 => "Clear icon example",
            4 => "Voice input icon example",
            5 => "Dropdown icon example",
            _ => "Payment image example",
        })
        .is_error(kind == 2)
        .presentation_state_optional(if matches!(kind, 2 | 3 | 6) {
            Some(InteractionVisualState::Focus)
        } else {
            None
        })
        .modifier(
            Modifier::new()
                .width(Dp(180.0 * scale))
                .height(Dp(56.0 * scale)),
        )
        .on_change(move |next: String| {
            value.set(next.clone());
            next
        });
    if kind == 1 {
        field = field.leading_icon_shared(RenderSlot::new(move || {
            icon()
                .painter(filled::CALENDAR_TODAY_SVG)
                .size(Dp(24.0 * scale));
        }));
    } else if kind == 2 {
        field = field.trailing_icon_shared(RenderSlot::new(move || {
            icon()
                .painter(filled::ERROR_SVG)
                .size(Dp(24.0 * scale))
                .tint(scheme.error);
        }));
    } else if kind == 3 && value.with(|value| !value.is_empty()) {
        field = field.trailing_icon_shared(RenderSlot::new(move || {
            icon_button()
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(filled::CANCEL_SVG)
                .accessibility_label("Clear icon example text")
                .on_click(move || {
                    controller.with(|controller| controller.request_focus());
                    controller.with_mut(|controller| controller.set_text(""));
                    value.set(String::new());
                });
        }));
    } else if kind == 4 {
        field = field.trailing_icon_shared(RenderSlot::new(move || {
            icon().painter(filled::MIC_SVG).size(Dp(24.0 * scale));
        }));
    } else if kind == 5 {
        field = field.trailing_icon_shared(RenderSlot::new(move || {
            icon_button()
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(filled::ARROW_DROP_DOWN_SVG)
                .accessibility_label("Choose icon example category")
                .on_click(move || menu.with_mut(MenuController::toggle));
        }));
    } else if kind == 6 {
        field = field.trailing_icon_shared(RenderSlot::new(move || {
            field_payment_image(scale);
        }));
    }
    drop(field);
}

#[tessera]
fn field_payment_image(scale: f64) {
    boxed()
        .modifier(
            Modifier::new()
                .size(Dp(32.0 * scale), Dp(24.0 * scale))
                .background(srgb(0, 0, 0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some("Payment card".into()),
                    ..Default::default()
                }),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(8.0 * scale), Dp(4.0 * scale))
                        .size(Dp(16.0 * scale), Dp(16.0 * scale))
                        .background_with_shape(srgb(247, 158, 27), Shape::Ellipse)
                        .clip_shape(Shape::Ellipse),
                )
                .children(move || {
                    spacer().modifier(
                        Modifier::new()
                            .offset(Dp(8.0 * scale), Dp::ZERO)
                            .size(Dp(8.0 * scale), Dp(16.0 * scale))
                            .background(srgb(255, 205, 64)),
                    );
                });
        });
}

#[tessera]
fn field_accessibility_labels() {
    super::text_fields_accessibility::labeling();
}

#[tessera]
fn field_anatomy_pair(outlined: bool) {
    field_canvas(
        1024.0,
        240.0,
        RenderSlot::new(move || {
            let geometry = remember(SpaceGeometry::new);
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch_in_space(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(240.0))
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    row()
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .padding_symmetric(Dp(48.0), Dp::ZERO)
                                .offset(Dp::ZERO, Dp(9.0)),
                        )
                        .children(move || {
                            field_anatomy_item(false, outlined, geometry);
                            spacer().modifier(Modifier::new().width(Dp(80.0)));
                            field_anatomy_item(true, outlined, geometry);
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            field_pair_annotations(geometry, outlined);
                        });
                });
        }),
    );

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
fn field_anatomy_item(populated: bool, outlined: bool, geometry: State<SpaceGeometry>) {
    let controller = remember(|| {
        let mut value = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        value.set_text(if populated { "Input text" } else { "" });
        value
    });
    let value = remember(|| {
        if populated {
            "Input text".to_owned()
        } else {
            String::new()
        }
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
                    watch_in_space(geometry, if populated { "populated" } else { "empty" })
                        .fill_max_width(),
                )
                .min_width(Dp::ZERO)
                .controller(controller)
                .label("Label text")
                .accessibility_label("Label text")
                .accessibility_description("Supporting text")
                .on_change(move |next: String| {
                    value.set(next.clone());
                    next
                })
                .leading_icon_shared(RenderSlot::new(|| {
                    icon().painter(filled::SEARCH_SVG).size(Dp(24.0));
                }));
            if value.with(|value| !value.is_empty()) {
                field = field.trailing_icon_shared(RenderSlot::new(move || {
                    icon_button()
                        .standard()
                        .icon(filled::CANCEL_SVG)
                        .accessibility_label("Clear anatomy input")
                        .on_click(move || {
                            controller.with(|controller| controller.request_focus());
                            controller.with_mut(|controller| controller.set_text(""));
                            value.set(String::new());
                        });
                }));
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
fn field_pair_annotations(geometry: State<SpaceGeometry>, outlined: bool) {
    field_pair_annotation_geometry(geometry.with(annotations::space_geometry), outlined);
}

#[tessera]
fn field_pair_annotation_geometry(
    values: std::collections::BTreeMap<&'static str, Rect>,
    outlined: bool,
) {
    let (Some(root), Some(empty), Some(pop)) = (
        values.get("figure"),
        values.get("empty").copied(),
        values.get("populated").copied(),
    ) else {
        return;
    };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let ink = field_svg_color(scheme.on_surface);
    let mut svg = String::new();
    let markers = if outlined {
        [
            (1, empty.x - 43.0, empty.y + 12.0, empty.x, empty.y + 28.0),
            (
                2,
                empty.x + 12.0,
                empty.y - 38.0,
                empty.x + 24.0,
                empty.y + 16.0,
            ),
            (
                3,
                empty.x + 78.0,
                empty.y - 38.0,
                empty.x + 90.0,
                empty.y + 15.0,
            ),
            (4, pop.x + 32.0, pop.y - 53.0, pop.x + 44.0, pop.y - 8.0),
            (
                5,
                pop.x + pop.w - 36.0,
                pop.y - 37.0,
                pop.x + pop.w - 24.0,
                pop.y + 16.0,
            ),
            (
                6,
                pop.x + pop.w + 20.0,
                pop.y + 12.0,
                pop.x + pop.w,
                pop.y + 28.0,
            ),
            (
                7,
                pop.x + 111.0,
                pop.y + pop.h + 25.0,
                pop.x + 123.0,
                pop.y + 35.0,
            ),
            (
                8,
                pop.x + 78.0,
                pop.y + pop.h + 25.0,
                pop.x + 90.0,
                pop.y + 40.0,
            ),
            (
                9,
                empty.x + 78.0,
                empty.y + empty.h + 34.0,
                empty.x + 90.0,
                empty.y + empty.h + 21.0,
            ),
        ]
    } else {
        [
            (1, empty.x - 44.0, empty.y, empty.x, empty.y + 16.0),
            (
                2,
                empty.x + 12.0,
                empty.y - 37.0,
                empty.x + 24.0,
                empty.y + 16.0,
            ),
            (
                3,
                empty.x + 78.0,
                empty.y - 37.0,
                empty.x + 90.0,
                empty.y + 16.0,
            ),
            (4, pop.x + 76.0, pop.y - 37.0, pop.x + 88.0, pop.y + 8.0),
            (
                5,
                pop.x + pop.w - 36.0,
                pop.y - 37.0,
                pop.x + pop.w - 24.0,
                pop.y + 16.0,
            ),
            (
                6,
                pop.x + pop.w + 19.0,
                pop.y + pop.h - 16.0,
                pop.x + pop.w - 1.0,
                pop.y + pop.h,
            ),
            (
                7,
                pop.x + pop.w - 68.0,
                pop.y + pop.h + 32.0,
                pop.x + pop.w - 56.0,
                pop.y + pop.h - 14.0,
            ),
            (
                8,
                pop.x + 78.0,
                pop.y + pop.h + 32.0,
                pop.x + 90.0,
                pop.y + pop.h - 8.0,
            ),
            (
                9,
                empty.x + 78.0,
                empty.y + empty.h + 32.0,
                empty.x + 90.0,
                empty.y + empty.h + 20.0,
            ),
        ]
    };
    for (number, x, y, tx, ty) in markers {
        field_anatomy_marker(number, Dp(x), Dp(y));
        if matches!(number, 1 | 6 | 7) {
            let (ex, ey) = if number == 7 {
                (tx, y)
            } else {
                (if x < tx { x + 24.0 } else { x }, ty)
            };
            write!(
                &mut svg,
                "<path d='M{tx} {ty}L{ex} {ey}' fill='none' stroke='{ink}' stroke-width='1'/><circle cx='{tx}' cy='{ty}' r='3.5' fill='none' stroke='{ink}' stroke-width='1'/>"
            )
            .expect("writing an anatomy connector");
        } else {
            let edge = if y + 32.0 <= ty { y + 32.0 } else { y };
            annotations::dotted(&mut svg, tx, ty, edge - ty, true, &ink);
        }
    }
    if !outlined {
        field_anatomy_marker(10, Dp(empty.x - 44.0), Dp(empty.y + empty.h - 16.0));
        let tx = empty.x;
        let ty = empty.y + empty.h;
        let edge = empty.x - 20.0;
        write!(
            &mut svg,
            "<path d='M{edge} {ty}H{tx}' fill='none' stroke='{ink}' stroke-width='1'/><circle cx='{tx}' cy='{ty}' r='3.5' fill='none' stroke='{ink}' stroke-width='1'/>"
        )
        .expect("writing an indicator connector");
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
            w: 76.0,
            h: 24.0,
        },
        Rect {
            x: pop.x + if outlined { 16.0 } else { 52.0 },
            y: pop.y + if outlined { -8.0 } else { 8.0 },
            w: if outlined { 64.0 } else { 76.0 },
            h: 16.0,
        },
        Rect {
            x: pop.x + 52.0,
            y: pop.y + if outlined { 16.0 } else { 24.0 },
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
        field_annotation_outline(&mut svg, rect, &ink);
    }
    field_annotation_outline(
        &mut svg,
        Rect {
            x: empty.x + if outlined { 12.0 } else { 16.0 },
            y: empty.y + empty.h + if outlined { 3.0 } else { 4.0 },
            w: 148.0,
            h: if outlined { 18.0 } else { 16.0 },
        },
        &ink,
    );
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn specs_anatomy(outlined: bool) {
    let dark = use_context::<MaterialTheme>()
        .unwrap()
        .get()
        .color_scheme
        .is_dark;
    field_canvas(
        1024.0,
        300.0,
        RenderSlot::new(move || {
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
                )
                .children(move || {
                    specs_anatomy_panel(outlined, dark);
                });
        }),
    );
}

#[tessera]
fn specs_anatomy_panel(outlined: bool, dark: bool) {
    field_canvas(
        1024.0,
        300.0,
        RenderSlot::new(move || {
            let geometry = remember(SpaceGeometry::new);
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch_in_space(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(300.0))
                        .background(scheme.surface_container_low)
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some(format!(
                                "{} {} anatomy panel",
                                if dark { "Dark" } else { "Light" },
                                if outlined { "outlined" } else { "filled" }
                            )),
                            merge_descendants: true,
                            ..Default::default()
                        }),
                )
                .children(move || {
                    row()
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .padding_symmetric(Dp(48.0), Dp::ZERO)
                                .offset(Dp::ZERO, Dp(if outlined { 4.0 } else { 10.0 })),
                        )
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .children(move || {
                            for populated in [false, true] {
                                if populated {
                                    spacer().modifier(Modifier::new().width(Dp(80.0)));
                                }
                                sample(
                                    format!(
                                        "{}{} {} anatomy",
                                        if dark { "Dark " } else { "" },
                                        if outlined { "Outlined" } else { "Filled" },
                                        if populated { "populated" } else { "empty" }
                                    ),
                                    outlined,
                                )
                                .populated(populated)
                                .icons(true)
                                .preview_width(Dp(180.0))
                                .anatomy_geometry(geometry);
                            }
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            field_pair_annotations(geometry, outlined);
                        });
                });
        }),
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "text_fields_tests.rs"]
mod regression_tests;

#[tessera]
fn field_color_scheme(outlined: bool, dark: bool) {
    field_canvas(
        1024.0,
        240.0,
        RenderSlot::new(move || {
            let scale = 1.0;
            material_theme()
                .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                .child(move || {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    surface()
                        .style(scheme.surface_container_low.into())
                        .shape(Shape::rounded_rectangle(Dp(24.0)))
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .height(Dp(240.0 * scale))
                                .semantics(SemanticsArgs {
                                    role: Some(Role::Group),
                                    label: Some(format!(
                                        "{} {} color panel",
                                        if dark { "Dark" } else { "Light" },
                                        if outlined { "outlined" } else { "filled" }
                                    )),
                                    merge_descendants: true,
                                    ..Default::default()
                                }),
                        )
                        .child(move || {
                            field_diagram_theme(
                                scale,
                                RenderSlot::new(move || {
                                    for populated in [false, true] {
                                        boxed()
                                            .modifier(Modifier::new().offset(
                                                Dp(if populated { 552.0 } else { 292.0 } * scale),
                                                Dp(91.0 * scale),
                                            ))
                                            .children(move || {
                                                field_state_item(
                                                    outlined,
                                                    populated,
                                                    if populated {
                                                        InteractionVisualState::Focus
                                                    } else {
                                                        InteractionVisualState::Rest
                                                    },
                                                    true,
                                                    false,
                                                    false,
                                                    scale,
                                                    format!(
                                                        "{} {} color {}",
                                                        if dark { "Dark" } else { "Light" },
                                                        if outlined {
                                                            "outlined"
                                                        } else {
                                                            "filled"
                                                        },
                                                        if populated {
                                                            "populated"
                                                        } else {
                                                            "empty"
                                                        }
                                                    ),
                                                );
                                            });
                                    }
                                    field_color_callouts(outlined, scale);
                                }),
                            );
                        });
                });
        }),
    );
}

#[tessera]
fn field_color_callouts(outlined: bool, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let roles = if outlined {
        vec![
            "Outline",
            "On surface variant",
            "On surface variant",
            "Primary",
            "On surface variant",
            "Primary",
            "Primary",
            "On surface",
            "On surface variant",
        ]
    } else {
        vec![
            "Surface container highest",
            "On surface variant",
            "On surface variant",
            "Primary",
            "On surface variant",
            "Primary",
            "Primary",
            "On surface",
            "On surface variant",
            "On surface",
        ]
    };
    let anchors = [
        (248.0, 92.0, 292.0, 107.0),
        (304.0, 54.0, 316.0, 107.0),
        (370.0, 54.0, 382.0, 107.0),
        (628.0, 54.0, 640.0, 101.0),
        (696.0, 54.0, 708.0, 107.0),
        (752.0, 131.0, 732.0, 147.0),
        (664.0, 179.0, 676.0, 132.0),
        (630.0, 179.0, 642.0, 139.0),
        (370.0, 179.0, 382.0, 159.0),
        (248.0, 131.0, 292.0, 147.0),
    ];
    let mut svg = String::new();
    for (index, role) in roles.into_iter().enumerate() {
        let (x, y, tx, ty) = anchors[index];
        let color = if scheme.on_surface.r > 0.5 {
            "#e6e1e5"
        } else {
            "#49454f"
        };
        if index == 0 || index == 5 || index == 9 {
            let edge = if x < tx { x + 24.0 } else { x };
            annotations::dotted(
                &mut svg,
                tx * scale,
                ty * scale,
                (edge - tx) * scale,
                false,
                color,
            );
        } else {
            let edge = if y < ty { y + 32.0 } else { y };
            annotations::dotted(
                &mut svg,
                tx * scale,
                ty * scale,
                (edge - ty) * scale,
                true,
                color,
            );
        }
        boxed()
            .alignment(Alignment::Center)
            .modifier(
                Modifier::new()
                    .offset(Dp(x * scale), Dp(y * scale))
                    .size(Dp(24.0 * scale), Dp(32.0 * scale))
                    .background_with_shape(scheme.on_surface, Shape::CAPSULE)
                    .semantics(SemanticsArgs {
                        role: Some(Role::Note),
                        label: Some(format!("Color role {}: {role}", index + 1)),
                        ..Default::default()
                    }),
            )
            .children(move || {
                text()
                    .content((index + 1).to_string())
                    .style(type_style(14.0 * scale, 20.0 * scale))
                    .color(scheme.surface);
            });
    }
    for (x, y, w, h) in [
        (304.0, 107.0, 24.0, 24.0),
        (344.0, 107.0, 76.0, 24.0),
        (604.0, if outlined { 83.0 } else { 99.0 }, 76.0, 16.0),
        (604.0, if outlined { 107.0 } else { 115.0 }, 76.0, 24.0),
        (696.0, 107.0, 24.0, 24.0),
        (308.0, 151.0, 148.0, 16.0),
    ] {
        annotations::outline(
            &mut svg,
            Rect {
                x: x * scale,
                y: y * scale,
                w: w * scale,
                h: h * scale,
            },
        );
    }
    annotations::lines(Dp(1024.0 * scale), Dp(240.0 * scale), svg);
}

#[tessera]
fn field_state_matrix(outlined: bool, error: bool, dark: bool) {
    field_canvas(
        1024.0,
        600.0,
        RenderSlot::new(move || {
            let scale = 1.0;
            material_theme()
                .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                .child(move || {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    boxed()
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .height(Dp(600.0 * scale))
                                .background(scheme.surface_container_low),
                        )
                        .children(move || {
                            field_dot_grid(1024.0, 600.0, 9.0);
                            field_diagram_theme(
                                scale,
                                RenderSlot::new(move || {
                                    let columns = if error { 3 } else { 4 };
                                    for populated in [false, true] {
                                        for column in 0..columns {
                                            let x = if error {
                                                167.0 + column as f64 * 256.0
                                            } else {
                                                40.0 + column as f64 * 256.0
                                            };
                                            let y = if populated { 425.0 } else { 124.0 };
                                            let number =
                                                column + 1 + if populated { columns } else { 0 };
                                            boxed()
                                                .alignment(Alignment::Center)
                                                .modifier(
                                                    Modifier::new()
                                                        .offset(
                                                            Dp((x + 78.0) * scale),
                                                            Dp((y - 54.0) * scale),
                                                        )
                                                        .size(Dp(24.0 * scale), Dp(24.0 * scale))
                                                        .background_with_shape(
                                                            scheme.on_surface,
                                                            Shape::Ellipse,
                                                        ),
                                                )
                                                .children(move || {
                                                    text()
                                                        .content(number.to_string())
                                                        .style(type_style(
                                                            14.0 * scale,
                                                            20.0 * scale,
                                                        ))
                                                        .color(scheme.surface);
                                                });
                                            boxed()
                                                .modifier(
                                                    Modifier::new()
                                                        .offset(Dp(x * scale), Dp(y * scale)),
                                                )
                                                .children(move || {
                                                    field_state_item(
                                                        outlined,
                                                        populated,
                                                        match column {
                                                            1 => InteractionVisualState::Focus,
                                                            2 => InteractionVisualState::Hover,
                                                            _ => InteractionVisualState::Rest,
                                                        },
                                                        column != 3,
                                                        error,
                                                        true,
                                                        scale,
                                                        format!(
                                                            "{} {} {} state {}",
                                                            if dark { "Dark" } else { "Light" },
                                                            if outlined {
                                                                "outlined"
                                                            } else {
                                                                "filled"
                                                            },
                                                            if error { "error" } else { "normal" },
                                                            number
                                                        ),
                                                    );
                                                });
                                        }
                                    }
                                }),
                            );
                        });
                });
        }),
    );
}

#[tessera]
fn field_state_item(
    outlined: bool,
    populated: bool,
    presentation: InteractionVisualState,
    enabled: bool,
    error: bool,
    state_example: bool,
    scale: f64,
    name: String,
) {
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale)));
        controller.set_text(if populated { "Input text" } else { "" });
        controller
    });
    let name = remember(|| name);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().width(Dp(180.0 * scale)))
        .children(move || {
            let mut field = if outlined {
                TextFieldBuilder::outlined()
            } else {
                TextFieldBuilder::filled()
            };
            field = field
                .controller(controller)
                .label(if !enabled && populated {
                    "Label"
                } else {
                    "Label text"
                })
                .accessibility_label(name.get())
                .enabled(enabled)
                .is_error(error)
                .presentation_state(presentation)
                .min_width(Dp::ZERO)
                .min_height(Dp(56.0 * scale))
                .font_size(Dp(16.0 * scale))
                .line_height(Dp(24.0 * scale))
                .padding(Dp(16.0 * scale))
                .modifier(
                    Modifier::new()
                        .width(Dp(180.0 * scale))
                        .height(Dp(56.0 * scale)),
                )
                .leading_icon_shared(RenderSlot::new(move || {
                    icon().painter(filled::SEARCH_SVG).size(Dp(24.0 * scale));
                }));
            if error
                || populated
                || (state_example && presentation == InteractionVisualState::Focus)
            {
                field = field.trailing_icon_shared(RenderSlot::new(move || {
                    if error {
                        icon()
                            .painter(filled::ERROR_SVG)
                            .size(Dp(24.0 * scale))
                            .tint(scheme.error);
                    } else {
                        icon_button()
                            .standard()
                            .color_style(IconButtonColorStyle::Inherited)
                            .icon(filled::CANCEL_SVG)
                            .enabled(enabled)
                            .accessibility_label(format!("Clear {}", name.get()))
                            .on_click(move || {
                                controller.with(|controller| controller.request_focus());
                                controller.with_mut(|controller| controller.set_text(""));
                            });
                    }
                }));
            }
            drop(field);
            gap(Dp(4.0 * scale));
            text()
                .content("Supporting text")
                .style(type_style(12.0 * scale, 16.0 * scale))
                .color(if error {
                    scheme.error
                } else if enabled {
                    scheme.on_surface_variant
                } else {
                    scheme.on_surface.with_alpha(0.38)
                })
                .modifier(Modifier::new().padding_symmetric(Dp(16.0 * scale), Dp::ZERO));
        });
}

#[tessera]
fn field_configurations(outlined: bool) {
    field_canvas(
        1024.0,
        840.0,
        RenderSlot::new(move || {
            let scale = 1.0;
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(840.0 * scale))
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    field_diagram_theme(
                        scale,
                        RenderSlot::new(move || {
                            for row in 0..7 {
                                let y = [80.0, 171.0, 256.0, 344.0, 432.0, 517.0, 605.0][row];
                                boxed()
                                    .alignment(Alignment::Center)
                                    .modifier(
                                        Modifier::new()
                                            .offset(Dp(272.0 * scale), Dp((y + 16.0) * scale))
                                            .size(Dp(24.0 * scale), Dp(24.0 * scale))
                                            .border_with_shape(
                                                Dp(scale),
                                                scheme.on_surface_variant,
                                                Shape::Ellipse,
                                            )
                                            .semantics(SemanticsArgs {
                                                role: Some(Role::Note),
                                                label: Some(format!("Configuration {}", row + 1)),
                                                ..Default::default()
                                            }),
                                    )
                                    .children(move || {
                                        text()
                                            .content((row + 1).to_string())
                                            .style(type_style(14.0 * scale, 20.0 * scale))
                                            .color(scheme.on_surface_variant);
                                    });
                                for populated in [false, true] {
                                    boxed()
                                        .modifier(Modifier::new().offset(
                                            Dp(if populated { 524.0 } else { 320.0 } * scale),
                                            Dp(y * scale),
                                        ))
                                        .children(move || {
                                            field_configuration_item(
                                                outlined, row, populated, scale,
                                            );
                                        });
                                }
                            }
                        }),
                    );
                });
        }),
    );
}

#[tessera]
fn field_configuration_item(outlined: bool, row: usize, populated: bool, scale: f64) {
    let initial = if !populated {
        ""
    } else {
        match row {
            4 => "1.43",
            5 => "25",
            6 => {
                "This is a long input in a multi-line text field that wraps overflow text onto a new line"
            }
            _ => "Input",
        }
    };
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0 * scale), Some(Dp(24.0 * scale)));
        controller.set_text(initial);
        controller
    });
    let value = remember(|| initial.to_owned());
    let name = format!(
        "{} configuration {} {}",
        if outlined { "Outlined" } else { "Filled" },
        row + 1,
        if populated { "populated" } else { "empty" }
    );
    let name = remember(|| name);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().width(Dp(180.0 * scale)))
        .children(move || {
            let mut field = if outlined {
                TextFieldBuilder::outlined()
            } else {
                TextFieldBuilder::filled()
            };
            field = field
                .controller(controller)
                .label("Label")
                .accessibility_label(name.get())
                .presentation_state(if populated {
                    InteractionVisualState::Focus
                } else {
                    InteractionVisualState::Rest
                })
                .min_width(Dp::ZERO)
                .min_height(Dp(56.0 * scale))
                .font_size(Dp(16.0 * scale))
                .line_height(Dp(24.0 * scale))
                .padding(Dp(16.0 * scale))
                .modifier(
                    Modifier::new()
                        .width(Dp(180.0 * scale))
                        .height(Dp(if row == 6 && value.with(|value| !value.is_empty()) {
                            if outlined { 136.0 } else { 156.0 }
                        } else {
                            56.0
                        } * scale)),
                )
                .line_limit(if row == 6 {
                    TextFieldLineLimit::MultiLine
                } else {
                    TextFieldLineLimit::SingleLine
                })
                .on_change(move |next: String| {
                    value.set(next.clone());
                    next
                });
            if row == 2 || row == 3 {
                field = field.leading_icon_shared(RenderSlot::new(move || {
                    icon().painter(filled::SEARCH_SVG).size(Dp(24.0 * scale));
                }));
            }
            if (row == 1 || row == 3) && value.with(|value| !value.is_empty()) {
                field = field.trailing_icon_shared(RenderSlot::new(move || {
                    icon_button()
                        .standard()
                        .color_style(IconButtonColorStyle::Inherited)
                        .icon(filled::CANCEL_SVG)
                        .accessibility_label(format!("Clear {}", name.get()))
                        .on_click(move || {
                            controller.with(|controller| controller.request_focus());
                            controller.with_mut(|controller| controller.set_text(""));
                            value.set(String::new());
                        });
                }));
            }
            if row == 4 {
                field = field.prefix_shared(RenderSlot::new(|| {
                    text().content("$");
                }));
            }
            if row == 5 {
                field = field.suffix_shared(RenderSlot::new(|| {
                    text().content("lbs");
                }));
            }
            drop(field);
            if row == 0 {
                gap(Dp(4.0 * scale));
                text()
                    .content("Supporting text")
                    .style(type_style(12.0 * scale, 16.0 * scale))
                    .color(scheme.on_surface_variant)
                    .modifier(Modifier::new().padding_symmetric(Dp(16.0 * scale), Dp::ZERO));
            }
        });
}

fn measurement_name(outlined: bool, variant: usize, populated: bool) -> String {
    format!(
        "{} measurement {} {}",
        if outlined { "Outlined" } else { "Filled" },
        ["without icons", "with icons", "support and counter"][variant],
        if populated { "populated" } else { "empty" }
    )
}

#[derive(Clone, Copy, Default, PartialEq)]
struct MeasurementDisplay {
    nonempty: [bool; 2],
    focused: [bool; 2],
}

#[tessera]
fn field_measurements(outlined: bool, variant: usize) {
    field_canvas(
        1024.0,
        300.0,
        RenderSlot::new(move || {
            let geometry = remember(SpaceGeometry::new);
            let display = remember(|| MeasurementDisplay {
                nonempty: [false, variant != 2],
                focused: [false; 2],
            });
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch_in_space(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(300.0))
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    field_dot_grid(1024.0, 300.0, 9.0);
                    row()
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .padding_symmetric(Dp(48.0), Dp::ZERO)
                                .offset(Dp::ZERO, Dp(if variant == 2 { -22.0 } else { 0.0 })),
                        )
                        .children(move || {
                            field_measurement_item(outlined, variant, false, geometry, display);
                            spacer().modifier(Modifier::new().width(Dp(120.0)));
                            field_measurement_item(outlined, variant, true, geometry, display);
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            field_measurement_annotations(geometry, outlined, variant, display);
                        });
                });
        }),
    );
}

#[tessera]
fn field_measurement_item(
    outlined: bool,
    variant: usize,
    populated: bool,
    geometry: State<SpaceGeometry>,
    display: State<MeasurementDisplay>,
) {
    let initial = if populated && variant != 2 {
        "Input text"
    } else {
        ""
    };
    let value = remember(|| initial.to_owned());
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        controller.set_text(initial);
        controller
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().width(Dp(240.0)))
        .children(move || {
            boxed()
                .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!(
                        "{} container",
                        measurement_name(outlined, variant, populated)
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .children(move || {
                    let field = if outlined {
                        TextFieldBuilder::outlined()
                    } else {
                        TextFieldBuilder::filled()
                    };
                    let mut field = field
                        .modifier(
                            watch_in_space(
                                geometry,
                                if populated {
                                    "populated-field"
                                } else {
                                    "empty-field"
                                },
                            )
                            .fill_max_width()
                            .focus_group()
                            .on_focus_changed(
                                move |focus: FocusState| {
                                    if display.is_alive()
                                        && display
                                            .with(|value| value.focused[usize::from(populated)])
                                            != focus.has_focus()
                                    {
                                        display.with_mut(|value| {
                                            value.focused[usize::from(populated)] =
                                                focus.has_focus()
                                        });
                                    }
                                },
                            ),
                        )
                        .min_width(Dp::ZERO)
                        .min_height(Dp(56.0))
                        .controller(controller)
                        .label("Label text")
                        .presentation_state_optional(if populated {
                            Some(InteractionVisualState::Focus)
                        } else {
                            None
                        })
                        .accessibility_label(measurement_name(outlined, variant, populated))
                        .on_change(move |next: String| {
                            let nonempty = !next.is_empty();
                            value.set(next.clone());
                            if display.with(|value| value.nonempty[usize::from(populated)])
                                != nonempty
                            {
                                display.with_mut(|value| {
                                    value.nonempty[usize::from(populated)] = nonempty
                                });
                            }
                            next
                        });
                    if variant == 1 {
                        field = field.leading_icon_shared(RenderSlot::new(move || {
                            boxed()
                                .modifier(
                                    watch_in_space(
                                        geometry,
                                        if populated {
                                            "populated-leading"
                                        } else {
                                            "empty-leading"
                                        },
                                    )
                                    .semantics(SemanticsArgs {
                                        role: Some(Role::Image),
                                        label: Some(format!(
                                            "{} leading icon",
                                            measurement_name(outlined, variant, populated)
                                        )),
                                        ..Default::default()
                                    }),
                                )
                                .children(|| {
                                    icon().painter(filled::SEARCH_SVG).size(Dp(24.0));
                                });
                        }));
                        if value.with(|value| !value.is_empty()) {
                            field = field.trailing_icon_shared(RenderSlot::new(move || {
                                icon_button()
                                    .standard()
                                    .icon(filled::CANCEL_SVG)
                                    .accessibility_label(format!(
                                        "Clear {}",
                                        measurement_name(outlined, variant, populated)
                                    ))
                                    .on_click(move || {
                                        controller.with(|controller| controller.request_focus());
                                        controller.with_mut(|controller| controller.set_text(""));
                                        value.set(String::new());
                                        display.with_mut(|value| {
                                            value.nonempty[usize::from(populated)] = false
                                        });
                                    });
                            }));
                        }
                    }
                    drop(field);
                });
            if variant == 2 {
                gap(Dp(4.0));
                row()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .padding_symmetric(Dp(16.0), Dp::ZERO),
                    )
                    .children(move || {
                        text()
                            .content("Supporting text")
                            .style(type_style(12.0, 16.0))
                            .color(scheme.on_surface_variant)
                            .modifier(
                                watch_in_space(
                                    geometry,
                                    if populated {
                                        "populated-support"
                                    } else {
                                        "empty-support"
                                    },
                                )
                                .weight(1.0)
                                .semantics(SemanticsArgs {
                                    role: Some(Role::Label),
                                    label: Some(format!(
                                        "{} supporting text",
                                        measurement_name(outlined, variant, populated)
                                    )),
                                    ..Default::default()
                                }),
                            );
                        if populated {
                            spacer()
                                .modifier(watch_in_space(geometry, "counter-gap").width(Dp(16.0)));
                            text()
                                .content(format!(
                                    "{}/20",
                                    value.with(|value| value.chars().count())
                                ))
                                .style(type_style(12.0, 16.0))
                                .color(scheme.on_surface_variant)
                                .modifier(watch_in_space(geometry, "counter").semantics(
                                    SemanticsArgs {
                                        role: Some(Role::Label),
                                        label: Some(format!(
                                            "Measurement character count: {} of 20",
                                            value.with(|value| value.chars().count())
                                        )),
                                        ..Default::default()
                                    },
                                ));
                        }
                    });
            }
        });
}

fn measurement_band(svg: &mut String, rect: Rect) {
    svg.push_str(&format!(
        "<rect x='{}' y='{}' width='{}' height='{}' fill='#ee7da4' fill-opacity='.35'/>",
        rect.x,
        rect.y,
        rect.w.max(0.0),
        rect.h.max(0.0)
    ));
}

#[tessera]
fn measurement_axis(
    x: f64,
    y: f64,
    length: f64,
    vertical: bool,
    label: String,
    after: Option<bool>,
) {
    let color = srgb(232, 121, 152);
    if vertical {
        spacer().modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(0.7), Dp(length.max(0.0)))
                .background(color),
        );
        for endpoint in [y, y + length] {
            spacer().modifier(
                Modifier::new()
                    .offset(Dp(x - 3.0), Dp(endpoint))
                    .size(Dp(6.0), Dp(0.7))
                    .background(color),
            );
        }
        annotations::dimension(
            label,
            Dp(if after.unwrap_or(false) {
                x + 8.0
            } else {
                x - 30.0
            }),
            Dp(y + length / 2.0 - 10.0),
        );
    } else {
        spacer().modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(length.max(0.0)), Dp(0.7))
                .background(color),
        );
        for endpoint in [x, x + length] {
            spacer().modifier(
                Modifier::new()
                    .offset(Dp(endpoint), Dp(y - 3.0))
                    .size(Dp(0.7), Dp(6.0))
                    .background(color),
            );
        }
        annotations::dimension(
            label,
            Dp(x + length / 2.0 - 8.0),
            Dp(if after.unwrap_or(false) {
                y + 8.0
            } else {
                y - 28.0
            }),
        );
    }
}

#[tessera]
fn field_measurement_annotations(
    geometry: State<SpaceGeometry>,
    outlined: bool,
    variant: usize,
    display: State<MeasurementDisplay>,
) {
    field_measurement_annotation_geometry(
        geometry.with(annotations::space_geometry),
        outlined,
        variant,
        display.get(),
    );
}

#[tessera]
fn field_measurement_annotation_geometry(
    values: std::collections::BTreeMap<&'static str, Rect>,
    outlined: bool,
    variant: usize,
    display: MeasurementDisplay,
) {
    let (Some(root), Some(empty), Some(pop)) = (
        values.get("figure"),
        values.get("empty-field").copied(),
        values.get("populated-field").copied(),
    ) else {
        return;
    };
    let mut svg = String::new();
    if variant == 2 {
        for (field, key) in [(empty, "empty-support"), (pop, "populated-support")] {
            let Some(support) = values.get(key).copied() else {
                continue;
            };
            measurement_band(
                &mut svg,
                Rect {
                    x: field.x,
                    y: field.y + field.h,
                    w: field.w,
                    h: 4.0,
                },
            );
            measurement_band(
                &mut svg,
                Rect {
                    x: field.x,
                    y: support.y,
                    w: 16.0,
                    h: support.h,
                },
            );
            measurement_band(
                &mut svg,
                Rect {
                    x: field.x + field.w - 16.0,
                    y: support.y,
                    w: 16.0,
                    h: support.h,
                },
            );
            annotations::outline(&mut svg, support);
            measurement_axis(
                field.x + field.w + 12.0,
                field.y + field.h,
                support.y - field.y - field.h,
                true,
                "4".into(),
            )
            .after(true);
            measurement_axis(
                field.x,
                support.y + support.h + 12.0,
                16.0,
                false,
                "16".into(),
            )
            .after(true);
            measurement_axis(
                field.x + field.w - 16.0,
                support.y + support.h + 12.0,
                16.0,
                false,
                "16".into(),
            )
            .after(true);
        }
        if let Some(counter) = values.get("counter").copied() {
            annotations::outline(&mut svg, counter);
            let gap = Rect {
                x: counter.x - 16.0,
                y: counter.y,
                w: 16.0,
                h: counter.h,
            };
            measurement_band(&mut svg, gap);
            measurement_axis(gap.x, gap.y + gap.h + 12.0, 16.0, false, "16".into()).after(true);
        }
    } else {
        measurement_axis(
            empty.x - 12.0,
            empty.y,
            empty.h,
            true,
            format!("{:.0}", empty.h),
        );
        for (field, populated) in [(empty, false), (pop, true)] {
            field_measurement_native_annotations(
                values.clone(),
                outlined,
                variant,
                populated,
                display,
            );
            let nonempty = display.nonempty[usize::from(populated)];
            let floating = nonempty || display.focused[usize::from(populated)];

            let edge = if variant == 1 { 12.0 } else { 16.0 };
            measurement_band(
                &mut svg,
                Rect {
                    x: field.x,
                    y: field.y,
                    w: edge,
                    h: field.h,
                },
            );
            measurement_band(
                &mut svg,
                Rect {
                    x: field.x + field.w
                        - if variant == 1 && !nonempty {
                            16.0
                        } else {
                            edge
                        },
                    y: field.y,
                    w: if variant == 1 && !nonempty {
                        16.0
                    } else {
                        edge
                    },
                    h: field.h,
                },
            );

            if variant == 1 {
                let icon = values
                    .get(if populated {
                        "populated-leading"
                    } else {
                        "empty-leading"
                    })
                    .copied()
                    .unwrap_or(Rect {
                        x: field.x + 12.0,
                        y: field.y + 16.0,
                        w: 24.0,
                        h: 24.0,
                    });
                annotations::outline(&mut svg, icon);
                measurement_band(
                    &mut svg,
                    Rect {
                        x: icon.x + icon.w,
                        y: field.y,
                        w: 16.0,
                        h: field.h,
                    },
                );

                if nonempty {
                    let trailing = Rect {
                        x: field.x + field.w - 36.0,
                        y: field.y + (field.h - 24.0) / 2.0,
                        w: 24.0,
                        h: 24.0,
                    };
                    annotations::outline(&mut svg, trailing);
                    measurement_band(
                        &mut svg,
                        Rect {
                            x: trailing.x - 16.0,
                            y: field.y,
                            w: 16.0,
                            h: field.h,
                        },
                    );
                }
            }
            if floating && !outlined {
                for y in [field.y, field.y + field.h - 8.0] {
                    measurement_band(
                        &mut svg,
                        Rect {
                            x: field.x,
                            y,
                            w: field.w,
                            h: 8.0,
                        },
                    );
                }
            }
            let inset = if variant == 1 { 52.0 } else { 16.0 };
            annotations::outline(
                &mut svg,
                Rect {
                    x: field.x + inset,
                    y: field.y + if floating && !outlined { 24.0 } else { 16.0 },
                    w: field.w - inset - if variant == 1 && nonempty { 52.0 } else { 16.0 },
                    h: 24.0,
                },
            );
        }
    }
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn field_measurement_native_annotations(
    values: std::collections::BTreeMap<&'static str, Rect>,
    outlined: bool,
    variant: usize,
    populated: bool,
    display: MeasurementDisplay,
) {
    let Some(field) = values
        .get(if populated {
            "populated-field"
        } else {
            "empty-field"
        })
        .copied()
    else {
        return;
    };
    let nonempty = display.nonempty[usize::from(populated)];
    let floating = nonempty || display.focused[usize::from(populated)];
    measurement_status_note(
        variant == 1 && nonempty,
        field.x + field.w - 36.0,
        field.y + (field.h - 24.0) / 2.0,
        24.0,
        24.0,
        format!(
            "{} trailing icon annotation",
            measurement_name(outlined, variant, populated)
        ),
    );
    measurement_status_note(
        floating && !outlined,
        field.x,
        field.y,
        field.w,
        8.0,
        format!(
            "{} floating inset annotation",
            measurement_name(outlined, variant, populated)
        ),
    );
    let edge = if variant == 1 { 12.0 } else { 16.0 };
    let right = if variant == 1 && !nonempty {
        16.0
    } else {
        edge
    };
    measurement_axis(field.x, field.y - 6.0, edge, false, format!("{edge:.0}"));
    measurement_axis(
        field.x + field.w - right,
        field.y - 6.0,
        right,
        false,
        format!("{right:.0}"),
    );
    if variant == 1 {
        let icon = values
            .get(if populated {
                "populated-leading"
            } else {
                "empty-leading"
            })
            .copied()
            .unwrap_or(Rect {
                x: field.x + 12.0,
                y: field.y + 16.0,
                w: 24.0,
                h: 24.0,
            });
        measurement_axis(icon.x + icon.w, field.y - 6.0, 16.0, false, "16".into());
        annotations::dimension("24".into(), Dp(icon.x + 4.0), Dp(field.y - 62.0));
        if nonempty {
            let x = field.x + field.w - 36.0;
            measurement_axis(x - 16.0, field.y - 6.0, 16.0, false, "16".into());
            annotations::dimension("24".into(), Dp(x + 4.0), Dp(field.y - 62.0));
        }
    }
    if floating && !outlined {
        for y in [field.y, field.y + field.h - 8.0] {
            measurement_axis(field.x - 12.0, y, 8.0, true, "8".into());
        }
    }
}

#[tessera]
fn measurement_status_note(enabled: bool, x: f64, y: f64, width: f64, height: f64, label: String) {
    if enabled {
        spacer().modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(width), Dp(height))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(label),
                    ..Default::default()
                }),
        );
    }
}

#[tessera]
pub(super) fn field_canvas(canonical_width: f64, canonical_height: f64, child: RenderSlot) {
    let available_width = remember(move || Dp(canonical_width));
    let scale = (available_width.get().0 / canonical_width) as f32;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if next != available_width.get() {
                    available_width.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box(scale).child(move || {
                boxed()
                    .modifier(
                        Modifier::new().required_size(Dp(canonical_width), Dp(canonical_height)),
                    )
                    .children_shared(child);
            });
        });
}

fn field_svg_color(color: Color) -> String {
    let channel = |value: f32| {
        let encoded = if value <= 0.0031308 {
            value * 12.92
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        };
        (encoded.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        channel(color.r),
        channel(color.g),
        channel(color.b)
    )
}

#[tessera]
fn field_dot_grid(width: f64, height: f64, origin_y: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let mut path = String::new();
    for row in 0..((height - origin_y) / 8.0).ceil().max(0.0) as usize {
        for column in 0..((width - 10.0) / 8.0).ceil().max(0.0) as usize {
            write!(
                &mut path,
                "M{} {}h1v1h-1z",
                10 + column * 8,
                origin_y + (row * 8) as f64
            )
            .expect("writing a vector background");
        }
    }
    annotations::lines(
        Dp(width),
        Dp(height),
        format!(
            "<path d='{path}' fill='{}' fill-opacity='.5'/>",
            field_svg_color(scheme.on_surface_variant)
        ),
    );
}

#[tessera]
fn field_anatomy_marker(number: usize, x: Dp, y: Dp) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0), Dp(32.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!(
                        "{}Anatomy marker {number}",
                        if scheme.is_dark { "Dark " } else { "" }
                    )),
                    merge_descendants: true,
                    ..Default::default()
                })
                .background_with_shape(scheme.surface_container_low, Shape::CAPSULE)
                .border_with_shape(Dp(1.0), scheme.on_surface, Shape::CAPSULE),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.on_surface);
        });
}

fn field_annotation_outline(svg: &mut String, rect: Rect, ink: &str) {
    annotations::dotted(svg, rect.x, rect.y, rect.w, false, ink);
    annotations::dotted(svg, rect.x, rect.y + rect.h, rect.w, false, ink);
    annotations::dotted(svg, rect.x, rect.y, rect.h, true, ink);
    annotations::dotted(svg, rect.x + rect.w, rect.y, rect.h, true, ink);
}
