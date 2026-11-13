//! Controlled list selection and accessibility regressions.
//!
//! ## Usage
//!
//! Keep selection requests, roles, focus, and disabled activation consistent.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, FocusDirection, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, NodeId, Role, Toggled},
    focus, remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::{column::column, theme::material_theme};

use super::{ListItemSelectionMode, list_item};

#[derive(Clone, PartialEq)]
struct Settings {
    mode: ListItemSelectionMode,
    selected: [bool; 3],
    enabled: bool,
    accept: bool,
    requests: Vec<(usize, bool)>,
    actions: usize,
}

#[tessera]
fn fixture(probe: Option<TestProbe<Option<State<Settings>>>>) {
    let settings = remember(|| Settings {
        mode: ListItemSelectionMode::Multiple,
        selected: [false; 3],
        enabled: true,
        accept: true,
        requests: Vec::new(),
        actions: 0,
    });
    if let Some(probe) = probe {
        probe.set(Some(settings));
    }
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().width(Dp(280.0)))
            .children(move || {
                for index in 0..3 {
                    list_item()
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            test_tag: Some(format!("choice-{index}")),
                            tooltip: Some("Choice tooltip".into()),
                            ..Default::default()
                        }))
                        .headline(format!("Choice {}", index + 1))
                        .accessibility_label(format!("Selection {}", index + 1))
                        .selection_mode(settings.get().mode)
                        .selected(settings.get().selected[index])
                        .enabled(settings.get().enabled)
                        .on_click(move || settings.with_mut(|s| s.actions += 1))
                        .on_selection_change(move |value| {
                            settings.with_mut(|s| {
                                s.requests.push((index, value));
                                if s.accept {
                                    if s.mode == ListItemSelectionMode::Single {
                                        s.selected = [false; 3];
                                    }
                                    s.selected[index] = value;
                                }
                            })
                        });
                }
                list_item()
                    .headline("Read-only choice")
                    .accessibility_label("Read-only selection")
                    .selection_mode(settings.get().mode)
                    .selected(true)
                    .enabled(settings.get().enabled);
            });
    });
}

fn action(renderer: &mut UiTest, index: usize, action: Action) -> bool {
    renderer.accessibility_action(
        renderer
            .accessibility_node(&format!("Selection {}", index + 1))
            .0,
        action,
    )
}

fn tap(renderer: &mut UiTest, index: usize) {
    let bounds = renderer
        .accessibility_node(&format!("Selection {}", index + 1))
        .1
        .bounds()
        .unwrap();
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(4);
}

fn verify(renderer: &UiTest, settings: &Settings, ids: &[NodeId; 3]) {
    let role = match settings.mode {
        ListItemSelectionMode::Action => Role::ListItem,
        ListItemSelectionMode::Single => Role::RadioButton,
        ListItemSelectionMode::Multiple => Role::CheckBox,
    };
    for (index, id) in ids.iter().enumerate() {
        let (actual_id, node) = renderer.accessibility_node(&format!("Selection {}", index + 1));
        assert_eq!(actual_id, *id);
        assert_eq!(node.role(), role);
        assert_eq!(node.tooltip(), Some("Choice tooltip"));
        assert_eq!(node.is_disabled(), !settings.enabled);
        assert_eq!(node.supports_action(Action::Click), settings.enabled);
        assert_eq!(node.supports_action(Action::Focus), settings.enabled);
        let checked = if settings.selected[index] {
            Toggled::True
        } else {
            Toggled::False
        };
        match settings.mode {
            ListItemSelectionMode::Action => {
                assert_eq!(node.toggled(), None);
                assert_eq!(node.is_selected(), Some(settings.selected[index]));
            }
            ListItemSelectionMode::Single => {
                assert_eq!(node.toggled(), Some(checked));
                assert_eq!(node.is_selected(), Some(settings.selected[index]));
            }
            ListItemSelectionMode::Multiple => {
                assert_eq!(node.toggled(), Some(checked));
                assert_eq!(node.is_selected(), None);
            }
        }
    }
    let read_only = renderer.accessibility_node("Read-only selection").1;
    assert!(!read_only.supports_action(Action::Click));
    assert!(!read_only.supports_action(Action::Focus));
    assert_eq!(read_only.role(), role);
}

fn check() {
    for scale in [1.0, 1.5] {
        let probe = TestProbe::<Option<State<Settings>>>::default();
        let observed = probe.clone();
        let mut renderer = UiTest::new()
            .viewport(600, 540)
            .scale_factor(scale)
            .rendered(crate::TesseraComponents)
            .content(move || fixture().probe(probe.clone()));
        renderer.frames(4);
        assert!(
            renderer
                .work_totals()
                .count("list_item", tessera_ui::testing::WorkPhase::Build)
                > 0
        );
        let settings = observed.get().unwrap();
        let ids = std::array::from_fn(|index| {
            renderer
                .accessibility_node(&format!("Selection {}", index + 1))
                .0
        });
        verify(&renderer, &settings.get(), &ids);
        tap(&mut renderer, 0);
        assert_eq!(settings.get().selected, [true, false, false]);
        tap(&mut renderer, 0);
        assert_eq!(settings.get().selected, [false; 3]);
        assert_eq!(settings.get().requests, [(0, true), (0, false)]);
        settings.with_mut(|s| s.selected[1] = true);
        renderer.frames(3);
        assert!(action(&mut renderer, 1, Action::Click));
        renderer.frames(3);
        assert_eq!(settings.get().requests.last(), Some(&(1, false)));
        assert_eq!(settings.get().actions, 0);
        settings.with_mut(|s| s.mode = ListItemSelectionMode::Single);
        renderer.frames(3);
        tap(&mut renderer, 1);
        assert_eq!(settings.get().selected, [false, true, false]);
        tap(&mut renderer, 2);
        tap(&mut renderer, 2);
        assert_eq!(settings.get().selected, [false, false, true]);
        assert_eq!(settings.get().requests.last(), Some(&(2, true)));
        verify(&renderer, &settings.get(), &ids);
        assert!(action(&mut renderer, 0, Action::Focus));
        renderer.frames(3);
        focus::request_move_focus(FocusDirection::Next);
        renderer.frames(3);
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, ids[1]);
        settings.with_mut(|s| {
            s.mode = ListItemSelectionMode::Multiple;
            s.selected = [false; 3];
            s.accept = false;
        });
        renderer.frames(3);
        for _ in 0..2 {
            assert!(action(&mut renderer, 0, Action::Click));
            renderer.frames(3);
            assert_eq!(settings.get().requests.last(), Some(&(0, true)));
            assert_eq!(settings.get().selected, [false; 3]);
        }
        settings.with_mut(|s| s.mode = ListItemSelectionMode::Action);
        renderer.frames(3);
        let request_count = settings.get().requests.len();
        tap(&mut renderer, 2);
        assert_eq!(settings.get().actions, 1);
        assert_eq!(settings.get().requests.len(), request_count);
        verify(&renderer, &settings.get(), &ids);
        settings.with_mut(|s| {
            s.mode = ListItemSelectionMode::Multiple;
            s.selected[2] = true;
            s.enabled = false;
        });
        renderer.frames(3);
        verify(&renderer, &settings.get(), &ids);
        assert!(!action(&mut renderer, 2, Action::Click));
        assert!(!action(&mut renderer, 2, Action::Focus));
        tap(&mut renderer, 2);
        assert_eq!(settings.get().actions, 1);
        assert_eq!(settings.get().requests.len(), request_count);
        settings.with_mut(|s| {
            s.mode = ListItemSelectionMode::Single;
            s.enabled = true;
        });
        renderer.frames(90);
        verify(&renderer, &settings.get(), &ids);
        let builds = renderer
            .work_totals()
            .count("list_item", tessera_ui::testing::WorkPhase::Build);
        renderer.frames(8);
        assert_eq!(
            renderer
                .work_totals()
                .count("list_item", tessera_ui::testing::WorkPhase::Build),
            builds,
            "settled activation handles must not keep rebuilding rows"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn selection_modes_request_latest_controlled_values_and_preserve_roles() {
    crate::test_utils::run_isolated_test(
        "list_item::selection_tests::selection_modes_request_latest_controlled_values_and_preserve_roles",
        check,
    );
}
