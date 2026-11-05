//! Checkbox and switch specification examples.
//!
//! ## Usage
//!
//! Compare independent selection controls and their optional icons.

use tessera_components::{
    alignment::CrossAxisAlignment,
    button::button,
    checkbox::checkbox,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    switch::switch,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Dp, FocusRequester, Modifier, RenderSlot, State, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
    ACCENT, INK,
    article::SpecSection,
    gap, panel,
    selection_articles::{Kind, option_label, selection_article},
    spec_table, type_style,
};

#[path = "switch_states.rs"]
mod switch_states;

#[path = "checkbox_states.rs"]
mod checkbox_states;

#[path = "selection_pointer.rs"]
mod pointer;

#[tessera]
pub(super) fn checkbox_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    selection_article(
        Kind::Checkbox,
        vec![
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(|| {
                    tokens(false);
                }),
            },
            SpecSection {
                title: "ui-checkbox",
                content: RenderSlot::new(checkbox_anatomy),
            },
            SpecSection {
                title: "ui-color",
                content: RenderSlot::new(|| {
                    colors(false);
                }),
            },
            SpecSection {
                title: "ui-states",
                content: RenderSlot::new(|| {
                    states(false);
                }),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(|| {
                    measurements(false);
                }),
            },
        ],
        RenderSlot::new(|| {
            settings(false);
        }),
        RenderSlot::new(checkbox_anatomy),
    )
    .route_optional(route);
}

#[tessera]
pub(super) fn switch_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    selection_article(
        Kind::Switch,
        vec![
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(|| {
                    tokens(true);
                }),
            },
            SpecSection {
                title: "ui-color",
                content: RenderSlot::new(|| {
                    colors(true);
                }),
            },
            SpecSection {
                title: "ui-states",
                content: RenderSlot::new(|| {
                    states(true);
                }),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(|| {
                    measurements(true);
                }),
            },
            SpecSection {
                title: "ui-configurations",
                content: RenderSlot::new(switch_figures::configurations),
            },
        ],
        RenderSlot::new(|| {
            settings(true);
        }),
        RenderSlot::new(switch_figures::anatomy),
    )
    .route_optional(route);
}

#[tessera]
pub(super) fn selection_control(
    is_switch: bool,
    initial: bool,
    indeterminate: bool,
    enabled: bool,
    icons: usize,
    #[prop(into)] label: String,
    state: Option<State<bool>>,
    focus_requester: Option<FocusRequester>,
) {
    let checked = if let Some(state) = state {
        state
    } else {
        remember(|| initial)
    };
    let mixed = remember(|| indeterminate);
    if is_switch {
        let show_icon = icons == 2 || (icons == 1 && checked.get());
        switch()
            .focus_requester_optional(focus_requester)
            .checked(checked.get())
            .enabled(enabled)
            .accessibility_label(I18N.text(label))
            .on_toggle(move |value| checked.set(value))
            .child_optional(if show_icon {
                Some(RenderSlot::new(move || {
                    icon().painter(if checked.get() {
                        filled::CHECK_SVG
                    } else {
                        filled::CLOSE_SVG
                    });
                }))
            } else {
                None
            });
    } else {
        checkbox()
            .focus_requester_optional(focus_requester)
            .checked(checked.get())
            .indeterminate(mixed.get())
            .disabled(!enabled)
            .accessibility_label(I18N.text(label))
            .on_toggle(move |value| {
                checked.set(value);
                mixed.set(false);
            });
    }
}

#[tessera]
fn setting_row(is_switch: bool, initial: bool, #[prop(into)] label: String) {
    let checked = remember(|| initial);
    let focus = remember(FocusRequester::new).get();
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().height(Dp(72.0)))
        .children(move || {
            option_label(
                label.clone(),
                Callback::new(move || {
                    checked.with_mut(|checked| *checked = !*checked);
                    focus.request_focus();
                }),
                true,
            )
            .style(type_style(20.0, 28.0))
            .color(scheme.on_surface);
            spacer().modifier(Modifier::new().weight(1.0));
            selection_control(
                is_switch,
                initial,
                false,
                true,
                usize::from(is_switch),
                label.clone(),
            )
            .state(checked)
            .focus_requester(focus);
        });
}

#[tessera]
fn settings(is_switch: bool) {
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().width(Dp(400.0)))
            .children(move || {
                text(I18N.text(if is_switch {
                    "ui-settings"
                } else {
                    "ui-allow-access"
                }))
                .style(type_style(24.0, 32.0))
                .color(INK);
                gap(Dp(24.0));
                if is_switch {
                    setting_row(true, true, "Wi-Fi");
                    setting_row(true, false, "ui-bluetooth");
                } else {
                    setting_row(false, true, "ui-microphone-access");
                    setting_row(false, true, "ui-location-access");
                    setting_row(false, false, "ui-haptics");
                }
            });
    }));
}

fn group_checked(items: State<[bool; 3]>, index: Option<usize>) -> bool {
    items.with(|items| {
        index.map_or_else(
            || items.iter().all(|selected| *selected),
            |index| items[index],
        )
    })
}

fn set_group_value(items: State<[bool; 3]>, index: Option<usize>, value: bool) {
    items.with_mut(|items| {
        if let Some(index) = index {
            items[index] = value;
        } else {
            items.fill(value);
        }
    });
}

#[tessera]
pub(super) fn checkbox_group() {
    let items = remember(|| [true, false, false]);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().width(Dp(400.0)).semantics(SemanticsArgs {
                role: Some(tessera_ui::accesskit::Role::Group),
                label: Some(I18N.text("ui-related-checkbox-choices")),
                merge_descendants: true,
                ..Default::default()
            }))
            .children(move || {
                checkbox_group_row(items, "ui-all-items".into());
                column()
                    .modifier(Modifier::new().padding_symmetric(Dp(24.0), Dp::ZERO))
                    .children(move || {
                        for (index, label) in ["ui-documents", "ui-photos", "ui-videos"]
                            .into_iter()
                            .enumerate()
                        {
                            checkbox_group_row(items, label.into()).index(index);
                        }
                    });
                gap(Dp(16.0));
                text(
                    I18N.format(
                        "ui-selected-items-value",
                        [(
                            "v0",
                            (items
                                .with(|items| items.iter().filter(|selected| **selected).count()))
                            .into(),
                        )],
                    ),
                )
                .style(type_style(16.0, 24.0));
            });
    }));
}

#[tessera]
fn checkbox_group_row(items: State<[bool; 3]>, index: Option<usize>, label: String) {
    let focus = remember(FocusRequester::new).get();
    let checked = group_checked(items, index);
    let mixed =
        index.is_none() && !checked && items.with(|items| items.iter().any(|selected| *selected));
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            checkbox()
                .checked(checked)
                .indeterminate(mixed)
                .accessibility_label(I18N.text(&(label)))
                .focus_requester(focus)
                .on_toggle(move |next| set_group_value(items, index, next));
            option_label(
                label.clone(),
                Callback::new(move || {
                    set_group_value(items, index, !group_checked(items, index));
                    focus.request_focus();
                }),
                true,
            );
        });
}

#[tessera]
fn tokens(is_switch: bool) {
    let dimensions = remember(|| false);
    flow_row().item_spacing(Dp(16.0)).children(move || {
        button()
            .text()
            .on_click(move || dimensions.set(false))
            .child(move || {
                text(I18N.text("ui-color-tokens"));
            });
        button()
            .text()
            .on_click(move || dimensions.set(true))
            .child(move || {
                text(I18N.text("ui-size-tokens"));
            });
    });
    gap(Dp(24.0));
    let rows = if dimensions.get() {
        if is_switch {
            vec![
                ["ui-track-height", "md.comp.switch.track.height", "32dp"],
                ["ui-track-width", "md.comp.switch.track.width", "52dp"],
                ["ui-handle-height", "md.comp.switch.handle.height", "16dp"],
                [
                    "ui-selected-handle-height",
                    "md.comp.switch.selected.handle.height",
                    "24dp",
                ],
                [
                    "ui-pressed-handle-height",
                    "md.comp.switch.pressed.handle.height",
                    "28dp",
                ],
                ["ui-icon-size", "md.comp.switch.icon.size", "16dp"],
                [
                    "ui-state-layer-size",
                    "md.comp.switch.state-layer.size",
                    "40dp",
                ],
            ]
        } else {
            vec![
                [
                    "ui-container-size",
                    "md.comp.checkbox.container.size",
                    "18dp",
                ],
                [
                    "ui-container-shape",
                    "md.comp.checkbox.container.shape",
                    "ui-2dp-corner",
                ],
                ["ui-icon-size", "md.comp.checkbox.icon.size", "18dp"],
                [
                    "ui-state-layer-size",
                    "md.comp.checkbox.state-layer.size",
                    "40dp",
                ],
                [
                    "ui-target-size",
                    "md.comp.checkbox.touch-target.size",
                    "48dp",
                ],
            ]
        }
    } else if is_switch {
        vec![
            [
                "ui-selected-track",
                "md.comp.switch.selected.track.color",
                "Primary",
            ],
            [
                "ui-unselected-track",
                "md.comp.switch.unselected.track.color",
                "Surface container highest",
            ],
            [
                "ui-selected-handle",
                "md.comp.switch.selected.handle.color",
                "On primary",
            ],
            [
                "ui-unselected-handle",
                "md.comp.switch.unselected.handle.color",
                "Outline",
            ],
            [
                "ui-selected-icon",
                "md.comp.switch.selected.icon.color",
                "On primary container",
            ],
            [
                "ui-unselected-icon",
                "md.comp.switch.unselected.icon.color",
                "Surface container highest",
            ],
        ]
    } else {
        vec![
            [
                "ui-selected-container",
                "md.comp.checkbox.selected.container.color",
                "Primary",
            ],
            [
                "ui-selected-icon",
                "md.comp.checkbox.selected.icon.color",
                "On primary",
            ],
            [
                "ui-unselected-outline",
                "md.comp.checkbox.unselected.outline.color",
                "On surface variant",
            ],
            [
                "ui-state-layer",
                "md.comp.checkbox.state-layer.color",
                "On surface",
            ],
        ]
    };
    spec_table(
        vec!["ui-element".into(), "ui-token".into(), "ui-value".into()],
        rows.into_iter()
            .map(|row| row.map(str::to_owned).to_vec())
            .collect(),
    );
}

#[tessera]
fn checkbox_anatomy() {
    panel(RenderSlot::new(move || {
        column().children(move || {
            flow_row()
                .item_spacing(Dp(48.0))
                .line_spacing(Dp(16.0))
                .children(|| {
                    selection_control(false, false, false, true, 0, "ui-unselected-checkbox");
                    selection_control(false, true, false, true, 0, "ui-selected-checkbox");
                    selection_control(false, false, true, true, 0, "ui-indeterminate-checkbox");
                });
            gap(Dp(24.0));
            text(I18N.text("ui-1-container")).style(type_style(16.0, 28.0));
            text(I18N.text("ui-2-icon")).style(type_style(16.0, 28.0));
        });
    }));
}

#[tessera]
fn color_scheme(is_switch: bool, dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            surface()
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(32.0)))
                        .children(move || {
                            text(I18N.text(if dark {
                                "ui-dark-scheme"
                            } else {
                                "ui-light-scheme"
                            }))
                            .style(type_style(24.0, 32.0));
                            gap(Dp(24.0));
                            setting_row(is_switch, true, "ui-selected");
                            setting_row(is_switch, false, "ui-unselected");
                        });
                });
        });
}

#[tessera]
fn colors(is_switch: bool) {
    color_scheme(is_switch, false);
    gap(Dp(24.0));
    color_scheme(is_switch, true);
    gap(Dp(32.0));
    super::subheading("ui-adjacent-text-label-color".into());
    gap(Dp(16.0));
    text(I18N.text(if is_switch {
            "ui-use-on-surface-for-the-setting-name-and-on-surface-variant-for-supporting-text-keep-these"
        } else {
            "ui-use-on-surface-for-adjacent-option-labels-selection-changes-the-checkbox-while-the-label-k"
        }))
        .style(type_style(16.0, 24.0))
        .color(INK);
    gap(Dp(24.0));
    selection_labels::figure(is_switch);
}

#[tessera]
fn states(is_switch: bool) {
    if is_switch {
        switch_states::states();
    } else {
        checkbox_states::states();
    }
}

#[tessera]
fn measurement_table(is_switch: bool) {
    if is_switch {
        let rows = [
            ("ui-track", "ui-height", "32dp", 4),
            ("", "ui-width", "52dp", 1),
            ("", "ui-outline-width", "2dp", 1),
            ("", "ui-shape", "md.sys.shape.corner.full", 1),
            ("ui-handle", "ui-height-unselected", "16dp", 9),
            ("", "ui-height-with-icon", "24dp", 1),
            ("", "ui-height-selected", "24dp", 1),
            ("", "ui-height-pressed", "28dp", 1),
            ("", "ui-width-unselected", "16dp", 1),
            ("", "ui-width-with-icon", "24dp", 1),
            ("", "ui-width-selected", "24dp", 1),
            ("", "ui-width-pressed", "28dp", 1),
            ("", "ui-shape", "md.sys.shape.corner.full", 1),
            ("ui-state-layer", "ui-size", "40dp", 2),
            ("", "ui-shape", "md.sys.shape.corner.full", 1),
            ("ui-target", "ui-size", "48dp", 1),
            ("ui-icon", "ui-size-selected", "16dp", 1),
            ("ui-icon", "ui-size-unselected", "16dp", 1),
        ];
        let cells = rows
            .into_iter()
            .enumerate()
            .flat_map(|(index, (element, attribute, value, span))| {
                [element, attribute, value]
                    .into_iter()
                    .enumerate()
                    .filter(|(column, text)| *column != 0 || !text.is_empty())
                    .map(
                        move |(column, text)| tessera_components::text_table::TextTableCell {
                            text: text.into(),
                            row: index + 1,
                            column,
                            row_span: if column == 0 { span } else { 1 },
                            ..Default::default()
                        },
                    )
            })
            .collect::<Vec<_>>();
        super::spec_table_cells(
            ["ui-element", "ui-attribute", "ui-value"]
                .map(str::to_owned)
                .to_vec(),
            cells,
        );
    } else {
        spec_table(
            vec!["ui-attribute".into(), "ui-value".into()],
            vec![
                vec!["ui-container".into(), "18 × 18dp".into()],
                vec!["ui-corner-radius".into(), "2dp".into()],
                vec!["ui-icon".into(), "ui-18dp-centered".into()],
                vec!["ui-state-layer".into(), "40dp".into()],
                vec!["ui-target".into(), "48 × 48dp".into()],
            ],
        );
    }
}

#[tessera]
fn measurements(is_switch: bool) {
    panel(RenderSlot::new(move || {
        column().children(move || {
            surface()
                .style(SurfaceStyle::Outlined {
                    color: ACCENT,
                    width: Dp(1.0),
                })
                .modifier(
                    Modifier::new().size(if is_switch { Dp(52.0) } else { Dp(48.0) }, Dp(48.0)),
                )
                .child(move || {
                    selection_control(is_switch, true, false, true, 0, "ui-measured-control");
                });
            gap(Dp(24.0));
            text(I18N.text(if is_switch {
                "ui-52-48dp-interaction-target"
            } else {
                "ui-48-48dp-interaction-target"
            }))
            .style(type_style(16.0, 24.0));
        });
    }));
    gap(Dp(32.0));
    measurement_table(is_switch);
}

#[path = "switch_figures.rs"]
mod switch_figures;

#[path = "selection_labels.rs"]
mod selection_labels;
