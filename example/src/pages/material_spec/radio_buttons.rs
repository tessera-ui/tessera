//! Interactive radio button specifications.
//!
//! ## Usage
//!
//! Compare exclusive choices, selection feedback and radio button dimensions.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    radio_button::{radio_button, radio_group},
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Dp, FocusRequester, Modifier, RenderSlot, modifier::FocusModifierExt as _, remember,
    tessera, use_context,
};

use super::{
    ACCENT,
    article::{ArticlePage, SpecSection, article},
    gap,
    selection_articles::option_label,
    spec_table, type_style,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "radio_figures_tests.rs"]
mod figures_tests;

#[path = "radio_page_figures.rs"]
mod page_figures;

#[tessera]
pub(super) fn radio_buttons_spec() {
    article(
        "Radio button",
        "Radio buttons let people select one option from a set of options",
        vec![
            SpecSection {
                title: "Anatomy",
                content: RenderSlot::new(|| {
                    page_figures::anatomy(false);
                }),
            },
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
                content: RenderSlot::new(measurements),
            },
        ],
    )
    .intro(ringtone)
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(page_figures::overview_article)),
            sections: vec![],
        },
        ArticlePage {
            title: "Guidelines",
            intro: Some(RenderSlot::new(ringtone)),
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(page_figures::usage),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(|| {
                        page_figures::anatomy(true);
                    }),
                },
                SpecSection {
                    title: "Placement",
                    content: RenderSlot::new(page_figures::placement),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(page_figures::behavior),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Use cases",
                    content: RenderSlot::new(|| {
                        page_figures::accessibility("Use cases".into());
                    }),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(|| {
                        page_figures::accessibility("Interaction & style".into());
                    }),
                },
                SpecSection {
                    title: "Initial focus",
                    content: RenderSlot::new(|| {
                        page_figures::accessibility("Initial focus".into());
                    }),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(|| {
                        page_figures::accessibility("Keyboard navigation".into());
                    }),
                },
                SpecSection {
                    title: "Labeling elements",
                    content: RenderSlot::new(|| {
                        page_figures::accessibility("Labeling elements".into());
                    }),
                },
            ],
        },
    ]);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn overview() {
    let selected = remember(|| 0_usize);
    let width = remember(|| Dp(1024.0));
    let scale = width.get().0 / 1024.0;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                    let next = Dp::from_pixels_f32(bounds.width.to_f32());
                    if width.get() != next {
                        width.set(next);
                    }
                })
                .fill_max_width()
                .height(Dp(300.0 * scale))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(16.0))),
        )
        .children(move || {
            radio_group()
                .accessibility_label("Radio button selection example")
                .content(move || {
                    row()
                        .modifier(Modifier::new().offset(Dp(-7.0 * scale), Dp::ZERO))
                        .children(move || {
                            for index in 0..2 {
                                if index > 0 {
                                    spacer().modifier(
                                        Modifier::new().width(Dp((88.0 * scale - 48.0).max(0.0))),
                                    );
                                }
                                radio_button()
                                    .selected(selected.get() == index)
                                    .size(Dp(20.0 * scale))
                                    .stroke_width(Dp(2.0 * scale))
                                    .dot_size(Dp(10.0 * scale))
                                    .accessibility_label(if index == 0 {
                                        "First option"
                                    } else {
                                        "Second option"
                                    })
                                    .on_select(move |_| selected.set(index));
                            }
                        });
                });
        });
}

#[tessera]
fn ringtone() {
    let selected = remember(|| 1_usize);
    let width = remember(|| Dp(1024.0));
    let scale = width.get().0 / 1024.0;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                    let next = Dp::from_pixels_f32(bounds.width.to_f32());
                    if width.get() != next {
                        width.set(next);
                    }
                })
                .fill_max_width()
                .height(Dp(413.0 * scale))
                .clip_to_bounds()
                .background_with_shape(
                    scheme.secondary_container,
                    Shape::rounded_rectangle(Dp(16.0)),
                ),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(298.0 * scale), Dp(46.0 * scale))
                        .size(Dp(428.0 * scale), Dp(480.0 * scale))
                        .background_with_shape(
                            scheme.surface,
                            Shape::rounded_rectangle(Dp(28.0 * scale)),
                        )
                        .border_with_shape(
                            Dp(8.0 * scale),
                            scheme.outline_variant,
                            Shape::rounded_rectangle(Dp(28.0 * scale)),
                        ),
                )
                .children(move || {
                    boxed().modifier(
                        Modifier::new()
                            .offset(Dp(8.0 * scale), Dp(8.0 * scale))
                            .size(Dp(412.0 * scale), Dp(116.0 * scale))
                            .background_with_shape(
                                scheme.surface_container,
                                Shape::RoundedRectangle {
                                    top_left: RoundedCorner::manual(Dp(20.0 * scale), 3.0),
                                    top_right: RoundedCorner::manual(Dp(20.0 * scale), 3.0),
                                    bottom_left: RoundedCorner::manual(Dp::ZERO, 3.0),
                                    bottom_right: RoundedCorner::manual(Dp::ZERO, 3.0),
                                },
                            ),
                    );
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .offset(Dp(32.0 * scale), Dp(30.0 * scale))
                                .width(Dp(364.0 * scale)),
                        )
                        .children(move || {
                            text()
                                .content("9:30")
                                .style(type_style(14.0 * scale, 20.0 * scale))
                                .modifier(Modifier::new().weight(1.0));
                            icon().vector(filled::WIFI_SVG).size(Dp(16.0 * scale));
                            icon()
                                .vector(filled::SIGNAL_CELLULAR_4_BAR_SVG)
                                .size(Dp(16.0 * scale));
                            icon()
                                .vector(filled::BATTERY_FULL_SVG)
                                .size(Dp(16.0 * scale));
                        });
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .offset(Dp(28.0 * scale), Dp(78.0 * scale))
                                .width(Dp(370.0 * scale)),
                        )
                        .children(move || {
                            icon().vector(filled::ARROW_BACK_SVG).size(Dp(24.0 * scale));
                            spacer().modifier(Modifier::new().width(Dp(20.0 * scale)));
                            text()
                                .content("Ringtone")
                                .style(type_style(22.0 * scale, 28.0 * scale))
                                .modifier(Modifier::new().weight(1.0));
                            icon().vector(filled::MORE_VERT_SVG).size(Dp(24.0 * scale));
                        });
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(12.0 * scale), Dp(144.0 * scale))
                                .width(Dp(392.0 * scale)),
                        )
                        .children(move || {
                            radio_group()
                                .accessibility_label("Ringtone")
                                .content(move || {
                                    column().children(move || {
                                        for (index, label) in
                                            ["None", "Callisto", "Ganymede", "Luna"]
                                                .into_iter()
                                                .enumerate()
                                        {
                                            let focus = remember(FocusRequester::new).get();
                                            row()
                                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                                .modifier(Modifier::new().height(Dp(56.0 * scale)))
                                                .children(move || {
                                                    radio_button()
                                                        .focus_requester(focus)
                                                        .selected(selected.get() == index)
                                                        .size(Dp(20.0 * scale))
                                                        .stroke_width(Dp(2.0 * scale))
                                                        .dot_size(Dp(10.0 * scale))
                                                        .accessibility_label(label)
                                                        .on_select(move |_| selected.set(index));
                                                    spacer().modifier(
                                                        Modifier::new().width(Dp(4.0 * scale)),
                                                    );
                                                    option_label(
                                                        label.into(),
                                                        Callback::new(move || {
                                                            selected.set(index);
                                                            focus.request_focus();
                                                        }),
                                                        true,
                                                    )
                                                    .style(type_style(16.0 * scale, 24.0 * scale));
                                                });
                                        }
                                    });
                                });
                        });
                });
        });
}

#[tessera]
fn choices(#[prop(into)] label: String, enabled: bool) {
    let selected = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    column().children(move || {
        text().content(label.clone()).style(type_style(24.0, 32.0));
        gap(Dp(16.0));
        let label = label.clone();
        radio_group()
            .accessibility_label(label.clone())
            .content(move || {
                let label = label.clone();
                column().children(move || {
                    for (index, option) in ["All notifications", "Important only", "None"]
                        .into_iter()
                        .enumerate()
                    {
                        let focus = remember(FocusRequester::new).get();
                        let label = label.clone();
                        row()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                radio_button()
                                    .focus_requester(focus)
                                    .selected(selected.get() == index)
                                    .enabled(enabled)
                                    .on_select(move |_| selected.set(index))
                                    .accessibility_label(format!("{label}: {option}"));
                                option_label(
                                    option.into(),
                                    Callback::new(move || {
                                        selected.set(index);
                                        focus.request_focus();
                                    }),
                                    enabled,
                                )
                                .color(if enabled {
                                    scheme.on_surface
                                } else {
                                    scheme.on_surface.with_alpha(0.38)
                                });
                            });
                    }
                });
            });
    });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Selected".into(), "Unselected".into()],
        vec![
            vec!["Icon".into(), "Primary".into(), "On surface variant".into()],
            vec![
                "Disabled icon".into(),
                "On surface · 38%".into(),
                "On surface · 38%".into(),
            ],
            vec![
                "Hovered state layer".into(),
                "Primary · 8%".into(),
                "On surface · 8%".into(),
            ],
            vec![
                "Focused / pressed state layer".into(),
                "Primary · 10%".into(),
                "On surface · 10%".into(),
            ],
            vec![
                "Adjacent label".into(),
                "On surface".into(),
                "On surface".into(),
            ],
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
                        .modifier(Modifier::new().padding_all(Dp(24.0)))
                        .children(move || {
                            choices(if dark { "Dark theme" } else { "Light theme" }, true);
                        });
                });
        });
}

#[tessera]
fn colors() {
    page_figures::colors();
}

#[tessera]
fn states() {
    page_figures::states();
}

#[tessera]
fn measurements() {
    page_figures::measurements();
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use tessera_ui::{HeadlessConfig, HeadlessRenderer};

    use super::*;

    struct Pipelines;

    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[tessera]
    fn fixture() {
        scheme(true);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn dark_radio_scheme_paints_the_content_padding() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 400,
                height: 300,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(4);
        let pixels = renderer.read_pixels().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 400 + x) * 4..][..4];
        assert_eq!(
            pixel(8, 8),
            pixel(200, 60),
            "the outer inset must share the dark surface fill"
        );
        assert!(pixel(8, 8)[..3].iter().all(|channel| *channel < 100));
    }
}
