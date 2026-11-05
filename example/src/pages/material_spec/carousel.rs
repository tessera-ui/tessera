//! Interactive carousel specifications and browsing patterns.
//!
//! ## Usage
//!
//! Compare media layouts, scrolling and accessible collection browsing.

use std::sync::Arc;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    app_bar::top_app_bar,
    boxed::boxed,
    button::button,
    carousel::{CarouselController, CarouselFlingBehavior, CarouselVariant, carousel},
    column::column,
    icon::icon,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{InteractionState, ModifierExt as _, SemanticsArgs},
    row::row,
    scrollable::ScrollableController,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
    ACCENT,
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

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const CAUTION_COLOR: Color = Color::from_rgb(0.612, 0.396, 0.0);
const PRESSED_INSET: f64 = 4.0;

fn name(variant: CarouselVariant) -> &'static str {
    match variant {
        CarouselVariant::MultiBrowse => "ui-multi-browse",
        CarouselVariant::Uncontained => "ui-uncontained",
        CarouselVariant::UncontainedMultiAspectRatio => "ui-uncontained-multi-aspect-ratio",
        CarouselVariant::Hero => "ui-hero",
        CarouselVariant::CenteredHero => "ui-center-aligned-hero",
        CarouselVariant::FullScreen => "ui-full-screen-7087",
    }
}

#[tessera]
pub(super) fn carousel_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    let modal_controller = Some(remember(
        crate::pages::material_spec::article::modal_host::ArticleModalState::default,
    ));
    let mut sections = vec![
        SpecSection {
            title: "ui-tokens-specs",
            content: RenderSlot::new(tokens),
        },
        SpecSection {
            title: "ui-color",
            content: RenderSlot::new(colors),
        },
        SpecSection {
            title: "ui-states",
            content: RenderSlot::new(move || {
                states().modal_controller_optional(modal_controller);
            }),
        },
        SpecSection {
            title: "ui-carousel-item-dynamic-widths",
            content: RenderSlot::new(dynamic_widths),
        },
    ];
    for variant in VARIANTS {
        sections.push(SpecSection {
            title: name(variant),
            content: RenderSlot::new(move || {
                layout_spec(variant);
            }),
        });
    }
    article(
        "ui-carousel",
        "ui-carousels-show-a-collection-of-items-that-can-be-scrolled-on-and-off-the-screen",
        sections,
    )
    .modal_controller_optional(modal_controller)
    .route_optional(route)
    .intro(|| {
        layout_figure(CarouselVariant::MultiBrowse, false);
    })
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(move || {
                overview_intro().modal_controller_optional(modal_controller);
            })),
            sections: vec![
                SpecSection {
                    title: "ui-availability-resources",
                    content: RenderSlot::new(availability),
                },
                SpecSection {
                    title: "ui-updates",
                    content: RenderSlot::new(move || {
                        updates().modal_controller_optional(modal_controller);
                    }),
                },
                SpecSection {
                    title: "ui-differences-from-m2",
                    content: RenderSlot::new(move || {
                        differences().modal_controller_optional(modal_controller);
                    }),
                },
                SpecSection {
                    title: "ui-research",
                    content: RenderSlot::new(research),
                },
            ],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: Some(RenderSlot::new(move || {
                guidelines_intro().modal_controller_optional(modal_controller);
            })),
            sections: vec![
                SpecSection {
                    title: "ui-usage",
                    content: RenderSlot::new(move || {
                        usage().modal_controller_optional(modal_controller);
                    }),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(anatomy),
                },
                SpecSection {
                    title: "ui-adaptive-design",
                    content: RenderSlot::new(move || {
                        adaptive_design().modal_controller_optional(modal_controller);
                    }),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(move || {
                        behavior().modal_controller_optional(modal_controller);
                    }),
                },
            ],
        },
        ArticlePage {
            title: "ui-accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-use-cases",
                    content: RenderSlot::new(use_cases),
                },
                SpecSection {
                    title: "ui-requirements-on-scrolling-pages",
                    content: RenderSlot::new(move || {
                        scrolling_requirements().modal_controller_optional(modal_controller);
                    }),
                },
                SpecSection {
                    title: "ui-interaction-style",
                    content: RenderSlot::new(move || {
                        interaction_style().modal_controller_optional(modal_controller);
                    }),
                },
                SpecSection {
                    title: "ui-keyboard-navigation",
                    content: RenderSlot::new(keyboard_navigation),
                },
                SpecSection {
                    title: "ui-labeling-elements",
                    content: RenderSlot::new(labeling),
                },
            ],
        },
    ]);
}

#[tessera]
fn layout_spec(variant: CarouselVariant) {
    paragraph(description(variant).into());
    gap(Dp(32.0));
    layout_figure(variant, false);
    gap(Dp(32.0));
    subheading("ui-measurements".into());
    gap(Dp(24.0));
    layout_figure(variant, true);
    gap(Dp(24.0));
    measurement_table(variant);
}

fn description(variant: CarouselVariant) -> &'static str {
    match variant {
        CarouselVariant::MultiBrowse => {
            "ui-the-multi-browse-layout-shows-at-least-one-large-medium-and-small-carousel-item"
        }
        CarouselVariant::Uncontained => {
            "ui-the-uncontained-layout-shows-items-that-scroll-to-the-edge-of-the-container"
        }
        CarouselVariant::UncontainedMultiAspectRatio => {
            "ui-the-uncontained-multi-aspect-ratio-layout-shows-carousel-items-of-various-widths"
        }
        CarouselVariant::Hero => {
            "ui-the-hero-layout-shows-at-least-one-large-item-and-one-small-item"
        }
        CarouselVariant::CenteredHero => {
            "ui-the-center-aligned-hero-layout-shows-at-least-one-large-item-and-two-small-items"
        }
        CarouselVariant::FullScreen => {
            "ui-the-full-screen-layout-shows-one-edge-to-edge-large-item"
        }
    }
}

fn measurement_caption(variant: CarouselVariant) -> &'static str {
    match variant {
        CarouselVariant::MultiBrowse => {
            "ui-multi-browse-carousels-have-padding-on-both-sides-of-the-container"
        }
        CarouselVariant::Uncontained => {
            "ui-uncontained-carousel-items-bleed-over-the-padding-on-each-side-when-scrolling"
        }
        CarouselVariant::UncontainedMultiAspectRatio => {
            "ui-uncontained-multi-aspect-ratio-carousels-only-have-leading-padding-with-8dp-of-padding-bet"
        }
        CarouselVariant::Hero => "ui-hero-carousels-have-padding-on-both-sides-of-the-container",
        CarouselVariant::CenteredHero => {
            "ui-center-aligned-hero-carousels-have-padding-on-both-sides-of-the-container"
        }
        CarouselVariant::FullScreen => "ui-full-screen-carousels-fill-the-window-edge-to-edge",
    }
}

fn anatomy_labels(variant: CarouselVariant) -> &'static [&'static str] {
    match variant {
        CarouselVariant::MultiBrowse => &[
            "ui-container",
            "ui-large-carousel-item",
            "ui-medium-carousel-item",
            "ui-small-carousel-item",
        ],
        CarouselVariant::Uncontained | CarouselVariant::FullScreen => {
            &["ui-container", "ui-large-carousel-item"]
        }
        CarouselVariant::UncontainedMultiAspectRatio => &[
            "ui-container",
            "ui-carousel-item-16-9",
            "ui-carousel-item-9-16",
            "ui-carousel-item-1-1",
            "ui-carousel-item-3-4",
        ],
        CarouselVariant::Hero | CarouselVariant::CenteredHero => &[
            "ui-container",
            "ui-large-carousel-item",
            "ui-small-carousel-item",
        ],
    }
}

fn guidance(variant: CarouselVariant) -> &'static [&'static str] {
    match variant {
        CarouselVariant::MultiBrowse => &[
            "ui-the-multi-browse-layout-is-best-for-browsing-many-items-at-once-like-photos-or-event-feeds",
            "ui-on-larger-screens-more-large-and-medium-items-are-visible-in-this-layout",
            "ui-avoid-using-this-layout-if-the-carousel-items-need-lots-of-text-or-have-complicated-imager",
        ],
        CarouselVariant::Uncontained => &[
            "ui-the-uncontained-layout-is-most-similar-to-a-traditional-carousel-where-items-are-a-single",
            "ui-both-default-scrolling-and-snap-scrolling-work-well-with-this-layout",
            "ui-since-items-don-t-change-size-this-layout-can-be-customized-to-show-more-text-or-other-ui",
        ],
        CarouselVariant::UncontainedMultiAspectRatio => &[
            "ui-the-uncontained-multi-aspect-ratio-carousel-has-the-same-layout-as-the-uncontained-carouse",
            "ui-item-widths-can-range-anywhere-between-9-16-for-min-width-size-to-16-9-for-max-width",
            "ui-only-use-this-carousel-layout-if-the-items-have-various-widths",
        ],
        CarouselVariant::Hero => &[
            "ui-the-hero-layout-is-best-for-spotlighting-content-that-needs-more-attention-like-movies-sho",
            "ui-on-larger-screens-more-large-items-are-visible",
            "ui-use-snap-scrolling-so-users-can-easily-cycle-through-items-one-at-a-time",
        ],
        CarouselVariant::CenteredHero => &[
            "ui-when-the-hero-layout-is-center-aligned-it-adds-an-additional-previewed-item-on-the-leading",
        ],
        CarouselVariant::FullScreen => &[
            "ui-the-full-screen-layout-is-best-used-for-immersive-experiences-like-video-articles-featured",
            "ui-this-layout-works-best-with-content-that-is-taller-than-it-is-wide-and-scrolls-vertically",
            "ui-use-snap-scrolling-for-full-screen-layouts-don-t-use-default-scrolling",
        ],
    }
}

fn usage_caption(variant: CarouselVariant) -> &'static str {
    match variant {
        CarouselVariant::MultiBrowse => {
            "ui-a-multi-browse-layout-has-different-sized-items-within-the-carousel"
        }
        CarouselVariant::Uncontained => {
            "ui-carousel-items-are-all-the-same-size-in-an-uncontained-layout"
        }
        CarouselVariant::UncontainedMultiAspectRatio => {
            "ui-carousel-items-are-of-various-sizes-in-an-uncontained-multi-aspect-ratio-layout"
        }
        CarouselVariant::Hero => {
            "ui-a-hero-carousel-at-a-compact-breakpoint-should-show-one-large-item-and-one-small-item"
        }
        CarouselVariant::CenteredHero => {
            "ui-a-centered-hero-carousel-has-two-small-carousel-items-with-a-large-item-centered"
        }
        CarouselVariant::FullScreen => {
            "ui-full-screen-carousels-fill-the-entire-content-area-of-the-screen"
        }
    }
}

#[tessera]
fn paragraph(content: String) {
    text()
        .content(I18N.text(content))
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] content: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(I18N.text(content))
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    column().children(move || {
        for item in items.clone() {
            text()
                .content(format!("•  {}", I18N.text(item)))
                .style(type_style(16.0, 24.0));
            gap(Dp(8.0));
        }
    });
}

#[tessera]
fn legend(number: usize, #[prop(into)] label: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(24.0), Dp(24.0))
                        .background_with_shape(scheme.inverse_surface, Shape::Ellipse),
                )
                .children(move || {
                    text()
                        .content((number).to_string())
                        .style(type_style(12.0, 16.0))
                        .color(scheme.inverse_on_surface);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text()
                .content(I18N.text(&(label)))
                .style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn advice(#[prop(into)] mark: String, #[prop(into)] note: String, color: Color) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let mark = mark.clone();
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(8.0)))
                        .padding_symmetric(Dp(12.0), Dp(4.0)),
                )
                .children(move || {
                    text()
                        .content(I18N.text(&(mark)))
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text()
                .content(I18N.text(&(note)))
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0));
        });
}

fn initial_item(variant: CarouselVariant) -> usize {
    usize::from(variant == CarouselVariant::CenteredHero)
}

#[tessera]
fn collection(
    variant: CarouselVariant,
    allow_all: bool,
    #[prop(into)] header: Option<String>,
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    let controller = remember(|| CarouselController::new(initial_item(variant)));
    let all = remember(|| false);
    let selected = remember(|| 0_usize);
    let title = header
        .clone()
        .unwrap_or_else(|| "ui-all-collections".into());
    hosted_dialog(all)
        .modal_controller_optional(modal_controller)
        .full_screen(true)
        .accessibility_label(I18N.text("ui-all-carousel-collections"))
        .on_close_request(move || all.set(false))
        .main_content(move || {
            let header = header.clone();
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    if let Some(header) = header.clone() {
                        row()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .modifier(
                                Modifier::new()
                                    .fill_max_width()
                                    .padding_symmetric(Dp(16.0), Dp(0.0)),
                            )
                            .children(move || {
                                text()
                                    .content(I18N.text(&(header)))
                                    .style(type_style(22.0, 28.0));
                                spacer().modifier(Modifier::new().weight(1.0));
                                icon_button()
                                    .icon(filled::ARROW_FORWARD_SVG)
                                    .accessibility_label(I18N.format(
                                        "ui-show-all-value",
                                        [("v0", I18N.text(&header).into())],
                                    ))
                                    .on_click(move || all.set(true));
                            });
                    }
                    carousel_content(variant, controller, selected);
                    if allow_all && header.is_none() {
                        gap(Dp(4.0));
                        button()
                            .text()
                            .accessibility_label(I18N.text("ui-show-all-carousel-items"))
                            .on_click(move || all.set(true))
                            .child(move || {
                                text().content(I18N.text("ui-show-all"));
                            });
                    }
                    if selected.get() != 0 {
                        gap(Dp(8.0));
                        text()
                            .content(I18N.format(
                                "ui-collection-value-opened",
                                [("v0", (selected.get()).into())],
                            ))
                            .style(type_style(14.0, 20.0));
                    }
                });
        })
        .dialog_content(move |_| {
            all_items_page(all, selected, title.clone());
        });
}

#[tessera]
fn all_items_page(open: State<bool>, selected: State<usize>, title: String) {
    let scroll = remember(ScrollableController::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    surface()
        .modifier(Modifier::new().fill_max_size())
        .style(scheme.surface.into())
        .child(move || {
            let title = title.clone();
            column()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    top_app_bar()
                        .title(I18N.text(&(title)))
                        .container_color(scheme.surface)
                        .navigation_icon(move || {
                            icon_button()
                                .icon(filled::ARROW_BACK_SVG)
                                .accessibility_label(I18N.text("ui-back-to-carousel"))
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
                                            .content(I18N.format(
                                                "ui-collection-value-opened",
                                                [("v0", (selected.get()).into())],
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
    enabled: Option<bool>,
) {
    let enabled = enabled.unwrap_or(true);
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
                    } else if matches!(
                        variant,
                        CarouselVariant::Hero | CarouselVariant::CenteredHero
                    ) {
                        modifier.clone().width(Dp(400.0))
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
                        .user_scroll_enabled(enabled)
                        .preferred_item_width(Dp(
                            if matches!(
                                variant,
                                CarouselVariant::Hero | CarouselVariant::CenteredHero
                            ) {
                                320.0
                            } else {
                                180.0
                            },
                        ))
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
                        .accessibility_label(I18N.format(
                            "ui-value-collections",
                            [("v0", I18N.text(&name(variant)).into())],
                        ))
                        .content(move |item| {
                            let modifier = if let Some(geometry) = geometry {
                                let name = item_name(item.index);
                                let _lease = remember(move || ItemGeometryLease { geometry, name });
                                item_geometry(geometry, name)
                            } else {
                                Modifier::new()
                            };
                            media_item(item.index, selected)
                                .enabled(enabled)
                                .modifier(modifier.fill_max_size());
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
fn media_item(
    index: usize,
    selected: State<usize>,
    modifier: Option<Modifier>,
    enabled: Option<bool>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (background, foreground) = match index % 3 {
        0 => (scheme.primary_container, scheme.on_primary_container),
        1 => (scheme.secondary_container, scheme.on_secondary_container),
        _ => (scheme.tertiary_container, scheme.on_tertiary_container),
    };
    let enabled = enabled.unwrap_or(true);
    let interaction = remember(InteractionState::new);
    let pressed = interaction.with(InteractionState::is_pressed);
    let surface_color = if enabled {
        background
    } else {
        background.with_alpha(0.38)
    };
    // The button keeps its full bounds so pointer capture, semantics and the
    // ripple stay stable; only the inner surface insets and rounds on press.
    button()
        .modifier(modifier.unwrap_or_default())
        .interaction_state(interaction)
        .enabled(enabled)
        .shape(Shape::RECTANGLE)
        .padding(Dp(0.0))
        .color(Color::TRANSPARENT)
        .disabled_container_color(Color::TRANSPARENT)
        .content_color(foreground)
        .accessibility_label(I18N.format(
            "ui-open-collection-value-of-6",
            [("v0", (index + 1).into())],
        ))
        .on_click(move || selected.set(index + 1))
        .child(move || {
            let shape = if pressed {
                Shape::rounded_rectangle(Dp(28.0))
            } else {
                Shape::RECTANGLE
            };
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .padding_all(Dp(if pressed { PRESSED_INSET } else { 0.0 }))
                        .background_with_shape(surface_color, shape),
                )
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
fn layout_figure(
    variant: CarouselVariant,
    measurements: bool,
    geometry: Option<State<Geometry>>,
    controller: Option<State<CarouselController>>,
) {
    let geometry = match geometry {
        Some(geometry) => geometry,
        None => remember(Geometry::new),
    };
    let controller = match controller {
        Some(controller) => controller,
        None => remember(|| CarouselController::new(initial_item(variant))),
    };
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
                    label: Some(
                        I18N.format(
                            "ui-value-value-figure",
                            [
                                ("v0", I18N.text(&name(variant)).into()),
                                (
                                    "v1",
                                    I18N.text(
                                        &(if measurements {
                                            "ui-measurement-7071"
                                        } else {
                                            "ui-anatomy-7072"
                                        }),
                                    )
                                    .into(),
                                ),
                            ],
                        ),
                    ),
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
    if measurements {
        caption(measurement_caption(variant));
    } else {
        for (index, label) in anatomy_labels(variant).iter().enumerate() {
            legend(index + 1, *label);
        }
    }
    if measurements && variant != CarouselVariant::FullScreen {
        gap(Dp(16.0));
        paragraph("ui-these-measurements-show-padding-at-the-collection-ends-not-permanent-viewport-gutters-insp".into());
        gap(Dp(8.0));
        row().children(move || {
            button()
                .text()
                .accessibility_label(I18N.text("ui-inspect-collection-start"))
                .on_click(move || controller.with_mut(|controller| controller.jump_to_item(0)))
                .child(move || {
                    text().content(I18N.text("ui-collection-start"));
                });
            button()
                .text()
                .accessibility_label(I18N.text("ui-inspect-collection-end"))
                .on_click(move || controller.with_mut(|controller| controller.jump_to_item(5)))
                .child(move || {
                    text().content(I18N.text("ui-collection-end"));
                });
        });
    }
}

fn collection_edge_padding(values: &Geometry) -> [f64; 2] {
    let Some(container) = values.get("container") else {
        return [0.0; 2];
    };
    let start = values
        .get("item0")
        .map(|item| (item.x - container.x).to_f32())
        .unwrap_or(0.0);
    let end = values
        .get("item5")
        .map(|item| (container.x + container.width - item.x - item.width).to_f32())
        .unwrap_or(0.0);
    [start, end].map(|gap| Dp::from_pixels_f32(gap.max(0.0)).0)
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
        let [start, end] = collection_edge_padding(&values);
        for (edge, x, padding, label_x) in [
            ("start", container.x, start, container.x),
            (
                "end",
                container.x + container.w - end,
                end,
                container.x + container.w - 72.0,
            ),
        ] {
            if padding <= 0.5 {
                continue;
            }
            svg.push_str(&format!(
                "<rect x='{x}' y='{}' width='{padding}' height='{}' fill='#e883aa' fill-opacity='.32'/>",
                container.y, container.h
            ));
            dotted(
                &mut svg,
                x,
                container.y + container.h + 20.0,
                padding,
                false,
                "#b14e70",
            );
            annotations::dimension(
                I18N.format(
                    "ui-value-value-format-2",
                    [
                        ("v0", format!("{:.0}", padding).into()),
                        ("v1", I18N.text(&edge).into()),
                    ],
                ),
                Dp(label_x),
                Dp(container.y + container.h + 24.0),
            );
        }
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
        if variant == CarouselVariant::UncontainedMultiAspectRatio {
            for (index, key) in ["item0", "item1", "item2", "item3"].into_iter().enumerate() {
                let Some(item) = relative(&values, key).filter(|rect| {
                    rect.w > 0.0
                        && rect.x + rect.w / 2.0 > container.x
                        && rect.x + rect.w / 2.0 < container.x + container.w
                }) else {
                    continue;
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
                annotations::marker(
                    index + 2,
                    Dp(x - 12.0),
                    Dp(container.y + container.h + 20.0),
                );
            }
            annotations::lines(width, height, svg);
            return;
        }
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
    let rows: Vec<Vec<String>> = match variant {
        CarouselVariant::MultiBrowse => vec![
            vec!["ui-alignment".into(), "ui-vertically-centered".into()],
            vec!["ui-leading-trailing-padding".into(), "16dp".into()],
            vec!["ui-top-bottom-padding".into(), "8dp".into()],
            vec!["ui-padding-between-elements".into(), "8dp".into()],
            vec![
                "ui-large-item-width".into(),
                "ui-dynamic-or-user-set".into(),
            ],
            vec!["ui-medium-item-width".into(), "ui-dynamic".into()],
            vec!["ui-small-item-width".into(), "ui-40-56dp-dynamic".into()],
            vec!["ui-item-corner-radius".into(), "28dp".into()],
        ],
        CarouselVariant::Uncontained | CarouselVariant::UncontainedMultiAspectRatio => vec![
            vec!["ui-alignment".into(), "ui-vertically-centered".into()],
            vec!["ui-leading-padding".into(), "16dp".into()],
            vec!["ui-top-bottom-padding".into(), "8dp".into()],
            vec!["ui-padding-between-elements".into(), "8dp".into()],
            vec!["ui-item-corner-radius".into(), "28dp".into()],
        ],
        CarouselVariant::Hero | CarouselVariant::CenteredHero => vec![
            vec!["ui-alignment".into(), "ui-vertically-centered".into()],
            vec!["ui-leading-trailing-padding-6902".into(), "16dp".into()],
            vec!["ui-top-bottom-padding".into(), "8dp".into()],
            vec!["ui-padding-between-elements".into(), "8dp".into()],
            vec!["ui-large-item-width".into(), "ui-dynamic".into()],
            vec![
                "ui-small-item-width".into(),
                "ui-40-56dp-dynamic-1449".into(),
            ],
            vec!["ui-item-corner-radius".into(), "28dp".into()],
        ],
        CarouselVariant::FullScreen => vec![
            vec!["ui-alignment".into(), "ui-centered".into()],
            vec!["ui-leading-trailing-padding-6902".into(), "0dp".into()],
            vec!["ui-top-bottom-padding".into(), "0dp".into()],
            vec!["ui-padding-between-elements".into(), "16dp".into()],
        ],
    };
    spec_table(vec!["ui-attribute".into(), "ui-value".into()], rows);
}

#[tessera]
fn overview_intro(
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    bullets(vec![
        "ui-contain-visual-items-like-images-or-video-along-with-optional-label-text".into(),
        "ui-six-layouts-multi-browse-uncontained-uncontained-multi-aspect-ratio-hero-center-aligned-he".into(),
        "ui-layouts-can-be-start-aligned-or-center-aligned".into(),
        "ui-item-visuals-have-a-parallax-effect-when-scrolled".into(),
        "ui-items-change-size-as-they-move-through-the-carousel".into(),
    ]);
    gap(Dp(32.0));
    collection(CarouselVariant::MultiBrowse, true).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    caption("ui-carousels-can-show-items-of-various-sizes");
}

#[tessera]
fn availability() {
    spec_table(
        vec!["ui-type".into(), "ui-resource".into(), "ui-status".into()],
        vec![
            vec![
                "ui-design".into(),
                "ui-design-kit-figma".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Flutter".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Jetpack Compose".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Android Views (MDC-Android)".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Web".into(),
                "ui-unavailable".into(),
            ],
        ],
    );
}

#[tessera]
fn updates(
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    subheading("ui-november-2025".into());
    gap(Dp(8.0));
    paragraph("ui-new-carousel-layout".into());
    bullets(vec![
        "ui-uncontained-multi-aspect-ratio-shows-carousel-items-of-various-widths".into(),
    ]);
    gap(Dp(24.0));
    collection(CarouselVariant::UncontainedMultiAspectRatio, false)
        .modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    caption("ui-new-carousel-layout-uncontained-multi-aspect-ratio");
    gap(Dp(32.0));
    subheading("2023".into());
    gap(Dp(8.0));
    paragraph("ui-additional-layouts-and-configurations".into());
    bullets(vec![
        "ui-uncontained-items-scroll-to-the-edge-of-the-container".into(),
        "ui-full-screen-one-edge-to-edge-large-item-at-a-time-scrolling-vertically".into(),
        "ui-centered-carousels-the-center-aligned-hero-layout-shows-at-least-one-large-and-two-small-i".into(),
        "ui-hero-carousel-layout-at-least-one-large-and-one-small-item-at-a-time".into(),
        "ui-multi-browse-layout-at-least-one-large-medium-and-small-carousel-item-at-a-time".into(),
    ]);
}

#[tessera]
fn differences(
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    paragraph("ui-this-component-is-new-in-material-3".into());
    gap(Dp(16.0));
    bullets(vec![
        "ui-shape-dynamic-carousel-items-change-shape-when-scrolled".into(),
        "ui-motion-carousel-items-move-at-a-different-speed-than-their-content-creating-a-parallax-eff".into(),
        "ui-interaction-when-scrolled-carousel-items-snap-into-place-to-maintain-the-same-layout-hero".into(),
    ]);
    gap(Dp(24.0));
    collection(CarouselVariant::Hero, false).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    caption("ui-hero-carousels-scroll-through-one-large-item-at-a-time");
}

#[tessera]
fn research() {
    paragraph("ui-the-material-research-team-conducted-two-studies-quantitative-and-qualitative-with-over-20".into());
    gap(Dp(16.0));
    paragraph("ui-summary-of-findings".into());
    bullets(vec![
        "ui-participants-thought-carousels-were-a-good-way-to-explore-many-different-kinds-of-content".into(),
        "ui-a-previewed-or-squished-item-strongly-indicated-that-there-was-more-content-to-swipe-throu".into(),
        "ui-participants-expected-around-10-items-in-a-carousel-that-scrolled-multiple-items-at-once".into(),
        "ui-while-some-contexts-were-considered-better-for-some-carousel-designs-all-designs-were-cons".into(),
    ]);
}

#[tessera]
fn tokens() {
    paragraph("ui-browse-the-component-elements-attributes-tokens-and-their-values".into());
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-state".into(),
            "ui-carousel-item-token".into(),
            "ui-value".into(),
        ],
        vec![
            vec![
                "ui-enabled".into(),
                "ui-outline-color".into(),
                "Outline".into(),
            ],
            vec![
                "ui-enabled".into(),
                "ui-container-color".into(),
                "Surface".into(),
            ],
            vec![
                "ui-enabled".into(),
                "ui-container-shape".into(),
                "ui-extra-large-28dp".into(),
            ],
            vec![
                "ui-hover".into(),
                "ui-state-layer-color".into(),
                "On surface".into(),
            ],
            vec![
                "ui-hover".into(),
                "ui-state-layer-opacity".into(),
                "0.08".into(),
            ],
            vec![
                "ui-hover".into(),
                "ui-outline-color".into(),
                "Outline".into(),
            ],
            vec![
                "ui-focus".into(),
                "ui-focus-indicator-color".into(),
                "Secondary".into(),
            ],
            vec![
                "ui-focus".into(),
                "ui-focus-indicator-thickness".into(),
                "3dp".into(),
            ],
            vec![
                "ui-focus".into(),
                "ui-focus-indicator-offset".into(),
                "2dp".into(),
            ],
            vec![
                "ui-focus".into(),
                "ui-outline-color".into(),
                "On surface".into(),
            ],
            vec![
                "ui-focus".into(),
                "ui-state-layer-color".into(),
                "On surface".into(),
            ],
            vec![
                "ui-focus".into(),
                "ui-state-layer-opacity".into(),
                "0.1".into(),
            ],
            vec![
                "ui-pressed-ripple".into(),
                "ui-outline-color".into(),
                "Outline".into(),
            ],
            vec![
                "ui-pressed-ripple".into(),
                "ui-state-layer-color".into(),
                "On surface".into(),
            ],
            vec![
                "ui-pressed-ripple".into(),
                "ui-state-layer-opacity".into(),
                "0.1".into(),
            ],
            vec![
                "ui-disabled".into(),
                "ui-container-color".into(),
                "Surface".into(),
            ],
            vec![
                "ui-disabled".into(),
                "ui-container-opacity".into(),
                "0.38".into(),
            ],
        ],
    );
}

#[tessera]
fn colors() {
    paragraph("ui-color-values-are-implemented-through-design-tokens-for-design-this-means-working-with-colo".into());
    gap(Dp(32.0));
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    merge_descendants: true,
                    label: Some(
                        I18N.text("ui-2-color-roles-of-a-carousel-in-light-and-dark-schemes"),
                    ),
                    ..Default::default()
                }),
        )
        .children(|| {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(|| {
                    for dark in [false, true] {
                        material_theme()
                            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                            .child(move || {
                                color_half(dark);
                            });
                    }
                });
        });
    gap(Dp(16.0));
    caption("ui-carousel-color-roles-used-for-light-and-dark-schemes");
    gap(Dp(8.0));
    legend(1, "ui-container");
    legend(2, "Surface");
}

#[tessera]
fn color_half(dark: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let stroke = if dark { "#cac4d0" } else { "#49454f" };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(248.0))
                .background(scheme.surface),
        )
        .children(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().size(Dp(384.0), Dp(232.0)))
                .children(move || {
                    let mut svg = String::new();
                    dotted(&mut svg, 64.0, 40.0, 16.0, false, stroke);
                    dotted(&mut svg, 158.0, 200.0, 12.0, true, stroke);
                    annotations::lines(Dp(384.0), Dp(232.0), svg);
                    annotations::marker(1, Dp(40.0), Dp(24.0));
                    annotations::marker(2, Dp(146.0), Dp(200.0));
                    row()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(80.0), Dp(32.0))
                                .size(Dp(304.0), Dp(168.0))
                                .border_with_shape(Dp(1.0), scheme.outline, Shape::RECTANGLE)
                                .padding_all(Dp(8.0)),
                        )
                        .children(move || {
                            for (index, width) in [148.0, 100.0, 40.0].into_iter().enumerate() {
                                if index > 0 {
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                }
                                spacer().modifier(
                                    Modifier::new()
                                        .size(Dp(width), Dp(152.0))
                                        .background_with_shape(
                                            scheme.surface,
                                            Shape::rounded_rectangle(Dp(28.0)),
                                        )
                                        .border_with_shape(
                                            Dp(1.0),
                                            scheme.outline,
                                            Shape::rounded_rectangle(Dp(28.0)),
                                        ),
                                );
                            }
                        });
                });
        });
}

const STATES: [&str; 5] = [
    "ui-enabled",
    "ui-hovered",
    "ui-focused",
    "ui-pressed",
    "ui-disabled",
];

#[tessera]
fn states(
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    paragraph("ui-states-are-visual-representations-used-to-communicate-the-status-of-a-component-or-interac".into());
    gap(Dp(32.0));
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    merge_descendants: true,
                    label: Some(I18N.text("ui-5-states-of-a-carousel-in-light-and-dark-schemes")),
                    ..Default::default()
                }),
        )
        .children(|| {
            for dark in [false, true] {
                material_theme()
                    .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                    .child(|| {
                        state_row();
                    });
            }
        });
    gap(Dp(16.0));
    for (index, label) in STATES.into_iter().enumerate() {
        legend(index + 1, label);
    }
    gap(Dp(32.0));
    paragraph("ui-the-live-collection-below-uses-the-real-state-treatment-of-its-interactive-items-hover-foc".into());
    gap(Dp(16.0));
    collection(CarouselVariant::MultiBrowse, true).modal_controller_optional(modal_controller);
    gap(Dp(24.0));
    caption("ui-a-disabled-collection-blocks-activation-and-scrolling");
    gap(Dp(16.0));
    let controller = remember(CarouselController::default);
    let selected = remember(|| 0_usize);
    carousel_content(CarouselVariant::MultiBrowse, controller, selected).enabled(false);
}

#[tessera]
fn state_row() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background(scheme.surface_container_low)
                .padding_symmetric(Dp(8.0), Dp(24.0)),
        )
        .children(|| {
            for (index, label) in STATES.into_iter().enumerate() {
                state_tile(index, label.to_string());
            }
        });
}

#[tessera]
fn state_tile(index: usize, label: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let disabled = index == 4;
    let content = if disabled {
        scheme.on_primary_container.with_alpha(0.38)
    } else {
        scheme.on_primary_container
    };
    let background = if disabled {
        scheme.primary_container.with_alpha(0.38)
    } else {
        scheme.primary_container
    };
    let layer = match index {
        1 => scheme.on_surface.with_alpha(0.08),
        2 | 3 => scheme.on_surface.with_alpha(0.1),
        _ => Color::TRANSPARENT,
    };
    let ring = if index == 2 {
        scheme.secondary
    } else {
        Color::TRANSPARENT
    };
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            let label = label.clone();
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(24.0), Dp(24.0))
                        .background_with_shape(scheme.inverse_surface, Shape::Ellipse),
                )
                .children(move || {
                    text()
                        .content((index + 1).to_string())
                        .style(type_style(12.0, 16.0))
                        .color(scheme.inverse_on_surface);
                });
            gap(Dp(12.0));
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(100.0), Dp(128.0))
                        .border_with_shape(Dp(3.0), ring, Shape::rounded_rectangle(Dp(30.0))),
                )
                .children(move || {
                    let label = label.clone();
                    boxed()
                        .alignment(Alignment::BottomStart)
                        .modifier(
                            Modifier::new()
                                .size(Dp(90.0), Dp(118.0))
                                .background_with_shape(
                                    background,
                                    Shape::rounded_rectangle(Dp(28.0)),
                                )
                                .background_with_shape(layer, Shape::rounded_rectangle(Dp(28.0)))
                                .padding_all(Dp(12.0)),
                        )
                        .children(move || {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(Modifier::new().fill_max_size())
                                .children(move || {
                                    icon()
                                        .vector(filled::LANDSCAPE_SVG)
                                        .tint(content)
                                        .size(Dp(32.0));
                                });
                            text()
                                .content(I18N.text(&(label)))
                                .color(content)
                                .style(type_style(12.0, 16.0));
                        });
                });
        });
}

#[tessera]
fn dynamic_widths() {
    paragraph(
        "ui-all-kinds-of-carousel-items-dynamically-adapt-to-the-width-of-the-container".into(),
    );
    gap(Dp(16.0));
    paragraph("ui-large-items-have-a-customizable-maximum-width-that-s-used-to-optimally-fit-carousel-items".into());
    gap(Dp(16.0));
    paragraph(
        "ui-small-carousel-items-have-a-minimum-width-of-40dp-and-a-maximum-width-of-56dp".into(),
    );
    gap(Dp(16.0));
    paragraph("ui-items-change-size-as-they-move-through-the-carousel-layout".into());
    gap(Dp(32.0));
    panel(RenderSlot::new(move || {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
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
    gap(Dp(16.0));
    caption("ui-small-carousel-items-have-a-minimum-and-maximum-width");
}

#[tessera]
fn guidelines_intro(
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    collection(CarouselVariant::MultiBrowse, false).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    caption("ui-carousel-items-adapt-dynamically-based-on-breakpoint");
}

#[tessera]
fn usage(
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    paragraph("ui-carousels-display-a-scrollable-list-of-items-carousel-items-emphasize-visuals-but-can-also".into());
    gap(Dp(16.0));
    paragraph("ui-there-are-four-carousel-layouts".into());
    bullets(vec![
        "ui-multi-browse".into(),
        "ui-uncontained".into(),
        "ui-hero".into(),
        "ui-full-screen-7087".into(),
    ]);
    paragraph("ui-all-of-these-layouts-can-be-centered-though-center-aligned-hero-is-the-most-common-centere".into());
    gap(Dp(16.0));
    paragraph("ui-carousels-can-scroll-in-two-ways".into());
    bullets(vec![
        "ui-default-standard-scrolling-recommended-for-uncontained-layouts".into(),
        "ui-snap-scrolling-scrolled-items-snap-to-the-carousel-s-layout-recommended-for-multi-browse-h".into(),
    ]);
    gap(Dp(24.0));
    collection(CarouselVariant::MultiBrowse, false).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    caption("ui-a-carousel-can-contain-different-sized-items");
    gap(Dp(32.0));
    paragraph("ui-choose-the-best-carousel-layout-for-your-product-some-layouts-are-more-visual-focused-whil".into());
    gap(Dp(16.0));
    spec_table(
        vec!["ui-layout".into(), "ui-best-used-for".into()],
        vec![
            vec![
                "ui-multi-browse".into(),
                "ui-browsing-many-visual-items-at-once-like-photos-dynamic-designs".into(),
            ],
            vec![
                "ui-uncontained".into(),
                "ui-highly-customized-or-text-heavy-carousels-stacked-image-and-text-traditional-carousel-beha".into(),
            ],
            vec![
                "ui-hero".into(),
                "ui-spotlighting-very-large-visual-items-like-a-movie-or-featured-app".into(),
            ],
            vec!["ui-center-aligned-hero".into(), "ui-centered-large-visual-items".into()],
            vec![
                "ui-full-screen-7087".into(),
                "ui-vertically-scrolling-video-or-image-feeds-immersive-experiences".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    paragraph("ui-carousel-items-must-be-fully-visible-on-screen-except-for-the-uncontained-layout-when-scro".into());
    gap(Dp(16.0));
    advice(
        "Do",
        "ui-set-the-large-carousel-item-size-to-ensure-the-images-and-text-are-easy-to-read-and-recogn",
        DO_COLOR,
    );
    gap(Dp(12.0));
    advice(
        "Don't",
        "ui-avoid-setting-carousel-items-so-small-that-the-image-isn-t-recognizable",
        DONT_COLOR,
    );
    gap(Dp(48.0));
    subheading("ui-accessibility-requirements-on-scrolling-pages".into());
    gap(Dp(16.0));
    paragraph("ui-on-vertically-scrolling-pages-carousels-require-an-accessible-way-to-view-all-the-items-wi".into());
    gap(Dp(16.0));
    paragraph("ui-material-recommends-adding-a-show-all-button-below-the-carousel-which-opens-a-dedicated-ve".into());
    gap(Dp(16.0));
    paragraph("ui-if-the-carousel-has-a-header-you-can-use-an-arrow-icon-button-instead".into());
    gap(Dp(16.0));
    paragraph("ui-view-the-accessibility-tab-for-more-details-and-alternate-solutions".into());
    gap(Dp(24.0));
    collection(CarouselVariant::MultiBrowse, true).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    caption("ui-make-sure-users-can-scroll-vertically-through-all-carousel-items");
    for variant in VARIANTS {
        layout_usage(variant).modal_controller_optional(modal_controller);
    }
}

#[tessera]
fn layout_usage(
    variant: CarouselVariant,
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    gap(Dp(48.0));
    subheading(name(variant).into());
    gap(Dp(16.0));
    for line in guidance(variant) {
        paragraph((*line).into());
        gap(Dp(16.0));
    }
    gap(Dp(8.0));
    collection(variant, false).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    caption(usage_caption(variant));
    if variant == CarouselVariant::MultiBrowse {
        gap(Dp(32.0));
        paragraph(
            "ui-at-compact-breakpoints-only-show-up-to-three-carousel-items-if-they-have-text"
                .into(),
        );
        gap(Dp(16.0));
        paragraph("ui-if-you-need-to-show-more-than-three-items-make-sure-the-images-and-content-are-easy-to-und".into());
        gap(Dp(16.0));
        advice(
            "ui-caution",
            "ui-at-compact-breakpoints-only-show-more-than-three-items-if-the-items-are-easy-to-understand",
            CAUTION_COLOR,
        );
    }
}

#[tessera]
fn anatomy() {
    layout_figure(CarouselVariant::MultiBrowse, false);
    gap(Dp(16.0));
    caption("ui-multi-browse-carousel-left-and-hero-carousel-right-the-hero-layout-is-shown-below");
    gap(Dp(16.0));
    layout_figure(CarouselVariant::Hero, false);
    gap(Dp(48.0));
    subheading("ui-container".into());
    gap(Dp(16.0));
    paragraph("ui-the-carousel-container-holds-all-carousel-items-the-number-of-carousel-items-visible-in-th".into());
    gap(Dp(16.0));
    paragraph("ui-the-carousel-container-is-a-rectangle-it-can-be-used-in-many-ways-and-stretched-to-any-siz".into());
    gap(Dp(48.0));
    subheading("ui-carousel-items".into());
    gap(Dp(16.0));
    paragraph("ui-carousel-items-hold-content-a-carousel-item-has-no-fixed-width-it-changes-width-depending".into());
    gap(Dp(16.0));
    paragraph("ui-large-items-have-a-max-width-that-can-be-adjusted-this-changes-how-all-other-large-medium".into());
    gap(Dp(16.0));
    paragraph(
        "ui-medium-items-adjust-width-dynamically-to-the-carousel-size-and-available-space".into(),
    );
    gap(Dp(16.0));
    paragraph("ui-small-items-have-a-width-range-of-40-56dp".into());
    gap(Dp(16.0));
    paragraph("ui-when-the-large-item-s-max-width-is-narrow-enough-more-items-can-be-shown-on-screen-at-once".into());
    gap(Dp(48.0));
    subheading("ui-item-text-optional".into());
    gap(Dp(16.0));
    paragraph(
        "ui-carousel-items-are-primarily-visual-item-text-is-optional-and-should-be-brief".into(),
    );
    gap(Dp(16.0));
    paragraph("ui-if-your-carousel-items-require-a-lot-of-text-consider-using-the-uncontained-layout-which-c".into());
    gap(Dp(16.0));
    advice(
        "ui-caution",
        "ui-avoid-exceeding-two-lines-of-text-in-carousel-items-at-compact-breakpoints-unless-the-back",
        CAUTION_COLOR,
    );
    gap(Dp(24.0));
    paragraph("ui-content-within-a-carousel-item-can-adapt-dynamically-based-on-the-container-and-breakpoint".into());
    gap(Dp(16.0));
    paragraph("ui-text-should-always-be-understandable-at-each-size-consider-adapting-the-text-to-use-brief".into());
}

#[tessera]
fn adaptive_design(
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    paragraph("ui-as-the-carousel-container-size-increases-so-do-the-number-of-carousel-items-visible-at-a-t".into());
    gap(Dp(16.0));
    bullets(vec![
        "ui-at-compact-breakpoints-carousels-can-comfortably-show-up-to-three-carousel-items-at-once".into(),
        "ui-at-larger-breakpoints-more-carousel-items-are-added-and-scale-in-size".into(),
        "ui-full-screen-carousels-only-ever-show-one-item-at-once".into(),
    ]);
    gap(Dp(24.0));
    collection(CarouselVariant::MultiBrowse, false).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    caption(
        "ui-carousels-can-show-more-items-at-once-on-larger-screens-resize-the-window-to-change-how-ma",
    );
}

#[tessera]
fn behavior(
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    subheading("ui-scrolling".into());
    gap(Dp(16.0));
    paragraph("ui-carousel-items-have-a-parallax-effect-when-they-re-scrolled".into());
    gap(Dp(16.0));
    paragraph("ui-the-default-scrolling-behavior-doesn-t-snap-items-to-a-layout-grid-they-can-stop-anywhere".into());
    gap(Dp(16.0));
    paragraph("ui-the-snap-scrolling-behavior-aligns-carousel-items-to-the-layout-grid-after-they-re-scrolle".into());
    gap(Dp(24.0));
    collection(CarouselVariant::Hero, false).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    caption("ui-carousel-items-with-snap-scrolling-maintain-the-same-layout-when-released");
    gap(Dp(32.0));
    paragraph("ui-in-full-screen-layouts-make-sure-the-carousel-items-snap-to-the-edges-of-the-carousel-cont".into());
    gap(Dp(16.0));
    paragraph("ui-full-screen-layouts-must-use-snap-scrolling".into());
    gap(Dp(24.0));
    collection(CarouselVariant::FullScreen, false).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    advice(
        "Don't",
        "ui-avoid-scrolling-freely-on-full-screen-carousels",
        DONT_COLOR,
    );
}

#[tessera]
fn use_cases() {
    paragraph("ui-users-should-be-able-to-do-the-following-with-assistive-technology".into());
    bullets(vec![
        "ui-navigate-to-the-carousel-container".into(),
        "ui-navigate-between-different-carousel-items".into(),
        "ui-activate-a-carousel-item".into(),
        "ui-skip-over-the-carousel-items".into(),
    ]);
}

#[tessera]
fn scrolling_requirements(
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    paragraph("ui-on-vertically-scrolling-pages-carousels-require-an-accessible-way-to-view-all-the-items-wi".into());
    gap(Dp(16.0));
    paragraph("ui-material-recommends-adding-a-show-all-button-below-the-carousel-which-opens-a-dedicated-ve".into());
    gap(Dp(24.0));
    collection(CarouselVariant::MultiBrowse, true).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    caption(
        "ui-carousels-without-headers-should-use-a-show-all-button-to-view-all-carousel-items-the-show",
    );
    gap(Dp(32.0));
    paragraph("ui-if-the-carousel-has-a-header-you-can-use-an-arrow-icon-button-instead-place-the-arrow-icon".into());
    gap(Dp(16.0));
    paragraph("ui-make-sure-the-header-is-also-displayed-on-the-page-of-all-carousel-items".into());
    gap(Dp(24.0));
    collection(CarouselVariant::MultiBrowse, true)
        .modal_controller_optional(modal_controller)
        .header(I18N.text("ui-recent-collections"));
    gap(Dp(16.0));
    caption(
        "ui-carousels-with-headers-should-use-an-arrow-to-view-all-carousel-items-headers-should-align",
    );
    gap(Dp(32.0));
    paragraph("ui-avoid-customizing-the-accessibility-solution-when-possible-however-if-your-product-needs-a".into());
    gap(Dp(16.0));
    paragraph(
        "ui-avoid-adding-ui-elements-like-arrows-or-other-icons-within-or-beside-the-carousel"
            .into(),
    );
    gap(Dp(16.0));
    advice(
        "Don't",
        "ui-avoid-adding-buttons-into-the-carousel-container-or-beside-it-place-any-buttons-above-or-b",
        DONT_COLOR,
    );
    gap(Dp(12.0));
    advice(
        "Don't",
        "ui-don-t-cover-the-carousel-with-buttons-or-other-ui",
        DONT_COLOR,
    );
}

#[tessera]
fn interaction_style(
    modal_controller: Option<
        crate::pages::material_spec::article::modal_host::ArticleModalController,
    >,
) {
    subheading("ui-touch".into());
    gap(Dp(16.0));
    paragraph("ui-tapping-on-a-carousel-item-changes-the-shape-slightly-and-creates-a-touch-ripple-for-inter".into());
    gap(Dp(16.0));
    caption(
        "ui-press-any-live-item-on-this-page-while-pressed-its-surface-insets-4dp-inside-the-carousel",
    );
    gap(Dp(32.0));
    subheading("ui-cursor".into());
    gap(Dp(16.0));
    paragraph(
        "ui-the-hover-state-provides-a-visual-cue-that-the-carousel-item-is-interactive".into(),
    );
    gap(Dp(16.0));
    paragraph("ui-when-the-carousel-item-is-clicked-in-both-active-and-inactive-states-a-ripple-appears-for".into());
    gap(Dp(32.0));
    subheading("ui-initial-focus".into());
    gap(Dp(16.0));
    paragraph("ui-when-navigating-to-a-carousel-using-assistive-technology-use-tab-to-place-initial-focus-on".into());
    gap(Dp(16.0));
    paragraph("ui-use-the-up-and-down-arrow-keys-to-leave-the-carousel-and-focus-on-the-next-element-on-the".into());
    gap(Dp(24.0));
    collection(CarouselVariant::MultiBrowse, true).modal_controller_optional(modal_controller);
    gap(Dp(16.0));
    advice(
        "Do",
        "ui-set-initial-focus-on-the-first-carousel-item-and-use-arrows-to-navigate-items",
        DO_COLOR,
    );
    gap(Dp(12.0));
    advice(
        "Don't",
        "ui-avoid-focusing-on-the-carousel-container",
        DONT_COLOR,
    );
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-tab-or-arrows".into(),
                "ui-moves-to-the-previous-or-next-carousel-item".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activates-the-focused-carousel-item".into(),
            ],
        ],
    );
}

#[tessera]
fn labeling() {
    paragraph("ui-the-carousel-container-has-the-container-role".into());
    gap(Dp(16.0));
    caption("ui-the-carousel-container-is-labelled-appropriately-and-has-the-container-role");
    gap(Dp(32.0));
    paragraph("ui-each-carousel-may-have-a-different-number-of-items-so-the-label-reads-out-the-total-amount".into());
    gap(Dp(16.0));
    caption(
        "ui-the-carousel-item-label-indicates-the-current-item-in-focus-and-the-total-number-of-items",
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        accesskit::{Action, ActionRequest, TreeId},
    };

    use super::*;

    type Observed = (State<Geometry>, State<CarouselController>, State<usize>);

    #[tessera]
    fn fixture(
        variant: CarouselVariant,
        enabled: bool,
        output: tessera_ui::testing::TestProbe<Option<Observed>>,
    ) {
        material_theme().child(move || {
            let controller = remember(|| CarouselController::new(initial_item(variant)));
            let selected = remember(|| 0_usize);
            let geometry = remember(Geometry::new);
            output.set(Some((geometry, controller, selected)));
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    column()
                        .modifier(Modifier::new().width(Dp(
                            if matches!(
                                variant,
                                CarouselVariant::Hero | CarouselVariant::CenteredHero
                            ) {
                                720.0
                            } else {
                                400.0
                            },
                        )))
                        .children(move || {
                            carousel_content(variant, controller, selected)
                                .geometry(geometry)
                                .enabled(enabled);
                        });
                });
        });
    }

    #[test]
    #[ignore = "Requires a GPU adapter for the real component pipelines"]
    fn disabled_carousel_preview_blocks_activation_and_scrolling() {
        for enabled in [false, true] {
            let output = tessera_ui::testing::TestProbe::default();
            let probe = output.clone();
            let mut renderer = HeadlessRenderer::new(
                Box::new(move || fixture(CarouselVariant::MultiBrowse, enabled, probe.clone())),
                vec![Box::new(crate::TestPipelines)],
                HeadlessConfig {
                    width: 800,
                    height: 600,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .expect("headless carousel renderer");
            renderer.render_frames(8);
            let tree = renderer
                .accessibility_snapshot()
                .expect("carousel semantics");
            let item = tree
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Open collection 1 of 6"))
                .expect("first collection item");
            assert_eq!(item.1.is_disabled(), !enabled);
            let bounds = item.1.bounds().expect("visible first item");
            let point = PxPosition::from_f32_xy(
                ((bounds.x0 + bounds.x1) * 0.5) as f32,
                ((bounds.y0 + bounds.y1) * 0.5) as f32,
            );
            renderer.pointer_move(MOUSE_POINTER_ID, point);
            renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.step_frame();
            renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.render_frames(4);
            assert_eq!(output.get().unwrap().2.get(), usize::from(enabled));
            renderer.pointer_scroll(MOUSE_POINTER_ID, point, -120.0, 0.0);
            renderer.render_frames(120);
            let offset = output
                .get()
                .unwrap()
                .1
                .with(CarouselController::scroll_offset);
            if enabled {
                assert!(offset.0 > 0.0, "enabled preview must respond to scrolling");
            } else {
                assert_eq!(offset, Dp(0.0), "disabled preview must not scroll");
            }
        }
    }

    #[tessera]
    fn measurement_fixture(output: tessera_ui::testing::TestProbe<Option<Observed>>) {
        let geometry = remember(Geometry::new);
        let controller = remember(|| CarouselController::new(0));
        let selected = remember(|| 0usize);
        output.set(Some((geometry, controller, selected)));
        material_theme().child(move || {
            column()
                .modifier(Modifier::new().width(Dp(496.0)))
                .children(move || {
                    layout_figure(CarouselVariant::MultiBrowse, true)
                        .geometry(geometry)
                        .controller(controller);
                });
        });
    }

    #[test]
    #[ignore = "Requires a GPU adapter for real carousel measurement annotations and actions"]
    fn measurement_annotations_follow_collection_edges_and_end_inspection_action() {
        let output = tessera_ui::testing::TestProbe::default();
        let probe = output.clone();
        let mut renderer = HeadlessRenderer::new(
            Box::new(move || measurement_fixture(probe.clone())),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 800,
                height: 700,
                sample_count: 1,
                ..Default::default()
            },
        )
        .expect("measurement renderer");
        renderer.render_frames(8);
        let geometry = output.get().unwrap().0;
        assert_eq!(collection_edge_padding(&geometry.get()), [16.0, 0.0]);
        let tree = renderer.accessibility_snapshot().unwrap();
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Dimension 16 start"))
        );
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Dimension 16 end"))
        );
        let target_node = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Inspect collection end"))
            .unwrap()
            .0;
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node,
            data: None
        }));
        renderer.render_frames(8);
        assert_eq!(
            output
                .get()
                .unwrap()
                .1
                .with(CarouselController::current_item),
            5
        );
        assert_eq!(collection_edge_padding(&geometry.get()), [0.0, 16.0]);
        let tree = renderer.accessibility_snapshot().unwrap();
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Dimension 16 end"))
        );
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Dimension 16 start"))
        );
    }

    fn assert_hero_geometry(values: &Geometry, variant: CarouselVariant, target: usize) {
        let container = values.get("container").unwrap();
        let large = values.get(item_name(target)).expect("hero item visible");
        let items: Vec<_> = values
            .iter()
            .filter(|(name, _)| name.starts_with("item"))
            .map(|(_, rect)| rect)
            .collect();
        assert!(
            large.width > Dp(240.0).to_px(),
            "hero remains the dominant item: {large:?}"
        );
        assert_eq!(
            items
                .iter()
                .filter(|item| item.width > Dp(180.0).to_px())
                .count(),
            1,
            "exactly one large item in the controlled hero viewport: {values:?}"
        );
        assert!(
            items
                .iter()
                .any(|item| item.width > tessera_ui::Px::ZERO && item.width <= Dp(56.0).to_px()),
            "hero retains a small preview: {values:?}"
        );
        if variant == CarouselVariant::CenteredHero {
            assert!(
                ((large.x + large.width / 2) - (container.x + container.width / 2))
                    .0
                    .abs()
                    <= 1,
                "large item is centered between previews: {values:?}"
            );
            assert!(items.iter().any(|item| item.x < large.x));
            assert!(items.iter().any(|item| item.x > large.x));
        }
    }

    #[test]
    #[ignore = "Requires a GPU adapter for the real component pipelines"]
    fn live_carousel_figures_keep_horizontal_and_portrait_viewports() {
        for variant in VARIANTS {
            let output = tessera_ui::testing::TestProbe::default();
            let probe = output.clone();
            let mut renderer = HeadlessRenderer::new(
                Box::new(move || fixture(variant, true, probe.clone())),
                vec![Box::new(crate::TestPipelines)],
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
            let geometry = output.get().unwrap().0;
            let controller = output.get().unwrap().1;
            if matches!(
                variant,
                CarouselVariant::Hero | CarouselVariant::CenteredHero
            ) {
                assert_hero_geometry(&geometry.get(), variant, initial_item(variant));
            }
            for target in [5, 0, 3] {
                controller.with_mut(|controller| controller.jump_to_item(target));
                renderer.render_frames(8);
                let values = geometry.get();
                let container = values.get("container").expect("container geometry");
                if variant == CarouselVariant::MultiBrowse {
                    if target == 0 {
                        assert_eq!(collection_edge_padding(&values), [16.0, 0.0]);
                    }
                    if target == 5 {
                        assert_eq!(collection_edge_padding(&values), [0.0, 16.0]);
                    }
                }
                if target == 3
                    && matches!(
                        variant,
                        CarouselVariant::Hero | CarouselVariant::CenteredHero
                    )
                {
                    assert_hero_geometry(&values, variant, target);
                }
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

    #[tessera]
    fn header_fixture() {
        material_theme().child(|| {
            column()
                .modifier(Modifier::new().width(Dp(400.0)))
                .children(|| {
                    collection(CarouselVariant::MultiBrowse, true).header("Recent collections");
                });
        });
    }

    fn labeled<'a>(
        tree: &'a tessera_ui::accesskit::TreeUpdate,
        label: &str,
    ) -> Option<(
        tessera_ui::accesskit::NodeId,
        &'a tessera_ui::accesskit::Node,
    )> {
        tree.nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .map(|(id, node)| (*id, node))
    }

    #[test]
    #[ignore = "Requires a GPU adapter for the real carousel header and full-screen page"]
    fn header_arrow_replaces_show_all_and_opens_a_page_with_the_same_header() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(header_fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 800,
                height: 700,
                sample_count: 1,
                ..Default::default()
            },
        )
        .expect("header renderer");
        renderer.render_frames(8);
        let tree = renderer.accessibility_snapshot().expect("header tree");
        assert!(
            labeled(&tree, "Show all carousel items").is_none(),
            "a carousel with a header uses the arrow instead of a Show all button"
        );
        let (_, title) = labeled(&tree, "Recent collections").expect("visible header");
        let (arrow_id, arrow) = labeled(&tree, "Show all Recent collections").expect("arrow");
        let (_, first) = labeled(&tree, "Open collection 1 of 6").expect("first item");
        let title = title.bounds().expect("header bounds");
        let arrow = arrow.bounds().expect("arrow bounds");
        let first = first.bounds().expect("first item bounds");
        assert!(
            (title.x0 - first.x0).abs() <= 1.0,
            "header aligns with the leading item edge: {title:?} vs {first:?}"
        );
        assert!(
            arrow.width() >= 48.0 && arrow.height() >= 48.0,
            "arrow keeps a 48dp target: {arrow:?}"
        );
        assert!(
            arrow.y1 <= first.y0,
            "the arrow stays outside the carousel container: {arrow:?} vs {first:?}"
        );
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: arrow_id,
            data: None
        }));
        renderer.render_frames(12);
        let tree = renderer.accessibility_snapshot().expect("all items tree");
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Dialog
                    && node.label() == Some("All carousel collections")),
            "the arrow opens the all-items page"
        );
        assert!(
            tree.nodes
                .iter()
                .filter(|(_, node)| node.label() == Some("Recent collections"))
                .count()
                >= 1,
            "the all-items page repeats the carousel header"
        );
        assert!(labeled(&tree, "Open collection 6 of 6").is_some());
        let (back, _) = labeled(&tree, "Back to carousel").expect("back control");
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: back,
            data: None
        }));
        renderer.render_frames(24);
        let tree = renderer.accessibility_snapshot().expect("restored tree");
        assert!(labeled(&tree, "Back to carousel").is_none());
        assert!(labeled(&tree, "Show all Recent collections").is_some());
    }
}
