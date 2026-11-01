//! Primary and secondary tab specification articles.
//!
//! ## Usage
//!
//! Explore content destinations, scrollable rows and keyboard activation.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    app_bar::{AppBarScrollBehavior, AppBarScrollState, app_bar_scroll_provider},
    badge::{badge, badge_with_content, badged_box},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    scaffold::scaffold,
    scrollable::ScrollableController,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    tabs::{TabActivationMode, TabsVariant, tab, tabs},
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, State, accesskit::Role, modifier::FocusModifierExt as _,
    remember, tessera, use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap,
    navigation::article_pages::figure_marker,
    panel, spec_table, subheading, type_style,
};

#[cfg(test)]
#[path = "tabs_tests.rs"]
mod tabs_tests;

const DESTINATIONS: [&str; 5] = ["Flights", "Hotels", "Trains", "Buses", "Ferries"];
const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const SWIPE_PAGES: [(&str, &str); 3] = [
    ("Flights", "Upcoming flights and saved routes."),
    ("Trips", "Itineraries grouped by destination."),
    ("Explore", "Places to visit near your next stop."),
];

#[tessera]
pub(super) fn tabs_spec() {
    article(
        "Tabs",
        "Tabs organize content across different screens and views",
        vec![
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Primary tabs",
                content: RenderSlot::new(|| {
                    section(true);
                }),
            },
            SpecSection {
                title: "Secondary tabs",
                content: RenderSlot::new(|| {
                    section(false);
                }),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    )
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(overview_intro)),
            sections: vec![
                SpecSection {
                    title: "Availability & resources",
                    content: RenderSlot::new(availability),
                },
                SpecSection {
                    title: "Differences from M2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "Guidelines",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(guidelines_usage),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(anatomy),
                },
                SpecSection {
                    title: "Choosing the tab variant",
                    content: RenderSlot::new(choosing_variant),
                },
                SpecSection {
                    title: "Placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "Responsive layout",
                    content: RenderSlot::new(responsive),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(behavior),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Use cases",
                    content: RenderSlot::new(use_cases),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "Initial focus",
                    content: RenderSlot::new(initial_focus),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(keyboard_navigation),
                },
                SpecSection {
                    title: "Labeling elements",
                    content: RenderSlot::new(labelling),
                },
            ],
        },
    ]);
}

#[tessera]
fn paragraph(value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .modifier(Modifier::new().fill_max_width());
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(value)
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0))
        .modifier(Modifier::new().fill_max_width());
}

#[tessera]
fn bullets(items: Vec<String>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for item in items.clone() {
                text()
                    .content(format!("•  {item}"))
                    .style(type_style(16.0, 24.0));
                gap(Dp(8.0));
            }
        });
}

#[tessera]
fn legend(number: usize, #[prop(into)] label: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(24.0), Dp(24.0))
                        .background_with_shape(scheme.inverse_surface, Shape::Ellipse),
                )
                .children(move || {
                    text()
                        .content(number.to_string())
                        .style(type_style(12.0, 16.0))
                        .color(scheme.inverse_on_surface);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text().content(label.clone()).style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn legend_list(entries: Vec<String>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, entry) in entries.clone().into_iter().enumerate() {
                legend(index + 1, entry);
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
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(8.0)))
                        .padding(Padding::symmetric(Dp(12.0), Dp(4.0))),
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
fn do_dont(
    #[prop(into)] good: String,
    #[prop(into)] bad: String,
    good_figure: Option<RenderSlot>,
    bad_figure: Option<RenderSlot>,
) {
    if let (Some(good_figure), Some(bad_figure)) = (good_figure, bad_figure) {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                let (good, bad) = (good.clone(), bad.clone());
                let (good_figure, bad_figure) = (good_figure.clone(), bad_figure.clone());
                column()
                    .modifier(Modifier::new().weight(1.0))
                    .children(move || {
                        guidance_card(true, good.clone(), good_figure.clone());
                    });
                spacer().modifier(Modifier::new().width(Dp(24.0)));
                column()
                    .modifier(Modifier::new().weight(1.0))
                    .children(move || {
                        guidance_card(false, bad.clone(), bad_figure.clone());
                    });
            });
    } else {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                guidance_line("Do", good.clone(), DO_COLOR);
                gap(Dp(12.0));
                guidance_line("Don't", bad.clone(), DONT_COLOR);
            });
    }
}

#[tessera]
fn overview_intro() {
    paragraph("Use tabs to group content into helpful categories.".into());
    gap(Dp(16.0));
    bullets(vec![
        "Two variants: primary and secondary".into(),
        "Primary tabs: Display main content destinations at the top of the screen, often under a top app bar".into(),
        "Secondary tabs: Display related content within a content area; always placed below primary tabs".into(),
        "Tabs can horizontally scroll, so a UI can have as many tabs as needed".into(),
        "Place tabs next to each other as peers".into(),
    ]);
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("Primary tabs");
    gap(Dp(24.0));
    native_tabs_figure(false, false, false, 0, 2);
    gap(Dp(8.0));
    caption("Secondary tabs");
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
                "Jetpack Compose".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Available".into()],
        ],
    );
}

#[tessera]
fn differences() {
    bullets(vec![
        "Color: New color mappings and compatibility with dynamic color".into(),
        "Layout: Icons and labels are vertically centered in the container".into(),
    ]);
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("Tab icons and labels are positioned in the vertical center of the container");
}

#[tessera]
fn guidelines_usage() {
    paragraph(
        "Tabs organize groups of related content that are at the same level of hierarchy.".into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("Tab labels can include icons and text. Text labels should be short.");
    gap(Dp(32.0));
    subheading("Types".into());
    gap(Dp(16.0));
    paragraph("There are two types of tabs:".into());
    gap(Dp(8.0));
    legend_list(vec!["Primary tabs".into(), "Secondary tabs".into()]);
    gap(Dp(24.0));
    native_tabs_figure(true, false, false, 0, 3);
    gap(Dp(24.0));
    native_tabs_figure(false, false, false, 0, 2);
    gap(Dp(24.0));
    paragraph(
        "Primary tabs are placed at the top of the content pane under an app bar. They display the main content destinations. Secondary tabs are used within a content area to further separate related content and establish hierarchy."
            .into(),
    );
    gap(Dp(32.0));
    subheading("Related content".into());
    gap(Dp(16.0));
    paragraph(
        "Use tabs to group content into clearly defined sets. Tabs shouldn't be used for content that must be read in a particular order; create hierarchy with typography and open space instead."
            .into(),
    );
    gap(Dp(16.0));
    do_dont(
        "Use tabs to categorize related groups of content into clearly defined sets",
        "Don't use tabs to move through sequential content that needs to be read in a particular order",
    )
    .good_figure(|| {
        mock_label_tabs(vec!["Flights".into(), "Trips".into(), "Explore".into()]);
    })
    .bad_figure(|| {
        mock_label_tabs(vec!["Step 1".into(), "Step 2".into(), "Step 3".into()]);
    });
}

#[tessera]
fn anatomy() {
    native_tabs_figure(true, true, false, 3, 3);
    gap(Dp(24.0));
    legend_list(vec![
        "Container".into(),
        "Icon (optional)".into(),
        "Badge (optional)".into(),
        "Label".into(),
        "Divider".into(),
        "Active indicator".into(),
    ]);
    gap(Dp(32.0));
    subheading("Container".into());
    gap(Dp(16.0));
    paragraph(
        "The container holds multiple tabs, which can be fixed or scrollable. It spans the full width of the window and is divided into equal sections, one for each tab. A divider runs along its bottom edge, and the content below may scroll under it."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, false, false, 0, 3);
    gap(Dp(8.0));
    caption("The container is the area that contains the tabs directly under the title above");
    gap(Dp(32.0));
    subheading("Icon (optional)".into());
    gap(Dp(16.0));
    paragraph(
        "Icons should be simple and recognizable, and clearly communicate the kind of content a tab holds. Icons alone are less effective than labels, so use them with caution."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("Tabs can use a combination of labels and icons");
    gap(Dp(16.0));
    do_dont(
        "Use globally recognized icons when using icons alone",
        "Don't use icons and labels on only some tabs; keep the treatment consistent across the set",
    )
    .good_figure(|| {
        mock_icon_tabs(vec![false, false, false]);
    })
    .bad_figure(|| {
        mock_icon_tabs(vec![true, true, false]);
    });
    gap(Dp(32.0));
    subheading("Label".into());
    gap(Dp(16.0));
    paragraph(
        "Labels should clearly and succinctly describe the content of a tab and fit on a single row. A label can use a second line with truncated text, or the set can switch to scrollable tabs to fit longer titles."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(false, false, false, 0, 2);
    gap(Dp(8.0));
    caption(
        "Tab labels should be short and succinct. There should be a clear relationship to the title above.",
    );
    gap(Dp(24.0));
    paragraph(
        "Scrollable tabs offset the first tab 52dp from the leading edge, and the width of each tab is set by its label. Avoid inconsistent padding between tabs."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, false, true, 0, 5);
    gap(Dp(16.0));
    do_dont(
        "Offset the first scrollable tab 52dp from the leading edge",
        "Don't truncate labels unless required",
    )
    .good_figure(|| {
        mock_label_tabs(vec![
            "Akita".into(),
            "Alaskan Malamute".into(),
            "Australian Shepherd".into(),
            "Azawakh".into(),
        ])
        .scrollable(true);
    })
    .bad_figure(|| {
        mock_label_tabs(vec![
            "Akita".into(),
            "Alaskan Malamute".into(),
            "Australian Shepherd".into(),
            "Azawakh".into(),
        ])
        .width(Dp(240.0));
    });
    gap(Dp(32.0));
    subheading("Badges (optional)".into());
    gap(Dp(16.0));
    paragraph(
        "Badges show notifications or updates related to the content of one tab. Limit badge text to four characters, including a plus sign, and update or remove the badge once its content has been viewed. Both small and large badges can be used."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("Badges are used to highlight notifications related to tab specific content");
    gap(Dp(32.0));
    subheading("Active indicator".into());
    gap(Dp(16.0));
    paragraph(
        "The active indicator is an underline combined with a color change of the active tab's text and icon."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("An underline and color change differentiate an active tab from the inactive ones");
}

#[tessera]
fn choosing_variant() {
    paragraph(
        "Use primary tabs when only one set of tabs is needed. Use secondary tabs when a screen needs more than one level of hierarchy; they have a simpler indicator but the same function."
            .into(),
    );
    gap(Dp(32.0));
    nested_tabs();
    gap(Dp(8.0));
    caption(
        "Tabs can be joined with components like app bars, embedded in a specific UI region, or nested within components like cards and sheets. Tabs control the UI region displayed below them.",
    );
}

#[tessera]
fn placement() {
    paragraph(
        "Tabs are displayed in a single row, with each tab connected to the content it represents. As a set, all tabs are unified by a shared topic. Secondary tabs are always placed below primary tabs."
            .into(),
    );
    gap(Dp(32.0));
    tab_placement();
    gap(Dp(8.0));
    caption("Secondary tabs are found within other content to assist users with greater detail");
}

#[tessera]
fn responsive() {
    paragraph(
        "The maximum width of fixed tabs is set by the widest tab. Fixed tabs use a fluid margin and are aligned to the center or leading edge of the body region. Avoid more than four fixed tabs; with five or more, the container becomes cramped."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, false, false, 0, 4);
    gap(Dp(8.0));
    caption("Tabs can grow in width in relation to the number of items contained within");
}

#[tessera]
fn behavior() {
    subheading("States".into());
    gap(Dp(16.0));
    paragraph(
        "Tabs are enabled with one active tab. Each tab can also show hover, focused and pressed states."
            .into(),
    );
    gap(Dp(24.0));
    state_matrix(true);
    gap(Dp(8.0));
    caption("Active, hover, focused, and pressed states");
    gap(Dp(32.0));
    subheading("Fixed tabs".into());
    gap(Dp(16.0));
    paragraph(
        "Fixed tabs display all tabs on one screen, which suits quick switching between related content. To navigate, tap a tab or swipe left and right in the content area."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, false, false, 0, 3);
    gap(Dp(8.0));
    caption("Fixed tabs allow users to see all possible kinds of content available");
    gap(Dp(32.0));
    subheading("Tap a tab".into()).level(4);
    gap(Dp(16.0));
    paragraph("Navigate to a tab by tapping on it.".into());
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("Tapping on a tab directly");
    gap(Dp(32.0));
    subheading("Swipe within the content area".into()).level(4);
    gap(Dp(16.0));
    paragraph(
        "Swipe left or right in the content area to move between fixed tabs. Use caution when the content area also holds swipeable content, such as swipe-to-delete list items."
            .into(),
    );
    gap(Dp(24.0));
    swipe_tabs();
    gap(Dp(8.0));
    caption("Users can swipe between fixed tabs to see related content quickly");
    gap(Dp(16.0));
    do_dont(
        "Use gestures in different directions from the tab swipe for content in the area",
        "Don't place swipeable items in the content area of swipeable tabs",
    )
    .good_figure(|| {
        mock_tabs_with_rows(false);
    })
    .bad_figure(|| {
        mock_tabs_with_rows(true);
    });
    gap(Dp(32.0));
    subheading("Scrollable tabs".into());
    gap(Dp(16.0));
    paragraph(
        "Scrollable tabs are used when tabs can't fit on screen or have longer labels, and they are best for browsing on touch interfaces. Padding between labels stays the same as the row scrolls."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, false, true, 0, 5);
    gap(Dp(8.0));
    caption("Padding should remain the same when using scrollable tabs and long labels");
    gap(Dp(32.0));
    subheading("Scrolling content".into());
    gap(Dp(16.0));
    paragraph(
        "When the screen scrolls, tabs can stay fixed to the top of the screen, or scroll offscreen and reappear when the user scrolls up. Tabs attached to an app bar or other component move with it as a single unit."
            .into(),
    );
    gap(Dp(24.0));
    scrolling_tabs();
    gap(Dp(8.0));
    caption("Tabs can be used to create elevation");
    gap(Dp(8.0));
    caption(
        "Scroll the list: the app bar and tabs collapse together, and scrolling back up brings them back",
    );
    gap(Dp(16.0));
    do_dont(
        "Tabs can scroll offscreen and reappear when the user scrolls up",
        "Don't scroll tabs behind an app bar; tabs attached to a component should move as a single unit",
    )
    .good_figure(|| {
        mock_scrolled_screen(false);
    })
    .bad_figure(|| {
        mock_scrolled_screen(true);
    });
}

#[tessera]
fn use_cases() {
    paragraph("Users should be able to:".into());
    gap(Dp(8.0));
    bullets(vec![
        "Undertake actions or invoke navigation to a new destination with assistive technology"
            .into(),
        "Select an action or destination from an off-screen tab using assistive technology".into(),
        "Maintain access to primary actions when the content is scrolled".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    subheading("Touch".into());
    gap(Dp(16.0));
    paragraph(
        "When a user taps a tab, a touch ripple appears as interaction feedback, and the selected indicator becomes active and shifts into position once the touch has been engaged."
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("Touch: Tap");
    gap(Dp(32.0));
    subheading("Scrollable".into());
    gap(Dp(16.0));
    bullets(vec![
        "Scrollable tabs are used when tabs can't fit on screen, and are best for browsing on touch interfaces".into(),
        "Swipe left or right, or use arrow or tab keys, to navigate through the tabs".into(),
        "Looping the tabs infinitely is not recommended, because it traps screen reader users".into(),
        "Select an individual tab with a tap, or with Space or Enter".into(),
        "Horizontally scrolling tabs meet accessibility requirements because labels need room to grow".into(),
    ]);
    gap(Dp(24.0));
    native_tabs_figure(true, false, true, 0, 5);
    gap(Dp(8.0));
    caption("Scrollable: Scrollable Tabs");
    gap(Dp(32.0));
    subheading("Cursor".into());
    gap(Dp(16.0));
    paragraph(
        "Hovering a tab shows a hover state as a visual cue that it is interactive. Clicking an active or inactive tab shows a ripple and shifts the indicator into position."
            .into(),
    );
    gap(Dp(24.0));
    state_matrix(true);
    gap(Dp(8.0));
    caption("Cursor: Hover, Click");
    gap(Dp(32.0));
    subheading("Keyboard/Switch".into());
    gap(Dp(16.0));
    bullets(vec![
        "A focus indicator appears when the user tabs into the set".into(),
        "Space or Enter on the selected tab navigates to a new destination".into(),
        "Within the tab menu, arrow or tab keys move through items, Space or Enter selects an item, and the tab key exits the menu".into(),
    ]);
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("Keyboard/Switch: Tab, Space/Enter, Arrow");
    gap(Dp(32.0));
    subheading("Avoid applying density by default".into());
    gap(Dp(16.0));
    paragraph(
        "Don't apply density to tabs by default, because it lowers their targets below the best practice of 48x48 CSS pixels. Instead, give people a way to choose a higher density, like selecting a denser layout or changing the theme."
            .into(),
    );
    gap(Dp(16.0));
    paragraph(
        "To ensure that the density setting can be easily reverted when it's active, keep every target that changes it at a minimum of 48x48 CSS pixels."
            .into(),
    );
}

#[tessera]
fn initial_focus() {
    paragraph(
        "On arrow or tab in a tab menu, the active indicator appears on the first interactive element, providing feedback to the user that it is selected. The user can then tab to additional interactive elements until all available items within the tab menu are complete."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(16.0));
    do_dont(
        "Use Arrow/Tab to navigate through items",
        "Don't use Space/Enter for navigating tabs. Space/Enter is only used for completing actions.",
    )
    .good_figure(|| {
        mock_focus_tabs(false);
    })
    .bad_figure(|| {
        mock_focus_tabs(true);
    });
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec![
                "Arrow".into(),
                "Focus lands on the next available navigation destination".into(),
            ],
            vec![
                "Space / Enter".into(),
                "Activates the focused navigation destination".into(),
            ],
            vec![
                "Arrow".into(),
                "Allows navigation through menu items".into(),
            ],
        ],
    );
}

#[tessera]
fn labelling() {
    paragraph(
        "When the visible UI text is ambiguous, or there is no visible UI text, accessibility labels need to be more descriptive. For example, an icon that visually represents a video camera requires additional information in its accessibility label to clarify the tab's intent."
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(16.0));
    spec_table(
        vec!["Element".into(), "Label".into(), "Role".into()],
        vec![vec![
            "Video tab".into(),
            "Video format media content".into(),
            "Tab".into(),
        ]],
    );
    gap(Dp(8.0));
    caption(
        "While the icon visually represents a \"Video camera\", the accessibility label for this tab clarifies its function: \"Video format media content\"",
    );
}

#[tessera]
fn sample(#[prop(into)] label: String, primary: bool, controls: Option<bool>) {
    let selected = remember(|| 0_usize);
    let icons = remember(|| false);
    let badge = remember(|| false);
    let scrollable = remember(|| false);
    let automatic = remember(|| false);
    let enabled = remember(|| true);
    let count = remember(|| 0_u32);
    let prefix = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(format!("{label} tabs preview")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if controls.unwrap_or(false) {
                let label = prefix.clone();
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        if primary {
                            button()
                                .tonal()
                                .selected(icons.get())
                                .accessibility_label("Show primary tab icons")
                                .on_click(move || icons.set(!icons.get()))
                                .child(|| {
                                    text().content("Icons");
                                });
                        }
                        button()
                            .tonal()
                            .selected(badge.get())
                            .accessibility_label(format!("{label} tab badge"))
                            .on_click(move || badge.set(!badge.get()))
                            .child(|| {
                                text().content("Badge");
                            });
                        button()
                            .tonal()
                            .selected(scrollable.get())
                            .accessibility_label(format!("{label} scrollable tabs"))
                            .on_click(move || {
                                scrollable.set(!scrollable.get());
                                if !scrollable.get() && selected.get() > 2 {
                                    selected.set(2);
                                }
                            })
                            .child(|| {
                                text().content("Scrollable");
                            });
                        button()
                            .tonal()
                            .selected(automatic.get())
                            .accessibility_label(format!("{label} automatic activation"))
                            .on_click(move || automatic.set(!automatic.get()))
                            .child(|| {
                                text().content("Automatic activation");
                            });
                        button()
                            .tonal()
                            .selected(!enabled.get())
                            .accessibility_label(format!("{label} disable Hotels"))
                            .on_click(move || enabled.set(!enabled.get()))
                            .child(|| {
                                text().content("Disable Hotels");
                            });
                        button()
                            .text()
                            .accessibility_label(format!("{label} select final tab"))
                            .on_click(move || selected.set(if scrollable.get() { 4 } else { 2 }))
                            .child(|| {
                                text().content("Select last");
                            });
                    });
                gap(Dp(24.0));
            }
            tabs()
                .variant(if primary {
                    TabsVariant::Primary
                } else {
                    TabsVariant::Secondary
                })
                .selected(selected)
                .scrollable(scrollable.get())
                .activation_mode(if automatic.get() {
                    TabActivationMode::Automatic
                } else {
                    TabActivationMode::Manual
                })
                .min_tab_height(Dp(if icons.get() && primary { 64.0 } else { 48.0 }))
                .modifier(Modifier::new().fill_max_width())
                .content(move || {
                    let visible = if scrollable.get() { 5 } else { 3 };
                    for (index, name) in DESTINATIONS.into_iter().take(visible).enumerate() {
                        let available = index != 1 || enabled.get();
                        if index == 1 && badge.get() {
                            tab()
                                .enabled(available)
                                .accessibility_label(format!("{name}, 3 new items"))
                                .on_click(move || count.set(count.get() + 1))
                                .title_shared(RenderSlot::new(move || {
                                    badge_title(name, icons.get() && primary, index);
                                }));
                        } else if icons.get() && primary {
                            tab()
                                .label(name)
                                .icon(destination_asset(index))
                                .enabled(available)
                                .accessibility_label(name.to_string())
                                .on_click(move || count.set(count.get() + 1));
                        } else {
                            tab()
                                .label(name)
                                .enabled(available)
                                .accessibility_label(name.to_string())
                                .on_click(move || count.set(count.get() + 1));
                        }
                    }
                });
            gap(Dp(24.0));
            text()
                .content(format!(
                    "{prefix} content: {}",
                    DESTINATIONS[selected.get()]
                ))
                .style(type_style(22.0, 30.0));
            gap(Dp(8.0));
            text()
                .content(
                    [
                        "Search flights and manage upcoming journeys.",
                        "Review hotel bookings and saved stays.",
                        "Find train departures and booked seats.",
                        "Check bus routes and ticket details.",
                        "View ferry crossings and boarding details.",
                    ][selected.get()],
                )
                .style(type_style(16.0, 24.0));
            text()
                .content(format!("{prefix} activations: {}", count.get()))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn badge_title(#[prop(into)] label: String, icon_shown: bool, index: usize) {
    if icon_shown {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                badged_box()
                    .content(move || {
                        icon().painter(destination_asset(index)).size(Dp(24.0));
                    })
                    .badge(|| {
                        badge_with_content().content(|| {
                            text().content("3");
                        });
                    });
                text().content(label.clone());
            });
    } else {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                text().content(label.clone());
                spacer().modifier(Modifier::new().width(Dp(4.0)));
                badge_with_content().content(|| {
                    text().content("3");
                });
            });
    }
}

#[tessera]
fn section(primary: bool) {
    panel(RenderSlot::new(move || {
        sample(if primary { "Primary" } else { "Secondary" }, primary).controls(true);
    }));
    gap(Dp(32.0));
    native_tabs_figure(primary, primary, false, 1, if primary { 3 } else { 2 });
    gap(Dp(24.0));
    legend_list(if primary {
        vec![
            "Container".into(),
            "Badge (optional)".into(),
            "Icon (optional)".into(),
            "Label".into(),
            "Divider".into(),
            "Active indicator".into(),
        ]
    } else {
        vec![
            "Container".into(),
            "Badge (optional)".into(),
            "Label".into(),
            "Divider".into(),
            "Active indicator".into(),
        ]
    });
    gap(Dp(32.0));
    subheading(if primary {
        "Primary tabs color".into()
    } else {
        "Secondary tabs color".into()
    });
    gap(Dp(16.0));
    paragraph("Color values are implemented through design tokens.".into());
    gap(Dp(24.0));
    for dark in [false, true] {
        themed_panel(
            dark,
            RenderSlot::new(move || {
                native_tabs_figure(primary, primary, false, 0, if primary { 3 } else { 2 });
            }),
        );
        gap(Dp(16.0));
    }
    legend_list(if primary {
        vec![
            "Surface".into(),
            "Primary".into(),
            "Primary".into(),
            "On surface variant".into(),
            "On surface variant".into(),
            "Outline variant".into(),
            "Primary".into(),
        ]
    } else {
        vec![
            "Surface".into(),
            "On surface".into(),
            "On surface variant".into(),
            "Outline variant".into(),
            "Primary".into(),
        ]
    });
    gap(Dp(32.0));
    subheading(if primary {
        "Primary tabs states".into()
    } else {
        "Secondary tabs states".into()
    });
    gap(Dp(24.0));
    for dark in [false, true] {
        themed_panel(
            dark,
            RenderSlot::new(move || {
                state_matrix(primary);
            }),
        );
        gap(Dp(16.0));
    }
    caption(
        "Enabled, hover, focused, and pressed states for active and inactive destinations, in light and dark themes",
    );
}

#[tessera]
fn themed_panel(dark: bool, content: RenderSlot) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            let content = content.clone();
            surface()
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    let content = content.clone();
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                        .children(move || {
                            text()
                                .content(if dark { "Dark" } else { "Light" })
                                .style(type_style(14.0, 20.0));
                            gap(Dp(16.0));
                            column()
                                .modifier(Modifier::new().fill_max_width())
                                .children_shared(content.clone());
                        });
                });
        });
}

#[tessera]
fn state_matrix(primary: bool) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for active in [true, false] {
                text()
                    .content(if active {
                        "Active destination"
                    } else {
                        "Inactive destination"
                    })
                    .style(type_style(14.0, 20.0));
                gap(Dp(8.0));
                flow_row()
                    .item_spacing(Dp(12.0))
                    .line_spacing(Dp(12.0))
                    .children(move || {
                        for state in 0..4 {
                            state_cell(primary, active, state);
                        }
                    });
                gap(Dp(16.0));
            }
        });
}

#[tessera]
fn state_cell(primary: bool, active: bool, state: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let content = if !active {
        if state == 0 {
            scheme.on_surface_variant
        } else {
            scheme.on_surface
        }
    } else if primary {
        scheme.primary
    } else {
        scheme.on_surface
    };
    let layer = if active && primary {
        scheme.primary
    } else {
        scheme.on_surface
    };
    let (container, focus_ring, indicator, divider) = (
        scheme.surface,
        scheme.secondary,
        scheme.primary,
        scheme.outline_variant,
    );
    let alpha = [0.0, 0.08, 0.1, 0.1][state];
    let name = ["Enabled", "Hover", "Focused", "Pressed"][state];
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(120.0), Dp(48.0))
                        .background(container)
                        .background(layer.with_alpha(alpha)),
                )
                .children(move || {
                    if state == 2 {
                        boxed().modifier(
                            Modifier::new().size(Dp(116.0), Dp(44.0)).border_with_shape(
                                Dp(3.0),
                                focus_ring,
                                Shape::rounded_rectangle(Dp(8.0)),
                            ),
                        );
                    }
                    text()
                        .content("Label")
                        .color(content)
                        .style(type_style(14.0, 20.0));
                    boxed()
                        .modifier(
                            Modifier::new()
                                .fill_max_size()
                                .align(Alignment::BottomCenter),
                        )
                        .alignment(Alignment::BottomCenter)
                        .children(move || {
                            if active {
                                boxed().modifier(if primary {
                                    Modifier::new()
                                        .size(Dp(40.0), Dp(3.0))
                                        .background_with_shape(
                                            indicator,
                                            Shape::rounded_rectangle(Dp(3.0)),
                                        )
                                } else {
                                    Modifier::new()
                                        .size(Dp(120.0), Dp(2.0))
                                        .background(indicator)
                                });
                            } else {
                                boxed().modifier(
                                    Modifier::new().size(Dp(120.0), Dp(1.0)).background(divider),
                                );
                            }
                        });
                });
            gap(Dp(4.0));
            text().content(name).style(type_style(12.0, 16.0));
        });
}

const TOKEN_GROUPS: [(&str, &[(&str, &str, &str)]); 4] = [
    (
        "Enabled",
        &[
            (
                "Container color",
                "md.comp.primary-navigation-tab.container.color",
                "Surface",
            ),
            (
                "Container height",
                "md.comp.primary-navigation-tab.container.height",
                "48dp",
            ),
            (
                "Container elevation",
                "md.comp.primary-navigation-tab.container.elevation",
                "Level 0",
            ),
            (
                "Container shape",
                "md.comp.primary-navigation-tab.container.shape",
                "0dp",
            ),
            (
                "Icon and label container height",
                "md.comp.primary-navigation-tab.with-icon-and-label-text.container.height",
                "64dp",
            ),
            (
                "Active indicator color",
                "md.comp.primary-navigation-tab.active-indicator.color",
                "Primary",
            ),
            (
                "Active indicator height",
                "md.comp.primary-navigation-tab.active-indicator.height",
                "3dp",
            ),
            (
                "Active indicator shape",
                "md.comp.primary-navigation-tab.active-indicator.shape",
                "3dp, 3dp, 0, 0",
            ),
            (
                "Active label text color",
                "md.comp.primary-navigation-tab.with-label-text.active.label-text.color",
                "Primary",
            ),
            (
                "Inactive label text color",
                "md.comp.primary-navigation-tab.with-label-text.inactive.label-text.color",
                "On surface variant",
            ),
            (
                "Label text font",
                "md.comp.primary-navigation-tab.with-label-text.label-text.font",
                "Title small",
            ),
            (
                "Label text size",
                "md.comp.primary-navigation-tab.with-label-text.label-text.size",
                "14sp",
            ),
            (
                "Label text weight",
                "md.comp.primary-navigation-tab.with-label-text.label-text.weight",
                "500",
            ),
            (
                "Label text line height",
                "md.comp.primary-navigation-tab.with-label-text.label-text.line-height",
                "20sp",
            ),
            (
                "Active icon color",
                "md.comp.primary-navigation-tab.with-icon.active.icon.color",
                "Primary",
            ),
            (
                "Inactive icon color",
                "md.comp.primary-navigation-tab.with-icon.inactive.icon.color",
                "On surface variant",
            ),
            (
                "Icon size",
                "md.comp.primary-navigation-tab.with-icon.icon.size",
                "24dp",
            ),
            (
                "Divider color",
                "md.comp.primary-navigation-tab.divider.color",
                "Outline variant",
            ),
            (
                "Divider height",
                "md.comp.primary-navigation-tab.divider.height",
                "1dp",
            ),
        ],
    ),
    (
        "Hovered",
        &[
            (
                "Active state layer color",
                "md.comp.primary-navigation-tab.active.hover.state-layer.color",
                "Primary",
            ),
            (
                "Inactive state layer color",
                "md.comp.primary-navigation-tab.inactive.hover.state-layer.color",
                "On surface",
            ),
            (
                "State layer opacity",
                "md.comp.primary-navigation-tab.hover.state-layer.opacity",
                "0.08",
            ),
            (
                "Active label text color",
                "md.comp.primary-navigation-tab.with-label-text.active.hover.label-text.color",
                "Primary",
            ),
            (
                "Inactive label text color",
                "md.comp.primary-navigation-tab.with-label-text.inactive.hover.label-text.color",
                "On surface",
            ),
            (
                "Active icon color",
                "md.comp.primary-navigation-tab.with-icon.active.hover.icon.color",
                "Primary",
            ),
            (
                "Inactive icon color",
                "md.comp.primary-navigation-tab.with-icon.inactive.hover.icon.color",
                "On surface",
            ),
        ],
    ),
    (
        "Focused",
        &[
            (
                "Active state layer color",
                "md.comp.primary-navigation-tab.active.focus.state-layer.color",
                "Primary",
            ),
            (
                "Inactive state layer color",
                "md.comp.primary-navigation-tab.inactive.focus.state-layer.color",
                "On surface",
            ),
            (
                "State layer opacity",
                "md.comp.primary-navigation-tab.focus.state-layer.opacity",
                "0.1",
            ),
            (
                "Active label text color",
                "md.comp.primary-navigation-tab.with-label-text.active.focus.label-text.color",
                "Primary",
            ),
            (
                "Inactive label text color",
                "md.comp.primary-navigation-tab.with-label-text.inactive.focus.label-text.color",
                "On surface",
            ),
            (
                "Active icon color",
                "md.comp.primary-navigation-tab.with-icon.active.focus.icon.color",
                "Primary",
            ),
            (
                "Inactive icon color",
                "md.comp.primary-navigation-tab.with-icon.inactive.focus.icon.color",
                "On surface",
            ),
        ],
    ),
    (
        "Pressed (ripple)",
        &[
            (
                "Active state layer color",
                "md.comp.primary-navigation-tab.active.pressed.state-layer.color",
                "Primary",
            ),
            (
                "Inactive state layer color",
                "md.comp.primary-navigation-tab.inactive.pressed.state-layer.color",
                "On surface",
            ),
            (
                "State layer opacity",
                "md.comp.primary-navigation-tab.pressed.state-layer.opacity",
                "0.1",
            ),
            (
                "Active label text color",
                "md.comp.primary-navigation-tab.with-label-text.active.pressed.label-text.color",
                "Primary",
            ),
            (
                "Inactive label text color",
                "md.comp.primary-navigation-tab.with-label-text.inactive.pressed.label-text.color",
                "On surface",
            ),
            (
                "Active icon color",
                "md.comp.primary-navigation-tab.with-icon.active.pressed.icon.color",
                "Primary",
            ),
            (
                "Inactive icon color",
                "md.comp.primary-navigation-tab.with-icon.inactive.pressed.icon.color",
                "On surface",
            ),
        ],
    ),
];

#[tessera]
fn tokens() {
    subheading("Tabs - Primary navigation".into());
    gap(Dp(24.0));
    for (state, rows) in TOKEN_GROUPS {
        subheading(state.to_string()).level(4);
        gap(Dp(16.0));
        spec_table(
            vec!["Element".into(), "Token".into(), "Value".into()],
            rows.iter()
                .map(|(element, token, value)| {
                    vec![element.to_string(), token.to_string(), value.to_string()]
                })
                .collect(),
        );
        gap(Dp(32.0));
    }
}

#[tessera]
fn measurements() {
    native_tabs_figure(true, true, false, 2, 4);
    gap(Dp(32.0));
    native_tabs_figure(true, false, false, 2, 2);
    gap(Dp(32.0));
    secondary_measure_figure(false);
    gap(Dp(8.0));
    caption(
        "Secondary tabs: 48dp container, 1/2 width tabs, 2dp full-width indicator, 4dp label-to-badge spacing",
    );
    gap(Dp(32.0));
    secondary_measure_figure(true);
    gap(Dp(8.0));
    caption(
        "Secondary tabs with inline icons: 24dp icon, 8dp icon-to-label spacing, 4dp label-to-badge spacing",
    );
    gap(Dp(32.0));
    badge_legend();
    gap(Dp(32.0));
    paragraph(
        "Tabs are divided into equal sections, with labels and icons positioned vertically centered. The divider is included in the height, placed inside the container."
            .into(),
    );
    gap(Dp(24.0));
    paragraph(
        "Primary tab active indicators are inset 2dp on each side, have a fully rounded corner radius, and a minimum length of 24dp."
            .into(),
    );
    gap(Dp(32.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Container height (label only)".into(), "48dp".into()],
            vec!["Container height (icon + label)".into(), "64dp".into()],
            vec!["Icon size".into(), "24dp".into()],
            vec!["Divider height".into(), "1dp".into()],
            vec!["Primary indicator height".into(), "3dp".into()],
            vec!["Secondary indicator height".into(), "2dp".into()],
            vec!["Active indicator shape".into(), "3dp, 3dp, 0, 0".into()],
            vec!["Active indicator minimum length".into(), "24dp".into()],
            vec!["Inline icon-text padding".into(), "8dp".into()],
            vec!["Inline text-badge padding".into(), "4dp".into()],
            vec!["Badge overlap on stacked icon".into(), "6dp".into()],
        ],
    );
}

#[tessera]
fn badge_legend() {
    flow_row()
        .item_spacing(Dp(32.0))
        .line_spacing(Dp(16.0))
        .children(|| {
            for (index, name) in [
                "Small",
                "Large single digit",
                "Large multiple digits",
                "Large 999+",
            ]
            .into_iter()
            .enumerate()
            {
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().size(Dp(48.0), Dp(24.0)))
                            .children(move || {
                                if index == 0 {
                                    badge();
                                } else {
                                    badge_with_content().content(move || {
                                        text().content(["", "3", "32", "999+"][index]);
                                    });
                                }
                            });
                        gap(Dp(8.0));
                        text().content(name).style(type_style(14.0, 20.0));
                    });
            }
        });
}

#[tessera]
fn swipe_tabs(selected: Option<State<usize>>) {
    let selected = if let Some(selected) = selected {
        selected
    } else {
        remember(|| 0usize)
    };
    let travel = remember(|| 0.0f32);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (container, hint) = (scheme.surface, scheme.on_surface_variant);
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(container, Shape::rounded_rectangle(Dp(24.0)))
                .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
        )
        .children(move || {
            tabs()
                .selected(selected)
                .modifier(Modifier::new().fill_max_width())
                .content(|| {
                    for (name, _) in SWIPE_PAGES {
                        tab().label(name);
                    }
                });
            let (title, detail) = SWIPE_PAGES[selected.get().min(SWIPE_PAGES.len() - 1)];
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(200.0))
                        .draggable_with(tessera_components::modifier::DraggableArgs {
                            on_drag_delta: tessera_ui::CallbackWith::new(
                                move |delta: tessera_components::modifier::DragDelta| {
                                    travel.set(travel.get() + delta.x.to_f32());
                                },
                            ),
                            on_drag_stopped: Some(tessera_ui::Callback::new(move || {
                                let threshold = Dp(48.0).to_pixels_f32();
                                let current = selected.get();
                                let distance = travel.get();
                                if distance <= -threshold && current + 1 < SWIPE_PAGES.len() {
                                    selected.set(current + 1);
                                } else if distance >= threshold && current > 0 {
                                    selected.set(current - 1);
                                }
                                travel.set(0.0);
                            })),
                            ..Default::default()
                        })
                        .semantics(SemanticsArgs {
                            role: Some(Role::TabPanel),
                            label: Some(format!("{title} content")),
                            ..Default::default()
                        })
                        .padding_all(Dp(24.0)),
                )
                .children(move || {
                    text().content(title).style(type_style(24.0, 32.0));
                    gap(Dp(8.0));
                    text().content(detail).style(type_style(16.0, 24.0));
                    gap(Dp(16.0));
                    text()
                        .content("Swipe left or right to change tabs")
                        .color(hint)
                        .style(type_style(14.0, 20.0));
                });
        });
}

fn destination_asset(index: usize) -> tessera_components::material_icons::Asset {
    [
        filled::FLIGHT_SVG,
        filled::HOTEL_SVG,
        filled::TRAIN_SVG,
        filled::DIRECTIONS_BUS_SVG,
        filled::DIRECTIONS_BOAT_SVG,
    ][index]
}

const LABEL_KEYS: [&str; 5] = ["label0", "label1", "label2", "label3", "label4"];
const ICON_KEYS: [&str; 5] = ["icon0", "icon1", "icon2", "icon3", "icon4"];

fn figure_name(primary: bool, index: usize) -> &'static str {
    if primary {
        ["Flights", "Trips", "Explore", "Buy", "Ferries"][index]
    } else {
        ["Overview", "Specifications", "Details", "Updates", "More"][index]
    }
}

fn figure_asset(primary: bool, index: usize) -> tessera_components::material_icons::Asset {
    if primary {
        [
            filled::FLIGHT_SVG,
            filled::LUGGAGE_SVG,
            filled::EXPLORE_SVG,
            filled::SHOPPING_CART_SVG,
            filled::DIRECTIONS_BOAT_SVG,
        ][index]
    } else {
        filled::DESCRIPTION_SVG
    }
}

#[tessera]
fn native_tabs_figure(
    primary: bool,
    icons: bool,
    scrollable: bool,
    annotation: usize,
    count: usize,
) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0usize);
    let unread = remember(|| true);
    let art_width = remember(|| Dp(900.0));
    let scale = art_width.get().0 / 1024.0;
    let count = if !primary && !scrollable { 2 } else { count };
    if !unread.get() && geometry.with(|values| values.contains_key("badge")) {
        geometry.with_mut(|values| {
            values.remove("badge");
        });
    }
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(300.0 * scale))
                        .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                            let next = Dp::from_pixels_f32(bounds.width.to_f32());
                            if art_width.get() != next {
                                art_width.set(next);
                            }
                        })
                        .background_with_shape(
                            scheme.surface_container,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_size()
                                .padding_symmetric(Dp::ZERO, Dp::ZERO),
                        )
                        .children(move || {
                            boxed()
                                .modifier(watch(geometry, "container").width(Dp(360.0 * scale)))
                                .children(move || {
                                    tabs()
                                        .variant(if primary {
                                            TabsVariant::Primary
                                        } else {
                                            TabsVariant::Secondary
                                        })
                                        .selected(selected)
                                        .scrollable(scrollable)
                                        .edge_padding(Dp(52.0 * scale))
                                        .tab_padding(Dp(16.0 * scale))
                                        .indicator_height(Dp(
                                            if primary { 3.0 } else { 2.0 } * scale
                                        ))
                                        .indicator_min_width(Dp(24.0 * scale))
                                        .min_tab_height(Dp(if icons { 64.0 } else { 48.0 } * scale))
                                        .modifier(Modifier::new().fill_max_width())
                                        .content(move || {
                                            for (index, name) in (0..count)
                                                .map(|index| (index, figure_name(primary, index)))
                                            {
                                                tab()
                                                    .accessibility_label(
                                                        if index == 0 && unread.get() {
                                                            format!("{name}, 3 new items")
                                                        } else {
                                                            name.into()
                                                        },
                                                    )
                                                    .on_click(move || {
                                                        if index == 0 {
                                                            unread.set(false);
                                                        }
                                                    })
                                                    .title_shared(RenderSlot::new(move || {
                                                        native_tab_title(
                                                            index,
                                                            icons,
                                                            index == 0 && unread.get(),
                                                            geometry,
                                                            scale,
                                                            primary,
                                                            false,
                                                        );
                                                    }));
                                            }
                                        });
                                });
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            if annotation > 0 {
                                tab_annotations(
                                    primary,
                                    icons,
                                    annotation == 2,
                                    annotation == 3,
                                    selected.get(),
                                    geometry,
                                );
                            }
                        });
                });
            gap(Dp(16.0));
            text()
                .content(format!(
                    "Figure content: {}",
                    figure_name(primary, selected.get())
                ))
                .style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn native_tab_title(
    index: usize,
    icons: bool,
    unread: bool,
    geometry: State<Geometry>,
    scale: f64,
    primary: bool,
    inline: bool,
) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let mut style = theme.typography.title_small;
    style.font_size = style.font_size * scale;
    style.line_height = style.line_height.map(|height| height * scale);
    let color = use_context::<tessera_components::theme::ContentColor>()
        .map(|color| color.get().current)
        .unwrap_or(theme.color_scheme.on_surface);
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            if icons && !inline {
                boxed()
                    .modifier(watch(geometry, ICON_KEYS[index]))
                    .children(move || {
                        badged_box()
                            .content(move || {
                                icon()
                                    .painter(figure_asset(primary, index))
                                    .size(Dp(24.0 * scale))
                                    .tint(color);
                            })
                            .badge(move || {
                                if unread {
                                    figure_badge(geometry, scale);
                                }
                            });
                    });
                gap(Dp(2.0 * scale));
                text()
                    .content(figure_name(primary, index))
                    .style(style)
                    .color(color)
                    .modifier(watch(geometry, LABEL_KEYS[index]));
            } else {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        if icons && inline {
                            boxed()
                                .modifier(watch(geometry, ICON_KEYS[index]))
                                .children(move || {
                                    icon()
                                        .painter(figure_asset(primary, index))
                                        .size(Dp(24.0 * scale))
                                        .tint(color);
                                });
                            spacer().modifier(Modifier::new().width(Dp(8.0 * scale)));
                        }
                        text()
                            .content(figure_name(primary, index))
                            .style(style)
                            .color(color)
                            .modifier(watch(geometry, LABEL_KEYS[index]));
                        if unread {
                            spacer().modifier(Modifier::new().width(Dp(4.0 * scale)));
                            figure_badge(geometry, scale);
                        }
                    });
            }
        });
}

#[tessera]
fn figure_badge(geometry: State<Geometry>, scale: f64) {
    let mut style = use_context::<MaterialTheme>()
        .unwrap()
        .get()
        .typography
        .label_small;
    style.font_size = style.font_size * scale;
    style.line_height = style.line_height.map(|height| height * scale);
    boxed()
        .modifier(watch(geometry, "badge").size(Dp(16.0 * scale), Dp(16.0 * scale)))
        .children(move || {
            badge_with_content().content(move || {
                text().content("3").style(style);
            });
        });
}

#[tessera]
fn tab_annotations(
    primary: bool,
    icons: bool,
    measurements: bool,
    guideline_order: bool,
    selected: usize,
    geometry: State<Geometry>,
) {
    let values = geometry.get();
    let (Some(root), Some(container), Some(label)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, LABEL_KEYS[selected]),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let scale = width.0 / 1024.0;
    let mut svg = String::new();
    if measurements {
        let tint = "#ec407a";
        annotations::outline(&mut svg, container);
        dotted(
            &mut svg,
            container.x + container.w + 12.0,
            container.y,
            container.h,
            true,
            tint,
        );
        annotations::dimension(
            if icons { "64" } else { "48" }.into(),
            Dp(container.x + container.w + 20.0),
            Dp(container.y + container.h * 0.5 - 10.0),
        );
        if let Some(icon) = relative(&values, ICON_KEYS[selected]) {
            annotations::outline(&mut svg, icon);
            annotations::dimension("24".into(), Dp(icon.x + icon.w + 16.0), Dp(icon.y));
        }
        let indicator_height = if primary { 3.0 } else { 2.0 };
        let indicator = Rect {
            x: label.x + 2.0,
            y: container.y + container.h - indicator_height,
            w: (label.w - 4.0).max(24.0),
            h: indicator_height,
        };
        svg.push_str(&format!("<rect x='{}' y='{}' width='2' height='{}' fill='{tint}' fill-opacity='.3'/><rect x='{}' y='{}' width='2' height='{}' fill='{tint}' fill-opacity='.3'/>",label.x,container.y,label.h,label.x+label.w-2.0,container.y,label.h));
        annotations::outline(&mut svg, indicator);
        annotations::dimension(
            if primary { "3" } else { "2" }.into(),
            Dp(indicator.x + indicator.w + 16.0),
            Dp(indicator.y),
        );
        annotations::dimension(
            "2".into(),
            Dp(label.x - 8.0),
            Dp(container.y + container.h + 24.0),
        );
        annotations::dimension(
            "2".into(),
            Dp(label.x + label.w),
            Dp(container.y + container.h + 24.0),
        );
    } else {
        let tint = "#49454f";
        let center_y = container.y + container.h * 0.5;
        figure_marker(
            1,
            Dp(container.x - 75.0 * scale),
            Dp(center_y - 16.0 * scale),
            scale,
        );
        dotted(
            &mut svg,
            container.x - 51.0 * scale,
            center_y,
            51.0 * scale,
            false,
            tint,
        );
        if let Some(badge) = relative(&values, "badge") {
            let x = badge.x + badge.w * 0.5;
            dotted(
                &mut svg,
                x,
                badge.y - 38.0 * scale,
                38.0 * scale,
                true,
                tint,
            );
            figure_marker(
                if guideline_order && primary && icons {
                    3
                } else {
                    2
                },
                Dp(x - 12.0 * scale),
                Dp(badge.y - 70.0 * scale),
                scale,
            );
        }
        if primary && icons {
            if let Some(icon) = relative(&values, ICON_KEYS[1]) {
                let x = icon.x + icon.w * 0.5;
                dotted(&mut svg, x, icon.y - 38.0 * scale, 38.0 * scale, true, tint);
                figure_marker(
                    if guideline_order { 2 } else { 3 },
                    Dp(x - 12.0 * scale),
                    Dp(icon.y - 70.0 * scale),
                    scale,
                );
            }
        }
        if let Some(label) = relative(&values, LABEL_KEYS[if primary { 2 } else { 1 }]) {
            let y = label.y + label.h * 0.5;
            dotted(&mut svg, label.x + label.w, y, 58.0 * scale, false, tint);
            figure_marker(
                if primary { 4 } else { 3 },
                Dp(label.x + label.w + 58.0 * scale),
                Dp(y - 16.0 * scale),
                scale,
            );
        }
        let divider_x = container.x + container.w * 0.75;
        let bottom = container.y + container.h;
        dotted(&mut svg, divider_x, bottom, 52.0 * scale, true, tint);
        figure_marker(
            if primary { 5 } else { 4 },
            Dp(divider_x - 12.0 * scale),
            Dp(bottom + 52.0 * scale),
            scale,
        );
        let indicator_x = label.x + label.w * 0.5;
        dotted(&mut svg, indicator_x, bottom, 52.0 * scale, true, tint);
        figure_marker(
            if primary { 6 } else { 5 },
            Dp(indicator_x - 12.0 * scale),
            Dp(bottom + 52.0 * scale),
            scale,
        );
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn nested_tabs() {
    let primary = remember(|| 0usize);
    let secondary = remember(|| 0usize);
    panel(RenderSlot::new(move || {
        tabs()
            .selected(primary)
            .modifier(Modifier::new().fill_max_width())
            .content(|| {
                tab().label("Travel");
                tab().label("Saved");
            });
        gap(Dp(24.0));
        tabs()
            .variant(TabsVariant::Secondary)
            .selected(secondary)
            .modifier(Modifier::new().fill_max_width())
            .content(|| {
                tab().label("Upcoming");
                tab().label("Past");
            });
        gap(Dp(32.0));
        text()
            .content(format!(
                "{} · {}",
                if primary.get() == 0 {
                    "Travel"
                } else {
                    "Saved"
                },
                if secondary.get() == 0 {
                    "Upcoming"
                } else {
                    "Past"
                }
            ))
            .style(type_style(22.0, 28.0));
    }));
}

#[tessera]
fn tab_placement() {
    let selected = remember(|| 0usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(360.0))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(24.0)))
                .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
        )
        .children(move || {
            tessera_components::app_bar::top_app_bar().title("Travel");
            tabs()
                .selected(selected)
                .modifier(Modifier::new().fill_max_width())
                .content(|| {
                    tab().label("Flights");
                    tab().label("Hotels");
                    tab().label("Trains");
                });
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .weight(1.0)
                        .padding_all(Dp(24.0)),
                )
                .children(move || {
                    text()
                        .content(DESTINATIONS[selected.get()])
                        .style(type_style(24.0, 32.0));
                    gap(Dp(16.0));
                    text()
                        .content("Related content appears below the tabs.")
                        .style(type_style(16.0, 24.0));
                });
        });
}

#[tessera]
fn guidance_card(good: bool, #[prop(into)] note: String, figure: RenderSlot) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (stage, hint) = (scheme.surface_container, scheme.on_surface_variant);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let figure = figure.clone();
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .clip_shape(Shape::rounded_rectangle(Dp(16.0))),
                )
                .children(move || {
                    let figure = figure.clone();
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .height(Dp(220.0))
                                .background(stage)
                                .padding_all(Dp(16.0)),
                        )
                        .children(move || {
                            column()
                                .modifier(Modifier::new().fill_max_width())
                                .children_shared(figure.clone());
                        });
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .height(Dp(36.0))
                                .background(if good { DO_COLOR } else { DONT_COLOR })
                                .padding(Padding::symmetric(Dp(12.0), Dp(0.0))),
                        )
                        .children(move || {
                            text()
                                .content(if good { "✓  Do" } else { "✕  Don't" })
                                .style(type_style(14.0, 20.0))
                                .color(Color::WHITE);
                        });
                });
            gap(Dp(12.0));
            text()
                .content(note.clone())
                .color(hint)
                .style(type_style(14.0, 20.0))
                .modifier(Modifier::new().fill_max_width());
        });
}

#[tessera]
fn mock_label_tabs(labels: Vec<String>, scrollable: Option<bool>, width: Option<Dp>) {
    let selected = remember(|| 0usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let surface_color = scheme.surface;
    boxed()
        .alignment(Alignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let labels = labels.clone();
            boxed()
                .modifier(
                    Modifier::new()
                        .width(width.unwrap_or(Dp(300.0)))
                        .background(surface_color),
                )
                .children(move || {
                    let labels = labels.clone();
                    tabs()
                        .selected(selected)
                        .scrollable(scrollable.unwrap_or(false))
                        .edge_padding(Dp(52.0))
                        .modifier(Modifier::new().fill_max_width())
                        .content(move || {
                            for label in labels.clone() {
                                tab().label(label);
                            }
                        });
                });
        });
}

#[tessera]
fn mock_icon_tabs(labelled: Vec<bool>) {
    let selected = remember(|| 0usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let surface_color = scheme.surface;
    boxed()
        .alignment(Alignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let labelled = labelled.clone();
            boxed()
                .modifier(Modifier::new().width(Dp(300.0)).background(surface_color))
                .children(move || {
                    let labelled = labelled.clone();
                    tabs()
                        .selected(selected)
                        .min_tab_height(Dp(64.0))
                        .modifier(Modifier::new().fill_max_width())
                        .content(move || {
                            for (index, with_label) in labelled.clone().into_iter().enumerate() {
                                let name = figure_name(true, index);
                                if with_label {
                                    tab().label(name).icon(figure_asset(true, index));
                                } else {
                                    tab()
                                        .icon(figure_asset(true, index))
                                        .accessibility_label(name.to_string());
                                }
                            }
                        });
                });
        });
}

#[tessera]
fn mock_list_rows(swipeable: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (row_color, error, hint) = (scheme.surface, scheme.error, scheme.on_surface_variant);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, title) in ["Lisbon", "Kyoto", "Oaxaca"].into_iter().enumerate() {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .height(Dp(40.0))
                            .background(row_color),
                    )
                    .children(move || {
                        if swipeable && index == 0 {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    Modifier::new().size(Dp(72.0), Dp(40.0)).background(error),
                                )
                                .children(|| {
                                    text()
                                        .content("Delete")
                                        .color(Color::WHITE)
                                        .style(type_style(12.0, 16.0));
                                });
                        }
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        text().content(title).style(type_style(14.0, 20.0));
                        if swipeable && index == 0 {
                            spacer().modifier(Modifier::new().width(Dp(12.0)));
                            text()
                                .content("← swipe")
                                .color(hint)
                                .style(type_style(12.0, 16.0));
                        }
                    });
                gap(Dp(2.0));
            }
        });
}

#[tessera]
fn mock_tabs_with_rows(swipeable: bool) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            mock_label_tabs(vec!["Flights".into(), "Trips".into(), "Explore".into()]);
            gap(Dp(8.0));
            mock_list_rows(swipeable);
        });
}

#[tessera]
fn mock_scrolled_screen(behind_bar: bool) {
    let selected = remember(|| 0usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (bar_color, page, hint) = (
        scheme.surface_container,
        scheme.surface,
        scheme.on_surface_variant,
    );
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(188.0))
                .background(page)
                .clip_to_bounds(),
        )
        .children(move || {
            if behind_bar {
                boxed()
                    .modifier(Modifier::new().fill_max_width().height(Dp(84.0)))
                    .children(move || {
                        boxed()
                            .modifier(Modifier::new().fill_max_width().offset(Dp(0.0), Dp(36.0)))
                            .children(move || {
                                tabs()
                                    .selected(selected)
                                    .modifier(Modifier::new().fill_max_width())
                                    .content(|| {
                                        tab().label("Flights");
                                        tab().label("Trips");
                                        tab().label("Explore");
                                    });
                            });
                        boxed()
                            .alignment(Alignment::CenterStart)
                            .modifier(
                                Modifier::new()
                                    .fill_max_width()
                                    .height(Dp(64.0))
                                    .background(bar_color)
                                    .padding(Padding::symmetric(Dp(16.0), Dp(0.0))),
                            )
                            .children(|| {
                                text().content("Travel").style(type_style(22.0, 28.0));
                            });
                    });
            } else {
                text()
                    .content("Scrolled down: the app bar and tabs moved offscreen together")
                    .color(hint)
                    .style(type_style(12.0, 16.0))
                    .modifier(Modifier::new().padding(Padding::symmetric(Dp(16.0), Dp(8.0))));
            }
            for title in ["Lisbon", "Kyoto", "Oaxaca", "Bergen"] {
                text()
                    .content(title)
                    .style(type_style(14.0, 20.0))
                    .modifier(Modifier::new().padding(Padding::symmetric(Dp(16.0), Dp(6.0))));
            }
        });
}

#[tessera]
fn mock_focus_tabs(space_enter: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let key_color = scheme.primary;
    let key_text = scheme.on_primary;
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .background_with_shape(key_color, Shape::rounded_rectangle(Dp(8.0)))
                        .padding(Padding::symmetric(Dp(12.0), Dp(6.0))),
                )
                .children(move || {
                    text()
                        .content(if space_enter {
                            "Space/Enter"
                        } else {
                            "Arrow/Tab"
                        })
                        .color(key_text)
                        .style(type_style(14.0, 20.0));
                });
            gap(Dp(16.0));
            row().children(|| {
                state_cell(true, true, 0);
                state_cell(true, false, 2);
            });
        });
}

#[tessera]
fn secondary_measure_figure(inline_icons: bool) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0usize);
    let art_width = remember(|| Dp(900.0));
    let scale = art_width.get().0 / 1024.0;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let stage = scheme.surface_container;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(300.0 * scale))
                        .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                            let next = Dp::from_pixels_f32(bounds.width.to_f32());
                            if art_width.get() != next {
                                art_width.set(next);
                            }
                        })
                        .background_with_shape(stage, Shape::rounded_rectangle(Dp(24.0))),
                )
                .children(move || {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            boxed()
                                .modifier(watch(geometry, "container").width(Dp(420.0 * scale)))
                                .children(move || {
                                    tabs()
                                        .variant(TabsVariant::Secondary)
                                        .selected(selected)
                                        .indicator_height(Dp(2.0 * scale))
                                        .min_tab_height(Dp(48.0 * scale))
                                        .tab_padding(Dp(16.0 * scale))
                                        .modifier(Modifier::new().fill_max_width())
                                        .content(move || {
                                            for index in 0..2 {
                                                tab()
                                                    .accessibility_label(
                                                        figure_name(false, index).to_string(),
                                                    )
                                                    .title_shared(RenderSlot::new(move || {
                                                        native_tab_title(
                                                            index,
                                                            inline_icons,
                                                            index == 0,
                                                            geometry,
                                                            scale,
                                                            false,
                                                            true,
                                                        );
                                                    }));
                                            }
                                        });
                                });
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            secondary_measure_annotations(inline_icons, geometry);
                        });
                });
            gap(Dp(16.0));
            text()
                .content(format!(
                    "Figure content: {}",
                    figure_name(false, selected.get())
                ))
                .style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn secondary_measure_annotations(inline_icons: bool, geometry: State<Geometry>) {
    let values = geometry.get();
    let (Some(root), Some(container), Some(label)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, LABEL_KEYS[0]),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let tint = "#ec407a";
    let mut svg = String::new();
    let bottom = container.y + container.h;
    let half = container.w * 0.5;
    annotations::outline(&mut svg, container);
    dotted(
        &mut svg,
        container.x + container.w + 12.0,
        container.y,
        container.h,
        true,
        tint,
    );
    annotations::dimension(
        "48".into(),
        Dp(container.x + container.w + 20.0),
        Dp(container.y + container.h * 0.5 - 10.0),
    );
    dotted(
        &mut svg,
        container.x + half,
        container.y - 12.0,
        container.h + 24.0,
        true,
        tint,
    );
    dotted(&mut svg, container.x, container.y - 12.0, half, false, tint);
    annotations::dimension(
        "1/2".into(),
        Dp(container.x + half * 0.5 - 12.0),
        Dp(container.y - 34.0),
    );
    let indicator = Rect {
        x: container.x,
        y: bottom - 2.0,
        w: half,
        h: 2.0,
    };
    annotations::outline(&mut svg, indicator);
    annotations::dimension("2".into(), Dp(container.x - 24.0), Dp(bottom - 12.0));
    dotted(&mut svg, container.x, bottom + 16.0, half, false, tint);
    annotations::dimension(
        "Full-width indicator".into(),
        Dp(container.x),
        Dp(bottom + 24.0),
    );
    if inline_icons {
        if let Some(icon) = relative(&values, ICON_KEYS[0]) {
            annotations::outline(&mut svg, icon);
            annotations::dimension("24".into(), Dp(icon.x - 8.0), Dp(icon.y - 26.0));
            let gap_x = icon.x + icon.w;
            svg.push_str(&format!(
                "<rect x='{gap_x}' y='{}' width='{}' height='{}' fill='{tint}' fill-opacity='.3'/>",
                icon.y,
                (label.x - gap_x).max(0.0),
                icon.h
            ));
            annotations::dimension(
                "8".into(),
                Dp(gap_x + (label.x - gap_x) * 0.5 - 4.0),
                Dp(icon.y + icon.h + 6.0),
            );
        }
    }
    annotations::outline(&mut svg, label);
    if !inline_icons {
        annotations::dimension("20".into(), Dp(label.x - 28.0), Dp(label.y));
    }
    if let Some(badge) = relative(&values, "badge") {
        let gap_x = label.x + label.w;
        svg.push_str(&format!(
            "<rect x='{gap_x}' y='{}' width='{}' height='{}' fill='{tint}' fill-opacity='.3'/>",
            label.y,
            (badge.x - gap_x).max(0.0),
            label.h
        ));
        annotations::dimension(
            "4".into(),
            Dp(gap_x + (badge.x - gap_x) * 0.5 - 4.0),
            Dp(label.y - 24.0),
        );
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn scrolling_tabs() {
    let bar = remember(AppBarScrollState::default);
    let content = remember(ScrollableController::new);
    let selected = remember(|| 0usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (page, divider) = (scheme.surface, scheme.outline_variant);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            flow_row().item_spacing(Dp(8.0)).children(move || {
                button()
                    .text()
                    .accessibility_label("Reset scrolling tabs")
                    .on_click(move || {
                        bar.with_mut(AppBarScrollState::reset);
                        content.with_mut(|content| {
                            content.set_scroll_position(tessera_ui::PxPosition::ZERO)
                        });
                    })
                    .child(|| {
                        text().content("Reset scroll");
                    });
            });
            gap(Dp(16.0));
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(360.0))
                        .background_with_shape(page, Shape::rounded_rectangle(Dp(24.0)))
                        .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
                )
                .children(move || {
                    app_bar_scroll_provider()
                        .state(bar)
                        .behavior(AppBarScrollBehavior::EnterAlways)
                        .scroll_controller(content)
                        .child(move || {
                            scaffold()
                                .modifier(Modifier::new().fill_max_size())
                                .top_bar(move || {
                                    let visible =
                                        1.0 - bar.with(AppBarScrollState::collapsed_fraction);
                                    column()
                                        .modifier(Modifier::new().fill_max_width().semantics(
                                            SemanticsArgs {
                                                role: Some(Role::Group),
                                                label: Some("Scrolling tabs header".into()),
                                                ..Default::default()
                                            },
                                        ))
                                        .children(move || {
                                            tessera_components::app_bar::top_app_bar()
                                                .title("Travel")
                                                .scroll_state(bar);
                                            boxed()
                                                .alignment(Alignment::BottomCenter)
                                                .modifier(
                                                    Modifier::new()
                                                        .fill_max_width()
                                                        .height(Dp(48.0 * visible as f64))
                                                        .clip_to_bounds(),
                                                )
                                                .children(move || {
                                                    tabs()
                                                        .selected(selected)
                                                        .modifier(Modifier::new().fill_max_width())
                                                        .content(|| {
                                                            tab().label("Flights");
                                                            tab().label("Trips");
                                                            tab().label("Explore");
                                                        });
                                                });
                                        });
                                })
                                .content(move || {
                                    column()
                                        .modifier(
                                            Modifier::new()
                                                .fill_max_size()
                                                .vertical_scroll(content)
                                                .semantics(SemanticsArgs {
                                                    role: Some(Role::Group),
                                                    label: Some("Scrolling tabs content".into()),
                                                    ..Default::default()
                                                }),
                                        )
                                        .children(move || {
                                            let destination =
                                                ["Flights", "Trips", "Explore"][selected.get()];
                                            for index in 1..=24 {
                                                text()
                                                    .content(format!("{destination} item {index}"))
                                                    .style(type_style(16.0, 24.0))
                                                    .modifier(Modifier::new().padding(
                                                        Padding::symmetric(Dp(24.0), Dp(12.0)),
                                                    ));
                                                boxed().modifier(
                                                    Modifier::new()
                                                        .fill_max_width()
                                                        .height(Dp(1.0))
                                                        .background(divider),
                                                );
                                            }
                                        });
                                });
                        });
                });
        });
}
