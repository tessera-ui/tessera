use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{Dp, MOUSE_POINTER_ID, Modifier, PxPosition, tessera, testing::UiTest};

use crate::{column::column, modifier::ModifierExt as _, spacer::spacer};

use super::scroll_view;

#[tessera]
fn row_item(index: usize) {
    spacer().modifier(
        Modifier::new()
            .size(Dp(100.0), Dp(20.0))
            .semantics(SemanticsArgs {
                test_tag: Some(format!("row-{index}")),
                ..Default::default()
            }),
    );
}

#[tessera]
fn fixture() {
    column().children(|| {
        scroll_view()
            .modifier(Modifier::new().size(Dp(100.0), Dp(100.0)))
            .child(|| {
                for index in 0..10 {
                    row_item(index);
                }
            });
    });
}

fn top(ui: &mut UiTest, index: usize) -> i32 {
    ui.frame().node(&format!("row-{index}")).bounds().y.0
}

#[test]
fn wheel_scrolling_moves_the_stacked_content_and_stops_at_the_end() {
    let mut ui = UiTest::new().viewport(200, 200).content(fixture);
    ui.frame();
    assert_eq!((top(&mut ui, 0), top(&mut ui, 1)), (0, 20));
    let inside = PxPosition::from_f32_xy(50.0, 50.0);
    ui.pointer_scroll(MOUSE_POINTER_ID, inside, 0.0, -30.0);
    ui.frames(2);
    assert_eq!(top(&mut ui, 0), -30);
    ui.pointer_scroll(MOUSE_POINTER_ID, inside, 0.0, -500.0);
    ui.frames(2);
    assert_eq!(top(&mut ui, 9), 80, "the last row rests at the bottom edge");
    ui.pointer_scroll(MOUSE_POINTER_ID, inside, 0.0, 1000.0);
    ui.frames(2);
    assert_eq!(top(&mut ui, 0), 0);
}

fn check_scroller_fades() {
    let mut ui = UiTest::new()
        .viewport(200, 200)
        .rendered(crate::TesseraComponents)
        .content(fixture);
    ui.frames(2);
    let rest = ui.read_pixels();
    ui.pointer_scroll(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(50.0, 50.0),
        0.0,
        -30.0,
    );
    ui.frames(3);
    let thumb = ui.pixel(95, 50);
    assert!(
        thumb[3] > 0,
        "the scroller appears while scrolling: {thumb:?}"
    );
    ui.advance(Duration::from_millis(1500));
    ui.frames(30);
    assert_eq!(
        ui.pixel(95, 50)[3],
        rest[(50 * 200 + 95) * 4 + 3],
        "the scroller fades out"
    );
}

#[test]
#[ignore = "requires a native GPU adapter for scroller pixels"]
fn scrollers_appear_while_scrolling_and_fade_after_a_pause() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::scroll_view::tests::scrollers_appear_while_scrolling_and_fade_after_a_pause",
        check_scroller_fades,
    );
}

#[tessera]
fn dark_content() {
    column().children(|| {
        scroll_view()
            .modifier(Modifier::new().size(Dp(100.0), Dp(100.0)))
            .child(|| {
                spacer().modifier(
                    Modifier::new()
                        .size(Dp(100.0), Dp(300.0))
                        .background(tessera_ui::Color::BLACK),
                );
            });
    });
}

fn check_soft_edge() {
    let mut ui = UiTest::new()
        .viewport(200, 200)
        .rendered(crate::TesseraComponents)
        .content(dark_content);
    ui.frames(2);
    assert!(
        ui.pixel(50, 1)[0] < 10,
        "unscrolled content shows no edge effect"
    );
    ui.pointer_scroll(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(50.0, 50.0),
        0.0,
        -40.0,
    );
    ui.frames(3);
    assert!(
        ui.pixel(50, 1)[0] > 150,
        "scrolled content fades under the top edge"
    );
    assert!(ui.pixel(50, 60)[0] < 10, "the fade stays near the edge");
}

#[test]
#[ignore = "requires a native GPU adapter for edge effect pixels"]
fn scrolled_content_fades_under_the_top_edge() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::scroll_view::tests::scrolled_content_fades_under_the_top_edge",
        check_soft_edge,
    );
}
