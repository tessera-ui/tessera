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
    Dp, LayoutCoordinates, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, remember_with_key, tessera, use_context,
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
            SpecSection {
                title: "Baseline app bars",
                content: RenderSlot::new(baseline_app_bars),
            },
        ],
    ).pages(vec![
        ArticlePage {
            title: "Overview", intro: Some(RenderSlot::new(|| {
                paragraph("App bars identify the current page and keep navigation and one or two essential actions within reach. Choose search, small, medium flexible or large flexible layouts.".into());
                gap(Dp(32.0)); variants();
            })), sections: vec![
                section("Availability & resources", RenderSlot::new(|| {
                    spec_table(
                        vec!["Type".into(), "Resource".into(), "Status".into()],
                        vec![
                            vec!["Design".into(), "Design Kit (Figma)".into(), "Available".into()],
                            vec!["Implementation".into(), "Flutter".into(), "Available".into()],
                            vec!["Implementation".into(), "Jetpack Compose".into(), "Available".into()],
                            vec!["Implementation".into(), "Jetpack Compose: Expressive".into(), "Available".into()],
                            vec!["Implementation".into(), "Android Views (MDC-Android)".into(), "Available".into()],
                            vec!["Implementation".into(), "Android Views (MDC-Android): Expressive".into(), "Available".into()],
                            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
                            vec!["Implementation".into(), "Web: Expressive".into(), "Unavailable".into()],
                        ],
                    );
                    gap(Dp(24.0));
                    paragraph("Tessera examples use the native top_app_bar component for the small, medium flexible and large flexible variants, and the native search bar inside an app bar for the search variant.".into());
                })),
                section("M3 Expressive update", RenderSlot::new(|| {
                    paragraph("Renamed from top app bar to app bar, and added the search app bar. Baseline medium and large app bars are no longer recommended. Medium flexible and large flexible app bars reduce overall height and add a larger title, an optional subtitle, leading or centered text alignment and text wrapping.".into());
                    gap(Dp(16.0));
                    paragraph("The small app bar also gained a subtitle, a center-aligned text option and more flexible elements for imagery and filled buttons.".into());
                    gap(Dp(24.0)); sample("Expressive overview", TopAppBarVariant::MediumFlexible).subtitle(true).controls(true);
                })),
                section("Differences from M2", RenderSlot::new(|| {
                    paragraph("Color: new color mappings with dynamic color support.\nOn scroll: no drop shadow; a color fill separates the bar from content.\nTypography: larger default text.\nLayout: smaller default height.".into());
                    gap(Dp(24.0)); app_figure(TopAppBarVariant::Small, false);
                })),
            ],
        },
        ArticlePage {
            title: "Guidelines", intro: None, sections: vec![
                section("Usage", RenderSlot::new(|| {
                    paragraph("App bars show information about the page, key actions, and navigation actions like Back or Menu. Use one action, or two only when the second is essential. The primary action changes or exits the whole page, such as Send, Save, or Edit.".into());
                    gap(Dp(16.0));
                    paragraph("When a product has many page actions, place them in a toolbar instead of adding them here, and avoid an overflow menu in the app bar when possible. To emphasize a single important action, give one icon button a filled or tonal style; never place two filled or tonal buttons side by side.".into());
                    gap(Dp(24.0)); app_figure(TopAppBarVariant::Small, false).filled_action(true);
                    gap(Dp(16.0));
                    paragraph("Use a filled or tonal button for important actions. Do not put multiple filled or tonal buttons in the app bar.".into());
                    gap(Dp(32.0)); subheading("Variants".into());
                    paragraph("Search app bar for home pages where search is key to the product. Small for dense layouts or a scrolled page. Medium flexible to show a larger headline that can collapse into a small app bar on scroll. Large flexible to emphasize the page headline.".into());
                })),
                section("Search app bar", RenderSlot::new(|| {
                    paragraph("A search app bar replaces heading text with a search field and provides an emphasized entry point to the search view. The search bar should always include the word Search, and can identify the search scope, such as Search in messages.".into());
                    gap(Dp(24.0)); search_sample();
                    gap(Dp(32.0)); subheading("Buttons in search app bar".into());
                    paragraph("The leading element can be a product logo, either purely cosmetic or opening a known destination. On compact windows, do not use more than two trailing icon buttons next to an avatar; move additional actions into a toolbar.".into());
                    gap(Dp(24.0)); subheading("Large screens".into());
                    paragraph("Larger windows allow up to four trailing icons. The search field fills the space between leading and trailing elements until it reaches 312dp, then grows at half the rate of the remaining space.".into());
                    gap(Dp(24.0)); subheading("Alternate color options".into());
                    paragraph("By default the search container uses surface container so it stands out from the app background. On darker backgrounds use a lighter role such as surface bright, and keep at least 3:1 contrast between the search label and its container.".into());
                })),
                section("Anatomy", RenderSlot::new(|| {
                    app_figure(TopAppBarVariant::MediumFlexible, false);
                    for (title, body) in [
                        ("Container", "The container holds navigation, headline and actions at the top of a screen. Keep its straight corners, default height and full window width; do not reduce the height or add curved corners, which imply the bar can expand."),
                        ("Adding logos", "Use a high-quality, relevant image. It can replace all text in a small app bar and appears above the text in other variants without disrupting navigation or actions."),
                        ("Leading button", "Use Back to return to the previous screen, or Menu to open product navigation. Keep the leading button aligned to the leading edge."),
                        ("Headline", "Describe the current page, section, or product briefly. Flexible headlines can wrap to a second line; small headlines stay on one line and are never truncated."),
                        ("Subtitle", "Small uses label medium, medium flexible uses label large, and large flexible uses title medium. Align it with its headline."),
                        ("Trailing icon buttons", "Place up to two actions after the headline, most-used closest to the leading edge. Prefer filled icons for visibility; only one action may use a filled or tonal style."),
                    ] { gap(Dp(24.0)); subheading(title.into()); paragraph(body.into()); }
                })),
                section("Adaptive design", RenderSlot::new(|| {
                    subheading("Resizing".into());
                    paragraph("The app bar container spans the available width. In an adaptive layout, move secondary actions into an overflow menu at smaller breakpoints and restore them when space returns. The examples here keep their actions visible while the search field resizes between its leading and trailing controls.".into());
                    gap(Dp(24.0)); search_sample();
                    gap(Dp(32.0)); subheading("Presentation".into());
                    paragraph("Align navigation and actions to the leading and trailing edges, and mirror their placement for right-to-left layouts.".into());
                })),
                section("Behavior", RenderSlot::new(|| {
                    subheading("Scrolling".into());
                    paragraph("A contrasting container separates scrolled content; the bar can hide while scrolling up and reveal while scrolling down. Medium flexible and large flexible bars can compress into a small app bar on scroll and stay small until the page returns to the top. Never transform an app bar into a search app bar.".into());
                    gap(Dp(24.0)); sample("Scrolling app bar", TopAppBarVariant::MediumFlexible).subtitle(true).controls(true);
                    gap(Dp(32.0)); subheading("Search and transparency".into());
                    paragraph("Selecting the search field opens the search view. To focus on body content, a bar can stay transparent on scroll so its actions float above the content; in that case give the icon buttons a container fill.".into());
                    gap(Dp(24.0)); search_sample();
                })),
            ],
        },
        ArticlePage {
            title: "Accessibility", intro: None, sections: vec![
                section("Use cases", RenderSlot::new(|| {
                    paragraph("Assistive technology should let people understand which page they are visiting, take actions or navigate to a new destination, and keep access to app bar actions while the content is scrolled.".into());
                    gap(Dp(24.0)); app_figure(TopAppBarVariant::Small, false);
                })),
                section("Interaction & style", RenderSlot::new(|| {
                    for (title, body) in [
                        ("Touch", "Tapping an app bar icon button shows a touch ripple as interaction feedback, and every action keeps a 48×48dp target."),
                        ("Cursor", "Hover shows a state layer and a click adds a ripple, in both active and inactive states."),
                        ("Keyboard or switch", "A focus indicator identifies the action that will activate. Put the leading button first in the app bar's keyboard traversal order."),
                        ("Tab and activation", "Tab moves focus from the leading button to each trailing action. Space or Enter activates the focused action."),
                    ] {
                        subheading(title.into()); paragraph(body.into()); gap(Dp(24.0));
                    }
                    sample("Accessible app bar", TopAppBarVariant::Small).controls(true);
                })),
                section("Color", RenderSlot::new(|| {
                    paragraph("On search app bars, use the default color roles when possible: the search container uses surface container and the search label uses on surface variant. On darker backgrounds the container can use surface bright. When mapping other roles, keep at least 3:1 contrast between the search text and its container.".into());
                    gap(Dp(24.0)); search_sample();
                })),
                section("Keyboard navigation", RenderSlot::new(|| {
                    spec_table(vec!["Keys".into(), "Actions".into()], vec![vec!["Tab".into(), "Focus the next action".into()], vec!["Space or Enter".into(), "Activate the focused action".into()]]);
                    gap(Dp(24.0)); sample("Keyboard app bar", TopAppBarVariant::Small).controls(true);
                })),
                section("Labeling elements", RenderSlot::new(|| {
                    paragraph("Keep the headline text and its accessibility label consistent. Expose a page title or heading role when implementing screen-reader heading navigation, and add context to the label when it helps people understand which page they are on. The native text titles in these examples currently expose a text label.".into());
                    gap(Dp(16.0));
                    paragraph("Label each icon button by the action it performs, such as Back, Search, or View on map. Screen readers read the label followed by the Button role.".into());
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
                                let logo_slot = RenderSlot::new(brand);
                                top_app_bar()
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
                                    .title_content_optional(logo.get().then_some(logo_slot))
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
                                        if filled_action.get() {
                                            icon_button()
                                                .filled()
                                                .icon(filled::SEARCH_SVG)
                                                .accessibility_label(format!(
                                                    "{} search",
                                                    label.get()
                                                ))
                                                .on_click(move || {
                                                    status.set("Search selected".to_owned())
                                                });
                                        } else {
                                            icon_button()
                                                .standard()
                                                .color_style(IconButtonColorStyle::Inherited)
                                                .icon(filled::SEARCH_SVG)
                                                .accessibility_label(format!(
                                                    "{} search",
                                                    label.get()
                                                ))
                                                .on_click(move || {
                                                    status.set("Search selected".to_owned())
                                                });
                                        }
                                    }));
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
                        .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
                            let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
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
                                    let leading_icon = RenderSlot::new(|| {
                                        icon().vector(filled::SEARCH_SVG).size(Dp(24.0));
                                    });
                                    let trailing_icon = RenderSlot::new(|| {
                                        icon().vector(filled::MIC_SVG).size(Dp(24.0));
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
                                        })
                                        .leading_icon_optional(
                                            specimen.is_none().then_some(leading_icon),
                                        )
                                        .trailing_icon_optional(
                                            (specimen == Some(1)).then_some(trailing_icon),
                                        );
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
            vec!["Search container".into(), "Surface container".into()],
            vec!["Search label".into(), "On surface variant".into()],
            vec![
                "Search container on darker surfaces".into(),
                "Surface bright".into(),
            ],
        ],
    );
}

#[tessera]
fn baseline_app_bars() {
    paragraph("The baseline medium and large app bars are no longer recommended in M3 Expressive. Use the medium flexible and large flexible app bars instead: they look similar but support multiple lines, use a shorter height, and can hold a wider variety of elements such as images.".into());
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        for (label, variant) in [
            (
                "Medium flexible replaces baseline medium",
                TopAppBarVariant::MediumFlexible,
            ),
            (
                "Large flexible replaces baseline large",
                TopAppBarVariant::LargeFlexible,
            ),
        ] {
            sample(label, variant).subtitle(true);
            gap(Dp(32.0));
        }
    }));
    gap(Dp(16.0));
    paragraph("Baseline medium and large placed the headline in a taller container and did not support multi-line headlines or subtitles. Their flexible replacements keep the same compact navigation and action targets while letting the title wrap to two lines.".into());
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
            vec!["Search field height".into(), "56dp".into()],
            vec!["Search field growth threshold".into(), "312dp".into()],
        ],
    );
    gap(Dp(24.0));
    paragraph("Flexible app bars hug their text contents, so a subtitle or a wrapping headline can make the bar taller. The small app bar stays 64dp tall with or without a subtitle. Navigation and action targets stay at 48dp.".into());
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
                .height(Dp(if figure_width < 800.0 {
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
    let stacked = width < 800.0;
    boxed()
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(if stacked {
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
                        .align(if stacked {
                            Alignment::Center
                        } else if variant == 0 {
                            Alignment::CenterStart
                        } else {
                            Alignment::CenterEnd
                        })
                        .width(Dp(if stacked {
                            (width - 68.0).max(44.0)
                        } else {
                            width * (824.0 / 2048.0) + 44.0
                        }))
                        .offset(
                            Dp(if stacked {
                                0.0
                            } else {
                                (width * (152.0 / 2048.0) - 22.0)
                                    * if variant == 0 { 1.0 } else { -1.0 }
                            }),
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
    top_app_bar()
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
            if filled_action.unwrap_or(false) {
                icon_button()
                    .modifier(watch(geometry, "trailing"))
                    .filled()
                    .icon(filled::SEARCH_SVG)
                    .accessibility_label("Figure search")
                    .on_click(move || actions.set(actions.get() + 1));
            } else {
                icon_button()
                    .modifier(watch(geometry, "trailing"))
                    .standard()
                    .color_style(IconButtonColorStyle::Inherited)
                    .icon(filled::SEARCH_SVG)
                    .accessibility_label("Figure search")
                    .on_click(move || actions.set(actions.get() + 1));
            }
            icon_button()
                .modifier(watch(geometry, "trailing-last"))
                .standard()
                .color_style(IconButtonColorStyle::Inherited)
                .icon(filled::CALENDAR_TODAY_SVG)
                .accessibility_label("Figure calendar")
                .on_click(move || actions.set(actions.get() + 1));
        }))
        .subtitle_optional((!measurements).then(|| "Subtitle".to_owned()));
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
    use std::cell::Cell;

    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition, PxRect,
        scaled_box,
    };

    use super::*;

    #[derive(Clone, Copy)]
    enum Case {
        Bar(TopAppBarVariant),
        Search(f64, f32),
        SearchMeasurements(f64),
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
                    Case::SearchMeasurements(width) => {
                        boxed()
                            .modifier(Modifier::new().width(Dp(width)))
                            .children(|| {
                                search_figure(true);
                            });
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
            let calendar = snapshot
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Button && node.label() == Some("Figure calendar")
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
                    .any(|(_, node)| node.label() == Some("Dimension 4"))
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
                        .any(|(_, node)| node.label() == Some("Dimension 0"))
                );
            }
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
            let mut renderer = renderer(Case::Search(width, 1.0));
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
        for scale in [0.5, 1.5] {
            let _renderer = renderer(Case::Search(600.0, scale));
            let geometry = GEOMETRY.get().expect("scaled search geometry").get();
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
            let renderer = renderer(Case::SearchMeasurements(width));
            let snapshot = renderer
                .accessibility_snapshot()
                .expect("measurement semantics");
            let labels: Vec<_> = snapshot
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == Role::Note && node.label() == Some("Dimension 56")
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
}
