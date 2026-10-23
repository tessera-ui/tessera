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

const DESTINATIONS: [&str; 4] = ["Inbox", "Outbox", "Favorites", "Trash"];

#[tessera]
pub(super) fn drawers_spec() {
    let open = remember(|| false);
    let selected = remember(|| 0usize);
    modal_side_sheet_provider().position(SideSheetPosition::Start).is_open(open.get()).accessibility_label("Mail navigation").on_close_request(move || open.set(false)).main_content(move || {
        article("Navigation drawer", "Navigation drawers let people switch between UI views on larger devices", vec![
            SpecSection { title: "Tokens & specs", content: RenderSlot::new(tokens) },
            SpecSection { title: "Color", content: RenderSlot::new(colors) },
            SpecSection { title: "States", content: RenderSlot::new(states) },
            SpecSection { title: "Measurements", content: RenderSlot::new(|| { subheading("Standard navigation drawer".into()); figure(false,true); gap(Dp(32.0)); subheading("Modal navigation drawer".into()); figure(true,true); measurements(); }) },
        ])
        .intro(|| { recommendation(); figure(true,false); legend(); })
        .pages(vec![
            ArticlePage { title:"Overview", intro:Some(RenderSlot::new(move || { recommendation(); prose("Drawers organize primary destinations in a persistent or modal surface. Their labels and selection indicators make the current destination visible."); gap(Dp(32.0)); launch(open,selected); gap(Dp(32.0)); figure(false,false); })), sections:vec![
                SpecSection {title:"M3 Expressive update",content:RenderSlot::new(|| { recommendation(); })},
                SpecSection {title:"Differences from M2",content:RenderSlot::new(|| { prose("M3 drawers use updated color roles, a pill-shaped active indicator, rounded modal surfaces and consistent 56dp destination rows."); gap(Dp(24.0)); figure(true,false); })},
            ]},
            ArticlePage {title:"Guidelines",intro:Some(RenderSlot::new(recommendation)),sections:vec![
                SpecSection {title:"Usage",content:RenderSlot::new(move || { prose("Use navigation destinations to move between top-level sections. Keep their order stable and group related destinations together."); gap(Dp(24.0)); subheading("Standard navigation drawer".into()); prose("A standard drawer remains alongside the content when the window has enough room."); gap(Dp(24.0)); figure(false,false); gap(Dp(32.0)); subheading("Modal navigation drawer".into()); prose("A modal drawer overlays the page with a scrim and dismisses after selecting a destination or closing the overlay."); gap(Dp(24.0)); launch(open,selected); })},
                SpecSection {title:"Anatomy",content:RenderSlot::new(|| { figure(true,false); legend(); gap(Dp(32.0)); for (title,body) in [("Sheet","Keep the drawer anchored to the start edge with a consistent width."),("Divider (optional)","Separate destination groups without introducing extra row spacing."),("Active indicator","A rounded container and filled icon distinguish the selected destination."),("Label text and icons","Use short destination labels with familiar leading icons."),("Section label (optional)","Name a group when its relationship is not already clear."),("Scrim (modal only)","Dim the main page while modal navigation is open.")] { subheading(title.into()); prose(body); gap(Dp(24.0)); } })},
                SpecSection {title:"Responsive layout",content:RenderSlot::new(|| { subheading("Compact breakpoint".into()); prose("Use a modal drawer only when the navigation cannot remain alongside the content."); gap(Dp(24.0)); subheading("Medium & expanded breakpoints".into()); prose("Consider a rail or a standard drawer when a larger window can support persistent navigation."); gap(Dp(24.0)); subheading("Large and extra-large breakpoints".into()); prose("Keep navigation readable and leave the additional space for the primary content. Current Expressive interfaces should use an expanded navigation rail."); gap(Dp(24.0)); figure(false,true); })},
                SpecSection {title:"Behavior",content:RenderSlot::new(move || { subheading("Scrolling".into()); prose("Allow destinations to scroll when they exceed the available height, keeping each label and indicator together."); gap(Dp(24.0)); subheading("Visibility".into()); prose("Keep a standard drawer visible while its destinations remain relevant. A modal drawer temporarily covers the main page."); gap(Dp(24.0)); subheading("Appearing".into()); prose("Open modal navigation from a clearly named menu control and return focus to it when dismissed."); gap(Dp(24.0)); launch(open,selected); })},
            ]},
            ArticlePage {title:"Accessibility",intro:Some(RenderSlot::new(recommendation)),sections:vec![
                SpecSection {title:"Use cases",content:RenderSlot::new(move || { prose("People must be able to find destinations, move between them and identify the active destination using assistive technology."); gap(Dp(24.0)); launch(open,selected); })},
                SpecSection {title:"Interaction & style",content:RenderSlot::new(|| { prose("Expose selection as well as focus. The active indicator, foreground color and filled icon communicate the selected destination; hover and press feedback communicate interaction."); gap(Dp(24.0)); figure(false,false); })},
                SpecSection {title:"Initial focus",content:RenderSlot::new(move || { prose("When opening a modal drawer, place focus on the first destination. Keep the drawer reachable in the page's logical traversal order."); gap(Dp(24.0)); launch(open,selected); })},
                SpecSection {title:"Closing",content:RenderSlot::new(move || { prose("Dismiss a modal drawer by selecting its scrim, pressing Escape or choosing a destination. Restore focus to the opening menu control."); gap(Dp(24.0)); launch(open,selected); })},
                SpecSection {title:"Visual indicators",content:RenderSlot::new(|| { prose("Pair the active indicator with a filled icon. Unselected destinations use outlined icons so selection remains apparent beyond a change of color."); gap(Dp(24.0)); figure(false,false); })},
                SpecSection {title:"Keyboard navigation",content:RenderSlot::new(|| { spec_table(vec!["Keys".into(),"Actions".into()],vec![vec!["Tab".into(),"Focus the first destination".into()],vec!["Arrow keys".into(),"Move between destinations".into()],vec!["Space or Enter".into(),"Select the focused destination".into()],vec!["Escape".into(),"Dismiss modal navigation".into()]]); })},
                SpecSection {title:"Labeling elements",content:RenderSlot::new(|| { prose("Use the destination name as its accessible label. Expand ambiguous names when needed and preserve the visible wording in the accessible name."); gap(Dp(24.0)); spec_table(vec!["Visible label".into(),"Accessible name".into(),"Role".into()],vec![vec!["Inbox".into(),"Inbox".into(),"Tab".into()],vec!["Recents".into(),"Recent images".into(),"Tab".into()]]); })},
            ]},
        ]);
    }).side_sheet_content(move || {
        drawer_content(true,selected).open(open);
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
fn drawer_content(
    modal: bool,
    selected: State<usize>,
    open: Option<State<bool>>,
    geometry: Option<State<Geometry>>,
) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let mut modifier = Modifier::new().height(Dp(440.0));
    if let Some(geometry) = geometry {
        modifier = modifier.then(watch(geometry, "container"));
    }
    navigation_drawer()
        .style(if modal {
            NavigationDrawerStyle::Modal
        } else {
            NavigationDrawerStyle::Standard
        })
        .modifier(modifier)
        .accessibility_label("Mail destinations")
        .content(move || {
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
            for (index, label) in DESTINATIONS.into_iter().enumerate() {
                let active = selected.get() == index;
                let mut item = navigation_drawer_item()
                    .label(label)
                    .selected(active)
                    .on_click(move || {
                        selected.set(index);
                        if let Some(open) = open {
                            open.set(false);
                        }
                    })
                    .icon(move || {
                        let asset = match (index, active) {
                            (0, true) => filled::INBOX_SVG,
                            (0, false) => outlined::INBOX_SVG,
                            (1, true) => filled::OUTBOX_SVG,
                            (1, false) => outlined::OUTBOX_SVG,
                            (2, true) => filled::STAR_SVG,
                            (2, false) => outlined::STAR_SVG,
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
                if index == 0 {
                    item = item
                        .modifier(
                            geometry
                                .map(|geometry| watch(geometry, "indicator"))
                                .unwrap_or_default(),
                        )
                        .badge(move || {
                            text()
                                .content("24")
                                .style(theme.typography.label_large)
                                .modifier(
                                    geometry
                                        .map(|geometry| watch(geometry, "badge"))
                                        .unwrap_or_default(),
                                );
                        });
                }
                drop(item);
            }
            gap(Dp(8.0));
            horizontal_divider();
            gap(Dp(8.0));
            text()
                .content("Labels")
                .style(theme.typography.title_small)
                .modifier(Modifier::new().padding_all(Dp(16.0)));
            navigation_drawer_item()
                .label("Personal")
                .icon(|| {
                    icon().vector(outlined::LABEL_SVG).size(Dp(24.0));
                })
                .on_click(move || {
                    selected.set(2);
                    if let Some(open) = open {
                        open.set(false);
                    }
                });
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
    let values = geometry.get();
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
                _ => (
                    rect.x + rect.w + 24.0,
                    rect.y + rect.h / 2.0 - 16.0,
                    rect.x + rect.w,
                    rect.y + rect.h / 2.0,
                ),
            };
            dotted(&mut svg, tx, ty, x - tx, false, "#49454f");
            annotations::marker(number, Dp(x), Dp(y));
        }
        annotations::marker(3, Dp(indicator.x + 88.0), Dp(indicator.y + 64.0));
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
