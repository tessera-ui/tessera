//! Interactive Expressive list specifications.
//!
//! ## Usage
//!
//! Compare content rows, independent actions, selection modes and grouped
//! shapes.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    divider::horizontal_divider,
    flow_row::flow_row,
    icon::icon,
    icon_button::icon_button,
    list_item::{ListItemPosition, ListItemSelectionMode, list_item},
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const ITEMS: [&str; 3] = ["Notes", "Projects", "Archive"];

#[derive(Clone, Copy, PartialEq)]
struct DemoState {
    single: State<usize>,
    checked: State<[bool; 3]>,
    primary: State<u32>,
    secondary: State<u32>,
}

#[tessera]
pub(super) fn lists_spec() {
    article(
        "Lists",
        "Present a continuous vertical index of related content and actions.",
        vec![
            SpecSection {
                title: "Variants",
                content: RenderSlot::new(variants),
            },
            SpecSection {
                title: "Configurations",
                content: RenderSlot::new(configurations),
            },
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Anatomy",
                content: RenderSlot::new(anatomy),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "States",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    ).pages(vec![
        ArticlePage {title:"Overview",intro:Some(RenderSlot::new(list_reading_examples)),sections:vec![
            SpecSection {title:"M3 Expressive update",content:RenderSlot::new(|| {prose("Expressive lists introduce segmented shapes, highlighted selection and flexible content slots. Use this treatment for new designs.");gap(Dp(24.0));panel(RenderSlot::new(|| {mode_demo(ListItemSelectionMode::Multiple,false,true,true,"Overview selection");}));})},
            SpecSection {title:"Differences from M2 to M3 baseline",content:RenderSlot::new(|| {prose("Baseline lists standardize color, spacing, heights and alignment. Content determines a one-, two- or three-line item's height; taller rows align their contents toward the top.");gap(Dp(24.0));list_reading_examples();})},
        ]},
        ArticlePage {title:"Guidelines",intro:None,sections:vec![
            SpecSection {title:"Usage",content:RenderSlot::new(|| {prose("Order related items logically and use consistent formatting so people can find and act on a particular item. Keep labels concise and supporting text easy to scan.");gap(Dp(24.0));list_reading_examples();})},
            SpecSection {title:"Anatomy",content:RenderSlot::new(list_guideline_anatomy)},
            SpecSection {title:"Adaptive design",content:RenderSlot::new(list_adaptive)},
            SpecSection {title:"Behavior",content:RenderSlot::new(list_behavior)},
        ]},
        ArticlePage {title:"Accessibility",intro:None,sections:vec![
            SpecSection {title:"Use cases",content:RenderSlot::new(|| {prose("People must be able to reach list items, identify their purpose and activate or select them with assistive technology.");gap(Dp(24.0));panel(RenderSlot::new(|| {mode_demo(ListItemSelectionMode::Action,false,true,true,"Accessible actions");}));})},
            SpecSection {title:"Indicate selection with more than color",content:RenderSlot::new(|| {prose("Add a checkmark or selection control so selected items remain identifiable without relying on color.");gap(Dp(24.0));panel(RenderSlot::new(|| {mode_demo(ListItemSelectionMode::Multiple,false,true,true,"Accessible multiple selection");}));})},
            SpecSection {title:"Interaction & style",content:RenderSlot::new(list_interaction)},
            SpecSection {title:"Focus",content:RenderSlot::new(list_focus)},
            SpecSection {title:"Keyboard navigation",content:RenderSlot::new(|| {spec_table(vec!["Keys".into(),"Action".into()],vec![vec!["Tab / Shift + Tab".into(),"Navigate between enabled row actions".into()],vec!["Space or Enter".into(),"Activate or select the focused action".into()]]);})},
            SpecSection {title:"Labeling elements",content:RenderSlot::new(|| {prose("The item's visible label and supporting text describe its purpose. Give secondary actions their own meaningful names.");gap(Dp(24.0));subheading("Platform-specific labels".into());prose("Single and multiple selection expose distinct roles and checked states. Action rows do not pretend to be selected. Decorative icons do not add redundant announcements.");gap(Dp(24.0));panel(RenderSlot::new(|| {mode_demo(ListItemSelectionMode::Single,false,true,true,"Labeled single selection");}));})},
        ]},
    ]);
}

#[tessera]
fn mode_demo(
    mode: ListItemSelectionMode,
    multi_action: bool,
    segmented: bool,
    enabled: bool,
    #[prop(into)] label: String,
) {
    let state = DemoState {
        single: remember(|| 0),
        checked: remember(|| [false; 3]),
        primary: remember(|| 0),
        secondary: remember(|| 0),
    };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let label = label.clone();
            let caption = label.clone();
            column()
                .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                    role: Some(if mode == ListItemSelectionMode::Single {
                        Role::RadioGroup
                    } else {
                        Role::List
                    }),
                    label: Some(label.clone()),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .children(move || {
                    for index in 0..ITEMS.len() {
                        item(
                            index,
                            mode,
                            multi_action,
                            segmented,
                            enabled,
                            label.clone(),
                            state,
                        );
                        if index < ITEMS.len() - 1 {
                            gap(Dp(2.0));
                        }
                    }
                });
            if mode == ListItemSelectionMode::Action {
                gap(Dp(16.0));
                text()
                    .content(format!(
                        "{caption} primary actions: {}",
                        state.primary.get()
                    ))
                    .style(type_style(14.0, 20.0));
                if multi_action {
                    text()
                        .content(format!(
                            "{caption} secondary actions: {}",
                            state.secondary.get()
                        ))
                        .style(type_style(14.0, 20.0));
                }
            }
        });
}

#[tessera]
fn item(
    index: usize,
    mode: ListItemSelectionMode,
    multi_action: bool,
    segmented: bool,
    enabled: bool,
    #[prop(into)] label: String,
    state: DemoState,
) {
    let selected = match mode {
        ListItemSelectionMode::Single => state.single.get() == index,
        ListItemSelectionMode::Multiple => state.checked.get()[index],
        ListItemSelectionMode::Action => false,
    };
    let mut control = list_item()
        .headline(ITEMS[index])
        .supporting_text("Folder and related documents")
        .accessibility_label(format!("{label}: {}", ITEMS[index]))
        .enabled(enabled)
        .selection_mode(mode)
        .selected_optional((mode != ListItemSelectionMode::Action).then_some(selected))
        .position(if segmented {
            match index {
                0 => ListItemPosition::First,
                2 => ListItemPosition::Last,
                _ => ListItemPosition::Middle,
            }
        } else {
            ListItemPosition::Standalone
        })
        .on_click(move || state.primary.set(state.primary.get() + 1))
        .on_selection_change(move |value| {
            if mode == ListItemSelectionMode::Single {
                state.single.set(index);
            } else {
                state.checked.with_mut(|checked| checked[index] = value);
            }
        })
        .leading(|| {
            icon().painter(filled::FOLDER_SVG);
        });
    if multi_action {
        control = control.trailing(move || {
            icon_button()
                .standard()
                .icon(filled::MORE_VERT_SVG)
                .enabled(enabled)
                .accessibility_label(format!("{label}: {} more actions", ITEMS[index]))
                .on_click(move || state.secondary.set(state.secondary.get() + 1));
        });
    } else if selected {
        control = control.trailing(|| {
            icon().painter(filled::CHECK_SVG);
        });
    }
    drop(control);
}

#[tessera]
fn variants() {
    text()
        .content("Expressive lists add grouped corners, selection treatment and flexible slots.")
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Multiple,
            false,
            true,
            true,
            "Expressive list",
        );
    }));
}

#[tessera]
fn expansion() {
    let expanded = remember(|| false);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            list_item()
                .headline(if expanded.get() {
                    "Hide related folders"
                } else {
                    "Show related folders"
                })
                .accessibility_label("Expand related folders")
                .on_click(move || expanded.set(!expanded.get()))
                .trailing(move || {
                    icon().painter(if expanded.get() {
                        filled::CHECK_SVG
                    } else {
                        filled::ADD_SVG
                    });
                });
            if expanded.get() {
                list_item()
                    .headline("Archive")
                    .supporting_text("Completed projects and previous notes")
                    .accessibility_label("Expanded archive folder")
                    .leading(|| {
                        icon().painter(filled::FOLDER_SVG);
                    });
            }
        });
}

#[tessera]
fn configurations() {
    subheading("Styles".into());
    gap(Dp(24.0));
    for segmented in [false, true] {
        style_configuration(segmented);
    }
    subheading("List selection".into());
    gap(Dp(24.0));
    for (mode, multi_action, label) in [
        (ListItemSelectionMode::Action, false, "Single action"),
        (ListItemSelectionMode::Action, true, "Multi action"),
        (ListItemSelectionMode::Single, false, "Single select"),
        (ListItemSelectionMode::Multiple, false, "Multi select"),
    ] {
        selection_configuration(mode, multi_action, label.to_owned());
    }
    subheading("Expand".into());
    gap(Dp(24.0));
    panel(RenderSlot::new(expansion));
}

#[tessera]
fn style_configuration(segmented: bool) {
    subheading(if segmented { "Segmented" } else { "Standard" }.to_owned());
    gap(Dp(16.0));
    panel(RenderSlot::new(move || {
        mode_demo(
            ListItemSelectionMode::Action,
            false,
            segmented,
            true,
            if segmented {
                "Segmented list"
            } else {
                "Standard list"
            },
        );
    }));
    gap(Dp(24.0));
}

#[tessera]
fn selection_configuration(mode: ListItemSelectionMode, multi_action: bool, label: String) {
    subheading(label.clone());
    gap(Dp(16.0));
    panel(RenderSlot::new(move || {
        mode_demo(mode, multi_action, true, true, label.clone());
    }));
    gap(Dp(24.0));
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Container".into(), "Surface".into()],
            vec!["Label".into(), "On surface".into()],
            vec![
                "Overline / supporting / trailing content".into(),
                "On surface variant".into(),
            ],
            vec!["Divider".into(), "Outline variant".into()],
            vec!["Inner / outer corners".into(), "4 / 16dp".into()],
            vec!["Selected corners".into(), "16dp".into()],
            vec!["Leading icon".into(), "20dp".into()],
        ],
    );
}

#[tessera]
fn anatomy() {
    list_anatomy_diagram();
    gap(Dp(24.0));
    let secondary = remember(|| 0_u32);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                list_item()
                    .headline("Quarterly summary")
                    .overline_text("Documents")
                    .supporting_text("Review the latest notes and decisions.")
                    .leading(|| {
                        icon().painter(filled::FOLDER_SVG);
                    })
                    .trailing(move || {
                        icon_button()
                            .standard()
                            .icon(filled::MORE_VERT_SVG)
                            .accessibility_label("Summary more actions")
                            .on_click(move || secondary.set(secondary.get() + 1));
                    });
                gap(Dp(16.0));
                text().content(format!("Summary actions: {}", secondary.get()));
            });
    }));
    gap(Dp(24.0));
    text().content("1  Container    2  Overline    3  Label    4  Trailing content    5  Supporting text    6  Leading icon")
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    text().content("Content occupies the largest slot. Selection rows expose one selection interaction; action rows may offer separate secondary actions.")
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn scheme(dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            let theme = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get();
            surface()
                .style(theme.color_scheme.surface.into())
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                        .children(move || {
                            subheading(if dark { "Dark theme" } else { "Light theme" }.to_owned());
                            gap(Dp(24.0));
                            mode_demo(
                                ListItemSelectionMode::Single,
                                false,
                                true,
                                true,
                                if dark { "Dark list" } else { "Light list" },
                            );
                        });
                });
        });
}

#[tessera]
fn colors() {
    scheme(false);
    gap(Dp(24.0));
    scheme(true);
}

#[tessera]
fn states() {
    text()
        .content("Hover, focus with Tab, press and hold, or select a row to inspect real feedback.")
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    for enabled in [true, false] {
        state_configuration(enabled);
    }
}

#[tessera]
fn state_configuration(enabled: bool) {
    subheading(if enabled { "Enabled" } else { "Disabled" }.to_owned());
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        mode_demo(
            ListItemSelectionMode::Single,
            false,
            true,
            enabled,
            if enabled {
                "Enabled list"
            } else {
                "Disabled list"
            },
        );
    }));
    gap(Dp(24.0));
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                list_item()
                    .headline("One line")
                    .accessibility_label("One-line list measurement");
                gap(Dp(2.0));
                list_item()
                    .headline("Two lines")
                    .supporting_text("Supporting text")
                    .accessibility_label("Two-line list measurement");
                gap(Dp(2.0));
                list_item()
                    .headline("Three lines")
                    .supporting_text("Supporting text\nSecond supporting line")
                    .accessibility_label("Three-line list measurement");
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec![
                "One / two / three-line minimum height".into(),
                "56 / 72 / 88dp".into(),
            ],
            vec!["Horizontal content padding".into(), "16dp".into()],
            vec!["Vertical content padding".into(), "10dp".into()],
            vec!["Between segmented rows".into(), "2dp".into()],
            vec![
                "Between leading / content / trailing slots".into(),
                "12dp".into(),
            ],
            vec!["Leading icon".into(), "20dp".into()],
            vec!["Unselected inner / outer corners".into(), "4 / 16dp".into()],
            vec!["Selected corner radius".into(), "16dp".into()],
            vec!["Minimum interactive target".into(), "48 × 48dp".into()],
        ],
    );
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn list_reading_examples() {
    panel(RenderSlot::new(|| {
        column().modifier(Modifier::new().fill_max_width()).children(|| {
        list_item().headline("Label text").leading(|| {icon().vector(filled::FOLDER_SVG);});
        gap(Dp(8.0));
        list_item().headline("Label text").supporting_text("Supporting text").leading(|| {icon().vector(filled::FOLDER_SVG);});
        gap(Dp(8.0));
        list_item().headline("Label text").supporting_text("Supporting text that wraps naturally onto another line when the available width is compact.")
            .leading(|| {icon().vector(filled::FOLDER_SVG);});
    });
    }));
}

#[tessera]
fn list_anatomy_diagram() {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(456.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().width(Dp(360.0)))
                .children(move || {
                    list_item()
                        .modifier(watch(geometry, "container"))
                        .min_height(Dp(84.0))
                        .leading(move || {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    watch(geometry, "media")
                                        .size(Dp(114.0), Dp(64.0))
                                        .background(scheme.surface_container_highest),
                                )
                                .children(move || {
                                    icon()
                                        .vector(filled::CATEGORY_SVG)
                                        .size(Dp(40.0))
                                        .tint(scheme.outline_variant);
                                });
                        })
                        .content(move || {
                            column().children(move || {
                                text()
                                    .content("Overline")
                                    .style(type_style(12.0, 16.0))
                                    .modifier(watch(geometry, "overline"));
                                text()
                                    .content("Label text")
                                    .style(type_style(16.0, 24.0))
                                    .modifier(watch(geometry, "label"));
                            });
                        })
                        .trailing(move || {
                            text()
                                .content("100+")
                                .style(type_style(14.0, 20.0))
                                .modifier(watch(geometry, "trailing-text"));
                        });
                    gap(Dp(16.0));
                    list_item()
                        .headline("Label text")
                        .min_height(Dp(64.0))
                        .leading(move || {
                            boxed().modifier(watch(geometry, "icon")).children(|| {
                                icon().vector(filled::PERSON_OUTLINE_SVG).size(Dp(20.0));
                            });
                        })
                        .supporting_content(move || {
                            text()
                                .content("Supporting text")
                                .style(type_style(14.0, 20.0))
                                .modifier(watch(geometry, "support"));
                        });
                    gap(Dp(16.0));
                    list_item()
                        .headline("Label text")
                        .supporting_text("Supporting text")
                        .min_height(Dp(64.0))
                        .leading(move || {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    watch(geometry, "avatar")
                                        .size(Dp(40.0), Dp(40.0))
                                        .background_with_shape(
                                            scheme.primary_container,
                                            Shape::Ellipse,
                                        ),
                                )
                                .children(move || {
                                    text().content("A").color(scheme.on_primary_container);
                                });
                        })
                        .trailing(move || {
                            boxed()
                                .modifier(watch(geometry, "trailing-icon"))
                                .children(|| {
                                    icon().vector(filled::ARROW_RIGHT_SVG).size(Dp(20.0));
                                });
                        });
                    horizontal_divider().modifier(watch(geometry, "divider"));
                });
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    list_callouts(geometry);
                });
        });
}

#[tessera]
fn list_callouts(geometry: State<Geometry>) {
    let values = geometry.get();
    let Some(root) = values.get("figure") else {
        return;
    };
    let Some(container) = relative(&values, "container") else {
        return;
    };
    let mut svg = String::new();
    for (number, name, side) in [
        (1, "container", 0),
        (2, "overline", 0),
        (3, "label", 0),
        (4, "trailing-text", 1),
        (5, "support", 1),
        (6, "trailing-icon", 1),
        (7, "divider", 2),
        (8, "avatar", 2),
        (9, "icon", 3),
        (10, "media", 3),
    ] {
        let Some(target) = relative(&values, name) else {
            continue;
        };
        let (x, y, tx, ty) = match side {
            0 => (
                target.x + target.w / 2.0 - 12.0,
                container.y
                    - 64.0
                    - (if number == 1 {
                        0.0
                    } else if number == 2 {
                        16.0
                    } else {
                        32.0
                    }),
                target.x + target.w / 2.0,
                target.y + 4.0,
            ),
            1 => (
                container.x + container.w + 40.0,
                target.y + target.h / 2.0 - 16.0,
                target.x + target.w,
                target.y + target.h / 2.0,
            ),
            2 => (
                if number == 7 {
                    target.x + target.w - 80.0
                } else {
                    target.x + target.w / 2.0 - 12.0
                },
                target.y + target.h + 40.0,
                if number == 7 {
                    target.x + target.w - 68.0
                } else {
                    target.x + target.w / 2.0
                },
                target.y + target.h,
            ),
            _ => (
                container.x - 72.0,
                target.y + target.h / 2.0 - 16.0,
                target.x,
                target.y + target.h / 2.0,
            ),
        };
        if side == 0 {
            dotted(&mut svg, tx, y + 32.0, ty - y - 32.0, true, "#49454f");
        } else if side == 2 {
            dotted(&mut svg, tx, ty, y - ty, true, "#49454f");
        } else {
            dotted(
                &mut svg,
                if side == 1 { tx } else { x + 24.0 },
                ty,
                if side == 1 { x - tx } else { tx - x - 24.0 },
                false,
                "#49454f",
            );
        }
        if number != 1 && number != 7 {
            annotations::outline(&mut svg, target);
        }
        annotations::marker(number, Dp(x), Dp(y));
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn list_guideline_anatomy() {
    prose(
        "The container and label are required. Supporting text, leading media and trailing information are optional.",
    );
    gap(Dp(24.0));
    list_anatomy_diagram();
    gap(Dp(24.0));
    subheading("Container".into());
    prose(
        "The tallest contained element determines the row's height. Keep the content hierarchy clear as visual styles change.",
    );
    gap(Dp(24.0));
    subheading("Label & supporting text".into());
    prose(
        "Keep the label brief. Limit supporting text to a few readable lines and reduce detail at compact widths.",
    );
    gap(Dp(24.0));
    subheading("Icons".into());
    prose(
        "A leading icon identifies the subject; a trailing icon can communicate status or another action.",
    );
    gap(Dp(24.0));
    subheading("Leading media".into());
    prose(
        "Anchor thumbnails and avatars to the leading edge for consistent scanning. Use circular avatars for people and rectangular media for other content.",
    );
    gap(Dp(24.0));
    subheading("Trailing text".into());
    prose(
        "Use concise supplemental details, such as a count, price or date, at the trailing edge.",
    );
    gap(Dp(24.0));
    subheading("Selection controls".into());
    prose(
        "Use radio-style semantics for one choice, checkboxes for several choices and switches for independent settings. A selectable row has one selection interaction.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Single,
            false,
            true,
            true,
            "Guideline single selection",
        );
    }));
    gap(Dp(24.0));
    subheading("Gaps & dividers".into());
    prose(
        "Use small gaps with contained segmented rows. Reserve dividers for uncontained or more complex lists that need stronger separation.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Action,
            false,
            true,
            true,
            "Guideline grouped list",
        );
    }));
}

#[tessera]
fn list_adaptive() {
    subheading("Line length".into());
    prose(
        "Adjust margins or columns to avoid long text lines. Preserve a comfortable reading measure instead of stretching rows across an expanded window.",
    );
    gap(Dp(24.0));
    subheading("Adapt list elements & layout".into());
    prose(
        "Expanded windows can show additional context and larger imagery while compact windows keep the most important information.",
    );
    gap(Dp(24.0));
    list_reading_examples();
    gap(Dp(24.0));
    subheading("Swap components".into());
    prose(
        "A card or carousel can present the same images, text and actions when more space is available.",
    );
    gap(Dp(24.0));
    subheading("Compact breakpoints".into());
    prose("Let the list use the available width and open details as another page.");
    gap(Dp(24.0));
    subheading("Medium & expanded breakpoints".into());
    prose("A list and its selected detail can share the same view.");
    gap(Dp(24.0));
    list_detail_example();
}

#[tessera]
fn list_detail_example() {
    let selected = remember(|| 0_usize);
    panel(RenderSlot::new(move || {
        flow_row().item_spacing(Dp(24.0)).line_spacing(Dp(24.0)).children(move || {
        column().modifier(Modifier::new().width(Dp(240.0))).children(move || {for index in 0..3 {detail_row(index,selected);}});
        column().modifier(Modifier::new().width(Dp(240.0)).padding_all(Dp(16.0))).children(move || {text().content(ITEMS[selected.get()]).style(type_style(24.0,32.0));gap(Dp(16.0));prose("The selected item's details remain next to the list in an expanded layout.");});
    });
    }));
}

#[tessera]
fn detail_row(index: usize, selected: State<usize>) {
    list_item()
        .headline(ITEMS[index])
        .selection_mode(ListItemSelectionMode::Single)
        .selected(selected.get() == index)
        .on_selection_change(move |value| {
            if value {
                selected.set(index);
            }
        });
}

#[tessera]
fn list_behavior() {
    subheading("List selection modes".into());
    prose(
        "Single-select and multi-select rows expose one selection action for the whole item. Action rows can navigate or offer clearly separated supplementary actions.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Multiple,
            false,
            true,
            true,
            "Behavior multiple selection",
        );
    }));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Action,
            true,
            true,
            true,
            "Behavior multiple actions",
        );
    }));
    gap(Dp(24.0));
    subheading("List interactions".into());
    prose(
        "Nested rows may expand and collapse. If a swipe reveals actions, provide a reachable secondary action as an alternative.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(expansion));
}

#[tessera]
fn list_interaction() {
    subheading("Touch".into());
    prose("A ripple confirms activation or selection.");
    gap(Dp(24.0));
    subheading("Cursor".into());
    prose("Hover feedback distinguishes actionable rows from passive information.");
    gap(Dp(24.0));
    subheading("Keyboard & switch".into());
    prose(
        "Focus identifies the action reached by traversal. Space or Enter activates the focused item.",
    );
    gap(Dp(24.0));
    subheading("Swipe".into());
    prose("Keep hidden swipe actions reachable through a visible secondary control.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Action,
            true,
            true,
            true,
            "Accessible multiple actions",
        );
    }));
}

#[tessera]
fn list_focus() {
    subheading("Single-action lists".into());
    prose("Each row exposes one action and an accessible name matching its content.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Action,
            false,
            true,
            true,
            "Focus single actions",
        );
    }));
    gap(Dp(24.0));
    subheading("Multi-action lists".into());
    prose(
        "The primary action and supplementary controls must each be reachable. Activating a secondary control must not activate the row's primary action.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Action,
            true,
            true,
            true,
            "Focus multiple actions",
        );
    }));
}

#[cfg(all(test, not(target_family = "wasm")))]
mod collection_article_tests {
    use std::{cell::Cell, time::Duration};

    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        accesskit::Node,
    };

    use super::*;

    struct Pipelines;
    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    thread_local! {static KIND: Cell<usize> = const {Cell::new(0)};}

    #[tessera]
    fn fixture() {
        material_theme().child(|| match KIND.get() {
            0 => {
                super::super::cards::cards_spec();
            }
            1 => {
                super::super::chips::chips_spec();
            }
            2 => {
                lists_spec();
            }
            _ => {
                super::super::dividers::divider_spec();
            }
        });
    }

    fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> Node {
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.role() == role && node.label() == Some(label))
            .unwrap_or_else(|| panic!("missing {label} {role:?}"))
            .1
    }

    fn click(renderer: &mut HeadlessRenderer, node: Node) {
        let bounds = node.bounds().unwrap();
        let position = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(4);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn collection_articles_keep_official_chapters_and_independent_live_actions() {
        for (kind, guidelines, accessibility) in [
            (
                0,
                vec![
                    "Usage",
                    "Anatomy",
                    "Cards in a collection",
                    "Adaptive design",
                    "Behavior",
                ],
                vec![
                    "Use cases",
                    "Interaction & style",
                    "Focus",
                    "Keyboard navigation",
                    "Labeling elements",
                ],
            ),
            (
                1,
                vec![
                    "Usage",
                    "Anatomy",
                    "Assist chips",
                    "Filter chips",
                    "Input chips",
                    "Suggestion chips",
                ],
                vec![
                    "Use cases",
                    "Interaction & style",
                    "Keyboard navigation",
                    "Labeling elements",
                ],
            ),
            (
                2,
                vec!["Usage", "Anatomy", "Adaptive design", "Behavior"],
                vec![
                    "Use cases",
                    "Indicate selection with more than color",
                    "Interaction & style",
                    "Focus",
                    "Keyboard navigation",
                    "Labeling elements",
                ],
            ),
            (
                3,
                vec![
                    "Usage",
                    "Anatomy",
                    "Full-width dividers",
                    "Inset dividers",
                    "Vertical divider",
                ],
                vec![],
            ),
        ] {
            KIND.set(kind);
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(Pipelines)],
                HeadlessConfig {
                    width: 900,
                    height: 8000,
                    sample_count: 1,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(5);
            for tab in ["Overview", "Specs", "Guidelines", "Accessibility"] {
                node(&renderer, tab, Role::Tab);
            }
            for (tab, expected) in [("Guidelines", guidelines), ("Accessibility", accessibility)] {
                let target = node(&renderer, tab, Role::Tab);
                click(&mut renderer, target);
                let mut headings = renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .into_iter()
                    .filter_map(|(_, node)| {
                        (node.role() == Role::Heading && node.level() == Some(2))
                            .then(|| node.label().unwrap().to_owned())
                    })
                    .collect::<Vec<_>>();
                headings.sort();
                let mut expected = expected;
                expected.sort();
                assert_eq!(headings, expected, "article {kind} {tab}");
                if tab == "Guidelines" && matches!(kind, 0 | 2 | 3) {
                    for number in 1..=if kind == 0 {
                        6
                    } else if kind == 2 {
                        10
                    } else {
                        1
                    } {
                        node(&renderer, &format!("Anatomy marker {number}"), Role::Note);
                    }
                }
            }
            if kind == 1 {
                let remove = node(&renderer, "Remove Alex", Role::Button);
                click(&mut renderer, remove);
                let snapshot = renderer.accessibility_snapshot().unwrap();
                assert!(
                    !snapshot
                        .nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some("Alex")
                            && node.role() == Role::CheckBox)
                );
                node(&renderer, "Sam", Role::CheckBox);
                let restore = node(&renderer, "Restore recipients", Role::Button);
                click(&mut renderer, restore);
                node(&renderer, "Alex", Role::CheckBox);
            } else if kind == 2 {
                let secondary = node(
                    &renderer,
                    "Accessible multiple actions: Notes more actions",
                    Role::Button,
                );
                click(&mut renderer, secondary);
                node(
                    &renderer,
                    "Accessible multiple actions secondary actions: 1",
                    Role::Label,
                );
                node(
                    &renderer,
                    "Accessible multiple actions primary actions: 0",
                    Role::Label,
                );
            }
        }
    }
}
