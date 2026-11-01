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

#[path = "menus_guides.rs"]
mod guides;

#[tessera]
pub(super) fn menus_spec() {
    article(
        "Menus",
        "Menus display a list of choices on a temporary surface",
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
                content: RenderSlot::new(|| {
                    subheading("Vertical menus".into());
                    gap(Dp(24.0));
                    menu_figure(true, false, MenuColorStyle::Standard);
                }),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "States",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(|| {
                    menu_figure(false, true, MenuColorStyle::Standard);
                }),
            },
            SpecSection {
                title: "Menu (baseline)",
                content: RenderSlot::new(guides::baseline_section),
            },
        ],
    )
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(overview_intro)),
            sections: vec![
                SpecSection {
                    title: "Availability & resources",
                    content: RenderSlot::new(guides::resources),
                },
                SpecSection {
                    title: "M3 Expressive update",
                    content: RenderSlot::new(expressive),
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
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(guideline_anatomy),
                },
                SpecSection {
                    title: "Flexibility & slots",
                    content: RenderSlot::new(|| {
                        slots(true);
                    }),
                },
                SpecSection {
                    title: "Placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "Adaptive design",
                    content: RenderSlot::new(adaptive),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(behavior),
                },
                SpecSection {
                    title: "Focus",
                    content: RenderSlot::new(guideline_focus),
                },
                SpecSection {
                    title: "Density",
                    content: RenderSlot::new(density),
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
                    content: RenderSlot::new(interaction),
                },
                SpecSection {
                    title: "Flexibility & slots",
                    content: RenderSlot::new(|| {
                        slots(false);
                    }),
                },
                SpecSection {
                    title: "Focus",
                    content: RenderSlot::new(accessible_focus),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(keyboard),
                },
                SpecSection {
                    title: "Interactability",
                    content: RenderSlot::new(interactability),
                },
                SpecSection {
                    title: "Labeling elements",
                    content: RenderSlot::new(labeling),
                },
            ],
        },
    ]);
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

#[tessera]
fn overview_intro() {
    prose(
        "Use a menu to show a temporary set of actions. To show actions on screen at all times, use a toolbar instead.",
    );
    gap(Dp(16.0));
    guides::bullets(strings(&[
        "Menus can open from many components, including icon buttons, split buttons, and text fields",
        "Context menus provide actions for a specific element, like an image or highlighted text, and usually open with a secondary click",
    ]));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                configuration(true, MenuColorStyle::Vibrant);
                guides::submenu_pair(
                    "Line spacing",
                    strings(&["Single", "1.15", "Double", "Custom 1.2"]),
                    3,
                );
            });
    }));
    gap(Dp(12.0));
    guides::caption(
        "Vertical menus can include vibrant colors, gaps, dividers, and submenus to organize a list of choices",
    );
    gap(Dp(32.0));
    launch();
    gap(Dp(8.0));
    guides::caption("Open the live menu, then hover or focus Share to reveal its submenu");
}

#[tessera]
fn expressive() {
    guides::caption("November 2025");
    gap(Dp(8.0));
    prose(
        "Vertical menus were introduced with new shapes, color styles, selection states, and refined submenu motion. Gaps can be used for a more flexible layout on Android.",
    );
    gap(Dp(16.0));
    prose("Variants:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Added vertical menus, recommended for new designs",
        "Baseline menu is still available",
    ]));
    prose("Color styles:");
    gap(Dp(8.0));
    guides::bullets(strings(&["Standard", "Vibrant"]));
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        menu_sample(true, MenuColorStyle::Vibrant);
    }));
    gap(Dp(12.0));
    guides::caption("Vibrant colors help selected menu items stand out");
}

#[tessera]
fn differences() {
    guides::bullets(strings(&[
        "Color: New color mappings and compatibility with dynamic color",
        "Variants: Dropdown menu and exposed dropdown menu are now both referred to as menu, since they differ only in the element which opens the menu surface",
    ]));
    gap(Dp(24.0));
    guides::m2_comparison();
}

#[tessera]
fn usage() {
    prose(
        "Use a menu to show a temporary set of actions. To show actions on screen at all times, use a toolbar instead.",
    );
    gap(Dp(8.0));
    prose("A menu takes up less space than a set of radio buttons or chips.");
    gap(Dp(24.0));
    subheading("Color options".into()).level(4);
    gap(Dp(8.0));
    prose("Menus have two color mappings:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Standard: Surface-based, lower visual emphasis",
        "Vibrant: Tertiary-based, higher visual emphasis",
    ]));
    prose("Vibrant menus are more prominent, and should be used sparingly.");
    gap(Dp(24.0));
    guides::line_spacing_figure();
    gap(Dp(12.0));
    guides::caption("On web, menus can open submenus");
    gap(Dp(32.0));
    subheading("Opening menus".into());
    gap(Dp(8.0));
    prose("Menus temporarily appear in front of all other permanent UI elements.");
    gap(Dp(8.0));
    prose("A menu should open when a person:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Selects an element, such as an icon, button, or text field",
        "Performs a specific action to trigger the menu, like right-click or press-and-hold",
    ]));
    prose("Use menus in situations that need extra actions, like:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Overflow menus",
        "Text field dropdown menus",
        "Select menus",
        "Context menus",
    ]));
    gap(Dp(16.0));
    guides::opening_figure();
    gap(Dp(12.0));
    guides::caption("Menus appear in front of all other UI elements");
    gap(Dp(32.0));
    subheading("Menu groups".into());
    gap(Dp(8.0));
    prose(
        "Vertical menu items can be grouped by adding a divider or small gap. Use groups to bundle similar actions together.",
    );
    gap(Dp(24.0));
    configurations_figure();
    gap(Dp(16.0));
    prose("Menu items can be grouped to be more scannable:");
    gap(Dp(8.0));
    guides::legend(strings(&[
        "Standard vertical menu",
        "Grouped vertical menu",
    ]));
    gap(Dp(32.0));
    subheading("Context menus".into());
    gap(Dp(8.0));
    prose(
        "Context menus provide a list of additional actions a person can take on an item. A secondary click, like a right-click on a mouse or a two-finger tap on a trackpad, opens a context menu.",
    );
    gap(Dp(24.0));
    guides::context_figure();
    gap(Dp(12.0));
    guides::caption(
        "A context menu appears when right clicking with a mouse or trackpad. It can reveal key actions related to the associated content.",
    );
}

#[tessera]
fn guideline_anatomy() {
    menu_figure(true, false, MenuColorStyle::Standard);
    gap(Dp(32.0));
    subheading("Menu items".into());
    gap(Dp(8.0));
    prose(
        "Menu items can include label text, leading icons, trailing icons, and keyboard commands.",
    );
    gap(Dp(8.0));
    prose(
        "When a menu item can only be used under specific conditions, it should appear disabled rather than be removed.",
    );
    gap(Dp(24.0));
    guides::disabled_figure();
    gap(Dp(12.0));
    guides::caption("The Redo action is disabled when that action isn't available");
    gap(Dp(32.0));
    subheading("Gaps & dividers (optional)".into());
    gap(Dp(8.0));
    prose("Gaps and dividers can be used to separate and group menu items.");
    gap(Dp(24.0));
    subheading("Gaps".into()).level(4);
    gap(Dp(8.0));
    prose(
        "Use a gap to visually divide menu items into distinct groups. Gaps are more expressive than dividers and make the relationship between items clear.",
    );
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Avoid changing the size of the gap",
        "Limit the number of gaps in a menu to one or two",
        "Don't use gaps in scrollable menus",
    ]));
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        menu_sample(true, MenuColorStyle::Standard);
    }));
    gap(Dp(12.0));
    guides::caption("Gaps separate menu items using expressive shapes");
    gap(Dp(8.0));
    guides::caption("Note: Gaps are not currently available in the Material web library");
    gap(Dp(24.0));
    subheading("Dividers".into()).level(4);
    gap(Dp(8.0));
    prose("Dividers create a more subtle separation between items. Use a divider for:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Scrollable menus",
        "Text fields with a dropdown menu, where a grouped treatment isn't appropriate",
    ]));
    prose("On web, use a divider to separate menu items.");
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        menu_sample(false, MenuColorStyle::Standard);
    }));
    gap(Dp(12.0));
    guides::caption("Dividers separate menu items in baseline menus and on web");
}

#[tessera]
fn slots(guidelines: bool) {
    if guidelines {
        prose("Menus have custom slots that support more flexible item layouts.");
        gap(Dp(8.0));
        prose(
            "When creating a complicated menu, think of the menu item as a container with a swappable slot.",
        );
        gap(Dp(8.0));
        prose("Slots work best with simple content such as:");
        gap(Dp(8.0));
        guides::bullets(strings(&[
            "Images",
            "Progress indicators",
            "Color swatches",
        ]));
        gap(Dp(16.0));
        guides::slots_figure();
        gap(Dp(12.0));
        guides::caption("Slots can appear anywhere in a menu");
        gap(Dp(8.0));
        guides::caption(
            "Note: This page draws the slot structure statically. The native menu item exposes fixed icon, label, supporting text and trailing text slots, not arbitrary slot content.",
        );
        gap(Dp(24.0));
        subheading("Slot accessibility".into()).level(4);
        gap(Dp(8.0));
    }
    prose("Use caution when adding slots to menus:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Make sure the menu remains accessible",
        "Elements must follow the rules and interaction patterns of the menu component",
        "Keep the same menu item padding",
        "Targets should be 48x48dp or larger",
    ]));
    prose(
        "Don't add buttons, switches, or other direct actions into the menu item. Nested elements should only perform one action. Adding multiple actions can break keyboard navigation and screen reader functionality.",
    );
    gap(Dp(12.0));
    guides::caption(
        "Caution: Reserve the use of slots for use cases that maintain the menu's accessibility and functionality",
    );
}

#[tessera]
fn placement() {
    prose(
        "A menu is positioned relative to the window edge. It typically appears below, next to, or in front of the element that generates it.",
    );
    gap(Dp(8.0));
    prose(
        "If a menu is in a position to be cut off, it should automatically reposition to appear to the left, right, or above the element that generates it.",
    );
    gap(Dp(24.0));
    guides::placement_figure();
    gap(Dp(12.0));
    guides::caption("Menus can appear around or in front of the element that opened them");
    gap(Dp(24.0));
    launch();
    gap(Dp(8.0));
    guides::caption("Open the live menu to see it positioned next to its trigger");
    gap(Dp(32.0));
    subheading("Submenus".into());
    gap(Dp(8.0));
    prose("Submenus should open next to the parent menu item without overlapping it.");
    gap(Dp(8.0));
    prose(
        "Submenus are best used on large screens where there's space. See adaptive guidance for alternatives on mobile.",
    );
    gap(Dp(24.0));
    guides::line_spacing_figure();
    gap(Dp(12.0));
    guides::caption("Position submenus to the side of the parent item");
    gap(Dp(8.0));
    guides::caption("Note: Submenus are not currently available on Jetpack Compose");
}

#[tessera]
fn adaptive() {
    subheading("Compact breakpoints".into());
    gap(Dp(8.0));
    prose(
        "Consider adapting menus into bottom sheets on small screens. They have more space to display additional items and longer labels.",
    );
    gap(Dp(24.0));
    guides::adaptive_figure(true);
    gap(Dp(12.0));
    guides::caption("A bottom sheet can replace a menu on smaller screens");
    gap(Dp(32.0));
    subheading("Other breakpoints".into());
    gap(Dp(8.0));
    prose(
        "On medium and expanded windows, menus are most effective as they appear in context with the content. On larger screens, menus can also display more items, and can use submenus to organize complex sets of options.",
    );
    gap(Dp(24.0));
    guides::adaptive_figure(false);
    gap(Dp(12.0));
    guides::caption("On large screens, a menu is often more appropriate than a bottom sheet");
}

#[tessera]
fn behavior() {
    subheading("Appearing".into());
    gap(Dp(8.0));
    prose(
        "A menu can appear when a person interacts with an element on the page, like a button, text field, filter chip, or highlighted text.",
    );
    gap(Dp(8.0));
    prose(
        "A menu's position on screen affects where and how it appears. If opened at the top of the screen, it expands downwards to avoid being cropped.",
    );
    gap(Dp(24.0));
    launch();
    gap(Dp(8.0));
    guides::caption(
        "Menus at different positions on a screen open in different directions, adapting to the available space",
    );
    gap(Dp(32.0));
    subheading("Motion".into());
    gap(Dp(8.0));
    prose(
        "Menus use an enter and exit transition. This animation creates a relationship between the menu and the element that generates it.",
    );
    gap(Dp(8.0));
    prose(
        "When a menu expands, the trigger element becomes pressed. When an item is selected, a ripple appears on touch.",
    );
    gap(Dp(8.0));
    prose("In dense products, such as on desktop, menus can open instantly to reduce motion.");
    gap(Dp(32.0));
    subheading("Filtering".into());
    gap(Dp(8.0));
    prose(
        "A menu can include a text field to filter options. This pattern is also known as autocomplete.",
    );
    gap(Dp(8.0));
    prose(
        "As someone types, the list of menu options filters to show relevant results. This helps people quickly find the right option from a long list.",
    );
    gap(Dp(32.0));
    subheading("Scrolling".into());
    gap(Dp(8.0));
    prose(
        "Menus can scroll when all menu items can't display at once. In this state, menus show a persistent scrollbar.",
    );
    gap(Dp(8.0));
    prose("Don't use gaps if a menu scrolls; this is currently unsupported.");
    gap(Dp(32.0));
    subheading("Selecting".into());
    gap(Dp(8.0));
    prose(
        "When a menu is opened, the corresponding button or icon button should remain the same visually, with the addition of a pressed state. This should happen even when opening from a keyboard shortcut.",
    );
    gap(Dp(8.0));
    prose("The overflow icon remains the same, even after the menu is opened.");
    gap(Dp(24.0));
    subheading("Single- and multi-select menus".into()).level(4);
    gap(Dp(8.0));
    prose("Menus can allow either single-select or multi-select actions:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Single-select menus can have one item selected at a time. When a new item is selected, the previously selected item is automatically unselected.",
        "Multi-select menus can have many selected items. They stay open until the person dismisses the menu.",
    ]));
    gap(Dp(16.0));
    selection_figures::selection_figure();
    gap(Dp(12.0));
    guides::caption("Menus can be single- or multi-select");
}

#[tessera]
fn guideline_focus() {
    prose(
        "When a menu has multiple submenus, focus follows the current hovered or focused submenu.",
    );
    gap(Dp(24.0));
    subheading("Shape morphing".into()).level(4);
    gap(Dp(8.0));
    prose(
        "As a person moves from one submenu to the next, the corners of the focused submenu become more rounded, while the unfocused submenu becomes less rounded. This adds a dynamic quality to menu interactions.",
    );
    gap(Dp(24.0));
    launch();
    gap(Dp(8.0));
    guides::caption(
        "Open the menu, then hover or focus Share to watch the corner shape follow the focused submenu",
    );
}

#[tessera]
fn density() {
    prose(
        "On web only, density levels control the spacing between elements. Increasing density decreases the top and bottom padding.",
    );
    gap(Dp(24.0));
    guides::density_figure();
    gap(Dp(12.0));
    guides::caption("Density of menus from 0 to -3");
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following using assistive technology:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Navigate to, open, and close a menu",
        "Navigate between and select menu items",
    ]));
    gap(Dp(16.0));
    launch();
}

#[tessera]
fn interaction() {
    prose("Menu items need certain cues to clearly show when they're selected:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "By default, menu items change shape and color when selected",
        "The default color contrast is 3:1 between selected and unselected menu items",
        "It's recommended to include another visual cue, like a checkmark",
    ]));
    gap(Dp(16.0));
    accessibility_figures::state_selection();
    gap(Dp(12.0));
    guides::caption(
        "Use multiple visual cues like color, shape, and icons to show that an item is selected",
    );
}

#[tessera]
fn accessible_focus() {
    subheading("Initial focus".into()).level(4);
    gap(Dp(8.0));
    prose(
        "When a menu opens, focus should be placed on the first menu item. This allows people using a keyboard or other assistive technologies to begin navigating the menu immediately.",
    );
    gap(Dp(24.0));
    subheading("Exiting a menu".into()).level(4);
    gap(Dp(8.0));
    prose("People expect to exit a menu by:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Selecting an option",
        "Tapping Escape or outside of the menu",
        "Using the system back button",
    ]));
    prose(
        "Where focus is placed after closing the menu depends on the app. This example returns focus to its trigger.",
    );
    gap(Dp(24.0));
    prose("Keyboard navigation on Android and web:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Tab to select a menu item",
        "Space or Enter to open a menu",
        "Space or Enter to select a menu item",
        "Escape to close a menu",
    ]));
    gap(Dp(16.0));
    launch();
}

#[tessera]
fn interactability() {
    prose("Disabled menu items can receive focus but aren't selectable.");
    gap(Dp(8.0));
    prose("Dividers and gaps can't receive focus.");
    gap(Dp(24.0));
    guides::focus_examples();
}

#[tessera]
fn labeling() {
    prose("Accessibility labels are used with assistive technology devices like screen readers.");
    gap(Dp(8.0));
    prose("The accessibility label should be the same as the menu item text.");
    gap(Dp(8.0));
    prose("The role is dependent on platform.");
    gap(Dp(24.0));
    accessibility_figures::labeling();
    gap(Dp(12.0));
    guides::caption("The menu item's accessibility label aligns with the UI text");
    gap(Dp(24.0));
    spec_table(
        strings(&[
            "Element",
            "A11y label",
            "Role (Web)",
            "Role (Android Views)",
            "Role (Jetpack Compose)",
        ]),
        vec![strings(&[
            "Menu item text",
            "Preview",
            "Menu item",
            "Generic actionable element",
            "Generic actionable element",
        ])],
    );
    gap(Dp(24.0));
    prose(
        "For menu items with text and an icon, the icon's accessibility label should be marked as decorative to avoid redundant verbalizations.",
    );
    gap(Dp(8.0));
    guides::caption(
        "For menu items with text and an icon, the icon's accessibility label is decorative",
    );
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
    gap(Dp(16.0));
    prose(
        "Use vertical menus for a more expressive look and feel, including rounded corners, standard and vibrant color styles, more selection states, and submenu motion.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                menu_tile(true, MenuColorStyle::Standard, 1);
                menu_tile(false, MenuColorStyle::Standard, 2);
            });
    }));
    gap(Dp(16.0));
    guides::legend(strings(&[
        "Vertical menu with gap",
        "Vertical menu with divider",
    ]));
    gap(Dp(32.0));
    subheading("Baseline variant".into());
    gap(Dp(16.0));
    prose(
        "In M3 Expressive, baseline menu is still available to use, but doesn't have the latest shapes, color styles, selection states, and motion.",
    );
    gap(Dp(24.0));
    guides::baseline_variant();
}

#[tessera]
fn configurations() {
    subheading("Vertical menus layout".into());
    gap(Dp(24.0));
    configurations_figure();
    gap(Dp(16.0));
    guides::legend(strings(&["Standard", "Grouped"]));
    gap(Dp(24.0));
    spec_table(
        strings(&["Category", "Configuration", "M3", "M3 Expressive"]),
        vec![
            strings(&["Color", "Standard", "Available", "Available"]),
            strings(&["Color", "Vibrant", "--", "Available"]),
            strings(&["Layout", "Standard", "Available", "Available"]),
            strings(&["Layout", "Grouped", "--", "Available"]),
        ],
    );
}

#[tessera]
fn configurations_figure() {
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                for (index, grouped) in [false, true].into_iter().enumerate() {
                    column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            configuration(grouped, MenuColorStyle::Standard);
                            gap(Dp(24.0));
                            guides::badge(index + 1);
                        });
                }
            });
    }));
}

#[tessera]
fn configuration(grouped: bool, style: MenuColorStyle) {
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
                    .color_style(style)
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
                menu_group()
                    .position(MenuPosition::Last)
                    .color_style(style)
                    .children(move || {
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
                menu_group().color_style(style).children(move || {
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
        });
}

#[tessera]
fn menu_tile(grouped: bool, style: MenuColorStyle, number: usize) {
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            menu_sample(grouped, style);
            gap(Dp(24.0));
            guides::badge(number);
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
        guides::legend(strings(&[
            "Menu item",
            "Leading icon (optional)",
            "Menu item text",
            "Trailing icon (optional)",
            "Badge (optional)",
            "Trailing text (optional)",
            "Container",
            "Supporting text (optional)",
            "Label text (optional)",
            "Gap (optional)",
            "Divider (optional)",
        ]));
    }
    if measurements {
        gap(Dp(12.0));
        guides::caption("Vertical menu padding and size measurements");
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
            ("12", c.x - 52.0, second.y + 16.0),
            ("20", c.x + c.w + 48.0, third.y + 16.0),
            ("8", c.x + c.w + 48.0, label.y - 16.0),
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
    prose(
        "Color values are implemented through design tokens. For designers, this means working with color values that correspond with tokens. In implementation, a color value will be a token that references a value.",
    );
    gap(Dp(16.0));
    prose("Menus have two color mappings:");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "Standard: Surface-based",
        "Vibrant: Tertiary-based",
    ]));
    prose(
        "These mappings provide options for lower or higher visual emphasis. Vibrant menus are more prominent so should be used sparingly.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                menu_tile(true, MenuColorStyle::Standard, 1);
                menu_tile(true, MenuColorStyle::Vibrant, 2);
            });
    }));
    gap(Dp(16.0));
    guides::legend(strings(&["Standard color scheme", "Vibrant color scheme"]));
    gap(Dp(32.0));
    for style in [MenuColorStyle::Standard, MenuColorStyle::Vibrant] {
        subheading(
            if style == MenuColorStyle::Standard {
                "Standard colors"
            } else {
                "Vibrant colors"
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
        gap(Dp(16.0));
        guides::caption("Vertical menus color roles used for light and dark themes:");
        gap(Dp(8.0));
        guides::legend(
            menu_color_roles(style)
                .iter()
                .map(|role| (*role).to_owned())
                .collect(),
        );
        gap(Dp(32.0));
    }
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
    prose(
        "States are visual representations used to communicate the status of a component or an interactive element.",
    );
    gap(Dp(8.0));
    prose(
        "Shape morphing in vertical menus creates an expressive active state. As focus moves between submenus, the corner shape changes to highlight the active menu.",
    );
    gap(Dp(24.0));
    guides::states_figure(false, false);
}

#[tessera]
fn tokens() {
    prose("Browse the component elements, attributes, tokens, and their values.");
    gap(Dp(16.0));
    spec_table(
        strings(&["Token", "Value"]),
        vec![
            strings(&["Menu gap", "2dp"]),
            strings(&["Menu group padding", "2dp"]),
            strings(&["Menu container elevation", "Level 2, 3dp"]),
            strings(&["Menu item height", "44dp"]),
            strings(&["Menu item top space", "8dp"]),
            strings(&["Menu item bottom space", "8dp"]),
            strings(&["Menu item leading space", "16dp"]),
            strings(&["Menu item trailing space", "16dp"]),
            strings(&["Menu item between space", "12dp"]),
            strings(&["Menu item leading icon size", "20dp"]),
            strings(&["Menu item trailing icon size", "20dp"]),
        ],
    );
}

#[tessera]
fn keyboard() {
    spec_table(
        strings(&["Keys", "Actions"]),
        vec![
            strings(&["Tab", "Focus lands on menu"]),
            strings(&[
                "Space or Enter",
                "For closed menus: Opens menu or submenu. For open menus: Selects a menu item",
            ]),
            strings(&[
                "Up and Down arrows",
                "For closed menus: Opens menu. For open menus: Moves focus to the next item",
            ]),
            strings(&["Left and Right arrows", "Opens or closes a submenu"]),
            strings(&[
                "Letters",
                "Focus moves to the next menu item starting with letter",
            ]),
            strings(&["Escape", "Closes menu"]),
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
