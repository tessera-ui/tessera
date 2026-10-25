//! Finite lazy article sections with retained visited examples.
//!
//! ## Usage
//!
//! Navigate long specifications without constructing every unseen figure.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    lazy_list::{LazyListController, lazy_column},
    modifier::{ClickableArgs, ModifierExt as _},
    row::row,
    scrollable::{ScrollBarBehavior, ScrollableController, scrollbar_v},
    spacer::spacer,
    text::text,
};
use tessera_ui::{
    Callback, Dp, LayoutCoordinates, Modifier, Px, RenderSlotWith, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, remember_with_key, tessera,
};

use super::{
    super::{ACCENT, INK, gap, media, type_style},
    ArticleData, ArticlePage, ArticleRoute, article_footer, article_section, article_tabs,
};

#[derive(Clone, PartialEq)]
struct ChapterBindings {
    controller: State<LazyListController>,
    pages: Option<Vec<ArticlePage>>,
    route: Option<ArticleRoute>,
}

#[tessera]
pub(super) fn viewport(
    data: State<ArticleData>,
    selected: State<&'static str>,
    scroll: State<ScrollableController>,
    width: State<Dp>,
    pages: Option<Vec<ArticlePage>>,
    route: Option<ArticleRoute>,
) {
    let page = data.get();
    let controller = remember_with_key((page.title.clone(), page.page), LazyListController::new);
    let height = remember(|| Px(1));
    let next = ChapterBindings {
        controller,
        pages,
        route,
    };
    let bindings = remember(|| next.clone());
    if bindings.with(|current| current != &next) {
        bindings.set(next);
    }
    let items = RenderSlotWith::new(move |item| {
        let page = data.get();
        let first_section = 3 + usize::from(page.intro.is_some());
        let compact = width.get().0 < 600.0;
        let inset = if compact { Dp(24.0) } else { Dp(40.0) };
        if item == 0 {
            media::page_header(page.title, page.description, width.get());
        } else if item == 1 {
            article_tabs(selected, scroll)
                .pages_optional(bindings.with(|bindings| bindings.pages.clone()));
        } else if item == 2 {
            contents(data, width, bindings);
        } else if item == 3 && page.intro.is_some() {
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .padding_symmetric(inset, Dp::ZERO),
                )
                .children(move || {
                    let page = data.get();
                    if !page.sections.is_empty() {
                        gap(Dp(80.0));
                    }
                    if let Some(intro) = page.intro {
                        intro.render();
                    }
                });
        } else if item < first_section + page.sections.len() {
            article_section(
                item - first_section,
                data,
                if compact {
                    page.compact_content_inset.unwrap_or(Dp(24.0))
                } else {
                    Dp(40.0)
                },
            );
        } else {
            column().children(move || {
                let page = data.get();
                let bindings = bindings.get();
                gap(Dp(48.0));
                article_footer(page.title, selected, scroll)
                    .pages_optional(bindings.pages)
                    .route_optional(bindings.route);
                spacer().modifier(Modifier::new().fill_max_width().height(Dp(80.0)));
            });
        }
    });
    row()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            lazy_column()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .weight(1.0)
                        .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
                            let next = Dp::from(coordinates.local_size.width);
                            if next != width.get() {
                                width.set(next);
                            }
                            let next = coordinates.local_size.height;
                            if next != height.get() {
                                height.set(next);
                            }
                        }),
                )
                .controller(bindings.with(|bindings| bindings.controller))
                .viewport_controller(scroll)
                .keep_composed_items(true)
                .item_spacing(Dp::ZERO)
                .content_padding(Dp::ZERO)
                .estimated_item_size(Dp(600.0))
                .max_viewport_main(height.get())
                .scrollbar_behavior(ScrollBarBehavior::Hidden)
                .items_with_key_shared(
                    1,
                    move |_| {
                        (
                            data.with(|page| page.title.clone()),
                            None::<&'static str>,
                            0,
                        )
                    },
                    items,
                )
                .sticky_header_with_key_shared(
                    (
                        data.with(|page| page.title.clone()),
                        None::<&'static str>,
                        1,
                    ),
                    RenderSlotWith::new(move |_| items.render(1)),
                )
                .items_with_key_shared(
                    {
                        let page = data.get();
                        1 + usize::from(page.intro.is_some()) + page.sections.len() + 1
                    },
                    move |item| {
                        let page = data.get();
                        (page.title, Some(page.page), item + 2)
                    },
                    RenderSlotWith::new(move |item| items.render(item + 2)),
                );
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().width(Dp(15.0)).fill_max_height())
                .children(move || {
                    scrollbar_v()
                        .state(scroll)
                        .thickness(Dp(15.0))
                        .scrollbar_behavior(ScrollBarBehavior::AlwaysVisible);
                });
        });
}

#[tessera]
fn contents(data: State<ArticleData>, width: State<Dp>, bindings: State<ChapterBindings>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let page = data.get();
            gap(Dp(64.0));
            if !page.sections.is_empty() && page.section_heading_level == 2 {
                let inset = if width.get().0 < 600.0 {
                    Dp(24.0)
                } else {
                    Dp(40.0)
                };
                column()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .padding_symmetric(inset, Dp::ZERO),
                    )
                    .children(move || {
                        text()
                            .content("On this page")
                            .style(type_style(14.0, 20.0))
                            .color(INK);
                        gap(Dp(8.0));
                        let page = data.get();
                        for (index, section) in page.sections.iter().enumerate() {
                            text()
                                .content(section.title)
                                .style(type_style(16.0, 24.0))
                                .color(ACCENT)
                                .modifier(
                                    Modifier::new()
                                        .height(Dp(40.0))
                                        .padding_symmetric(Dp::ZERO, Dp(8.0))
                                        .clickable_with(ClickableArgs {
                                            role: Some(Role::Link),
                                            on_click: Callback::new(move || {
                                                if !data.is_alive() || !bindings.is_alive() {
                                                    return;
                                                }
                                                let controller =
                                                    bindings.with(|bindings| bindings.controller);
                                                if !controller.is_alive() {
                                                    return;
                                                }
                                                let first = 3 + usize::from(
                                                    data.with(|data| data.intro.is_some()),
                                                );
                                                controller.with_mut(|controller| {
                                                    controller
                                                        .scroll_to_item(first + index, Dp(88.0));
                                                });
                                            }),
                                            ..Default::default()
                                        }),
                                );
                        }
                    });
            }
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "article_lazy_tests.rs"]
mod tests;
