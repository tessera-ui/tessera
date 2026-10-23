//! Expressive menu specifications and contextual actions.
//!
//! ## Usage
//!
//! Inspect menu groups, selection and keyboard interaction.

use tessera_components::{
    alignment::{Alignment, MainAxisAlignment},
    badge::badge_with_content,
    boxed::boxed,
    button::button,
    column::column,
    divider::horizontal_divider,
    flow_row::flow_row,
    icon::icon,
    icon_button::icon_button,
    material_icons::filled,
    menus::{
        MenuAnchor, MenuColorStyle, MenuController, MenuPosition, menu_group, menu_item,
        menu_provider,
    },
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    split_buttons::{
        SplitButtonVariant, split_button_layout, split_leading_button, split_trailing_button,
    },
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Dp, Modifier, PxPosition, PxRect, PxSize, RenderSlot, State, modifier::FocusModifierExt as _,
    remember, tessera, use_context,
};

use super::{
    INK,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn menus_spec() {
    article("Menus", "Menus display a list of choices on a temporary surface", vec![
        SpecSection { title: "Variants", content: RenderSlot::new(variants) },
        SpecSection { title: "Configurations", content: RenderSlot::new(configurations) },
        SpecSection { title: "Tokens & specs", content: RenderSlot::new(tokens) },
        SpecSection { title: "Anatomy", content: RenderSlot::new(|| {menu_figure(true, false, MenuColorStyle::Standard);}) },
        SpecSection { title: "Color", content: RenderSlot::new(colors) },
        SpecSection { title: "States", content: RenderSlot::new(states) },
        SpecSection { title: "Measurements", content: RenderSlot::new(|| {menu_figure(false, true, MenuColorStyle::Standard);}) },
        SpecSection { title: "Menu (baseline)", content: RenderSlot::new(baseline) },
    ]).pages(vec![
        ArticlePage { title: "Overview", intro: Some(RenderSlot::new(|| { prose("Show related actions in context without keeping every action on screen. Vertical menus use expressive shapes, optional groups, selection and two color styles."); gap(Dp(32.0)); launch(); })), sections: vec![
            SpecSection { title: "Usage", content: RenderSlot::new(|| { prose("Use menus for temporary choices, overflow actions and contextual commands. Use a toolbar when actions need to remain visible."); gap(Dp(24.0)); variants(); }) },
            SpecSection { title: "What's new", content: RenderSlot::new(|| { prose("Vertical menus add rounded groups, standard and vibrant colors, additional selection states and shape changes as focus moves between submenus."); }) },
        ] },
        ArticlePage { title: "Guidelines", intro: None, sections: vec![
            SpecSection { title: "Usage", content: RenderSlot::new(|| { prose("Use menus for temporary sets of actions. Keep vibrant menus for higher emphasis. Open them from a button, icon, field or contextual secondary click."); gap(Dp(24.0)); launch(); gap(Dp(32.0)); subheading("Menu groups".into()); prose("Bundle related actions with a divider or a small gap. Use no more than one or two gaps. Use dividers instead of gaps in scrolling menus."); gap(Dp(24.0)); configurations(); }) },
            SpecSection { title: "Anatomy", content: RenderSlot::new(|| { menu_figure(true, false, MenuColorStyle::Standard); gap(Dp(24.0)); subheading("Flexibility & slots".into()); prose("Keep a single action per menu item. Preserve menu item padding and a target of at least 48 × 48dp. Nested buttons or switches introduce competing actions and break the menu interaction model."); }) },
            SpecSection { title: "Placement", content: RenderSlot::new(|| { prose("Position the menu next to its trigger, below it or over it. Reposition near a window edge so the surface stays visible. Submenus open beside the parent item without covering it."); gap(Dp(24.0)); launch(); }) },
            SpecSection { title: "Adaptive design", content: RenderSlot::new(|| { subheading("Compact breakpoints".into()); prose("Consider a bottom sheet on compact screens when actions need more room or longer labels."); gap(Dp(24.0)); subheading("Other breakpoints".into()); prose("Keep menus near their content on medium and expanded screens. Submenus can organize more complex sets of choices when horizontal space is available."); }) },
            SpecSection { title: "Behavior", content: RenderSlot::new(|| { subheading("Appearing".into()); prose("The menu expands from its trigger and adapts its direction to available space. A selected option shows a ripple; the trigger remains visually pressed while the menu is open."); gap(Dp(24.0)); launch(); gap(Dp(24.0)); subheading("Selecting".into()); prose("Single selection replaces the previous choice. Multiple selection keeps the menu open until dismissal. Use shape, color and a checkmark to reinforce selection."); gap(Dp(24.0)); states(); gap(Dp(24.0)); subheading("Scrolling".into()); prose("Use a persistent scrollbar when every item cannot fit at once. Avoid gaps in a scrolling menu."); }) },
            SpecSection { title: "Density", content: RenderSlot::new(|| {prose("A denser menu reduces vertical spacing on desktop. Keep interaction targets large enough for the intended input method.");}) },
        ] },
        ArticlePage { title: "Accessibility", intro: None, sections: vec![
            SpecSection { title: "Use cases", content: RenderSlot::new(|| { prose("People must be able to reach, open and dismiss menus, navigate their items and select an action using assistive technology."); gap(Dp(24.0)); launch(); }) },
            SpecSection { title: "Interaction & style", content: RenderSlot::new(|| { prose("Selection changes both shape and color. Add a checkmark so selection is recognizable without color alone. Keep one action per item, including custom slot content."); gap(Dp(24.0)); states(); }) },
            SpecSection { title: "Focus", content: RenderSlot::new(|| { prose("Opening a menu places focus on its first item. Escape, selection or an outside press dismisses it and restores focus to its trigger. Dividers and gaps are not focus targets."); }) },
            SpecSection { title: "Keyboard navigation", content: RenderSlot::new(keyboard) },
            SpecSection { title: "Interactability", content: RenderSlot::new(|| {prose("Disabled choices remain visible to explain unavailable actions. They cannot activate. Separators convey grouping without adding an action.");}) },
            SpecSection { title: "Labeling elements", content: RenderSlot::new(|| { prose("Use each visible menu item label as its accessible name. Icons accompanying text are decorative. A submenu trigger must communicate that it opens another menu."); spec_table(vec!["Element".into(),"Label".into(),"Role".into()],vec![vec!["Menu item".into(),"Preview".into(),"Menu item".into()]]); }) },
        ] },
    ]);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn variants() {
    subheading("Vertical menus".into());
    prose(
        "Vertical menus support rounded corners, standard and vibrant colors, selection and grouped actions.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                menu_tile(
                    true,
                    MenuColorStyle::Standard,
                    "1  Vertical menu with gap".into(),
                );
                menu_tile(
                    false,
                    MenuColorStyle::Standard,
                    "2  Vertical menu with divider".into(),
                );
            });
    }));
    gap(Dp(24.0));
    subheading("Baseline variant".into());
    prose("The baseline menu remains available. Vertical menus are recommended for new designs.");
}

#[tessera]
fn configurations() {
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                configuration(false);
                configuration(true);
            });
    }));
}

#[tessera]
fn configuration(grouped: bool) {
    let selected = remember(|| 2_usize);
    column()
        .modifier(Modifier::new().width(Dp(216.0)))
        .children(move || {
            row()
                .main_axis_alignment(MainAxisAlignment::End)
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    if grouped {
                        split_button_layout()
                            .leading_button(|| {
                                split_leading_button()
                                    .variant(SplitButtonVariant::Tonal)
                                    .on_click(|| {})
                                    .content(|| {
                                        icon().vector(filled::EDIT_SVG).size(Dp(20.0));
                                        text().content("Label");
                                    });
                            })
                            .trailing_button(|| {
                                split_trailing_button()
                                    .variant(SplitButtonVariant::Tonal)
                                    .expanded(true)
                                    .accessibility_label("Grouped menu options")
                                    .on_click(|| {});
                            });
                    } else {
                        icon_button()
                            .filled_tonal()
                            .icon(filled::MORE_VERT_SVG)
                            .accessibility_label("Standard menu options")
                            .on_click(|| {});
                    }
                });
            if grouped {
                menu_group()
                    .position(MenuPosition::First)
                    .children(move || {
                        menu_item()
                            .label("Item 1")
                            .leading_icon(filled::VISIBILITY_SVG)
                            .selected(selected.get() == 0)
                            .on_click(move || selected.set(0));
                        menu_item()
                            .label("Item 2")
                            .leading_icon(filled::CONTENT_COPY_SVG)
                            .trailing_text("⌘C")
                            .selected(selected.get() == 1)
                            .on_click(move || selected.set(1));
                        menu_item()
                            .label("Item 3")
                            .leading_icon(filled::CHECK_SVG)
                            .selected(selected.get() == 2)
                            .on_click(move || selected.set(2));
                    });
                menu_group().position(MenuPosition::Last).children(move || {
                    menu_item()
                        .label("Item 4")
                        .leading_icon(filled::PERSON_SVG)
                        .trailing_icon(filled::ARROW_RIGHT_SVG)
                        .on_click(move || selected.set(3));
                    menu_item()
                        .label("Item 5")
                        .leading_icon(filled::SETTINGS_SVG)
                        .on_click(move || selected.set(4));
                });
            } else {
                menu_group().children(move || {
                    menu_item()
                        .label("Refresh")
                        .trailing_icon(filled::REFRESH_SVG)
                        .on_click(move || selected.set(0));
                    menu_item()
                        .label("Settings")
                        .trailing_icon(filled::SETTINGS_SVG)
                        .on_click(move || selected.set(1));
                    menu_item()
                        .label("Help")
                        .trailing_icon(filled::HELP_OUTLINE_SVG)
                        .on_click(move || selected.set(2));
                    menu_item()
                        .label("More")
                        .trailing_icon(filled::MORE_VERT_SVG)
                        .on_click(move || selected.set(3));
                });
            }
            gap(Dp(24.0));
            prose(if grouped { "2  Grouped" } else { "1  Standard" });
        });
}

#[tessera]
fn menu_tile(grouped: bool, style: MenuColorStyle, caption: String) {
    column().children(move || {
        menu_sample(grouped, style);
        gap(Dp(24.0));
        prose(caption.clone());
    });
}

#[tessera]
fn menu_sample(grouped: bool, style: MenuColorStyle) {
    let selected = remember(|| 2_usize);
    column()
        .modifier(Modifier::new().width(Dp(216.0)))
        .children(move || {
            menu_group()
                .color_style(style)
                .position(if grouped {
                    MenuPosition::First
                } else {
                    MenuPosition::Standalone
                })
                .children(move || {
                    for (index, label) in ["Item 1", "Item 2", "Item 3"].into_iter().enumerate() {
                        menu_item()
                            .label(label)
                            .selected(selected.get() == index)
                            .close_on_click(false)
                            .leading_icon(if index == 0 {
                                filled::VISIBILITY_SVG
                            } else if index == 1 {
                                filled::CONTENT_COPY_SVG
                            } else {
                                filled::EDIT_SVG
                            })
                            .trailing_text(if index == 1 { "⌘C" } else { "" })
                            .on_click(move || selected.set(index));
                    }
                    if !grouped {
                        horizontal_divider();
                        menu_item()
                            .label("Item 4")
                            .leading_icon(filled::CLOUD_SVG)
                            .trailing_icon(filled::ARROW_RIGHT_SVG)
                            .on_click(move || selected.set(3));
                    }
                });
            if grouped {
                menu_group()
                    .position(MenuPosition::Last)
                    .color_style(style)
                    .children(move || {
                        menu_item()
                            .label("Item 4")
                            .leading_icon(filled::CLOUD_SVG)
                            .trailing_icon(filled::ARROW_RIGHT_SVG)
                            .on_click(move || selected.set(3));
                    });
            }
        });
}

#[tessera]
fn launch() {
    let menu = remember(MenuController::new);
    let anchor = remember(PxRect::default);
    let origin = remember(PxRect::default);
    let notice = remember(|| "Choose an action".to_owned());
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(280.0))
                .on_globally_positioned(move |rect| {
                    if origin.get() != rect {
                        origin.set(rect);
                    }
                }),
        )
        .children(move || {
            let a = anchor.get();
            let o = origin.get();
            menu_provider()
                .controller(menu)
                .grouped(true)
                .anchor(MenuAnchor::new(
                    PxPosition::new(a.x - o.x, a.y - o.y),
                    PxSize::new(a.width, a.height),
                ))
                .main_content(move || {
                    column().children(move || {
                        button()
                            .outlined()
                            .modifier(Modifier::new().on_globally_positioned(move |rect| {
                                if anchor.get() != rect {
                                    anchor.set(rect);
                                }
                            }))
                            .on_click(move || menu.with_mut(MenuController::toggle))
                            .child(|| {
                                text().content("Open menu");
                            });
                        gap(Dp(16.0));
                        prose(notice.get());
                    });
                })
                .menu_content(move || {
                    menu_group()
                        .position(MenuPosition::First)
                        .children(move || {
                            menu_item()
                                .label("Preview")
                                .leading_icon(filled::VISIBILITY_SVG)
                                .on_click(move || notice.set("Preview selected".into()));
                            menu_item()
                                .label("Share")
                                .leading_icon(filled::SHARE_SVG)
                                .submenu_content(move || {
                                    menu_item()
                                        .label("Copy link")
                                        .leading_icon(filled::LINK_SVG)
                                        .on_click(move || notice.set("Link copied".into()));
                                    menu_item()
                                        .label("Email")
                                        .leading_icon(filled::EMAIL_SVG)
                                        .on_click(move || notice.set("Email selected".into()));
                                });
                        });
                    menu_group().position(MenuPosition::Last).children(move || {
                        menu_item()
                            .label("Delete")
                            .leading_icon(filled::DELETE_SVG)
                            .on_click(move || notice.set("Delete selected".into()));
                    });
                });
        });
}

#[tessera]
fn menu_figure(anatomy: bool, measurements: bool, style: MenuColorStyle) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(480.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(watch(geometry, "container").width(Dp(240.0)))
                .children(move || {
                    menu_group()
                        .position(MenuPosition::First)
                        .color_style(style)
                        .children(move || {
                            boxed()
                                .modifier(watch(geometry, "first").fill_max_width())
                                .children(move || {
                                    menu_item()
                                        .label("Item 1")
                                        .leading_icon(filled::VISIBILITY_SVG)
                                        .selected(selected.get() == 1)
                                        .on_click(move || selected.set(1));
                                });
                            boxed()
                                .modifier(watch(geometry, "second").fill_max_width())
                                .children(move || {
                                    menu_item()
                                        .label("Item 2")
                                        .leading_icon(filled::CONTENT_COPY_SVG)
                                        .trailing_icon(filled::ARROW_RIGHT_SVG)
                                        .selected(selected.get() == 2)
                                        .on_click(move || selected.set(2));
                                });
                            if anatomy {
                                boxed()
                                    .modifier(watch(geometry, "divider").fill_max_width())
                                    .children(|| {
                                        horizontal_divider();
                                    });
                            }
                            boxed()
                                .modifier(watch(geometry, "third").fill_max_width())
                                .children(move || {
                                    menu_item()
                                        .label("Item 3")
                                        .leading_icon(filled::EDIT_SVG)
                                        .trailing_icon(filled::ARROW_RIGHT_SVG)
                                        .on_click(move || selected.set(3));
                                });
                        });
                    menu_group()
                        .position(MenuPosition::Last)
                        .color_style(style)
                        .children(move || {
                            boxed()
                                .modifier(
                                    watch(geometry, "label")
                                        .fill_max_width()
                                        .height(Dp(32.0))
                                        .padding_symmetric(Dp(16.0), Dp::ZERO),
                                )
                                .alignment(Alignment::CenterStart)
                                .children(|| {
                                    text().content("Label text").style(type_style(14.0, 20.0));
                                });
                            boxed()
                                .modifier(watch(geometry, "fourth").fill_max_width())
                                .children(move || {
                                    menu_item()
                                        .label("Item 4")
                                        .supporting_text("Supporting text")
                                        .leading_icon(filled::CLOUD_SVG)
                                        .trailing_text("⌘C")
                                        .on_click(move || selected.set(4));
                                });
                        });
                });
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    menu_annotations(geometry, anatomy, measurements);
                    if anatomy && let Some(third) = relative(&geometry.get(), "third") {
                        badge_with_content()
                            .container_color(scheme.tertiary_container)
                            .content_color(scheme.on_tertiary_container)
                            .modifier(Modifier::new().offset(
                                Dp(third.x + third.w - 80.0),
                                Dp(third.y + third.h / 2.0 - 9.0),
                            ))
                            .content(|| {
                                text().content("New").style(type_style(12.0, 16.0));
                            });
                    }
                });
        });
    if anatomy {
        gap(Dp(16.0));
        prose(
            "1  Menu item    2  Leading icon    3  Menu item text    4  Trailing icon\n5  Badge    6  Trailing text    7  Container    8  Supporting text\n9  Label text    10  Gap    11  Divider",
        );
    }
    if measurements {
        gap(Dp(24.0));
        tokens();
    }
}

#[tessera]
fn menu_annotations(geometry: State<Geometry>, anatomy: bool, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(c), Some(first), Some(second), Some(third), Some(last), Some(label)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "first"),
        relative(&values, "second"),
        relative(&values, "third"),
        relative(&values, "fourth"),
        relative(&values, "label"),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let mut svg = String::new();
    if anatomy {
        for (n, x, y, tx, ty) in [
            (1, c.x - 64.0, second.y + 8.0, c.x + 4.0, second.y + 24.0),
            (2, c.x + 12.0, c.y - 64.0, c.x + 26.0, first.y + 24.0),
            (3, c.x + 62.0, c.y - 64.0, c.x + 78.0, first.y + 24.0),
            (
                4,
                c.x + c.w + 36.0,
                second.y + 8.0,
                c.x + c.w - 24.0,
                second.y + 24.0,
            ),
            (
                5,
                c.x + c.w + 36.0,
                third.y + 8.0,
                c.x + c.w - 60.0,
                third.y + 24.0,
            ),
            (
                6,
                c.x + c.w + 36.0,
                last.y + 8.0,
                c.x + c.w - 24.0,
                last.y + 24.0,
            ),
            (
                7,
                c.x + 136.0,
                c.y + c.h + 24.0,
                c.x + 148.0,
                c.y + c.h - 8.0,
            ),
            (8, c.x + 76.0, c.y + c.h + 24.0, c.x + 88.0, last.y + 34.0),
            (9, c.x - 64.0, label.y, c.x + 24.0, label.y + 16.0),
            (10, c.x - 64.0, label.y - 32.0, c.x + 4.0, label.y),
            (11, c.x - 64.0, third.y - 16.0, c.x + 4.0, third.y),
        ] {
            if (tx - x).abs() > (ty - y).abs() {
                dotted(&mut svg, x + 12.0, ty, tx - x - 12.0, false, "#49454f");
            } else {
                dotted(&mut svg, tx, y + 16.0, ty - y - 16.0, true, "#49454f");
            }
            annotations::marker(n, Dp(x), Dp(y));
        }
    }
    if measurements {
        svg.push_str(&format!("<rect x='{}' y='{}' width='12' height='{}' fill='#ee7da4' fill-opacity='.3'/><rect x='{}' y='{}' width='{}' height='2' fill='#ee7da4' fill-opacity='.5'/>",c.x,c.y,c.h,c.x,c.y+first.h,c.w));
        for (label, x, y) in [
            ("16", c.x + c.w + 48.0, c.y),
            ("48", c.x + c.w + 48.0, second.y + 16.0),
            ("20", c.x - 52.0, third.y + 16.0),
            ("2", c.x - 36.0, label.y - 8.0),
            ("32", c.x - 48.0, label.y + 10.0),
            ("12", c.x - 2.0, c.y + c.h + 24.0),
            ("8", c.x + c.w - 42.0, c.y + c.h + 24.0),
            ("12", c.x + c.w - 8.0, c.y + c.h + 24.0),
        ] {
            annotations::dimension(label.into(), Dp(x), Dp(y));
        }
        dotted(
            &mut svg,
            c.x + c.w,
            second.y + second.h / 2.0,
            44.0,
            false,
            "#ee7da4",
        );
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn colors() {
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                menu_tile(true, MenuColorStyle::Standard, "1  Standard".into());
                menu_tile(true, MenuColorStyle::Vibrant, "2  Vibrant".into());
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Standard".into(), "Vibrant".into()],
        vec![
            vec![
                "Container".into(),
                "Surface container low".into(),
                "Tertiary container".into(),
            ],
            vec![
                "Content".into(),
                "On surface".into(),
                "On tertiary container".into(),
            ],
            vec![
                "Selected container".into(),
                "Tertiary container".into(),
                "Tertiary".into(),
            ],
            vec![
                "Selected content".into(),
                "On tertiary container".into(),
                "On tertiary".into(),
            ],
        ],
    );
}

#[tessera]
fn states() {
    let selected = remember(|| true);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().width(Dp(240.0)))
            .children(move || {
                menu_group().children(move || {
                    menu_item()
                        .label("Selected item")
                        .selected(selected.get())
                        .leading_icon(filled::CHECK_SVG)
                        .close_on_click(false)
                        .on_click(move || selected.set(!selected.get()));
                    menu_item().label("Enabled item").on_click(|| {});
                    menu_item().label("Disabled item").enabled(false);
                });
            });
    }));
    gap(Dp(16.0));
    prose(
        "Enabled, disabled, hovered, focused, pressed and active submenu states. Hover, focus or press the live items to inspect their state layers.",
    );
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Menu item height".into(), "48dp touch target".into()],
            vec!["Leading / trailing icon".into(), "20dp".into()],
            vec!["Leading item padding".into(), "12dp".into()],
            vec!["Space between icon and label".into(), "8dp".into()],
            vec!["Group gap".into(), "2dp".into()],
            vec!["Group vertical padding".into(), "2dp".into()],
            vec!["Label row height".into(), "32dp".into()],
            vec!["Outer corner radius".into(), "16dp".into()],
            vec!["Corners adjoining gap".into(), "8dp".into()],
            vec!["Selected item radius".into(), "12dp".into()],
        ],
    );
}

#[tessera]
fn baseline() {
    prose(
        "Baseline menus retain square-shaped 4dp corners. Expressive vertical menus are recommended for new designs.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_group()
            .shape(Shape::rounded_rectangle(Dp(4.0)))
            .modifier(Modifier::new().width(Dp(240.0)))
            .children(|| {
                menu_item()
                    .height(Dp(48.0))
                    .label("Cut")
                    .leading_icon(filled::CONTENT_CUT_SVG)
                    .on_click(|| {});
                menu_item()
                    .height(Dp(48.0))
                    .label("Copy")
                    .leading_icon(filled::CONTENT_COPY_SVG)
                    .on_click(|| {});
                horizontal_divider();
                menu_item()
                    .height(Dp(48.0))
                    .label("Paste")
                    .leading_icon(filled::CONTENT_PASTE_SVG)
                    .on_click(|| {});
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Width".into(), "112–280dp".into()],
            vec!["Corner radius".into(), "4dp".into()],
            vec!["Item height".into(), "48dp".into()],
            vec!["Horizontal padding".into(), "12dp".into()],
            vec!["Icon size".into(), "24dp".into()],
            vec!["Divider padding".into(), "8dp top and bottom".into()],
        ],
    );
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Action".into()],
        vec![
            vec!["Tab".into(), "Focus a menu".into()],
            vec![
                "Space or Enter".into(),
                "Open a menu or select an item".into(),
            ],
            vec!["Up / Down".into(), "Move between items".into()],
            vec!["Left / Right".into(), "Close / open a submenu".into()],
            vec!["Letters".into(), "Focus the next matching item".into()],
            vec!["Escape".into(), "Close the menu".into()],
        ],
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use std::time::Duration;

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    };

    use super::*;

    struct Pipelines;
    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[tessera]
    fn fixture() {
        material_theme().child(|| {
            launch();
        });
    }

    fn click(renderer: &mut HeadlessRenderer, label: &str) {
        let node = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap_or_else(|| panic!("missing {label}"))
            .1;
        let bounds = node.bounds().unwrap();
        let position = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(20);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn contextual_menu_executes_action_and_closes_its_popup() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 600,
                height: 500,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(4);
        click(&mut renderer, "Open menu");
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Preview")
                    && node.role() == tessera_ui::accesskit::Role::MenuItem)
        );
        click(&mut renderer, "Preview");
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Preview selected"))
        );
        assert!(
            !snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Preview")
                    && node.role() == tessera_ui::accesskit::Role::MenuItem)
        );
    }
}
