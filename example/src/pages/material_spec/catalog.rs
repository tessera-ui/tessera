//! Component directory and independent interactive articles.
//!
//! ## Usage
//!
//! Browse components by category and open interactive specifications.

use tessera_components::{
    boxed::boxed,
    column::column,
    flow_row::flow_row,
    modifier::{ModifierExt as _, SemanticsArgs},
    scrollable::ScrollableController,
    shape_def::Shape,
    surface::surface,
    text::text,
};
use tessera_ui::{Dp, Modifier, State, modifier::FocusModifierExt as _, remember, tessera};

use super::{BACKGROUND, INK, PANEL, gap, heading, slider_spec, type_style};

#[derive(Clone, Copy, PartialEq)]
struct Component {
    title: &'static str,
    description: &'static str,
    category: usize,
    demo: usize,
}

const COMPONENTS: &[Component] = &[
    Component {
        title: "Button groups",
        description: "Button groups organize buttons and add interactions between them",
        category: 0,
        demo: 18,
    },
    Component {
        title: "Buttons",
        description: "Buttons prompt most actions in a UI",
        category: 0,
        demo: 36,
    },
    Component {
        title: "Extended FABs",
        description: "Extended floating action buttons (extended FABs) help people take primary actions",
        category: 0,
        demo: 9,
    },
    Component {
        title: "FAB menu",
        description: "The floating action button (FAB) menu opens from a FAB to display multiple related actions",
        category: 0,
        demo: 8,
    },
    Component {
        title: "Floating action buttons (FABs)",
        description: "Floating action buttons (FABs) help people take primary actions",
        category: 0,
        demo: 19,
    },
    Component {
        title: "Icon buttons",
        description: "Icon buttons help people take actions with a single tap",
        category: 0,
        demo: 3,
    },
    Component {
        title: "Segmented buttons",
        description: "Segmented buttons help people select options, switch views, or sort elements",
        category: 0,
        demo: 20,
    },
    Component {
        title: "Split buttons",
        description: "Split buttons open a menu to give people more options related to an action",
        category: 0,
        demo: 22,
    },
    Component {
        title: "Date pickers",
        description: "Date pickers let people select a date, or a range of dates",
        category: 1,
        demo: 28,
    },
    Component {
        title: "Time pickers",
        description: "Time pickers help people select and set a specific time",
        category: 1,
        demo: 30,
    },
    Component {
        title: "Loading indicator",
        description: "Loading indicators show the progress of a process for a short wait time",
        category: 2,
        demo: 6,
    },
    Component {
        title: "Progress indicators",
        description: "Progress indicators show the status of a process in real time",
        category: 2,
        demo: 7,
    },
    Component {
        title: "Navigation bar",
        description: "Navigation bars let people switch between UI views on smaller devices",
        category: 3,
        demo: 31,
    },
    Component {
        title: "Navigation drawer",
        description: "Navigation drawers let people switch between UI views on larger devices",
        category: 3,
        demo: 11,
    },
    Component {
        title: "Navigation rail",
        description: "Navigation rails let people switch between UI views on mid-sized devices",
        category: 3,
        demo: 32,
    },
    Component {
        title: "Bottom sheets",
        description: "Bottom sheets show secondary content anchored to the bottom of the screen",
        category: 4,
        demo: 33,
    },
    Component {
        title: "Side sheets",
        description: "Side sheets show secondary content anchored to the side of the screen",
        category: 4,
        demo: 34,
    },
    Component {
        title: "App bars",
        description: "App bars are placed at the top of the screen to help people navigate through a product ",
        category: 5,
        demo: 1,
    },
    Component {
        title: "Badges",
        description: "Badges show notifications, counts, or status information on navigation items and icons",
        category: 5,
        demo: 12,
    },
    Component {
        title: "Cards",
        description: "Cards display content and actions about a single subject",
        category: 5,
        demo: 16,
    },
    Component {
        title: "Carousel",
        description: "Carousels show a collection of items that can be scrolled on and off the screen",
        category: 5,
        demo: 2,
    },
    Component {
        title: "Checkbox",
        description: "Checkboxes let users select one or more items from a list, or turn an item on or off",
        category: 5,
        demo: 37,
    },
    Component {
        title: "Chips",
        description: "Chips help people enter information, make selections, filter content, or trigger actions",
        category: 5,
        demo: 17,
    },
    Component {
        title: "Dialogs",
        description: "Dialogs provide important prompts in a user flow",
        category: 5,
        demo: 13,
    },
    Component {
        title: "Divider",
        description: "Dividers are thin lines that group content in lists or other containers",
        category: 5,
        demo: 14,
    },
    Component {
        title: "Lists",
        description: "Lists are continuous, vertical indexes of text and images",
        category: 5,
        demo: 15,
    },
    Component {
        title: "Menus",
        description: "Menus display a list of choices on a temporary surface",
        category: 5,
        demo: 26,
    },
    Component {
        title: "Radio button",
        description: "Radio buttons let people select one option from a set of options",
        category: 5,
        demo: 23,
    },
    Component {
        title: "Search",
        description: "Search lets people enter a keyword or phrase to get relevant information",
        category: 5,
        demo: 27,
    },
    Component {
        title: "Sliders",
        description: "Sliders allow users to make selections from a range of values",
        category: 5,
        demo: 5,
    },
    Component {
        title: "Snackbar",
        description: "Snackbars show short updates about app processes at the bottom of the screen",
        category: 5,
        demo: 25,
    },
    Component {
        title: "Switch",
        description: "Switches toggle the selection of an item on and off",
        category: 5,
        demo: 38,
    },
    Component {
        title: "Tabs",
        description: "Tabs organize content across different screens and views",
        category: 5,
        demo: 21,
    },
    Component {
        title: "Text fields",
        description: "Text fields let users enter text into a UI",
        category: 5,
        demo: 24,
    },
    Component {
        title: "Toolbars",
        description: "Toolbars display frequently used actions relevant to the current page",
        category: 5,
        demo: 10,
    },
    Component {
        title: "Tooltips",
        description: "Tooltips display brief labels or messages",
        category: 5,
        demo: 4,
    },
];

#[tessera]
pub(super) fn component_pages() {
    super::media::load_fonts();
    let fallback = remember(|| None::<usize>);
    let selected = tessera_ui::use_context::<crate::app::MaterialNavigation>()
        .map(|navigation| navigation.get().0)
        .unwrap_or(fallback);
    let directory_scroll = remember(ScrollableController::new);
    let width = remember(|| Dp(900.0));
    if let Some(index) = selected.get() {
        let component = COMPONENTS[index];
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                boxed()
                    .modifier(Modifier::new().fill_max_width().weight(1.0).semantics(
                        SemanticsArgs {
                            test_tag: Some(format!("component-article-{}", component.demo)),
                            role: Some(tessera_ui::accesskit::Role::Group),
                            label: Some(format!("{} specifications", component.title)),
                            merge_descendants: true,
                            ..Default::default()
                        },
                    ))
                    .children(move || {
                        if component.demo == 13 {
                            super::dialogs::dialogs_spec();
                        } else if component.demo == 25 {
                            super::snackbars::snackbars_spec();
                        } else if component.demo == 1 {
                            super::app_bars::app_bars_spec();
                        } else if component.demo == 20 {
                            super::segmented::segmented_spec();
                        } else if component.demo == 24 {
                            super::text_fields::text_fields_spec();
                        } else if component.demo == 4 {
                            super::tooltips::tooltips_spec();
                        } else if component.demo == 31 || component.demo == 32 {
                            super::navigation::navigation_spec(component.demo == 32);
                        } else if component.demo == 21 {
                            super::tabs::tabs_spec();
                        } else if component.demo == 6 || component.demo == 7 {
                            super::activity::activity_spec(component.demo == 6);
                        } else if component.demo == 12 {
                            super::badges::badges_spec();
                        } else if component.demo == 18 {
                            super::button_groups::button_groups_spec();
                        } else if component.demo == 22 {
                            super::split_buttons::split_buttons_spec();
                        } else if component.demo == 8 {
                            super::fab_menu::fab_menu_spec();
                        } else if component.demo == 9 || component.demo == 19 {
                            super::floating_actions::floating_actions_spec(component.demo == 9);
                        } else if component.demo == 28 {
                            super::date_pickers::date_pickers_spec();
                        } else if component.demo == 30 {
                            super::time_pickers::time_pickers_spec();
                        } else if component.demo == 16 {
                            super::cards::cards_spec();
                        } else if component.demo == 15 {
                            super::lists::lists_spec();
                        } else if component.demo == 14 {
                            super::dividers::divider_spec();
                        } else if component.demo == 3 {
                            super::icon_buttons::icon_buttons_spec();
                        } else if component.demo == 17 {
                            super::chips::chips_spec();
                        } else if component.demo == 23 {
                            super::radio_buttons::radio_buttons_spec();
                        } else if component.demo == 5 {
                            slider_spec();
                        } else if component.demo == 36 {
                            super::buttons::buttons_spec();
                        } else if component.demo == 37 {
                            super::selection_controls::checkbox_spec();
                        } else if component.demo == 38 {
                            super::selection_controls::switch_spec();
                        } else if component.demo == 26 {
                            super::menus::menus_spec();
                        } else if component.demo == 27 {
                            super::search::search_spec();
                        } else if component.demo == 2 {
                            super::carousel::carousel_spec();
                        } else if component.demo == 10 {
                            super::toolbars::toolbars_spec();
                        } else if component.demo == 11 {
                            super::drawers::drawers_spec();
                        } else if component.demo == 33 || component.demo == 34 {
                            super::sheets::sheets_spec(component.demo == 34);
                        } else {
                            unreachable!("unregistered component article");
                        }
                    });
            });
        return;
    }
    surface()
        .style(BACKGROUND.into())
        .modifier(Modifier::new().fill_max_size())
        .child(move || {
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                            let next = Dp::from_pixels_f32(bounds.width.0 as f32);
                            if width.get() != next {
                                width.set(next);
                            }
                        })
                        .vertical_scroll(directory_scroll),
                )
                .children(move || {
                    super::media::page_header("Components", "", width.get());
                    for (category, title) in [
                        "Buttons",
                        "Date & time pickers",
                        "Loading & progress",
                        "Navigation",
                        "Sheets",
                        "All other components",
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        gap(Dp(if category == 0 { 72.0 } else { 80.0 }));
                        boxed()
                            .modifier(Modifier::new().padding_symmetric(Dp(33.0), Dp::ZERO))
                            .children(move || {
                                heading(title.to_owned()).compact(width.get().0 <= 600.0);
                            });
                        gap(Dp(24.0));
                        let entries = COMPONENTS
                            .iter()
                            .copied()
                            .enumerate()
                            .filter(|(_, component)| component.category == category)
                            .collect::<Vec<_>>();
                        for (group_index, group) in entries.chunks(3).enumerate() {
                            if group_index > 0 {
                                gap(Dp(8.0));
                            }
                            directory_group(group.to_vec(), selected, width.get());
                        }
                    }
                    gap(Dp(64.0));
                });
        });
}

pub(super) fn component_index(demo: usize) -> Option<usize> {
    COMPONENTS
        .iter()
        .position(|component| component.demo == demo)
}

#[tessera]
fn directory_group(entries: Vec<(usize, Component)>, selected: State<Option<usize>>, width: Dp) {
    let columns = if width.0 <= 600.0 {
        1.0
    } else if width.0 < 1295.0 {
        2.0
    } else {
        3.0
    };
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
                component_card(component, index, selected, card_width);
            }
        });
}

#[tessera]
fn component_card(component: Component, index: usize, selected: State<Option<usize>>, width: Dp) {
    surface()
        .style(PANEL.into())
        .shape(Shape::rounded_rectangle(Dp(24.0)))
        .modifier(Modifier::new().width(width).semantics(SemanticsArgs {
            test_tag: Some(format!("component-card-{index}")),
            ..Default::default()
        }))
        .on_click(move || selected.set(Some(index)))
        .accessibility_label(format!("Open {} specifications", component.title))
        .child(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    super::media::picture(component.title, width, Dp(200.0)).thumbnail(true);

                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                        .children(move || {
                            text()
                                .content(component.title)
                                .style(type_style(24.0, 32.0))
                                .color(INK);
                            gap(Dp(8.0));
                            text()
                                .content(component.description)
                                .style(type_style(16.0, 24.0))
                                .color(INK);
                        });
                });
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use tessera_components::theme::material_theme;
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        accesskit::{Node, Rect, Role, Toggled},
    };

    use super::*;

    struct ComponentPipelines;

    impl tessera_ui::RenderModule for ComponentPipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[tessera]
    fn fixture() {
        material_theme().child(|| {
            let selected = remember(|| None::<usize>);
            let drawer =
                remember(|| tessera_components::side_sheet::SideSheetController::new(false));
            tessera_ui::provide_context(
                || crate::app::MaterialNavigation(selected),
                || {
                    column()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            tessera_components::app_bar::top_app_bar()
                                .title("Material Design 3")
                                .navigation_icon(move || {
                                    crate::app::navigation_button(selected, drawer, true);
                                });
                            boxed()
                                .modifier(Modifier::new().fill_max_width().weight(1.0))
                                .children(move || {
                                    component_pages();
                                });
                        });
                },
            );
        });
    }

    fn bounds(renderer: &HeadlessRenderer, label: &str) -> Rect {
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| node.label() == Some(label))
            .unwrap_or_else(|| panic!("missing {label}"))
            .1
            .bounds()
            .unwrap()
    }

    fn click(renderer: &mut HeadlessRenderer, mut bounds: Rect) {
        let viewport = renderer.physical_size();
        if bounds.y1 > f64::from(viewport.height) || bounds.y0 < 64.0 {
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
                node.label() == Some(label)
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
        let nodes = renderer.accessibility_snapshot().unwrap().nodes;
        let root = nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Group && node.label() == Some(group))
            .unwrap_or_else(|| panic!("missing group {group}"));
        let mut descendants = root.1.children().to_vec();
        while let Some(id) = descendants.pop() {
            let (_, node) = nodes
                .iter()
                .find(|(node_id, _)| *node_id == id)
                .expect("published child");
            let mut content = node.children().to_vec();
            let mut named = node.label() == Some(label);
            while !named && node.label().is_none() && !content.is_empty() {
                let id = content.pop().unwrap();
                let (_, child) = nodes.iter().find(|(key, _)| *key == id).unwrap();
                named = child.label() == Some(label);
                content.extend_from_slice(child.children());
            }
            if named && node.role() == role {
                return Some(node.clone());
            }
            descendants.extend_from_slice(node.children());
        }
        None
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

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn app_bar_crossfades_to_back_and_restores_directory_navigation() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(ComponentPipelines)],
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
            next_group.y0,
            third.y1 + 8.0,
            "three-card groups must retain the official row breaks"
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
            (390, 8.0, 374.0, 187.0),
            (1440, 724.0, 708.0, 544.0),
            (900, 8.0, 884.0, 442.0),
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
            vec![Box::new(ComponentPipelines)],
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
        let tabs = || ["Overview", "Specs", "Guidelines", "Accessibility"];
        let nodes = renderer.accessibility_snapshot().unwrap().nodes;
        let mut previous: Option<Rect> = None;
        for label in tabs() {
            let (_, node) = nodes
                .iter()
                .find(|(_, node)| node.role() == Role::Tab && node.label() == Some(label))
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
            assert_eq!(rect.height(), 80.0);
            if let Some(previous) = previous {
                assert_eq!(rect.y0, previous.y0);
                assert_eq!(rect.x0, previous.x1);
            }
            previous = Some(rect);
        }
        let figure = bounds(&renderer, "Basic dialog measurement figure");
        let container = bounds(&renderer, "Basic dialog measurement container");
        assert_eq!(container.width(), 312.0);
        assert!(((container.x0 + container.x1) - (figure.x0 + figure.x1)).abs() < 1.0);
        let marker = bounds(&renderer, "Anatomy marker 2");
        assert!(marker.y1 < bounds(&renderer, "Basic dialog anatomy container").y0);
        let pixels = renderer.read_pixels().unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 900 + x) * 4..(y * 900 + x) * 4 + 4];
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
                .any(|(_, node)| node.label() == Some("Basic dialog measurement figure")),
            "inactive tab figures must be unmounted"
        );
        let overview = bounds(&renderer, "Overview");
        click(&mut renderer, overview);
        let open = control(&renderer, "Open basic dialog");
        let full = control(&renderer, "Open full-screen dialog");
        let illustration = bounds(&renderer, "Basic dialog anatomy figure");
        assert!(full.bounds().unwrap().y0 >= open.bounds().unwrap().y1 + 12.0);
        assert!(
            illustration.y0 >= full.bounds().unwrap().y1 + 32.0,
            "intro content must stack instead of painting over its controls"
        );
        click(&mut renderer, open.bounds().unwrap());
        renderer.render_frames(30);
        assert!(renderer.accessibility_snapshot().unwrap().nodes.iter().any(
            |(_, node)| node.role() == Role::Dialog && node.label() == Some("Reset settings?")
        ));
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
            vec![Box::new(ComponentPipelines)],
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
            let tab = |renderer: &HeadlessRenderer, label: &str| {
                renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .into_iter()
                    .find(|(_, node)| node.role() == Role::Tab && node.label() == Some(label))
                    .unwrap()
                    .1
            };
            for label in ["Overview", "Specs", "Guidelines", "Accessibility"] {
                assert_eq!(tab(&renderer, label).is_selected(), Some(label == "Specs"));
            }
            let overview = tab(&renderer, "Overview").bounds().unwrap();
            click(&mut renderer, overview);
            if title == "Radio button" {
                let radio = control(&renderer, "Notification preference: Important only")
                    .bounds()
                    .unwrap();
                let label = Rect::new(radio.x1 + 8.0, radio.y0, radio.x1 + 40.0, radio.y1);
                click(&mut renderer, label);
                assert_eq!(
                    control(&renderer, "Notification preference: Important only").toggled(),
                    Some(Toggled::True)
                );
                assert_eq!(
                    control(&renderer, "Notification preference: All notifications").toggled(),
                    Some(Toggled::False)
                );
                let snapshot = renderer.accessibility_snapshot().unwrap();
                assert_eq!(
                    snapshot
                        .nodes
                        .iter()
                        .find(|(id, _)| *id == snapshot.focus)
                        .unwrap()
                        .1
                        .label(),
                    Some("Notification preference: Important only")
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
                    assert_eq!(
                        snapshot
                            .nodes
                            .iter()
                            .find(|(id, _)| *id == snapshot.focus)
                            .unwrap()
                            .1
                            .label(),
                        Some(name)
                    );
                }
            }
            let guidelines = tab(&renderer, "Guidelines").bounds().unwrap();
            click(&mut renderer, guidelines);
            bounds(&renderer, "Usage");
            if title == "Checkbox" {
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
                    .any(|(_, node)| node.role() == Role::Heading && node.label() == Some("Usage")),
                "inactive Guidelines chapters must be removed"
            );
            let back = bounds(&renderer, "Back to components");
            click(&mut renderer, back);
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn action_articles_switch_pages_and_preserve_live_actions() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(ComponentPipelines)],
            HeadlessConfig {
                width: 900,
                height: 1800,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(4);
        for demo in [18, 36, 9, 8, 19, 3, 20, 22] {
            let component = COMPONENTS[component_index(demo).unwrap()];
            let card = bounds(
                &renderer,
                &format!("Open {} specifications", component.title),
            );
            click(&mut renderer, card);
            let tab = |renderer: &HeadlessRenderer, label: &str| {
                renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .into_iter()
                    .find(|(_, node)| node.role() == Role::Tab && node.label() == Some(label))
                    .unwrap()
                    .1
            };
            for label in ["Overview", "Specs", "Guidelines", "Accessibility"] {
                assert_eq!(
                    tab(&renderer, label).is_selected(),
                    Some(label == "Specs"),
                    "initial page of {}",
                    component.title
                );
            }
            let overview = tab(&renderer, "Overview").bounds().unwrap();
            click(&mut renderer, overview);
            match demo {
                18 => {
                    let one =
                        scoped_node(&renderer, "Standard group", "One", Role::CheckBox).unwrap();
                    click(&mut renderer, one.bounds().unwrap());
                    bounds(&renderer, "Standard group selection: 1");
                }
                36 => {
                    toggle(
                        &mut renderer,
                        "Toggle button",
                        Toggled::False,
                        Toggled::True,
                    );
                }
                9 | 19 => {
                    let label = if demo == 9 {
                        "Small floating action"
                    } else {
                        "Regular floating action"
                    };
                    let target = control(&renderer, label);
                    click(&mut renderer, target.bounds().unwrap());
                    bounds(&renderer, &format!("{label}: 1"));
                }
                8 => {
                    let open = control(&renderer, "Open Overview actions");
                    click(&mut renderer, open.bounds().unwrap());
                    renderer.render_frames(30);
                    let item = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(_, node)| {
                            node.role() == Role::MenuItem && node.label() == Some("New document")
                        })
                        .unwrap()
                        .1;
                    click(&mut renderer, item.bounds().unwrap());
                    renderer.render_frames(30);
                    bounds(&renderer, "Overview: Document created");
                }
                3 => {
                    let target = control(&renderer, "Toggle favorite").bounds().unwrap();
                    renderer.render_frames(30);
                    let icon_is_filled = |renderer: &mut HeadlessRenderer| {
                        let pixels = renderer.read_pixels().unwrap();
                        let x = ((target.x0 + target.x1) * 0.5) as usize;
                        let y = ((target.y0 + target.y1) * 0.5) as usize;
                        let center = &pixels[(y * 900 + x) * 4..(y * 900 + x) * 4 + 3];
                        let empty = &pixels[((y - 20) * 900 + x) * 4..((y - 20) * 900 + x) * 4 + 3];
                        center.iter().zip(empty).any(|(a, b)| a.abs_diff(*b) > 20)
                    };
                    assert!(
                        !icon_is_filled(&mut renderer),
                        "unselected favorite must have an outlined icon"
                    );
                    click(&mut renderer, target);
                    renderer.render_frames(30);
                    assert!(
                        icon_is_filled(&mut renderer),
                        "selected favorite must have a filled icon"
                    );
                }
                20 => {
                    toggle(
                        &mut renderer,
                        "Multi-select Month",
                        Toggled::False,
                        Toggled::True,
                    );
                }
                22 => {
                    let send = control(&renderer, "Overview send");
                    click(&mut renderer, send.bounds().unwrap());
                    bounds(&renderer, "Overview actions: 1");
                    let more = control(&renderer, "Overview options");
                    click(&mut renderer, more.bounds().unwrap());
                    renderer.render_frames(30);
                    let item = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(_, node)| {
                            node.role() == Role::MenuItem && node.label() == Some("Schedule send")
                        })
                        .unwrap()
                        .1;
                    click(&mut renderer, item.bounds().unwrap());
                    renderer.render_frames(30);
                    bounds(&renderer, "Scheduled send selected");
                }
                _ => {}
            }
            let guide = tab(&renderer, "Guidelines").bounds().unwrap();
            click(&mut renderer, guide);
            bounds(&renderer, "Usage");
            let access = tab(&renderer, "Accessibility").bounds().unwrap();
            click(&mut renderer, access);
            assert!(
                renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Keyboard navigation")),
                "missing keyboard chapter for {}: {:?}",
                component.title,
                renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .filter_map(|(_, node)| node.label().map(|label| (node.role(), label)))
                    .collect::<Vec<_>>()
            );
            if demo == 3 {
                let trigger = control(&renderer, "Hover favorite action");
                click(&mut renderer, trigger.bounds().unwrap());
                renderer.render_frames(40);
                bounds(&renderer, "Hover example actions: 1");
                assert!(
                    renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .iter()
                        .any(|(_, node)| node.role() == Role::Tooltip
                            && node.label() == Some("Add to favorites"))
                );
            }
            assert!(
                !renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Heading && node.label() == Some("Usage"))
            );
            if demo == 20 {
                let recommended = control(&renderer, "Open recommended button groups");
                click(&mut renderer, recommended.bounds().unwrap());
                bounds(&renderer, "Button groups specifications");
            }
            let back = bounds(&renderer, "Back to components");
            click(&mut renderer, back);
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn new_articles_keep_four_pages_and_route_live_collection_and_modal_actions() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(ComponentPipelines)],
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
            let card = bounds(
                &renderer,
                &format!("Open {} specifications", component.title),
            );
            click(&mut renderer, card);
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
                    .find(|(_, node)| node.role() == Role::Tab && node.label() == Some(page))
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
                            && node.label() == Some(page)
                            && node.is_selected() == Some(true)),
                    "{} did not select {page}",
                    component.title
                );
            }
            match demo {
                2 => {
                    let all = control(&renderer, "Show all carousel items");
                    click(&mut renderer, all.bounds().unwrap());
                    let item = control(&renderer, "Open collection 2 of 6");
                    click(&mut renderer, item.bounds().unwrap());
                    bounds(&renderer, "Collection 2 opened");
                    let back = control(&renderer, "Back to carousel");
                    click(&mut renderer, back.bounds().unwrap());
                    control(&renderer, "Show all carousel items");
                }
                10 => {
                    let access = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(_, node)| {
                            node.role() == Role::Tab && node.label() == Some("Accessibility")
                        })
                        .unwrap()
                        .1
                        .bounds()
                        .unwrap();
                    click(&mut renderer, access);
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
                                node.role() == Role::Button && node.label() == Some("Brush")
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
                                node.role() == Role::Tab && node.label() == Some("Outbox")
                            })
                            .expect("modal destination")
                            .1;
                        click(&mut renderer, modal.bounds().unwrap());
                        renderer.render_frames(30);
                        bounds(&renderer, "Current destination: Outbox");
                    } else {
                        let share = control(
                            &renderer,
                            if demo == 33 {
                                "Bottom sheet Share"
                            } else {
                                "Side sheet Share"
                            },
                        );
                        click(&mut renderer, share.bounds().unwrap());
                        renderer.render_frames(30);
                        bounds(&renderer, "Share selected");
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
    fn every_directory_card_opens_its_article_and_returns_to_the_directory() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(ComponentPipelines)],
            HeadlessConfig {
                width: 900,
                height: 8000,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(3);
        for component in COMPONENTS {
            let card = bounds(
                &renderer,
                &format!("Open {} specifications", component.title),
            );
            click(&mut renderer, card);
            let snapshot = renderer.accessibility_snapshot().unwrap();
            assert!(
                snapshot
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Group
                        && node.label()
                            == Some(format!("{} specifications", component.title).as_str())),
                "wrong article for {}",
                component.title
            );
            for page in ["Overview", "Specs", "Guidelines", "Accessibility"] {
                assert!(
                    snapshot.nodes.iter().any(|(_, node)| {
                        node.role() == Role::Tab && node.label() == Some(page)
                    }),
                    "missing {page} page in {}",
                    component.title
                );
            }
            match component.demo {
                20 => {
                    renderer.render_frames(5);
                    let day = control(&renderer, "Single-select Day");
                    assert_eq!(day.toggled(), Some(Toggled::True));
                    assert_eq!(day.bounds().unwrap().height(), 48.0);
                    let multi = control(&renderer, "Multi-select Day");
                    assert!(
                        multi.bounds().unwrap().y0 >= day.bounds().unwrap().y1 + 32.0,
                        "a shared panel must arrange examples vertically without overlapping hit targets"
                    );
                    let month = control(&renderer, "Single-select Month");
                    assert!(
                        month.bounds().unwrap().x0 > day.bounds().unwrap().x0,
                        "day {:?}, month {:?}",
                        day.bounds(),
                        month.bounds()
                    );
                    assert!(!month.is_disabled(), "month disabled");
                    click(&mut renderer, month.bounds().unwrap());
                    assert_eq!(
                        control(&renderer, "Single-select Day").toggled(),
                        Some(Toggled::False)
                    );
                    assert_eq!(
                        control(&renderer, "Single-select Month").toggled(),
                        Some(Toggled::True)
                    );
                    bounds(&renderer, "Single-select selection: Month");
                    let month = control(&renderer, "Multi-select Month");
                    click(&mut renderer, month.bounds().unwrap());
                    bounds(&renderer, "Multi-select selection: Day, Month");
                    let day = control(&renderer, "Multi-select Day");
                    click(&mut renderer, day.bounds().unwrap());
                    bounds(&renderer, "Multi-select selection: Month");
                    toggle(
                        &mut renderer,
                        "Single-select segment icons",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Single-select selection indicator",
                        Toggled::True,
                        Toggled::False,
                    );
                    for level in 0..4 {
                        assert_eq!(
                            control(&renderer, &format!("Density {level} Day"))
                                .bounds()
                                .unwrap()
                                .height(),
                            48.0
                        );
                    }
                }
                25 => {
                    for (name, height) in [
                        ("Single line", 48.0),
                        ("Single line with action", 48.0),
                        ("Two lines", 68.0),
                        ("Two lines with action", 68.0),
                        ("Two lines with longer action", 120.0),
                    ] {
                        assert_eq!(
                            bounds(&renderer, name).height(),
                            height,
                            "snackbar configuration {name}"
                        );
                    }
                    let tokens = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(_, node)| {
                            node.role() == Role::Heading && node.label() == Some("Tokens and specs")
                        })
                        .unwrap()
                        .1
                        .bounds()
                        .unwrap();
                    assert!(
                        bounds(&renderer, "Snackbar anatomy").y1 < tokens.y0,
                        "the unheaded anatomy figure precedes the first Specs chapter"
                    );
                    let undo =
                        scoped_node(&renderer, "Queued snackbar surface", "Undo", Role::Button)
                            .unwrap();
                    click(&mut renderer, undo.bounds().unwrap());
                    renderer.render_frames(30);
                    bounds(&renderer, "Snackbar result: Action performed");
                    bounds(&renderer, "Snackbar visible: false");
                    let enqueue = control(&renderer, "Enqueue snackbar");
                    click(&mut renderer, enqueue.bounds().unwrap());
                    renderer.render_frames(30);
                    assert_eq!(bounds(&renderer, "Queued snackbar surface").height(), 48.0);
                    toggle(
                        &mut renderer,
                        "Next snackbar two lines",
                        Toggled::False,
                        Toggled::True,
                    );
                    let enqueue = control(&renderer, "Enqueue snackbar");
                    click(&mut renderer, enqueue.bounds().unwrap());
                    assert_eq!(
                        bounds(&renderer, "Queued snackbar surface").height(),
                        48.0,
                        "queued settings must not replace the active message"
                    );
                    let dismiss = control(&renderer, "Dismiss queued snackbar");
                    click(&mut renderer, dismiss.bounds().unwrap());
                    renderer.render_frames(30);
                    assert_eq!(bounds(&renderer, "Queued snackbar surface").height(), 68.0);
                    bounds(&renderer, "Snackbar result: Dismissed");
                    toggle(
                        &mut renderer,
                        "Stack snackbar action",
                        Toggled::False,
                        Toggled::True,
                    );
                    assert_eq!(bounds(&renderer, "Queued snackbar surface").height(), 120.0);
                    let undo =
                        scoped_node(&renderer, "Queued snackbar surface", "Undo", Role::Button)
                            .unwrap();
                    click(&mut renderer, undo.bounds().unwrap());
                    renderer.render_frames(30);
                    bounds(&renderer, "Snackbar visible: false");
                    let undo =
                        scoped_node(&renderer, "Single line with action", "Undo", Role::Button)
                            .unwrap();
                    click(&mut renderer, undo.bounds().unwrap());
                    let restore = control(&renderer, "Restore Single line with action");
                    click(&mut renderer, restore.bounds().unwrap());
                    scoped_node(&renderer, "Single line with action", "Undo", Role::Button)
                        .unwrap();
                }
                1 => {
                    for (name, height) in [
                        ("Small", 64.0),
                        ("Medium flexible", 112.0),
                        ("Large flexible", 120.0),
                    ] {
                        assert_eq!(
                            bounds(&renderer, &format!("Variant {name} bar")).height(),
                            height
                        );
                    }
                    assert_eq!(bounds(&renderer, "Configured app bar bar").height(), 136.0);
                    let small = control(&renderer, "Configured app bar Small");
                    click(&mut renderer, small.bounds().unwrap());
                    assert_eq!(bounds(&renderer, "Configured app bar bar").height(), 64.0);
                    let large = control(&renderer, "Configured app bar Large flexible");
                    click(&mut renderer, large.bounds().unwrap());
                    assert_eq!(bounds(&renderer, "Configured app bar bar").height(), 152.0);
                    toggle(
                        &mut renderer,
                        "Configured app bar subtitle",
                        Toggled::True,
                        Toggled::False,
                    );
                    assert_eq!(bounds(&renderer, "Configured app bar bar").height(), 120.0);
                    toggle(
                        &mut renderer,
                        "Configured app bar centered",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Configured app bar logo",
                        Toggled::False,
                        Toggled::True,
                    );
                    let brand = scoped_node(
                        &renderer,
                        "Configured app bar bar",
                        "Tessera brand",
                        Role::Image,
                    )
                    .expect("custom brand")
                    .bounds()
                    .unwrap();
                    assert_eq!(brand.width(), 32.0);
                    let bar = bounds(&renderer, "Configured app bar bar");
                    assert!(((brand.x0 + brand.x1) - (bar.x0 + bar.x1)).abs() < 0.01);
                    toggle(
                        &mut renderer,
                        "Configured app bar logo",
                        Toggled::True,
                        Toggled::False,
                    );
                    toggle(
                        &mut renderer,
                        "Configured app bar filled action",
                        Toggled::False,
                        Toggled::True,
                    );
                    let search = control(&renderer, "Configured app bar search");
                    click(&mut renderer, search.bounds().unwrap());
                    bounds(&renderer, "Configured app bar action: Search selected");
                    let content = bounds(&renderer, "Configured app bar scroll content");
                    renderer.pointer_scroll(
                        MOUSE_POINTER_ID,
                        PxPosition::from_f32_xy(
                            ((content.x0 + content.x1) * 0.5) as f32,
                            (content.y0 + 32.0) as f32,
                        ),
                        0.0,
                        -200.0,
                    );
                    renderer.render_frames(60);
                    assert_eq!(bounds(&renderer, "Configured app bar bar").height(), 64.0);
                    let reset = control(&renderer, "Reset Configured app bar scroll");
                    click(&mut renderer, reset.bounds().unwrap());
                    renderer.render_frames(60);
                    assert_eq!(bounds(&renderer, "Configured app bar bar").height(), 120.0);
                }
                24 => {
                    let container = bounds(&renderer, "Configured Filled container");
                    assert_eq!(container.height(), 56.0);
                    let label = scoped_node(
                        &renderer,
                        "Configured Filled field preview",
                        "Label",
                        Role::Label,
                    )
                    .unwrap()
                    .bounds()
                    .unwrap();
                    assert_eq!(
                        label.y0 - container.y0,
                        8.0,
                        "the filled floating label starts 8dp below the container top"
                    );
                    let clear = control(&renderer, "Clear Configured Filled");
                    click(&mut renderer, clear.bounds().unwrap());
                    let input = bounds(&renderer, "Configured Filled input");
                    click(&mut renderer, input);
                    renderer.inject_text("Ada");
                    renderer.render_frames(3);
                    bounds(&renderer, "Configured Filled value: Ada");
                    toggle(
                        &mut renderer,
                        "Configured Filled read only",
                        Toggled::False,
                        Toggled::True,
                    );
                    let input = bounds(&renderer, "Configured Filled input");
                    click(&mut renderer, input);
                    renderer.inject_text("ignored");
                    renderer.render_frames(3);
                    bounds(&renderer, "Configured Filled value: Ada");
                    toggle(
                        &mut renderer,
                        "Configured Filled read only",
                        Toggled::True,
                        Toggled::False,
                    );
                    toggle(
                        &mut renderer,
                        "Configured Filled icons",
                        Toggled::False,
                        Toggled::True,
                    );
                    let clear = control(&renderer, "Clear Configured Filled icon");
                    let container = bounds(&renderer, "Configured Filled container");
                    let leading = bounds(&renderer, "Configured Filled leading icon");
                    assert_eq!(leading.x0 - container.x0, 12.0);
                    assert_eq!(leading.y0 - container.y0, 16.0);
                    assert_eq!(leading.width(), 24.0);
                    assert_eq!(clear.bounds().unwrap().height(), 48.0);
                    assert_eq!(
                        (clear.bounds().unwrap().y0 + clear.bounds().unwrap().y1) * 0.5
                            - container.y0,
                        28.0,
                    );
                    click(&mut renderer, clear.bounds().unwrap());
                    bounds(&renderer, "Configured Filled value: ");
                    toggle(
                        &mut renderer,
                        "Configured Outlined error",
                        Toggled::False,
                        Toggled::True,
                    );
                    let clear = control(&renderer, "Clear Configured Outlined");
                    click(&mut renderer, clear.bounds().unwrap());
                    let input = bounds(&renderer, "Configured Outlined input");
                    click(&mut renderer, input);
                    renderer.inject_text("Paris");
                    renderer.render_frames(3);
                    bounds(&renderer, "Configured Outlined value: Paris");
                    toggle(
                        &mut renderer,
                        "Configured Outlined secure",
                        Toggled::False,
                        Toggled::True,
                    );
                    bounds(&renderer, "Configured Outlined value: •••••");
                    toggle(
                        &mut renderer,
                        "Configured Outlined multiline",
                        Toggled::False,
                        Toggled::True,
                    );
                    assert_eq!(
                        bounds(&renderer, "Configured Outlined container").height(),
                        96.0
                    );
                }
                4 => {
                    let plain = scoped_node(
                        &renderer,
                        "Configured Plain tooltip preview",
                        "Add to favorites",
                        Role::Tooltip,
                    )
                    .unwrap()
                    .bounds()
                    .unwrap();
                    assert_eq!(plain.height(), 24.0);
                    assert!(plain.width() <= 200.0);
                    let dismiss = control(&renderer, "Dismiss Configured Plain");
                    click(&mut renderer, dismiss.bounds().unwrap());
                    bounds(&renderer, "Configured Plain visibility: Hidden");
                    let show = control(&renderer, "Show Configured Plain");
                    click(&mut renderer, show.bounds().unwrap());
                    bounds(&renderer, "Configured Plain visibility: Shown");
                    toggle(
                        &mut renderer,
                        "Configured Plain long message",
                        Toggled::False,
                        Toggled::True,
                    );
                    let popup=scoped_node(&renderer,"Configured Plain tooltip preview","Add this item to your collection so you can find it again later. You can change the collection at any time.",Role::Tooltip).unwrap().bounds().unwrap();
                    assert!(popup.height() > 24.0 && popup.width() <= 200.0);
                    let rich = scoped_node(
                        &renderer,
                        "Configured Rich tooltip preview",
                        "Rich tooltip",
                        Role::Dialog,
                    )
                    .unwrap()
                    .bounds()
                    .unwrap();
                    assert!(rich.width() <= 320.0);
                    let action = scoped_node(
                        &renderer,
                        "Configured Rich tooltip preview",
                        "Learn more",
                        Role::Button,
                    )
                    .unwrap();
                    click(&mut renderer, action.bounds().unwrap());
                    renderer.render_frames(5);
                    bounds(&renderer, "Configured Rich actions: 1");
                    bounds(&renderer, "Configured Rich visibility: Hidden");
                    let show = control(&renderer, "Show Configured Rich");
                    click(&mut renderer, show.bounds().unwrap());
                    bounds(&renderer, "Configured Rich visibility: Shown");
                }
                31 => {
                    renderer.render_frames(5);
                    let horizontal = scoped_node(
                        &renderer,
                        "Configured navigation preview",
                        "Inbox",
                        Role::Tab,
                    )
                    .unwrap()
                    .bounds()
                    .unwrap();
                    let layout = control(&renderer, "Cycle navigation item layout");
                    click(&mut renderer, layout.bounds().unwrap());
                    renderer.render_frames(30);
                    let vertical = scoped_node(
                        &renderer,
                        "Configured navigation preview",
                        "Inbox",
                        Role::Tab,
                    )
                    .unwrap()
                    .bounds()
                    .unwrap();
                    assert!(
                        vertical.width() > horizontal.width(),
                        "vertical {vertical:?}, horizontal {horizontal:?}"
                    );
                    assert_eq!(vertical.height(), 64.0);
                    click(&mut renderer, vertical);
                    bounds(&renderer, "Configured destination: Inbox");
                    for _ in 0..2 {
                        let count = control(&renderer, "Cycle navigation destination count");
                        click(&mut renderer, count.bounds().unwrap());
                    }
                    let settings = scoped_node(
                        &renderer,
                        "Configured navigation preview",
                        "Settings",
                        Role::Tab,
                    )
                    .unwrap();
                    click(&mut renderer, settings.bounds().unwrap());
                    bounds(&renderer, "Configured destination: Settings");
                    let count = control(&renderer, "Cycle navigation destination count");
                    click(&mut renderer, count.bounds().unwrap());
                    bounds(&renderer, "Configured destination: Saved");
                    assert!(
                        scoped_node(
                            &renderer,
                            "Configured navigation preview",
                            "Settings",
                            Role::Tab
                        )
                        .is_none()
                    );
                }
                32 => {
                    let collapsed = scoped_node(
                        &renderer,
                        "Configured navigation preview",
                        "Inbox",
                        Role::Tab,
                    )
                    .unwrap()
                    .bounds()
                    .unwrap();
                    click(&mut renderer, collapsed);
                    bounds(&renderer, "Configured destination: Inbox");
                    toggle(
                        &mut renderer,
                        "Expand navigation rail",
                        Toggled::False,
                        Toggled::True,
                    );
                    renderer.render_frames(30);
                    let expanded = scoped_node(
                        &renderer,
                        "Configured navigation preview",
                        "Inbox",
                        Role::Tab,
                    )
                    .unwrap()
                    .bounds()
                    .unwrap();
                    assert_eq!(collapsed.width(), 96.0);
                    assert_eq!(expanded.width(), 220.0);
                    toggle(
                        &mut renderer,
                        "Hide collapsed rail",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Expand navigation rail",
                        Toggled::True,
                        Toggled::False,
                    );
                    renderer.render_frames(30);
                    assert!(
                        scoped_node(
                            &renderer,
                            "Configured navigation preview",
                            "Inbox",
                            Role::Tab
                        )
                        .is_none()
                    );
                    bounds(&renderer, "Configured destination: Inbox");
                    toggle(
                        &mut renderer,
                        "Expand navigation rail",
                        Toggled::False,
                        Toggled::True,
                    );
                    renderer.render_frames(30);
                    let compose = control(&renderer, "Configured compose");
                    click(&mut renderer, compose.bounds().unwrap());
                    bounds(&renderer, "Configured compositions: 1");
                }
                21 => {
                    let flights =
                        scoped_node(&renderer, "Primary tabs preview", "Flights", Role::Tab)
                            .unwrap();
                    assert_eq!(flights.bounds().unwrap().height(), 48.0);
                    let hotels =
                        scoped_node(&renderer, "Primary tabs preview", "Hotels", Role::Tab)
                            .unwrap();
                    click(&mut renderer, hotels.bounds().unwrap());
                    bounds(&renderer, "Primary content: Hotels");
                    bounds(&renderer, "Primary activations: 1");
                    toggle(
                        &mut renderer,
                        "Show primary tab icons",
                        Toggled::False,
                        Toggled::True,
                    );
                    assert_eq!(
                        scoped_node(&renderer, "Primary tabs preview", "Hotels", Role::Tab)
                            .unwrap()
                            .bounds()
                            .unwrap()
                            .height(),
                        64.0
                    );
                    toggle(
                        &mut renderer,
                        "Primary tab badge",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Primary disable Hotels",
                        Toggled::False,
                        Toggled::True,
                    );
                    let disabled = scoped_node(
                        &renderer,
                        "Primary tabs preview",
                        "Hotels, 3 new items",
                        Role::Tab,
                    )
                    .unwrap();
                    assert!(disabled.is_disabled());
                    click(&mut renderer, disabled.bounds().unwrap());
                    bounds(&renderer, "Primary activations: 1");
                    toggle(
                        &mut renderer,
                        "Primary disable Hotels",
                        Toggled::True,
                        Toggled::False,
                    );
                    toggle(
                        &mut renderer,
                        "Primary scrollable tabs",
                        Toggled::False,
                        Toggled::True,
                    );
                    let last = control(&renderer, "Primary select final tab");
                    click(&mut renderer, last.bounds().unwrap());
                    bounds(&renderer, "Primary content: Ferries");
                    renderer.resize(390, 8000);
                    renderer.render_frames(10);
                    let ferries =
                        scoped_node(&renderer, "Primary tabs preview", "Ferries", Role::Tab)
                            .unwrap()
                            .bounds()
                            .unwrap();
                    assert!(ferries.x0 >= 0.0 && ferries.x1 <= 390.0);
                    renderer.resize(900, 8000);
                    renderer.render_frames(3);
                    let trains =
                        scoped_node(&renderer, "Secondary tabs preview", "Trains", Role::Tab)
                            .unwrap();
                    click(&mut renderer, trains.bounds().unwrap());
                    bounds(&renderer, "Secondary content: Trains");
                }
                6 => {
                    let indicator = scoped_node(
                        &renderer,
                        "Loading indicator specifications",
                        "Configured loading",
                        Role::ProgressIndicator,
                    )
                    .expect("configured loading indicator");
                    assert_eq!(indicator.bounds().unwrap().width(), 48.0);
                    assert_eq!(indicator.bounds().unwrap().height(), 48.0);
                    assert_eq!(indicator.numeric_value(), None);
                    toggle(
                        &mut renderer,
                        "Contained loading",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Pause loading animation",
                        Toggled::False,
                        Toggled::True,
                    );
                    let contained = scoped_node(
                        &renderer,
                        "Loading indicator specifications",
                        "Configured loading",
                        Role::ProgressIndicator,
                    )
                    .unwrap();
                    assert_eq!(contained.bounds(), indicator.bounds());
                }
                7 => {
                    for label in ["Configured linear progress", "Configured circular progress"] {
                        let indicator = scoped_node(
                            &renderer,
                            "Progress indicators specifications",
                            label,
                            Role::ProgressIndicator,
                        )
                        .unwrap();
                        assert_eq!(indicator.numeric_value(), Some(0.5));
                    }
                    let advance = control(&renderer, "Advance progress");
                    click(&mut renderer, advance.bounds().unwrap());
                    bounds(&renderer, "Task progress: 75%");
                    toggle(
                        &mut renderer,
                        "Wavy progress",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Thick progress",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Pause wave animation",
                        Toggled::False,
                        Toggled::True,
                    );
                    let circular = scoped_node(
                        &renderer,
                        "Progress indicators specifications",
                        "Configured circular progress",
                        Role::ProgressIndicator,
                    )
                    .unwrap();
                    assert_eq!(circular.bounds().unwrap().height(), 52.0);
                    assert_eq!(circular.numeric_value(), Some(0.75));
                    toggle(
                        &mut renderer,
                        "Indeterminate progress",
                        Toggled::False,
                        Toggled::True,
                    );
                    for label in ["Configured linear progress", "Configured circular progress"] {
                        assert_eq!(
                            scoped_node(
                                &renderer,
                                "Progress indicators specifications",
                                label,
                                Role::ProgressIndicator
                            )
                            .unwrap()
                            .numeric_value(),
                            None
                        );
                    }
                    bounds(&renderer, "Task progress: indeterminate");
                    toggle(
                        &mut renderer,
                        "Indeterminate progress",
                        Toggled::True,
                        Toggled::False,
                    );
                    let reset = control(&renderer, "Reset progress");
                    click(&mut renderer, reset.bounds().unwrap());
                    bounds(&renderer, "Task progress: 0%");
                }
                12 => {
                    assert_eq!(bounds(&renderer, "Small badge specimen").width(), 6.0);
                    assert_eq!(bounds(&renderer, "Small badge specimen").height(), 6.0);
                    assert_eq!(bounds(&renderer, "Large badge specimen").width(), 16.0);
                    assert_eq!(bounds(&renderer, "Large badge specimen").height(), 16.0);
                    assert_eq!(bounds(&renderer, "Maximum badge specimen").height(), 16.0);
                    let inbox =
                        scoped_node(&renderer, "Configured badge navigation", "Inbox", Role::Tab)
                            .expect("badged destination");
                    click(&mut renderer, inbox.bounds().unwrap());
                    bounds(&renderer, "Configured destination: Inbox");
                    let add = control(&renderer, "Add unread item");
                    click(&mut renderer, add.bounds().unwrap());
                    bounds(&renderer, "Unread count: 8");
                    toggle(
                        &mut renderer,
                        "Use badge navigation rail",
                        Toggled::False,
                        Toggled::True,
                    );
                    let home =
                        scoped_node(&renderer, "Configured badge navigation", "Home", Role::Tab)
                            .expect("badged rail destination");
                    click(&mut renderer, home.bounds().unwrap());
                    bounds(&renderer, "Configured destination: Home");
                    let clear = control(&renderer, "Clear unread items");
                    click(&mut renderer, clear.bounds().unwrap());
                    bounds(&renderer, "Unread count: 0");
                }
                18 => {
                    let one = scoped_node(&renderer, "Standard group", "One", Role::CheckBox)
                        .expect("standard choice");
                    click(&mut renderer, one.bounds().unwrap());
                    bounds(&renderer, "Standard group selection: 1");
                    let two = scoped_node(&renderer, "Standard group", "Two", Role::CheckBox)
                        .expect("standard second choice");
                    click(&mut renderer, two.bounds().unwrap());
                    bounds(&renderer, "Standard group selection: 2");
                    toggle(
                        &mut renderer,
                        "Group multiple selection",
                        Toggled::False,
                        Toggled::True,
                    );
                    for name in ["One", "Two"] {
                        let item = scoped_node(&renderer, "Configured group", name, Role::CheckBox)
                            .expect("configured choice");
                        click(&mut renderer, item.bounds().unwrap());
                    }
                    bounds(&renderer, "Configured group selection: 1, 2");
                    toggle(
                        &mut renderer,
                        "Group required selection",
                        Toggled::False,
                        Toggled::True,
                    );
                    for name in ["Two", "One"] {
                        let item = scoped_node(&renderer, "Configured group", name, Role::CheckBox)
                            .expect("required choice");
                        click(&mut renderer, item.bounds().unwrap());
                    }
                    bounds(&renderer, "Configured group selection: 1");
                    toggle(
                        &mut renderer,
                        "Group disabled",
                        Toggled::False,
                        Toggled::True,
                    );
                    let item = scoped_node(&renderer, "Configured group", "Two", Role::CheckBox)
                        .expect("disabled choice");
                    assert!(item.is_disabled());
                    click(&mut renderer, item.bounds().unwrap());
                    bounds(&renderer, "Configured group selection: 1");
                }
                22 => {
                    let send = control(&renderer, "Variant send");
                    click(&mut renderer, send.bounds().unwrap());
                    bounds(&renderer, "Variant actions: 1");
                    let more = control(&renderer, "Variant options");
                    click(&mut renderer, more.bounds().unwrap());
                    renderer.render_frames(30);
                    assert_eq!(
                        control(&renderer, "Variant options").is_expanded(),
                        Some(true)
                    );
                    let item = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(_, node)| {
                            node.role() == Role::MenuItem && node.label() == Some("Schedule send")
                        })
                        .expect("anchored send option")
                        .1;
                    let popup = item.bounds().unwrap();
                    assert!(popup.width() <= 280.0 && popup.height() <= 64.0);
                    click(&mut renderer, item.bounds().unwrap());
                    renderer.render_frames(30);
                    bounds(&renderer, "Scheduled send selected");
                    assert_eq!(
                        control(&renderer, "Variant options").is_expanded(),
                        Some(false)
                    );
                    toggle(
                        &mut renderer,
                        "Disable split action",
                        Toggled::False,
                        Toggled::True,
                    );
                    let disabled = control(&renderer, "State send");
                    assert!(disabled.is_disabled());
                    click(&mut renderer, disabled.bounds().unwrap());
                    bounds(&renderer, "State actions: 0");
                }
                8 => {
                    let open = control(&renderer, "Open Variant actions");
                    click(&mut renderer, open.bounds().unwrap());
                    renderer.render_frames(60);
                    let item =
                        scoped_node(&renderer, "Variant FAB menu", "New folder", Role::MenuItem)
                            .expect("creation action");
                    click(&mut renderer, item.bounds().unwrap());
                    renderer.render_frames(60);
                    bounds(&renderer, "Variant: Folder created");
                    let open = control(&renderer, "Open Variant actions");
                    click(&mut renderer, open.bounds().unwrap());
                    renderer.render_frames(60);
                    let close = control(&renderer, "Close Variant actions");
                    click(&mut renderer, close.bounds().unwrap());
                    renderer.render_frames(60);
                    control(&renderer, "Open Variant actions");
                }
                9 | 19 => {
                    for (name, height) in if component.demo == 9 {
                        [("Small", 56.0), ("Medium", 80.0), ("Large", 96.0)]
                    } else {
                        [("Regular", 56.0), ("Medium", 80.0), ("Large", 96.0)]
                    } {
                        let label = format!("{name} floating action");
                        let action = control(&renderer, &label);
                        assert_eq!(action.bounds().unwrap().height(), height);
                        if component.demo == 19 {
                            assert_eq!(action.bounds().unwrap().width(), height);
                        }
                        click(&mut renderer, action.bounds().unwrap());
                        bounds(&renderer, &format!("{label}: 1"));
                    }
                    if component.demo == 9 {
                        let expanded = control(&renderer, "Compose action").bounds().unwrap();
                        toggle(
                            &mut renderer,
                            "Toggle Compose label",
                            Toggled::True,
                            Toggled::False,
                        );
                        renderer.render_frames(90);
                        let collapsed = control(&renderer, "Compose action").bounds().unwrap();
                        assert_eq!(collapsed.width(), 56.0);
                        assert!(expanded.width() > collapsed.width());
                        toggle(
                            &mut renderer,
                            "Toggle Compose label",
                            Toggled::False,
                            Toggled::True,
                        );
                        renderer.render_frames(90);
                        let restored = control(&renderer, "Compose action").bounds().unwrap();
                        assert_eq!(restored.width(), expanded.width());
                    }
                }
                28 => {
                    let day = scoped_node(
                        &renderer,
                        "Calendar date preview",
                        "July 20, 2026",
                        Role::Button,
                    )
                    .expect("calendar day");
                    click(&mut renderer, day.bounds().unwrap());
                    bounds(&renderer, "Calendar date: 2026-07-20");
                    let confirm = control(&renderer, "Calendar confirm date");
                    click(&mut renderer, confirm.bounds().unwrap());
                    bounds(&renderer, "Confirmed date: 2026-07-20");
                    toggle(
                        &mut renderer,
                        "Calendar range selection",
                        Toggled::False,
                        Toggled::True,
                    );
                    for day in ["July 18, 2026", "July 22, 2026"] {
                        let day =
                            scoped_node(&renderer, "Calendar date preview", day, Role::Button)
                                .expect("range day");
                        click(&mut renderer, day.bounds().unwrap());
                    }
                    bounds(&renderer, "Calendar range: 2026-07-18 – 2026-07-22");
                    let confirm = control(&renderer, "Calendar confirm date");
                    click(&mut renderer, confirm.bounds().unwrap());
                    bounds(&renderer, "Confirmed range: 2026-07-18 – 2026-07-22");
                    toggle(
                        &mut renderer,
                        "Input range selection",
                        Toggled::False,
                        Toggled::True,
                    );
                    for (label, value) in [("Start date", "07/10/2026"), ("End date", "07/16/2026")]
                    {
                        let input =
                            scoped_node(&renderer, "Input date preview", label, Role::TextInput)
                                .expect("range input field");
                        assert_eq!(input.value(), Some(value));
                    }
                    let forbidden = scoped_node(
                        &renderer,
                        "Restricted date preview",
                        "July 19, 2026",
                        Role::Button,
                    )
                    .expect("disabled day");
                    assert!(forbidden.is_disabled());
                    click(&mut renderer, forbidden.bounds().unwrap());
                    bounds(&renderer, "Restricted date: 2026-07-18");
                    let allowed = scoped_node(
                        &renderer,
                        "Restricted date preview",
                        "July 20, 2026",
                        Role::Button,
                    )
                    .expect("enabled day");
                    click(&mut renderer, allowed.bounds().unwrap());
                    bounds(&renderer, "Restricted date: 2026-07-20");
                    let open = control(&renderer, "Open Calendar date dialog");
                    click(&mut renderer, open.bounds().unwrap());
                    renderer.render_frames(24);
                    renderer.resize(900, 1200);
                    renderer.render_frames(3);
                    let mode = scoped_node(
                        &renderer,
                        "Modal date preview",
                        "Switch to date range input",
                        Role::Button,
                    )
                    .expect("modal input mode toggle");
                    click(&mut renderer, mode.bounds().unwrap());
                    for label in ["Start date", "End date"] {
                        let input =
                            scoped_node(&renderer, "Modal date preview", label, Role::TextInput)
                                .expect("modal date input");
                        assert!(input.bounds().unwrap().height() <= 56.0);
                    }
                    let dialog = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .iter()
                        .find(|(_, node)| node.role() == Role::Dialog)
                        .and_then(|(_, node)| node.bounds())
                        .expect("date dialog bounds");
                    assert!(dialog.height() < 400.0);
                    let cancel = control(&renderer, "Modal cancel date");
                    let cancel_bounds = cancel.bounds().unwrap();
                    assert!(cancel_bounds.y0 >= dialog.y0 && cancel_bounds.y1 <= dialog.y1);
                    click(&mut renderer, cancel.bounds().unwrap());
                    renderer.render_frames(24);
                    assert!(
                        !renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .iter()
                            .any(|(_, node)| node.role() == Role::Dialog)
                    );
                    renderer.resize(900, 8000);
                    renderer.render_frames(3);
                }
                30 => {
                    let vertical = bounds(&renderer, "Anatomy time preview");
                    toggle(
                        &mut renderer,
                        "Anatomy horizontal layout",
                        Toggled::False,
                        Toggled::True,
                    );
                    let horizontal = bounds(&renderer, "Anatomy time preview");
                    assert!(horizontal.width() > vertical.width());
                    assert!(horizontal.height() < vertical.height());
                    renderer.resize(390, 8000);
                    renderer.render_frames(3);
                    let compact = bounds(&renderer, "Anatomy time preview");
                    assert!(compact.width() < 400.0 && compact.height() > horizontal.height());
                    renderer.resize(900, 8000);
                    renderer.render_frames(3);
                    toggle(
                        &mut renderer,
                        "Anatomy 24-hour format",
                        Toggled::False,
                        Toggled::True,
                    );
                    assert!(
                        scoped_node(&renderer, "Anatomy time preview", "AM/PM", Role::RadioGroup)
                            .is_none()
                    );
                    let hour = scoped_node(
                        &renderer,
                        "Anatomy time preview",
                        "Hour 10 of 24",
                        Role::Button,
                    )
                    .expect("24-hour dial value");
                    click(&mut renderer, hour.bounds().unwrap());
                    bounds(&renderer, "Anatomy time: 10:41");
                    let confirm = control(&renderer, "Anatomy confirm time");
                    click(&mut renderer, confirm.bounds().unwrap());
                    bounds(&renderer, "Confirmed 10:41");
                    toggle(
                        &mut renderer,
                        "Anatomy keyboard input",
                        Toggled::False,
                        Toggled::True,
                    );
                    assert!(
                        scoped_node(
                            &renderer,
                            "Anatomy time preview",
                            "Hour 10 of 24",
                            Role::Button
                        )
                        .is_none()
                    );
                    for label in ["Hour", "Minute"] {
                        scoped_node(&renderer, "Anatomy time preview", label, Role::TextInput)
                            .expect("time input field");
                    }
                    let open = control(&renderer, "Open time picker dialog");
                    click(&mut renderer, open.bounds().unwrap());
                    renderer.render_frames(24);
                    let cancel = control(&renderer, "Modal cancel time");
                    click(&mut renderer, cancel.bounds().unwrap());
                    renderer.render_frames(24);
                    assert!(
                        !renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .iter()
                            .any(|(_, node)| node.role() == Role::Dialog)
                    );
                }
                14 => {
                    let full = bounds(&renderer, "Full width measurement");
                    let inset = bounds(&renderer, "Inset measurement");
                    let middle = bounds(&renderer, "Middle inset measurement");
                    assert_eq!(full.height(), 1.0);
                    assert_eq!(inset.height(), 1.0);
                    assert_eq!(middle.height(), 1.0);
                    assert_eq!(inset.x0 - full.x0, 16.0);
                    assert_eq!(inset.x1, full.x1);
                    assert_eq!(middle.x0 - full.x0, 16.0);
                    assert_eq!(full.x1 - middle.x1, 16.0);
                }
                15 => {
                    toggle(
                        &mut renderer,
                        "Expressive list: Notes",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Single select: Projects",
                        Toggled::False,
                        Toggled::True,
                    );
                    assert_eq!(
                        control(&renderer, "Single select: Notes").toggled(),
                        Some(Toggled::False)
                    );
                    toggle(
                        &mut renderer,
                        "Multi select: Notes",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Multi select: Projects",
                        Toggled::False,
                        Toggled::True,
                    );
                    assert_eq!(
                        control(&renderer, "Multi select: Notes").toggled(),
                        Some(Toggled::True)
                    );
                    let secondary = control(&renderer, "Multi action: Notes more actions");
                    click(&mut renderer, secondary.bounds().unwrap());
                    bounds(&renderer, "Multi action secondary actions: 1");
                    bounds(&renderer, "Multi action primary actions: 0");
                    let primary = control(&renderer, "Multi action: Notes");
                    click(&mut renderer, primary.bounds().unwrap());
                    bounds(&renderer, "Multi action secondary actions: 1");
                    bounds(&renderer, "Multi action primary actions: 1");
                    assert!(
                        !renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .iter()
                            .any(|(_, node)| node.label() == Some("Expanded archive folder"))
                    );
                    let expand = control(&renderer, "Expand related folders");
                    click(&mut renderer, expand.bounds().unwrap());
                    bounds(&renderer, "Expanded archive folder");
                    let expand = control(&renderer, "Expand related folders");
                    click(&mut renderer, expand.bounds().unwrap());
                    assert!(
                        !renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .iter()
                            .any(|(_, node)| node.label() == Some("Expanded archive folder"))
                    );
                    for (label, height) in [
                        ("One-line list measurement", 56.0),
                        ("Two-line list measurement", 72.0),
                        ("Three-line list measurement", 88.0),
                    ] {
                        assert_eq!(bounds(&renderer, label).height(), height, "{label}");
                    }
                }
                16 => {
                    let card = control(&renderer, "Elevated card");
                    bounds(&renderer, "Elevated card activations: 0");
                    click(&mut renderer, card.bounds().unwrap());
                    bounds(&renderer, "Elevated card activations: 1");
                    let disabled = control(&renderer, "Elevated disabled");
                    click(&mut renderer, disabled.bounds().unwrap());
                    bounds(&renderer, "Elevated disabled activations: 0");
                    let first = control(&renderer, "First card").bounds().unwrap();
                    let second = control(&renderer, "Second card").bounds().unwrap();
                    assert_eq!(second.x0 - first.x1, 8.0);
                }
                3 => {
                    toggle(
                        &mut renderer,
                        "Toggle favorite",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Toggle favorite",
                        Toggled::True,
                        Toggled::False,
                    );
                    let action = control(&renderer, "Default icon action");
                    assert_eq!(action.description(), Some("Activations: 0"));
                    click(&mut renderer, action.bounds().unwrap());
                    assert_eq!(
                        control(&renderer, "Default icon action").description(),
                        Some("Activations: 1")
                    );
                }
                17 => {
                    toggle(
                        &mut renderer,
                        "Filter example",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Input example",
                        Toggled::False,
                        Toggled::True,
                    );
                    let selector = control(&renderer, "Filter chip tokens");
                    click(&mut renderer, selector.bounds().unwrap());
                    assert!(
                        renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .iter()
                            .any(|(_, node)| node.role() == Role::Cell
                                && node.label() == Some("Secondary container"))
                    );
                }
                23 => {
                    toggle(
                        &mut renderer,
                        "Notification preference: Important only",
                        Toggled::False,
                        Toggled::True,
                    );
                    assert_eq!(
                        control(&renderer, "Notification preference: All notifications").toggled(),
                        Some(Toggled::False)
                    );
                }
                36 => {
                    toggle(
                        &mut renderer,
                        "Toggle button",
                        Toggled::False,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Toggle button",
                        Toggled::True,
                        Toggled::False,
                    );
                    let initial = renderer.accessibility_snapshot().unwrap();
                    assert!(
                        initial
                            .nodes
                            .iter()
                            .any(|(_, node)| node.role() == Role::Cell
                                && node.label() == Some("On primary"))
                    );
                    let selector = control(&renderer, "Outlined color tokens")
                        .bounds()
                        .unwrap();
                    click(&mut renderer, selector);
                    let changed = renderer.accessibility_snapshot().unwrap();
                    assert!(
                        changed
                            .nodes
                            .iter()
                            .any(|(_, node)| node.role() == Role::Cell
                                && node.label() == Some("On surface variant"))
                    );
                    assert!(
                        !changed
                            .nodes
                            .iter()
                            .any(|(_, node)| node.role() == Role::Cell
                                && node.label() == Some("On primary"))
                    );
                }
                37 => {
                    toggle(
                        &mut renderer,
                        "Microphone access",
                        Toggled::True,
                        Toggled::False,
                    );
                    toggle(
                        &mut renderer,
                        "Indeterminate checkbox",
                        Toggled::Mixed,
                        Toggled::True,
                    );
                    toggle(
                        &mut renderer,
                        "Indeterminate checkbox",
                        Toggled::True,
                        Toggled::False,
                    );
                }
                38 => {
                    toggle(&mut renderer, "Wi-Fi", Toggled::True, Toggled::False);
                    toggle(
                        &mut renderer,
                        "Icons in both states: unselected",
                        Toggled::False,
                        Toggled::True,
                    );
                }
                _ => {}
            }
            if matches!(
                component.demo,
                1 | 3
                    | 4
                    | 25
                    | 20
                    | 24
                    | 5
                    | 6
                    | 7
                    | 8
                    | 9
                    | 12
                    | 14
                    | 15
                    | 16
                    | 17
                    | 18
                    | 19
                    | 21
                    | 22
                    | 23
                    | 28
                    | 30
                    | 31
                    | 32
                    | 36
                    | 37
                    | 38
            ) {
                renderer.resize(900, 1200);
                renderer.render_frames(3);
                for section in [
                    if matches!(
                        component.demo,
                        1 | 3 | 5 | 6 | 7 | 8 | 9 | 15 | 18 | 19 | 22 | 31 | 32 | 36
                    ) {
                        "Variants"
                    } else if matches!(component.demo, 14 | 20 | 21 | 25) {
                        "Tokens and specs"
                    } else {
                        "Tokens & specs"
                    },
                    if component.demo == 25 {
                        "Configurations"
                    } else if component.demo == 4 {
                        "Rich tooltip"
                    } else if component.demo == 24 {
                        "Outlined text field"
                    } else if component.demo == 20 {
                        "Density"
                    } else if component.demo == 32 {
                        "Common layouts"
                    } else if component.demo == 12 {
                        "Configuration"
                    } else if component.demo == 18 {
                        "Density"
                    } else if component.demo == 17 {
                        "Suggestion chip"
                    } else if component.demo == 28 {
                        "Element states"
                    } else {
                        "Measurements"
                    },
                ] {
                    {
                        let back = bounds(&renderer, "Back to components");
                        click(&mut renderer, back);
                        renderer.resize(900, 8000);
                        renderer.render_frames(3);
                        let card = bounds(
                            &renderer,
                            &format!("Open {} specifications", component.title),
                        );
                        click(&mut renderer, card);
                        renderer.resize(900, 1200);
                        renderer.render_frames(3);
                    }
                    let link = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(_, node)| {
                            node.role() == Role::Link && node.label() == Some(section)
                        })
                        .expect("section link")
                        .1
                        .bounds()
                        .unwrap();
                    click(&mut renderer, link);
                    let heading = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(_, node)| {
                            node.role() == Role::Heading && node.label() == Some(section)
                        })
                        .expect("section heading")
                        .1
                        .bounds()
                        .unwrap();
                    let back = bounds(&renderer, "Back to components");
                    assert!(
                        heading.y0 >= back.y1 && heading.y1 <= 1200.0,
                        "section link must reveal {section} below the back button for {}: {heading:?}; back: {back:?}",
                        component.title
                    );
                    if component.demo == 12 && section == "Configuration" {
                        let add = control(&renderer, "Add unread item");
                        click(&mut renderer, add.bounds().unwrap());
                        bounds(&renderer, "Unread count: 8");
                        renderer.render_frames(5);
                        let after = renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .into_iter()
                            .find(|(_, node)| {
                                node.role() == Role::Heading && node.label() == Some(section)
                            })
                            .unwrap()
                            .1
                            .bounds()
                            .unwrap();
                        assert_eq!(
                            after, heading,
                            "activating a control must not reclamp a section jump"
                        );
                    }
                }
                renderer.resize(390, 900);
                renderer.render_frames(3);
                let title = renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .into_iter()
                    .find(|(_, node)| {
                        node.role() == Role::Heading
                            && node.level() == Some(1)
                            && node.label()
                                == Some(if component.demo == 19 {
                                    "Floating action buttons (FABs)"
                                } else {
                                    component.title
                                })
                    })
                    .expect("article title")
                    .1
                    .bounds()
                    .unwrap();
                assert!(
                    title.x0 >= 0.0 && title.x1 <= 390.0,
                    "compact title must stay within the viewport: {title:?}"
                );
                if component.demo == 28 {
                    let preview = bounds(&renderer, "Calendar date preview");
                    assert_eq!(preview.width(), 360.0);
                    assert!(preview.x0 >= 0.0 && preview.x1 <= 390.0);
                } else if component.demo == 30 {
                    let preview = bounds(&renderer, "Anatomy time preview");
                    assert!(preview.width() >= 304.0 && preview.width() < 400.0);
                    assert!(preview.x0 >= 0.0 && preview.x1 <= 390.0);
                } else if component.demo == 9 || component.demo == 19 {
                    for name in if component.demo == 9 {
                        ["Small", "Medium", "Large"]
                    } else {
                        ["Regular", "Medium", "Large"]
                    } {
                        let preview = control(&renderer, &format!("{name} floating action"))
                            .bounds()
                            .unwrap();
                        assert!(preview.x0 >= 0.0 && preview.x1 <= 390.0);
                    }
                } else if component.demo == 22 {
                    for label in ["Size XL send", "Size XL options"] {
                        let preview = control(&renderer, label).bounds().unwrap();
                        assert!(preview.x0 >= 0.0 && preview.x1 <= 390.0);
                    }
                } else if component.demo == 18 {
                    for label in ["One", "Two", "Three"] {
                        let preview =
                            scoped_node(&renderer, "Connected group", label, Role::CheckBox)
                                .unwrap()
                                .bounds()
                                .unwrap();
                        assert!(preview.x0 >= 0.0 && preview.x1 <= 390.0);
                    }
                }
                renderer.resize(900, 8000);
                renderer.render_frames(3);
            }
            let back = bounds(&renderer, "Back to components");
            click(&mut renderer, back);
            bounds(
                &renderer,
                &format!("Open {} specifications", component.title),
            );
        }
        renderer.resize(390, 900);
        renderer.render_frames(3);
        let first = bounds(&renderer, "Open Button groups specifications");
        let second = bounds(&renderer, "Open Buttons specifications");
        assert!(first.x0 >= 0.0 && first.x1 <= 390.0);
        assert!(
            second.y0 >= first.y1,
            "compact directory must use one column"
        );
    }
}
