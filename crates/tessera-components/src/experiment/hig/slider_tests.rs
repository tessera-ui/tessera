use std::time::Instant;

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    CursorEventContent, Dp, FocusRequester, GestureState, KeyboardEvent, MOUSE_POINTER_ID,
    Modifier, PointerChange, PressKeyEventType, PxPosition, State,
    accesskit::{Action, Role},
    remember, scaled_box, tessera,
    testing::{TestProbe, UiTest},
    winit::{event::ElementState, keyboard::NamedKey},
};

use crate::column::column;

use super::slider;

#[derive(Clone, Copy)]
struct Controls {
    value: State<f64>,
    enabled: State<bool>,
    accept: State<bool>,
    visible: State<bool>,
    width: State<Dp>,
    requests: State<Vec<f64>>,
    focus: FocusRequester,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let value = remember(|| 0.5);
    let enabled = remember(|| true);
    let accept = remember(|| true);
    let visible = remember(|| true);
    let width = remember(|| Dp(240.0));
    let requests = remember(Vec::new);
    let focus = remember(FocusRequester::new).get();
    probe.set(Some(Controls {
        value,
        enabled,
        accept,
        visible,
        width,
        requests,
        focus,
    }));
    column().children(move || {
        if visible.get() {
            slider(value.get())
                .enabled(enabled.get())
                .focus_requester(focus)
                .accessibility_label("Volume")
                .modifier(Modifier::new().width(width.get()).height(Dp(24.0)))
                .on_value_change(move |next| {
                    requests.with_mut(|requests| requests.push(next));
                    if accept.get() {
                        value.set(next);
                    }
                });
        }
    });
}

fn point(x: f32) -> PxPosition {
    PxPosition::from_f32_xy(x, 12.0)
}

#[test]
fn dragging_preserves_grab_offset_and_rejected_values_remain_controlled() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 100)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    ui.pointer_press(MOUSE_POINTER_ID, point(124.0), PressKeyEventType::Left);
    ui.frames(1);
    assert!(controls.requests.get().is_empty());
    ui.pointer_move(MOUSE_POINTER_ID, point(179.0));
    ui.frames(1);
    assert_eq!(controls.value.get(), 0.75);
    ui.pointer_release(MOUSE_POINTER_ID, point(179.0), PressKeyEventType::Left);
    ui.frames(1);
    controls.accept.set(false);
    ui.frames(1);
    ui.click_at(point(10.0));
    assert_eq!(controls.value.get(), 0.75);
    assert_eq!(*controls.requests.get().last().unwrap(), 0.0);
    controls.value.set(0.25);
    ui.frames(1);
    assert_eq!(
        ui.accessibility_node("Volume").1.numeric_value(),
        Some(0.25)
    );
    ui.click_at(point(230.0));
    assert_eq!(controls.value.get(), 0.25);
    assert_eq!(*controls.requests.get().last().unwrap(), 1.0);
}

fn check_transformed_drag() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(600, 120)
        .scale_factor(1.5)
        .content(|| {
            let probe = probe.clone();
            scaled_box(1.25).child(move || fixture(probe.clone()));
        });
    ui.frame();
    let controls = probe.get().unwrap();
    let (_, node) = ui.accessibility_node("Volume");
    let bounds = node.bounds().unwrap();
    assert_eq!(bounds.x1 - bounds.x0, 450.0);
    let point = |fraction: f64| {
        PxPosition::from_f32_xy(
            (bounds.x0 + 18.75 + 412.5 * fraction) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        )
    };
    ui.pointer_press(41, point(0.5), PressKeyEventType::Left);
    ui.frames(1);
    ui.pointer_press(42, point(0.0), PressKeyEventType::Left);
    ui.pointer_move(42, point(1.0));
    ui.pointer_release(42, point(1.0), PressKeyEventType::Left);
    ui.frames(1);
    assert_eq!(controls.value.get(), 0.5);
    ui.pointer_move(41, point(0.75));
    ui.frames(1);
    assert_eq!(controls.value.get(), 0.75);
    ui.inject_pointer(
        PointerChange::new(
            Instant::now(),
            41,
            CursorEventContent::Cancelled(PressKeyEventType::Left),
            GestureState::Dragged,
        )
        .with_position(point(0.75)),
    );
    ui.frames(1);
    ui.pointer_move(41, point(1.0));
    ui.pointer_release(41, point(1.0), PressKeyEventType::Left);
    ui.frames(1);
    assert_eq!(controls.value.get(), 0.75);
    assert_eq!(controls.requests.get(), [0.75]);
}

#[test]
fn fractional_density_and_ancestor_scaling_preserve_pointer_identity_and_cancellation() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::slider::tests::fractional_density_and_ancestor_scaling_preserve_pointer_identity_and_cancellation",
        check_transformed_drag,
    );
}

#[test]
fn custom_ranges_clamp_presentation_and_accessibility_uses_the_current_range() {
    let value = TestProbe::default();
    let range = TestProbe::default();
    let mut ui = UiTest::new().content(|| range_fixture(value.clone(), range.clone()));
    ui.frame();
    let current = value.get().unwrap();
    let limits = range.get().unwrap();
    let (id, node) = ui.accessibility_node("Level");
    assert_eq!(
        (
            node.min_numeric_value(),
            node.max_numeric_value(),
            node.numeric_value_step()
        ),
        (Some(20.0), Some(80.0), Some(3.0))
    );
    assert!(ui.accessibility_action(id, Action::Increment));
    ui.frame();
    assert_eq!(current.get(), 53.0);
    limits.set(0.0..=10.0);
    ui.frame();
    assert_eq!(ui.accessibility_node("Level").1.numeric_value(), Some(10.0));
    assert!(ui.accessibility_action(id, Action::Decrement));
    ui.frame();
    assert_eq!(current.get(), 9.5);
}

#[tessera]
fn range_fixture(
    value: TestProbe<Option<State<f64>>>,
    range: TestProbe<Option<State<std::ops::RangeInclusive<f64>>>>,
) {
    let current = remember(|| 50.0);
    let limits = remember(|| 20.0..=80.0);
    value.set(Some(current));
    range.set(Some(limits));
    slider(current.get())
        .range(limits.get())
        .accessibility_label("Level")
        .on_value_change(move |next| current.set(next));
}

#[test]
fn disabling_removal_and_narrow_layouts_cancel_dragging_without_losing_the_value() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 100)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    ui.pointer_press(MOUSE_POINTER_ID, point(120.0), PressKeyEventType::Left);
    ui.frames(1);
    controls.enabled.set(false);
    ui.frames(1);
    ui.pointer_move(MOUSE_POINTER_ID, point(230.0));
    ui.pointer_release(MOUSE_POINTER_ID, point(230.0), PressKeyEventType::Left);
    ui.frames(1);
    assert_eq!(controls.value.get(), 0.5);
    let (id, node) = ui.accessibility_node("Volume");
    assert!(node.is_disabled());
    assert_eq!(node.role(), Role::Slider);
    assert!(!ui.accessibility_action(id, Action::Increment));
    controls.enabled.set(true);
    controls.width.set(Dp(0.0));
    ui.frames(1);
    assert_eq!(ui.frames(1).node("Volume").bounds().width.0, 0);
    controls.width.set(Dp(8.0));
    ui.frames(1);
    ui.click_at(point(4.0));
    assert_eq!(controls.value.get(), 0.5);
    controls.width.set(Dp(240.0));
    ui.frames(1);
    assert_eq!(ui.frames(1).node("slider_thumb").bounds().x.0, 110);
    ui.pointer_press(MOUSE_POINTER_ID, point(120.0), PressKeyEventType::Left);
    ui.frames(1);
    controls.visible.set(false);
    ui.frames(1);
    ui.pointer_release(MOUSE_POINTER_ID, point(230.0), PressKeyEventType::Left);
    ui.frames(1);
    assert_eq!(controls.value.get(), 0.5);
    assert!(controls.requests.get().is_empty());
}

fn check_keyboard() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 100)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(2);
    let controls = probe.get().unwrap();
    let (id, _) = ui.accessibility_node("Volume");
    let resting = ui.read_pixels();
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frames(1);
    assert!(ui.run(|| controls.focus.is_focused()));
    assert!(controls.requests.get().is_empty());
    assert_ne!(resting, ui.read_pixels());
    ui.press_key(NamedKey::ArrowRight);
    ui.frames(1);
    assert_eq!(controls.value.get(), 0.55);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::ArrowRight,
        ElementState::Released,
    ));
    ui.frames(1);
    assert_eq!(controls.requests.get(), [0.55]);
    let mut repeated = KeyboardEvent::named(NamedKey::ArrowRight, ElementState::Pressed);
    repeated.repeat = true;
    ui.inject_key_event(repeated);
    ui.frames(1);
    assert!((controls.value.get() - 0.6).abs() < 1e-12);
    controls.value.set(0.55);
    ui.frames(1);
    assert!(ui.accessibility_action(id, Action::Decrement));
    ui.frames(1);
    assert_eq!(controls.value.get(), 0.5);
    controls.value.set(1.0);
    ui.frames(1);
    ui.press_key(NamedKey::ArrowUp);
    ui.frames(1);
    assert_eq!(controls.requests.get().len(), 3);
    assert_eq!(*controls.requests.get().last().unwrap(), 0.5);
    ui.press_key(NamedKey::PageDown);
    ui.frames(1);
    assert_eq!(controls.value.get(), 0.0);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::PageDown,
        ElementState::Released,
    ));
    ui.frames(1);
    assert_eq!(controls.requests.get().len(), 4);
    ui.press_key(NamedKey::PageUp);
    ui.frames(1);
    assert_eq!(controls.value.get(), 1.0);
    controls.enabled.set(false);
    ui.frames(1);
    assert!(!ui.run(|| controls.focus.is_focused()));
    assert!(!ui.accessibility_action(id, Action::Increment));
    assert_eq!(ui.accessibility_node("Volume").1.numeric_value(), Some(1.0));
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch"]
fn keyboard_and_accessibility_adjust_the_latest_controlled_value_without_focus_activation() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::slider::tests::keyboard_and_accessibility_adjust_the_latest_controlled_value_without_focus_activation",
        check_keyboard,
    );
}
