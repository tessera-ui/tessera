//! Navigation drawer specifications and destination figures.
//!
//! ## Usage
//!
//! Inspect drawer destinations, geometry and the Expressive recommendation.

use std::collections::BTreeMap;

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    button::button,
    column::column,
    divider::horizontal_divider,
    icon::icon,
    material_icons::{filled, outlined},
    modifier::{InteractionVisualState, ModifierExt as _, SemanticsArgs},
    navigation_drawer::{NavigationDrawerStyle, navigation_drawer, navigation_drawer_item},
    row::row,
    scrollable::ScrollableController,
    shape_def::{RoundedCorner, Shape},
    side_sheet::{SideSheetController, SideSheetPosition, modal_side_sheet_provider},
    spacer::spacer,
    surface::{RipplePresentation, surface},
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_shard::RouterController;
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, modifier::FocusModifierExt as _,
    provide_context, remember, scaled_box, tessera, use_context,
};

use super::{
    INK,
    annotations::{self, Rect, SpaceGeometry as Geometry, space_geometry, watch_in_space as watch},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, srgb, subheading, type_style,
};

const DESTINATIONS: [&str; 5] = ["Inbox", "Outbox", "Favorites", "Trash", "Personal"];

#[path = "drawers_guides.rs"]
mod guides;

#[cfg(all(target_family = "wasm", feature = "md3-diagnostics"))]
struct DiagnosticDrawerObservation {
    geometry: State<Geometry>,
    initial_geometry_empty: bool,
    wrapper_window_bounds: Option<tessera_ui::PxRect>,
}

#[cfg(all(target_family = "wasm", feature = "md3-diagnostics"))]
thread_local! {
    static DIAGNOSTIC_DRAWER_OBSERVATIONS: std::cell::RefCell<Vec<DiagnosticDrawerObservation>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

#[cfg(all(target_family = "wasm", feature = "md3-diagnostics"))]
#[tessera]
pub(crate) fn diagnostic_drawer_figures(count: usize) {
    assert!(
        (1..=2).contains(&count),
        "diagnostic drawer count must be one or two"
    );
    let mut geometries = Vec::with_capacity(count);
    for index in 0..count {
        let geometry = remember(Geometry::new);
        DIAGNOSTIC_DRAWER_OBSERVATIONS.with(|observations| {
            let mut observations = observations.borrow_mut();
            if observations
                .get(index)
                .is_none_or(|entry| entry.geometry != geometry)
            {
                let observation = DiagnosticDrawerObservation {
                    geometry,
                    initial_geometry_empty: geometry.with(|geometry| geometry.is_empty()),
                    wrapper_window_bounds: None,
                };
                if let Some(entry) = observations.get_mut(index) {
                    *entry = observation;
                } else {
                    observations.push(observation);
                }
            }
        });
        geometries.push(geometry);
    }
    DIAGNOSTIC_DRAWER_OBSERVATIONS.with(|observations| observations.borrow_mut().truncate(count));
    column()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(tessera_ui::Color::WHITE),
        )
        .children(move || {
            for (index, geometry) in geometries.iter().copied().enumerate() {
                boxed()
                    .modifier(Modifier::new().fill_max_width().on_globally_positioned(
                        move |bounds: tessera_ui::PxRect| {
                            DIAGNOSTIC_DRAWER_OBSERVATIONS.with(|observations| {
                                if let Some(entry) = observations.borrow_mut().get_mut(index)
                                    && entry.geometry == geometry
                                {
                                    entry.wrapper_window_bounds = Some(bounds);
                                }
                            });
                        },
                    ))
                    .children(move || {
                        drawer_figure(true, 0).geometry(geometry);
                    });
                if index + 1 < count {
                    gap(Dp(24.0));
                }
            }
        });
}

#[cfg(all(target_family = "wasm", feature = "md3-diagnostics"))]
pub(crate) fn diagnostic_drawer_observations() -> String {
    DIAGNOSTIC_DRAWER_OBSERVATIONS.with(|observations| {
        let observations = observations.borrow();
        let mut json = String::from(
            r#"{"window_units":"px","canonical_units":"dp","rect_encoding":"x,y,width,height","figures":["#,
        );
        for (index, observation) in observations.iter().enumerate() {
            if index != 0 {
                json.push(',');
            }
            json.push_str(&format!(
                "{{\"index\":{index},\"initial_geometry_empty\":{},\"wrapper_window_bounds\":",
                observation.initial_geometry_empty
            ));
            if let Some(bounds) = &observation.wrapper_window_bounds {
                json.push_str(&format!(
                    "[{},{},{},{}]",
                    bounds.x.to_f32(), bounds.y.to_f32(), bounds.width.to_f32(), bounds.height.to_f32()
                ));
            } else {
                json.push_str("null");
            }
            let alive = observation.geometry.is_alive();
            json.push_str(&format!(",\"geometry_alive\":{alive},\"canonical_geometry\":"));
            if alive {
                let geometry = observation.geometry.with(space_geometry);
                json.push('{');
                for (index, (name, rect)) in geometry.iter().enumerate() {
                    if index != 0 {
                        json.push(',');
                    }
                    json.push_str(&format!("\"{name}\":[{},{},{},{}]", rect.x, rect.y, rect.w, rect.h));
                }
                json.push('}');
            } else {
                json.push_str("null");
            }
            json.push('}');
        }
        json.push_str("]}");
        json
    })
}

#[derive(Clone, Copy)]
struct LiveDrawerSession {
    open: State<bool>,
    selected: State<usize>,
    scroll: State<ScrollableController>,
    directory: State<SideSheetController>,
    router: State<RouterController>,
    material: State<Option<usize>>,
}

impl LiveDrawerSession {
    fn route_active(self) -> bool {
        self.router.with(|router| {
            router
                .current_is::<crate::pages::material_components::MaterialComponentsPageDestination>(
                )
        }) && super::drawer_article_selected(self.material.get())
    }

    fn can_open(self) -> bool {
        self.route_active()
            && self
                .directory
                .with(|directory| !directory.is_open() && !directory.is_animating())
    }
}

#[tessera]
pub(crate) fn live_drawer_host(
    directory: State<SideSheetController>,
    router: State<RouterController>,
    material: State<Option<usize>>,
    main_content: RenderSlot,
) {
    let open = remember(|| false);
    let selected = remember(|| 0usize);
    let scroll = remember(ScrollableController::new);
    let session = LiveDrawerSession {
        open,
        selected,
        scroll,
        directory,
        router,
        material,
    };
    if !session.route_active() && open.get() {
        open.set(false);
    }
    let modal = RenderSlot::new(move || {
        modal_side_sheet_provider()
            .max_width(Dp(360.0))
            .content_padding(Dp::ZERO)
            .position(SideSheetPosition::Start)
            .is_open(open.get())
            .accessibility_label("Mail navigation")
            .on_close_request(move || open.set(false))
            .main_content_shared(main_content.clone())
            .side_sheet_content(move || {
                live_drawer(selected, open, scroll);
            });
    });
    provide_context(move || session, move || modal.render());
}

#[tessera]
pub(super) fn drawers_spec() {
    let root = use_context::<LiveDrawerSession>().map(|context| context.get());
    let local_open = remember(|| false);
    let local_selected = remember(|| 0usize);
    let local_scroll = remember(ScrollableController::new);
    let (open, selected, scroll) = if let Some(session) = root {
        (session.open, session.selected, session.scroll)
    } else {
        (local_open, local_selected, local_scroll)
    };
    let content = RenderSlot::new(move || {
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
                        gap(Dp(24.0));
                        measurement_pair(false);
                        gap(Dp(32.0));
                        measurements(false);
                        gap(Dp(48.0));
                        subheading("Modal navigation drawer".into());
                        gap(Dp(24.0));
                        measurement_pair(true);
                        gap(Dp(32.0));
                        measurements(true);
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
                    bullets(vec![
                        "Use standard navigation drawers in expanded, large, and extra-large breakpoints".into(),
                        "Use modal navigation drawers in compact and medium breakpoints".into(),
                        "Can be open or closed by default".into(),
                        "Two variants: standard and modal".into(),
                        "Put the most frequent destinations at the top and group related destinations together".into(),
                    ]);
                    gap(Dp(32.0));
                    overview_figure();
                    gap(Dp(32.0));
                    launch(open, selected);
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
                intro: Some(RenderSlot::new(|| {
                    recommendation();
                    guides::guide(0);
                })),
                sections: vec![
                    SpecSection {
                        title: "Usage",
                        content: RenderSlot::new(move || {
                            guides::usage(open, selected);
                        }),
                    },
                    SpecSection {
                        title: "Anatomy",
                        content: RenderSlot::new(|| {
                            guides::anatomy();
                        }),
                    },
                    SpecSection {
                        title: "Responsive layout",
                        content: RenderSlot::new(|| {
                            guides::responsive_layout();
                        }),
                    },
                    SpecSection {
                        title: "Behavior",
                        content: RenderSlot::new(move || {
                            guides::behavior(open, selected);
                        }),
                    },
                ],
            },
            ArticlePage {
                title: "Accessibility",
                intro: Some(RenderSlot::new(|| {
                    recommendation().window_size_classes(true);
                })),
                sections: vec![
                    SpecSection {
                        title: "Use cases",
                        content: RenderSlot::new(move || {
                            guides::use_cases(open, selected);
                        }),
                    },
                    SpecSection {
                        title: "Interaction & style",
                        content: RenderSlot::new(|| {
                            guides::interaction();
                        }),
                    },
                    SpecSection {
                        title: "Initial focus",
                        content: RenderSlot::new(move || {
                            guides::initial_focus(open, selected);
                        }),
                    },
                    SpecSection {
                        title: "Closing",
                        content: RenderSlot::new(move || {
                            guides::closing(open, selected);
                        }),
                    },
                    SpecSection {
                        title: "Visual indicators",
                        content: RenderSlot::new(|| {
                            guides::visual_indicators();
                        }),
                    },
                    SpecSection {
                        title: "Keyboard navigation",
                        content: RenderSlot::new(|| {
                            guides::keyboard();
                        }),
                    },
                    SpecSection {
                        title: "Labeling elements",
                        content: RenderSlot::new(|| {
                            guides::labeling();
                        }),
                    },
                ],
            },
        ]);
    });
    if root.is_some() {
        content.render();
    } else {
        modal_side_sheet_provider()
            .max_width(Dp(360.0))
            .content_padding(Dp::ZERO)
            .position(SideSheetPosition::Start)
            .is_open(open.get())
            .accessibility_label("Mail navigation")
            .on_close_request(move || open.set(false))
            .main_content_shared(content)
            .side_sheet_content(move || {
                live_drawer(selected, open, scroll);
            });
    }
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
}

#[tessera]
fn expressive_update() {
    text()
        .content("May 2025")
        .style(guides::medium_style(16.0, 24.0))
        .color(INK);
    gap(Dp(16.0));
    prose(
        "The navigation drawer is no longer recommended. Use the expanded navigation rail instead. More on M3 Expressive",
    );
}

#[tessera]
fn differences() {
    bullets(vec![
        "Color: New color mappings and compatibility with dynamic color".into(),
        "Variants: Distinguishes two separate variants of navigation drawer: Standard and modal"
            .into(),
        "Shape: Rounded corners at the ending edge of the drawer".into(),
        "States: Updated color and shape for indicating selected state".into(),
    ]);
    gap(Dp(32.0));
    differences_figure();
}

#[tessera]
fn bullets(items: Vec<String>) {
    for item in items {
        row().children(move || {
            text()
                .content("•")
                .style(type_style(16.0, 24.0))
                .modifier(Modifier::new().width(Dp(24.0)));
            text()
                .content(item.clone())
                .style(type_style(16.0, 24.0))
                .modifier(Modifier::new().weight(1.0));
        });
        gap(Dp(12.0));
    }
}

#[tessera]
fn mail_messages(selected: State<usize>, width: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().modifier(Modifier::new().width(Dp(width)).padding_all(Dp(16.0))).children(move || {
        surface().shape(Shape::CAPSULE).style(scheme.surface_container_high.into())
            .modifier(Modifier::new().fill_max_width().height(Dp(48.0)))
            .child(|| { text().content("Search replies").style(type_style(14.0,20.0)); });
        gap(Dp(16.0));
        text().content(DESTINATIONS[selected.get().min(4)]).style(type_style(22.0,28.0));
        for (sender, subject, body) in [
            ("Ellie Pietersen", "Adoption volunteering", "I just learned about this volunteering opportunity. The organization is looking for help with the upcoming event."),
            ("In Yeong-hui", "Museum field trip", "About 26 students have signed up. We are looking for parent volunteers to share the day."),
            ("B. Bhatia", "Weekend plans", "Here are the details for our next community meeting."),
        ] {
            gap(Dp(12.0));
            surface().shape(Shape::rounded_rectangle(Dp(16.0))).style(scheme.surface_container.into())
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    column().modifier(Modifier::new().padding_all(Dp(16.0))).children(move || {
                        text().content(sender).style(type_style(12.0,16.0));
                        gap(Dp(8.0));
                        text().content(subject).style(type_style(18.0,24.0));
                        gap(Dp(8.0));
                        text().content(body).style(type_style(14.0,20.0));
                    });
                });
        }
    });
}

#[tessera]
fn small_rail() {
    tessera_components::navigation_rail::navigation_rail().content(|| {
        for (index, label) in ["Inbox", "Outbox", "Favorites"].into_iter().enumerate() {
            tessera_components::navigation_rail::navigation_rail_item()
                .label(label)
                .selected(index == 0)
                .icon(move || {
                    icon()
                        .vector(match index {
                            0 => filled::INBOX_SVG,
                            1 => outlined::SEND_SVG,
                            _ => outlined::FAVORITE_BORDER_SVG,
                        })
                        .size(Dp(24.0));
                });
        }
    });
}

#[tessera]
fn responsive_figure(kind: u8) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0usize);
    drawer_stage(
        1024.0,
        440.0,
        format!("Drawer responsive layout example {kind}"),
        geometry,
    )
    .child(move || {
        if kind == 0 {
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(48.0), Dp(64.0))
                        .required_size(Dp(384.0), Dp(440.0)),
                )
                .children(move || {
                    row().children(move || {
                        small_rail();
                        mail_messages(selected, 288.0);
                    });
                });
            drawer_scene(false, selected, 512.0, 64.0, 464.0, 440.0);
            text()
                .content("Navigation rail to navigation drawer")
                .style(type_style(16.0, 24.0))
                .modifier(Modifier::new().offset(Dp(48.0), Dp(16.0)));
        } else if kind == 1 {
            drawer_scene(true, selected, 304.0, 48.0, 416.0, 440.0);
        } else if kind == 2 {
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(48.0), Dp(48.0))
                        .required_size(Dp(928.0), Dp(440.0)),
                )
                .children(move || {
                    row().children(move || {
                        small_rail();
                        mail_messages(selected, 832.0);
                    });
                    drawer_scene(true, selected, 96.0, 0.0, 832.0, 440.0);
                });
        } else {
            material_theme()
                .theme(|| MaterialTheme::from_seed(srgb(103, 80, 164), true))
                .child(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(48.0), Dp(48.0))
                                .required_size(Dp(928.0), Dp(440.0)),
                        )
                        .children(move || {
                            row().children(move || {
                                drawer_content(false, selected).height(Dp(440.0));
                                mail_messages(selected, 568.0);
                            });
                        });
                });
        }
    });
}

#[tessera]
fn visibility_panel(number: usize, initial: bool) {
    let visible = remember(move || initial);
    let selected = remember(|| 0usize);
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            button()
                .text()
                .accessibility_label(format!("Toggle standard drawer example {number}"))
                .on_click(move || visible.set(!visible.get()))
                .child(move || {
                    text().content(if visible.get() {
                        "Hide navigation"
                    } else {
                        "Show navigation"
                    });
                });
            drawer_stage(
                1024.0,
                712.0,
                format!("Standard drawer visibility example {number}"),
                geometry,
            )
            .child(move || {
                text()
                    .content(if visible.get() {
                        "Drawer visible: content shares the window"
                    } else {
                        "Drawer hidden: content uses the full window"
                    })
                    .style(type_style(16.0, 24.0))
                    .modifier(Modifier::new().offset(Dp(48.0), Dp(20.0)));
                row()
                    .modifier(
                        Modifier::new()
                            .offset(Dp(48.0), Dp(64.0))
                            .required_size(Dp(928.0), Dp(600.0))
                            .background(scheme.surface)
                            .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                            .semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                merge_descendants: true,
                                label: Some(format!("Standard drawer viewport {number}")),
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        if visible.get() {
                            boxed()
                                .modifier(
                                    Modifier::new()
                                        .required_size(Dp(360.0), Dp(600.0))
                                        .semantics(SemanticsArgs {
                                            role: Some(Role::Group),
                                            merge_descendants: true,
                                            label: Some(format!(
                                                "Standard drawer surface {number}"
                                            )),
                                            ..Default::default()
                                        }),
                                )
                                .children(move || {
                                    drawer_content(false, selected)
                                        .height(Dp(600.0))
                                        .interactive(true);
                                });
                        }
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .required_size(
                                        Dp(if visible.get() { 568.0 } else { 928.0 }),
                                        Dp(600.0),
                                    )
                                    .semantics(SemanticsArgs {
                                        role: Some(Role::Group),
                                        merge_descendants: true,
                                        label: Some(format!("Standard drawer content {number}")),
                                        ..Default::default()
                                    }),
                            )
                            .children(move || {
                                mail_messages(selected, if visible.get() { 568.0 } else { 928.0 });
                            });
                    });
            });
        });
}

#[tessera]
fn visibility_examples() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            visibility_panel(1, true);
            gap(Dp(24.0));
            visibility_panel(2, false);
        });
}

#[tessera]
fn recommendation(window_size_classes: Option<bool>) {
    let navigation = use_context::<crate::app::MaterialNavigation>().map(|context| context.get().0);
    let ending = if window_size_classes.unwrap_or(false) {
        "window size classes"
    } else {
        "breakpoints"
    };
    let ink = srgb(33, 0, 93);
    row()
        .cross_axis_alignment(tessera_components::alignment::CrossAxisAlignment::Start)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(srgb(160, 137, 255), Shape::rounded_rectangle(Dp(28.0)))
                .padding_symmetric(Dp(24.0), Dp(20.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some("M3 Expressive navigation recommendation".into()),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            icon().vector(outlined::STAR_BORDER_SVG).size(Dp(24.0)).tint(ink);
            spacer().modifier(Modifier::new().width(Dp(16.0)));
            column()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    text()
                        .content(format!("Note: The navigation drawer is no longer recommended in the Material 3 Expressive update. For those who have updated, use an expanded navigation rail, which has mostly the same functionality of the navigation drawer and adapts better across {ending}."))
                        .style(type_style(14.0, 24.0))
                        .color(ink);
                    if let Some(navigation) = navigation {
                        gap(Dp(8.0));
                        button()
                            .text()
                            .accessibility_label("Open recommended navigation rail")
                            .on_click(move || {
                                if let Some(index) = super::catalog::component_index(32) {
                                    navigation.set(Some(index));
                                }
                            })
                            .child(move || {
                                text().content("Expanded navigation rail").color(ink);
                            });
                    }
                });
        });
    gap(Dp(48.0));
}

#[tessera]
fn launch(open: State<bool>, selected: State<usize>) {
    let session = use_context::<LiveDrawerSession>().map(|context| context.get());
    column().children(move || {
        button()
            .outlined()
            .accessibility_label("Open modal navigation drawer")
            .on_click(move || {
                if session.is_none_or(LiveDrawerSession::can_open) {
                    open.set(true);
                }
            })
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
fn drawer_content(
    modal: bool,
    selected: State<usize>,
    geometry: Option<State<Geometry>>,
    height: Option<Dp>,
    interactive: Option<bool>,
    folders: Option<bool>,
) {
    let container_watch = geometry
        .map(|geometry| watch(geometry, "container"))
        .unwrap_or_default();
    navigation_drawer()
        .style(if modal {
            NavigationDrawerStyle::Modal
        } else {
            NavigationDrawerStyle::Standard
        })
        .modifier(
            Modifier::new()
                .height(height.unwrap_or(Dp(440.0)))
                .then(container_watch),
        )
        .accessibility_label("Mail destinations")
        .content(move || {
            drawer_destinations(selected)
                .geometry_optional(geometry)
                .interactive_optional(interactive)
                .folders_optional(folders);
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
    interactive: Option<bool>,
    folders: Option<bool>,
) {
    let interactive = interactive.unwrap_or(false);
    let folders = folders.unwrap_or(false);
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
        } else if index == 1 {
            observe(geometry, "outbox")
        } else if index == 3 {
            observe(geometry, "trash")
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
                if let Some(open) = open {
                    selected.set(index);
                    open.set(false);
                } else if interactive {
                    selected.set(index);
                }
            })
            .modifier(indicator_watch)
            .badge_optional(badge_slot)
            .icon(move || {
                let active = selected.get() == index;
                let asset = match (index, active) {
                    (0, true) => filled::INBOX_SVG,
                    (0, false) => outlined::INBOX_SVG,
                    (1, true) => filled::SEND_SVG,
                    (1, false) => outlined::SEND_SVG,
                    (2, true) => filled::FAVORITE_SVG,
                    (2, false) => outlined::FAVORITE_BORDER_SVG,
                    (_, true) => filled::DELETE_SVG,
                    (_, false) => outlined::DELETE_SVG,
                };
                let modifier = if index == 0 {
                    geometry
                        .map(|geometry| watch(geometry, "icon"))
                        .unwrap_or_default()
                } else {
                    observe(
                        geometry,
                        if index == 1 {
                            "inactive_icon"
                        } else if index == 2 {
                            "favorite_icon"
                        } else {
                            "trash_icon"
                        },
                    )
                };
                boxed().modifier(modifier).children(move || {
                    icon().vector(asset).size(Dp(24.0));
                });
            });
    }
    gap(Dp(16.0));
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(16.0), Dp::ZERO),
        )
        .children(move || {
            horizontal_divider().modifier(observe(geometry, "divider"));
        });
    gap(Dp(16.0));
    boxed()
        .alignment(Alignment::CenterStart)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(56.0))
                .padding_symmetric(Dp(16.0), Dp::ZERO),
        )
        .children(move || {
            text()
                .content(if open.is_some() {
                    "Personal folders"
                } else if folders {
                    "Personal Folders"
                } else {
                    "Labels"
                })
                .style(theme.typography.title_small)
                .modifier(observe(geometry, "subhead"));
        });
    navigation_drawer_item()
        .modifier(observe(geometry, "label_row").semantics(SemanticsArgs {
            test_tag: open.map(|_| "drawer-personal".into()),
            merge_descendants: true,
            ..Default::default()
        }))
        .label(if open.is_some() {
            "Personal"
        } else if folders {
            "Friends"
        } else {
            "Label"
        })
        .selected(selected.get() == 4)
        .icon(move || {
            let asset = if selected.get() == 4 {
                filled::LABEL_SVG
            } else if open.is_some() {
                outlined::LABEL_SVG
            } else if folders {
                outlined::FOLDER_SVG
            } else {
                outlined::RADIO_BUTTON_UNCHECKED_SVG
            };
            boxed()
                .modifier(observe(geometry, "label_icon"))
                .children(move || {
                    icon().vector(asset).size(Dp(24.0));
                });
        })
        .on_click(move || {
            if let Some(open) = open {
                selected.set(4);
                open.set(false);
            } else if interactive {
                selected.set(4);
            }
        });
    if open.is_none() {
        let rows = if folders {
            [
                ("Volunteering", outlined::FOLDER_SVG),
                ("Work", outlined::FOLDER_SVG),
            ]
        } else {
            [
                ("Label", outlined::CHANGE_HISTORY_SVG),
                ("Label", outlined::CROP_SQUARE_SVG),
            ]
        };
        for (label, asset) in rows {
            navigation_drawer_item()
                .label(label)
                .selected(false)
                .icon(move || {
                    icon().vector(asset).size(Dp(24.0));
                });
        }
    }
}

fn observe(geometry: Option<State<Geometry>>, name: &'static str) -> Modifier {
    geometry
        .map(|geometry| watch(geometry, name))
        .unwrap_or_default()
}

#[tessera]
fn drawer_stage(
    width: f64,
    height: f64,
    label: String,
    geometry: State<Geometry>,
    dark: Option<bool>,
    square_bottom: Option<bool>,
    background: Option<tessera_ui::Color>,
    child: Option<RenderSlot>,
) {
    let child = child.unwrap_or_else(RenderSlot::empty);
    let dark = dark.unwrap_or(false);
    let square_bottom = square_bottom.unwrap_or(false);
    material_theme()
        .theme(move || MaterialTheme::from_seed(srgb(103, 80, 164), dark))
        .child(move || {
            drawer_stage_canvas(width, height, label.clone(), geometry)
                .square_bottom(square_bottom)
                .background_optional(background)
                .child_shared(child);
        });
}

fn stage_shape(square_bottom: bool) -> Shape {
    let round = RoundedCorner::manual(Dp(24.0), 3.0);
    let bottom = if square_bottom {
        RoundedCorner::manual(Dp::ZERO, 3.0)
    } else {
        round
    };
    Shape::RoundedRectangle {
        top_left: round,
        top_right: round,
        bottom_right: bottom,
        bottom_left: bottom,
    }
}

#[tessera]
fn drawer_stage_canvas(
    width: f64,
    height: f64,
    label: String,
    geometry: State<Geometry>,
    square_bottom: Option<bool>,
    background: Option<tessera_ui::Color>,
    child: Option<RenderSlot>,
) {
    let child = child.unwrap_or_else(RenderSlot::empty);
    let square_bottom = square_bottom.unwrap_or(false);
    let measured = remember(move || Dp(width));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let background = background.unwrap_or(scheme.secondary_container);
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: tessera_ui::LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if measured.is_alive() && next.0 > 0.0 && measured.get() != next {
                    measured.set(next);
                }
            },
        ))
        .children(move || {
            let label = label.clone();
            scaled_box((measured.get().0 / width) as f32).child(move || {
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(
                        watch(geometry, "figure")
                            .required_size(Dp(width), Dp(height))
                            .background_with_shape(background, stage_shape(square_bottom))
                            .clip_shape(stage_shape(square_bottom))
                            .semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some(label.clone()),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || child.render());
            });
        });
}

#[tessera]
fn drawer_scene(
    modal: bool,
    selected: State<usize>,
    geometry: Option<State<Geometry>>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    folders: Option<bool>,
    backdrop: Option<bool>,
) {
    let folders = folders.unwrap_or(false);
    let backdrop = backdrop.unwrap_or(false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            observe(geometry, "screen")
                .offset(Dp(x), Dp(y))
                .required_size(Dp(width), Dp(height))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(24.0)))
                .border_with_shape(
                    Dp(8.0),
                    scheme.outline_variant,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
        )
        .children(move || {
            if backdrop {
                let start = if modal { 8.0 } else { 368.0 };
                boxed()
                    .modifier(Modifier::new().offset(Dp(start), Dp(8.0)))
                    .children(move || {
                        mail_messages(selected, (width - start - 8.0).max(0.0));
                    });
            }
            if modal {
                boxed().modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background(scheme.scrim.with_alpha(0.32)),
                );
                boxed().modifier(
                    observe(geometry, "scrim")
                        .offset(Dp(368.0), Dp::ZERO)
                        .required_size(Dp((width - 368.0).max(0.0)), Dp(height)),
                );
            }
            boxed()
                .modifier(Modifier::new().offset(Dp(8.0), Dp(8.0)))
                .children(move || {
                    drawer_content(modal, selected)
                        .geometry_optional(geometry)
                        .folders(folders)
                        .height(Dp(height - 8.0));
                });
        });
}

#[tessera]
fn drawer_label_probes(geometry: State<Geometry>) {
    let values = geometry.with(space_geometry);
    let style = use_context::<MaterialTheme>()
        .unwrap()
        .get()
        .typography
        .label_large;
    for (name, row_name, icon_name, label) in [
        ("label", "indicator", "icon", "Inbox"),
        ("inactive_label", "outbox", "inactive_icon", "Outbox"),
        ("folder_label", "label_row", "label_icon", "Label"),
    ] {
        let (Some(row), Some(icon)) = (values.get(row_name), values.get(icon_name)) else {
            continue;
        };
        let x = icon.x + icon.w + 12.0;
        let end = if name == "label" {
            values
                .get("badge")
                .map_or(row.x + row.w - 16.0, |badge| badge.x - 12.0)
        } else {
            row.x + row.w - 16.0
        };
        boxed()
            .alignment(Alignment::CenterStart)
            .modifier(
                Modifier::new()
                    .offset(Dp(x), Dp(row.y))
                    .required_size(Dp((end - x).max(0.0)), Dp(row.h)),
            )
            .children(move || {
                text()
                    .content(label)
                    .style(style)
                    .color(tessera_ui::Color::TRANSPARENT)
                    .accessibility_label(format!("Drawer {name} measurement probe"))
                    .modifier(watch(geometry, name).semantics(SemanticsArgs {
                        hidden: true,
                        ..Default::default()
                    }));
            });
    }
}

#[tessera]
fn drawer_marker(number: usize, x: f64, y: f64) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(24.0), Dp(32.0))
                .background_with_shape(srgb(48, 48, 52), Shape::CAPSULE)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Drawer marker {number}")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(tessera_ui::Color::WHITE);
        });
}

fn leader(svg: &mut String, from: (f64, f64), to: (f64, f64)) {
    svg.push_str(&format!("<path d='M{},{}L{},{}' stroke='#49454f' stroke-width='1' fill='none'/><circle cx='{}' cy='{}' r='3' fill='none' stroke='#49454f'/>", from.0, from.1, to.0, to.1, to.0, to.1));
}

fn ruler(svg: &mut String, from: (f64, f64), to: (f64, f64)) {
    svg.push_str(&format!(
        "<path d='M{},{}L{},{}' stroke='#e85c79' stroke-width='1' fill='none'/>",
        from.0, from.1, to.0, to.1
    ));
    for (x, y) in [from, to] {
        let (dx, dy) = if from.1 == to.1 {
            (0.0, 4.0)
        } else {
            (4.0, 0.0)
        };
        svg.push_str(&format!(
            "<path d='M{},{}L{},{}' stroke='#e85c79' stroke-width='1' fill='none'/>",
            x - dx,
            y - dy,
            x + dx,
            y + dy
        ));
    }
}

#[tessera]
fn figure(modal: bool, measurements: bool) {
    drawer_figure(modal, if measurements { 1 } else { 0 });
}

#[tessera]
fn drawer_figure(modal: bool, mode: u8, geometry: Option<State<Geometry>>) {
    let geometry = if let Some(geometry) = geometry {
        geometry
    } else {
        remember(Geometry::new)
    };
    let selected = remember(|| 0usize);
    let (width, height) = if matches!(mode, 1 | 2) {
        (512.0, 584.0)
    } else if matches!(mode, 3 | 4) {
        (1024.0, 784.0)
    } else {
        (1024.0, 488.0)
    };
    let label = match mode {
        1 => "element sizes",
        2 => "padding and margins",
        3 => "color roles",
        4 => "guidelines anatomy",
        _ => "anatomy",
    };
    drawer_stage(
        width,
        height,
        format!(
            "{} navigation drawer {label}",
            if modal { "Modal" } else { "Standard" }
        ),
        geometry,
    )
    .child(move || {
        drawer_scene(
            modal,
            selected,
            88.0,
            if matches!(mode, 1 | 2) { 80.0 } else { 96.0 },
            if matches!(mode, 1 | 2) { 480.0 } else { 848.0 },
            696.0,
        )
        .geometry(geometry);
        drawer_label_probes(geometry);
        boxed()
            .modifier(Modifier::new().required_size(Dp(width), Dp(height)))
            .children(move || {
                figure_annotation_content(mode, geometry.with(space_geometry));
            });
    });
}

#[tessera]
fn figure_annotation_content(mode: u8, values: BTreeMap<&'static str, Rect>) {
    let (Some(root), Some(container), Some(indicator)) = (
        values.get("figure"),
        values.get("container"),
        values.get("indicator"),
    ) else {
        return;
    };
    let mut svg = String::new();
    let mut markers = Vec::new();
    if mode == 1 {
        ruler(
            &mut svg,
            (container.x, container.y - 28.0),
            (container.x + container.w, container.y - 28.0),
        );
        annotations::dimension(
            format!("{:.0}", container.w),
            Dp(container.x + container.w * 0.5 - 12.0),
            Dp(container.y - 56.0),
        );
        ruler(
            &mut svg,
            (indicator.x + indicator.w * 0.5, indicator.y),
            (indicator.x + indicator.w * 0.5, indicator.y + indicator.h),
        );
        annotations::dimension(
            format!("{:.0}", indicator.h),
            Dp(indicator.x + indicator.w * 0.5 - 12.0),
            Dp(indicator.y - 32.0),
        );
        for name in ["icon", "trash_icon"] {
            if let Some(icon) = values.get(name) {
                annotations::outline(&mut svg, *icon);
                ruler(
                    &mut svg,
                    (icon.x - 16.0, icon.y),
                    (icon.x - 16.0, icon.y + icon.h),
                );
                annotations::dimension(
                    format!("{:.0}", icon.h),
                    Dp(icon.x - 52.0),
                    Dp(icon.y + 2.0),
                );
            }
        }
        let corner_x = container.x + container.w;
        svg.push_str(&format!(
            "<path d='M{},{}Q{},{} {},{}' stroke='#e85c79' stroke-width='1' fill='none'/>",
            corner_x - 16.0,
            container.y,
            corner_x,
            container.y,
            corner_x,
            container.y + 16.0
        ));
        annotations::dimension("16".into(), Dp(corner_x + 8.0), Dp(container.y + 4.0));
    } else if mode == 2 {
        if let (Some(icon), Some(label), Some(badge), Some(divider)) = (
            values.get("icon"),
            values.get("label"),
            values.get("badge"),
            values.get("divider"),
        ) {
            for (from, to, y) in [
                (badge.x - 12.0, badge.x, indicator.y - 20.0),
                (indicator.x, icon.x, indicator.y + indicator.h + 12.0),
                (icon.x + icon.w, label.x, indicator.y + indicator.h + 40.0),
                (
                    badge.x + badge.w,
                    indicator.x + indicator.w,
                    indicator.y - 20.0,
                ),
                (
                    indicator.x + indicator.w,
                    container.x + container.w,
                    indicator.y + indicator.h + 12.0,
                ),
                (container.x, divider.x, divider.y - 24.0),
                (
                    divider.x + divider.w,
                    container.x + container.w,
                    divider.y - 24.0,
                ),
            ] {
                ruler(&mut svg, (from, y), (to, y));
                annotations::dimension(
                    format!("{:.0}", to - from),
                    Dp((from + to) * 0.5 - 8.0),
                    Dp(y - 26.0),
                );
            }
            if let Some(trash) = values.get("trash") {
                ruler(
                    &mut svg,
                    (container.x - 16.0, trash.y + trash.h),
                    (container.x - 16.0, divider.y),
                );
                annotations::dimension(
                    format!("{:.0}", divider.y - trash.y - trash.h),
                    Dp(container.x - 48.0),
                    Dp(divider.y - 20.0),
                );
            }
            if let (Some(icon), Some(label)) =
                (values.get("label_icon"), values.get("folder_label"))
            {
                let y = icon.y + icon.h + 12.0;
                ruler(&mut svg, (icon.x + icon.w, y), (label.x, y));
                annotations::dimension(
                    format!("{:.0}", label.x - icon.x - icon.w),
                    Dp(icon.x + icon.w),
                    Dp(y + 4.0),
                );
            }
            for (x, y, w, h) in [
                (badge.x - 12.0, indicator.y, 12.0, indicator.h),
                (indicator.x, indicator.y, icon.x - indicator.x, indicator.h),
                (
                    badge.x + badge.w,
                    indicator.y,
                    indicator.x + indicator.w - badge.x - badge.w,
                    indicator.h,
                ),
                (
                    container.x,
                    divider.y - 16.0,
                    divider.x - container.x,
                    root.h - divider.y + 16.0,
                ),
                (
                    divider.x + divider.w,
                    divider.y - 16.0,
                    container.x + container.w - divider.x - divider.w,
                    root.h - divider.y + 16.0,
                ),
            ] {
                svg.push_str(&format!("<rect x='{x}' y='{y}' width='{w}' height='{h}' fill='#e883aa' fill-opacity='.28'/>", h=h.max(0.0)));
            }
        }
    } else {
        let names: Vec<(&str, usize)> = if mode == 3 {
            vec![
                ("container", 1),
                ("headline", 2),
                ("label", 3),
                ("icon", 4),
                ("indicator", 5),
                ("badge", 6),
                ("inactive_icon", 7),
                ("inactive_label", 8),
                ("scrim", 9),
            ]
        } else if mode == 4 {
            vec![
                ("indicator", 1),
                ("icon", 2),
                ("label", 3),
                ("badge", 4),
                ("container", 5),
                ("divider", 6),
                ("subhead", 7),
                ("scrim", 8),
            ]
        } else {
            vec![
                ("container", 1),
                ("headline", 2),
                ("label", 3),
                ("indicator", 4),
                ("badge", 5),
                ("scrim", 6),
                ("icon", 7),
            ]
        };
        for (name, number) in names {
            let Some(rect) = values.get(name) else {
                continue;
            };
            let target = match name {
                "container" => (container.x + 16.0, container.y + 12.0),
                "indicator" => (
                    indicator.x + indicator.w * 0.62,
                    indicator.y + indicator.h * 0.5,
                ),
                "scrim" => (rect.x + rect.w * 0.7, rect.y + 220.0),
                _ => (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5),
            };
            let (mx, my, start) = if name == "scrim" {
                (root.w - 48.0, target.1 - 16.0, (root.w - 48.0, target.1))
            } else if (mode == 3 && matches!(name, "icon" | "inactive_icon"))
                || (mode != 3 && name == "icon")
            {
                (
                    container.x - 48.0,
                    target.1 - 16.0,
                    (container.x - 24.0, target.1),
                )
            } else if name == "inactive_label" {
                (
                    container.x + container.w + 96.0,
                    target.1 - 16.0,
                    (container.x + container.w + 96.0, target.1),
                )
            } else {
                (target.0 - 12.0, 48.0, (target.0, 80.0))
            };
            leader(&mut svg, start, target);
            if matches!(
                name,
                "headline" | "label" | "icon" | "badge" | "inactive_icon" | "inactive_label"
            ) {
                annotations::outline(&mut svg, *rect);
            }
            markers.push((number, mx, my));
        }
    }
    annotations::lines(Dp(root.w), Dp(root.h), svg);
    for (number, x, y) in markers {
        drawer_marker(number, x, y);
    }
}

#[tessera]
fn legend_entries(entries: Vec<(usize, String)>) {
    for (number, label) in entries {
        row()
            .cross_axis_alignment(tessera_components::alignment::CrossAxisAlignment::Center)
            .children(move || {
                boxed()
                    .modifier(Modifier::new().size(Dp(24.0), Dp(32.0)))
                    .children(move || {
                        drawer_marker(number, 0.0, 0.0);
                    });
                spacer().modifier(Modifier::new().width(Dp(16.0)));
                text().content(label.clone()).style(type_style(16.0, 24.0));
            });
        gap(Dp(12.0));
    }
}

#[tessera]
fn legend() {
    gap(Dp(16.0));
    legend_entries(vec![
        (1, "Container".into()),
        (2, "Headline".into()),
        (3, "Label text".into()),
        (4, "Active indicator".into()),
        (5, "Badge label text".into()),
        (6, "Scrim".into()),
        (7, "Icon".into()),
    ]);
}

#[tessera]
fn overview_figure() {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0usize);
    drawer_stage(
        1024.0,
        572.0,
        "Standard and modal navigation drawers".into(),
        geometry,
    )
    .child(move || {
        drawer_scene(false, selected, 56.0, 120.0, 640.0, 640.0);
        drawer_scene(true, selected, 548.0, 84.0, 424.0, 640.0);
        drawer_marker(1, 284.0, 32.0);
        drawer_marker(2, 718.0, 32.0);
    });
    gap(Dp(16.0));
    legend_entries(vec![
        (1, "Standard navigation drawer".into()),
        (2, "Modal navigation drawer".into()),
    ]);
}

#[tessera]
fn differences_figure() {
    row().children(|| {
        for legacy in [true, false] {
            column().modifier(Modifier::new().weight(1.0)).children(move || {
                let geometry = remember(Geometry::new);
                let selected = remember(|| 0usize);
                drawer_stage(512.0, 356.0, if legacy { "M2 rectangular active indicator".into() } else { "M3 pill-shaped active indicator".into() }, geometry).background_optional(if legacy { Some(srgb(230, 230, 230)) } else { None }).child(move || {
                    if legacy {
                        let theme = use_context::<MaterialTheme>().unwrap().get();
                        surface().shape(Shape::rounded_rectangle(Dp::ZERO)).style(tessera_ui::Color::WHITE.into())
                            .modifier(Modifier::new().offset(Dp(48.0), Dp(40.0)).required_size(Dp(360.0), Dp(440.0)))
                            .child(move || {
                                column().modifier(Modifier::new().padding_all(Dp(12.0))).children(move || {
                                    for (index, label) in ["Inbox", "Outbox", "Favorites", "Archive"].into_iter().enumerate() {
                                        surface().shape(Shape::rounded_rectangle(Dp::ZERO))
                                            .style(if index == 0 { srgb(238, 228, 253) } else { tessera_ui::Color::WHITE }.into())
                                            .modifier(Modifier::new().fill_max_width().height(Dp(56.0)))
                                            .child(move || {
                                                row().modifier(Modifier::new().fill_max_size().padding_symmetric(Dp(16.0), Dp::ZERO))
                                                    .cross_axis_alignment(tessera_components::alignment::CrossAxisAlignment::Center)
                                                    .children(move || {
                                                        icon().vector(match index { 0=>filled::MAIL_SVG,1=>filled::SEND_SVG,2=>filled::FAVORITE_SVG,_=>filled::ARCHIVE_SVG }).size(Dp(24.0)).tint(if index == 0 { srgb(98, 0, 238) } else { srgb(97, 97, 97) });
                                                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                                                        text().content(label).style(theme.typography.label_large).color(if index == 0 { srgb(98, 0, 238) } else { srgb(33, 33, 33) });
                                                    });
                                            });
                                    }
                                });
                            });
                    } else {
                        drawer_scene(true, selected, 48.0, 40.0, 480.0, 440.0);
                    }
                });
                gap(Dp(16.0));
                guides::caption(if legacy { "M2: Navigation drawer had square corners and a rectangular shape indicating the active destination" } else { "M3: Navigation drawer has rounded corners, new color mappings, and an updated style for indicating the active destination" });
            });
            if legacy { spacer().modifier(Modifier::new().width(Dp(24.0))); }
        }
    });
}

#[tessera]
fn measurement_pair(modal: bool) {
    row().children(move || {
        for mode in [1, 2] {
            column()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    drawer_figure(modal, mode);
                    gap(Dp(16.0));
                    prose(if mode == 1 {
                        "Element size measurements"
                    } else {
                        "Padding and margins"
                    });
                });
            if mode == 1 {
                spacer().modifier(Modifier::new().width(Dp(24.0)));
            }
        }
    });
}

#[tessera]
fn state_panel(number: usize, label: String, state: InteractionVisualState, x: f64, y: f64) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    boxed().modifier(Modifier::new().offset(Dp(x),Dp(y)).required_size(Dp(512.0),Dp(392.0)).clip_to_bounds()
        .semantics(SemanticsArgs { role:Some(Role::Group),label:Some(format!("Drawer {label} state presentation")),merge_descendants:true,..Default::default() }))
        .children(move || {
            drawer_marker(number, 20.0, 20.0);
            text().content(label.clone()).style(type_style(14.0,20.0)).modifier(Modifier::new().offset(Dp(60.0),Dp(26.0)));
            boxed().modifier(Modifier::new().offset(Dp(128.0),Dp(88.0)).required_size(Dp(400.0),Dp(320.0))
                .background_with_shape(theme.color_scheme.surface, Shape::rounded_rectangle(Dp(24.0)))
                .border_with_shape(Dp(8.0), theme.color_scheme.outline_variant, Shape::rounded_rectangle(Dp(24.0))));
            boxed().modifier(Modifier::new().offset(Dp(136.0),Dp(96.0)))
                .children(move || {
                    navigation_drawer().modifier(Modifier::new().height(Dp(400.0))).content(move || {
                        text().content("Mail").style(theme.typography.title_small).modifier(Modifier::new().padding_all(Dp(16.0)));
                        for (index, destination) in DESTINATIONS.into_iter().take(4).enumerate() {
                            if index == 2 {
                                surface().shape(Shape::CAPSULE).style(tessera_ui::Color::TRANSPARENT.into())
                                    .content_color(theme.color_scheme.on_surface_variant)
                                    .presentation_state(state).show_focus_indicator(true)
                                    .ripple_color(theme.color_scheme.on_surface_variant)
                                    .presentation_ripple(RipplePresentation { origin:[0.6,0.5],radius:0.9,opacity:0.12 })
                                    .modifier(Modifier::new().fill_max_width().height(Dp(56.0)))
                                    .child(move || {
                                        row().cross_axis_alignment(tessera_components::alignment::CrossAxisAlignment::Center)
                                            .modifier(Modifier::new().fill_max_size().padding_symmetric(Dp(16.0),Dp::ZERO))
                                            .children(move || {
                                                icon().vector(outlined::FAVORITE_BORDER_SVG).size(Dp(24.0));
                                                spacer().modifier(Modifier::new().width(Dp(12.0)));
                                                text().content("Favorites").style(theme.typography.label_large);
                                            });
                                    });
                            } else {
                                navigation_drawer_item().label(destination).selected(index == 0)
                                    .icon(move || { icon().vector(match index {0=>filled::INBOX_SVG,1=>outlined::SEND_SVG,_=>outlined::DELETE_SVG}).size(Dp(24.0)); })
                                    .badge(move || { if index == 0 { text().content("24").style(theme.typography.label_large); } });
                            }
                        }
                    });
                });
        });
}

#[tessera]
fn tokens() {
    prose("The navigation drawer has one token set. Learn about design tokens");
    gap(Dp(24.0));
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
fn measurements(modal: bool) {
    let mut rows = vec![
        vec!["Container height".into(), "100%".into()],
        vec!["Container width".into(), "360dp".into()],
    ];
    if !modal {
        rows.push(vec![
            "Container shape".into(),
            "0,16,16,0dp corner radii".into(),
        ]);
    }
    for (attribute, value) in [
        ("Icon size", "24dp"),
        ("Active indicator height", "56dp"),
        ("Active indicator shape", "28dp"),
        ("Active indicator width", "336dp"),
        ("Horizontal label alignment", "Start-aligned"),
        ("Left padding", "28dp"),
        ("Right padding", "28dp"),
        ("Active indicator padding", "12dp"),
        ("Padding between elements", "0dp"),
    ] {
        rows.push(vec![attribute.into(), value.into()]);
    }
    spec_table(vec!["Attribute".into(), "Value".into()], rows);
}

#[tessera]
fn colors() {
    prose(
        "Color values are implemented through design tokens. For design, this means working with color values that correspond with tokens. For implementation, a color value will be a token that references a value. Learn more about design tokens",
    );
    gap(Dp(32.0));
    drawer_figure(true, 3);
    gap(Dp(24.0));
    prose("Navigation drawer color roles used for light and dark schemes:");
    gap(Dp(16.0));
    legend_entries(vec![
        (1, "Surface container low".into()),
        (2, "On surface variant".into()),
        (3, "On secondary container".into()),
        (4, "On secondary container".into()),
        (5, "Secondary container".into()),
        (6, "On secondary container".into()),
        (7, "On surface variant".into()),
        (8, "On surface variant".into()),
        (9, "Scrim".into()),
    ]);
    gap(Dp(24.0));
    prose("For divider color roles, go to divider specs.");
}

#[tessera]
fn states() {
    prose(
        "States are visual representations used to communicate the status of a component or interactive element. Learn more about interaction states",
    );
    gap(Dp(32.0));
    let geometry = remember(Geometry::new);
    drawer_stage(
        1024.0,
        784.0,
        "Drawer enabled hovered focused pressed states".into(),
        geometry,
    )
    .child(|| {
        for (number, label, state, x, y) in [
            (1, "Enabled", InteractionVisualState::Rest, 0.0, 0.0),
            (2, "Hovered", InteractionVisualState::Hover, 512.0, 0.0),
            (3, "Focused", InteractionVisualState::Focus, 0.0, 392.0),
            (4, "Pressed", InteractionVisualState::Pressed, 512.0, 392.0),
        ] {
            state_panel(number, label.into(), state, x, y);
        }
        let outline = use_context::<MaterialTheme>()
            .unwrap()
            .get()
            .color_scheme
            .outline_variant;
        boxed().modifier(
            Modifier::new()
                .offset(Dp(511.5), Dp::ZERO)
                .required_size(Dp(1.0), Dp(784.0))
                .background(outline),
        );
        boxed().modifier(
            Modifier::new()
                .offset(Dp::ZERO, Dp(391.5))
                .required_size(Dp(1024.0), Dp(1.0))
                .background(outline),
        );
    });
    gap(Dp(24.0));
    prose("Navigation drawer states:");
    gap(Dp(16.0));
    legend_entries(vec![
        (1, "Enabled".into()),
        (2, "Hovered".into()),
        (3, "Focused".into()),
        (4, "Pressed".into()),
    ]);
    gap(Dp(24.0));
    prose("State specs are in the tokens module above");
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "drawers_tests.rs"]
mod tests;
