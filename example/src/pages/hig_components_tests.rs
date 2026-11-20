use tessera_components::experiment::hig::button::button;
use tessera_ui::{
    MOUSE_POINTER_ID,
    accesskit::{Action, Node, Role, Toggled},
    testing::{TestProbe, UiTest},
};

use super::*;

#[tessera]
fn directory_fixture() {
    hig_catalog(
        remember(ScrollState::default),
        remember(|| Dp(900.0)),
        CallbackWith::new(|_| {}),
    );
}

fn setup(ui: &UiTest) {
    ui.run(|| tessera_ui::EntryRegistry::new().register_package(crate::res::i18n()));
}

fn control(ui: &UiTest, label: &str, role: Role) -> (tessera_ui::accesskit::NodeId, Node) {
    ui.inspect_accessibility()
        .unwrap()
        .nodes()
        .iter()
        .find(|(_, node)| node.role() == role && crate::has_label(node, label))
        .map(|(id, node)| (*id, node.clone()))
        .unwrap_or_else(|| panic!("missing {role:?}: {label}"))
}

fn activate(ui: &mut UiTest, label: &str, role: Role) {
    let (id, _) = control(ui, label, role);
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frames(2);
}

#[tessera]
fn article_fixture(index: usize) {
    hig_article(index, CallbackWith::new(|_| {}));
}

#[test]
fn directory_cards_follow_the_breakpoints_and_margins_after_resize() {
    let mut ui = UiTest::new().viewport(900, 800).content(directory_fixture);
    setup(&ui);
    for (width, columns) in [(900, 2), (600, 1), (601, 2), (1294, 2), (1295, 3), (320, 1)] {
        ui.resize(width, 800);
        ui.frames(3);
        let cards = (0..3)
            .map(|index| ui.node(&format!("hig-component-card-{index}")).bounds())
            .collect::<Vec<_>>();
        for (index, card) in cards.iter().enumerate() {
            let margin = MARGIN as i32;
            assert!(card.x.0 >= margin && card.x.0 + card.width.0 <= width as i32 - margin);
            if index > 0 {
                if index % columns == 0 {
                    assert!(card.y.0 > cards[index - 1].y.0);
                    assert_eq!(card.x, cards[0].x);
                } else {
                    assert_eq!(card.y, cards[index - 1].y);
                    assert_eq!(
                        card.x.0,
                        cards[index - 1].x.0 + cards[index - 1].width.0 + GAP as i32
                    );
                }
            }
        }
    }
}

#[tessera]
fn scroll_fixture(focus: TestProbe<Option<tessera_ui::FocusRequester>>) {
    let requester = remember(tessera_ui::FocusRequester::new).get();
    focus.set(Some(requester));
    hig_theme().child(move || {
        viewport().child(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    button("Top");
                    gap(Dp(1000.0));
                    button("Bottom").focus_requester(requester).modifier(
                        Modifier::new().semantics(SemanticsArgs {
                            test_tag: Some("bottom".into()),
                            ..Default::default()
                        }),
                    );
                });
        });
    });
}

#[test]
#[cfg(not(target_family = "wasm"))]
#[ignore = "requires a native GPU adapter for focus reveal"]
fn viewport_clamps_wheel_overshoot_and_reveals_keyboard_focus() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 300)
        .rendered(crate::TestPipelines)
        .content(|| scroll_fixture(probe.clone()));
    ui.frames(2);
    let point = PxPosition::from_f32_xy(200.0, 150.0);
    ui.pointer_scroll(MOUSE_POINTER_ID, point, 0.0, -100_000.0);
    ui.frames(2);
    let bottom = ui.node("bottom").bounds();
    assert_eq!(bottom.y.0 + bottom.height.0, 300);
    ui.pointer_scroll(MOUSE_POINTER_ID, point, 0.0, 10.0);
    ui.frames(2);
    assert_eq!(ui.node("bottom").bounds().y.0, bottom.y.0 + 10);
    ui.pointer_scroll(MOUSE_POINTER_ID, point, 0.0, 100_000.0);
    ui.frames(2);
    assert!(ui.node("bottom").bounds().y.0 > 300);
    ui.press_key(NamedKey::Tab);
    ui.frames(1);
    ui.press_key(NamedKey::Tab);
    ui.frames(3);
    let bottom = ui.node("bottom").bounds();
    assert!(
        bottom.y.0 >= 0 && bottom.y.0 + bottom.height.0 <= 300,
        "revealed bounds: {bottom:?}; focused: {}",
        ui.run(|| probe.get().unwrap().is_focused())
    );
    ui.press_key(NamedKey::Home);
    ui.frames(2);
    assert!(ui.node("bottom").bounds().y.0 > 300);
    ui.press_key(NamedKey::End);
    ui.frames(2);
    assert_eq!(ui.node("bottom").bounds().y.0 + bottom.height.0, 300);
}

#[test]
fn article_examples_update_values_and_state_grids_publish_disabled_controls() {
    for index in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
        let mut ui = UiTest::new().viewport(600, 1200).content(move || {
            article_fixture(index);
        });
        setup(&ui);
        ui.frames(3);
        match index {
            0 => {
                activate(&mut ui, "Continue", Role::Button);
                assert!(
                    ui.inspect_accessibility()
                        .unwrap()
                        .nodes()
                        .iter()
                        .any(|(_, node)| crate::has_label(node, "1"))
                );
            }
            1 => {
                activate(&mut ui, "Help buttons", Role::Button);
                assert!(
                    ui.inspect_accessibility()
                        .unwrap()
                        .nodes()
                        .iter()
                        .any(|(_, node)| crate::has_label(
                            node,
                            "Change an option to try the control."
                        ))
                );
            }
            2 => {
                activate(&mut ui, "Additional information", Role::DisclosureTriangle);
                assert_eq!(
                    control(&ui, "Additional information", Role::DisclosureTriangle)
                        .1
                        .is_expanded(),
                    Some(true)
                );
            }
            3 => {
                activate(&mut ui, "Enable notifications", Role::CheckBox);
                assert_eq!(
                    control(&ui, "Enable notifications", Role::CheckBox)
                        .1
                        .toggled(),
                    Some(Toggled::Mixed)
                );
                activate(&mut ui, "Enable notifications", Role::CheckBox);
                assert_eq!(
                    control(&ui, "Enable notifications", Role::CheckBox)
                        .1
                        .toggled(),
                    Some(Toggled::True)
                );
            }
            4 => {
                activate(&mut ui, "Express", Role::RadioButton);
                assert_eq!(
                    control(&ui, "Express", Role::RadioButton).1.toggled(),
                    Some(Toggled::True)
                );
                assert_eq!(
                    control(&ui, "Standard", Role::RadioButton).1.toggled(),
                    Some(Toggled::False)
                );
            }
            5 => {
                activate(&mut ui, "Enable notifications", Role::Switch);
                assert_eq!(
                    control(&ui, "Enable notifications", Role::Switch)
                        .1
                        .toggled(),
                    Some(Toggled::True)
                );
            }
            6 => {
                activate(&mut ui, "Express", Role::RadioButton);
                assert_eq!(
                    control(&ui, "Express", Role::RadioButton).1.toggled(),
                    Some(Toggled::True)
                );
            }
            7..=10 => {
                let label = match index {
                    9 => "Completion",
                    10 => "Capacity",
                    _ => "Value",
                };
                let role = if index == 8 {
                    Role::SpinButton
                } else {
                    Role::Slider
                };
                let (id, before) = control(&ui, label, role);
                assert!(ui.accessibility_action(id, Action::Increment));
                ui.frames(2);
                assert!(
                    control(&ui, label, role).1.numeric_value().unwrap()
                        > before.numeric_value().unwrap()
                );
                if index == 9 {
                    assert_eq!(
                        control(&ui, "Value", Role::ProgressIndicator)
                            .1
                            .numeric_value(),
                        control(&ui, label, role).1.numeric_value()
                    );
                }
            }
            _ => unreachable!(),
        }
        if COMPONENTS[index].sections[1..]
            .iter()
            .any(|section| section.rows.contains(&"ui-hig-disabled"))
        {
            assert!(
                ui.inspect_accessibility()
                    .unwrap()
                    .nodes()
                    .iter()
                    .any(|(_, node)| node.is_disabled())
            );
        }
    }
}

#[test]
fn app_drawer_opens_hig_articles_and_back_returns_to_the_directory() {
    let navigation = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(1200, 1600)
        .content(|| crate::app::app_fixture(navigation.clone()));
    setup(&ui);
    ui.frames(2);
    activate(&mut ui, "HIG components", Role::Tab);
    ui.advance(std::time::Duration::from_secs(1));
    let router = navigation.get().unwrap().router;
    assert!(ui.run(|| router.with(RouterController::current_is::<HigComponentsPageDestination>)));
    activate(&mut ui, "Open Buttons specifications", Role::Button);
    ui.advance(std::time::Duration::from_millis(400));
    assert_eq!(ui.run(|| router.with(current_article)), Some(0));
    activate(&mut ui, "Next", Role::Button);
    ui.advance(std::time::Duration::from_millis(400));
    assert_eq!(ui.run(|| router.with(current_article)), Some(1));
    activate(&mut ui, "Back to components", Role::Button);
    ui.advance(std::time::Duration::from_millis(400));
    assert!(ui.run(|| router.with(RouterController::current_is::<HigComponentsPageDestination>)));
}
