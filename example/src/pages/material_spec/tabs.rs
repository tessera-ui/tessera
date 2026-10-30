//! Primary and secondary tab specification articles.
//!
//! ## Usage
//!
//! Explore content destinations, scrollable rows and keyboard activation.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    badge::{badge_with_content, badged_box},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    tabs::{TabActivationMode, TabsVariant, tab, tabs},
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, modifier::FocusModifierExt as _, remember,
    tessera, use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap,
    navigation::article_pages::figure_marker,
    panel, spec_table, subheading, type_style,
};

const DESTINATIONS: [&str; 5] = ["Flights", "Hotels", "Trains", "Buses", "Ferries"];

#[tessera]
pub(super) fn tabs_spec() {
    article(
        "Tabs",
        "Tabs organize content across different screens and views",
        vec![
            SpecSection {
                title: "Tokens and specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Primary tabs",
                content: RenderSlot::new(|| {
                    section(true);
                }),
            },
            SpecSection {
                title: "Secondary tabs",
                content: RenderSlot::new(|| {
                    section(false);
                }),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    )
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(overview_intro)),
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(overview_usage),
                },
                SpecSection {
                    title: "Differences from M2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "Guidelines",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(guidelines_usage),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(anatomy),
                },
                SpecSection {
                    title: "Choosing the tab variant",
                    content: RenderSlot::new(choosing_variant),
                },
                SpecSection {
                    title: "Placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "Responsive layout",
                    content: RenderSlot::new(responsive),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(behavior),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Use cases",
                    content: RenderSlot::new(use_cases),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "Initial focus",
                    content: RenderSlot::new(initial_focus),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(keyboard_navigation),
                },
                SpecSection {
                    title: "Labeling elements",
                    content: RenderSlot::new(labelling),
                },
            ],
        },
    ]);
}

#[tessera]
fn paragraph(value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .modifier(Modifier::new().fill_max_width());
}

#[tessera]
fn overview_intro() {
    paragraph(
        "Use tabs to group content into helpful categories. Primary tabs display an app's main content destinations at the top of the screen, often under a top app bar. Secondary tabs display related content within a content area and always sit below primary tabs."
            .into(),
    );
}

#[tessera]
fn overview_usage() {
    paragraph(
        "Tabs can scroll horizontally, so a UI can include as many tabs as needed. Keep every tab at the same hierarchy level and place tabs next to each other as peers."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, false, false, 0, 3);
    gap(Dp(24.0));
    native_tabs_figure(false, false, false, 0, 2);
    gap(Dp(32.0));
    paragraph(
        "Use tabs to group related content into clearly defined sets. Avoid tabs for sequential content that must be read in a particular order; create hierarchy inside the content with typography and open space instead."
            .into(),
    );
}

#[tessera]
fn differences() {
    paragraph(
        "Color: tabs use new color mappings and remain compatible with dynamic color. The container, active indicator and label colors follow the same roles as other Material components."
            .into(),
    );
    gap(Dp(24.0));
    paragraph(
        "Layout: icons and labels are vertically centered within the container, and the active indicator hugs the label instead of the full tab width."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
}

#[tessera]
fn guidelines_usage() {
    paragraph(
        "Tabs organize groups of related content that share the same level of hierarchy. Labels can combine an icon with short text, and every set is unified by one topic."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(24.0));
    native_tabs_figure(false, false, false, 0, 2);
    gap(Dp(32.0));
    paragraph(
        "Use tabs to categorize related groups of content into clearly defined sets. Don't use tabs to move through sequential content that needs to be read in a particular order; instead create hierarchy with typography and open space."
            .into(),
    );
}

#[tessera]
fn anatomy() {
    paragraph(
        "A tab row combines a container, an optional icon, an optional badge, the label, the divider and one active indicator."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 1, 3);
    gap(Dp(24.0));
    native_tabs_figure(false, false, false, 1, 2);
    gap(Dp(32.0));
    subheading("Container".into());
    gap(Dp(16.0));
    paragraph(
        "The container holds every tab in the set, extends across its content region and is divided into equal sections, one per tab. Its divider separates the row from the content below, which may scroll under it."
            .into(),
    );
    gap(Dp(24.0));
    subheading("Icon and badge (optional)".into());
    gap(Dp(16.0));
    paragraph(
        "Icons communicate the kind of content inside a tab and should be simple and recognizable. Keep the same icon treatment across a set, and rely on text labels when an icon's meaning may be unclear. Badges highlight notifications or updates for one tab and are limited to four characters, including a plus sign."
            .into(),
    );
    gap(Dp(24.0));
    subheading("Label and active indicator".into());
    gap(Dp(16.0));
    paragraph(
        "Labels clearly and succinctly describe the content of a tab. The active indicator underlines the selected tab while its label and icon change color, so selection never relies on the underline alone."
            .into(),
    );
}

#[tessera]
fn choosing_variant() {
    paragraph(
        "Use primary tabs when a single set is needed at the top of a content pane. Add secondary tabs when a screen needs another level of hierarchy; their indicator is simpler, but their function is identical."
            .into(),
    );
    gap(Dp(32.0));
    nested_tabs();
    gap(Dp(32.0));
    paragraph(
        "Tabs can join components such as app bars, sit inside a specific UI region, or be nested in cards and sheets. A tab row always controls the region displayed below it."
            .into(),
    );
}

#[tessera]
fn placement() {
    paragraph(
        "Tabs are displayed in a single row, and each tab is connected to the content it represents. As a set, all tabs are unified by a shared topic."
            .into(),
    );
    gap(Dp(32.0));
    tab_placement();
    gap(Dp(32.0));
    paragraph(
        "Secondary tabs are always placed below primary tabs. They are found within existing content to help users reach greater detail."
            .into(),
    );
}

#[tessera]
fn responsive() {
    paragraph(
        "For fixed tabs, size every tab from the width of the widest label and align the set to the center or leading edge of the body region with fluid margins. Avoid more than four visible fixed tabs; from five tabs the container becomes cramped."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, false, false, 0, 4);
    gap(Dp(32.0));
    paragraph(
        "When a set cannot fit on screen, use scrollable tabs. The first visible tab is offset 52dp from the leading edge and each tab is as wide as its label, which keeps longer titles readable."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, false, true, 0, 5);
}

#[tessera]
fn behavior() {
    paragraph(
        "Tab selection updates the content region below the set. Every tab can be hovered, focused and pressed, and the indicator moves into position once a new tab is engaged."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(32.0));
    subheading("Fixed tabs".into());
    gap(Dp(16.0));
    paragraph(
        "Fixed tabs display every tab at once, which suits quick switching between related content such as transportation methods on a map. Tap an individual tab, or swipe left and right inside the content area when it holds no competing swipeable content."
            .into(),
    );
    gap(Dp(24.0));
    subheading("Scrollable tabs".into());
    gap(Dp(16.0));
    paragraph(
        "Scrollable tabs hold longer labels and larger sets when all tabs cannot fit on screen. Padding stays consistent and offscreen tabs remain reachable by scrolling."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, false, true, 0, 5);
    gap(Dp(24.0));
    subheading("Scrolling content".into());
    gap(Dp(16.0));
    paragraph(
        "Tabs attached to a screen can stay at the top while content scrolls, or move offscreen and return when the page scrolls upward. Keep a tab row attached to its app bar so both move as one unit instead of scrolling behind it."
            .into(),
    );
}

#[tessera]
fn use_cases() {
    paragraph(
        "Users should be able to invoke navigation to a new destination with assistive technology, select an action or destination from an offscreen tab, and keep access to primary actions while the content is scrolled."
            .into(),
    );
}

#[tessera]
fn interaction_style() {
    subheading("Touch".into());
    gap(Dp(16.0));
    paragraph(
        "When a user taps a tab, a touch ripple appears as interaction feedback and the selected indicator becomes active and shifts into position once the touch has been engaged."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(24.0));
    subheading("Scrollable".into());
    gap(Dp(16.0));
    paragraph(
        "Scrollable tabs are used when a set cannot fit on screen and are best for browsing on touch interfaces. Swipe the row left or right, or use arrow keys, to reach every tab; avoid looping the row infinitely, which traps screen reader users. Select an individual tab with a tap, or with Space or Enter."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, false, true, 0, 5);
    gap(Dp(24.0));
    subheading("Cursor".into());
    gap(Dp(16.0));
    paragraph(
        "Hovering a tab shows a hover state as a visual cue that the tab is interactive. Clicking an active or inactive tab ripples and shifts the indicator into position."
            .into(),
    );
    gap(Dp(24.0));
    subheading("Keyboard".into());
    gap(Dp(16.0));
    paragraph(
        "Tabbing into the row shows a focus indicator on the active tab. Arrow keys move focus between tabs, Space or Enter activates the focused tab, and the same keys operate the items of an overflow menu."
            .into(),
    );
    gap(Dp(24.0));
    subheading("Density".into());
    gap(Dp(16.0));
    paragraph(
        "Don't apply density to tabs by default, because it lowers targets below 48 x 48dp. Offer density as an explicit choice and keep every control that changes it at least 48 x 48dp."
            .into(),
    );
}

#[tessera]
fn initial_focus() {
    paragraph(
        "On arrow or tab entry, the active indicator appears on the first interactive tab so the user knows which destination is selected. Further presses move to additional tabs, then to the next element on the page."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(32.0));
    paragraph(
        "Use arrow and tab keys to move through the set, and reserve Space or Enter for completing the action."
            .into(),
    );
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec![
                "Arrow".into(),
                "Focus lands on the next available navigation destination".into(),
            ],
            vec![
                "Space / Enter".into(),
                "Activates the focused navigation destination".into(),
            ],
            vec![
                "Arrow".into(),
                "Allows navigation through the items of an overflow menu".into(),
            ],
        ],
    );
}

#[tessera]
fn labelling() {
    paragraph(
        "When the visible text is ambiguous or a tab shows an icon alone, its accessibility label must describe the destination. Include extra context such as an unread count without changing the visible label."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
}

#[tessera]
fn sample(#[prop(into)] label: String, primary: bool, controls: Option<bool>) {
    let selected = remember(|| 0_usize);
    let icons = remember(|| false);
    let badge = remember(|| false);
    let scrollable = remember(|| false);
    let automatic = remember(|| false);
    let enabled = remember(|| true);
    let count = remember(|| 0_u32);
    let prefix = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(format!("{label} tabs preview")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if controls.unwrap_or(false) {
                let label = prefix.clone();
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        if primary {
                            button()
                                .tonal()
                                .selected(icons.get())
                                .accessibility_label("Show primary tab icons")
                                .on_click(move || icons.set(!icons.get()))
                                .child(|| {
                                    text().content("Icons");
                                });
                        }
                        button()
                            .tonal()
                            .selected(badge.get())
                            .accessibility_label(format!("{label} tab badge"))
                            .on_click(move || badge.set(!badge.get()))
                            .child(|| {
                                text().content("Badge");
                            });
                        button()
                            .tonal()
                            .selected(scrollable.get())
                            .accessibility_label(format!("{label} scrollable tabs"))
                            .on_click(move || {
                                scrollable.set(!scrollable.get());
                                if !scrollable.get() && selected.get() > 2 {
                                    selected.set(2);
                                }
                            })
                            .child(|| {
                                text().content("Scrollable");
                            });
                        button()
                            .tonal()
                            .selected(automatic.get())
                            .accessibility_label(format!("{label} automatic activation"))
                            .on_click(move || automatic.set(!automatic.get()))
                            .child(|| {
                                text().content("Automatic activation");
                            });
                        button()
                            .tonal()
                            .selected(!enabled.get())
                            .accessibility_label(format!("{label} disable Hotels"))
                            .on_click(move || enabled.set(!enabled.get()))
                            .child(|| {
                                text().content("Disable Hotels");
                            });
                        button()
                            .text()
                            .accessibility_label(format!("{label} select final tab"))
                            .on_click(move || selected.set(if scrollable.get() { 4 } else { 2 }))
                            .child(|| {
                                text().content("Select last");
                            });
                    });
                gap(Dp(24.0));
            }
            tabs()
                .variant(if primary {
                    TabsVariant::Primary
                } else {
                    TabsVariant::Secondary
                })
                .selected(selected)
                .scrollable(scrollable.get())
                .activation_mode(if automatic.get() {
                    TabActivationMode::Automatic
                } else {
                    TabActivationMode::Manual
                })
                .min_tab_height(Dp(if icons.get() && primary { 64.0 } else { 48.0 }))
                .modifier(Modifier::new().fill_max_width())
                .content(move || {
                    let visible = if scrollable.get() { 5 } else { 3 };
                    for (index, name) in DESTINATIONS.into_iter().take(visible).enumerate() {
                        let available = index != 1 || enabled.get();
                        if index == 1 && badge.get() {
                            tab()
                                .enabled(available)
                                .accessibility_label(format!("{name}, 3 new items"))
                                .on_click(move || count.set(count.get() + 1))
                                .title_shared(RenderSlot::new(move || {
                                    badge_title(name, icons.get() && primary, index);
                                }));
                        } else if icons.get() && primary {
                            tab()
                                .label(name)
                                .icon(destination_asset(index))
                                .enabled(available)
                                .accessibility_label(name.to_string())
                                .on_click(move || count.set(count.get() + 1));
                        } else {
                            tab()
                                .label(name)
                                .enabled(available)
                                .accessibility_label(name.to_string())
                                .on_click(move || count.set(count.get() + 1));
                        }
                    }
                });
            gap(Dp(24.0));
            text()
                .content(format!(
                    "{prefix} content: {}",
                    DESTINATIONS[selected.get()]
                ))
                .style(type_style(22.0, 30.0));
            gap(Dp(8.0));
            text()
                .content(
                    [
                        "Search flights and manage upcoming journeys.",
                        "Review hotel bookings and saved stays.",
                        "Find train departures and booked seats.",
                        "Check bus routes and ticket details.",
                        "View ferry crossings and boarding details.",
                    ][selected.get()],
                )
                .style(type_style(16.0, 24.0));
            text()
                .content(format!("{prefix} activations: {}", count.get()))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn badge_title(#[prop(into)] label: String, icon_shown: bool, index: usize) {
    if icon_shown {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                badged_box()
                    .content(move || {
                        icon().painter(destination_asset(index)).size(Dp(24.0));
                    })
                    .badge(|| {
                        badge_with_content().content(|| {
                            text().content("3");
                        });
                    });
                text().content(label.clone());
            });
    } else {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                text().content(label.clone());
                spacer().modifier(Modifier::new().width(Dp(4.0)));
                badge_with_content().content(|| {
                    text().content("3");
                });
            });
    }
}

#[tessera]
fn section(primary: bool) {
    panel(RenderSlot::new(move || {
        sample(if primary { "Primary" } else { "Secondary" }, primary).controls(true);
    }));
    gap(Dp(24.0));
    text().content(if primary{"1  Container    2  Optional badge    3  Optional icon    4  Label    5  Divider    6  Active indicator"}else{"1  Container    2  Optional badge    3  Label    4  Divider    5  Active indicator"}).style(type_style(16.0,24.0));
    gap(Dp(32.0));
    subheading(if primary {
        "Primary tabs color".into()
    } else {
        "Secondary tabs color".into()
    });
    gap(Dp(24.0));
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
                                sample(
                                    format!(
                                        "{} {}",
                                        if dark { "Dark" } else { "Light" },
                                        if primary { "primary" } else { "secondary" }
                                    ),
                                    primary,
                                );
                            });
                    });
            });
        gap(Dp(24.0));
    }
    subheading(if primary {
        "Primary tabs states".into()
    } else {
        "Secondary tabs states".into()
    });
    gap(Dp(24.0));
    text().content("Tab enters the active destination. Arrow keys move focus; Enter or Space activates it. Automatic activation selects on focus movement. Disabled destinations are skipped.").style(type_style(16.0,24.0));
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Container".into(), "Surface".into()],
            vec!["Primary active content".into(), "Primary".into()],
            vec!["Secondary active content".into(), "On surface".into()],
            vec!["Inactive content".into(), "On surface variant".into()],
            vec!["Active indicator".into(), "Primary".into()],
            vec!["Divider".into(), "Outline variant".into()],
        ],
    );
}

#[tessera]
fn measurements() {
    native_tabs_figure(true, true, false, 2, 4);
    gap(Dp(32.0));
    native_tabs_figure(true, false, false, 2, 2);
    gap(Dp(32.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec![
                "Label-only / icon-and-label height".into(),
                "48 / 64dp".into(),
            ],
            vec!["Icon".into(), "24dp".into()],
            vec!["Divider".into(), "1dp · inside the container".into()],
            vec![
                "Primary / secondary indicator height".into(),
                "3 / 2dp".into(),
            ],
            vec!["Primary indicator minimum width".into(), "24dp".into()],
            vec!["Primary indicator inset".into(), "2dp per side".into()],
            vec!["Inline label-to-badge spacing".into(), "4dp".into()],
        ],
    );
}

fn destination_asset(index: usize) -> tessera_components::material_icons::Asset {
    [
        filled::FLIGHT_SVG,
        filled::HOTEL_SVG,
        filled::TRAIN_SVG,
        filled::DIRECTIONS_BUS_SVG,
        filled::DIRECTIONS_BOAT_SVG,
    ][index]
}

const LABEL_KEYS: [&str; 5] = ["label0", "label1", "label2", "label3", "label4"];
const ICON_KEYS: [&str; 5] = ["icon0", "icon1", "icon2", "icon3", "icon4"];

fn figure_name(primary: bool, index: usize) -> &'static str {
    if primary {
        ["Flights", "Trips", "Explore", "Buy", "Ferries"][index]
    } else {
        ["Overview", "Specifications", "Details", "Updates", "More"][index]
    }
}

fn figure_asset(primary: bool, index: usize) -> tessera_components::material_icons::Asset {
    if primary {
        [
            filled::FLIGHT_SVG,
            filled::LUGGAGE_SVG,
            filled::EXPLORE_SVG,
            filled::SHOPPING_CART_SVG,
            filled::DIRECTIONS_BOAT_SVG,
        ][index]
    } else {
        filled::DESCRIPTION_SVG
    }
}

#[tessera]
fn native_tabs_figure(
    primary: bool,
    icons: bool,
    scrollable: bool,
    annotation: usize,
    count: usize,
) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0usize);
    let unread = remember(|| true);
    let art_width = remember(|| Dp(900.0));
    let scale = art_width.get().0 / 1024.0;
    let count = if !primary && !scrollable { 2 } else { count };
    if !unread.get() && geometry.with(|values| values.contains_key("badge")) {
        geometry.with_mut(|values| {
            values.remove("badge");
        });
    }
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(300.0 * scale))
                        .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                            let next = Dp::from_pixels_f32(bounds.width.to_f32());
                            if art_width.get() != next {
                                art_width.set(next);
                            }
                        })
                        .background_with_shape(
                            scheme.surface_container,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_size()
                                .padding_symmetric(Dp::ZERO, Dp::ZERO),
                        )
                        .children(move || {
                            boxed()
                                .modifier(watch(geometry, "container").width(Dp(360.0 * scale)))
                                .children(move || {
                                    tabs()
                                        .variant(if primary {
                                            TabsVariant::Primary
                                        } else {
                                            TabsVariant::Secondary
                                        })
                                        .selected(selected)
                                        .scrollable(scrollable)
                                        .edge_padding(Dp(52.0 * scale))
                                        .tab_padding(Dp(16.0 * scale))
                                        .indicator_height(Dp(
                                            if primary { 3.0 } else { 2.0 } * scale
                                        ))
                                        .indicator_min_width(Dp(24.0 * scale))
                                        .min_tab_height(Dp(if icons { 64.0 } else { 48.0 } * scale))
                                        .modifier(Modifier::new().fill_max_width())
                                        .content(move || {
                                            for (index, name) in (0..count)
                                                .map(|index| (index, figure_name(primary, index)))
                                            {
                                                tab()
                                                    .accessibility_label(
                                                        if index == 0 && unread.get() {
                                                            format!("{name}, 3 new items")
                                                        } else {
                                                            name.into()
                                                        },
                                                    )
                                                    .on_click(move || {
                                                        if index == 0 {
                                                            unread.set(false);
                                                        }
                                                    })
                                                    .title_shared(RenderSlot::new(move || {
                                                        native_tab_title(
                                                            index,
                                                            icons,
                                                            index == 0 && unread.get(),
                                                            geometry,
                                                            scale,
                                                            primary,
                                                            false,
                                                        );
                                                    }));
                                            }
                                        });
                                });
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            if annotation > 0 {
                                tab_annotations(
                                    primary,
                                    icons,
                                    annotation == 2,
                                    selected.get(),
                                    geometry,
                                );
                            }
                        });
                });
            gap(Dp(16.0));
            text()
                .content(format!(
                    "Figure content: {}",
                    figure_name(primary, selected.get())
                ))
                .style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn native_tab_title(
    index: usize,
    icons: bool,
    unread: bool,
    geometry: State<Geometry>,
    scale: f64,
    primary: bool,
    inline: bool,
) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let mut style = theme.typography.title_small;
    style.font_size = style.font_size * scale;
    style.line_height = style.line_height.map(|height| height * scale);
    let color = use_context::<tessera_components::theme::ContentColor>()
        .map(|color| color.get().current)
        .unwrap_or(theme.color_scheme.on_surface);
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            if icons && !inline {
                boxed()
                    .modifier(watch(geometry, ICON_KEYS[index]))
                    .children(move || {
                        badged_box()
                            .content(move || {
                                icon()
                                    .painter(figure_asset(primary, index))
                                    .size(Dp(24.0 * scale))
                                    .tint(color);
                            })
                            .badge(move || {
                                if unread {
                                    figure_badge(geometry, scale);
                                }
                            });
                    });
                gap(Dp(2.0 * scale));
                text()
                    .content(figure_name(primary, index))
                    .style(style)
                    .color(color)
                    .modifier(watch(geometry, LABEL_KEYS[index]));
            } else {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        if icons && inline {
                            boxed()
                                .modifier(watch(geometry, ICON_KEYS[index]))
                                .children(move || {
                                    icon()
                                        .painter(figure_asset(primary, index))
                                        .size(Dp(24.0 * scale))
                                        .tint(color);
                                });
                            spacer().modifier(Modifier::new().width(Dp(8.0 * scale)));
                        }
                        text()
                            .content(figure_name(primary, index))
                            .style(style)
                            .color(color)
                            .modifier(watch(geometry, LABEL_KEYS[index]));
                        if unread {
                            spacer().modifier(Modifier::new().width(Dp(4.0 * scale)));
                            figure_badge(geometry, scale);
                        }
                    });
            }
        });
}

#[tessera]
fn figure_badge(geometry: State<Geometry>, scale: f64) {
    let mut style = use_context::<MaterialTheme>()
        .unwrap()
        .get()
        .typography
        .label_small;
    style.font_size = style.font_size * scale;
    style.line_height = style.line_height.map(|height| height * scale);
    boxed()
        .modifier(watch(geometry, "badge").size(Dp(16.0 * scale), Dp(16.0 * scale)))
        .children(move || {
            badge_with_content().content(move || {
                text().content("3").style(style);
            });
        });
}

#[tessera]
fn tab_annotations(
    primary: bool,
    icons: bool,
    measurements: bool,
    selected: usize,
    geometry: State<Geometry>,
) {
    let values = geometry.get();
    let (Some(root), Some(container), Some(label)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, LABEL_KEYS[selected]),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let scale = width.0 / 1024.0;
    let mut svg = String::new();
    if measurements {
        let tint = "#ec407a";
        annotations::outline(&mut svg, container);
        dotted(
            &mut svg,
            container.x + container.w + 12.0,
            container.y,
            container.h,
            true,
            tint,
        );
        annotations::dimension(
            if icons { "64" } else { "48" }.into(),
            Dp(container.x + container.w + 20.0),
            Dp(container.y + container.h * 0.5 - 10.0),
        );
        if let Some(icon) = relative(&values, ICON_KEYS[selected]) {
            annotations::outline(&mut svg, icon);
            annotations::dimension("24".into(), Dp(icon.x + icon.w + 16.0), Dp(icon.y));
        }
        let indicator_height = if primary { 3.0 } else { 2.0 };
        let indicator = Rect {
            x: label.x + 2.0,
            y: container.y + container.h - indicator_height,
            w: (label.w - 4.0).max(24.0),
            h: indicator_height,
        };
        svg.push_str(&format!("<rect x='{}' y='{}' width='2' height='{}' fill='{tint}' fill-opacity='.3'/><rect x='{}' y='{}' width='2' height='{}' fill='{tint}' fill-opacity='.3'/>",label.x,container.y,label.h,label.x+label.w-2.0,container.y,label.h));
        annotations::outline(&mut svg, indicator);
        annotations::dimension(
            if primary { "3" } else { "2" }.into(),
            Dp(indicator.x + indicator.w + 16.0),
            Dp(indicator.y),
        );
        annotations::dimension(
            "2".into(),
            Dp(label.x - 8.0),
            Dp(container.y + container.h + 24.0),
        );
        annotations::dimension(
            "2".into(),
            Dp(label.x + label.w),
            Dp(container.y + container.h + 24.0),
        );
    } else {
        let tint = "#49454f";
        let center_y = container.y + container.h * 0.5;
        figure_marker(
            1,
            Dp(container.x - 75.0 * scale),
            Dp(center_y - 16.0 * scale),
            scale,
        );
        dotted(
            &mut svg,
            container.x - 51.0 * scale,
            center_y,
            51.0 * scale,
            false,
            tint,
        );
        if let Some(badge) = relative(&values, "badge") {
            let x = badge.x + badge.w * 0.5;
            dotted(
                &mut svg,
                x,
                badge.y - 38.0 * scale,
                38.0 * scale,
                true,
                tint,
            );
            figure_marker(2, Dp(x - 12.0 * scale), Dp(badge.y - 70.0 * scale), scale);
        }
        if primary && icons {
            if let Some(icon) = relative(&values, ICON_KEYS[1]) {
                let x = icon.x + icon.w * 0.5;
                dotted(&mut svg, x, icon.y - 38.0 * scale, 38.0 * scale, true, tint);
                figure_marker(3, Dp(x - 12.0 * scale), Dp(icon.y - 70.0 * scale), scale);
            }
        }
        if let Some(label) = relative(&values, LABEL_KEYS[if primary { 2 } else { 1 }]) {
            let y = label.y + label.h * 0.5;
            dotted(&mut svg, label.x + label.w, y, 58.0 * scale, false, tint);
            figure_marker(
                if primary { 4 } else { 3 },
                Dp(label.x + label.w + 58.0 * scale),
                Dp(y - 16.0 * scale),
                scale,
            );
        }
        let divider_x = container.x + container.w * 0.75;
        let bottom = container.y + container.h;
        dotted(&mut svg, divider_x, bottom, 52.0 * scale, true, tint);
        figure_marker(
            if primary { 5 } else { 4 },
            Dp(divider_x - 12.0 * scale),
            Dp(bottom + 52.0 * scale),
            scale,
        );
        let indicator_x = label.x + label.w * 0.5;
        dotted(&mut svg, indicator_x, bottom, 52.0 * scale, true, tint);
        figure_marker(
            if primary { 6 } else { 5 },
            Dp(indicator_x - 12.0 * scale),
            Dp(bottom + 52.0 * scale),
            scale,
        );
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn nested_tabs() {
    let primary = remember(|| 0usize);
    let secondary = remember(|| 0usize);
    panel(RenderSlot::new(move || {
        tabs()
            .selected(primary)
            .modifier(Modifier::new().fill_max_width())
            .content(|| {
                tab().label("Travel");
                tab().label("Saved");
            });
        gap(Dp(24.0));
        tabs()
            .variant(TabsVariant::Secondary)
            .selected(secondary)
            .modifier(Modifier::new().fill_max_width())
            .content(|| {
                tab().label("Upcoming");
                tab().label("Past");
            });
        gap(Dp(32.0));
        text()
            .content(format!(
                "{} · {}",
                if primary.get() == 0 {
                    "Travel"
                } else {
                    "Saved"
                },
                if secondary.get() == 0 {
                    "Upcoming"
                } else {
                    "Past"
                }
            ))
            .style(type_style(22.0, 28.0));
    }));
}

#[tessera]
fn tab_placement() {
    let selected = remember(|| 0usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(360.0))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(24.0)))
                .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
        )
        .children(move || {
            tessera_components::app_bar::top_app_bar().title("Travel");
            tabs()
                .selected(selected)
                .modifier(Modifier::new().fill_max_width())
                .content(|| {
                    tab().label("Flights");
                    tab().label("Hotels");
                    tab().label("Trains");
                });
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .weight(1.0)
                        .padding_all(Dp(24.0)),
                )
                .children(move || {
                    text()
                        .content(DESTINATIONS[selected.get()])
                        .style(type_style(24.0, 32.0));
                    gap(Dp(16.0));
                    text()
                        .content("Related content appears below the tabs.")
                        .style(type_style(16.0, 24.0));
                });
        });
}
