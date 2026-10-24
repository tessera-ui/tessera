//! Scroll extents reconciled during responsive layout changes.
//!
//! ## Usage
//!
//! Keep an existing scroll position valid when content shrinks after resizing.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    CursorEventContent, Dp, FocusModifierExt as _, GestureState, HeadlessConfig, HeadlessRenderer,
    LayoutCoordinates, MOUSE_POINTER_ID, Modifier, PointerChange, Px, PxPosition, ScrollDeltaUnit,
    ScrollEventContent, ScrollEventSource, State, current_frame_time, remember, scaled_box,
    tessera,
};

use crate::{
    alignment::Alignment, boxed::boxed, column::column, modifier::ModifierExt as _, spacer::spacer,
    theme::material_theme,
};

use super::ScrollableController;

thread_local! {
    static SCROLL: Cell<Option<State<ScrollableController>>> = const { Cell::new(None) };
    static SCENE: Cell<Option<LayoutCoordinates>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let scroll = remember(ScrollableController::new);
    let width = remember(|| 1024_i32);
    SCROLL.set(Some(scroll));
    material_theme().child(move || {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size().on_layout_coordinates(
                move |coordinates: LayoutCoordinates| {
                    let next = coordinates.local_size.width.0;
                    if width.get() != next {
                        width.set(next);
                    }
                },
            ))
            .children(move || {
                let scale = (width.get() as f32 / 1024.0).min(1.0);
                column()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .height(Dp(320.0))
                            .vertical_scroll(scroll),
                    )
                    .children(move || {
                        spacer().modifier(Modifier::new().height(Dp(48.0)));
                        spacer().modifier(Modifier::new().height(Dp(600.0)));
                        scaled_box(scale).child(|| {
                            spacer().modifier(
                                Modifier::new()
                                    .width(Dp(800.0))
                                    .height(Dp(300.0))
                                    .on_layout_coordinates(|coordinates| {
                                        SCENE.set(Some(coordinates))
                                    }),
                            );
                        });
                    });
            });
    });
}

#[test]
#[ignore = "requires a native GPU adapter for responsive scroll extent publication"]
fn wheel_bottom_is_clamped_during_resize_before_further_input() {
    crate::test_utils::run_isolated_test(
        "scrollable::extent_tests::wheel_bottom_is_clamped_during_resize_before_further_input",
        || {
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 1024,
                    height: 800,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .expect("Native GPU adapter");
            renderer.render_frames(12);
            let scroll = SCROLL.get().expect("Scrollable controller");
            assert_eq!(scroll.with(|scroll| scroll.child_size().height), Px(948));
            assert_eq!(scroll.with(|scroll| scroll.visible_size().height), Px(320));
            renderer.inject_pointer(
                PointerChange::new(
                    current_frame_time().expect("Explicit headless frame timestamp"),
                    MOUSE_POINTER_ID,
                    CursorEventContent::Scroll(ScrollEventContent {
                        delta_x: 0.0,
                        delta_y: -10_000.0,
                        unit: ScrollDeltaUnit::Pixel,
                        source: ScrollEventSource::Wheel,
                    }),
                    GestureState::default(),
                )
                .with_position(PxPosition::from_f32_xy(40.0, 100.0)),
            );
            renderer.render_frames(120);
            assert_eq!(
                scroll.with(|scroll| scroll.child_position().y),
                -628.0,
                "Real wheel input must establish a valid old bottom without a programmatic setter"
            );
            renderer.resize(640, 1600);
            renderer.render_frames(12);
            assert_eq!(scroll.with(|scroll| scroll.child_size().height), Px(836));
            assert_eq!(
                SCENE
                    .get()
                    .expect("Resized native scene")
                    .local_to_window
                    .scale(),
                0.625
            );
            assert_eq!(
                scroll.with(|scroll| scroll.child_position().y),
                -516.0,
                "Resize must clamp the old valid offset before a later click or focus reveal"
            );
            assert_eq!(scroll.with(|scroll| scroll.target_position().y), -516.0);
            let scene = SCENE.get().expect("Resized native scene").window_bounds();
            assert_eq!(scene.y, 132.0);
            assert_eq!(scene.y + scene.height, 319.5);
            renderer.render_frames(12);
            assert_eq!(
                scroll.with(|scroll| scroll.child_position().y),
                -516.0,
                "The reconciled bottom must remain stable without input"
            );
        },
    );
}
