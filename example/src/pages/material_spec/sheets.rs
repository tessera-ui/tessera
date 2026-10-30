//! Bottom and side sheet specification figures.
//!
//! ## Usage
//!
//! Explore supplementary content, modal dismissal and sheet measurements.

use std::{collections::BTreeMap, sync::OnceLock};

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    bottom_sheet::{BottomSheetController, BottomSheetExpansion, bottom_sheet_provider},
    boxed::boxed,
    button::{ButtonSize, button},
    checkbox::checkbox,
    column::column,
    divider::horizontal_divider,
    icon::icon,
    icon_button::icon_button,
    image::{ImageData, TryIntoImageData as _, image},
    list_item::list_item,
    material_icons::filled,
    modifier::ModifierExt as _,
    row::row,
    shape_def::{RoundedCorner, Shape},
    side_sheet::{SideSheetPosition, modal_side_sheet_provider},
    spacer::spacer,
    surface::surface,
    text::text,
    text_field::text_field,
    theme::MaterialTheme,
};
use tessera_ui::{
    Callback, Color, Dp, LayoutCoordinates, Modifier, RenderSlot, State,
    modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use super::{
    INK,
    annotations::{self, PreciseGeometry as Geometry, Rect, dotted, watch_precise as watch},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn sheets_spec(side: bool) {
    let open = remember(|| false);
    let action = remember(|| String::from("No action selected"));
    let main = RenderSlot::new(move || {
        sheet_article(side, open, action);
    });
    let body = RenderSlot::new(move || {
        sheet_content(side, Callback::new(move || open.set(false)), action);
    });
    if side {
        modal_side_sheet_provider()
            .position(SideSheetPosition::End)
            .is_open(open.get())
            .accessibility_label("Details")
            .on_close_request(move || open.set(false))
            .main_content_shared(main)
            .side_sheet_content_shared(body);
    } else {
        bottom_sheet_provider()
            .is_open(open.get())
            .accessibility_label("Share this place")
            .on_close_request(move || open.set(false))
            .main_content_shared(main)
            .bottom_sheet_content_shared(body);
    }
}

#[tessera]
fn sheet_article(side: bool, open: State<bool>, action: State<String>) {
    let mut sections = vec![SpecSection {
        title: if side {
            "Tokens & specs"
        } else {
            "Tokens and specs"
        },
        content: RenderSlot::new(move || {
            tokens(side);
        }),
    }];
    if side {
        sections.extend([
            SpecSection {
                title: "Standard side sheet",
                content: RenderSlot::new(|| {
                    figure(true, false, false);
                    legend(true, false);
                    gap(Dp(64.0));
                    subheading("Standard side sheet color".into());
                    gap(Dp(24.0));
                    colors(true, false);
                    gap(Dp(64.0));
                    subheading("Standard side sheet measurements".into());
                    gap(Dp(24.0));
                    figure(true, false, true);
                    measurements(true);
                }),
            },
            SpecSection {
                title: "Modal side sheet",
                content: RenderSlot::new(|| {
                    figure(true, true, false);
                    legend(true, true);
                    gap(Dp(64.0));
                    subheading("Modal side sheet color".into());
                    gap(Dp(24.0));
                    colors(true, true);
                    gap(Dp(64.0));
                    subheading("Modal side sheet measurements".into());
                    gap(Dp(24.0));
                    figure(true, true, true);
                    measurements(true);
                }),
            },
        ]);
    } else {
        sections.extend([
            SpecSection {
                title: "Color",
                content: RenderSlot::new(|| {
                    colors(false, true);
                }),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(|| {
                    figure(false, true, true);
                    measurements(false);
                }),
            },
        ]);
    }
    let mut article = article(
        if side { "Side sheets" } else { "Bottom sheets" },
        if side { "Side sheets show secondary content anchored to the side of the screen" } else { "Bottom sheets show secondary content anchored to the bottom of the screen" },
        sections,
    )
    .pages(vec![
        ArticlePage { title: "Overview", intro: Some(RenderSlot::new(move || {
            prose(if side { "Present optional tools or details beside the main task. Use standard sheets when both regions remain interactive and modal sheets when the secondary task needs focus." } else { "Present additional content and contextual actions near the bottom of compact or medium windows. Choose a standard sheet for persistent content or a modal sheet for a temporary task." });
            gap(Dp(32.0)); launch(side, open, action); gap(Dp(32.0)); figure(side, false, false);
        })), sections: vec![] },
        ArticlePage { title: "Guidelines", intro: None, sections: if side { vec![
        SpecSection{title:"Usage",content:RenderSlot::new(move || { prose("Use a standard side sheet to show supporting information or tools while the main task remains available. Use a modal sheet to focus on a temporary supporting task."); gap(Dp(24.0)); filter_context(); gap(Dp(24.0)); launch(true,open,action); })},
        SpecSection{title:"Anatomy",content:RenderSlot::new(|| { figure(true,true,false); legend(true,true); gap(Dp(32.0)); for (title,body) in [("Container","Anchor the sheet to a side edge or detach it with consistent margins."),("Back icon button (optional)","Use back for navigation within the sheet. Keep a separate close action available."),("Close icon button (optional)","A visible close affordance dismisses the sheet without relying on gestures."),("Action buttons (optional)","Place actions at the bottom, aligned to the start edge."),("Divider (optional)","A divider separates the supporting region from adjacent content."),("Content (optional)","Keep secondary content relevant to the main task.")] { subheading(title.into()); prose(body); gap(Dp(24.0)); } })},
        SpecSection{title:"Adaptive design",content:RenderSlot::new(|| { prose("Standard sheets work alongside primary content in medium and expanded windows. Use a modal treatment when space does not support simultaneous interaction."); gap(Dp(24.0)); figure(true,false,true); gap(Dp(24.0)); subheading("RTL language support".into()); prose("Mirror start and end placement and directional navigation icons for right-to-left layouts."); })},
        SpecSection{title:"Behavior",content:RenderSlot::new(move || { prose("Keep supplementary content open while it is useful. Provide clear close and back actions, and preserve page state when the sheet closes."); gap(Dp(24.0)); subheading("Predictive back".into()); prose("On platforms that support predictive back, the gesture previews dismissal before the sheet leaves the screen."); gap(Dp(24.0)); launch(true,open,action); })},
    ] } else { vec![
        SpecSection{title:"Usage",content:RenderSlot::new(move || { prose("Use a bottom sheet for content that supports the current task. Keep the main screen visible so the relationship between the page and the sheet remains clear."); gap(Dp(24.0)); figure(false,false,false); })},
        SpecSection{title:"Anatomy",content:RenderSlot::new(|| { figure(false,true,false); legend(false,true); gap(Dp(32.0)); subheading("Container".into()); prose("The container anchors to the bottom edge, with rounded top corners. An optional handle indicates that its position can change."); gap(Dp(24.0)); subheading("List items (optional)".into()); prose("Use lists for a small set of contextual actions or destinations."); gap(Dp(24.0)); subheading("Dividers (optional)".into()); prose("Separate related groups without adding unnecessary visual weight."); gap(Dp(24.0)); subheading("Media (optional)".into()); prose("Images and media may accompany supporting content while the sheet remains secondary to the page."); })},
        SpecSection{title:"Standard bottom sheets",content:RenderSlot::new(|| { prose("Standard sheets coexist with the page. Keep both the main task and the supporting content available."); gap(Dp(24.0)); music_context(); })},
        SpecSection{title:"Modal bottom sheets",content:RenderSlot::new(move || { prose("A modal sheet blocks the main content with a scrim. Dismiss it with a close action, the scrim or Escape."); gap(Dp(24.0)); launch(false,open,action); gap(Dp(24.0)); subheading("Visibility".into()); prose("Show a modal sheet in response to an action and remove it when its task is complete."); })},
        SpecSection{title:"Responsive layout",content:RenderSlot::new(|| { subheading("Compact breakpoint".into()); prose("At compact widths the sheet spans the available window width and keeps the top of the main content visible."); gap(Dp(24.0)); subheading("Medium and expanded breakpoints".into()); prose("At wider widths, constrain the sheet and maintain margins instead of stretching secondary content across the entire window."); gap(Dp(24.0)); figure(false,true,true); })},
        SpecSection{title:"Behavior",content:RenderSlot::new(move || { subheading("Custom positioning".into()); prose("A sheet may rest at supported positions appropriate to the amount of content. Provide an alternative to dragging when changing the position."); gap(Dp(24.0)); subheading("Scrolling".into()); prose("Scroll the content when it exceeds the sheet. Keep the relationship between the sheet and the underlying page stable."); gap(Dp(24.0)); subheading("Back".into()); prose("Back dismisses a modal sheet and restores the preceding page state."); gap(Dp(24.0)); launch(false,open,action); })},
    ] } },
        ArticlePage { title: "Accessibility", intro: None, sections: { let mut result=vec![
        SpecSection{title:"Use cases",content:RenderSlot::new(move || { prose(if side { "People must be able to discover and dismiss supporting content using assistive technology." } else { "People must be able to reach sheet content and dismiss it without relying on a drag gesture." }); gap(Dp(24.0)); launch(side,open,action); })},
        SpecSection{title:"Interaction & style",content:RenderSlot::new(move || { if !side { subheading("Touch target area".into()); } prose(if side { "Provide a visible close action with a minimum 48 × 48dp target. Preserve text legibility and sufficient contrast at larger text sizes." } else { "The top 48dp drag handle is a focusable button. Select it to switch between the partial and expanded heights; use the close action or scrim to dismiss the sheet." }); gap(Dp(24.0)); touch_target(side); })},
        SpecSection{title:"Initial focus",content:RenderSlot::new(move || { prose("When a modal sheet opens, move focus into its content. Keep traversal inside the modal until it closes, then restore focus to the opening control."); gap(Dp(24.0)); launch(side,open,action); })},
    ];
    if !side { result.push(SpecSection{title:"Dragging",content:RenderSlot::new(|| { prose("A drag gesture must not be the only way to dismiss or resize a sheet. Expose named actions to assistive technology and provide an explicit close control."); })}); }
    result.extend([
        SpecSection{title:"Keyboard navigation",content:RenderSlot::new(move || { keyboard(side); })},
        SpecSection{title:"Labeling",content:RenderSlot::new(|| { prose("Give the sheet a name that describes its content or task. Name icon controls by the action they perform, and distinguish closing the sheet from navigating within it."); })},
    ]);
    result } },
    ]);
    if !side {
        article = article.intro_shared(RenderSlot::new(move || {
        prose(if side { "A standard sheet leaves the main content available. A modal sheet appears above a scrim and requires dismissal before returning to the page." } else { "Modal bottom sheets appear above a scrim. Standard sheets remain part of the page without a scrim; both share container and drag-handle measurements." });
        gap(Dp(32.0));
        figure(side, true, false);
        legend(side, true);
    }));
    }
    drop(article);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn launch(side: bool, open: State<bool>, action: State<String>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            button()
                .outlined()
                .accessibility_label(if side {
                    "Open modal side sheet"
                } else {
                    "Open modal bottom sheet"
                })
                .on_click(move || open.set(true))
                .child(move || {
                    text().content(if side {
                        "Open modal side sheet"
                    } else {
                        "Open modal bottom sheet"
                    });
                });
            gap(Dp(16.0));
            prose(action.get());
        });
}

#[tessera]
fn sheet_content(side: bool, close: Callback, action: State<String>) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    column()
        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    text()
                        .content(if side { "Details" } else { "Share this place" })
                        .style(theme.typography.title_large)
                        .modifier(Modifier::new().weight(1.0));
                    icon_button()
                        .standard()
                        .icon(filled::CLOSE_SVG)
                        .accessibility_label(if side {
                            "Close side sheet"
                        } else {
                            "Close bottom sheet"
                        })
                        .on_click_shared(close);
                });
            gap(Dp(16.0));
            for (index, label) in ["Share", "Get link", "Save"].into_iter().enumerate() {
                list_item()
                    .headline(label)
                    .accessibility_label(format!(
                        "{} {label}",
                        if side { "Side sheet" } else { "Bottom sheet" }
                    ))
                    .leading(move || {
                        icon()
                            .vector(match index {
                                0 => filled::SHARE_SVG,
                                1 => filled::LINK_SVG,
                                _ => filled::BOOKMARK_SVG,
                            })
                            .size(Dp(24.0));
                    })
                    .on_click(move || {
                        action.set(format!("{label} selected"));
                        close.call();
                    });
            }
            gap(Dp(24.0));
            button()
                .text()
                .accessibility_label("Dismiss sheet")
                .on_click_shared(close)
                .child(|| {
                    text().content("Cancel");
                });
        });
}

#[tessera]
fn tokens(side: bool) {
    spec_table(
        vec!["Element".into(), "Token / value".into()],
        if side {
            vec![
                vec!["Standard container".into(), "surface".into()],
                vec!["Modal container".into(), "surfaceContainerLow".into()],
                vec!["Headline / icons".into(), "onSurfaceVariant".into()],
                vec!["Optional divider".into(), "outlineVariant".into()],
                vec!["Modal scrim".into(), "scrim, 32%".into()],
                vec!["Container maximum width".into(), "400dp".into()],
            ]
        } else {
            vec![
                vec!["Container".into(), "surfaceContainerLow".into()],
                vec!["Container top corners".into(), "28dp".into()],
                vec!["Drag handle".into(), "onSurfaceVariant, 40%".into()],
                vec!["Drag handle size".into(), "32 × 4dp".into()],
                vec!["Scrim".into(), "scrim, 32%".into()],
                vec!["Maximum width".into(), "640dp".into()],
            ]
        },
    );
}

#[tessera]
fn measurements(side: bool) {
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        if side {
            vec![
                vec!["Start/end padding".into(), "24dp".into()],
                vec!["Start padding with back icon".into(), "16dp".into()],
                vec!["Padding between top elements".into(), "12dp".into()],
                vec!["Bottom actions height".into(), "72dp".into()],
                vec![
                    "Bottom actions top / bottom padding".into(),
                    "16dp / 24dp".into(),
                ],
                vec!["Bottom actions alignment".into(), "Start".into()],
                vec!["Maximum width".into(), "400dp".into()],
                vec!["Detached margins".into(), "16dp".into()],
            ]
        } else {
            vec![
                vec!["Drag handle alignment".into(), "Center".into()],
                vec!["Drag handle top/bottom padding".into(), "22dp".into()],
                vec!["Top margin".into(), "72dp".into()],
                vec!["Top margin above 640dp window width".into(), "56dp".into()],
                vec![
                    "Start/end margin above 640dp window width".into(),
                    "56dp".into(),
                ],
                vec!["Width".into(), "Full window width, maximum 640dp".into()],
                vec!["Height".into(), "Variable".into()],
            ]
        },
    );
}

#[tessera]
fn colors(side: bool, modal: bool) {
    color::sheet_colors(side, modal);
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Color role".into()],
        if side {
            if modal {
                vec![
                    vec!["1  Back icon button".into(), "On surface variant".into()],
                    vec!["2  Headline".into(), "On surface variant".into()],
                    vec!["3  Container".into(), "Surface container low".into()],
                    vec!["4  Close icon button".into(), "On surface variant".into()],
                ]
            } else {
                vec![
                    vec!["1  Divider".into(), "Outline variant".into()],
                    vec!["2  Headline".into(), "On surface variant".into()],
                    vec!["3  Container".into(), "Surface".into()],
                    vec!["4  Close icon button".into(), "On surface variant".into()],
                ]
            }
        } else {
            vec![
                vec!["1  Scrim".into(), "Scrim".into()],
                vec!["2  Drag handle".into(), "On surface variant".into()],
                vec!["3  Container".into(), "Surface container low".into()],
            ]
        },
    );
}

#[tessera]
fn figure(side: bool, modal: bool, measurements: bool) {
    let geometry = remember(Geometry::new);
    #[cfg(all(test, not(target_family = "wasm")))]
    scaled_tests::GEOMETRY.set(Some(geometry));
    let notice = remember(String::new);
    #[cfg(all(test, not(target_family = "wasm")))]
    scaled_tests::NOTICE.set(Some(notice));
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let canonical_width = if side { 1280.0 } else { 1024.0 };
    let scale = 1.0_f64;
    let figure_height = Dp(if side {
        if measurements { 500.0 } else { 1045.0 }
    } else if measurements {
        836.0
    } else {
        502.0
    });
    let inset = Dp(canonical_width * 0.08125);
    let sheet_width = Dp(if side { 400.0 } else { 640.0 } * scale);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            sheet_canvas(
                canonical_width,
                figure_height.0,
                RenderSlot::new(move || {
                    boxed()
                        .modifier(
                            watch(geometry, "figure")
                                .fill_max_width()
                                .height(figure_height)
                                .background_with_shape(
                                    scheme.surface_container,
                                    Shape::rounded_rectangle(Dp(24.0 * scale)),
                                ),
                        )
                        .children(move || {
                            boxed()
                                .alignment(if side {
                                    Alignment::CenterEnd
                                } else {
                                    Alignment::BottomCenter
                                })
                                .modifier(Modifier::new().fill_max_size().padding_all(inset))
                                .children(move || {
                                    surface()
                                        .modifier(watch(geometry, "screen").fill_max_size())
                                        .style(scheme.surface.into())
                                        .shape(Shape::rounded_rectangle(Dp(24.0 * scale)))
                                        .child(move || {
                                            boxed()
                                                .modifier(Modifier::new().fill_max_size())
                                                .children(|| {});
                                        });
                                    if modal {
                                        boxed()
                                            .modifier(
                                                watch(geometry, "scrim")
                                                    .fill_max_size()
                                                    .background_with_shape(
                                                        scheme.scrim.with_alpha(0.32),
                                                        Shape::rounded_rectangle(Dp(24.0 * scale)),
                                                    ),
                                            )
                                            .children(|| {});
                                    }
                                    surface()
                                        .modifier(
                                            watch(geometry, "container")
                                                .offset(
                                                    if side { Dp(-8.0 * scale) } else { Dp::ZERO },
                                                    if side { Dp::ZERO } else { Dp(-8.0 * scale) },
                                                )
                                                .width(sheet_width)
                                                .height(if side {
                                                    Dp(figure_height.0
                                                        - inset.0 * 2.0
                                                        - 16.0 * scale)
                                                } else {
                                                    if measurements {
                                                        Dp(400.0 * scale)
                                                    } else {
                                                        Dp(256.0 * scale)
                                                    }
                                                }),
                                        )
                                        .style(
                                            if modal || !side {
                                                scheme.surface_container_low
                                            } else {
                                                scheme.surface
                                            }
                                            .into(),
                                        )
                                        .shape(if side {
                                            if modal {
                                                Shape::rounded_rectangle(Dp(16.0 * scale))
                                            } else {
                                                Shape::RECTANGLE
                                            }
                                        } else {
                                            Shape::RoundedRectangle {
                                                top_left: RoundedCorner::manual(
                                                    Dp(28.0 * scale),
                                                    3.0,
                                                ),
                                                top_right: RoundedCorner::manual(
                                                    Dp(28.0 * scale),
                                                    3.0,
                                                ),
                                                bottom_left: RoundedCorner::manual(Dp::ZERO, 3.0),
                                                bottom_right: RoundedCorner::manual(Dp::ZERO, 3.0),
                                            }
                                        })
                                        .child(move || {
                                            specimen_content(side, modal, geometry, notice, scale);
                                        });
                                    boxed()
                                        .modifier(
                                            Modifier::new().fill_max_size().border_with_shape(
                                                Dp(8.0 * scale),
                                                scheme.on_surface_variant.with_alpha(0.4),
                                                Shape::rounded_rectangle(Dp(24.0 * scale)),
                                            ),
                                        )
                                        .children(|| {});
                                    if side {
                                        boxed()
                                            .alignment(Alignment::BottomCenter)
                                            .modifier(
                                                Modifier::new()
                                                    .fill_max_size()
                                                    .padding_all(Dp(16.0 * scale)),
                                            )
                                            .children(move || {
                                                boxed()
                                                    .modifier(
                                                        Modifier::new()
                                                            .size(
                                                                Dp(108.0 * scale),
                                                                Dp(4.0 * scale),
                                                            )
                                                            .background_with_shape(
                                                                scheme.on_surface,
                                                                Shape::rounded_rectangle(Dp(
                                                                    2.0 * scale
                                                                )),
                                                            ),
                                                    )
                                                    .children(|| {});
                                            });
                                    }
                                });
                            boxed()
                                .modifier(Modifier::new().fill_max_size())
                                .children(move || {
                                    figure_annotations(side, modal, measurements, geometry);
                                });
                        });
                }),
            );
            if !notice.with(String::is_empty) {
                gap(Dp(12.0));
                prose(notice.get());
            }
        });
}

#[tessera]
fn specimen_content(
    side: bool,
    modal: bool,
    geometry: State<Geometry>,
    notice: State<String>,
    scale: f64,
) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    if side {
        boxed()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                column()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        row()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                            .children(move || {
                                if modal {
                                    boxed().modifier(watch(geometry, "back")).children(move || {
                                        icon_button()
                                            .standard()
                                            .icon(filled::ARROW_BACK_SVG)
                                            .accessibility_label("Sheet figure back")
                                            .on_click(move || {
                                                notice.set("Back action selected".into())
                                            });
                                    });
                                    spacer().modifier(Modifier::new().width(Dp(12.0)));
                                }
                                boxed()
                                    .alignment(Alignment::CenterStart)
                                    .modifier(Modifier::new().weight(1.0))
                                    .children(move || {
                                        text()
                                            .content("Headline")
                                            .style(theme.typography.title_large)
                                            .modifier(watch(geometry, "headline"));
                                    });
                                boxed()
                                    .modifier(watch(geometry, "close"))
                                    .children(move || {
                                        icon_button()
                                            .standard()
                                            .icon(filled::CLOSE_SVG)
                                            .accessibility_label("Sheet figure close")
                                            .on_click(move || {
                                                notice.set("Sheet close action selected".into())
                                            });
                                    });
                            });
                        spacer().modifier(Modifier::new().weight(1.0));
                        if modal {
                            boxed()
                                .modifier(watch(geometry, "divider").fill_max_width())
                                .children(|| {
                                    horizontal_divider();
                                });
                            row()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .modifier(
                                    watch(geometry, "actions")
                                        .fill_max_width()
                                        .height(Dp(72.0))
                                        .padding_symmetric(Dp(24.0), Dp::ZERO),
                                )
                                .children(move || {
                                    button()
                                        .text()
                                        .size(ButtonSize::ExtraSmall)
                                        .on_click(move || notice.set("Action 1 selected".into()))
                                        .child(|| {
                                            text().content("Action 1");
                                        });
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                    button()
                                        .text()
                                        .size(ButtonSize::ExtraSmall)
                                        .on_click(move || notice.set("Action 2 selected".into()))
                                        .child(|| {
                                            text().content("Action 2");
                                        });
                                });
                        }
                    });
                if !modal {
                    boxed()
                        .modifier(
                            watch(geometry, "divider")
                                .width(Dp(1.0))
                                .fill_max_height()
                                .background(scheme.outline_variant),
                        )
                        .children(|| {});
                }
            });
    } else {
        boxed()
            .alignment(Alignment::TopCenter)
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(Modifier::new().fill_max_width().height(Dp(48.0 * scale)))
                    .children(move || {
                        boxed()
                            .modifier(
                                watch(geometry, "handle")
                                    .size(Dp(32.0 * scale), Dp(4.0 * scale))
                                    .background_with_shape(
                                        scheme.on_surface_variant.with_alpha(0.4),
                                        Shape::rounded_rectangle(Dp(2.0 * scale)),
                                    ),
                            )
                            .children(|| {});
                    });
            });
    }
}

#[tessera]
fn figure_annotations(side: bool, modal: bool, measurements: bool, geometry: State<Geometry>) {
    figure_annotation_content(
        side,
        modal,
        measurements,
        geometry.with(annotations::relative_precise_geometry),
    );
}

#[tessera]
fn figure_annotation_content(
    side: bool,
    modal: bool,
    measurements: bool,
    values: BTreeMap<&'static str, Rect>,
) {
    let (Some(root), Some(container), Some(screen)) = (
        values.get("figure"),
        values.get("container").copied(),
        values.get("screen").copied(),
    ) else {
        return;
    };
    let width = Dp(root.w);
    let height = Dp(root.h);
    let scale = width.0 / if side { 1280.0 } else { 1024.0 };
    let mut svg = String::new();
    if measurements {
        let pink = "#ec407a";
        for rect in if side {
            vec![
                Rect {
                    x: container.x,
                    y: container.y,
                    w: 24.0 * scale,
                    h: container.h,
                },
                Rect {
                    x: container.x + container.w - 24.0 * scale,
                    y: container.y,
                    w: 24.0 * scale,
                    h: container.h,
                },
            ]
        } else {
            vec![
                Rect {
                    x: screen.x,
                    y: container.y - 72.0 * scale,
                    w: screen.w,
                    h: 72.0 * scale,
                },
                Rect {
                    x: screen.x,
                    y: container.y,
                    w: container.x - screen.x,
                    h: container.h,
                },
                Rect {
                    x: container.x + container.w,
                    y: container.y,
                    w: screen.x + screen.w - container.x - container.w,
                    h: container.h,
                },
            ]
        } {
            svg.push_str(&format!(
                "<rect x='{}' y='{}' width='{}' height='{}' fill='{pink}' fill-opacity='.18'/>",
                rect.x,
                rect.y,
                rect.w.max(0.0),
                rect.h
            ));
        }
        if side {
            dotted(
                &mut svg,
                container.x,
                container.y - 12.0,
                container.w,
                false,
                pink,
            );
            annotations::dimension(
                "Max 400".into(),
                Dp(container.x + container.w / 2.0 - 28.0),
                Dp(container.y - 36.0),
            );
            annotations::dimension(
                "24".into(),
                Dp(container.x),
                Dp(container.y + container.h + 8.0),
            );
            annotations::dimension(
                "24".into(),
                Dp(container.x + container.w - 24.0 * scale),
                Dp(container.y + container.h + 8.0),
            );
        } else {
            dotted(
                &mut svg,
                screen.x + screen.w + 12.0,
                container.y - 72.0 * scale,
                72.0,
                true,
                pink,
            );
            annotations::dimension(
                "72".into(),
                Dp(screen.x + screen.w + 18.0),
                Dp(container.y - 46.0),
            );
            if let Some(handle) = values.get("handle").copied() {
                annotations::outline(
                    &mut svg,
                    Rect {
                        x: handle.x - 8.0 * scale,
                        y: handle.y - 22.0 * scale,
                        w: handle.w + 16.0 * scale,
                        h: 48.0 * scale,
                    },
                );
                annotations::dimension(
                    "22".into(),
                    Dp(handle.x + handle.w + 16.0 * scale),
                    Dp(handle.y - 22.0 * scale),
                );
                annotations::dimension(
                    "22".into(),
                    Dp(handle.x + handle.w + 16.0 * scale),
                    Dp(handle.y + handle.h),
                );
            }
        }
    } else if side {
        for (name, number) in if modal {
            vec![
                ("back", 1),
                ("headline", 2),
                ("container", 3),
                ("close", 4),
                ("divider", 5),
                ("actions", 6),
                ("scrim", 7),
            ]
        } else {
            vec![
                ("divider", 1),
                ("headline", 2),
                ("container", 3),
                ("close", 4),
            ]
        } {
            let Some(rect) = values.get(name).copied() else {
                continue;
            };
            if name == "back"
                || name == "headline"
                || name == "container"
                || (name == "divider" && !modal)
            {
                let x = if name == "container" {
                    rect.x + rect.w * 0.65
                } else if name == "divider" {
                    rect.x
                } else {
                    rect.x + rect.w * 0.5
                };
                let y = container.y - 48.0;
                dotted(&mut svg, x, rect.y, y + 32.0 - rect.y, true, "#49454f");
                annotations::marker(number, Dp(x - 12.0), Dp(y));
            } else if name == "scrim" {
                annotations::marker(number, Dp(screen.x + 24.0), Dp(screen.y + screen.h * 0.6));
            } else {
                let x = container.x + container.w + 20.0;
                let y = rect.y + rect.h * 0.5;
                dotted(
                    &mut svg,
                    rect.x + rect.w,
                    y,
                    x - rect.x - rect.w,
                    false,
                    "#49454f",
                );
                annotations::marker(number, Dp(x), Dp(y - 16.0));
            }
        }
    } else {
        for (name, number) in [("container", 1), ("handle", 2), ("scrim", 3)] {
            if name == "scrim" && !modal {
                continue;
            }
            let Some(rect) = values.get(name).copied() else {
                continue;
            };
            if name == "handle" {
                let x = rect.x + rect.w * 0.5;
                let y = container.y - 64.0;
                dotted(&mut svg, x, rect.y, y + 32.0 - rect.y, true, "#49454f");
                annotations::marker(number, Dp(x - 12.0), Dp(y));
            } else {
                let x = if name == "scrim" {
                    screen.x + 24.0
                } else {
                    container.x - 40.0
                };
                let y = if name == "scrim" {
                    screen.y + 48.0
                } else {
                    container.y + container.h * 0.5
                };
                if name != "scrim" {
                    dotted(
                        &mut svg,
                        x + 24.0,
                        y,
                        container.x - x - 24.0,
                        false,
                        "#49454f",
                    );
                }
                annotations::marker(number, Dp(x), Dp(y - 16.0));
            }
        }
    }

    annotations::lines(width, height, svg);
}

#[tessera]
fn legend(side: bool, modal: bool) {
    gap(Dp(16.0));
    for (number, label) in if side && modal {
        vec![
            (1, "Back icon button (optional)"),
            (2, "Headline"),
            (3, "Container"),
            (4, "Close icon button"),
            (5, "Divider (optional)"),
            (6, "Action buttons (optional)"),
            (7, "Scrim"),
        ]
    } else if side {
        vec![
            (1, "Divider (optional)"),
            (2, "Headline"),
            (3, "Container"),
            (4, "Close icon button"),
        ]
    } else {
        vec![
            (1, "Container"),
            (2, "Drag handle (optional)"),
            (3, "Scrim"),
        ]
    } {
        prose(format!("{number}  {label}"));
    }
}

#[tessera]
fn keyboard(side: bool) {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Move to the next control in the sheet".into()],
            vec!["Shift + Tab".into(), "Move to the previous control".into()],
            vec![
                "Space or Enter".into(),
                if side {
                    "Activate the focused control"
                } else {
                    "Toggle the drag handle between available sheet heights"
                }
                .into(),
            ],
            vec![
                "Escape".into(),
                if side {
                    "Dismiss the modal side sheet"
                } else {
                    "Dismiss the modal bottom sheet"
                }
                .into(),
            ],
        ],
    );
}

#[tessera]
fn touch_target(side: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let controller = remember(|| BottomSheetController::new(false));
    let side_open = remember(|| false);
    let close = Callback::new(move || {
        if side {
            side_open.set(false);
        } else {
            controller.with_mut(BottomSheetController::close);
        }
    });
    let main = RenderSlot::new(move || {
        boxed()
            .alignment(Alignment::Center)
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                button()
                    .accessibility_label(if side {
                        "Open native side sheet controls"
                    } else {
                        "Open native bottom sheet handle"
                    })
                    .on_click(move || {
                        if side {
                            side_open.set(true);
                        } else {
                            controller.with_mut(BottomSheetController::open);
                        }
                    })
                    .child(move || {
                        text().content(if side {
                            "Open side sheet"
                        } else {
                            "Open bottom sheet"
                        });
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
                        text()
                            .content("Sheet controls")
                            .style(type_style(20.0, 28.0))
                            .modifier(Modifier::new().weight(1.0));
                        icon_button()
                            .standard()
                            .icon(filled::CLOSE_SVG)
                            .accessibility_label(if side {
                                "Close native side sheet controls"
                            } else {
                                "Close native bottom sheet controls"
                            })
                            .on_click_shared(close);
                    });
                gap(Dp(8.0));
                text()
                    .content(if side {
                        "Use Close, Escape or the scrim."
                    } else {
                        "Select the drag handle to change height."
                    })
                    .style(type_style(14.0, 20.0));
            });
    });
    super::panel(RenderSlot::new(move || {
        boxed()
            .modifier(
                Modifier::new()
                    .width(Dp(400.0))
                    .height(Dp(360.0))
                    .clip_to_bounds(),
            )
            .children(move || {
                if side {
                    modal_side_sheet_provider()
                        .is_open(side_open.get())
                        .accessibility_label("Native side sheet controls")
                        .on_close_request_shared(close)
                        .main_content_shared(main)
                        .side_sheet_content_shared(body);
                } else {
                    bottom_sheet_provider()
                        .controller(controller)
                        .partial_height(Dp(192.0))
                        .accessibility_label("Native bottom sheet controls")
                        .on_close_request_shared(close)
                        .main_content_shared(main)
                        .bottom_sheet_content_shared(body);
                }
            });
        gap(Dp(12.0));
        text()
            .content(if side {
                "The native close control has a 48 × 48dp target.".to_owned()
            } else {
                format!(
                    "48dp drag handle · {} · Space or Enter changes height",
                    if controller.with(BottomSheetController::expansion)
                        == BottomSheetExpansion::Expanded
                    {
                        "Expanded"
                    } else {
                        "Partially expanded"
                    }
                )
            })
            .color(scheme.on_surface);
    }));
}

fn sheet_album(index: usize) -> ImageData {
    static ALBUMS: OnceLock<[ImageData; 4]> = OnceLock::new();
    ALBUMS.get_or_init(|| {
        [
            include_bytes!("../../../assets/material_spec/sheets-player-album-1.png").as_slice(),
            include_bytes!("../../../assets/material_spec/sheets-player-album-2.png").as_slice(),
            include_bytes!("../../../assets/material_spec/sheets-player-album-3.png").as_slice(),
            include_bytes!("../../../assets/material_spec/sheets-player-cover.png").as_slice(),
        ]
        .map(|bytes| bytes.try_into_image_data().expect("sheet album artwork"))
    })[index]
        .clone()
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
    text()
        .content(content)
        .style(type_style(size * scale, line * scale))
        .color(color)
        .modifier(Modifier::new().offset(Dp(x * scale), Dp(y * scale)));
}

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
                                        image().raster(sheet_album(index)).modifier(
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
                                            image().raster(sheet_album(cover.get())).modifier(
                                                Modifier::new()
                                                    .size(Dp(64.0 * scale), Dp(68.0 * scale))
                                                    .clip_shape(Shape::rounded_rectangle(Dp(
                                                        8.0 * scale
                                                    ))),
                                            );
                                            sheet_context_text(
                                                "Cassette Futurism".into(),
                                                80.0,
                                                15.0,
                                                14.0,
                                                20.0,
                                                scale,
                                                scheme.on_surface,
                                            );
                                            sheet_context_text(
                                                "Various artists".into(),
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
                                                .accessibility_label(if playing.get() {
                                                    "Pause"
                                                } else {
                                                    "Play"
                                                })
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
                                                .accessibility_label("Next track")
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
        "The music player in this standard bottom sheet allows people to control their music while browsing albums.",
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
                                                .accessibility_label("Open filters")
                                                .modifier(
                                                    Modifier::new()
                                                        .offset(Dp(170.0 * scale), Dp(4.0 * scale)),
                                                )
                                                .on_click(move || open.set(true));
                                        });
                                    sheet_context_text(
                                        "Last modified".into(),
                                        112.0,
                                        140.0,
                                        14.0,
                                        20.0,
                                        scale,
                                        scheme.on_surface_variant,
                                    );
                                    for (index, label) in [
                                        "2 hours ago",
                                        "5 hours ago",
                                        "Yesterday",
                                        "November 8",
                                        "November 5",
                                        "September 26",
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
                                                    "Filters".into(),
                                                    24.0,
                                                    17.0,
                                                    22.0,
                                                    28.0,
                                                    scale,
                                                    scheme.on_surface,
                                                );
                                                icon_button()
                                                    .icon(filled::CLOSE_SVG)
                                                    .accessibility_label("Close filters")
                                                    .modifier(
                                                        Modifier::new().offset(
                                                            Dp(260.0 * scale),
                                                            Dp(8.0 * scale),
                                                        ),
                                                    )
                                                    .on_click(move || open.set(false));
                                                sheet_context_text(
                                                    "Labels".into(),
                                                    26.0,
                                                    91.0,
                                                    12.0,
                                                    16.0,
                                                    scale,
                                                    scheme.on_surface_variant,
                                                );
                                                for (index, label) in [
                                                    "Events",
                                                    "Personal",
                                                    "Projects",
                                                    "Reminders",
                                                    "Family",
                                                ]
                                                .into_iter()
                                                .enumerate()
                                                {
                                                    checkbox()
                                                        .checked(labels.get()[index])
                                                        .accessibility_label(label)
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
                                                    "Format".into(),
                                                    26.0,
                                                    418.0,
                                                    12.0,
                                                    16.0,
                                                    scale,
                                                    scheme.on_surface_variant,
                                                );
                                                text_field()
                                                    .initial_text("All")
                                                    .read_only(true)
                                                    .accessibility_label("Format")
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
    prose("Modal side sheet with filter controls.");
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
