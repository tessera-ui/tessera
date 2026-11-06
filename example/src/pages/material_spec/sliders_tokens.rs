use tessera_components::{
    alignment::CrossAxisAlignment,
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    menus::{MenuController, MenuItemSelectionMode, menu_item, menu_provider},
    modifier::ModifierExt as _,
    row::row,
    text::text,
};
use tessera_ui::{
    Dp, LayoutCoordinates, Modifier, State, modifier::FocusModifierExt as _, remember, tessera,
};

use crate::res::I18N;

use super::{gap, spec_table, subheading, type_style};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "sliders_tokens_tests.rs"]
mod tests;

const SET_NAMES: [&str; 6] = [
    "ui-slider",
    "ui-slider-size-xsmall",
    "ui-slider-size-small",
    "ui-slider-size-medium",
    "ui-slider-size-large",
    "ui-slider-size-xlarge",
];

const GROUPS: [&[&str]; 6] = [
    &[
        "ui-enabled",
        "ui-disabled",
        "ui-hovered",
        "ui-focused",
        "ui-pressed-ripple",
    ],
    &["ui-active-track", "ui-inactive-track", "ui-handle"],
    &["ui-active-track", "ui-inactive-track", "ui-handle"],
    &[
        "ui-active-track",
        "ui-inactive-track",
        "ui-handle",
        "ui-icon",
    ],
    &[
        "ui-active-track",
        "ui-inactive-track",
        "ui-handle",
        "ui-icon",
    ],
    &[
        "ui-active-track",
        "ui-inactive-track",
        "ui-handle",
        "ui-icon",
    ],
];

struct Token {
    group: &'static str,
    section: &'static str,
    name: &'static str,
    value: &'static str,
}

#[tessera]
pub(super) fn tokens() {
    let selected = remember(|| 0_usize);
    let expanded = remember(|| [[false; 5]; 6]);
    let controller = remember(MenuController::new);
    let anchor = remember(LayoutCoordinates::default);
    token_menu(selected, expanded, controller, anchor);
}

#[tessera]
fn token_menu(
    selected: State<usize>,
    expanded: State<[[bool; 5]; 6]>,
    controller: State<MenuController>,
    anchor: State<LayoutCoordinates>,
) {
    menu_provider()
        .controller(controller)
        .window_anchor(anchor)
        .modifier(Modifier::new().fill_max_width())
        .main_content(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    text(I18N.text(
                        "ui-switch-between-common-slider-tokens-and-the-five-size-token-sets",
                    ))
                    .style(type_style(16.0, 24.0));
                    gap(Dp(16.0));
                    flow_row()
                        .item_spacing(Dp(16.0))
                        .line_spacing(Dp(8.0))
                        .children(move || {
                            button()
                                .outlined()
                                .accessibility_label(I18N.text("ui-slider-token-set-selector"))
                                .modifier(Modifier::new().on_layout_coordinates(
                                    move |coordinates| {
                                        anchor.set_if_changed(coordinates);
                                    },
                                ))
                                .on_click(move || controller.with_mut(MenuController::open))
                                .child(move || {
                                    row()
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .children(move || {
                                            text(I18N.text(SET_NAMES[selected.get()]));
                                            boxed().modifier(Modifier::new().width(Dp(8.0)));
                                            icon()
                                                .painter(filled::ARROW_DROP_DOWN_SVG)
                                                .size(Dp(18.0));
                                        });
                                });
                            button()
                                .text()
                                .accessibility_label(I18N.text(
                                    if expanded.with(|state| {
                                        state[selected.get()][..GROUPS[selected.get()].len()]
                                            .iter()
                                            .all(|open| *open)
                                    }) {
                                        "ui-collapse-all"
                                    } else {
                                        "ui-expand-all"
                                    },
                                ))
                                .on_click(move || {
                                    let index = selected.get();
                                    expanded.with_mut(|state| {
                                        let open = !state[index][..GROUPS[index].len()]
                                            .iter()
                                            .all(|open| *open);
                                        state[index].fill(open);
                                    });
                                })
                                .child(move || {
                                    let index = selected.get();
                                    let all_open = expanded.with(|state| {
                                        state[index][..GROUPS[index].len()].iter().all(|open| *open)
                                    });
                                    text(I18N.text(if all_open {
                                        "ui-collapse-all"
                                    } else {
                                        "ui-expand-all"
                                    }));
                                });
                        });
                    gap(Dp(24.0));
                    let index = selected.get();
                    for (group_index, group) in GROUPS[index].iter().copied().enumerate() {
                        let open = expanded.with(|state| state[index][group_index]);
                        button()
                            .text()
                            .accessibility_label(I18N.text(group))
                            .on_click(move || {
                                let index = selected.get();
                                expanded.with_mut(|state| {
                                    state[index][group_index] = !state[index][group_index]
                                });
                            })
                            .child(move || {
                                let index = selected.get();
                                let open = expanded.with(|state| state[index][group_index]);
                                let group = GROUPS[index][group_index];
                                text(I18N.format(
                                    "ui-value-value",
                                    [
                                        ("v0", I18N.text(&(if open { "−" } else { "+" })).into()),
                                        ("v1", I18N.text(&group).into()),
                                    ],
                                ));
                            });
                        if open {
                            token_table(index, group.to_owned());
                        }
                    }
                });
        })
        .menu_content(move |menu| {
            for (index, label) in SET_NAMES.into_iter().enumerate() {
                menu_item()
                    .menu(menu)
                    .label(I18N.text(label))
                    .selected(selected.get() == index)
                    .selection_mode(MenuItemSelectionMode::Radio)
                    .on_click(move || {
                        selected.set(index);
                    });
            }
        });
}

#[tessera]
fn token_table(index: usize, group: String) {
    let mut sections = Vec::new();
    for token in TOKEN_SETS[index] {
        if token.group == group && !sections.contains(&token.section) {
            sections.push(token.section);
        }
    }
    for section in sections {
        if index == 0 {
            subheading(section.to_owned());
        }
        let rows = TOKEN_SETS[index]
            .iter()
            .filter(|token| token.group == group && token.section == section)
            .map(|token| vec![token.name.to_owned(), token.value.to_owned()])
            .collect();
        spec_table(vec!["ui-token".into(), "ui-value".into()], rows);
        gap(Dp(16.0));
    }
}

const TOKEN_SETS: [&[Token]; 6] = [
    &[
        Token {
            group: "ui-disabled",
            section: "ui-stop-indicator",
            name: "ui-slider-disabled-active-stop-indicator-container-color-md-comp-slider-disabled-active",
            value: "Inverse on surface",
        },
        Token {
            group: "ui-disabled",
            section: "ui-stop-indicator",
            name: "ui-slider-disabled-inactive-stop-indicator-container-color-md-comp-slider-disabled-inact",
            value: "On surface",
        },
        Token {
            group: "ui-disabled",
            section: "ui-track",
            name: "ui-slider-disabled-active-track-color-md-comp-slider-disabled-active-track-color",
            value: "On surface",
        },
        Token {
            group: "ui-disabled",
            section: "ui-track",
            name: "ui-slider-disabled-active-track-opacity-md-comp-slider-disabled-active-track-opacity",
            value: "38%",
        },
        Token {
            group: "ui-disabled",
            section: "ui-track",
            name: "ui-slider-disabled-inactive-track-color-md-comp-slider-disabled-inactive-track-color",
            value: "On surface",
        },
        Token {
            group: "ui-disabled",
            section: "ui-track",
            name: "ui-slider-disabled-inactive-track-opacity-md-comp-slider-disabled-inactive-track-opacity",
            value: "12%",
        },
        Token {
            group: "ui-disabled",
            section: "ui-handle",
            name: "ui-slider-disabled-handle-color-md-comp-slider-disabled-handle-color",
            value: "On surface",
        },
        Token {
            group: "ui-disabled",
            section: "ui-handle",
            name: "ui-slider-disabled-handle-opacity-md-comp-slider-disabled-handle-opacity",
            value: "38%",
        },
        Token {
            group: "ui-disabled",
            section: "ui-handle",
            name: "ui-slider-disabled-handle-width-md-comp-slider-disabled-handle-width",
            value: "4dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-stop-indicator",
            name: "ui-slider-stop-indicator-size-md-comp-slider-stop-indicator-size",
            value: "4dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-stop-indicator",
            name: "ui-slider-stop-indicator-shape-md-comp-slider-stop-indicator-shape",
            value: "ui-full",
        },
        Token {
            group: "ui-enabled",
            section: "ui-stop-indicator",
            name: "ui-slider-stop-indicator-trailing-space-md-comp-slider-stop-indicator-trailing-space",
            value: "4dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-stop-indicator",
            name: "ui-slider-stop-indicator-color-md-comp-slider-stop-indicator-color",
            value: "On secondary container",
        },
        Token {
            group: "ui-enabled",
            section: "ui-stop-indicator",
            name: "ui-slider-stop-indicator-color-selected-md-comp-slider-stop-indicator-color-selected",
            value: "On primary",
        },
        Token {
            group: "ui-enabled",
            section: "ui-container",
            name: "ui-slider-active-stop-indicator-container-color-md-comp-slider-active-stop-indicator-con",
            value: "On primary",
        },
        Token {
            group: "ui-enabled",
            section: "ui-container",
            name: "ui-slider-active-stop-indicator-container-opacity-md-comp-slider-active-stop-indicator-c",
            value: "100%",
        },
        Token {
            group: "ui-enabled",
            section: "ui-container",
            name: "ui-slider-inactive-stop-indicator-container-color-md-comp-slider-inactive-stop-indicator",
            value: "On secondary container",
        },
        Token {
            group: "ui-enabled",
            section: "ui-container",
            name: "ui-slider-inactive-stop-indicator-container-opacity-md-comp-slider-inactive-stop-indicat",
            value: "100%",
        },
        Token {
            group: "ui-enabled",
            section: "ui-track",
            name: "ui-slider-active-track-height-md-comp-slider-active-track-height",
            value: "16dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-track",
            name: "ui-slider-inactive-track-height-md-comp-slider-inactive-track-height",
            value: "16dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-track",
            name: "ui-slider-active-track-shape-md-comp-slider-active-track-shape",
            value: "ui-full",
        },
        Token {
            group: "ui-enabled",
            section: "ui-track",
            name: "ui-slider-active-track-outer-corner-size-md-comp-slider-active-track-outer-corner-corner",
            value: "ui-full",
        },
        Token {
            group: "ui-enabled",
            section: "ui-track",
            name: "ui-slider-active-track-inner-corner-size-md-comp-slider-active-track-inner-corner-corner",
            value: "0dp 2dp 2dp 0dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-track",
            name: "ui-slider-inactive-track-shape-md-comp-slider-inactive-track-shape",
            value: "ui-full",
        },
        Token {
            group: "ui-enabled",
            section: "ui-track",
            name: "ui-slider-active-track-color-md-comp-slider-active-track-color",
            value: "Primary",
        },
        Token {
            group: "ui-enabled",
            section: "ui-track",
            name: "ui-slider-inactive-track-color-md-comp-slider-inactive-track-color",
            value: "Secondary container",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-handle-height-md-comp-slider-handle-height",
            value: "44dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-handle-width-md-comp-slider-handle-width",
            value: "4dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-handle-shape-md-comp-slider-handle-shape",
            value: "ui-full",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-handle-color-md-comp-slider-handle-color",
            value: "Primary",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-active-handle-color-md-comp-slider-active-handle-color",
            value: "Primary",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-active-handle-height-md-comp-slider-active-handle-height",
            value: "44dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-active-handle-width-md-comp-slider-active-handle-width",
            value: "4dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-active-handle-shape-md-comp-slider-active-handle-shape",
            value: "ui-full",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-active-handle-leading-space-md-comp-slider-active-handle-leading-space",
            value: "6dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-active-handle-trailing-space-md-comp-slider-active-handle-trailing-space",
            value: "6dp",
        },
        Token {
            group: "ui-enabled",
            section: "ui-handle",
            name: "ui-slider-active-handle-padding-md-comp-slider-active-handle-padding",
            value: "6dp",
        },
        Token {
            group: "ui-focused",
            section: "ui-track",
            name: "ui-slider-focus-active-track-color-md-comp-slider-focus-active-track-color",
            value: "Primary",
        },
        Token {
            group: "ui-focused",
            section: "ui-track",
            name: "ui-slider-focus-inactive-track-color-md-comp-slider-focus-inactive-track-color",
            value: "Secondary container",
        },
        Token {
            group: "ui-focused",
            section: "ui-handle",
            name: "ui-slider-focus-handle-width-md-comp-slider-focus-handle-width",
            value: "2dp",
        },
        Token {
            group: "ui-hovered",
            section: "ui-handle",
            name: "ui-slider-hover-handle-width-md-comp-slider-hover-handle-width",
            value: "4dp",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-handle",
            name: "ui-slider-pressed-handle-color-md-comp-slider-pressed-handle-color",
            value: "Primary",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-handle",
            name: "ui-slider-pressed-handle-width-md-comp-slider-pressed-handle-width",
            value: "2dp",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-track",
            name: "ui-slider-pressed-active-track-color-md-comp-slider-pressed-active-track-color",
            value: "Primary",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-track",
            name: "ui-slider-pressed-inactive-track-color-md-comp-slider-pressed-inactive-track-color",
            value: "Secondary container",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-value-indicator",
            name: "ui-slider-value-indicator-container-color-md-comp-slider-value-indicator-container-color",
            value: "Inverse surface",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-value-indicator",
            name: "ui-slider-value-indicator-label-font-md-comp-slider-value-indicator-label-label-text-fon",
            value: "md.sys.typescale.label-large.font",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-value-indicator",
            name: "ui-slider-value-indicator-label-font-color-md-comp-slider-value-indicator-label-label-te",
            value: "Inverse on surface",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-value-indicator",
            name: "ui-slider-value-indicator-label-line-height-md-comp-slider-value-indicator-label-label-t",
            value: "md.sys.typescale.label-large.line-height",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-value-indicator",
            name: "ui-slider-value-indicator-label-size-md-comp-slider-value-indicator-label-label-text-siz",
            value: "md.sys.typescale.label-large.size",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-value-indicator",
            name: "ui-slider-value-indicator-label-tracking-md-comp-slider-value-indicator-label-label-text",
            value: "md.sys.typescale.body-large.tracking",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-value-indicator",
            name: "ui-slider-value-indicator-label-weight-md-comp-slider-value-indicator-label-label-text-w",
            value: "md.sys.typescale.body-large.weight",
        },
        Token {
            group: "ui-pressed-ripple",
            section: "ui-value-indicator",
            name: "ui-slider-value-indicator-active-bottom-space-md-comp-slider-value-indicator-active-bott",
            value: "12dp",
        },
    ],
    &[
        Token {
            group: "ui-active-track",
            section: "ui-active-track",
            name: "ui-slider-xsmall-active-track-height-md-comp-slider-xsmall-active-track-height",
            value: "16dp",
        },
        Token {
            group: "ui-active-track",
            section: "ui-active-track",
            name: "ui-slider-xsmall-active-track-leading-shape-md-comp-slider-xsmall-active-track-shape-lea",
            value: "8dp",
        },
        Token {
            group: "ui-handle",
            section: "ui-handle",
            name: "ui-slider-xsmall-active-handle-height-md-comp-slider-xsmall-active-handle-height",
            value: "44dp",
        },
        Token {
            group: "ui-inactive-track",
            section: "ui-inactive-track",
            name: "ui-slider-xsmall-inactive-track-height-md-comp-slider-xsmall-inactive-track-height",
            value: "16dp",
        },
        Token {
            group: "ui-inactive-track",
            section: "ui-inactive-track",
            name: "ui-slider-xsmall-inactive-track-trailing-shape-md-comp-slider-xsmall-inactive-track-shap",
            value: "8dp",
        },
    ],
    &[
        Token {
            group: "ui-active-track",
            section: "ui-active-track",
            name: "ui-slider-small-active-track-height-md-comp-slider-small-active-track-height",
            value: "24dp",
        },
        Token {
            group: "ui-active-track",
            section: "ui-active-track",
            name: "ui-slider-small-active-track-leading-shape-md-comp-slider-small-active-track-shape-leadi",
            value: "8dp",
        },
        Token {
            group: "ui-handle",
            section: "ui-handle",
            name: "ui-slider-small-active-handle-height-md-comp-slider-small-active-handle-height",
            value: "44dp",
        },
        Token {
            group: "ui-inactive-track",
            section: "ui-inactive-track",
            name: "ui-slider-small-inactive-track-height-md-comp-slider-small-inactive-track-height",
            value: "24dp",
        },
        Token {
            group: "ui-inactive-track",
            section: "ui-inactive-track",
            name: "ui-slider-small-inactive-track-trailing-shape-md-comp-slider-small-inactive-track-shape",
            value: "8dp",
        },
    ],
    &[
        Token {
            group: "ui-active-track",
            section: "ui-active-track",
            name: "ui-slider-medium-active-track-height-md-comp-slider-medium-active-track-height",
            value: "40dp",
        },
        Token {
            group: "ui-active-track",
            section: "ui-active-track",
            name: "ui-slider-medium-active-track-leading-shape-md-comp-slider-medium-active-track-shape-lea",
            value: "12dp",
        },
        Token {
            group: "ui-handle",
            section: "ui-handle",
            name: "ui-slider-medium-active-handle-height-md-comp-slider-medium-active-handle-height",
            value: "44dp",
        },
        Token {
            group: "ui-icon",
            section: "ui-icon",
            name: "ui-slider-medium-icon-padding-md-comp-slider-medium-icon-padding",
            value: "6dp",
        },
        Token {
            group: "ui-icon",
            section: "ui-icon",
            name: "ui-slider-medium-icon-size-md-comp-slider-medium-icon-size",
            value: "24dp",
        },
        Token {
            group: "ui-inactive-track",
            section: "ui-inactive-track",
            name: "ui-slider-medium-inactive-track-height-md-comp-slider-medium-inactive-track-height",
            value: "40dp",
        },
        Token {
            group: "ui-inactive-track",
            section: "ui-inactive-track",
            name: "ui-slider-medium-inactive-track-trailing-shape-md-comp-slider-medium-inactive-track-shap",
            value: "12dp",
        },
    ],
    &[
        Token {
            group: "ui-active-track",
            section: "ui-active-track",
            name: "ui-slider-large-active-track-height-md-comp-slider-large-active-track-height",
            value: "56dp",
        },
        Token {
            group: "ui-active-track",
            section: "ui-active-track",
            name: "ui-slider-large-active-track-leading-shape-md-comp-slider-large-active-track-shape-leadi",
            value: "16dp",
        },
        Token {
            group: "ui-handle",
            section: "ui-handle",
            name: "ui-slider-large-active-handle-height-md-comp-slider-large-active-handle-height",
            value: "68dp",
        },
        Token {
            group: "ui-icon",
            section: "ui-icon",
            name: "ui-slider-large-icon-padding-md-comp-slider-large-icon-padding",
            value: "6dp",
        },
        Token {
            group: "ui-icon",
            section: "ui-icon",
            name: "ui-slider-large-icon-size-md-comp-slider-large-icon-size",
            value: "24dp",
        },
        Token {
            group: "ui-inactive-track",
            section: "ui-inactive-track",
            name: "ui-slider-large-inactive-track-height-md-comp-slider-large-inactive-track-height",
            value: "56dp",
        },
        Token {
            group: "ui-inactive-track",
            section: "ui-inactive-track",
            name: "ui-slider-large-inactive-track-trailing-shape-md-comp-slider-large-inactive-track-shape",
            value: "16dp",
        },
    ],
    &[
        Token {
            group: "ui-active-track",
            section: "ui-active-track",
            name: "ui-slider-xlarge-active-track-height-md-comp-slider-xlarge-active-track-height",
            value: "96dp",
        },
        Token {
            group: "ui-active-track",
            section: "ui-active-track",
            name: "ui-slider-xlarge-active-track-leading-shape-md-comp-slider-xlarge-active-track-shape-lea",
            value: "28dp",
        },
        Token {
            group: "ui-handle",
            section: "ui-handle",
            name: "ui-slider-xlarge-active-handle-height-md-comp-slider-xlarge-active-handle-height",
            value: "108dp",
        },
        Token {
            group: "ui-icon",
            section: "ui-icon",
            name: "ui-slider-xlarge-icon-padding-md-comp-slider-xlarge-icon-padding",
            value: "8dp",
        },
        Token {
            group: "ui-icon",
            section: "ui-icon",
            name: "ui-slider-xlarge-icon-size-md-comp-slider-xlarge-icon-size",
            value: "32dp",
        },
        Token {
            group: "ui-inactive-track",
            section: "ui-inactive-track",
            name: "ui-slider-xlarge-inactive-track-height-md-comp-slider-xlarge-inactive-track-height",
            value: "96dp",
        },
        Token {
            group: "ui-inactive-track",
            section: "ui-inactive-track",
            name: "ui-slider-xlarge-inactive-track-trailing-shape-md-comp-slider-xlarge-inactive-track-shap",
            value: "28dp",
        },
    ],
];
