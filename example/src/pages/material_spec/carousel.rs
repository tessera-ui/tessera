//! Interactive carousel specifications and browsing patterns.
//!
//! ## Usage
//!
//! Compare media layouts, scrolling and accessible collection browsing.

use std::sync::Arc;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    app_bar::top_app_bar,
    boxed::boxed,
    button::button,
    carousel::{CarouselController, CarouselFlingBehavior, CarouselVariant, carousel},
    column::column,
    icon::icon,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    scrollable::ScrollableController,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article, modal_host::hosted_dialog},
    gap, panel, spec_table, subheading, type_style,
};

const VARIANTS: [CarouselVariant; 6] = [
    CarouselVariant::MultiBrowse,
    CarouselVariant::Uncontained,
    CarouselVariant::UncontainedMultiAspectRatio,
    CarouselVariant::Hero,
    CarouselVariant::CenteredHero,
    CarouselVariant::FullScreen,
];

fn name(variant: CarouselVariant) -> &'static str {
    match variant {
        CarouselVariant::MultiBrowse => "Multi-browse",
        CarouselVariant::Uncontained => "Uncontained",
        CarouselVariant::UncontainedMultiAspectRatio => "Uncontained multi-aspect ratio",
        CarouselVariant::Hero => "Hero",
        CarouselVariant::CenteredHero => "Center-aligned hero",
        CarouselVariant::FullScreen => "Full-screen",
    }
}

#[tessera]
pub(super) fn carousel_spec() {
    let mut sections = vec![
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
            title: "Carousel item dynamic widths",
            content: RenderSlot::new(dynamic_widths),
        },
    ];
    for variant in VARIANTS {
        sections.push(SpecSection {
            title: name(variant),
            content: RenderSlot::new(move || {
                paragraph(description(variant).into());
                gap(Dp(32.0));
                layout_figure(variant, false);
                gap(Dp(32.0));
                subheading("Measurements".into());
                gap(Dp(24.0));
                layout_figure(variant, true);
                gap(Dp(24.0));
                measurement_table(variant);
            }),
        });
    }
    article("Carousel", "Carousels show a collection of items that can be scrolled on and off the screen", sections)
        .intro(|| { layout_figure(CarouselVariant::MultiBrowse, false); })
        .pages(vec![
            ArticlePage {
                title: "Overview",
                intro: Some(RenderSlot::new(|| {
                    paragraph("Browse visual collections with six layouts. Items mask and reveal their content as they move, keeping the next item visible without shrinking the content inside.".into());
                    gap(Dp(32.0));
                    collection(CarouselVariant::MultiBrowse, true);
                })),
                sections: vec![
                    SpecSection { title: "Updates", content: RenderSlot::new(|| {
                        paragraph("November 2025: uncontained multi-aspect ratio joins multi-browse, uncontained, hero, center-aligned hero and full-screen layouts.".into());
                        gap(Dp(24.0));
                        collection(CarouselVariant::UncontainedMultiAspectRatio, true);
                    }) },
                    SpecSection { title: "Differences from M2", content: RenderSlot::new(|| {
                        paragraph("Carousels were introduced in Material 3. Dynamic masks, parallax and snapping preserve a recognizable layout as items move through the collection.".into());
                        gap(Dp(24.0));
                        collection(CarouselVariant::Hero, true);
                    }) },
                    SpecSection { title: "Research", content: RenderSlot::new(|| {
                        paragraph("Material's carousel studies found that a previewed item signals more content to browse. Multi-browse supports exploring a collection; hero emphasizes one item at a time.".into());
                    }) },
                ],
            },
            ArticlePage {
                title: "Guidelines",
                intro: None,
                sections: vec![
                    SpecSection { title: "Usage", content: RenderSlot::new(usage) },
                    SpecSection { title: "Anatomy", content: RenderSlot::new(|| {
                        layout_figure(CarouselVariant::MultiBrowse, false);
                        gap(Dp(32.0));
                        subheading("Container".into());
                        paragraph("The rectangular container holds items and adapts their number to its available width.".into());
                        gap(Dp(24.0));
                        subheading("Carousel items".into());
                        paragraph("Large, medium and small masks reveal different portions of full-size content. Large item width is configurable; small items stay within 40–56dp.".into());
                        gap(Dp(24.0));
                        subheading("Item text (optional)".into());
                        paragraph("Keep text brief and readable. Prefer uncontained layouts for longer text, and avoid more than two lines at compact breakpoints.".into());
                    }) },
                    SpecSection { title: "Adaptive design", content: RenderSlot::new(|| {
                        paragraph("Wider containers reveal more large and medium items. At compact breakpoints, show up to three items when they contain text. Full-screen always shows one item.".into());
                        gap(Dp(24.0));
                        collection(CarouselVariant::CenteredHero, true);
                    }) },
                    SpecSection { title: "Behavior", content: RenderSlot::new(|| {
                        subheading("Scrolling".into());
                        paragraph("Use free scrolling for uncontained collections. Multi-browse, hero and full-screen snap into their layout when released. Full-screen scrolls vertically and must stop at an item boundary.".into());
                        gap(Dp(24.0));
                        collection(CarouselVariant::Hero, true);
                        gap(Dp(32.0));
                        collection(CarouselVariant::FullScreen, false);
                    }) },
                ],
            },
            ArticlePage {
                title: "Accessibility",
                intro: None,
                sections: vec![
                    SpecSection { title: "Use cases", content: RenderSlot::new(|| {
                        paragraph("People must be able to enter the collection, move between items, activate an item and skip the collection using assistive technology.".into());
                    }) },
                    SpecSection { title: "Requirements on scrolling pages", content: RenderSlot::new(|| {
                        paragraph("Provide a vertically arranged alternative on a scrolling page. Show all opens a separate vertically scrolling collection page; Back to carousel restores the horizontal view. Keep these controls outside the item container.".into());
                        gap(Dp(24.0));
                        collection(CarouselVariant::MultiBrowse, true);
                    }) },
                    SpecSection { title: "Interaction & style", content: RenderSlot::new(|| {
                        subheading("Touch".into());
                        paragraph("Items respond to activation with a ripple. Drag horizontally to browse, or vertically in the full-screen layout.".into());
                        gap(Dp(24.0));
                        subheading("Cursor".into());
                        paragraph("Hover and focus treatments communicate which item is interactive.".into());
                        gap(Dp(24.0));
                        subheading("Initial focus".into());
                        paragraph("Focus should begin on the first interactive item rather than a decorative container. Keep the alternative collection control reachable after the items.".into());
                    }) },
                    SpecSection { title: "Keyboard navigation", content: RenderSlot::new(|| {
                        spec_table(vec!["Keys".into(), "Actions".into()], vec![
                            vec!["Tab or arrows".into(), "Move between carousel items".into()],
                            vec!["Space or Enter".into(), "Activate the focused item".into()],
                        ]);
                        gap(Dp(24.0));
                        collection(CarouselVariant::MultiBrowse, true);
                    }) },
                    SpecSection { title: "Labeling elements", content: RenderSlot::new(|| {
                        paragraph("Give the collection a descriptive label and name each item's content and position in the set. The controls below expose individual labels such as Open collection 1 of 6.".into());
                    }) },
                    SpecSection { title: "Reduced motion", content: RenderSlot::new(|| {
                        paragraph("Without dynamic masking, use equal-size items and remove parallax. The uncontained alternative keeps all content at its natural size and lets it reach the container edge.".into());
                        gap(Dp(24.0));
                        collection(CarouselVariant::Uncontained, true);
                    }) },
                ],
            },
        ]);
}

fn description(variant: CarouselVariant) -> &'static str {
    match variant {
        CarouselVariant::MultiBrowse => {
            "The multi-browse layout shows large, medium and small items together."
        }
        CarouselVariant::Uncontained => {
            "Uncontained items preserve their width and scroll to the container edge."
        }
        CarouselVariant::UncontainedMultiAspectRatio => {
            "Uncontained multi-aspect ratio items preserve independent widths, from portrait to landscape."
        }
        CarouselVariant::Hero => "Hero emphasizes a large item while previewing the next item.",
        CarouselVariant::CenteredHero => {
            "A center-aligned hero places the large item between two small previews."
        }
        CarouselVariant::FullScreen => {
            "Full-screen presents one edge-to-edge item in a vertical, portrait feed."
        }
    }
}

#[tessera]
fn paragraph(content: String) {
    text().content(content).style(type_style(16.0, 24.0));
}

#[tessera]
fn collection(variant: CarouselVariant, allow_all: bool) {
    let controller = remember(CarouselController::default);
    let all = remember(|| false);
    let selected = remember(|| 0_usize);
    hosted_dialog(all)
        .full_screen(true)
        .accessibility_label("All carousel collections")
        .on_close_request(move || all.set(false))
        .main_content(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    carousel_content(variant, controller, selected);
                    if allow_all {
                        gap(Dp(4.0));
                        button()
                            .text()
                            .accessibility_label("Show all carousel items")
                            .on_click(move || all.set(true))
                            .child(|| {
                                text().content("Show all");
                            });
                    }
                    if selected.get() != 0 {
                        gap(Dp(8.0));
                        text()
                            .content(format!("Collection {} opened", selected.get()))
                            .style(type_style(14.0, 20.0));
                    }
                });
        })
        .dialog_content(move || {
            all_items_page(all, selected);
        });
}

#[tessera]
fn all_items_page(open: State<bool>, selected: State<usize>) {
    let scroll = remember(ScrollableController::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    surface()
        .modifier(Modifier::new().fill_max_size())
        .style(scheme.surface.into())
        .child(move || {
            column()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    top_app_bar()
                        .title("All collections")
                        .container_color(scheme.surface)
                        .navigation_icon(move || {
                            icon_button()
                                .icon(filled::ARROW_BACK_SVG)
                                .accessibility_label("Back to carousel")
                                .on_click(move || open.set(false));
                        });
                    boxed()
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .weight(1.0)
                                .vertical_scroll(scroll),
                        )
                        .children(move || {
                            column()
                                .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0)))
                                .children(move || {
                                    if selected.get() != 0 {
                                        text()
                                            .content(format!(
                                                "Collection {} opened",
                                                selected.get()
                                            ))
                                            .style(type_style(14.0, 20.0));
                                        gap(Dp(16.0));
                                    }
                                    for index in 0..6 {
                                        media_item(index, selected).modifier(
                                            Modifier::new().fill_max_width().height(Dp(180.0)),
                                        );
                                        gap(Dp(8.0));
                                    }
                                });
                        });
                });
        });
}

#[tessera]
fn carousel_content(
    variant: CarouselVariant,
    controller: State<CarouselController>,
    selected: State<usize>,
    geometry: Option<State<Geometry>>,
) {
    let modifier = geometry
        .map(|geometry| watch(geometry, "container"))
        .unwrap_or_default();
    let height = if variant == CarouselVariant::FullScreen {
        Dp(420.0)
    } else {
        Dp(180.0)
    };
    let ratios: Arc<[f32]> = Arc::from([16.0 / 9.0, 9.0 / 16.0, 1.0, 3.0 / 4.0, 4.0 / 3.0, 1.0]);
    boxed()
        .alignment(Alignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let ratios = ratios.clone();
            boxed()
                .modifier(
                    (if variant == CarouselVariant::FullScreen {
                        modifier.clone().width(Dp(280.0))
                    } else {
                        modifier.clone().fill_max_width()
                    })
                    .padding_symmetric(
                        Dp(0.0),
                        Dp(if variant == CarouselVariant::FullScreen {
                            0.0
                        } else {
                            8.0
                        }),
                    ),
                )
                .children(move || {
                    carousel(6)
                        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                            test_tag: Some("spec-carousel-container".into()),
                            merge_descendants: true,
                            ..Default::default()
                        }))
                        .variant(variant)
                        .height(height)
                        .controller(controller)
                        .preferred_item_width(Dp(180.0))
                        .item_aspect_ratios(ratios.clone())
                        .min_small_item_width(Dp(40.0))
                        .max_small_item_width(Dp(56.0))
                        .content_padding_start(Dp(if variant == CarouselVariant::FullScreen {
                            0.0
                        } else {
                            16.0
                        }))
                        .content_padding_end(Dp(
                            if matches!(
                                variant,
                                CarouselVariant::FullScreen
                                    | CarouselVariant::Uncontained
                                    | CarouselVariant::UncontainedMultiAspectRatio
                            ) {
                                0.0
                            } else {
                                16.0
                            },
                        ))
                        .item_spacing(Dp(if variant == CarouselVariant::FullScreen {
                            16.0
                        } else {
                            8.0
                        }))
                        .shape(if variant == CarouselVariant::FullScreen {
                            Shape::RECTANGLE
                        } else {
                            Shape::rounded_rectangle(Dp(28.0))
                        })
                        .fling_behavior(
                            if matches!(
                                variant,
                                CarouselVariant::Uncontained
                                    | CarouselVariant::UncontainedMultiAspectRatio
                            ) {
                                CarouselFlingBehavior::Free
                            } else {
                                CarouselFlingBehavior::SingleAdvance
                            },
                        )
                        .accessibility_label(format!("{} collections", name(variant)))
                        .content(move |item| {
                            let modifier = if let Some(geometry) = geometry {
                                let name = item_name(item.index);
                                let _lease = remember(move || ItemGeometryLease { geometry, name });
                                item_geometry(geometry, name)
                            } else {
                                Modifier::new()
                            };
                            media_item(item.index, selected).modifier(modifier.fill_max_size());
                        });
                });
        });
}

fn item_name(index: usize) -> &'static str {
    ["item0", "item1", "item2", "item3", "item4", "item5"][index]
}

struct ItemGeometryLease {
    geometry: State<Geometry>,
    name: &'static str,
}

impl Drop for ItemGeometryLease {
    fn drop(&mut self) {
        if self.geometry.is_alive() {
            self.geometry.with_mut(|geometry| {
                geometry.remove(self.name);
            });
        }
    }
}

fn item_geometry(geometry: State<Geometry>, name: &'static str) -> Modifier {
    Modifier::new().on_visible_window_bounds(move |bounds: Option<tessera_ui::FloatRect>| {
        if !geometry.is_alive() {
            return;
        }
        let bounds = bounds.map(|bounds| bounds.outward());
        if geometry.with(|geometry| geometry.get(name).copied()) != bounds {
            geometry.with_mut(|geometry| {
                if let Some(bounds) = bounds {
                    geometry.insert(name, bounds);
                } else {
                    geometry.remove(name);
                }
            });
        }
    })
}

#[tessera]
fn media_item(index: usize, selected: State<usize>, modifier: Option<Modifier>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (background, foreground) = match index % 3 {
        0 => (scheme.primary_container, scheme.on_primary_container),
        1 => (scheme.secondary_container, scheme.on_secondary_container),
        _ => (scheme.tertiary_container, scheme.on_tertiary_container),
    };
    button()
        .modifier(modifier.unwrap_or_default())
        .shape(Shape::RECTANGLE)
        .padding(Dp(0.0))
        .color(background)
        .content_color(foreground)
        .accessibility_label(format!("Open collection {} of 6", index + 1))
        .on_click(move || selected.set(index + 1))
        .child(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    icon()
                        .vector(match index % 3 {
                            0 => filled::LANDSCAPE_SVG,
                            1 => filled::LOCAL_FLORIST_SVG,
                            _ => filled::WATER_DROP_SVG,
                        })
                        .size(Dp(64.0));
                });
        });
}

#[tessera]
fn layout_figure(variant: CarouselVariant, measurements: bool) {
    let geometry = remember(Geometry::new);
    let controller = remember(CarouselController::default);
    let selected = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(if variant == CarouselVariant::FullScreen {
                    552.0
                } else {
                    360.0
                }))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    merge_descendants: true,
                    label: Some(format!(
                        "{} {} figure",
                        name(variant),
                        if measurements {
                            "measurement"
                        } else {
                            "anatomy"
                        }
                    )),
                    ..Default::default()
                }),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .align(Alignment::Center)
                        .width(Dp(496.0))
                        .padding_all(Dp(48.0)),
                )
                .children(move || {
                    carousel_content(variant, controller, selected).geometry(geometry);
                });
            carousel_annotations(geometry, measurements, variant);
        });
    gap(Dp(16.0));
    paragraph(if measurements {
        format!("{} carousel padding and size measurements", name(variant))
    } else if variant == CarouselVariant::FullScreen {
        "1  Container\n2  Large carousel item".into()
    } else if matches!(
        variant,
        CarouselVariant::Uncontained | CarouselVariant::UncontainedMultiAspectRatio
    ) {
        "1  Container\n2  Carousel item".into()
    } else if variant == CarouselVariant::MultiBrowse {
        "1  Container\n2  Large carousel item\n3  Medium carousel item\n4  Small carousel item"
            .into()
    } else {
        "1  Container\n2  Large carousel item\n3  Small carousel item".into()
    });
}

#[tessera]
fn carousel_annotations(geometry: State<Geometry>, measurements: bool, variant: CarouselVariant) {
    carousel_annotation_content(
        geometry.with(annotations::relative_geometry),
        measurements,
        variant,
    );
}

#[tessera]
fn carousel_annotation_content(values: Geometry, measurements: bool, variant: CarouselVariant) {
    let (Some(root), Some(container)) = (values.get("figure"), relative(&values, "container"))
    else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let mut svg = String::new();
    annotations::outline(&mut svg, container);
    if measurements {
        let full = variant == CarouselVariant::FullScreen;
        let start = if full { 0.0 } else { 16.0 };
        let end = if matches!(
            variant,
            CarouselVariant::FullScreen
                | CarouselVariant::Uncontained
                | CarouselVariant::UncontainedMultiAspectRatio
        ) {
            0.0
        } else {
            16.0
        };
        for (x, padding) in [(container.x, start), (container.x + container.w - end, end)] {
            svg.push_str(&format!(
                "<rect x='{x}' y='{}' width='{padding}' height='{}' fill='#e883aa' fill-opacity='.32'/>",
                container.y, container.h
            ));
        }
        dotted(
            &mut svg,
            container.x,
            container.y + container.h + 20.0,
            start,
            false,
            "#b14e70",
        );
        dotted(
            &mut svg,
            container.x + container.w - end,
            container.y + container.h + 20.0,
            end,
            false,
            "#b14e70",
        );
        annotations::dimension(
            format!("{start:.0}"),
            Dp(container.x),
            Dp(container.y + container.h + 24.0),
        );
        annotations::dimension(
            format!("{end:.0}"),
            Dp(container.x + container.w - 20.0),
            Dp(container.y + container.h + 24.0),
        );
        annotations::dimension(
            if full { "0" } else { "8" }.into(),
            Dp(container.x + container.w / 2.0),
            Dp(container.y - 28.0),
        );
    } else {
        dotted(
            &mut svg,
            container.x - 20.0,
            container.y + 16.0,
            20.0,
            false,
            "#49454f",
        );
        annotations::marker(1, Dp(container.x - 44.0), Dp(container.y));
        let count = if variant == CarouselVariant::FullScreen {
            1
        } else if variant == CarouselVariant::MultiBrowse {
            3
        } else if matches!(
            variant,
            CarouselVariant::Hero | CarouselVariant::CenteredHero
        ) {
            2
        } else {
            1
        };
        let mut items = ["item0", "item1", "item2", "item3", "item4", "item5"]
            .into_iter()
            .filter_map(|key| relative(&values, key))
            .filter(|rect| {
                rect.w > 0.0 && rect.x + rect.w > container.x && rect.x < container.x + container.w
            })
            .collect::<Vec<_>>();
        items.sort_by(|a, b| b.w.total_cmp(&a.w));
        items.dedup_by(|a, b| (a.w - b.w).abs() <= 1.0);
        if items.len() > count {
            let last = *items.last().unwrap();
            items = if count == 3 {
                vec![items[0], items[(items.len() - 1) / 2], last]
            } else if count == 2 {
                vec![items[0], last]
            } else {
                vec![items[0]]
            };
        }
        let item_count = items.len();
        for (index, item) in items.into_iter().enumerate() {
            let number = if variant == CarouselVariant::MultiBrowse
                && index > 0
                && index + 1 == item_count
            {
                4
            } else {
                index + 2
            };
            let x = item.x + item.w / 2.0;
            annotations::outline(&mut svg, item);
            dotted(
                &mut svg,
                x,
                container.y + container.h,
                20.0,
                true,
                "#49454f",
            );
            annotations::marker(number, Dp(x - 12.0), Dp(container.y + container.h + 20.0));
        }
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn measurement_table(variant: CarouselVariant) {
    let full = variant == CarouselVariant::FullScreen;
    let mut rows = vec![
        vec!["Alignment".into(), "Vertically centered".into()],
        vec![
            "Leading padding".into(),
            if full { "0dp" } else { "16dp" }.into(),
        ],
        vec![
            "Top/bottom padding".into(),
            if full { "0dp" } else { "8dp" }.into(),
        ],
        vec![
            "Padding between elements".into(),
            if full { "16dp" } else { "8dp" }.into(),
        ],
    ];
    if !full {
        rows.push(vec!["Item corner radius".into(), "28dp".into()]);
    }
    if matches!(
        variant,
        CarouselVariant::MultiBrowse | CarouselVariant::Hero | CarouselVariant::CenteredHero
    ) {
        rows.push(vec![
            "Large item width".into(),
            "Dynamic, or user-set".into(),
        ]);
        rows.push(vec!["Small item width".into(), "40–56dp, dynamic".into()]);
    }
    spec_table(vec!["Attribute".into(), "Value".into()], rows);
}

#[tessera]
fn dynamic_widths() {
    paragraph("All items adapt to the container. Large item maximum width controls how many items fit. Small items have a minimum width of 40dp and a maximum width of 56dp.".into());
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                for width in [40.0, 56.0] {
                    column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            spacer().modifier(
                                Modifier::new()
                                    .size(Dp(width), Dp(180.0))
                                    .background_with_shape(
                                        use_context::<MaterialTheme>()
                                            .unwrap()
                                            .get()
                                            .color_scheme
                                            .surface_variant,
                                        Shape::rounded_rectangle(Dp(28.0)),
                                    ),
                            );
                            gap(Dp(16.0));
                            text()
                                .content(format!("{width:.0}dp"))
                                .style(type_style(14.0, 20.0));
                        });
                    spacer().modifier(Modifier::new().width(Dp(40.0)));
                }
            });
    }));
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Token".into(), "Value".into()],
        vec![
            vec!["Container color".into(), "Surface".into()],
            vec!["Item shape".into(), "28dp".into()],
            vec!["Small item width".into(), "40–56dp".into()],
            vec!["Item spacing".into(), "8dp; full-screen 16dp".into()],
        ],
    );
}

#[tessera]
fn colors() {
    paragraph("Container: surface. Item content supplies its own imagery and content colors, with readable foreground contrast at every visible width.".into());
    gap(Dp(24.0));
    layout_figure(CarouselVariant::MultiBrowse, false);
}

#[tessera]
fn states() {
    paragraph("Enabled, hovered, focused, pressed and disabled item states use the state treatment of the interactive content. Hover or activate an item to inspect its real state layer and ripple.".into());
    gap(Dp(24.0));
    collection(CarouselVariant::MultiBrowse, true);
}

#[tessera]
fn usage() {
    paragraph("Choose multi-browse for visual collections, uncontained for stable widths and longer text, hero to emphasize one item, and full-screen for immersive portrait media. Provide an alternative vertical collection on scrolling pages.".into());
    gap(Dp(32.0));
    for variant in VARIANTS {
        subheading(name(variant).into());
        gap(Dp(16.0));
        paragraph(description(variant).into());
        gap(Dp(24.0));
        collection(variant, variant != CarouselVariant::FullScreen);
        gap(Dp(40.0));
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use std::cell::Cell;

    use tessera_components::theme::material_theme;
    use tessera_ui::{HeadlessConfig, HeadlessRenderer};

    use super::*;

    thread_local! {
        static VARIANT: Cell<CarouselVariant> = const { Cell::new(CarouselVariant::MultiBrowse) };
        static GEOMETRY: Cell<Option<State<Geometry>>> = const { Cell::new(None) };
        static CONTROLLER: Cell<Option<State<CarouselController>>> = const { Cell::new(None) };
    }

    struct ComponentPipelines;

    impl tessera_ui::RenderModule for ComponentPipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[tessera]
    fn fixture() {
        let variant = VARIANT.get();
        material_theme().child(move || {
            let controller = remember(CarouselController::default);
            let selected = remember(|| 0_usize);
            let geometry = remember(Geometry::new);
            GEOMETRY.set(Some(geometry));
            CONTROLLER.set(Some(controller));
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    column()
                        .modifier(Modifier::new().width(Dp(400.0)))
                        .children(move || {
                            carousel_content(variant, controller, selected).geometry(geometry);
                        });
                });
        });
    }

    #[test]
    #[ignore = "Requires a GPU adapter for the real component pipelines"]
    fn live_carousel_figures_keep_horizontal_and_portrait_viewports() {
        for variant in VARIANTS {
            VARIANT.set(variant);
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(ComponentPipelines)],
                HeadlessConfig {
                    width: 800,
                    height: 600,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .expect("headless carousel renderer");
            renderer.render_frames(4);
            let snapshot = renderer
                .accessibility_snapshot()
                .expect("published carousel");
            let label = format!("{} collections", name(variant));
            let viewport = snapshot
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::List && node.label() == Some(label.as_str()))
                .unwrap_or_else(|| {
                    panic!(
                        "missing {label}: {:?}",
                        snapshot
                            .nodes
                            .iter()
                            .map(|(_, node)| (node.role(), node.label()))
                            .collect::<Vec<_>>()
                    )
                });
            let bounds = viewport.1.bounds().expect("laid-out carousel viewport");
            assert_eq!(
                bounds.width(),
                if variant == CarouselVariant::FullScreen {
                    280.0
                } else {
                    400.0
                },
                "{} viewport width",
                name(variant)
            );
            assert_eq!(
                bounds.height(),
                if variant == CarouselVariant::FullScreen {
                    420.0
                } else {
                    180.0
                },
                "{} viewport height",
                name(variant)
            );
            let geometry = GEOMETRY.get().expect("observed figure geometry");
            let controller = CONTROLLER.get().expect("figure controller");
            for target in [5, 0, 3] {
                controller.with_mut(|controller| controller.jump_to_item(target));
                renderer.render_frames(8);
                let values = geometry.get();
                let container = values.get("container").expect("container geometry");
                let visible: Vec<_> = values
                    .iter()
                    .filter(|(name, _)| name.starts_with("item"))
                    .collect();
                assert!(!visible.is_empty(), "{} target {target}", name(variant));
                for (item, rect) in visible {
                    assert!(rect.width.0 > 0 && rect.height.0 > 0);
                    assert!(
                        rect.x >= container.x
                            && rect.y >= container.y
                            && rect.x + rect.width <= container.x + container.width
                            && rect.y + rect.height <= container.y + container.height,
                        "{} target {target}: {item} {rect:?} outside {container:?}",
                        name(variant)
                    );
                }
                assert!(
                    values.contains_key(item_name(target)),
                    "{} target {target}: {values:?}",
                    name(variant)
                );
                if target == 5 {
                    assert!(
                        !values.contains_key("item0"),
                        "unmounted first item must release its annotation: {values:?}"
                    );
                }
            }
            assert!(
                snapshot.nodes.iter().any(|(_, node)| matches!(
                    node.role(),
                    Role::Button | Role::ListItem
                ) && node.label()
                    == Some("Open collection 1 of 6")),
                "{} mounts real interactive content",
                name(variant)
            );
        }
    }
}
