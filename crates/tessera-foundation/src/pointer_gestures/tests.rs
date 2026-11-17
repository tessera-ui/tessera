use std::{cell::RefCell, rc::Rc};

use tessera_ui::{
    Dp, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition,
    layout::{Stack, layout},
    testing::UiTest,
};

use crate::modifier::ModifierExt as _;

use super::*;

#[derive(Default, Debug)]
struct Drags {
    starts: usize,
    deltas: Vec<f32>,
    ends: usize,
    cancellations: usize,
}

#[tessera_ui::tessera]
fn drag_fixture(
    enabled: tessera_ui::State<bool>,
    key: tessera_ui::State<i32>,
    #[prop(skip_eq)] observations: Rc<RefCell<Drags>>,
) {
    let modifier = Modifier::new().size(Dp(80.0), Dp(80.0));
    let modifier = if enabled.get() {
        let observations = observations.clone();
        modifier.pointer_input(key.get(), move |mut scope| {
            let observations = observations.clone();
            async move {
                detect_drag_gestures(
                    &mut scope,
                    DragGestureConfig {
                        slop_px: 8.0,
                        axis: Some(DragAxis::Horizontal),
                        ..Default::default()
                    },
                    |_| observations.borrow_mut().starts += 1,
                    |_, _, delta| observations.borrow_mut().deltas.push(delta.x),
                    |_, _| observations.borrow_mut().ends += 1,
                    || observations.borrow_mut().cancellations += 1,
                )
                .await;
            }
        })
    } else {
        modifier
    };
    layout(Stack).modifier(modifier);
}

#[test]
fn drag_slop_accumulates_from_press_and_cancellation_follows_attachment() {
    let observations = Rc::new(RefCell::new(Drags::default()));
    let test = UiTest::new();
    let enabled = test.state(true);
    let key = test.state(0);
    let mut test = test
        .content(|| drag_fixture(enabled, key, observations.clone()))
        .viewport(80, 80);
    test.frame();
    test.pointer_press(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(10.0, 10.0),
        PressKeyEventType::Left,
    );
    test.frame();
    for x in [13.0, 16.0] {
        test.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(x, 10.0));
        test.frame();
    }
    assert_eq!(observations.borrow().starts, 0);
    test.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(19.0, 10.0));
    test.frame();
    assert_eq!(observations.borrow().starts, 1);
    assert_eq!(observations.borrow().deltas, [3.0]);
    test.run(|| key.set(1));
    test.frame();
    assert_eq!(observations.borrow().cancellations, 1);
    test.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(19.0, 10.0),
        PressKeyEventType::Left,
    );
    test.frame();
    assert_eq!(observations.borrow().ends, 0);
    test.pointer_press(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(10.0, 10.0),
        PressKeyEventType::Left,
    );
    test.frame();
    test.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(25.0, 10.0));
    test.frame();
    test.run(|| enabled.set(false));
    test.frame();
    assert_eq!(observations.borrow().starts, 2);
    assert_eq!(observations.borrow().cancellations, 2);
    assert_eq!(observations.borrow().ends, 0);
}

#[tessera_ui::tessera]
fn tap_fixture(
    label: usize,
    #[prop(skip_eq)] events: Rc<RefCell<Vec<(usize, &'static str)>>>,
    hold: Option<Duration>,
    child: Option<tessera_ui::RenderSlot>,
) {
    layout(Stack)
        .modifier(
            Modifier::new()
                .size(Dp(80.0), Dp(80.0))
                .pointer_input_once(move |mut scope| {
                    let events = events.clone();
                    async move {
                        detect_tap_gestures(
                            &mut scope,
                            TapGestureConfig {
                                long_press: hold,
                                ..Default::default()
                            },
                            |gesture| {
                                let event = match gesture {
                                    TapGesture::Pressed(_) => "press",
                                    TapGesture::Released(_, _) => "release",
                                    TapGesture::Tapped(_, _) => "tap",
                                    TapGesture::LongPressed(_) => "long",
                                    TapGesture::Cancelled => "cancel",
                                };
                                events.borrow_mut().push((label, event));
                            },
                        )
                        .await;
                    }
                }),
        )
        .child(move || {
            if let Some(child) = child {
                child.render();
            }
        });
}

#[test]
fn nested_tap_claims_exclude_ancestors_without_consuming_the_press() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut test = UiTest::new().content(|| {
        let child_events = events.clone();
        tap_fixture(0, events.clone()).child(move || {
            tap_fixture(1, child_events.clone());
        });
    });
    test.frame();
    test.click_at(PxPosition::from_f32_xy(10.0, 10.0));
    assert_eq!(
        events.borrow().as_slice(),
        [(1, "press"), (1, "release"), (1, "tap")]
    );
    events.borrow_mut().clear();
    test.pointer_press(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(10.0, 10.0),
        PressKeyEventType::Left,
    );
    test.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(35.0, 10.0));
    test.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(35.0, 10.0),
        PressKeyEventType::Left,
    );
    test.frame();
    assert_eq!(events.borrow().as_slice(), [(1, "press"), (1, "cancel")]);
}

#[test]
fn stationary_long_press_is_one_shot_and_movement_or_cancel_prevents_it() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut test = UiTest::new().content(|| {
        tap_fixture(2, events.clone()).hold(Duration::from_millis(500));
    });
    test.frame();
    let point = PxPosition::from_f32_xy(10.0, 10.0);
    test.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    test.advance(Duration::from_millis(499));
    assert_eq!(events.borrow().as_slice(), [(2, "press")]);
    test.advance(Duration::from_millis(1));
    assert_eq!(events.borrow().as_slice(), [(2, "press"), (2, "long")]);
    test.advance(Duration::from_secs(1));
    test.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    assert_eq!(
        events.borrow().as_slice(),
        [(2, "press"), (2, "long"), (2, "release")]
    );
    events.borrow_mut().clear();
    test.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    test.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(30.0, 10.0));
    test.frame();
    test.advance(Duration::from_secs(1));
    test.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    assert_eq!(events.borrow().as_slice(), [(2, "press"), (2, "cancel")]);
    events.borrow_mut().clear();
    test.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    test.inject_pointer(
        PointerChange::new(
            test.frame_origin() + test.frame_time(),
            MOUSE_POINTER_ID,
            CursorEventContent::Cancelled(PressKeyEventType::Left),
            tessera_ui::GestureState::TapCandidate,
        )
        .with_position(point),
    );
    test.frame();
    test.advance(Duration::from_secs(1));
    assert_eq!(events.borrow().as_slice(), [(2, "press"), (2, "cancel")]);
}

#[tessera_ui::tessera]
fn consumed_motion_fixture(#[prop(skip_eq)] events: Rc<RefCell<Vec<(usize, &'static str)>>>) {
    layout(Stack)
        .modifier(Modifier::new().size(Dp(80.0), Dp(80.0)).pointer_input_once(
            |mut scope| async move {
                loop {
                    let event = scope.await_pointer_event(PointerEventPass::Main).await;
                    if event.is_cancelled() {
                        return;
                    }
                    for mut change in event.changes {
                        if matches!(change.content, CursorEventContent::Moved(_)) {
                            change.consume();
                        }
                    }
                }
            },
        ))
        .child(move || {
            tap_fixture(3, events.clone()).hold(Duration::from_millis(500));
        });
}

#[test]
fn ancestor_consumption_after_child_main_cancels_a_pending_long_press() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut test = UiTest::new().content(|| consumed_motion_fixture(events.clone()));
    test.frame();
    test.pointer_press(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(10.0, 10.0),
        PressKeyEventType::Left,
    );
    test.frame();
    test.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(11.0, 10.0));
    test.frame();
    test.advance(Duration::from_millis(500));
    assert_eq!(events.borrow().as_slice(), [(3, "press"), (3, "cancel")]);
}

#[tessera_ui::tessera]
fn repeated_gesture_fixture(#[prop(skip_eq)] starts: Rc<RefCell<Vec<u64>>>) {
    layout(Stack).modifier(Modifier::new().size(Dp(80.0), Dp(80.0)).pointer_input_once(
        move |mut scope| {
            let starts = starts.clone();
            async move {
                scope
                    .await_each_gesture(move |mut scope| {
                        let starts = starts.clone();
                        async move {
                            if let Some(down) = scope
                                .await_first_down(
                                    PointerEventPass::Main,
                                    None,
                                    DownRequirement::Any,
                                )
                                .await
                            {
                                starts.borrow_mut().push(down.change.pointer_id);
                                scope.await_pointer_event(PointerEventPass::Main).await;
                            }
                        }
                    })
                    .await;
            }
        },
    ));
}

#[test]
fn completed_gesture_waits_for_every_observed_pointer_before_restarting() {
    let starts = Rc::new(RefCell::new(Vec::new()));
    let mut test = UiTest::new().content(|| repeated_gesture_fixture(starts.clone()));
    test.frame();
    let point = PxPosition::from_f32_xy(10.0, 10.0);
    test.pointer_press(1, point, PressKeyEventType::Left);
    test.frame();
    test.pointer_press(2, point, PressKeyEventType::Left);
    test.frame();
    test.pointer_release(1, point, PressKeyEventType::Left);
    test.frame();
    test.pointer_press(3, point, PressKeyEventType::Left);
    test.frame();
    assert_eq!(starts.borrow().as_slice(), [1]);
    test.pointer_release(2, point, PressKeyEventType::Left);
    test.pointer_release(3, point, PressKeyEventType::Left);
    test.frame();
    test.pointer_press(4, point, PressKeyEventType::Left);
    test.frame();
    assert_eq!(starts.borrow().as_slice(), [1, 4]);
}

#[tessera_ui::tessera]
fn latest_position_fixture(#[prop(skip_eq)] positions: Rc<RefCell<Vec<Option<PxPosition>>>>) {
    layout(Stack).modifier(Modifier::new().size(Dp(80.0), Dp(80.0)).pointer_input_once(
        move |mut scope| {
            let positions = positions.clone();
            async move {
                let Some(down) = scope
                    .await_first_down(PointerEventPass::Main, None, DownRequirement::Any)
                    .await
                else {
                    return;
                };
                if let LongPressResult::Triggered { position } = scope
                    .await_long_press_or_cancellation(
                        PointerEventPass::Main,
                        &down,
                        Duration::from_millis(500),
                        8.0,
                    )
                    .await
                {
                    positions.borrow_mut().push(position);
                }
            }
        },
    ));
}

#[test]
fn long_press_returns_the_latest_sample_within_slop() {
    let positions = Rc::new(RefCell::new(Vec::new()));
    let mut test = UiTest::new().content(|| latest_position_fixture(positions.clone()));
    test.frame();
    test.pointer_press(
        1,
        PxPosition::from_f32_xy(10.0, 10.0),
        PressKeyEventType::Left,
    );
    test.frame();
    let moved = PxPosition::from_f32_xy(13.0, 14.0);
    test.pointer_move(1, moved);
    test.frame();
    test.advance(Duration::from_millis(500));
    assert_eq!(positions.borrow().as_slice(), [Some(moved)]);
}

#[tessera_ui::tessera]
fn keyed_capture_fixture(
    key: tessera_ui::State<i32>,
    revision: tessera_ui::State<i32>,
    #[prop(skip_eq)] values: Rc<RefCell<Vec<(i32, i32)>>>,
) {
    let captured = revision.get();
    layout(Stack).modifier(Modifier::new().size(Dp(80.0), Dp(80.0)).pointer_input(
        key.get(),
        move |mut scope| {
            let values = values.clone();
            async move {
                detect_tap_gestures(&mut scope, TapGestureConfig::default(), |gesture| {
                    if matches!(gesture, TapGesture::Tapped(_, _)) {
                        values.borrow_mut().push((captured, revision.get()));
                    }
                })
                .await;
            }
        },
    ));
}

#[test]
fn stable_keys_keep_the_running_capture_while_state_reads_stay_live() {
    let values = Rc::new(RefCell::new(Vec::new()));
    let test = UiTest::new();
    let key = test.state(0);
    let revision = test.state(1);
    let mut test = test.content(|| keyed_capture_fixture(key, revision, values.clone()));
    test.frame();
    let point = PxPosition::from_f32_xy(10.0, 10.0);
    test.click_at(point);
    test.run(|| revision.set(2));
    test.frame();
    test.click_at(point);
    test.run(|| key.set(1));
    test.frame();
    test.click_at(point);
    assert_eq!(values.borrow().as_slice(), [(1, 1), (1, 2), (2, 2)]);
}
