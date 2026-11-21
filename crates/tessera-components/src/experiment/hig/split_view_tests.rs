use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, tessera, testing::UiTest,
};

use crate::spacer::spacer;

use super::h_split_view;

#[tessera]
fn filler(tag: String) {
    spacer().modifier(Modifier::new().fill_max_size().semantics(SemanticsArgs {
        test_tag: Some(tag),
        ..Default::default()
    }));
}

#[tessera]
fn fixture() {
    h_split_view()
        .leading_width(Dp(100.0))
        .leading(|| filler("leading".into()))
        .trailing(|| filler("trailing".into()));
}

#[test]
fn dragging_the_divider_resizes_the_panes_within_their_minimum_widths() {
    let mut ui = UiTest::new().viewport(400, 200).content(fixture);
    let frame = ui.frame();
    assert_eq!(frame.node("leading").bounds().width.0, 100);
    assert_eq!(frame.node("trailing").bounds().x.0, 101);
    assert_eq!(frame.node("trailing").bounds().width.0, 299);
    let divider = PxPosition::from_f32_xy(100.5, 100.0);
    ui.pointer_press(MOUSE_POINTER_ID, divider, PressKeyEventType::Left);
    ui.frame();
    ui.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(150.5, 100.0));
    ui.frame();
    ui.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(150.5, 100.0),
        PressKeyEventType::Left,
    );
    let frame = ui.frames(2);
    assert_eq!(frame.node("leading").bounds().width.0, 150);
    ui.pointer_press(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(150.5, 100.0),
        PressKeyEventType::Left,
    );
    ui.frame();
    ui.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(5.0, 100.0));
    ui.frame();
    ui.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(5.0, 100.0),
        PressKeyEventType::Left,
    );
    let frame = ui.frames(2);
    assert_eq!(
        frame.node("leading").bounds().width.0,
        60,
        "panes keep their minimum width"
    );
}
