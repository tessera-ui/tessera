use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    app_bar::{
        AppBarScrollBehavior, AppBarScrollController, AppBarTitleAlignment, TopAppBarVariant, app_bar,
        app_bar_scroll_provider, top_app_bar,
    },
    boxed::boxed,
    button::button,
    column::column,
    dialog::{DialogController, dialog_provider},
    flow_row::flow_row,
    icon::icon,
    icon_button::{IconButtonColorStyle, icon_button},
    material_icons::{Asset, filled},
    modifier::ModifierExt as _,
    row::row,
    scaffold::scaffold,
    scrollable::ScrollableController,
    search::{SearchBarColors, SearchBarController, search_bar},
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_foundation::modifier::{ModifierExt as _, Padding, SemanticsArgs};
use tessera_ui::{
    Color, Dp, LayoutCoordinates, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    article_helpers::{bullets, caption},
    gap, panel, spec_table, subheading, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);

#[tessera]
pub(super) fn app_bars_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-app-bars",
        "ui-navigate-between-screens-and-expose-relevant-actions",
        vec![
            section("ui-variants", RenderSlot::new(variants)),
            section("ui-configurations", RenderSlot::new(configurations)),
            section("ui-tokens-specs", RenderSlot::new(tokens)),
            section("ui-anatomy", RenderSlot::new(anatomy)),
            section("ui-color", RenderSlot::new(colors)),
            section("ui-measurements", RenderSlot::new(measurements)),
            section("ui-baseline-app-bars", RenderSlot::new(baseline_app_bars)),
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
            intro: Some(RenderSlot::new(guidelines_intro)),
            sections: vec![
                section("ui-usage", RenderSlot::new(usage)),
                section("ui-search-app-bar", RenderSlot::new(search_guidelines)),
                section("ui-anatomy", RenderSlot::new(anatomy_guidelines)),
                section("ui-adaptive-design", RenderSlot::new(adaptive_design)),
                section("ui-behavior", RenderSlot::new(behavior)),
            ],
        },
        ArticlePage {
            title: "ui-accessibility",
            intro: None,
            sections: vec![
                section("ui-use-cases", RenderSlot::new(use_cases)),
                section("ui-interaction-style", RenderSlot::new(interaction_style)),
                section("ui-color", RenderSlot::new(accessible_color)),
                section(
                    "ui-keyboard-navigation",
                    RenderSlot::new(keyboard_navigation),
                ),
                section("ui-labeling-elements", RenderSlot::new(labeling_elements)),
            ],
        },
    ]);
}

#[tessera]
fn numbered(items: Vec<String>) {
    column().children(move || {
        for (index, item) in items.clone().into_iter().enumerate() {
            text(I18N.format(
                "ui-value-value",
                [("v0", (index + 1).into()), ("v1", I18N.text(&item).into())],
            ))
            .style(type_style(16.0, 24.0));
            gap(Dp(8.0));
        }
    });
}

#[tessera]
fn verdict_band(good: bool) {
    let color = if good { DO_COLOR } else { DONT_COLOR };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed().modifier(
                Modifier::new()
                    .fill_max_width()
                    .height(Dp(4.0))
                    .background_with_shape(color, Shape::rounded_rectangle(Dp(2.0))),
            );
            gap(Dp(12.0));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    icon()
                        .painter(if good {
                            filled::CHECK_CIRCLE_SVG
                        } else {
                            filled::CANCEL_SVG
                        })
                        .tint(color)
                        .size(Dp(20.0));
                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                    text(I18N.text(if good { "ui-do" } else { "ui-dont" }))
                        .color(color)
                        .style(type_style(16.0, 24.0));
                });
        });
}

#[tessera]
fn guidance_item(#[prop(into)] note: String, verdict: Option<bool>, content: Option<RenderSlot>) {
    let content = content.unwrap_or_else(RenderSlot::empty);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            panel(content.clone());
            if let Some(good) = verdict {
                gap(Dp(4.0));
                verdict_band(good);
            }
            gap(Dp(16.0));
            caption(note.clone());
        });
}

#[tessera]
fn guidance_pair(
    #[prop(into)] first_note: String,
    #[prop(into)] second_note: String,
    first: Option<RenderSlot>,
    second: Option<RenderSlot>,
    first_verdict: Option<bool>,
    second_verdict: Option<bool>,
) {
    let width = remember(|| Dp(1024.0));
    let first = first.unwrap_or_else(RenderSlot::empty);
    let second = second.unwrap_or_else(RenderSlot::empty);
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if width.get() != next {
                    width.set(next);
                }
            },
        ))
        .children(move || {
            let (first, second) = (first.clone(), second.clone());
            let (first_note, second_note) = (first_note.clone(), second_note.clone());
            if width.get().0 < 640.0 {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        guidance_item(first_note.clone())
                            .verdict_optional(first_verdict)
                            .content_shared(first.clone());
                        gap(Dp(32.0));
                        guidance_item(second_note.clone())
                            .verdict_optional(second_verdict)
                            .content_shared(second.clone());
                    });
            } else {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        let (first, second) = (first.clone(), second.clone());
                        let (first_note, second_note) = (first_note.clone(), second_note.clone());
                        boxed()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || {
                                guidance_item(first_note.clone())
                                    .verdict_optional(first_verdict)
                                    .content_shared(first.clone());
                            });
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                        boxed()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || {
                                guidance_item(second_note.clone())
                                    .verdict_optional(second_verdict)
                                    .content_shared(second.clone());
                            });
                    });
            }
        });
}

#[derive(Clone, Copy, PartialEq)]
enum Glyph {
    Bookmark,
    Share,
    Send,
    Done,
    More,
    Search,
    Map,
    Event,
    PersonAdd,
    Archive,
    Star,
}

impl Glyph {
    fn asset(self) -> Asset {
        match self {
            Self::Bookmark => filled::BOOKMARK_SVG,
            Self::Share => filled::SHARE_SVG,
            Self::Send => filled::SEND_SVG,
            Self::Done => filled::DONE_SVG,
            Self::More => filled::MORE_VERT_SVG,
            Self::Search => filled::SEARCH_SVG,
            Self::Map => filled::MAP_SVG,
            Self::Event => filled::EVENT_SVG,
            Self::PersonAdd => filled::PERSON_ADD_SVG,
            Self::Archive => filled::ARCHIVE_SVG,
            Self::Star => filled::STAR_SVG,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Bookmark => "ui-bookmark",
            Self::Share => "ui-share",
            Self::Send => "ui-send",
            Self::Done => "ui-mark-as-watched",
            Self::More => "ui-more-options",
            Self::Search => "ui-search",
            Self::Map => "ui-view-on-map",
            Self::Event => "ui-add-event",
            Self::PersonAdd => "ui-add-person",
            Self::Archive => "ui-archive",
            Self::Star => "ui-star",
        }
    }
}

#[derive(Clone, Copy)]
struct Preset {
    figure: &'static str,
    variant: TopAppBarVariant,
    title: &'static str,
    subtitle: Option<&'static str>,
    actions: &'static [Glyph],
    filled: usize,
    tonal: bool,
    logo: bool,
    curved: bool,
    short: bool,
    scrolled: bool,
    floating: bool,
    heading: bool,
    content: bool,
}

const BASE: Preset = Preset {
    figure: "",
    variant: TopAppBarVariant::Small,
    title: "",
    subtitle: None,
    actions: &[],
    filled: 0,
    tonal: false,
    logo: false,
    curved: false,
    short: false,
    scrolled: false,
    floating: false,
    heading: false,
    content: false,
};

const HIKING: usize = 0;
const SUSHI: usize = 1;
const SEND: usize = 2;
const FILMS_DO: usize = 3;
const FILMS_DONT: usize = 4;
const LOGO: usize = 5;
const CURVED: usize = 6;
const STRAIGHT: usize = 7;
const DEFAULT_HEIGHT: usize = 8;
const SHORT: usize = 9;
const WRAP: usize = 10;
const ORDERED: usize = 11;
const WIDE: usize = 12;
const NARROW: usize = 13;
const FLAT: usize = 14;
const SCROLLED: usize = 15;
const FLOATING: usize = 16;
const LABELED: usize = 19;
const SUBTITLED: usize = 20;
const FILLED_SINGLE: usize = 21;

fn preset_for(kind: usize) -> Preset {
    match kind {
        HIKING => Preset {
            figure: "ui-hiking-trails-app-bar",
            variant: TopAppBarVariant::MediumFlexible,
            title: "ui-top-10-hiking-trails",
            subtitle: Some("ui-discover-popular-trails"),
            actions: &[Glyph::Bookmark],
            filled: 1,
            tonal: true,
            ..BASE
        },
        SUSHI => Preset {
            figure: "ui-contextual-app-bar",
            variant: TopAppBarVariant::MediumFlexible,
            title: "ui-the-best-sushi-spots-in-los-angeles-and-orange-county",
            actions: &[Glyph::Share],
            filled: 1,
            tonal: true,
            ..BASE
        },
        SEND => Preset {
            figure: "ui-primary-action-app-bar",
            variant: TopAppBarVariant::MediumFlexible,
            title: "ui-image",
            subtitle: Some("ui-7-2-mb-png"),
            actions: &[Glyph::Send],
            filled: 1,
            ..BASE
        },
        FILMS_DO => Preset {
            figure: "ui-single-filled-action-app-bar",
            title: "ui-must-watch-films",
            actions: &[Glyph::Done],
            filled: 1,
            ..BASE
        },
        FILMS_DONT => Preset {
            figure: "ui-multiple-filled-actions-app-bar",
            title: "ui-must-watch-films",
            actions: &[Glyph::More, Glyph::Done],
            filled: 2,
            ..BASE
        },
        LOGO => Preset {
            figure: "ui-logo-app-bar",
            title: "",
            actions: &[Glyph::Bookmark],
            logo: true,
            ..BASE
        },
        CURVED => Preset {
            figure: "ui-curved-container-app-bar",
            title: "ui-hike-trails",
            actions: &[Glyph::Search, Glyph::Bookmark],
            curved: true,
            content: true,
            ..BASE
        },
        STRAIGHT => Preset {
            figure: "ui-straight-container-app-bar",
            title: "ui-hike-trails",
            actions: &[Glyph::Search, Glyph::Bookmark],
            content: true,
            ..BASE
        },
        DEFAULT_HEIGHT => Preset {
            figure: "ui-default-height-app-bar",
            title: "ui-astral-meadow",
            actions: &[Glyph::Search, Glyph::Bookmark],
            content: true,
            ..BASE
        },
        SHORT => Preset {
            figure: "ui-shortened-app-bar",
            title: "ui-astral-meadow",
            actions: &[Glyph::Search, Glyph::Bookmark],
            short: true,
            content: true,
            ..BASE
        },
        WRAP => Preset {
            figure: "ui-wrapped-headline-app-bar",
            variant: TopAppBarVariant::MediumFlexible,
            title: "ui-hiking-trips-with-elie-in-santa-monica-and-joshua-tree",
            actions: &[Glyph::Map, Glyph::Bookmark],
            ..BASE
        },
        ORDERED => Preset {
            figure: "ui-ordered-actions-app-bar",
            title: "ui-saved-podcasts",
            actions: &[Glyph::Search, Glyph::Event, Glyph::More],
            ..BASE
        },
        WIDE => Preset {
            figure: "ui-expanded-window-app-bar",
            title: "ui-aki-aro",
            actions: &[
                Glyph::Search,
                Glyph::Event,
                Glyph::PersonAdd,
                Glyph::Archive,
            ],
            ..BASE
        },
        NARROW => Preset {
            figure: "ui-compact-window-app-bar",
            title: "ui-aki-aro",
            actions: &[Glyph::Search, Glyph::More],
            ..BASE
        },
        FLAT => Preset {
            figure: "ui-flat-app-bar",
            title: "ui-top-stories",
            actions: &[Glyph::More],
            content: true,
            ..BASE
        },
        SCROLLED => Preset {
            figure: "ui-scrolled-app-bar",
            title: "ui-top-stories",
            actions: &[Glyph::More],
            scrolled: true,
            content: true,
            ..BASE
        },
        FLOATING => Preset {
            figure: "ui-transparent-app-bar",
            title: "",
            actions: &[Glyph::Star],
            floating: true,
            content: true,
            ..BASE
        },
        LABELED => Preset {
            figure: "ui-labeled-app-bar",
            title: "ui-saved-podcasts",
            subtitle: Some("ui-80-episodes"),
            actions: &[Glyph::Map],
            heading: true,
            ..BASE
        },
        SUBTITLED => Preset {
            figure: "ui-subtitle-app-bar",
            title: "ui-saved-podcasts",
            subtitle: Some("ui-80-episodes"),
            actions: &[Glyph::Share],
            ..BASE
        },
        _ => Preset {
            figure: "ui-filled-icon-button-app-bar",
            title: "ui-saved-podcasts",
            actions: &[Glyph::Share],
            filled: 1,
            ..BASE
        },
    }
}

#[tessera]
fn specimen(kind: usize) {
    let status = remember(String::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let figure = preset_for(kind).figure;
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(I18N.text(&(figure))),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            let preset = preset_for(kind);
            let width = if kind == WRAP {
                Modifier::new().size_in(None, Some(Dp(412.0)), None, None)
            } else {
                Modifier::new()
            };
            column()
                .modifier(
                    width
                        .fill_max_width()
                        .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(12.0)))
                        .clip_shape(Shape::rounded_rectangle(Dp(12.0))),
                )
                .children(move || {
                    let preset = preset_for(kind);
                    if preset.floating {
                        boxed()
                            .alignment(Alignment::TopStart)
                            .modifier(Modifier::new().fill_max_width())
                            .children(move || {
                                specimen_content(true);
                                specimen_bar(kind, status);
                            });
                    } else if preset.short {
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .fill_max_width()
                                    .height(Dp(44.0))
                                    .clip_to_bounds(),
                            )
                            .children(move || {
                                specimen_bar(kind, status);
                            });
                        specimen_content(false);
                    } else {
                        specimen_bar(kind, status);
                        if preset.content {
                            specimen_content(false);
                        }
                    }
                });
            if !status.get().is_empty() {
                gap(Dp(8.0));
                text(I18N.format(
                    "ui-value-value-format-1",
                    [
                        ("v0", I18N.text(&preset.figure).into()),
                        ("v1", I18N.text(&status.get()).into()),
                    ],
                ))
                .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn specimen_content(floating: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            if floating {
                boxed().modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(160.0))
                        .background_with_shape(
                            scheme.tertiary_container,
                            Shape::rounded_rectangle(Dp(0.0)),
                        )
                        .semantics(SemanticsArgs {
                            role: Some(Role::Image),
                            label: Some(I18N.text("ui-vacation-rental-photo")),
                            ..Default::default()
                        }),
                );
            }
            specimen_lines(floating);
        });
}

#[tessera]
fn specimen_lines(floating: bool) {
    column()
        .modifier(Modifier::new().fill_max_width().padding_symmetric(
            Dp(16.0),
            Dp(if floating { 20.0 } else { 12.0 }),
        ))
        .children(move || {
            let lines: &[&str] = if floating {
                &[
                    "ui-the-kitchen-is-well-equipped-perfect-for-whipping-up-some-delicious-meals-together",
                    "ui-let-s-not-forget-the-amazing-outdoor-area-sounds-like-the-perfect-place-to-soak-up-the-sun",
                ]
            } else {
                &["ui-saved-item-1", "ui-saved-item-2"]
            };
            for line in lines {
                text(I18N.text(*line)).style(type_style(14.0, 20.0));
                gap(Dp(8.0));
            }
        });
}

#[tessera]
fn specimen_bar(kind: usize, status: State<String>) {
    let preset = preset_for(kind);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let container = if preset.floating {
        Some(Color::TRANSPARENT)
    } else if preset.scrolled {
        Some(scheme.surface_container)
    } else if kind != FLAT && (preset.curved || preset.short || preset.content) {
        Some(scheme.surface_container_low)
    } else {
        None
    };
    let heading = match preset.subtitle {
        Some(subtitle) => I18N.format(
            "ui-title-subtitle",
            [
                ("title", I18N.text(preset.title).into()),
                ("subtitle", I18N.text(subtitle).into()),
            ],
        ),
        None => I18N.text(preset.title),
    };
    let title_modifier = if preset.heading {
        Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Heading),
            label: Some(heading),
            merge_descendants: true,
            ..Default::default()
        })
    } else {
        Modifier::new()
    };
    let logo = RenderSlot::new(brand);
    top_app_bar()
        .modifier(if preset.curved {
            Modifier::new().clip_shape(Shape::rounded_rectangle(Dp(28.0)))
        } else {
            Modifier::new()
        })
        .variant(preset.variant)
        .title(I18N.text(preset.title))
        .subtitle_optional(preset.subtitle.map(str::to_owned))
        .title_area_modifier(title_modifier)
        .title_content_optional(preset.logo.then_some(logo))
        .elevation(Dp(0.0))
        .container_color_optional(container)
        .navigation_icon(move || {
            let preset = preset_for(kind);
            if preset.floating {
                icon_button()
                    .filled_tonal()
                    .icon(filled::ARROW_BACK_SVG)
                    .accessibility_label(I18N.text("ui-back"))
                    .on_click(move || status.set("ui-back-selected".to_owned()));
            } else {
                icon_button()
                    .standard()
                    .color_style(IconButtonColorStyle::Inherited)
                    .icon(filled::ARROW_BACK_SVG)
                    .accessibility_label(I18N.text("ui-back"))
                    .on_click(move || status.set("ui-back-selected".to_owned()));
            }
        })
        .action(move || {
            let preset = preset_for(kind);
            let count = preset.actions.len();
            for (index, glyph) in preset.actions.iter().copied().enumerate() {
                let selected = I18N.format(
                    "ui-value-selected",
                    [("v0", I18N.text(&glyph.label()).into())],
                );
                if preset.floating || preset.tonal && index + preset.filled >= count {
                    icon_button()
                        .filled_tonal()
                        .icon(glyph.asset())
                        .accessibility_label(I18N.text(glyph.label()))
                        .on_click(move || status.set(selected.clone()));
                } else if index + preset.filled >= count {
                    icon_button()
                        .filled()
                        .icon(glyph.asset())
                        .accessibility_label(I18N.text(glyph.label()))
                        .on_click(move || status.set(selected.clone()));
                } else {
                    icon_button()
                        .standard()
                        .color_style(IconButtonColorStyle::Inherited)
                        .icon(glyph.asset())
                        .accessibility_label(I18N.text(glyph.label()))
                        .on_click(move || status.set(selected.clone()));
                }
            }
        });
}

#[tessera]
fn overview_intro() {
    bullets(vec![
        "ui-focus-on-describing-the-current-page-and-provide-1-2-essential-actions".into(),
        "ui-displays-labels-and-page-navigation-controls-at-the-top-of-the-page-use-a-toolbar-to-displ".into(),
        "ui-four-variants-search-app-bar-small-medium-flexible-large-flexible".into(),
        "ui-on-scroll-apply-a-fill-color-to-separate-from-body-content".into(),
        "ui-can-animate-on-and-off-screen-with-another-bar-of-controls-like-a-row-of-chips".into(),
    ]);
    gap(Dp(32.0));
    variants_figure("ui-variant");
    gap(Dp(16.0));
    numbered(vec![
        "ui-search-app-bar".into(),
        "ui-small".into(),
        "ui-medium-flexible".into(),
        "ui-large-flexible".into(),
    ]);
    gap(Dp(24.0));
    paragraph(
        "ui-tessera-examples-use-the-native-top-app-bar-component-for-the-small-medium-flexible-a"
            .into(),
    );
}

#[tessera]
fn guidelines_intro() {
    panel(RenderSlot::new(|| {
        specimen(HIKING);
    }));
    gap(Dp(16.0));
    paragraph("ui-app-bars-show-information-about-the-page-key-actions-and-navigation-actions-like-back-or-m".into());
}

#[tessera]
fn usage() {
    paragraph("ui-use-an-app-bar-to-provide-content-and-actions-related-to-the-current-page-such-as-page-nav".into());
    gap(Dp(16.0));
    paragraph("ui-the-information-and-actions-in-the-app-bar-should-be-contextual-and-specific-to-a-page-but".into());
    gap(Dp(24.0));
    guidance_item("ui-app-bars-provide-content-and-actions-related-to-the-current-page").content(
        || {
            specimen(SUSHI);
        },
    );
    gap(Dp(48.0));
    paragraph("ui-app-bars-should-only-have-one-action-two-if-necessary-the-primary-action-should-alter-or-e".into());
    gap(Dp(16.0));
    paragraph("ui-if-a-product-needs-many-page-actions-place-those-in-a-toolbar-avoid-placing-an-overflow-me".into());
    gap(Dp(24.0));
    guidance_item("ui-app-bars-can-display-one-high-visibility-action-to-boost-its-prominence")
        .content(|| {
            specimen(SEND);
        });
    gap(Dp(48.0));
    paragraph("ui-to-boost-visibility-of-a-primary-action-change-the-style-of-the-icon-button-to-filled-or-t".into());
    gap(Dp(24.0));
    guidance_pair(
        "ui-use-a-filled-or-tonal-button-for-important-actions",
        "ui-don-t-put-multiple-filled-or-tonal-buttons-in-the-app-bar",
    )
    .first(|| {
        specimen(FILMS_DO);
    })
    .second(|| {
        specimen(FILMS_DONT);
    })
    .first_verdict(true)
    .second_verdict(false);
    gap(Dp(48.0));
    subheading("ui-variants".into());
    gap(Dp(16.0));
    paragraph("ui-the-four-variants-of-app-bars-are".into());
    gap(Dp(8.0));
    numbered(vec![
        "ui-search-app-bar-use-on-home-pages-when-search-is-key-to-the-product".into(),
        "ui-small-use-in-dense-layouts-or-when-a-page-is-scrolled".into(),
        "ui-medium-flexible-use-to-display-a-larger-headline-it-can-collapse-into-a-small-app-bar-on-s".into(),
        "ui-large-flexible-use-to-emphasize-the-headline-of-the-page".into(),
    ]);
    gap(Dp(24.0));
    variants_figure("ui-guideline-variant");
    gap(Dp(48.0));
    subheading("ui-baseline-app-bars".into());
    gap(Dp(16.0));
    paragraph("ui-there-are-two-baseline-app-bars-that-are-no-longer-recommended".into());
    gap(Dp(8.0));
    numbered(vec![
        "ui-medium-replace-with-medium-flexible".into(),
        "ui-large-replace-with-large-flexible".into(),
    ]);
    gap(Dp(24.0));
    baseline_figure("ui-guideline-baseline");
}

#[tessera]
fn search_guidelines() {
    paragraph(
        "ui-use-a-search-app-bar-to-provide-an-emphasized-entry-point-to-open-the-search-view"
            .into(),
    );
    gap(Dp(24.0));
    guidance_item("ui-search-app-bars-have-a-search-field-instead-of-heading-text").content(
        move || {
            search_sample().placeholder(I18N.text("ui-search-product"));
        },
    );
    gap(Dp(48.0));
    paragraph("ui-search-bars-should-always-include-the-word-search-they-can-use-various-capitalization-styl".into());
    gap(Dp(8.0));
    numbered(vec![
        "ui-search".into(),
        "ui-searching-a-specific-area-example-search-inbox".into(),
        "ui-search-product-example-search-photos".into(),
    ]);
    gap(Dp(24.0));
    guidance_item("ui-use-proper-capitalization-depending-on-what-s-being-searched").content(
        move || {
            for hint in ["ui-search", "ui-search-inbox", "ui-search-photos"] {
                search_sample().placeholder(I18N.text(hint));
                gap(Dp(16.0));
            }
        },
    );
    gap(Dp(48.0));
    subheading("ui-buttons-in-search-app-bar".into());
    gap(Dp(16.0));
    paragraph("ui-in-addition-to-a-trailing-avatar-search-app-bars-can-have-up-to-two-trailing-icons-on-mobi".into());
    gap(Dp(24.0));
    guidance_item("ui-put-the-most-used-actions-on-the-left-and-least-used-on-the-right").content(
        move || {
            search_sample()
                .specimen(1)
                .placeholder(I18N.text("ui-search-music"));
        },
    );
    gap(Dp(48.0));
    paragraph("ui-the-leading-element-of-a-search-app-bar-can-be-used-for-a-product-s-logo-to-brand-the-app".into());
    gap(Dp(24.0));
    guidance_item("ui-the-leading-element-can-be-a-product-logo").content(move || {
        search_sample()
            .specimen(2)
            .logo(true)
            .placeholder(I18N.text("ui-search-tessera"));
    });
    gap(Dp(48.0));
    paragraph("ui-don-t-use-more-than-two-trailing-icon-buttons-with-an-avatar-if-more-actions-are-needed-co".into());
    gap(Dp(24.0));
    guidance_pair(
        "ui-use-up-to-two-trailing-icons-with-an-avatar",
        "ui-don-t-use-three-icons-and-an-avatar-in-a-search-app-bar",
    )
    .first(move || {
        search_sample()
            .specimen(1)
            .placeholder(I18N.text("ui-search-news"));
    })
    .second(move || {
        search_sample()
            .specimen(5)
            .placeholder(I18N.text("ui-search-news"));
    })
    .first_verdict(true)
    .second_verdict(false);
    gap(Dp(48.0));
    subheading("ui-large-screens".into());
    gap(Dp(16.0));
    paragraph("ui-the-search-app-bar-dynamically-adapts-to-available-width-there-should-be-up-to-four-traili".into());
    gap(Dp(24.0));
    guidance_item(
        "ui-on-a-larger-window-the-search-field-stays-centered-between-leading-and-trailing-elements",
    )
    .content(move || {
        search_sample().specimen(6).placeholder(I18N.text("ui-search-news"));
    });
    gap(Dp(48.0));
    subheading("ui-alternate-color-options".into());
    gap(Dp(16.0));
    paragraph("ui-by-default-search-containers-in-app-bars-use-the-surface-container-color-to-distinguish-it".into());
    gap(Dp(16.0));
    paragraph("ui-when-choosing-alternate-colors-make-sure-the-search-text-and-container-have-at-least-3-1-c".into());
    gap(Dp(24.0));
    guidance_item("ui-search-app-bars-can-use-different-colors-like-surface-bright-for-improved-contrast-with-su")
        .content(move || {
search_sample().bright(true).placeholder(I18N.text("ui-search-mail"));
});
}

#[tessera]
fn anatomy_guidelines() {
    app_figure(TopAppBarVariant::Small, false);
    gap(Dp(48.0));
    subheading("ui-container".into());
    gap(Dp(16.0));
    paragraph("ui-the-app-bar-container-holds-all-information-and-actions-at-the-top-of-a-screen-including-n".into());
    gap(Dp(24.0));
    guidance_pair(
        "ui-use-straight-corners-for-app-bars",
        "ui-don-t-use-curved-shapes-this-implies-that-the-container-can-expand-upon-interaction",
    )
    .first(|| {
        specimen(STRAIGHT);
    })
    .second(|| {
        specimen(CURVED);
    })
    .first_verdict(true)
    .second_verdict(false);
    gap(Dp(32.0));
    paragraph("ui-always-use-the-default-height-of-the-app-bar-and-make-it-span-the-full-width-of-the-window".into());
    gap(Dp(24.0));
    guidance_pair(
        "ui-default-heights-were-chosen-to-ensure-readability-of-on-screen-elements",
        "ui-don-t-make-an-app-bar-shorter-than-its-default-height",
    )
    .first(|| {
        specimen(DEFAULT_HEIGHT);
    })
    .second(|| {
        specimen(SHORT);
    })
    .first_verdict(true)
    .second_verdict(false);
    gap(Dp(48.0));
    subheading("ui-adding-logos".into());
    gap(Dp(16.0));
    paragraph("ui-image-logos-can-be-used-in-app-bars-to-bolster-brand-identity-or-visual-appeal-the-image-s".into());
    gap(Dp(24.0));
    guidance_item("ui-a-logo-can-replace-the-headline-in-a-small-app-bar").content(|| {
        specimen(LOGO);
    });
    gap(Dp(48.0));
    subheading("ui-leading-button".into());
    gap(Dp(16.0));
    paragraph("ui-use-back-to-return-to-the-previous-screen-or-menu-to-open-product-navigation-keep-the-lead".into());
    gap(Dp(48.0));
    subheading("ui-headline".into());
    gap(Dp(16.0));
    paragraph("ui-the-headline-can-describe-the-current-page-the-current-section-or-the-product-headline-tex".into());
    gap(Dp(16.0));
    paragraph("ui-in-medium-flexible-and-large-flexible-app-bars-the-headline-can-wrap-to-a-second-line-don".into());
    gap(Dp(24.0));
    guidance_item("ui-if-headline-text-is-long-use-a-medium-flexible-or-large-flexible-app-bar-and-wrap-the-head")
        .verdict(true)
        .content(|| {
specimen(WRAP);
});
    gap(Dp(32.0));
    paragraph("ui-headlines-can-be-aligned-to-the-leading-edge-or-centered-the-headline-s-typography-style-d".into());
    gap(Dp(24.0));
    guidance_item("ui-headline-typography-small-uses-title-large-medium-flexible-uses-headline-medium-and-large").content(move || {
        for (variant, centered, name) in [
            (TopAppBarVariant::Small, false, "ui-small"),
            (TopAppBarVariant::MediumFlexible, false, "ui-medium-flexible"),
            (TopAppBarVariant::LargeFlexible, true, "ui-large-flexible"),
        ] {
            sample(I18N.format("ui-headline-value", [("v0", I18N.text(&name).into())]), variant).centered(centered);
            gap(Dp(16.0));
        }
    });
    gap(Dp(48.0));
    subheading("ui-subtitle".into());
    gap(Dp(16.0));
    paragraph("ui-subtitles-are-aligned-with-their-headline-their-typography-style-depends-on-the-app-bar-si".into());
    gap(Dp(24.0));
    guidance_item("ui-subtitle-typography-small-uses-label-medium-medium-flexible-uses-label-large-and-large-fle").content(move || {
        for (variant, name) in [
            (TopAppBarVariant::Small, "ui-small"),
            (TopAppBarVariant::MediumFlexible, "ui-medium-flexible"),
            (TopAppBarVariant::LargeFlexible, "ui-large-flexible"),
        ] {
            sample(I18N.format("ui-subtitle-style-value", [("v0", I18N.text(&name).into())]), variant).subtitle(true);
            gap(Dp(16.0));
        }
    });
    gap(Dp(48.0));
    subheading("ui-trailing-icon-buttons".into());
    gap(Dp(16.0));
    paragraph("ui-trailing-icon-buttons-are-aligned-to-the-trailing-edge-of-the-app-bar-place-most-used-acti".into());
    gap(Dp(16.0));
    paragraph("ui-avoid-using-these-buttons-to-open-a-menu-with-more-actions-if-more-actions-are-needed-plac".into());
    gap(Dp(16.0));
    paragraph("ui-use-filled-icons-when-possible-for-the-best-visibility-outlined-icons-can-also-be-used-par".into());
    gap(Dp(24.0));
    guidance_item("ui-put-the-most-used-actions-on-the-left-and-least-used-on-the-right").content(
        || {
            specimen(ORDERED);
        },
    );
}

#[tessera]
fn adaptive_design() {
    paragraph("ui-adaptive-design-allows-an-interface-to-respond-or-change-based-on-context-such-as-the-user".into());
    gap(Dp(32.0));
    subheading("ui-resizing".into());
    gap(Dp(16.0));
    paragraph("ui-the-width-of-the-app-bar-container-responds-to-the-view-or-device-width-it-should-always-s".into());
    gap(Dp(24.0));
    guidance_item("ui-the-app-bar-s-container-responds-to-always-fill-the-window-width").content(
        || {
            sample("ui-resizing-app-bar", TopAppBarVariant::Small);
        },
    );
    gap(Dp(32.0));
    paragraph("ui-resizing-may-cause-actions-at-the-trailing-edge-of-the-app-bar-to-collapse-into-an-overflo".into());
    gap(Dp(24.0));
    guidance_pair(
        "ui-in-an-expanded-window-every-trailing-action-stays-visible",
        "ui-actions-at-the-trailing-edge-collapse-into-an-overflow-menu",
    )
    .first(|| {
        specimen(WIDE);
    })
    .second(|| {
        specimen(NARROW);
    });
    gap(Dp(32.0));
    paragraph("ui-the-search-container-of-the-search-app-bar-should-fill-100-of-the-space-between-leading-an".into());
    gap(Dp(24.0));
    guidance_item("ui-the-search-field-grows-with-the-window-until-it-reaches-312dp-then-grows-at-half-the-rate")
        .content(|| {
search_sample();
});
    gap(Dp(48.0));
    subheading("ui-presentation".into());
    gap(Dp(16.0));
    paragraph("ui-the-app-bar-automatically-supports-right-to-left-rtl-languages-by-aligning-the-layout-of-e".into());
    gap(Dp(16.0));
    caption(
        "ui-tessera-does-not-provide-a-layout-direction-yet-the-right-to-left-bar-below-is-arranged-by",
    );
    gap(Dp(24.0));
    guidance_pair(
        "ui-left-to-right-navigation-at-the-leading-edge-actions-at-the-trailing-edge",
        "ui-right-to-left-the-layout-is-mirrored-with-navigation-on-the-right-and-actions-on-the-left",
    )
    .first(|| {
        mirrored_bar(false);
    })
    .second(|| {
        mirrored_bar(true);
    });
}

#[tessera]
fn mirrored_bar(rtl: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let direction = if rtl {
        "ui-right-to-left"
    } else {
        "ui-left-to-right"
    };
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(64.0))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(0.0)))
                .padding_symmetric(Dp(4.0), Dp(0.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Toolbar),
                    label: Some(
                        I18N.format("ui-value-app-bar", [("v0", I18N.text(&direction).into())]),
                    ),
                    ..Default::default()
                }),
        )
        .children(move || {
            if rtl {
                mirrored_actions(direction.to_owned());
                boxed().modifier(Modifier::new().weight(1.0)).children(|| {
                    mirrored_title(true);
                });
                icon_button()
                    .standard()
                    .icon(filled::ARROW_FORWARD_SVG)
                    .accessibility_label(
                        I18N.format("ui-value-back", [("v0", I18N.text(&direction).into())]),
                    );
            } else {
                icon_button()
                    .standard()
                    .icon(filled::ARROW_BACK_SVG)
                    .accessibility_label(
                        I18N.format("ui-value-back", [("v0", I18N.text(&direction).into())]),
                    );
                boxed().modifier(Modifier::new().weight(1.0)).children(|| {
                    mirrored_title(false);
                });
                mirrored_actions(direction.to_owned());
            }
        });
}

#[tessera]
fn mirrored_title(rtl: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(if rtl {
            Alignment::CenterEnd
        } else {
            Alignment::CenterStart
        })
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(4.0), Dp(0.0)),
        )
        .children(move || {
            text(I18N.text("ui-headline"))
                .color(scheme.on_surface)
                .style(type_style(22.0, 28.0));
        });
}

#[tessera]
fn mirrored_actions(#[prop(into)] direction: String) {
    for (glyph, name) in [
        (filled::SEARCH_SVG, "search"),
        (filled::EVENT_SVG, "calendar"),
    ] {
        icon_button()
            .standard()
            .icon(glyph)
            .accessibility_label(I18N.format(
                "ui-value-value-format-2",
                [
                    ("v0", I18N.text(&direction).into()),
                    ("v1", I18N.text(&name).into()),
                ],
            ));
    }
}

#[tessera]
fn behavior() {
    subheading("ui-scrolling".into());
    gap(Dp(16.0));
    paragraph("ui-app-bars-should-initially-be-the-same-color-as-the-background-then-fill-with-a-contrasting".into());
    gap(Dp(24.0));
    guidance_pair(
        "ui-a-flat-app-bar-matches-the-background",
        "ui-upon-scrolling-an-app-bar-container-fills-with-contrasting-color-to-create-a-visual-separa",
    )
    .first(|| {
specimen(FLAT);
})
    .second(|| {
specimen(SCROLLED);
});
    gap(Dp(32.0));
    paragraph("ui-to-focus-more-on-body-content-consider-setting-the-app-bar-container-to-be-transparent-on".into());
    gap(Dp(24.0));
    guidance_item("ui-a-transparent-app-bar-lets-filled-icon-buttons-float-above-the-content")
        .content(|| {
            specimen(FLOATING);
        });
    gap(Dp(32.0));
    paragraph("ui-when-scrolled-medium-flexible-and-large-flexible-app-bars-can-transform-into-small-app-bar".into());
    gap(Dp(24.0));
    guidance_pair(
        "ui-the-app-bar-can-hide-when-scrolling-up-and-reveal-when-scrolling-down",
        "ui-medium-and-large-flexible-app-bars-can-use-the-compress-effect-to-transform-into-small-app",
    )
    .first(|| {
        sample("ui-hiding-app-bar", TopAppBarVariant::Small)
            .behavior(AppBarScrollBehavior::EnterAlways)
            .controls(true);
    })
    .second(move || {
        sample("ui-scrolling-app-bar", TopAppBarVariant::MediumFlexible)
            .subtitle(true)
            .controls(true);
    });
}

#[tessera]
fn use_cases() {
    paragraph("ui-people-should-be-able-to-do-the-following-using-assistive-technology".into());
    gap(Dp(8.0));
    bullets(vec![
        "ui-understand-what-page-they-re-currently-visiting".into(),
        "ui-take-actions-or-navigate-to-a-new-page-destination".into(),
        "ui-maintain-access-to-app-bar-actions-when-the-content-is-scrolled".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    subheading("ui-touch".into());
    gap(Dp(16.0));
    paragraph(
        "ui-when-tapping-on-an-icon-button-a-touch-ripple-appears-indicating-interaction-feedback"
            .into(),
    );
    gap(Dp(32.0));
    subheading("ui-cursor".into());
    gap(Dp(16.0));
    paragraph("ui-when-hovered-the-hover-state-provides-a-visual-cue-to-the-user-that-the-element-is-interac".into());
    gap(Dp(32.0));
    subheading("ui-keyboard-or-switch".into());
    gap(Dp(16.0));
    paragraph("ui-when-navigating-to-an-interactive-element-a-focus-indicator-appears-to-show-that-action-ca".into());
    gap(Dp(32.0));
    subheading("ui-initial-focus".into());
    gap(Dp(16.0));
    paragraph("ui-focus-should-initially-land-on-the-leading-button-since-it-s-the-first-interactive-element".into());
    gap(Dp(24.0));
    guidance_item("ui-tab-moves-through-the-app-bar-from-the-leading-button-space-or-enter-activates-the-focused")
        .content(|| {
sample("ui-accessible-app-bar", TopAppBarVariant::Small).controls(true);
});
}

#[tessera]
fn accessible_color() {
    paragraph("ui-on-search-app-bars-use-the-default-color-roles-when-possible".into());
    gap(Dp(8.0));
    bullets(vec![
        "ui-search-container-surface-container".into(),
        "ui-search-label-on-surface-variant".into(),
    ]);
    gap(Dp(8.0));
    paragraph("ui-on-darker-backgrounds-search-bar-containers-can-use-the-surface-bright-role-to-maintain-st".into());
    gap(Dp(24.0));
    guidance_pair(
        "ui-make-sure-search-bars-and-their-labels-have-at-least-3-1-contrast-use-the-default-colors-w",
        "ui-avoid-using-custom-color-roles-for-the-search-bar-container-and-search-label-text-if-custo",
    )
    .first(move || {
search_sample().placeholder(I18N.text("ui-search-messages"));
})
    .second(low_contrast_search)
    .first_verdict(true)
    .second_verdict(false);
}

#[tessera]
fn low_contrast_search() {
    let base = use_context::<MaterialTheme>().unwrap().get();
    material_theme()
        .theme(move || {
            let mut theme = base.clone();
            theme.color_scheme.on_surface_variant = theme.color_scheme.outline_variant;
            theme
        })
        .child(move || {
            search_sample().placeholder(I18N.text("ui-search-messages"));
        });
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-tab".into(),
                "ui-move-focus-to-the-next-interactive-element".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activate-the-focused-element".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    sample("ui-keyboard-app-bar", TopAppBarVariant::Small).controls(true);
}

#[tessera]
fn labeling_elements() {
    paragraph("ui-the-accessibility-label-for-a-title-should-be-the-same-as-the-content-within-the-title-if".into());
    gap(Dp(16.0));
    paragraph("ui-screen-readers-will-read-the-ui-text-followed-by-the-component-s-role-the-headline-has-the".into());
    gap(Dp(16.0));
    paragraph("ui-label-icon-buttons-according-to-their-accessibility-guidelines-the-app-bar-icon-button-has".into());
    gap(Dp(24.0));
    guidance_item("ui-an-app-bar-s-accessibility-label-can-incorporate-its-ui-text-as-well-as-additional-context")
        .content(|| {
specimen(LABELED);
});
}

fn section(title: &'static str, content: RenderSlot) -> SpecSection {
    SpecSection { title, content }
}

#[tessera]
fn paragraph(content: String) {
    text(I18N.text(content)).style(type_style(16.0, 24.0));
}

#[tessera]
fn sample(
    #[prop(into)] name: String,
    variant: TopAppBarVariant,
    subtitle: Option<bool>,
    centered: Option<bool>,
    controls: Option<bool>,
    logo: Option<bool>,
    behavior: Option<AppBarScrollBehavior>,
) {
    let label = remember(|| name.clone());
    let variant = remember(|| variant);
    let subtitle = remember(|| subtitle.unwrap_or(false));
    let centered = remember(|| centered.unwrap_or(false));
    let filled_action = remember(|| false);
    let logo = remember(|| logo.unwrap_or(false));
    let status = remember(|| "ui-no-action".to_owned());
    let scroll = remember(AppBarScrollController::default);
    let content = remember(ScrollableController::new);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            if controls.unwrap_or(false) {
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        for (value, title) in [
                            (TopAppBarVariant::Small, "ui-small"),
                            (TopAppBarVariant::MediumFlexible, "ui-medium-flexible"),
                            (TopAppBarVariant::LargeFlexible, "ui-large-flexible"),
                        ] {
                            button()
                                .tonal()
                                .selected(variant.get() == value)
                                .accessibility_label(I18N.format(
                                    "ui-value-value-format-2",
                                    [
                                        ("v0", I18N.text(&label.get()).into()),
                                        ("v1", I18N.text(&title).into()),
                                    ],
                                ))
                                .on_click(move || {
                                    variant.set(value);
                                    scroll.with_mut(AppBarScrollController::reset);
                                })
                                .child(move || {
                                    text(I18N.text(title));
                                });
                        }
                        button()
                            .tonal()
                            .selected(subtitle.get())
                            .accessibility_label(I18N.format(
                                "ui-value-subtitle",
                                [("v0", I18N.text(&label.get()).into())],
                            ))
                            .on_click(move || subtitle.set(!subtitle.get()))
                            .child(move || {
                                text(I18N.text("ui-subtitle"));
                            });
                        button()
                            .tonal()
                            .selected(centered.get())
                            .accessibility_label(I18N.format(
                                "ui-value-centered",
                                [("v0", I18N.text(&label.get()).into())],
                            ))
                            .on_click(move || centered.set(!centered.get()))
                            .child(move || {
                                text(I18N.text("ui-centered"));
                            });
                        button()
                            .tonal()
                            .selected(filled_action.get())
                            .accessibility_label(I18N.format(
                                "ui-value-filled-action",
                                [("v0", I18N.text(&label.get()).into())],
                            ))
                            .on_click(move || filled_action.set(!filled_action.get()))
                            .child(move || {
                                text(I18N.text("ui-filled-action"));
                            });
                        button()
                            .tonal()
                            .selected(logo.get())
                            .accessibility_label(
                                I18N.format(
                                    "ui-value-logo",
                                    [("v0", I18N.text(&label.get()).into())],
                                ),
                            )
                            .on_click(move || logo.set(!logo.get()))
                            .child(move || {
                                text(I18N.text("ui-logo"));
                            });
                        button()
                            .text()
                            .accessibility_label(I18N.format(
                                "ui-reset-value-scroll",
                                [("v0", I18N.text(&label.get()).into())],
                            ))
                            .on_click(move || {
                                scroll.with_mut(AppBarScrollController::reset);
                                content.with_mut(|content| {
                                    content.set_scroll_position(tessera_ui::PxPosition::ZERO)
                                });
                            })
                            .child(move || {
                                text(I18N.text("ui-reset-scroll"));
                            });
                    });
                gap(Dp(24.0));
            }
            app_bar_scroll_provider()
                .controller(scroll)
                .behavior(behavior.unwrap_or(AppBarScrollBehavior::ExitUntilCollapsed))
                .scroll_controller(content)
                .child(move || {
                    let bar = RenderSlot::new(move || {
                        boxed()
                            .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some(I18N.format(
                                    "ui-value-bar",
                                    [("v0", I18N.text(&label.get()).into())],
                                )),
                                merge_descendants: true,
                                ..Default::default()
                            }))
                            .children(move || {
                                let logo_slot = RenderSlot::new(brand);
                                top_app_bar()
                                    .scroll_controller(scroll)
                                    .variant(variant.get())
                                    .title(I18N.text("ui-library"))
                                    .subtitle_optional(
                                        subtitle.get().then(|| "ui-recently-saved".to_owned()),
                                    )
                                    .title_alignment(if centered.get() {
                                        AppBarTitleAlignment::Center
                                    } else {
                                        AppBarTitleAlignment::Start
                                    })
                                    .title_area_modifier(Modifier::new().semantics(SemanticsArgs {
                                        role: Some(Role::Group),
                                        label: Some(I18N.format(
                                            "ui-value-title",
                                            [("v0", I18N.text(&label.get()).into())],
                                        )),
                                        merge_descendants: true,
                                        ..Default::default()
                                    }))
                                    .title_content_optional(logo.get().then_some(logo_slot))
                                    .navigation_icon(move || {
                                        icon_button()
                                            .standard()
                                            .color_style(IconButtonColorStyle::Inherited)
                                            .icon(filled::ARROW_BACK_SVG)
                                            .accessibility_label(I18N.format(
                                                "ui-value-back",
                                                [("v0", I18N.text(&label.get()).into())],
                                            ))
                                            .on_click(move || {
                                                status.set("ui-back-selected".to_owned())
                                            });
                                    })
                                    .action(move || {
                                        if filled_action.get() {
                                            icon_button()
                                                .filled()
                                                .icon(filled::SEARCH_SVG)
                                                .accessibility_label(I18N.format(
                                                    "ui-value-search",
                                                    [("v0", I18N.text(&label.get()).into())],
                                                ))
                                                .on_click(move || {
                                                    status.set("ui-search-selected".to_owned())
                                                });
                                        } else {
                                            icon_button()
                                                .standard()
                                                .color_style(IconButtonColorStyle::Inherited)
                                                .icon(filled::SEARCH_SVG)
                                                .accessibility_label(I18N.format(
                                                    "ui-value-search",
                                                    [("v0", I18N.text(&label.get()).into())],
                                                ))
                                                .on_click(move || {
                                                    status.set("ui-search-selected".to_owned())
                                                });
                                        }
                                    });
                            });
                    });
                    if controls.unwrap_or(false) {
                        scaffold()
                            .modifier(Modifier::new().fill_max_width().height(Dp(360.0)))
                            .top_bar_shared(bar)
                            .content(move || {
                                column()
                                    .modifier(
                                        Modifier::new()
                                            .fill_max_size()
                                            .vertical_scroll(content)
                                            .semantics(SemanticsArgs {
                                                role: Some(Role::Group),
                                                label: Some(I18N.format(
                                                    "ui-value-scroll-content",
                                                    [("v0", I18N.text(&label.get()).into())],
                                                )),
                                                merge_descendants: true,
                                                ..Default::default()
                                            }),
                                    )
                                    .children(move || {
                                        for index in 0..20 {
                                            text(I18N.format(
                                                "ui-saved-item-value",
                                                [("v0", (index + 1).into())],
                                            ))
                                            .style(type_style(16.0, 24.0))
                                            .modifier(
                                                Modifier::new()
                                                    .fill_max_width()
                                                    .padding_all(Dp(16.0)),
                                            );
                                        }
                                    });
                            });
                    } else {
                        bar.render();
                    }
                });
            if controls.unwrap_or(false) {
                gap(Dp(16.0));
                text(I18N.format(
                    "ui-value-action-value",
                    [
                        ("v0", I18N.text(&label.get()).into()),
                        ("v1", I18N.text(&status.get()).into()),
                    ],
                ))
                .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn brand() {
    let dark = use_context::<MaterialTheme>()
        .unwrap()
        .with(|theme| theme.color_scheme.is_dark);
    let asset = if dark {
        crate::res::LOGO_DARK_PNG
    } else {
        crate::res::LOGO_PNG
    };
    boxed()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Image),
            label: Some(I18N.text("ui-tessera-brand")),
            ..Default::default()
        }))
        .children(move || {
            icon().painter(asset).size(Dp(32.0));
        });
}

#[tessera]
fn search_avatar(status: State<String>, geometry: Option<State<Geometry>>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    button()
        .text()
        .shape(Shape::Ellipse)
        .padding(Dp(8.0))
        .modifier(
            geometry
                .map(|geometry| watch(geometry, "trailing"))
                .unwrap_or_default()
                .size(Dp(48.0), Dp(48.0)),
        )
        .accessibility_label(I18N.text("ui-search-app-bar-account"))
        .on_click(move || status.set("ui-account-selected".into()))
        .child(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(32.0), Dp(32.0))
                        .background_with_shape(scheme.primary, Shape::Ellipse),
                )
                .children(move || {
                    text("A")
                        .color(scheme.on_primary)
                        .style(type_style(24.0, 32.0));
                });
        });
}

#[tessera]
fn search_sample(
    geometry: Option<State<Geometry>>,
    centered: Option<bool>,
    specimen: Option<usize>,
    #[prop(into)] placeholder: Option<String>,
    bright: Option<bool>,
    logo: Option<bool>,
    scrolled: Option<bool>,
) {
    let controller = remember(|| SearchBarController::new(false));
    let query = remember(String::new);
    let opened = remember(DialogController::default);
    let width = remember(|| Dp(600.0));
    let status = remember(String::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let bright = bright.unwrap_or(false);
    let scrolled = scrolled.unwrap_or(false);
    let bar_color = if bright || scrolled {
        scheme.surface_container
    } else {
        scheme.surface
    };
    let field_color = if bright {
        scheme.surface_bright
    } else if scrolled {
        scheme.surface_container_highest
    } else {
        scheme.surface_container
    };
    let content_color = scheme.on_surface_variant;
    let divider = scheme.outline;
    let logo = logo.unwrap_or(false);
    let mic = matches!(specimen, Some(1 | 3 | 5 | 6));
    let outside = match specimen {
        Some(1 | 4) => 1,
        Some(5) => 3,
        Some(6) => 4,
        _ => 0,
    };
    let hint = match (specimen, placeholder) {
        (Some(0), _) => String::new(),
        (_, Some(value)) => value,
        (Some(_), None) => "Body Large".to_owned(),
        (None, None) => "ui-search-library".to_owned(),
    };
    dialog_provider()
        .accessibility_label(I18N.text("ui-search-library-view"))
        .full_screen(true)
        .controller(opened)
        .on_close_request(move || opened.with_mut(DialogController::close))
        .main_content(move || {
            let hint = hint.clone();
            let centered = centered.unwrap_or(false);
            boxed()
                .alignment(if centered {
                    Alignment::Center
                } else {
                    Alignment::TopStart
                })
                .modifier(if centered {
                    Modifier::new().fill_max_width().height(Dp(320.0))
                } else {
                    Modifier::new().fill_max_width()
                })
                .children(move || {
                    let hint = hint.clone();
                    let modifier = geometry
                        .map(|geometry| watch(geometry, "container"))
                        .unwrap_or_default()
                        .semantics(SemanticsArgs {
                            role: Some(Role::Toolbar),
                            label: Some(I18N.text("ui-search-app-bar")),
                            merge_descendants: true,
                            ..Default::default()
                        })
                        .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
                            let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                            if width.get() != next {
                                width.set(next);
                            }
                        });
                    app_bar()
                        .modifier(modifier)
                        .container_color(bar_color)
                        .content_color(content_color)
                        .content_padding(Padding::symmetric(Dp(4.0), Dp(0.0)))
                        .content(move || {
                            let hint = hint.clone();
                            if logo {
                                boxed()
                                    .alignment(Alignment::Center)
                                    .modifier(
                                        geometry
                                            .map(|geometry| watch(geometry, "leading"))
                                            .unwrap_or_default()
                                            .size(Dp(48.0), Dp(48.0)),
                                    )
                                    .children(|| {
                                        brand();
                                    });
                            } else {
                                icon_button()
                                    .modifier(
                                        geometry
                                            .map(|geometry| watch(geometry, "leading"))
                                            .unwrap_or_default(),
                                    )
                                    .standard()
                                    .color_style(IconButtonColorStyle::Inherited)
                                    .icon(if specimen == Some(0) {
                                        filled::MORE_VERT_SVG
                                    } else {
                                        filled::MENU_SVG
                                    })
                                    .accessibility_label(I18N.text("ui-search-app-bar-menu"))
                                    .on_click(move || status.set("ui-menu-selected".into()));
                            }
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(Modifier::new().weight(1.0))
                                .children(move || {
                                    let available =
                                        (width.get().0 - 120.0 - 52.0 * outside as f64).max(0.0);
                                    let field_width = if specimen.is_some() || available <= 312.0 {
                                        available
                                    } else {
                                        312.0 + (available - 312.0) * 0.5
                                    };
                                    let leading_icon = RenderSlot::new(|| {
                                        icon().painter(filled::SEARCH_SVG).size(Dp(24.0));
                                    });
                                    let trailing_icon = RenderSlot::new(|| {
                                        icon().painter(filled::MIC_SVG).size(Dp(24.0));
                                    });
                                    search_bar()
                                        .modifier(
                                            geometry
                                                .map(|geometry| watch(geometry, "search"))
                                                .unwrap_or_default()
                                                .width(Dp(field_width)),
                                        )
                                        .controller(controller)
                                        .read_only(true)
                                        .colors(SearchBarColors {
                                            container_color: field_color,
                                            divider_color: divider,
                                        })
                                        .placeholder(I18N.text(&(hint)))
                                        .on_active_change(move |active| {
                                            if active {
                                                opened.with_mut(DialogController::open);
                                            }
                                        })
                                        .leading_icon_optional(
                                            specimen.is_none().then_some(leading_icon),
                                        )
                                        .trailing_icon_optional(mic.then_some(trailing_icon));
                                    if specimen == Some(0) {
                                        boxed()
                                            .alignment(Alignment::Center)
                                            .modifier(
                                                Modifier::new().size(Dp(field_width), Dp(56.0)),
                                            )
                                            .children(move || {
                                                text("Body Large").style(type_style(16.0, 24.0));
                                            });
                                    }
                                });
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            for (glyph, label, selected) in [
                                (
                                    filled::AUTO_AWESOME_SVG,
                                    "ui-search-app-bar-suggestions",
                                    "ui-suggestions-selected",
                                ),
                                (
                                    filled::NOTIFICATIONS_SVG,
                                    "ui-search-app-bar-notifications",
                                    "ui-notifications-selected",
                                ),
                                (
                                    filled::SETTINGS_SVG,
                                    "ui-search-app-bar-settings",
                                    "ui-settings-selected",
                                ),
                                (
                                    filled::HELP_OUTLINE_SVG,
                                    "ui-search-app-bar-help",
                                    "ui-help-selected",
                                ),
                            ]
                            .into_iter()
                            .take(outside)
                            {
                                icon_button()
                                    .standard()
                                    .color_style(IconButtonColorStyle::Inherited)
                                    .icon(glyph)
                                    .accessibility_label(I18N.text(label))
                                    .on_click(move || status.set(selected.to_owned()));
                                spacer().modifier(Modifier::new().width(Dp(4.0)));
                            }
                            if specimen.is_some() {
                                search_avatar(status).geometry_optional(geometry);
                            } else {
                                icon_button()
                                    .modifier(
                                        geometry
                                            .map(|geometry| watch(geometry, "trailing"))
                                            .unwrap_or_default(),
                                    )
                                    .standard()
                                    .color_style(IconButtonColorStyle::Inherited)
                                    .icon(filled::ACCOUNT_CIRCLE_SVG)
                                    .accessibility_label(I18N.text("ui-search-app-bar-account"))
                                    .on_click(move || status.set("ui-account-selected".into()));
                            }
                        });
                });
        })
        .dialog_content(move |_| {
            let initial_query = query.get();
            let view = remember(move || {
                let mut view = SearchBarController::new(true);
                view.set_query(initial_query);
                view
            });
            boxed()
                .modifier(Modifier::new().fill_max_width().height(Dp(400.0)))
                .children(move || {
                    search_bar()
                        .controller(view)
                        .on_active_change(move |active| {
                            opened.with_mut(|opened| {
                                if active {
                                    opened.open();
                                } else {
                                    opened.close();
                                }
                            })
                        })
                        .placeholder(I18N.text("ui-search-library"))
                        .leading_icon(move || {
                            icon_button()
                                .standard()
                                .icon(filled::ARROW_BACK_SVG)
                                .accessibility_label(I18N.text("ui-close-app-bar-search"))
                                .on_click(move || opened.with_mut(DialogController::close));
                        })
                        .on_query_change(move |value: String| {
                            query.set(value.clone());
                            value
                        })
                        .content(move || {
                            column()
                                .modifier(Modifier::new().padding_all(Dp(16.0)))
                                .children(move || {
                                    text(if query.get().is_empty() {
                                        I18N.text("ui-search-saved-items")
                                    } else {
                                        I18N.format(
                                            "ui-results-for-value",
                                            [("v0", query.get().into())],
                                        )
                                    });
                                });
                        });
                });
        });
    if !status.get().is_empty() {
        text(I18N.text(status.get())).style(type_style(14.0, 20.0));
    }
}

#[tessera]
fn variants_figure(#[prop(into)] prefix: String) {
    panel(RenderSlot::new(move || {
        search_sample();
        gap(Dp(32.0));
        for (variant, label) in [
            (TopAppBarVariant::Small, "ui-small"),
            (TopAppBarVariant::MediumFlexible, "ui-medium-flexible"),
            (TopAppBarVariant::LargeFlexible, "ui-large-flexible"),
        ] {
            sample(
                I18N.format(
                    "ui-value-value-format-2",
                    [
                        ("v0", I18N.text(&prefix).into()),
                        ("v1", I18N.text(&label).into()),
                    ],
                ),
                variant,
            );
            gap(Dp(32.0));
        }
    }));
}

#[tessera]
fn baseline_figure(#[prop(into)] prefix: String) {
    panel(RenderSlot::new(move || {
        baseline_bar(false)
            .label(I18N.format("ui-value-medium", [("v0", I18N.text(&prefix).into())]));
        gap(Dp(32.0));
        baseline_bar(true)
            .label(I18N.format("ui-value-large", [("v0", I18N.text(&prefix).into())]));
    }));
    gap(Dp(16.0));
    caption(
        "ui-baseline-reference-drawings-a-64dp-top-row-with-the-headline-in-a-fixed-height-area-below",
    );
}

#[tessera]
fn baseline_bar(large: bool, #[prop(into)] label: Option<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let label = label.unwrap_or_else(|| "ui-baseline-app-bar".to_owned());
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(if large { 152.0 } else { 112.0 }))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(0.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Toolbar),
                    label: Some(I18N.text(&(label))),
                    ..Default::default()
                }),
        )
        .children(move || {
            let label = label.clone();
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(64.0))
                        .padding_symmetric(Dp(4.0), Dp(0.0)),
                )
                .children(move || {
                    icon_button()
                        .standard()
                        .icon(filled::ARROW_BACK_SVG)
                        .accessibility_label(
                            I18N.format("ui-value-back", [("v0", I18N.text(&label).into())]),
                        );
                    spacer().modifier(Modifier::new().weight(1.0));
                    for (glyph, name) in [
                        (filled::SEARCH_SVG, "search"),
                        (filled::EVENT_SVG, "calendar"),
                        (filled::MORE_VERT_SVG, "more"),
                    ] {
                        icon_button()
                            .standard()
                            .icon(glyph)
                            .accessibility_label(I18N.format(
                                "ui-value-value-format-2",
                                [
                                    ("v0", I18N.text(&label).into()),
                                    ("v1", I18N.text(&name).into()),
                                ],
                            ));
                    }
                });
            text(I18N.text(if large {
                "Headline Medium"
            } else {
                "Headline Small"
            }))
            .color(scheme.on_surface)
            .style(if large {
                type_style(28.0, 36.0)
            } else {
                type_style(24.0, 32.0)
            })
            .modifier(
                Modifier::new()
                    .align(Alignment::BottomStart)
                    .padding(Padding::new(
                        Dp(16.0),
                        Dp(0.0),
                        Dp(16.0),
                        Dp(if large { 28.0 } else { 24.0 }),
                    )),
            );
        });
}

#[tessera]
fn variants() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            variants_figure("ui-variant");
            gap(Dp(16.0));
            numbered(vec![
                "ui-search-app-bar".into(),
                "ui-small".into(),
                "ui-medium-flexible".into(),
                "ui-large-flexible".into(),
            ]);
            gap(Dp(48.0));
            subheading("ui-baseline-variants".to_owned());
            gap(Dp(16.0));
            paragraph("ui-the-baseline-m3-medium-and-large-app-bars-are-no-longer-recommended-in-m3-expressive-and-s".into());
            gap(Dp(24.0));
            baseline_figure("ui-baseline");
            gap(Dp(16.0));
            numbered(vec!["ui-medium".into(), "ui-large".into()]);
        });
}

#[tessera]
fn configurations() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            subheading("ui-text-alignment".to_owned());
            gap(Dp(24.0));
            panel(RenderSlot::new(move || {
                for centered in [false, true] {
                    sample(
                        I18N.format("ui-value-alignment", [("v0", I18N.text(&(if centered { "ui-centered" } else { "ui-leading" })).into())]),
                        TopAppBarVariant::MediumFlexible,
                    )
                    .centered(centered)
                    .subtitle(true);
                    gap(Dp(32.0));
                }
            }));
            gap(Dp(16.0));
            caption("ui-text-labels-including-supporting-text-can-be-aligned-to-the-leading-edge-or-centered");
            gap(Dp(112.0));
            subheading("ui-interactive-configuration".to_owned());
            gap(Dp(24.0));
            panel(RenderSlot::new(move || {
                sample("ui-configured-app-bar", TopAppBarVariant::MediumFlexible)
                    .subtitle(true)
                    .controls(true);
            }));
        });
}

const COMMON_TOKENS: &[(&str, &str)] = &[
    ("ui-app-bar-container-color", "#FEF7FF (Surface)"),
    ("ui-search-container-color", "#F3EDF7 (Surface container)"),
    ("ui-search-label-color", "#49454F (On surface variant)"),
    (
        "ui-app-bar-container-color-on-scroll",
        "#F3EDF7 (Surface container)",
    ),
    (
        "ui-search-container-color-on-scroll",
        "#E6E0E9 (Surface container highest)",
    ),
    ("ui-app-bar-container-elevation", "0dp"),
    ("ui-app-bar-container-elevation-on-scroll", "3dp"),
    ("ui-app-bar-title-text", "#1D1B20 (On surface)"),
    ("ui-app-bar-subtitle-text", "#49454F (On surface variant)"),
    ("ui-app-bar-leading-icon", "#1D1B20 (On surface)"),
    ("ui-app-bar-trailing-icon", "#49454F (On surface variant)"),
    ("ui-app-bar-left-padding", "4dp"),
    ("ui-app-bar-right-padding", "4dp"),
    ("ui-app-bar-icon-spacing", "0dp"),
    ("ui-search-left-padding", "8dp"),
    ("ui-search-right-padding", "8dp"),
    ("ui-app-bar-container-shape", "ui-rounded-corners-0dp"),
    ("ui-app-bar-avatar-size", "32dp"),
    ("ui-app-bar-icon-size", "24dp"),
];

const SMALL_TOKENS: &[(&str, &str)] = &[
    ("ui-app-bar-small-container-height", "64dp"),
    (
        "ui-app-bar-small-title-font",
        "ui-roboto-22pt-line-height-28pt-weight-400-tracking-0pt",
    ),
    (
        "ui-app-bar-small-subtitle-font",
        "ui-roboto-12pt-line-height-16pt-weight-500-tracking-0-5pt",
    ),
    ("ui-app-bar-small-icon-button-size", "24dp"),
    ("ui-search-container-height", "56dp"),
    ("ui-search-container-shape", "ui-circular"),
    (
        "ui-search-title-font",
        "ui-roboto-16pt-line-height-24pt-weight-400-tracking-0-5pt",
    ),
];

const MEDIUM_FLEXIBLE_TOKENS: &[(&str, &str)] = &[
    ("ui-app-bar-medium-flexible-container-height", "112dp"),
    (
        "ui-app-bar-medium-flexible-container-height-with-subtitle",
        "136dp",
    ),
    (
        "ui-app-bar-medium-title-font",
        "ui-roboto-28pt-line-height-36pt-weight-400-tracking-0pt",
    ),
    (
        "ui-app-bar-medium-subtitle-font",
        "ui-roboto-14pt-line-height-20pt-weight-500-tracking-0-1pt",
    ),
];

const LARGE_FLEXIBLE_TOKENS: &[(&str, &str)] = &[
    ("ui-app-bar-large-flexible-container-height", "120dp"),
    (
        "ui-app-bar-large-flexible-container-height-with-subtitle",
        "152dp",
    ),
    (
        "ui-app-bar-large-title-font",
        "ui-roboto-36pt-line-height-44pt-weight-400-tracking-0pt",
    ),
    (
        "ui-app-bar-large-subtitle-font",
        "ui-roboto-16pt-line-height-24pt-weight-500-tracking-0-15pt",
    ),
];

const SEARCH_BAR_TOKENS: &[(&str, &str)] = &[
    (
        "ui-search-bar-container-color",
        "#ECE6F0 (Surface container high)",
    ),
    ("ui-search-bar-container-elevation", "6dp"),
    ("ui-search-bar-leading-icon-color", "#1D1B20 (On surface)"),
    (
        "ui-search-bar-trailing-icon-color",
        "#49454F (On surface variant)",
    ),
    (
        "ui-search-bar-supporting-text-color",
        "#49454F (On surface variant)",
    ),
    ("ui-search-bar-input-text-color", "#1D1B20 (On surface)"),
    (
        "ui-search-bar-hover-state-layer-color",
        "#1D1B20 (On surface)",
    ),
    ("ui-search-bar-hover-state-layer-opacity", "0.08"),
    (
        "ui-search-bar-pressed-state-layer-color",
        "#1D1B20 (On surface)",
    ),
    ("ui-search-bar-contained-pane-leading-margin", "24dp"),
    ("ui-search-bar-contained-pane-trailing-margin", "24dp"),
    (
        "ui-search-bar-contained-motion",
        "ui-damping-0-9-stiffness-1400",
    ),
    ("ui-search-bar-contained-leading-space", "4dp"),
    ("ui-search-bar-contained-trailing-space", "4dp"),
    ("ui-search-bar-contained-no-actions-leading-space", "16dp"),
    ("ui-search-bar-contained-no-actions-trailing-space", "16dp"),
];

fn token_rows(rows: &[(&str, &str)]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|(token, value)| vec![(*token).to_owned(), (*value).to_owned()])
        .collect()
}

#[tessera]
fn tokens() {
    paragraph("ui-select-a-token-set-to-view-in-the-table-s-menu-app-bar-token-sets-are-organized-into-a-com".into());
    for (title, rows) in [
        ("ui-app-bar-common", COMMON_TOKENS),
        ("ui-app-bar-size-small", SMALL_TOKENS),
        ("ui-app-bar-size-medium-flexible", MEDIUM_FLEXIBLE_TOKENS),
        ("ui-app-bar-size-large-flexible", LARGE_FLEXIBLE_TOKENS),
    ] {
        gap(Dp(32.0));
        text(I18N.text(title)).style(type_style(16.0, 24.0));
        gap(Dp(16.0));
        spec_table(vec!["ui-token".into(), "ui-value".into()], token_rows(rows));
    }
    gap(Dp(48.0));
    subheading("ui-search-component-tokens-specs".to_owned());
    gap(Dp(16.0));
    paragraph("ui-the-default-search-component-tokens-are-used-in-the-search-app-bar-inside-an-app-bar-the-s".into());
    gap(Dp(24.0));
    text(I18N.text("ui-search-bar-enabled-and-contained-expressive")).style(type_style(16.0, 24.0));
    gap(Dp(16.0));
    spec_table(
        vec!["ui-token".into(), "ui-value".into()],
        token_rows(SEARCH_BAR_TOKENS),
    );
}

#[tessera]
fn anatomy() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            combined_app_anatomy();
            gap(Dp(32.0));
            paragraph("ui-app-bars-can-be-customized-to-include".into());
            gap(Dp(8.0));
            bullets(vec![
                "ui-an-image-or-logo".into(),
                "ui-a-subtitle".into(),
                "ui-a-filled-icon-button".into(),
            ]);
            gap(Dp(8.0));
            paragraph("ui-avoid-customizing-the-size-of-the-heading-and-subtitle-or-adding-too-many-actions".into());
            gap(Dp(24.0));
            panel(RenderSlot::new(|| {
                for kind in [LOGO, SUBTITLED, FILLED_SINGLE] {
                    specimen(kind);
                    gap(Dp(24.0));
                }
            }));
            gap(Dp(16.0));
            caption("ui-the-app-bar-can-have-different-layouts-depending-on-which-elements-are-shown");
            gap(Dp(112.0));
            subheading("ui-search".to_owned());
            gap(Dp(16.0));
            paragraph("ui-the-search-app-bar-can-include-trailing-actions-inside-and-outside-the-search-bar-when-the".into());
            gap(Dp(24.0));
            search_figure(false);
            gap(Dp(32.0));
            panel(RenderSlot::new(|| {
                for kind in [2_usize, 3, 4] {
                    search_sample().specimen(kind);
                    gap(Dp(24.0));
                }
            }));
            gap(Dp(16.0));
            numbered(vec![
                "ui-a-leading-element-and-a-trailing-element-outside-search".into(),
                "ui-a-leading-element-a-trailing-element-inside-search-and-a-trailing-element-outside-search".into(),
                "ui-a-leading-element-and-two-trailing-elements-outside-search".into(),
            ]);
            gap(Dp(112.0));
            subheading("ui-image".to_owned());
            gap(Dp(16.0));
            paragraph("ui-an-image-can-be-placed-in-the-app-bar-in-small-app-bars-this-can-replace-the-label-text".into());
            gap(Dp(24.0));
            panel(RenderSlot::new(|| {
                sample("ui-image", TopAppBarVariant::Small).logo(true);
            }));
            gap(Dp(16.0));
            caption("ui-images-can-be-added-to-app-bars-and-can-replace-label-text-on-small-app-bars");
            gap(Dp(112.0));
            subheading("ui-filled-trailing-icon-button".to_owned());
            gap(Dp(16.0));
            paragraph("ui-the-app-bar-s-trailing-icon-buttons-can-be-replaced-with-a-single-primary-or-tonal-filled".into());
            gap(Dp(24.0));
            app_figure(TopAppBarVariant::Small, false).filled_action(true);
            gap(Dp(16.0));
            caption("ui-the-trailing-icons-can-be-configured-to-be-a-single-filled-icon-button");
            gap(Dp(112.0));
            subheading("ui-subtitle".to_owned());
            gap(Dp(24.0));
            panel(RenderSlot::new(move || {
                for (variant, name) in [
                    (TopAppBarVariant::Small, "ui-small"),
                    (TopAppBarVariant::MediumFlexible, "ui-medium-flexible"),
                    (TopAppBarVariant::LargeFlexible, "ui-large-flexible"),
                ] {
                    sample(I18N.format("ui-without-subtitle-value", [("v0", I18N.text(&name).into())]), variant);
                    gap(Dp(24.0));
                    sample(I18N.format("ui-subtitle-value", [("v0", I18N.text(&name).into())]), variant).subtitle(true);
                    gap(Dp(32.0));
                }
            }));
            gap(Dp(16.0));
            caption("ui-the-medium-flexible-and-large-flexible-app-bars-hug-the-text-contents-so-they-are-taller-w");
            gap(Dp(8.0));
            numbered(vec![
                "ui-small".into(),
                "ui-small-with-subtitle".into(),
                "ui-medium-flexible".into(),
                "ui-medium-flexible-with-subtitle".into(),
                "ui-large-flexible".into(),
                "ui-large-flexible-with-subtitle".into(),
            ]);
        });
}

#[tessera]
fn colors() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            paragraph("ui-color-values-are-implemented-through-design-tokens-for-design-this-means-working-with-colo".into());
            gap(Dp(16.0));
            paragraph("ui-all-app-bars-share-the-same-color-roles-on-scroll-the-container-changes-color-to-surface-c".into());
            gap(Dp(24.0));
            for dark in [false, true] {
                material_theme()
                    .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                    .child(move || {
                        color_figure(false, dark);
                    });
                gap(Dp(24.0));
            }
            caption("ui-app-bar-color-roles-used-for-light-and-dark-themes");
            gap(Dp(8.0));
            numbered(vec![
                "ui-container-surface".into(),
                "ui-leading-icon-on-surface".into(),
                "ui-trailing-icons-on-surface-variant".into(),
                "ui-headline-on-surface".into(),
                "ui-subtitle-on-surface-variant".into(),
                "ui-container-on-scroll-surface-container".into(),
            ]);
            gap(Dp(48.0));
            for dark in [false, true] {
                material_theme()
                    .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                    .child(move || {
                        color_figure(true, dark);
                    });
                gap(Dp(24.0));
            }
            caption("ui-search-app-bar-color-roles-used-for-light-and-dark-themes");
            gap(Dp(8.0));
            numbered(vec![
                "ui-container-surface".into(),
                "ui-leading-icon-on-surface-variant".into(),
                "ui-trailing-icon-on-surface-variant".into(),
                "ui-hinted-search-text-on-surface-variant".into(),
                "ui-search-container-surface-container".into(),
                "ui-container-on-scroll-surface-container".into(),
                "ui-search-container-on-scroll-surface-container-highest".into(),
            ]);
            gap(Dp(88.0));
            subheading("ui-scroll-states".to_owned());
            gap(Dp(24.0));
            guidance_pair("ui-flat", "ui-on-scroll")
                .first(|| {
                    scroll_state_column(false);
                })
                .second(|| {
                    scroll_state_column(true);
                });
            gap(Dp(16.0));
            caption("ui-the-app-bar-changes-color-when-flat-or-on-scroll-the-search-bar-can-also-change-color-on-s");
        });
}

const COLOR_FIGURE_GAP: f64 = 72.0;

#[tessera]
fn color_figure(search: bool, dark: bool) {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(
                        I18N.format(
                            "ui-value-color-roles-value-scheme",
                            [
                                (
                                    "v0",
                                    I18N.text(
                                        &(if search {
                                            "ui-search-app-bar"
                                        } else {
                                            "ui-app-bar"
                                        }),
                                    )
                                    .into(),
                                ),
                                (
                                    "v1",
                                    I18N.text(
                                        &(if dark {
                                            "ui-dark-7069"
                                        } else {
                                            "ui-light-7070"
                                        }),
                                    )
                                    .into(),
                                ),
                            ],
                        ),
                    ),
                    ..Default::default()
                }),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                    Dp(96.0),
                    Dp(88.0),
                    Dp(36.0),
                    Dp(72.0),
                )))
                .children(move || {
                    if search {
                        search_sample()
                            .geometry(geometry)
                            .specimen(3)
                            .placeholder(I18N.text("ui-search-product"));
                        gap(Dp(COLOR_FIGURE_GAP));
                        search_sample()
                            .specimen(3)
                            .placeholder(I18N.text("ui-search-product"))
                            .scrolled(true);
                    } else {
                        theme_sample_bar(false).geometry(geometry);
                        gap(Dp(COLOR_FIGURE_GAP));
                        theme_sample_bar(true);
                    }
                });
            color_annotations(geometry, search, dark);
        });
}

fn leader(svg: &mut String, marker: (f64, f64), target: (f64, f64), ink: &str) {
    let (x, y) = marker;
    let (target_x, target_y) = target;
    if y + 32.0 <= target_y {
        dotted(svg, x + 12.0, y + 32.0, target_y - y - 32.0, true, ink);
    } else if y >= target_y {
        dotted(svg, x + 12.0, target_y, y - target_y, true, ink);
    } else {
        dotted(svg, x + 24.0, y + 16.0, target_x - x - 24.0, false, ink);
    }
}

#[tessera]
fn color_annotations(geometry: State<Geometry>, search: bool, dark: bool) {
    let values = geometry.get();
    let (Some(root), Some(container), Some(leading), Some(trailing)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "leading"),
        relative(&values, if search { "search" } else { "trailing" }),
    ) else {
        return;
    };
    let title = relative(&values, "title");
    let ink = if dark { "#cac4d0" } else { "#49454f" };
    let above = container.y - 72.0;
    let below = container.y + container.h + 20.0;
    let left = |rect: Rect| (rect.x - 68.0, rect.y + rect.h * 0.5 - 16.0);
    let icon_top = |rect: Rect| rect.y + rect.h * 0.5 - 12.0;
    let scrolled = Rect {
        y: container.y + container.h + COLOR_FIGURE_GAP,
        ..container
    };
    let leading_x = leading.x + leading.w * 0.5;
    let mut callouts = vec![
        (
            1,
            left(container),
            (container.x, container.y + container.h * 0.5),
        ),
        (2, (leading_x - 12.0, above), (leading_x, icon_top(leading))),
    ];
    if search {
        let field = trailing;
        let mic_x = field.x + field.w - 28.0;
        let hint_x = field.x + 48.0;
        let field_x = field.x + field.w * 0.5;
        let offset = COLOR_FIGURE_GAP + container.h;
        callouts.extend([
            (3, (mic_x - 12.0, above), (mic_x, icon_top(field))),
            (
                4,
                (hint_x - 12.0, below),
                (hint_x, field.y + field.h * 0.5 + 12.0),
            ),
            (5, (field_x - 12.0, below), (field_x, field.y + field.h)),
            (
                6,
                left(scrolled),
                (scrolled.x, scrolled.y + scrolled.h * 0.5),
            ),
            (
                7,
                (field_x - 12.0, below + offset),
                (field_x, field.y + field.h + offset),
            ),
        ]);
    } else {
        let Some(title) = title else {
            return;
        };
        let trailing_x = trailing.x + trailing.w * 0.5;
        let headline_x = title.x + 40.0;
        let subtitle_x = title.x + 24.0;
        callouts.extend([
            (
                3,
                (trailing_x - 12.0, above),
                (trailing_x, icon_top(trailing)),
            ),
            (4, (headline_x - 12.0, above), (headline_x, title.y + 4.0)),
            (
                5,
                (subtitle_x - 12.0, below),
                (subtitle_x, title.y + title.h),
            ),
            (
                6,
                left(scrolled),
                (scrolled.x, scrolled.y + scrolled.h * 0.5),
            ),
        ]);
    }
    let mut svg = String::new();
    for (number, marker, target) in callouts {
        leader(&mut svg, marker, target, ink);
        annotations::marker(number, Dp(marker.0), Dp(marker.1));
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn theme_sample_bar(scrolled: bool, geometry: Option<State<Geometry>>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let tracked = move |name: &'static str| {
        geometry
            .map(|geometry| watch(geometry, name))
            .unwrap_or_default()
    };
    top_app_bar()
        .modifier(tracked("container"))
        .variant(TopAppBarVariant::Small)
        .title(I18N.text("ui-headline"))
        .subtitle(I18N.text("ui-subtitle"))
        .title_area_modifier(tracked("title"))
        .container_color(if scrolled {
            scheme.surface_container
        } else {
            scheme.surface
        })
        .navigation_icon(move || {
            icon_button()
                .modifier(tracked("leading"))
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(filled::ARROW_BACK_SVG)
                .accessibility_label(I18N.text("ui-color-figure-back"));
        })
        .action(move || {
            icon_button()
                .modifier(tracked("trailing"))
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(filled::SEARCH_SVG)
                .accessibility_label(I18N.text("ui-color-figure-search"));
        })
        .action(move || {
            icon_button()
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(filled::EVENT_SVG)
                .accessibility_label(I18N.text("ui-color-figure-calendar"));
        });
}

#[tessera]
fn scroll_state_column(scrolled: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let container = if scrolled {
        scheme.surface_container
    } else {
        scheme.surface
    };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            top_app_bar()
                .title(I18N.text("ui-product"))
                .title_alignment(AppBarTitleAlignment::Center)
                .container_color(container)
                .navigation_icon(move || {
                    icon_button()
                        .standard()
                        .color_style(IconButtonColorStyle::Inherited)
                        .icon(filled::MENU_SVG)
                        .accessibility_label(I18N.text("ui-scroll-state-menu"));
                })
                .action(move || {
                    icon_button()
                        .standard()
                        .color_style(IconButtonColorStyle::Inherited)
                        .icon(filled::ACCOUNT_CIRCLE_SVG)
                        .accessibility_label(I18N.text("ui-scroll-state-account"));
                });
            gap(Dp(16.0));
            search_sample()
                .specimen(3)
                .placeholder(I18N.text("ui-search-product"))
                .scrolled(scrolled);
            gap(Dp(16.0));
            for (variant, title) in [
                (TopAppBarVariant::Small, "Title large"),
                (TopAppBarVariant::MediumFlexible, "Headline medium"),
                (TopAppBarVariant::LargeFlexible, "Display small"),
            ] {
                top_app_bar()
                    .variant(variant)
                    .title(I18N.text(title))
                    .container_color(container)
                    .navigation_icon(move || {
                        icon_button()
                            .standard()
                            .color_style(IconButtonColorStyle::Inherited)
                            .icon(filled::ARROW_BACK_SVG)
                            .accessibility_label(I18N.text("ui-scroll-state-back"));
                    })
                    .action(move || {
                        icon_button()
                            .standard()
                            .color_style(IconButtonColorStyle::Inherited)
                            .icon(filled::SEARCH_SVG)
                            .accessibility_label(I18N.text("ui-scroll-state-search"));
                    });
                gap(Dp(16.0));
            }
        });
}

#[tessera]
fn measurements() {
    subheading("ui-search-app-bar".to_owned());
    gap(Dp(24.0));
    search_figure(true);
    gap(Dp(48.0));
    for (variant, name) in [
        (TopAppBarVariant::Small, "ui-small-app-bar"),
        (
            TopAppBarVariant::MediumFlexible,
            "ui-medium-flexible-app-bar",
        ),
        (TopAppBarVariant::LargeFlexible, "ui-large-flexible-app-bar"),
    ] {
        subheading(name.to_owned());
        gap(Dp(24.0));
        app_figure(variant, true).name(name);
        gap(Dp(48.0));
    }
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-small-height".into(), "64dp".into()],
            vec!["ui-medium-flexible-height".into(), "112dp".into()],
            vec!["ui-medium-flexible-with-subtitle".into(), "136dp".into()],
            vec!["ui-large-flexible-height".into(), "120dp".into()],
            vec!["ui-large-flexible-with-subtitle".into(), "152dp".into()],
            vec!["ui-navigation-and-action-target".into(), "48dp".into()],
            vec!["ui-icon-size".into(), "24dp".into()],
            vec!["ui-title-inset-without-navigation".into(), "16dp".into()],
            vec!["ui-search-field-height".into(), "56dp".into()],
            vec!["ui-search-field-growth-threshold".into(), "312dp".into()],
        ],
    );
    gap(Dp(24.0));
    paragraph("ui-flexible-app-bars-hug-their-text-contents-so-a-subtitle-or-a-wrapping-headline-can-make-th".into());
}

#[tessera]
fn baseline_app_bars() {
    paragraph("ui-the-medium-and-large-app-bars-are-no-longer-recommended-in-m3-expressive-use-the-medium-fl".into());
    gap(Dp(24.0));
    baseline_figure("ui-baseline-section");
    gap(Dp(24.0));
    paragraph("ui-medium-and-large-app-bars-have-the-same-elements".into());
    gap(Dp(8.0));
    numbered(vec![
        "ui-container".into(),
        "ui-leading-button".into(),
        "ui-trailing-icons".into(),
        "ui-headline".into(),
    ]);
    gap(Dp(48.0));
    subheading("ui-tokens-specs".to_owned());
    gap(Dp(16.0));
    paragraph("ui-baseline-app-bar-token-sets-are-organized-into-medium-large-and-older-baseline-token-sets".into());
    gap(Dp(24.0));
    for (title, rows) in [
        (
            "ui-app-bar-size-medium-baseline",
            [
                ("ui-app-bar-medium-container-height", "112dp"),
                (
                    "ui-app-bar-medium-title-font-603",
                    "ui-roboto-24pt-line-height-32pt-weight-400-tracking-0pt",
                ),
                ("ui-app-bar-medium-icon-button-size", "24dp"),
                (
                    "ui-app-bar-medium-subtitle-font-606",
                    "ui-roboto-14pt-line-height-20pt-weight-500-tracking-0-1pt",
                ),
            ],
        ),
        (
            "ui-app-bar-size-large-baseline",
            [
                ("ui-app-bar-large-container-height", "152dp"),
                (
                    "ui-app-bar-large-title-font-609",
                    "ui-roboto-28pt-line-height-36pt-weight-400-tracking-0pt",
                ),
                ("ui-app-bar-large-icon-button-size", "24dp"),
                (
                    "ui-app-bar-large-subtitle-font-611",
                    "ui-roboto-16pt-line-height-24pt-weight-500-tracking-0-15pt",
                ),
            ],
        ),
    ] {
        text(I18N.text(title)).style(type_style(16.0, 24.0));
        gap(Dp(16.0));
        spec_table(
            vec!["ui-token".into(), "ui-value".into()],
            token_rows(&rows),
        );
        gap(Dp(32.0));
    }
    gap(Dp(16.0));
    subheading("ui-color".to_owned());
    gap(Dp(16.0));
    paragraph("ui-medium-top-app-bar-color-roles-used-for-light-and-dark-schemes".into());
    gap(Dp(8.0));
    numbered(vec![
        "Surface".into(),
        "On surface".into(),
        "On surface".into(),
        "On surface variant".into(),
    ]);
    gap(Dp(48.0));
    subheading("ui-measurements".to_owned());
    gap(Dp(16.0));
    paragraph("ui-medium-and-large-app-bars-share-the-same-layout-a-64dp-top-row-for-the-leading-button-and".into());
    for (title, large) in [("ui-medium-app-bar", false), ("ui-large-app-bar", true)] {
        gap(Dp(24.0));
        subheading(title.to_owned());
        gap(Dp(24.0));
        panel(RenderSlot::new(move || {
            boxed()
                .modifier(Modifier::new().width(Dp(360.0)))
                .children(move || {
                    baseline_bar(large).label(
                        I18N.format("ui-value-measurement", [("v0", I18N.text(&title).into())]),
                    );
                });
        }));
        gap(Dp(16.0));
        caption(I18N.format(
            "ui-value-padding-and-size-measurements",
            [("v0", I18N.text(&title).into())],
        ));
    }
    gap(Dp(32.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-medium".into(), "ui-large".into()],
        vec![
            vec!["ui-container-height".into(), "112dp".into(), "152dp".into()],
            vec!["ui-top-padding".into(), "20dp".into(), "20dp".into()],
            vec![
                "ui-leading-and-trailing-padding".into(),
                "16dp".into(),
                "16dp".into(),
            ],
            vec!["ui-trailing-icon-size".into(), "24dp".into(), "24dp".into()],
            vec![
                "ui-headline-bottom-padding".into(),
                "24dp".into(),
                "28dp".into(),
            ],
            vec![
                "ui-headline-type-style".into(),
                "Headline small".into(),
                "Headline medium".into(),
            ],
        ],
    );
}

#[tessera]
fn combined_app_anatomy() {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let width = geometry
        .with(|values| {
            values
                .get("figure")
                .map(|r| Dp::from_pixels_f32(r.width.to_f32()).0)
        })
        .unwrap_or(1024.0);
    let height = width * (1768.0 / 2048.0);
    boxed()
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(height))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            for (variant, fraction) in [
                (TopAppBarVariant::Small, 0.156),
                (TopAppBarVariant::MediumFlexible, 0.449),
                (TopAppBarVariant::LargeFlexible, 0.8),
            ] {
                anatomy_bar(variant, Dp(height * fraction));
            }
        });
    gap(Dp(16.0));
    paragraph("ui-1-container-2-leading-button-3-trailing-elements-4-headline-5-subtitle".into());
}

#[tessera]
fn anatomy_bar(variant: TopAppBarVariant, center_y: Dp) {
    let geometry = remember(Geometry::new);
    let actions = remember(|| 0_usize);
    let height = match variant {
        TopAppBarVariant::Small => 64.0,
        TopAppBarVariant::MediumFlexible => 136.0,
        TopAppBarVariant::LargeFlexible => 152.0,
    };
    boxed()
        .modifier(watch(geometry, "figure").fill_max_size())
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .align(Alignment::TopCenter)
                        .offset(Dp(0.0), Dp(center_y.0 - height * 0.5))
                        .width(Dp(412.0)),
                )
                .children(move || {
                    figure_bar(geometry, variant, false, actions);
                });
            app_annotations(geometry, false, variant);
        });
}

#[tessera]
fn search_figure(measurements: bool) {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let figure_width = geometry
        .with(|values| {
            values
                .get("figure")
                .map(|rect| Dp::from_pixels_f32(rect.width.to_f32()).0)
        })
        .unwrap_or(1024.0);
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(if !measurements {
                    260.0
                } else if figure_width < 800.0 {
                    520.0
                } else {
                    (figure_width / (2048.0 / 610.0)).max(260.0)
                }))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(
                        I18N.format(
                            "ui-search-app-bar-value-figure",
                            [(
                                "v0",
                                I18N.text(
                                    &(if measurements {
                                        "ui-measurement-7071"
                                    } else {
                                        "ui-anatomy-7072"
                                    }),
                                )
                                .into(),
                            )],
                        ),
                    ),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            if measurements {
                for variant in [0_usize, 1] {
                    search_pair_item(variant, measurements);
                }
            } else {
                search_pair_item(3, measurements);
            }
        });
    gap(Dp(16.0));
    paragraph(if measurements {
        "ui-search-app-bar-padding-and-size-measurements".into()
    } else {
        "ui-1-container-2-leading-icon-button-3-hinted-search-text-4-trailing-icon-or-avatar-5-search".into()
    });
}

#[tessera]
fn search_pair_item(variant: usize, measurements: bool) {
    let geometry = remember(Geometry::new);
    let width = geometry
        .with(|values| {
            values
                .get("figure")
                .map(|r| Dp::from_pixels_f32(r.width.to_f32()).0)
        })
        .unwrap_or(1024.0);
    let single = !measurements;
    let stacked = !single && width < 800.0;
    // Two specimens leave a fixed gap so the inner height callouts of both
    // bars fit between them without overlapping.
    let pair_gap = 120.0;
    let container = ((width - pair_gap) * 0.5 - 76.0).max(0.0);
    boxed()
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(if stacked || single {
                    260.0
                } else {
                    (width / (2048.0 / 610.0)).max(260.0)
                }))
                .offset(
                    Dp(0.0),
                    Dp(if stacked { variant as f64 * 260.0 } else { 0.0 }),
                ),
        )
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .align(if stacked || single {
                            Alignment::Center
                        } else if variant == 0 {
                            Alignment::CenterStart
                        } else {
                            Alignment::CenterEnd
                        })
                        .width(Dp(if single {
                            (width - 136.0).clamp(44.0, 456.0)
                        } else if stacked {
                            (width - 68.0).max(44.0)
                        } else {
                            container + 44.0
                        }))
                        .offset(
                            Dp(if stacked || single {
                                0.0
                            } else {
                                54.0 * if variant == 0 { 1.0 } else { -1.0 }
                            }),
                            Dp(0.0),
                        )
                        .padding_all(Dp(22.0)),
                )
                .children(move || {
                    if single {
                        search_sample()
                            .geometry(geometry)
                            .centered(true)
                            .specimen(variant)
                            .placeholder(I18N.text("ui-search-product"));
                    } else {
                        search_sample()
                            .geometry(geometry)
                            .centered(true)
                            .specimen(variant);
                    }
                });
            search_annotations(geometry, measurements);
        });
}

#[tessera]
fn search_annotations(geometry: State<Geometry>, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(container), Some(search), Some(leading), Some(trailing)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "search"),
        relative(&values, "leading"),
        relative(&values, "trailing"),
    ) else {
        return;
    };
    let mut svg = String::new();
    if measurements {
        dotted(
            &mut svg,
            container.x - 24.0,
            container.y,
            container.h,
            true,
            "#b14e70",
        );
        annotations::dimension(
            format!("{:.0}", container.h),
            Dp(container.x - 56.0),
            Dp(container.y + container.h * 0.5 - 10.0),
        );
        dotted(
            &mut svg,
            container.x + container.w + 12.0,
            search.y,
            search.h,
            true,
            "#b14e70",
        );
        annotations::dimension(
            format!("{:.0}", search.h),
            Dp(container.x + container.w + 16.0),
            Dp(search.y + 12.0),
        );
        for rect in [
            Rect {
                x: search.x,
                y: container.y,
                w: search.w,
                h: search.y - container.y,
            },
            Rect {
                x: search.x,
                y: search.y + search.h,
                w: search.w,
                h: container.y + container.h - search.y - search.h,
            },
        ] {
            svg.push_str(&format!(
                "<rect x='{}' y='{}' width='{}' height='{}' fill='#e883aa' fill-opacity='.32'/>",
                rect.x, rect.y, rect.w, rect.h
            ));
        }
        annotations::dimension(
            format!("{:.0}", search.y - container.y),
            Dp(search.x + 16.0),
            Dp(container.y - 28.0),
        );
        annotations::outline(&mut svg, search);
    } else {
        let hint = Rect {
            x: search.x + 56.0,
            y: search.y + 16.0,
            w: (search.w - 72.0).max(0.0),
            h: 24.0,
        };
        for (number, rect, x, y) in [
            (1, container, container.x - 44.0, container.y),
            (2, leading, leading.x + 12.0, leading.y - 56.0),
            (3, hint, hint.x + 12.0, hint.y + 72.0),
            (4, trailing, trailing.x + trailing.w + 24.0, trailing.y),
            (5, search, search.x + search.w * 0.5 - 12.0, search.y - 72.0),
        ] {
            if number == 1 || number == 4 {
                let end = if number == 1 { rect.x } else { x };
                let start = if number == 1 {
                    x + 24.0
                } else {
                    rect.x + rect.w
                };
                dotted(
                    &mut svg,
                    start,
                    rect.y + rect.h * 0.5,
                    end - start,
                    false,
                    "#49454f",
                );
            } else {
                let start = if number == 3 {
                    rect.y + rect.h
                } else {
                    y + 32.0
                };
                let end = if number == 3 { y } else { rect.y };
                dotted(&mut svg, x + 12.0, start, end - start, true, "#49454f");
            }
            annotations::marker(number, Dp(x), Dp(y));
        }
        annotations::outline(&mut svg, hint);
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn app_figure(
    variant: TopAppBarVariant,
    measurements: bool,
    filled_action: Option<bool>,
    #[prop(into)] name: Option<String>,
) {
    let geometry = remember(Geometry::new);
    let actions = remember(|| 0_usize);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let figure_width = geometry
        .with(|values| {
            values
                .get("figure")
                .map(|rect| Dp::from_pixels_f32(rect.width.to_f32()).0)
        })
        .unwrap_or(1024.0);
    let figure_height = if measurements {
        (figure_width / (2048.0 / 600.0)).max(240.0)
    } else {
        360.0
    };
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(figure_height))
                .background_with_shape(
                    theme.color_scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(
                        I18N.format(
                            "ui-app-bar-value-figure",
                            [(
                                "v0",
                                I18N.text(
                                    &(if measurements {
                                        "ui-measurement-7071"
                                    } else {
                                        "ui-anatomy-7072"
                                    }),
                                )
                                .into(),
                            )],
                        ),
                    ),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .align(Alignment::Center)
                        .width(Dp(456.0))
                        .padding_all(Dp(48.0)),
                )
                .children(move || {
                    figure_bar(geometry, variant, measurements, actions)
                        .filled_action_optional(filled_action);
                });
            app_annotations(geometry, measurements, variant);
        });
    gap(Dp(16.0));
    paragraph(if measurements {
        I18N.format(
            "ui-value-padding-and-size-measurements",
            [(
                "v0",
                I18N.text(&name.as_deref().unwrap_or("ui-app-bar")).into(),
            )],
        )
    } else {
        "ui-1-container-2-leading-button-3-trailing-elements-4-headline-5-subtitle".into()
    });
    if actions.get() > 0 {
        text(I18N.format(
            "ui-app-bar-figure-actions-value",
            [("v0", (actions.get()).into())],
        ))
        .style(type_style(14.0, 20.0));
    }
}

#[tessera]
fn figure_bar(
    geometry: State<Geometry>,
    variant: TopAppBarVariant,
    measurements: bool,
    filled_action: Option<bool>,
    actions: State<usize>,
) {
    top_app_bar()
        .modifier(watch(geometry, "container"))
        .variant(variant)
        .title(I18N.text(if measurements {
            match variant {
                TopAppBarVariant::Small => "Title Large",
                TopAppBarVariant::MediumFlexible => "Headline Medium",
                TopAppBarVariant::LargeFlexible => "Display Small",
            }
        } else {
            "ui-headline"
        }))
        .title_area_modifier(watch(geometry, "title"))
        .navigation_icon(move || {
            icon_button()
                .modifier(watch(geometry, "leading"))
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(filled::ARROW_BACK_SVG)
                .accessibility_label(I18N.text("ui-figure-back"))
                .on_click(move || actions.set(actions.get() + 1));
        })
        .action(move || {
            if filled_action.unwrap_or(false) {
                icon_button()
                    .modifier(watch(geometry, "trailing"))
                    .filled()
                    .icon(filled::SEARCH_SVG)
                    .accessibility_label(I18N.text("ui-figure-search"))
                    .on_click(move || actions.set(actions.get() + 1));
            } else {
                icon_button()
                    .modifier(watch(geometry, "trailing"))
                    .standard()
                    .color_style(IconButtonColorStyle::Inherited)
                    .icon(filled::SEARCH_SVG)
                    .accessibility_label(I18N.text("ui-figure-search"))
                    .on_click(move || actions.set(actions.get() + 1));
            }
            icon_button()
                .modifier(watch(geometry, "trailing-last"))
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(filled::CALENDAR_TODAY_SVG)
                .accessibility_label(I18N.text("ui-figure-calendar"))
                .on_click(move || actions.set(actions.get() + 1));
        })
        .subtitle_optional((!measurements).then(|| "ui-subtitle".to_owned()));
}

#[tessera]
fn app_annotations(geometry: State<Geometry>, measurements: bool, variant: TopAppBarVariant) {
    let values = geometry.get();
    let (
        Some(root),
        Some(container),
        Some(leading),
        Some(trailing),
        Some(trailing_last),
        Some(title),
    ) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "leading"),
        relative(&values, "trailing"),
        relative(&values, "trailing-last"),
        relative(&values, "title"),
    )
    else {
        return;
    };
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let subtitle_height = if measurements {
        0.0
    } else {
        match variant {
            TopAppBarVariant::Small => theme.typography.label_medium,
            TopAppBarVariant::MediumFlexible => theme.typography.label_large,
            TopAppBarVariant::LargeFlexible => theme.typography.title_medium,
        }
        .line_height
        .unwrap_or(Dp(20.0))
        .0
    };
    let subtitle = Rect {
        x: title.x,
        y: title.y + title.h - subtitle_height,
        w: title.w,
        h: subtitle_height,
    };
    let headline = Rect {
        h: (title.h - subtitle_height).max(0.0),
        ..title
    };
    let mut svg = String::new();
    if measurements {
        dotted(
            &mut svg,
            container.x - 24.0,
            container.y,
            container.h,
            true,
            "#b14e70",
        );
        annotations::dimension(
            format!("{:.0}", container.h),
            Dp(container.x - 60.0),
            Dp(container.y + container.h * 0.5 - 10.0),
        );
        for rect in [
            Rect {
                x: container.x,
                y: container.y,
                w: leading.x - container.x,
                h: container.h,
            },
            Rect {
                x: trailing_last.x + trailing_last.w,
                y: container.y,
                w: (container.x + container.w - trailing_last.x - trailing_last.w).max(0.0),
                h: container.h,
            },
            Rect {
                x: title.x,
                y: title.y + title.h,
                w: title.w,
                h: container.y + container.h - title.y - title.h,
            },
        ] {
            if rect.w > 0.0 && rect.h > 0.0 {
                svg.push_str(&format!("<rect x='{}' y='{}' width='{}' height='{}' fill='#e883aa' fill-opacity='.32'/>", rect.x, rect.y, rect.w, rect.h));
            }
        }
        for (label, x, y) in [
            (
                format!("{:.0}", leading.x - container.x),
                container.x - 2.0,
                container.y - 32.0,
            ),
            (
                format!(
                    "{:.0}",
                    (container.x + container.w - trailing_last.x - trailing_last.w).max(0.0)
                ),
                container.x + container.w - 8.0,
                container.y - 32.0,
            ),
        ] {
            annotations::dimension(label, Dp(x), Dp(y));
        }
        if variant != TopAppBarVariant::Small {
            annotations::dimension(
                format!(
                    "{:.0}",
                    (container.y + container.h - title.y - title.h).max(0.0)
                ),
                Dp(title.x + title.w + 8.0),
                Dp(title.y + title.h),
            );
        }
        annotations::outline(&mut svg, trailing_last);
        annotations::outline(&mut svg, title);
        annotations::outline(&mut svg, leading);
        annotations::outline(&mut svg, trailing);
    } else {
        for (number, rect, x, y) in [
            (1, container, container.x - 44.0, container.y),
            (
                2,
                leading,
                leading.x + leading.w * 0.5 - 12.0,
                leading.y - 56.0,
            ),
            (
                3,
                Rect {
                    w: trailing_last.x + trailing_last.w - trailing.x,
                    ..trailing
                },
                trailing.x + 24.0,
                leading.y - 56.0,
            ),
            (
                4,
                headline,
                headline.x + 56.0,
                container.y + container.h + 24.0,
            ),
            (
                5,
                subtitle,
                subtitle.x + 8.0,
                container.y + container.h + 24.0,
            ),
        ] {
            if number == 1 {
                dotted(
                    &mut svg,
                    x + 24.0,
                    rect.y + rect.h * 0.5,
                    rect.x - x - 24.0,
                    false,
                    "#49454f",
                );
            } else if number == 2 || number == 3 {
                dotted(
                    &mut svg,
                    x + 12.0,
                    y + 32.0,
                    rect.y - y - 32.0,
                    true,
                    "#49454f",
                );
            } else {
                dotted(
                    &mut svg,
                    x + 12.0,
                    rect.y + rect.h,
                    y - rect.y - rect.h,
                    true,
                    "#49454f",
                );
            }
            annotations::marker(number, Dp(x), Dp(y));
        }
        annotations::outline(&mut svg, leading);
        annotations::outline(&mut svg, trailing);
        annotations::outline(&mut svg, headline);
        annotations::outline(&mut svg, subtitle);
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {

    use tessera_ui::{
        MOUSE_POINTER_ID, PressKeyEventType, PxPosition, PxRect,
        headless::{HeadlessConfig, HeadlessRenderer},
        scaled_box,
    };

    use super::*;

    #[derive(Clone, Copy, PartialEq)]
    enum Case {
        Bar(TopAppBarVariant),
        Search(f64, f32),
        SearchMeasurements(f64),
        Variants,
        Labeled,
        SearchColor,
        Hiding,
    }

    #[tessera]
    fn fixture(
        case: Case,
        observed_geometry: tessera_ui::testing::TestProbe<Option<State<Geometry>>>,
    ) {
        let geometry = remember(Geometry::new);
        observed_geometry.set(Some(geometry));
        material_theme().child(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(move || match case {
                    Case::Bar(variant) => {
                        app_figure(variant, true);
                    }
                    Case::SearchMeasurements(width) => {
                        boxed()
                            .modifier(Modifier::new().width(Dp(width)))
                            .children(|| {
                                search_figure(true);
                            });
                    }
                    Case::Variants => {
                        variants_figure("Probe");
                    }
                    Case::Labeled => {
                        boxed()
                            .modifier(Modifier::new().width(Dp(412.0)))
                            .children(|| {
                                specimen(LABELED);
                            });
                    }
                    Case::SearchColor => {
                        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                        column()
                            .modifier(Modifier::new().width(Dp(600.0)))
                            .children(move || {
                                for color in
                                    [scheme.surface_container, scheme.surface_container_high]
                                {
                                    boxed().modifier(
                                        Modifier::new()
                                            .size(Dp(40.0), Dp(40.0))
                                            .background_with_shape(
                                                color,
                                                Shape::rounded_rectangle(Dp(0.0)),
                                            ),
                                    );
                                }
                                search_sample().geometry(geometry);
                            });
                    }
                    Case::Hiding => {
                        sample("Hiding app bar", TopAppBarVariant::Small)
                            .behavior(AppBarScrollBehavior::EnterAlways)
                            .controls(true);
                    }
                    Case::Search(width, scale) => {
                        scaled_box(scale).child(move || {
                            boxed()
                                .modifier(Modifier::new().required_size(Dp(width), Dp(320.0)))
                                .children(move || {
                                    search_sample().geometry(geometry);
                                });
                        });
                    }
                });
        });
    }

    fn renderer(case: Case) -> (HeadlessRenderer, State<Geometry>) {
        let geometry = tessera_ui::testing::TestProbe::default();
        let observed = geometry.clone();
        let mut renderer = HeadlessRenderer::new(
            Box::new(move || {
                fixture(case, observed.clone());
            }),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 1200,
                height: 800,
                sample_count: 1,
                ..Default::default()
            },
        )
        .expect("native app bar figures");
        renderer.render_frames(6);
        (renderer, geometry.get().expect("mounted app bar geometry"))
    }

    fn click(renderer: &mut HeadlessRenderer, bounds: PxRect) {
        let position = PxPosition::new(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(30);
    }

    #[test]
    #[ignore = "Requires a GPU adapter for the real component pipelines"]
    fn native_app_bar_figures_measure_real_variants_and_search_opens_a_view() {
        for (variant, height) in [
            (TopAppBarVariant::Small, 64.0),
            (TopAppBarVariant::MediumFlexible, 112.0),
            (TopAppBarVariant::LargeFlexible, 120.0),
        ] {
            let (renderer, _geometry_state) = renderer(Case::Bar(variant));
            let snapshot = renderer
                .accessibility_snapshot()
                .expect("app bar semantics");
            let bar = snapshot
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::Toolbar)
                .expect("native app bar")
                .1
                .bounds()
                .expect("native bar bounds");
            assert_eq!(bar.width(), 360.0);
            assert_eq!(bar.height(), height);
            let calendar = snapshot
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Button && crate::has_label(&node, "Figure calendar")
                })
                .expect("last native action")
                .1
                .bounds()
                .unwrap();
            assert_eq!(bar.x1 - calendar.x1, 4.0);
            assert!(
                snapshot
                    .nodes
                    .iter()
                    .any(|(_, node)| crate::has_label(&node, "Dimension 4"))
            );
            assert!(
                !snapshot
                    .nodes
                    .iter()
                    .any(|(_, node)| matches!(node.label(), Some("Dimension 52" | "Dimension -0")))
            );
            if variant == TopAppBarVariant::Small {
                assert!(
                    !snapshot
                        .nodes
                        .iter()
                        .any(|(_, node)| crate::has_label(&node, "Dimension 0"))
                );
            }
            let back = snapshot
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Button && crate::has_label(&node, "Figure back")
                })
                .expect("actual navigation button")
                .1
                .bounds()
                .unwrap();
            assert!(back.width() >= 48.0 && back.height() >= 48.0);
        }
        for (width, field_width) in [(400.0, 280.0), (600.0, 396.0), (1000.0, 596.0)] {
            let (mut renderer, geometry_state) = renderer(Case::Search(width, 1.0));
            let geometry = geometry_state.get();
            let bar = geometry.get("container").expect("actual app bar container");
            let field = *geometry.get("search").expect("actual native search field");
            assert_eq!(bar.width.0 as f64, width);
            assert_eq!(bar.height.0, 64);
            assert_eq!(field.width.0 as f64, field_width);
            assert_eq!(field.height.0, 56);
            assert_eq!(field.y.0 - bar.y.0, 4);
            click(&mut renderer, field);
            let snapshot = renderer
                .accessibility_snapshot()
                .expect("search view semantics");
            assert!(
                snapshot
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Dialog
                        && crate::has_label(&node, "Search library view")),
                "search app bar opens its real search view"
            );
            let close = snapshot
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Button && crate::has_label(&node, "Close app bar search")
                })
                .expect("search view back button")
                .1
                .bounds()
                .unwrap();
            click(
                &mut renderer,
                PxRect::new(
                    tessera_ui::Px(close.x0 as i32),
                    tessera_ui::Px(close.y0 as i32),
                    tessera_ui::Px(close.width() as i32),
                    tessera_ui::Px(close.height() as i32),
                ),
            );
            assert!(
                !renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Dialog
                        && crate::has_label(&node, "Search library view"))
            );
        }
        for scale in [0.5, 1.5] {
            let (_renderer, geometry_state) = renderer(Case::Search(600.0, scale));
            let geometry = geometry_state.get();
            let bar = geometry.get("container").expect("scaled app bar container");
            let field = geometry.get("search").expect("scaled native search field");
            assert_eq!(bar.width.to_f32(), 600.0 * scale);
            assert_eq!(bar.height.to_f32(), 64.0 * scale);
            assert_eq!(field.width.to_f32(), 396.0 * scale);
            assert_eq!(field.height.to_f32(), 56.0 * scale);
            assert_eq!((field.y - bar.y).to_f32(), 4.0 * scale);
        }
    }
    #[test]
    #[ignore = "Requires a GPU adapter for the real component pipelines"]
    fn search_measurement_callouts_stay_outside_native_controls_at_both_layouts() {
        for width in [1200.0, 600.0, 320.0] {
            let (renderer, _geometry_state) = renderer(Case::SearchMeasurements(width));
            let snapshot = renderer
                .accessibility_snapshot()
                .expect("measurement semantics");
            let labels: Vec<_> = snapshot
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == Role::Note && crate::has_label(&node, "Dimension 56")
                })
                .map(|(_, node)| node.bounds().unwrap())
                .collect();
            assert_eq!(labels.len(), 2, "both real search measurements at {width}");
            let controls: Vec<_> = snapshot
                .nodes
                .iter()
                .filter(|(_, node)| {
                    matches!(
                        node.role(),
                        Role::Button | Role::TextInput | Role::SearchInput
                    )
                })
                .filter_map(|(_, node)| node.bounds())
                .collect();
            assert!(!controls.is_empty());
            for label in &labels {
                assert!(
                    label.x0 >= 0.0 && label.x1 <= width,
                    "callout within figure at {width}: {label:?}"
                );
                for control in &controls {
                    assert!(
                        label.x1 <= control.x0
                            || label.x0 >= control.x1
                            || label.y1 <= control.y0
                            || label.y0 >= control.y1,
                        "56dp callout overlaps a native control at {width}: {label:?} / {control:?}"
                    );
                }
            }
            if width < 800.0 {
                assert_eq!((labels[0].y0 - labels[1].y0).abs(), 260.0);
            } else {
                assert_eq!(labels[0].y0, labels[1].y0);
            }
        }
    }

    fn find_bounds(
        renderer: &HeadlessRenderer,
        role: Option<Role>,
        label: &str,
    ) -> Option<[f64; 4]> {
        renderer
            .accessibility_snapshot()
            .expect("app bar semantics")
            .nodes
            .into_iter()
            .find(|(_, node)| {
                role.is_none_or(|role| node.role() == role) && crate::has_label(&node, label)
            })
            .and_then(|(_, node)| node.bounds())
            .map(|rect| [rect.x0, rect.y0, rect.x1, rect.y1])
    }

    fn click_at(renderer: &mut HeadlessRenderer, bounds: [f64; 4]) {
        let position = PxPosition::from_f32_xy(
            ((bounds[0] + bounds[2]) * 0.5) as f32,
            ((bounds[1] + bounds[3]) * 0.5) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(30);
    }

    #[test]
    #[ignore = "Requires a GPU adapter for the real component pipelines"]
    fn variant_figure_search_bar_keeps_its_natural_height_and_spacing() {
        let (renderer, _geometry_state) = renderer(Case::Variants);
        let search = find_bounds(&renderer, Some(Role::Toolbar), "Search app bar")
            .expect("native search app bar");
        assert_eq!(search[3] - search[1], 64.0);
        let small = find_bounds(&renderer, None, "Probe Small bar").expect("native small app bar");
        assert_eq!(
            small[1] - search[3],
            32.0,
            "the search app bar must not reserve extra height above the next variant"
        );
    }

    #[test]
    #[ignore = "Requires a GPU adapter for the real component pipelines"]
    fn search_app_bar_field_paints_the_surface_container_role() {
        let (renderer, geometry_state) = renderer(Case::SearchColor);
        let geometry = geometry_state.get();
        let field = *geometry.get("search").expect("native search field");
        let pixels = renderer.read_pixels().unwrap();
        let at = |x: i32, y: i32| {
            let index = (y as usize * 1200 + x as usize) * 4;
            [pixels[index], pixels[index + 1], pixels[index + 2]]
        };
        let container = at(20, 20);
        let standalone = at(20, 60);
        assert_ne!(
            container, standalone,
            "reference roles must be distinguishable"
        );
        assert_eq!(
            at(
                field.x.0 + field.width.0 - 24,
                field.y.0 + field.height.0 / 2
            ),
            container,
            "search fields inside app bars use surface container"
        );
    }

    #[test]
    #[ignore = "Requires a GPU adapter for the real component pipelines"]
    fn labeled_app_bar_exposes_a_heading_and_a_labeled_action() {
        let (mut renderer, _geometry_state) = renderer(Case::Labeled);
        find_bounds(
            &renderer,
            Some(Role::Heading),
            "Saved podcasts, 80 episodes",
        )
        .expect("headline exposed as a heading with its subtitle context");
        let map = find_bounds(&renderer, Some(Role::Button), "View on map")
            .expect("icon button labeled with its action");
        click_at(&mut renderer, map);
        find_bounds(&renderer, None, "Labeled app bar: View on map selected")
            .expect("the labeled action runs its callback");
    }

    #[test]
    #[ignore = "Requires a GPU adapter for the real component pipelines"]
    fn hiding_app_bar_hides_on_scroll_up_and_returns_on_scroll_down() {
        let (mut renderer, _geometry_state) = renderer(Case::Hiding);
        let height = |renderer: &HeadlessRenderer| {
            find_bounds(renderer, None, "Hiding app bar bar").map_or(0.0, |rect| rect[3] - rect[1])
        };
        assert_eq!(height(&renderer), 64.0);
        let content =
            find_bounds(&renderer, None, "Hiding app bar scroll content").expect("scroll content");
        let point = PxPosition::from_f32_xy(
            ((content[0] + content[2]) * 0.5) as f32,
            (content[1] + 120.0) as f32,
        );
        renderer.pointer_scroll(MOUSE_POINTER_ID, point, 0.0, -200.0);
        renderer.render_frames(60);
        assert_eq!(height(&renderer), 0.0, "scrolling content up hides the bar");
        renderer.pointer_scroll(MOUSE_POINTER_ID, point, 0.0, 120.0);
        renderer.render_frames(60);
        assert_eq!(
            height(&renderer),
            64.0,
            "scrolling down reveals the bar again"
        );
    }
}
