//! Split action specifications with anchored option menus.
//!
//! ## Usage
//!
//! Compare a primary action with related choices across sizes and styles.

use tessera_components::{
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    menus::{MenuAnchor, MenuController, menu_item, menu_provider},
    modifier::ModifierExt as _,
    spacer::spacer,
    split_buttons::{
        SplitButtonDefaults, SplitButtonSize, SplitButtonVariant, split_button_layout,
        split_leading_button, split_trailing_button,
    },
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, PxPosition, PxRect, PxSize, RenderSlot, State, modifier::FocusModifierExt as _,
    remember, tessera,
};

use super::{
    ACCENT,
    action_articles::{Kind, action_article},
    article::SpecSection,
    gap, panel, spec_table, subheading, type_style,
};

const SIZES: [SplitButtonSize; 5] = [
    SplitButtonSize::ExtraSmall,
    SplitButtonSize::Small,
    SplitButtonSize::Medium,
    SplitButtonSize::Large,
    SplitButtonSize::ExtraLarge,
];
const STYLES: [SplitButtonVariant; 4] = [
    SplitButtonVariant::Elevated,
    SplitButtonVariant::Filled,
    SplitButtonVariant::Tonal,
    SplitButtonVariant::Outlined,
];

#[derive(Clone, Copy, PartialEq)]
struct Menu {
    controller: State<MenuController>,
    anchor: State<PxRect>,
    origin: State<PxRect>,
    source: State<String>,
    result: State<String>,
}

#[tessera]
pub(super) fn split_buttons_spec() {
    let menu = Menu {
        controller: remember(MenuController::new),
        anchor: remember(PxRect::default),
        origin: remember(PxRect::default),
        source: remember(String::new),
        result: remember(|| "No option selected".to_owned()),
    };
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .on_globally_positioned(move |rect| {
                    if menu.origin.get() != rect {
                        menu.origin.set(rect);
                    }
                }),
        )
        .children(move || {
            let anchor = menu.anchor.get();
            let origin = menu.origin.get();
            menu_provider()
                .controller(menu.controller)
                .anchor(MenuAnchor::new(
                    PxPosition::new(anchor.x - origin.x, anchor.y - origin.y),
                    PxSize::new(anchor.width, anchor.height),
                ))
                .main_content(move || {
                    action_article(
                        Kind::Split,
                        vec![
                            SpecSection {
                                title: "Variants",
                                content: RenderSlot::new(move || {
                                    panel(RenderSlot::new(move || {
                                        example(menu, "Variant", 1, SplitButtonVariant::Filled)
                                            .compact(true);
                                    }));
                                }),
                            },
                            SpecSection {
                                title: "Configurations",
                                content: RenderSlot::new(move || {
                                    configurations(menu);
                                }),
                            },
                            SpecSection {
                                title: "Tokens & specs",
                                content: RenderSlot::new(tokens),
                            },
                            SpecSection {
                                title: "Anatomy",
                                content: RenderSlot::new(move || {
                                    anatomy(menu);
                                }),
                            },
                            SpecSection {
                                title: "Color",
                                content: RenderSlot::new(move || {
                                    colors(menu);
                                }),
                            },
                            SpecSection {
                                title: "States",
                                content: RenderSlot::new(move || {
                                    states(menu);
                                }),
                            },
                            SpecSection {
                                title: "Measurements",
                                content: RenderSlot::new(measurements),
                            },
                        ],
                        RenderSlot::new(move || {
                            panel(RenderSlot::new(move || {
                                example(menu, "Overview", 1, SplitButtonVariant::Filled)
                                    .compact(true);
                            }));
                        }),
                        RenderSlot::new(move || {
                            anatomy(menu);
                        }),
                        RenderSlot::new(move || {
                            panel(RenderSlot::new(move || {
                                example(menu, "Behavior", 1, SplitButtonVariant::Filled)
                                    .compact(true);
                            }));
                        }),
                    );
                })
                .menu_content(move || {
                    menu_item()
                        .label("Schedule send")
                        .leading_icon(filled::SCHEDULE_SVG)
                        .on_click(move || {
                            menu.result.set("Scheduled send selected".into());
                        });
                    menu_item()
                        .label("Save draft")
                        .leading_icon(filled::EDIT_SVG)
                        .on_click(move || {
                            menu.result.set("Draft saved".into());
                        });
                });
        });
}

#[tessera]
fn example(
    menu: Menu,
    #[prop(into)] label: String,
    size: usize,
    style: SplitButtonVariant,
    controls: Option<bool>,
    compact: Option<bool>,
) {
    let enabled = remember(|| true);
    let count = remember(|| 0_u32);
    let bounds = remember(PxRect::default);
    let anchor_label = label.clone();
    let caption = label.clone();
    column()
        .modifier(if compact.unwrap_or(false) {
            Modifier::new()
        } else {
            Modifier::new().fill_max_width()
        })
        .children(move || {
            if controls.unwrap_or(false) {
                button()
                    .tonal()
                    .selected(!enabled.get())
                    .accessibility_label("Disable split action")
                    .on_click(move || {
                        enabled.set(!enabled.get());
                        if !enabled.get() {
                            menu.controller.with_mut(MenuController::close);
                        }
                    })
                    .child(|| {
                        text().content("Disabled");
                    });
                gap(Dp(24.0));
            }
            let label = label.clone();
            let anchor_label = anchor_label.clone();
            split_button_layout()
                .leading_button(move || {
                    split_leading_button()
                        .size(SIZES[size])
                        .variant(style)
                        .enabled(enabled.get())
                        .accessibility_label(format!("{label} send"))
                        .on_click(move || count.set(count.get() + 1))
                        .content(move || {
                            icon().painter(filled::SEND_SVG);
                            spacer().modifier(
                                Modifier::new()
                                    .width(SplitButtonDefaults::icon_spacing(SIZES[size])),
                            );
                            text().content("Send");
                        });
                })
                .trailing_button(move || {
                    let name = anchor_label.clone();
                    let selected = anchor_label.clone();
                    split_trailing_button()
                        .size(SIZES[size])
                        .variant(style)
                        .enabled(enabled.get())
                        .expanded(
                            menu.controller.with(MenuController::is_open)
                                && menu.source.get() == anchor_label,
                        )
                        .accessibility_label(format!("{anchor_label} options"))
                        .modifier(Modifier::new().on_globally_positioned(move |rect| {
                            if bounds.get() != rect {
                                bounds.set(rect);
                            }
                            if menu.source.get() == name && menu.anchor.get() != rect {
                                menu.anchor.set(rect);
                            }
                        }))
                        .on_click(move || {
                            if menu.controller.with(MenuController::is_open)
                                && menu.source.get() == selected
                            {
                                menu.controller.with_mut(MenuController::close);
                            } else {
                                menu.source.set(selected.clone());
                                menu.anchor.set(bounds.get());
                                menu.controller.with_mut(MenuController::open);
                            }
                        })
                        .content(|| {
                            icon().painter(filled::ARROW_DROP_DOWN_SVG);
                        });
                });
            gap(Dp(16.0));
            text()
                .content(format!("{caption} actions: {}", count.get()))
                .style(type_style(14.0, 20.0));
            text()
                .content(menu.result.get())
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn configurations(menu: Menu) {
    panel(RenderSlot::new(move || {
        column().children(move || {
            for (index, name) in ["XS", "S", "M", "L", "XL"].into_iter().enumerate() {
                example(
                    menu,
                    format!("Size {name}"),
                    index,
                    SplitButtonVariant::Filled,
                );
                gap(Dp(24.0));
            }
        });
    }));
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Value".into()],
        vec![
            vec!["Between buttons".into(), "2dp".into()],
            vec!["Outer corners".into(), "Fully round".into()],
            vec![
                "Inner corners · XS / S / M / L / XL".into(),
                "4 / 4 / 4 / 8 / 12dp".into(),
            ],
            vec!["Expanded trailing shape".into(), "Fully round".into()],
            vec![
                "Color".into(),
                "Same roles as the corresponding button style".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy(menu: Menu) {
    panel(RenderSlot::new(move || {
        example(menu, "Anatomy", 1, SplitButtonVariant::Tonal);
    }));
    gap(Dp(24.0));
    text()
        .content("1  Leading action    2  Optional icon    3  Label    4  Trailing menu action")
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn colors(menu: Menu) {
    for dark in [false, true] {
        subheading(if dark {
            "Dark scheme".into()
        } else {
            "Light scheme".into()
        });
        gap(Dp(24.0));
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                surface()
                    .modifier(Modifier::new().fill_max_width())
                    .child(move || {
                        flow_row()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                            .item_spacing(Dp(24.0))
                            .line_spacing(Dp(24.0))
                            .children(move || {
                                for (style, name) in STYLES
                                    .into_iter()
                                    .zip(["Elevated", "Filled", "Tonal", "Outlined"])
                                {
                                    example(
                                        menu,
                                        format!("{} {name}", if dark { "Dark" } else { "Light" }),
                                        1,
                                        style,
                                    )
                                    .compact(true);
                                }
                            });
                    });
            });
        gap(Dp(32.0));
    }
}

#[tessera]
fn states(menu: Menu) {
    text().content("Activate Send independently from its menu. Open the menu, choose an alternative, or dismiss it with Escape. Hover and focus change the inner corners without changing the action's color style.").style(type_style(16.0,24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        example(menu, "State", 1, SplitButtonVariant::Filled).controls(true);
    }));
}

#[tessera]
fn measurements() {
    spec_table(
        vec![
            "Size".into(),
            "Height".into(),
            "Menu icon".into(),
            "Resting icon offset".into(),
        ],
        vec![
            vec!["XS".into(), "32dp".into(), "22dp".into(), "−1dp".into()],
            vec!["S".into(), "40dp".into(), "22dp".into(), "−1dp".into()],
            vec!["M".into(), "56dp".into(), "26dp".into(), "−2dp".into()],
            vec!["L".into(), "96dp".into(), "38dp".into(), "−3dp".into()],
            vec!["XL".into(), "136dp".into(), "50dp".into(), "−6dp".into()],
        ],
    );
}
