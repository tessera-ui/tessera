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

use crate::res::I18N;

use super::{
    INK,
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) enum Kind {
    #[default]
    Checkbox,
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
                .content(I18N.text(&(label)))
                .style(style.unwrap_or(type_style(16.0, 24.0)))
                .color(color.unwrap_or(scheme.on_surface));
        });
}

fn guidelines(kind: Kind) -> Vec<Chapter> {
    match kind {
        Kind::Checkbox => vec![
            Chapter {
                title: "ui-usage",
                text: "ui-use-checkboxes-for-related-choices-that-can-be-selected-independently-a-parent-control-mak",
                figure: Figure::Children,
            },
            Chapter {
                title: "ui-anatomy",
                text: "ui-a-checkbox-contains-a-container-and-a-selection-icon",
                figure: Figure::Anatomy,
            },
            Chapter {
                title: "ui-responsive-layout",
                text: "ui-at-wider-breakpoints-a-contained-region-can-group-related-choices-and-actions",
                figure: Figure::Example,
            },
            Chapter {
                title: "ui-behavior",
                text: "ui-select-children-independently-the-parent-is-mixed-when-only-some-children-are-selected-sel",
                figure: Figure::Children,
            },
        ],
        Kind::Switch => vec![
            Chapter {
                title: "ui-usage",
                text: "ui-use-a-switch-for-an-independent-binary-setting-that-takes-effect-immediately-use-radio-but",
                figure: Figure::Example,
            },
            Chapter {
                title: "ui-anatomy",
                text: "ui-the-track-contains-a-handle-and-an-optional-icon-icons-should-communicate-on-or-off-clearl",
                figure: Figure::Anatomy,
            },
            Chapter {
                title: "ui-placement",
                text: "ui-settings-commonly-use-vertically-stacked-rows-with-an-inline-label-for-each-switch",
                figure: Figure::Example,
            },
            Chapter {
                title: "ui-behavior",
                text: "ui-the-handle-moves-to-the-opposite-side-and-changes-size-as-the-value-changes-the-setting-ta",
                figure: Figure::Example,
            },
        ],
    }
}

fn accessibility(kind: Kind) -> Vec<Chapter> {
    let mut chapters = vec![
        Chapter {
            title: "ui-use-cases",
            text: "ui-people-using-assistive-technology-need-to-find-the-control-change-its-selection-and-receiv",
            figure: Figure::Example,
        },
        Chapter {
            title: "ui-interaction-style",
            text: match kind {
                Kind::Checkbox => {
                    "ui-both-the-label-and-checkbox-select-the-option-child-selections-remain-independent-a-partia"
                }
                Kind::Switch => {
                    "ui-touch-and-pointer-feedback-communicate-that-the-handle-is-interactive-keep-the-current-on"
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
        chapters.push(Chapter {title:"ui-avoid-applying-density-by-default",text:"ui-keep-default-targets-at-least-48-by-48dp-a-denser-presentation-should-be-an-explicit-prefe",figure:Figure::None});
    } else {
        chapters.push(Chapter {
            title: "ui-initial-focus",
            text: "ui-focus-lands-on-the-switch-handle-space-or-enter-changes-its-value",
            figure: Figure::None,
        });
    }
    chapters.push(Chapter {
        title: "ui-keyboard-navigation",
        text: "ui-use-the-controls-below-to-try-keyboard-navigation",
        figure: Figure::Keyboard,
    });
    chapters.push(Chapter {title:"ui-labeling-elements",text:"ui-use-a-concise-label-that-describes-the-setting-an-accessible-name-should-clarify-ambiguous",figure:Figure::Example});
    chapters
}

#[tessera]
pub(super) fn selection_article(
    kind: Kind,
    sections: Vec<SpecSection>,
    example: RenderSlot,
    anatomy: RenderSlot,

    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    let title = match kind {
        Kind::Checkbox => "ui-checkbox",
        Kind::Switch => "ui-switch",
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
    let mut page = article(title, description, sections)
        .route_optional(route)
        .hero_shared(example);
    if kind == Kind::Switch {
        page = page.intro_shared(anatomy);
    }
    page.pages(vec![
        ArticlePage {title:"ui-overview",intro:Some(RenderSlot::new(move || {
            paragraph(match kind {
                Kind::Checkbox=>"ui-choose-multiple-related-options-use-concise-labels-and-make-selected-unselected-and-mixed",
                Kind::Switch=>"ui-turn-an-independent-setting-on-or-off-its-effect-is-immediate-and-the-handle-makes-the-cur",
            });
            gap(Dp(32.0)); anatomy.render();
            gap(Dp(32.0)); example.render();
        })),sections:vec![]},
        ArticlePage {title:"ui-guidelines",intro:None,sections:guide},
        ArticlePage {title:"ui-accessibility",intro:None,sections:access},
    ]);
}

#[tessera]
fn paragraph(#[prop(into)] value: String) {
    text()
        .content(I18N.text(value))
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn chapter_content(kind: Kind, chapter: Chapter, example: RenderSlot, anatomy: RenderSlot) {
    column().modifier(Modifier::new().fill_max_width()).children(move || {
        paragraph(chapter.text);
        if chapter.title=="ui-usage" {
            gap(Dp(24.0)); subheading("ui-alternate-selection-controls".into());
            paragraph("ui-checkboxes-allow-several-related-choices-radio-buttons-select-one-option-and-switches-cont");
        }
        if chapter.title=="ui-interaction-style" && kind!=Kind::Checkbox {
            gap(Dp(24.0)); subheading("ui-avoid-applying-density-by-default".into());
            paragraph("ui-retain-the-default-minimum-48dp-targets-let-people-explicitly-opt-into-a-denser-presentati");
        }
        gap(Dp(32.0));
        match chapter.figure {
            Figure::None=>{},
            Figure::Example=>example.render(),
            Figure::Anatomy=>anatomy.render(),
            Figure::Children=>{ super::selection_controls::checkbox_group(); },
            Figure::Keyboard=>{
                spec_table(vec!["ui-keys".into(),"ui-action".into()],vec![vec!["ui-tab-shift-tab".into(),"ui-move-focus-between-enabled-controls".into()],vec!["ui-space-enter".into(),"ui-toggle-the-focused-control".into()]]);
                gap(Dp(32.0)); example.render();
            },
        }
    });
}
