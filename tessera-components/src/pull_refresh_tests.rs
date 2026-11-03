//! Native pull-to-refresh gesture regressions.
//!
//! ## Usage
//!
//! Keep threshold, reversal and custom indicator behavior on real scroll input.

use tessera_ui::{HeadlessConfig, HeadlessRenderer, PxPosition};

use crate::{scrollable::ScrollableController, text::text, theme::material_theme};

use super::*;

#[derive(Clone, Copy, Default)]
struct FixtureHandles {
    refreshing: Option<State<bool>>,
    requests: Option<State<usize>>,
    controller: Option<State<PullRefreshController>>,
}

#[tessera]
fn fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let refreshing = remember(|| false);
    let requests = remember(|| 0_usize);
    let controller = remember(PullRefreshController::new);
    let scroll = remember(ScrollableController::new);
    observed.with_mut(|handles| handles.refreshing = Some(refreshing));
    observed.with_mut(|handles| handles.requests = Some(requests));
    observed.with_mut(|handles| handles.controller = Some(controller));
    material_theme().child(move || {
        boxed()
            .modifier(Modifier::new().fill_max_size())
            .alignment(Alignment::TopStart)
            .children(move || {
                pull_refresh()
                    .modifier(Modifier::new().size(Dp(300.0), Dp(300.0)))
                    .controller(controller)
                    .refreshing(refreshing.get())
                    .indicator_size(Dp(48.0))
                    .on_refresh(move || {
                        requests.with_mut(|requests| *requests += 1);
                        refreshing.set(true);
                    })
                    .indicator(|| {
                        text().content("Custom refresh indicator");
                    })
                    .child(move || {
                        boxed()
                            .modifier(Modifier::new().fill_max_size().vertical_scroll(scroll))
                            .children(|| {
                                boxed()
                                    .modifier(Modifier::new().fill_max_width().height(Dp(600.0)))
                                    .children(|| {
                                        text().content("Refreshable content");
                                    });
                            });
                    });
            });
    });
}

fn drag(renderer: &mut HeadlessRenderer, positions: &[f32]) {
    let point = |y| PxPosition::from_f32_xy(150.0, y);
    renderer.touch_start(1, point(30.0));
    renderer.step_frame();
    for &y in positions {
        renderer.touch_move(1, point(y));
        renderer.step_frame();
    }
    renderer.touch_end(1);
    renderer.render_frames(32);
}

#[test]
#[ignore = "Requires a native GPU adapter"]
fn native_pull_refresh_uses_scroll_threshold_and_reversal_with_custom_indicator() {
    let observed = tessera_ui::testing::TestProbe::default();
    let mut renderer = HeadlessRenderer::new(
        Box::new({
            let observed = observed.clone();
            move || fixture(observed.clone())
        }),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 400,
            height: 400,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    let requests = observed.get().requests.unwrap();
    let refreshing = observed.get().refreshing.unwrap();
    let controller = observed.get().controller.unwrap();
    renderer.touch_start(2, PxPosition::from_f32_xy(150.0, 30.0));
    renderer.step_frame();
    renderer.touch_move(2, PxPosition::from_f32_xy(150.0, 260.0));
    renderer.step_frame();
    assert!(controller.with(PullRefreshController::progress) > 1.0);
    renderer.touch_cancel(2);
    renderer.render_frames(48);
    assert_eq!(requests.get(), 0, "a cancelled contact must not refresh");
    assert_eq!(
        controller.with(PullRefreshController::position),
        Px(0),
        "cancellation retracts the custom indicator"
    );
    drag(&mut renderer, &[60.0, 110.0]);
    assert_eq!(requests.get(), 0, "a short drag must not refresh");
    assert!(!controller.with(PullRefreshController::refreshing));
    assert_eq!(controller.with(PullRefreshController::position), Px(0));
    drag(&mut renderer, &[60.0, 260.0, 80.0]);
    assert_eq!(
        requests.get(),
        0,
        "reversing below the threshold cancels the pull"
    );
    drag(&mut renderer, &[60.0, 270.0]);
    assert_eq!(
        requests.get(),
        1,
        "releasing beyond the threshold refreshes once"
    );
    assert!(refreshing.get());
    assert!(controller.with(PullRefreshController::refreshing));
    let tree = renderer.accessibility_snapshot().unwrap();
    let indicator = tree
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Custom refresh indicator"))
        .expect("the custom indicator is actually rendered");
    let bounds = indicator.1.bounds().unwrap();
    assert!(
        bounds.y0 >= 0.0 && bounds.y1 <= 300.0,
        "custom indicator follows the refresh offset: {bounds:?}"
    );
    drag(&mut renderer, &[60.0, 270.0]);
    assert_eq!(requests.get(), 1, "a running refresh rejects another pull");
    refreshing.set(false);
    renderer.render_frames(48);
    assert!(!controller.with(PullRefreshController::refreshing));
    assert_eq!(controller.with(PullRefreshController::position), Px(0));
}
