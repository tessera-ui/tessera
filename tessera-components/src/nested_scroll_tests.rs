use tessera_ui::{
    CallbackWith, LocalToWindow, PxPosition, ScrollEventSource, remember, tessera,
    testing::TestProbe,
};

use super::*;

fn handlers(
    connection: NestedScrollConnection,
    inner: bool,
    observed: TestProbe<Vec<(bool, u8, f32)>>,
) -> NestedScrollConnection {
    let pre_scroll = observed.clone();
    let post_scroll = observed.clone();
    let pre_fling = observed.clone();
    connection
        .with_pre_scroll_handler(CallbackWith::new(move |input: PreScrollInput| {
            pre_scroll.with_mut(|items| items.push((inner, 0, input.available.y)));
            ScrollDelta::new(0.0, if inner { 8.0 } else { 6.0 })
        }))
        .with_post_scroll_handler(CallbackWith::new(move |input: PostScrollInput| {
            post_scroll.with_mut(|items| items.push((inner, 1, input.available.y)));
            ScrollDelta::new(0.0, if inner { 8.0 } else { 6.0 })
        }))
        .with_pre_fling_handler(CallbackWith::new(move |input: PreFlingInput| {
            pre_fling.with_mut(|items| items.push((inner, 2, input.available.y)));
            ScrollVelocity::new(0.0, if inner { 8.0 } else { 6.0 })
        }))
        .with_post_fling_handler(CallbackWith::new(move |available: ScrollVelocity| {
            observed.with_mut(|items| items.push((inner, 3, available.y)));
            ScrollVelocity::new(0.0, if inner { 8.0 } else { 6.0 })
        }))
}

#[tessera]
fn connection_fixture(
    connection: TestProbe<Option<NestedScrollConnection>>,
    observed: TestProbe<Vec<(bool, u8, f32)>>,
) {
    let outer_scale = remember(|| 1.0_f32);
    let inner_scale = remember(|| 0.5_f32);
    let outer = handlers(
        NestedScrollConnection::new().with_coordinate_scale(outer_scale),
        false,
        observed.clone(),
    );
    let inner = handlers(
        NestedScrollConnection::new()
            .with_coordinate_scale(inner_scale)
            .with_parent(Some(outer)),
        true,
        observed,
    );
    connection.set(Some(inner));
    tessera_ui::layout::layout();
}

#[test]
fn nested_scroll_consumption_and_fling_cross_each_providers_local_space_once() {
    use tessera_ui::testing::{layout_test, pump_layout_test_session, start_layout_test_session};

    let connection = TestProbe::default();
    let observed = TestProbe::default();
    let fixture_connection = connection.clone();
    let fixture_observed = observed.clone();
    let mut session = start_layout_test_session(
        layout_test(move || {
            connection_fixture(fixture_connection.clone(), fixture_observed.clone());
        })
        .viewport_px(100, 100),
    );
    pump_layout_test_session(&mut session, 0);
    let connection = connection.get().expect("Connection fixture");
    let transform = LocalToWindow::new(0.625, PxPosition::from_f32_xy(10.25, 20.5)).unwrap();
    let scoped = connection.in_space(transform);
    observed.with_mut(Vec::clear);
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
        observed.get(),
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

#[tessera]
fn observe_ancestor(observed: TestProbe<Option<NestedScrollConnection>>) {
    observed.set(NestedScrollConnection::nearest());
    tessera_ui::layout::layout();
}

#[tessera]
fn replaceable_scroll_owner(
    mode: tessera_ui::State<u8>,
    first: NestedScrollConnection,
    second: NestedScrollConnection,
    content: Option<tessera_ui::RenderSlot>,
) {
    let content = content.unwrap_or_else(tessera_ui::RenderSlot::empty);
    let modifier = match mode.get() {
        0 => tessera_ui::Modifier::new().nested_scroll(first),
        1 => tessera_ui::Modifier::new().nested_scroll(second),
        _ => tessera_ui::Modifier::new(),
    };
    tessera_ui::layout::layout()
        .modifier(modifier)
        .child_shared(content);
}

#[tessera]
fn ancestor_fixture(
    observed: TestProbe<Option<NestedScrollConnection>>,
    mode_observed: TestProbe<Option<tessera_ui::State<u8>>>,
) {
    let mode = remember(|| 2_u8);
    mode_observed.set(Some(mode));
    let first = NestedScrollConnection::new().with_pre_scroll_handler(CallbackWith::new(
        |_: PreScrollInput| ScrollDelta::new(0.0, 3.0),
    ));
    let second = NestedScrollConnection::new().with_pre_scroll_handler(CallbackWith::new(
        |_: PreScrollInput| ScrollDelta::new(0.0, 7.0),
    ));
    let content = tessera_ui::RenderSlot::new(move || observe_ancestor(observed.clone()));
    replaceable_scroll_owner(mode, first, second).content_shared(content);
}

#[test]
fn changing_attached_scroll_ancestors_refreshes_retained_readers() {
    use tessera_ui::testing::{layout_test, pump_layout_test_session, start_layout_test_session};

    let observed = TestProbe::default();
    let mode = TestProbe::default();
    let fixture_observed = observed.clone();
    let fixture_mode = mode.clone();
    let mut session = start_layout_test_session(layout_test(move || {
        ancestor_fixture(fixture_observed.clone(), fixture_mode.clone());
    }));
    pump_layout_test_session(&mut session, 0);
    assert!(observed.get().is_none());
    for (index, value) in [0_u8, 1, 2, 0].into_iter().enumerate() {
        session.run(|| mode.get().unwrap().set(value));
        pump_layout_test_session(&mut session, (index as u64 + 1) * 16_666_667);
        session.run(|| {
            let actual = observed.get().map(|connection| {
                connection
                    .pre_scroll(ScrollDelta::new(0.0, 10.0), ScrollEventSource::Touch)
                    .y
            });
            assert_eq!(
                actual,
                match value {
                    0 => Some(3.0),
                    1 => Some(7.0),
                    _ => None,
                }
            );
        });
    }
}
