//! Shared layout for interactive specification articles.
//!
//! ## Usage
//!
//! Read component specifications and navigate directly to their live examples.

use std::collections::BTreeMap;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    icon::icon,
    icon_button::icon_button,
    interaction_state::InteractionState,
    material_icons::filled,
    modifier::{ClickableArgs, ModifierExt as _, Padding, SemanticsArgs, ShadowArgs},
    row::row,
    scrollable::ScrollableController,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Callback, Dp, FocusManager, LayoutCoordinates, Modifier, Px, PxPosition, RenderSlot, State,
    accesskit::Role, current_frame_nanos, modifier::FocusModifierExt as _, receive_frame_nanos,
    remember, tessera, use_context,
};

use super::{
    BACKGROUND, INK,
    article_navigation::{
        ARTICLE_TAB_TITLES, ArticleNavigationState, article_tab_focus_visible,
        article_tab_modifier, article_tablist_modifier, focus_article_tab, focused_article_tab,
        sync_article_navigation,
    },
    article_tab_visual, gap, heading, media, srgb, type_style,
};

#[path = "article_modal.rs"]
pub(super) mod modal_host;

#[path = "article_lazy.rs"]
mod lazy_sections;

#[derive(Clone, PartialEq)]
pub(super) struct ArticleDestination {
    pub title: &'static str,
    pub on_open: Callback,
}

#[derive(Clone, PartialEq)]
pub(super) struct ArticleRoute {
    pub initial_page: &'static str,
    pub previous: Option<ArticleDestination>,
    pub next: Option<ArticleDestination>,
}

#[derive(Clone, PartialEq)]
pub(super) struct SpecSection {
    pub title: &'static str,
    pub content: RenderSlot,
}

#[derive(Clone, PartialEq)]
pub(super) struct ArticlePage {
    pub title: &'static str,
    pub intro: Option<RenderSlot>,
    pub sections: Vec<SpecSection>,
}

#[derive(Clone, PartialEq)]
struct ArticleData {
    title: String,
    description: String,
    hero: Option<RenderSlot>,
    intro: Option<RenderSlot>,
    sections: Vec<SpecSection>,
    compact_content_inset: Option<Dp>,
    page: &'static str,
    section_heading_level: u8,
}

#[tessera]
pub(super) fn article(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    sections: Vec<SpecSection>,
    hero: Option<RenderSlot>,
    intro: Option<RenderSlot>,
    compact_content_inset: Option<Dp>,
    pages: Option<Vec<ArticlePage>>,
    page_heading_levels: Option<BTreeMap<&'static str, u8>>,
) {
    let route = use_context::<ArticleRoute>().map(|route| route.get());
    let initial_page = route.as_ref().map_or("Specs", |route| route.initial_page);
    let initial_page = if initial_page == "Specs"
        || pages
            .as_ref()
            .is_some_and(|pages| pages.iter().any(|page| page.title == initial_page))
    {
        initial_page
    } else {
        "Specs"
    };
    let selected = remember(|| initial_page);
    let alternate = pages
        .as_ref()
        .and_then(|pages| pages.iter().find(|page| page.title == selected.get()));
    let (sections, intro, hero) = if let Some(page) = alternate {
        (page.sections.clone(), page.intro, None)
    } else {
        (sections, intro, hero)
    };
    let current = ArticleData {
        title,
        description,
        sections,
        hero,
        intro,
        compact_content_inset,
        page: selected.get(),
        section_heading_level: page_heading_levels
            .as_ref()
            .and_then(|levels| levels.get(selected.get()))
            .copied()
            .unwrap_or(2),
    };
    let data = remember(|| current.clone());
    let scroll = remember(ScrollableController::new);
    let width = remember(|| Dp(900.0));
    if data.with(|data| data != &current) {
        if data.with(|data| data.title != current.title) {
            scroll.with_mut(|scroll| scroll.set_scroll_position(PxPosition::ZERO));
        }
        data.set(current);
    }
    modal_host::article_modal_host(RenderSlot::new(move || {
        let pages = pages.clone();
        let route = route.clone();
        surface()
            .style(BACKGROUND.into())
            .modifier(Modifier::new().fill_max_size())
            .child(move || {
                let pages = pages.clone();
                let route = route.clone();
                lazy_sections::viewport(data, selected, scroll, width)
                    .pages_optional(pages.clone())
                    .route_optional(route.clone());
            });
    }));
}

#[tessera]
fn article_section(index: usize, data: State<ArticleData>, inset: Dp) {
    let section = data.with(|data| data.sections[index].clone());
    column()
        .modifier(Modifier::new().fill_max_width().padding(Padding::new(
            inset,
            Dp(80.0),
            inset,
            Dp(32.0),
        )))
        .children(move || {
            boxed()
                .modifier(Modifier::new().padding_symmetric(Dp(1.0), Dp::ZERO))
                .children(move || {
                    heading(section.title.to_owned())
                        .compact(inset.0 <= 24.0)
                        .level(data.with(|page| page.section_heading_level));
                });
            gap(Dp(80.0));
            section.content.render();
            let page = data.get();
            let live_section = page
                .sections
                .iter()
                .position(|section| matches!(section.title, "Configurations" | "Configuration"))
                .or_else(|| {
                    page.sections
                        .iter()
                        .position(|section| section.title == "States")
                })
                .unwrap_or(0);
            if index == live_section
                && let Some(hero) = page.hero
            {
                gap(Dp(32.0));
                hero.render();
            }
        });
}

#[derive(Clone, PartialEq)]
enum FooterAction {
    Page(&'static str),
    Component(Callback),
}

#[derive(Clone, PartialEq)]
struct FooterLink {
    title: String,
    action: FooterAction,
}

#[derive(Clone, PartialEq)]
struct FooterData {
    previous: Option<FooterLink>,
    next: Option<FooterLink>,
}

fn footer_links(
    title: &str,
    selected: &str,
    pages: &[ArticlePage],
    route: Option<&ArticleRoute>,
) -> FooterData {
    let available = ARTICLE_TAB_TITLES
        .into_iter()
        .filter(|label| *label == "Specs" || pages.iter().any(|page| page.title == *label))
        .collect::<Vec<_>>();
    let index = available
        .iter()
        .position(|label| *label == selected)
        .unwrap_or(0);
    let local = |page: &'static str| FooterLink {
        title: format!("{title}: {page}"),
        action: FooterAction::Page(page),
    };
    let external = |destination: &ArticleDestination| FooterLink {
        title: format!("{}: Overview", destination.title),
        action: FooterAction::Component(destination.on_open),
    };
    FooterData {
        previous: index
            .checked_sub(1)
            .map(|index| local(available[index]))
            .or_else(|| {
                route
                    .and_then(|route| route.previous.as_ref())
                    .map(external)
            }),
        next: available
            .get(index + 1)
            .copied()
            .map(local)
            .or_else(|| route.and_then(|route| route.next.as_ref()).map(external)),
    }
}

#[tessera]
fn article_footer(
    title: String,
    selected: State<&'static str>,
    scroll: State<ScrollableController>,
    pages: Option<Vec<ArticlePage>>,
    route: Option<ArticleRoute>,
) {
    let next = footer_links(
        &title,
        selected.get(),
        pages.as_deref().unwrap_or_default(),
        route.as_ref(),
    );
    let data = remember(|| next.clone());
    if data.with(|data| data != &next) {
        data.set(next);
    }
    row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(8.0), Dp::ZERO)
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some("Previous and up next articles".into()),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            if data.with(|data| data.previous.is_some()) {
                article_footer_link(true, data, selected, scroll);
            } else {
                spacer().modifier(Modifier::new().weight(1.0));
            }
            spacer().modifier(Modifier::new().width(Dp(8.0)));
            if data.with(|data| data.next.is_some()) {
                article_footer_link(false, data, selected, scroll);
            } else {
                spacer().modifier(Modifier::new().weight(1.0));
            }
        });
}

#[tessera]
fn article_footer_link(
    previous: bool,
    data: State<FooterData>,
    selected: State<&'static str>,
    scroll: State<ScrollableController>,
) {
    let link = data
        .with(|data| {
            if previous {
                data.previous.clone()
            } else {
                data.next.clone()
            }
        })
        .expect("article footer destination");
    let label = if previous { "Previous" } else { "Up next" };
    let corner = RoundedCorner::manual(Dp(24.0), 2.0);
    boxed()
        .modifier(
            Modifier::new()
                .weight(1.0)
                .background_with_shape(
                    srgb(248, 241, 246),
                    Shape::RoundedRectangle {
                        top_left: corner,
                        top_right: corner,
                        bottom_left: corner,
                        bottom_right: corner,
                    },
                )
                .clickable_with(ClickableArgs {
                    role: Some(Role::Link),
                    label: Some(format!("{label} {}", link.title)),
                    on_click: Callback::new(move || {
                        let link = data.with(|data| {
                            if previous {
                                data.previous.clone()
                            } else {
                                data.next.clone()
                            }
                        });
                        if let Some(link) = link {
                            FocusManager::current().clear_focus(false);
                            FocusManager::current().defer_keyboard_until_layout();
                            match link.action {
                                FooterAction::Page(page) => {
                                    selected.set(page);
                                    scroll.with_mut(|scroll| {
                                        scroll.set_scroll_position(PxPosition::ZERO)
                                    });
                                }
                                FooterAction::Component(on_open) => on_open.call(),
                            }
                        }
                    }),
                    ..Default::default()
                })
                .padding_all(Dp(24.0)),
        )
        .children(move || {
            let title = link.title.clone();
            column().children(move || {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        if previous {
                            icon()
                                .vector(filled::ARROW_RIGHT_ALT_SVG)
                                .rotation(180.0)
                                .size(Dp(24.0))
                                .tint(INK);
                        }
                        text()
                            .content(label)
                            .style(type_style(16.0, 24.0))
                            .color(INK);
                        if !previous {
                            icon()
                                .vector(filled::ARROW_RIGHT_ALT_SVG)
                                .size(Dp(24.0))
                                .tint(INK);
                        }
                    });
                gap(Dp(8.0));
                text()
                    .content(title.clone())
                    .style(type_style(24.0, 32.0))
                    .color(INK);
            });
        });
}

#[tessera]
fn article_tabs(
    selected: State<&'static str>,
    scroll: State<ScrollableController>,
    pages: Option<Vec<ArticlePage>>,
    pinned: Option<State<bool>>,
) {
    let width = remember(|| Dp(900.0));
    let pinned = if let Some(pinned) = pinned {
        pinned
    } else {
        remember(|| false)
    };
    // Preserve the inline flow extent when the pinned navigation contracts.
    boxed()
        .alignment(Alignment::TopCenter)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(88.0))
                .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
                    let next = Dp::from(coordinates.local_size.width);
                    if width.get() != next {
                        width.set(next);
                    }
                }),
        )
        .children(move || {
            let corner = RoundedCorner::manual(Dp(40.0), 2.0);
            spacer().modifier(
                Modifier::new()
                    .fill_max_width()
                    .height(Dp(if pinned.get() { 72.0 } else { 88.0 }))
                    .background_with_shape(
                        BACKGROUND,
                        Shape::RoundedRectangle {
                            top_left: RoundedCorner::manual(Dp::ZERO, 2.0),
                            top_right: RoundedCorner::manual(Dp::ZERO, 2.0),
                            bottom_left: corner,
                            bottom_right: corner,
                        },
                    ),
            );
            article_tab_row(selected, scroll, width, pinned).pages_optional(pages.clone());
        });
}

#[tessera]
fn article_tab_row(
    selected: State<&'static str>,
    scroll: State<ScrollableController>,
    width: State<Dp>,
    pinned: State<bool>,
    pages: Option<Vec<ArticlePage>>,
) {
    media::load_fonts();
    let horizontal = remember(ScrollableController::new);
    let contents = remember(|| [Dp(160.0); 4]);
    let density = tessera_ui::dp::scale_factor();
    let content_density = remember(|| density);
    if content_density.get() != density {
        content_density.set(density);
        contents.set([Dp(160.0); 4]);
    }
    let available = ["Overview", "Specs", "Guidelines", "Accessibility"].map(|label| {
        label == "Specs"
            || pages
                .as_ref()
                .is_some_and(|pages| pages.iter().any(|page| page.title == label))
    });
    let navigation = remember(ArticleNavigationState::default);
    sync_article_navigation(navigation, available, selected.get());
    let now = current_frame_nanos();
    let targets = ARTICLE_TAB_TITLES.map(|title| if selected.get() == title { 1.0 } else { 0.0 });
    let motions =
        remember(|| targets.map(|target| article_tab_visual::SelectionMotion::new(target, now)));
    if motions.with(|motions| {
        motions
            .iter()
            .zip(targets)
            .any(|(motion, target)| motion.target() != target)
    }) {
        motions.with_mut(|motions| {
            for (motion, target) in motions.iter_mut().zip(targets) {
                motion.retarget(target, now);
            }
        });
    }
    if motions.with(|motions| motions.iter().any(|motion| motion.is_running())) {
        receive_frame_nanos(move |frame| {
            motions.with_mut(|motions| {
                for motion in motions.iter_mut().filter(|motion| motion.is_running()) {
                    motion.advance(frame);
                }
            });
            if motions.with(|motions| motions.iter().any(|motion| motion.is_running())) {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let progress = motions.with(|motions| motions.map(|motion| motion.value()));
    let full_width = article_tab_width(width.get(), pinned.get());
    let minimums = article_tab_minimums(contents);
    let minimum: i32 = minimums
        .iter()
        .enumerate()
        .filter(|(index, _)| available[*index])
        .map(|(_, width)| width.0)
        .sum();
    let overflow = minimum > full_width.0;
    let previous = overflow && adjacent_article_tab(selected.get(), available, true).is_some();
    let next = overflow && adjacent_article_tab(selected.get(), available, false).is_some();
    let control_width = Dp(48.0).to_px().0 + Dp(8.0).to_px().0;
    let viewport_width =
        Px((full_width.0 - control_width * (i32::from(previous) + i32::from(next))).max(0));
    row()
        .modifier(
            Modifier::new()
                .width(Dp::from(full_width))
                .offset(Dp::ZERO, Dp(9.0)),
        )
        .children(move || {
            if previous {
                article_navigation_arrow(
                    selected, scroll, horizontal, width, pinned, contents, navigation, true,
                );
                spacer().modifier(Modifier::new().width(Dp(8.0)));
            }
            boxed()
                .modifier(
                    Modifier::new()
                        .width(Dp::from(viewport_width))
                        .height(Dp(if pinned.get() { 63.0 } else { 79.0 })),
                )
                .children(move || {
                    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                    spacer().modifier(
                        Modifier::new()
                            .fill_max_size()
                            .shadow(
                                &scheme,
                                &ShadowArgs {
                                    elevation: if pinned.get() { Dp(1.0) } else { Dp::ZERO },
                                    shape: Shape::CAPSULE,
                                    ..Default::default()
                                },
                            )
                            .background_with_shape(srgb(248, 241, 246), Shape::CAPSULE),
                    );
                    article_tab_strip(
                        selected,
                        scroll,
                        horizontal,
                        pinned,
                        contents,
                        navigation,
                        available,
                        progress,
                        viewport_width,
                    );
                });
            if next {
                spacer().modifier(Modifier::new().width(Dp(8.0)));
                article_navigation_arrow(
                    selected, scroll, horizontal, width, pinned, contents, navigation, false,
                );
            }
        });
}

fn article_tab_width(width: Dp, pinned: bool) -> Px {
    let available = (width.0 - 16.0).max(0.0);
    Dp(if pinned {
        available.min(1024.0)
    } else {
        available
    })
    .to_px()
}

fn article_tab_minimums(contents: State<[Dp; 4]>) -> [Px; 4] {
    let side_padding = Dp(24.0).to_px();
    contents.with(|contents| contents.map(|width| Px(width.to_px().0 + 2 * side_padding.0)))
}

fn article_tab_sizes(minimums: [Px; 4], available: [bool; 4], viewport: Px) -> [Px; 4] {
    let minimum: i32 = minimums
        .iter()
        .enumerate()
        .filter(|(index, _)| available[*index])
        .map(|(_, width)| width.0)
        .sum();
    let mut remaining = available.iter().filter(|value| **value).count() as i32;
    let mut extra = (viewport.0 - minimum).max(0);
    std::array::from_fn(|index| {
        if !available[index] {
            return Px::ZERO;
        }
        let share = extra / remaining;
        extra -= share;
        remaining -= 1;
        Px(minimums[index].0 + share)
    })
}

fn reveal_article_tab(
    horizontal: State<ScrollableController>,
    sizes: [Px; 4],
    target: usize,
    viewport: Px,
    reset: bool,
) {
    let start: i32 = sizes.iter().take(target).map(|width| width.0).sum();
    let end = start + sizes[target].0;
    let position = horizontal.with(|scroll| scroll.child_position());
    let offset = if reset { 0.0 } else { position.x };
    let offset = if start as f32 + offset < 0.0 {
        -(start as f32)
    } else if end as f32 + offset > viewport.0 as f32 {
        (viewport.0 - end) as f32
    } else {
        offset
    };
    let next = PxPosition::from_f32_xy(offset, position.y);
    if next != position {
        horizontal.with_mut(|scroll| scroll.set_scroll_position(next));
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ArticleTabReveal {
    selected: &'static str,
    minimums: [Px; 4],
    available: [bool; 4],
    viewport: Px,
}

fn adjacent_article_tab(selected: &str, available: [bool; 4], previous: bool) -> Option<usize> {
    let current = ARTICLE_TAB_TITLES
        .iter()
        .position(|title| *title == selected)?;
    if previous {
        (0..current).rev().find(|index| available[*index])
    } else {
        (current + 1..4).find(|index| available[*index])
    }
}

#[tessera]
fn article_navigation_arrow(
    selected: State<&'static str>,
    scroll: State<ScrollableController>,
    horizontal: State<ScrollableController>,
    width: State<Dp>,
    pinned: State<bool>,
    contents: State<[Dp; 4]>,
    navigation: State<ArticleNavigationState>,
    previous: bool,
) {
    icon_button()
        .standard()
        .modifier(Modifier::new().size(Dp(48.0), Dp(if pinned.get() { 63.0 } else { 79.0 })))
        .shape(Shape::CAPSULE)
        .color(srgb(248, 241, 246))
        .content_color(srgb(77, 66, 86))
        .icon(if previous {
            filled::ARROW_BACK_SVG
        } else {
            filled::ARROW_FORWARD_SVG
        })
        .accessibility_label(if previous {
            "Go to previous tab"
        } else {
            "Go to next tab"
        })
        .on_click(move || {
            let available = navigation.with(|navigation| navigation.available);
            let Some(target) = adjacent_article_tab(selected.get(), available, previous) else {
                return;
            };
            let minimums = article_tab_minimums(contents);
            let first = available.iter().position(|value| *value).unwrap();
            let last = available.iter().rposition(|value| *value).unwrap();
            let controls = i32::from(target > first) + i32::from(target < last);
            let viewport = article_tab_width(width.get(), pinned.get()).0
                - controls * (Dp(48.0).to_px().0 + Dp(8.0).to_px().0);
            let sizes = article_tab_sizes(minimums, available, Px(viewport));
            reveal_article_tab(horizontal, sizes, target, Px(viewport), false);
            selected.set(ARTICLE_TAB_TITLES[target]);
            scroll.with_mut(|scroll| scroll.set_scroll_position(PxPosition::ZERO));
            if (previous && target == first) || (!previous && target == last) {
                focus_article_tab(navigation, target);
            } else {
                FocusManager::current().defer_keyboard_until_layout();
            }
        });
}

#[tessera]
fn article_tab_strip(
    selected: State<&'static str>,
    scroll: State<ScrollableController>,
    horizontal: State<ScrollableController>,
    pinned: State<bool>,
    contents: State<[Dp; 4]>,
    navigation: State<ArticleNavigationState>,
    available: [bool; 4],
    progress: [f32; 4],
    viewport_width: Px,
) {
    let minimums = article_tab_minimums(contents);
    let reveal = ArticleTabReveal {
        selected: selected.get(),
        minimums,
        available,
        viewport: viewport_width,
    };
    let last_reveal = remember(|| None::<ArticleTabReveal>);
    let previous_reveal = last_reveal.get();
    if previous_reveal != Some(reveal) {
        last_reveal.set(Some(reveal));
        let selected_index = ARTICLE_TAB_TITLES
            .iter()
            .position(|title| *title == selected.get())
            .unwrap_or(1);
        let target = if previous_reveal.is_none_or(|previous| previous.selected != reveal.selected)
        {
            selected_index
        } else {
            focused_article_tab(navigation).unwrap_or(selected_index)
        };
        let sizes = article_tab_sizes(minimums, available, viewport_width);
        reveal_article_tab(
            horizontal,
            sizes,
            target,
            viewport_width,
            previous_reveal.is_some_and(|previous| previous.minimums != minimums),
        );
    }
    row()
        .modifier(
            article_tablist_modifier(Modifier::new(), navigation)
                .width(Dp::from(viewport_width))
                .clip_shape(Shape::CAPSULE)
                .horizontal_scroll(horizontal)
                .semantics(SemanticsArgs {
                    role: Some(Role::TabList),
                    label: Some("Article sections".into()),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            let sizes =
                article_tab_sizes(article_tab_minimums(contents), available, viewport_width);
            for (index, label) in ARTICLE_TAB_TITLES.into_iter().enumerate() {
                if !available[index] {
                    continue;
                }
                let active = selected.get() == label;
                let interaction = remember(InteractionState::default);
                let (hovered, pressed) =
                    interaction.with(|value| (value.is_hovered(), value.is_pressed()));
                let focus_visible = article_tab_focus_visible(navigation, index);
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(article_tab_visual::focus_modifier(
                        article_tab_modifier(
                            Modifier::new()
                                .width(Dp::from(sizes[index]))
                                .height(Dp(if pinned.get() { 63.0 } else { 79.0 }))
                                .background_with_shape(
                                    article_tab_visual::background(false, hovered, pressed),
                                    Shape::CAPSULE,
                                )
                                .background_with_scaled_shape(
                                    article_tab_visual::pill_color(
                                        progress[index],
                                        hovered,
                                        pressed,
                                    ),
                                    Shape::CAPSULE,
                                    [0.32 + 0.68 * progress[index], 1.0],
                                ),
                            navigation,
                            index,
                            Callback::new(move || {
                                selected.set(label);
                                scroll.with_mut(|scroll| {
                                    scroll.set_scroll_position(PxPosition::ZERO)
                                });
                                FocusManager::current().defer_keyboard_until_layout();
                            }),
                            Some(interaction),
                        ),
                        focus_visible,
                        hovered,
                    ))
                    .children(move || {
                        row()
                            .modifier({
                                let modifier = Modifier::new().on_layout_coordinates(
                                    move |coordinates: LayoutCoordinates| {
                                        let next = Dp::from(coordinates.local_size.width);
                                        if contents.with(|contents| contents[index]) != next {
                                            contents.with_mut(|contents| contents[index] = next);
                                        }
                                    },
                                );
                                #[cfg(all(test, not(target_family = "wasm")))]
                                let modifier = modifier.on_globally_positioned(move |bounds| {
                                    visual_tests::record_content(index, bounds)
                                });
                                modifier
                            })
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                icon()
                                    .vector(article_tab_visual::vector(
                                        index, active, hovered, pressed,
                                    ))
                                    .size(Dp(24.0))
                                    .tint(article_tab_visual::foreground(active, hovered, pressed));
                                spacer().modifier(Modifier::new().width(Dp(8.0)));
                                text()
                                    .content(label)
                                    .style(tessera_components::theme::TextStyle {
                                        font_family: Some(article_tab_visual::font_family(
                                            active, hovered, pressed,
                                        )),
                                        font_weight: 500,
                                        ..type_style(16.0, 24.0)
                                    })
                                    .color(article_tab_visual::foreground(
                                        active, hovered, pressed,
                                    ));
                            });
                    });
            }
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "article_tabs_tests.rs"]
mod visual_tests;
