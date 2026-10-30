//! Bottom sheet specifications, tokens and modal demonstrations.
//!
//! ## Usage
//!
//! Compare sheet variants, adjust modal sheet height and review dismissal
//! behaviour.

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
fn prose(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK);
}

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);

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
                    tokens(false);
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
        prose("Modal bottom sheets appear above a scrim while standard bottom sheets don't have a scrim. Besides this, both variants of bottom sheets have the same specs.");
        gap(Dp(32.0));
        figure(false, true, false);
        legend(false, true);
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
                prose("Two variants of bottom sheets.");
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
                    content: RenderSlot::new(move || {
                        use_cases(controller);
                    }),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(|| { interaction_style(); }),
                },
                SpecSection {
                    title: "Initial focus",
                    content: RenderSlot::new(|| { initial_focus(); }),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(|| { keyboard_navigation(); }),
                },
                SpecSection {
                    title: "Labeling elements",
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
                .style(type_style(16.0, 24.0));
            gap(Dp(8.0));
        }
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
                        .padding_symmetric(Dp(12.0), Dp(4.0))
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(8.0))),
                )
                .children(move || {
                    text()
                        .content(mark.clone())
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text()
                .content(note.clone())
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
fn overview_intro() {
    prose(
        "Use bottom sheets in compact widths smaller than 600dp, such as a phone in portrait orientation, and in medium widths from 600dp to 839dp, such as a tablet or foldable in portrait orientation.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Two variants: standard sheets display supplementary content without blocking access to the screen's primary content, such as an audio player at the bottom of a music app".into(),
        "Modal sheets appear in front of app content, disable all other app functionality and stay on screen until confirmed, dismissed or a required action is taken".into(),
        "Content should be additional or secondary, not the app's main content".into(),
        "Bottom sheets can be dismissed in order to interact with the main content".into(),
    ]);
    gap(Dp(32.0));
    figure(false, false, false);
    caption("Standard bottom sheet");
    gap(Dp(32.0));
    figure(false, true, false);
    caption("Modal bottom sheet");
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
    gap(Dp(24.0));
    prose(
        "Tessera examples on this page use the native bottom_sheet_provider with a scrim, an optional drag handle and two height presets.",
    );
}

#[tessera]
fn differences_from_m2() {
    spec_table(
        vec!["Attribute".into(), "M3".into()],
        vec![
            vec![
                "Color".into(),
                "New color mappings and compatibility with dynamic color".into(),
            ],
            vec![
                "Shape".into(),
                "Bottom sheets have a 28dp top corner radius".into(),
            ],
            vec![
                "Layout".into(),
                "New max-width of 640dp and an optional drag handle with an accessible 48dp hit target".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The container is the only required element. The drag handle is optional, and a sheet can always be dismissed without dragging.",
    );
}

#[tessera]
fn usage() {
    prose("Bottom sheets display supplementary content and actions on a mobile screen.");
    gap(Dp(24.0));
    prose(
        "Bottom sheets are a versatile component that can contain a wide variety of information and layouts, including menu items in list or grid layouts, actions and supplemental content.",
    );
    gap(Dp(24.0));
    do_dont(
        "Use a bottom sheet for supplementary content and actions that support the current screen",
        "Don't use a bottom sheet for the app's main content or for content that must stay permanently visible",
    );
    gap(Dp(16.0));
    do_dont(
        "Use a modal bottom sheet for a temporary, blocking task such as choosing a filter",
        "Don't use a modal bottom sheet when people still need to work with the content behind it",
    );
    gap(Dp(24.0));
    music_context();
    caption(
        "The music player in this standard bottom sheet allows people to control their music while browsing albums",
    );
}

#[tessera]
fn guide_anatomy() {
    prose(
        "A container is the only required element of a bottom sheet. Bottom sheet layouts can vary widely to support the kinds of content they contain.",
    );
    gap(Dp(32.0));
    figure(false, true, false);
    legend(false, true);
    gap(Dp(32.0));
    subheading("Container".into());
    prose(
        "Bottom sheet containers hold all bottom sheet elements. Their size is determined by the space those elements occupy, and the container is the only required element.",
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
    prose(
        "Thumbnails, images and video can accompany the content while the sheet remains secondary to the page.",
    );
}

#[tessera]
fn standard_sheets() {
    prose(
        "Standard bottom sheets co-exist with the screen's main UI region and allow for simultaneously viewing and interacting with both regions, especially when the main UI region is frequently scrolled or panned.",
    );
    gap(Dp(24.0));
    prose(
        "Use a standard bottom sheet to display content that complements the screen's primary content. It has no scrim, so the page behind it stays interactive.",
    );
    gap(Dp(24.0));
    music_context();
    gap(Dp(24.0));
    prose(
        "At full-screen height, standard bottom sheets contain a collapse icon in an app bar to return to their initial position.",
    );
}

#[tessera]
fn modal_sheets(controller: State<BottomSheetController>) {
    prose(
        "Like dialogs, modal bottom sheets appear in front of app content, disabling all other app functionality when they appear, and remaining on screen until confirmed, dismissed or a required action has been taken.",
    );
    gap(Dp(24.0));
    prose(
        "Use a modal bottom sheet as an alternative to inline menus or simple dialogs on mobile, especially when offering a long list of action items, or when items require longer descriptions and icons. Modal bottom sheets are used in mobile apps only.",
    );
    gap(Dp(24.0));
    launch(controller);
    gap(Dp(24.0));
    prose("Dismiss a modal bottom sheet by:");
    gap(Dp(12.0));
    bullets(vec![
        "Tapping a menu item or action within the bottom sheet".into(),
        "Tapping the scrim".into(),
        "Swiping the sheet down".into(),
        "Pressing Escape or using a close affordance in the sheet".into(),
    ]);
    gap(Dp(24.0));
    subheading("Visibility".into());
    prose(
        "To provide access to its top actions, the initial vertical position of a modal bottom sheet is capped at 50% of the screen height. Sheets whose content exceeds that height can be expanded and scrolled internally to reach the remaining items.",
    );
    gap(Dp(24.0));
    filter_context();
    caption(
        "A modal bottom sheet must be interacted with or dismissed before the page behind it can be used",
    );
}

#[tessera]
fn responsive_layout() {
    subheading("Compact breakpoints".into());
    prose(
        "In compact breakpoints, such as mobile devices, bottom sheets extend across the width of a screen and are elevated above the primary content.",
    );
    gap(Dp(24.0));
    subheading("Medium and expanded breakpoints".into());
    prose(
        "For larger screens, bottom sheets have a default max-width of 640dp to prevent undesired layouts and awkward spacing. The max-width can be overridden if needed, and more complex tasks can use a non-transient surface such as a floating sheet.",
    );
    gap(Dp(16.0));
    prose(
        "On larger expanded breakpoints, such as desktop, a bottom sheet can be swapped for a side sheet that shows similar content.",
    );
    gap(Dp(24.0));
    figure(false, true, true);
    caption("Bottom sheet on a larger device with 56dp top and 56dp side margins");
}

#[tessera]
fn behavior_section(action: State<String>, controller: State<BottomSheetController>) {
    subheading("Expansion".into());
    prose(
        "Bottom sheets can offer an expansion option where the sheet is fully raised and toggled between a collapsed and expanded state. This provides a more predictable footprint of the sheet, and can be set by the system or toggled by the user.",
    );
    gap(Dp(24.0));
    expansion_controls(controller);
    gap(Dp(24.0));
    subheading("Custom positioning".into());
    prose(
        "The drag handle can be dragged or selected to change the bottom sheet height. Sheets should be able to cycle through preset heights and close completely without dragging, while selecting the scrim should always close the bottom sheet.",
    );
    gap(Dp(16.0));
    prose(
        "If the bottom sheet has multiple preset heights but can't use a drag handle, Material requires the inclusion of a single-pointer alternative to change height.",
    );
    gap(Dp(24.0));
    subheading("Scrolling".into());
    prose(
        "Bottom sheets can be scrolled independently of the rest of the screen's content when their content exceeds the initial viewable height.",
    );
    gap(Dp(24.0));
    subheading("Back".into());
    prose(
        "A predictive back gesture previews dismissal before the sheet leaves the screen. Where a gesture is not available, Escape and the close action dismiss the sheet.",
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
fn use_cases(controller: State<BottomSheetController>) {
    prose("People should be able to do the following using assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Open the sheet and read its content".into(),
        "Move between the preset sheet heights without dragging".into(),
        "Dismiss the sheet without performing a gesture".into(),
        "Return to the page they were viewing before the sheet opened".into(),
    ]);
    gap(Dp(24.0));
    launch(controller);
}

#[tessera]
fn interaction_style() {
    prose(
        "Give the drag handle a 48 × 48dp target. A drag gesture must not be the only way to resize or dismiss a sheet: the drag handle is a focusable control that toggles the preset heights, and the scrim and close action both dismiss the sheet.",
    );
    gap(Dp(24.0));
    touch_target(false);
}

#[tessera]
fn initial_focus() {
    prose(
        "When a modal bottom sheet opens, move focus into the sheet. Keep traversal inside the modal until it closes, then restore focus to the control that opened it.",
    );
    gap(Dp(16.0));
    prose(
        "Selecting the scrim or pressing Escape dismisses the sheet without changing the page behind it.",
    );
}

#[tessera]
fn keyboard_navigation() {
    keyboard(false);
    gap(Dp(16.0));
    prose(
        "Traversal wraps inside the sheet, so Tab and Shift + Tab keep focus within the open modal until it is dismissed.",
    );
}

#[tessera]
fn labeling() {
    prose(
        "Give the sheet a name that describes its content or task. Name icon controls by the action they perform, and distinguish closing the sheet from navigating within it.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Label".into(), "Role".into()],
        vec![
            vec!["Sheet".into(), "Share this place".into(), "Dialog".into()],
            vec![
                "Drag handle".into(),
                "\"Drag handle\"".into(),
                "Button".into(),
            ],
            vec![
                "Close icon button".into(),
                "\"Close bottom sheet\"".into(),
                "Button".into(),
            ],
        ],
    );
}

#[tessera]
fn sheet_colors_section() {
    prose(
        "Color values are implemented through design tokens. For design, this means working with color values that correspond with tokens; for implementation, a color value is a token that references a value.",
    );
    gap(Dp(24.0));
    colors(false, true);
    gap(Dp(24.0));
    prose("Bottom sheet color roles used for both light and dark schemes:");
    gap(Dp(12.0));
    bullets(vec![
        "Scrim".into(),
        "On surface variant".into(),
        "Surface container low".into(),
    ]);
    gap(Dp(16.0));
    caption("On Android platforms the scrim color and opacity is handled by the system UI");
}

#[tessera]
fn sheet_measurements_section() {
    measurements(false);
    gap(Dp(32.0));
    figure(false, true, true);
    caption("Bottom sheet padding and size measurements");
    gap(Dp(24.0));
    prose(
        "Bottom sheets span the full window width up to 640dp, keep a 56dp top and side margin on larger windows, and the drag handle keeps a 48dp hit target.",
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
    // The side sheet specimen lives in its own module; this figure shows the
    // bottom sheet drag handle.
    let _ = side;
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
                "48dp drag handle · {} · Space or Enter changes height",
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
