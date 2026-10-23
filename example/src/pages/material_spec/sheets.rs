//! Bottom and side sheet specification figures.
//!
//! ## Usage
//!
//! Explore supplementary content, modal dismissal and sheet measurements.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    bottom_sheet::bottom_sheet_provider,
    boxed::boxed,
    button::{ButtonSize, button},
    column::column,
    divider::horizontal_divider,
    icon::icon,
    icon_button::icon_button,
    list_item::list_item,
    material_icons::filled,
    modifier::ModifierExt as _,
    row::row,
    shape_def::{RoundedCorner, Shape},
    side_sheet::{SideSheetPosition, modal_side_sheet_provider},
    spacer::spacer,
    surface::surface,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Callback, Dp, Modifier, RenderSlot, State, modifier::FocusModifierExt as _, remember, tessera,
    use_context,
};

use super::{
    INK,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, srgb, subheading, type_style,
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
        SpecSection{title:"Usage",content:RenderSlot::new(move || { prose("Use a standard side sheet to show supporting information or tools while the main task remains available. Use a modal sheet to focus on a temporary supporting task."); gap(Dp(24.0)); launch(true,open,action); })},
        SpecSection{title:"Anatomy",content:RenderSlot::new(|| { figure(true,true,false); legend(true,true); gap(Dp(32.0)); for (title,body) in [("Container","Anchor the sheet to a side edge or detach it with consistent margins."),("Back icon button (optional)","Use back for navigation within the sheet. Keep a separate close action available."),("Close icon button (optional)","A visible close affordance dismisses the sheet without relying on gestures."),("Action buttons (optional)","Place actions at the bottom, aligned to the start edge."),("Divider (optional)","A divider separates the supporting region from adjacent content."),("Content (optional)","Keep secondary content relevant to the main task.")] { subheading(title.into()); prose(body); gap(Dp(24.0)); } })},
        SpecSection{title:"Adaptive design",content:RenderSlot::new(|| { prose("Standard sheets work alongside primary content in medium and expanded windows. Use a modal treatment when space does not support simultaneous interaction."); gap(Dp(24.0)); figure(true,false,true); gap(Dp(24.0)); subheading("RTL language support".into()); prose("Mirror start and end placement and directional navigation icons for right-to-left layouts."); })},
        SpecSection{title:"Behavior",content:RenderSlot::new(move || { prose("Keep supplementary content open while it is useful. Provide clear close and back actions, and preserve page state when the sheet closes."); gap(Dp(24.0)); subheading("Predictive back".into()); prose("On platforms that support predictive back, the gesture previews dismissal before the sheet leaves the screen."); gap(Dp(24.0)); launch(true,open,action); })},
    ] } else { vec![
        SpecSection{title:"Usage",content:RenderSlot::new(move || { prose("Use a bottom sheet for content that supports the current task. Keep the main screen visible so the relationship between the page and the sheet remains clear."); gap(Dp(24.0)); figure(false,false,false); })},
        SpecSection{title:"Anatomy",content:RenderSlot::new(|| { figure(false,true,false); legend(false,true); gap(Dp(32.0)); subheading("Container".into()); prose("The container anchors to the bottom edge, with rounded top corners. An optional handle indicates that its position can change."); gap(Dp(24.0)); subheading("List items (optional)".into()); prose("Use lists for a small set of contextual actions or destinations."); gap(Dp(24.0)); subheading("Dividers (optional)".into()); prose("Separate related groups without adding unnecessary visual weight."); gap(Dp(24.0)); subheading("Media (optional)".into()); prose("Images and media may accompany supporting content while the sheet remains secondary to the page."); })},
        SpecSection{title:"Standard bottom sheets",content:RenderSlot::new(|| { prose("Standard sheets coexist with the page. Keep both the main task and the supporting content available."); gap(Dp(24.0)); figure(false,false,false); })},
        SpecSection{title:"Modal bottom sheets",content:RenderSlot::new(move || { prose("A modal sheet blocks the main content with a scrim. Dismiss it with a close action, the scrim or Escape."); gap(Dp(24.0)); launch(false,open,action); gap(Dp(24.0)); subheading("Visibility".into()); prose("Show a modal sheet in response to an action and remove it when its task is complete."); })},
        SpecSection{title:"Responsive layout",content:RenderSlot::new(|| { subheading("Compact breakpoint".into()); prose("At compact widths the sheet spans the available window width and keeps the top of the main content visible."); gap(Dp(24.0)); subheading("Medium and expanded breakpoints".into()); prose("At wider widths, constrain the sheet and maintain margins instead of stretching secondary content across the entire window."); gap(Dp(24.0)); figure(false,true,true); })},
        SpecSection{title:"Behavior",content:RenderSlot::new(move || { subheading("Custom positioning".into()); prose("A sheet may rest at supported positions appropriate to the amount of content. Provide an alternative to dragging when changing the position."); gap(Dp(24.0)); subheading("Scrolling".into()); prose("Scroll the content when it exceeds the sheet. Keep the relationship between the sheet and the underlying page stable."); gap(Dp(24.0)); subheading("Back".into()); prose("Back dismisses a modal sheet and restores the preceding page state."); gap(Dp(24.0)); launch(false,open,action); })},
    ] } },
        ArticlePage { title: "Accessibility", intro: None, sections: { let mut result=vec![
        SpecSection{title:"Use cases",content:RenderSlot::new(move || { prose(if side { "People must be able to discover and dismiss supporting content using assistive technology." } else { "People must be able to reach sheet content and dismiss it without relying on a drag gesture." }); gap(Dp(24.0)); launch(side,open,action); })},
        SpecSection{title:"Interaction & style",content:RenderSlot::new(move || { if !side { subheading("Touch target area".into()); } prose("Provide a visible close action with a minimum 48 × 48dp target. Preserve text legibility and sufficient contrast at larger text sizes."); gap(Dp(24.0)); touch_target(); })},
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
        article = article.intro(move || {
        prose(if side { "A standard sheet leaves the main content available. A modal sheet appears above a scrim and requires dismissal before returning to the page." } else { "Modal bottom sheets appear above a scrim. Standard sheets remain part of the page without a scrim; both share container and drag-handle measurements." });
        gap(Dp(32.0));
        figure(side, true, false);
        legend(side, true);
    });
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
    figure(side, modal, false);
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Color role".into()],
        if side {
            vec![
                vec![
                    "Container".into(),
                    if modal {
                        "surfaceContainerLow"
                    } else {
                        "surface"
                    }
                    .into(),
                ],
                vec![
                    "Headline, close and back icons".into(),
                    "onSurfaceVariant".into(),
                ],
                vec!["Divider".into(), "outlineVariant".into()],
                vec!["Scrim (modal)".into(), "scrim".into()],
            ]
        } else {
            vec![
                vec!["Container".into(), "surfaceContainerLow".into()],
                vec!["Handle".into(), "onSurfaceVariant".into()],
                vec!["Scrim (modal)".into(), "scrim".into()],
            ]
        },
    );
}

#[tessera]
fn figure(side: bool, modal: bool, measurements: bool) {
    let geometry = remember(Geometry::new);
    let width = remember(|| Dp(800.0));
    let notice = remember(|| String::new());
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let scale = width.get().0 / if side { 1280.0 } else { 1024.0 };
    let figure_height = Dp(width.get().0
        * if side {
            if measurements {
                0.4150390625
            } else {
                0.81640625
            }
        } else if measurements {
            0.81640625
        } else {
            0.490234375
        });
    let inset = Dp(width.get().0 * 0.08125);
    let sheet_width = Dp(if side { 400.0 } else { 640.0 } * scale);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .modifier(
                    watch(geometry, "figure")
                        .fill_max_width()
                        .height(figure_height)
                        .background_with_shape(
                            scheme.surface_container,
                            Shape::rounded_rectangle(Dp(24.0 * scale)),
                        )
                        .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                            let next = Dp::from_pixels_f32(bounds.width.to_f32());
                            if width.get() != next {
                                width.set(next);
                            }
                        }),
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
                                            Dp(figure_height.0 - inset.0 * 2.0 - 16.0 * scale)
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
                                        top_left: RoundedCorner::manual(Dp(28.0 * scale), 3.0),
                                        top_right: RoundedCorner::manual(Dp(28.0 * scale), 3.0),
                                        bottom_left: RoundedCorner::manual(Dp::ZERO, 3.0),
                                        bottom_right: RoundedCorner::manual(Dp::ZERO, 3.0),
                                    }
                                })
                                .child(move || {
                                    specimen_content(side, modal, geometry, notice, scale);
                                });
                            boxed()
                                .modifier(Modifier::new().fill_max_size().border_with_shape(
                                    Dp(8.0 * scale),
                                    scheme.on_surface_variant.with_alpha(0.4),
                                    Shape::rounded_rectangle(Dp(24.0 * scale)),
                                ))
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
                                                    .size(Dp(108.0 * scale), Dp(4.0 * scale))
                                                    .background_with_shape(
                                                        scheme.on_surface,
                                                        Shape::rounded_rectangle(Dp(2.0 * scale)),
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
    let values = geometry.get();
    let (Some(root), Some(container), Some(screen)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "screen"),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
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
            if let Some(handle) = relative(&values, "handle") {
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
            let Some(rect) = relative(&values, name) else {
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
            let Some(rect) = relative(&values, name) else {
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
                "Activate the focused control".into(),
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
fn touch_target() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    super::panel(RenderSlot::new(move || {
        boxed()
            .alignment(Alignment::Center)
            .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)).border_with_shape(
                Dp(1.0),
                srgb(229, 79, 134),
                Shape::RECTANGLE,
            ))
            .children(move || {
                icon_button()
                    .standard()
                    .icon(filled::CLOSE_SVG)
                    .accessibility_label("48dp sheet close target")
                    .on_click(|| {});
            });
        gap(Dp(12.0));
        text()
            .content("48 × 48dp minimum target")
            .color(scheme.on_surface);
    }));
}
