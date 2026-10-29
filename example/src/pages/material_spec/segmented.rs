//! Segmented button specifications with live single- and multi-select groups.
//!
//! ## Usage
//!
//! Compare selection models, states, density and dimensions of connected segments.

use tessera_components::{
    button::button,
    column::column,
    flow_row::flow_row,
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    painter::Painter,
    segmented_buttons::{
        SegmentedButtonDefaults, multi_choice_segmented_button_row, segmented_button,
        single_choice_segmented_button_row,
    },
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context};

use super::{
    ACCENT,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const LABELS: [&str; 3] = ["Day", "Week", "Month"];

fn segment_icon(index: usize) -> Painter {
    match index {
        0 => Painter::from(filled::CALENDAR_VIEW_DAY_SVG),
        1 => Painter::from(filled::CALENDAR_VIEW_WEEK_SVG),
        _ => Painter::from(filled::CALENDAR_VIEW_MONTH_SVG),
    }
}

fn selection_summary(mask: u8) -> String {
    let names = LABELS
        .into_iter()
        .enumerate()
        .filter(|(index, _)| mask & (1 << index) != 0)
        .map(|(_, label)| label)
        .collect::<Vec<_>>();
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

#[tessera]
pub(super) fn segmented_spec() {
    article(
        "Segmented buttons",
        "Segmented buttons help people select options, switch views, or sort elements.",
        vec![
            SpecSection {
                title: "Tokens and specs",
                content: RenderSlot::new(tokens_and_specs),
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
    )
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(overview_intro)),
            sections: vec![
                SpecSection {
                    title: "Availability & resources",
                    content: RenderSlot::new(availability),
                },
                SpecSection {
                    title: "M3 Expressive update",
                    content: RenderSlot::new(expressive_update),
                },
                SpecSection {
                    title: "Differences from M2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "Guidelines",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(anatomy),
                },
                SpecSection {
                    title: "Single-select",
                    content: RenderSlot::new(single_select),
                },
                SpecSection {
                    title: "Multi-select",
                    content: RenderSlot::new(multi_select),
                },
                SpecSection {
                    title: "Placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(behavior),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Use cases",
                    content: RenderSlot::new(accessibility_use_cases),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(accessibility_interaction),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(accessibility_keyboard),
                },
                SpecSection {
                    title: "Labeling elements",
                    content: RenderSlot::new(accessibility_labeling),
                },
            ],
        },
    ])
    .hero(|| {
        panel(RenderSlot::new(|| {
            let selected = remember(|| 1_u8);
            segmented_row(
                "Preview".to_owned(),
                selected,
                false,
                true,
                true,
                true,
                Dp(40.0),
            );
        }));
    });
}

#[tessera]
fn segments(
    #[prop(into)] label: String,
    selected: State<u8>,
    multi: bool,
    enabled: bool,
    show_icons: bool,
    show_checks: bool,
    height: Dp,
) {
    for (index, name) in LABELS.into_iter().enumerate() {
        let painter = show_icons.then(|| segment_icon(index));
        segmented_button()
            .label(name)
            .selected(selected.get() & (1 << index) != 0)
            .enabled(enabled)
            .show_selected_icon(show_checks)
            .icon_optional(painter)
            .min_height(height)
            .content_padding(Padding::symmetric(
                Dp(12.0),
                Dp(((height.0 - 20.0) * 0.5).clamp(0.0, 8.0)),
            ))
            .shape(SegmentedButtonDefaults::item_shape(
                index,
                LABELS.len(),
                SegmentedButtonDefaults::shape(),
            ))
            .accessibility_label(format!("{label} {name}"))
            .on_click(move || {
                selected.set(if multi {
                    selected.get() ^ (1 << index)
                } else {
                    1 << index
                })
            });
    }
}

#[tessera]
fn segmented_row(
    #[prop(into)] label: String,
    selected: State<u8>,
    multi: bool,
    enabled: bool,
    show_icons: bool,
    show_checks: bool,
    height: Dp,
) {
    let caption = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(format!("{label} segmented buttons")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if multi {
                multi_choice_segmented_button_row()
                    .modifier(Modifier::new().fill_max_width())
                    .content({
                        let segment_caption = caption.clone();
                        move || {
                            segments(
                                segment_caption.clone(),
                                selected,
                                true,
                                enabled,
                                show_icons,
                                show_checks,
                                height,
                            );
                        }
                    });
            } else {
                single_choice_segmented_button_row()
                    .modifier(Modifier::new().fill_max_width())
                    .content({
                        let segment_caption = caption.clone();
                        move || {
                            segments(
                                segment_caption.clone(),
                                selected,
                                false,
                                enabled,
                                show_icons,
                                show_checks,
                                height,
                            );
                        }
                    });
            }
            gap(Dp(16.0));
            text()
                .content(format!(
                    "{label} selection: {}",
                    selection_summary(selected.get())
                ))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn overview_intro() {
    prose(
        "Segmented buttons are connected groups of two to five segments. Each segment can show a label, an icon or both, and a selected segment displays a checkmark.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "Overview".to_owned(),
            selected,
            false,
            true,
            true,
            true,
            Dp(40.0),
        );
    }));
    gap(Dp(24.0));
    for point in [
        "Use single-select segments to switch views or choose one option.",
        "Use multi-select segments to apply several filters at once.",
        "Keep the group between two and five segments.",
        "Segments are for selection, not for navigation.",
    ] {
        prose(format!("• {point}"));
    }
}

#[tessera]
fn availability() {
    spec_table(
        vec!["Type".into(), "Resource".into(), "Status".into()],
        vec![
            vec![
                "Design".into(),
                "Design Kit (Figma)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Flutter".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
        ],
    );
}

#[tessera]
fn expressive_update() {
    prose(
        "Segmented buttons are no longer recommended in the Material 3 expressive update. Products that adopt the expressive style should use a connected button group instead, which keeps most of the same functionality with an updated visual design.",
    );
}

#[tessera]
fn differences() {
    prose(
        "Color: segmented buttons use the secondary container role for selected segments and support dynamic color. Shape: segments use fully rounded ends with a shared outline instead of separate rectangles. Selection: a checkmark and a container fill replace the M2 underline.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "Differences".to_owned(),
            selected,
            false,
            true,
            false,
            true,
            Dp(40.0),
        );
    }));
}

#[tessera]
fn tokens_and_specs() {
    subheading("Variants".into());
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let single = remember(|| 1_u8);
        let multi = remember(|| 5_u8);
        segmented_row(
            "Single-select".to_owned(),
            single,
            false,
            true,
            false,
            true,
            Dp(40.0),
        );
        gap(Dp(32.0));
        segmented_row(
            "Multi-select".to_owned(),
            multi,
            true,
            true,
            false,
            true,
            Dp(40.0),
        );
    }));
    gap(Dp(32.0));
    subheading("Configurations".into());
    gap(Dp(16.0));
    configurations();
    gap(Dp(32.0));
    subheading("Tokens".into());
    gap(Dp(16.0));
    spec_table(
        vec!["Element".into(), "Token".into(), "Value".into()],
        vec![
            vec![
                "Container".into(),
                "md.comp.segmented-button.container.color".into(),
                "Surface".into(),
            ],
            vec![
                "Selected container".into(),
                "md.comp.segmented-button.selected.container.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "Selected label".into(),
                "md.comp.segmented-button.selected.label-text.color".into(),
                "On secondary container".into(),
            ],
            vec![
                "Unselected label".into(),
                "md.comp.segmented-button.label-text.color".into(),
                "On surface".into(),
            ],
            vec![
                "Outline".into(),
                "md.comp.segmented-button.outline.color".into(),
                "Outline".into(),
            ],
            vec![
                "Height".into(),
                "md.comp.segmented-button.container.height".into(),
                "40dp".into(),
            ],
            vec![
                "Shape".into(),
                "md.comp.segmented-button.container.shape".into(),
                "Full".into(),
            ],
            vec![
                "Label".into(),
                "md.comp.segmented-button.label-text.text-style".into(),
                "Label large · 14dp / 20dp".into(),
            ],
            vec![
                "Horizontal padding".into(),
                "md.comp.segmented-button.leading-space / trailing-space".into(),
                "12dp".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    subheading("Anatomy".into());
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "Anatomy".to_owned(),
            selected,
            false,
            true,
            true,
            true,
            Dp(40.0),
        );
    }));
    gap(Dp(16.0));
    prose("1  Container    2  Segment    3  Icon    4  Label text    5  Divider");
}

#[tessera]
fn configurations() {
    let icons = remember(|| false);
    let checks = remember(|| true);
    let enabled = remember(|| true);
    let compact = remember(|| false);
    let selected = remember(|| 1_u8);
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            button()
                .tonal()
                .selected(icons.get())
                .accessibility_label("Segmented configuration icons".to_owned())
                .on_click(move || icons.set(!icons.get()))
                .child(|| {
                    text().content("Icons");
                });
            button()
                .tonal()
                .selected(checks.get())
                .accessibility_label("Segmented configuration checkmark".to_owned())
                .on_click(move || checks.set(!checks.get()))
                .child(|| {
                    text().content("Checkmark");
                });
            button()
                .tonal()
                .selected(enabled.get())
                .accessibility_label("Segmented configuration enabled".to_owned())
                .on_click(move || enabled.set(!enabled.get()))
                .child(|| {
                    text().content("Enabled");
                });
            button()
                .tonal()
                .selected(compact.get())
                .accessibility_label("Segmented configuration compact".to_owned())
                .on_click(move || compact.set(!compact.get()))
                .child(|| {
                    text().content("Compact height");
                });
        });
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        segmented_row(
            "Configuration".to_owned(),
            selected,
            false,
            enabled.get(),
            icons.get(),
            checks.get(),
            if compact.get() { Dp(32.0) } else { Dp(40.0) },
        );
    }));
}

#[tessera]
fn colors() {
    for dark in [false, true] {
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                let scheme = use_context::<MaterialTheme>()
                    .expect("MaterialTheme must be provided")
                    .get()
                    .color_scheme;
                surface()
                    .style(scheme.surface.into())
                    .modifier(Modifier::new().fill_max_width())
                    .child(move || {
                        column()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                            .children(move || {
                                subheading(
                                    if dark { "Dark scheme" } else { "Light scheme" }.to_owned(),
                                );
                                gap(Dp(24.0));
                                let selected = remember(|| 1_u8);
                                segmented_row(
                                    if dark { "Dark" } else { "Light" }.to_owned(),
                                    selected,
                                    false,
                                    true,
                                    false,
                                    true,
                                    Dp(40.0),
                                );
                            });
                    });
            });
        gap(Dp(24.0));
    }
    subheading("Color roles".into());
    spec_table(
        vec!["Element".into(), "Color role".into()],
        vec![
            vec!["Selected container".into(), "Secondary container".into()],
            vec!["Selected label and icon".into(), "On secondary container".into()],
            vec!["Unselected label and icon".into(), "On surface".into()],
            vec!["Outline".into(), "Outline".into()],
            vec!["State layer".into(), "On surface".into()],
        ],
    );
}

#[tessera]
fn states() {
    prose(
        "Hover, focus with Tab, press and hold, or select a segment to inspect real feedback. Disabled groups keep their selection but stop responding.",
    );
    gap(Dp(24.0));
    for (selected_mask, enabled) in [(0_u8, true), (1, true), (0, false), (1, false)] {
        let title = match (selected_mask, enabled) {
            (0, true) => "Unselected enabled",
            (_, true) => "Selected enabled",
            (0, false) => "Unselected disabled",
            _ => "Selected disabled",
        };
        subheading(title.to_owned());
        gap(Dp(24.0));
        panel(RenderSlot::new(move || {
            let selected = remember(move || selected_mask);
            segmented_row(
                title.to_owned(),
                selected,
                false,
                enabled,
                false,
                true,
                Dp(40.0),
            );
        }));
        gap(Dp(24.0));
    }
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "Measurements".to_owned(),
            selected,
            false,
            true,
            true,
            true,
            Dp(40.0),
        );
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Container height".into(), "40dp".into()],
            vec!["Minimum interactive target".into(), "48dp".into()],
            vec!["Corner radius".into(), "Full · 20dp".into()],
            vec!["Icon size".into(), "18dp inside the segment".into()],
            vec!["Label style".into(), "Label large · 14dp / 20dp".into()],
            vec!["Horizontal padding".into(), "12dp".into()],
            vec!["Icon and label spacing".into(), "8dp".into()],
            vec!["Between segments".into(), "0dp, shared outline".into()],
            vec!["Minimum / maximum segments".into(), "2 / 5".into()],
        ],
    );
}

#[tessera]
fn usage() {
    prose(
        "Use segmented buttons to select options, switch between views or sort a list. Keep labels short and use the same content type in every segment.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Do".into(), "Don't".into()],
        vec![
            vec![
                "Use two to five segments in one connected group".into(),
                "Stretch a group beyond five segments".into(),
            ],
            vec![
                "Use single-select for views and multi-select for filters".into(),
                "Mix single-select and multi-select in the same group".into(),
            ],
            vec![
                "Keep every segment label short and parallel".into(),
                "Use segments to navigate between screens".into(),
            ],
            vec![
                "Let the selected segment show a checkmark".into(),
                "Rely on color alone to show selection".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 0_u8);
        segmented_row(
            "Usage".to_owned(),
            selected,
            false,
            true,
            false,
            true,
            Dp(40.0),
        );
    }));
}

#[tessera]
fn anatomy() {
    prose(
        "A segmented button group contains a container, two to five segments, an optional icon, a label and dividers between segments.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "Guideline anatomy".to_owned(),
            selected,
            false,
            true,
            true,
            true,
            Dp(40.0),
        );
    }));
    gap(Dp(16.0));
    prose("1  Container    2  Segment    3  Icon    4  Label text    5  Divider");
}

#[tessera]
fn single_select() {
    prose(
        "Single-select segments behave like radio buttons: exactly one segment stays selected, and choosing another segment moves the selection.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "Single-select".to_owned(),
            selected,
            false,
            true,
            true,
            true,
            Dp(40.0),
        );
    }));
}

#[tessera]
fn multi_select() {
    prose(
        "Multi-select segments behave like checkboxes: several segments can be selected at once, and each segment toggles independently.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 5_u8);
        segmented_row(
            "Multi-select".to_owned(),
            selected,
            true,
            true,
            false,
            true,
            Dp(40.0),
        );
    }));
}

#[tessera]
fn placement() {
    prose(
        "Place segmented buttons close to the content they affect. A group can sit above a list to switch views or inside a toolbar to sort and filter.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Context".into(), "Do".into(), "Avoid".into()],
        vec![
            vec![
                "View switching".into(),
                "Single-select group above the content".into(),
                "One segment per screen".into(),
            ],
            vec![
                "Filtering".into(),
                "Multi-select group in a toolbar".into(),
                "Hiding the current filter state".into(),
            ],
            vec![
                "Sorting".into(),
                "Three or four ordered options".into(),
                "More than five sort options".into(),
            ],
            vec![
                "Navigation".into(),
                "Tabs, a navigation bar or a rail".into(),
                "Segments as primary navigation".into(),
            ],
        ],
    );
}

#[tessera]
fn behavior() {
    prose(
        "Selecting a segment applies the change immediately. In a single-select group the previous segment clears, while a multi-select group keeps every selected segment until it is tapped again.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 2_u8);
        segmented_row(
            "Behavior".to_owned(),
            selected,
            false,
            true,
            false,
            true,
            Dp(40.0),
        );
    }));
}

#[tessera]
fn accessibility_use_cases() {
    prose("People should be able to use assistive technology to:");
    gap(Dp(16.0));
    for point in [
        "Navigate to a segmented button group and activate its segments.",
        "Understand what each segment selection will do.",
        "Hear which segment is currently selected.",
    ] {
        prose(format!("• {point}"));
    }
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "Accessible".to_owned(),
            selected,
            false,
            true,
            false,
            true,
            Dp(40.0),
        );
    }));
}

#[tessera]
fn accessibility_interaction() {
    prose(
        "The group owns the selection; each segment reports its own selected state through toggled semantics. Selecting a segment updates the group immediately without moving focus out of the group.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 5_u8);
        segmented_row(
            "Accessible multi-select".to_owned(),
            selected,
            true,
            true,
            false,
            true,
            Dp(40.0),
        );
    }));
}

#[tessera]
fn accessibility_keyboard() {
    spec_table(
        vec!["Keys".into(), "Action".into()],
        vec![
            vec![
                "Tab".into(),
                "Moves focus into the group, then out to the next control".into(),
            ],
            vec![
                "Left / Right arrow".into(),
                "Moves focus between the segments of the group".into(),
            ],
            vec![
                "Space or Enter".into(),
                "Selects the focused segment; in a single-select group it clears the previous one".into(),
            ],
            vec![
                "Space or Enter on a selected segment".into(),
                "Deselects it in a multi-select group".into(),
            ],
        ],
    );
}

#[tessera]
fn accessibility_labeling() {
    prose(
        "Give every segment a short, readable label. The group itself needs a name so the selection is understandable out of context.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Accessible information".into()],
        vec![
            vec![
                "Group".into(),
                "Name describing the choice, such as Time range".into(),
            ],
            vec![
                "Segment".into(),
                "Its visible label, for example Week".into(),
            ],
            vec![
                "Selected segment".into(),
                "Toggled state announced together with the label".into(),
            ],
            vec![
                "Icon".into(),
                "Decorative when the label is present".into(),
            ],
        ],
    );
}
