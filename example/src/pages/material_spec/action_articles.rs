//! Button family overview, guidance and accessibility pages.
//!
//! ## Usage
//!
//! Explore action hierarchy, selection, menu behavior and keyboard access.

use tessera_components::{
    button::button,
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{Dp, Modifier, RenderSlot, accesskit::Role, tessera, use_context};

use super::{
    INK,
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) enum Kind {
    #[default]
    Buttons,
    Groups,
    Icons,
    Fab,
    ExtendedFab,
    FabMenu,
    Segmented,
    Split,
}

#[derive(Clone, Copy, Default, PartialEq)]
enum Figure {
    #[default]
    Example,
    Anatomy,
    Behavior,
    Keyboard,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Chapter {
    title: &'static str,
    text: &'static str,
    subheads: &'static [&'static str],
    figure: Figure,
}

fn chapter(
    title: &'static str,
    text: &'static str,
    subheads: &'static [&'static str],
    figure: Figure,
) -> Chapter {
    Chapter {
        title,
        text,
        subheads,
        figure,
    }
}

fn title(kind: Kind) -> &'static str {
    match kind {
        Kind::Buttons => "Buttons",
        Kind::Groups => "Button groups",
        Kind::Icons => "Icon buttons",
        Kind::Fab => "Floating action buttons",
        Kind::ExtendedFab => "Extended FABs",
        Kind::FabMenu => "FAB menu",
        Kind::Segmented => "Segmented buttons",
        Kind::Split => "Split buttons",
    }
}

fn summary(kind: Kind) -> &'static str {
    match kind {
        Kind::Buttons => {
            "Represent actions with default or toggle buttons. Choose among five sizes, two shapes and five color styles. Keep the label short and clear."
        }
        Kind::Groups => {
            "Group related buttons using standard or connected layouts. Coordinated shapes communicate pressing and selection. Groups support single, multiple and required selection."
        }
        Kind::Icons => {
            "Use recognizable icons for common actions. Default buttons perform actions; toggle buttons communicate selection through icon and shape changes."
        }
        Kind::Fab => {
            "Keep one constructive primary action prominent above the content. Choose among FAB, medium and large sizes; use an extended FAB when an icon needs a label."
        }
        Kind::ExtendedFab => {
            "Keep a primary action visible with a concise text label and optional icon. Small, medium and large sizes provide different emphasis."
        }
        Kind::FabMenu => {
            "Open two to six related actions from a FAB. Match the menu color set to its trigger and retain clear labels and icons."
        }
        Kind::Segmented => {
            "Compare single and multiple selection among two to five related options. Connected button groups are recommended for current Expressive interfaces."
        }
        Kind::Split => {
            "Pair a primary action with a separate menu button for related alternatives. Both targets remain independently operable across five size options."
        }
    }
}

fn guidelines(kind: Kind) -> Vec<Chapter> {
    use Figure::{Anatomy, Behavior, Example};
    match kind {
        Kind::Buttons => vec![
            chapter(
                "Usage",
                "Make important actions prominent without crowding the screen. Let width accommodate the label.",
                &[],
                Example,
            ),
            chapter(
                "Toggle buttons",
                "Use toggles for binary choices. Communicate selection through shape, color and an appropriate icon; keep changing labels similar in length.",
                &[],
                Behavior,
            ),
            chapter(
                "Anatomy",
                "Use concise sentence-case labels, sufficient container width and at most one leading icon.",
                &["Label text", "Container", "Icon (optional)"],
                Anatomy,
            ),
            chapter(
                "Color styles",
                "Use filled for primary actions, tonal or outlined for secondary actions, and text for lower emphasis. Reserve elevation for needed separation.",
                &[
                    "Elevated style",
                    "Filled style",
                    "Tonal style",
                    "Outlined style",
                    "Text style",
                ],
                Example,
            ),
            chapter(
                "Adaptive design",
                "Preserve action hierarchy as size and arrangement change. Keep the icon and text grouped and centered.",
                &["Resizing", "Presentation"],
                Example,
            ),
        ],
        Kind::Groups => vec![
            chapter(
                "Usage",
                "Use consistent sizes and shapes in a group. Connected groups support related toggle choices; keep their color styles consistent.",
                &["Color"],
                Example,
            ),
            chapter(
                "Anatomy",
                "Standard groups accommodate motion around their contents. Connected groups expand within their container.",
                &["Container"],
                Anatomy,
            ),
            chapter(
                "Adaptive design",
                "Keep each group on one line. Adjust size and width while preserving emphasis; move secondary trailing actions into overflow when necessary.",
                &["Resizing", "Presentation"],
                Example,
            ),
            chapter(
                "Behavior",
                "Pressing changes shape. Standard groups also adjust neighboring widths; selection changes round and square shapes.",
                &["Pressed", "Selected"],
                Behavior,
            ),
        ],
        Kind::Icons => vec![
            chapter(
                "Usage",
                "Use default icon buttons for actions and toggle icon buttons for binary choices.",
                &[],
                Example,
            ),
            chapter(
                "Anatomy",
                "Use recognizable icons and distinguish selection through filled and outlined symbols.",
                &[],
                Anatomy,
            ),
            chapter(
                "Color",
                "Choose filled, tonal, outlined or standard emphasis for the surrounding content.",
                &[],
                Example,
            ),
            chapter(
                "Size",
                "Choose among five sizes according to action hierarchy, not the available gap.",
                &[],
                Example,
            ),
            chapter(
                "Width",
                "Use narrow, default and wide proportions while preserving minimum targets.",
                &[],
                Example,
            ),
            chapter(
                "Shape",
                "Use round and square resting shapes with clear pressed and selected transformations.",
                &[],
                Behavior,
            ),
            chapter(
                "Placement",
                "Group meaningful actions in app bars, cards and toolbars.",
                &[],
                Example,
            ),
            chapter(
                "Behavior",
                "Default actions perform a command; toggles retain selection and expose a useful hover label.",
                &[],
                Behavior,
            ),
        ],
        Kind::Fab => vec![
            chapter(
                "Usage",
                "Promote one primary action. Medium suits many layouts; larger sizes provide more emphasis. The old small FAB is no longer recommended.",
                &[],
                Example,
            ),
            chapter(
                "Actions",
                "Prefer constructive actions such as creating, sharing or starting a process. Keep minor or destructive commands elsewhere.",
                &[],
                Behavior,
            ),
            chapter(
                "Anatomy",
                "Use a clear filled icon and a container that contrasts with its background.",
                &["Container", "Icon"],
                Anatomy,
            ),
            chapter(
                "Adaptive design",
                "Use reachable placement in compact windows and consider a navigation rail in expanded layouts.",
                &[],
                Example,
            ),
            chapter(
                "Behaviors",
                "Keep the FAB above scrolling content. Transitions may expand it or replace it as the surrounding page changes.",
                &["Appearing", "Expanding", "Scrolling", "Moving across tabs"],
                Behavior,
            ),
        ],
        Kind::ExtendedFab => vec![
            chapter(
                "Usage",
                "Use one extended FAB when a primary action needs additional clarity. Choose small, medium or large according to emphasis.",
                &["Additional emphasis", "Choosing a size"],
                Example,
            ),
            chapter(
                "Anatomy",
                "Let the rounded container hug its contents. Keep the label concise and visible; an icon is optional.",
                &["Container", "Icon (optional)", "Label text"],
                Anatomy,
            ),
            chapter(
                "Placement",
                "Keep the action above the page content. Avoid covering other actions or placing it inside cards.",
                &[],
                Example,
            ),
            chapter(
                "Responsive layout",
                "Adapt between FAB and extended FAB as space changes. Mirror icon and label ordering in right-to-left layouts.",
                &["Right-to-left languages", "Breakpoints"],
                Example,
            ),
            chapter(
                "Behavior",
                "Expansion reveals the label; collapsing reduces occupied space. Scrolling can drive the transition.",
                &["Appearing", "Expanding", "Transforming", "Scrolling"],
                Behavior,
            ),
        ],
        Kind::FabMenu => vec![
            chapter(
                "Usage",
                "Open two to six closely related actions from a FAB at the same anchor. Match the primary, secondary or tertiary color set.",
                &["Color sets"],
                Example,
            ),
            chapter(
                "Anatomy",
                "Retain readable labels and meaningful icons. Rounded menu items hug their content; the close button stays available.",
                &[],
                Anatomy,
            ),
            chapter(
                "Adaptive layout",
                "Keep the same corner anchor. Increase margins for large windows and let short layouts scroll behind the close button.",
                &[],
                Example,
            ),
            chapter(
                "Behavior",
                "The FAB becomes the close button. Menu items enter from the anchored corner and may expand into their destination.",
                &["Appearing", "Scrolling", "Expanding"],
                Behavior,
            ),
        ],
        Kind::Segmented => vec![
            chapter(
                "Usage",
                "Use a small set of related choices to switch views or filters.",
                &[],
                Example,
            ),
            chapter(
                "Anatomy",
                "Keep two to five segments on one line with consistent label types and short text.",
                &["Segments", "Container", "Icons", "Label text"],
                Anatomy,
            ),
            chapter(
                "Single-select",
                "Select one option from the set.",
                &[],
                Example,
            ),
            chapter(
                "Multi-select",
                "Allow independent choices, including selecting all or clearing the entire set.",
                &[],
                Behavior,
            ),
            chapter(
                "Placement",
                "Retain margins and avoid excessive stretching in larger windows.",
                &[],
                Example,
            ),
            chapter(
                "Behavior",
                "A selected icon can become a checkmark. Selection remains distinguishable beyond color.",
                &[],
                Behavior,
            ),
        ],
        Kind::Split => vec![
            chapter(
                "Usage",
                "Use the leading action directly and conceal related alternatives in the trailing menu. Match size to neighboring actions and emphasis.",
                &[],
                Example,
            ),
            chapter(
                "Anatomy",
                "Keep the leading label brief and retain the trailing expand icon. Mirror the order for right-to-left layouts.",
                &[],
                Anatomy,
            ),
            chapter(
                "Behavior",
                "The trailing button rotates inward by 180 degrees and changes shape. Align the menu to a button edge with a 4dp gap.",
                &["Menu placement"],
                Behavior,
            ),
        ],
    }
}

fn accessibility(kind: Kind) -> Vec<Chapter> {
    use Figure::{Anatomy, Behavior, Example, Keyboard};
    if kind == Kind::Segmented {
        return vec![
            chapter(
                "Use cases",
                "People need to understand the choices, navigate between segments and change selection.",
                &[],
                Example,
            ),
            chapter(
                "Interaction & style",
                "Use Tab to enter the group, arrows to move between choices, and activation keys to select.",
                &[],
                Behavior,
            ),
            chapter(
                "Color contrast",
                "Keep outlines distinguishable from the surface and communicate selection with a non-color cue.",
                &[],
                Anatomy,
            ),
            chapter(
                "Initial focus",
                "Start at the first segment in reading order.",
                &[],
                Example,
            ),
            chapter(
                "Keyboard navigation",
                "Try traversal and activation in the live example.",
                &[],
                Keyboard,
            ),
            chapter(
                "Labeling elements",
                "Name each choice from its visible label or the meaning of its icon.",
                &[],
                Example,
            ),
        ];
    }
    let mut pages = vec![
        chapter(
            "Use cases",
            "People need to reach the action, understand its purpose, activate it and identify any selected or expanded state.",
            &[],
            Example,
        ),
        chapter(
            "Interaction & style",
            match kind {
                Kind::Buttons => {
                    "Maintain readable contrast and concise labels at enlarged text sizes. Motion should remain clear during repeated activation."
                }
                Kind::Groups => {
                    "Retain at least 48dp targets, including the smaller sizes. The group container is not a keyboard stop."
                }
                Kind::Icons => {
                    "Keep the icon distinguishable from its surface with sufficient contrast."
                }
                Kind::Fab => {
                    "Keep the icon legible against its container. Omit an unavailable FAB rather than displaying it disabled."
                }
                Kind::ExtendedFab => {
                    "Place the primary action where it is easy to reach without obscuring other targets."
                }
                Kind::FabMenu => {
                    "Retain 48dp targets and keep the close button visible while menu items scroll."
                }
                Kind::Segmented => {
                    "Use keyboard traversal and a checkmark or other non-color cue for selection."
                }
                Kind::Split => {
                    "Both parts need at least 48dp targets, including extra-small and small presentations."
                }
            },
            match kind {
                Kind::Buttons => &["Color contrast", "200% text size", "Rapid clicks"],
                Kind::Groups => &["Initial focus"],
                _ => &[],
            },
            Anatomy,
        ),
    ];
    match kind {
        Kind::Fab=>{
            pages.push(chapter("Focus","Give the primary action an efficient position in the focus order and expose its purpose when focused.",&[],Behavior));
            pages.push(chapter("Layout & position","Keep focus indicators and underlying interactive elements visible. Adapt position to the available window.",&[],Example));
        },
        Kind::ExtendedFab|Kind::FabMenu|Kind::Split=>pages.push(chapter("Initial focus",match kind {
            Kind::FabMenu=>"Opening retains focus on the close button. Continue through the menu items from top to bottom.",
            Kind::Split=>"Focus the leading action before the trailing menu button.",
            _=>"Treat the icon and visible label as one target and prioritize the primary action in the page order.",
        },&[],Behavior)),
        _=>{},
    }
    pages.push(chapter(
        "Keyboard navigation",
        "Try keyboard traversal and activation in the live example.",
        &[],
        Keyboard,
    ));
    pages.push(chapter("Labeling elements",match kind {
        Kind::Icons|Kind::Fab=>"Name the action rather than the appearance of the icon.",
        Kind::FabMenu=>"Name each menu action and communicate whether its trigger is expanded or collapsed.",
        Kind::Split=>"Use the visible leading label and a distinct trailing label that describes the additional choices and expanded state.",
        _=>"Use names consistent with visible labels and provide useful context without repeating the control role.",
    },&[],Example));
    if kind == Kind::Icons {
        pages.push(chapter("Layout & density","Retain default 48dp targets; denser presentation should be an explicit, reversible preference.",&["Avoid applying density by default"],Example));
        pages.push(chapter(
            "Hover",
            "Show a concise tooltip that describes the action.",
            &[],
            Behavior,
        ));
    }
    pages
}

fn detail(kind: Kind, title: &str) -> &'static str {
    match title {
        "Color" | "Color sets" => {
            if kind == Kind::Groups {
                "Keep connected choices in one consistent color style."
            } else {
                "Choose emphasis and color appropriate to the surrounding actions."
            }
        }
        "Label text" => {
            if kind == Kind::ExtendedFab {
                "Prefer one or two words without wrapping or truncation."
            } else {
                "Use short, clear labels in sentence case."
            }
        }
        "Container" => {
            if kind == Kind::Groups {
                "Standard groups hug contents; connected groups expand within available space."
            } else {
                "Allow enough room for the complete icon and label."
            }
        }
        "Icon" | "Icons" | "Icon (optional)" => {
            "Use a recognizable symbol that communicates the action or selection."
        }
        "Size & width" | "Choosing a size" => {
            "Choose size and width according to the action's visual priority."
        }
        "Elevated style" => {
            "Use a shadow when separation from a prominent background is necessary."
        }
        "Filled style" => "Reserve strong visual emphasis for important primary actions.",
        "Tonal style" => "Use tonal fills for secondary actions needing moderate emphasis.",
        "Outlined style" => "Use on simple backgrounds for supporting actions.",
        "Text style" => "Use for low-emphasis actions, including dialog and snackbar actions.",
        "Resizing" | "Breakpoints" => {
            "Adapt dimensions while preserving the hierarchy of primary and secondary actions."
        }
        "Presentation" => "Rearrange secondary actions when the available space changes.",
        "Pressed" => "Pressing changes shape; standard groups also adjust neighboring widths.",
        "Selected" | "Selection" => {
            "Communicate selection using shape and icon changes as well as color."
        }
        "Additional emphasis" => "Use a label when an icon alone would be ambiguous.",
        "Right-to-left languages" => {
            "Mirror the icon and label ordering for right-to-left reading."
        }
        "Appearing" => "Animate the action from its anchor without moving unrelated content.",
        "Expanding" => "A container transform can connect the action to its destination.",
        "Transforming" => "The extended action can reduce to an icon-only FAB.",
        "Scrolling" => match kind {
            Kind::FabMenu => "Keep the close button visible while items scroll behind it.",
            Kind::ExtendedFab => {
                "Collapse the label on downward scrolling and restore it on return."
            }
            _ => "Keep the floating action above the scrolling content.",
        },
        "Moving across tabs" => "Hide and reintroduce the FAB as the new page settles.",
        "Menu placement" => "Align to a button edge and retain a 4dp gap.",
        "Segments" => "Keep two to five related choices on one line.",
        "Hover" => "Describe the action in a concise tooltip.",
        "Color contrast" => "Keep enabled actions visually distinguishable from their background.",
        "200% text size" => "Choose concise labels that remain readable when text is enlarged.",
        "Rapid clicks" => "Motion should settle clearly during repeated presses.",
        "Initial focus" => "Focus enters the buttons rather than stopping on their container.",
        "Avoid applying density by default" => {
            "Keep 48dp default targets and make denser layouts an explicit preference."
        }
        _ => unreachable!("unregistered action article subheading"),
    }
}

#[tessera]
pub(super) fn action_article(
    kind: Kind,
    sections: Vec<SpecSection>,
    example: RenderSlot,
    anatomy: RenderSlot,
    behavior: RenderSlot,
    overview: Option<RenderSlot>,
) {
    let overview = overview.unwrap_or(example);
    let title = title(kind);
    let description = super::media::page_media(title)
        .expect("action article media")
        .description;
    let mut guide = Vec::new();
    for section in guidelines(kind) {
        guide.push(SpecSection {
            title: section.title,
            content: RenderSlot::new(move || {
                chapter_content(kind, section, example, anatomy, behavior);
            }),
        });
    }
    let mut access = Vec::new();
    if kind == Kind::Segmented {
        access.push(SpecSection {
            title: "Use cases",
            content: RenderSlot::new(move || {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        for (index, section) in accessibility(kind).into_iter().enumerate() {
                            segmented_access_chapter(
                                section,
                                index > 0,
                                example,
                                anatomy,
                                behavior,
                            );
                        }
                    });
            }),
        });
    } else {
        for section in accessibility(kind) {
            access.push(SpecSection {
                title: section.title,
                content: RenderSlot::new(move || {
                    chapter_content(kind, section, example, anatomy, behavior);
                }),
            });
        }
    }
    let mut article = article(title, description, sections).pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(move || {
                recommendation(kind);
                paragraph(summary(kind));
                gap(Dp(32.0));
                overview.render();
            })),
            sections: vec![],
        },
        ArticlePage {
            title: "Guidelines",
            intro: if kind == Kind::Segmented {
                Some(RenderSlot::new(move || {
                    recommendation(kind);
                }))
            } else {
                None
            },
            sections: guide,
        },
        ArticlePage {
            title: "Accessibility",
            intro: if kind == Kind::Segmented {
                Some(RenderSlot::new(move || {
                    recommendation(kind);
                }))
            } else {
                None
            },
            sections: access,
        },
    ]);
    if kind == Kind::Segmented {
        article = article.intro_shared(RenderSlot::new(move || {
            recommendation(kind);
        }));
    }
    drop(article);
}

#[tessera]
fn segmented_access_chapter(
    chapter: Chapter,
    show_heading: bool,
    example: RenderSlot,
    anatomy: RenderSlot,
    behavior: RenderSlot,
) {
    if show_heading {
        gap(Dp(64.0));
        subheading(chapter.title.into());
    }
    chapter_content(Kind::Segmented, chapter, example, anatomy, behavior);
}

#[tessera]
fn paragraph(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn chapter_content(
    kind: Kind,
    chapter: Chapter,
    example: RenderSlot,
    anatomy: RenderSlot,
    behavior: RenderSlot,
) {
    if kind == Kind::Icons {
        match chapter.title {
            title @ ("Usage" | "Anatomy" | "Color" | "Size" | "Width" | "Shape" | "Placement"
            | "Behavior") => {
                super::icon_buttons::articles::guidance(title.into());
                return;
            }
            "Interaction & style" => {
                super::icon_buttons::articles::contrast();
                return;
            }
            "Labeling elements" => {
                super::icon_buttons::context::labeling();
                return;
            }
            "Layout & density" => {
                super::icon_buttons::context::density();
                return;
            }
            _ => {}
        }
    }
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            paragraph(chapter.text);
            for title in chapter.subheads {
                gap(Dp(24.0));
                subheading((*title).into());
                paragraph(detail(kind, title));
            }
            gap(Dp(32.0));
            match chapter.figure {
                Figure::Example => example.render(),
                Figure::Anatomy => anatomy.render(),
                Figure::Behavior => behavior.render(),
                Figure::Keyboard => {
                    let mut rows = vec![
                        vec![
                            "Tab / Shift + Tab".into(),
                            "Move between available actions".into(),
                        ],
                        vec!["Space / Enter".into(), "Activate the focused action".into()],
                    ];
                    if matches!(kind, Kind::Groups | Kind::Segmented) {
                        rows.push(vec![
                            "Arrow keys".into(),
                            "Move between items in the group".into(),
                        ]);
                    }
                    spec_table(vec!["Keys".into(), "Action".into()], rows);
                    gap(Dp(32.0));
                    behavior.render();
                }
            }
        });
}

#[tessera]
fn recommendation(kind: Kind) {
    if kind != Kind::Segmented {
        return;
    }
    let navigation = use_context::<crate::app::MaterialNavigation>().map(|context| context.get().0);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().modifier(Modifier::new().fill_max_width().background_with_shape(scheme.secondary_container,Shape::rounded_rectangle(Dp(16.0))).padding_all(Dp(24.0)).semantics(SemanticsArgs {role:Some(Role::Note),label:Some("M3 Expressive recommendation".into()),merge_descendants:true,..Default::default()})).children(move || {
        paragraph("Segmented buttons are no longer recommended for M3 Expressive. Use a connected button group for new interfaces.");
        if let Some(navigation)=navigation {
            gap(Dp(16.0)); button().text().accessibility_label("Open recommended button groups").on_click(move || {
                if let Some(index)=super::catalog::component_index(18) { navigation.set(Some(index)); }
            }).child(|| { text().content("Explore button groups"); });
        }
    });
    gap(Dp(32.0));
}
