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

use super::{gap, spec_table, type_style};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "dialogs_tokens_tests.rs"]
mod tests;

const SET_NAMES: [&str; 2] = ["Dialog - Basic", "Dialog - Full screen"];
const GROUPS: [&str; 4] = ["Enabled", "Hovered", "Focused", "Pressed (ripple)"];

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
                        .content("Select a component variant below to see its elements, attributes, tokens, and their values.")
                        .style(type_style(16.0, 24.0));
                    gap(Dp(16.0));
                    row()
                        .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            button()
                                .tonal()
                                .accessibility_label("Dialog token set selector")
                                .modifier(Modifier::new().on_layout_coordinates(move |coordinates| {
                                    anchor.set_if_changed(coordinates);
                                }))
                                .on_click(move || controller.with_mut(MenuController::open))
                                .child(move || {
                                    row()
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .children(move || {
                                            text().content(SET_NAMES[selected.get()]);
                                            boxed().modifier(Modifier::new().width(Dp(8.0)));
                                            icon()
                                                .vector(filled::ARROW_DROP_DOWN_SVG)
                                                .size(Dp(18.0));
                                        });
                                });
                            button()
                                .text()
                                .accessibility_label(if all_open(selected, expanded) {
                                    "Collapse all"
                                } else {
                                    "Expand all"
                                })
                                .on_click(move || {
                                    let index = selected.get();
                                    let open = !all_open(selected, expanded);
                                    expanded.with_mut(|state| state[index].fill(open));
                                })
                                .child(move || {
                                    text().content(if all_open(selected, expanded) {
                                        "Collapse all"
                                    } else {
                                        "Expand all"
                                    });
                                });
                        });
                    gap(Dp(8.0));
                    text()
                        .content("Token values shown for Default, Light")
                        .style(type_style(14.0, 20.0));
                    gap(Dp(4.0));
                    text()
                        .content("Values are the published token values. Shape, elevation and type style tokens are published as previews only. Entries in the mapping column are inferred and are not published token values.")
                        .style(type_style(14.0, 20.0));
                    gap(Dp(16.0));
                    for (group_index, group) in GROUPS.into_iter().enumerate() {
                        let open = expanded.with(|state| state[selected.get()][group_index]);
                        button()
                            .text()
                            .accessibility_label(group)
                            .on_click(move || {
                                let index = selected.get();
                                expanded.with_mut(|state| {
                                    state[index][group_index] = !state[index][group_index];
                                });
                            })
                            .child(move || {
                                let open =
                                    expanded.with(|state| state[selected.get()][group_index]);
                                text().content(format!(
                                    "{}  {group}",
                                    if open { "−" } else { "+" }
                                ));
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
                    .label(label)
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
            .content(format!("{group} / {section}"))
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
            vec!["Token".into(), "Value".into(), "Mapping (inferred)".into()],
            rows,
        );
        gap(Dp(16.0));
    }
}

const TOKEN_SETS: [&[Token]; 2] = [BASIC, FULL_SCREEN];

const BASIC: &[Token] = &[
    inferred(
        "Enabled",
        "Container",
        "Dialog container color",
        "#ECE6F0",
        "Surface container high (inferred role)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Dialog container elevation",
        "Preview only",
        "Level 3, 6dp (inferred)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Dialog container surface tint layer color (deprecated)",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Dialog container shape",
        "Preview only",
        "Extra large, 28dp corner radius (inferred)",
    ),
    token(
        "Enabled",
        "Label text",
        "Dialog action label text font",
        "Roboto",
    ),
    token(
        "Enabled",
        "Label text",
        "Dialog action label text line height",
        "20pt",
    ),
    token(
        "Enabled",
        "Label text",
        "Dialog action label text size",
        "14pt",
    ),
    token(
        "Enabled",
        "Label text",
        "Dialog action label text weight",
        "500",
    ),
    token(
        "Enabled",
        "Label text",
        "Dialog action label text tracking",
        "0.1pt",
    ),
    inferred(
        "Enabled",
        "Label text",
        "Label text type style",
        "Preview only",
        "Label large (inferred)",
    ),
    inferred(
        "Enabled",
        "Label text",
        "Dialog action label text color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token("Enabled", "Icon", "Dialog icon size", "24dp"),
    inferred(
        "Enabled",
        "Icon",
        "Dialog icon color",
        "#625B71",
        "Secondary (inferred role)",
    ),
    token(
        "Enabled",
        "Subhead",
        "Dialog subhead font (deprecated)",
        "Roboto",
    ),
    token(
        "Enabled",
        "Subhead",
        "Dialog subhead line height (deprecated)",
        "32pt",
    ),
    token(
        "Enabled",
        "Subhead",
        "Dialog subhead size (deprecated)",
        "24pt",
    ),
    token(
        "Enabled",
        "Subhead",
        "Dialog subhead weight (deprecated)",
        "400",
    ),
    token(
        "Enabled",
        "Subhead",
        "Dialog subhead tracking (deprecated)",
        "0pt",
    ),
    inferred(
        "Enabled",
        "Subhead",
        "Subhead type style (deprecated)",
        "Preview only",
        "Headline small (inferred)",
    ),
    inferred(
        "Enabled",
        "Subhead",
        "Dialog subhead color (deprecated)",
        "#1D1B20",
        "On surface (inferred role)",
    ),
    token("Enabled", "Headline", "Dialog headline font", "Roboto"),
    token("Enabled", "Headline", "Dialog headline line height", "32pt"),
    token("Enabled", "Headline", "Dialog headline size", "24pt"),
    token("Enabled", "Headline", "Dialog headline weight", "400"),
    token("Enabled", "Headline", "Dialog headline tracking", "0pt"),
    inferred(
        "Enabled",
        "Headline",
        "Headline type style",
        "Preview only",
        "Headline small (inferred)",
    ),
    inferred(
        "Enabled",
        "Headline",
        "Dialog headline color",
        "#1D1B20",
        "On surface (inferred role)",
    ),
    inferred(
        "Enabled",
        "Divider",
        "Dialog divider color (deprecated)",
        "#79747E",
        "Outline (inferred role)",
    ),
    token(
        "Enabled",
        "Divider",
        "Dialog divider height (deprecated)",
        "1dp",
    ),
    token(
        "Enabled",
        "Supporting text",
        "Dialog supporting text font",
        "Roboto",
    ),
    token(
        "Enabled",
        "Supporting text",
        "Dialog supporting text line height",
        "20pt",
    ),
    token(
        "Enabled",
        "Supporting text",
        "Dialog supporting text size",
        "14pt",
    ),
    token(
        "Enabled",
        "Supporting text",
        "Dialog supporting text weight",
        "400",
    ),
    token(
        "Enabled",
        "Supporting text",
        "Dialog supporting text tracking",
        "0.25pt",
    ),
    inferred(
        "Enabled",
        "Supporting text",
        "Supporting text type style",
        "Preview only",
        "Body medium (inferred)",
    ),
    inferred(
        "Enabled",
        "Supporting text",
        "Dialog supporting text color",
        "#49454F",
        "On surface variant (inferred role)",
    ),
    inferred(
        "Hovered",
        "Label text",
        "Dialog action hover label text color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Hovered",
        "State layer",
        "Dialog action hover state layer color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token(
        "Hovered",
        "State layer",
        "Dialog action hover state layer opacity",
        "0.08",
    ),
    inferred(
        "Focused",
        "Label text",
        "Dialog action focus label text color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Focused",
        "State layer",
        "Dialog action focus state layer color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token(
        "Focused",
        "State layer",
        "Dialog action focus state layer opacity",
        "0.1",
    ),
    inferred(
        "Pressed (ripple)",
        "Label text",
        "Dialog action pressed label text color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Pressed (ripple)",
        "State layer",
        "Dialog action pressed state layer color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token(
        "Pressed (ripple)",
        "State layer",
        "Dialog action pressed state layer opacity",
        "0.1",
    ),
];

const FULL_SCREEN: &[Token] = &[
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog on-scroll container color",
        "#F3EDF7",
        "Surface container (inferred role)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog container color",
        "#FEF7FF",
        "Surface (inferred role)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog container elevation",
        "Preview only",
        "Level 0, 0dp (inferred)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog container shape",
        "Preview only",
        "None, 0dp corner radius (inferred)",
    ),
    token(
        "Enabled",
        "Container",
        "Full screen dialog header container height",
        "56dp",
    ),
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog header container color",
        "#FEF7FF",
        "Surface (inferred role)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog header container elevation",
        "Preview only",
        "Level 0, 0dp (inferred)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog header container surface tint layer color (deprecated)",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog header on scroll container elevation",
        "Preview only",
        "Level 2, 3dp (inferred)",
    ),
    token(
        "Enabled",
        "Container",
        "Full screen dialog action bar container height (deprecated)",
        "56dp",
    ),
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog action bar container color (deprecated)",
        "#FEF7FF",
        "Surface (inferred role)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog action bar container elevation (deprecated)",
        "Preview only",
        "Level 0, 0dp (inferred)",
    ),
    inferred(
        "Enabled",
        "Container",
        "Full screen dialog action bar on scroll container elevation (deprecated)",
        "Preview only",
        "Level 2, 3dp (inferred)",
    ),
    token(
        "Enabled",
        "Label text",
        "Full screen dialog header action label text font",
        "Roboto",
    ),
    token(
        "Enabled",
        "Label text",
        "Full screen dialog header action label text line height",
        "20pt",
    ),
    token(
        "Enabled",
        "Label text",
        "Full screen dialog header action label text size",
        "14pt",
    ),
    token(
        "Enabled",
        "Label text",
        "Full screen dialog header action label text weight",
        "500",
    ),
    token(
        "Enabled",
        "Label text",
        "Full screen dialog header action label text tracking",
        "0.1pt",
    ),
    inferred(
        "Enabled",
        "Label text",
        "Full screen dialog header action label text type",
        "Preview only",
        "Label large (inferred)",
    ),
    inferred(
        "Enabled",
        "Label text",
        "Full screen dialog header action label text color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token(
        "Enabled",
        "Label text",
        "Full screen dialog action bar label text font (deprecated)",
        "Roboto",
    ),
    token(
        "Enabled",
        "Label text",
        "Full screen dialog action bar label text line height (deprecated)",
        "20pt",
    ),
    token(
        "Enabled",
        "Label text",
        "Full screen dialog action bar label text size (deprecated)",
        "14pt",
    ),
    token(
        "Enabled",
        "Label text",
        "Full screen dialog action bar label text weight (deprecated)",
        "500",
    ),
    token(
        "Enabled",
        "Label text",
        "Full screen dialog action bar label text tracking (deprecated)",
        "0.1pt",
    ),
    inferred(
        "Enabled",
        "Label text",
        "Full screen dialog action bar label text type (deprecated)",
        "Preview only",
        "Label large (inferred)",
    ),
    inferred(
        "Enabled",
        "Label text",
        "Full screen dialog action bar label text color (deprecated)",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Enabled",
        "Icon",
        "Full screen dialog header icon color",
        "#1D1B20",
        "On surface (inferred role)",
    ),
    token(
        "Enabled",
        "Icon",
        "Full screen dialog header icon size",
        "24dp",
    ),
    inferred(
        "Enabled",
        "Headline",
        "Full screen dialog header headline color",
        "#1D1B20",
        "On surface (inferred role)",
    ),
    token(
        "Enabled",
        "Headline",
        "Full screen dialog header headline font",
        "Roboto",
    ),
    token(
        "Enabled",
        "Headline",
        "Full screen dialog header headline line height",
        "28pt",
    ),
    token(
        "Enabled",
        "Headline",
        "Full screen dialog header headline size",
        "22pt",
    ),
    token(
        "Enabled",
        "Headline",
        "Full screen dialog header headline weight",
        "400",
    ),
    token(
        "Enabled",
        "Headline",
        "Full screen dialog header headline tracking",
        "0pt",
    ),
    inferred(
        "Enabled",
        "Headline",
        "Header type style",
        "Preview only",
        "Title large (inferred)",
    ),
    inferred(
        "Enabled",
        "Divider",
        "Full screen dialog divider color (deprecated)",
        "#E7E0EC",
        "Surface variant (inferred role)",
    ),
    token(
        "Enabled",
        "Divider",
        "Full screen dialog divider height (deprecated)",
        "1dp",
    ),
    inferred(
        "Hovered",
        "Label text",
        "Full screen dialog header action hover label text color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Hovered",
        "Label text",
        "Full screen dialog action bar hover label text color (deprecated)",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Hovered",
        "State layer",
        "Full screen dialog header action hover state layer color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token(
        "Hovered",
        "State layer",
        "Full screen dialog header action hover state layer opacity",
        "0.08",
    ),
    inferred(
        "Hovered",
        "State layer",
        "Full screen dialog action bar hover state layer color (deprecated)",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token(
        "Hovered",
        "State layer",
        "Full screen dialog action bar hover state layer opacity (deprecated)",
        "0.08",
    ),
    inferred(
        "Focused",
        "Label text",
        "Full screen dialog header action focus label text color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Focused",
        "Label text",
        "Full screen dialog action bar focus label text color (deprecated)",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Focused",
        "State layer",
        "Full screen dialog header action focus state layer color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token(
        "Focused",
        "State layer",
        "Full screen dialog header action focus state layer opacity",
        "0.1",
    ),
    inferred(
        "Focused",
        "State layer",
        "Full screen dialog action bar focus state layer color (deprecated)",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token(
        "Focused",
        "State layer",
        "Full screen dialog action bar focus state layer opacity (deprecated)",
        "0.1",
    ),
    inferred(
        "Pressed (ripple)",
        "Label text",
        "Full screen dialog header action pressed label text color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Pressed (ripple)",
        "Label text",
        "Full screen dialog action bar pressed label text color (deprecated)",
        "#6750A4",
        "Primary (inferred role)",
    ),
    inferred(
        "Pressed (ripple)",
        "State layer",
        "Full screen dialog header action pressed state layer color",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token(
        "Pressed (ripple)",
        "State layer",
        "Full screen dialog header action pressed state layer opacity",
        "0.1",
    ),
    inferred(
        "Pressed (ripple)",
        "State layer",
        "Full screen dialog action bar pressed state layer color (deprecated)",
        "#6750A4",
        "Primary (inferred role)",
    ),
    token(
        "Pressed (ripple)",
        "State layer",
        "Full screen dialog action bar pressed state layer opacity (deprecated)",
        "0.1",
    ),
];
