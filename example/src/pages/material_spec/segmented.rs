//! Segmented button specifications with live single- and multi-select groups.
//!
//! ## Usage
//!
//! Compare selection models, states, density and dimensions of connected
//! segments.

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
use tessera_ui::{
    Callback, Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
    ACCENT,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const LABELS: [&str; 3] = ["ui-day", "ui-week", "ui-month"];

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
        .map(|(_, label)| I18N.text(label))
        .collect::<Vec<_>>();
    if names.is_empty() {
        I18N.text("ui-selection-none")
    } else {
        names.join(", ")
    }
}

#[tessera]
pub(super) fn segmented_spec(
    on_recommended: Callback,
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    article(
        "ui-segmented-buttons",
        "ui-segmented-buttons-help-people-select-options-switch-views-or-sort-elements-4746",
        vec![
            SpecSection {
                title: "ui-tokens-and-specs",
                content: RenderSlot::new(tokens_and_specs),
            },
            SpecSection {
                title: "ui-color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "ui-states",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    )
    .route_optional(route.clone())
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview_intro)),
            sections: vec![
                SpecSection {
                    title: "ui-availability-resources",
                    content: RenderSlot::new(availability),
                },
                SpecSection {
                    title: "ui-m3-expressive-update",
                    content: RenderSlot::new(move || {
                        expressive_update(on_recommended.clone());
                    }),
                },
                SpecSection {
                    title: "ui-differences-from-m2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(anatomy),
                },
                SpecSection {
                    title: "ui-single-select-7063",
                    content: RenderSlot::new(single_select),
                },
                SpecSection {
                    title: "ui-multi-select-7064",
                    content: RenderSlot::new(multi_select),
                },
                SpecSection {
                    title: "ui-placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(behavior),
                },
            ],
        },
        ArticlePage {
            title: "ui-accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-use-cases",
                    content: RenderSlot::new(accessibility_use_cases),
                },
                SpecSection {
                    title: "ui-interaction-style",
                    content: RenderSlot::new(accessibility_interaction),
                },
                SpecSection {
                    title: "ui-keyboard-navigation",
                    content: RenderSlot::new(accessibility_keyboard),
                },
                SpecSection {
                    title: "ui-labeling-elements",
                    content: RenderSlot::new(accessibility_labeling),
                },
            ],
        },
    ])
    .hero(|| {
        panel(RenderSlot::new(|| {
            let selected = remember(|| 1_u8);
            segmented_row(
                "ui-preview".to_owned(),
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
    selection_mode: tessera_components::segmented_buttons::SegmentedButtonSelectionMode,
) {
    for (index, name) in LABELS.into_iter().enumerate() {
        let painter = show_icons.then(|| segment_icon(index));
        segmented_button()
            .selection_mode(selection_mode)
            .label(I18N.text(name))
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
            .accessibility_label(I18N.format(
                "ui-value-value-format-2",
                [
                    ("v0", I18N.text(&label).into()),
                    ("v1", I18N.text(&name).into()),
                ],
            ))
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
            label: Some(I18N.format(
                "ui-value-segmented-buttons",
                [("v0", I18N.text(&label).into())],
            )),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if multi {
                multi_choice_segmented_button_row()
                    .modifier(Modifier::new().fill_max_width())
                    .content({
                        let segment_caption = caption.clone();
                        move |selection_mode| {
                            segments(
                                segment_caption.clone(),
                                selected,
                                true,
                                enabled,
                                show_icons,
                                show_checks,
                                height,
                                selection_mode,
                            );
                        }
                    });
            } else {
                single_choice_segmented_button_row()
                    .modifier(Modifier::new().fill_max_width())
                    .content({
                        let segment_caption = caption.clone();
                        move |selection_mode| {
                            segments(
                                segment_caption.clone(),
                                selected,
                                false,
                                enabled,
                                show_icons,
                                show_checks,
                                height,
                                selection_mode,
                            );
                        }
                    });
            }
            gap(Dp(16.0));
            text()
                .content(I18N.format(
                    "ui-value-selection-value",
                    [
                        ("v0", I18N.text(&label).into()),
                        ("v1", I18N.text(&selection_summary(selected.get())).into()),
                    ],
                ))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(I18N.text(value))
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn overview_intro() {
    prose(
        "ui-segmented-buttons-are-connected-groups-of-two-to-five-segments-each-segment-can-show-a-lab",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "ui-overview".to_owned(),
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
        "ui-use-single-select-segments-to-switch-views-or-choose-one-option",
        "ui-use-multi-select-segments-to-apply-several-filters-at-once",
        "ui-keep-the-group-between-two-and-five-segments",
        "ui-segments-are-for-selection-not-for-navigation",
    ] {
        prose(format!("• {}", I18N.text(point)));
    }
}

#[tessera]
fn availability() {
    spec_table(
        vec!["ui-type".into(), "ui-resource".into(), "ui-status".into()],
        vec![
            vec![
                "ui-design".into(),
                "ui-design-kit-figma".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Flutter".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Jetpack Compose".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Android Views (MDC-Android)".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Web".into(),
                "ui-unavailable".into(),
            ],
        ],
    );
}

#[tessera]
fn expressive_update(on_recommended: Callback) {
    prose(
        "ui-segmented-buttons-are-no-longer-recommended-in-the-material-3-expressive-update-products-t",
    );
    gap(Dp(16.0));
    button()
        .text()
        .accessibility_label(I18N.text("ui-open-recommended-button-groups"))
        .on_click_shared(on_recommended)
        .child(move || {
            text().content(I18N.text("ui-open-recommended-button-groups"));
        });
}

#[tessera]
fn differences() {
    prose(
        "ui-color-segmented-buttons-use-the-secondary-container-role-for-selected-segments-and-support",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "ui-differences".to_owned(),
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
    subheading("ui-variants".into());
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let single = remember(|| 1_u8);
        let multi = remember(|| 5_u8);
        segmented_row(
            "ui-single-select-7063".to_owned(),
            single,
            false,
            true,
            false,
            true,
            Dp(40.0),
        );
        gap(Dp(32.0));
        segmented_row(
            "ui-multi-select-7064".to_owned(),
            multi,
            true,
            true,
            false,
            true,
            Dp(40.0),
        );
    }));
    gap(Dp(32.0));
    subheading("ui-configurations".into());
    gap(Dp(16.0));
    configurations();
    gap(Dp(32.0));
    subheading("ui-tokens".into());
    gap(Dp(16.0));
    spec_table(
        vec!["ui-element".into(), "ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "ui-container".into(),
                "md.comp.segmented-button.container.color".into(),
                "Surface".into(),
            ],
            vec![
                "ui-selected-container".into(),
                "md.comp.segmented-button.selected.container.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "ui-selected-label".into(),
                "md.comp.segmented-button.selected.label-text.color".into(),
                "On secondary container".into(),
            ],
            vec![
                "ui-unselected-label".into(),
                "md.comp.segmented-button.label-text.color".into(),
                "On surface".into(),
            ],
            vec![
                "ui-outline-element".into(),
                "md.comp.segmented-button.outline.color".into(),
                "Outline".into(),
            ],
            vec![
                "ui-height".into(),
                "md.comp.segmented-button.container.height".into(),
                "40dp".into(),
            ],
            vec![
                "ui-shape".into(),
                "md.comp.segmented-button.container.shape".into(),
                "ui-full".into(),
            ],
            vec![
                "ui-label".into(),
                "md.comp.segmented-button.label-text.text-style".into(),
                "Label large · 14dp / 20dp".into(),
            ],
            vec![
                "ui-horizontal-padding".into(),
                "md.comp.segmented-button.leading-space / trailing-space".into(),
                "12dp".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    subheading("ui-anatomy".into());
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "ui-anatomy".to_owned(),
            selected,
            false,
            true,
            true,
            true,
            Dp(40.0),
        );
    }));
    gap(Dp(16.0));
    prose("ui-1-container-2-segment-3-icon-4-label-text-5-divider");
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
                .accessibility_label(I18N.text("ui-segmented-configuration-icons"))
                .on_click(move || icons.set(!icons.get()))
                .child(move || {
                    text().content(I18N.text("ui-icons"));
                });
            button()
                .tonal()
                .selected(checks.get())
                .accessibility_label(I18N.text("ui-segmented-configuration-checkmark"))
                .on_click(move || checks.set(!checks.get()))
                .child(move || {
                    text().content(I18N.text("ui-checkmark"));
                });
            button()
                .tonal()
                .selected(enabled.get())
                .accessibility_label(I18N.text("ui-segmented-configuration-enabled"))
                .on_click(move || enabled.set(!enabled.get()))
                .child(move || {
                    text().content(I18N.text("ui-enabled"));
                });
            button()
                .tonal()
                .selected(compact.get())
                .accessibility_label(I18N.text("ui-segmented-configuration-compact"))
                .on_click(move || compact.set(!compact.get()))
                .child(move || {
                    text().content(I18N.text("ui-compact-height"));
                });
        });
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        segmented_row(
            "ui-configuration".to_owned(),
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
                                    if dark {
                                        "ui-dark-scheme"
                                    } else {
                                        "ui-light-scheme"
                                    }
                                    .to_owned(),
                                );
                                gap(Dp(24.0));
                                let selected = remember(|| 1_u8);
                                segmented_row(
                                    if dark { "ui-dark" } else { "ui-light" }.to_owned(),
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
    subheading("ui-color-roles-4475".into());
    spec_table(
        vec!["ui-element".into(), "ui-color-role".into()],
        vec![
            vec!["ui-selected-container".into(), "Secondary container".into()],
            vec![
                "ui-selected-label-and-icon".into(),
                "On secondary container".into(),
            ],
            vec!["ui-unselected-label-and-icon".into(), "On surface".into()],
            vec!["ui-outline-element".into(), "Outline".into()],
            vec!["ui-state-layer".into(), "On surface".into()],
        ],
    );
}

#[tessera]
fn states() {
    prose(
        "ui-hover-focus-with-tab-press-and-hold-or-select-a-segment-to-inspect-real-feedback-disabled",
    );
    gap(Dp(24.0));
    for (selected_mask, enabled) in [(0_u8, true), (1, true), (0, false), (1, false)] {
        let title = match (selected_mask, enabled) {
            (0, true) => "ui-unselected-enabled",
            (_, true) => "ui-selected-enabled",
            (0, false) => "ui-unselected-disabled",
            _ => "ui-selected-disabled",
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
            "ui-measurements".to_owned(),
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
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-container-height".into(), "40dp".into()],
            vec!["ui-minimum-interactive-target".into(), "48dp".into()],
            vec!["ui-corner-radius".into(), "ui-full-20dp".into()],
            vec!["ui-icon-size".into(), "ui-18dp-inside-the-segment".into()],
            vec!["ui-label-style".into(), "Label large · 14dp / 20dp".into()],
            vec!["ui-horizontal-padding".into(), "12dp".into()],
            vec!["ui-icon-and-label-spacing".into(), "8dp".into()],
            vec!["ui-between-segments".into(), "ui-0dp-shared-outline".into()],
            vec!["ui-minimum-maximum-segments".into(), "2 / 5".into()],
        ],
    );
}

#[tessera]
fn usage() {
    prose(
        "ui-use-segmented-buttons-to-select-options-switch-between-views-or-sort-a-list-keep-labels-sh",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Do".into(), "Don't".into()],
        vec![
            vec![
                "ui-use-two-to-five-segments-in-one-connected-group".into(),
                "ui-stretch-a-group-beyond-five-segments".into(),
            ],
            vec![
                "ui-use-single-select-for-views-and-multi-select-for-filters".into(),
                "ui-mix-single-select-and-multi-select-in-the-same-group".into(),
            ],
            vec![
                "ui-keep-every-segment-label-short-and-parallel".into(),
                "ui-use-segments-to-navigate-between-screens".into(),
            ],
            vec![
                "ui-let-the-selected-segment-show-a-checkmark".into(),
                "ui-rely-on-color-alone-to-show-selection".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 0_u8);
        segmented_row(
            "ui-usage".to_owned(),
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
        "ui-a-segmented-button-group-contains-a-container-two-to-five-segments-an-optional-icon-a-labe",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "ui-guideline-anatomy".to_owned(),
            selected,
            false,
            true,
            true,
            true,
            Dp(40.0),
        );
    }));
    gap(Dp(16.0));
    prose("ui-1-container-2-segment-3-icon-4-label-text-5-divider");
}

#[tessera]
fn single_select() {
    prose(
        "ui-single-select-segments-behave-like-radio-buttons-exactly-one-segment-stays-selected-and-ch",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "ui-single-select-7063".to_owned(),
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
        "ui-multi-select-segments-behave-like-checkboxes-several-segments-can-be-selected-at-once-and",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 5_u8);
        segmented_row(
            "ui-multi-select-7064".to_owned(),
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
        "ui-place-segmented-buttons-close-to-the-content-they-affect-a-group-can-sit-above-a-list-to-s",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-context".into(), "Do".into(), "ui-avoid".into()],
        vec![
            vec![
                "ui-view-switching".into(),
                "ui-single-select-group-above-the-content".into(),
                "ui-one-segment-per-screen".into(),
            ],
            vec![
                "ui-filtering".into(),
                "ui-multi-select-group-in-a-toolbar".into(),
                "ui-hiding-the-current-filter-state".into(),
            ],
            vec![
                "ui-sorting".into(),
                "ui-three-or-four-ordered-options".into(),
                "ui-more-than-five-sort-options".into(),
            ],
            vec![
                "ui-navigation".into(),
                "ui-tabs-a-navigation-bar-or-a-rail".into(),
                "ui-segments-as-primary-navigation".into(),
            ],
        ],
    );
}

#[tessera]
fn behavior() {
    prose(
        "ui-selecting-a-segment-applies-the-change-immediately-in-a-single-select-group-the-previous-s",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 2_u8);
        segmented_row(
            "ui-behavior".to_owned(),
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
    prose("ui-people-should-be-able-to-use-assistive-technology-to");
    gap(Dp(16.0));
    for point in [
        "ui-navigate-to-a-segmented-button-group-and-activate-its-segments",
        "ui-understand-what-each-segment-selection-will-do",
        "ui-hear-which-segment-is-currently-selected",
    ] {
        prose(format!("• {}", I18N.text(point)));
    }
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 1_u8);
        segmented_row(
            "ui-accessible".to_owned(),
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
        "ui-the-group-owns-the-selection-each-segment-reports-its-own-selected-state-through-toggled-s",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        let selected = remember(|| 5_u8);
        segmented_row(
            "ui-accessible-multi-select".to_owned(),
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
        vec!["ui-keys".into(), "ui-action".into()],
        vec![
            vec![
                "ui-tab".into(),
                "ui-moves-focus-into-the-group-then-out-to-the-next-control".into(),
            ],
            vec![
                "ui-left-right-arrow".into(),
                "ui-moves-focus-between-the-segments-of-the-group".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-selects-the-focused-segment-in-a-single-select-group-it-clears-the-previous-one"
                    .into(),
            ],
            vec![
                "ui-space-or-enter-on-a-selected-segment".into(),
                "ui-deselects-it-in-a-multi-select-group".into(),
            ],
        ],
    );
}

#[tessera]
fn accessibility_labeling() {
    prose(
        "ui-give-every-segment-a-short-readable-label-the-group-itself-needs-a-name-so-the-selection-i",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-element".into(), "ui-accessible-information".into()],
        vec![
            vec![
                "ui-group".into(),
                "ui-name-describing-the-choice-such-as-time-range".into(),
            ],
            vec![
                "ui-segment".into(),
                "ui-its-visible-label-for-example-week".into(),
            ],
            vec![
                "ui-selected-segment".into(),
                "ui-toggled-state-announced-together-with-the-label".into(),
            ],
            vec![
                "ui-icon".into(),
                "ui-decorative-when-the-label-is-present".into(),
            ],
        ],
    );
}
