//! Bottom sheet specifications, tokens and modal demonstrations.
//!
//! ## Usage
//!
//! Compare sheet variants, adjust modal sheet height and review dismissal
//! behaviour.

use std::sync::OnceLock;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    bottom_sheet::{BottomSheetController, BottomSheetExpansion, bottom_sheet_provider},
    boxed::boxed,
    button::button,
    checkbox::checkbox,
    column::column,
    icon::icon,
    icon_button::icon_button,
    image::{ImageData, TryIntoImageData as _, image},
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

use super::{
    INK,
    annotations::{self, PreciseGeometry as Geometry},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
pub(super) fn sheets_spec(side: bool) {
    // The side sheet article lives in its own module; this page always presents
    // the bottom sheet specification.
    let _ = side;
    let action = remember(|| String::from("No action selected"));
    let controller = remember(|| BottomSheetController::new(false));
    let main = RenderSlot::new(move || {
        sheet_article(action, controller);
    });
    let body = RenderSlot::new(move || {
        sheet_content(Callback::new(move || close_sheet(controller)), action);
    });
    bottom_sheet_provider()
        .controller(controller)
        .accessibility_label("Share this place")
        .partial_height(Dp(360.0))
        .on_close_request(move || close_sheet(controller))
        .main_content_shared(main)
        .bottom_sheet_content_shared(body);
}

fn close_sheet(controller: State<BottomSheetController>) {
    controller.with_mut(BottomSheetController::close);
}

#[tessera]
fn sheet_article(action: State<String>, controller: State<BottomSheetController>) {
    article(
        "Bottom sheets",
        "Bottom sheets show secondary content anchored to the bottom of the screen",
        vec![
            SpecSection {
                title: "Tokens and specs",
                content: RenderSlot::new(|| {
                    tokens();
                }),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(|| {
                    sheet_colors_section();
                }),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(|| {
                    sheet_measurements_section();
                }),
            },
        ],
    )
    .intro(|| {
        prose("Modal bottom sheets are above a scrim while standard bottom sheets don't have a scrim. Besides this, both variants of bottom sheets have the same specs.");
        gap(Dp(32.0));
        figures::anatomy_figure(false);
        figures::badge_legend(vec![
            "Container".into(),
            "Drag handle (optional)".into(),
            "Scrim".into(),
        ]);
    })
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(|| { overview_intro(); })),
            sections: vec![
                SpecSection {
                    title: "Availability & resources",
                    content: RenderSlot::new(|| { availability(); }),
                },
                SpecSection {
                    title: "Differences from M2",
                    content: RenderSlot::new(|| { differences_from_m2(); }),
                },
            ],
        },
        ArticlePage {
            title: "Guidelines",
            intro: Some(RenderSlot::new(|| {
                figures::variants_figure();
                figures::badge_legend(vec![
                    "Standard bottom sheets".into(),
                    "Modal bottom sheets".into(),
                ]);
            })),
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(|| { usage(); }),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(|| { guide_anatomy(); }),
                },
                SpecSection {
                    title: "Standard bottom sheets",
                    content: RenderSlot::new(|| { standard_sheets(); }),
                },
                SpecSection {
                    title: "Modal bottom sheets",
                    content: RenderSlot::new(move || {
                        modal_sheets(controller);
                    }),
                },
                SpecSection {
                    title: "Responsive layout",
                    content: RenderSlot::new(|| { responsive_layout(); }),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(move || {
                        behavior_section(action, controller);
                    }),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Use cases",
                    content: RenderSlot::new(|| { use_cases(); }),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(|| { interaction_style(); }),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(|| { keyboard_navigation(); }),
                },
                SpecSection {
                    title: "Labeling",
                    content: RenderSlot::new(|| { labeling(); }),
                },
            ],
        },
    ]);
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(value)
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    column().children(move || {
        for item in items.clone() {
            text()
                .content(format!("•  {item}"))
                .style(type_style(16.0, 24.0))
                .color(INK);
            gap(Dp(8.0));
        }
    });
}

#[tessera]
fn overview_intro() {
    bullets(vec![
        "Use bottom sheets in compact and medium breakpoints".into(),
        "Two variants: standard and modal".into(),
        "Content should be additional or secondary (not the app's main content)".into(),
        "Bottom sheets can be dismissed in order to interact with the main content".into(),
    ]);
    gap(Dp(32.0));
    figures::variants_figure();
    figures::badge_legend(vec![
        "Standard bottom sheet".into(),
        "Modal bottom sheet".into(),
    ]);
}

#[tessera]
fn availability() {
    spec_table(
        vec!["Type".into(), "Resource".into(), "Status".into()],
        vec![
            vec![
                "Design".into(),
                "Design Kit (Figma)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Flutter".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
        ],
    );
}

#[tessera]
fn differences_from_m2() {
    figures::split()
        .start(|| {
            bullets(vec![
                "Color: New color mappings and compatibility with dynamic color".into(),
                "Shape: Bottom sheets have a 28dp top corner radius".into(),
                "Layout: New max-width of 640dp and an optional drag handle with an accessible 48dp hit target".into(),
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
            prose("Bottom sheets display supplementary content and actions on a mobile screen.");
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::share_figure();
                }),
                "Bottom sheet containing contacts and applications",
            );
        });
    figures::split()
        .start(|| {
            prose(
                "Bottom sheets are a versatile component that can contain a wide variety of information and layouts, including menu items (in list or grid layouts), actions, and supplemental content.",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::menu_figure(false);
                }),
                "Bottom sheet with menu items in a list",
            );
        });
}

#[tessera]
fn guide_anatomy() {
    prose(
        "A container is the only required element of a bottom sheet. Bottom sheet layouts can vary widely to support the kinds of content they contain.",
    );
    gap(Dp(32.0));
    figures::anatomy_figure(true);
    figures::badge_legend(vec![
        "Container".into(),
        "Drag handle (optional)".into(),
        "Scrim (modal only)".into(),
    ]);
    gap(Dp(32.0));
    subheading("Container".into());
    prose(
        "Bottom sheets are flexible containers that adapt to their content and available space. The container is the only required element.",
    );
    gap(Dp(24.0));
    subheading("Drag handle (optional)".into());
    prose(
        "The drag handle shows that the sheet can move between heights. It keeps a 48 × 48dp target and toggles the preset heights when it is selected.",
    );
    gap(Dp(24.0));
    subheading("Scrim (modal only)".into());
    prose(
        "A modal sheet appears above a scrim that blocks the rest of the app. Selecting the scrim always dismisses the sheet.",
    );
    gap(Dp(24.0));
    subheading("List items (optional)".into());
    prose(
        "Lists are a continuous group of text or images. List items can include label text, icons and text buttons.",
    );
    gap(Dp(24.0));
    subheading("Dividers (optional)".into());
    prose("Dividers can be used to separate related content in bottom sheets.");
    gap(Dp(24.0));
    subheading("Media (optional)".into());
    prose("Bottom sheets can contain thumbnails, images, and video.");
}

#[tessera]
fn standard_sheets() {
    prose(
        "Standard bottom sheets co-exist with the screen's main UI region and allow for simultaneously viewing and interacting with both regions, especially when the main UI region is frequently scrolled or panned.",
    );
    gap(Dp(24.0));
    prose(
        "Use a standard bottom sheet to display content that complements the screen's primary content, such as an audio player in a music app.",
    );
    gap(Dp(24.0));
    music_context();
    gap(Dp(24.0));
    prose(
        "A bottom sheet can have preset positions from full-screen height to preview. At full-screen height, standard bottom sheets contain a collapse icon in an app bar to return to their initial position.",
    );
}

#[tessera]
fn modal_sheets(controller: State<BottomSheetController>) {
    figures::split()
        .start(|| {
            prose(
                "Like dialogs, modal bottom sheets appear in front of app content, disabling all other app functionality when they appear, and remaining on screen until confirmed, dismissed, or a required action has been taken.",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::files_figure();
                }),
                "A modal bottom sheet must be interacted with or dismissed. Its blocking behavior makes it suitable for a menu, such as in this files app, to help people focus on their available choices.",
            );
        });
    figures::split()
        .start(move || {
            prose(
                "Use a modal bottom sheet as an alternative to inline menus or simple dialogs on mobile, especially when offering a long list of action items, or when items require longer descriptions and icons.",
            );
            gap(Dp(24.0));
            launch(controller);
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::menu_figure(true);
                }),
                "Modal bottom sheets can be used instead of menus to present additional actions",
            );
        });
    subheading("Visibility".into());
    figures::split()
        .start(|| {
            prose(
                "To provide access to its top actions, the initial vertical position of modal bottom sheets is capped at 50% of the screen height.",
            );
            gap(Dp(16.0));
            prose(
                "Modal bottom sheets whose contents exceed 50% of the screen height can then be pulled across the full screen and scrolled internally to access their remaining items.",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::photo_actions_figure(314.0);
                }),
                "The initial vertical position of modal bottom sheets can't exceed 50% of the screen height",
            );
        });
    figures::split()
        .start(|| {
            prose(
                "Modal bottom sheets appear when triggered by a user action, such as tapping a button or an overflow icon. They can be dismissed by:",
            );
            gap(Dp(12.0));
            bullets(vec![
                "Tapping a menu item or action within the bottom sheet".into(),
                "Tapping the scrim".into(),
                "Swiping the sheet down".into(),
                "Using a close affordance within the bottom sheet's app bar, if available".into(),
            ]);
            prose("Display a close affordance in a full-screen modal bottom sheet.");
        })
        .end(|| {
            figures::figure_pair(
                RenderSlot::new(|| {
                    figures::dismiss_scrim_figure();
                }),
                "Tapping the scrim dismisses a modal bottom sheet",
                RenderSlot::new(|| {
                    figures::dismiss_swipe_figure();
                }),
                "A modal bottom sheet can be dismissed by swiping the sheet down",
            );
        });
}

#[tessera]
fn responsive_layout() {
    figures::split()
        .start(|| {
            subheading("Compact breakpoint".into());
            prose(
                "In compact breakpoints, like mobile devices, bottom sheets extend across the width of a screen and are elevated above the primary content.",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::files_figure();
                }),
                "Bottom sheets should extend to the width of the screen on mobile",
            );
        });
    figures::split()
        .start(|| {
            subheading("Medium and expanded breakpoints".into());
            prose(
                "For larger screens with medium and expanded breakpoints, bottom sheets have a default max-width to prevent undesired layouts and awkward spacing. However, this can be overridden if needed. For more complex tasks and flows, consider using a non-transient surface such as a floating sheet.",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::tablet_figure();
                }),
                "Bottom sheets on larger screens like tablet have a max width that can be overridden",
            );
        });
    figures::split()
        .start(|| {
            prose(
                "On larger expanded breakpoints, like desktop, a bottom sheet can be swapped for a side sheet that shows similar content.",
            );
        })
        .end(|| {
            filter_context();
        });
}

#[tessera]
fn behavior_section(action: State<String>, controller: State<BottomSheetController>) {
    subheading("Expansion".into());
    prose(
        "Bottom sheets can offer an expansion option where the sheet is fully raised and toggled between a collapsed and expanded state. This provides a more predictable footprint of the sheet, and can be set by the system or toggled by the user.",
    );
    gap(Dp(24.0));
    figures::figure_pair(
        RenderSlot::new(|| {
            figures::photo_actions_figure(48.0);
        }),
        "A bottom sheet for sharing can appear fully raised if needed",
        RenderSlot::new(|| {
            figures::photo_actions_figure(470.0);
        }),
        "Alternately, a bottom sheet for sharing can appear collapsed for a more focused set of actions",
    );
    expansion_controls(controller);
    gap(Dp(32.0));
    subheading("Custom positioning".into());
    prose(
        "The drag handle can be dragged or selected to change the bottom sheet height. Sheets should be able to cycle through preset heights and close completely without dragging, while selecting the scrim should always close the bottom sheet.",
    );
    gap(Dp(16.0));
    prose(
        "If the bottom sheet has multiple preset heights but can't use a drag handle, Material requires the inclusion of a single-pointer alternative to change height.",
    );
    gap(Dp(24.0));
    figures::figure_pair(
        RenderSlot::new(|| {
            figures::drag_figure(false);
        }),
        "Interacting with the drag handle can quickly move a bottom sheet through preset heights",
        RenderSlot::new(|| {
            figures::drag_figure(true);
        }),
        "A bottom sheet can automatically resize to another height after interacting with the drag handle",
    );
    subheading("Scrolling".into());
    prose(
        "Bottom sheets should be scrollable when their content exceeds the initial viewable height.",
    );
    gap(Dp(24.0));
    subheading("Back".into());
    prose(
        "A predictive back gesture previews the result of the gesture: release to commit, fling to commit, or cancel. Where a gesture is not available, Escape and the close action dismiss the sheet.",
    );
    gap(Dp(24.0));
    launch(controller);
    gap(Dp(16.0));
    caption(format!("Last sheet action: {}", action.get()));
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
                .accessibility_label("Open expanded bottom sheet")
                .on_click(move || {
                    controller.with_mut(|sheet: &mut BottomSheetController| {
                        sheet.open();
                        sheet.expand();
                    });
                })
                .child(|| {
                    text().content("Open expanded");
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            button()
                .outlined()
                .accessibility_label("Open partially expanded bottom sheet")
                .on_click(move || {
                    controller.with_mut(|sheet: &mut BottomSheetController| {
                        sheet.open();
                        sheet.partially_expand();
                    });
                })
                .child(|| {
                    text().content("Open partial");
                });
        });
    gap(Dp(16.0));
    caption(format!(
        "Sheet: {}",
        match (open, expansion) {
            (false, _) => "closed",
            (true, BottomSheetExpansion::Expanded) => "open, expanded",
            (true, BottomSheetExpansion::PartiallyExpanded) => "open, partially expanded",
        }
    ));
}

#[tessera]
fn use_cases() {
    prose("Users should be able to:");
    gap(Dp(16.0));
    bullets(vec![
        "Resize bottom sheets without having to rely on touch gestures".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    figures::split()
        .start(|| {
            subheading("Touch target area".into());
            prose(
                "The top 48dp portion of the bottom sheet is interactive when user-initiated resizing is available and the drag handle is present.",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::touch_area_figure();
                }),
                "To ensure touch target accessibility, the top portion of a bottom sheet can be reserved for resize interactions",
            );
        });
    figures::split()
        .start(|| {
            subheading("Initial focus".into());
            prose(
                "The optional drag handle can be focused in the tab order and interacted with using non-touch inputs, such as keyboard or switch controls.",
            );
            gap(Dp(16.0));
            prose(
                "When a modal bottom sheet opens, focus moves into the sheet and returns to the control that opened it after the sheet closes.",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::focus_figure();
                }),
                "Visible focus shown on the drag handle affordance",
            );
        });
    figures::split()
        .start(|| {
            subheading("Dragging".into());
            prose(
                "Include a single-pointer alternative for any action that can be completed by dragging.",
            );
            gap(Dp(16.0));
            prose(
                "Drag handles should cycle the bottom sheet through available heights when selected. If a drag handle can't be used, add a button to do this action.",
            );
        })
        .end(|| {
            figures::figure_pair(
                RenderSlot::new(|| {
                    figures::drag_figure(false);
                }),
                "Interacting with the drag handle can quickly move a bottom sheet through preset heights",
                RenderSlot::new(|| {
                    figures::drag_figure(true);
                }),
                "A bottom sheet can automatically resize to another height after interacting with the drag handle",
            );
        });
    touch_target();
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Focus lands on drag handle".into()],
            vec![
                "Space / Enter".into(),
                "Toggles between available heights".into(),
            ],
        ],
    );
    gap(Dp(16.0));
    prose(
        "In the modal sheet, Escape or selecting the scrim dismisses the sheet without changing the page behind it.",
    );
}

#[tessera]
fn labeling() {
    figures::split()
        .start(|| {
            prose(
                "Label only the drag handle. The accessibility role for the drag handle is \"button.\"",
            );
        })
        .end(|| {
            figures::captioned(
                RenderSlot::new(|| {
                    figures::label_figure();
                }),
                "Label the drag handle",
            );
        });
}

#[tessera]
fn sheet_colors_section() {
    prose(
        "Color values are implemented through design tokens. For design, this means working with color values that correspond with tokens. For implementation, a color value will be a token that references a value.",
    );
    gap(Dp(24.0));
    color::sheet_colors(false, true);
    gap(Dp(16.0));
    caption("Bottom sheet color roles used for both light and dark schemes:");
    figures::badge_legend(vec![
        "Scrim*".into(),
        "On surface variant".into(),
        "Surface container low".into(),
    ]);
    caption(
        "*On Android platforms, the scrim color and opacity is automatically handled by the system UI.",
    );
}

#[tessera]
fn sheet_measurements_section() {
    figures::captioned(
        RenderSlot::new(|| {
            figures::measurements_figure();
        }),
        "Bottom sheet padding and size measurements",
    );
    prose(
        "Bottom sheets span the full window width up to 640dp. When the window width exceeds 640dp, bottom sheets adjust to have a top margin of 56dp and side margins of 56dp.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Drag handle alignment (horizontal)".into(), "Center".into()],
            vec!["Drag handle padding top/bottom".into(), "22dp".into()],
            vec!["Top margin".into(), "72dp".into()],
            vec!["Top margin (window width > 640dp)".into(), "56dp".into()],
            vec![
                "Start/end margin (window width > 640dp)".into(),
                "56dp".into(),
            ],
            vec!["Width".into(), "Full width, up to max-width 640dp".into()],
            vec!["Height".into(), "Variable".into()],
        ],
    );
}

#[tessera]
fn launch(controller: State<BottomSheetController>) {
    button()
        .outlined()
        .accessibility_label("Open modal bottom sheet")
        .on_click(move || {
            controller.with_mut(BottomSheetController::open);
        })
        .child(|| {
            text().content("Open modal bottom sheet");
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
                    text()
                        .content("Share this place")
                        .style(theme.typography.title_large)
                        .modifier(Modifier::new().weight(1.0));
                    icon_button()
                        .standard()
                        .icon(filled::CLOSE_SVG)
                        .accessibility_label("Close bottom sheet")
                        .on_click_shared(close);
                });
            gap(Dp(16.0));
            for (index, label) in ["Share", "Get link", "Save"].into_iter().enumerate() {
                list_item()
                    .headline(label)
                    .accessibility_label(format!("Bottom sheet {label}"))
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

const TOKEN_GROUPS: [&str; 3] = ["Container", "Drag handle", "Focus indicator"];

fn token_values(
    group: usize,
    scheme: &tessera_components::theme::MaterialColorScheme,
) -> Vec<(&'static str, String, Option<Color>)> {
    match group {
        0 => vec![
            (
                "Docked container color",
                "md.sys.color.surface-container-low".into(),
                Some(scheme.surface_container_low),
            ),
            (
                "Surface tint layer color",
                "md.sys.color.surface-tint".into(),
                Some(scheme.surface_tint),
            ),
            (
                "Modal container elevation",
                "md.sys.elevation.level1".into(),
                None,
            ),
            (
                "Standard container elevation",
                "md.sys.elevation.level1".into(),
                None,
            ),
            (
                "Container shape",
                "md.sys.shape.corner.extra-large.top (28dp)".into(),
                None,
            ),
            (
                "Minimized container shape",
                "md.sys.shape.corner.none".into(),
                None,
            ),
        ],
        1 => vec![
            (
                "Drag handle color",
                "md.sys.color.on-surface-variant".into(),
                Some(scheme.on_surface_variant),
            ),
            ("Drag handle opacity", "0.4".into(), None),
            ("Drag handle width", "32dp".into(), None),
            ("Drag handle height", "4dp".into(), None),
        ],
        _ => vec![
            (
                "Focus indicator color",
                "md.sys.color.secondary".into(),
                Some(scheme.secondary),
            ),
            ("Focus indicator thickness", "3dp".into(), None),
            ("Focus indicator outline offset", "2dp".into(), None),
        ],
    }
}

#[tessera]
fn tokens() {
    prose("Browse the component elements, attributes, tokens, and their values.");
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
                            text()
                                .content("Sheets - Bottom")
                                .style(type_style(14.0, 20.0))
                                .color(scheme.on_secondary_container);
                        });
                    spacer().modifier(Modifier::new().weight(1.0));
                    text()
                        .content("Default, Light")
                        .style(type_style(14.0, 20.0))
                        .color(scheme.on_surface_variant);
                });
            token_folder("Enabled", 0, 0, open);
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
        .headline(name.clone())
        .accessibility_label(format!("{name} tokens"))
        .accessibility_description(if expanded { "Expanded" } else { "Collapsed" })
        .content_padding(tessera_components::modifier::Padding::new(
            Dp(16.0 + depth as f64 * 24.0),
            Dp(8.0),
            Dp(16.0),
            Dp(8.0),
        ))
        .leading(|| {
            icon().vector(filled::FOLDER_SVG).size(Dp(20.0));
        })
        .trailing(move || {
            icon()
                .vector(if expanded {
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
                    label: Some(format!("{name}: {value}")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(name.clone())
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
            text()
                .content(value.clone())
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
                    .accessibility_label("Open native bottom sheet handle")
                    .on_click(move || {
                        controller.with_mut(BottomSheetController::open);
                    })
                    .child(|| {
                        text().content("Open bottom sheet");
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
                            .accessibility_label("Close native bottom sheet controls")
                            .on_click_shared(close);
                    });
                gap(Dp(8.0));
                text()
                    .content("Select the drag handle to change height.")
                    .style(type_style(14.0, 20.0));
            });
    });
    subheading("Try it".into());
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
                    .accessibility_label("Native bottom sheet controls")
                    .on_close_request_shared(close)
                    .main_content_shared(main)
                    .bottom_sheet_content_shared(body);
            });
        gap(Dp(12.0));
        text()
            .content(format!(
                "48dp drag handle · {} · Tab to the handle, then Space or Enter changes height",
                if controller.with(BottomSheetController::expansion)
                    == BottomSheetExpansion::Expanded
                {
                    "Expanded"
                } else {
                    "Partially expanded"
                }
            ))
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
    caption(
        "Side sheets can contain the same content as bottom sheets and may be more suitable for desktop",
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
