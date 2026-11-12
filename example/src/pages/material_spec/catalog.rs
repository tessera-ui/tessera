use tessera_components::{
    boxed::boxed,
    column::column,
    flow_row::flow_row,
    lazy_list::{LazyListController, lazy_column},
    modifier::{ModifierExt as _, SemanticsArgs},
    scrollable::{ScrollBarBehavior, ScrollableController},
    shape_def::Shape,
    surface::surface,
    text::text,
};
use tessera_shard::RouterController;
use tessera_ui::{
    Callback, CallbackWith, Dp, Modifier, Px, State, modifier::FocusModifierExt as _, tessera,
};

use crate::{
    pages::material_components::{MaterialArticlePageDestination, current_article},
    res::I18N,
};

use super::{
    BACKGROUND, INK, PANEL,
    article::{ArticleDestination, ArticleRoute},
    gap, heading, slider_spec, type_style,
};

#[derive(Clone, Copy, PartialEq)]
struct Component {
    title: &'static str,
    description: &'static str,
    category: usize,
    demo: usize,
}

const COMPONENTS: &[Component] = &[
    Component {
        title: "ui-button-groups",
        description: "ui-button-groups-organize-buttons-and-add-interactions-between-them",
        category: 0,
        demo: 18,
    },
    Component {
        title: "ui-buttons",
        description: "ui-buttons-prompt-most-actions-in-a-ui",
        category: 0,
        demo: 36,
    },
    Component {
        title: "ui-extended-fabs",
        description: "ui-extended-floating-action-buttons-extended-fabs-help-people-take-primary-actions",
        category: 0,
        demo: 9,
    },
    Component {
        title: "ui-fab-menu",
        description: "ui-the-floating-action-button-fab-menu-opens-from-a-fab-to-display-multiple-related-actions",
        category: 0,
        demo: 8,
    },
    Component {
        title: "ui-floating-action-buttons-fabs",
        description: "ui-floating-action-buttons-fabs-help-people-take-primary-actions",
        category: 0,
        demo: 19,
    },
    Component {
        title: "ui-icon-buttons",
        description: "ui-icon-buttons-help-people-take-actions-with-a-single-tap",
        category: 0,
        demo: 3,
    },
    Component {
        title: "ui-segmented-buttons",
        description: "ui-segmented-buttons-help-people-select-options-switch-views-or-sort-elements",
        category: 0,
        demo: 20,
    },
    Component {
        title: "ui-split-buttons",
        description: "ui-split-buttons-open-a-menu-to-give-people-more-options-related-to-an-action",
        category: 0,
        demo: 22,
    },
    Component {
        title: "ui-date-pickers",
        description: "ui-date-pickers-let-people-select-a-date-or-a-range-of-dates",
        category: 1,
        demo: 28,
    },
    Component {
        title: "ui-time-pickers",
        description: "ui-time-pickers-help-people-select-and-set-a-specific-time",
        category: 1,
        demo: 30,
    },
    Component {
        title: "ui-loading-indicator",
        description: "ui-loading-indicators-show-the-progress-of-a-process-for-a-short-wait-time",
        category: 2,
        demo: 6,
    },
    Component {
        title: "ui-progress-indicators",
        description: "ui-progress-indicators-show-the-status-of-a-process-in-real-time",
        category: 2,
        demo: 7,
    },
    Component {
        title: "ui-navigation-bar",
        description: "ui-navigation-bars-let-people-switch-between-ui-views-on-smaller-devices",
        category: 3,
        demo: 31,
    },
    Component {
        title: "ui-navigation-drawer",
        description: "ui-navigation-drawers-let-people-switch-between-ui-views-on-larger-devices",
        category: 3,
        demo: 11,
    },
    Component {
        title: "ui-navigation-rail",
        description: "ui-navigation-rails-let-people-switch-between-ui-views-on-mid-sized-devices",
        category: 3,
        demo: 32,
    },
    Component {
        title: "ui-bottom-sheets",
        description: "ui-bottom-sheets-show-secondary-content-anchored-to-the-bottom-of-the-screen",
        category: 4,
        demo: 33,
    },
    Component {
        title: "ui-side-sheets",
        description: "ui-side-sheets-show-secondary-content-anchored-to-the-side-of-the-screen",
        category: 4,
        demo: 34,
    },
    Component {
        title: "ui-app-bars",
        description: "ui-app-bars-are-placed-at-the-top-of-the-screen-to-help-people-navigate-through-a-product",
        category: 5,
        demo: 1,
    },
    Component {
        title: "ui-badges",
        description: "ui-badges-show-notifications-counts-or-status-information-on-navigation-items-and-icons-1606",
        category: 5,
        demo: 12,
    },
    Component {
        title: "ui-cards",
        description: "ui-cards-display-content-and-actions-about-a-single-subject",
        category: 5,
        demo: 16,
    },
    Component {
        title: "ui-carousel",
        description: "ui-carousels-show-a-collection-of-items-that-can-be-scrolled-on-and-off-the-screen",
        category: 5,
        demo: 2,
    },
    Component {
        title: "ui-checkbox",
        description: "ui-checkboxes-let-users-select-one-or-more-items-from-a-list-or-turn-an-item-on-or-off",
        category: 5,
        demo: 37,
    },
    Component {
        title: "ui-chips",
        description: "ui-chips-help-people-enter-information-make-selections-filter-content-or-trigger-actions",
        category: 5,
        demo: 17,
    },
    Component {
        title: "ui-dialogs",
        description: "ui-dialogs-provide-important-prompts-in-a-user-flow",
        category: 5,
        demo: 13,
    },
    Component {
        title: "ui-divider",
        description: "ui-dividers-are-thin-lines-that-group-content-in-lists-or-other-containers",
        category: 5,
        demo: 14,
    },
    Component {
        title: "ui-lists",
        description: "ui-lists-are-continuous-vertical-indexes-of-text-and-images",
        category: 5,
        demo: 15,
    },
    Component {
        title: "ui-menus",
        description: "ui-menus-display-a-list-of-choices-on-a-temporary-surface",
        category: 5,
        demo: 26,
    },
    Component {
        title: "ui-radio-button",
        description: "ui-radio-buttons-let-people-select-one-option-from-a-set-of-options",
        category: 5,
        demo: 23,
    },
    Component {
        title: "ui-search",
        description: "ui-search-lets-people-enter-a-keyword-or-phrase-to-get-relevant-information",
        category: 5,
        demo: 27,
    },
    Component {
        title: "ui-sliders",
        description: "ui-sliders-allow-users-to-make-selections-from-a-range-of-values",
        category: 5,
        demo: 5,
    },
    Component {
        title: "ui-snackbar",
        description: "ui-snackbars-show-short-updates-about-app-processes-at-the-bottom-of-the-screen",
        category: 5,
        demo: 25,
    },
    Component {
        title: "ui-switch",
        description: "ui-switches-toggle-the-selection-of-an-item-on-and-off",
        category: 5,
        demo: 38,
    },
    Component {
        title: "ui-tabs",
        description: "ui-tabs-organize-content-across-different-screens-and-views",
        category: 5,
        demo: 21,
    },
    Component {
        title: "ui-text-fields",
        description: "ui-text-fields-let-users-enter-text-into-a-ui",
        category: 5,
        demo: 24,
    },
    Component {
        title: "ui-toolbars",
        description: "ui-toolbars-display-frequently-used-actions-relevant-to-the-current-page",
        category: 5,
        demo: 10,
    },
    Component {
        title: "ui-tooltips",
        description: "ui-tooltips-display-brief-labels-or-messages",
        category: 5,
        demo: 4,
    },
];

#[tessera]
pub(crate) fn component_article(
    index: usize,
    router: State<RouterController>,
    sessions: Option<crate::app::AppLiveSessions>,
) {
    let open_article = CallbackWith::new(move |index| {
        router
            .with_mut(|router| router.replace(MaterialArticlePageDestination { index, sessions }));
    });
    let previous_article = Callback::new(move || {
        if let Some(index) = index.checked_sub(1) {
            open_article.call(index);
        }
    });
    let next_article = Callback::new(move || {
        if index + 1 < COMPONENTS.len() {
            open_article.call(index + 1);
        }
    });
    let component = COMPONENTS[index];
    let route = ArticleRoute {
        // Component pages open on Overview, matching the landing article of
        // the official Material Design 3 component pages.
        initial_page: "ui-overview",
        previous: index.checked_sub(1).map(|index| ArticleDestination {
            title: COMPONENTS[index].title,
            on_open: previous_article,
        }),
        next: COMPONENTS
            .get(index + 1)
            .map(|component| ArticleDestination {
                title: component.title,
                on_open: next_article,
            }),
    };
    let content = tessera_ui::RenderSlot::new(move || {
        let route = route.clone();
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                let route = route.clone();
                boxed()
                    .modifier(Modifier::new().fill_max_width().weight(1.0).semantics(
                        SemanticsArgs {
                            test_tag: Some(format!("component-article-{}", component.demo)),
                            role: Some(tessera_ui::accesskit::Role::Group),
                            label: Some(I18N.format(
                                "ui-value-specifications",
                                [("v0", I18N.text(&component.title).into())],
                            )),
                            merge_descendants: true,
                            ..Default::default()
                        },
                    ))
                    .children(move || {
                        if component.demo == 13 {
                            super::dialogs::dialogs_spec().route(route.clone());
                        } else if component.demo == 25 {
                            super::snackbars::snackbars_spec().route(route.clone());
                        } else if component.demo == 1 {
                            super::app_bars::app_bars_spec().route(route.clone());
                        } else if component.demo == 20 {
                            super::segmented::segmented_spec(Callback::new(move || {
                                open_article.call(0);
                            }))
                            .route(route.clone());
                        } else if component.demo == 24 {
                            super::text_fields::text_fields_spec().route(route.clone());
                        } else if component.demo == 4 {
                            super::tooltips::tooltips_spec().route(route.clone());
                        } else if component.demo == 31 {
                            super::navigation_bar::navigation_bar_spec().route(route.clone());
                        } else if component.demo == 32 {
                            super::navigation_rail::navigation_rail_spec()
                                .session_optional(sessions.map(|sessions| sessions.rail))
                                .route(route.clone());
                        } else if component.demo == 21 {
                            super::tabs::tabs_spec().route(route.clone());
                        } else if component.demo == 6 {
                            super::loading_indicator::loading_indicator_spec().route(route.clone());
                        } else if component.demo == 7 {
                            super::progress_indicators::progress_indicators_spec()
                                .route(route.clone());
                        } else if component.demo == 12 {
                            super::badges::badges_spec().route(route.clone());
                        } else if component.demo == 18 {
                            super::button_groups::button_groups_spec().route(route.clone());
                        } else if component.demo == 22 {
                            super::split_buttons::split_buttons_spec().route(route.clone());
                        } else if component.demo == 8 {
                            super::fab_menus::fab_menu_spec().route(route.clone());
                        } else if component.demo == 9 {
                            super::extended_fabs::extended_fabs_spec().route(route.clone());
                        } else if component.demo == 19 {
                            super::floating_actions::floating_actions_spec().route(route.clone());
                        } else if component.demo == 28 {
                            super::date_pickers::date_pickers_spec().route(route.clone());
                        } else if component.demo == 30 {
                            super::time_pickers::time_pickers_spec().route(route.clone());
                        } else if component.demo == 16 {
                            super::cards::cards_spec().route(route.clone());
                        } else if component.demo == 15 {
                            super::lists::lists_spec().route(route.clone());
                        } else if component.demo == 14 {
                            super::dividers::divider_spec().route(route.clone());
                        } else if component.demo == 3 {
                            super::icon_buttons::icon_buttons_spec().route(route.clone());
                        } else if component.demo == 17 {
                            super::chips::chips_spec().route(route.clone());
                        } else if component.demo == 23 {
                            super::radio_buttons::radio_buttons_spec().route(route.clone());
                        } else if component.demo == 5 {
                            slider_spec().route(route.clone());
                        } else if component.demo == 36 {
                            super::buttons::buttons_spec().route(route.clone());
                        } else if component.demo == 37 {
                            super::selection_controls::checkbox_spec().route(route.clone());
                        } else if component.demo == 38 {
                            super::selection_controls::switch_spec().route(route.clone());
                        } else if component.demo == 26 {
                            super::menus::menus_spec().route(route.clone());
                        } else if component.demo == 27 {
                            super::search::search_spec().route(route.clone());
                        } else if component.demo == 2 {
                            super::carousel::carousel_spec().route(route.clone());
                        } else if component.demo == 10 {
                            super::toolbars::toolbars_spec().route(route.clone());
                        } else if component.demo == 11 {
                            super::drawers::drawers_spec()
                                .session_optional(sessions.map(|sessions| sessions.drawer))
                                .route(route.clone())
                                .on_recommended(move || {
                                    if let Some(index) = component_index(32) {
                                        open_article.call(index);
                                    }
                                });
                        } else if component.demo == 33 {
                            super::sheets::sheets_spec(false).route(route.clone());
                        } else if component.demo == 34 {
                            super::side_sheets::side_sheets_spec()
                                .session_optional(sessions.map(|sessions| sessions.side_sheet))
                                .route(route.clone());
                        } else {
                            unreachable!("ui-unregistered-component-article");
                        }
                    });
            });
    });
    content.render();
}

#[tessera]
pub(crate) fn component_directory(
    directory_scroll: State<ScrollableController>,
    directory_list: State<LazyListController>,
    width: State<Dp>,
    height: State<Px>,
    on_open: CallbackWith<usize>,
) {
    surface()
        .style(BACKGROUND.into())
        .modifier(Modifier::new().fill_max_size())
        .child(move || {
            super::scroll_viewport::spec_scroll_viewport(directory_scroll)
                .scroll_content(false)
                .viewport_modifier(Modifier::new().on_layout_coordinates(
                    move |coordinates: tessera_ui::LayoutCoordinates| {
                        let next = Dp::from(coordinates.local_size.width);
                        if width.get() != next {
                            width.set(next);
                        }
                        let next = coordinates.local_size.height;
                        if height.get() != next {
                            height.set(next);
                        }
                    },
                ))
                .children(move || {
                    let columns = directory_columns(width.get());
                    let list = lazy_column()
                        .modifier(Modifier::new().fill_max_size().weight(1.0))
                        .controller(directory_list)
                        .viewport_controller(directory_scroll)
                        .scrollbar_behavior(ScrollBarBehavior::Hidden)
                        .estimated_item_size(Dp(320.0))
                        .max_viewport_main(height.get())
                        .item(move || {
                            super::media::page_header("ui-components", "", width.get());
                        });
                    for (category, title) in [
                        "ui-buttons",
                        "ui-date-time-pickers",
                        "ui-loading-progress",
                        "ui-navigation",
                        "ui-sheets",
                        "ui-all-other-components",
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        list.item(move || {
                            column().children(move || {
                                gap(Dp(if category == 0 { 72.0 } else { 80.0 }));
                                boxed()
                                    .modifier(Modifier::new().padding_symmetric(Dp(33.0), Dp::ZERO))
                                    .children(move || {
                                        heading(title.to_owned()).compact(width.get().0 <= 600.0);
                                    });
                                gap(Dp(24.0));
                            });
                        });
                        let entries = COMPONENTS
                            .iter()
                            .copied()
                            .enumerate()
                            .filter(|(_, component)| component.category == category)
                            .collect::<Vec<_>>();
                        let rows = entries.len().div_ceil(columns);
                        for (row, entries) in entries.chunks(columns).enumerate() {
                            let entries = entries.to_vec();
                            list.item(move || {
                                let entries = entries.clone();
                                column().children(move || {
                                    directory_group(entries.clone(), on_open, width.get());
                                    if row + 1 < rows {
                                        gap(Dp(8.0));
                                    }
                                });
                            });
                        }
                    }
                    list.item(|| gap(Dp(64.0)));
                });
        });
}

pub(super) fn current_demo(router: &RouterController) -> Option<usize> {
    current_article(router).map(|index| COMPONENTS[index].demo)
}

pub(super) fn component_index(demo: usize) -> Option<usize> {
    COMPONENTS
        .iter()
        .position(|component| component.demo == demo)
}

#[tessera]
fn directory_group(entries: Vec<(usize, Component)>, on_open: CallbackWith<usize>, width: Dp) {
    let columns = directory_columns(width) as f64;
    let card_width = Dp(((width.0 - 16.0 - (columns - 1.0) * 8.0) / columns).max(0.0));
    flow_row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(8.0), Dp::ZERO),
        )
        .cross_axis_alignment(tessera_components::alignment::CrossAxisAlignment::Stretch)
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for (index, component) in entries.iter().copied() {
                component_card(component, index, on_open, card_width);
            }
        });
}

fn directory_columns(width: Dp) -> usize {
    if width.0 <= 600.0 {
        1
    } else if width.0 < 1295.0 {
        2
    } else {
        3
    }
}

#[tessera]
fn component_card(component: Component, index: usize, on_open: CallbackWith<usize>, width: Dp) {
    surface()
        .style(PANEL.into())
        .shape(Shape::rounded_rectangle(Dp(24.0)))
        .modifier(Modifier::new().width(width).semantics(SemanticsArgs {
            test_tag: Some(format!("component-card-{index}")),
            ..Default::default()
        }))
        .on_click(move || on_open.call(index))
        .accessibility_label(I18N.format(
            "ui-open-value-specifications",
            [("v0", I18N.text(&component.title).into())],
        ))
        .child(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    super::media::picture(component.title, width, Dp(200.0)).thumbnail(true);

                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                        .children(move || {
                            text(I18N.text(component.title))
                                .style(type_style(24.0, 32.0))
                                .color(INK);
                            gap(Dp(8.0));
                            text(I18N.text(component.description))
                                .style(type_style(16.0, 24.0))
                                .color(INK);
                        });
                });
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use std::time::Duration;

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        accesskit::{Action, ActionRequest, Node, Rect, Role, Toggled, TreeId},
        headless::{HeadlessConfig, HeadlessRenderer},
        remember,
    };

    use super::*;

    #[tessera]
    fn fixture(initial_component: Option<usize>) {
        material_theme().child(move || {
            let router = remember(move || {
                let mut router = RouterController::with_root(
                    crate::pages::material_components::MaterialComponentsPageDestination {
                        sessions: None,
                    },
                );
                if let Some(index) = initial_component {
                    router.push(MaterialArticlePageDestination {
                        index,
                        sessions: None,
                    });
                }
                router
            });
            let drawer =
                remember(|| tessera_components::side_sheet::SideSheetController::new(false));
            column()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    tessera_components::app_bar::top_app_bar()
                        .title("Material Design 3")
                        .navigation_icon(move || {
                            crate::app::navigation_button(router, drawer);
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_width().weight(1.0))
                        .children(move || {
                            tessera_shard::shard_home().controller(router);
                        });
                });
        });
    }

    fn bounds(renderer: &HeadlessRenderer, label: &str) -> Rect {
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| crate::has_label(&node, label))
            .unwrap_or_else(|| panic!("missing {label}"))
            .1
            .bounds()
            .unwrap()
    }

    fn click(renderer: &mut HeadlessRenderer, mut bounds: Rect) {
        let viewport = renderer.physical_size();
        let snapshot = renderer.accessibility_snapshot().unwrap();
        let article_tab = snapshot.nodes.iter().any(|(_, node)| {
            node.bounds() == Some(bounds)
                && node.role() == Role::Tab
                && node.label().map(crate::visible_text).is_some_and(|label| {
                    ["Overview", "Specs", "Guidelines", "Accessibility"].contains(&label.as_str())
                })
        });
        let content_top = if bounds.y1 <= 64.0 {
            0.0
        } else if article_tab {
            64.0
        } else {
            snapshot
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::TabList && crate::has_label(&node, "Article sections")
                })
                .and_then(|(_, node)| node.bounds())
                .filter(|bounds| bounds.y0 <= 73.0)
                .map_or(64.0, |bounds| bounds.y1)
        };
        if bounds.y1 > f64::from(viewport.height) || bounds.y0 < content_top {
            let target = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| node.bounds() == Some(bounds))
                .map(|(id, _)| id)
                .expect("scroll target");
            let center = (bounds.y0 + bounds.y1) * 0.5;
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(viewport.width as f32 - 4.0, viewport.height as f32 * 0.5),
                0.0,
                (f64::from(viewport.height) * 0.5 - center) as f32,
            );
            renderer.render_frames(40);
            bounds = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .find(|(id, _)| *id == target)
                .expect("retained scroll target")
                .1
                .bounds()
                .unwrap();
        }
        let position = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            if bounds.y1 - bounds.y0 < 64.0 {
                ((bounds.y0 + bounds.y1) * 0.5) as f32
            } else {
                (bounds.y1 - 32.0) as f32
            },
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(2);
    }

    fn control(renderer: &HeadlessRenderer, label: &str) -> Node {
        let nodes = renderer.accessibility_snapshot().unwrap().nodes;
        nodes
            .iter()
            .find(|(_, node)| {
                crate::has_label(&node, label)
                    && matches!(
                        node.role(),
                        Role::Button
                            | Role::CheckBox
                            | Role::Switch
                            | Role::RadioButton
                            | Role::ListItem
                    )
            })
            .unwrap_or_else(|| {
                panic!(
                    "missing control {label}; published controls: {:?}",
                    nodes
                        .iter()
                        .filter_map(|(_, node)| node.label().map(|label| (node.role(), label)))
                        .collect::<Vec<_>>()
                )
            })
            .1
            .clone()
    }

    fn scoped_node(
        renderer: &HeadlessRenderer,
        group: &str,
        label: &str,
        role: Role,
    ) -> Option<Node> {
        let tree = renderer
            .inspect_accessibility()
            .expect("published accessibility");
        let root = tree
            .reading_order()
            .find(|(_, node)| {
                matches!(node.role(), Role::Group | Role::Navigation)
                    && crate::has_label(&node, group)
            })
            .unwrap_or_else(|| panic!("missing group {group}"));
        tree.subtree(root.0)
            .skip(1)
            .find(|(id, node)| {
                node.role() == role
                    && (crate::has_label(&node, label)
                        || node.label().is_none()
                            && tree
                                .subtree(*id)
                                .any(|(_, child)| crate::has_label(&child, label)))
            })
            .map(|(_, node)| node.clone())
    }

    fn toggle(renderer: &mut HeadlessRenderer, label: &str, before: Toggled, after: Toggled) {
        let node = control(renderer, label);
        assert_eq!(node.toggled(), Some(before), "initial {label}");
        click(renderer, node.bounds().unwrap());
        assert_eq!(
            control(renderer, label).toggled(),
            Some(after),
            "updated {label}"
        );
    }

    fn tab(renderer: &HeadlessRenderer, label: &str) -> Node {
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.role() == Role::Tab && crate::has_label(&node, label))
            .unwrap_or_else(|| panic!("missing article tab {label}"))
            .1
    }

    fn enter_specs(renderer: &mut HeadlessRenderer) {
        compose_navigation_target(renderer, "Overview", Role::Tab);
        assert_eq!(tab(renderer, "Overview").is_selected(), Some(true));
        let specs = tab(renderer, "Specs").bounds().unwrap();
        click(renderer, specs);
        renderer.render_frames(24);
        assert_eq!(tab(renderer, "Specs").is_selected(), Some(true));
    }

    fn select_article_page(renderer: &mut HeadlessRenderer, page: &str) {
        compose_navigation_target(renderer, page, Role::Tab);
        let target = tab(renderer, page).bounds().unwrap();
        click(renderer, target);
        renderer.render_frames(24);
        assert_eq!(tab(renderer, page).is_selected(), Some(true));
    }

    fn compose_navigation_target(renderer: &mut HeadlessRenderer, label: &str, role: Role) {
        let toward_header = matches!(role, Role::Tab | Role::Heading)
            || (role == Role::Link
                && !label.starts_with("Previous ")
                && !label.starts_with("Up next "));
        for _ in 0..128 {
            if renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == role && crate::has_label(&node, label))
            {
                return;
            }
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(
                    renderer.physical_size().width as f32 - 30.0,
                    renderer.physical_size().height as f32 * 0.5,
                ),
                0.0,
                if toward_header { 1200.0 } else { -1200.0 },
            );
            renderer.render_frames(40);
        }
        let visible = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .filter(|(_, node)| matches!(node.role(), Role::Tab | Role::Heading | Role::Group))
            .map(|(id, node)| {
                (
                    id,
                    node.role(),
                    node.label().map(str::to_owned),
                    node.bounds(),
                )
            })
            .collect::<Vec<_>>();
        panic!("scrolling did not compose actual navigation target {label} {role:?}: {visible:?}");
    }

    fn activate_navigation(renderer: &mut HeadlessRenderer, label: &str, role: Role) {
        compose_navigation_target(renderer, label, role);
        for action in [Action::Focus, Action::Click] {
            let id = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| node.role() == role && crate::has_label(&node, label))
                .unwrap_or_else(|| panic!("missing navigation {label}"))
                .0;
            assert!(renderer.dispatch_accessibility_action(ActionRequest {
                action,
                target_tree: TreeId::ROOT,
                target_node: id,
                data: None,
            }));
            renderer.render_frames(4);
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn article_measurement_clear_keeps_scrolled_geometry_through_pointer_press_and_release() {
        for same_frame in [false, true] {
            let initial_component = COMPONENTS
                .iter()
                .position(|component| component.title == "ui-text-fields");
            let mut renderer = HeadlessRenderer::new(
                Box::new(move || {
                    fixture().initial_component_optional(initial_component);
                }),
                vec![Box::new(crate::TestPipelines)],
                HeadlessConfig {
                    width: 900,
                    height: 900,
                    sample_count: 1,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(5);
            let specs = tab(&renderer, "Specs").bounds().unwrap();
            click(&mut renderer, specs);
            let focused_label = |renderer: &HeadlessRenderer| {
                let tree = renderer.accessibility_snapshot().unwrap();
                tree.nodes
                    .iter()
                    .find(|(id, _)| *id == tree.focus)
                    .and_then(|(_, node)| node.label())
                    .map(str::to_owned)
            };
            assert!(crate::text_matches(
                focused_label(&renderer).as_deref(),
                "Specs"
            ));
            let label = "Outlined measurement with icons populated";
            let clear = format!("Clear {label}");
            let container = format!("{label} container");
            let mut observations = Vec::new();
            for _ in 0..128 {
                let current = renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .into_iter()
                    .find(|(_, node)| {
                        node.role() == Role::Button && crate::has_label(&node, clear.as_str())
                    })
                    .and_then(|(_, node)| node.bounds());
                let Some(current) = current else {
                    renderer.pointer_scroll(
                        MOUSE_POINTER_ID,
                        PxPosition::from_f32_xy(870.0, 450.0),
                        0.0,
                        -1200.0,
                    );
                    renderer.render_frames(40);
                    continue;
                };
                observations.push(current);
                if current.y0 >= 64.0 && current.y1 <= 900.0 {
                    break;
                }
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    PxPosition::from_f32_xy(895.0, 450.0),
                    0.0,
                    (450.0 - (current.y0 + current.y1) * 0.5) as f32,
                );
                renderer.render_frames(40);
            }
            let margin = PxPosition::from_f32_xy(875.0, 850.0);
            renderer.pointer_move(MOUSE_POINTER_ID, margin);
            renderer.pointer_press(MOUSE_POINTER_ID, margin, PressKeyEventType::Left);
            renderer.render_frames(1);
            renderer.pointer_release(MOUSE_POINTER_ID, margin, PressKeyEventType::Left);
            renderer.render_frames(5);
            let target = control(&renderer, &clear).bounds().unwrap();
            assert!(
                target.y0 >= 64.0 && target.y1 <= 900.0,
                "Clear must be visible before pressing: {target:?}; wheel observations: {observations:?}"
            );
            assert!(
                crate::text_matches(focused_label(&renderer).as_deref(), "Specs"),
                "scrolling into view must retain the original Specs focus before Clear is pressed"
            );
            let before = bounds(&renderer, &container);
            let position = PxPosition::from_f32_xy(
                ((target.x0 + target.x1) * 0.5) as f32,
                ((target.y0 + target.y1) * 0.5) as f32,
            );
            renderer.pointer_move(MOUSE_POINTER_ID, position);
            renderer.render_frames(1);
            renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            if !same_frame {
                renderer.render_frames(1);
                assert_eq!(
                    bounds(&renderer, &container),
                    before,
                    "pointer press must not reveal the earlier Specs focus; current focus={:?}",
                    focused_label(&renderer)
                );
                let tree = renderer.accessibility_snapshot().unwrap();
                let input = tree
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.role() == Role::TextInput && crate::has_label(&node, label)
                    })
                    .unwrap();
                assert_eq!(
                    input.1.value(),
                    Some("Input text"),
                    "Clear has not activated before release"
                );
            }
            renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.render_frames(5);
            assert_eq!(
                bounds(&renderer, &container),
                before,
                "release must preserve the scrolled figure position"
            );
            let tree = renderer.accessibility_snapshot().unwrap();
            let input = tree
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::TextInput && crate::has_label(&node, label))
                .unwrap();
            assert_eq!(input.1.value().unwrap_or_default(), "");
            assert_eq!(tree.focus, input.0, "Clear returns focus to its own editor");
            assert!(
                !tree
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Button
                        && crate::has_label(&node, clear.as_str()))
            );
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn article_footer_routes_between_pages_and_components_and_resets_directory_entry() {
        let initial_component = COMPONENTS
            .iter()
            .position(|component| component.title == "ui-text-fields");
        let mut renderer = HeadlessRenderer::new(
            Box::new(move || {
                fixture().initial_component_optional(initial_component);
            }),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 900,
                height: 1500,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(5);
        enter_specs(&mut renderer);
        compose_navigation_target(&mut renderer, "Previous and up next articles", Role::Group);
        let footer_tree = renderer.accessibility_snapshot().unwrap();
        let footer = footer_tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::Group
                    && crate::has_label(&node, "Previous and up next articles")
            })
            .expect("native article footer group");
        let mut pending = vec![footer.0];
        let mut observed = Vec::new();
        while let Some(id) = pending.pop() {
            let (_, node) = footer_tree
                .nodes
                .iter()
                .find(|(node_id, _)| *node_id == id)
                .unwrap();
            observed.push((
                id,
                node.role(),
                node.label(),
                node.bounds(),
                node.children(),
            ));
            pending.extend_from_slice(node.children());
        }
        let links = footer
            .1
            .children()
            .iter()
            .filter_map(|id| {
                footer_tree
                    .nodes
                    .iter()
                    .find(|(node_id, _)| node_id == id)
                    .map(|(_, node)| node)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            links.len(),
            2,
            "footer must expose both native navigation targets: {observed:#?}"
        );
        for label in [
            "Previous Text fields: Overview",
            "Up next Text fields: Guidelines",
        ] {
            assert!(
                links.iter().any(|node| node.role() == Role::Link
                    && crate::has_label(&node, label)
                    && node.supports_action(Action::Click)),
                "footer is missing actionable link {label}: {observed:#?}"
            );
        }
        let previous = bounds(&renderer, "Previous Text fields: Overview");
        let next = bounds(&renderer, "Up next Text fields: Guidelines");
        assert_eq!(previous.y0, next.y0);
        assert_eq!(bounds(&renderer, "Vertical scrollbar").width(), 15.0);
        assert_eq!(previous.width(), 430.0);
        assert_eq!(previous.height(), 112.0);
        assert_eq!(next.x0 - previous.x1, 8.0);
        activate_navigation(&mut renderer, "Previous Text fields: Overview", Role::Link);
        assert_eq!(tab(&renderer, "Overview").is_selected(), Some(true));
        activate_navigation(&mut renderer, "Previous Tabs: Overview", Role::Link);
        assert_eq!(tab(&renderer, "Overview").is_selected(), Some(true));
        bounds(&renderer, "Tabs specifications");
        for (label, selected) in [
            ("Up next Tabs: Specs", "Specs"),
            ("Up next Tabs: Guidelines", "Guidelines"),
            ("Up next Tabs: Accessibility", "Accessibility"),
            ("Up next Text fields: Overview", "Overview"),
            ("Up next Text fields: Specs", "Specs"),
        ] {
            activate_navigation(&mut renderer, label, Role::Link);
            assert_eq!(tab(&renderer, selected).is_selected(), Some(true));
            let title = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| node.role() == Role::Heading && node.level() == Some(1))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert!(
                title.y0 >= 64.0 && title.y0 < 300.0,
                "page navigation returns to its header"
            );
        }
        activate_navigation(&mut renderer, "Back to components", Role::Button);
        activate_navigation(
            &mut renderer,
            "Open Text fields specifications",
            Role::Button,
        );
        assert_eq!(
            tab(&renderer, "Overview").is_selected(),
            Some(true),
            "a fresh directory entry opens on the Overview article"
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn app_bar_crossfades_to_back_and_restores_directory_navigation() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 900,
                height: 1800,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(3);
        let menu = bounds(&renderer, "Open navigation drawer");
        assert!(menu.y1 <= 64.0 && menu.x0 < 16.0);
        let card = bounds(&renderer, "Open Button groups specifications");
        let adjacent = bounds(&renderer, "Open Buttons specifications");
        let third = bounds(&renderer, "Open Extended FABs specifications");
        let next_group = bounds(&renderer, "Open FAB menu specifications");
        assert_eq!(card.height(), 336.0);
        assert_eq!(card.y0, adjacent.y0);
        assert_eq!(
            card.y1, adjacent.y1,
            "short descriptions must not shorten cards within the same row"
        );
        assert_eq!(third.y0, card.y1 + 8.0);
        assert_eq!(
            (next_group.x0, next_group.y0),
            (adjacent.x0, third.y0),
            "a category fills the slot beside its third card"
        );
        click(&mut renderer, card);
        let back = bounds(&renderer, "Back to components");
        assert_eq!(back, menu);
        assert_eq!(bounds(&renderer, "Button groups specifications").y0, 64.0);
        let icon_pixels = |renderer: &HeadlessRenderer| {
            let pixels = renderer.read_pixels().unwrap();
            (16..48)
                .flat_map(|y| pixels[(y * 900 + 8) * 4..(y * 900 + 48) * 4].to_vec())
                .collect::<Vec<_>>()
        };
        let transitioning = icon_pixels(&renderer);
        renderer.render_frames(30);
        let finished = icon_pixels(&renderer);
        assert_ne!(
            transitioning, finished,
            "the icon must continue fading without pointer events"
        );
        renderer.render_frames(10);
        assert_eq!(
            finished,
            icon_pixels(&renderer),
            "the completed icon must remain stable"
        );
        for (width, image_x, image_width, image_height) in [
            (390, 8.0, 359.0, 179.0),
            (1440, 716.0, 700.0, 544.0),
            (900, 8.0, 869.0, 434.0),
        ] {
            renderer.resize(width, 1800);
            renderer.render_frames(5);
            let illustration = bounds(&renderer, "Button groups illustration");
            assert_eq!(illustration.x0, image_x);
            assert_eq!(illustration.width(), image_width);
            assert_eq!(illustration.height(), image_height);
            if width == 1440 {
                assert_eq!(illustration.y0, 72.0);
            }
            let title = bounds(&renderer, "Button groups");
            assert_eq!(title.x0, if width <= 600 { 40.0 } else { 64.0 });
            if width == 1440 {
                let description = bounds(
                    &renderer,
                    "Button groups organize buttons and add interactions between them",
                );
                assert!(
                    (title.y0 + description.y1 - 688.0).abs() <= 1.0,
                    "the wide header text block must be vertically centered"
                );
            }
        }
        click(&mut renderer, back);
        assert_eq!(bounds(&renderer, "Open navigation drawer"), menu);
        bounds(&renderer, "Open Button groups specifications");
        let transitioning = icon_pixels(&renderer);
        renderer.render_frames(30);
        assert_ne!(
            transitioning,
            icon_pixels(&renderer),
            "returning must fade back to the drawer icon"
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn dialog_article_switches_tabs_and_annotates_live_component_bounds() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 900,
                height: 8000,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(4);
        let card = bounds(&renderer, "Open Dialogs specifications");
        click(&mut renderer, card);
        renderer.render_frames(8);
        enter_specs(&mut renderer);
        let tabs = || ["Overview", "Specs", "Guidelines", "Accessibility"];
        let nodes = renderer.accessibility_snapshot().unwrap().nodes;
        let mut previous: Option<Rect> = None;
        for label in tabs() {
            let (_, node) = nodes
                .iter()
                .find(|(_, node)| node.role() == Role::Tab && crate::has_label(&node, label))
                .unwrap_or_else(|| {
                    panic!(
                        "Missing tab {label}: {:?}",
                        nodes
                            .iter()
                            .filter(|(_, node)| node.label().is_some())
                            .take(60)
                            .map(|(_, node)| (node.role(), node.label()))
                            .collect::<Vec<_>>()
                    )
                });
            assert_eq!(node.is_selected(), Some(label == "Specs"));
            let rect = node.bounds().unwrap();
            assert_eq!(rect.height(), 79.0);
            if let Some(previous) = previous {
                assert_eq!(rect.y0, previous.y0);
                assert_eq!(rect.x0, previous.x1);
            }
            previous = Some(rect);
        }
        let figure = bounds(&renderer, "Basic dialog measurement figure");
        let container = bounds(&renderer, "Basic dialog measurement container");
        assert!((container.width() - figure.width() * 312.0 / 1024.0).abs() <= 1.0);
        assert!((container.x0 - figure.x0 - figure.width() * 360.0 / 1024.0).abs() <= 1.0);
        let marker = bounds(&renderer, "Anatomy marker 2");
        assert!(marker.y1 < bounds(&renderer, "Basic dialog anatomy container").y0);
        let pixels = renderer.read_pixels().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 900 + x) * 4..(y * 900 + x) * 4 + 4];
        let selected_tab = tab(&renderer, "Specs").bounds().unwrap();
        let idle_tab = tab(&renderer, "Overview").bounds().unwrap();
        let background = pixel(
            (idle_tab.x0 + 10.0) as usize,
            ((idle_tab.y0 + idle_tab.y1) * 0.5) as usize,
        );
        assert_eq!(
            pixel(
                (selected_tab.x0 + 10.0) as usize,
                (selected_tab.y0 + 10.0) as usize
            ),
            background,
            "article tabs must have circular capsule ends rather than squircle corners"
        );
        assert_ne!(
            pixel(
                (selected_tab.x0 + 40.0) as usize,
                (selected_tab.y0 + 10.0) as usize
            ),
            background,
            "the selected capsule must still paint its interior"
        );
        assert_ne!(
            pixel(
                (container.x0 + 32.0) as usize,
                (container.y0 + 12.0) as usize
            ),
            pixel(
                (container.x0 + 32.0) as usize,
                (container.y0 + 28.0) as usize
            ),
            "padding overlay must be painted above the actual container"
        );
        let tab = bounds(&renderer, "Accessibility");
        click(&mut renderer, tab);
        bounds(&renderer, "Keyboard navigation");
        assert!(
            !renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| crate::has_label(&node, "Basic dialog measurement figure")),
            "inactive tab figures must be unmounted"
        );
        let overview = bounds(&renderer, "Overview");
        click(&mut renderer, overview);
        let open = control(&renderer, "Open basic dialog");
        let full = control(&renderer, "Open full-screen dialog");
        let illustration = bounds(&renderer, "Dialogs overview figure");
        assert!(full.bounds().unwrap().y0 >= open.bounds().unwrap().y1 + 12.0);
        assert!(
            open.bounds().unwrap().y0 >= illustration.y1 + 32.0,
            "intro content must stack instead of painting over its controls"
        );
        click(&mut renderer, open.bounds().unwrap());
        renderer.render_frames(30);
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::AlertDialog
                    && crate::has_label(&node, "Reset settings?"))
        );
        let cancel = control(&renderer, "Cancel");
        click(&mut renderer, cancel.bounds().unwrap());
        renderer.render_frames(30);
        let guidelines = bounds(&renderer, "Guidelines");
        click(&mut renderer, guidelines);
        bounds(&renderer, "Adaptive design");
        let specs = bounds(&renderer, "Specs");
        click(&mut renderer, specs);
        bounds(&renderer, "Basic dialog measurement figure");
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn selection_articles_keep_four_pages_and_labels_operate_their_controls() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 900,
                height: 8000,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(4);
        for title in ["Checkbox", "Radio button", "Switch"] {
            let card = bounds(&renderer, &format!("Open {title} specifications"));
            click(&mut renderer, card);
            renderer.render_frames(4);
            enter_specs(&mut renderer);
            let tab = |renderer: &HeadlessRenderer, label: &str| {
                renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .into_iter()
                    .find(|(_, node)| node.role() == Role::Tab && crate::has_label(&node, label))
                    .unwrap()
                    .1
            };
            for label in ["Overview", "Specs", "Guidelines", "Accessibility"] {
                assert_eq!(tab(&renderer, label).is_selected(), Some(label == "Specs"));
            }
            let overview = tab(&renderer, "Overview").bounds().unwrap();
            click(&mut renderer, overview);
            if title == "Radio button" {
                let second = control(&renderer, "Second option").bounds().unwrap();
                click(&mut renderer, second);
                assert_eq!(
                    control(&renderer, "Second option").toggled(),
                    Some(Toggled::True)
                );
                assert_eq!(
                    control(&renderer, "First option").toggled(),
                    Some(Toggled::False)
                );
            } else {
                let name = if title == "Checkbox" {
                    "Microphone access"
                } else {
                    "Wi-Fi"
                };
                for expected in [Toggled::False, Toggled::True] {
                    let target = control(&renderer, name).bounds().unwrap();
                    let label =
                        Rect::new(target.x0 - 334.0, target.y0, target.x0 - 326.0, target.y1);
                    click(&mut renderer, label);
                    assert_eq!(
                        control(&renderer, name).toggled(),
                        Some(expected),
                        "label must read the current selection"
                    );
                    let snapshot = renderer.accessibility_snapshot().unwrap();
                    assert!(crate::text_matches(
                        snapshot
                            .nodes
                            .iter()
                            .find(|(id, _)| *id == snapshot.focus)
                            .unwrap()
                            .1
                            .label(),
                        name
                    ));
                }
            }
            let guidelines = tab(&renderer, "Guidelines").bounds().unwrap();
            click(&mut renderer, guidelines);
            bounds(&renderer, "Usage");
            if title == "Radio button" {
                assert_eq!(
                    control(&renderer, "Callisto").toggled(),
                    Some(Toggled::True)
                );
                let ringtone = control(&renderer, "Luna").bounds().unwrap();
                click(
                    &mut renderer,
                    Rect::new(
                        ringtone.x1 + 8.0,
                        ringtone.y0,
                        ringtone.x1 + 40.0,
                        ringtone.y1,
                    ),
                );
                assert_eq!(control(&renderer, "Luna").toggled(), Some(Toggled::True));
                assert_eq!(
                    control(&renderer, "Callisto").toggled(),
                    Some(Toggled::False)
                );
                let radio = control(&renderer, "Turn off notifications")
                    .bounds()
                    .unwrap();
                assert_eq!(
                    control(&renderer, "Turn off notifications").toggled(),
                    Some(Toggled::False)
                );
                let label = Rect::new(radio.x1 + 8.0, radio.y0, radio.x1 + 40.0, radio.y1);
                click(&mut renderer, label);
                assert_eq!(
                    control(&renderer, "Turn off notifications").toggled(),
                    Some(Toggled::True)
                );
                assert_eq!(
                    control(&renderer, "Allow notifications").toggled(),
                    Some(Toggled::False)
                );
                let snapshot = renderer.accessibility_snapshot().unwrap();
                assert!(crate::text_matches(
                    snapshot
                        .nodes
                        .iter()
                        .find(|(id, _)| *id == snapshot.focus)
                        .unwrap()
                        .1
                        .label(),
                    "Turn off notifications"
                ));
            } else if title == "Checkbox" {
                assert_eq!(
                    control(&renderer, "All items").toggled(),
                    Some(Toggled::Mixed)
                );
                for expected in [Toggled::True, Toggled::False] {
                    let target = control(&renderer, "Photos").bounds().unwrap();
                    click(
                        &mut renderer,
                        Rect::new(target.x1 + 8.0, target.y0, target.x1 + 40.0, target.y1),
                    );
                    assert_eq!(control(&renderer, "Photos").toggled(), Some(expected));
                    assert_eq!(
                        control(&renderer, "All items").toggled(),
                        Some(Toggled::Mixed)
                    );
                }
                for (expected, count) in [(Toggled::True, 3), (Toggled::False, 0)] {
                    let target = control(&renderer, "All items").bounds().unwrap();
                    click(
                        &mut renderer,
                        Rect::new(target.x1 + 8.0, target.y0, target.x1 + 40.0, target.y1),
                    );
                    assert_eq!(control(&renderer, "All items").toggled(), Some(expected));
                    for label in ["Documents", "Photos", "Videos"] {
                        assert_eq!(control(&renderer, label).toggled(), Some(expected));
                    }
                    bounds(&renderer, &format!("Selected items: {count}"));
                }
                let child = control(&renderer, "Documents").bounds().unwrap();
                click(&mut renderer, child);
                assert_eq!(
                    control(&renderer, "All items").toggled(),
                    Some(Toggled::Mixed)
                );
                let parent = control(&renderer, "All items").bounds().unwrap();
                click(&mut renderer, parent);
                for label in ["Documents", "Photos", "Videos"] {
                    assert_eq!(control(&renderer, label).toggled(), Some(Toggled::True));
                }
                bounds(&renderer, "Selected items: 3");
            }
            let accessibility = tab(&renderer, "Accessibility").bounds().unwrap();
            click(&mut renderer, accessibility);
            bounds(&renderer, "Keyboard navigation");
            assert!(
                !renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Heading
                        && crate::has_label(&node, "Usage")),
                "inactive Guidelines chapters must be removed"
            );
            let back = bounds(&renderer, "Back to components");
            click(&mut renderer, back);
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn new_articles_keep_four_pages_and_route_live_collection_and_modal_actions() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 900,
                height: 1800,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(4);
        for demo in [2, 10, 11, 26, 27, 33, 34] {
            let component = COMPONENTS[component_index(demo).unwrap()];
            let card = reveal_card(
                &mut renderer,
                &format!("Open {} specifications", I18N.text(component.title)),
            );
            click(&mut renderer, card);
            renderer.render_frames(4);
            for page in [
                "Overview",
                "Guidelines",
                "Accessibility",
                "Specs",
                "Overview",
            ] {
                let tabs = renderer.accessibility_snapshot().unwrap().nodes;
                let tab = tabs
                    .iter()
                    .find(|(_, node)| node.role() == Role::Tab && crate::has_label(&node, page))
                    .unwrap_or_else(|| panic!("missing {page} tab for {}", component.title))
                    .1
                    .bounds()
                    .unwrap();
                click(&mut renderer, tab);
                assert!(
                    renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .iter()
                        .any(|(_, node)| node.role() == Role::Tab
                            && crate::has_label(&node, page)
                            && node.is_selected() == Some(true)),
                    "{} did not select {page}",
                    component.title
                );
            }
            match demo {
                2 => {
                    let all = control(&renderer, "Show all carousel items");
                    let all_id = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(_, node)| {
                            node.role() == Role::Button
                                && crate::has_label(&node, "Show all carousel items")
                        })
                        .unwrap()
                        .0;
                    click(&mut renderer, all.bounds().unwrap());
                    let item = control(&renderer, "Open collection 2 of 6");
                    click(&mut renderer, item.bounds().unwrap());
                    bounds(&renderer, "Collection 2 opened");
                    let back = control(&renderer, "Back to carousel");
                    click(&mut renderer, back.bounds().unwrap());
                    renderer.render_frames(30);
                    let restored = renderer.accessibility_snapshot().unwrap();
                    assert!(
                        !restored.nodes.iter().any(|(_, node)| {
                            node.role() == Role::Dialog
                                && crate::has_label(&node, "All carousel collections")
                        }),
                        "the real return action must close the collection page"
                    );
                    assert_eq!(
                        restored
                            .nodes
                            .iter()
                            .find(|(_, node)| {
                                node.role() == Role::Button
                                    && crate::has_label(&node, "Show all carousel items")
                            })
                            .expect("restored carousel action")
                            .0,
                        all_id,
                        "returning must restore the original carousel action identity"
                    );
                }
                10 => {
                    let access = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(_, node)| {
                            node.role() == Role::Tab && crate::has_label(&node, "Accessibility")
                        })
                        .unwrap()
                        .1
                        .bounds()
                        .unwrap();
                    click(&mut renderer, access);
                    activate_navigation(&mut renderer, "Keyboard navigation", Role::Link);
                    let brush = control(&renderer, "Brush");
                    click(&mut renderer, brush.bounds().unwrap());
                    bounds(&renderer, "Brush activated");
                    let brush_count = |renderer: &HeadlessRenderer| {
                        renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .iter()
                            .filter(|(_, node)| {
                                node.role() == Role::Button && crate::has_label(&node, "Brush")
                            })
                            .count()
                    };
                    let expanded_count = brush_count(&renderer);
                    let toggle = control(&renderer, "Toggle toolbar expansion");
                    click(&mut renderer, toggle.bounds().unwrap());
                    assert_eq!(brush_count(&renderer), expanded_count - 1);
                    let toggle = control(&renderer, "Toggle toolbar expansion");
                    click(&mut renderer, toggle.bounds().unwrap());
                    assert_eq!(brush_count(&renderer), expanded_count);
                }
                11 | 33 | 34 => {
                    if demo == 33 {
                        select_article_page(&mut renderer, "Guidelines");
                        activate_navigation(&mut renderer, "Modal bottom sheets", Role::Link);
                    }
                    let label = match demo {
                        11 => "Open modal navigation drawer",
                        33 => "Open modal bottom sheet",
                        _ => "Open modal side sheet",
                    };
                    let launch = control(&renderer, label);
                    click(&mut renderer, launch.bounds().unwrap());
                    renderer.render_frames(30);
                    if demo == 11 {
                        let modal = renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .into_iter()
                            .find(|(_, node)| {
                                node.role() == Role::Tab && crate::has_label(&node, "Outbox")
                            })
                            .expect("modal destination")
                            .1;
                        click(&mut renderer, modal.bounds().unwrap());
                        renderer.render_frames(30);
                        bounds(&renderer, "Current destination: Outbox");
                    } else {
                        if demo == 34 {
                            toggle(&mut renderer, "Events", Toggled::False, Toggled::True);
                        }
                        let action = control(
                            &renderer,
                            if demo == 33 {
                                "Bottom sheet Share"
                            } else {
                                "Save filters"
                            },
                        );
                        click(&mut renderer, action.bounds().unwrap());
                        renderer.render_frames(30);
                        assert!(
                            !renderer
                                .accessibility_snapshot()
                                .unwrap()
                                .nodes
                                .iter()
                                .any(|(_, node)| node.role() == Role::Dialog),
                            "the modal action must close the sheet"
                        );
                        select_article_page(&mut renderer, "Guidelines");
                        activate_navigation(&mut renderer, "Behavior", Role::Link);
                        bounds(
                            &renderer,
                            if demo == 33 {
                                "Last sheet action: Share selected"
                            } else {
                                "Filters applied"
                            },
                        );
                        let reopen = control(&renderer, label);
                        click(&mut renderer, reopen.bounds().unwrap());
                        renderer.render_frames(30);
                        assert!(
                            renderer
                                .accessibility_snapshot()
                                .unwrap()
                                .nodes
                                .iter()
                                .any(|(_, node)| node.role() == Role::Dialog),
                            "the owner must reopen its actual modal sheet"
                        );
                        if demo == 34 {
                            assert_eq!(
                                control(&renderer, "Events").toggled(),
                                Some(Toggled::True),
                                "saved filter input must survive closing and reopening"
                            );
                        }
                        let close = control(
                            &renderer,
                            if demo == 33 {
                                "Close bottom sheet"
                            } else {
                                "Close side sheet"
                            },
                        );
                        click(&mut renderer, close.bounds().unwrap());
                        renderer.render_frames(30);
                        assert!(
                            !renderer
                                .accessibility_snapshot()
                                .unwrap()
                                .nodes
                                .iter()
                                .any(|(_, node)| node.role() == Role::Dialog)
                        );
                        bounds(
                            &renderer,
                            if demo == 33 {
                                "Last sheet action: Share selected"
                            } else {
                                "Filters applied"
                            },
                        );
                    }
                }
                _ => {}
            }
            let back = bounds(&renderer, "Back to components");
            click(&mut renderer, back);
            bounds(&renderer, "Open navigation drawer");
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn time_modal_commits_only_on_confirmation_and_reopens_from_saved_time() {
        let initial_component = COMPONENTS
            .iter()
            .position(|component| component.title == "ui-time-pickers");
        let mut renderer = HeadlessRenderer::new(
            Box::new(move || {
                fixture().initial_component_optional(initial_component);
            }),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 900,
                height: 1200,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        enter_specs(&mut renderer);
        activate_navigation(&mut renderer, "Anatomy", Role::Link);
        renderer.render_frames(8);
        for confirm in [false, true] {
            let open = control(&renderer, "Open time picker dialog");
            click(&mut renderer, open.bounds().unwrap());
            renderer.render_frames(24);
            assert_eq!(
                scoped_node(&renderer, "Modal time preview", "Hour", Role::Button)
                    .unwrap()
                    .value(),
                Some("09")
            );
            let hour = scoped_node(
                &renderer,
                "Modal time preview",
                "Hour 3 of 12",
                Role::Button,
            )
            .unwrap();
            click(&mut renderer, hour.bounds().unwrap());
            let minute = scoped_node(
                &renderer,
                "Modal time preview",
                "Minute 15 of 60",
                Role::Button,
            )
            .unwrap();
            click(&mut renderer, minute.bounds().unwrap());
            let action = control(
                &renderer,
                if confirm {
                    "Modal confirm time"
                } else {
                    "Modal cancel time"
                },
            );
            click(&mut renderer, action.bounds().unwrap());
            renderer.render_frames(24);
            bounds(
                &renderer,
                if confirm {
                    "Saved time: 03:15"
                } else {
                    "Saved time: 09:41"
                },
            );
            assert!(
                !renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Dialog)
            );
        }
        let open = control(&renderer, "Open time picker dialog");
        click(&mut renderer, open.bounds().unwrap());
        renderer.render_frames(24);
        assert_eq!(
            scoped_node(&renderer, "Modal time preview", "Hour", Role::Button)
                .unwrap()
                .value(),
            Some("03")
        );
        assert_eq!(
            scoped_node(&renderer, "Modal time preview", "Minute", Role::Button)
                .unwrap()
                .value(),
            Some("15")
        );
        let hour = scoped_node(
            &renderer,
            "Modal time preview",
            "Hour 8 of 12",
            Role::Button,
        )
        .unwrap();
        click(&mut renderer, hour.bounds().unwrap());
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(4.0, 180.0));
        renderer.pointer_press(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(4.0, 180.0),
            PressKeyEventType::Left,
        );
        renderer.pointer_release(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(4.0, 180.0),
            PressKeyEventType::Left,
        );
        renderer.render_frames(24);
        assert!(
            !renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Dialog)
        );
        bounds(&renderer, "Saved time: 03:15");
        let open = control(&renderer, "Open time picker dialog");
        click(&mut renderer, open.bounds().unwrap());
        renderer.render_frames(24);
        assert_eq!(
            scoped_node(&renderer, "Modal time preview", "Hour", Role::Button)
                .unwrap()
                .value(),
            Some("03")
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn initial_app_bar_article_updates_its_variant_after_compact_resize() {
        let initial_component = COMPONENTS
            .iter()
            .position(|component| component.title == "ui-app-bars");
        let mut renderer = HeadlessRenderer::new(
            Box::new(move || {
                fixture().initial_component_optional(initial_component);
            }),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 900,
                height: 8000,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        enter_specs(&mut renderer);
        activate_navigation(&mut renderer, "Configurations", Role::Link);
        let configured_bar = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| crate::has_label(&node, "Configured app bar bar"))
            .expect("configured native app bar")
            .0;
        renderer.resize(390, 900);
        renderer.render_frames(8);
        renderer.resize(900, 8000);
        renderer.render_frames(8);
        assert_eq!(bounds(&renderer, "Configured app bar bar").height(), 136.0);
        let small = control(&renderer, "Configured app bar Small");
        click(&mut renderer, small.bounds().unwrap());
        assert_eq!(
            control(&renderer, "Configured app bar Small").toggled(),
            Some(Toggled::True),
            "The variant control must receive the real pointer click"
        );
        assert_eq!(
            bounds(&renderer, "Configured app bar bar").height(),
            64.0,
            "The preview must remeasure after changing the variant"
        );
        assert_eq!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| crate::has_label(&node, "Configured app bar bar"))
                .expect("retained configured native app bar")
                .0,
            configured_bar,
            "resize and variant input must retain the native preview identity"
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter for actual directory and compact split button navigation"]
    fn split_button_xl_configuration_keeps_both_segments_visible_and_live_at_compact_width() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 900,
                height: 8000,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let card = bounds(&renderer, "Open Split buttons specifications");
        click(&mut renderer, card);
        renderer.render_frames(8);
        enter_specs(&mut renderer);
        activate_navigation(&mut renderer, "Configurations", Role::Link);
        renderer.resize(390, 900);
        renderer.render_frames(8);
        for label in ["XL main action", "XL menu actions"] {
            let preview = control(&renderer, label).bounds().unwrap();
            assert!(
                preview.x0 >= 0.0 && preview.x1 <= 390.0,
                "the actual XL {label} must fit the compact viewport: {preview:?}"
            );
        }
        bounds(&renderer, "XL: 0 main actions");
        let main = control(&renderer, "XL main action");
        click(&mut renderer, main.bounds().unwrap());
        bounds(&renderer, "XL: 1 main actions");
        let menu = control(&renderer, "XL menu actions");
        click(&mut renderer, menu.bounds().unwrap());
        renderer.render_frames(30);
        assert_eq!(
            control(&renderer, "XL menu actions").is_expanded(),
            Some(true)
        );
        let draft = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| {
                node.role() == Role::MenuItem && crate::has_label(&node, "Save draft")
            })
            .expect("the open XL menu must expose its actual Save draft item")
            .1;
        click(&mut renderer, draft.bounds().unwrap());
        renderer.render_frames(30);
        bounds(&renderer, "Menu choice: Save draft");
        assert_eq!(
            control(&renderer, "XL menu actions").is_expanded(),
            Some(false)
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter for real directory and article tracks"]
    fn reserved_track_preserves_directory_groups_wheel_and_article_routes() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 907,
                height: 1800,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let track = |renderer: &HeadlessRenderer| {
            let bars = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .filter(|(_, node)| node.role() == Role::ScrollBar)
                .collect::<Vec<_>>();
            assert_eq!(bars.len(), 1, "Each page owns exactly one outer track");
            bars[0].1.bounds().unwrap()
        };
        let bar = track(&renderer);
        assert_eq!((bar.x0, bar.width()), (892.0, 15.0));
        let first = bounds(&renderer, "Open Button groups specifications");
        let second = bounds(&renderer, "Open Buttons specifications");
        let third = bounds(&renderer, "Open Extended FABs specifications");
        let next_group = bounds(&renderer, "Open FAB menu specifications");
        assert_eq!((first.width(), second.width()), (434.0, 434.0));
        assert_eq!(first.y0, second.y0);
        assert_eq!(third.y0, first.y1 + 8.0);
        assert_eq!(
            (next_group.x0, next_group.y0),
            (second.x0, third.y0),
            "a category fills the slot beside its third card"
        );
        renderer.pointer_scroll(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(900.0, 800.0),
            0.0,
            -120.0,
        );
        renderer.render_frames(120);
        assert_eq!(
            bounds(&renderer, "Open Button groups specifications").y0,
            first.y0 - 120.0
        );
        renderer.pointer_scroll(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(400.0, 800.0),
            0.0,
            120.0,
        );
        renderer.render_frames(120);
        assert_eq!(
            bounds(&renderer, "Open Button groups specifications"),
            first
        );
        click(&mut renderer, first);
        renderer.render_frames(8);
        assert_eq!(track(&renderer), bar);
        let guidelines = tab(&renderer, "Guidelines").bounds().unwrap();
        click(&mut renderer, guidelines);
        assert_eq!(tab(&renderer, "Guidelines").is_selected(), Some(true));
        assert_eq!(track(&renderer), bar);
        let back = bounds(&renderer, "Back to components");
        click(&mut renderer, back);
        renderer.render_frames(8);
        assert_eq!(track(&renderer), bar);
        assert_eq!(
            bounds(&renderer, "Open Button groups specifications").width(),
            434.0
        );
    }

    fn reveal_card(renderer: &mut HeadlessRenderer, label: &str) -> Rect {
        let viewport = renderer.physical_size();
        for step in 0..64 {
            let target = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| crate::has_label(node, label))
                .and_then(|(_, node)| node.bounds());
            let delta = if let Some(bounds) = target {
                if bounds.y0 >= 64.0 && bounds.y1 <= f64::from(viewport.height) {
                    return bounds;
                }
                (f64::from(viewport.height) * 0.5 - (bounds.y0 + bounds.y1) * 0.5) as f32
            } else if step == 0 {
                1_000_000.0
            } else {
                -(viewport.height as f32 * 0.7)
            };
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(viewport.width as f32 * 0.5, viewport.height as f32 * 0.5),
                0.0,
                delta,
            );
            renderer.render_frames(40);
        }
        panic!("missing {label}");
    }

    fn assert_directory_packing(renderer: &HeadlessRenderer, columns: usize) {
        let nodes = renderer.accessibility_snapshot().unwrap().nodes;
        let gap = f64::from(tessera_ui::Px::from(Dp(8.0)).0);
        let mut observed = 0;
        for category in 0..6 {
            let cards = COMPONENTS
                .iter()
                .filter(|component| component.category == category)
                .enumerate()
                .filter_map(|(index, component)| {
                    let label = format!("Open {} specifications", I18N.text(component.title));
                    let mut matches = nodes
                        .iter()
                        .filter(|(_, node)| crate::has_label(node, &label));
                    let card = matches.next()?.1.bounds().unwrap();
                    assert!(matches.next().is_none(), "duplicate {label}");
                    Some((index, card))
                })
                .collect::<Vec<_>>();
            observed += cards.len();
            for pair in cards.windows(2) {
                let [(previous_index, previous), (index, card)] = pair else {
                    unreachable!()
                };
                assert_eq!(card.width(), previous.width());
                if index / columns == previous_index / columns {
                    assert_eq!(card.y0, previous.y0);
                    assert_eq!(card.height(), previous.height());
                    assert_eq!(card.x0, previous.x1 + gap);
                } else if index / columns == previous_index / columns + 1 {
                    assert_eq!(card.y0, previous.y1 + gap);
                    assert_eq!(
                        card.x0,
                        previous.x0 - (previous_index % columns) as f64 * (card.width() + gap)
                    );
                }
            }
        }
        assert!(observed > 0, "directory cards must be mounted");
    }

    #[test]
    #[ignore = "requires a native GPU adapter; runs in a fresh child process"]
    fn directory_packs_visible_rows_across_breakpoints_and_resizes() {
        let name = "pages::material_spec::catalog::tests::directory_packs_visible_rows_across_breakpoints_and_resizes";
        if std::env::var("TESSERA_DIRECTORY_PACKING_TEST").as_deref() != Ok(name) {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    name,
                    "--include-ignored",
                    "--test-threads=1",
                    "--nocapture",
                ])
                .env("TESSERA_DIRECTORY_PACKING_TEST", name)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("test result: ok. 1 passed;"));
            return;
        }
        // A 1650x1200 browser viewport at device pixel ratio 1.5 is the
        // default window, where the directory shows two columns.
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 1650,
                height: 1200,
                scale_factor: 1.5,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        reveal_card(&mut renderer, "Open Button groups specifications");
        assert_directory_packing(&renderer, 2);
        for (width, columns) in [
            (930, 2),
            (900, 1),
            (930, 2),
            (1950, 2),
            (1980, 3),
            (1650, 2),
            (1980, 3),
            (900, 1),
            (1650, 2),
        ] {
            renderer.resize(width, 1200);
            renderer.render_frames(8);
            reveal_card(&mut renderer, "Open Button groups specifications");
            assert_directory_packing(&renderer, columns);
        }
        let top = bounds(&renderer, "Open Button groups specifications");
        renderer.pointer_scroll(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(800.0, 600.0),
            0.0,
            -1500.0,
        );
        renderer.render_frames(120);
        assert!(
            !renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(
                    |(_, node)| crate::has_label(node, "Open Button groups specifications")
                        && node.bounds() == Some(top)
                )
        );
        assert_directory_packing(&renderer, 2);
        let cards = reveal_card(&mut renderer, "Open Cards specifications");
        click(&mut renderer, cards);
        renderer.render_frames(8);
        let back = bounds(&renderer, "Back to components");
        click(&mut renderer, back);
        renderer.render_frames(8);
        reveal_card(&mut renderer, "Open Button groups specifications");
        assert_directory_packing(&renderer, 2);
    }
}
