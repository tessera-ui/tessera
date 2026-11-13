//! Button family overview, guidance and accessibility pages.
//!
//! ## Usage
//!
//! Explore action hierarchy, selection, menu behavior and keyboard access.

use tessera_components::{
    column::column,
    md3::{button::button, text::text, theme::MaterialTheme},
    modifier::ModifierExt as _,
    shape_def::Shape,
};
use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{Callback, Dp, Modifier, RenderSlot, accesskit::Role, tessera, use_context};

use crate::res::I18N;

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
    ExtendedFab,
    Segmented,
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
        Kind::Buttons => "ui-buttons",
        Kind::Groups => "ui-button-groups",
        Kind::Icons => "ui-icon-buttons",
        Kind::ExtendedFab => "ui-extended-fabs",
        Kind::Segmented => "ui-segmented-buttons",
    }
}

fn summary(kind: Kind) -> &'static str {
    match kind {
        Kind::Buttons => {
            "ui-represent-actions-with-default-or-toggle-buttons-choose-among-five-sizes-two-shapes-and-fi"
        }
        Kind::Groups => {
            "ui-group-related-buttons-using-standard-or-connected-layouts-coordinated-shapes-communicate-p"
        }
        Kind::Icons => {
            "ui-use-recognizable-icons-for-common-actions-default-buttons-perform-actions-toggle-buttons-c"
        }
        Kind::ExtendedFab => {
            "ui-keep-a-primary-action-visible-with-a-concise-text-label-and-optional-icon-small-medium-and"
        }
        Kind::Segmented => {
            "ui-compare-single-and-multiple-selection-among-two-to-five-related-options-connected-button-g"
        }
    }
}

fn guidelines(kind: Kind) -> Vec<Chapter> {
    use Figure::{Anatomy, Behavior, Example};
    match kind {
        Kind::Buttons => vec![
            chapter(
                "ui-usage",
                "ui-make-important-actions-prominent-without-crowding-the-screen-let-width-accommodate-the-lab",
                &[],
                Example,
            ),
            chapter(
                "ui-toggle-buttons",
                "ui-use-toggles-for-binary-choices-communicate-selection-through-shape-color-and-an-appropriat",
                &[],
                Behavior,
            ),
            chapter(
                "ui-anatomy",
                "ui-use-concise-sentence-case-labels-sufficient-container-width-and-at-most-one-leading-icon",
                &["ui-label-text", "ui-container", "ui-icon-optional"],
                Anatomy,
            ),
            chapter(
                "ui-color-styles",
                "ui-use-filled-for-primary-actions-tonal-or-outlined-for-secondary-actions-and-text-for-lower",
                &[
                    "ui-elevated-style",
                    "ui-filled-style",
                    "ui-tonal-style",
                    "ui-outlined-style",
                    "ui-text-style",
                ],
                Example,
            ),
            chapter(
                "ui-adaptive-design",
                "ui-preserve-action-hierarchy-as-size-and-arrangement-change-keep-the-icon-and-text-grouped-an",
                &["ui-resizing", "ui-presentation"],
                Example,
            ),
        ],
        Kind::Groups => vec![
            chapter(
                "ui-usage",
                "ui-use-consistent-sizes-and-shapes-in-a-group-connected-groups-support-related-toggle-choices",
                &["ui-color"],
                Example,
            ),
            chapter(
                "ui-anatomy",
                "ui-standard-groups-accommodate-motion-around-their-contents-connected-groups-expand-within-th",
                &["ui-container"],
                Anatomy,
            ),
            chapter(
                "ui-adaptive-design",
                "ui-keep-each-group-on-one-line-adjust-size-and-width-while-preserving-emphasis-move-secondary",
                &["ui-resizing", "ui-presentation"],
                Example,
            ),
            chapter(
                "ui-behavior",
                "ui-pressing-changes-shape-standard-groups-also-adjust-neighboring-widths-selection-changes-ro",
                &["ui-pressed", "ui-selected"],
                Behavior,
            ),
        ],
        Kind::Icons => vec![
            chapter(
                "ui-usage",
                "ui-use-default-icon-buttons-for-actions-and-toggle-icon-buttons-for-binary-choices",
                &[],
                Example,
            ),
            chapter(
                "ui-anatomy",
                "ui-use-recognizable-icons-and-distinguish-selection-through-filled-and-outlined-symbols",
                &[],
                Anatomy,
            ),
            chapter(
                "ui-color",
                "ui-choose-filled-tonal-outlined-or-standard-emphasis-for-the-surrounding-content",
                &[],
                Example,
            ),
            chapter(
                "ui-size",
                "ui-choose-among-five-sizes-according-to-action-hierarchy-not-the-available-gap",
                &[],
                Example,
            ),
            chapter(
                "ui-width",
                "ui-use-narrow-default-and-wide-proportions-while-preserving-minimum-targets",
                &[],
                Example,
            ),
            chapter(
                "ui-shape",
                "ui-use-round-and-square-resting-shapes-with-clear-pressed-and-selected-transformations",
                &[],
                Behavior,
            ),
            chapter(
                "ui-placement",
                "ui-group-meaningful-actions-in-app-bars-cards-and-toolbars",
                &[],
                Example,
            ),
            chapter(
                "ui-behavior",
                "ui-default-actions-perform-a-command-toggles-retain-selection-and-expose-a-useful-hover-label",
                &[],
                Behavior,
            ),
        ],
        Kind::ExtendedFab => vec![
            chapter(
                "ui-usage",
                "ui-use-one-extended-fab-when-a-primary-action-needs-additional-clarity-choose-small-medium-or",
                &["ui-additional-emphasis", "ui-choosing-a-size"],
                Example,
            ),
            chapter(
                "ui-anatomy",
                "ui-let-the-rounded-container-hug-its-contents-keep-the-label-concise-and-visible-an-icon-is-o",
                &["ui-container", "ui-icon-optional", "ui-label-text"],
                Anatomy,
            ),
            chapter(
                "ui-placement",
                "ui-keep-the-action-above-the-page-content-avoid-covering-other-actions-or-placing-it-inside-c",
                &[],
                Example,
            ),
            chapter(
                "ui-responsive-layout",
                "ui-adapt-between-fab-and-extended-fab-as-space-changes-mirror-icon-and-label-ordering-in-righ",
                &["ui-right-to-left-languages", "ui-breakpoints"],
                Example,
            ),
            chapter(
                "ui-behavior",
                "ui-expansion-reveals-the-label-collapsing-reduces-occupied-space-scrolling-can-drive-the-tran",
                &[
                    "ui-appearing",
                    "ui-expanding",
                    "ui-transforming",
                    "ui-scrolling",
                ],
                Behavior,
            ),
        ],
        Kind::Segmented => vec![
            chapter(
                "ui-usage",
                "ui-use-a-small-set-of-related-choices-to-switch-views-or-filters",
                &[],
                Example,
            ),
            chapter(
                "ui-anatomy",
                "ui-keep-two-to-five-segments-on-one-line-with-consistent-label-types-and-short-text",
                &["ui-segments", "ui-container", "ui-icons", "ui-label-text"],
                Anatomy,
            ),
            chapter(
                "ui-single-select-7063",
                "ui-select-one-option-from-the-set",
                &[],
                Example,
            ),
            chapter(
                "ui-multi-select-7064",
                "ui-allow-independent-choices-including-selecting-all-or-clearing-the-entire-set",
                &[],
                Behavior,
            ),
            chapter(
                "ui-placement",
                "ui-retain-margins-and-avoid-excessive-stretching-in-larger-windows",
                &[],
                Example,
            ),
            chapter(
                "ui-behavior",
                "ui-a-selected-icon-can-become-a-checkmark-selection-remains-distinguishable-beyond-color",
                &[],
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
                "ui-use-cases",
                "ui-people-need-to-understand-the-choices-navigate-between-segments-and-change-selection",
                &[],
                Example,
            ),
            chapter(
                "ui-interaction-style",
                "ui-use-tab-to-enter-the-group-arrows-to-move-between-choices-and-activation-keys-to-select",
                &[],
                Behavior,
            ),
            chapter(
                "ui-color-contrast",
                "ui-keep-outlines-distinguishable-from-the-surface-and-communicate-selection-with-a-non-color",
                &[],
                Anatomy,
            ),
            chapter(
                "ui-initial-focus",
                "ui-start-at-the-first-segment-in-reading-order",
                &[],
                Example,
            ),
            chapter(
                "ui-keyboard-navigation",
                "ui-try-traversal-and-activation-in-the-live-example",
                &[],
                Keyboard,
            ),
            chapter(
                "ui-labeling-elements",
                "ui-name-each-choice-from-its-visible-label-or-the-meaning-of-its-icon",
                &[],
                Example,
            ),
        ];
    }
    let mut pages = vec![
        chapter(
            "ui-use-cases",
            "ui-people-need-to-reach-the-action-understand-its-purpose-activate-it-and-identify-any-select",
            &[],
            Example,
        ),
        chapter(
            "ui-interaction-style",
            match kind {
                Kind::Buttons => {
                    "ui-maintain-readable-contrast-and-concise-labels-at-enlarged-text-sizes-motion-should-remain"
                }
                Kind::Groups => {
                    "ui-retain-at-least-48dp-targets-including-the-smaller-sizes-the-group-container-is-not-a-keyb"
                }
                Kind::Icons => {
                    "ui-keep-the-icon-distinguishable-from-its-surface-with-sufficient-contrast"
                }
                Kind::ExtendedFab => {
                    "ui-place-the-primary-action-where-it-is-easy-to-reach-without-obscuring-other-targets"
                }
                Kind::Segmented => {
                    "ui-use-keyboard-traversal-and-a-checkmark-or-other-non-color-cue-for-selection"
                }
            },
            match kind {
                Kind::Buttons => &["ui-color-contrast", "ui-200-text-size", "ui-rapid-clicks"],
                Kind::Groups => &["ui-initial-focus"],
                _ => &[],
            },
            Anatomy,
        ),
    ];
    if kind == Kind::ExtendedFab {
        pages.push(chapter(
            "ui-initial-focus",
            "ui-treat-the-icon-and-visible-label-as-one-target-and-prioritize-the-primary-action-in-the-pa",
            &[],
            Behavior,
        ));
    }
    pages.push(chapter(
        "ui-keyboard-navigation",
        "ui-try-keyboard-traversal-and-activation-in-the-live-example",
        &[],
        Keyboard,
    ));
    pages.push(chapter("ui-labeling-elements",match kind {
        Kind::Icons=>"ui-name-the-action-rather-than-the-appearance-of-the-icon",
        _=>"ui-use-names-consistent-with-visible-labels-and-provide-useful-context-without-repeating-the",
    },&[],Example));
    if kind == Kind::Icons {
        pages.push(chapter("ui-layout-density","ui-retain-default-48dp-targets-denser-presentation-should-be-an-explicit-reversible-preferenc",&["ui-avoid-applying-density-by-default"],Example));
        pages.push(chapter(
            "ui-hover",
            "ui-show-a-concise-tooltip-that-describes-the-action",
            &[],
            Behavior,
        ));
    }
    pages
}

fn detail(kind: Kind, title: &str) -> &'static str {
    match title {
        "ui-color" | "ui-color-sets" => {
            if kind == Kind::Groups {
                "ui-keep-connected-choices-in-one-consistent-color-style"
            } else {
                "ui-choose-emphasis-and-color-appropriate-to-the-surrounding-actions"
            }
        }
        "ui-label-text" => {
            if kind == Kind::ExtendedFab {
                "ui-prefer-one-or-two-words-without-wrapping-or-truncation"
            } else {
                "ui-use-short-clear-labels-in-sentence-case"
            }
        }
        "ui-container" => {
            if kind == Kind::Groups {
                "ui-standard-groups-hug-contents-connected-groups-expand-within-available-space"
            } else {
                "ui-allow-enough-room-for-the-complete-icon-and-label"
            }
        }
        "ui-icon" | "ui-icons" | "ui-icon-optional" => {
            "ui-use-a-recognizable-symbol-that-communicates-the-action-or-selection"
        }
        "ui-size-width" | "ui-choosing-a-size" => {
            "ui-choose-size-and-width-according-to-the-action-s-visual-priority"
        }
        "ui-elevated-style" => {
            "ui-use-a-shadow-when-separation-from-a-prominent-background-is-necessary"
        }
        "ui-filled-style" => "ui-reserve-strong-visual-emphasis-for-important-primary-actions",
        "ui-tonal-style" => "ui-use-tonal-fills-for-secondary-actions-needing-moderate-emphasis",
        "ui-outlined-style" => "ui-use-on-simple-backgrounds-for-supporting-actions",
        "ui-text-style" => "ui-use-for-low-emphasis-actions-including-dialog-and-snackbar-actions",
        "ui-resizing" | "ui-breakpoints" => {
            "ui-adapt-dimensions-while-preserving-the-hierarchy-of-primary-and-secondary-actions"
        }
        "ui-presentation" => "ui-rearrange-secondary-actions-when-the-available-space-changes",
        "ui-pressed" => "ui-pressing-changes-shape-standard-groups-also-adjust-neighboring-widths",
        "ui-selected" | "ui-selection" => {
            "ui-communicate-selection-using-shape-and-icon-changes-as-well-as-color"
        }
        "ui-additional-emphasis" => "ui-use-a-label-when-an-icon-alone-would-be-ambiguous",
        "ui-right-to-left-languages" => {
            "ui-mirror-the-icon-and-label-ordering-for-right-to-left-reading"
        }
        "ui-appearing" => "ui-animate-the-action-from-its-anchor-without-moving-unrelated-content",
        "ui-expanding" => "ui-a-container-transform-can-connect-the-action-to-its-destination",
        "ui-transforming" => "ui-the-extended-action-can-reduce-to-an-icon-only-fab",
        "ui-scrolling" => match kind {
            Kind::ExtendedFab => {
                "ui-collapse-the-label-on-downward-scrolling-and-restore-it-on-return"
            }
            _ => "ui-keep-the-floating-action-above-the-scrolling-content",
        },
        "ui-moving-across-tabs" => "ui-hide-and-reintroduce-the-fab-as-the-new-page-settles",
        "ui-menu-placement" => "ui-align-to-a-button-edge-and-retain-a-4dp-gap",
        "ui-segments" => "ui-keep-two-to-five-related-choices-on-one-line",
        "ui-hover" => "ui-describe-the-action-in-a-concise-tooltip",
        "ui-color-contrast" => {
            "ui-keep-enabled-actions-visually-distinguishable-from-their-background"
        }
        "ui-200-text-size" => "ui-choose-concise-labels-that-remain-readable-when-text-is-enlarged",
        "ui-rapid-clicks" => "ui-motion-should-settle-clearly-during-repeated-presses",
        "ui-initial-focus" => "ui-focus-enters-the-buttons-rather-than-stopping-on-their-container",
        "ui-avoid-applying-density-by-default" => {
            "ui-keep-48dp-default-targets-and-make-denser-layouts-an-explicit-preference"
        }
        _ => unreachable!("ui-unregistered-action-article-subheading"),
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
    on_recommended: Option<Callback>,

    route: Option<crate::pages::material_spec::article::ArticleRoute>,
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
            title: "ui-use-cases",
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
    let article = article(title, description, sections)
        .route_optional(route)
        .pages(vec![
            ArticlePage {
                title: "ui-overview",
                intro: Some(RenderSlot::new(move || {
                    recommendation(kind).on_recommended_optional(on_recommended);
                    paragraph(summary(kind));
                    gap(Dp(32.0));
                    overview.render();
                })),
                sections: vec![],
            },
            ArticlePage {
                title: "ui-guidelines",
                intro: if kind == Kind::Segmented {
                    Some(RenderSlot::new(move || {
                        recommendation(kind).on_recommended_optional(on_recommended);
                    }))
                } else {
                    None
                },
                sections: guide,
            },
            ArticlePage {
                title: "ui-accessibility",
                intro: if kind == Kind::Segmented {
                    Some(RenderSlot::new(move || {
                        recommendation(kind).on_recommended_optional(on_recommended);
                    }))
                } else {
                    None
                },
                sections: access,
            },
        ]);
    if kind == Kind::Segmented {
        article.intro(move || {
            recommendation(kind).on_recommended_optional(on_recommended);
        });
    }
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
    text(I18N.text(value))
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
            title @ ("ui-usage" | "ui-anatomy" | "ui-color" | "ui-size" | "ui-width"
            | "ui-shape" | "ui-placement" | "ui-behavior") => {
                super::icon_buttons::articles::guidance(title.into());
                return;
            }
            "ui-interaction-style" => {
                super::icon_buttons::articles::contrast();
                return;
            }
            "ui-labeling-elements" => {
                super::icon_buttons::context::labeling();
                return;
            }
            "ui-layout-density" => {
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
                            "ui-tab-shift-tab".into(),
                            "ui-move-between-available-actions".into(),
                        ],
                        vec![
                            "ui-space-enter".into(),
                            "ui-activate-the-focused-action".into(),
                        ],
                    ];
                    if matches!(kind, Kind::Groups | Kind::Segmented) {
                        rows.push(vec![
                            "ui-arrow-keys".into(),
                            "ui-move-between-items-in-the-group".into(),
                        ]);
                    }
                    spec_table(vec!["ui-keys".into(), "ui-action".into()], rows);
                    gap(Dp(32.0));
                    behavior.render();
                }
            }
        });
}

#[tessera]
fn recommendation(kind: Kind, on_recommended: Option<Callback>) {
    if kind != Kind::Segmented {
        return;
    }
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().modifier(Modifier::new().fill_max_width().background_with_shape(scheme.secondary_container,Shape::rounded_rectangle(Dp(16.0))).padding_all(Dp(24.0)).semantics(SemanticsArgs {role:Some(Role::Note),label:Some(I18N.text("ui-m3-expressive-recommendation")),merge_descendants:true,..Default::default()})).children(move || {
        paragraph("ui-segmented-buttons-are-no-longer-recommended-for-m3-expressive-use-a-connected-button-group");
        if let Some(on_recommended)=on_recommended {
            gap(Dp(16.0)); button().text().accessibility_label(I18N.text("ui-open-recommended-button-groups")).on_click_shared(on_recommended).child(move || { text(I18N.text("ui-explore-button-groups")); });
        }
    });
    gap(Dp(32.0));
}
