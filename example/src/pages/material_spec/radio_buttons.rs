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
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Callback, Dp, FocusRequester, Modifier, RenderSlot, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
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
pub(super) fn radio_buttons_spec(
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    article(
        "ui-radio-button",
        "ui-radio-buttons-let-people-select-one-option-from-a-set-of-options",
        vec![
            SpecSection {
                title: "ui-anatomy",
                content: RenderSlot::new(|| {
                    page_figures::anatomy(false);
                }),
            },
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
            intro: Some(RenderSlot::new(page_figures::overview_article)),
            sections: vec![],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-usage",
                    content: RenderSlot::new(|| {
                        page_figures::usage();
                        gap(Dp(24.0));
                        ringtone();
                    }),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(|| {
                        page_figures::anatomy(true);
                    }),
                },
                SpecSection {
                    title: "ui-placement",
                    content: RenderSlot::new(page_figures::placement),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(page_figures::behavior),
                },
            ],
        },
        ArticlePage {
            title: "ui-accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-use-cases",
                    content: RenderSlot::new(|| {
                        page_figures::accessibility("ui-use-cases".into());
                    }),
                },
                SpecSection {
                    title: "ui-interaction-style",
                    content: RenderSlot::new(|| {
                        page_figures::accessibility("ui-interaction-style".into());
                    }),
                },
                SpecSection {
                    title: "ui-initial-focus",
                    content: RenderSlot::new(|| {
                        page_figures::accessibility("ui-initial-focus".into());
                    }),
                },
                SpecSection {
                    title: "ui-keyboard-navigation",
                    content: RenderSlot::new(|| {
                        page_figures::accessibility("ui-keyboard-navigation".into());
                    }),
                },
                SpecSection {
                    title: "ui-labeling-elements",
                    content: RenderSlot::new(|| {
                        page_figures::accessibility("ui-labeling-elements".into());
                    }),
                },
            ],
        },
    ]);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(I18N.text(value))
        .style(type_style(16.0, 24.0));
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
                .on_layout_size_changed(move |size: tessera_ui::PxSize| {
                    let next = Dp::from_pixels_f32(size.width.to_f32());
                    width.set_if_changed(next);
                })
                .fill_max_width()
                .height(Dp(300.0 * scale))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(16.0))),
        )
        .children(move || {
            radio_group()
                .accessibility_label(I18N.text("ui-radio-button-selection-example"))
                .content(move |group| {
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
                                    .group(group)
                                    .selected(selected.get() == index)
                                    .size(Dp(20.0 * scale))
                                    .stroke_width(Dp(2.0 * scale))
                                    .dot_size(Dp(10.0 * scale))
                                    .accessibility_label(I18N.text(if index == 0 {
                                        "ui-first-option"
                                    } else {
                                        "ui-second-option"
                                    }))
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
                .on_layout_size_changed(move |size: tessera_ui::PxSize| {
                    let next = Dp::from_pixels_f32(size.width.to_f32());
                    width.set_if_changed(next);
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
                                .content(I18N.text("9:30"))
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
                                .content(I18N.text("ui-ringtone"))
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
                                .accessibility_label(I18N.text("ui-ringtone"))
                                .content(move |group| {
                                    column().children(move || {
                                        for (index, label) in
                                            ["ui-none", "ui-callisto", "ui-ganymede", "ui-luna"]
                                                .into_iter()
                                                .enumerate()
                                        {
                                            let focus = remember(FocusRequester::new).get();
                                            row()
                                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                                .modifier(Modifier::new().height(Dp(56.0 * scale)))
                                                .children(move || {
                                                    radio_button()
                                                        .group(group)
                                                        .focus_requester(focus)
                                                        .selected(selected.get() == index)
                                                        .size(Dp(20.0 * scale))
                                                        .stroke_width(Dp(2.0 * scale))
                                                        .dot_size(Dp(10.0 * scale))
                                                        .accessibility_label(I18N.text(label))
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
fn tokens() {
    spec_table(
        vec![
            "ui-element".into(),
            "ui-selected".into(),
            "ui-unselected".into(),
        ],
        vec![
            vec![
                "ui-icon".into(),
                "ui-primary".into(),
                "ui-on-surface-variant".into(),
            ],
            vec![
                "ui-disabled-icon".into(),
                "ui-on-surface-38".into(),
                "ui-on-surface-38".into(),
            ],
            vec![
                "ui-hovered-state-layer".into(),
                "ui-primary-8".into(),
                "ui-on-surface-8".into(),
            ],
            vec![
                "ui-focused-pressed-state-layer".into(),
                "ui-primary-10".into(),
                "ui-on-surface-10".into(),
            ],
            vec![
                "ui-adjacent-label".into(),
                "ui-on-surface".into(),
                "ui-on-surface".into(),
            ],
        ],
    );
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
