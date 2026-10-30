//! Slider token sets and design configuration comparisons.
//!
//! ## Usage
//!
//! Inspect dimensions and color roles across slider sizes.

use tessera_components::{
    alignment::CrossAxisAlignment,
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    menus::{MenuAnchor, MenuController, MenuItemSelectionMode, menu_item, menu_provider},
    modifier::ModifierExt as _,
    row::row,
    text::text,
};
use tessera_ui::{
    Dp, Modifier, PxPosition, PxRect, PxSize, State, modifier::FocusModifierExt as _, remember,
    tessera,
};

use super::{gap, spec_table, subheading, type_style};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "sliders_tokens_tests.rs"]
mod tests;

const SET_NAMES: [&str; 6] = [
    "Slider",
    "Slider - Size - Xsmall",
    "Slider - Size - Small",
    "Slider - Size - Medium",
    "Slider - Size - Large",
    "Slider - Size - Xlarge",
];

const GROUPS: [&[&str]; 6] = [
    &[
        "Enabled",
        "Disabled",
        "Hovered",
        "Focused",
        "Pressed (ripple)",
    ],
    &["Active track", "Inactive track", "Handle"],
    &["Active track", "Inactive track", "Handle"],
    &["Active track", "Inactive track", "Handle", "Icon"],
    &["Active track", "Inactive track", "Handle", "Icon"],
    &["Active track", "Inactive track", "Handle", "Icon"],
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
    let anchor = remember(PxRect::default);
    let origin = remember(PxRect::default);
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .on_globally_positioned(move |rect| {
                    if origin.get() != rect {
                        origin.set(rect);
                    }
                }),
        )
        .children(move || {
            let anchor_rect = anchor.get();
            let origin_rect = origin.get();
            token_menu(
                selected,
                expanded,
                controller,
                anchor,
                MenuAnchor::new(
                    PxPosition::new(anchor_rect.x - origin_rect.x, anchor_rect.y - origin_rect.y),
                    PxSize::new(anchor_rect.width, anchor_rect.height),
                ),
            );
        });
}

#[tessera]
fn token_menu(
    selected: State<usize>,
    expanded: State<[[bool; 5]; 6]>,
    controller: State<MenuController>,
    anchor: State<PxRect>,
    menu_anchor: MenuAnchor,
) {
    menu_provider()
        .controller(controller)
        .anchor(menu_anchor)
        .modifier(Modifier::new().fill_max_width())
        .main_content(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    text()
                        .content(
                            "Switch between common slider tokens and the five size token sets.",
                        )
                        .style(type_style(16.0, 24.0));
                    gap(Dp(16.0));
                    flow_row()
                        .item_spacing(Dp(16.0))
                        .line_spacing(Dp(8.0))
                        .children(move || {
                            button()
                                .outlined()
                                .accessibility_label("Slider token set selector")
                                .modifier(Modifier::new().on_globally_positioned(move |rect| {
                                    if anchor.get() != rect {
                                        anchor.set(rect);
                                    }
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
                                .accessibility_label(
                                    if expanded.with(|state| {
                                        state[selected.get()][..GROUPS[selected.get()].len()]
                                            .iter()
                                            .all(|open| *open)
                                    }) {
                                        "Collapse all"
                                    } else {
                                        "Expand all"
                                    },
                                )
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
                                    text().content(if all_open {
                                        "Collapse all"
                                    } else {
                                        "Expand all"
                                    });
                                });
                        });
                    gap(Dp(24.0));
                    let index = selected.get();
                    for (group_index, group) in GROUPS[index].iter().copied().enumerate() {
                        let open = expanded.with(|state| state[index][group_index]);
                        button()
                            .text()
                            .accessibility_label(group)
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
                                text()
                                    .content(format!("{}  {group}", if open { "−" } else { "+" }));
                            });
                        if open {
                            token_table(index, group.to_owned());
                        }
                    }
                });
        })
        .menu_content(move || {
            for (index, label) in SET_NAMES.into_iter().enumerate() {
                menu_item()
                    .label(label)
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
        if (group.is_empty() || token.group == group) && !sections.contains(&token.section) {
            sections.push(token.section);
        }
    }
    for section in sections {
        if index == 0 {
            subheading(section.to_owned());
        }
        let rows = TOKEN_SETS[index]
            .iter()
            .filter(|token| (group.is_empty() || token.group == group) && token.section == section)
            .map(|token| vec![token.name.to_owned(), token.value.to_owned()])
            .collect();
        spec_table(vec!["Token".into(), "Value".into()], rows);
        gap(Dp(16.0));
    }
}

#[tessera]
pub(super) fn availability() {
    let rows = [
        ["Inset icon", "No (default)", "Available", "Available"],
        ["", "Yes", "--", "Available"],
        [
            "Orientation",
            "Horizontal (default)",
            "Available",
            "Available",
        ],
        ["", "Vertical", "--", "Available"],
        ["Size", "XS (default)", "Available", "Available"],
        [
            "",
            "S, M, L, XL",
            "--",
            "Available on Android Views (MDC-Android).\nAvailable as tokens on other platforms.*",
        ],
        [
            "Stop indicators",
            "No (default), Yes",
            "Available as “discrete” slider",
            "Available",
        ],
        [
            "Value Indicator",
            "No (default), Yes",
            "Available",
            "Available",
        ],
    ];
    let mut cells = Vec::new();
    for (index, row) in rows.into_iter().enumerate() {
        for (column, text) in row.into_iter().enumerate() {
            if column == 0 && text.is_empty() {
                continue;
            }
            cells.push(tessera_components::text_table::TextTableCell {
                text: text.into(),
                row: index + 1,
                column,
                row_span: if column == 0 && index < 6 { 2 } else { 1 },
                ..Default::default()
            });
        }
    }
    super::spec_table_cells(
        ["Category", "Configuration", "M3", "M3 Expressive"]
            .map(str::to_owned)
            .to_vec(),
        cells,
    );
    gap(Dp(16.0));
    text().content("*Configurations only available using tokens don’t have implemented presets in code. To change the size, swap the default size tokens md.comp.slider.xsmall.[...] with those of the desired size.")
        .style(type_style(16.0,24.0));
}

const TOKEN_SETS: [&[Token]; 6] = [
    &[
        Token {
            group: "Disabled",
            section: "Stop indicator",
            name: "Slider disabled active stop indicator container color\nmd.comp.slider.disabled.active.stop-indicator.container.color",
            value: "Inverse on surface",
        },
        Token {
            group: "Disabled",
            section: "Stop indicator",
            name: "Slider disabled inactive stop indicator container color\nmd.comp.slider.disabled.inactive.stop-indicator.container.color",
            value: "On surface",
        },
        Token {
            group: "Disabled",
            section: "Track",
            name: "Slider disabled active track color\nmd.comp.slider.disabled.active.track.color",
            value: "On surface",
        },
        Token {
            group: "Disabled",
            section: "Track",
            name: "Slider disabled active track opacity\nmd.comp.slider.disabled.active.track.opacity",
            value: "38%",
        },
        Token {
            group: "Disabled",
            section: "Track",
            name: "Slider disabled inactive track color\nmd.comp.slider.disabled.inactive.track.color",
            value: "On surface",
        },
        Token {
            group: "Disabled",
            section: "Track",
            name: "Slider disabled inactive track opacity\nmd.comp.slider.disabled.inactive.track.opacity",
            value: "12%",
        },
        Token {
            group: "Disabled",
            section: "Handle",
            name: "Slider disabled handle color\nmd.comp.slider.disabled.handle.color",
            value: "On surface",
        },
        Token {
            group: "Disabled",
            section: "Handle",
            name: "Slider disabled handle opacity\nmd.comp.slider.disabled.handle.opacity",
            value: "38%",
        },
        Token {
            group: "Disabled",
            section: "Handle",
            name: "Slider disabled handle width\nmd.comp.slider.disabled.handle.width",
            value: "4dp",
        },
        Token {
            group: "Enabled",
            section: "Stop indicator",
            name: "Slider stop indicator size\nmd.comp.slider.stop-indicator.size",
            value: "4dp",
        },
        Token {
            group: "Enabled",
            section: "Stop indicator",
            name: "Slider stop indicator shape\nmd.comp.slider.stop-indicator.shape",
            value: "Full",
        },
        Token {
            group: "Enabled",
            section: "Stop indicator",
            name: "Slider stop indicator trailing space\nmd.comp.slider.stop-indicator.trailing-space",
            value: "4dp",
        },
        Token {
            group: "Enabled",
            section: "Stop indicator",
            name: "Slider stop indicator color\nmd.comp.slider.stop-indicator.color",
            value: "On secondary container",
        },
        Token {
            group: "Enabled",
            section: "Stop indicator",
            name: "Slider stop indicator color selected\nmd.comp.slider.stop-indicator.color-selected",
            value: "On primary",
        },
        Token {
            group: "Enabled",
            section: "Container",
            name: "Slider active stop indicator container color\nmd.comp.slider.active.stop-indicator.container.color",
            value: "On primary",
        },
        Token {
            group: "Enabled",
            section: "Container",
            name: "Slider active stop indicator container opacity\nmd.comp.slider.active.stop-indicator.container.opacity",
            value: "100%",
        },
        Token {
            group: "Enabled",
            section: "Container",
            name: "Slider inactive stop indicator container color\nmd.comp.slider.inactive.stop-indicator.container.color",
            value: "On secondary container",
        },
        Token {
            group: "Enabled",
            section: "Container",
            name: "Slider inactive stop indicator container opacity\nmd.comp.slider.inactive.stop-indicator.container.opacity",
            value: "100%",
        },
        Token {
            group: "Enabled",
            section: "Track",
            name: "Slider active track height\nmd.comp.slider.active.track.height",
            value: "16dp",
        },
        Token {
            group: "Enabled",
            section: "Track",
            name: "Slider inactive track height\nmd.comp.slider.inactive.track.height",
            value: "16dp",
        },
        Token {
            group: "Enabled",
            section: "Track",
            name: "Slider active track shape\nmd.comp.slider.active.track.shape",
            value: "Full",
        },
        Token {
            group: "Enabled",
            section: "Track",
            name: "Slider active track outer corner size\nmd.comp.slider.active.track.outer-corner.corner-size",
            value: "Full",
        },
        Token {
            group: "Enabled",
            section: "Track",
            name: "Slider active track inner corner size\nmd.comp.slider.active.track.inner-corner.corner-size",
            value: "0dp 2dp 2dp 0dp",
        },
        Token {
            group: "Enabled",
            section: "Track",
            name: "Slider inactive track shape\nmd.comp.slider.inactive.track.shape",
            value: "Full",
        },
        Token {
            group: "Enabled",
            section: "Track",
            name: "Slider active track color\nmd.comp.slider.active.track.color",
            value: "Primary",
        },
        Token {
            group: "Enabled",
            section: "Track",
            name: "Slider inactive track color\nmd.comp.slider.inactive.track.color",
            value: "Secondary container",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider handle height\nmd.comp.slider.handle.height",
            value: "44dp",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider handle width\nmd.comp.slider.handle.width",
            value: "4dp",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider handle shape\nmd.comp.slider.handle.shape",
            value: "Full",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider handle color\nmd.comp.slider.handle.color",
            value: "Primary",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider active handle color\nmd.comp.slider.active.handle.color",
            value: "Primary",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider active handle height\nmd.comp.slider.active.handle.height",
            value: "44dp",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider active handle width\nmd.comp.slider.active.handle.width",
            value: "4dp",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider active handle shape\nmd.comp.slider.active.handle.shape",
            value: "Full",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider active handle leading space\nmd.comp.slider.active.handle.leading-space",
            value: "6dp",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider active handle trailing space\nmd.comp.slider.active.handle.trailing-space",
            value: "6dp",
        },
        Token {
            group: "Enabled",
            section: "Handle",
            name: "Slider active handle padding\nmd.comp.slider.active.handle.padding",
            value: "6dp",
        },
        Token {
            group: "Focused",
            section: "Track",
            name: "Slider focus active track color\nmd.comp.slider.focus.active.track.color",
            value: "Primary",
        },
        Token {
            group: "Focused",
            section: "Track",
            name: "Slider focus inactive track color\nmd.comp.slider.focus.inactive.track.color",
            value: "Secondary container",
        },
        Token {
            group: "Focused",
            section: "Handle",
            name: "Slider focus handle width\nmd.comp.slider.focus.handle.width",
            value: "2dp",
        },
        Token {
            group: "Hovered",
            section: "Handle",
            name: "Slider hover handle width\nmd.comp.slider.hover.handle.width",
            value: "4dp",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Handle",
            name: "Slider pressed handle color\nmd.comp.slider.pressed.handle.color",
            value: "Primary",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Handle",
            name: "Slider pressed handle width\nmd.comp.slider.pressed.handle.width",
            value: "2dp",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Track",
            name: "Slider pressed active track color\nmd.comp.slider.pressed.active.track.color",
            value: "Primary",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Track",
            name: "Slider pressed inactive track color\nmd.comp.slider.pressed.inactive.track.color",
            value: "Secondary container",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Value indicator",
            name: "Slider value indicator container color\nmd.comp.slider.value-indicator.container.color",
            value: "Inverse surface",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Value indicator",
            name: "Slider value indicator label font\nmd.comp.slider.value-indicator.label.label-text.font",
            value: "md.sys.typescale.label-large.font",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Value indicator",
            name: "Slider value indicator label font color\nmd.comp.slider.value-indicator.label.label-text.color",
            value: "Inverse on surface",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Value indicator",
            name: "Slider value indicator label line height\nmd.comp.slider.value-indicator.label.label-text.line-height",
            value: "md.sys.typescale.label-large.line-height",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Value indicator",
            name: "Slider value indicator label size\nmd.comp.slider.value-indicator.label.label-text.size",
            value: "md.sys.typescale.label-large.size",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Value indicator",
            name: "Slider value indicator label tracking\nmd.comp.slider.value-indicator.label.label-text.tracking",
            value: "md.sys.typescale.body-large.tracking",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Value indicator",
            name: "Slider value indicator label weight\nmd.comp.slider.value-indicator.label.label-text.weight",
            value: "md.sys.typescale.body-large.weight",
        },
        Token {
            group: "Pressed (ripple)",
            section: "Value indicator",
            name: "Slider value indicator active bottom space\nmd.comp.slider.value-indicator.active.bottom-space",
            value: "12dp",
        },
    ],
    &[
        Token {
            group: "Active track",
            section: "Active track",
            name: "Slider xsmall active track height\nmd.comp.slider.xsmall.active.track.height",
            value: "16dp",
        },
        Token {
            group: "Active track",
            section: "Active track",
            name: "Slider xsmall active track leading shape\nmd.comp.slider.xsmall.active.track.shape.leading",
            value: "8dp",
        },
        Token {
            group: "Handle",
            section: "Handle",
            name: "Slider xsmall active handle height\nmd.comp.slider.xsmall.active.handle.height",
            value: "44dp",
        },
        Token {
            group: "Inactive track",
            section: "Inactive track",
            name: "Slider xsmall inactive track height\nmd.comp.slider.xsmall.inactive.track.height",
            value: "16dp",
        },
        Token {
            group: "Inactive track",
            section: "Inactive track",
            name: "Slider xsmall inactive track trailing shape\nmd.comp.slider.xsmall.inactive.track.shape.trailing",
            value: "8dp",
        },
    ],
    &[
        Token {
            group: "Active track",
            section: "Active track",
            name: "Slider small active track height\nmd.comp.slider.small.active.track.height",
            value: "24dp",
        },
        Token {
            group: "Active track",
            section: "Active track",
            name: "Slider small active track leading shape\nmd.comp.slider.small.active.track.shape.leading",
            value: "8dp",
        },
        Token {
            group: "Handle",
            section: "Handle",
            name: "Slider small active handle height\nmd.comp.slider.small.active.handle.height",
            value: "44dp",
        },
        Token {
            group: "Inactive track",
            section: "Inactive track",
            name: "Slider small inactive track height\nmd.comp.slider.small.inactive.track.height",
            value: "24dp",
        },
        Token {
            group: "Inactive track",
            section: "Inactive track",
            name: "Slider small inactive track trailing shape\nmd.comp.slider.small.inactive.track.shape.trailing",
            value: "8dp",
        },
    ],
    &[
        Token {
            group: "Active track",
            section: "Active track",
            name: "Slider medium active track height\nmd.comp.slider.medium.active.track.height",
            value: "40dp",
        },
        Token {
            group: "Active track",
            section: "Active track",
            name: "Slider medium active track leading shape\nmd.comp.slider.medium.active.track.shape.leading",
            value: "12dp",
        },
        Token {
            group: "Handle",
            section: "Handle",
            name: "Slider medium active handle height\nmd.comp.slider.medium.active.handle.height",
            value: "44dp",
        },
        Token {
            group: "Icon",
            section: "Icon",
            name: "Slider medium icon padding\nmd.comp.slider.medium.icon.padding",
            value: "6dp",
        },
        Token {
            group: "Icon",
            section: "Icon",
            name: "Slider medium icon size\nmd.comp.slider.medium.icon.size",
            value: "24dp",
        },
        Token {
            group: "Inactive track",
            section: "Inactive track",
            name: "Slider medium inactive track height\nmd.comp.slider.medium.inactive.track.height",
            value: "40dp",
        },
        Token {
            group: "Inactive track",
            section: "Inactive track",
            name: "Slider medium inactive track trailing shape\nmd.comp.slider.medium.inactive.track.shape.trailing",
            value: "12dp",
        },
    ],
    &[
        Token {
            group: "Active track",
            section: "Active track",
            name: "Slider large active track height\nmd.comp.slider.large.active.track.height",
            value: "56dp",
        },
        Token {
            group: "Active track",
            section: "Active track",
            name: "Slider large active track leading shape\nmd.comp.slider.large.active.track.shape.leading",
            value: "16dp",
        },
        Token {
            group: "Handle",
            section: "Handle",
            name: "Slider large active handle height\nmd.comp.slider.large.active.handle.height",
            value: "68dp",
        },
        Token {
            group: "Icon",
            section: "Icon",
            name: "Slider large icon padding\nmd.comp.slider.large.icon.padding",
            value: "6dp",
        },
        Token {
            group: "Icon",
            section: "Icon",
            name: "Slider large icon size\nmd.comp.slider.large.icon.size",
            value: "24dp",
        },
        Token {
            group: "Inactive track",
            section: "Inactive track",
            name: "Slider large inactive track trailing shape\nmd.comp.slider.large.inactive.track.height",
            value: "56dp",
        },
        Token {
            group: "Inactive track",
            section: "Inactive track",
            name: "Slider large inactive track trailing shape\nmd.comp.slider.large.inactive.track.shape.trailing",
            value: "16dp",
        },
    ],
    &[
        Token {
            group: "Active track",
            section: "Active track",
            name: "Slider xlarge active track height\nmd.comp.slider.xlarge.active.track.height",
            value: "96dp",
        },
        Token {
            group: "Active track",
            section: "Active track",
            name: "Slider xlarge active track leading shape\nmd.comp.slider.xlarge.active.track.shape.leading",
            value: "28dp",
        },
        Token {
            group: "Handle",
            section: "Handle",
            name: "Slider xlarge active handle height\nmd.comp.slider.xlarge.active.handle.height",
            value: "108dp",
        },
        Token {
            group: "Icon",
            section: "Icon",
            name: "Slider xlarge icon padding\nmd.comp.slider.xlarge.icon.padding",
            value: "8dp",
        },
        Token {
            group: "Icon",
            section: "Icon",
            name: "Slider xlarge icon size\nmd.comp.slider.xlarge.icon.size",
            value: "32dp",
        },
        Token {
            group: "Inactive track",
            section: "Inactive track",
            name: "Slider xlarge inactive track height\nmd.comp.slider.xlarge.inactive.track.height",
            value: "96dp",
        },
        Token {
            group: "Inactive track",
            section: "Inactive track",
            name: "Slider xlarge inactive track trailing shape\nmd.comp.slider.xlarge.inactive.track.shape.trailing",
            value: "28dp",
        },
    ],
];
