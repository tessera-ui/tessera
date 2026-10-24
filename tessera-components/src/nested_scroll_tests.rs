use std::cell::RefCell;

use tessera_ui::{CallbackWith, LocalToWindow, PxPosition, ScrollEventSource, remember, tessera};

use super::*;

thread_local! {
    static CONNECTION: RefCell<Option<NestedScrollConnection>> = const { RefCell::new(None) };
    static OBSERVED: RefCell<Vec<(bool, u8, f32)>> = const { RefCell::new(Vec::new()) };
}

fn record(inner: bool, kind: u8, value: f32) {
    OBSERVED.with_borrow_mut(|items| items.push((inner, kind, value)));
}

fn handlers(connection: NestedScrollConnection, inner: bool) -> NestedScrollConnection {
    connection
        .with_pre_scroll_handler(CallbackWith::new(move |input: PreScrollInput| {
            record(inner, 0, input.available.y);
            ScrollDelta::new(0.0, if inner { 8.0 } else { 6.0 })
        }))
        .with_post_scroll_handler(CallbackWith::new(move |input: PostScrollInput| {
            record(inner, 1, input.available.y);
            ScrollDelta::new(0.0, if inner { 8.0 } else { 6.0 })
        }))
        .with_pre_fling_handler(CallbackWith::new(move |input: PreFlingInput| {
            record(inner, 2, input.available.y);
            ScrollVelocity::new(0.0, if inner { 8.0 } else { 6.0 })
        }))
        .with_post_fling_handler(CallbackWith::new(move |available: ScrollVelocity| {
            record(inner, 3, available.y);
            ScrollVelocity::new(0.0, if inner { 8.0 } else { 6.0 })
        }))
}

#[tessera]
fn connection_fixture() {
    let outer_scale = remember(|| 1.0_f32);
    let inner_scale = remember(|| 0.5_f32);
    let outer = handlers(
        NestedScrollConnection::new().with_coordinate_scale(outer_scale),
        false,
    );
    let inner = handlers(
        NestedScrollConnection::new()
            .with_coordinate_scale(inner_scale)
            .with_parent(Some(outer)),
        true,
    );
    CONNECTION.with_borrow_mut(|value| *value = Some(inner));
    tessera_ui::layout::layout();
}

#[test]
fn nested_scroll_consumption_and_fling_cross_each_providers_local_space_once() {
    use tessera_ui::testing::{layout_test, pump_layout_test_session, start_layout_test_session};

    let mut session = start_layout_test_session(
        layout_test(|| {
            connection_fixture();
        })
        .viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let connection = CONNECTION.with_borrow(|value| value.clone().expect("Connection fixture"));
    let transform = LocalToWindow::new(0.625, PxPosition::from_f32_xy(10.25, 20.5)).unwrap();
    let scoped = connection.in_space(transform);
    OBSERVED.with_borrow_mut(Vec::clear);
    assert_eq!(
        scoped.pre_scroll(ScrollDelta::new(0.0, 32.0), ScrollEventSource::Touch),
        ScrollDelta::new(0.0, 16.0)
    );
    assert_eq!(
        scoped.post_scroll(
            ScrollDelta::new(0.0, 4.0),
            ScrollDelta::new(0.0, 32.0),
            ScrollEventSource::Touch
        ),
        ScrollDelta::new(0.0, 16.0)
    );
    assert_eq!(
        scoped.pre_fling(ScrollVelocity::new(0.0, 32.0)),
        ScrollVelocity::new(0.0, 16.0)
    );
    assert_eq!(
        scoped.post_fling(ScrollVelocity::new(0.0, 32.0)),
        ScrollVelocity::new(0.0, 16.0)
    );
    assert_eq!(
        OBSERVED.with_borrow(Clone::clone),
        [
            (true, 0, 40.0),
            (false, 0, 16.0),
            (true, 1, 40.0),
            (false, 1, 16.0),
            (true, 2, 40.0),
            (false, 2, 16.0),
            (true, 3, 40.0),
            (false, 3, 16.0),
        ]
    );
}
