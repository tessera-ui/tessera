//! Ripple repainting with stationary input and retained content.
//!
//! ## Usage
//!
//! Let click feedback finish without pointer movement or application updates.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Color, ComputedData, Dp, HeadlessConfig, HeadlessRenderer, LayoutResult, MOUSE_POINTER_ID,
    MeasurementError, Modifier, PressKeyEventType, Px, PxPosition,
    accesskit::{Action, ActionRequest, TreeId},
    layout::{MeasureScope, layout},
    tessera,
};

use crate::{button::button, modifier::ModifierExt as _, theme::material_theme};

thread_local! {
    static BUILDS: Cell<usize> = const {Cell::new(0)};
    static MEASURES: Cell<usize> = const {Cell::new(0)};
    static CLICKS: Cell<usize> = const {Cell::new(0)};
}

#[derive(Clone, PartialEq)]
struct ContentMeasure;

impl tessera_ui::LayoutModifierNode for ContentMeasure {
    fn measure(&self, _input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        MEASURES.set(MEASURES.get() + 1);
        Ok(LayoutResult::new(ComputedData {
            width: Px(20),
            height: Px(20),
        }))
    }
}

#[tessera]
fn retained_content() {
    BUILDS.set(BUILDS.get() + 1);
    layout().policy(ContentMeasure);
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        button()
            .modifier(Modifier::new().size(Dp(160.0), Dp(64.0)))
            .color(Color::WHITE)
            .content_color(Color::BLACK)
            .ripple_color(Color::BLACK)
            .elevation(Dp(0.0))
            .accessibility_label("No-op button")
            .on_click(|| CLICKS.set(CLICKS.get() + 1))
            .child(|| {
                retained_content();
            });
    });
}

fn check_stationary_ripple(samples: u32) {
    BUILDS.set(0);
    MEASURES.set(0);
    CLICKS.set(0);
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 240,
            height: 120,
            sample_count: samples,
            frame_time: Duration::from_millis(20),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(5);
    let (id, node) = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("No-op button"))
        .unwrap();
    let bounds = node.bounds().unwrap();
    let center = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, center);
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Focus,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None
    }));
    renderer.render_frames(40);
    let baseline = renderer.read_pixels().unwrap();
    let builds = BUILDS.get();
    let measures = MEASURES.get();
    for click in 1..=2 {
        renderer.pointer_press(MOUSE_POINTER_ID, center, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, center, PressKeyEventType::Left);
        renderer.render_frames(2);
        let early = renderer.read_pixels().unwrap();
        assert!(early != baseline, "click must paint a visible ripple");
        renderer.render_frames(4);
        let later = renderer.read_pixels().unwrap();
        assert!(
            later != early,
            "ripple pixels must advance while the pointer and application state are unchanged"
        );
        renderer.render_frames(20);
        assert!(
            renderer.read_pixels().unwrap() == baseline,
            "the final frame must clear the ripple without another input event"
        );
        renderer.render_frames(10);
        assert!(renderer.read_pixels().unwrap() == baseline);
        assert_eq!(CLICKS.get(), click);
        assert_eq!(
            BUILDS.get(),
            builds,
            "ripple must retain the button content"
        );
        assert_eq!(
            MEASURES.get(),
            measures,
            "ripple must not remeasure content"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn stationary_noop_button_ripple_advances_finishes_and_restarts_msaa1() {
    crate::test_utils::run_isolated_test(
        "surface::ripple_tests::stationary_noop_button_ripple_advances_finishes_and_restarts_msaa1",
        || check_stationary_ripple(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn stationary_noop_button_ripple_advances_finishes_and_restarts_msaa4() {
    crate::test_utils::run_isolated_test(
        "surface::ripple_tests::stationary_noop_button_ripple_advances_finishes_and_restarts_msaa4",
        || check_stationary_ripple(4),
    );
}
