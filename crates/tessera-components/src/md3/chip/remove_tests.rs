//! Input-chip body and removal interaction regressions.
//!
//! ## Usage
//!
//! Preserve separate selection and removal targets in compact chips.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::UiTest,
};

use crate::md3::{material_icons::filled, theme::material_theme};

use super::chip;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Counts {
    body: usize,
    remove: usize,
    selected: bool,
}

#[tessera]
fn fixture(probe: tessera_ui::testing::TestProbe<Option<State<Counts>>>) {
    material_theme().child(move || {
        let counts = remember(Counts::default);
        probe.set(Some(counts));
        chip()
            .input("Alice")
            .selected(counts.get().selected)
            .leading_icon(filled::PERSON_SVG)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some("removable-chip".into()),
                ..Default::default()
            }))
            .on_click(move || {
                counts.with_mut(|value| {
                    value.body += 1;
                    value.selected = !value.selected;
                })
            })
            .on_remove(move || counts.with_mut(|value| value.remove += 1));
    });
}

fn click(renderer: &mut UiTest, x: f64, y: f64) {
    let position = PxPosition::from_f32_xy(x as f32, y as f32);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(3);
}

fn check_removal() {
    let probe = tessera_ui::testing::TestProbe::default();
    let mut renderer = UiTest::new()
        .viewport(360, 120)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    renderer.frames(4);
    let counts = probe.get().unwrap();
    let remove = renderer.accessibility_node("Remove Alice");
    assert_eq!(remove.1.role(), Role::Button);
    let target = remove.1.bounds().unwrap();
    assert_eq!((target.width(), target.height()), (48.0, 48.0));
    click(
        &mut renderer,
        (target.x0 + target.x1) / 2.0,
        (target.y0 + target.y1) / 2.0,
    );
    assert_eq!(
        counts.get(),
        Counts {
            remove: 1,
            ..Default::default()
        }
    );
    let body = renderer.accessibility_node("Alice").1.bounds().unwrap();
    click(&mut renderer, body.x0 + 16.0, (body.y0 + body.y1) / 2.0);
    assert_eq!(
        counts.get(),
        Counts {
            body: 1,
            remove: 1,
            selected: true
        }
    );
    assert!(
        renderer.accessibility_action(renderer.accessibility_node("Remove Alice").0, Action::Focus)
    );
    renderer.frames(2);
    let snapshot = renderer.accessibility_snapshot().unwrap();
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|(id, _)| *id == snapshot.focus)
            .unwrap()
            .1
            .label(),
        Some("Remove Alice")
    );
    assert_eq!(
        counts.get().remove,
        1,
        "focusing removal must not activate it"
    );
    assert!(
        renderer.accessibility_action(renderer.accessibility_node("Remove Alice").0, Action::Click)
    );
    renderer.frames(2);
    assert_eq!(
        counts.get(),
        Counts {
            body: 1,
            remove: 2,
            selected: true
        }
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn input_chip_removal_has_independent_48dp_target_focus_and_activation() {
    crate::test_utils::run_isolated_test(
        "md3::chip::remove_tests::input_chip_removal_has_independent_48dp_target_focus_and_activation",
        check_removal,
    );
}
