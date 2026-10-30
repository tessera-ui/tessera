//! Expressive menu specifications and contextual actions.
//!
//! ## Usage
//!
//! Inspect menu groups, selection and keyboard interaction.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    badge::badge_with_content,
    boxed::boxed,
    button::button,
    column::column,
    divider::horizontal_divider,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    menus::{
        MenuAnchor, MenuColorStyle, MenuController, MenuItemSelectionMode, MenuPosition,
        menu_group, menu_item, menu_provider,
    },
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    split_buttons::{SplitButtonDefaults, SplitButtonSize},
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, PxPosition, PxRect, PxSize, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use super::{
    ACCENT, INK,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[path = "menus_accessibility.rs"]
mod accessibility_figures;

#[path = "menus_selection.rs"]
mod selection_figures;

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
        ArticlePage { title: "Overview", intro: Some(RenderSlot::new(|| { prose("Menus display a list of choices on a temporary surface. Use a menu to show a temporary set of actions; use a toolbar when actions must stay on screen. Menus can open from icon buttons, split buttons, text fields or a secondary click."); gap(Dp(32.0)); launch(); })), sections: vec![
            SpecSection { title: "M3 Expressive update", content: RenderSlot::new(|| { prose("Vertical menus add rounded groups, refined submenu motion, standard and vibrant colors and additional selection states. Gaps can join groups for a more flexible layout. Vertical menus are recommended for new designs, and the baseline menu remains available."); gap(Dp(24.0)); variants(); }) },
            SpecSection { title: "Differences from M2", content: RenderSlot::new(|| { prose("Color: new color mappings and compatibility with dynamic color keep menu surfaces distinct from the content behind them. Variants: the dropdown menu and the exposed dropdown menu are both referred to as a menu, because they differ only in the element that opens them."); }) },
        ] },
        ArticlePage { title: "Guidelines", intro: None, sections: vec![
            SpecSection { title: "Usage", content: RenderSlot::new(|| { prose("Use menus for temporary sets of actions. Keep vibrant menus for higher emphasis. Open them from a button, icon, field or contextual secondary click."); gap(Dp(24.0)); launch(); gap(Dp(32.0)); subheading("Menu groups".into()); prose("Bundle related actions with a divider or a small gap. Use no more than one or two gaps. Use dividers instead of gaps in scrolling menus."); gap(Dp(24.0)); configurations(); }) },
            SpecSection { title: "Anatomy", content: RenderSlot::new(|| { menu_figure(true, false, MenuColorStyle::Standard); }) },
            SpecSection { title: "Flexibility & slots", content: RenderSlot::new(|| { prose("Use simple slot content such as images, progress indicators or color swatches. Keep a single action per menu item. Preserve menu item padding and a target of at least 48 × 48dp. Nested buttons or switches introduce competing actions and break the menu interaction model."); }) },
            SpecSection { title: "Placement", content: RenderSlot::new(|| { prose("Position the menu next to its trigger, below it or over it. Reposition near a window edge so the surface stays visible. Submenus open beside the parent item without covering it."); gap(Dp(24.0)); launch(); }) },
            SpecSection { title: "Adaptive design", content: RenderSlot::new(|| { subheading("Compact breakpoints".into()); prose("Consider a bottom sheet on compact screens when actions need more room or longer labels."); gap(Dp(24.0)); subheading("Other breakpoints".into()); prose("Keep menus near their content on medium and expanded screens. Submenus can organize more complex sets of choices when horizontal space is available."); }) },
            SpecSection { title: "Behavior", content: RenderSlot::new(|| { subheading("Appearing".into()); prose("The menu expands from its trigger and adapts its direction to available space. A selected option shows a ripple; the trigger remains visually pressed while the menu is open."); gap(Dp(24.0)); launch(); gap(Dp(24.0)); subheading("Selecting".into()); prose("Single selection replaces the previous choice. Multiple selection keeps the menu open until dismissal. Use shape, color and a checkmark to reinforce selection."); gap(Dp(24.0)); selection_figures::selection_figure(); gap(Dp(24.0)); subheading("Scrolling".into()); prose("Use a persistent scrollbar when every item cannot fit at once. Avoid gaps in a scrolling menu."); }) },
            SpecSection { title: "Focus", content: RenderSlot::new(|| { prose("As focus moves between submenus, the active menu becomes more rounded and the inactive menu becomes less rounded. Hover or focus Share to open its submenu and inspect the shape change."); gap(Dp(24.0)); launch(); }) },
            SpecSection { title: "Density", content: RenderSlot::new(|| {prose("On web, density settings reduce vertical spacing. Keep interaction targets large enough for the intended input method.");}) },
        ] },
        ArticlePage { title: "Accessibility", intro: None, sections: vec![
            SpecSection { title: "Use cases", content: RenderSlot::new(|| { prose("People must be able to reach, open and dismiss menus, navigate their items and select an action using assistive technology."); gap(Dp(24.0)); launch(); }) },
            SpecSection { title: "Interaction & style", content: RenderSlot::new(|| { prose("Selection changes both shape and color. Add a checkmark so selection is recognizable without color alone."); gap(Dp(24.0)); accessibility_figures::state_selection(); }) },
            SpecSection { title: "Flexibility & slots", content: RenderSlot::new(|| { prose("Preserve each menu item's padding and a target of at least 48 × 48dp. Slot content must follow menu interaction patterns and perform one action. Avoid nested buttons, switches or other independent actions."); }) },
            SpecSection { title: "Focus", content: RenderSlot::new(|| { prose("Opening a menu places focus on its first item. Focus after dismissal depends on the surrounding task; this example returns focus to its trigger. Dividers and gaps are not focus targets."); }) },
            SpecSection { title: "Keyboard navigation", content: RenderSlot::new(keyboard) },
            SpecSection { title: "Interactability", content: RenderSlot::new(|| {prose("Disabled choices remain visible and can receive focus, but cannot activate. Dividers and gaps cannot receive focus.");}) },
            SpecSection { title: "Labeling elements", content: RenderSlot::new(|| { prose("Use each visible menu item label as its accessible name. Icons accompanying text are decorative. A submenu trigger must communicate that it opens another menu."); gap(Dp(24.0)); accessibility_figures::labeling(); }) },
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
    subheading("Vertical menus layout".into());
    gap(Dp(24.0));
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
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().width(Dp(216.0)))
        .children(move || {
            row()
                .main_axis_alignment(MainAxisAlignment::End)
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    if grouped {
                        row().children(move || {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(Modifier::new().height(Dp(40.0)).background_with_shape(
                                    scheme.secondary_container,
                                    SplitButtonDefaults::leading_shape(SplitButtonSize::Small),
                                ))
                                .children(move || {
                                    row()
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .modifier(
                                            Modifier::new().padding_symmetric(Dp(16.0), Dp::ZERO),
                                        )
                                        .children(move || {
                                            icon()
                                                .vector(filled::EDIT_SVG)
                                                .size(Dp(20.0))
                                                .tint(scheme.on_secondary_container);
                                            boxed().modifier(Modifier::new().width(Dp(8.0)));
                                            text()
                                                .content("Label")
                                                .color(scheme.on_secondary_container);
                                        });
                                });
                            boxed().modifier(Modifier::new().width(Dp(2.0)));
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    Modifier::new()
                                        .size(Dp(48.0), Dp(40.0))
                                        .background_with_shape(
                                            scheme.secondary_container,
                                            SplitButtonDefaults::trailing_shape(
                                                SplitButtonSize::Small,
                                            ),
                                        ),
                                )
                                .children(move || {
                                    icon()
                                        .vector(filled::ARROW_DROP_DOWN_SVG)
                                        .rotation(180.0)
                                        .size(Dp(22.0))
                                        .tint(scheme.on_secondary_container);
                                });
                        });
                    } else {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(
                                Modifier::new()
                                    .size(Dp(40.0), Dp(40.0))
                                    .background_with_shape(
                                        scheme.secondary_container,
                                        Shape::rounded_rectangle(Dp(20.0)),
                                    ),
                            )
                            .children(move || {
                                icon()
                                    .vector(filled::MORE_VERT_SVG)
                                    .tint(scheme.on_secondary_container);
                            });
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
                            .selection_mode(MenuItemSelectionMode::Radio)
                            .on_click(move || selected.set(0));
                        menu_item()
                            .label("Item 2")
                            .leading_icon(filled::CONTENT_COPY_SVG)
                            .trailing_text("⌘C")
                            .selected(selected.get() == 1)
                            .selection_mode(MenuItemSelectionMode::Radio)
                            .on_click(move || selected.set(1));
                        menu_item()
                            .label("Item 3")
                            .leading_icon(filled::CHECK_SVG)
                            .selected(selected.get() == 2)
                            .selection_mode(MenuItemSelectionMode::Radio)
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
                            .selection_mode(MenuItemSelectionMode::Radio)
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
                                        .selection_mode(MenuItemSelectionMode::Radio)
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
                                        .selection_mode(MenuItemSelectionMode::Radio)
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
    for style in [MenuColorStyle::Standard, MenuColorStyle::Vibrant] {
        subheading(
            if style == MenuColorStyle::Standard {
                "Standard color roles"
            } else {
                "Vibrant color roles"
            }
            .into(),
        );
        gap(Dp(16.0));
        row()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                for dark in [false, true] {
                    boxed()
                        .modifier(Modifier::new().weight(1.0))
                        .children(move || {
                            material_theme()
                                .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                                .child(move || {
                                    menu_color_figure(style, dark);
                                });
                        });
                }
            });
        gap(Dp(24.0));
        spec_table(
            vec!["Element".into(), "Color role".into()],
            menu_color_roles(style)
                .iter()
                .enumerate()
                .map(|(index, role)| vec![(index + 1).to_string(), (*role).into()])
                .collect(),
        );
        gap(Dp(32.0));
    }
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

const COLOR_ROWS: [&str; 5] = [
    "color-row-1",
    "color-row-2",
    "color-row-3",
    "color-row-4",
    "color-row-5",
];

fn menu_color_roles(style: MenuColorStyle) -> [&'static str; 11] {
    if style == MenuColorStyle::Standard {
        [
            "On surface variant",
            "On surface",
            "On surface (state layer)",
            "Surface container low",
            "On surface variant",
            "On surface variant",
            "Tertiary container (selected)",
            "On tertiary container (selected)",
            "On surface variant",
            "On surface variant",
            "On tertiary container (selected)",
        ]
    } else {
        [
            "On tertiary container",
            "On tertiary container",
            "On tertiary container (state layer)",
            "Tertiary container",
            "On tertiary container",
            "On tertiary container",
            "Tertiary (selected)",
            "On tertiary (selected)",
            "On tertiary container",
            "On tertiary container",
            "On tertiary (selected)",
        ]
    }
}

#[tessera]
fn menu_color_figure(style: MenuColorStyle, dark: bool) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 3_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let mode = if dark { "Dark" } else { "Light" };
    let emphasis = if style == MenuColorStyle::Standard {
        "standard"
    } else {
        "vibrant"
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(536.0))
                .background(scheme.surface)
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!("{mode} {emphasis} menu color roles")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            column()
                .modifier(watch(geometry, "color-container").width(Dp(210.0)))
                .children(move || {
                    menu_group()
                        .position(MenuPosition::First)
                        .color_style(style)
                        .children(move || {
                            for index in 0..4 {
                                boxed()
                                    .modifier(watch(geometry, COLOR_ROWS[index]).fill_max_width())
                                    .children(move || {
                                        if index == 0 {
                                            menu_item()
                                                .label(format!("Item {}", index + 1))
                                                .selection_mode(MenuItemSelectionMode::Radio)
                                                .selected(selected.get() == index)
                                                .color_style(style)
                                                .close_on_click(false)
                                                .leading_icon(filled::VISIBILITY_SVG)
                                                .on_click(move || selected.set(index));
                                        } else if index == 1 {
                                            menu_item()
                                                .label(format!("Item {}", index + 1))
                                                .selection_mode(MenuItemSelectionMode::Radio)
                                                .selected(selected.get() == index)
                                                .color_style(style)
                                                .close_on_click(false)
                                                .leading_icon(filled::CONTENT_COPY_SVG)
                                                .trailing_text("⌘C")
                                                .on_click(move || selected.set(index));
                                        } else if index == 2 {
                                            menu_item()
                                                .label(format!("Item {}", index + 1))
                                                .selection_mode(MenuItemSelectionMode::Radio)
                                                .selected(selected.get() == index)
                                                .color_style(style)
                                                .close_on_click(false)
                                                .leading_icon(filled::EDIT_SVG)
                                                .trailing_icon(filled::ARROW_RIGHT_SVG)
                                                .on_click(move || selected.set(index));
                                        } else {
                                            menu_item()
                                                .label(format!("Item {}", index + 1))
                                                .selection_mode(MenuItemSelectionMode::Radio)
                                                .selected(selected.get() == index)
                                                .color_style(style)
                                                .close_on_click(false)
                                                .on_click(move || selected.set(index));
                                        }
                                    });
                            }
                        });
                    menu_group()
                        .position(MenuPosition::Last)
                        .color_style(style)
                        .children(move || {
                            boxed()
                                .alignment(Alignment::CenterStart)
                                .modifier(
                                    watch(geometry, "color-label")
                                        .fill_max_width()
                                        .height(Dp(32.0))
                                        .padding_symmetric(Dp(12.0), Dp::ZERO),
                                )
                                .children(move || {
                                    text()
                                        .content("Label text")
                                        .style(type_style(14.0, 20.0))
                                        .color(if style == MenuColorStyle::Standard {
                                            scheme.on_surface_variant
                                        } else {
                                            scheme.on_tertiary_container
                                        });
                                });
                            boxed()
                                .modifier(watch(geometry, COLOR_ROWS[4]).fill_max_width())
                                .children(move || {
                                    menu_item()
                                        .label("Item 5")
                                        .supporting_text("Supporting text")
                                        .trailing_icon(filled::ARROW_RIGHT_SVG)
                                        .selection_mode(MenuItemSelectionMode::Radio)
                                        .selected(selected.get() == 4)
                                        .color_style(style)
                                        .close_on_click(false)
                                        .on_click(move || selected.set(4));
                                });
                        });
                });
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    menu_color_annotations(
                        style,
                        dark,
                        selected.get(),
                        geometry.with(annotations::relative_geometry),
                    );
                });
        });
}

#[tessera]
fn menu_color_annotations(style: MenuColorStyle, dark: bool, selected: usize, values: Geometry) {
    let (
        Some(root),
        Some(container),
        Some(first),
        Some(second),
        Some(last),
        Some(label),
        Some(active),
    ) = (
        values.get("figure"),
        relative(&values, "color-container"),
        relative(&values, COLOR_ROWS[0]),
        relative(&values, COLOR_ROWS[1]),
        relative(&values, COLOR_ROWS[4]),
        relative(&values, "color-label"),
        relative(&values, COLOR_ROWS[selected]),
    )
    else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let roles = menu_color_roles(style);
    let tone = if dark { "#cac4d0" } else { "#49454f" };
    let mut svg = String::new();
    for (index, (x, y, tx, ty)) in [
        (
            first.x + 14.0,
            container.y - 80.0,
            first.x + 26.0,
            first.y + 24.0,
        ),
        (
            first.x + 53.0,
            container.y - 80.0,
            first.x + 65.0,
            first.y + 24.0,
        ),
        (
            second.x + 134.0,
            container.y - 80.0,
            second.x + 146.0,
            second.y + 24.0,
        ),
        (
            container.x + container.w + 38.0,
            container.y - 2.0,
            container.x + container.w - 18.0,
            container.y + 14.0,
        ),
        (
            container.x + container.w + 38.0,
            second.y + 8.0,
            second.x + second.w - 22.0,
            second.y + 24.0,
        ),
        (
            container.x + container.w + 38.0,
            last.y + 12.0,
            last.x + last.w - 24.0,
            last.y + 28.0,
        ),
        (
            active.x + 122.0,
            container.y + container.h + 28.0,
            active.x + 134.0,
            active.y + active.h - 12.0,
        ),
        (
            active.x + 77.0,
            container.y + container.h + 28.0,
            active.x + 89.0,
            active.y + 24.0,
        ),
        (
            last.x + 27.0,
            container.y + container.h + 28.0,
            last.x + 39.0,
            last.y + 40.0,
        ),
        (
            container.x - 64.0,
            label.y + 2.0,
            label.x + 24.0,
            label.y + 18.0,
        ),
        (
            container.x - 64.0,
            active.y + 8.0,
            active.x + 26.0,
            active.y + 24.0,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if (tx - x).abs() > (ty - y).abs() {
            dotted(
                &mut svg,
                (x + 12.0).min(tx),
                ty,
                (tx - x - 12.0).abs(),
                false,
                tone,
            );
        } else {
            dotted(
                &mut svg,
                tx,
                (y + 16.0).min(ty),
                (ty - y - 16.0).abs(),
                true,
                tone,
            );
        }
        menu_color_marker(index + 1, roles[index].into(), Dp(x), Dp(y));
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn menu_color_marker(number: usize, role: String, x: Dp, y: Dp) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0), Dp(32.0))
                .background_with_shape(scheme.on_surface, Shape::rounded_rectangle(Dp(16.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Menu color role {number}: {role}")),
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.surface);
        });
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
                        .selection_mode(MenuItemSelectionMode::Radio)
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
            vec![
                "Up / Down".into(),
                "Open a closed menu or move between items in an open menu".into(),
            ],
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
        click(&mut renderer, "Open menu");
        click(&mut renderer, "Share");
        click(&mut renderer, "Copy link");
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Link copied"))
        );
        assert!(!snapshot.nodes.iter().any(|(_, node)| {
            node.role() == tessera_ui::accesskit::Role::MenuItem
                && matches!(
                    node.label(),
                    Some("Preview" | "Share" | "Copy link" | "Email")
                )
        }));
    }
}
