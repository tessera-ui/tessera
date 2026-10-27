//! Interactive card specifications by container style.
//!
//! ## Usage
//!
//! Compare related-content containers, emphasis and live interaction feedback.

use tessera_components::{
    alignment::{Alignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    card::{CardVariant, card},
    carousel::{CarouselVariant, carousel},
    column::column,
    flow_row::flow_row,
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Dp, Modifier, RenderSlot, State, remember, tessera, use_context};

use super::{
    ACCENT,
    annotations::{self, Geometry, SpaceGeometry, dotted, relative, watch, watch_in_space},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const VARIANTS: [CardVariant; 3] = [
    CardVariant::Elevated,
    CardVariant::Filled,
    CardVariant::Outlined,
];

fn name(variant: CardVariant) -> &'static str {
    match variant {
        CardVariant::Elevated => "Elevated",
        CardVariant::Filled => "Filled",
        CardVariant::Outlined => "Outlined",
    }
}

#[tessera]
pub(super) fn cards_spec() {
    article(
        "Cards",
        "Present content and actions about one subject in a distinct container.",
        vec![
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Elevated card",
                content: RenderSlot::new(|| {
                    detail(CardVariant::Elevated);
                }),
            },
            SpecSection {
                title: "Filled card",
                content: RenderSlot::new(|| {
                    detail(CardVariant::Filled);
                }),
            },
            SpecSection {
                title: "Outlined card",
                content: RenderSlot::new(|| {
                    detail(CardVariant::Outlined);
                }),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    ).pages(vec![
        ArticlePage {title:"Overview",intro:Some(RenderSlot::new(card_variants)),sections:vec![
            SpecSection {title:"Differences from M2",content:RenderSlot::new(|| {prose("Cards use dynamic color mappings and lower resting elevation. Choose elevated, filled or outlined containment according to the visual emphasis needed.");gap(Dp(24.0));card_variants();})},
        ]},
        ArticlePage {title:"Guidelines",intro:None,sections:vec![
            SpecSection {title:"Usage",content:RenderSlot::new(|| {prose("Keep each card about one subject. Establish a readable hierarchy and use a collection when several subjects belong together. Prefer spacing or dividers when a container would add unnecessary complexity.");gap(Dp(24.0));card_variants();})},
            SpecSection {title:"Anatomy",content:RenderSlot::new(card_guideline_anatomy)},
            SpecSection {title:"Cards in a collection",content:RenderSlot::new(card_collections)},
            SpecSection {title:"Adaptive design",content:RenderSlot::new(card_adaptive)},
            SpecSection {title:"Behavior",content:RenderSlot::new(card_behavior)},
        ]},
        ArticlePage {title:"Accessibility",intro:None,sections:vec![
            SpecSection {title:"Use cases",content:RenderSlot::new(|| {prose("People must be able to reach a card and its controls, understand its content and receive feedback appropriate to touch, pointer or keyboard input.");gap(Dp(24.0));action_models();})},
            SpecSection {title:"Interaction & style",content:RenderSlot::new(card_interaction)},
            SpecSection {title:"Focus",content:RenderSlot::new(|| {prose("A directly actionable card is one tab stop. A passive card is skipped while its buttons and links remain individual tab stops. Avoid nesting controls inside an actionable card.");gap(Dp(24.0));action_models();})},
            SpecSection {title:"Keyboard navigation",content:RenderSlot::new(|| {spec_table(vec!["Keys".into(),"Action".into()],vec![vec!["Tab".into(),"Next actionable card or contained control".into()],vec!["Space or Enter".into(),"Activate the focused action".into()]]);})},
            SpecSection {title:"Labeling elements",content:RenderSlot::new(|| {prose("Expose meaningful headings, text and actions in reading order. Hide purely decorative media. Name directly actionable cards by their destination; passive card containers do not need an interactive role.");gap(Dp(24.0));card_figure(true);})},
        ]},
    ]);
}

#[tessera]
fn sample(
    variant: CardVariant,
    enabled: bool,
    #[prop(into)] label: String,
    modifier: Option<Modifier>,
) {
    let actions = remember(|| 0_u32);
    column().children(move || {
        let caption = label.clone();
        let label = label.clone();
        card()
            .variant(variant)
            .enabled(enabled)
            .modifier(modifier.clone().unwrap_or_default().width(Dp(280.0)))
            .accessibility_label(label.clone())
            .on_click(move || actions.set(actions.get() + 1))
            .content(move || {
                body(label.clone());
            });
        gap(Dp(8.0));
        text()
            .content(format!("{caption} activations: {}", actions.get()))
            .style(type_style(12.0, 16.0));
    });
}

#[tessera]
fn body(#[prop(into)] label: String) {
    column()
        .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0)))
        .children(move || {
            text().content(label.clone()).style(type_style(24.0, 32.0));
            gap(Dp(8.0));
            text()
                .content("A focused collection of related content.")
                .style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn tokens() {
    let variant = remember(|| CardVariant::Elevated);
    flow_row().item_spacing(Dp(8.0)).children(move || {
        for style in VARIANTS {
            button()
                .text()
                .accessibility_label(format!("{} card tokens", name(style)))
                .on_click(move || variant.set(style))
                .child(move || {
                    text().content(name(style));
                });
        }
    });
    gap(Dp(24.0));
    let (container, outline, elevation, hover, dragged) = match variant.get() {
        CardVariant::Elevated => ("Surface container low", "None", "1dp", "2dp", "4dp"),
        CardVariant::Filled => ("Surface container highest", "None", "0dp", "1dp", "3dp"),
        CardVariant::Outlined => ("Surface", "Outline variant · 1dp", "0dp", "1dp", "3dp"),
    };
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Container".into(), container.into()],
            vec!["Outline".into(), outline.into()],
            vec!["Shape".into(), "12dp corners".into()],
            vec![
                "Resting / focused / pressed elevation".into(),
                elevation.into(),
            ],
            vec!["Hovered elevation".into(), hover.into()],
            vec!["Dragged elevation".into(), dragged.into()],
        ],
    );
}

#[tessera]
fn scheme(variant: CardVariant, dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            let theme = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get();
            surface()
                .style(theme.color_scheme.surface.into())
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                        .children(move || {
                            subheading(if dark { "Dark theme" } else { "Light theme" }.to_owned());
                            gap(Dp(24.0));
                            sample(
                                variant,
                                true,
                                format!(
                                    "{} · {}",
                                    name(variant),
                                    if dark { "dark" } else { "light" }
                                ),
                            );
                        });
                });
        });
}

#[tessera]
fn detail(variant: CardVariant) {
    let geometry = remember(SpaceGeometry::new);
    boxed()
        .modifier(watch_in_space(geometry, "figure").fill_max_width())
        .children(move || {
            panel(RenderSlot::new(move || {
                sample(variant, true, format!("{} card", name(variant)))
                    .modifier(watch_in_space(geometry, "container"));
            }));
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    card_container_annotations(variant, geometry);
                });
        });
    gap(Dp(24.0));
    text()
        .content(if variant == CardVariant::Outlined {
            "1  Container    2  Outline"
        } else {
            "1  Container"
        })
        .style(type_style(16.0, 24.0));
    gap(Dp(48.0));
    subheading(format!("{} card color", name(variant)));
    gap(Dp(24.0));
    scheme(variant, false);
    gap(Dp(24.0));
    scheme(variant, true);
    gap(Dp(48.0));
    subheading(format!("{} card states", name(variant)));
    gap(Dp(24.0));
    text().content("Hover, focus with Tab, or hold a card to inspect actual feedback. Disabled cards do not activate.")
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(move || {
                sample(variant, true, format!("{} enabled", name(variant)));
                sample(variant, false, format!("{} disabled", name(variant)));
            });
    }));
}

#[tessera]
fn card_container_annotations(variant: CardVariant, geometry: State<SpaceGeometry>) {
    let values = geometry.with(annotations::space_geometry);
    let (Some(root), Some(container)) = (values.get("figure"), values.get("container")) else {
        return;
    };
    let mut svg = String::new();
    let y = container.y + container.h / 2.0;
    let left = container.x - 48.0;
    dotted(
        &mut svg,
        left + 24.0,
        y,
        container.x - left - 20.0,
        false,
        "#49454f",
    );
    annotations::marker(1, Dp(left), Dp(y - 16.0));
    if variant == CardVariant::Outlined {
        let right = container.x + container.w;
        dotted(&mut svg, right, y, 32.0, false, "#49454f");
        annotations::marker(2, Dp(right + 32.0), Dp(y - 16.0));
    }
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(|| {
                sample(CardVariant::Outlined, true, "First card");
                sample(CardVariant::Outlined, true, "Second card");
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Shape".into(), "12dp corner radius".into()],
            vec!["Content left / right padding".into(), "16dp".into()],
            vec!["Between cards".into(), "8dp maximum".into()],
            vec!["Label alignment".into(), "Start-aligned".into()],
        ],
    );
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn card_variants() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                for variant in VARIANTS {
                    sample(variant, true, format!("{} overview card", name(variant)));
                }
            });
    }));
}

#[tessera]
fn card_figure(annotated: bool) {
    let geometry = remember(Geometry::new);
    let calls = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed().alignment(Alignment::Center).modifier(watch(geometry,"figure").fill_max_width().height(Dp(584.0))
        .background_with_shape(scheme.surface_container_low,Shape::rounded_rectangle(Dp(24.0))))
        .children(move || {
            card().elevated().modifier(watch(geometry,"container").width(Dp(448.0))).content(move || {
                column().modifier(Modifier::new().fill_max_width()).children(move || {
                    spacer().modifier(watch(geometry,"media").fill_max_width().height(Dp(194.0)).background(scheme.tertiary_container));
                    column().modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0))).children(move || {
                        text().content("Headline").style(type_style(40.0,48.0)).modifier(watch(geometry,"headline"));
                        text().content("Subhead").style(type_style(24.0,32.0)).modifier(watch(geometry,"subhead"));
                        gap(Dp(32.0));
                        text().content("Explain more about the topic shown in the headline and subhead through supporting text.")
                            .style(type_style(14.0,20.0)).modifier(watch(geometry,"support"));
                        gap(Dp(32.0));
                        row().main_axis_alignment(MainAxisAlignment::End).modifier(Modifier::new().fill_max_width()).children(move || {
                            button().outlined().accessibility_label("Card secondary action").on_click(move || calls.set(calls.get()+1)).child(|| {text().content("Action");});
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            button().modifier(watch(geometry,"action")).accessibility_label("Card primary button").on_click(move || calls.set(calls.get()+1)).child(|| {text().content("Action");});
                        });
                    });
                });
            });
            if annotated {boxed().modifier(Modifier::new().fill_max_size()).children(move || {card_callouts(geometry);});}
        });
    gap(Dp(8.0));
    text()
        .content(format!("Card button actions: {}", calls.get()))
        .style(type_style(12.0, 16.0));
}

#[tessera]
fn card_callouts(geometry: State<Geometry>) {
    let values = geometry.get();
    let (
        Some(root),
        Some(card),
        Some(media),
        Some(headline),
        Some(subhead),
        Some(support),
        Some(action),
    ) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "media"),
        relative(&values, "headline"),
        relative(&values, "subhead"),
        relative(&values, "support"),
        relative(&values, "action"),
    )
    else {
        return;
    };
    let mut svg = String::new();
    for (number, target, right) in [
        (1, card, false),
        (2, media, true),
        (3, action, true),
        (4, support, false),
        (5, subhead, false),
        (6, headline, false),
    ] {
        let y = if number == 1 {
            card.y + 96.0
        } else if number == 2 {
            media.y + media.h - 16.0
        } else {
            target.y + target.h / 2.0
        };
        let x = if right {
            card.x + card.w + 32.0
        } else {
            card.x - 48.0
        };
        let tx = if right {
            target.x + target.w - 4.0
        } else {
            target.x + 4.0
        };
        dotted(
            &mut svg,
            if right { tx } else { x + 24.0 },
            y,
            if right { x - tx } else { tx - x - 24.0 },
            false,
            "#49454f",
        );
        annotations::marker(number, Dp(x), Dp(y - 16.0));
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn card_guideline_anatomy() {
    prose(
        "Only the container is required. Optional content blocks establish hierarchy through media, a headline, subhead, supporting text and actions.",
    );
    gap(Dp(24.0));
    card_figure(true);
    gap(Dp(24.0));
    subheading("Container".into());
    prose(
        "The content determines the container's dimensions. Its surface and elevation distinguish the card from its surroundings.",
    );
    gap(Dp(24.0));
    subheading("Content blocks".into());
    prose(
        "Group related elements into clear blocks and give the most important information stronger visual emphasis.",
    );
    gap(Dp(24.0));
    subheading("Dividers".into());
    prose(
        "Use full-width dividers around expandable content and inset dividers between related regions.",
    );
    gap(Dp(24.0));
    subheading("Media".into());
    prose(
        "A card can contain a thumbnail, image, illustration or video. Keep decorative media outside the accessibility reading order.",
    );
    gap(Dp(24.0));
    subheading("Text".into());
    prose(
        "Use the headline for the subject, a subhead for context and supporting text for details. Prefer placing text on a solid surface instead of over an image.",
    );
    gap(Dp(24.0));
    subheading("Actions".into());
    action_models();
}

#[tessera]
fn collection_card(index: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    card()
        .filled()
        .modifier(Modifier::new().width(Dp(176.0)))
        .content(move || {
            column().children(move || {
                spacer().modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(96.0))
                        .background(if index % 2 == 0 {
                            scheme.secondary_container
                        } else {
                            scheme.tertiary_container
                        }),
                );
                text()
                    .content(format!("Collection {}", index + 1))
                    .style(type_style(18.0, 24.0))
                    .modifier(Modifier::new().padding_all(Dp(16.0)));
            });
        });
}

#[tessera]
fn card_collections() {
    prose(
        "A collection shares a resting elevation. Place sorting and filtering controls outside the cards and apply them to the whole collection.",
    );
    gap(Dp(24.0));
    subheading("Grid".into());
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(|| {
                for index in 0..6 {
                    collection_card(index);
                }
            });
    }));
    gap(Dp(24.0));
    subheading("Vertical list".into());
    panel(RenderSlot::new(|| {
        column().children(|| {
            for index in 0..3 {
                collection_card(index);
                gap(Dp(8.0));
            }
        });
    }));
    gap(Dp(24.0));
    subheading("Carousel".into());
    panel(RenderSlot::new(|| {
        carousel(5)
            .variant(CarouselVariant::Uncontained)
            .height(Dp(152.0))
            .preferred_item_width(Dp(176.0))
            .item_spacing(Dp(8.0))
            .accessibility_label("Card collection carousel")
            .content(|item| {
                collection_card(item.index);
            });
    }));
}

#[tessera]
fn card_adaptive() {
    subheading("Ergonomics".into());
    prose(
        "Adapt the orientation and size of content blocks to the available space, keeping the main content prominent.",
    );
    gap(Dp(24.0));
    subheading("Visual presentation".into());
    prose(
        "Adjust spacing before stretching text-heavy cards. Let media and content blocks respond to the window's proportions.",
    );
    gap(Dp(24.0));
    subheading("Column-based layouts".into());
    prose("Use multiple columns on wide screens and a single readable column in compact layouts.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(|| {
                for index in 0..4 {
                    collection_card(index);
                }
            });
    }));
    gap(Dp(24.0));
    subheading("Small screens".into());
    prose(
        "A compact list can replace a card collection when space is limited. Preserve the same information and actions.",
    );
}

#[tessera]
fn action_models() {
    let clicks = remember(|| 0_usize);
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(move || {
                card()
                    .outlined()
                    .modifier(Modifier::new().width(Dp(248.0)))
                    .content(move || {
                        column()
                            .modifier(Modifier::new().padding_all(Dp(16.0)))
                            .children(move || {
                                text()
                                    .content("Non-actionable card")
                                    .style(type_style(20.0, 28.0));
                                gap(Dp(16.0));
                                button()
                                    .on_click(move || clicks.set(clicks.get() + 1))
                                    .child(|| {
                                        text().content("Action");
                                    });
                            });
                    });
                card()
                    .outlined()
                    .modifier(Modifier::new().width(Dp(248.0)))
                    .accessibility_label("Open card details")
                    .on_click(move || clicks.set(clicks.get() + 1))
                    .content(|| {
                        body("Directly actionable card");
                    });
            });
        gap(Dp(16.0));
        prose(format!("Card actions: {}", clicks.get()));
    }));
}

#[tessera]
fn card_behavior() {
    subheading("Expanding".into());
    prose(
        "Reveal additional content by expanding the card. Avoid an independent scrolling region inside a card on compact screens.",
    );
    gap(Dp(24.0));
    expanding_card();
    gap(Dp(24.0));
    subheading("Navigation".into());
    prose(
        "A card may navigate to details at the next level of hierarchy. Keep its destination clear from the headline and accessible name.",
    );
    gap(Dp(24.0));
    subheading("Gestures".into());
    prose(
        "Assign one swipe action per card and provide a visible alternative for moving or dismissing it. A dragged card rises above its neighbors without displacing them.",
    );
}

#[tessera]
fn expanding_card() {
    let expanded = remember(|| false);
    panel(RenderSlot::new(move || {
        card().outlined().modifier(Modifier::new().width(Dp(360.0))).content(move || {column().modifier(Modifier::new().padding_all(Dp(16.0))).children(move || {text().content("Explore the details").style(type_style(24.0,32.0));gap(Dp(16.0));if expanded.get() {prose("Expanded content stays within the card's hierarchy and grows the card instead of introducing a nested scrolling area.");gap(Dp(16.0));}button().text().accessibility_label("Toggle card details").on_click(move || expanded.set(!expanded.get())).child(move || {text().content(if expanded.get() {"Show less"} else {"Learn more"});});});});
    }));
}

#[tessera]
fn card_interaction() {
    prose(
        "Use either a directly actionable card or a passive container with internal actions. Avoid stacking actions on an actionable surface.",
    );
    gap(Dp(24.0));
    subheading("Touch".into());
    prose(
        "Directly actionable cards ripple across their surface. Passive cards leave feedback to their controls.",
    );
    gap(Dp(24.0));
    subheading("Dragging and dismissing".into());
    prose(
        "Offer a single-pointer action as an alternative to dragging or swiping. Keep that action reachable without covering the dragged card.",
    );
    gap(Dp(24.0));
    subheading("Cursor".into());
    prose("Hover and click feedback belong to the actionable surface or control.");
    gap(Dp(24.0));
    subheading("Keyboard".into());
    prose("Tab reaches every action. Space or Enter activates the focused card or button.");
    gap(Dp(24.0));
    action_models();
}
