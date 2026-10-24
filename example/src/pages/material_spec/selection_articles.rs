//! Selection control article pages.
//!
//! ## Usage
//!
//! Explore selection patterns, interaction requirements and keyboard
//! navigation.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs},
    text::text,
    theme::{MaterialTheme, TextStyle},
};
use tessera_ui::{Callback, Color, Dp, Modifier, RenderSlot, tessera, use_context};

use super::{
    INK,
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) enum Kind {
    #[default]
    Checkbox,
    Radio,
    Switch,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Chapter {
    title: &'static str,
    text: &'static str,
    figure: Figure,
}

#[derive(Clone, Copy, Default, PartialEq)]
enum Figure {
    #[default]
    None,
    Example,
    Anatomy,
    Children,
    Keyboard,
}

fn label_modifier(on_click: Callback) -> Modifier {
    Modifier::new()
        .clickable_with(ClickableArgs {
            on_click,
            ..Default::default()
        })
        .semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        })
}

#[tessera]
pub(super) fn option_label(
    label: String,
    on_click: Callback,
    enabled: bool,
    style: Option<TextStyle>,
    color: Option<Color>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let modifier = if enabled {
        label_modifier(on_click)
    } else {
        Modifier::new().semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        })
    };
    boxed()
        .alignment(Alignment::CenterStart)
        .modifier(modifier.default_min_size(Dp(48.0), Dp(48.0)))
        .children(move || {
            text()
                .content(label.clone())
                .style(style.unwrap_or(type_style(16.0, 24.0)))
                .color(color.unwrap_or(scheme.on_surface));
        });
}

fn guidelines(kind: Kind) -> Vec<Chapter> {
    match kind {
        Kind::Checkbox => vec![
            Chapter {
                title: "Usage",
                text: "Use checkboxes for related choices that can be selected independently. A parent control makes selecting many children easier.",
                figure: Figure::Children,
            },
            Chapter {
                title: "Anatomy",
                text: "A checkbox contains a container and a selection icon.",
                figure: Figure::Anatomy,
            },
            Chapter {
                title: "Responsive layout",
                text: "At wider breakpoints, a contained region can group related choices and actions.",
                figure: Figure::Example,
            },
            Chapter {
                title: "Behavior",
                text: "Select children independently. The parent is mixed when only some children are selected. Selecting a mixed parent selects every child; clearing it clears them all.",
                figure: Figure::Children,
            },
        ],
        Kind::Radio => vec![
            Chapter {
                title: "Usage",
                text: "Use radio buttons to expose a short list of mutually exclusive options. Prefer a menu when space is limited. Avoid nested radio choices.",
                figure: Figure::Example,
            },
            Chapter {
                title: "Anatomy",
                text: "Pair every selected or unselected icon with a clear, adjacent label.",
                figure: Figure::Anatomy,
            },
            Chapter {
                title: "Placement",
                text: "Stack choices vertically and provide a suitable initial selection.",
                figure: Figure::Example,
            },
            Chapter {
                title: "Behavior",
                text: "Selecting an icon or its label chooses that option and clears the previous choice. Apply the result immediately unless a surrounding form requires confirmation.",
                figure: Figure::Example,
            },
        ],
        Kind::Switch => vec![
            Chapter {
                title: "Usage",
                text: "Use a switch for an independent binary setting that takes effect immediately. Use radio buttons for one choice from a set, checkboxes for related multiple choices, and buttons for commands.",
                figure: Figure::Example,
            },
            Chapter {
                title: "Anatomy",
                text: "The track contains a handle and an optional icon. Icons should communicate on or off clearly, such as a checkmark and a cross.",
                figure: Figure::Anatomy,
            },
            Chapter {
                title: "Label text",
                text: "An adjacent label describes what enabling the setting does. Keep text outside the handle.",
                figure: Figure::Example,
            },
            Chapter {
                title: "Placement",
                text: "Settings commonly use vertically stacked rows with an inline label for each switch.",
                figure: Figure::Example,
            },
            Chapter {
                title: "Behavior",
                text: "The handle moves to the opposite side and changes size as the value changes. The setting takes effect without a separate Save action.",
                figure: Figure::Example,
            },
        ],
    }
}

fn accessibility(kind: Kind) -> Vec<Chapter> {
    let mut chapters = vec![
        Chapter {
            title: "Use cases",
            text: "People using assistive technology need to find the control, change its selection and receive feedback about the result.",
            figure: Figure::Example,
        },
        Chapter {
            title: "Interaction & style",
            text: match kind {
                Kind::Checkbox => {
                    "Both the label and checkbox select the option. Child selections remain independent; a partially selected parent reports a mixed state."
                }
                Kind::Radio => {
                    "An icon and its label select the same option. Selecting another option clears the previous one; activating the selected option does not clear the group."
                }
                Kind::Switch => {
                    "Touch and pointer feedback communicate that the handle is interactive. Keep the current on or off value easy to distinguish."
                }
            },
            figure: if kind == Kind::Checkbox {
                Figure::Children
            } else {
                Figure::Example
            },
        },
    ];
    if kind == Kind::Checkbox {
        chapters.push(Chapter {title:"Avoid applying density by default",text:"Keep default targets at least 48 by 48dp. A denser presentation should be an explicit preference that remains easy to reverse.",figure:Figure::None});
    } else {
        chapters.push(Chapter {title:"Initial focus",text:if kind==Kind::Radio { "Tab enters at the selected radio. If nothing is selected, forward traversal starts at the first option and reverse traversal starts at the last." } else { "Focus lands on the switch handle. Space or Enter changes its value." },figure:Figure::None});
    }
    chapters.push(Chapter {
        title: "Keyboard navigation",
        text: "Use the controls below to try keyboard navigation.",
        figure: Figure::Keyboard,
    });
    chapters.push(Chapter {title:"Labeling elements",text:if kind==Kind::Radio { "Name the group after its category, and name each option after its adjacent label." } else { "Use a concise label that describes the setting. An accessible name should clarify ambiguous visible wording." },figure:Figure::Example});
    chapters
}

#[tessera]
pub(super) fn selection_article(
    kind: Kind,
    sections: Vec<SpecSection>,
    example: RenderSlot,
    anatomy: RenderSlot,
) {
    let title = match kind {
        Kind::Checkbox => "Checkbox",
        Kind::Radio => "Radio button",
        Kind::Switch => "Switch",
    };
    let description = super::media::page_media(title)
        .expect("selection article media")
        .description;
    let mut guide = Vec::new();
    for chapter in guidelines(kind) {
        guide.push(SpecSection {
            title: chapter.title,
            content: RenderSlot::new(move || {
                chapter_content(kind, chapter, example, anatomy);
            }),
        });
    }
    let mut access = Vec::new();
    for chapter in accessibility(kind) {
        access.push(SpecSection {
            title: chapter.title,
            content: RenderSlot::new(move || {
                chapter_content(kind, chapter, example, anatomy);
            }),
        });
    }
    let mut page = article(title, description, sections).hero_shared(example);
    if kind == Kind::Switch {
        page = page.intro_shared(anatomy);
    }
    page.pages(vec![
        ArticlePage {title:"Overview",intro:Some(RenderSlot::new(move || {
            paragraph(match kind {
                Kind::Checkbox=>"Choose multiple related options. Use concise labels and make selected, unselected and mixed states easy to scan.",
                Kind::Radio=>"Choose one option from a visible set. Give each option a concise label and make the current selection prominent.",
                Kind::Switch=>"Turn an independent setting on or off. Its effect is immediate, and the handle makes the current value clear.",
            });
            gap(Dp(32.0)); anatomy.render();
            gap(Dp(32.0)); example.render();
        })),sections:vec![]},
        ArticlePage {title:"Guidelines",intro:None,sections:guide},
        ArticlePage {title:"Accessibility",intro:None,sections:access},
    ]);
}

#[tessera]
fn paragraph(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn chapter_content(kind: Kind, chapter: Chapter, example: RenderSlot, anatomy: RenderSlot) {
    column().modifier(Modifier::new().fill_max_width()).children(move || {
        paragraph(chapter.text);
        if chapter.title=="Usage" {
            gap(Dp(24.0)); subheading("Alternate selection controls".into());
            paragraph("Checkboxes allow several related choices, radio buttons select one option, and switches control independent settings.");
        }
        if chapter.title=="Interaction & style" && kind!=Kind::Checkbox {
            gap(Dp(24.0)); subheading("Avoid applying density by default".into());
            paragraph("Retain the default minimum 48dp targets. Let people explicitly opt into a denser presentation.");
        }
        gap(Dp(32.0));
        match chapter.figure {
            Figure::None=>{},
            Figure::Example=>example.render(),
            Figure::Anatomy=>anatomy.render(),
            Figure::Children=>{ super::selection_controls::checkbox_group(); },
            Figure::Keyboard=>{
                if kind==Kind::Radio {
                    spec_table(vec!["Keys".into(),"Action".into()],vec![vec!["Tab / Shift + Tab".into(),"Enter or leave the radio group".into()],vec!["Arrow keys".into(),"Focus and select the next or previous option; wrap at the ends".into()],vec!["Space".into(),"Select the focused option".into()]]);
                } else {
                    spec_table(vec!["Keys".into(),"Action".into()],vec![vec!["Tab / Shift + Tab".into(),"Move focus between enabled controls".into()],vec!["Space / Enter".into(),"Toggle the focused control".into()]]);
                }
                gap(Dp(32.0)); example.render();
            },
        }
    });
}
