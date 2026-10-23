//! Interactive divider specifications.
//!
//! ## Usage
//!
//! Compare full-width and inset separation with content spacing.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    divider::{horizontal_divider, vertical_divider},
    list_item::list_item,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn divider_spec() {
    article(
        "Divider",
        "Group related content and separate neighboring sections with a thin line.",
        vec![
            SpecSection {
                title: "Tokens and specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    )
    .hero(|| {
        panel(RenderSlot::new(grouped_content));
    }).pages(vec![
        ArticlePage {title:"Overview",intro:Some(RenderSlot::new(|| {prose("Use a subtle divider when spacing alone cannot communicate groups. Group related content rather than drawing a line between every item.");gap(Dp(24.0));mail_groups(true);})),sections:vec![
            SpecSection {title:"Differences from M2",content:RenderSlot::new(|| {prose("Dividers use updated dynamic color mappings and can also be vertical.");gap(Dp(24.0));vertical_example();})},
        ]},
        ArticlePage {title:"Guidelines",intro:None,sections:vec![
            SpecSection {title:"Usage",content:RenderSlot::new(|| {prose("Use full-width lines between unrelated sections and inset lines within a related group. Prefer open space when it already communicates the hierarchy.");gap(Dp(24.0));divider_comparison();})},
            SpecSection {title:"Anatomy",content:RenderSlot::new(divider_anatomy)},
            SpecSection {title:"Full-width dividers",content:RenderSlot::new(|| {prose("Separate larger sections or distinguish interactive controls from informational content. Too many full-width lines create visual clutter.");gap(Dp(24.0));full_width_card();})},
            SpecSection {title:"Inset dividers",content:RenderSlot::new(|| {prose("Use equal end insets to group related content. Leading avatars or icons anchor the rows and make the inset alignment easy to scan.");gap(Dp(24.0));mail_groups(true);gap(Dp(24.0));subheading("Using dividers both ways on the same screen".into());prose("Use full width for a change in content type and inset lines for nested items. Repeated rows may only need spacing.");gap(Dp(24.0));mail_groups(false);})},
            SpecSection {title:"Vertical divider",content:RenderSlot::new(|| {prose("On larger screens, a vertical divider can distinguish text from related media or another pane.");gap(Dp(24.0));vertical_example();})},
        ]},
        ArticlePage {title:"Accessibility",intro:Some(RenderSlot::new(|| {prose("Dividers are decorative elements, which have no contrast minimums.");gap(Dp(24.0));decorative_example();})),sections:vec![]},
    ]);
}

#[tessera]
fn grouped_content() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            text().content("Account").style(type_style(20.0, 28.0));
            gap(Dp(16.0));
            text()
                .content("Profile and preferences")
                .style(type_style(16.0, 24.0));
            gap(Dp(16.0));
            horizontal_divider();
            gap(Dp(16.0));
            text()
                .content("Notifications")
                .style(type_style(20.0, 28.0));
            gap(Dp(16.0));
            text()
                .content("Delivery and quiet hours")
                .style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Color".into(), "Outline variant".into()],
            vec!["Thickness".into(), "1dp".into()],
        ],
    );
}

#[tessera]
fn scheme(dark: bool) {
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
                            grouped_content();
                        });
                });
        });
}

#[tessera]
fn colors() {
    scheme(false);
    gap(Dp(24.0));
    scheme(true);
}

#[tessera]
fn inset_example(#[prop(into)] label: String, left: Dp, right: Dp) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            text().content(label.clone()).style(type_style(16.0, 24.0));
            gap(Dp(24.0));
            let label = label.clone();
            boxed()
                .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                    left,
                    Dp::ZERO,
                    right,
                    Dp::ZERO,
                )))
                .children(move || {
                    horizontal_divider().modifier(Modifier::new().semantics(SemanticsArgs {
                        role: Some(Role::Group),
                        label: Some(format!("{label} measurement")),
                        ..Default::default()
                    }));
                });
            gap(Dp(32.0));
        });
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                inset_example("Full width", Dp::ZERO, Dp::ZERO);
                inset_example("Inset", Dp(16.0), Dp::ZERO);
                inset_example("Middle inset", Dp(16.0), Dp(16.0));
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Full-width divider".into(), "100%".into()],
            vec!["Inset · left / right".into(), "16 / 0dp".into()],
            vec!["Middle inset · left / right".into(), "16 / 16dp".into()],
            vec!["Divider to supporting text".into(), "4dp".into()],
            vec!["Divider right margin".into(), "8dp".into()],
            vec!["Divider bottom margin".into(), "8dp".into()],
        ],
    );
    gap(Dp(32.0));
    subheading("Supporting text and spacing".into());
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                text()
                    .content("Supporting text")
                    .style(type_style(14.0, 20.0));
                gap(Dp(4.0));
                boxed()
                    .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                        Dp::ZERO,
                        Dp::ZERO,
                        Dp(8.0),
                        Dp(8.0),
                    )))
                    .children(|| {
                        horizontal_divider();
                    });
            });
    }));
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn divider_comparison() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                text()
                    .content("Full-width divider")
                    .style(type_style(18.0, 24.0));
                gap(Dp(24.0));
                horizontal_divider();
                gap(Dp(48.0));
                text()
                    .content("Inset divider")
                    .style(type_style(18.0, 24.0));
                gap(Dp(24.0));
                boxed()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .padding_symmetric(Dp(16.0), Dp::ZERO),
                    )
                    .children(|| {
                        horizontal_divider();
                    });
            });
    }));
}

#[tessera]
fn divider_anatomy() {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(224.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            horizontal_divider().modifier(watch(geometry, "line").width(Dp(320.0)));
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    divider_callout(geometry);
                });
        });
    gap(Dp(16.0));
    prose("1  Divider");
}

#[tessera]
fn divider_callout(geometry: State<Geometry>) {
    let values = geometry.get();
    let (Some(root), Some(line)) = (values.get("figure"), relative(&values, "line")) else {
        return;
    };
    let mut svg = String::new();
    dotted(
        &mut svg,
        line.x + line.w / 2.0,
        line.y - 48.0,
        48.0,
        true,
        "#49454f",
    );
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
    annotations::marker(1, Dp(line.x + line.w / 2.0 - 12.0), Dp(line.y - 80.0));
    annotations::dimension("1dp".into(), Dp(line.x + line.w + 16.0), Dp(line.y - 10.0));
}

#[tessera]
fn mail_row(index: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    list_item()
        .headline(["Alex", "Sam", "Taylor"][index])
        .supporting_text(["Project update", "Weekend plans", "Dinner invitation"][index])
        .leading(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(40.0), Dp(40.0))
                        .background_with_shape(scheme.secondary_container, Shape::Ellipse),
                )
                .children(move || {
                    text()
                        .content(["A", "S", "T"][index])
                        .color(scheme.on_secondary_container);
                });
        })
        .trailing(|| {
            text().content("10:30").style(type_style(12.0, 16.0));
        });
}

#[tessera]
fn mail_groups(inset: bool) {
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                text()
                    .content("Inbox")
                    .style(type_style(24.0, 32.0))
                    .modifier(Modifier::new().padding_all(Dp(16.0)));
                for index in 0..3 {
                    mail_entry(index, inset);
                }
                gap(Dp(16.0));
                horizontal_divider();
                gap(Dp(16.0));
                text()
                    .content("Earlier messages")
                    .style(type_style(18.0, 24.0))
                    .modifier(Modifier::new().padding_all(Dp(16.0)));
                mail_row(0);
            });
    }));
}

#[tessera]
fn mail_entry(index: usize, inset: bool) {
    mail_row(index);
    if inset && index < 2 {
        boxed()
            .modifier(
                Modifier::new()
                    .fill_max_width()
                    .padding_symmetric(Dp(16.0), Dp::ZERO),
            )
            .children(|| {
                horizontal_divider();
            });
    }
}

#[tessera]
fn full_width_card() {
    let saved = remember(|| false);
    panel(RenderSlot::new(move || {
        surface()
            .modifier(Modifier::new().width(Dp(360.0)))
            .child(move || {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        column()
                            .modifier(Modifier::new().padding_all(Dp(24.0)))
                            .children(|| {
                                text()
                                    .content("Account preferences")
                                    .style(type_style(24.0, 32.0));
                                gap(Dp(16.0));
                                prose("Manage your profile and notification preferences.");
                            });
                        horizontal_divider();
                        button()
                            .text()
                            .on_click(move || saved.set(!saved.get()))
                            .child(move || {
                                text().content(if saved.get() { "Saved" } else { "Save changes" });
                            });
                    });
            });
    }));
}

#[tessera]
fn vertical_example() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        row().cross_axis_alignment(CrossAxisAlignment::Center).modifier(Modifier::new().fill_max_width()).children(move || {
        column().modifier(Modifier::new().weight(1.0).padding_all(Dp(16.0))).children(|| {text().content("Explore the collection").style(type_style(24.0,32.0));gap(Dp(16.0));prose("A vertical divider separates this description from its related media while preserving their shared context.");});
        vertical_divider().modifier(Modifier::new().height(Dp(180.0)));
        spacer().modifier(Modifier::new().weight(1.0).height(Dp(160.0)).padding_all(Dp(16.0)).background(scheme.tertiary_container));
    });
    }));
}

#[tessera]
fn decorative_example() {
    panel(RenderSlot::new(|| {
        column().modifier(Modifier::new().fill_max_width()).children(|| {
        prose("Decorative elements have no contrast minimums");gap(Dp(24.0));
        text().content("Related content").style(type_style(20.0,28.0));gap(Dp(16.0));horizontal_divider();gap(Dp(16.0));
        prose("Content and controls keep their own labels and reading order. The decorative line adds no focus stop.");
    });
    }));
}
