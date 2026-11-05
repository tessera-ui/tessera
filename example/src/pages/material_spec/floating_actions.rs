//! Floating action button specifications.
//!
//! ## Usage
//!
//! Compare FAB sizes, tones, placement and interaction states.

use tessera_components::{
    boxed::boxed,
    button::button,
    column::column,
    floating_action_button::{FloatingActionButtonSize, floating_action_button},
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialColorScheme, MaterialTheme, material_theme},
};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, remember, tessera, use_context};

use crate::res::I18N;

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const SIZES: [FloatingActionButtonSize; 4] = [
    FloatingActionButtonSize::Small,
    FloatingActionButtonSize::Standard,
    FloatingActionButtonSize::Medium,
    FloatingActionButtonSize::Large,
];

pub(super) const TONES: [&str; 6] = [
    "Primary container",
    "Secondary container",
    "Tertiary container",
    "Primary",
    "Secondary",
    "Tertiary",
];

fn size_name(size: FloatingActionButtonSize) -> &'static str {
    match size {
        FloatingActionButtonSize::Small => "ui-small",
        FloatingActionButtonSize::Standard => "FAB",
        FloatingActionButtonSize::Medium => "ui-medium",
        FloatingActionButtonSize::Large => "ui-large",
    }
}

fn size_height(size: FloatingActionButtonSize) -> &'static str {
    match size {
        FloatingActionButtonSize::Small => "40dp",
        FloatingActionButtonSize::Standard => "56dp",
        FloatingActionButtonSize::Medium => "80dp",
        FloatingActionButtonSize::Large => "96dp",
    }
}

fn size_icon(size: FloatingActionButtonSize) -> f64 {
    match size {
        FloatingActionButtonSize::Small => 24.0,
        FloatingActionButtonSize::Standard => 24.0,
        FloatingActionButtonSize::Medium => 28.0,
        FloatingActionButtonSize::Large => 36.0,
    }
}

pub(super) fn tone_colors(scheme: &MaterialColorScheme, tone: usize) -> (Color, Color) {
    match tone {
        0 => (scheme.primary_container, scheme.on_primary_container),
        1 => (scheme.secondary_container, scheme.on_secondary_container),
        2 => (scheme.tertiary_container, scheme.on_tertiary_container),
        3 => (scheme.primary, scheme.on_primary),
        4 => (scheme.secondary, scheme.on_secondary),
        _ => (scheme.tertiary, scheme.on_tertiary),
    }
}

#[tessera]
pub(super) fn floating_actions_spec(
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    article(
        "ui-floating-action-buttons-fabs",
        "ui-floating-action-buttons-fabs-help-people-take-primary-actions",
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
            SpecSection {
                title: "ui-baseline-tokens-specs",
                content: RenderSlot::new(baseline_tokens),
            },
        ],
    )
    .route_optional(route.clone())
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![
                SpecSection {
                    title: "ui-availability-resources",
                    content: RenderSlot::new(resources),
                },
                SpecSection {
                    title: "ui-m3-expressive-update",
                    content: RenderSlot::new(expressive),
                },
                SpecSection {
                    title: "ui-differences-from-m2",
                    content: RenderSlot::new(differences),
                },
            ],
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
                    title: "ui-actions",
                    content: RenderSlot::new(actions),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(guidelines_anatomy),
                },
                SpecSection {
                    title: "ui-color",
                    content: RenderSlot::new(guidelines_color),
                },
                SpecSection {
                    title: "ui-placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "ui-adaptive-design",
                    content: RenderSlot::new(adaptive),
                },
                SpecSection {
                    title: "ui-behaviors",
                    content: RenderSlot::new(behaviors),
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
                    title: "ui-focus",
                    content: RenderSlot::new(focus),
                },
                SpecSection {
                    title: "ui-layout-position",
                    content: RenderSlot::new(layout_position),
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
fn prose(#[prop(into)] value: String) {
    text(I18N.text(value)).style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    gap(Dp(16.0));
    text(I18N.text(value))
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for item in items.clone() {
                row()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .padding_symmetric(Dp::ZERO, Dp(4.0)),
                    )
                    .children(move || {
                        spacer().modifier(
                            Modifier::new()
                                .size(Dp(8.0), Dp(8.0))
                                .background_with_shape(scheme.on_surface_variant, Shape::Ellipse),
                        );
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        text(I18N.text(&(item))).style(type_style(16.0, 24.0));
                    });
            }
        });
}

/// Renders one FAB with optional configuration overrides.
#[tessera]
fn fab(
    #[prop(into)] label: String,
    size: Option<FloatingActionButtonSize>,
    tone: Option<usize>,
    enabled: Option<bool>,
    counted: Option<bool>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let size = size.unwrap_or(FloatingActionButtonSize::Standard);
    let (container, content) = tone_colors(&scheme, tone.unwrap_or(0));
    let activations = remember(|| 0_u32);
    let icon_size = size_icon(size);
    column().children(move || {
        floating_action_button()
            .size(size)
            .container_color(container)
            .content_color(content)
            .ripple_color(content)
            .enabled(enabled.unwrap_or(true))
            .accessibility_label(I18N.text(&(label)))
            .on_click(move || activations.set(activations.get() + 1))
            .content(move || {
                icon().painter(filled::ADD_SVG).size(Dp(icon_size));
            });
        if counted.unwrap_or(false) {
            gap(Dp(16.0));
            text(I18N.format("ui-activations-value", [("v0", (activations.get()).into())]))
                .style(type_style(14.0, 20.0));
        }
    });
}

#[tessera]
fn overview() {
    prose(
        "ui-floating-action-buttons-help-people-take-a-primary-action-use-one-fab-for-the-most-common",
    );
    gap(Dp(16.0));
    bullets(vec![
        "ui-use-a-fab-for-the-most-common-or-important-action-on-a-screen".into(),
        "ui-make-sure-the-icon-in-a-fab-is-clear-and-understandable".into(),
        "ui-fabs-persist-on-the-screen-when-content-is-scrolling".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                for size in SIZES {
                    fab(size_name(size)).size(size);
                }
            });
    }));
    gap(Dp(16.0));
    prose("ui-small-fab-medium-fab-large-fab");
}

#[tessera]
fn resources() {
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
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Web: Expressive".into(),
                "ui-unavailable".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-tessera-examples-on-this-page-use-the-native-floating-action-button-component-four-si",
    );
}

#[tessera]
fn expressive() {
    prose(
        "ui-may-2025-the-fab-family-gained-small-medium-and-large-sizes-and-the-surface-color-styles-a",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-change".into(), "ui-details".into()],
        vec![
            vec![
                "ui-sizes".into(),
                "ui-small-40dp-fab-56dp-medium-80dp-and-large-96dp".into(),
            ],
            vec![
                "ui-color".into(),
                "ui-primary-container-is-the-default-secondary-and-tertiary-tones-are-available"
                    .into(),
            ],
            vec![
                "ui-baseline".into(),
                "ui-surface-color-styles-remain-available-but-are-no-longer-recommended".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        fab("ui-add");
    }));
    caption("ui-the-56dp-fab-with-a-primary-container-remains-the-recommended-default");
}

#[tessera]
fn differences() {
    prose(
        "ui-m2-the-fab-used-a-fixed-56dp-container-a-single-accent-color-and-elevation-6-m3-introduced",
    );
    gap(Dp(16.0));
    prose(
        "ui-m3-expressive-the-family-grew-to-four-sizes-the-icon-scales-with-the-container-and-the-cor",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "M2".into(), "M3 Expressive".into()],
        vec![
            vec![
                "ui-size".into(),
                "56dp".into(),
                "40 / 56 / 80 / 96dp".into(),
            ],
            vec!["ui-icon".into(), "24dp".into(), "24 / 28 / 36dp".into()],
            vec![
                "ui-corner-radius".into(),
                "28dp".into(),
                "12 / 16 / 20 / 28dp".into(),
            ],
            vec![
                "ui-color".into(),
                "ui-accent-fill".into(),
                "ui-primary-secondary-or-tertiary-container".into(),
            ],
        ],
    );
}

#[tessera]
fn variants() {
    prose(
        "ui-the-fab-family-has-four-sizes-that-share-one-anatomy-the-56dp-fab-is-the-default-small-sui",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                for size in SIZES {
                    fab(size_name(size)).size(size);
                }
            });
    }));
    gap(Dp(16.0));
    caption("ui-1-small-fab-2-fab-3-medium-fab-4-large-fab");
    gap(Dp(32.0));
    spec_table(
        vec!["ui-variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec!["ui-small-fab".into(), "--".into(), "ui-available".into()],
            vec!["FAB".into(), "ui-available".into(), "ui-available".into()],
            vec!["ui-medium-fab".into(), "--".into(), "ui-available".into()],
            vec!["ui-large-fab".into(), "--".into(), "ui-available".into()],
            vec![
                "ui-surface-fab-baseline".into(),
                "ui-available".into(),
                "ui-not-recommended".into(),
            ],
        ],
    );
}

#[tessera]
fn configurations() {
    prose(
        "ui-choose-a-size-and-a-container-tone-the-icon-scales-with-the-size-and-the-container-keeps-t",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                subheading("ui-1-size".into());
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(40.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        for size in SIZES {
                            fab(size_name(size)).size(size);
                        }
                    });
                gap(Dp(40.0));
                subheading("ui-2-color".into());
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(24.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        for (tone, name) in TONES.into_iter().enumerate() {
                            fab(name).tone(tone);
                        }
                    });
            });
    }));
    gap(Dp(32.0));
    spec_table(
        vec!["ui-configuration".into(), "ui-options".into()],
        vec![
            vec!["ui-size".into(), "ui-small-fab-medium-large".into()],
            vec![
                "ui-container-tones".into(),
                "Primary container · secondary container · tertiary container".into(),
            ],
            vec![
                "ui-accent-tones".into(),
                "Primary · secondary · tertiary".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    prose(
        "ui-fab-tokens-are-organized-by-size-and-color-values-below-follow-the-native-component-defaul",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "md.comp.fab.primary-container.color".into(),
                "Primary container".into(),
            ],
            vec![
                "md.comp.fab.primary-container.icon.color".into(),
                "On primary container".into(),
            ],
            vec![
                "md.comp.fab.secondary-container.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "md.comp.fab.tertiary-container.color".into(),
                "Tertiary container".into(),
            ],
            vec![
                "md.comp.fab.container.height".into(),
                "ui-40dp-small-56dp-80dp-medium-96dp-large".into(),
            ],
            vec![
                "md.comp.fab.container.width".into(),
                "ui-matches-the-container-height".into(),
            ],
            vec![
                "md.comp.fab.icon.size".into(),
                "24dp · 24dp · 28dp · 36dp".into(),
            ],
            vec![
                "md.comp.fab.container.shape".into(),
                "ui-12dp-16dp-20dp-28dp-corners".into(),
            ],
            vec![
                "md.comp.fab.container.elevation".into(),
                "ui-level-3-level-4-when-hovered".into(),
            ],
            vec![
                "md.comp.fab.state-layer".into(),
                "ui-icon-color-at-8-hover-and-10-focus-or-press".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-keep-the-state-layer-color-equal-to-the-icon-color-when-a-non-default-color-mapping-is-use",
    );
}

#[tessera]
fn anatomy() {
    prose(
        "ui-a-fab-is-a-single-container-with-one-icon-it-has-no-visible-label-the-icon-carries-the-mea",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column().children(move || {
            fab("ui-add");
            gap(Dp(32.0));
            for line in ["ui-1-container", "ui-2-icon"] {
                text(I18N.text(line))
                    .color(INK)
                    .style(type_style(16.0, 28.0));
            }
        });
    }));
    gap(Dp(16.0));
    caption("ui-one-container-one-icon-one-action");
}

#[tessera]
fn colors() {
    prose(
        "ui-container-tones-map-to-the-primary-secondary-and-tertiary-color-roles-either-as-containers-3175",
    );
    gap(Dp(24.0));
    for dark in [false, true] {
        subheading(if dark {
            "ui-dark-scheme".into()
        } else {
            "ui-light-scheme".into()
        });
        gap(Dp(24.0));
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                surface()
                    .modifier(Modifier::new().fill_max_width())
                    .child(move || {
                        column()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                            .children(move || {
                                flow_row()
                                    .item_spacing(Dp(24.0))
                                    .line_spacing(Dp(24.0))
                                    .children(move || {
                                        for (tone, name) in TONES.into_iter().enumerate() {
                                            fab(name).tone(tone);
                                        }
                                    });
                                gap(Dp(16.0));
                                text("1  Primary container    2  Secondary container    3  Tertiary container    4  Primary    5  Secondary    6  Tertiary")
                                    .style(type_style(14.0, 20.0));
                            });
                    });
            });
        gap(Dp(32.0));
    }
    prose(
        "ui-surface-color-styles-are-still-available-but-no-longer-recommended-prefer-a-container-tone",
    );
}

#[tessera]
fn states() {
    prose(
        "ui-states-communicate-elevation-and-interaction-a-fab-rests-at-level-3-rises-to-level-4-on-ho",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(|| {
                        fab("ui-enabled");
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        fab("ui-disabled").enabled(false);
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        fab("ui-accent").tone(3);
                    });
                gap(Dp(32.0));
                text(I18N.text("ui-hover-focus-with-tab-or-press-the-actions-above-to-see-the-state-layer-and-elevation-respo"))
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-state".into(),
            "ui-elevation".into(),
            "ui-state-layer".into(),
        ],
        vec![
            vec!["ui-enabled".into(), "ui-level-3".into(), "--".into()],
            vec!["ui-hovered".into(), "ui-level-4".into(), "8%".into()],
            vec!["ui-focused".into(), "ui-level-3".into(), "10%".into()],
            vec!["ui-pressed".into(), "ui-level-3".into(), "10%".into()],
            vec![
                "ui-disabled".into(),
                "ui-none".into(),
                "ui-faded-container".into(),
            ],
        ],
    );
}

#[tessera]
fn measurements() {
    prose(
        "ui-height-icon-size-and-corner-radius-scale-together-reserve-16dp-margins-around-a-floating-a",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-attribute".into(),
            "ui-small".into(),
            "FAB".into(),
            "ui-medium".into(),
            "ui-large".into(),
        ],
        vec![
            vec![
                "ui-container".into(),
                "40dp".into(),
                "56dp".into(),
                "80dp".into(),
                "96dp".into(),
            ],
            vec![
                "ui-icon".into(),
                "24dp".into(),
                "24dp".into(),
                "28dp".into(),
                "36dp".into(),
            ],
            vec![
                "ui-corner-radius".into(),
                "12dp".into(),
                "16dp".into(),
                "20dp".into(),
                "28dp".into(),
            ],
            vec![
                "ui-padding-around-icon".into(),
                "8dp".into(),
                "16dp".into(),
                "26dp".into(),
                "30dp".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(24.0))
            .children(move || {
                for size in SIZES {
                    fab(I18N.format(
                        "ui-value-value-format-2",
                        [
                            ("v0", I18N.text(&size_name(size)).into()),
                            ("v1", I18N.text(&size_height(size)).into()),
                        ],
                    ))
                    .size(size);
                }
            });
    }));
    gap(Dp(16.0));
    caption("ui-container-sizes-of-the-small-regular-medium-and-large-fabs");
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-outer-margin".into(), "16dp".into()],
            vec!["ui-minimum-interactive-target".into(), "48 × 48dp".into()],
            vec![
                "ui-icon-to-container".into(),
                "ui-centered-in-the-container".into(),
            ],
        ],
    );
}

#[tessera]
fn baseline_tokens() {
    prose(
        "ui-baseline-fab-token-sets-are-still-published-but-the-surface-color-styles-are-no-longer-rec",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-token".into(), "ui-baseline-value".into()],
        vec![
            vec![
                "md.comp.fab.surface-container.color".into(),
                "Surface container high".into(),
            ],
            vec![
                "md.comp.fab.surface-container.icon.color".into(),
                "Primary".into(),
            ],
            vec!["md.comp.fab.container.height".into(), "56dp".into()],
            vec![
                "md.comp.fab.container.shape".into(),
                "ui-16dp-corners".into(),
            ],
            vec!["md.comp.fab.icon.size".into(), "24dp".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-both-baseline-and-expressive-mappings-keep-the-same-interaction-model-elevation-level-3-at",
    );
}

#[tessera]
fn usage() {
    prose(
        "ui-use-one-fab-for-the-most-common-or-important-action-on-a-screen-keep-the-icon-clear-and-un",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                fab("ui-add");
                gap(Dp(16.0));
                text(I18N.text("ui-do-one-fab-with-a-familiar-icon-for-the-primary-action"))
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(|| {
                        fab("ui-add");
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                        fab("ui-edit").tone(2);
                    });
                gap(Dp(16.0));
                text(I18N.text("ui-don-t-two-fabs-competing-for-equally-important-actions"))
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(16.0));
    caption("ui-a-fab-represents-one-action-not-a-collection-of-actions");
    gap(Dp(32.0));
    bullets(vec![
        "ui-do-not-use-a-fab-for-destructive-or-dismissive-actions".into(),
        "ui-keep-the-icon-understandable-without-extra-explanation".into(),
        "ui-prefer-a-fab-menu-when-several-related-actions-belong-together".into(),
    ]);
}

#[tessera]
fn actions() {
    prose(
        "ui-a-fab-performs-its-action-immediately-when-several-related-actions-belong-together-use-a-f",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-component".into(), "ui-use-when".into()],
        vec![
            vec![
                "FAB".into(),
                "ui-one-primary-action-with-a-universally-understood-icon".into(),
            ],
            vec![
                "ui-extended-fab".into(),
                "ui-the-action-needs-a-visible-text-label".into(),
            ],
            vec![
                "ui-fab-menu".into(),
                "ui-two-to-six-related-actions-belong-to-one-trigger".into(),
            ],
        ],
    );
}

#[tessera]
fn guidelines_anatomy() {
    prose(
        "ui-the-fab-is-a-single-container-with-a-centered-icon-keep-the-icon-simple-and-recognizable-a",
    );
    gap(Dp(24.0));
    anatomy();
}

#[tessera]
fn guidelines_color() {
    prose(
        "ui-default-to-the-primary-container-tone-secondary-and-tertiary-tones-separate-a-floating-act",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                for tone in [0_usize, 1, 2, 3] {
                    fab(TONES[tone]).tone(tone);
                }
            });
    }));
    gap(Dp(16.0));
    caption("ui-keep-one-tone-across-a-screen-so-the-floating-action-stays-identifiable");
}

#[tessera]
fn placement() {
    prose(
        "ui-place-a-fab-at-the-bottom-end-of-the-screen-above-the-bottom-navigation-bar-or-any-bottom",
    );
    gap(Dp(24.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(240.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().fill_max_size().padding_all(Dp(24.0)))
                .children(move || {
                    text(I18N.text("ui-content"))
                        .color(scheme.on_surface_variant)
                        .style(type_style(14.0, 20.0));
                    spacer().modifier(Modifier::new().weight(1.0));
                    row()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            spacer().modifier(Modifier::new().weight(1.0));
                            floating_action_button()
                                .size(FloatingActionButtonSize::Standard)
                                .container_color(scheme.primary_container)
                                .content_color(scheme.on_primary_container)
                                .ripple_color(scheme.on_primary_container)
                                .accessibility_label(I18N.text("ui-add"))
                                .on_click(|| {})
                                .content(|| {
                                    icon().painter(filled::ADD_SVG).size(Dp(24.0));
                                });
                        });
                });
        });
    gap(Dp(16.0));
    caption("ui-fabs-should-have-margins-of-16dp-and-must-not-cover-the-navigation-bar");
}

#[tessera]
fn adaptive() {
    prose(
        "ui-choose-the-size-by-window-class-the-small-and-regular-fabs-suit-compact-layouts-while-the",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                subheading("ui-compact".into());
                gap(Dp(16.0));
                fab("ui-add");
                gap(Dp(32.0));
                subheading("ui-expanded".into());
                gap(Dp(16.0));
                fab("ui-add").size(FloatingActionButtonSize::Large);
            });
    }));
    gap(Dp(16.0));
    caption("ui-the-same-action-at-two-size-presets");
}

#[tessera]
fn behaviors() {
    prose(
        "ui-a-fab-can-hide-while-content-scrolls-down-and-reappear-when-scrolling-up-so-it-never-block",
    );
    gap(Dp(24.0));
    let visible = remember(|| true);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                button()
                    .tonal()
                    .selected(!visible.get())
                    .accessibility_label(I18N.text("ui-hide-or-show-the-floating-action"))
                    .on_click(move || visible.set(!visible.get()))
                    .child(move || {
                        text(I18N.text(if visible.get() {
                            "ui-hide-fab"
                        } else {
                            "ui-show-fab"
                        }));
                    });
                gap(Dp(32.0));
                if visible.get() {
                    fab("ui-add").counted(true);
                } else {
                    text(I18N.text("ui-fab-hidden-while-scrolling")).style(type_style(14.0, 20.0));
                }
            });
    }));
    gap(Dp(24.0));
    prose(
        "ui-hiding-is-a-scroll-behavior-not-a-state-change-the-action-keeps-its-position-and-its-acces",
    );
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-using-assistive-technology");
    gap(Dp(16.0));
    bullets(vec![
        "ui-navigate-to-and-activate-the-fab".into(),
        "ui-identify-the-action-from-its-accessible-label".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("ui-add").counted(true);
    }));
    gap(Dp(16.0));
    caption(
        "ui-the-fab-exposes-one-action-named-by-its-accessibility-label-activating-it-increments-the-c",
    );
}

#[tessera]
fn interaction_style() {
    prose(
        "ui-the-fab-is-a-large-target-the-smallest-container-is-40dp-and-the-interactive-target-never",
    );
    gap(Dp(24.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        row()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                floating_action_button()
                    .size(FloatingActionButtonSize::Standard)
                    .container_color(scheme.primary_container)
                    .content_color(scheme.on_primary_container)
                    .ripple_color(scheme.on_primary_container)
                    .accessibility_label(I18N.text("ui-above-three-to-one-add"))
                    .on_click(|| {})
                    .content(|| {
                        icon().painter(filled::ADD_SVG).size(Dp(24.0));
                    });
                spacer().modifier(Modifier::new().width(Dp(32.0)));
                floating_action_button()
                    .size(FloatingActionButtonSize::Standard)
                    .container_color(scheme.surface_container)
                    .content_color(scheme.outline)
                    .ripple_color(scheme.outline)
                    .accessibility_label(I18N.text("ui-below-three-to-one-add"))
                    .on_click(|| {})
                    .content(|| {
                        icon().painter(filled::ADD_SVG).size(Dp(24.0));
                    });
            });
    }));
    gap(Dp(16.0));
    caption("ui-above-3-1-on-primary-container-versus-a-low-contrast-pairing");
}

#[tessera]
fn focus() {
    prose(
        "ui-a-fab-is-a-single-focus-stop-focus-is-drawn-as-a-ring-around-the-container-and-it-should-f",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("ui-add");
    }));
    gap(Dp(16.0));
    caption("ui-press-tab-to-reach-the-action-the-container-receives-focus-once");
}

#[tessera]
fn layout_position() {
    prose(
        "ui-keep-the-fab-clear-of-navigation-bars-sheets-and-snackbars-it-should-stay-in-the-same-corn",
    );
    gap(Dp(24.0));
    placement();
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec!["ui-tab".into(), "ui-navigate-to-the-fab".into()],
            vec!["ui-space-or-enter".into(), "ui-activate-the-fab".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-the-fab-behaves-like-a-button-for-keyboard-input-one-focus-stop-activated-by-space-or-ente",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("ui-add").counted(true);
    }));
}

#[tessera]
fn labeling() {
    prose(
        "ui-a-fab-has-no-visible-label-so-its-accessibility-label-must-name-the-action-do-not-describe",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                fab("ui-add");
                gap(Dp(16.0));
                text(I18N.text("ui-do-add-button-the-label-names-the-action"))
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                fab("ui-add").tone(1);
                gap(Dp(16.0));
                text(
                    I18N.text(
                        "ui-don-t-plus-icon-describing-the-glyph-does-not-explain-the-action",
                    ),
                )
                .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(32.0));
    prose(
        "ui-keep-the-same-label-when-the-fab-hides-and-returns-and-provide-a-tooltip-only-as-a-visual",
    );
}
