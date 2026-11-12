//! Sheet drag release and cancellation regressions.
//!
//! ## Usage
//!
//! Settle sheets smoothly and retain correct dismissal at different densities.

use std::time::Duration;

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, CursorEventContent, Dp, GestureState, MOUSE_POINTER_ID, Modifier, PointerChange,
    PressKeyEventType, PxPosition, ScrollEventSource, State, accesskit::Role, remember, tessera,
    testing::UiTest, time::Instant,
};

use crate::{
    bottom_sheet::{BottomSheetController, bottom_sheet_provider},
    nested_scroll::{NestedScrollConnection, ScrollDelta, ScrollVelocity},
    side_sheet::{
        SideSheetController, SideSheetPosition, modal_side_sheet_provider,
        standard_side_sheet_provider,
    },
    surface::surface,
    test_utils::OverlayController,
    theme::material_theme,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Bottom,
    Start,
    End,
    StandardStart,
    StandardEnd,
}

#[derive(Clone, Default)]
struct FixtureHandles {
    open: Option<OverlayController>,
    edge: Option<State<SideSheetPosition>>,
    accept_close: Option<State<bool>>,
    closes: usize,
    connection: Option<NestedScrollConnection>,
}

fn request_close(
    open: OverlayController,
    accept_close: State<bool>,
    observed: tessera_ui::testing::TestProbe<FixtureHandles>,
) {
    observed.with_mut(|handles| handles.closes += 1);
    if accept_close.get() {
        open.set(false);
    }
}

#[tessera]
fn main_content() {
    surface()
        .style(Color::WHITE.into())
        .modifier(Modifier::new().fill_max_size())
        .child(|| {});
}

#[tessera]
fn content(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    observed.with_mut(|handles| handles.connection = NestedScrollConnection::nearest());
    surface()
        .style(Color::RED.into())
        .modifier(Modifier::new().size(Dp(200.0), Dp(120.0)))
        .accessibility_role(Role::Group)
        .accessibility_label("Drag marker")
        .child(|| {});
}

#[tessera]
fn fixture(kind: Kind, observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let accept_close = remember(|| true);
    observed.with_mut(|handles| handles.accept_close = Some(accept_close));
    let open = match kind {
        Kind::Bottom => OverlayController::Bottom(remember(|| BottomSheetController::new(true))),
        _ => OverlayController::Side(remember(|| SideSheetController::new(true))),
    };
    let edge = remember(|| {
        if matches!(kind, Kind::Start | Kind::StandardStart) {
            SideSheetPosition::Start
        } else {
            SideSheetPosition::End
        }
    });
    observed.with_mut(|handles| handles.open = Some(open));
    observed.with_mut(|handles| handles.edge = Some(edge));
    material_theme().child(move || match open {
        OverlayController::Bottom(controller) => {
            bottom_sheet_provider()
                .controller(controller)
                .on_close_request({
                    let observed = observed.clone();
                    move || request_close(open, accept_close, observed.clone())
                })
                .main_content(|| {
                    main_content();
                })
                .bottom_sheet_content({
                    let observed = observed.clone();
                    move || {
                        content(observed.clone());
                    }
                });
        }
        OverlayController::Side(controller) => {
            let position = edge.get();
            if matches!(kind, Kind::StandardStart | Kind::StandardEnd) {
                standard_side_sheet_provider()
                    .position(position)
                    .controller(controller)
                    .on_close_request({
                        let observed = observed.clone();
                        move || request_close(open, accept_close, observed.clone())
                    })
                    .main_content(|| {
                        main_content();
                    })
                    .side_sheet_content({
                        let observed = observed.clone();
                        move || {
                            content(observed.clone());
                        }
                    });
            } else {
                modal_side_sheet_provider()
                    .position(position)
                    .controller(controller)
                    .on_close_request({
                        let observed = observed.clone();
                        move || request_close(open, accept_close, observed.clone())
                    })
                    .main_content(|| {
                        main_content();
                    })
                    .side_sheet_content({
                        let observed = observed.clone();
                        move || {
                            content(observed.clone());
                        }
                    });
            }
        }
        OverlayController::Dialog(_) => unreachable!(),
    });
}

fn bounds(renderer: &UiTest) -> tessera_ui::accesskit::Rect {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, n)| n.label() == Some("Drag marker"))
        .unwrap()
        .1
        .bounds()
        .unwrap()
}

fn scroll(kind: Kind, distance: f32) -> ScrollDelta {
    match kind {
        Kind::Bottom => ScrollDelta::new(0.0, distance),
        Kind::Start | Kind::StandardStart => ScrollDelta::new(-distance, 0.0),
        _ => ScrollDelta::new(distance, 0.0),
    }
}

fn displacement(
    kind: Kind,
    from: tessera_ui::accesskit::Rect,
    to: tessera_ui::accesskit::Rect,
) -> f64 {
    match kind {
        Kind::Bottom => to.y0 - from.y0,
        Kind::Start | Kind::StandardStart => from.x0 - to.x0,
        _ => to.x0 - from.x0,
    }
}

fn connection(observed: &tessera_ui::testing::TestProbe<FixtureHandles>) -> NestedScrollConnection {
    observed.get().connection.unwrap()
}

fn assert_reused_measurements(renderer: &UiTest, kind: Kind) {
    assert!(
        renderer
            .work()
            .phase(tessera_ui::testing::WorkPhase::Measure)
            .next()
            .is_none(),
        "{kind:?}: fixed sheet geometry must reuse measurements: {:?}",
        renderer.work()
    );
}

fn drag_handle(renderer: &mut UiTest, scale: f64) -> PxPosition {
    let handle = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, n)| n.label() == Some("Drag handle"))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let start = PxPosition::from_f32_xy(
        ((handle.x0 + handle.x1) * 0.5) as f32,
        (handle.y0 + 24.0 * scale) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, start);
    renderer.pointer_press(MOUSE_POINTER_ID, start, PressKeyEventType::Left);
    renderer.frames(1);
    renderer.pointer_move(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(start.x, start.y + 4.0 * scale as f32),
    );
    renderer.frames(1);
    let end = PxPosition::from_f32_xy(start.x, start.y + 40.0 * scale as f32);
    renderer.pointer_move(MOUSE_POINTER_ID, end);
    renderer.frames(2);
    end
}

fn check(samples: u32) {
    for kind in [
        Kind::Bottom,
        Kind::Start,
        Kind::End,
        Kind::StandardStart,
        Kind::StandardEnd,
    ] {
        for scale in [1.0, 1.5] {
            let observed = tessera_ui::testing::TestProbe::default();
            let mut renderer = UiTest::new()
                .viewport(800, 600)
                .scale_factor(scale)
                .samples(samples)
                .frame_step(Duration::from_millis(10))
                .rendered(crate::TesseraComponents)
                .content(|| fixture(kind, observed.clone()));
            renderer.frames(10);
            let origin = bounds(&renderer);
            let baseline = renderer.read_pixels();
            assert_reused_measurements(&renderer, kind);
            let short = if matches!(kind, Kind::Bottom) {
                40.0
            } else {
                150.0
            };
            let delta = scroll(kind, short * scale as f32);
            assert_eq!(
                connection(&observed).post_scroll(
                    ScrollDelta::ZERO,
                    delta,
                    ScrollEventSource::Touch
                ),
                delta
            );
            renderer.frames(1);
            assert!(
                (displacement(kind, origin, bounds(&renderer)) - short as f64 * scale).abs() < 0.01
            );
            assert_reused_measurements(&renderer, kind);
            connection(&observed).pre_fling(ScrollVelocity::ZERO);
            renderer.frames(1);
            let after_release = displacement(kind, origin, bounds(&renderer));
            assert!(
                after_release > 0.0 && after_release < short as f64 * scale,
                "{kind:?}: release must animate rather than snap"
            );
            assert_reused_measurements(&renderer, kind);
            renderer.frames(80);
            assert_eq!(observed.get().closes, 0);
            assert_eq!(bounds(&renderer), origin);
            assert!(renderer.read_pixels() == baseline);

            if matches!(kind, Kind::Bottom) {
                drag_handle(&mut renderer, scale);
                assert!(
                    (displacement(kind, origin, bounds(&renderer)) - 36.0 * scale).abs() < 0.01,
                    "origin {origin:?}, actual {:?}",
                    bounds(&renderer)
                );
                renderer.inject_pointer(PointerChange::new(
                    Instant::now(),
                    MOUSE_POINTER_ID,
                    CursorEventContent::Cancelled(PressKeyEventType::Left),
                    GestureState::TapCandidate,
                ));
                renderer.frames(2);
                let cancelled_offset = displacement(kind, origin, bounds(&renderer));
                assert!(cancelled_offset > 0.0 && cancelled_offset < 36.0 * scale);
                renderer.frames(80);
                assert_eq!(observed.get().closes, 0);
                assert_eq!(bounds(&renderer), origin);
            }

            let long = if matches!(kind, Kind::Bottom) {
                64.0
            } else {
                210.0
            };
            connection(&observed).post_scroll(
                ScrollDelta::ZERO,
                scroll(kind, long * scale as f32),
                ScrollEventSource::Touch,
            );
            renderer.frames(1);
            let dragged = bounds(&renderer);
            connection(&observed).pre_fling(ScrollVelocity::ZERO);
            renderer.frames(1);
            assert_eq!(
                observed.get().closes,
                1,
                "{kind:?}: dismissal threshold must follow density/extent"
            );
            assert!(displacement(kind, dragged, bounds(&renderer)).abs() < 32.0 * scale);
            renderer.frames(5);
            let closing = bounds(&renderer);
            observed.get().open.unwrap().set(true);
            renderer.frames(1);
            assert!(
                displacement(kind, closing, bounds(&renderer)).abs() < 32.0 * scale,
                "{kind:?}: reopening must preserve drag displacement"
            );
            renderer.frames(80);
            assert_eq!(bounds(&renderer), origin);
            assert!(renderer.read_pixels() == baseline);

            connection(&observed).post_scroll(
                ScrollDelta::ZERO,
                scroll(kind, 8.0 * scale as f32),
                ScrollEventSource::Touch,
            );
            renderer.frames(1);
            let speed = if matches!(kind, Kind::Bottom) {
                150.0
            } else {
                600.0
            } * scale as f32;
            let velocity = scroll(kind, speed);
            connection(&observed).pre_fling(ScrollVelocity::new(velocity.x, velocity.y));
            renderer.frames(1);
            assert_eq!(
                observed.get().closes,
                2,
                "{kind:?}: fast outward flings must dismiss"
            );
            renderer.frames(80);
            assert!(
                !renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, n)| n.role() == Role::Dialog)
            );
            observed.get().open.unwrap().set(true);
            renderer.frames(80);
            assert!(renderer.read_pixels() == baseline);

            connection(&observed).post_scroll(
                ScrollDelta::ZERO,
                scroll(kind, long * scale as f32),
                ScrollEventSource::Touch,
            );
            renderer.frames(1);
            connection(&observed).pre_fling(ScrollVelocity::new(-velocity.x, -velocity.y));
            renderer.frames(80);
            assert_eq!(
                observed.get().closes,
                2,
                "{kind:?}: fast inward flings must return to the open anchor"
            );
            assert!(renderer.read_pixels() == baseline);

            observed.get().accept_close.unwrap().set(false);
            connection(&observed).post_scroll(
                ScrollDelta::ZERO,
                scroll(kind, long * scale as f32),
                ScrollEventSource::Touch,
            );
            renderer.frames(1);
            connection(&observed).pre_fling(ScrollVelocity::ZERO);
            renderer.frames(80);
            assert_eq!(observed.get().closes, 3);
            assert!(observed.get().open.unwrap().get());
            assert!(
                renderer.read_pixels() == baseline,
                "{kind:?}: a declined dismissal must settle back"
            );
            if !matches!(kind, Kind::Bottom) {
                observed.get().accept_close.unwrap().set(true);
                renderer.resize(320, 600);
                renderer.frames(10);
                let narrow = bounds(&renderer);
                connection(&observed).post_scroll(
                    ScrollDelta::ZERO,
                    scroll(kind, 90.0 * scale as f32),
                    ScrollEventSource::Touch,
                );
                renderer.frames(1);
                connection(&observed).pre_fling(ScrollVelocity::ZERO);
                renderer.frames(80);
                assert_eq!(observed.get().closes, 3);
                assert_eq!(bounds(&renderer), narrow);
                connection(&observed).post_scroll(
                    ScrollDelta::ZERO,
                    scroll(kind, 180.0 * scale as f32),
                    ScrollEventSource::Touch,
                );
                renderer.frames(1);
                connection(&observed).pre_fling(ScrollVelocity::ZERO);
                renderer.frames(80);
                assert_eq!(
                    observed.get().closes,
                    4,
                    "{kind:?}: thresholds must use the resized sheet width"
                );
                assert!(!observed.get().open.unwrap().get());
                let (edge, reversed) = match kind {
                    Kind::Start => (SideSheetPosition::End, Kind::End),
                    Kind::End => (SideSheetPosition::Start, Kind::Start),
                    Kind::StandardStart => (SideSheetPosition::End, Kind::StandardEnd),
                    _ => (SideSheetPosition::Start, Kind::StandardStart),
                };
                observed.get().edge.unwrap().set(edge);
                observed.get().open.unwrap().set(true);
                renderer.frames(80);
                let flipped = bounds(&renderer);
                let delta = scroll(reversed, 20.0 * scale as f32);
                assert_eq!(
                    connection(&observed).post_scroll(
                        ScrollDelta::ZERO,
                        delta,
                        ScrollEventSource::Touch
                    ),
                    delta
                );
                renderer.frames(1);
                assert!(
                    (displacement(reversed, flipped, bounds(&renderer)) - 20.0 * scale).abs()
                        < 0.01
                );
                connection(&observed).pre_fling(ScrollVelocity::ZERO);
                renderer.frames(80);
                assert_eq!(observed.get().closes, 4);
                assert_eq!(bounds(&renderer), flipped);
            } else {
                observed.get().accept_close.unwrap().set(true);
                for width in [600.0, 700.0] {
                    let width = (width * scale) as u32;
                    renderer.resize(width, 600);
                    renderer.frames(1);
                    assert!(
                        renderer
                            .work()
                            .phase(tessera_ui::testing::WorkPhase::Measure)
                            .any(|work| work.target.ends_with("bottom_sheet::BottomSheetLayout")),
                        "resizing across the sheet width limit must update constraints"
                    );
                    renderer.frames(10);
                    let resized = bounds(&renderer);
                    assert_eq!(resized.width(), origin.width());
                    assert_eq!(resized.height(), origin.height());
                    assert_eq!(resized.y1, origin.y1);
                    assert!((resized.x0 - (f64::from(width) - resized.width()) / 2.0).abs() < 0.01);
                    let resized_pixels = renderer.read_pixels();
                    connection(&observed).post_scroll(
                        ScrollDelta::ZERO,
                        scroll(kind, 20.0 * scale as f32),
                        ScrollEventSource::Touch,
                    );
                    renderer.frames(1);
                    assert!(
                        (displacement(kind, resized, bounds(&renderer)) - 20.0 * scale).abs()
                            < 0.01
                    );
                    assert_reused_measurements(&renderer, kind);
                    connection(&observed).pre_fling(ScrollVelocity::ZERO);
                    renderer.frames(80);
                    assert_eq!(bounds(&renderer), resized);
                    assert_eq!(observed.get().closes, 3);
                    assert!(renderer.read_pixels() == resized_pixels);
                }
                renderer.resize(800, 600);
                renderer.frames(10);
                assert_eq!(bounds(&renderer), origin);
                assert!(renderer.read_pixels() == baseline);
                let end = drag_handle(&mut renderer, scale);
                renderer.frames(10);
                renderer.pointer_release(MOUSE_POINTER_ID, end, PressKeyEventType::Left);
                renderer.frames(2);
                assert_eq!(observed.get().closes, 3);
                let released = displacement(kind, origin, bounds(&renderer));
                assert!(released > 0.0 && released < 36.0 * scale);
                renderer.frames(80);
                assert!(renderer.read_pixels() == baseline);
                let end = drag_handle(&mut renderer, scale);
                renderer.pointer_release(MOUSE_POINTER_ID, end, PressKeyEventType::Left);
                renderer.frames(2);
                assert_eq!(
                    observed.get().closes,
                    4,
                    "a fast handle fling must dismiss below the distance threshold"
                );
                renderer.frames(80);
                assert!(!observed.get().open.unwrap().get());
            }
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn drag_release_cancellation_dismissal_and_reversal_msaa1() {
    crate::test_utils::run_isolated_test(
        "sheet_drag_tests::drag_release_cancellation_dismissal_and_reversal_msaa1",
        || check(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn drag_release_cancellation_dismissal_and_reversal_msaa4() {
    crate::test_utils::run_isolated_test(
        "sheet_drag_tests::drag_release_cancellation_dismissal_and_reversal_msaa4",
        || check(4),
    );
}
