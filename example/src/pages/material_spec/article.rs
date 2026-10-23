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
    material_icons::filled,
    modifier::{ClickableArgs, ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    scrollable::ScrollableController,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
};
use tessera_ui::{
    Callback, Dp, Modifier, PxPosition, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, tessera,
};

use super::{ACCENT, BACKGROUND, INK, gap, heading, media, srgb, type_style};

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
    let selected = remember(|| "Specs");
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
    let origin = remember(|| 0.0_f32);
    let end = remember(|| 0.0_f32);
    let anchors = remember(|| vec![0.0_f32; current.sections.len()]);
    if data.with(|data| data != &current) {
        if data.with(|data| data.title != current.title || data.page != current.page) {
            scroll.with_mut(|scroll| scroll.set_scroll_position(PxPosition::ZERO));
        }
        if anchors.with(Vec::len) != current.sections.len() {
            anchors.set(vec![0.0; current.sections.len()]);
        }
        data.set(current);
    }
    surface()
        .style(BACKGROUND.into())
        .modifier(Modifier::new().fill_max_size().on_globally_positioned(
            move |bounds: tessera_ui::PxRect| {
                let next = Dp::from_pixels_f32(bounds.width.0 as f32);
                if width.get() != next {
                    width.set(next);
                }
                if origin.get() != bounds.y.0 as f32 {
                    origin.set(bounds.y.0 as f32);
                }
            },
        ))
        .child(move || {
            let pages = pages.clone();
            column()
                .modifier(Modifier::new().fill_max_size().vertical_scroll(scroll))
                .children(move || {
                    let compact = width.get().0 < 600.0;
                    let inset = if compact { Dp(24.0) } else { Dp(40.0) };
                    let page = data.get();
                    media::page_header(page.title.clone(), page.description.clone(), width.get());
                    article_tabs(selected, scroll).pages_optional(pages.clone());
                    gap(Dp(64.0));
                    if !page.sections.is_empty() && page.section_heading_level == 2 {
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
                                let sections = data.with(|data| data.sections.clone());
                                for (index, section) in sections.iter().enumerate() {
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
                                                        let y = anchors
                                                            .with(|anchors| anchors[index])
                                                            - origin.get();
                                                        let maximum = (end.get()
                                                            - origin.get()
                                                            - scroll.with(|scroll| {
                                                                scroll
                                                                    .visible_size()
                                                                    .height
                                                                    .to_f32()
                                                            }))
                                                        .max(0.0);
                                                        let y = y.clamp(0.0, maximum);
                                                        scroll.with_mut(|scroll| {
                                                            scroll.set_scroll_position(
                                                                PxPosition::from_f32_xy(0.0, -y),
                                                            )
                                                        });
                                                    }),
                                                    ..Default::default()
                                                }),
                                        );
                                }
                            });
                    }
                    if let Some(intro) = page.intro {
                        if !page.sections.is_empty() {
                            gap(Dp(80.0));
                        }
                        column()
                            .modifier(
                                Modifier::new()
                                    .fill_max_width()
                                    .padding_symmetric(inset, Dp::ZERO),
                            )
                            .children(move || intro.render());
                    }
                    for index in 0..page.sections.len() {
                        article_section(
                            index,
                            data,
                            scroll,
                            anchors,
                            end,
                            if compact {
                                page.compact_content_inset.unwrap_or(Dp(24.0))
                            } else {
                                Dp(40.0)
                            },
                        );
                    }
                });
        });
}

#[tessera]
fn article_section(
    index: usize,
    data: State<ArticleData>,
    scroll: State<ScrollableController>,
    anchors: State<Vec<f32>>,
    end: State<f32>,
    inset: Dp,
) {
    let section = data.with(|data| data.sections[index].clone());
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                    let y = bounds.y.0 as f32 - scroll.with(|scroll| scroll.child_position().y);
                    if (anchors.with(|anchors| anchors[index]) - y).abs() > 0.5 {
                        anchors.with_mut(|anchors| anchors[index] = y);
                    }
                    if index + 1 == anchors.with(Vec::len) {
                        let next = y + bounds.height.to_f32();
                        if (end.get() - next).abs() > 0.5 {
                            end.set(next);
                        }
                    }
                })
                .padding(Padding::new(inset, Dp(80.0), inset, Dp(32.0))),
        )
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

#[tessera]
fn article_tabs(
    selected: State<&'static str>,
    scroll: State<ScrollableController>,
    pages: Option<Vec<ArticlePage>>,
) {
    let horizontal = remember(ScrollableController::new);
    let width = remember(|| Dp(900.0));
    let contents = remember(|| [Dp(160.0); 4]);
    let available = ["Overview", "Specs", "Guidelines", "Accessibility"].map(|label| {
        label == "Specs"
            || pages
                .as_ref()
                .is_some_and(|pages| pages.iter().any(|page| page.title == label))
    });
    let minimum = contents.with(|contents| {
        contents
            .iter()
            .enumerate()
            .filter(|(index, _)| available[*index])
            .map(|(_, width)| width.0 + 48.0)
            .sum::<f64>()
    });
    let extra = ((width.get().0 - 16.0 - minimum)
        / available.iter().filter(|available| **available).count() as f64)
        .max(0.0);
    row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                    let next = Dp::from_pixels_f32(bounds.width.to_f32());
                    if width.get() != next {
                        width.set(next);
                    }
                })
                .padding_all(Dp(8.0))
                .background_with_shape(srgb(248, 241, 246), Shape::rounded_rectangle(Dp(40.0)))
                .horizontal_scroll(horizontal)
                .semantics(SemanticsArgs {
                    role: Some(Role::TabList),
                    label: Some("Article sections".into()),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            for (index, (label, vector)) in [
                ("Overview", filled::INFO_SVG),
                ("Specs", filled::STYLE_SVG),
                ("Guidelines", filled::DESIGN_SERVICES_SVG),
                ("Accessibility", filled::ACCESSIBILITY_NEW_SVG),
            ]
            .into_iter()
            .enumerate()
            {
                if label != "Specs"
                    && !pages
                        .as_ref()
                        .is_some_and(|pages| pages.iter().any(|page| page.title == label))
                {
                    continue;
                }
                let active = selected.get() == label;
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .width(Dp(contents.with(|contents| contents[index].0)
                                + 48.0
                                + extra))
                            .height(Dp(80.0))
                            .background_with_shape(
                                if active {
                                    srgb(220, 218, 245)
                                } else {
                                    srgb(248, 241, 246)
                                },
                                Shape::rounded_rectangle(Dp(40.0)),
                            )
                            .semantics(SemanticsArgs {
                                selected: Some(active),
                                merge_descendants: true,
                                ..Default::default()
                            })
                            .clickable_with(ClickableArgs {
                                role: Some(Role::Tab),
                                label: Some(label.into()),
                                on_click: Callback::new(move || {
                                    selected.set(label);
                                    scroll.with_mut(|scroll| {
                                        scroll.set_scroll_position(PxPosition::ZERO)
                                    });
                                }),
                                ..Default::default()
                            })
                            .padding_symmetric(Dp(24.0), Dp::ZERO),
                    )
                    .children(move || {
                        row()
                            .modifier(Modifier::new().on_globally_positioned(
                                move |bounds: tessera_ui::PxRect| {
                                    let next = Dp::from_pixels_f32(bounds.width.to_f32());
                                    if contents.with(|contents| contents[index]) != next {
                                        contents.with_mut(|contents| contents[index] = next);
                                    }
                                },
                            ))
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                icon().vector(vector).size(Dp(24.0)).tint(srgb(77, 66, 86));
                                spacer().modifier(Modifier::new().width(Dp(8.0)));
                                text()
                                    .content(label)
                                    .style(tessera_components::theme::TextStyle {
                                        font_family: Some("Example Sans Display"),
                                        font_weight: 500,
                                        ..type_style(16.0, 24.0)
                                    })
                                    .color(if active {
                                        srgb(33, 24, 43)
                                    } else {
                                        srgb(77, 66, 86)
                                    });
                            });
                    });
            }
        });
}
