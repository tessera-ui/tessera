//! Segmented button specification examples.
//!
//! ## Usage
//!
//! Compare legacy connected selections, interaction states and compact density.

use tessera_components::{
    button::button,
    column::column,
    flow_row::flow_row,
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    segmented_buttons::{
        SegmentedButtonDefaults, multi_choice_segmented_button_row, segmented_button,
        single_choice_segmented_button_row,
    },
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Dp, Modifier, RenderSlot, accesskit::Role, remember, tessera};

use super::{
    ACCENT,
    action_articles::{Kind, action_article},
    article::SpecSection,
    gap, panel, spec_table, subheading, type_style,
};

const LABELS: [&str; 3] = ["Day", "Week", "Month"];

#[tessera]
pub(super) fn segmented_spec() {
    action_article(
        Kind::Segmented,
        vec![
            SpecSection {
                title: "Tokens and specs",
                content: RenderSlot::new(tokens),
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
            SpecSection {
                title: "Density",
                content: RenderSlot::new(density),
            },
        ],
        RenderSlot::new(|| {
            panel(RenderSlot::new(|| {
                sample("Single-select", false).controls(true);
                gap(Dp(32.0));
                sample("Multi-select", true).controls(true);
            }));
        }),
        RenderSlot::new(|| {
            panel(RenderSlot::new(|| {
                sample("Anatomy", false).controls(true);
            }));
        }),
        RenderSlot::new(|| {
            panel(RenderSlot::new(|| {
                sample("Multi-select", true).controls(true);
            }));
        }),
    );
}

#[tessera]
fn sample(
    #[prop(into)] label: String,
    multi: bool,
    controls: Option<bool>,
    enabled: Option<bool>,
    initial: Option<u8>,
    height: Option<Dp>,
) {
    let selected = remember(|| initial.unwrap_or(1));
    let icons = remember(|| false);
    let checks = remember(|| true);
    let active = remember(|| enabled.unwrap_or(true));
    let height = height.unwrap_or(Dp(40.0));
    let name = remember(|| label.clone());
    let content = RenderSlot::new(move || {
        for (index, text) in LABELS.into_iter().enumerate() {
            let mut item = segmented_button()
                .label(text)
                .selected(selected.get() & (1 << index) != 0)
                .enabled(active.get())
                .show_selected_icon(checks.get())
                .min_height(height)
                .content_padding(Padding::symmetric(
                    Dp(12.0),
                    Dp(((height.0 - 20.0) * 0.5).clamp(0.0, 8.0)),
                ))
                .shape(SegmentedButtonDefaults::item_shape(
                    index,
                    3,
                    SegmentedButtonDefaults::shape(),
                ))
                .accessibility_label(format!("{} {text}", name.get()))
                .on_click(move || {
                    selected.set(if multi {
                        selected.get() ^ (1 << index)
                    } else {
                        1 << index
                    })
                });
            if icons.get() {
                item = item.icon(
                    [
                        filled::CALENDAR_VIEW_DAY_SVG,
                        filled::CALENDAR_VIEW_WEEK_SVG,
                        filled::CALENDAR_VIEW_MONTH_SVG,
                    ][index],
                );
            }
            drop(item);
        }
    });
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(format!("{label} segmented preview")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if controls.unwrap_or(false) {
                let label = label.clone();
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        button()
                            .tonal()
                            .selected(icons.get())
                            .accessibility_label(format!("{label} segment icons"))
                            .on_click(move || icons.set(!icons.get()))
                            .child(|| {
                                text().content("Icons");
                            });
                        button()
                            .tonal()
                            .selected(checks.get())
                            .accessibility_label(format!("{label} selection indicator"))
                            .on_click(move || checks.set(!checks.get()))
                            .child(|| {
                                text().content("Checkmark");
                            });
                        button()
                            .tonal()
                            .selected(active.get())
                            .accessibility_label(format!("{label} segments enabled"))
                            .on_click(move || active.set(!active.get()))
                            .child(|| {
                                text().content("Enabled");
                            });
                    });
                gap(Dp(24.0));
            }
            if multi {
                multi_choice_segmented_button_row()
                    .modifier(Modifier::new().fill_max_width())
                    .content_shared(content);
            } else {
                single_choice_segmented_button_row()
                    .modifier(Modifier::new().fill_max_width())
                    .content_shared(content);
            }
            if controls.unwrap_or(false) {
                gap(Dp(16.0));
                text()
                    .content(format!(
                        "{} selection: {}",
                        name.get(),
                        LABELS
                            .into_iter()
                            .enumerate()
                            .filter(|(index, _)| selected.get() & (1 << index) != 0)
                            .map(|(_, label)| label)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn tokens() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            panel(RenderSlot::new(|| {
                sample("Single-select", false).controls(true);
                gap(Dp(32.0));
                sample("Multi-select", true).controls(true);
            }));
            gap(Dp(32.0));
            spec_table(
                vec!["Element".into(), "Token value".into()],
                vec![
                    vec!["Selected container".into(), "Secondary container".into()],
                    vec!["Selected content".into(), "On secondary container".into()],
                    vec!["Unselected content".into(), "On surface".into()],
                    vec!["Outline".into(), "Outline".into()],
                    vec!["Label".into(), "Label large".into()],
                ],
            );
        });
}

#[tessera]
fn colors() {
    column().modifier(Modifier::new().fill_max_width()).children(move || {

    for dark in [false, true] {
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                surface()
                    .modifier(Modifier::new().fill_max_width())
                    .child(move || {
                        column()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                            .children(move || {
                                sample(if dark { "Dark" } else { "Light" }, false);
                            });
                    });
            });
        gap(Dp(24.0));
    }
    text().content("Selected segments use secondary container and on secondary container. Unselected segments retain an unfilled container with an outline.").style(type_style(16.0,24.0));

    });
}

#[tessera]
fn states() {
    column().modifier(Modifier::new().fill_max_width()).children(move || {

    for selected in [false, true] {
        subheading(if selected { "Selected" } else { "Unselected" }.to_owned());
        gap(Dp(24.0));
        panel(RenderSlot::new(move || {
            sample(
                if selected {
                    "Selected enabled"
                } else {
                    "Unselected enabled"
                },
                true,
            )
            .initial(if selected { 7 } else { 0 });
            gap(Dp(24.0));
            sample(
                if selected {
                    "Selected disabled"
                } else {
                    "Unselected disabled"
                },
                true,
            )
            .initial(if selected { 7 } else { 0 })
            .enabled(false);
        }));
        gap(Dp(32.0));
    }
    text().content("Hover, focus and press the enabled segments to inspect state layers. Arrow keys move between destinations; single-select groups select the focused destination.").style(type_style(16.0,24.0));

    });
}

#[tessera]
fn measurements() {
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Container width".into(), "Dynamic based on labels".into()],
            vec![
                "Segment width".into(),
                "Equal share of the container".into(),
            ],
            vec!["Height".into(), "40dp".into()],
            vec!["Outline".into(), "1dp".into()],
            vec!["Horizontal padding".into(), "At least 12dp".into()],
            vec!["Icon-to-label spacing".into(), "8dp".into()],
            vec!["Touch target".into(), "48dp".into()],
        ],
    );
}

#[tessera]
fn density() {
    column().modifier(Modifier::new().fill_max_width()).children(move || {

    text().content("Each density step reduces the container height by 4dp. Keep interactive targets at least 48dp when space permits.").style(type_style(16.0,24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        for (index, height) in [40.0, 36.0, 32.0, 28.0].into_iter().enumerate() {
            text()
                .content(format!("Density −{index} · {height:.0}dp"))
                .style(type_style(14.0, 20.0));
            gap(Dp(8.0));
            sample(format!("Density {index}"), false).height(Dp(height));
            gap(Dp(24.0));
        }
    }));

    });
}
