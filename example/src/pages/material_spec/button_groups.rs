//! Button group specifications and selection examples.
//!
//! ## Usage
//!
//! Compare related actions, connected choices and selection policies.

use tessera_components::{
    button::{ButtonShape, button},
    button_groups::{
        ButtonGroupsSelectionMode, ButtonGroupsSize, ButtonGroupsState, ButtonGroupsStyle,
        button_groups,
    },
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    spacer::spacer,
    text::text,
};
use tessera_ui::{Dp, Modifier, RenderSlot, accesskit::Role, remember, tessera};

use super::{
    action_articles::{Kind, action_article},
    article::SpecSection,
    gap, panel, spec_table, type_style,
};

#[tessera]
pub(super) fn button_groups_spec() {
    action_article(
        Kind::Groups,
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
                title: "Selection & activation",
                content: RenderSlot::new(selection),
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
        RenderSlot::new(variants),
        RenderSlot::new(anatomy),
        RenderSlot::new(selection),
    );
}

#[tessera]
fn sample(
    #[prop(into)] label: String,
    connected: bool,
    interactive: Option<bool>,
    icons: Option<bool>,
) {
    let state = remember(ButtonGroupsState::new);
    let square = remember(|| false);
    let multiple = remember(|| false);
    let required = remember(|| false);
    let enabled = remember(|| true);
    let size = remember(|| 1_usize);
    let status = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(label),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if interactive.unwrap_or(false) {
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        button()
                            .tonal()
                            .selected(square.get())
                            .accessibility_label("Group square shape")
                            .on_click(move || square.set(!square.get()))
                            .child(|| {
                                text().content("Square");
                            });
                        button()
                            .tonal()
                            .selected(multiple.get())
                            .accessibility_label("Group multiple selection")
                            .on_click(move || multiple.set(!multiple.get()))
                            .child(|| {
                                text().content("Multiple");
                            });
                        button()
                            .tonal()
                            .selected(required.get())
                            .accessibility_label("Group required selection")
                            .on_click(move || required.set(!required.get()))
                            .child(|| {
                                text().content("Required");
                            });
                        button()
                            .tonal()
                            .selected(!enabled.get())
                            .accessibility_label("Group disabled")
                            .on_click(move || enabled.set(!enabled.get()))
                            .child(|| {
                                text().content("Disabled");
                            });
                        button()
                            .text()
                            .accessibility_label("Cycle group size")
                            .on_click(move || size.set((size.get() + 1) % 5))
                            .child(move || {
                                text().content(format!(
                                    "Size: {}",
                                    ["XS", "S", "M", "L", "XL"][size.get()]
                                ));
                            });
                    });
                gap(Dp(24.0));
            }
            button_groups()
                .state(state)
                .style(if connected {
                    ButtonGroupsStyle::Connected
                } else {
                    ButtonGroupsStyle::Standard
                })
                .size(
                    [
                        ButtonGroupsSize::ExtraSmall,
                        ButtonGroupsSize::Small,
                        ButtonGroupsSize::Medium,
                        ButtonGroupsSize::Large,
                        ButtonGroupsSize::ExtraLarge,
                    ][size.get()],
                )
                .shape_style(if square.get() {
                    ButtonShape::Square
                } else {
                    ButtonShape::Round
                })
                .selection_mode(if multiple.get() {
                    ButtonGroupsSelectionMode::Multiple
                } else {
                    ButtonGroupsSelectionMode::Single
                })
                .selection_required(required.get())
                .enabled(enabled.get())
                .child(
                    move || {
                        if icons.unwrap_or(false) {
                            row().children(|| {
                                icon().painter(filled::FORMAT_BOLD_SVG);
                                spacer().modifier(Modifier::new().width(Dp(8.0)));
                                text().content("Bold");
                            });
                        } else {
                            text().content("One");
                        }
                    },
                    |_| {},
                )
                .child(
                    move || {
                        if icons.unwrap_or(false) {
                            row().children(|| {
                                icon().painter(filled::FORMAT_ITALIC_SVG);
                                spacer().modifier(Modifier::new().width(Dp(8.0)));
                                text().content("Italic");
                            });
                        } else {
                            text().content("Two");
                        }
                    },
                    |_| {},
                )
                .child(
                    move || {
                        if icons.unwrap_or(false) {
                            row().children(|| {
                                icon().painter(filled::FORMAT_UNDERLINED_SVG);
                                spacer().modifier(Modifier::new().width(Dp(8.0)));
                                text().content("Underline");
                            });
                        } else {
                            text().content("Three");
                        }
                    },
                    |_| {},
                );
            gap(Dp(16.0));
            text()
                .content(format!(
                    "{status} selection: {}",
                    (0..3)
                        .filter(|index| state.with(|state| state.is_selected(*index)))
                        .map(|index| (index + 1).to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(|| {
        column().children(|| {
            sample("Standard group", false);
            gap(Dp(40.0));
            sample("Connected group", true);
        });
    }));
}

#[tessera]
fn configurations() {
    panel(RenderSlot::new(|| {
        sample("Configured group", true).interactive(true);
    }));
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Property".into(), "Value".into()],
        vec![
            vec![
                "Standard gaps · XS / S / M / L / XL".into(),
                "18 / 12 / 8 / 8 / 8dp".into(),
            ],
            vec!["Connected gaps".into(), "2dp".into()],
            vec![
                "Connected inner corners · XS / S / M / L / XL".into(),
                "8 / 8 / 8 / 16 / 20dp".into(),
            ],
            vec!["Minimum interactive target".into(), "48dp".into()],
        ],
    );
}

#[tessera]
fn anatomy() {
    text().content("The group coordinates the spacing and shapes of its children. These examples use filled toggle choices; icons can replace labels when their meaning is clear.").style(type_style(16.0,24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("Formatting group", true).icons(true);
    }));
}

#[tessera]
fn selection() {
    text().content("Standard groups exchange width between adjacent choices during activation. Connected groups keep their widths and change the selected choice's shape. Use the configuration controls to compare single, multiple and required selection.").style(type_style(16.0,24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("Activation group", false);
    }));
}

#[tessera]
fn states() {
    text().content("Hover, focus with Tab and the arrow keys, or activate a choice. The configured group also exposes a disabled state while retaining its selection.").style(type_style(16.0,24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("State group", true);
    }));
}

#[tessera]
fn measurements() {
    spec_table(
        vec!["Size".into(), "Visual height".into(), "Standard gap".into()],
        vec![
            vec!["XS".into(), "32dp".into(), "18dp".into()],
            vec!["S".into(), "40dp".into(), "12dp".into()],
            vec!["M".into(), "56dp".into(), "8dp".into()],
            vec!["L".into(), "96dp".into(), "8dp".into()],
            vec!["XL".into(), "136dp".into(), "8dp".into()],
        ],
    );
}

#[tessera]
fn density() {
    text().content("Group spacing follows the size of its children. Extra-small and small choices retain a 48dp minimum target even though their visible containers are shorter.").style(type_style(16.0,24.0));
}
