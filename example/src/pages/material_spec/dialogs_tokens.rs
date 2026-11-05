//! Dialog token sets for basic and full-screen dialogs.
//!
//! ## Usage
//!
//! Inspect the color, type, shape and state tokens of each dialog variant.

use tessera_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
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

use super::{gap, spec_table, type_style};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "dialogs_tokens_tests.rs"]
mod tests;

const SET_NAMES: [&str; 2] = ["ui-dialog-basic", "ui-dialog-full-screen"];
const GROUPS: [&str; 4] = [
    "ui-enabled",
    "ui-hovered",
    "ui-focused",
    "ui-pressed-ripple",
];

struct Token {
    group: &'static str,
    section: &'static str,
    name: &'static str,
    value: &'static str,
    note: &'static str,
}

const fn token(
    group: &'static str,
    section: &'static str,
    name: &'static str,
    value: &'static str,
) -> Token {
    Token {
        group,
        section,
        name,
        value,
        note: "",
    }
}

const fn inferred(
    group: &'static str,
    section: &'static str,
    name: &'static str,
    value: &'static str,
    note: &'static str,
) -> Token {
    Token {
        group,
        section,
        name,
        value,
        note,
    }
}

#[tessera]
pub(super) fn tokens() {
    let selected = remember(|| 0_usize);
    let expanded = remember(|| [[false; 4]; 2]);
    let controller = remember(MenuController::new);
    let anchor = remember(LayoutCoordinates::default);
    token_menu(selected, expanded, controller, anchor);
}

#[tessera]
fn token_menu(
    selected: State<usize>,
    expanded: State<[[bool; 4]; 2]>,
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
                    text()
                        .content(I18N.text("ui-select-a-component-variant-below-to-see-its-elements-attributes-tokens-and-their-values"))
                        .style(type_style(16.0, 24.0));
                    gap(Dp(16.0));
                    row()
                        .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            button()
                                .tonal()
                                .accessibility_label(I18N.text("ui-dialog-token-set-selector"))
                                .modifier(Modifier::new().on_layout_coordinates(move |coordinates| {
                                    anchor.set_if_changed(coordinates);
                                }))
                                .on_click(move || controller.with_mut(MenuController::open))
                                .child(move || {
                                    row()
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .children(move || {
                                            text().content(I18N.text(SET_NAMES[selected.get()]));
                                            boxed().modifier(Modifier::new().width(Dp(8.0)));
                                            icon()
                                                .vector(filled::ARROW_DROP_DOWN_SVG)
                                                .size(Dp(18.0));
                                        });
                                });
                            button()
                                .text()
                                .accessibility_label(I18N.text(if all_open(selected, expanded) {
                                    "ui-collapse-all"
                                } else {
                                    "ui-expand-all"
                                }))
                                .on_click(move || {
                                    let index = selected.get();
                                    let open = !all_open(selected, expanded);
                                    expanded.with_mut(|state| state[index].fill(open));
                                })
                                .child(move || {
                                    text().content(I18N.text(if all_open(selected, expanded) {
                                        "ui-collapse-all"
                                    } else {
                                        "ui-expand-all"
                                    }));
                                });
                        });
                    gap(Dp(8.0));
                    text()
                        .content(I18N.text("ui-token-values-shown-for-default-light"))
                        .style(type_style(14.0, 20.0));
                    gap(Dp(4.0));
                    text()
                        .content(I18N.text("ui-values-are-the-published-token-values-shape-elevation-and-type-style-tokens-are-published"))
                        .style(type_style(14.0, 20.0));
                    gap(Dp(16.0));
                    for (group_index, group) in GROUPS.into_iter().enumerate() {
                        let open = expanded.with(|state| state[selected.get()][group_index]);
                        button()
                            .text()
                            .accessibility_label(I18N.text(group))
                            .on_click(move || {
                                let index = selected.get();
                                expanded.with_mut(|state| {
                                    state[index][group_index] = !state[index][group_index];
                                });
                            })
                            .child(move || {
                                let open =
                                    expanded.with(|state| state[selected.get()][group_index]);
                                text().content(I18N.format("ui-value-value", [("v0", I18N.text(&(if open { "−" } else { "+" })).into()), ("v1", I18N.text(&group).into())]));
                            });
                        if open {
                            token_group(selected.get(), group_index);
                        }
                    }
                });
        })
        .menu_content(move |menu| {
            for (index, label) in SET_NAMES.into_iter().enumerate() {
                menu_item().menu(menu)
                    .label(I18N.text(label))
                    .selected(selected.get() == index)
                    .selection_mode(MenuItemSelectionMode::Radio)
                    .on_click(move || selected.set(index));
            }
        });
}

fn all_open(selected: State<usize>, expanded: State<[[bool; 4]; 2]>) -> bool {
    expanded.with(|state| state[selected.get()].iter().all(|open| *open))
}

#[tessera]
fn token_group(index: usize, group_index: usize) {
    let group = GROUPS[group_index];
    let set = TOKEN_SETS[index];
    let mut sections = Vec::new();
    for token in set {
        if token.group == group && !sections.contains(&token.section) {
            sections.push(token.section);
        }
    }
    for section in sections {
        gap(Dp(8.0));
        text()
            .content(I18N.format(
                "ui-value-value-format-31",
                [
                    ("v0", I18N.text(&group).into()),
                    ("v1", I18N.text(&section).into()),
                ],
            ))
            .style(type_style(14.0, 20.0));
        gap(Dp(8.0));
        let rows = set
            .iter()
            .filter(|token| token.group == group && token.section == section)
            .map(|token| {
                vec![
                    token.name.to_owned(),
                    token.value.to_owned(),
                    if token.note.is_empty() {
                        "\u{2014}".to_owned()
                    } else {
                        token.note.to_owned()
                    },
                ]
            })
            .collect();
        spec_table(
            vec![
                "ui-token".into(),
                "ui-value".into(),
                "ui-mapping-inferred".into(),
            ],
            rows,
        );
        gap(Dp(16.0));
    }
}

const TOKEN_SETS: [&[Token]; 2] = [BASIC, FULL_SCREEN];

const BASIC: &[Token] = &[
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-dialog-container-color",
        "#ECE6F0",
        "ui-surface-container-high-inferred-role",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-dialog-container-elevation",
        "ui-preview-only",
        "ui-level-3-6dp-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-dialog-container-surface-tint-layer-color-deprecated",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-dialog-container-shape",
        "ui-preview-only",
        "ui-extra-large-28dp-corner-radius-inferred",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-dialog-action-label-text-font",
        "ui-roboto",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-dialog-action-label-text-line-height",
        "20pt",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-dialog-action-label-text-size",
        "14pt",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-dialog-action-label-text-weight",
        "500",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-dialog-action-label-text-tracking",
        "0.1pt",
    ),
    inferred(
        "ui-enabled",
        "ui-label-text",
        "ui-label-text-type-style",
        "ui-preview-only",
        "ui-label-large-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-label-text",
        "ui-dialog-action-label-text-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token("ui-enabled", "ui-icon", "ui-dialog-icon-size", "24dp"),
    inferred(
        "ui-enabled",
        "ui-icon",
        "ui-dialog-icon-color",
        "#625B71",
        "ui-secondary-inferred-role",
    ),
    token(
        "ui-enabled",
        "ui-subhead",
        "ui-dialog-subhead-font-deprecated",
        "ui-roboto",
    ),
    token(
        "ui-enabled",
        "ui-subhead",
        "ui-dialog-subhead-line-height-deprecated",
        "32pt",
    ),
    token(
        "ui-enabled",
        "ui-subhead",
        "ui-dialog-subhead-size-deprecated",
        "24pt",
    ),
    token(
        "ui-enabled",
        "ui-subhead",
        "ui-dialog-subhead-weight-deprecated",
        "400",
    ),
    token(
        "ui-enabled",
        "ui-subhead",
        "ui-dialog-subhead-tracking-deprecated",
        "0pt",
    ),
    inferred(
        "ui-enabled",
        "ui-subhead",
        "ui-subhead-type-style-deprecated",
        "ui-preview-only",
        "ui-headline-small-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-subhead",
        "ui-dialog-subhead-color-deprecated",
        "#1D1B20",
        "ui-on-surface-inferred-role",
    ),
    token(
        "ui-enabled",
        "ui-headline",
        "ui-dialog-headline-font",
        "ui-roboto",
    ),
    token(
        "ui-enabled",
        "ui-headline",
        "ui-dialog-headline-line-height",
        "32pt",
    ),
    token(
        "ui-enabled",
        "ui-headline",
        "ui-dialog-headline-size",
        "24pt",
    ),
    token(
        "ui-enabled",
        "ui-headline",
        "ui-dialog-headline-weight",
        "400",
    ),
    token(
        "ui-enabled",
        "ui-headline",
        "ui-dialog-headline-tracking",
        "0pt",
    ),
    inferred(
        "ui-enabled",
        "ui-headline",
        "ui-headline-type-style",
        "ui-preview-only",
        "ui-headline-small-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-headline",
        "ui-dialog-headline-color",
        "#1D1B20",
        "ui-on-surface-inferred-role",
    ),
    inferred(
        "ui-enabled",
        "ui-divider",
        "ui-dialog-divider-color-deprecated",
        "#79747E",
        "ui-outline-inferred-role",
    ),
    token(
        "ui-enabled",
        "ui-divider",
        "ui-dialog-divider-height-deprecated",
        "1dp",
    ),
    token(
        "ui-enabled",
        "ui-supporting-text",
        "ui-dialog-supporting-text-font",
        "ui-roboto",
    ),
    token(
        "ui-enabled",
        "ui-supporting-text",
        "ui-dialog-supporting-text-line-height",
        "20pt",
    ),
    token(
        "ui-enabled",
        "ui-supporting-text",
        "ui-dialog-supporting-text-size",
        "14pt",
    ),
    token(
        "ui-enabled",
        "ui-supporting-text",
        "ui-dialog-supporting-text-weight",
        "400",
    ),
    token(
        "ui-enabled",
        "ui-supporting-text",
        "ui-dialog-supporting-text-tracking",
        "0.25pt",
    ),
    inferred(
        "ui-enabled",
        "ui-supporting-text",
        "ui-supporting-text-type-style",
        "ui-preview-only",
        "ui-body-medium-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-supporting-text",
        "ui-dialog-supporting-text-color",
        "#49454F",
        "ui-on-surface-variant-inferred-role",
    ),
    inferred(
        "ui-hovered",
        "ui-label-text",
        "ui-dialog-action-hover-label-text-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-hovered",
        "ui-state-layer",
        "ui-dialog-action-hover-state-layer-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token(
        "ui-hovered",
        "ui-state-layer",
        "ui-dialog-action-hover-state-layer-opacity",
        "0.08",
    ),
    inferred(
        "ui-focused",
        "ui-label-text",
        "ui-dialog-action-focus-label-text-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-focused",
        "ui-state-layer",
        "ui-dialog-action-focus-state-layer-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token(
        "ui-focused",
        "ui-state-layer",
        "ui-dialog-action-focus-state-layer-opacity",
        "0.1",
    ),
    inferred(
        "ui-pressed-ripple",
        "ui-label-text",
        "ui-dialog-action-pressed-label-text-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-pressed-ripple",
        "ui-state-layer",
        "ui-dialog-action-pressed-state-layer-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token(
        "ui-pressed-ripple",
        "ui-state-layer",
        "ui-dialog-action-pressed-state-layer-opacity",
        "0.1",
    ),
];

const FULL_SCREEN: &[Token] = &[
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-on-scroll-container-color",
        "#F3EDF7",
        "ui-surface-container-inferred-role",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-container-color",
        "#FEF7FF",
        "ui-surface-inferred-role",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-container-elevation",
        "ui-preview-only",
        "ui-level-0-0dp-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-container-shape",
        "ui-preview-only",
        "ui-none-0dp-corner-radius-inferred",
    ),
    token(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-header-container-height",
        "56dp",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-header-container-color",
        "#FEF7FF",
        "ui-surface-inferred-role",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-header-container-elevation",
        "ui-preview-only",
        "ui-level-0-0dp-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-header-container-surface-tint-layer-color-deprecated",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-header-on-scroll-container-elevation",
        "ui-preview-only",
        "ui-level-2-3dp-inferred",
    ),
    token(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-action-bar-container-height-deprecated",
        "56dp",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-action-bar-container-color-deprecated",
        "#FEF7FF",
        "ui-surface-inferred-role",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-action-bar-container-elevation-deprecated",
        "ui-preview-only",
        "ui-level-0-0dp-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-container",
        "ui-full-screen-dialog-action-bar-on-scroll-container-elevation-deprecated",
        "ui-preview-only",
        "ui-level-2-3dp-inferred",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-header-action-label-text-font",
        "ui-roboto",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-header-action-label-text-line-height",
        "20pt",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-header-action-label-text-size",
        "14pt",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-header-action-label-text-weight",
        "500",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-header-action-label-text-tracking",
        "0.1pt",
    ),
    inferred(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-header-action-label-text-type",
        "ui-preview-only",
        "ui-label-large-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-header-action-label-text-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-action-bar-label-text-font-deprecated",
        "ui-roboto",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-action-bar-label-text-line-height-deprecated",
        "20pt",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-action-bar-label-text-size-deprecated",
        "14pt",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-action-bar-label-text-weight-deprecated",
        "500",
    ),
    token(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-action-bar-label-text-tracking-deprecated",
        "0.1pt",
    ),
    inferred(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-action-bar-label-text-type-deprecated",
        "ui-preview-only",
        "ui-label-large-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-label-text",
        "ui-full-screen-dialog-action-bar-label-text-color-deprecated",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-enabled",
        "ui-icon",
        "ui-full-screen-dialog-header-icon-color",
        "#1D1B20",
        "ui-on-surface-inferred-role",
    ),
    token(
        "ui-enabled",
        "ui-icon",
        "ui-full-screen-dialog-header-icon-size",
        "24dp",
    ),
    inferred(
        "ui-enabled",
        "ui-headline",
        "ui-full-screen-dialog-header-headline-color",
        "#1D1B20",
        "ui-on-surface-inferred-role",
    ),
    token(
        "ui-enabled",
        "ui-headline",
        "ui-full-screen-dialog-header-headline-font",
        "ui-roboto",
    ),
    token(
        "ui-enabled",
        "ui-headline",
        "ui-full-screen-dialog-header-headline-line-height",
        "28pt",
    ),
    token(
        "ui-enabled",
        "ui-headline",
        "ui-full-screen-dialog-header-headline-size",
        "22pt",
    ),
    token(
        "ui-enabled",
        "ui-headline",
        "ui-full-screen-dialog-header-headline-weight",
        "400",
    ),
    token(
        "ui-enabled",
        "ui-headline",
        "ui-full-screen-dialog-header-headline-tracking",
        "0pt",
    ),
    inferred(
        "ui-enabled",
        "ui-headline",
        "ui-header-type-style",
        "ui-preview-only",
        "ui-title-large-inferred",
    ),
    inferred(
        "ui-enabled",
        "ui-divider",
        "ui-full-screen-dialog-divider-color-deprecated",
        "#E7E0EC",
        "ui-surface-variant-inferred-role",
    ),
    token(
        "ui-enabled",
        "ui-divider",
        "ui-full-screen-dialog-divider-height-deprecated",
        "1dp",
    ),
    inferred(
        "ui-hovered",
        "ui-label-text",
        "ui-full-screen-dialog-header-action-hover-label-text-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-hovered",
        "ui-label-text",
        "ui-full-screen-dialog-action-bar-hover-label-text-color-deprecated",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-hovered",
        "ui-state-layer",
        "ui-full-screen-dialog-header-action-hover-state-layer-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token(
        "ui-hovered",
        "ui-state-layer",
        "ui-full-screen-dialog-header-action-hover-state-layer-opacity",
        "0.08",
    ),
    inferred(
        "ui-hovered",
        "ui-state-layer",
        "ui-full-screen-dialog-action-bar-hover-state-layer-color-deprecated",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token(
        "ui-hovered",
        "ui-state-layer",
        "ui-full-screen-dialog-action-bar-hover-state-layer-opacity-deprecated",
        "0.08",
    ),
    inferred(
        "ui-focused",
        "ui-label-text",
        "ui-full-screen-dialog-header-action-focus-label-text-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-focused",
        "ui-label-text",
        "ui-full-screen-dialog-action-bar-focus-label-text-color-deprecated",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-focused",
        "ui-state-layer",
        "ui-full-screen-dialog-header-action-focus-state-layer-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token(
        "ui-focused",
        "ui-state-layer",
        "ui-full-screen-dialog-header-action-focus-state-layer-opacity",
        "0.1",
    ),
    inferred(
        "ui-focused",
        "ui-state-layer",
        "ui-full-screen-dialog-action-bar-focus-state-layer-color-deprecated",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token(
        "ui-focused",
        "ui-state-layer",
        "ui-full-screen-dialog-action-bar-focus-state-layer-opacity-deprecated",
        "0.1",
    ),
    inferred(
        "ui-pressed-ripple",
        "ui-label-text",
        "ui-full-screen-dialog-header-action-pressed-label-text-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-pressed-ripple",
        "ui-label-text",
        "ui-full-screen-dialog-action-bar-pressed-label-text-color-deprecated",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    inferred(
        "ui-pressed-ripple",
        "ui-state-layer",
        "ui-full-screen-dialog-header-action-pressed-state-layer-color",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token(
        "ui-pressed-ripple",
        "ui-state-layer",
        "ui-full-screen-dialog-header-action-pressed-state-layer-opacity",
        "0.1",
    ),
    inferred(
        "ui-pressed-ripple",
        "ui-state-layer",
        "ui-full-screen-dialog-action-bar-pressed-state-layer-color-deprecated",
        "#6750A4",
        "ui-primary-inferred-role",
    ),
    token(
        "ui-pressed-ripple",
        "ui-state-layer",
        "ui-full-screen-dialog-action-bar-pressed-state-layer-opacity-deprecated",
        "0.1",
    ),
];
