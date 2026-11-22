use std::time::Duration;

use tessera_ui::{
    FocusRequester, KeyboardEvent, MOUSE_POINTER_ID, PressKeyEventType, PxPosition, State,
    accesskit::Action,
    remember, scaled_box, tessera,
    testing::{TestProbe, UiTest},
    winit::{event::ElementState, keyboard::NamedKey},
};

use crate::column::column;

use super::stepper;

#[derive(Clone, Copy)]
struct Controls {
    value: State<f64>,
    accept: State<bool>,
    enabled: State<bool>,
    wraps: State<bool>,
    increment: State<f64>,
    repeat: State<bool>,
    visible: State<bool>,
    requests: State<Vec<f64>>,
    focus: FocusRequester,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let value = remember(|| 5.0);
    let accept = remember(|| true);
    let enabled = remember(|| true);
    let wraps = remember(|| false);
    let increment = remember(|| 1.0);
    let repeat = remember(|| true);
    let visible = remember(|| true);
    let requests = remember(Vec::new);
    let focus = remember(FocusRequester::new).get();
    probe.set(Some(Controls {
        value,
        accept,
        enabled,
        wraps,
        increment,
        repeat,
        visible,
        requests,
        focus,
    }));
    column().children(move || {
        if visible.get() {
            stepper(value.get())
                .range(0.0..=10.0)
                .wraps(wraps.get())
                .increment(increment.get())
                .enabled(enabled.get())
                .autorepeat(repeat.get())
                .focus_requester(focus)
                .accessibility_label("Quantity")
                .on_value_change(move |next| {
                    requests.with_mut(|requests| requests.push(next));
                    if accept.get() {
                        value.set(next);
                    }
                });
        }
    });
}

fn point(up: bool) -> PxPosition {
    PxPosition::from_f32_xy(10.0, if up { 6.0 } else { 18.0 })
}

#[test]
fn pointer_press_requests_latest_controlled_steps_and_release_does_not_step_again() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(120, 80)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    ui.pointer_press(MOUSE_POINTER_ID, point(true), PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.value.get(), 6.0);
    ui.pointer_release(MOUSE_POINTER_ID, point(true), PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.requests.get(), [6.0]);
    ui.click_at(point(false));
    assert_eq!(controls.value.get(), 5.0);
    controls.accept.set(false);
    ui.frame();
    ui.click_at(point(true));
    ui.click_at(point(true));
    assert_eq!(controls.value.get(), 5.0);
    assert_eq!(controls.requests.get(), [6.0, 5.0, 6.0, 6.0]);
    controls.value.set(10.0);
    ui.frame();
    ui.click_at(point(true));
    assert_eq!(controls.requests.get().len(), 4);
    controls.wraps.set(true);
    ui.frame();
    ui.click_at(point(true));
    assert_eq!(*controls.requests.get().last().unwrap(), 0.0);
    controls.value.set(0.4);
    ui.frame();
    ui.click_at(point(false));
    assert_eq!(*controls.requests.get().last().unwrap(), 10.0);
    controls.enabled.set(false);
    ui.frame();
    ui.click_at(point(true));
    assert_eq!(controls.requests.get().len(), 6);
    let (id, node) = ui.accessibility_node("Quantity");
    assert!(node.is_disabled());
    assert!(
        !ui.accessibility_node("Quantity")
            .1
            .supports_action(Action::Increment)
    );
    let count = controls.requests.get().len();
    ui.accessibility_action(id, Action::Increment);
    ui.frame();
    assert_eq!(controls.requests.get().len(), count);
}

#[test]
fn fractional_steps_clamp_or_wrap_and_accessibility_reads_the_current_increment() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(120, 80)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    controls.increment.set(0.25);
    controls.value.set(9.9);
    ui.frame();
    let (id, node) = ui.accessibility_node("Quantity");
    assert_eq!(node.numeric_value_step(), Some(0.25));
    assert!(ui.accessibility_action(id, Action::Increment));
    ui.frame();
    assert_eq!(controls.value.get(), 10.0);
    assert!(
        !ui.accessibility_node("Quantity")
            .1
            .supports_action(Action::Increment)
    );
    let count = controls.requests.get().len();
    ui.accessibility_action(id, Action::Increment);
    ui.frame();
    assert_eq!(controls.requests.get().len(), count);
    controls.wraps.set(true);
    ui.frame();
    assert!(ui.accessibility_action(id, Action::Increment));
    ui.frame();
    assert_eq!(controls.value.get(), 0.0);
    assert!(ui.accessibility_action(id, Action::Decrement));
    ui.frame();
    assert_eq!(controls.value.get(), 10.0);
    controls.increment.set(2.0);
    controls.value.set(5.5);
    ui.frame();
    assert!(ui.accessibility_action(id, Action::Decrement));
    ui.frame();
    assert_eq!(controls.value.get(), 3.5);
    assert_eq!(controls.requests.get(), [10.0, 0.0, 10.0, 3.5]);
}

fn check_repeat_and_keyboard() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(120, 80)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(2);
    let controls = probe.get().unwrap();
    let (id, _) = ui.accessibility_node("Quantity");
    let resting = ui.read_pixels();
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frame();
    assert!(ui.run(|| controls.focus.is_focused()));
    assert!(controls.requests.get().is_empty());
    assert_ne!(resting, ui.read_pixels());
    ui.press_key(NamedKey::ArrowUp);
    ui.frame();
    assert_eq!(controls.value.get(), 6.0);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::ArrowUp,
        ElementState::Released,
    ));
    ui.frame();
    assert_eq!(controls.requests.get(), [6.0]);
    assert!(ui.accessibility_action(id, Action::Decrement));
    ui.frame();
    assert_eq!(controls.value.get(), 5.0);
    controls.value.set(0.0);
    ui.frame();
    assert!(
        !ui.accessibility_node("Quantity")
            .1
            .supports_action(Action::Decrement)
    );
    ui.pointer_press(MOUSE_POINTER_ID, point(true), PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.value.get(), 1.0);
    ui.advance(Duration::from_millis(400));
    assert_eq!(controls.value.get(), 1.0);
    ui.advance(Duration::from_millis(120));
    assert_eq!(controls.value.get(), 2.0);
    ui.advance(Duration::from_millis(100));
    assert_eq!(controls.value.get(), 3.0);
    ui.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(100.0, 60.0));
    ui.frame();
    ui.advance(Duration::from_millis(300));
    assert_eq!(controls.value.get(), 3.0);
    ui.pointer_move(MOUSE_POINTER_ID, point(true));
    ui.frames(2);
    assert_eq!(controls.value.get(), 4.0);
    controls.enabled.set(false);
    ui.frame();
    ui.advance(Duration::from_millis(300));
    ui.pointer_release(MOUSE_POINTER_ID, point(true), PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.value.get(), 4.0);
    assert!(!ui.run(|| controls.focus.is_focused()));
    controls.enabled.set(true);
    controls.repeat.set(false);
    ui.frame();
    ui.pointer_press(MOUSE_POINTER_ID, point(true), PressKeyEventType::Left);
    ui.frame();
    ui.advance(Duration::from_millis(800));
    assert_eq!(controls.value.get(), 5.0);
    ui.pointer_release(MOUSE_POINTER_ID, point(true), PressKeyEventType::Left);
    ui.frame();
    controls.repeat.set(true);
    ui.frame();
    ui.pointer_press(MOUSE_POINTER_ID, point(true), PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.value.get(), 6.0);
    controls.visible.set(false);
    ui.frame();
    ui.advance(Duration::from_millis(800));
    ui.pointer_release(MOUSE_POINTER_ID, point(true), PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.value.get(), 6.0);
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch and pixels"]
fn keyboard_accessibility_repeat_cancellation_and_removal_use_the_stepper() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::stepper::tests::keyboard_accessibility_repeat_cancellation_and_removal_use_the_stepper",
        check_repeat_and_keyboard,
    );
}

fn check_scaled_input() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(240, 120)
        .scale_factor(1.5)
        .content(|| {
            let probe = probe.clone();
            scaled_box(1.25).child(move || fixture(probe.clone()));
        });
    ui.frame();
    let controls = probe.get().unwrap();
    let (_, node) = ui.accessibility_node("Quantity");
    let bounds = node.bounds().unwrap();
    assert_eq!((bounds.x1 - bounds.x0, bounds.y1 - bounds.y0), (37.5, 45.0));
    let point = |up: bool| {
        PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            (bounds.y0 + (bounds.y1 - bounds.y0) * if up { 0.25 } else { 0.75 }) as f32,
        )
    };
    ui.click_at(point(true));
    ui.click_at(point(false));
    assert_eq!(controls.requests.get(), [6.0, 5.0]);
}

#[test]
fn fractional_density_and_ancestor_scaling_keep_the_two_pointer_targets_aligned() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::stepper::tests::fractional_density_and_ancestor_scaling_keep_the_two_pointer_targets_aligned",
        check_scaled_input,
    );
}
