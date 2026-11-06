use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    badge::badge_with_content,
    boxed::boxed,
    button::button,
    column::column,
    divider::horizontal_divider,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    menus::{
        MenuColorStyle, MenuController, MenuItemSelectionMode, MenuPosition, menu_group, menu_item,
        menu_provider,
    },
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    split_buttons::{SplitButtonDefaults, SplitButtonSize},
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, modifier::FocusModifierExt as _, remember,
    tessera, use_context,
};

use crate::res::I18N;

use super::{
    ACCENT, INK,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[path = "menus_accessibility.rs"]
mod accessibility_figures;

#[path = "menus_selection.rs"]
mod selection_figures;

#[path = "menus_guides.rs"]
mod guides;

#[tessera]
pub(super) fn menus_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-menus",
        "ui-menus-display-a-list-of-choices-on-a-temporary-surface",
        vec![
            SpecSection {
                title: "ui-variants",
                content: RenderSlot::new(variants),
            },
            SpecSection {
                title: "ui-configurations",
                content: RenderSlot::new(configurations),
            },
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "ui-anatomy",
                content: RenderSlot::new(|| {
                    subheading("ui-vertical-menus".into());
                    gap(Dp(24.0));
                    menu_figure(true, false, MenuColorStyle::Standard);
                }),
            },
            SpecSection {
                title: "ui-color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "ui-states",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(|| {
                    menu_figure(false, true, MenuColorStyle::Standard);
                }),
            },
            SpecSection {
                title: "ui-menu-baseline",
                content: RenderSlot::new(guides::baseline_section),
            },
        ],
    )
    .route_optional(route.clone())
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview_intro)),
            sections: vec![],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(guideline_anatomy),
                },
                SpecSection {
                    title: "ui-flexibility-slots",
                    content: RenderSlot::new(|| {
                        slots(true);
                    }),
                },
                SpecSection {
                    title: "ui-placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "ui-adaptive-design",
                    content: RenderSlot::new(adaptive),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(behavior),
                },
                SpecSection {
                    title: "ui-focus",
                    content: RenderSlot::new(guideline_focus),
                },
                SpecSection {
                    title: "ui-density",
                    content: RenderSlot::new(density),
                },
            ],
        },
        ArticlePage {
            title: "ui-accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-use-cases",
                    content: RenderSlot::new(use_cases),
                },
                SpecSection {
                    title: "ui-interaction-style",
                    content: RenderSlot::new(interaction),
                },
                SpecSection {
                    title: "ui-flexibility-slots",
                    content: RenderSlot::new(|| {
                        slots(false);
                    }),
                },
                SpecSection {
                    title: "ui-focus",
                    content: RenderSlot::new(accessible_focus),
                },
                SpecSection {
                    title: "ui-keyboard-navigation",
                    content: RenderSlot::new(keyboard),
                },
                SpecSection {
                    title: "ui-interactability",
                    content: RenderSlot::new(interactability),
                },
                SpecSection {
                    title: "ui-labeling-elements",
                    content: RenderSlot::new(labeling),
                },
            ],
        },
    ]);
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

#[tessera]
fn overview_intro() {
    prose(
        "ui-use-a-menu-to-show-a-temporary-set-of-actions-to-show-actions-on-screen-at-all-times-use-a",
    );
    gap(Dp(16.0));
    guides::bullets(strings(&[
        "ui-menus-can-open-from-many-components-including-icon-buttons-split-buttons-and-text-fields",
        "ui-context-menus-provide-actions-for-a-specific-element-like-an-image-or-highlighted-text-and",
    ]));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                configuration(true, MenuColorStyle::Vibrant);
                guides::submenu_pair(
                    "ui-line-spacing",
                    strings(&["ui-single", "1.15", "ui-double", "ui-custom-1-2"]),
                    3,
                );
            });
    }));
    gap(Dp(12.0));
    guides::caption(
        "ui-vertical-menus-can-include-vibrant-colors-gaps-dividers-and-submenus-to-organize-a-list-of",
    );
    gap(Dp(32.0));
    launch();
    gap(Dp(8.0));
    guides::caption("ui-open-the-live-menu-then-hover-or-focus-share-to-reveal-its-submenu");
}

#[tessera]
fn usage() {
    prose(
        "ui-use-a-menu-to-show-a-temporary-set-of-actions-to-show-actions-on-screen-at-all-times-use-a",
    );
    gap(Dp(8.0));
    prose("ui-a-menu-takes-up-less-space-than-a-set-of-radio-buttons-or-chips");
    gap(Dp(24.0));
    subheading("ui-color-options".into()).level(4);
    gap(Dp(8.0));
    prose("ui-menus-have-two-color-mappings");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-standard-surface-based-lower-visual-emphasis",
        "ui-vibrant-tertiary-based-higher-visual-emphasis",
    ]));
    prose("ui-vibrant-menus-are-more-prominent-and-should-be-used-sparingly");
    gap(Dp(24.0));
    guides::line_spacing_figure();
    gap(Dp(32.0));
    subheading("ui-opening-menus".into());
    gap(Dp(8.0));
    prose("ui-menus-temporarily-appear-in-front-of-all-other-permanent-ui-elements");
    gap(Dp(8.0));
    prose("ui-a-menu-should-open-when-a-person");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-selects-an-element-such-as-an-icon-button-or-text-field",
        "ui-performs-a-specific-action-to-trigger-the-menu-like-right-click-or-press-and-hold",
    ]));
    prose("ui-use-menus-in-situations-that-need-extra-actions-like");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-overflow-menus",
        "ui-text-field-dropdown-menus",
        "ui-select-menus",
        "ui-context-menus",
    ]));
    gap(Dp(16.0));
    guides::opening_figure();
    gap(Dp(12.0));
    guides::caption("ui-menus-appear-in-front-of-all-other-ui-elements");
    gap(Dp(32.0));
    subheading("ui-menu-groups".into());
    gap(Dp(8.0));
    prose(
        "ui-vertical-menu-items-can-be-grouped-by-adding-a-divider-or-small-gap-use-groups-to-bundle-s",
    );
    gap(Dp(24.0));
    configurations_figure();
    gap(Dp(16.0));
    prose("ui-menu-items-can-be-grouped-to-be-more-scannable");
    gap(Dp(8.0));
    guides::legend(strings(&[
        "ui-standard-vertical-menu",
        "ui-grouped-vertical-menu",
    ]));
    gap(Dp(32.0));
    subheading("ui-context-menus".into());
    gap(Dp(8.0));
    prose(
        "ui-context-menus-provide-a-list-of-additional-actions-a-person-can-take-on-an-item-a-secondar",
    );
    gap(Dp(24.0));
    guides::context_figure();
    gap(Dp(12.0));
    guides::caption(
        "ui-a-context-menu-appears-when-right-clicking-with-a-mouse-or-trackpad-it-can-reveal-key-acti",
    );
}

#[tessera]
fn guideline_anatomy() {
    menu_figure(true, false, MenuColorStyle::Standard);
    gap(Dp(32.0));
    subheading("ui-menu-items".into());
    gap(Dp(8.0));
    prose(
        "ui-menu-items-can-include-label-text-leading-icons-trailing-icons-and-keyboard-commands",
    );
    gap(Dp(8.0));
    prose(
        "ui-when-a-menu-item-can-only-be-used-under-specific-conditions-it-should-appear-disabled-rath",
    );
    gap(Dp(24.0));
    guides::disabled_figure();
    gap(Dp(12.0));
    guides::caption("ui-the-redo-action-is-disabled-when-that-action-isn-t-available");
    gap(Dp(32.0));
    subheading("ui-gaps-dividers-optional".into());
    gap(Dp(8.0));
    prose("ui-gaps-and-dividers-can-be-used-to-separate-and-group-menu-items");
    gap(Dp(24.0));
    subheading("ui-gaps".into()).level(4);
    gap(Dp(8.0));
    prose(
        "ui-use-a-gap-to-visually-divide-menu-items-into-distinct-groups-gaps-are-more-expressive-than",
    );
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-avoid-changing-the-size-of-the-gap",
        "ui-limit-the-number-of-gaps-in-a-menu-to-one-or-two",
        "ui-don-t-use-gaps-in-scrollable-menus",
    ]));
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        menu_sample(true, MenuColorStyle::Standard);
    }));
    gap(Dp(12.0));
    guides::caption("ui-gaps-separate-menu-items-using-expressive-shapes");
    gap(Dp(24.0));
    subheading("ui-dividers".into()).level(4);
    gap(Dp(8.0));
    prose("ui-dividers-create-a-more-subtle-separation-between-items-use-a-divider-for");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-scrollable-menus",
        "ui-text-fields-with-a-dropdown-menu-where-a-grouped-treatment-isn-t-appropriate",
    ]));
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        menu_sample(false, MenuColorStyle::Standard);
    }));
    gap(Dp(12.0));
    guides::caption("ui-dividers-separate-menu-items-in-baseline-menus-and-on-web");
}

#[tessera]
fn slots(guidelines: bool) {
    if guidelines {
        prose("ui-menus-have-custom-slots-that-support-more-flexible-item-layouts");
        gap(Dp(8.0));
        prose(
            "ui-when-creating-a-complicated-menu-think-of-the-menu-item-as-a-container-with-a-swappable-sl",
        );
        gap(Dp(8.0));
        prose("ui-slots-work-best-with-simple-content-such-as");
        gap(Dp(8.0));
        guides::bullets(strings(&[
            "ui-images",
            "ui-progress-indicators",
            "ui-color-swatches",
        ]));
        gap(Dp(16.0));
        guides::slots_figure();
        gap(Dp(12.0));
        guides::caption("ui-slots-can-appear-anywhere-in-a-menu");
        gap(Dp(8.0));
        guides::caption(
            "ui-note-this-page-draws-the-slot-structure-statically-the-native-menu-item-exposes-fixed-icon",
        );
        gap(Dp(24.0));
        subheading("ui-slot-accessibility".into()).level(4);
        gap(Dp(8.0));
    }
    prose("ui-use-caution-when-adding-slots-to-menus");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-make-sure-the-menu-remains-accessible",
        "ui-elements-must-follow-the-rules-and-interaction-patterns-of-the-menu-component",
        "ui-keep-the-same-menu-item-padding",
        "ui-targets-should-be-48x48dp-or-larger",
    ]));
    prose(
        "ui-don-t-add-buttons-switches-or-other-direct-actions-into-the-menu-item-nested-elements-shou",
    );
    gap(Dp(12.0));
    guides::caption(
        "ui-caution-reserve-the-use-of-slots-for-use-cases-that-maintain-the-menu-s-accessibility-and",
    );
}

#[tessera]
fn placement() {
    prose(
        "ui-a-menu-is-positioned-relative-to-the-window-edge-it-typically-appears-below-next-to-or-in",
    );
    gap(Dp(8.0));
    prose(
        "ui-if-a-menu-is-in-a-position-to-be-cut-off-it-should-automatically-reposition-to-appear-to-t",
    );
    gap(Dp(24.0));
    guides::placement_figure();
    gap(Dp(12.0));
    guides::caption("ui-menus-can-appear-around-or-in-front-of-the-element-that-opened-them");
    gap(Dp(24.0));
    launch();
    gap(Dp(8.0));
    guides::caption("ui-open-the-live-menu-to-see-it-positioned-next-to-its-trigger");
    gap(Dp(32.0));
    subheading("ui-submenus".into());
    gap(Dp(8.0));
    prose("ui-submenus-should-open-next-to-the-parent-menu-item-without-overlapping-it");
    gap(Dp(8.0));
    prose(
        "ui-submenus-are-best-used-on-large-screens-where-there-s-space-see-adaptive-guidance-for-alte",
    );
    gap(Dp(24.0));
    guides::line_spacing_figure();
    gap(Dp(12.0));
    guides::caption("ui-position-submenus-to-the-side-of-the-parent-item");
}

#[tessera]
fn adaptive() {
    subheading("ui-compact-breakpoints".into());
    gap(Dp(8.0));
    prose(
        "ui-consider-adapting-menus-into-bottom-sheets-on-small-screens-they-have-more-space-to-displa",
    );
    gap(Dp(24.0));
    guides::adaptive_figure(true);
    gap(Dp(12.0));
    guides::caption("ui-a-bottom-sheet-can-replace-a-menu-on-smaller-screens");
    gap(Dp(32.0));
    subheading("ui-other-breakpoints".into());
    gap(Dp(8.0));
    prose(
        "ui-on-medium-and-expanded-windows-menus-are-most-effective-as-they-appear-in-context-with-the",
    );
    gap(Dp(24.0));
    guides::adaptive_figure(false);
    gap(Dp(12.0));
    guides::caption("ui-on-large-screens-a-menu-is-often-more-appropriate-than-a-bottom-sheet");
}

#[tessera]
fn behavior() {
    subheading("ui-appearing".into());
    gap(Dp(8.0));
    prose(
        "ui-a-menu-can-appear-when-a-person-interacts-with-an-element-on-the-page-like-a-button-text-f",
    );
    gap(Dp(8.0));
    prose(
        "ui-a-menu-s-position-on-screen-affects-where-and-how-it-appears-if-opened-at-the-top-of-the-s",
    );
    gap(Dp(24.0));
    launch();
    gap(Dp(8.0));
    guides::caption(
        "ui-menus-at-different-positions-on-a-screen-open-in-different-directions-adapting-to-the-avai",
    );
    gap(Dp(32.0));
    subheading("ui-motion".into());
    gap(Dp(8.0));
    prose(
        "ui-menus-use-an-enter-and-exit-transition-this-animation-creates-a-relationship-between-the-m",
    );
    gap(Dp(8.0));
    prose(
        "ui-when-a-menu-expands-the-trigger-element-becomes-pressed-when-an-item-is-selected-a-ripple",
    );
    gap(Dp(8.0));
    prose("ui-in-dense-products-such-as-on-desktop-menus-can-open-instantly-to-reduce-motion");
    gap(Dp(32.0));
    subheading("ui-filtering".into());
    gap(Dp(8.0));
    prose(
        "ui-a-menu-can-include-a-text-field-to-filter-options-this-pattern-is-also-known-as-autocomple",
    );
    gap(Dp(8.0));
    prose(
        "ui-as-someone-types-the-list-of-menu-options-filters-to-show-relevant-results-this-helps-peop",
    );
    gap(Dp(32.0));
    subheading("ui-scrolling".into());
    gap(Dp(8.0));
    prose(
        "ui-menus-can-scroll-when-all-menu-items-can-t-display-at-once-in-this-state-menus-show-a-pers",
    );
    gap(Dp(8.0));
    prose("ui-don-t-use-gaps-if-a-menu-scrolls-this-is-currently-unsupported");
    gap(Dp(32.0));
    subheading("ui-selecting".into());
    gap(Dp(8.0));
    prose(
        "ui-when-a-menu-is-opened-the-corresponding-button-or-icon-button-should-remain-the-same-visua",
    );
    gap(Dp(8.0));
    prose("ui-the-overflow-icon-remains-the-same-even-after-the-menu-is-opened");
    gap(Dp(24.0));
    subheading("ui-single-and-multi-select-menus".into()).level(4);
    gap(Dp(8.0));
    prose("ui-menus-can-allow-either-single-select-or-multi-select-actions");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-single-select-menus-can-have-one-item-selected-at-a-time-when-a-new-item-is-selected-the-p",
        "ui-multi-select-menus-can-have-many-selected-items-they-stay-open-until-the-person-dismisses",
    ]));
    gap(Dp(16.0));
    selection_figures::selection_figure();
    gap(Dp(12.0));
    guides::caption("ui-menus-can-be-single-or-multi-select");
}

#[tessera]
fn guideline_focus() {
    prose(
        "ui-when-a-menu-has-multiple-submenus-focus-follows-the-current-hovered-or-focused-submenu",
    );
    gap(Dp(24.0));
    subheading("ui-shape-morphing".into()).level(4);
    gap(Dp(8.0));
    prose(
        "ui-as-a-person-moves-from-one-submenu-to-the-next-the-corners-of-the-focused-submenu-become-m",
    );
    gap(Dp(24.0));
    launch();
    gap(Dp(8.0));
    guides::caption(
        "ui-open-the-menu-then-hover-or-focus-share-to-watch-the-corner-shape-follow-the-focused-subme",
    );
}

#[tessera]
fn density() {
    prose(
        "ui-on-web-only-density-levels-control-the-spacing-between-elements-increasing-density-decreas",
    );
    gap(Dp(24.0));
    guides::density_figure();
    gap(Dp(12.0));
    guides::caption("ui-density-of-menus-from-0-to-3");
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-using-assistive-technology");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-navigate-to-open-and-close-a-menu",
        "ui-navigate-between-and-select-menu-items",
    ]));
    gap(Dp(16.0));
    launch();
}

#[tessera]
fn interaction() {
    prose("ui-menu-items-need-certain-cues-to-clearly-show-when-they-re-selected");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-by-default-menu-items-change-shape-and-color-when-selected",
        "ui-the-default-color-contrast-is-3-1-between-selected-and-unselected-menu-items",
        "ui-it-s-recommended-to-include-another-visual-cue-like-a-checkmark",
    ]));
    gap(Dp(16.0));
    accessibility_figures::state_selection();
    gap(Dp(12.0));
    guides::caption(
        "ui-use-multiple-visual-cues-like-color-shape-and-icons-to-show-that-an-item-is-selected",
    );
}

#[tessera]
fn accessible_focus() {
    subheading("ui-initial-focus".into()).level(4);
    gap(Dp(8.0));
    prose(
        "ui-when-a-menu-opens-focus-should-be-placed-on-the-first-menu-item-this-allows-people-using-a",
    );
    gap(Dp(24.0));
    subheading("ui-exiting-a-menu".into()).level(4);
    gap(Dp(8.0));
    prose("ui-people-expect-to-exit-a-menu-by");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-selecting-an-option",
        "ui-tapping-escape-or-outside-of-the-menu",
        "ui-using-the-system-back-button",
    ]));
    prose(
        "ui-where-focus-is-placed-after-closing-the-menu-depends-on-the-app-this-example-returns-focus",
    );
    gap(Dp(24.0));
    prose("ui-keyboard-navigation-on-android-and-web");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-tab-to-select-a-menu-item",
        "ui-space-or-enter-to-open-a-menu",
        "ui-space-or-enter-to-select-a-menu-item",
        "ui-escape-to-close-a-menu",
    ]));
    gap(Dp(16.0));
    launch();
}

#[tessera]
fn interactability() {
    prose("ui-disabled-menu-items-can-receive-focus-but-aren-t-selectable");
    gap(Dp(8.0));
    prose("ui-dividers-and-gaps-can-t-receive-focus");
    gap(Dp(24.0));
    guides::focus_examples();
}

#[tessera]
fn labeling() {
    prose("ui-accessibility-labels-are-used-with-assistive-technology-devices-like-screen-readers");
    gap(Dp(8.0));
    prose("ui-the-accessibility-label-should-be-the-same-as-the-menu-item-text");
    gap(Dp(24.0));
    accessibility_figures::labeling();
    gap(Dp(12.0));
    guides::caption("ui-the-menu-item-s-accessibility-label-aligns-with-the-ui-text");
    gap(Dp(24.0));
    prose(
        "ui-for-menu-items-with-text-and-an-icon-the-icon-s-accessibility-label-should-be-marked-as-de",
    );
    gap(Dp(8.0));
    guides::caption(
        "ui-for-menu-items-with-text-and-an-icon-the-icon-s-accessibility-label-is-decorative",
    );
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text(I18N.text(value))
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn variants() {
    subheading("ui-vertical-menus".into());
    gap(Dp(16.0));
    prose(
        "ui-use-vertical-menus-for-a-more-expressive-look-and-feel-including-rounded-corners-standard",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                menu_tile(true, MenuColorStyle::Standard, 1);
                menu_tile(false, MenuColorStyle::Standard, 2);
            });
    }));
    gap(Dp(16.0));
    guides::legend(strings(&[
        "ui-vertical-menu-with-gap",
        "ui-vertical-menu-with-divider",
    ]));
    gap(Dp(32.0));
    subheading("ui-baseline-variant".into());
    gap(Dp(16.0));
    prose(
        "ui-in-m3-expressive-baseline-menu-is-still-available-to-use-but-doesn-t-have-the-latest-shape",
    );
    gap(Dp(24.0));
    guides::baseline_variant();
}

#[tessera]
fn configurations() {
    subheading("ui-vertical-menus-layout".into());
    gap(Dp(24.0));
    configurations_figure();
    gap(Dp(16.0));
    guides::legend(strings(&["ui-standard", "ui-grouped"]));
}

#[tessera]
fn configurations_figure() {
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                for (index, grouped) in [false, true].into_iter().enumerate() {
                    column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            configuration(grouped, MenuColorStyle::Standard);
                            gap(Dp(24.0));
                            guides::badge(index + 1);
                        });
                }
            });
    }));
}

#[tessera]
fn configuration(grouped: bool, style: MenuColorStyle) {
    let selected = remember(|| 2_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().width(Dp(216.0)))
        .children(move || {
            row()
                .main_axis_alignment(MainAxisAlignment::End)
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    if grouped {
                        row().children(move || {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(Modifier::new().height(Dp(40.0)).background_with_shape(
                                    scheme.secondary_container,
                                    SplitButtonDefaults::leading_shape(SplitButtonSize::Small),
                                ))
                                .children(move || {
                                    row()
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .modifier(
                                            Modifier::new().padding_symmetric(Dp(16.0), Dp::ZERO),
                                        )
                                        .children(move || {
                                            icon()
                                                .painter(filled::EDIT_SVG)
                                                .size(Dp(20.0))
                                                .tint(scheme.on_secondary_container);
                                            boxed().modifier(Modifier::new().width(Dp(8.0)));
                                            text(I18N.text("ui-label"))
                                                .color(scheme.on_secondary_container);
                                        });
                                });
                            boxed().modifier(Modifier::new().width(Dp(2.0)));
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    Modifier::new()
                                        .size(Dp(48.0), Dp(40.0))
                                        .background_with_shape(
                                            scheme.secondary_container,
                                            SplitButtonDefaults::trailing_shape(
                                                SplitButtonSize::Small,
                                            ),
                                        ),
                                )
                                .children(move || {
                                    icon()
                                        .painter(filled::ARROW_DROP_DOWN_SVG)
                                        .rotation(180.0)
                                        .size(Dp(22.0))
                                        .tint(scheme.on_secondary_container);
                                });
                        });
                    } else {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(
                                Modifier::new()
                                    .size(Dp(40.0), Dp(40.0))
                                    .background_with_shape(
                                        scheme.secondary_container,
                                        Shape::rounded_rectangle(Dp(20.0)),
                                    ),
                            )
                            .children(move || {
                                icon()
                                    .painter(filled::MORE_VERT_SVG)
                                    .tint(scheme.on_secondary_container);
                            });
                    }
                });
            if grouped {
                menu_group()
                    .position(MenuPosition::First)
                    .color_style(style)
                    .children(move || {
                        menu_item()
                            .label(I18N.text("ui-item-1"))
                            .leading_icon(filled::VISIBILITY_SVG)
                            .selected(selected.get() == 0)
                            .selection_mode(MenuItemSelectionMode::Radio)
                            .on_click(move || selected.set(0));
                        menu_item()
                            .label(I18N.text("ui-item-2"))
                            .leading_icon(filled::CONTENT_COPY_SVG)
                            .trailing_text("⌘C".to_owned())
                            .selected(selected.get() == 1)
                            .selection_mode(MenuItemSelectionMode::Radio)
                            .on_click(move || selected.set(1));
                        menu_item()
                            .label(I18N.text("ui-item-3"))
                            .leading_icon(filled::CHECK_SVG)
                            .selected(selected.get() == 2)
                            .selection_mode(MenuItemSelectionMode::Radio)
                            .on_click(move || selected.set(2));
                    });
                menu_group()
                    .position(MenuPosition::Last)
                    .color_style(style)
                    .children(move || {
                        menu_item()
                            .label(I18N.text("ui-item-4"))
                            .leading_icon(filled::PERSON_SVG)
                            .trailing_icon(filled::ARROW_RIGHT_SVG)
                            .on_click(move || selected.set(3));
                        menu_item()
                            .label(I18N.text("ui-item-5"))
                            .leading_icon(filled::SETTINGS_SVG)
                            .on_click(move || selected.set(4));
                    });
            } else {
                menu_group().color_style(style).children(move || {
                    menu_item()
                        .label(I18N.text("ui-refresh"))
                        .trailing_icon(filled::REFRESH_SVG)
                        .on_click(move || selected.set(0));
                    menu_item()
                        .label(I18N.text("ui-settings"))
                        .trailing_icon(filled::SETTINGS_SVG)
                        .on_click(move || selected.set(1));
                    menu_item()
                        .label(I18N.text("ui-help"))
                        .trailing_icon(filled::HELP_OUTLINE_SVG)
                        .on_click(move || selected.set(2));
                    menu_item()
                        .label(I18N.text("ui-more"))
                        .trailing_icon(filled::MORE_VERT_SVG)
                        .on_click(move || selected.set(3));
                });
            }
        });
}

#[tessera]
fn menu_tile(grouped: bool, style: MenuColorStyle, number: usize) {
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            menu_sample(grouped, style);
            gap(Dp(24.0));
            guides::badge(number);
        });
}

#[tessera]
fn menu_sample(grouped: bool, style: MenuColorStyle) {
    let selected = remember(|| 2_usize);
    column()
        .modifier(Modifier::new().width(Dp(216.0)))
        .children(move || {
            menu_group()
                .color_style(style)
                .position(if grouped {
                    MenuPosition::First
                } else {
                    MenuPosition::Standalone
                })
                .children(move || {
                    for (index, label) in ["ui-item-1", "ui-item-2", "ui-item-3"]
                        .into_iter()
                        .enumerate()
                    {
                        menu_item()
                            .label(I18N.text(label))
                            .selected(selected.get() == index)
                            .selection_mode(MenuItemSelectionMode::Radio)
                            .close_on_click(false)
                            .leading_icon(if index == 0 {
                                filled::VISIBILITY_SVG
                            } else if index == 1 {
                                filled::CONTENT_COPY_SVG
                            } else {
                                filled::EDIT_SVG
                            })
                            .trailing_text(I18N.text(if index == 1 { "⌘C" } else { "" }))
                            .on_click(move || selected.set(index));
                    }
                    if !grouped {
                        horizontal_divider();
                        menu_item()
                            .label(I18N.text("ui-item-4"))
                            .leading_icon(filled::CLOUD_SVG)
                            .trailing_icon(filled::ARROW_RIGHT_SVG)
                            .on_click(move || selected.set(3));
                    }
                });
            if grouped {
                menu_group()
                    .position(MenuPosition::Last)
                    .color_style(style)
                    .children(move || {
                        menu_item()
                            .label(I18N.text("ui-item-4"))
                            .leading_icon(filled::CLOUD_SVG)
                            .trailing_icon(filled::ARROW_RIGHT_SVG)
                            .on_click(move || selected.set(3));
                    });
            }
        });
}

#[tessera]
fn launch() {
    let menu = remember(MenuController::new);
    let anchor = remember(tessera_ui::LayoutCoordinates::default);
    let notice = remember(|| "ui-choose-an-action".to_owned());
    boxed()
        .modifier(Modifier::new().fill_max_width().height(Dp(280.0)))
        .children(move || {
            menu_provider()
                .controller(menu)
                .grouped(true)
                .window_anchor(anchor)
                .main_content(move || {
                    column().children(move || {
                        button()
                            .outlined()
                            .modifier(Modifier::new().on_layout_coordinates(move |coordinates| {
                                anchor.set_if_changed(coordinates);
                            }))
                            .on_click(move || menu.with_mut(MenuController::toggle))
                            .child(move || {
                                text(I18N.text("ui-open-menu"));
                            });
                        gap(Dp(16.0));
                        prose(notice.get());
                    });
                })
                .menu_content(move |menu| {
                    menu_group()
                        .menu(menu)
                        .position(MenuPosition::First)
                        .children(move || {
                            menu_item()
                                .menu(menu)
                                .label(I18N.text("ui-preview"))
                                .leading_icon(filled::VISIBILITY_SVG)
                                .on_click(move || notice.set("ui-preview-selected".into()));
                            menu_item()
                                .menu(menu)
                                .label(I18N.text("ui-share"))
                                .leading_icon(filled::SHARE_SVG)
                                .submenu_content(move |menu| {
                                    menu_item()
                                        .menu(menu)
                                        .label(I18N.text("ui-copy-link"))
                                        .leading_icon(filled::LINK_SVG)
                                        .on_click(move || notice.set("ui-link-copied".into()));
                                    menu_item()
                                        .menu(menu)
                                        .label(I18N.text("ui-email"))
                                        .leading_icon(filled::EMAIL_SVG)
                                        .on_click(move || notice.set("ui-email-selected".into()));
                                });
                        });
                    menu_group()
                        .menu(menu)
                        .position(MenuPosition::Last)
                        .children(move || {
                            menu_item()
                                .menu(menu)
                                .label(I18N.text("ui-delete"))
                                .leading_icon(filled::DELETE_SVG)
                                .on_click(move || notice.set("ui-delete-selected".into()));
                        });
                });
        });
}

#[tessera]
fn menu_figure(anatomy: bool, measurements: bool, style: MenuColorStyle) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(480.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(watch(geometry, "container").width(Dp(240.0)))
                .children(move || {
                    menu_group()
                        .position(MenuPosition::First)
                        .color_style(style)
                        .children(move || {
                            boxed()
                                .modifier(watch(geometry, "first").fill_max_width())
                                .children(move || {
                                    menu_item()
                                        .label(I18N.text("ui-item-1"))
                                        .leading_icon(filled::VISIBILITY_SVG)
                                        .selected(selected.get() == 1)
                                        .selection_mode(MenuItemSelectionMode::Radio)
                                        .on_click(move || selected.set(1));
                                });
                            boxed()
                                .modifier(watch(geometry, "second").fill_max_width())
                                .children(move || {
                                    menu_item()
                                        .label(I18N.text("ui-item-2"))
                                        .leading_icon(filled::CONTENT_COPY_SVG)
                                        .trailing_icon(filled::ARROW_RIGHT_SVG)
                                        .selected(selected.get() == 2)
                                        .selection_mode(MenuItemSelectionMode::Radio)
                                        .on_click(move || selected.set(2));
                                });
                            if anatomy {
                                boxed()
                                    .modifier(watch(geometry, "divider").fill_max_width())
                                    .children(|| {
                                        horizontal_divider();
                                    });
                            }
                            boxed()
                                .modifier(watch(geometry, "third").fill_max_width())
                                .children(move || {
                                    menu_item()
                                        .label(I18N.text("ui-item-3"))
                                        .leading_icon(filled::EDIT_SVG)
                                        .trailing_icon(filled::ARROW_RIGHT_SVG)
                                        .on_click(move || selected.set(3));
                                });
                        });
                    menu_group()
                        .position(MenuPosition::Last)
                        .color_style(style)
                        .children(move || {
                            boxed()
                                .modifier(
                                    watch(geometry, "label")
                                        .fill_max_width()
                                        .height(Dp(32.0))
                                        .padding_symmetric(Dp(16.0), Dp::ZERO),
                                )
                                .alignment(Alignment::CenterStart)
                                .children(move || {
                                    text(I18N.text("ui-label-text")).style(type_style(14.0, 20.0));
                                });
                            boxed()
                                .modifier(watch(geometry, "fourth").fill_max_width())
                                .children(move || {
                                    menu_item()
                                        .label(I18N.text("ui-item-4"))
                                        .supporting_text(I18N.text("ui-supporting-text"))
                                        .leading_icon(filled::CLOUD_SVG)
                                        .trailing_text("⌘C".to_owned())
                                        .on_click(move || selected.set(4));
                                });
                        });
                });
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    menu_annotations(geometry, anatomy, measurements);
                    if anatomy && let Some(third) = relative(&geometry.get(), "third") {
                        badge_with_content()
                            .container_color(scheme.tertiary_container)
                            .content_color(scheme.on_tertiary_container)
                            .modifier(Modifier::new().offset(
                                Dp(third.x + third.w - 80.0),
                                Dp(third.y + third.h / 2.0 - 9.0),
                            ))
                            .content(move || {
                                text(I18N.text("ui-new")).style(type_style(12.0, 16.0));
                            });
                    }
                });
        });
    if anatomy {
        gap(Dp(16.0));
        guides::legend(strings(&[
            "ui-menu-item",
            "ui-leading-icon-optional",
            "ui-menu-item-text",
            "ui-trailing-icon-optional",
            "ui-badge-optional",
            "ui-trailing-text-optional",
            "ui-container",
            "ui-supporting-text-optional",
            "ui-label-text-optional",
            "ui-gap-optional",
            "ui-divider-optional",
        ]));
    }
    if measurements {
        gap(Dp(12.0));
        guides::caption("ui-vertical-menu-padding-and-size-measurements");
    }
}

#[tessera]
fn menu_annotations(geometry: State<Geometry>, anatomy: bool, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(c), Some(first), Some(second), Some(third), Some(last), Some(label)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "first"),
        relative(&values, "second"),
        relative(&values, "third"),
        relative(&values, "fourth"),
        relative(&values, "label"),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let mut svg = String::new();
    if anatomy {
        for (n, x, y, tx, ty) in [
            (1, c.x - 64.0, second.y + 8.0, c.x + 4.0, second.y + 24.0),
            (2, c.x + 12.0, c.y - 64.0, c.x + 26.0, first.y + 24.0),
            (3, c.x + 62.0, c.y - 64.0, c.x + 78.0, first.y + 24.0),
            (
                4,
                c.x + c.w + 36.0,
                second.y + 8.0,
                c.x + c.w - 24.0,
                second.y + 24.0,
            ),
            (
                5,
                c.x + c.w + 36.0,
                third.y + 8.0,
                c.x + c.w - 60.0,
                third.y + 24.0,
            ),
            (
                6,
                c.x + c.w + 36.0,
                last.y + 8.0,
                c.x + c.w - 24.0,
                last.y + 24.0,
            ),
            (
                7,
                c.x + 136.0,
                c.y + c.h + 24.0,
                c.x + 148.0,
                c.y + c.h - 8.0,
            ),
            (8, c.x + 76.0, c.y + c.h + 24.0, c.x + 88.0, last.y + 34.0),
            (9, c.x - 64.0, label.y, c.x + 24.0, label.y + 16.0),
            (10, c.x - 64.0, label.y - 32.0, c.x + 4.0, label.y),
            (11, c.x - 64.0, third.y - 16.0, c.x + 4.0, third.y),
        ] {
            if (tx - x).abs() > (ty - y).abs() {
                dotted(&mut svg, x + 12.0, ty, tx - x - 12.0, false, "#49454f");
            } else {
                dotted(&mut svg, tx, y + 16.0, ty - y - 16.0, true, "#49454f");
            }
            annotations::marker(n, Dp(x), Dp(y));
        }
    }
    if measurements {
        svg.push_str(&format!("<rect x='{}' y='{}' width='12' height='{}' fill='#ee7da4' fill-opacity='.3'/><rect x='{}' y='{}' width='{}' height='2' fill='#ee7da4' fill-opacity='.5'/>",c.x,c.y,c.h,c.x,c.y+first.h,c.w));
        for (label, x, y) in [
            ("16", c.x + c.w + 48.0, c.y),
            ("48", c.x + c.w + 48.0, second.y + 16.0),
            ("20", c.x - 52.0, third.y + 16.0),
            ("12", c.x - 52.0, second.y + 16.0),
            ("20", c.x + c.w + 48.0, third.y + 16.0),
            ("8", c.x + c.w + 48.0, label.y - 16.0),
            ("2", c.x - 36.0, label.y - 8.0),
            ("32", c.x - 48.0, label.y + 10.0),
            ("12", c.x - 2.0, c.y + c.h + 24.0),
            ("8", c.x + c.w - 42.0, c.y + c.h + 24.0),
            ("12", c.x + c.w - 8.0, c.y + c.h + 24.0),
        ] {
            annotations::dimension(label.into(), Dp(x), Dp(y));
        }
        dotted(
            &mut svg,
            c.x + c.w,
            second.y + second.h / 2.0,
            44.0,
            false,
            "#ee7da4",
        );
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn colors() {
    prose(
        "ui-color-values-are-implemented-through-design-tokens-for-designers-this-means-working-with-c",
    );
    gap(Dp(16.0));
    prose("ui-menus-have-two-color-mappings");
    gap(Dp(8.0));
    guides::bullets(strings(&[
        "ui-standard-surface-based",
        "ui-vibrant-tertiary-based",
    ]));
    prose(
        "ui-these-mappings-provide-options-for-lower-or-higher-visual-emphasis-vibrant-menus-are-more",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                menu_tile(true, MenuColorStyle::Standard, 1);
                menu_tile(true, MenuColorStyle::Vibrant, 2);
            });
    }));
    gap(Dp(16.0));
    guides::legend(strings(&[
        "ui-standard-color-scheme",
        "ui-vibrant-color-scheme",
    ]));
    gap(Dp(32.0));
    for style in [MenuColorStyle::Standard, MenuColorStyle::Vibrant] {
        subheading(
            if style == MenuColorStyle::Standard {
                "ui-standard-colors"
            } else {
                "ui-vibrant-colors"
            }
            .into(),
        );
        gap(Dp(16.0));
        row()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                for dark in [false, true] {
                    boxed()
                        .modifier(Modifier::new().weight(1.0))
                        .children(move || {
                            material_theme()
                                .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                                .child(move || {
                                    menu_color_figure(style, dark);
                                });
                        });
                }
            });
        gap(Dp(16.0));
        guides::caption("ui-vertical-menus-color-roles-used-for-light-and-dark-themes");
        gap(Dp(8.0));
        guides::legend(
            menu_color_roles(style)
                .iter()
                .map(|role| (*role).to_owned())
                .collect(),
        );
        gap(Dp(32.0));
    }
}

const COLOR_ROWS: [&str; 5] = [
    "color-row-1",
    "color-row-2",
    "color-row-3",
    "color-row-4",
    "color-row-5",
];

fn menu_color_roles(style: MenuColorStyle) -> [&'static str; 11] {
    if style == MenuColorStyle::Standard {
        [
            "On surface variant",
            "On surface",
            "ui-on-surface-state-layer",
            "Surface container low",
            "On surface variant",
            "On surface variant",
            "ui-tertiary-container-selected",
            "ui-on-tertiary-container-selected",
            "On surface variant",
            "On surface variant",
            "ui-on-tertiary-container-selected",
        ]
    } else {
        [
            "On tertiary container",
            "On tertiary container",
            "ui-on-tertiary-container-state-layer",
            "Tertiary container",
            "On tertiary container",
            "On tertiary container",
            "ui-tertiary-selected",
            "ui-on-tertiary-selected",
            "On tertiary container",
            "On tertiary container",
            "ui-on-tertiary-selected",
        ]
    }
}

#[tessera]
fn menu_color_figure(style: MenuColorStyle, dark: bool) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 3_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let mode = if dark { "ui-dark" } else { "ui-light" };
    let emphasis = if style == MenuColorStyle::Standard {
        "ui-standard"
    } else {
        "ui-vibrant"
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(536.0))
                .background(scheme.surface)
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(I18N.format(
                        "ui-value-value-menu-color-roles",
                        [
                            ("v0", I18N.text(&mode).into()),
                            ("v1", I18N.text(&emphasis).into()),
                        ],
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            column()
                .modifier(watch(geometry, "color-container").width(Dp(210.0)))
                .children(move || {
                    menu_group()
                        .position(MenuPosition::First)
                        .color_style(style)
                        .children(move || {
                            for index in 0..4 {
                                boxed()
                                    .modifier(watch(geometry, COLOR_ROWS[index]).fill_max_width())
                                    .children(move || {
                                        let item = menu_item()
                                            .label(I18N.format(
                                                "ui-item-value",
                                                [("v0", (index + 1).into())],
                                            ))
                                            .selection_mode(MenuItemSelectionMode::Radio)
                                            .selected(selected.get() == index)
                                            .color_style(style)
                                            .close_on_click(false)
                                            .on_click(move || selected.set(index));
                                        match index {
                                            0 => {
                                                item.leading_icon(filled::VISIBILITY_SVG);
                                            }
                                            1 => {
                                                item.leading_icon(filled::CONTENT_COPY_SVG)
                                                    .trailing_text("⌘C".to_owned());
                                            }
                                            2 => {
                                                item.leading_icon(filled::EDIT_SVG)
                                                    .trailing_icon(filled::ARROW_RIGHT_SVG);
                                            }
                                            _ => {}
                                        }
                                    });
                            }
                        });
                    menu_group()
                        .position(MenuPosition::Last)
                        .color_style(style)
                        .children(move || {
                            boxed()
                                .alignment(Alignment::CenterStart)
                                .modifier(
                                    watch(geometry, "color-label")
                                        .fill_max_width()
                                        .height(Dp(32.0))
                                        .padding_symmetric(Dp(12.0), Dp::ZERO),
                                )
                                .children(move || {
                                    text(I18N.text("ui-label-text"))
                                        .style(type_style(14.0, 20.0))
                                        .color(if style == MenuColorStyle::Standard {
                                            scheme.on_surface_variant
                                        } else {
                                            scheme.on_tertiary_container
                                        });
                                });
                            boxed()
                                .modifier(watch(geometry, COLOR_ROWS[4]).fill_max_width())
                                .children(move || {
                                    menu_item()
                                        .label(I18N.text("ui-item-5"))
                                        .supporting_text(I18N.text("ui-supporting-text"))
                                        .trailing_icon(filled::ARROW_RIGHT_SVG)
                                        .selection_mode(MenuItemSelectionMode::Radio)
                                        .selected(selected.get() == 4)
                                        .color_style(style)
                                        .close_on_click(false)
                                        .on_click(move || selected.set(4));
                                });
                        });
                });
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    menu_color_annotations(
                        style,
                        dark,
                        selected.get(),
                        geometry.with(annotations::relative_geometry),
                    );
                });
        });
}

#[tessera]
fn menu_color_annotations(style: MenuColorStyle, dark: bool, selected: usize, values: Geometry) {
    let (
        Some(root),
        Some(container),
        Some(first),
        Some(second),
        Some(last),
        Some(label),
        Some(active),
    ) = (
        values.get("figure"),
        relative(&values, "color-container"),
        relative(&values, COLOR_ROWS[0]),
        relative(&values, COLOR_ROWS[1]),
        relative(&values, COLOR_ROWS[4]),
        relative(&values, "color-label"),
        relative(&values, COLOR_ROWS[selected]),
    )
    else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let roles = menu_color_roles(style);
    let tone = if dark { "#cac4d0" } else { "#49454f" };
    let mut svg = String::new();
    for (index, (x, y, tx, ty)) in [
        (
            first.x + 14.0,
            container.y - 80.0,
            first.x + 26.0,
            first.y + 24.0,
        ),
        (
            first.x + 53.0,
            container.y - 80.0,
            first.x + 65.0,
            first.y + 24.0,
        ),
        (
            second.x + 134.0,
            container.y - 80.0,
            second.x + 146.0,
            second.y + 24.0,
        ),
        (
            container.x + container.w + 38.0,
            container.y - 2.0,
            container.x + container.w - 18.0,
            container.y + 14.0,
        ),
        (
            container.x + container.w + 38.0,
            second.y + 8.0,
            second.x + second.w - 22.0,
            second.y + 24.0,
        ),
        (
            container.x + container.w + 38.0,
            last.y + 12.0,
            last.x + last.w - 24.0,
            last.y + 28.0,
        ),
        (
            active.x + 122.0,
            container.y + container.h + 28.0,
            active.x + 134.0,
            active.y + active.h - 12.0,
        ),
        (
            active.x + 77.0,
            container.y + container.h + 28.0,
            active.x + 89.0,
            active.y + 24.0,
        ),
        (
            last.x + 27.0,
            container.y + container.h + 28.0,
            last.x + 39.0,
            last.y + 40.0,
        ),
        (
            container.x - 64.0,
            label.y + 2.0,
            label.x + 24.0,
            label.y + 18.0,
        ),
        (
            container.x - 64.0,
            active.y + 8.0,
            active.x + 26.0,
            active.y + 24.0,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if (tx - x).abs() > (ty - y).abs() {
            dotted(
                &mut svg,
                (x + 12.0).min(tx),
                ty,
                (tx - x - 12.0).abs(),
                false,
                tone,
            );
        } else {
            dotted(
                &mut svg,
                tx,
                (y + 16.0).min(ty),
                (ty - y - 16.0).abs(),
                true,
                tone,
            );
        }
        menu_color_marker(index + 1, roles[index].into(), Dp(x), Dp(y));
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn menu_color_marker(number: usize, role: String, x: Dp, y: Dp) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0), Dp(32.0))
                .background_with_shape(scheme.on_surface, Shape::rounded_rectangle(Dp(16.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(I18N.format(
                        "ui-menu-color-role-value-value",
                        [("v0", (number).into()), ("v1", I18N.text(&role).into())],
                    )),
                    ..Default::default()
                }),
        )
        .children(move || {
            text((number).to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.surface);
        });
}

#[tessera]
fn states() {
    prose(
        "ui-states-are-visual-representations-used-to-communicate-the-status-of-a-component-or-an-inte",
    );
    gap(Dp(8.0));
    prose(
        "ui-shape-morphing-in-vertical-menus-creates-an-expressive-active-state-as-focus-moves-between",
    );
    gap(Dp(24.0));
    guides::states_figure(false, false);
}

#[tessera]
fn tokens() {
    prose("ui-browse-the-component-elements-attributes-tokens-and-their-values");
    gap(Dp(16.0));
    spec_table(
        strings(&["ui-token", "ui-value"]),
        vec![
            strings(&["ui-menu-gap", "2dp"]),
            strings(&["ui-menu-group-padding", "2dp"]),
            strings(&["ui-menu-container-elevation", "ui-level-2-3dp"]),
            strings(&["ui-menu-item-height", "44dp"]),
            strings(&["ui-menu-item-top-space", "8dp"]),
            strings(&["ui-menu-item-bottom-space", "8dp"]),
            strings(&["ui-menu-item-leading-space", "16dp"]),
            strings(&["ui-menu-item-trailing-space", "16dp"]),
            strings(&["ui-menu-item-between-space", "12dp"]),
            strings(&["ui-menu-item-leading-icon-size", "20dp"]),
            strings(&["ui-menu-item-trailing-icon-size", "20dp"]),
        ],
    );
}

#[tessera]
fn keyboard() {
    spec_table(
        strings(&["ui-keys", "ui-actions"]),
        vec![
            strings(&["ui-tab", "ui-focus-lands-on-menu"]),
            strings(&[
                "ui-space-or-enter",
                "ui-for-closed-menus-opens-menu-or-submenu-for-open-menus-selects-a-menu-item",
            ]),
            strings(&[
                "ui-up-and-down-arrows",
                "ui-for-closed-menus-opens-menu-for-open-menus-moves-focus-to-the-next-item",
            ]),
            strings(&["ui-left-and-right-arrows", "ui-opens-or-closes-a-submenu"]),
            strings(&[
                "ui-letters",
                "ui-focus-moves-to-the-next-menu-item-starting-with-letter",
            ]),
            strings(&["ui-escape", "ui-closes-menu"]),
        ],
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use std::time::Duration;

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    };

    use super::*;

    #[tessera]
    fn fixture() {
        material_theme().child(|| {
            launch();
        });
    }

    fn click(renderer: &mut HeadlessRenderer, label: &str) {
        let node = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| crate::has_label(&node, label))
            .unwrap_or_else(|| panic!("missing {label}"))
            .1;
        let bounds = node.bounds().unwrap();
        let position = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(20);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn contextual_menu_executes_action_and_closes_its_popup() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 600,
                height: 500,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(4);
        click(&mut renderer, "Open menu");
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| crate::has_label(&node, "Preview")
                    && node.role() == tessera_ui::accesskit::Role::MenuItem)
        );
        click(&mut renderer, "Preview");
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| crate::has_label(&node, "Preview selected"))
        );
        assert!(
            !snapshot
                .nodes
                .iter()
                .any(|(_, node)| crate::has_label(&node, "Preview")
                    && node.role() == tessera_ui::accesskit::Role::MenuItem)
        );
        click(&mut renderer, "Open menu");
        click(&mut renderer, "Share");
        click(&mut renderer, "Copy link");
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| crate::has_label(&node, "Link copied"))
        );
        assert!(!snapshot.nodes.iter().any(|(_, node)| {
            node.role() == tessera_ui::accesskit::Role::MenuItem
                && matches!(
                    node.label(),
                    Some("Preview" | "Share" | "Copy link" | "Email")
                )
        }));
    }
}
