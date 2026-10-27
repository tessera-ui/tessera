//! Keyboard navigation for specification article sections.
//!
//! ## Usage
//!
//! Move focus within article section tabs and activate a section explicitly.

use tessera_components::{
    interaction_state::InteractionState,
    modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs},
};
use tessera_ui::{
    Callback, FocusManager, FocusProperties, FocusRequester, FocusState, KeyboardInput,
    KeyboardInputModifierNode, Modifier, State,
    accesskit::Role,
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    winit::{
        event::ElementState,
        keyboard::{Key, ModifiersState, NamedKey},
    },
};

pub(super) const ARTICLE_TAB_TITLES: [&str; 4] =
    ["Overview", "Specs", "Guidelines", "Accessibility"];

pub(super) struct ArticleNavigationState {
    requesters: [FocusRequester; 4],
    pub(super) available: [bool; 4],
    selected: usize,
    focused: Option<usize>,
}

impl Default for ArticleNavigationState {
    fn default() -> Self {
        Self {
            requesters: std::array::from_fn(|_| FocusRequester::new()),
            available: [true; 4],
            selected: 1,
            focused: None,
        }
    }
}

pub(super) fn sync_article_navigation(
    navigation: State<ArticleNavigationState>,
    available: [bool; 4],
    selected: &str,
) {
    let selected = ARTICLE_TAB_TITLES
        .iter()
        .position(|title| *title == selected)
        .filter(|index| available[*index])
        .or_else(|| available.iter().position(|value| *value))
        .unwrap_or(0);
    if navigation.with(|value| value.available != available || value.selected != selected) {
        navigation.with_mut(|value| {
            value.available = available;
            value.selected = selected;
            if value.focused.is_some_and(|index| !available[index]) {
                value.focused = None;
            }
        });
    }
}

pub(super) fn article_tablist_modifier(
    modifier: Modifier,
    navigation: State<ArticleNavigationState>,
) -> Modifier {
    modifier
        .focus_group()
        .on_focus_changed(move |focus: FocusState| {
            if !focus.has_focus() && navigation.with(|value| value.focused.is_some()) {
                navigation.with_mut(|value| value.focused = None);
                FocusManager::current().defer_keyboard_until_layout();
            }
        })
}

pub(super) fn focused_article_tab(navigation: State<ArticleNavigationState>) -> Option<usize> {
    let requesters = navigation.with(|navigation| navigation.requesters);
    requesters.iter().position(FocusRequester::is_focused)
}

pub(super) fn focus_article_tab(navigation: State<ArticleNavigationState>, index: usize) {
    let requester = navigation.with(|navigation| navigation.requesters[index]);
    requester.request_focus();
    FocusManager::current().defer_keyboard_until_layout();
}

pub(super) fn article_tab_focus_visible(
    navigation: State<ArticleNavigationState>,
    index: usize,
) -> bool {
    navigation
        .with(|value| value.requesters[index])
        .is_focus_visible()
}

pub(super) fn article_tab_modifier(
    modifier: Modifier,
    navigation: State<ArticleNavigationState>,
    index: usize,
    on_select: Callback,
    interaction: Option<State<InteractionState>>,
) -> Modifier {
    let (requester, selected, tab_stop, available) = navigation.with(|value| {
        (
            value.requesters[index],
            value.selected == index,
            value.focused.unwrap_or(value.selected) == index,
            value.available[index],
        )
    });
    modifier
        .semantics(SemanticsArgs {
            selected: Some(selected),
            merge_descendants: true,
            ..Default::default()
        })
        .push_keyboard_input(ArticleTabKeyboard { navigation, index })
        .on_focus_changed(move |focus: FocusState| {
            if focus.has_focus() && navigation.with(|value| value.focused != Some(index)) {
                navigation.with_mut(|value| value.focused = Some(index));
                FocusManager::current().defer_keyboard_until_layout();
            }
        })
        .clickable_with(ClickableArgs {
            role: Some(Role::Tab),
            label: Some(ARTICLE_TAB_TITLES[index].into()),
            enabled: available,
            interaction_state: interaction,
            focus_requester: Some(requester),
            focus_properties: Some(
                FocusProperties::new()
                    .skip_traversal(!tab_stop)
                    .can_focus(available),
            ),
            on_click: Callback::new(move || {
                requester.request_focus();
                on_select.call();
            }),
            ..Default::default()
        })
}

#[derive(Clone, Copy)]
struct ArticleTabKeyboard {
    navigation: State<ArticleNavigationState>,
    index: usize,
}

impl ArticleTabKeyboard {
    fn handle_key(&self, key: &Key, state: ElementState, modifiers: ModifiersState) -> bool {
        if state != ElementState::Pressed
            || modifiers.control_key()
            || modifiers.alt_key()
            || modifiers.super_key()
            || modifiers.shift_key()
        {
            return false;
        }
        let Key::Named(key) = key else {
            return false;
        };
        let target = self.navigation.with(|value| {
            let target = match key {
                NamedKey::Home => value.available.iter().position(|available| *available),
                NamedKey::End => value.available.iter().rposition(|available| *available),
                NamedKey::ArrowLeft | NamedKey::ArrowRight => (1..=4)
                    .map(|step| {
                        if *key == NamedKey::ArrowRight {
                            (self.index + step) % 4
                        } else {
                            (self.index + 4 - step) % 4
                        }
                    })
                    .find(|index| value.available[*index]),
                _ => None,
            };
            target.map(|index| value.requesters[index])
        });
        let Some(target) = target else {
            return false;
        };
        target.request_focus();
        FocusManager::current().defer_keyboard_until_layout();
        true
    }
}

impl KeyboardInputModifierNode for ArticleTabKeyboard {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        for event in input.keyboard_events.iter() {
            if self.handle_key(&event.logical_key, event.state, input.key_modifiers) {
                input.block_keyboard();
                return;
            }
        }
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use std::{cell::Cell, env, process::Command, time::Duration};

    use tessera_components::{
        alignment::Alignment, boxed::boxed, column::column, row::row,
        scrollable::ScrollableController, spacer::spacer, text::text,
    };
    use tessera_ui::{
        Dp, FocusDirection, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType,
        PxPosition,
        accesskit::{Action, ActionRequest, TreeId},
        remember, tessera,
    };

    use super::*;

    struct Pipelines;

    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    thread_local! {
        static AVAILABLE: Cell<[bool; 4]> = const { Cell::new([true; 4]) };
        static NAVIGATION: Cell<Option<State<ArticleNavigationState>>> = const { Cell::new(None) };
        static SELECTED: Cell<Option<State<&'static str>>> = const { Cell::new(None) };
        static ARTICLE_SCROLL: Cell<Option<State<ScrollableController>>> = const { Cell::new(None) };
    }

    #[tessera]
    fn scrolling_tab_fixture() {
        let selected = remember(|| "Overview");
        let navigation = remember(ArticleNavigationState::default);
        let vertical = remember(ScrollableController::new);
        let horizontal = remember(ScrollableController::new);
        SELECTED.set(Some(selected));
        NAVIGATION.set(Some(navigation));
        ARTICLE_SCROLL.set(Some(vertical));
        sync_article_navigation(navigation, [true; 4], selected.get());
        column()
            .modifier(Modifier::new().fill_max_size().vertical_scroll(vertical))
            .children(move || {
                spacer().modifier(Modifier::new().height(Dp(80.0)));
                row()
                    .modifier(
                        article_tablist_modifier(Modifier::new(), navigation)
                            .fill_max_width()
                            .height(Dp(80.0))
                            .horizontal_scroll(horizontal),
                    )
                    .children(move || {
                        for (index, title) in ARTICLE_TAB_TITLES.into_iter().enumerate() {
                            boxed()
                                .modifier(article_tab_modifier(
                                    Modifier::new().size(Dp(140.0), Dp(80.0)),
                                    navigation,
                                    index,
                                    Callback::new(move || {
                                        selected.set(title);
                                        vertical.with_mut(|scroll| {
                                            scroll.set_scroll_position(PxPosition::ZERO);
                                        });
                                        FocusManager::current().defer_keyboard_until_layout();
                                    }),
                                    None,
                                ))
                                .children(move || {
                                    text().content(title);
                                });
                        }
                    });
                spacer().modifier(Modifier::new().height(Dp(2000.0)));
            });
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for actual tab focus and queued wheel input"]
    fn tab_pointer_focus_preserves_a_newer_wheel_before_its_first_layout() {
        let name = concat!(
            module_path!(),
            "::tab_pointer_focus_preserves_a_newer_wheel_before_its_first_layout"
        );
        let name = name.split_once("::").unwrap().1;
        if env::var("TESSERA_ARTICLE_WHEEL_TEST").as_deref() != Ok(name) {
            let output = Command::new(env::current_exe().unwrap())
                .args([
                    "--exact",
                    name,
                    "--include-ignored",
                    "--test-threads=1",
                    "--nocapture",
                ])
                .env("TESSERA_ARTICLE_WHEEL_TEST", name)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "isolated test {name} failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("test result: ok. 1 passed;"));
            return;
        }
        for settle_focus in [true, false] {
            let mut renderer = HeadlessRenderer::new(
                Box::new(scrolling_tab_fixture),
                vec![Box::new(Pipelines)],
                HeadlessConfig {
                    width: 400,
                    height: 300,
                    sample_count: 1,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(3);
            let tree = renderer.accessibility_snapshot().unwrap();
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::Tab && node.label() == Some("Specs"))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            let position = PxPosition::from_f32_xy(
                ((bounds.x0 + bounds.x1) * 0.5) as f32,
                ((bounds.y0 + bounds.y1) * 0.5) as f32,
            );
            renderer.pointer_move(MOUSE_POINTER_ID, position);
            renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            if settle_focus {
                renderer.render_frames(4);
                assert_eq!(focused(&renderer), "Specs");
            }
            for _ in 0..10 {
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    PxPosition::from_f32_xy(380.0, 250.0),
                    0.0,
                    -24.0,
                );
            }
            renderer.step_frame();
            let scroll = ARTICLE_SCROLL.get().unwrap();
            assert_eq!(focused(&renderer), "Specs");
            assert_eq!(SELECTED.get().unwrap().get(), "Specs");
            assert_eq!(
                scroll.with(|scroll| scroll.child_position().y),
                -240.0,
                "newer wheel input must win over the earlier pointer focus reveal; settled={settle_focus}"
            );
            renderer.render_frames(8);
            assert_eq!(focused(&renderer), "Specs");
            assert_eq!(scroll.with(|scroll| scroll.child_position().y), -240.0);

            assert!(
                ArticleTabKeyboard {
                    navigation: NAVIGATION.get().unwrap(),
                    index: 1,
                }
                .handle_key(
                    &Key::Named(NamedKey::End),
                    ElementState::Pressed,
                    ModifiersState::empty(),
                )
            );
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(380.0, 250.0),
                0.0,
                0.0,
            );
            renderer.render_frames(4);
            assert_focused_tab_visible(&renderer, "Accessibility");
            assert!(scroll.with(|scroll| scroll.child_position().y) > -240.0);

            for _ in 0..10 {
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    PxPosition::from_f32_xy(380.0, 250.0),
                    0.0,
                    -24.0,
                );
            }
            renderer.step_frame();
            let manual_position = scroll.with(|scroll| scroll.child_position().y);
            NAVIGATION
                .get()
                .unwrap()
                .with(|navigation| navigation.requesters[0])
                .request_focus();
            renderer.render_frames(4);
            assert_focused_tab_visible(&renderer, "Overview");
            assert!(scroll.with(|scroll| scroll.child_position().y) > manual_position);
        }
    }

    fn assert_focused_tab_visible(renderer: &HeadlessRenderer, label: &str) {
        assert_eq!(focused(renderer), label);
        let tree = renderer.accessibility_snapshot().unwrap();
        let bounds = tree
            .nodes
            .iter()
            .find(|(id, _)| *id == tree.focus)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert!(bounds.y0 >= 0.0);
        assert!(bounds.y1 <= f64::from(renderer.physical_size().height));
    }

    #[tessera]
    fn fixture() {
        let available = AVAILABLE.get();
        let selected = remember(|| if available[1] { "Specs" } else { "Overview" });
        let navigation = remember(ArticleNavigationState::default);
        NAVIGATION.set(Some(navigation));
        SELECTED.set(Some(selected));
        sync_article_navigation(navigation, available, selected.get());
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                column().children(move || {
                    boxed()
                        .modifier(Modifier::new().size(Dp(100.0), Dp(48.0)).clickable_with(
                            ClickableArgs {
                                label: Some("Before sections".into()),
                                role: Some(Role::Button),
                                ..Default::default()
                            },
                        ))
                        .children(|| {
                            text().content("Before");
                        });
                    row()
                        .modifier(article_tablist_modifier(
                            Modifier::new().semantics(SemanticsArgs {
                                role: Some(Role::TabList),
                                label: Some("Article sections".into()),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                            navigation,
                        ))
                        .children(move || {
                            for (index, title) in ARTICLE_TAB_TITLES.into_iter().enumerate() {
                                if !available[index] {
                                    continue;
                                }
                                boxed()
                                    .modifier(article_tab_modifier(
                                        Modifier::new().size(Dp(140.0), Dp(80.0)),
                                        navigation,
                                        index,
                                        Callback::new(move || selected.set(title)),
                                        None,
                                    ))
                                    .children(move || {
                                        text().content(title);
                                    });
                            }
                        });
                    boxed()
                        .modifier(Modifier::new().size(Dp(100.0), Dp(48.0)).clickable_with(
                            ClickableArgs {
                                label: Some("After sections".into()),
                                role: Some(Role::Button),
                                ..Default::default()
                            },
                        ))
                        .children(|| {
                            text().content("After");
                        });
                });
            });
    }

    fn focused(renderer: &HeadlessRenderer) -> String {
        let tree = renderer.accessibility_snapshot().unwrap();
        tree.nodes
            .iter()
            .find(|(id, _)| *id == tree.focus)
            .unwrap()
            .1
            .label()
            .unwrap()
            .into()
    }

    fn action(renderer: &mut HeadlessRenderer, label: &str, action: Action) {
        let tree = renderer.accessibility_snapshot().unwrap();
        let target_node = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label) && node.role() == Role::Tab)
            .unwrap_or_else(|| panic!("missing native tab {label}: {tree:#?}"));
        assert!(
            target_node.1.supports_action(action),
            "native tab {label} lacks {action:?}: {:?}",
            target_node.1
        );
        let target_node = target_node.0;
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            target_node,
            target_tree: TreeId::ROOT,
            action,
            data: None,
        }));
        renderer.render_frames(4);
    }

    fn key(renderer: &mut HeadlessRenderer, from: usize, key: NamedKey) {
        assert!(
            ArticleTabKeyboard {
                navigation: NAVIGATION.get().unwrap(),
                index: from
            }
            .handle_key(
                &Key::Named(key),
                ElementState::Pressed,
                ModifiersState::empty()
            )
        );
        renderer.render_frames(4);
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for actual tab focus and accessibility actions"]
    fn production_tab_key_handler_moves_native_focus_without_activation_and_roving_traversal_exits()
    {
        for available in [[true; 4], [true, false, false, true]] {
            AVAILABLE.set(available);
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(Pipelines)],
                HeadlessConfig {
                    width: 640,
                    height: 240,
                    sample_count: 1,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(5);
            let initial = if available[1] { 1 } else { 0 };
            let next = if available[1] { 2 } else { 3 };
            let tree = renderer.accessibility_snapshot().unwrap();
            let tab = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Tab && node.label() == Some(ARTICLE_TAB_TITLES[initial])
                })
                .unwrap()
                .1
                .bounds()
                .unwrap();
            let position = PxPosition::from_f32_xy(
                ((tab.x0 + tab.x1) * 0.5) as f32,
                ((tab.y0 + tab.y1) * 0.5) as f32,
            );
            renderer.pointer_move(MOUSE_POINTER_ID, position);
            renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
            renderer.render_frames(4);
            assert_eq!(
                focused(&renderer),
                ARTICLE_TAB_TITLES[initial],
                "pointer activation must focus the tab before keyboard navigation"
            );
            assert_eq!(SELECTED.get().unwrap().get(), ARTICLE_TAB_TITLES[initial]);
            action(&mut renderer, "Accessibility", Action::Focus);
            assert_eq!(focused(&renderer), "Accessibility");
            assert_eq!(
                SELECTED.get().unwrap().get(),
                ARTICLE_TAB_TITLES[initial],
                "accessibility focus alone must not activate a tab"
            );
            action(&mut renderer, ARTICLE_TAB_TITLES[initial], Action::Focus);
            key(&mut renderer, initial, NamedKey::ArrowRight);
            assert_eq!(focused(&renderer), ARTICLE_TAB_TITLES[next]);
            assert_eq!(
                SELECTED.get().unwrap().get(),
                ARTICLE_TAB_TITLES[initial],
                "manual navigation must not activate a focused tab"
            );
            key(&mut renderer, next, NamedKey::End);
            assert_eq!(focused(&renderer), "Accessibility");
            key(&mut renderer, 3, NamedKey::ArrowRight);
            assert_eq!(focused(&renderer), "Overview");
            key(&mut renderer, 0, NamedKey::ArrowLeft);
            assert_eq!(focused(&renderer), "Accessibility");
            key(&mut renderer, 3, NamedKey::Home);
            assert_eq!(focused(&renderer), "Overview");
            key(&mut renderer, 0, NamedKey::End);
            action(&mut renderer, "Accessibility", Action::Click);
            assert_eq!(SELECTED.get().unwrap().get(), "Accessibility");
            let tree = renderer.accessibility_snapshot().unwrap();
            let tabs: Vec<_> = tree
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Tab)
                .collect();
            assert_eq!(
                tabs.len(),
                available.into_iter().filter(|value| *value).count()
            );
            for (_, tab) in tabs {
                assert_eq!(
                    tab.is_selected(),
                    Some(tab.label() == Some("Accessibility"))
                );
            }
            SELECTED.get().unwrap().set("Overview");
            renderer.render_frames(4);
            assert_eq!(
                focused(&renderer),
                "Accessibility",
                "external page selection must not steal current tab focus"
            );
            assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
            renderer.render_frames(4);
            assert_eq!(
                focused(&renderer),
                "After sections",
                "one Tab traversal leaves the tab list"
            );
            assert!(FocusManager::current().request_move_focus(FocusDirection::Previous));
            renderer.render_frames(4);
            assert_eq!(
                focused(&renderer),
                "Overview",
                "re-entry uses the latest selected section"
            );
            let handler = ArticleTabKeyboard {
                navigation: NAVIGATION.get().unwrap(),
                index: 0,
            };
            for ignored in [NamedKey::ArrowUp, NamedKey::ArrowDown, NamedKey::Tab] {
                assert!(!handler.handle_key(
                    &Key::Named(ignored),
                    ElementState::Pressed,
                    ModifiersState::empty()
                ));
            }
            assert!(!handler.handle_key(
                &Key::Named(NamedKey::ArrowRight),
                ElementState::Released,
                ModifiersState::empty()
            ));
            assert!(!handler.handle_key(
                &Key::Named(NamedKey::ArrowRight),
                ElementState::Pressed,
                ModifiersState::CONTROL
            ));
        }
    }
}
