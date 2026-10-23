//! Article chapters for related and primary destinations.
//!
//! ## Usage
//!
//! Compare navigation, content hierarchy and accessible destination controls.

use tessera_components::{
    alignment::Alignment, boxed::boxed, modifier::ModifierExt as _, shape_def::Shape, text::text,
    theme::MaterialTheme,
};
use tessera_ui::{Dp, Modifier, RenderSlot, RenderSlotWith, tessera, use_context};

use super::super::{
    INK,
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum Kind {
    #[default]
    Bar,
    Rail,
    Tabs,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum Figure {
    #[default]
    None,
    Overview,
    Anatomy,
    Selection,
    Badges,
    Placement,
    Adaptive,
    Scaling,
    Keyboard,
    Labels,
    Fixed,
    Scrollable,
    Nested,
    States,
}

#[derive(Clone, Default, PartialEq)]
struct Chapter {
    title: &'static str,
    text: &'static str,
    figure: Figure,
    children: Vec<Chapter>,
}

fn chapter(
    title: &'static str,
    text: &'static str,
    figure: Figure,
    children: Vec<Chapter>,
) -> Chapter {
    Chapter {
        title,
        text,
        figure,
        children,
    }
}

fn leaf(title: &'static str, text: &'static str, figure: Figure) -> Chapter {
    chapter(title, text, figure, vec![])
}

fn guidelines(kind: Kind) -> Vec<Chapter> {
    use Figure as F;
    match kind {
        Kind::Bar => vec![
            leaf(
                "Usage",
                "Provide three to five primary destinations of equal importance. Keep their positions fixed, retain visible labels, and reserve tabs for related content within a page.",
                F::Overview,
            ),
            chapter(
                "Anatomy",
                "Each destination combines an icon, label and active indicator inside a shared container.",
                F::Anatomy,
                vec![
                    leaf(
                        "Container",
                        "Anchor the container to the bottom edge and span the window. Its fill separates navigation from content.",
                        F::None,
                    ),
                    leaf(
                        "Navigation items",
                        "Use vertical items in compact windows and horizontal items in medium windows. Center horizontal items with consistent outer margins.",
                        F::Adaptive,
                    ),
                    leaf(
                        "Icons",
                        "Use a filled icon for the active destination and an outlined icon for inactive destinations. Keep recognizable shapes and sufficient contrast.",
                        F::Selection,
                    ),
                    leaf(
                        "Active indicator",
                        "Show exactly one active indicator for the page currently displayed.",
                        F::None,
                    ),
                    leaf(
                        "Label text",
                        "Use concise, meaningful labels of one or two words. Avoid shrinking type or truncating ordinary destination labels.",
                        F::Labels,
                    ),
                    leaf(
                        "Badges (optional)",
                        "Place badges at the upper trailing corner of the icon. A small badge signals an update; a large badge can express its count.",
                        F::Badges,
                    ),
                ],
            ),
            leaf(
                "Placement",
                "Keep navigation along the bottom of the window. Place a FAB above it at the trailing edge, keeping destinations unobstructed.",
                F::Placement,
            ),
            chapter(
                "Adaptive design",
                "Choose navigation according to the available window space and the number of destinations.",
                F::None,
                vec![
                    leaf(
                        "Resizing",
                        "Use a bar in compact and medium windows. At larger sizes, use a navigation rail instead of stretching a bar across a desktop layout.",
                        F::Adaptive,
                    ),
                    leaf(
                        "Presentation",
                        "In medium windows, keep horizontal destinations centered with consistent spacing.",
                        F::None,
                    ),
                ],
            ),
            chapter(
                "Behavior",
                "Selecting a destination displays its top-level page. Retain each page's state when frequent switching is useful.",
                F::None,
                vec![
                    leaf(
                        "Navigation",
                        "Choose whether returning to a destination restores its state or resets it. Reselecting the active destination returns its content to the top; swiping does not switch top-level pages.",
                        F::Selection,
                    ),
                    leaf(
                        "Scrolling",
                        "The bar may hide on downward scroll and reappear on upward scroll. Keep it visible when a screen reader is active.",
                        F::Placement,
                    ),
                    leaf(
                        "Selection",
                        "Fill the selected icon and expand its indicator along one axis. Keep focus distinct from activation.",
                        F::States,
                    ),
                ],
            ),
        ],
        Kind::Rail => vec![
            chapter(
                "Usage",
                "Use three to seven primary destinations along the leading edge of medium and larger windows.",
                F::Overview,
                vec![
                    leaf(
                        "Collapsed",
                        "Keep the collapsed rail visible. In compact windows use a navigation bar instead.",
                        F::Fixed,
                    ),
                    leaf(
                        "Expanded",
                        "Open an expanded rail from a menu button. A standard rail sits beside content; a modal rail overlaps it and can reveal additional destinations.",
                        F::Adaptive,
                    ),
                ],
            ),
            chapter(
                "Anatomy",
                "A rail may combine destinations with an optional menu, FAB and notification badges.",
                F::Anatomy,
                vec![
                    leaf(
                        "Container",
                        "Place the vertical container on the leading window edge. Keep the menu and FAB at the top, with destinations grouped below.",
                        F::Placement,
                    ),
                    leaf(
                        "Menu (optional)",
                        "Use the menu to expand and collapse the rail. Change its icon to communicate the available reverse action.",
                        F::Selection,
                    ),
                    leaf(
                        "Floating action button (FAB) (optional)",
                        "Place the primary action above destinations. A nested FAB rests without elevation; an expanded rail can show its text label.",
                        F::None,
                    ),
                    leaf(
                        "Active indicator",
                        "The expanded indicator hugs its contents while the interactive target spans the rail width.",
                        F::None,
                    ),
                    leaf(
                        "Icons",
                        "Represent the destination with a familiar icon. Fill the active icon and outline inactive icons.",
                        F::None,
                    ),
                    leaf(
                        "Label text",
                        "Prefer a single meaningful word. Preserve the text size and avoid ellipsis; break longer phrases between words when necessary.",
                        F::Labels,
                    ),
                    leaf(
                        "Badges",
                        "In collapsed rails, badges overlap the icon's upper trailing corner. In expanded rails, put the badge after the label.",
                        F::Badges,
                    ),
                    leaf(
                        "Divider (optional)",
                        "Place a vertical divider on the content-facing edge when additional separation is needed.",
                        F::None,
                    ),
                ],
            ),
            leaf(
                "Placement",
                "Place the rail outside content panes along the leading edge. Tabs can provide a second navigation level within a content pane.",
                F::Placement,
            ),
            chapter(
                "Adaptive design",
                "Choose a bar or rail from the available space without showing both navigation forms simultaneously.",
                F::None,
                vec![
                    leaf(
                        "Resizing",
                        "Use a bar in compact windows and a rail from medium widths upward. Consider a modal expanded rail when many destinations must fit in limited space.",
                        F::Adaptive,
                    ),
                    leaf(
                        "Presentation",
                        "When a standard rail expands, adjust the adjacent content width. Its FAB can expand to reveal a label, and secondary destinations can appear.",
                        F::Selection,
                    ),
                ],
            ),
            chapter(
                "Behavior",
                "Keep navigation independent of scrolling page content.",
                F::None,
                vec![
                    leaf(
                        "Scrolling",
                        "Keep rail destinations fixed during vertical page scrolling. A divider or elevation can separate horizontally scrolling content.",
                        F::Placement,
                    ),
                    leaf(
                        "Selection",
                        "Activate one destination, fill its icon and expand its indicator. Keep other destinations available.",
                        F::States,
                    ),
                    leaf(
                        "Back",
                        "Dismiss a modal expanded rail to return to the page. Platform predictive-back behavior applies only to the modal presentation.",
                        F::Adaptive,
                    ),
                ],
            ),
        ],
        Kind::Tabs => vec![
            chapter(
                "Usage",
                "Group related content at the same level of hierarchy, with short labels and consistent icon treatment.",
                F::Overview,
                vec![leaf(
                    "Related content",
                    "Use tabs to categorize related information. Use typography and spacing to organize a sequence that must be read in order.",
                    F::Nested,
                )],
            ),
            chapter(
                "Anatomy",
                "A shared container holds labels, optional icons and badges, the divider and one active indicator.",
                F::Anatomy,
                vec![
                    leaf(
                        "Container",
                        "Extend the container across its content region. Its bottom divider separates the tab strip from the page below.",
                        F::None,
                    ),
                    leaf(
                        "Icon (optional)",
                        "Use recognizable icons consistently across the set, with labels when the meaning needs explanation.",
                        F::None,
                    ),
                    leaf(
                        "Label",
                        "Keep labels concise. A scrollable set can accommodate longer titles; use consistent padding and the 52dp leading offset.",
                        F::Scrollable,
                    ),
                    leaf(
                        "Badges (optional)",
                        "Use a small or large badge to identify updates. Limit counts to four characters including a plus sign, and clear the badge once its content has been viewed.",
                        F::Badges,
                    ),
                    leaf(
                        "Active indicator",
                        "Distinguish the selected tab with an underline and changes to its label and icon color.",
                        F::Selection,
                    ),
                ],
            ),
            leaf(
                "Choosing the tab variant",
                "Use primary tabs for a single content hierarchy. Add secondary tabs only when another level is needed within that content.",
                F::Nested,
            ),
            leaf(
                "Placement",
                "Keep a set in one row connected to the content it controls. Place secondary tabs below the primary set.",
                F::Placement,
            ),
            leaf(
                "Responsive layout",
                "Use fluid margins and size fixed tabs from the widest label. Prefer no more than four visible fixed tabs; use scrolling when more room is needed.",
                F::Adaptive,
            ),
            chapter(
                "Behavior",
                "Tab selection updates the content region below the set.",
                F::None,
                vec![
                    leaf(
                        "States",
                        "Active and inactive tabs can each be hovered, focused or pressed. Focus alone need not change the selected page.",
                        F::States,
                    ),
                    leaf(
                        "Fixed tabs",
                        "Keep all destinations visible when they fit. Activate one directly; related content can support horizontal swiping without conflicting with other gestures.",
                        F::Fixed,
                    ),
                    leaf(
                        "Scrollable tabs",
                        "Let the strip scroll horizontally when labels do not fit, preserving equal label padding and access to offscreen destinations.",
                        F::Scrollable,
                    ),
                    leaf(
                        "Scrolling content",
                        "Keep attached tabs and their app bar moving together. A strip may stay at the top or leave and return with scrolling content.",
                        F::Placement,
                    ),
                ],
            ),
        ],
    }
}

fn accessibility(kind: Kind) -> Vec<Chapter> {
    use Figure as F;
    let interaction = if kind == Kind::Rail {
        chapter(
            "Interaction & style",
            "Keep a full-width target even when the visual indicator hugs its contents. Use distinguishable icon shapes, state feedback and sufficient contrast.",
            F::None,
            vec![
                leaf(
                    "Text scaling and truncation",
                    "Let items grow with labels and preserve the default padding. Keep the full text visible through twice the normal text size.",
                    F::Scaling,
                ),
                leaf(
                    "Initial focus",
                    "Start at the first interactive item: menu, FAB or first destination. Tab and arrow keys continue through destinations.",
                    F::Keyboard,
                ),
                leaf(
                    "Visual indicators",
                    "Use a filled active icon against outlined inactive icons. Preserve an additional visual cue beyond color.",
                    F::Selection,
                ),
            ],
        )
    } else if kind == Kind::Tabs {
        chapter(
            "Interaction & style",
            "Allow keyboard and assistive-technology users to reach every tab, including offscreen labels. Keep the strip finite instead of looping indefinitely.",
            F::Scrollable,
            vec![leaf(
                "Avoid applying density by default",
                "Preserve targets of at least 48 × 48dp. If a denser mode is offered, keep the control that reverses it comfortably reachable.",
                F::None,
            )],
        )
    } else {
        chapter(
            "Interaction & style",
            "Provide press and hover feedback, and expose the selected destination independently from keyboard focus.",
            F::States,
            vec![leaf(
                "Text scaling and truncation",
                "Grow the bar vertically to accommodate larger labels while retaining its padding. Keep the complete label visible through twice the normal text size.",
                F::Scaling,
            )],
        )
    };
    let mut chapters = vec![
        leaf(
            "Use cases",
            "People must be able to discover destinations, move between them, activate a choice and receive feedback using their preferred input or assistive technology.",
            F::Overview,
        ),
        interaction,
    ];
    if kind != Kind::Rail {
        chapters.push(leaf("Initial focus", if kind == Kind::Tabs { "Enter the tab set through its active control. Use arrows to move focus and Space or Enter to activate in manual mode." } else { "Initial focus reaches the first navigation item. Use Space or Enter to activate the focused destination." }, F::Keyboard));
    }
    if kind == Kind::Bar {
        chapters.push(leaf("Visual indicators", "Fill the active destination's icon and retain its indicator. Use outlined icons for other destinations, or a heavier weight when no filled icon exists.", F::Selection));
    }
    chapters.extend([
        leaf("Keyboard navigation", "Keep navigation order logical and show a visible focus state.", F::Keyboard),
        leaf("Labeling elements", "Use the visible destination name in its accessible name. Expand ambiguous wording and announce relevant notification counts without changing the visible label.", F::Labels),
    ]);
    chapters
}

#[tessera]
pub(crate) fn detailed_article(
    kind: Kind,
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    sections: Vec<SpecSection>,
    figures: RenderSlotWith<Figure>,
) {
    let mut pages = Vec::new();
    let overview = if kind == Kind::Tabs {
        vec![leaf(
            "Differences from M2",
            "Primary and secondary variants establish content hierarchy, with updated color roles and active indicator treatments.",
            Figure::Nested,
        )]
    } else {
        vec![
            leaf(
                "M3 Expressive update",
                if kind == Kind::Rail {
                    "Collapsed and expanded rails replace the baseline rail and drawer. Expanded labels and actions provide more room while retaining the same destinations."
                } else {
                    "The flexible bar is shorter and adapts between top and start icon placement. It replaces the baseline navigation bar."
                },
                Figure::Adaptive,
            ),
            leaf(
                "Differences from M2",
                "Updated colors, indicator geometry and sizing strengthen selection feedback while keeping navigation destinations stable.",
                Figure::Selection,
            ),
        ]
    };
    for (page, chapters) in [
        ("Overview", overview),
        ("Guidelines", guidelines(kind)),
        ("Accessibility", accessibility(kind)),
    ] {
        let mut sections = Vec::new();
        for chapter in chapters {
            let title = chapter.title;
            sections.push(SpecSection {
                title,
                content: RenderSlot::new(move || {
                    paragraph(chapter.text);
                    if chapter.figure != Figure::None {
                        gap(Dp(24.0));
                        figures.render(chapter.figure);
                    }
                    for child in &chapter.children {
                        gap(Dp(40.0));
                        subheading(child.title.into());
                        gap(Dp(16.0));
                        paragraph(child.text);
                        if child.figure != Figure::None {
                            gap(Dp(24.0));
                            figures.render(child.figure);
                        }
                    }
                    if page == "Accessibility" && title == "Keyboard navigation" {
                        gap(Dp(24.0));
                        keyboard(kind);
                    }
                }),
            });
        }
        pages.push(ArticlePage { title:page, intro: if page == "Overview" { Some(RenderSlot::new(move || {
            paragraph(if kind == Kind::Tabs { "Use primary and secondary tabs to organize related content. Keep labels short and choose fixed or scrollable sets to fit the available space." } else if kind == Kind::Rail { "Use a navigation rail for three to seven primary destinations in medium and larger windows. Collapse or expand it from a menu to fit the available space." } else { "Use a navigation bar for three to five primary destinations in compact and medium windows. Keep icons, labels and one selected destination visible." });
            gap(Dp(32.0)); figures.render(Figure::Overview);
        })) } else {None}, sections });
    }
    article(title, description, sections).pages(pages);
}

#[tessera]
fn paragraph(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK)
        .modifier(Modifier::new().fill_max_width());
}

#[tessera]
fn keyboard(kind: Kind) {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec![
                if kind == Kind::Tabs {
                    "Arrow keys"
                } else if kind == Kind::Rail {
                    "Tab / Arrow keys"
                } else {
                    "Tab"
                }
                .into(),
                "Move focus between available destinations".into(),
            ],
            vec![
                "Space / Enter".into(),
                "Activate the focused destination".into(),
            ],
        ],
    );
}

#[tessera]
pub(crate) fn figure_marker(number: usize, x: Dp, y: Dp, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0 * scale), Dp(32.0 * scale))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(16.0 * scale)))
                .border_with_shape(
                    Dp(scale),
                    scheme.on_surface,
                    Shape::rounded_rectangle(Dp(16.0 * scale)),
                ),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(16.0 * scale, 20.0 * scale))
                .color(scheme.on_surface);
        });
}
