//! Primary and secondary tab specification articles.
//!
//! ## Usage
//!
//! Explore content destinations, scrollable rows and keyboard activation.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    flow_row::flow_row,
    md3::{
        app_bar::{AppBarScrollBehavior, AppBarScrollController, app_bar_scroll_provider},
        badge::{badge, badge_with_content, badged_box},
        button::button,
        icon::icon,
        material_icons::filled,
        modifier::ModifierExt as _,
        scaffold::scaffold,
        scrollable::ScrollableController,
        surface::surface,
        tabs::{TabActivationMode, TabsVariant, tab, tabs},
        text::text,
        theme::{MaterialTheme, material_theme},
    },
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
};
use tessera_foundation::modifier::{ModifierExt as _, Padding, SemanticsArgs};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use crate::res::I18N;

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

const DESTINATIONS: [&str; 5] = [
    "ui-flights",
    "ui-hotels",
    "ui-trains",
    "ui-buses",
    "ui-ferries",
];
const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const SWIPE_PAGES: [(&str, &str); 3] = [
    ("ui-flights", "ui-upcoming-flights-and-saved-routes"),
    ("ui-trips", "ui-itineraries-grouped-by-destination"),
    ("ui-explore", "ui-places-to-visit-near-your-next-stop"),
];

#[tessera]
pub(super) fn tabs_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-tabs",
        "ui-tabs-organize-content-across-different-screens-and-views",
        vec![
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "ui-primary-tabs",
                content: RenderSlot::new(|| {
                    section(true);
                }),
            },
            SpecSection {
                title: "ui-secondary-tabs",
                content: RenderSlot::new(|| {
                    section(false);
                }),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    )
    .route_optional(route)
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
                    content: RenderSlot::new(guidelines_usage),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(anatomy),
                },
                SpecSection {
                    title: "ui-choosing-the-tab-variant",
                    content: RenderSlot::new(choosing_variant),
                },
                SpecSection {
                    title: "ui-placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "ui-responsive-layout",
                    content: RenderSlot::new(responsive),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(behavior),
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
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "ui-initial-focus",
                    content: RenderSlot::new(initial_focus),
                },
                SpecSection {
                    title: "ui-keyboard-navigation",
                    content: RenderSlot::new(keyboard_navigation),
                },
                SpecSection {
                    title: "ui-labeling-elements",
                    content: RenderSlot::new(labelling),
                },
            ],
        },
    ]);
}

#[tessera]
fn paragraph(value: String) {
    text(I18N.text(value))
        .style(type_style(16.0, 24.0))
        .modifier(Modifier::new().fill_max_width());
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text(I18N.text(value))
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
                text(format!("•  {}", I18N.text(item))).style(type_style(16.0, 24.0));
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
                    text((number).to_string())
                        .style(type_style(12.0, 16.0))
                        .color(scheme.inverse_on_surface);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text(I18N.text(&(label))).style(type_style(16.0, 24.0));
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
                    text(I18N.text(&(mark)))
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text(I18N.text(&(note)))
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
                column()
                    .modifier(Modifier::new().weight(1.0))
                    .children(move || {
                        guidance_card(true, good.clone(), good_figure);
                    });
                spacer().modifier(Modifier::new().width(Dp(24.0)));
                column()
                    .modifier(Modifier::new().weight(1.0))
                    .children(move || {
                        guidance_card(false, bad.clone(), bad_figure);
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
    paragraph("ui-use-tabs-to-group-content-into-helpful-categories".into());
    gap(Dp(16.0));
    bullets(vec![
        "ui-two-variants-primary-and-secondary".into(),
        "ui-primary-tabs-display-main-content-destinations-at-the-top-of-the-screen-often-under-a-top".into(),
        "ui-secondary-tabs-display-related-content-within-a-content-area-always-placed-below-primary-t".into(),
        "ui-tabs-can-horizontally-scroll-so-a-ui-can-have-as-many-tabs-as-needed".into(),
        "ui-place-tabs-next-to-each-other-as-peers".into(),
    ]);
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("ui-primary-tabs");
    gap(Dp(24.0));
    native_tabs_figure(false, false, false, 0, 2);
    gap(Dp(8.0));
    caption("ui-secondary-tabs");
}

#[tessera]
fn guidelines_usage() {
    paragraph(
        "ui-tabs-organize-groups-of-related-content-that-are-at-the-same-level-of-hierarchy".into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("ui-tab-labels-can-include-icons-and-text-text-labels-should-be-short");
    gap(Dp(32.0));
    subheading("ui-types".into());
    gap(Dp(16.0));
    paragraph("ui-there-are-two-types-of-tabs".into());
    gap(Dp(8.0));
    legend_list(vec!["ui-primary-tabs".into(), "ui-secondary-tabs".into()]);
    gap(Dp(24.0));
    native_tabs_figure(true, false, false, 0, 3);
    gap(Dp(24.0));
    native_tabs_figure(false, false, false, 0, 2);
    gap(Dp(24.0));
    paragraph(
        "ui-primary-tabs-are-placed-at-the-top-of-the-content-pane-under-an-app-bar-they-display-the-m"
            .into(),
    );
    gap(Dp(32.0));
    subheading("ui-related-content".into());
    gap(Dp(16.0));
    paragraph(
        "ui-use-tabs-to-group-content-into-clearly-defined-sets-tabs-shouldn-t-be-used-for-content-tha"
            .into(),
    );
    gap(Dp(16.0));
    do_dont(
        "ui-use-tabs-to-categorize-related-groups-of-content-into-clearly-defined-sets",
        "ui-don-t-use-tabs-to-move-through-sequential-content-that-needs-to-be-read-in-a-particular-or",
    )
    .good_figure(|| {
        mock_label_tabs(vec!["ui-flights".into(), "ui-trips".into(), "ui-explore".into()]);
    })
    .bad_figure(|| {
        mock_label_tabs(vec!["ui-step-1".into(), "ui-step-2".into(), "ui-step-3".into()]);
    });
}

#[tessera]
fn anatomy() {
    native_tabs_figure(true, true, false, 3, 3);
    gap(Dp(24.0));
    legend_list(vec![
        "ui-container".into(),
        "ui-icon-optional".into(),
        "ui-badge-optional".into(),
        "ui-label".into(),
        "ui-divider".into(),
        "ui-active-indicator".into(),
    ]);
    gap(Dp(32.0));
    subheading("ui-container".into());
    gap(Dp(16.0));
    paragraph(
        "ui-the-container-holds-multiple-tabs-which-can-be-fixed-or-scrollable-it-spans-the-full-width"
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, false, false, 0, 3);
    gap(Dp(8.0));
    caption("ui-the-container-is-the-area-that-contains-the-tabs-directly-under-the-title-above");
    gap(Dp(32.0));
    subheading("ui-icon-optional".into());
    gap(Dp(16.0));
    paragraph(
        "ui-icons-should-be-simple-and-recognizable-and-clearly-communicate-the-kind-of-content-a-tab"
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("ui-tabs-can-use-a-combination-of-labels-and-icons");
    gap(Dp(16.0));
    do_dont(
        "ui-use-globally-recognized-icons-when-using-icons-alone",
        "ui-don-t-use-icons-and-labels-on-only-some-tabs-keep-the-treatment-consistent-across-the-set",
    )
    .good_figure(|| {
        mock_icon_tabs(vec![false, false, false]);
    })
    .bad_figure(|| {
        mock_icon_tabs(vec![true, true, false]);
    });
    gap(Dp(32.0));
    subheading("ui-label".into());
    gap(Dp(16.0));
    paragraph(
        "ui-labels-should-clearly-and-succinctly-describe-the-content-of-a-tab-and-fit-on-a-single-row"
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(false, false, false, 0, 2);
    gap(Dp(8.0));
    caption(
        "ui-tab-labels-should-be-short-and-succinct-there-should-be-a-clear-relationship-to-the-title",
    );
    gap(Dp(24.0));
    paragraph(
        "ui-scrollable-tabs-offset-the-first-tab-52dp-from-the-leading-edge-and-the-width-of-each-tab"
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, false, true, 0, 5);
    gap(Dp(16.0));
    do_dont(
        "ui-offset-the-first-scrollable-tab-52dp-from-the-leading-edge",
        "ui-don-t-truncate-labels-unless-required",
    )
    .good_figure(|| {
        mock_label_tabs(vec![
            "ui-akita".into(),
            "ui-alaskan-malamute".into(),
            "ui-australian-shepherd".into(),
            "ui-azawakh".into(),
        ])
        .scrollable(true);
    })
    .bad_figure(|| {
        mock_label_tabs(vec![
            "ui-akita".into(),
            "ui-alaskan-malamute".into(),
            "ui-australian-shepherd".into(),
            "ui-azawakh".into(),
        ])
        .width(Dp(240.0));
    });
    gap(Dp(32.0));
    subheading("ui-badges-optional".into());
    gap(Dp(16.0));
    paragraph(
        "ui-badges-show-notifications-or-updates-related-to-the-content-of-one-tab-limit-badge-text-to"
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("ui-badges-are-used-to-highlight-notifications-related-to-tab-specific-content");
    gap(Dp(32.0));
    subheading("ui-active-indicator".into());
    gap(Dp(16.0));
    paragraph(
        "ui-the-active-indicator-is-an-underline-combined-with-a-color-change-of-the-active-tab-s-text"
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("ui-an-underline-and-color-change-differentiate-an-active-tab-from-the-inactive-ones");
}

#[tessera]
fn choosing_variant() {
    paragraph(
        "ui-use-primary-tabs-when-only-one-set-of-tabs-is-needed-use-secondary-tabs-when-a-screen-need"
            .into(),
    );
    gap(Dp(32.0));
    nested_tabs();
    gap(Dp(8.0));
    caption(
        "ui-tabs-can-be-joined-with-components-like-app-bars-embedded-in-a-specific-ui-region-or-neste",
    );
}

#[tessera]
fn placement() {
    paragraph(
        "ui-tabs-are-displayed-in-a-single-row-with-each-tab-connected-to-the-content-it-represents-as"
            .into(),
    );
    gap(Dp(32.0));
    tab_placement();
    gap(Dp(8.0));
    caption("ui-secondary-tabs-are-found-within-other-content-to-assist-users-with-greater-detail");
}

#[tessera]
fn responsive() {
    paragraph(
        "ui-the-maximum-width-of-fixed-tabs-is-set-by-the-widest-tab-fixed-tabs-use-a-fluid-margin-and"
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, false, false, 0, 4);
    gap(Dp(8.0));
    caption("ui-tabs-can-grow-in-width-in-relation-to-the-number-of-items-contained-within");
}

#[tessera]
fn behavior() {
    subheading("ui-states".into());
    gap(Dp(16.0));
    paragraph(
        "ui-tabs-are-enabled-with-one-active-tab-each-tab-can-also-show-hover-focused-and-pressed-stat"
            .into(),
    );
    gap(Dp(24.0));
    state_matrix(true);
    gap(Dp(8.0));
    caption("ui-active-hover-focused-and-pressed-states");
    gap(Dp(32.0));
    subheading("ui-fixed-tabs".into());
    gap(Dp(16.0));
    paragraph(
        "ui-fixed-tabs-display-all-tabs-on-one-screen-which-suits-quick-switching-between-related-cont"
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, false, false, 0, 3);
    gap(Dp(8.0));
    caption("ui-fixed-tabs-allow-users-to-see-all-possible-kinds-of-content-available");
    gap(Dp(32.0));
    subheading("ui-tap-a-tab".into()).level(4);
    gap(Dp(16.0));
    paragraph("ui-navigate-to-a-tab-by-tapping-on-it".into());
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("ui-tapping-on-a-tab-directly");
    gap(Dp(32.0));
    subheading("ui-swipe-within-the-content-area".into()).level(4);
    gap(Dp(16.0));
    paragraph(
        "ui-swipe-left-or-right-in-the-content-area-to-move-between-fixed-tabs-use-caution-when-the-co"
            .into(),
    );
    gap(Dp(24.0));
    swipe_tabs(remember(|| 0usize));
    gap(Dp(8.0));
    caption("ui-users-can-swipe-between-fixed-tabs-to-see-related-content-quickly");
    gap(Dp(16.0));
    do_dont(
        "ui-use-gestures-in-different-directions-from-the-tab-swipe-for-content-in-the-area",
        "ui-don-t-place-swipeable-items-in-the-content-area-of-swipeable-tabs",
    )
    .good_figure(|| {
        mock_tabs_with_rows(false);
    })
    .bad_figure(|| {
        mock_tabs_with_rows(true);
    });
    gap(Dp(32.0));
    subheading("ui-scrollable-tabs".into());
    gap(Dp(16.0));
    paragraph(
        "ui-scrollable-tabs-are-used-when-tabs-can-t-fit-on-screen-or-have-longer-labels-and-they-are"
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, false, true, 0, 5);
    gap(Dp(8.0));
    caption("ui-padding-should-remain-the-same-when-using-scrollable-tabs-and-long-labels");
    gap(Dp(32.0));
    subheading("ui-scrolling-content".into());
    gap(Dp(16.0));
    paragraph(
        "ui-when-the-screen-scrolls-tabs-can-stay-fixed-to-the-top-of-the-screen-or-scroll-offscreen-a"
            .into(),
    );
    gap(Dp(24.0));
    scrolling_tabs();
    gap(Dp(8.0));
    caption("ui-tabs-can-be-used-to-create-elevation");
    gap(Dp(8.0));
    caption(
        "ui-scroll-the-list-the-app-bar-and-tabs-collapse-together-and-scrolling-back-up-brings-them-b",
    );
    gap(Dp(16.0));
    do_dont(
        "ui-tabs-can-scroll-offscreen-and-reappear-when-the-user-scrolls-up",
        "ui-don-t-scroll-tabs-behind-an-app-bar-tabs-attached-to-a-component-should-move-as-a-single-u",
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
    paragraph("ui-users-should-be-able-to".into());
    gap(Dp(8.0));
    bullets(vec![
        "ui-undertake-actions-or-invoke-navigation-to-a-new-destination-with-assistive-technology"
            .into(),
        "ui-select-an-action-or-destination-from-an-off-screen-tab-using-assistive-technology"
            .into(),
        "ui-maintain-access-to-primary-actions-when-the-content-is-scrolled".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    subheading("ui-touch".into());
    gap(Dp(16.0));
    paragraph(
        "ui-when-a-user-taps-a-tab-a-touch-ripple-appears-as-interaction-feedback-and-the-selected-ind"
            .into(),
    );
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("ui-touch-tap");
    gap(Dp(32.0));
    subheading("ui-scrollable".into());
    gap(Dp(16.0));
    bullets(vec![
        "ui-scrollable-tabs-are-used-when-tabs-can-t-fit-on-screen-and-are-best-for-browsing-on-touch".into(),
        "ui-swipe-left-or-right-or-use-arrow-or-tab-keys-to-navigate-through-the-tabs".into(),
        "ui-looping-the-tabs-infinitely-is-not-recommended-because-it-traps-screen-reader-users".into(),
        "ui-select-an-individual-tab-with-a-tap-or-with-space-or-enter".into(),
        "ui-horizontally-scrolling-tabs-meet-accessibility-requirements-because-labels-need-room-to-gr".into(),
    ]);
    gap(Dp(24.0));
    native_tabs_figure(true, false, true, 0, 5);
    gap(Dp(8.0));
    caption("ui-scrollable-scrollable-tabs");
    gap(Dp(32.0));
    subheading("ui-cursor".into());
    gap(Dp(16.0));
    paragraph(
        "ui-hovering-a-tab-shows-a-hover-state-as-a-visual-cue-that-it-is-interactive-clicking-an-acti"
            .into(),
    );
    gap(Dp(24.0));
    state_matrix(true);
    gap(Dp(8.0));
    caption("ui-cursor-hover-click");
    gap(Dp(32.0));
    subheading("Keyboard/Switch".into());
    gap(Dp(16.0));
    bullets(vec![
        "ui-a-focus-indicator-appears-when-the-user-tabs-into-the-set".into(),
        "ui-space-or-enter-on-the-selected-tab-navigates-to-a-new-destination".into(),
        "ui-within-the-tab-menu-arrow-or-tab-keys-move-through-items-space-or-enter-selects-an-item-an".into(),
    ]);
    gap(Dp(24.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(8.0));
    caption("ui-keyboard-switch-tab-space-enter-arrow");
    gap(Dp(32.0));
    subheading("ui-avoid-applying-density-by-default".into());
    gap(Dp(16.0));
    paragraph(
        "ui-don-t-apply-density-to-tabs-by-default-because-it-lowers-their-targets-below-the-best-prac"
            .into(),
    );
    gap(Dp(16.0));
    paragraph(
        "ui-to-ensure-that-the-density-setting-can-be-easily-reverted-when-it-s-active-keep-every-targ"
            .into(),
    );
}

#[tessera]
fn initial_focus() {
    paragraph(
        "ui-on-arrow-or-tab-in-a-tab-menu-the-active-indicator-appears-on-the-first-interactive-elemen"
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(16.0));
    do_dont(
        "ui-use-arrow-tab-to-navigate-through-items",
        "ui-don-t-use-space-enter-for-navigating-tabs-space-enter-is-only-used-for-completing-actions",
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
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-arrow".into(),
                "ui-focus-lands-on-the-next-available-navigation-destination".into(),
            ],
            vec![
                "ui-space-enter".into(),
                "ui-activates-the-focused-navigation-destination".into(),
            ],
            vec![
                "ui-arrow".into(),
                "ui-allows-navigation-through-menu-items".into(),
            ],
        ],
    );
}

#[tessera]
fn labelling() {
    paragraph(
        "ui-when-the-visible-ui-text-is-ambiguous-or-there-is-no-visible-ui-text-accessibility-labels"
            .into(),
    );
    gap(Dp(32.0));
    native_tabs_figure(true, true, false, 0, 3);
    gap(Dp(16.0));
    spec_table(
        vec!["ui-element".into(), "ui-label".into(), "ui-role".into()],
        vec![vec![
            "ui-video-tab".into(),
            "ui-video-format-media-content".into(),
            "ui-tab".into(),
        ]],
    );
    gap(Dp(8.0));
    caption(
        "ui-while-the-icon-visually-represents-a-video-camera-the-accessibility-label-for-this-tab-cla",
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
            label: Some(I18N.format("ui-value-tabs-preview", [("v0", I18N.text(&label).into())])),
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
                                .accessibility_label(I18N.text("ui-show-primary-tab-icons"))
                                .on_click(move || icons.set(!icons.get()))
                                .child(move || {
                                    text(I18N.text("ui-icons"));
                                });
                        }
                        button()
                            .tonal()
                            .selected(badge.get())
                            .accessibility_label(
                                I18N.format(
                                    "ui-value-tab-badge",
                                    [("v0", I18N.text(&label).into())],
                                ),
                            )
                            .on_click(move || badge.set(!badge.get()))
                            .child(move || {
                                text(I18N.text("ui-badge"));
                            });
                        button()
                            .tonal()
                            .selected(scrollable.get())
                            .accessibility_label(I18N.format(
                                "ui-value-scrollable-tabs",
                                [("v0", I18N.text(&label).into())],
                            ))
                            .on_click(move || {
                                scrollable.set(!scrollable.get());
                                if !scrollable.get() && selected.get() > 2 {
                                    selected.set(2);
                                }
                            })
                            .child(move || {
                                text(I18N.text("ui-scrollable"));
                            });
                        button()
                            .tonal()
                            .selected(automatic.get())
                            .accessibility_label(I18N.format(
                                "ui-value-automatic-activation",
                                [("v0", I18N.text(&label).into())],
                            ))
                            .on_click(move || automatic.set(!automatic.get()))
                            .child(move || {
                                text(I18N.text("ui-automatic-activation"));
                            });
                        button()
                            .tonal()
                            .selected(!enabled.get())
                            .accessibility_label(I18N.format(
                                "ui-value-disable-hotels",
                                [("v0", I18N.text(&label).into())],
                            ))
                            .on_click(move || enabled.set(!enabled.get()))
                            .child(move || {
                                text(I18N.text("ui-disable-hotels"));
                            });
                        button()
                            .text()
                            .accessibility_label(I18N.format(
                                "ui-value-select-final-tab",
                                [("v0", I18N.text(&label).into())],
                            ))
                            .on_click(move || selected.set(if scrollable.get() { 4 } else { 2 }))
                            .child(move || {
                                text(I18N.text("ui-select-last"));
                            });
                    });
                gap(Dp(24.0));
            }
            tabs(selected.get())
                .on_select(move |index| selected.set(index))
                .variant(if primary {
                    TabsVariant::Primary
                } else {
                    TabsVariant::Secondary
                })
                .scrollable(scrollable.get())
                .activation_mode(if automatic.get() {
                    TabActivationMode::Automatic
                } else {
                    TabActivationMode::Manual
                })
                .min_tab_height(Dp(if icons.get() && primary { 64.0 } else { 48.0 }))
                .modifier(Modifier::new().fill_max_width())
                .content(move |tab_row| {
                    let visible = if scrollable.get() { 5 } else { 3 };
                    for (index, name) in DESTINATIONS.into_iter().take(visible).enumerate() {
                        let available = index != 1 || enabled.get();
                        if index == 1 && badge.get() {
                            tab(tab_row)
                                .enabled(available)
                                .accessibility_label(I18N.format(
                                    "ui-value-3-new-items",
                                    [("v0", I18N.text(&name).into())],
                                ))
                                .on_click(move || count.set(count.get() + 1))
                                .title(move || {
                                    badge_title(name, icons.get() && primary, index);
                                });
                        } else {
                            let tab = tab(tab_row)
                                .label(I18N.text(name))
                                .enabled(available)
                                .accessibility_label(I18N.text(&name))
                                .on_click(move || count.set(count.get() + 1));
                            if icons.get() && primary {
                                tab.icon(destination_asset(index));
                            }
                        }
                    }
                });
            gap(Dp(24.0));
            text(I18N.format(
                "ui-value-content-value",
                [
                    ("v0", I18N.text(&prefix).into()),
                    ("v1", I18N.text(&(DESTINATIONS[selected.get()])).into()),
                ],
            ))
            .style(type_style(22.0, 30.0));
            gap(Dp(8.0));
            text(I18N.text(
                [
                    "ui-search-flights-and-manage-upcoming-journeys",
                    "ui-review-hotel-bookings-and-saved-stays",
                    "ui-find-train-departures-and-booked-seats",
                    "ui-check-bus-routes-and-ticket-details",
                    "ui-view-ferry-crossings-and-boarding-details",
                ][selected.get()],
            ))
            .style(type_style(16.0, 24.0));
            text(I18N.format(
                "ui-value-activations-value",
                [
                    ("v0", I18N.text(&prefix).into()),
                    ("v1", (count.get()).into()),
                ],
            ))
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
                    .badge(move || {
                        badge_with_content().content(move || {
                            text("3");
                        });
                    });
                text(I18N.text(&(label)));
            });
    } else {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                text(I18N.text(&(label)));
                spacer().modifier(Modifier::new().width(Dp(4.0)));
                badge_with_content().content(move || {
                    text("3");
                });
            });
    }
}

#[tessera]
fn section(primary: bool) {
    panel(RenderSlot::new(move || {
        sample(
            if primary {
                "ui-primary"
            } else {
                "ui-secondary"
            },
            primary,
        )
        .controls(true);
    }));
    gap(Dp(32.0));
    native_tabs_figure(primary, primary, false, 1, if primary { 3 } else { 2 });
    gap(Dp(24.0));
    legend_list(if primary {
        vec![
            "ui-container".into(),
            "ui-badge-optional".into(),
            "ui-icon-optional".into(),
            "ui-label".into(),
            "ui-divider".into(),
            "ui-active-indicator".into(),
        ]
    } else {
        vec![
            "ui-container".into(),
            "ui-badge-optional".into(),
            "ui-label".into(),
            "ui-divider".into(),
            "ui-active-indicator".into(),
        ]
    });
    gap(Dp(32.0));
    subheading(if primary {
        "ui-primary-tabs-color".into()
    } else {
        "ui-secondary-tabs-color".into()
    });
    gap(Dp(16.0));
    paragraph("ui-color-values-are-implemented-through-design-tokens".into());
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
        "ui-primary-tabs-states".into()
    } else {
        "ui-secondary-tabs-states".into()
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
        "ui-enabled-hover-focused-and-pressed-states-for-active-and-inactive-destinations-in-light-and",
    );
}

#[tessera]
fn themed_panel(dark: bool, content: RenderSlot) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            surface()
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                        .children(move || {
                            text(I18N.text(if dark { "ui-dark" } else { "ui-light" }))
                                .style(type_style(14.0, 20.0));
                            gap(Dp(16.0));
                            column()
                                .modifier(Modifier::new().fill_max_width())
                                .children_shared(content);
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
                text(I18N.text(if active {
                    "ui-active-destination"
                } else {
                    "ui-inactive-destination"
                }))
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
    let name = ["ui-enabled", "ui-hover", "ui-focused", "ui-pressed"][state];
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
                    text(I18N.text("ui-label"))
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
            text(I18N.text(name)).style(type_style(12.0, 16.0));
        });
}

const TOKEN_GROUPS: [(&str, &[(&str, &str, &str)]); 4] = [
    (
        "ui-enabled",
        &[
            (
                "ui-container-color",
                "md.comp.primary-navigation-tab.container.color",
                "Surface",
            ),
            (
                "ui-container-height",
                "md.comp.primary-navigation-tab.container.height",
                "48dp",
            ),
            (
                "ui-container-elevation",
                "md.comp.primary-navigation-tab.container.elevation",
                "ui-level-0",
            ),
            (
                "ui-container-shape",
                "md.comp.primary-navigation-tab.container.shape",
                "0dp",
            ),
            (
                "ui-icon-and-label-container-height",
                "md.comp.primary-navigation-tab.with-icon-and-label-text.container.height",
                "64dp",
            ),
            (
                "ui-active-indicator-color",
                "md.comp.primary-navigation-tab.active-indicator.color",
                "Primary",
            ),
            (
                "ui-active-indicator-height",
                "md.comp.primary-navigation-tab.active-indicator.height",
                "3dp",
            ),
            (
                "ui-active-indicator-shape",
                "md.comp.primary-navigation-tab.active-indicator.shape",
                "3dp, 3dp, 0, 0",
            ),
            (
                "ui-active-label-text-color",
                "md.comp.primary-navigation-tab.with-label-text.active.label-text.color",
                "Primary",
            ),
            (
                "ui-inactive-label-text-color",
                "md.comp.primary-navigation-tab.with-label-text.inactive.label-text.color",
                "On surface variant",
            ),
            (
                "ui-label-text-font",
                "md.comp.primary-navigation-tab.with-label-text.label-text.font",
                "Title small",
            ),
            (
                "ui-label-text-size",
                "md.comp.primary-navigation-tab.with-label-text.label-text.size",
                "14sp",
            ),
            (
                "ui-label-text-weight",
                "md.comp.primary-navigation-tab.with-label-text.label-text.weight",
                "500",
            ),
            (
                "ui-label-text-line-height",
                "md.comp.primary-navigation-tab.with-label-text.label-text.line-height",
                "20sp",
            ),
            (
                "ui-active-icon-color",
                "md.comp.primary-navigation-tab.with-icon.active.icon.color",
                "Primary",
            ),
            (
                "ui-inactive-icon-color",
                "md.comp.primary-navigation-tab.with-icon.inactive.icon.color",
                "On surface variant",
            ),
            (
                "ui-icon-size",
                "md.comp.primary-navigation-tab.with-icon.icon.size",
                "24dp",
            ),
            (
                "ui-divider-color",
                "md.comp.primary-navigation-tab.divider.color",
                "Outline variant",
            ),
            (
                "ui-divider-height",
                "md.comp.primary-navigation-tab.divider.height",
                "1dp",
            ),
        ],
    ),
    (
        "ui-hovered",
        &[
            (
                "ui-active-state-layer-color",
                "md.comp.primary-navigation-tab.active.hover.state-layer.color",
                "Primary",
            ),
            (
                "ui-inactive-state-layer-color",
                "md.comp.primary-navigation-tab.inactive.hover.state-layer.color",
                "On surface",
            ),
            (
                "ui-state-layer-opacity",
                "md.comp.primary-navigation-tab.hover.state-layer.opacity",
                "0.08",
            ),
            (
                "ui-active-label-text-color",
                "md.comp.primary-navigation-tab.with-label-text.active.hover.label-text.color",
                "Primary",
            ),
            (
                "ui-inactive-label-text-color",
                "md.comp.primary-navigation-tab.with-label-text.inactive.hover.label-text.color",
                "On surface",
            ),
            (
                "ui-active-icon-color",
                "md.comp.primary-navigation-tab.with-icon.active.hover.icon.color",
                "Primary",
            ),
            (
                "ui-inactive-icon-color",
                "md.comp.primary-navigation-tab.with-icon.inactive.hover.icon.color",
                "On surface",
            ),
        ],
    ),
    (
        "ui-focused",
        &[
            (
                "ui-active-state-layer-color",
                "md.comp.primary-navigation-tab.active.focus.state-layer.color",
                "Primary",
            ),
            (
                "ui-inactive-state-layer-color",
                "md.comp.primary-navigation-tab.inactive.focus.state-layer.color",
                "On surface",
            ),
            (
                "ui-state-layer-opacity",
                "md.comp.primary-navigation-tab.focus.state-layer.opacity",
                "0.1",
            ),
            (
                "ui-active-label-text-color",
                "md.comp.primary-navigation-tab.with-label-text.active.focus.label-text.color",
                "Primary",
            ),
            (
                "ui-inactive-label-text-color",
                "md.comp.primary-navigation-tab.with-label-text.inactive.focus.label-text.color",
                "On surface",
            ),
            (
                "ui-active-icon-color",
                "md.comp.primary-navigation-tab.with-icon.active.focus.icon.color",
                "Primary",
            ),
            (
                "ui-inactive-icon-color",
                "md.comp.primary-navigation-tab.with-icon.inactive.focus.icon.color",
                "On surface",
            ),
        ],
    ),
    (
        "ui-pressed-ripple",
        &[
            (
                "ui-active-state-layer-color",
                "md.comp.primary-navigation-tab.active.pressed.state-layer.color",
                "Primary",
            ),
            (
                "ui-inactive-state-layer-color",
                "md.comp.primary-navigation-tab.inactive.pressed.state-layer.color",
                "On surface",
            ),
            (
                "ui-state-layer-opacity",
                "md.comp.primary-navigation-tab.pressed.state-layer.opacity",
                "0.1",
            ),
            (
                "ui-active-label-text-color",
                "md.comp.primary-navigation-tab.with-label-text.active.pressed.label-text.color",
                "Primary",
            ),
            (
                "ui-inactive-label-text-color",
                "md.comp.primary-navigation-tab.with-label-text.inactive.pressed.label-text.color",
                "On surface",
            ),
            (
                "ui-active-icon-color",
                "md.comp.primary-navigation-tab.with-icon.active.pressed.icon.color",
                "Primary",
            ),
            (
                "ui-inactive-icon-color",
                "md.comp.primary-navigation-tab.with-icon.inactive.pressed.icon.color",
                "On surface",
            ),
        ],
    ),
];

#[tessera]
fn tokens() {
    subheading("ui-tabs-primary-navigation".into());
    gap(Dp(24.0));
    for (state, rows) in TOKEN_GROUPS {
        subheading(state.to_string()).level(4);
        gap(Dp(16.0));
        spec_table(
            vec!["ui-element".into(), "ui-token".into(), "ui-value".into()],
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
        "ui-secondary-tabs-48dp-container-1-2-width-tabs-2dp-full-width-indicator-4dp-label-to-badge-s",
    );
    gap(Dp(32.0));
    secondary_measure_figure(true);
    gap(Dp(8.0));
    caption(
        "ui-secondary-tabs-with-inline-icons-24dp-icon-8dp-icon-to-label-spacing-4dp-label-to-badge-sp",
    );
    gap(Dp(32.0));
    badge_legend();
    gap(Dp(32.0));
    paragraph(
        "ui-tabs-are-divided-into-equal-sections-with-labels-and-icons-positioned-vertically-centered"
            .into(),
    );
    gap(Dp(24.0));
    paragraph(
        "ui-primary-tab-active-indicators-are-inset-2dp-on-each-side-have-a-fully-rounded-corner-radiu"
            .into(),
    );
    gap(Dp(32.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-container-height-label-only".into(), "48dp".into()],
            vec!["ui-container-height-icon-label".into(), "64dp".into()],
            vec!["ui-icon-size".into(), "24dp".into()],
            vec!["ui-divider-height".into(), "1dp".into()],
            vec!["ui-primary-indicator-height".into(), "3dp".into()],
            vec!["ui-secondary-indicator-height".into(), "2dp".into()],
            vec!["ui-active-indicator-shape".into(), "3dp, 3dp, 0, 0".into()],
            vec!["ui-active-indicator-minimum-length".into(), "24dp".into()],
            vec!["ui-inline-icon-text-padding".into(), "8dp".into()],
            vec!["ui-inline-text-badge-padding".into(), "4dp".into()],
            vec!["ui-badge-overlap-on-stacked-icon".into(), "6dp".into()],
        ],
    );
}

#[tessera]
fn badge_legend() {
    flow_row()
        .item_spacing(Dp(32.0))
        .line_spacing(Dp(16.0))
        .children(move || {
            for (index, name) in [
                "ui-small",
                "ui-large-single-digit",
                "ui-large-multiple-digits",
                "ui-large-999",
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
                                        text(I18N.text(["", "3", "32", "999+"][index]));
                                    });
                                }
                            });
                        gap(Dp(8.0));
                        text(I18N.text(name)).style(type_style(14.0, 20.0));
                    });
            }
        });
}

#[tessera]
fn swipe_tabs(selected: State<usize>) {
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
            tabs(selected.get())
                .on_select(move |index| selected.set(index))
                .modifier(Modifier::new().fill_max_width())
                .content(move |tab_row| {
                    for (name, _) in SWIPE_PAGES {
                        tab(tab_row).label(I18N.text(name));
                    }
                });
            let (title, detail) = SWIPE_PAGES[selected.get().min(SWIPE_PAGES.len() - 1)];
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(200.0))
                        .draggable(tessera_foundation::modifier::DraggableArgs {
                            on_drag_delta: tessera_ui::CallbackWith::new(
                                move |delta: tessera_foundation::modifier::DragDelta| {
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
                            label: Some(
                                I18N.format("ui-value-content", [("v0", I18N.text(&title).into())]),
                            ),
                            ..Default::default()
                        })
                        .padding_all(Dp(24.0)),
                )
                .children(move || {
                    text(I18N.text(title)).style(type_style(24.0, 32.0));
                    gap(Dp(8.0));
                    text(I18N.text(detail)).style(type_style(16.0, 24.0));
                    gap(Dp(16.0));
                    text(I18N.text("ui-swipe-left-or-right-to-change-tabs"))
                        .color(hint)
                        .style(type_style(14.0, 20.0));
                });
        });
}

fn destination_asset(index: usize) -> tessera_components::md3::material_icons::Asset {
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
        [
            "ui-flights",
            "ui-trips",
            "ui-explore",
            "ui-buy",
            "ui-ferries",
        ][index]
    } else {
        [
            "ui-overview",
            "ui-specifications",
            "ui-details",
            "ui-updates",
            "ui-more",
        ][index]
    }
}

fn figure_asset(primary: bool, index: usize) -> tessera_components::md3::material_icons::Asset {
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
                        .on_layout_size_changed(move |size: tessera_ui::PxSize| {
                            let next = Dp::from_pixels_f32(size.width.to_f32());
                            art_width.set_if_changed(next);
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
                                    tabs(selected.get())
                                        .on_select(move |index| selected.set(index))
                                        .variant(if primary {
                                            TabsVariant::Primary
                                        } else {
                                            TabsVariant::Secondary
                                        })
                                        .scrollable(scrollable)
                                        .edge_padding(Dp(52.0 * scale))
                                        .tab_padding(Dp(16.0 * scale))
                                        .indicator_height(Dp(
                                            if primary { 3.0 } else { 2.0 } * scale
                                        ))
                                        .indicator_min_width(Dp(24.0 * scale))
                                        .min_tab_height(Dp(if icons { 64.0 } else { 48.0 } * scale))
                                        .modifier(Modifier::new().fill_max_width())
                                        .content(move |tab_row| {
                                            for (index, name) in (0..count)
                                                .map(|index| (index, figure_name(primary, index)))
                                            {
                                                tab(tab_row)
                                                    .accessibility_label(I18N.text(
                                                        if index == 0 && unread.get() {
                                                            I18N.format(
                                                                "ui-value-3-new-items",
                                                                [("v0", I18N.text(&name).into())],
                                                            )
                                                        } else {
                                                            name.to_owned()
                                                        },
                                                    ))
                                                    .on_click(move || {
                                                        if index == 0 {
                                                            unread.set(false);
                                                        }
                                                    })
                                                    .title(move || {
                                                        native_tab_title(
                                                            index,
                                                            icons,
                                                            index == 0 && unread.get(),
                                                            geometry,
                                                            scale,
                                                            primary,
                                                            false,
                                                        );
                                                    });
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
            text(I18N.format(
                "ui-figure-content-value",
                [(
                    "v0",
                    I18N.text(&figure_name(primary, selected.get())).into(),
                )],
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
    let color = use_context::<tessera_components::md3::theme::ContentColor>()
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
                text(I18N.text(figure_name(primary, index)))
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
                        text(I18N.text(figure_name(primary, index)))
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
                text("3").style(style);
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
        tabs(primary.get())
            .on_select(move |index| primary.set(index))
            .modifier(Modifier::new().fill_max_width())
            .content(move |tab_row| {
                tab(tab_row).label(I18N.text("ui-travel"));
                tab(tab_row).label(I18N.text("ui-saved"));
            });
        gap(Dp(24.0));
        tabs(secondary.get())
            .on_select(move |index| secondary.set(index))
            .variant(TabsVariant::Secondary)
            .modifier(Modifier::new().fill_max_width())
            .content(move |tab_row| {
                tab(tab_row).label(I18N.text("ui-upcoming"));
                tab(tab_row).label(I18N.text("ui-past"));
            });
        gap(Dp(32.0));
        text(
            I18N.format(
                "ui-value-value-format-25",
                [
                    (
                        "v0",
                        I18N.text(
                            &(if primary.get() == 0 {
                                "ui-travel"
                            } else {
                                "ui-saved"
                            }),
                        )
                        .into(),
                    ),
                    (
                        "v1",
                        I18N.text(
                            &(if secondary.get() == 0 {
                                "ui-upcoming"
                            } else {
                                "ui-past"
                            }),
                        )
                        .into(),
                    ),
                ],
            ),
        )
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
            tessera_components::md3::app_bar::top_app_bar().title(I18N.text("ui-travel"));
            tabs(selected.get())
                .on_select(move |index| selected.set(index))
                .modifier(Modifier::new().fill_max_width())
                .content(move |tab_row| {
                    tab(tab_row).label(I18N.text("ui-flights"));
                    tab(tab_row).label(I18N.text("ui-hotels"));
                    tab(tab_row).label(I18N.text("ui-trains"));
                });
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .weight(1.0)
                        .padding_all(Dp(24.0)),
                )
                .children(move || {
                    text(I18N.text(DESTINATIONS[selected.get()])).style(type_style(24.0, 32.0));
                    gap(Dp(16.0));
                    text(I18N.text("ui-related-content-appears-below-the-tabs"))
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
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .clip_shape(Shape::rounded_rectangle(Dp(16.0))),
                )
                .children(move || {
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
                                .children_shared(figure);
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
                            text(I18N.text(if good { "✓  Do" } else { "ui-don-t" }))
                                .style(type_style(14.0, 20.0))
                                .color(Color::WHITE);
                        });
                });
            gap(Dp(12.0));
            text(I18N.text(&(note)))
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
                    tabs(selected.get())
                        .on_select(move |index| selected.set(index))
                        .scrollable(scrollable.unwrap_or(false))
                        .edge_padding(Dp(52.0))
                        .modifier(Modifier::new().fill_max_width())
                        .content(move |tab_row| {
                            for label in labels.clone() {
                                tab(tab_row).label(I18N.text(label));
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
                    tabs(selected.get())
                        .on_select(move |index| selected.set(index))
                        .min_tab_height(Dp(64.0))
                        .modifier(Modifier::new().fill_max_width())
                        .content(move |tab_row| {
                            for (index, with_label) in labelled.clone().into_iter().enumerate() {
                                let name = figure_name(true, index);
                                if with_label {
                                    tab(tab_row)
                                        .label(I18N.text(name))
                                        .icon(figure_asset(true, index));
                                } else {
                                    tab(tab_row)
                                        .icon(figure_asset(true, index))
                                        .accessibility_label(I18N.text(&name));
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
            for (index, title) in ["ui-lisbon", "ui-kyoto", "ui-oaxaca"]
                .into_iter()
                .enumerate()
            {
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
                                .children(move || {
                                    text(I18N.text("ui-delete"))
                                        .color(Color::WHITE)
                                        .style(type_style(12.0, 16.0));
                                });
                        }
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        text(I18N.text(title)).style(type_style(14.0, 20.0));
                        if swipeable && index == 0 {
                            spacer().modifier(Modifier::new().width(Dp(12.0)));
                            text(I18N.text("ui-swipe-5798"))
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
            mock_label_tabs(vec![
                "ui-flights".into(),
                "ui-trips".into(),
                "ui-explore".into(),
            ]);
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
                                tabs(selected.get())
                                    .on_select(move |index| selected.set(index))
                                    .modifier(Modifier::new().fill_max_width())
                                    .content(move |tab_row| {
                                        tab(tab_row).label(I18N.text("ui-flights"));
                                        tab(tab_row).label(I18N.text("ui-trips"));
                                        tab(tab_row).label(I18N.text("ui-explore"));
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
                            .children(move || {
                                text(I18N.text("ui-travel")).style(type_style(22.0, 28.0));
                            });
                    });
            } else {
                text(I18N.text("ui-scrolled-down-the-app-bar-and-tabs-moved-offscreen-together"))
                    .color(hint)
                    .style(type_style(12.0, 16.0))
                    .modifier(Modifier::new().padding(Padding::symmetric(Dp(16.0), Dp(8.0))));
            }
            for title in ["ui-lisbon", "ui-kyoto", "ui-oaxaca", "ui-bergen"] {
                text(I18N.text(title))
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
                    text(I18N.text(if space_enter {
                        "Space/Enter"
                    } else {
                        "Arrow/Tab"
                    }))
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
                        .on_layout_size_changed(move |size: tessera_ui::PxSize| {
                            let next = Dp::from_pixels_f32(size.width.to_f32());
                            art_width.set_if_changed(next);
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
                                    tabs(selected.get())
                                        .on_select(move |index| selected.set(index))
                                        .variant(TabsVariant::Secondary)
                                        .indicator_height(Dp(2.0 * scale))
                                        .min_tab_height(Dp(48.0 * scale))
                                        .tab_padding(Dp(16.0 * scale))
                                        .modifier(Modifier::new().fill_max_width())
                                        .content(move |tab_row| {
                                            for index in 0..2 {
                                                tab(tab_row)
                                                    .accessibility_label(
                                                        I18N.text(&figure_name(false, index)),
                                                    )
                                                    .title(move || {
                                                        native_tab_title(
                                                            index,
                                                            inline_icons,
                                                            index == 0,
                                                            geometry,
                                                            scale,
                                                            false,
                                                            true,
                                                        );
                                                    });
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
            text(I18N.format(
                "ui-figure-content-value",
                [("v0", I18N.text(&figure_name(false, selected.get())).into())],
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
        "ui-full-width-indicator".into(),
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
    let bar = remember(AppBarScrollController::default);
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
                    .accessibility_label(I18N.text("ui-reset-scrolling-tabs"))
                    .on_click(move || {
                        bar.with_mut(AppBarScrollController::reset);
                        content.with_mut(|content| {
                            content.set_scroll_position(tessera_ui::PxPosition::ZERO)
                        });
                    })
                    .child(move || {
                        text(I18N.text("ui-reset-scroll"));
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
                        .controller(bar)
                        .behavior(AppBarScrollBehavior::EnterAlways)
                        .scroll_controller(content)
                        .child(move || {
                            scaffold()
                                .modifier(Modifier::new().fill_max_size())
                                .top_bar(move || {
                                    let visible =
                                        1.0 - bar.with(AppBarScrollController::collapsed_fraction);
                                    column()
                                        .modifier(Modifier::new().fill_max_width().semantics(
                                            SemanticsArgs {
                                                role: Some(Role::Group),
                                                label: Some(I18N.text("ui-scrolling-tabs-header")),
                                                ..Default::default()
                                            },
                                        ))
                                        .children(move || {
                                            tessera_components::md3::app_bar::top_app_bar()
                                                .title(I18N.text("ui-travel"))
                                                .scroll_controller(bar);
                                            boxed()
                                                .alignment(Alignment::BottomCenter)
                                                .modifier(
                                                    Modifier::new()
                                                        .fill_max_width()
                                                        .height(Dp(48.0 * visible as f64))
                                                        .clip_to_bounds(),
                                                )
                                                .children(move || {
                                                    tabs(selected.get())
                                                        .on_select(move |index| selected.set(index))
                                                        .modifier(Modifier::new().fill_max_width())
                                                        .content(move |tab_row| {
                                                            tab(tab_row)
                                                                .label(I18N.text("ui-flights"));
                                                            tab(tab_row)
                                                                .label(I18N.text("ui-trips"));
                                                            tab(tab_row)
                                                                .label(I18N.text("ui-explore"));
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
                                                    label: Some(
                                                        I18N.text("ui-scrolling-tabs-content"),
                                                    ),
                                                    ..Default::default()
                                                }),
                                        )
                                        .children(move || {
                                            let destination =
                                                ["ui-flights", "ui-trips", "ui-explore"]
                                                    [selected.get()];
                                            for index in 1..=24 {
                                                text(I18N.format(
                                                    "ui-value-item-value",
                                                    [
                                                        ("v0", I18N.text(&destination).into()),
                                                        ("v1", (index).into()),
                                                    ],
                                                ))
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
