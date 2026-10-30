//! Expressive app bar specification examples.
//!
//! ## Usage
//!
//! Compare screen titles, navigation actions and scrolling app bars.

use tessera_components::{
    alignment::Alignment,
    app_bar::{
        AppBarScrollBehavior, AppBarScrollState, AppBarTitleAlignment, TopAppBarVariant, app_bar,
        app_bar_scroll_provider, top_app_bar,
    },
    boxed::boxed,
    button::button,
    column::column,
    dialog::dialog_provider,
    flow_row::flow_row,
    icon::icon,
    icon_button::{IconButtonColorStyle, icon_button},
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    painter::try_painter_asset,
    scaffold::scaffold,
    scrollable::ScrollableController,
    search::{SearchBarController, search_bar},
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, PxRect, RenderSlot, State, accesskit::Role, modifier::FocusModifierExt as _,
    remember, remember_with_key, tessera, use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn app_bars_spec() {
    article(
        "App bars",
        "Navigate between screens and expose relevant actions.",
        vec![
            SpecSection {
                title: "Variants",
                content: RenderSlot::new(variants),
            },
            SpecSection {
                title: "Configurations",
                content: RenderSlot::new(configurations),
            },
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Anatomy",
                content: RenderSlot::new(anatomy),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    ).pages(vec![
        ArticlePage {
            title: "Overview", intro: Some(RenderSlot::new(|| {
                paragraph("App bars identify the current page and keep navigation and one or two essential actions within reach. Choose search, small, medium flexible or large flexible layouts.".into());
                gap(Dp(32.0)); variants();
            })), sections: vec![
                section("M3 Expressive update", RenderSlot::new(|| {
                    paragraph("Search app bar adds an emphasized search entry point. Flexible app bars replace baseline medium and large variants, with larger titles, optional subtitles, text alignment and richer action treatments.".into());
                    gap(Dp(24.0)); sample("Expressive overview", TopAppBarVariant::MediumFlexible).subtitle(true).controls(true);
                })),
                section("Differences from M2", RenderSlot::new(|| {
                    paragraph("App bars use theme color roles and a contrasting container on scroll. Flexible variants make page titles more prominent while retaining compact navigation and action targets.".into());
                    gap(Dp(24.0)); app_figure(TopAppBarVariant::Small, false);
                })),
            ],
        },
        ArticlePage {
            title: "Guidelines", intro: None, sections: vec![
                section("Usage", RenderSlot::new(|| {
                    paragraph("Use page-specific navigation, a brief headline and one primary action. If a second action is essential, keep its emphasis lower. Put larger action collections in a toolbar.".into());
                    gap(Dp(24.0)); app_figure(TopAppBarVariant::Small, false);
                    gap(Dp(32.0)); subheading("Baseline app bars".into());
                    paragraph("Use medium flexible and large flexible for new layouts instead of the baseline medium and large app bars.".into());
                })),
                section("Search app bar", RenderSlot::new(|| {
                    paragraph("Search should open a search view. The hint includes Search and identifies the scope. Keep common actions closest to the leading edge.".into());
                    gap(Dp(24.0)); search_sample();
                    gap(Dp(32.0)); subheading("Buttons in search app bar".into());
                    paragraph("Use a leading navigation or product element and at most two trailing icons with an avatar on compact windows.".into());
                    gap(Dp(24.0)); subheading("Large screens".into());
                    paragraph("Larger windows can reveal up to four trailing actions. The search field fills available space until 312dp, then grows at half the rate.".into());
                    gap(Dp(24.0)); subheading("Alternate color options".into());
                    paragraph("Prefer surface container for search; surface bright can distinguish the field on darker backgrounds.".into());
                })),
                section("Anatomy", RenderSlot::new(|| {
                    app_figure(TopAppBarVariant::MediumFlexible, false);
                    for (title, body) in [
                        ("Container", "Keep the container rectangular, at its default height and across the full window width."),
                        ("Adding logos", "Use a meaningful image without disrupting navigation or actions."),
                        ("Leading button", "Use Back to return or Menu to open product navigation."),
                        ("Headline", "Keep small headlines on one line; flexible headlines can wrap to two lines without truncating."),
                        ("Subtitle", "Align supporting context with its headline."),
                        ("Trailing icon buttons", "Prefer filled symbols and emphasize only one primary action."),
                    ] { gap(Dp(24.0)); subheading(title.into()); paragraph(body.into()); }
                })),
                section("Adaptive design", RenderSlot::new(|| {
                    subheading("Resizing".into());
                    paragraph("The app bar fills its window. Reduce trailing actions into overflow when needed, and keep search between its leading and trailing controls.".into());
                    gap(Dp(24.0)); search_sample();
                    gap(Dp(32.0)); subheading("Presentation".into());
                    paragraph("Align navigation and actions to the leading and trailing edges so right-to-left layouts can mirror their placement.".into());
                })),
                section("Behavior", RenderSlot::new(|| {
                    subheading("Scrolling".into());
                    paragraph("A contrasting container separates scrolled content. Flexible bars can compress into a small bar and expand again at the page top.".into());
                    gap(Dp(24.0)); sample("Scrolling app bar", TopAppBarVariant::MediumFlexible).subtitle(true).controls(true);
                })),
            ],
        },
        ArticlePage {
            title: "Accessibility", intro: None, sections: vec![
                section("Use cases", RenderSlot::new(|| { paragraph("Communicate the current page, expose navigation and page actions, and keep those actions accessible as content scrolls.".into()); })),
                section("Interaction & style", RenderSlot::new(|| {
                    for (title, body) in [("Touch", "Each action retains a 48×48dp target and ripple feedback."), ("Cursor", "Hover and pressed states identify the interactive icon button."), ("Keyboard or switch", "A focus indicator identifies the action that will activate."), ("Initial focus", "The leading navigation button is the first focus target, followed by trailing actions.")] {
                        subheading(title.into()); paragraph(body.into()); gap(Dp(24.0));
                    }
                    sample("Accessible app bar", TopAppBarVariant::Small).controls(true);
                })),
                section("Color", RenderSlot::new(|| {
                    paragraph("Prefer the default search container and label colors. Custom colors need at least 3:1 contrast between label and container.".into());
                    gap(Dp(24.0)); search_sample();
                })),
                section("Keyboard navigation", RenderSlot::new(|| {
                    spec_table(vec!["Keys".into(), "Actions".into()], vec![vec!["Tab".into(), "Focus the next action".into()], vec!["Space or Enter".into(), "Activate the focused action".into()]]);
                    gap(Dp(24.0)); sample("Keyboard app bar", TopAppBarVariant::Small).controls(true);
                })),
                section("Labeling elements", RenderSlot::new(|| {
                    paragraph("The headline names the page. Label icon buttons with their action, such as Back or Search, and add context only when it clarifies the destination.".into());
                    gap(Dp(24.0)); app_figure(TopAppBarVariant::Small, false);
                })),
            ],
        },
    ]);
}

fn section(title: &'static str, content: RenderSlot) -> SpecSection {
    SpecSection { title, content }
}

#[tessera]
fn paragraph(content: String) {
    text().content(content).style(type_style(16.0, 24.0));
}

#[tessera]
fn sample(
    #[prop(into)] name: String,
    variant: TopAppBarVariant,
    subtitle: Option<bool>,
    centered: Option<bool>,
    controls: Option<bool>,
    logo: Option<bool>,
) {
    let label = remember(|| name.clone());
    let variant = remember(|| variant);
    let subtitle = remember(|| subtitle.unwrap_or(false));
    let centered = remember(|| centered.unwrap_or(false));
    let filled_action = remember(|| false);
    let logo = remember(|| logo.unwrap_or(false));
    let status = remember(|| "No action".to_owned());
    let scroll = remember(AppBarScrollState::default);
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
                            (TopAppBarVariant::Small, "Small"),
                            (TopAppBarVariant::MediumFlexible, "Medium flexible"),
                            (TopAppBarVariant::LargeFlexible, "Large flexible"),
                        ] {
                            button()
                                .tonal()
                                .selected(variant.get() == value)
                                .accessibility_label(format!("{} {title}", label.get()))
                                .on_click(move || {
                                    variant.set(value);
                                    scroll.with_mut(AppBarScrollState::reset);
                                })
                                .child(move || {
                                    text().content(title);
                                });
                        }
                        button()
                            .tonal()
                            .selected(subtitle.get())
                            .accessibility_label(format!("{} subtitle", label.get()))
                            .on_click(move || subtitle.set(!subtitle.get()))
                            .child(|| {
                                text().content("Subtitle");
                            });
                        button()
                            .tonal()
                            .selected(centered.get())
                            .accessibility_label(format!("{} centered", label.get()))
                            .on_click(move || centered.set(!centered.get()))
                            .child(|| {
                                text().content("Centered");
                            });
                        button()
                            .tonal()
                            .selected(filled_action.get())
                            .accessibility_label(format!("{} filled action", label.get()))
                            .on_click(move || filled_action.set(!filled_action.get()))
                            .child(|| {
                                text().content("Filled action");
                            });
                        button()
                            .tonal()
                            .selected(logo.get())
                            .accessibility_label(format!("{} logo", label.get()))
                            .on_click(move || logo.set(!logo.get()))
                            .child(|| {
                                text().content("Logo");
                            });
                        button()
                            .text()
                            .accessibility_label(format!("Reset {} scroll", label.get()))
                            .on_click(move || {
                                scroll.with_mut(AppBarScrollState::reset);
                                content.with_mut(|content| {
                                    content.set_scroll_position(tessera_ui::PxPosition::ZERO)
                                });
                            })
                            .child(|| {
                                text().content("Reset scroll");
                            });
                    });
                gap(Dp(24.0));
            }
            app_bar_scroll_provider()
                .state(scroll)
                .behavior(AppBarScrollBehavior::ExitUntilCollapsed)
                .scroll_controller(content)
                .child(move || {
                    let bar = RenderSlot::new(move || {
                        boxed()
                            .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some(format!("{} bar", label.get())),
                                merge_descendants: true,
                                ..Default::default()
                            }))
                            .children(move || {
                                let mut bar = top_app_bar()
                                    .variant(variant.get())
                                    .title("Library")
                                    .subtitle_optional(
                                        subtitle.get().then(|| "Recently saved".to_owned()),
                                    )
                                    .title_alignment(if centered.get() {
                                        AppBarTitleAlignment::Center
                                    } else {
                                        AppBarTitleAlignment::Start
                                    })
                                    .title_area_modifier(Modifier::new().semantics(SemanticsArgs {
                                        role: Some(Role::Group),
                                        label: Some(format!("{} title", label.get())),
                                        merge_descendants: true,
                                        ..Default::default()
                                    }))
                                    .navigation_icon(move || {
                                        icon_button()
                                            .standard()
                                            .color_style(IconButtonColorStyle::Inherited)
                                            .icon(filled::ARROW_BACK_SVG)
                                            .accessibility_label(format!("{} back", label.get()))
                                            .on_click(move || {
                                                status.set("Back selected".to_owned())
                                            });
                                    })
                                    .action_shared(RenderSlot::new(move || {
                                        let mut action = icon_button()
                                            .icon(filled::SEARCH_SVG)
                                            .accessibility_label(format!("{} search", label.get()))
                                            .on_click(move || {
                                                status.set("Search selected".to_owned())
                                            });
                                        action = if filled_action.get() {
                                            action.filled()
                                        } else {
                                            action
                                                .standard()
                                                .color_style(IconButtonColorStyle::Inherited)
                                        };
                                        drop(action);
                                    }));
                                if logo.get() {
                                    bar = bar.title_content(brand);
                                }
                                drop(bar);
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
                                                label: Some(format!(
                                                    "{} scroll content",
                                                    label.get()
                                                )),
                                                merge_descendants: true,
                                                ..Default::default()
                                            }),
                                    )
                                    .children(|| {
                                        for index in 0..20 {
                                            text()
                                                .content(format!("Saved item {}", index + 1))
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
                text()
                    .content(format!("{} action: {}", label.get(), status.get()))
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
    let painter = remember_with_key(asset, move || {
        try_painter_asset(asset).expect("bundled brand image")
    });
    boxed()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Image),
            label: Some("Tessera brand".to_owned()),
            ..Default::default()
        }))
        .children(move || {
            icon().painter(painter.get()).size(Dp(32.0));
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
        .accessibility_label("Search app bar account")
        .on_click(move || status.set("Account selected".into()))
        .child(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(32.0), Dp(32.0))
                        .background_with_shape(scheme.primary, Shape::Ellipse),
                )
                .children(move || {
                    text()
                        .content("A")
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
) {
    let controller = remember(|| SearchBarController::new(false));
    let query = remember(String::new);
    let opened = remember(|| false);
    let width = remember(|| Dp(600.0));
    let status = remember(String::new);
    dialog_provider()
        .accessibility_label("Search library view")
        .full_screen(true)
        .is_open(opened.get())
        .on_close_request(move || opened.set(false))
        .main_content(move || {
            boxed()
                .alignment(if centered.unwrap_or(false) {
                    Alignment::Center
                } else {
                    Alignment::TopStart
                })
                .modifier(Modifier::new().fill_max_width().height(Dp(320.0)))
                .children(move || {
                    let modifier = geometry
                        .map(|geometry| watch(geometry, "container"))
                        .unwrap_or_default()
                        .semantics(SemanticsArgs {
                            role: Some(Role::Toolbar),
                            label: Some("Search app bar".into()),
                            merge_descendants: true,
                            ..Default::default()
                        })
                        .on_globally_positioned(move |bounds: PxRect| {
                            let next = Dp::from_pixels_f32(bounds.width.to_f32());
                            if width.get() != next {
                                width.set(next);
                            }
                        });
                    app_bar()
                        .modifier(modifier)
                        .content_padding(Padding::symmetric(Dp(4.0), Dp(0.0)))
                        .content(move || {
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
                                .accessibility_label("Search app bar menu")
                                .on_click(move || status.set("Menu selected".into()));
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(Modifier::new().weight(1.0))
                                .children(move || {
                                    let available = (width.get().0
                                        - if specimen == Some(1) { 172.0 } else { 120.0 })
                                    .max(0.0);
                                    let field_width = if specimen.is_some() || available <= 312.0 {
                                        available
                                    } else {
                                        312.0 + (available - 312.0) * 0.5
                                    };
                                    let mut field = search_bar()
                                        .modifier(
                                            geometry
                                                .map(|geometry| watch(geometry, "search"))
                                                .unwrap_or_default()
                                                .width(Dp(field_width)),
                                        )
                                        .controller(controller)
                                        .read_only(true)
                                        .is_active(false)
                                        .placeholder(if specimen == Some(0) {
                                            ""
                                        } else if specimen.is_some() {
                                            "Body Large"
                                        } else {
                                            "Search library"
                                        })
                                        .on_active_change(move |active| {
                                            if active {
                                                opened.set(true);
                                            }
                                        });
                                    if specimen == Some(1) {
                                        field = field.trailing_icon_shared(RenderSlot::new(|| {
                                            icon().vector(filled::MIC_SVG).size(Dp(24.0));
                                        }));
                                    } else if specimen.is_none() {
                                        field = field.leading_icon_shared(RenderSlot::new(|| {
                                            icon().vector(filled::SEARCH_SVG).size(Dp(24.0));
                                        }));
                                    }
                                    drop(field);
                                    if specimen == Some(0) {
                                        boxed()
                                            .alignment(Alignment::Center)
                                            .modifier(
                                                Modifier::new().size(Dp(field_width), Dp(56.0)),
                                            )
                                            .children(|| {
                                                text()
                                                    .content("Body Large")
                                                    .style(type_style(16.0, 24.0));
                                            });
                                    }
                                });
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            if specimen == Some(1) {
                                icon_button()
                                    .standard()
                                    .color_style(IconButtonColorStyle::Inherited)
                                    .icon(filled::AUTO_AWESOME_SVG)
                                    .accessibility_label("Search app bar suggestions")
                                    .on_click(move || status.set("Suggestions selected".into()));
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
                                    .accessibility_label("Search app bar account")
                                    .on_click(move || status.set("Account selected".into()));
                            }
                        });
                });
        })
        .dialog_content(move || {
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
                        .is_active(true)
                        .on_active_change(move |active| opened.set(active))
                        .placeholder("Search library")
                        .leading_icon(move || {
                            icon_button()
                                .standard()
                                .icon(filled::ARROW_BACK_SVG)
                                .accessibility_label("Close app bar search")
                                .on_click(move || opened.set(false));
                        })
                        .on_query_change(move |value: String| {
                            query.set(value.clone());
                            value
                        })
                        .content(move || {
                            column()
                                .modifier(Modifier::new().padding_all(Dp(16.0)))
                                .children(move || {
                                    text().content(if query.get().is_empty() {
                                        "Search saved items".to_owned()
                                    } else {
                                        format!("Results for {}", query.get())
                                    });
                                });
                        });
                });
        });
    if !status.get().is_empty() {
        text().content(status.get()).style(type_style(14.0, 20.0));
    }
}

#[tessera]
fn variants() {
    column().modifier(Modifier::new().fill_max_width()).children(|| {
        panel(RenderSlot::new(|| {
            search_sample();
            gap(Dp(32.0));
            for (variant, label) in [(TopAppBarVariant::Small, "Small"), (TopAppBarVariant::MediumFlexible, "Medium flexible"), (TopAppBarVariant::LargeFlexible, "Large flexible")] {
                sample(format!("Variant {label}"), variant);
                gap(Dp(32.0));
            }
        }));
        gap(Dp(24.0));
        text().content("1  Search app bar    2  Small    3  Medium flexible    4  Large flexible").style(type_style(16.0,24.0));
        gap(Dp(32.0));
        subheading("Baseline variants".to_owned());
        gap(Dp(16.0));
        text().content("M3 Expressive replaces baseline medium and large app bars with medium flexible and large flexible app bars. Flexible titles support multiple lines and optional subtitles.").style(type_style(16.0,24.0));
    });
}

#[tessera]
fn configurations() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            subheading("Text alignment".to_owned());
            gap(Dp(24.0));
            panel(RenderSlot::new(|| {
                for centered in [false, true] {
                    sample(
                        format!(
                            "{} alignment",
                            if centered { "Centered" } else { "Leading" }
                        ),
                        TopAppBarVariant::MediumFlexible,
                    )
                    .centered(centered)
                    .subtitle(true);
                    gap(Dp(32.0));
                }
            }));
            gap(Dp(112.0));
            subheading("Interactive configuration".to_owned());
            gap(Dp(24.0));
            panel(RenderSlot::new(|| {
                sample("Configured app bar", TopAppBarVariant::MediumFlexible)
                    .subtitle(true)
                    .controls(true);
            }));
        });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Container".into(), "Surface".into()],
            vec!["Scrolled container".into(), "Surface container".into()],
            vec!["Headline and navigation icon".into(), "On surface".into()],
            vec![
                "Subtitle and trailing icons".into(),
                "On surface variant".into(),
            ],
            vec!["Small headline".into(), "Title large".into()],
            vec!["Medium flexible headline".into(), "Headline medium".into()],
            vec!["Large flexible headline".into(), "Display small".into()],
        ],
    );
}

#[tessera]
fn anatomy() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            combined_app_anatomy();
            gap(Dp(112.0));
            subheading("Search".to_owned());
            gap(Dp(24.0));
            search_figure(false);
            gap(Dp(112.0));
            subheading("Image".to_owned());
            gap(Dp(24.0));
            panel(RenderSlot::new(|| {
                sample("Image", TopAppBarVariant::Small).logo(true);
            }));
            gap(Dp(112.0));
            subheading("Filled trailing icon button".to_owned());
            gap(Dp(24.0));
            app_figure(TopAppBarVariant::Small, false).filled_action(true);
            gap(Dp(112.0));
            subheading("Subtitle".to_owned());
            gap(Dp(24.0));
            panel(RenderSlot::new(|| {
                for (variant, name) in [
                    (TopAppBarVariant::Small, "Small"),
                    (TopAppBarVariant::MediumFlexible, "Medium flexible"),
                    (TopAppBarVariant::LargeFlexible, "Large flexible"),
                ] {
                    sample(format!("Subtitle {name}"), variant).subtitle(true);
                    gap(Dp(32.0));
                }
            }));
        });
}

#[tessera]
fn colors() {
    column().modifier(Modifier::new().fill_max_width()).children(|| {
        for dark in [false, true] {
            material_theme().theme(move || MaterialTheme::from_seed(ACCENT, dark)).child(move || {
                panel(RenderSlot::new(move || {sample(format!("{} app bar", if dark {"Dark"} else {"Light"}), TopAppBarVariant::MediumFlexible).subtitle(true);}));
            });
            gap(Dp(24.0));
        }
        gap(Dp(88.0));
        subheading("Scroll states".to_owned()); gap(Dp(24.0));
        text().content("Scroll the saved items in the configured app bar to inspect its collapsed title and elevated container color. Reset scroll restores the expanded state.").style(type_style(16.0,24.0));
    });
}

#[tessera]
fn measurements() {
    search_figure(true);
    gap(Dp(32.0));
    for variant in [
        TopAppBarVariant::Small,
        TopAppBarVariant::MediumFlexible,
        TopAppBarVariant::LargeFlexible,
    ] {
        app_figure(variant, true);
        gap(Dp(32.0));
    }
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Small height".into(), "64dp".into()],
            vec!["Medium flexible height".into(), "112dp".into()],
            vec!["Medium flexible with subtitle".into(), "136dp".into()],
            vec!["Large flexible height".into(), "120dp".into()],
            vec!["Large flexible with subtitle".into(), "152dp".into()],
            vec!["Navigation and action target".into(), "48dp".into()],
            vec!["Icon size".into(), "24dp".into()],
            vec!["Title inset without navigation".into(), "16dp".into()],
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
    paragraph(
        "1  Container\n2  Leading button\n3  Trailing elements\n4  Headline\n5  Subtitle".into(),
    );
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
                .height(Dp(figure_width / (2048.0 / 610.0)))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!(
                        "Search app bar {} figure",
                        if measurements {
                            "measurement"
                        } else {
                            "anatomy"
                        }
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            for variant in [0_usize, 1] {
                search_pair_item(variant, measurements);
            }
        });
    gap(Dp(16.0));
    paragraph(if measurements {
        "Search app bar padding and size measurements".into()
    } else {
        "1  Container\n2  Leading icon button\n3  Hinted search text\n4  Trailing icon or avatar\n5  Search container".into()
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
    boxed()
        .modifier(watch(geometry, "figure").fill_max_size())
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .align(if variant == 0 {
                            Alignment::CenterStart
                        } else {
                            Alignment::CenterEnd
                        })
                        .width(Dp(width * (824.0 / 2048.0) + 44.0))
                        .offset(
                            Dp((width * (152.0 / 2048.0) - 22.0)
                                * if variant == 0 { 1.0 } else { -1.0 }),
                            Dp(0.0),
                        )
                        .padding_all(Dp(22.0)),
                )
                .children(move || {
                    search_sample()
                        .geometry(geometry)
                        .centered(true)
                        .specimen(variant);
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
            search.x + search.w + 12.0,
            search.y,
            search.h,
            true,
            "#b14e70",
        );
        annotations::dimension(
            format!("{:.0}", search.h),
            Dp(search.x + search.w + 16.0),
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
fn app_figure(variant: TopAppBarVariant, measurements: bool, filled_action: Option<bool>) {
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
        figure_width / (2048.0 / 600.0)
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
                    label: Some(format!(
                        "App bar {} figure",
                        if measurements {
                            "measurement"
                        } else {
                            "anatomy"
                        }
                    )),
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
        "App bar padding and size measurements".into()
    } else {
        "1  Container\n2  Leading button\n3  Trailing elements\n4  Headline\n5  Subtitle".into()
    });
    if actions.get() > 0 {
        text()
            .content(format!("App bar figure actions: {}", actions.get()))
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
    let mut bar = top_app_bar()
        .modifier(watch(geometry, "container"))
        .variant(variant)
        .title(if measurements {
            match variant {
                TopAppBarVariant::Small => "Title Large",
                TopAppBarVariant::MediumFlexible => "Headline Medium",
                TopAppBarVariant::LargeFlexible => "Display Small",
            }
        } else {
            "Headline"
        })
        .title_area_modifier(watch(geometry, "title"))
        .navigation_icon(move || {
            icon_button()
                .modifier(watch(geometry, "leading"))
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(filled::ARROW_BACK_SVG)
                .accessibility_label("Figure back")
                .on_click(move || actions.set(actions.get() + 1));
        })
        .action_shared(RenderSlot::new(move || {
            let mut action = icon_button()
                .modifier(watch(geometry, "trailing"))
                .icon(filled::SEARCH_SVG)
                .accessibility_label("Figure search")
                .on_click(move || actions.set(actions.get() + 1));
            action = if filled_action.unwrap_or(false) {
                action.filled()
            } else {
                action
                    .standard()
                    .color_style(IconButtonColorStyle::Inherited)
            };
            drop(action);
            icon_button()
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(filled::CALENDAR_TODAY_SVG)
                .accessibility_label("Figure calendar")
                .on_click(move || actions.set(actions.get() + 1));
        }));
    if !measurements {
        bar = bar.subtitle("Subtitle");
    }
    drop(bar);
}

#[tessera]
fn app_annotations(geometry: State<Geometry>, measurements: bool, variant: TopAppBarVariant) {
    let values = geometry.get();
    let (Some(root), Some(container), Some(leading), Some(trailing), Some(title)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "leading"),
        relative(&values, "trailing"),
        relative(&values, "title"),
    ) else {
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
                x: trailing.x + trailing.w,
                y: container.y,
                w: container.x + container.w - trailing.x - trailing.w,
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
                format!("{:.0}", container.x + container.w - trailing.x - trailing.w),
                container.x + container.w - 8.0,
                container.y - 32.0,
            ),
            (
                format!("{:.0}", container.y + container.h - title.y - title.h),
                title.x + title.w + 8.0,
                title.y + title.h,
            ),
        ] {
            annotations::dimension(label, Dp(x), Dp(y));
        }
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
                    w: trailing.w + 48.0,
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
    use std::cell::Cell;

    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    };

    use super::*;

    #[derive(Clone, Copy)]
    enum Case {
        Bar(TopAppBarVariant),
        Search(f64),
    }

    thread_local! {
        static CASE: Cell<Case> = const { Cell::new(Case::Bar(TopAppBarVariant::Small)) };
        static GEOMETRY: Cell<Option<State<Geometry>>> = const { Cell::new(None) };
    }

    struct ComponentPipelines;
    impl tessera_ui::RenderModule for ComponentPipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[tessera]
    fn fixture() {
        let case = CASE.get();
        let geometry = remember(Geometry::new);
        GEOMETRY.set(Some(geometry));
        material_theme().child(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(move || match case {
                    Case::Bar(variant) => {
                        app_figure(variant, true);
                    }
                    Case::Search(width) => {
                        boxed()
                            .modifier(Modifier::new().width(Dp(width)))
                            .children(move || {
                                search_sample().geometry(geometry);
                            });
                    }
                });
        });
    }

    fn renderer(case: Case) -> HeadlessRenderer {
        CASE.set(case);
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(ComponentPipelines)],
            HeadlessConfig {
                width: 1200,
                height: 800,
                sample_count: 1,
                ..Default::default()
            },
        )
        .expect("native app bar figures");
        renderer.render_frames(6);
        renderer
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
            let renderer = renderer(Case::Bar(variant));
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
            let back = snapshot
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Button && node.label() == Some("Figure back")
                })
                .expect("actual navigation button")
                .1
                .bounds()
                .unwrap();
            assert!(back.width() >= 48.0 && back.height() >= 48.0);
        }
        for (width, field_width) in [(400.0, 280.0), (600.0, 396.0), (1000.0, 596.0)] {
            let mut renderer = renderer(Case::Search(width));
            let geometry = GEOMETRY.get().expect("search geometry").get();
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
                        && node.label() == Some("Search library view")),
                "search app bar opens its real search view"
            );
            let close = snapshot
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Button && node.label() == Some("Close app bar search")
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
                        && node.label() == Some("Search library view"))
            );
        }
    }
}
