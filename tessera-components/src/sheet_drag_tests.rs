//! Sheet drag release and cancellation regressions.
//!
//! ## Usage
//!
//! Settle sheets smoothly and retain correct dismissal at different densities.

use std::{
    cell::{Cell, RefCell},
    time::Duration,
};

use tessera_ui::{
    Color, CursorEventContent, Dp, GestureState, HeadlessConfig, HeadlessRenderer,
    MOUSE_POINTER_ID, Modifier, PointerChange, PressKeyEventType, PxPosition, ScrollEventSource,
    State, accesskit::Role, remember, tessera, time::Instant, use_context,
};

use crate::{
    bottom_sheet::bottom_sheet_provider,
    modifier::ModifierExt as _,
    nested_scroll::{NestedScrollConnection, ScrollDelta, ScrollVelocity},
    side_sheet::{SideSheetPosition, modal_side_sheet_provider, standard_side_sheet_provider},
    surface::surface,
    theme::material_theme,
};

#[derive(Clone, Copy, Debug)]
enum Kind {
    Bottom,
    Start,
    End,
    StandardStart,
    StandardEnd,
}

thread_local! {
    static KIND: Cell<Kind> = const { Cell::new(Kind::Bottom) };
    static OPEN: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static EDGE: Cell<Option<State<SideSheetPosition>>> = const { Cell::new(None) };
    static CLOSES: Cell<usize> = const { Cell::new(0) };
    static ACCEPT_CLOSE: Cell<bool> = const { Cell::new(true) };
    static CONNECTION: RefCell<Option<NestedScrollConnection>> = const { RefCell::new(None) };
}

fn request_close(open: State<bool>) {
    CLOSES.set(CLOSES.get() + 1);
    if ACCEPT_CLOSE.get() {
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
fn content() {
    CONNECTION.set(use_context::<NestedScrollConnection>().map(|c| c.get()));
    surface()
        .style(Color::RED.into())
        .modifier(Modifier::new().size(Dp(200.0), Dp(120.0)))
        .accessibility_role(Role::Group)
        .accessibility_label("Drag marker")
        .child(|| {});
}

#[tessera]
fn fixture() {
    let open = remember(|| true);
    let edge = remember(|| {
        if matches!(KIND.get(), Kind::Start | Kind::StandardStart) {
            SideSheetPosition::Start
        } else {
            SideSheetPosition::End
        }
    });
    OPEN.set(Some(open));
    EDGE.set(Some(edge));
    material_theme().child(move || match KIND.get() {
        Kind::Bottom => {
            bottom_sheet_provider()
                .is_open(open.get())
                .on_close_request(move || request_close(open))
                .main_content(|| {
                    main_content();
                })
                .bottom_sheet_content(|| {
                    content();
                });
        }
        kind => {
            let position = edge.get();
            if matches!(kind, Kind::StandardStart | Kind::StandardEnd) {
                standard_side_sheet_provider()
                    .position(position)
                    .is_open(open.get())
                    .on_close_request(move || request_close(open))
                    .main_content(|| {
                        main_content();
                    })
                    .side_sheet_content(|| {
                        content();
                    });
            } else {
                modal_side_sheet_provider()
                    .position(position)
                    .is_open(open.get())
                    .on_close_request(move || request_close(open))
                    .main_content(|| {
                        main_content();
                    })
                    .side_sheet_content(|| {
                        content();
                    });
            }
        }
    });
}

fn bounds(renderer: &HeadlessRenderer) -> tessera_ui::accesskit::Rect {
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

fn connection() -> NestedScrollConnection {
    CONNECTION.with_borrow(|c| c.clone().unwrap())
}

fn assert_reused_measurements(renderer: &HeadlessRenderer, kind: Kind) {
    let diagnostics = renderer.diagnostics();
    assert_eq!(
        (
            diagnostics.cache_miss_no_entry,
            diagnostics.cache_miss_constraint,
            diagnostics.cache_miss_dirty_self,
            diagnostics.cache_miss_child_size,
        ),
        (0, 0, 0, 0),
        "{kind:?}: fixed sheet geometry must reuse measurements: {diagnostics:?}"
    );
}

fn drag_handle(renderer: &mut HeadlessRenderer, scale: f64) -> PxPosition {
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
    renderer.step_frame();
    renderer.pointer_move(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(start.x, start.y + 4.0 * scale as f32),
    );
    renderer.step_frame();
    let end = PxPosition::from_f32_xy(start.x, start.y + 40.0 * scale as f32);
    renderer.pointer_move(MOUSE_POINTER_ID, end);
    renderer.render_frames(2);
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
        KIND.set(kind);
        for scale in [1.0, 1.5] {
            CLOSES.set(0);
            ACCEPT_CLOSE.set(true);
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 800,
                    height: 600,
                    scale_factor: scale,
                    sample_count: samples,
                    frame_time: Duration::from_millis(10),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(10);
            let origin = bounds(&renderer);
            let baseline = renderer.read_pixels().unwrap();
            assert_reused_measurements(&renderer, kind);
            let short = if matches!(kind, Kind::Bottom) {
                40.0
            } else {
                150.0
            };
            let delta = scroll(kind, short * scale as f32);
            assert_eq!(
                connection().post_scroll(ScrollDelta::ZERO, delta, ScrollEventSource::Touch),
                delta
            );
            renderer.step_frame();
            assert!(
                (displacement(kind, origin, bounds(&renderer)) - short as f64 * scale).abs() < 0.01
            );
            assert_reused_measurements(&renderer, kind);
            connection().pre_fling(ScrollVelocity::ZERO);
            renderer.step_frame();
            let after_release = displacement(kind, origin, bounds(&renderer));
            assert!(
                after_release > 0.0 && after_release < short as f64 * scale,
                "{kind:?}: release must animate rather than snap"
            );
            assert_reused_measurements(&renderer, kind);
            renderer.render_frames(80);
            assert_eq!(CLOSES.get(), 0);
            assert_eq!(bounds(&renderer), origin);
            assert!(renderer.read_pixels().unwrap() == baseline);

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
                renderer.render_frames(2);
                let cancelled_offset = displacement(kind, origin, bounds(&renderer));
                assert!(cancelled_offset > 0.0 && cancelled_offset < 36.0 * scale);
                renderer.render_frames(80);
                assert_eq!(CLOSES.get(), 0);
                assert_eq!(bounds(&renderer), origin);
            }

            let long = if matches!(kind, Kind::Bottom) {
                64.0
            } else {
                210.0
            };
            connection().post_scroll(
                ScrollDelta::ZERO,
                scroll(kind, long * scale as f32),
                ScrollEventSource::Touch,
            );
            renderer.step_frame();
            let dragged = bounds(&renderer);
            connection().pre_fling(ScrollVelocity::ZERO);
            renderer.step_frame();
            assert_eq!(
                CLOSES.get(),
                1,
                "{kind:?}: dismissal threshold must follow density/extent"
            );
            assert!(displacement(kind, dragged, bounds(&renderer)).abs() < 32.0 * scale);
            renderer.render_frames(5);
            let closing = bounds(&renderer);
            OPEN.get().unwrap().set(true);
            renderer.step_frame();
            assert!(
                displacement(kind, closing, bounds(&renderer)).abs() < 32.0 * scale,
                "{kind:?}: reopening must preserve drag displacement"
            );
            renderer.render_frames(80);
            assert_eq!(bounds(&renderer), origin);
            assert!(renderer.read_pixels().unwrap() == baseline);

            connection().post_scroll(
                ScrollDelta::ZERO,
                scroll(kind, 8.0 * scale as f32),
                ScrollEventSource::Touch,
            );
            renderer.step_frame();
            let speed = if matches!(kind, Kind::Bottom) {
                150.0
            } else {
                600.0
            } * scale as f32;
            let velocity = scroll(kind, speed);
            connection().pre_fling(ScrollVelocity::new(velocity.x, velocity.y));
            renderer.step_frame();
            assert_eq!(
                CLOSES.get(),
                2,
                "{kind:?}: fast outward flings must dismiss"
            );
            renderer.render_frames(80);
            assert!(
                !renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, n)| n.role() == Role::Dialog)
            );
            OPEN.get().unwrap().set(true);
            renderer.render_frames(80);
            assert!(renderer.read_pixels().unwrap() == baseline);

            connection().post_scroll(
                ScrollDelta::ZERO,
                scroll(kind, long * scale as f32),
                ScrollEventSource::Touch,
            );
            renderer.step_frame();
            connection().pre_fling(ScrollVelocity::new(-velocity.x, -velocity.y));
            renderer.render_frames(80);
            assert_eq!(
                CLOSES.get(),
                2,
                "{kind:?}: fast inward flings must return to the open anchor"
            );
            assert!(renderer.read_pixels().unwrap() == baseline);

            ACCEPT_CLOSE.set(false);
            connection().post_scroll(
                ScrollDelta::ZERO,
                scroll(kind, long * scale as f32),
                ScrollEventSource::Touch,
            );
            renderer.step_frame();
            connection().pre_fling(ScrollVelocity::ZERO);
            renderer.render_frames(80);
            assert_eq!(CLOSES.get(), 3);
            assert!(OPEN.get().unwrap().get());
            assert!(
                renderer.read_pixels().unwrap() == baseline,
                "{kind:?}: a declined dismissal must settle back"
            );
            if !matches!(kind, Kind::Bottom) {
                ACCEPT_CLOSE.set(true);
                renderer.resize(320, 600);
                renderer.render_frames(10);
                let narrow = bounds(&renderer);
                connection().post_scroll(
                    ScrollDelta::ZERO,
                    scroll(kind, 90.0 * scale as f32),
                    ScrollEventSource::Touch,
                );
                renderer.step_frame();
                connection().pre_fling(ScrollVelocity::ZERO);
                renderer.render_frames(80);
                assert_eq!(CLOSES.get(), 3);
                assert_eq!(bounds(&renderer), narrow);
                connection().post_scroll(
                    ScrollDelta::ZERO,
                    scroll(kind, 180.0 * scale as f32),
                    ScrollEventSource::Touch,
                );
                renderer.step_frame();
                connection().pre_fling(ScrollVelocity::ZERO);
                renderer.render_frames(80);
                assert_eq!(
                    CLOSES.get(),
                    4,
                    "{kind:?}: thresholds must use the resized sheet width"
                );
                assert!(!OPEN.get().unwrap().get());
                let (edge, reversed) = match kind {
                    Kind::Start => (SideSheetPosition::End, Kind::End),
                    Kind::End => (SideSheetPosition::Start, Kind::Start),
                    Kind::StandardStart => (SideSheetPosition::End, Kind::StandardEnd),
                    _ => (SideSheetPosition::Start, Kind::StandardStart),
                };
                EDGE.get().unwrap().set(edge);
                OPEN.get().unwrap().set(true);
                renderer.render_frames(80);
                let flipped = bounds(&renderer);
                let delta = scroll(reversed, 20.0 * scale as f32);
                assert_eq!(
                    connection().post_scroll(ScrollDelta::ZERO, delta, ScrollEventSource::Touch),
                    delta
                );
                renderer.step_frame();
                assert!(
                    (displacement(reversed, flipped, bounds(&renderer)) - 20.0 * scale).abs()
                        < 0.01
                );
                connection().pre_fling(ScrollVelocity::ZERO);
                renderer.render_frames(80);
                assert_eq!(CLOSES.get(), 4);
                assert_eq!(bounds(&renderer), flipped);
            } else {
                ACCEPT_CLOSE.set(true);
                for width in [600.0, 700.0] {
                    let width = (width * scale) as u32;
                    renderer.resize(width, 600);
                    renderer.step_frame();
                    assert!(
                        renderer.diagnostics().cache_miss_constraint > 0,
                        "resizing across the sheet width limit must update constraints"
                    );
                    renderer.render_frames(10);
                    let resized = bounds(&renderer);
                    assert_eq!(resized.width(), origin.width());
                    assert_eq!(resized.height(), origin.height());
                    assert_eq!(resized.y1, origin.y1);
                    assert!((resized.x0 - (f64::from(width) - resized.width()) / 2.0).abs() < 0.01);
                    let resized_pixels = renderer.read_pixels().unwrap();
                    connection().post_scroll(
                        ScrollDelta::ZERO,
                        scroll(kind, 20.0 * scale as f32),
                        ScrollEventSource::Touch,
                    );
                    renderer.step_frame();
                    assert!(
                        (displacement(kind, resized, bounds(&renderer)) - 20.0 * scale).abs()
                            < 0.01
                    );
                    assert_reused_measurements(&renderer, kind);
                    connection().pre_fling(ScrollVelocity::ZERO);
                    renderer.render_frames(80);
                    assert_eq!(bounds(&renderer), resized);
                    assert_eq!(CLOSES.get(), 3);
                    assert!(renderer.read_pixels().unwrap() == resized_pixels);
                }
                renderer.resize(800, 600);
                renderer.render_frames(10);
                assert_eq!(bounds(&renderer), origin);
                assert!(renderer.read_pixels().unwrap() == baseline);
                let end = drag_handle(&mut renderer, scale);
                renderer.render_frames(10);
                renderer.pointer_release(MOUSE_POINTER_ID, end, PressKeyEventType::Left);
                renderer.render_frames(2);
                assert_eq!(CLOSES.get(), 3);
                let released = displacement(kind, origin, bounds(&renderer));
                assert!(released > 0.0 && released < 36.0 * scale);
                renderer.render_frames(80);
                assert!(renderer.read_pixels().unwrap() == baseline);
                let end = drag_handle(&mut renderer, scale);
                renderer.pointer_release(MOUSE_POINTER_ID, end, PressKeyEventType::Left);
                renderer.render_frames(2);
                assert_eq!(
                    CLOSES.get(),
                    4,
                    "a fast handle fling must dismiss below the distance threshold"
                );
                renderer.render_frames(80);
                assert!(!OPEN.get().unwrap().get());
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
