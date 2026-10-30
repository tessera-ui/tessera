//! Navigation drawer specifications and destination figures.
//!
//! ## Usage
//!
//! Inspect drawer destinations, geometry and the Expressive recommendation.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    button::button,
    column::column,
    divider::horizontal_divider,
    icon::icon,
    material_icons::{filled, outlined},
    modifier::{ModifierExt as _, SemanticsArgs},
    navigation_drawer::{NavigationDrawerStyle, navigation_drawer, navigation_drawer_item},
    scrollable::ScrollableController,
    shape_def::Shape,
    side_sheet::{SideSheetPosition, modal_side_sheet_provider},
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, modifier::FocusModifierExt as _, remember,
    tessera, use_context,
};

use super::{
    INK,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

const DESTINATIONS: [&str; 5] = ["Inbox", "Outbox", "Favorites", "Trash", "Personal"];

#[tessera]
pub(super) fn drawers_spec() {
    let open = remember(|| false);
    let selected = remember(|| 0usize);
    let scroll = remember(ScrollableController::new);
    modal_side_sheet_provider()
        .max_width(Dp(360.0))
        .content_padding(Dp::ZERO)
        .position(SideSheetPosition::Start)
        .is_open(open.get())
        .accessibility_label("Mail navigation")
        .on_close_request(move || open.set(false))
        .main_content(move || {
            article(
                "Navigation drawer",
                "Navigation drawers let people switch between UI views on larger devices",
                vec![
                    SpecSection {
                        title: "Tokens & specs",
                        content: RenderSlot::new(tokens),
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
                            subheading("Standard navigation drawer".into());
                            figure(false, true);
                            gap(Dp(32.0));
                            subheading("Modal navigation drawer".into());
                            figure(true, true);
                            measurements();
                        }),
                    },
                ],
            )
            .intro(|| {
                recommendation();
                figure(true, false);
                legend();
            })
            .pages(vec![
                ArticlePage {
                    title: "Overview",
                    intro: Some(RenderSlot::new(move || {
                        recommendation();
                        prose("Drawers organize primary destinations in a persistent or modal surface. Labels and the active indicator keep the current destination visible while the rail of secondary content stays reachable.");
                        gap(Dp(24.0));
                        launch(open, selected);
                        gap(Dp(32.0));
                        figure(false, false);
                    })),
                    sections: vec![
                        SpecSection {
                            title: "Availability & resources",
                            content: RenderSlot::new(availability),
                        },
                        SpecSection {
                            title: "M3 Expressive update",
                            content: RenderSlot::new(expressive_update),
                        },
                        SpecSection {
                            title: "Differences from M2",
                            content: RenderSlot::new(differences),
                        },
                    ],
                },
                ArticlePage {
                    title: "Guidelines",
                    intro: Some(RenderSlot::new(recommendation)),
                    sections: vec![
                        SpecSection {
                            title: "Usage",
                            content: RenderSlot::new(move || {
                                prose("Use navigation destinations to move between top-level sections. Keep their order stable and group related destinations together.");
                                gap(Dp(24.0));
                                subheading("Standard navigation drawer".into());
                                prose("A standard drawer remains alongside the content when the window has enough room.");
                                gap(Dp(24.0));
                                figure(false, false);
                                gap(Dp(32.0));
                                subheading("Modal navigation drawer".into());
                                prose("A modal drawer overlays the page with a scrim and dismisses after selecting a destination or closing the overlay.");
                                gap(Dp(24.0));
                                launch(open, selected);
                            }),
                        },
                        SpecSection {
                            title: "Anatomy",
                            content: RenderSlot::new(guide_anatomy),
                        },
                        SpecSection {
                            title: "Responsive layout",
                            content: RenderSlot::new(responsive_layout),
                        },
                        SpecSection {
                            title: "Behavior",
                            content: RenderSlot::new(move || {
                                prose("Allow destinations to scroll when they exceed the available height, keeping each label and indicator together.");
                                gap(Dp(24.0));
                                prose("Keep a standard drawer visible while its destinations remain relevant. A modal drawer temporarily covers the main page.");
                                gap(Dp(24.0));
                                prose("Open modal navigation from a clearly named menu control and return focus to it when dismissed.");
                                gap(Dp(24.0));
                                launch(open, selected);
                            }),
                        },
                    ],
                },
                ArticlePage {
                    title: "Accessibility",
                    intro: Some(RenderSlot::new(recommendation)),
                    sections: vec![
                        SpecSection {
                            title: "Use cases",
                            content: RenderSlot::new(move || {
                                prose("People must be able to find destinations, move between them and identify the active destination using assistive technology.");
                                gap(Dp(24.0));
                                launch(open, selected);
                            }),
                        },
                        SpecSection {
                            title: "Interaction & style",
                            content: RenderSlot::new(|| {
                                prose("Expose selection as well as focus. The active indicator, foreground color and filled icon communicate the selected destination; hover and press feedback communicate interaction.");
                                gap(Dp(24.0));
                                figure(false, false);
                            }),
                        },
                        SpecSection {
                            title: "Initial focus",
                            content: RenderSlot::new(move || {
                                prose("When opening a modal drawer, place focus on the first destination. Keep the drawer reachable in the page's logical traversal order.");
                                gap(Dp(24.0));
                                launch(open, selected);
                            }),
                        },
                        SpecSection {
                            title: "Closing",
                            content: RenderSlot::new(move || {
                                prose("Dismiss a modal drawer by selecting its scrim, pressing Escape or choosing a destination. Restore focus to the opening menu control.");
                                gap(Dp(24.0));
                                launch(open, selected);
                            }),
                        },
                        SpecSection {
                            title: "Visual indicators",
                            content: RenderSlot::new(|| {
                                prose("Pair the active indicator with a filled icon. Unselected destinations use outlined icons so selection remains apparent beyond a change of color.");
                                gap(Dp(24.0));
                                figure(false, false);
                            }),
                        },
                        SpecSection {
                            title: "Keyboard navigation",
                            content: RenderSlot::new(keyboard),
                        },
                        SpecSection {
                            title: "Labeling elements",
                            content: RenderSlot::new(labeling),
                        },
                    ],
                },
            ]);
        })
        .side_sheet_content(move || {
            live_drawer(selected, open, scroll);
        });
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn availability() {
    spec_table(
        vec!["Type".into(), "Resource".into(), "Status".into()],
        vec![
            vec![
                "Design".into(),
                "Design Kit (Figma)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Flutter".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "This page documents the native navigation_drawer and navigation_drawer_item components. Published values are recorded next to the native ones wherever the two differ.",
    );
}

#[tessera]
fn expressive_update() {
    prose(
        "May 2025 — The navigation drawer is no longer recommended. Use the expanded navigation rail instead: it shows text labels and an extended FAB, can be standard or modal, and adapts better across breakpoints.",
    );
    gap(Dp(24.0));
    prose(
        "Existing interfaces that still ship a drawer keep the standard and modal variants documented here; new interfaces should open the navigation rail page from the recommendation above.",
    );
    gap(Dp(32.0));
    figure(true, false);
}

#[tessera]
fn differences() {
    prose("Color: New color mappings and compatibility with dynamic color.");
    gap(Dp(16.0));
    prose(
        "Variants: Distinguishes two separate variants of navigation drawer — standard and modal.",
    );
    gap(Dp(16.0));
    prose("Shape: Rounded corners at the ending edge of the drawer.");
    gap(Dp(16.0));
    prose("States: Updated color and shape for indicating the selected state.");
    gap(Dp(32.0));
    figure(true, false);
    gap(Dp(16.0));
    prose(
        "M2 used square corners and a rectangular selected row; M3 adds rounded corners, new color mappings and a pill-shaped indicator.",
    );
}

#[tessera]
fn guide_anatomy() {
    figure(true, false);
    legend();
    gap(Dp(32.0));
    for (title, body) in [
        (
            "Sheet",
            "Keep the drawer anchored to the start edge with a consistent width.",
        ),
        (
            "Divider (optional)",
            "Separate destination groups without introducing extra row spacing.",
        ),
        (
            "Active indicator",
            "A rounded container and filled icon distinguish the selected destination.",
        ),
        (
            "Label text and icons",
            "Use short destination labels with familiar leading icons.",
        ),
        (
            "Section label (optional)",
            "Name a group when its relationship is not already clear.",
        ),
        (
            "Scrim (modal only)",
            "Dim the main page while modal navigation is open.",
        ),
    ] {
        subheading(title.into());
        prose(body);
        gap(Dp(24.0));
    }
}

#[tessera]
fn responsive_layout() {
    subheading("Compact breakpoint".into());
    prose(
        "Use a modal navigation drawer, opened from a menu control, when the navigation cannot remain beside the content.",
    );
    gap(Dp(24.0));
    subheading("Medium breakpoint".into());
    prose(
        "Consider a modal drawer or, preferably, an expanded navigation rail when the window can support persistent navigation.",
    );
    gap(Dp(24.0));
    subheading("Expanded, large and extra-large breakpoints".into());
    prose(
        "Use a standard navigation drawer beside the content. Expressive interfaces should replace it with an expanded navigation rail and leave the additional space for primary content.",
    );
    gap(Dp(32.0));
    figure(false, true);
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Focus the first destination".into()],
            vec!["Arrow keys".into(), "Move between destinations".into()],
            vec![
                "Space or Enter".into(),
                "Select the focused destination".into(),
            ],
            vec!["Escape".into(), "Dismiss modal navigation".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Focus moves through destinations before leaving the drawer, so a modal drawer never traps the keyboard with no way back to the page.",
    );
}

#[tessera]
fn labeling() {
    prose(
        "Use the destination name as its accessible label. Expand ambiguous names when needed and preserve the visible wording in the accessible name.",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "Visible label".into(),
            "Accessible name".into(),
            "Role".into(),
        ],
        vec![
            vec!["Inbox".into(), "Inbox".into(), "Tab".into()],
            vec!["Recents".into(), "Recent images".into(), "Tab".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "A badge count belongs to the destination's accessible description, not to its name, so the label stays stable when the count changes.",
    );
}

#[tessera]
fn recommendation() {
    let navigation = use_context::<crate::app::MaterialNavigation>().map(|context| context.get().0);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().modifier(Modifier::new().fill_max_width().background_with_shape(scheme.secondary_container,Shape::rounded_rectangle(Dp(16.0))).padding_all(Dp(24.0)).semantics(SemanticsArgs{role:Some(Role::Note),label:Some("M3 Expressive navigation recommendation".into()),merge_descendants:true,..Default::default()})).children(move || {
        prose("Navigation drawers are no longer recommended for M3 Expressive. Use an expanded navigation rail for new interfaces; it provides similar destinations and adapts across window sizes.");
        if let Some(navigation)=navigation {
            gap(Dp(16.0));
            button().text().accessibility_label("Open recommended navigation rail").on_click(move || { if let Some(index)=super::catalog::component_index(32) {navigation.set(Some(index));} }).child(||{text().content("Explore navigation rail");});
        }
    });
    gap(Dp(32.0));
}

#[tessera]
fn launch(open: State<bool>, selected: State<usize>) {
    column().children(move || {
        button()
            .outlined()
            .accessibility_label("Open modal navigation drawer")
            .on_click(move || open.set(true))
            .child(|| {
                text().content("Open navigation drawer");
            });
        gap(Dp(16.0));
        prose(format!(
            "Current destination: {}",
            DESTINATIONS[selected.get()]
        ));
    });
}

#[tessera]
fn drawer_content(modal: bool, selected: State<usize>, geometry: Option<State<Geometry>>) {
    let container_watch = geometry
        .map(|geometry| watch(geometry, "container"))
        .unwrap_or_default();
    navigation_drawer()
        .style(if modal {
            NavigationDrawerStyle::Modal
        } else {
            NavigationDrawerStyle::Standard
        })
        .modifier(Modifier::new().height(Dp(440.0)).then(container_watch))
        .accessibility_label("Mail destinations")
        .content(move || {
            drawer_destinations(selected).geometry_optional(geometry);
        });
}

#[tessera]
fn live_drawer(selected: State<usize>, open: State<bool>, scroll: State<ScrollableController>) {
    column()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .focus_group()
                .semantics(SemanticsArgs {
                    role: Some(Role::Navigation),
                    label: Some("Mail destinations".into()),
                    test_tag: Some("live-drawer".into()),
                    merge_descendants: true,
                    ..Default::default()
                })
                .vertical_scroll(scroll)
                .padding_all(Dp(12.0)),
        )
        .children(move || {
            drawer_destinations(selected).open(open);
        });
}

#[tessera]
fn drawer_destinations(
    selected: State<usize>,
    open: Option<State<bool>>,
    geometry: Option<State<Geometry>>,
) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    boxed()
        .alignment(Alignment::CenterStart)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(56.0))
                .padding_symmetric(Dp(16.0), Dp::ZERO),
        )
        .children(move || {
            let modifier = geometry
                .map(|geometry| watch(geometry, "headline"))
                .unwrap_or_default();
            text()
                .content("Mail")
                .style(theme.typography.title_small)
                .modifier(modifier);
        });
    for (index, label) in DESTINATIONS.into_iter().take(4).enumerate() {
        let active = selected.get() == index;
        let indicator_watch = if index == 0 {
            geometry
                .map(|geometry| watch(geometry, "indicator"))
                .unwrap_or_default()
        } else {
            Modifier::new()
        };
        let badge_style = theme.typography.label_large;
        let badge_slot = if index == 0 {
            Some(RenderSlot::new(move || {
                text().content("24").style(badge_style).modifier(
                    geometry
                        .map(|geometry| watch(geometry, "badge"))
                        .unwrap_or_default(),
                );
            }))
        } else {
            None
        };
        navigation_drawer_item()
            .label(label)
            .selected(active)
            .on_click(move || {
                selected.set(index);
                if let Some(open) = open {
                    open.set(false);
                }
            })
            .modifier(indicator_watch)
            .badge_optional(badge_slot)
            .icon(move || {
                let active = selected.get() == index;
                let asset = match (index, active) {
                    (0, true) => filled::INBOX_SVG,
                    (0, false) => outlined::INBOX_SVG,
                    (1, true) => filled::OUTBOX_SVG,
                    (1, false) => outlined::OUTBOX_SVG,
                    (2, true) => filled::STAR_SVG,
                    (2, false) => outlined::STAR_BORDER_SVG,
                    (_, true) => filled::DELETE_SVG,
                    (_, false) => outlined::DELETE_SVG,
                };
                let modifier = if index == 0 {
                    geometry
                        .map(|geometry| watch(geometry, "icon"))
                        .unwrap_or_default()
                } else {
                    Modifier::new()
                };
                boxed().modifier(modifier).children(move || {
                    icon().vector(asset).size(Dp(24.0));
                });
            });
    }
    gap(Dp(8.0));
    horizontal_divider();
    gap(Dp(8.0));
    text()
        .content("Personal folders")
        .style(theme.typography.title_small)
        .modifier(Modifier::new().padding_all(Dp(16.0)));
    navigation_drawer_item()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            test_tag: open.map(|_| "drawer-personal".into()),
            merge_descendants: true,
            ..Default::default()
        }))
        .label("Personal")
        .selected(selected.get() == 4)
        .icon(move || {
            icon()
                .vector(if selected.get() == 4 {
                    filled::LABEL_SVG
                } else {
                    outlined::LABEL_SVG
                })
                .size(Dp(24.0));
        })
        .on_click(move || {
            selected.set(4);
            if let Some(open) = open {
                open.set(false);
            }
        });
}

#[tessera]
fn figure(modal: bool, measurements: bool) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let width = remember(|| Dp(800.0));
    let inset = if width.get().0 < 600.0 {
        Dp(24.0)
    } else {
        Dp(64.0)
    };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .modifier(
                    watch(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(568.0))
                        .background_with_shape(
                            scheme.surface_container,
                            Shape::rounded_rectangle(Dp(24.0)),
                        )
                        .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                            let next = Dp::from_pixels_f32(bounds.width.to_f32());
                            if width.get() != next {
                                width.set(next);
                            }
                        }),
                )
                .children(move || {
                    boxed()
                        .alignment(Alignment::CenterStart)
                        .modifier(Modifier::new().fill_max_size().padding_all(inset))
                        .children(move || {
                            boxed()
                                .modifier(
                                    watch(geometry, "screen")
                                        .fill_max_size()
                                        .background_with_shape(
                                            scheme.surface,
                                            Shape::rounded_rectangle(Dp(16.0)),
                                        ),
                                )
                                .children(|| {});
                            if modal {
                                boxed()
                                    .modifier(
                                        watch(geometry, "scrim")
                                            .fill_max_size()
                                            .background_with_shape(
                                                scheme.scrim.with_alpha(0.32),
                                                Shape::rounded_rectangle(Dp(16.0)),
                                            ),
                                    )
                                    .children(|| {});
                            }
                            drawer_content(modal, selected).geometry(geometry);
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            figure_annotations(modal, measurements, geometry);
                        });
                });
            gap(Dp(12.0));
            prose(format!(
                "Selected destination: {}",
                DESTINATIONS[selected.get()]
            ));
        });
}

#[tessera]
fn figure_annotations(modal: bool, measurements: bool, geometry: State<Geometry>) {
    figure_annotation_content(
        modal,
        measurements,
        geometry.with(annotations::relative_geometry),
    );
}

#[tessera]
fn figure_annotation_content(modal: bool, measurements: bool, values: Geometry) {
    let (Some(root), Some(container), Some(indicator)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "indicator"),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let mut svg = String::new();
    if measurements {
        let color = "#ec407a";
        for x in [container.x, container.x + container.w - 12.0] {
            svg.push_str(&format!(
                "<rect x='{x}' y='{}' width='12' height='{}' fill='{color}' fill-opacity='.18'/>",
                container.y, container.h
            ));
        }
        dotted(
            &mut svg,
            container.x,
            container.y - 16.0,
            container.w,
            false,
            color,
        );
        dotted(
            &mut svg,
            indicator.x + indicator.w + 12.0,
            indicator.y,
            indicator.h,
            true,
            color,
        );
        annotations::dimension(
            "360".into(),
            Dp(container.x + container.w / 2.0 - 12.0),
            Dp(container.y - 40.0),
        );
        annotations::dimension(
            "56".into(),
            Dp(indicator.x + indicator.w + 18.0),
            Dp(indicator.y + 16.0),
        );
        annotations::dimension(
            "12".into(),
            Dp(container.x - 6.0),
            Dp(container.y + container.h + 8.0),
        );
        if let Some(icon) = relative(&values, "icon") {
            annotations::outline(&mut svg, icon);
            annotations::dimension("24".into(), Dp(icon.x), Dp(icon.y - 24.0));
        }
    } else {
        for (name, number) in [
            ("container", 1),
            ("headline", 2),
            ("indicator", 4),
            ("badge", 5),
            ("scrim", 6),
            ("icon", 7),
        ] {
            if name == "scrim" && !modal {
                continue;
            }
            let Some(rect) = relative(&values, name) else {
                continue;
            };
            let (x, y, tx, ty) = match name {
                "container" => (
                    container.x - 32.0,
                    container.y + container.h / 2.0,
                    container.x,
                    container.y + container.h / 2.0,
                ),
                "scrim" => (
                    container.x + container.w + 48.0,
                    container.y + 300.0,
                    container.x + container.w + 24.0,
                    container.y + 316.0,
                ),
                "headline" => (
                    rect.x + rect.w + 40.0,
                    rect.y - 6.0,
                    rect.x + rect.w,
                    rect.y + 10.0,
                ),
                "icon" => (rect.x - 48.0, rect.y - 4.0, rect.x, rect.y + 12.0),
                "indicator" => (
                    container.x + container.w + 48.0,
                    rect.y + rect.h / 2.0 - 48.0,
                    rect.x + rect.w,
                    rect.y + rect.h / 2.0,
                ),
                "badge" => (
                    container.x + container.w + 96.0,
                    rect.y + rect.h / 2.0 - 16.0,
                    rect.x + rect.w,
                    rect.y + rect.h / 2.0,
                ),
                _ => (
                    rect.x + rect.w + 24.0,
                    rect.y + rect.h / 2.0 - 16.0,
                    rect.x + rect.w,
                    rect.y + rect.h / 2.0,
                ),
            };
            if name == "indicator" {
                let elbow = x - 8.0;
                dotted(&mut svg, tx, ty, elbow - tx, false, "#49454f");
                dotted(&mut svg, elbow, ty, y + 16.0 - ty, true, "#49454f");
                dotted(&mut svg, elbow, y + 16.0, 8.0, false, "#49454f");
            } else {
                dotted(&mut svg, tx, ty, x - tx, false, "#49454f");
            }
            annotations::marker(number, Dp(x), Dp(y));
        }
        let label_x = indicator.x + 72.0;
        let label_y = indicator.y - 88.0;
        let marker_x = label_x + 96.0;
        let elbow_y = indicator.y - 4.0;
        dotted(
            &mut svg,
            marker_x,
            label_y + 32.0,
            elbow_y - label_y - 32.0,
            true,
            "#49454f",
        );
        dotted(
            &mut svg,
            marker_x,
            elbow_y,
            label_x - marker_x,
            false,
            "#49454f",
        );
        dotted(
            &mut svg,
            label_x,
            elbow_y,
            indicator.h / 2.0 + 4.0,
            true,
            "#49454f",
        );
        annotations::marker(3, Dp(marker_x - 12.0), Dp(label_y));
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn legend() {
    gap(Dp(16.0));
    for (number, label) in [
        (1, "Container"),
        (2, "Headline"),
        (3, "Label text"),
        (4, "Active indicator"),
        (5, "Badge label text"),
        (6, "Scrim (modal)"),
        (7, "Icon"),
    ] {
        prose(format!("{number}  {label}"));
    }
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token / value".into()],
        vec![
            vec!["Modal container".into(), "surfaceContainerLow".into()],
            vec!["Active indicator".into(), "secondaryContainer".into()],
            vec![
                "Active label and icon".into(),
                "onSecondaryContainer".into(),
            ],
            vec!["Inactive label and icon".into(), "onSurfaceVariant".into()],
            vec!["Destination height".into(), "56dp".into()],
            vec!["Active indicator radius".into(), "28dp".into()],
            vec!["Icon size".into(), "24dp".into()],
        ],
    );
}

#[tessera]
fn measurements() {
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Container width".into(), "360dp".into()],
            vec!["Container height".into(), "100%".into()],
            vec!["Modal corner radii".into(), "0, 16, 16, 0dp".into()],
            vec!["Icon size".into(), "24dp".into()],
            vec!["Active indicator height".into(), "56dp".into()],
            vec!["Active indicator width".into(), "336dp".into()],
            vec!["Active indicator radius".into(), "28dp".into()],
            vec!["Label alignment".into(), "Start".into()],
            vec!["Start/end content padding".into(), "28dp".into()],
            vec!["Active indicator inset".into(), "12dp".into()],
            vec!["Gap between destinations".into(), "0dp".into()],
        ],
    );
}

#[tessera]
fn colors() {
    figure(true, false);
    gap(Dp(24.0));
    tokens();
}

#[tessera]
fn states() {
    prose(
        "Hover, focus and press a destination to inspect its state layer and ripple. Activating a destination changes the selected indicator and switches its icon from outlined to filled.",
    );
    gap(Dp(24.0));
    figure(false, false);
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use std::cell::Cell;

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        CursorEventContent, GestureState, PointerChange, PressKeyEventType, PxPosition,
        testing::{
            layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
            start_layout_test_session,
        },
    };

    use super::*;

    thread_local! {
        static HOST: Cell<(i32, i32)> = const { Cell::new((600, 240)) };
        static SELECTED: Cell<Option<State<usize>>> = const { Cell::new(None) };
        static OPEN: Cell<Option<State<bool>>> = const { Cell::new(None) };
        static SCROLL: Cell<Option<State<ScrollableController>>> = const { Cell::new(None) };
    }

    #[tessera]
    fn fixture() {
        let selected = remember(|| 0usize);
        let open = remember(|| true);
        let scroll = remember(ScrollableController::new);
        SELECTED.set(Some(selected));
        OPEN.set(Some(open));
        SCROLL.set(Some(scroll));
        material_theme().child(move || {
            column()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    let (width, height) = HOST.get();
                    boxed()
                        .modifier(
                            Modifier::new()
                                .size(Dp(f64::from(width)), Dp(f64::from(height)))
                                .semantics(SemanticsArgs {
                                    test_tag: Some("drawer-host".into()),
                                    merge_descendants: true,
                                    ..Default::default()
                                }),
                        )
                        .children(move || {
                            modal_side_sheet_provider()
                                .position(SideSheetPosition::Start)
                                .max_width(Dp(360.0))
                                .content_padding(Dp::ZERO)
                                .is_open(open.get())
                                .on_close_request(move || open.set(false))
                                .main_content(|| {
                                    boxed().modifier(Modifier::new().fill_max_size());
                                })
                                .side_sheet_content(move || {
                                    live_drawer(selected, open, scroll);
                                });
                        });
                });
        });
    }

    #[test]
    fn live_drawer_fills_available_height_and_scrolls_to_an_activatable_last_destination() {
        for (width, height) in [(600, 240), (600, 720), (320, 240), (320, 720)] {
            HOST.set((width, height));
            let mut session = start_layout_test_session(layout_test(fixture).viewport_px(800, 800));
            pump_layout_test_session(&mut session, 0);
            let before = pump_layout_test_session(&mut session, 1_000_000_000);
            before.node("drawer-host").size(width, height);
            before.node("live-drawer").size(width.min(360), height);
            before.node("drawer-personal").width(width.min(360) - 24);
            let initial = before.node("drawer-personal").bounds();
            if height == 240 {
                assert!(initial.y.0 >= height);
                SCROLL.get().unwrap().with_mut(|scroll| {
                    scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -240.0));
                });
            }
            let after = pump_layout_test_session(&mut session, 1_016_666_667);
            let bounds = after.node("drawer-personal").bounds();
            assert!(bounds.y.0 >= 0);
            assert!(bounds.y.0 + bounds.height.0 <= height);
            let cursor = Some(PxPosition::from_f32_xy(
                bounds.x.to_f32() + bounds.width.to_f32() * 0.5,
                bounds.y.to_f32() + bounds.height.to_f32() * 0.5,
            ));
            for (index, content) in [
                CursorEventContent::Pressed(PressKeyEventType::Left),
                CursorEventContent::Released(PressKeyEventType::Left),
            ]
            .into_iter()
            .enumerate()
            {
                pump_layout_test_session_with_input(
                    &mut session,
                    1_000_000_000 + (index as u64 + 2) * 16_666_667,
                    vec![PointerChange::new(
                        tessera_ui::time::Instant::now(),
                        1,
                        content,
                        GestureState::TapCandidate,
                    )],
                    cursor,
                );
            }
            pump_layout_test_session(&mut session, 1_100_000_000);
            assert_eq!(SELECTED.get().unwrap().get(), 4);
            assert!(!OPEN.get().unwrap().get());
        }
    }
}
