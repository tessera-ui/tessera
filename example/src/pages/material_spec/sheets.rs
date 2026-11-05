use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    bottom_sheet::{BottomSheetController, BottomSheetExpansion, bottom_sheet_provider},
    boxed::boxed,
    button::button,
    checkbox::checkbox,
    column::column,
    icon::icon,
    icon_button::icon_button,
    image::image,
    list_item::list_item,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    text_field::text_field,
    theme::MaterialTheme,
};
use tessera_ui::{
    Callback, Color, Dp, LayoutCoordinates, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use crate::res::{I18N, material_spec as assets};

use super::{
    INK,
    annotations::{self, PreciseGeometry as Geometry},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[tessera]
fn prose(#[prop(into)] value: String) {
    text(I18N.text(value))
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
pub(super) fn sheets_spec(
    side: bool,
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    // The side sheet article lives in its own module; this page always presents
    // the bottom sheet specification.
    let _ = side;
    let action = remember(|| String::from("ui-no-action-selected"));
    let controller = remember(|| BottomSheetController::new(false));
    let main = RenderSlot::new(move || {
        sheet_article(action, controller).route_optional(route.clone());
    });
    let body = RenderSlot::new(move || {
        sheet_content(Callback::new(move || close_sheet(controller)), action);
    });
    bottom_sheet_provider()
        .controller(controller)
        .accessibility_label(I18N.text("ui-share-this-place"))
        .partial_height(Dp(360.0))
        .on_close_request(move || close_sheet(controller))
        .main_content_shared(main)
        .bottom_sheet_content_shared(body);
}

fn close_sheet(controller: State<BottomSheetController>) {
    controller.with_mut(BottomSheetController::close);
}

#[tessera]
fn sheet_article(
    action: State<String>,
    controller: State<BottomSheetController>,
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    article(
        "ui-bottom-sheets",
        "ui-bottom-sheets-show-secondary-content-anchored-to-the-bottom-of-the-screen",
        vec![
            SpecSection {
                title: "ui-tokens-and-specs",
                content: RenderSlot::new(|| {
                    tokens();
                }),
            },
            SpecSection {
                title: "ui-color",
                content: RenderSlot::new(|| {
                    sheet_colors_section();
                }),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(|| {
                    sheet_measurements_section();
                }),
            },
        ],
    ).route_optional(route.clone())
    .intro(|| {
        prose("ui-modal-bottom-sheets-are-above-a-scrim-while-standard-bottom-sheets-don-t-have-a-scrim-besi");
        gap(Dp(32.0));
        figures::anatomy_figure(false);
        figures::badge_legend(vec![
            "ui-container".into(),
            "ui-drag-handle-optional".into(),
            "Scrim".into(),
        ]);
    })
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(|| { overview_intro(); })),
            sections: vec![
                SpecSection {
                    title: "ui-availability-resources",
                    content: RenderSlot::new(|| { availability(); }),
                },
                SpecSection {
                    title: "ui-differences-from-m2",
                    content: RenderSlot::new(|| { differences_from_m2(); }),
                },
            ],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: Some(RenderSlot::new(|| {
                figures::variants_figure();
                figures::badge_legend(vec![
                    "ui-standard-bottom-sheets".into(),
                    "ui-modal-bottom-sheets".into(),
                ]);
            })),
            sections: vec![
                SpecSection {
                    title: "ui-usage",
                    content: RenderSlot::new(|| { usage(); }),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(|| { guide_anatomy(); }),
                },
                SpecSection {
                    title: "ui-standard-bottom-sheets",
                    content: RenderSlot::new(|| { standard_sheets(); }),
                },
                SpecSection {
                    title: "ui-modal-bottom-sheets",
                    content: RenderSlot::new(move || {
                        modal_sheets(controller);
                    }),
                },
                SpecSection {
                    title: "ui-responsive-layout",
                    content: RenderSlot::new(|| { responsive_layout(); }),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(move || {
                        behavior_section(action, controller);
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
                    content: RenderSlot::new(|| { use_cases(); }),
                },
                SpecSection {
                    title: "ui-interaction-style",
                    content: RenderSlot::new(|| { interaction_style(); }),
                },
                SpecSection {
                    title: "ui-keyboard-navigation",
                    content: RenderSlot::new(|| { keyboard_navigation(); }),
                },
                SpecSection {
                    title: "ui-labeling",
                    content: RenderSlot::new(|| { labeling(); }),
                },
            ],
        },
    ]);
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
            text(format!("•  {}", I18N.text(item)))
                .style(type_style(16.0, 24.0))
                .color(INK);
            gap(Dp(8.0));
        }
    });
}

#[tessera]
fn overview_intro() {
    bullets(vec![
        "ui-use-bottom-sheets-in-compact-and-medium-breakpoints".into(),
        "ui-two-variants-standard-and-modal".into(),
        "ui-content-should-be-additional-or-secondary-not-the-app-s-main-content".into(),
        "ui-bottom-sheets-can-be-dismissed-in-order-to-interact-with-the-main-content".into(),
    ]);
    gap(Dp(32.0));
    figures::variants_figure();
    figures::badge_legend(vec![
        "ui-standard-bottom-sheet".into(),
        "ui-modal-bottom-sheet".into(),
    ]);
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
                "Android Views (MDC-Android)".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Jetpack Compose".into(),
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
fn differences_from_m2() {
    figures::split()
        .start(|| {
            bullets(vec![
                "ui-color-new-color-mappings-and-compatibility-with-dynamic-color".into(),
                "ui-shape-bottom-sheets-have-a-28dp-top-corner-radius".into(),
                "ui-layout-new-max-width-of-640dp-and-an-optional-drag-handle-with-an-accessible-48dp-hit-targ".into(),
            ]);
        })
        .end(|| {
            figures::m2_figure();
        });
}

#[tessera]
fn usage() {
    figures::split()
        .start(|| {
            prose("ui-bottom-sheets-display-supplementary-content-and-actions-on-a-mobile-screen");
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::share_figure();
                }),
                "ui-bottom-sheet-containing-contacts-and-applications",
            );
        });
    figures::split()
        .start(|| {
            prose(
                "ui-bottom-sheets-are-a-versatile-component-that-can-contain-a-wide-variety-of-information-and",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::menu_figure(false);
                }),
                "ui-bottom-sheet-with-menu-items-in-a-list",
            );
        });
}

#[tessera]
fn guide_anatomy() {
    prose(
        "ui-a-container-is-the-only-required-element-of-a-bottom-sheet-bottom-sheet-layouts-can-vary-w",
    );
    gap(Dp(32.0));
    figures::anatomy_figure(true);
    figures::badge_legend(vec![
        "ui-container".into(),
        "ui-drag-handle-optional".into(),
        "ui-scrim-modal-only".into(),
    ]);
    gap(Dp(32.0));
    subheading("ui-container".into());
    prose(
        "ui-bottom-sheets-are-flexible-containers-that-adapt-to-their-content-and-available-space-the",
    );
    gap(Dp(24.0));
    subheading("ui-drag-handle-optional".into());
    prose(
        "ui-the-drag-handle-shows-that-the-sheet-can-move-between-heights-it-keeps-a-48-48dp-target-an",
    );
    gap(Dp(24.0));
    subheading("ui-scrim-modal-only".into());
    prose(
        "ui-a-modal-sheet-appears-above-a-scrim-that-blocks-the-rest-of-the-app-selecting-the-scrim-al",
    );
    gap(Dp(24.0));
    subheading("ui-list-items-optional".into());
    prose(
        "ui-lists-are-a-continuous-group-of-text-or-images-list-items-can-include-label-text-icons-and",
    );
    gap(Dp(24.0));
    subheading("ui-dividers-optional".into());
    prose("ui-dividers-can-be-used-to-separate-related-content-in-bottom-sheets");
    gap(Dp(24.0));
    subheading("ui-media-optional".into());
    prose("ui-bottom-sheets-can-contain-thumbnails-images-and-video");
}

#[tessera]
fn standard_sheets() {
    prose(
        "ui-standard-bottom-sheets-co-exist-with-the-screen-s-main-ui-region-and-allow-for-simultaneou",
    );
    gap(Dp(24.0));
    prose(
        "ui-use-a-standard-bottom-sheet-to-display-content-that-complements-the-screen-s-primary-conte",
    );
    gap(Dp(24.0));
    music_context();
    gap(Dp(24.0));
    prose(
        "ui-a-bottom-sheet-can-have-preset-positions-from-full-screen-height-to-preview-at-full-screen",
    );
}

#[tessera]
fn modal_sheets(controller: State<BottomSheetController>) {
    figures::split()
        .start(|| {
            prose(
                "ui-like-dialogs-modal-bottom-sheets-appear-in-front-of-app-content-disabling-all-other-app-fu",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::files_figure();
                }),
                "ui-a-modal-bottom-sheet-must-be-interacted-with-or-dismissed-its-blocking-behavior-makes-it-s",
            );
        });
    figures::split()
        .start(move || {
            prose(
                "ui-use-a-modal-bottom-sheet-as-an-alternative-to-inline-menus-or-simple-dialogs-on-mobile-esp",
            );
            gap(Dp(24.0));
            launch(controller);
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::menu_figure(true);
                }),
                "ui-modal-bottom-sheets-can-be-used-instead-of-menus-to-present-additional-actions",
            );
        });
    subheading("ui-visibility".into());
    figures::split()
        .start(|| {
            prose(
                "ui-to-provide-access-to-its-top-actions-the-initial-vertical-position-of-modal-bottom-sheets",
            );
            gap(Dp(16.0));
            prose(
                "ui-modal-bottom-sheets-whose-contents-exceed-50-of-the-screen-height-can-then-be-pulled-acros",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::photo_actions_figure(314.0);
                }),
                "ui-the-initial-vertical-position-of-modal-bottom-sheets-can-t-exceed-50-of-the-screen-height",
            );
        });
    figures::split()
        .start(|| {
            prose(
                "ui-modal-bottom-sheets-appear-when-triggered-by-a-user-action-such-as-tapping-a-button-or-an",
            );
            gap(Dp(12.0));
            bullets(vec![
                "ui-tapping-a-menu-item-or-action-within-the-bottom-sheet".into(),
                "ui-tapping-the-scrim".into(),
                "ui-swiping-the-sheet-down".into(),
                "ui-using-a-close-affordance-within-the-bottom-sheet-s-app-bar-if-available".into(),
            ]);
            prose("ui-display-a-close-affordance-in-a-full-screen-modal-bottom-sheet");
        })
        .end(|| {
            figures::figure_pair(
                RenderSlot::new(|| {
                    figures::dismiss_scrim_figure();
                }),
                "ui-tapping-the-scrim-dismisses-a-modal-bottom-sheet",
                RenderSlot::new(|| {
                    figures::dismiss_swipe_figure();
                }),
                "ui-a-modal-bottom-sheet-can-be-dismissed-by-swiping-the-sheet-down",
            );
        });
}

#[tessera]
fn responsive_layout() {
    figures::split()
        .start(|| {
            subheading("ui-compact-breakpoint".into());
            prose(
                "ui-in-compact-breakpoints-like-mobile-devices-bottom-sheets-extend-across-the-width-of-a-scre",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::files_figure();
                }),
                "ui-bottom-sheets-should-extend-to-the-width-of-the-screen-on-mobile",
            );
        });
    figures::split()
        .start(|| {
            subheading("ui-medium-and-expanded-breakpoints".into());
            prose(
                "ui-for-larger-screens-with-medium-and-expanded-breakpoints-bottom-sheets-have-a-default-max-w",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::tablet_figure();
                }),
                "ui-bottom-sheets-on-larger-screens-like-tablet-have-a-max-width-that-can-be-overridden",
            );
        });
    figures::split()
        .start(|| {
            prose(
                "ui-on-larger-expanded-breakpoints-like-desktop-a-bottom-sheet-can-be-swapped-for-a-side-sheet",
            );
        })
        .end(|| {
            filter_context();
        });
}

#[tessera]
fn behavior_section(action: State<String>, controller: State<BottomSheetController>) {
    subheading("ui-expansion".into());
    prose(
        "ui-bottom-sheets-can-offer-an-expansion-option-where-the-sheet-is-fully-raised-and-toggled-be",
    );
    gap(Dp(24.0));
    figures::figure_pair(
        RenderSlot::new(|| {
            figures::photo_actions_figure(48.0);
        }),
        "ui-a-bottom-sheet-for-sharing-can-appear-fully-raised-if-needed",
        RenderSlot::new(|| {
            figures::photo_actions_figure(470.0);
        }),
        "ui-alternately-a-bottom-sheet-for-sharing-can-appear-collapsed-for-a-more-focused-set-of-acti",
    );
    expansion_controls(controller);
    gap(Dp(32.0));
    subheading("ui-custom-positioning".into());
    prose(
        "ui-the-drag-handle-can-be-dragged-or-selected-to-change-the-bottom-sheet-height-sheets-should",
    );
    gap(Dp(16.0));
    prose(
        "ui-if-the-bottom-sheet-has-multiple-preset-heights-but-can-t-use-a-drag-handle-material-requi",
    );
    gap(Dp(24.0));
    figures::figure_pair(
        RenderSlot::new(|| {
            figures::drag_figure(false);
        }),
        "ui-interacting-with-the-drag-handle-can-quickly-move-a-bottom-sheet-through-preset-heights",
        RenderSlot::new(|| {
            figures::drag_figure(true);
        }),
        "ui-a-bottom-sheet-can-automatically-resize-to-another-height-after-interacting-with-the-drag",
    );
    subheading("ui-scrolling".into());
    prose(
        "ui-bottom-sheets-should-be-scrollable-when-their-content-exceeds-the-initial-viewable-height",
    );
    gap(Dp(24.0));
    subheading("ui-back".into());
    prose(
        "ui-a-predictive-back-gesture-previews-the-result-of-the-gesture-release-to-commit-fling-to-co",
    );
    gap(Dp(24.0));
    launch(controller);
    gap(Dp(16.0));
    caption(I18N.format(
        "ui-last-sheet-action-value",
        [("v0", I18N.text(&action.get()).into())],
    ));
}

#[tessera]
fn expansion_controls(controller: State<BottomSheetController>) {
    let open = controller.with(BottomSheetController::is_open);
    let expansion = controller.with(BottomSheetController::expansion);
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            button()
                .outlined()
                .accessibility_label(I18N.text("ui-open-expanded-bottom-sheet"))
                .on_click(move || {
                    controller.with_mut(|sheet: &mut BottomSheetController| {
                        sheet.open();
                        sheet.expand();
                    });
                })
                .child(move || {
                    text(I18N.text("ui-open-expanded"));
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            button()
                .outlined()
                .accessibility_label(I18N.text("ui-open-partially-expanded-bottom-sheet"))
                .on_click(move || {
                    controller.with_mut(|sheet: &mut BottomSheetController| {
                        sheet.open();
                        sheet.partially_expand();
                    });
                })
                .child(move || {
                    text(I18N.text("ui-open-partial"));
                });
        });
    gap(Dp(16.0));
    caption(
        I18N.format(
            "ui-sheet-value",
            [(
                "v0",
                I18N.text(
                    &(match (open, expansion) {
                        (false, _) => "ui-closed",
                        (true, BottomSheetExpansion::Expanded) => "ui-open-expanded-6986",
                        (true, BottomSheetExpansion::PartiallyExpanded) => {
                            "ui-open-partially-expanded"
                        }
                    }),
                )
                .into(),
            )],
        ),
    );
}

#[tessera]
fn use_cases() {
    prose("ui-users-should-be-able-to");
    gap(Dp(16.0));
    bullets(vec![
        "ui-resize-bottom-sheets-without-having-to-rely-on-touch-gestures".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    figures::split()
        .start(|| {
            subheading("ui-touch-target-area".into());
            prose(
                "ui-the-top-48dp-portion-of-the-bottom-sheet-is-interactive-when-user-initiated-resizing-is-av",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::touch_area_figure();
                }),
                "ui-to-ensure-touch-target-accessibility-the-top-portion-of-a-bottom-sheet-can-be-reserved-for",
            );
        });
    figures::split()
        .start(|| {
            subheading("ui-initial-focus".into());
            prose(
                "ui-the-optional-drag-handle-can-be-focused-in-the-tab-order-and-interacted-with-using-non-tou",
            );
            gap(Dp(16.0));
            prose(
                "ui-when-a-modal-bottom-sheet-opens-focus-moves-into-the-sheet-and-returns-to-the-control-that",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::focus_figure();
                }),
                "ui-visible-focus-shown-on-the-drag-handle-affordance",
            );
        });
    figures::split()
        .start(|| {
            subheading("ui-dragging".into());
            prose(
                "ui-include-a-single-pointer-alternative-for-any-action-that-can-be-completed-by-dragging",
            );
            gap(Dp(16.0));
            prose(
                "ui-drag-handles-should-cycle-the-bottom-sheet-through-available-heights-when-selected-if-a-dr",
            );
        })
        .end(|| {
            figures::figure_pair(
                RenderSlot::new(|| {
                    figures::drag_figure(false);
                }),
                "ui-interacting-with-the-drag-handle-can-quickly-move-a-bottom-sheet-through-preset-heights",
                RenderSlot::new(|| {
                    figures::drag_figure(true);
                }),
                "ui-a-bottom-sheet-can-automatically-resize-to-another-height-after-interacting-with-the-drag",
            );
        });
    touch_target();
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec!["ui-tab".into(), "ui-focus-lands-on-drag-handle".into()],
            vec![
                "ui-space-enter".into(),
                "ui-toggles-between-available-heights".into(),
            ],
        ],
    );
    gap(Dp(16.0));
    prose(
        "ui-in-the-modal-sheet-escape-or-selecting-the-scrim-dismisses-the-sheet-without-changing-the",
    );
}

#[tessera]
fn labeling() {
    figures::split()
        .start(|| {
            prose(
                "ui-label-only-the-drag-handle-the-accessibility-role-for-the-drag-handle-is-button",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::label_figure();
                }),
                "ui-label-the-drag-handle",
            );
        });
}

#[tessera]
fn sheet_colors_section() {
    prose(
        "ui-color-values-are-implemented-through-design-tokens-for-design-this-means-working-with-colo",
    );
    gap(Dp(24.0));
    color::sheet_colors(false, true);
    gap(Dp(16.0));
    caption("ui-bottom-sheet-color-roles-used-for-both-light-and-dark-schemes");
    figures::badge_legend(vec![
        "Scrim*".into(),
        "On surface variant".into(),
        "Surface container low".into(),
    ]);
    caption(
        "ui-on-android-platforms-the-scrim-color-and-opacity-is-automatically-handled-by-the-system-ui",
    );
}

#[tessera]
fn sheet_measurements_section() {
    figures::captioned(
        RenderSlot::new(|| {
            figures::measurements_figure();
        }),
        "ui-bottom-sheet-padding-and-size-measurements",
    );
    prose(
        "ui-bottom-sheets-span-the-full-window-width-up-to-640dp-when-the-window-width-exceeds-640dp-b",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec![
                "ui-drag-handle-alignment-horizontal".into(),
                "ui-center".into(),
            ],
            vec!["ui-drag-handle-padding-top-bottom".into(), "22dp".into()],
            vec!["ui-top-margin".into(), "72dp".into()],
            vec!["ui-top-margin-window-width-640dp".into(), "56dp".into()],
            vec![
                "ui-start-end-margin-window-width-640dp".into(),
                "56dp".into(),
            ],
            vec![
                "ui-width".into(),
                "ui-full-width-up-to-max-width-640dp".into(),
            ],
            vec!["ui-height".into(), "ui-variable".into()],
        ],
    );
}

#[tessera]
fn launch(controller: State<BottomSheetController>) {
    button()
        .outlined()
        .accessibility_label(I18N.text("ui-open-modal-bottom-sheet"))
        .on_click(move || {
            controller.with_mut(BottomSheetController::open);
        })
        .child(move || {
            text(I18N.text("ui-open-modal-bottom-sheet"));
        });
}

#[tessera]
fn sheet_content(close: Callback, action: State<String>) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    column()
        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    text(I18N.text("ui-share-this-place"))
                        .style(theme.typography.title_large)
                        .modifier(Modifier::new().weight(1.0));
                    icon_button()
                        .standard()
                        .icon(filled::CLOSE_SVG)
                        .accessibility_label(I18N.text("ui-close-bottom-sheet"))
                        .on_click_shared(close);
                });
            gap(Dp(16.0));
            for (index, label) in ["ui-share", "ui-get-link", "ui-save"]
                .into_iter()
                .enumerate()
            {
                list_item()
                    .headline(I18N.text(label))
                    .accessibility_label(
                        I18N.format("ui-bottom-sheet-value", [("v0", I18N.text(&label).into())]),
                    )
                    .leading(move || {
                        icon()
                            .painter(match index {
                                0 => filled::SHARE_SVG,
                                1 => filled::LINK_SVG,
                                _ => filled::BOOKMARK_SVG,
                            })
                            .size(Dp(24.0));
                    })
                    .on_click(move || {
                        action.set(
                            I18N.format("ui-value-selected", [("v0", I18N.text(&label).into())]),
                        );
                        close.call();
                    });
            }
            gap(Dp(24.0));
            button()
                .text()
                .accessibility_label(I18N.text("ui-dismiss-sheet"))
                .on_click_shared(close)
                .child(move || {
                    text(I18N.text("ui-cancel"));
                });
        });
}

const TOKEN_GROUPS: [&str; 3] = ["ui-container", "ui-drag-handle", "ui-focus-indicator"];

fn token_values(
    group: usize,
    scheme: &tessera_components::theme::MaterialColorScheme,
) -> Vec<(&'static str, String, Option<Color>)> {
    match group {
        0 => vec![
            (
                "ui-docked-container-color",
                "md.sys.color.surface-container-low".into(),
                Some(scheme.surface_container_low),
            ),
            (
                "ui-surface-tint-layer-color",
                "md.sys.color.surface-tint".into(),
                Some(scheme.surface_tint),
            ),
            (
                "ui-modal-container-elevation",
                "md.sys.elevation.level1".into(),
                None,
            ),
            (
                "ui-standard-container-elevation",
                "md.sys.elevation.level1".into(),
                None,
            ),
            (
                "ui-container-shape",
                "md.sys.shape.corner.extra-large.top (28dp)".into(),
                None,
            ),
            (
                "ui-minimized-container-shape",
                "md.sys.shape.corner.none".into(),
                None,
            ),
        ],
        1 => vec![
            (
                "ui-drag-handle-color",
                "md.sys.color.on-surface-variant".into(),
                Some(scheme.on_surface_variant),
            ),
            ("ui-drag-handle-opacity", "0.4".into(), None),
            ("ui-drag-handle-width", "32dp".into(), None),
            ("ui-drag-handle-height", "4dp".into(), None),
        ],
        _ => vec![
            (
                "ui-focus-indicator-color",
                "md.sys.color.secondary".into(),
                Some(scheme.secondary),
            ),
            ("ui-focus-indicator-thickness", "3dp".into(), None),
            ("ui-focus-indicator-outline-offset", "2dp".into(), None),
        ],
    }
}

#[tessera]
fn tokens() {
    prose("ui-browse-the-component-elements-attributes-tokens-and-their-values");
    gap(Dp(24.0));
    token_browser();
}

/// Expandable token folders for the enabled bottom sheet state.
#[tessera]
fn token_browser() {
    let open = remember(|| [false; 4]);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(16.0)),
                )
                .border_with_shape(
                    Dp(1.0),
                    scheme.outline_variant,
                    Shape::rounded_rectangle(Dp(16.0)),
                )
                .clip_shape(Shape::rounded_rectangle(Dp(16.0))),
        )
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .padding_symmetric(Dp(16.0), Dp(12.0)),
                )
                .children(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .padding_symmetric(Dp(12.0), Dp(6.0))
                                .background_with_shape(
                                    scheme.secondary_container,
                                    Shape::rounded_rectangle(Dp(8.0)),
                                ),
                        )
                        .children(move || {
                            text(I18N.text("ui-sheets-bottom"))
                                .style(type_style(14.0, 20.0))
                                .color(scheme.on_secondary_container);
                        });
                    spacer().modifier(Modifier::new().weight(1.0));
                    text(I18N.text("ui-default-light"))
                        .style(type_style(14.0, 20.0))
                        .color(scheme.on_surface_variant);
                });
            token_folder("ui-enabled", 0, 0, open);
            if open.get()[0] {
                for (index, name) in TOKEN_GROUPS.into_iter().enumerate() {
                    token_folder(name, index + 1, 1, open);
                    if open.get()[index + 1] {
                        for (name, value, swatch) in token_values(index, &scheme) {
                            match swatch {
                                Some(color) => {
                                    token_row(name, value).swatch(color);
                                }
                                None => {
                                    token_row(name, value);
                                }
                            }
                        }
                    }
                }
            }
        });
}

#[tessera]
fn token_folder(#[prop(into)] name: String, index: usize, depth: usize, open: State<[bool; 4]>) {
    let expanded = open.get()[index];
    list_item()
        .headline(I18N.text(&(name)))
        .accessibility_label(I18N.format("ui-value-tokens", [("v0", I18N.text(&name).into())]))
        .accessibility_description(I18N.text(if expanded {
            "ui-expanded"
        } else {
            "ui-collapsed"
        }))
        .content_padding(tessera_components::modifier::Padding::new(
            Dp(16.0 + depth as f64 * 24.0),
            Dp(8.0),
            Dp(16.0),
            Dp(8.0),
        ))
        .leading(|| {
            icon().painter(filled::FOLDER_SVG).size(Dp(20.0));
        })
        .trailing(move || {
            icon()
                .painter(if expanded {
                    filled::EXPAND_LESS_SVG
                } else {
                    filled::EXPAND_MORE_SVG
                })
                .size(Dp(24.0));
        })
        .on_click(move || {
            open.with_mut(|open| open[index] = !open[index]);
        });
}

#[tessera]
fn token_row(#[prop(into)] name: String, value: String, swatch: Option<Color>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(64.0), Dp(10.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::ListItem),
                    label: Some(I18N.format(
                        "ui-value-value-format-1",
                        [
                            ("v0", I18N.text(&name).into()),
                            ("v1", I18N.text(&value).into()),
                        ],
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text(I18N.text(&(name)))
                .style(type_style(14.0, 20.0))
                .color(INK)
                .modifier(Modifier::new().weight(1.0));
            if let Some(color) = swatch {
                spacer().modifier(
                    Modifier::new()
                        .size(Dp(20.0), Dp(20.0))
                        .background_with_shape(color, Shape::Ellipse)
                        .border_with_shape(Dp(1.0), scheme.outline_variant, Shape::Ellipse),
                );
                spacer().modifier(Modifier::new().width(Dp(8.0)));
            }
            text(I18N.text(&(value)))
                .style(type_style(14.0, 20.0))
                .color(scheme.on_surface_variant)
                .modifier(Modifier::new().weight(1.0));
        });
}

#[tessera]
fn touch_target() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let controller = remember(|| BottomSheetController::new(false));
    let close = Callback::new(move || {
        controller.with_mut(BottomSheetController::close);
    });
    let main = RenderSlot::new(move || {
        boxed()
            .alignment(Alignment::Center)
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                button()
                    .accessibility_label(I18N.text("ui-open-native-bottom-sheet-handle"))
                    .on_click(move || {
                        controller.with_mut(BottomSheetController::open);
                    })
                    .child(move || {
                        text(I18N.text("ui-open-bottom-sheet"));
                    });
            });
    });
    let body = RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0)))
            .children(move || {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        text(I18N.text("ui-sheet-controls"))
                            .style(type_style(20.0, 28.0))
                            .modifier(Modifier::new().weight(1.0));
                        icon_button()
                            .standard()
                            .icon(filled::CLOSE_SVG)
                            .accessibility_label(I18N.text("ui-close-native-bottom-sheet-controls"))
                            .on_click_shared(close);
                    });
                gap(Dp(8.0));
                text(I18N.text("ui-select-the-drag-handle-to-change-height"))
                    .style(type_style(14.0, 20.0));
            });
    });
    subheading("ui-try-it".into());
    super::panel(RenderSlot::new(move || {
        boxed()
            .modifier(
                Modifier::new()
                    .width(Dp(400.0))
                    .height(Dp(360.0))
                    .clip_to_bounds(),
            )
            .children(move || {
                bottom_sheet_provider()
                    .controller(controller)
                    .partial_height(Dp(192.0))
                    .accessibility_label(I18N.text("ui-native-bottom-sheet-controls"))
                    .on_close_request_shared(close)
                    .main_content_shared(main)
                    .bottom_sheet_content_shared(body);
            });
        gap(Dp(12.0));
        text(
            I18N.format(
                "ui-48dp-drag-handle-value-tab-to-the-handle-then-space-or-enter-changes-height",
                [(
                    "v0",
                    I18N.text(
                        &(if controller.with(BottomSheetController::expansion)
                            == BottomSheetExpansion::Expanded
                        {
                            "ui-expanded"
                        } else {
                            "ui-partially-expanded"
                        }),
                    )
                    .into(),
                )],
            ),
        )
        .color(scheme.on_surface);
    }));
}

#[tessera]
fn sheet_context_text(
    content: String,
    x: f64,
    y: f64,
    size: f64,
    line: f64,
    scale: f64,
    color: Color,
) {
    text(I18N.text(content))
        .style(type_style(size * scale, line * scale))
        .color(color)
        .modifier(Modifier::new().offset(Dp(x * scale), Dp(y * scale)));
}

const ALBUMS: [tessera_ui::Asset; 4] = [
    assets::SHEETS_PLAYER_ALBUM_1_PNG,
    assets::SHEETS_PLAYER_ALBUM_2_PNG,
    assets::SHEETS_PLAYER_ALBUM_3_PNG,
    assets::SHEETS_PLAYER_COVER_PNG,
];

#[tessera]
fn music_context() {
    let playing = remember(|| true);
    let cover = remember(|| 3usize);
    let scale = 1.0_f64;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    sheet_canvas(
        676.0,
        320.0,
        RenderSlot::new(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(320.0 * scale))
                        .clip_shape(Shape::rounded_rectangle(Dp(16.0)))
                        .background(scheme.surface_container_highest),
                )
                .children(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(124.0 * scale), Dp(-24.0 * scale))
                                .size(Dp(428.0 * scale), Dp(265.0 * scale))
                                .background_with_shape(
                                    scheme.outline_variant,
                                    Shape::rounded_rectangle(Dp(24.0 * scale)),
                                ),
                        )
                        .children(move || {
                            boxed()
                                .modifier(
                                    Modifier::new()
                                        .offset(Dp(8.0 * scale), Dp::ZERO)
                                        .size(Dp(412.0 * scale), Dp(257.0 * scale))
                                        .clip_shape(Shape::rounded_rectangle(Dp(18.0 * scale)))
                                        .background(scheme.surface),
                                )
                                .children(move || {
                                    for (index, x, w) in
                                        [(0, 19.0, 151.0), (1, 186.0, 151.0), (2, 354.0, 58.0)]
                                    {
                                        image().painter(ALBUMS[index]).modifier(
                                            Modifier::new()
                                                .offset(Dp(x * scale), Dp(24.0 * scale))
                                                .size(Dp(w * scale), Dp(124.0 * scale))
                                                .clip_shape(Shape::rounded_rectangle(Dp(
                                                    8.0 * scale
                                                ))),
                                        );
                                    }
                                    boxed()
                                        .modifier(
                                            Modifier::new()
                                                .offset(Dp::ZERO, Dp(164.0 * scale))
                                                .size(Dp(412.0 * scale), Dp(93.0 * scale))
                                                .background(scheme.surface_container),
                                        )
                                        .children(move || {
                                            image().painter(ALBUMS[cover.get()]).modifier(
                                                Modifier::new()
                                                    .size(Dp(64.0 * scale), Dp(68.0 * scale))
                                                    .clip_shape(Shape::rounded_rectangle(Dp(
                                                        8.0 * scale
                                                    ))),
                                            );
                                            sheet_context_text(
                                                "ui-cassette-futurism".into(),
                                                80.0,
                                                15.0,
                                                14.0,
                                                20.0,
                                                scale,
                                                scheme.on_surface,
                                            );
                                            sheet_context_text(
                                                "ui-various-artists".into(),
                                                80.0,
                                                35.0,
                                                12.0,
                                                16.0,
                                                scale,
                                                scheme.on_surface_variant,
                                            );
                                            icon_button()
                                                .icon(if playing.get() {
                                                    filled::PAUSE_SVG
                                                } else {
                                                    filled::PLAY_ARROW_SVG
                                                })
                                                .content_color(scheme.on_surface)
                                                .accessibility_label(I18N.text(if playing.get() {
                                                    "ui-pause"
                                                } else {
                                                    "ui-play"
                                                }))
                                                .modifier(
                                                    Modifier::new().offset(
                                                        Dp(315.0 * scale),
                                                        Dp(10.0 * scale),
                                                    ),
                                                )
                                                .on_click(move || playing.set(!playing.get()));
                                            icon_button()
                                                .icon(filled::SKIP_NEXT_SVG)
                                                .content_color(scheme.on_surface)
                                                .accessibility_label(I18N.text("ui-next-track"))
                                                .modifier(
                                                    Modifier::new().offset(
                                                        Dp(353.0 * scale),
                                                        Dp(10.0 * scale),
                                                    ),
                                                )
                                                .on_click(move || cover.set((cover.get() + 1) % 4));
                                            spacer().modifier(
                                                Modifier::new()
                                                    .offset(Dp(170.0 * scale), Dp(83.0 * scale))
                                                    .size(Dp(72.0 * scale), Dp(2.0 * scale))
                                                    .background_with_shape(
                                                        scheme.on_surface,
                                                        Shape::CAPSULE,
                                                    ),
                                            );
                                        });
                                });
                        });
                });
        }),
    );
    gap(Dp(16.0));
    prose(
        "ui-the-music-player-in-this-standard-bottom-sheet-allows-people-to-control-their-music-while",
    );
}

#[tessera]
fn filter_context() {
    let open = remember(|| true);
    let labels = remember(|| [false, false, true, true, false]);
    let scale = 1.0_f64;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    sheet_canvas(
        676.0,
        572.0,
        RenderSlot::new(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(572.0 * scale))
                        .clip_shape(Shape::rounded_rectangle(Dp(16.0)))
                        .background(scheme.surface_container),
                )
                .children(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(-40.0 * scale), Dp(67.0 * scale))
                                .size(Dp(637.0 * scale), Dp(600.0 * scale))
                                .background_with_shape(
                                    scheme.outline_variant,
                                    Shape::rounded_rectangle(Dp(28.0 * scale)),
                                ),
                        )
                        .children(move || {
                            boxed()
                                .modifier(
                                    Modifier::new()
                                        .offset(Dp::ZERO, Dp(8.0 * scale))
                                        .size(Dp(629.0 * scale), Dp(592.0 * scale))
                                        .clip_shape(Shape::rounded_rectangle(Dp(20.0 * scale)))
                                        .background(scheme.surface),
                                )
                                .children(move || {
                                    boxed()
                                        .modifier(
                                            Modifier::new()
                                                .offset(Dp(16.0 * scale), Dp(8.0 * scale))
                                                .size(Dp(223.0 * scale), Dp(56.0 * scale))
                                                .background_with_shape(
                                                    scheme.surface_container_highest,
                                                    Shape::CAPSULE,
                                                ),
                                        )
                                        .children(move || {
                                            icon_button()
                                                .icon(filled::TUNE_SVG)
                                                .accessibility_label(I18N.text("ui-open-filters"))
                                                .modifier(
                                                    Modifier::new()
                                                        .offset(Dp(170.0 * scale), Dp(4.0 * scale)),
                                                )
                                                .on_click(move || open.set(true));
                                        });
                                    sheet_context_text(
                                        "ui-last-modified".into(),
                                        112.0,
                                        140.0,
                                        14.0,
                                        20.0,
                                        scale,
                                        scheme.on_surface_variant,
                                    );
                                    for (index, label) in [
                                        "ui-2-hours-ago",
                                        "ui-5-hours-ago",
                                        "ui-yesterday",
                                        "ui-november-8",
                                        "ui-november-5",
                                        "ui-september-26",
                                    ]
                                    .into_iter()
                                    .enumerate()
                                    {
                                        sheet_context_text(
                                            label.into(),
                                            112.0,
                                            195.0 + index as f64 * 56.0,
                                            14.0,
                                            20.0,
                                            scale,
                                            scheme.on_surface_variant,
                                        );
                                        spacer().modifier(
                                            Modifier::new()
                                                .offset(
                                                    Dp(24.0 * scale),
                                                    Dp((230.0 + index as f64 * 56.0) * scale),
                                                )
                                                .size(Dp(580.0 * scale), Dp(1.0))
                                                .background(scheme.outline_variant),
                                        );
                                    }
                                    if open.get() {
                                        boxed().modifier(
                                            Modifier::new()
                                                .fill_max_size()
                                                .background(scheme.scrim.with_alpha(0.32)),
                                        );
                                        boxed()
                                            .modifier(
                                                Modifier::new()
                                                    .offset(Dp(309.0 * scale), Dp::ZERO)
                                                    .size(Dp(320.0 * scale), Dp(592.0 * scale))
                                                    .background_with_shape(
                                                        scheme.surface_container_low,
                                                        Shape::rounded_rectangle(Dp(16.0 * scale)),
                                                    ),
                                            )
                                            .children(move || {
                                                sheet_context_text(
                                                    "ui-filters".into(),
                                                    24.0,
                                                    17.0,
                                                    22.0,
                                                    28.0,
                                                    scale,
                                                    scheme.on_surface,
                                                );
                                                icon_button()
                                                    .icon(filled::CLOSE_SVG)
                                                    .accessibility_label(
                                                        I18N.text("ui-close-filters"),
                                                    )
                                                    .modifier(
                                                        Modifier::new().offset(
                                                            Dp(260.0 * scale),
                                                            Dp(8.0 * scale),
                                                        ),
                                                    )
                                                    .on_click(move || open.set(false));
                                                sheet_context_text(
                                                    "ui-labels".into(),
                                                    26.0,
                                                    91.0,
                                                    12.0,
                                                    16.0,
                                                    scale,
                                                    scheme.on_surface_variant,
                                                );
                                                for (index, label) in [
                                                    "ui-events",
                                                    "ui-personal",
                                                    "ui-projects",
                                                    "ui-reminders",
                                                    "ui-family",
                                                ]
                                                .into_iter()
                                                .enumerate()
                                                {
                                                    checkbox()
                                                        .checked(labels.get()[index])
                                                        .accessibility_label(I18N.text(label))
                                                        .modifier(Modifier::new().offset(
                                                            Dp(18.0 * scale),
                                                            Dp((123.0 + index as f64 * 56.0)
                                                                * scale),
                                                        ))
                                                        .on_toggle(move |value| {
                                                            labels.with_mut(|labels| {
                                                                labels[index] = value
                                                            })
                                                        });
                                                    sheet_context_text(
                                                        label.into(),
                                                        70.0,
                                                        135.0 + index as f64 * 56.0,
                                                        16.0,
                                                        24.0,
                                                        scale,
                                                        scheme.on_surface,
                                                    );
                                                }
                                                sheet_context_text(
                                                    "ui-format".into(),
                                                    26.0,
                                                    418.0,
                                                    12.0,
                                                    16.0,
                                                    scale,
                                                    scheme.on_surface_variant,
                                                );
                                                text_field()
                                                    .initial_text(I18N.text("ui-all"))
                                                    .read_only(true)
                                                    .accessibility_label(I18N.text("ui-format"))
                                                    .modifier(
                                                        Modifier::new()
                                                            .offset(
                                                                Dp(26.0 * scale),
                                                                Dp(453.0 * scale),
                                                            )
                                                            .width(Dp(268.0 * scale)),
                                                    );
                                            });
                                    }
                                });
                        });
                });
        }),
    );
    gap(Dp(16.0));
    caption(
        "ui-side-sheets-can-contain-the-same-content-as-bottom-sheets-and-may-be-more-suitable-for-des",
    );
}

#[cfg(test)]
#[path = "sheets_figures_tests.rs"]
mod tests;

#[tessera]
fn sheet_canvas(canonical_width: f64, canonical_height: f64, child: RenderSlot) {
    let available_width = remember(move || Dp(canonical_width));
    let scale = (available_width.get().0 / canonical_width) as f32;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if next != available_width.get() {
                    available_width.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box(scale).child(move || {
                boxed()
                    .modifier(
                        Modifier::new().required_size(Dp(canonical_width), Dp(canonical_height)),
                    )
                    .children_shared(child);
            });
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "sheets_scaled_tests.rs"]
mod scaled_tests;

#[path = "sheets_color.rs"]
mod color;

#[path = "sheets_figures.rs"]
mod figures;
