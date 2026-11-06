use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    floating_action_button_menu::{
        FloatingActionButtonMenuAlignment, FloatingActionButtonMenuColor,
        FloatingActionButtonMenuItem, FloatingActionButtonMenuState, floating_action_button_menu,
    },
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Color, Dp, Modifier, RenderSlot, accesskit::Role, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);

const ITEMS: [(&str, &str); 3] = [
    ("ui-new-document", "ui-document"),
    ("ui-new-folder", "ui-folder"),
    ("ui-upload-file", "ui-file"),
];

#[tessera]
pub(super) fn fab_menu_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-fab-menu",
        "ui-the-fab-menu-expands-from-a-floating-action-button-to-show-related-actions",
        vec![
            SpecSection {
                title: "ui-variants",
                content: RenderSlot::new(variants),
            },
            SpecSection {
                title: "ui-configurations",
                content: RenderSlot::new(configurations),
            },
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "ui-anatomy",
                content: RenderSlot::new(anatomy),
            },
            SpecSection {
                title: "ui-color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "ui-states",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    )
    .route_optional(route.clone())
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview_intro)),
            sections: vec![],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(guide_anatomy),
                },
                SpecSection {
                    title: "ui-color",
                    content: RenderSlot::new(guide_color),
                },
                SpecSection {
                    title: "ui-adaptive-layout",
                    content: RenderSlot::new(adaptive_layout),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(behavior),
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
                    title: "ui-interaction-style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "ui-initial-focus",
                    content: RenderSlot::new(initial_focus),
                },
                SpecSection {
                    title: "ui-keyboard-navigation",
                    content: RenderSlot::new(keyboard),
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
fn prose(#[prop(into)] value: String) {
    text(I18N.text(value)).style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text(I18N.text(value))
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    column().children(move || {
        for item in items.clone() {
            text(format!("•  {}", I18N.text(item))).style(type_style(16.0, 24.0));
            gap(Dp(8.0));
        }
    });
}

#[tessera]
fn badge(number: usize) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(INK, Shape::Ellipse),
        )
        .children(move || {
            text((number).to_string())
                .style(type_style(12.0, 16.0))
                .color(Color::WHITE);
        });
}

#[tessera]
fn legend(number: usize, #[prop(into)] label: String) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
        .children(move || {
            badge(number);
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text(I18N.text(&(label))).style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn guidance_line(#[prop(into)] mark: String, #[prop(into)] note: String, color: Color) {
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
                        .padding(Padding::symmetric(Dp(12.0), Dp(4.0)))
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(8.0))),
                )
                .children(move || {
                    text(I18N.text(&(mark)))
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text(I18N.text(&(note)))
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn do_dont(#[prop(into)] good: String, #[prop(into)] bad: String) {
    column().children(move || {
        guidance_line("Do", good.clone(), DO_COLOR);
        gap(Dp(12.0));
        guidance_line("Don't", bad.clone(), DONT_COLOR);
    });
}

#[tessera]
fn menu_demo(
    #[prop(into)] label: String,
    color: FloatingActionButtonMenuColor,
    mirrored: bool,
    expanded: bool,
) {
    let state = remember(move || {
        let mut state = FloatingActionButtonMenuState::new();
        if expanded {
            state.open();
        }
        state
    });
    let status = remember(|| "ui-no-action-selected".to_owned());
    let theme = use_context::<MaterialTheme>().expect("MaterialTheme").get();
    let caption_text = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(I18N.format(
                "ui-value-fab-menu-example",
                [("v0", I18N.text(&label).into())],
            )),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            floating_action_button_menu()
                .state(state)
                .color(color)
                .alignment(if mirrored {
                    FloatingActionButtonMenuAlignment::Start
                } else {
                    FloatingActionButtonMenuAlignment::End
                })
                .open_label(I18N.format("ui-open-value-menu", [("v0", I18N.text(&label).into())]))
                .close_label(I18N.format("ui-close-value-menu", [("v0", I18N.text(&label).into())]))
                .modifier(Modifier::new().fill_max_width().height(Dp(300.0)))
                .items(vec![
                    FloatingActionButtonMenuItem::new(
                        I18N.text(ITEMS[0].0),
                        Some(filled::EDIT_SVG.into()),
                        Callback::new(move || status.set(ITEMS[0].1.into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        I18N.text(ITEMS[1].0),
                        Some(filled::FOLDER_SVG.into()),
                        Callback::new(move || status.set(ITEMS[1].1.into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        I18N.text(ITEMS[2].0),
                        Some(filled::UPLOAD_SVG.into()),
                        Callback::new(move || status.set(ITEMS[2].1.into())),
                    ),
                ])
                .main_content(move || {
                    surface()
                        .style(theme.color_scheme.surface_container_low.into())
                        .modifier(Modifier::new().fill_max_size())
                        .child(move || {
                            column()
                                .modifier(Modifier::new().padding(Padding::new(
                                    Dp(if mirrored { 220.0 } else { 24.0 }),
                                    Dp(24.0),
                                    Dp(24.0),
                                    Dp(24.0),
                                )))
                                .children(move || {
                                    text(I18N.text("ui-create"))
                                        .style(theme.typography.title_large);
                                    gap(Dp(8.0));
                                    text(I18N.text(status.get())).style(type_style(16.0, 24.0));
                                });
                        });
                });
            gap(Dp(12.0));
            caption(I18N.format(
                "ui-last-action-caption",
                [
                    ("caption", I18N.text(&caption_text).into()),
                    ("action", I18N.text(status.get()).into()),
                ],
            ));
        });
}

#[tessera]
fn overview_intro() {
    bullets(vec![
        "ui-opens-from-a-fab-to-show-2-6-related-actions-floating-on-screen".into(),
        "ui-one-fab-menu-size-for-all-sizes-of-fabs".into(),
        "ui-not-used-with-extended-fabs".into(),
        "ui-available-in-primary-secondary-and-tertiary-color-sets".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-overview",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-the-fab-menu-comes-in-three-color-sets-primary-secondary-tertiary");
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-variant",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-there-s-one-variant-of-fab-menu");
}

#[tessera]
fn configurations() {
    subheading("ui-color-set".into());
    prose(
        "ui-fab-menus-come-in-three-color-sets-each-set-colors-the-close-button-and-the-items-together",
    );
    gap(Dp(24.0));
    for (number, color) in [
        FloatingActionButtonMenuColor::Primary,
        FloatingActionButtonMenuColor::Secondary,
        FloatingActionButtonMenuColor::Tertiary,
    ]
    .into_iter()
    .enumerate()
    {
        legend(
            number + 1,
            I18N.format("ui-value-set", [("v0", format!("{:?}", color).into())]),
        );
        gap(Dp(16.0));
        panel(RenderSlot::new(move || {
            menu_demo(format!("{:?}", color), color, false, true);
        }));
        gap(Dp(24.0));
    }
    subheading("ui-alignment".into());
    prose(
        "ui-the-fab-menu-is-aligned-to-the-trailing-edge-of-the-window-in-right-to-left-languages-it-i",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-trailing",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-leading",
            FloatingActionButtonMenuColor::Primary,
            true,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-trailing-alignment-by-default-leading-alignment-mirrors-the-layout-for-rtl");
}

#[tessera]
fn tokens() {
    prose(
        "ui-the-fab-menu-has-a-common-token-set-and-six-color-sets-three-for-each-element-close-button",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-element".into(), "ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "ui-close-button-container".into(),
                "md.comp.fab-menu.close-button.container.size".into(),
                "ui-56-56dp-full-round".into(),
            ],
            vec![
                "ui-close-button-color".into(),
                "md.comp.fab-menu.close-button.container.color".into(),
                "Primary · secondary · tertiary".into(),
            ],
            vec![
                "ui-close-button-icon".into(),
                "md.comp.fab-menu.close-button.icon.color".into(),
                "On primary · on secondary · on tertiary".into(),
            ],
            vec![
                "ui-item-container".into(),
                "md.comp.fab-menu.menu-item.container.color".into(),
                "Primary container · secondary container · tertiary container".into(),
            ],
            vec![
                "ui-item-label".into(),
                "md.comp.fab-menu.menu-item.label.text.color".into(),
                "On primary container · on secondary container · on tertiary container".into(),
            ],
            vec![
                "ui-item-shape".into(),
                "md.comp.fab-menu.menu-item.container.shape".into(),
                "ui-full-capsule".into(),
            ],
            vec![
                "ui-item-height".into(),
                "md.comp.fab-menu.menu-item.container.height".into(),
                "56dp".into(),
            ],
            vec![
                "ui-item-spacing".into(),
                "md.comp.fab-menu.menu-item.container.spacing".into(),
                "4dp".into(),
            ],
            vec![
                "ui-menu-margins".into(),
                "md.comp.fab-menu.menu.margin".into(),
                "16dp".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy() {
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-anatomy",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "ui-close-button");
        legend(2, "ui-menu-item");
    });
    gap(Dp(16.0));
    caption("ui-the-close-button-replaces-the-fab-each-menu-item-pairs-an-icon-with-a-label");
    gap(Dp(32.0));
    panel(RenderSlot::new(six_items));
    gap(Dp(16.0));
    caption("ui-the-fab-menu-can-have-up-to-six-items");
}

#[tessera]
fn six_items() {
    let state = remember(|| {
        let mut state = FloatingActionButtonMenuState::new();
        state.open();
        state
    });
    let theme = use_context::<MaterialTheme>().expect("MaterialTheme").get();
    let status = remember(|| "ui-six-actions".to_owned());
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            floating_action_button_menu()
                .state(state)
                .color(FloatingActionButtonMenuColor::Primary)
                .open_label(I18N.text("ui-open-six-action-menu"))
                .close_label(I18N.text("ui-close-six-action-menu"))
                .modifier(Modifier::new().fill_max_width().height(Dp(480.0)))
                .items(vec![
                    FloatingActionButtonMenuItem::new(
                        I18N.text("ui-first"),
                        Some(filled::EDIT_SVG.into()),
                        Callback::new(move || status.set("ui-first".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        I18N.text("ui-second"),
                        Some(filled::FOLDER_SVG.into()),
                        Callback::new(move || status.set("ui-second".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        I18N.text("ui-third"),
                        Some(filled::UPLOAD_SVG.into()),
                        Callback::new(move || status.set("ui-third".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        I18N.text("ui-fourth"),
                        Some(filled::IMAGE_SVG.into()),
                        Callback::new(move || status.set("ui-fourth".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        I18N.text("ui-fifth"),
                        Some(filled::EVENT_SVG.into()),
                        Callback::new(move || status.set("ui-fifth".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        I18N.text("ui-sixth"),
                        Some(filled::CHECK_SVG.into()),
                        Callback::new(move || status.set("ui-sixth".into())),
                    ),
                ])
                .main_content(move || {
                    surface()
                        .style(theme.color_scheme.surface_container_low.into())
                        .modifier(Modifier::new().fill_max_size())
                        .child(move || {
                            column()
                                .modifier(Modifier::new().padding_all(Dp(24.0)))
                                .children(move || {
                                    text(I18N.text("ui-up-to-six-actions"))
                                        .style(theme.typography.title_large);
                                    gap(Dp(8.0));
                                    text(I18N.text(status.get()));
                                });
                        });
                });
        });
}

#[tessera]
fn colors() {
    prose(
        "ui-fab-menu-colors-come-from-the-active-color-set-primary-secondary-or-tertiary-the-close-but",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Primary",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(24.0));
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(|| {
            panel(RenderSlot::new(|| {
                menu_demo(
                    "ui-dark",
                    FloatingActionButtonMenuColor::Primary,
                    false,
                    true,
                );
            }));
        });
    gap(Dp(24.0));
    spec_table(
        vec!["ui-set".into(), "ui-element".into(), "ui-color-role".into()],
        vec![
            vec![
                "Primary".into(),
                "ui-item-container".into(),
                "Primary container".into(),
            ],
            vec![
                "Primary".into(),
                "ui-item-label-and-icon".into(),
                "On primary container".into(),
            ],
            vec!["Primary".into(), "ui-close-button".into(), "Primary".into()],
            vec![
                "Primary".into(),
                "ui-close-icon".into(),
                "On primary".into(),
            ],
            vec![
                "Secondary".into(),
                "ui-item-container".into(),
                "Secondary container".into(),
            ],
            vec![
                "Secondary".into(),
                "ui-item-label-and-icon".into(),
                "On secondary container".into(),
            ],
            vec![
                "Secondary".into(),
                "ui-close-button".into(),
                "Secondary".into(),
            ],
            vec![
                "Secondary".into(),
                "ui-close-icon".into(),
                "On secondary".into(),
            ],
            vec![
                "Tertiary".into(),
                "ui-item-container".into(),
                "Tertiary container".into(),
            ],
            vec![
                "Tertiary".into(),
                "ui-item-label-and-icon".into(),
                "On tertiary container".into(),
            ],
            vec![
                "Tertiary".into(),
                "ui-close-button".into(),
                "Tertiary".into(),
            ],
            vec![
                "Tertiary".into(),
                "ui-close-icon".into(),
                "On tertiary".into(),
            ],
        ],
    );
}

#[tessera]
fn states() {
    prose(
        "ui-states-are-visual-representations-used-to-communicate-the-status-of-a-component-or-interac",
    );
    gap(Dp(32.0));
    subheading("ui-close-button".into());
    prose(
        "ui-enabled-hovered-focused-and-pressed-states-share-the-same-close-button-colors-hover-focus",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-close-states",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-close-button-states-enabled-hovered-focused-pressed");
    gap(Dp(32.0));
    subheading("ui-menu-item".into());
    prose(
        "ui-menu-items-keep-their-own-hover-focus-and-pressed-state-layers-while-the-menu-stays-open",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-item-states",
            FloatingActionButtonMenuColor::Secondary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-menu-item-states-enabled-hovered-focused-pressed");
}

#[tessera]
fn measurements() {
    prose(
        "ui-fab-menu-items-share-the-same-measurements-as-the-medium-button-specs-the-close-button-sho",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-element".into(),
            "ui-attribute".into(),
            "ui-value".into(),
        ],
        vec![
            vec![
                "ui-menu-item".into(),
                "ui-container-height".into(),
                "56dp".into(),
            ],
            vec![
                "ui-menu-item".into(),
                "ui-horizontal-padding".into(),
                "24dp".into(),
            ],
            vec!["ui-menu-item".into(), "ui-icon-size".into(), "24dp".into()],
            vec![
                "ui-menu-item".into(),
                "ui-icon-to-label-gap".into(),
                "8dp".into(),
            ],
            vec![
                "ui-menu-items".into(),
                "ui-spacing-between-items".into(),
                "4dp".into(),
            ],
            vec![
                "ui-close-button".into(),
                "ui-container-size".into(),
                "56 × 56dp".into(),
            ],
            vec!["ui-menu".into(), "ui-window-margin".into(), "16dp".into()],
            vec![
                "ui-menu-items".into(),
                "ui-minimum-target-size".into(),
                "48dp".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-measurements",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-fab-menu-size-measurements");
    gap(Dp(24.0));
    bullets(vec![
        "ui-the-fab-menu-animates-from-the-top-trailing-edge-of-the-fab-to-ensure-a-smooth-animation"
            .into(),
        "ui-the-fab-should-always-have-a-16dp-margin-from-the-window-edge".into(),
        "ui-the-close-button-and-fab-share-the-same-trailing-edge".into(),
    ]);
}

#[tessera]
fn usage() {
    prose(
        "ui-a-fab-menu-opens-from-a-fab-to-show-multiple-related-actions-it-should-always-appear-in-th",
    );
    gap(Dp(16.0));
    prose(
        "ui-this-makes-actions-immediately-accessible-and-keeps-the-ui-clean-by-concealing-actions-whe",
    );
    gap(Dp(24.0));
    do_dont(
        "ui-open-the-fab-menu-from-a-fab",
        "ui-don-t-open-a-fab-menu-from-an-extended-fab-or-any-other-extended-component",
    );
    gap(Dp(32.0));
    prose(
        "ui-the-fab-menu-should-be-aligned-to-the-trailing-edge-of-the-window-in-right-to-left-rtl-lan",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-mirrored",
            FloatingActionButtonMenuColor::Primary,
            true,
            true,
        );
    }));
    gap(Dp(16.0));
    caption(
        "ui-in-rtl-languages-the-fab-menu-is-left-aligned-and-the-icon-and-text-placement-is-mirrored",
    );
    gap(Dp(32.0));
    prose(
        "ui-fab-menus-can-contain-2-6-items-these-should-be-closely-related-under-a-single-action-like",
    );
    gap(Dp(16.0));
    prose("ui-avoid-grouping-unrelated-actions-in-the-same-fab-menu");
    gap(Dp(24.0));
    do_dont(
        "ui-fab-menus-can-have-2-6-items",
        "ui-don-t-use-a-fab-menu-with-one-item",
    );
    gap(Dp(32.0));
    prose(
        "ui-when-a-fab-is-paired-with-other-components-like-the-floating-toolbar-or-navigation-rail-do",
    );
    gap(Dp(24.0));
    do_dont(
        "ui-fabs-can-be-placed-next-to-toolbars-and-other-components",
        "ui-don-t-use-a-fab-menu-with-a-toolbar-or-navigation-rail",
    );
}

#[tessera]
fn guide_anatomy() {
    prose("ui-the-fab-menu-has-two-elements-the-close-button-and-the-menu-items");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-anatomy",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "ui-close-button");
        legend(2, "ui-menu-item");
    });
    gap(Dp(16.0));
    caption(
        "ui-the-close-button-takes-the-place-of-the-fab-and-each-item-pairs-an-icon-with-a-label",
    );
}

#[tessera]
fn guide_color() {
    prose(
        "ui-fab-menus-have-three-color-sets-primary-secondary-and-tertiary-use-the-color-set-that-best",
    );
    gap(Dp(16.0));
    prose(
        "ui-use-the-primary-fab-menu-color-set-with-the-primary-or-primary-container-fab-color-styles",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-primary-pair",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-a-primary-fab-is-paired-with-a-primary-fab-menu");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-tertiary-pair",
            FloatingActionButtonMenuColor::Tertiary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-a-tertiary-fab-is-paired-with-a-tertiary-fab-menu");
}

#[tessera]
fn adaptive_layout() {
    prose(
        "ui-the-fab-menu-can-scroll-when-there-are-more-actions-than-fit-on-screen-make-sure-the-items",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-adaptive",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(24.0));
    do_dont(
        "ui-allow-the-menu-items-to-scroll-behind-the-close-button",
        "ui-don-t-obstruct-the-close-button-in-short-screens-like-horizontal-orientation",
    );
    gap(Dp(32.0));
    prose(
        "ui-each-item-keeps-a-minimum-48dp-target-so-density-changes-must-preserve-the-target-size-of",
    );
}

#[tessera]
fn behavior() {
    prose(
        "ui-the-fab-menu-animates-from-the-top-trailing-edge-of-the-fab-items-expand-upward-in-order-a",
    );
    gap(Dp(16.0));
    prose(
        "ui-selecting-an-item-closes-the-menu-and-runs-that-action-pressing-escape-or-activating-the-c",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-behavior",
            FloatingActionButtonMenuColor::Primary,
            false,
            false,
        );
    }));
    gap(Dp(16.0));
    caption(
        "ui-open-the-menu-with-the-fab-choose-an-action-then-reopen-it-to-confirm-the-reported-action",
    );
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-using-assistive-technology");
    gap(Dp(16.0));
    bullets(vec![
        "ui-navigate-and-interact-with-the-fab-menu".into(),
        "ui-ensure-focus-is-correct-when-navigating-through-the-menu".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-use-cases",
            FloatingActionButtonMenuColor::Primary,
            false,
            false,
        );
    }));
}

#[tessera]
fn interaction_style() {
    prose("ui-fab-menu-elements-meet-the-minimum-target-size-of-48dp");
    gap(Dp(16.0));
    caption("ui-fab-menus-have-48x48dp-minimum-width-and-sufficient-spacing-by-default");
    gap(Dp(24.0));
    prose(
        "ui-when-the-fab-menu-can-scroll-make-sure-the-items-scroll-behind-the-close-button-the-close",
    );
    gap(Dp(24.0));
    do_dont(
        "ui-allow-the-menu-items-to-scroll-behind-the-close-button",
        "ui-don-t-obstruct-the-close-button-in-short-screens-like-horizontal-orientation",
    );
}

#[tessera]
fn initial_focus() {
    prose(
        "ui-when-the-fab-is-selected-the-fab-menu-opens-and-initial-focus-remains-on-the-close-button",
    );
    gap(Dp(16.0));
    prose("ui-then-the-focus-moves-from-the-top-menu-item-to-the-bottom");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-initial-focus",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-focus-lands-on-the-close-button-people-can-then-navigate-through-all-the-items");
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-tab".into(),
                "ui-navigate-to-the-next-interactive-element".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activate-the-focused-button-or-item".into(),
            ],
            vec![
                "ui-escape".into(),
                "ui-close-the-menu-and-return-focus-to-the-fab".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-the-native-sample-follows-this-table-tab-reaches-the-close-button-and-each-item-space-or-e",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-keyboard",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
}

#[tessera]
fn labeling() {
    prose(
        "ui-on-android-a-fab-menu-s-close-button-should-include-a-state-to-tell-screen-readers-what-ac",
    );
    gap(Dp(16.0));
    spec_table(
        vec![
            "ui-element".into(),
            "ui-label".into(),
            "ui-role".into(),
            "ui-state".into(),
        ],
        vec![
            vec![
                "ui-close-button".into(),
                "ui-toggle-menu".into(),
                "ui-button".into(),
                "ui-expanded-or-collapsed".into(),
            ],
            vec![
                "ui-menu-item".into(),
                "ui-match-the-item-s-ui-text".into(),
                "ui-menu-item".into(),
                "—".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    caption(
        "ui-on-android-the-close-button-accessibility-labels-should-include-a-toggle-menu-label-button",
    );
    gap(Dp(24.0));
    prose(
        "ui-fab-menu-items-should-be-labeled-with-the-same-text-people-see-on-screen-so-the-announced",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "ui-labeling",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
}
